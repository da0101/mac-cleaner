#!/usr/bin/env python3
"""
Mac Cleaner Server — local web dashboard + auto-clean scheduler.
Runs on localhost:3333, cleans garbage every 15 minutes.
Zero dependencies — pure Python stdlib.
"""

import os
import sys
import json
import subprocess
import threading
import time
import signal
from pathlib import Path
from datetime import datetime
from http.server import HTTPServer, BaseHTTPRequestHandler
from urllib.parse import urlparse
from scanner import clean_scan_item, scan_known_items, scan_system_data_items

HOME = Path.home()
PORT = 3333
AUTO_CLEAN_INTERVAL = 15 * 60  # 15 minutes in seconds
LOG_FILE = Path(__file__).parent / "cleanup_history.json"

# ── Shared state ──
state = {
    "last_scan": None,
    "last_clean": None,
    "history": [],
    "auto_clean_enabled": False,
    "scanning": False,
    "cleaning": False,
}


def load_history():
    if LOG_FILE.exists():
        try:
            with open(LOG_FILE) as f:
                state["history"] = json.load(f)
        except Exception:
            state["history"] = []


def save_history():
    try:
        with open(LOG_FILE, "w") as f:
            json.dump(state["history"][-100:], f, indent=2)  # Keep last 100
    except Exception:
        pass


def fmt_size(size_bytes):
    if size_bytes < 1024:
        return f"{size_bytes} B"
    elif size_bytes < 1024 ** 2:
        return f"{size_bytes / 1024:.1f} KB"
    elif size_bytes < 1024 ** 3:
        return f"{size_bytes / 1024**2:.1f} MB"
    else:
        return f"{size_bytes / 1024**3:.2f} GB"


def dir_size(path):
    total = 0
    try:
        if path.is_file() or path.is_symlink():
            return path.stat().st_size
        for entry in path.rglob("*"):
            try:
                if entry.is_file() and not entry.is_symlink():
                    total += entry.stat().st_size
            except (PermissionError, OSError):
                pass
    except (PermissionError, OSError):
        pass
    return total


def get_system_info():
    info = {
        "free_ram": 0,
        "available_ram": 0,
        "total_ram": 0,
        "disk_used": "",
        "disk_available": "",
        "disk_total": "",
        "disk_percent": 0,
    }

    # RAM
    try:
        out = subprocess.run(["vm_stat"], capture_output=True, text=True, timeout=5)
        if out.returncode == 0:
            pages = {}
            for line in out.stdout.strip().split("\n")[1:]:
                if ":" in line:
                    key, val = line.split(":", 1)
                    try:
                        pages[key.strip()] = int(val.strip().rstrip("."))
                    except ValueError:
                        pass
            ps = 16384
            free = pages.get("Pages free", 0) * ps
            active = pages.get("Pages active", 0) * ps
            inactive = pages.get("Pages inactive", 0) * ps
            wired = pages.get("Pages wired down", 0) * ps
            compressed = pages.get("Pages occupied by compressor", 0) * ps
            speculative = pages.get("Pages speculative", 0) * ps
            purgeable = pages.get("Pages purgeable", 0) * ps

            total = free + active + inactive + wired + compressed + speculative
            available = free + inactive + speculative + purgeable
            info["free_ram"] = free
            info["available_ram"] = available
            info["total_ram"] = total
            info["ram_app"] = active
            info["ram_wired"] = wired
            info["ram_compressed"] = compressed
            info["ram_cached"] = inactive
            info["ram_inactive"] = inactive
            info["ram_speculative"] = speculative
            info["ram_purgeable"] = purgeable
    except Exception:
        pass

    # Disk
    try:
        out = subprocess.run(["df", "-h", "/"], capture_output=True, text=True, timeout=5)
        if out.returncode == 0:
            parts = out.stdout.strip().split("\n")[1].split()
            info["disk_total"] = parts[1]
            info["disk_used"] = parts[2]
            info["disk_available"] = parts[3]
            info["disk_percent"] = int(parts[4].replace("%", ""))
    except Exception:
        pass

    return info


def scan_garbage():
    """Scan and return list of garbage items."""
    state["scanning"] = True
    try:
        items = scan_known_items()
        items.extend(scan_system_data_items())
        items.sort(key=lambda x: x["size"], reverse=True)
        state["last_scan"] = datetime.now().isoformat()
        return items
    finally:
        state["scanning"] = False


def clean_item(item):
    """Clean a single item. Returns bytes cleaned."""
    return clean_scan_item(item)


def clean_all_garbage():
    """Scan and clean all garbage. Returns summary."""
    state["cleaning"] = True
    items = [item for item in scan_known_items() if item.get("cleanable", False)]
    total_before = sum(i["size"] for i in items)
    cleaned = 0
    cleaned_items = []

    for item in items:
        freed = clean_item(item)
        if freed > 0:
            cleaned += freed
            cleaned_items.append({"name": item["name"], "freed": fmt_size(freed)})

    summary = {
        "timestamp": datetime.now().isoformat(),
        "items_found": len(items),
        "items_cleaned": len(cleaned_items),
        "total_garbage": fmt_size(total_before),
        "total_cleaned": fmt_size(cleaned),
        "total_cleaned_bytes": cleaned,
        "details": cleaned_items,
    }

    state["last_clean"] = summary["timestamp"]
    state["history"].insert(0, summary)
    save_history()
    state["cleaning"] = False
    return summary


def docker_prune():
    """Run docker system prune."""
    try:
        result = subprocess.run(
            ["docker", "system", "prune", "-a", "-f", "--volumes"],
            capture_output=True, text=True, timeout=120
        )
        return {"success": result.returncode == 0, "output": result.stdout[-500:] if result.stdout else ""}
    except Exception as e:
        return {"success": False, "output": str(e)}


PROCESS_CATEGORIES = {
    # name_fragment -> (category, label, can_close, suggestion)
    "kernel_task": ("system", "macOS Kernel", False, "Cannot close — core OS process"),
    "WindowServer": ("system", "macOS Display", False, "Cannot close — manages your screen"),
    "launchd": ("system", "macOS Services", False, "Cannot close — system launcher"),
    "Finder.app": ("system", "Finder", False, "Protected macOS process"),
    "Dock.app": ("system", "Dock", False, "Protected macOS process"),
    "ControlCenter.app": ("system", "Control Center", False, "Protected macOS process"),
    "Spotlight.app": ("system", "Spotlight", False, "Protected macOS process"),
    "mds_stores": ("system", "Spotlight Index", False, "Indexing files — will settle down"),
    "mds": ("system", "Spotlight", False, "Indexing files — will settle down"),
    "cloudd": ("system", "iCloud Sync", False, "Syncing iCloud — temporary"),
    "nsurlsessiond": ("system", "macOS Downloads", False, "Background downloads — temporary"),
    "mediaanalysisd": ("system", "Photos Analysis", False, "Analyzing photos — temporary"),
    "photolibraryd": ("system", "Photos Library", False, "Processing photos — temporary"),
    "Terminal": ("dev", "Terminal", False, "Protected shell — ask before closing"),
    "iTerm": ("dev", "iTerm", False, "Protected shell — ask before closing"),
    "zsh": ("dev", "zsh", False, "Protected shell — ask before killing"),
    "bash": ("dev", "bash", False, "Protected shell — ask before killing"),

    "Google Chrome": ("browser", "Chrome", True, "Close unused tabs — each tab ~50-150MB"),
    "Chrome": ("browser", "Chrome", True, "Close unused tabs — each tab ~50-150MB"),
    "Safari": ("browser", "Safari", True, "Close unused tabs"),
    "Firefox": ("browser", "Firefox", True, "Close unused tabs"),
    "Microsoft Edge": ("browser", "Edge", True, "Close unused tabs"),
    "Arc": ("browser", "Arc", True, "Close unused tabs"),

    "Electron": ("ide", "Electron App", True, "Close if not needed"),
    "Visual Studio Code.app": ("ide", "VS Code", False, "Protected developer tool — ask before closing"),
    "Code Helper": ("ide", "VS Code", False, "Protected developer tool — ask before closing"),
    "Code": ("ide", "VS Code", False, "Protected developer tool — ask before closing"),
    "Visual": ("ide", "VS Code", False, "Protected developer tool — ask before closing"),
    "Cursor": ("ide", "Cursor IDE", False, "Protected developer tool — ask before closing"),
    "cursor": ("ide", "Cursor IDE", False, "Protected developer tool — ask before closing"),
    "WebStorm": ("ide", "WebStorm", False, "Protected developer tool — ask before closing"),

    "claude": ("dev", "Claude Code", False, "Protected agent session — ask before closing"),
    "codex": ("dev", "Codex", False, "Protected agent session — ask before closing"),
    "gemini": ("dev", "Gemini CLI", False, "Protected agent session — ask before closing"),
    "node": ("dev", "Node.js", False, "Possible dev server or build job — ask before killing"),
    "python": ("dev", "Python", False, "Possible script or active tool — ask before killing"),
    "ruby": ("dev", "Ruby", True, "Check for idle scripts"),
    "java": ("dev", "Java/Gradle", True, "Kill idle Java/Gradle daemons"),
    "gradle": ("dev", "Gradle Daemon", True, "Run: gradle --stop"),
    "dart": ("dev", "Dart/Flutter", True, "Close Flutter dev servers"),
    "flutter": ("dev", "Flutter", True, "Close Flutter dev servers"),

    "Docker": ("docker", "Docker", True, "Close Docker Desktop when not using containers"),
    "docker": ("docker", "Docker", True, "Close Docker Desktop when not using containers"),
    "com.docker": ("docker", "Docker", True, "Close Docker Desktop when not using containers"),
    "vpnkit": ("docker", "Docker VPN", True, "Part of Docker — close Docker Desktop"),

    "Postman": ("app", "Postman", True, "Close when not testing APIs"),
    "Microsoft Teams": ("app", "MS Teams", True, "Close or use web version — very heavy"),
    "MSTeams": ("app", "MS Teams", True, "Close or use web version — very heavy"),
    "Slack": ("app", "Slack", True, "Use web version to save ~200MB"),
    "Spotify": ("app", "Spotify", True, "Close or use web player"),
    "Discord": ("app", "Discord", True, "Close or use web version"),
    "Telegram": ("app", "Telegram", True, "Close when not chatting"),
    "WhatsApp": ("app", "WhatsApp", True, "Close when not chatting"),
    "Figma": ("app", "Figma", True, "Close when not designing"),
    "Teams": ("app", "MS Teams", True, "Close or use web version — very heavy"),
    "Notion": ("app", "Notion", True, "Close or use web version"),
    "zoom.us": ("app", "Zoom", True, "Close when not in a meeting"),
    "Linear": ("app", "Linear", True, "Use web version to save RAM"),
}

CATEGORY_COLORS = {
    "system": ("\033[90m", "skip"),       # gray — can't close
    "browser": ("\033[91m", "danger"),     # red — usually biggest hog
    "ide": ("\033[93m", "warning"),        # yellow
    "dev": ("\033[94m", "info"),           # blue
    "docker": ("\033[95m", "purple"),      # purple
    "app": ("\033[96m", "closeable"),      # cyan
    "other": ("\033[37m", "default"),      # white
}


def classify_process(name):
    """Classify a process and return (category, label, can_close, suggestion)."""
    for fragment, info in PROCESS_CATEGORIES.items():
        if fragment.lower() in name.lower():
            return info
    return ("other", name[:25], True, "Unknown — check if needed")


def display_process_name(command):
    """Return a stable executable display name without being confused by path-like args."""
    executable = command.split(None, 1)[0]
    return Path(executable).name[:40] or command[:40]


def get_top_memory_processes(limit=15):
    """Get top memory-consuming processes, classified and grouped."""
    try:
        out = subprocess.run(
            ["ps", "aux", "-m"],
            capture_output=True, text=True, timeout=5
        )
        if out.returncode != 0:
            return []

        raw_procs = []
        for line in out.stdout.strip().split("\n")[1:]:
            parts = line.split(None, 10)
            if len(parts) < 11:
                continue
            try:
                mem_pct = float(parts[3])
                command = parts[10]
                short_name = display_process_name(command)
                pid = parts[1]
                rss_kb = int(parts[5])
                if rss_kb > 10240:  # > 10MB
                    raw_procs.append({
                        "pid": pid,
                        "name": short_name,
                        "full_path": command,
                        "mem_pct": mem_pct,
                        "mem_bytes": rss_kb * 1024,
                    })
            except (ValueError, IndexError):
                continue

        # Classify and group by label
        grouped = {}
        for proc in raw_procs:
            cat, label, can_close, suggestion = classify_process(proc["full_path"])
            if label not in grouped:
                grouped[label] = {
                    "label": label,
                    "category": cat,
                    "can_close": can_close,
                    "suggestion": suggestion,
                    "mem_bytes": 0,
                    "mem_pct": 0,
                    "count": 0,
                    "pids": [],
                }
            grouped[label]["mem_bytes"] += proc["mem_bytes"]
            grouped[label]["mem_pct"] += proc["mem_pct"]
            grouped[label]["count"] += 1
            grouped[label]["pids"].append(proc["pid"])

        result = sorted(grouped.values(), key=lambda x: x["mem_bytes"], reverse=True)

        # Add formatted size
        for item in result:
            item["mem"] = fmt_size(item["mem_bytes"])
            item["pids"] = item["pids"][:5]  # Limit PIDs in response

        return result[:limit]
    except Exception:
        return []


def get_ram_summary():
    """Get a full RAM breakdown with suggestions."""
    procs = get_top_memory_processes(20)
    info = get_system_info()
    total_ram = info.get("total_ram", 0)
    free_ram = info.get("free_ram", 0)
    available_ram = info.get("available_ram", free_ram)

    closeable_ram = sum(p["mem_bytes"] for p in procs if p["can_close"])
    system_ram = sum(p["mem_bytes"] for p in procs if not p["can_close"])

    suggestions = []
    for p in procs:
        if p["can_close"] and p["mem_bytes"] > 50 * 1024 * 1024:  # > 50MB
            suggestions.append({
                "action": f"Close {p['label']}",
                "saves": p["mem"],
                "saves_bytes": p["mem_bytes"],
                "reason": p["suggestion"],
                "priority": "high" if p["mem_bytes"] > 300 * 1024 * 1024 else "medium" if p["mem_bytes"] > 100 * 1024 * 1024 else "low",
            })

    return {
        "total_ram": total_ram,
        "free_ram": free_ram,
        "free_ram_human": fmt_size(free_ram),
        "available_ram": available_ram,
        "available_ram_human": fmt_size(available_ram),
        "closeable_ram": closeable_ram,
        "closeable_ram_human": fmt_size(closeable_ram),
        "system_ram": system_ram,
        "system_ram_human": fmt_size(system_ram),
        "processes": procs,
        "suggestions": suggestions,
    }


def purge_ram():
    """Purge inactive RAM using the purge command."""
    try:
        # Try without sudo first (works on some setups)
        result = subprocess.run(["purge"], capture_output=True, text=True, timeout=15)
        if result.returncode == 0:
            return True
        # Try with sudo -n (non-interactive, needs NOPASSWD in sudoers)
        result = subprocess.run(["sudo", "-n", "purge"], capture_output=True, text=True, timeout=15)
        return result.returncode == 0
    except Exception:
        return False


def try_reclaim_app_memory(label, pids):
    """Try to reclaim memory from an app WITHOUT killing it.
    Uses multiple strategies depending on the app type."""
    reclaimed = False
    ts = datetime.now().strftime("%H:%M:%S")

    # Strategy 1: Aggressive system-level memory pressure
    purge_ram()

    # Strategy 2: For Electron apps (VS Code, Cursor, Slack, Teams, Discord)
    # Send SIGUSR1 which triggers Node.js garbage collection in some builds
    electron_apps = {"VS Code", "Cursor IDE", "Slack", "MS Teams", "Discord", "Postman"}
    if label in electron_apps:
        for pid in pids[:10]:
            try:
                # SIGUSR1 can trigger GC in Node.js/Electron
                subprocess.run(["kill", "-USR1", str(pid)], capture_output=True, timeout=5)
                reclaimed = True
            except Exception:
                pass
        print(f"  [{ts}] \033[93m  Sent GC signal to {label} processes\033[0m")

    # Strategy 3: For Chrome/browsers — use AppleScript to discard background tabs
    browser_apps = {"Chrome": "Google Chrome", "Firefox": "Firefox", "Safari": "Safari"}
    if label in browser_apps:
        app_name = browser_apps[label]
        try:
            # Tell Chrome to purge memory by running JS garbage collection
            subprocess.run([
                "osascript", "-e",
                f'tell application "{app_name}" to activate',
            ], capture_output=True, timeout=5)
            time.sleep(0.5)
            # Minimize to trigger memory release
            subprocess.run([
                "osascript", "-e",
                f'tell application "System Events" to tell process "{app_name}" to set miniaturized of every window to true',
            ], capture_output=True, timeout=5)
            time.sleep(1)
            # Restore
            subprocess.run([
                "osascript", "-e",
                f'tell application "System Events" to tell process "{app_name}" to set miniaturized of every window to false',
            ], capture_output=True, timeout=5)
            reclaimed = True
            print(f"  [{ts}] \033[93m  Triggered {label} memory cleanup via minimize/restore\033[0m")
        except Exception:
            pass

    # Strategy 4: For any app — send memory pressure notification via system
    try:
        # This forces macOS to ask all apps to release cached memory
        subprocess.run(["memory_pressure", "-l", "critical"], capture_output=True, text=True, timeout=10)
        reclaimed = True
        print(f"  [{ts}] \033[93m  Sent system memory pressure signal\033[0m")
    except Exception:
        pass

    # Final purge to clean up whatever was released
    purge_ram()
    return reclaimed


RAM_PURGE_INTERVAL = 3 * 60  # 3 minutes

# ── Memory leak thresholds (per app) ──
# If a single app group exceeds this, it's a memory leak and should be restarted
MEMORY_LEAK_THRESHOLD = 4 * 1024 * 1024 * 1024  # 4 GB — no single app should use this much on 18GB
LOW_AVAILABLE_RAM = 1024 * 1024 * 1024  # 1 GB reusable memory target

# Apps we know how to gracefully restart (macOS app name → what to reopen)
RESTARTABLE_APPS = {
    "VS Code": "Visual Studio Code",
    "Code": "Visual Studio Code",
    "Cursor IDE": "Cursor",
    "Chrome": "Google Chrome",
    "Firefox": "Firefox",
    "Microsoft Edge": "Microsoft Edge",
    "Slack": "Slack",
    "MS Teams": "Microsoft Teams",
    "Discord": "Discord",
    "Postman": "Postman",
    "Spotify": "Spotify",
}

# Apps that should NEVER be auto-killed
NEVER_KILL = {"macOS Kernel", "macOS Display", "macOS Services", "Spotlight Index", "Spotlight",
              "iCloud Sync", "macOS Downloads", "Photos Analysis", "Photos Library"}


def send_macos_notification(title, message, subtitle=""):
    """Send a macOS notification banner."""
    try:
        script = f'display notification "{message}" with title "{title}"'
        if subtitle:
            script = f'display notification "{message}" with title "{title}" subtitle "{subtitle}"'
        subprocess.run(["osascript", "-e", script], capture_output=True, timeout=5)
    except Exception:
        pass


def memory_watchdog():
    """Detect memory leaks and alert user. Runs every minute. Never auto-kills."""
    # Track which apps we've already alerted about (avoid spamming)
    alerted = {}  # label -> last_alert_time

    while True:
        time.sleep(60)
        if not state["auto_clean_enabled"]:
            continue

        try:
            procs = get_top_memory_processes(20)
            info = get_system_info()
            available_ram = info.get("available_ram", info.get("free_ram", 0))
            ts = datetime.now().strftime("%H:%M:%S")
            now = time.time()

            # Clear alerts list for state (dashboard reads this)
            active_alerts = []

            for p in procs:
                if p["label"] in NEVER_KILL or not p["can_close"]:
                    continue

                mem_gb = p["mem_bytes"] / (1024**3)

                # Check if this app has a memory leak (> 4 GB)
                if p["mem_bytes"] > MEMORY_LEAK_THRESHOLD:
                    alert = {
                        "level": "critical",
                        "label": p["label"],
                        "mem": p["mem"],
                        "mem_gb": round(mem_gb, 1),
                        "mem_bytes": p["mem_bytes"],
                        "pids": p["pids"],
                        "category": p["category"],
                        "message": f"{p['label']} is using {mem_gb:.1f} GB — MEMORY LEAK!",
                        "suggestion": p.get("suggestion", ""),
                        "timestamp": datetime.now().isoformat(),
                    }
                    active_alerts.append(alert)

                    # Only print + notify every 5 minutes per app (avoid spam)
                    last_alert = alerted.get(p["label"], 0)
                    if now - last_alert > 300:
                        alerted[p["label"]] = now

                        print(f"\n  [{ts}] \033[91m{'═' * 58}\033[0m")
                        print(f"  [{ts}] \033[91m\033[1m  ⚠ MEMORY LEAK DETECTED\033[0m")
                        print(f"  [{ts}] \033[91m  {p['label']} is using {mem_gb:.1f} GB!\033[0m")
                        print(f"  [{ts}] \033[93m  Attempting to reclaim memory without killing...\033[0m")

                        # Try to reclaim memory without killing
                        reclaimed = try_reclaim_app_memory(p["label"], p["pids"])

                        # Check how much we freed
                        time.sleep(2)
                        info_after = get_system_info()
                        free_after = fmt_size(info_after.get("available_ram", info_after.get("free_ram", 0)))

                        if reclaimed:
                            print(f"  [{ts}] \033[92m  Memory pressure applied — available RAM now: {free_after}\033[0m")
                        else:
                            print(f"  [{ts}] \033[93m  Could not reclaim — consider restarting {p['label']} manually\033[0m")

                        print(f"  [{ts}] \033[93m  Dashboard: http://localhost:{PORT}\033[0m")
                        print(f"  [{ts}] \033[91m{'═' * 58}\033[0m\n")

                        # macOS notification
                        send_macos_notification(
                            "⚠ Memory Leak Detected",
                            f"{p['label']} is using {mem_gb:.1f} GB! Attempting cleanup...",
                            "Mac Cleaner"
                        )

                # Warning level: 2-4 GB
                elif p["mem_bytes"] > 2 * 1024 * 1024 * 1024:
                    active_alerts.append({
                        "level": "warning",
                        "label": p["label"],
                        "mem": p["mem"],
                        "mem_gb": round(mem_gb, 1),
                        "mem_bytes": p["mem_bytes"],
                        "pids": p["pids"],
                        "category": p["category"],
                        "message": f"{p['label']} is using {mem_gb:.1f} GB — getting high",
                        "suggestion": p.get("suggestion", ""),
                        "timestamp": datetime.now().isoformat(),
                    })

            # Emergency: available RAM critically low
            if available_ram < LOW_AVAILABLE_RAM and available_ram > 0:
                last_emergency = alerted.get("__emergency__", 0)
                if now - last_emergency > 120:  # Alert every 2 min max
                    alerted["__emergency__"] = now
                    print(f"\n  [{ts}] \033[91m{'═' * 58}\033[0m")
                    print(f"  [{ts}] \033[91m\033[1m  🚨 LOW MEMORY: Only {fmt_size(available_ram)} available RAM!\033[0m")
                    print(f"  [{ts}] \033[91m  Purging RAM and asking you to close apps if pressure stays high.\033[0m")
                    print(f"  [{ts}] \033[91m{'═' * 58}\033[0m\n")

                    send_macos_notification(
                        "🚨 Low Available RAM",
                        f"Only {fmt_size(available_ram)} available. Purging RAM now.",
                        "Mac Cleaner — Emergency"
                    )

                    # Purge RAM as emergency measure
                    purge_ram()

            # Store alerts in state so dashboard can show them
            state["alerts"] = active_alerts

        except Exception:
            pass  # Watchdog must never crash


# ── Auto-clean scheduler ──
def auto_clean_loop():
    """Background thread: clean garbage every 15 min, purge RAM every 3 min."""
    ram_counter = 0
    while True:
        time.sleep(60)  # Check every minute
        ram_counter += 1

        # Purge RAM every 3 minutes
        if ram_counter >= 3 and state["auto_clean_enabled"]:
            ram_counter = 0
            ts = datetime.now().strftime("%H:%M:%S")

            # Show RAM before purge
            info_before = get_system_info()
            available_before = fmt_size(info_before["available_ram"])

            success = purge_ram()

            # Show RAM after purge + top consumers
            info_after = get_system_info()
            available_after = fmt_size(info_after["available_ram"])

            if success:
                print(f"  [{ts}] \033[92mRAM purged: {available_before} → {available_after} available\033[0m")
            else:
                print(f"  [{ts}] \033[91mRAM purge failed (sudo not configured)\033[0m")

            # Show classified RAM breakdown
            summary = get_ram_summary()
            procs = summary["processes"]
            if procs:
                print(f"  [{ts}] \033[1m{'─' * 58}\033[0m")
                print(f"  [{ts}] \033[1m RAM BREAKDOWN (18 GB total, {summary['available_ram_human']} available)\033[0m")
                print(f"  [{ts}] \033[1m{'─' * 58}\033[0m")
                for p in procs[:10]:
                    cat_color = CATEGORY_COLORS.get(p["category"], CATEGORY_COLORS["other"])[0]
                    status = "\033[92m[OK]\033[0m" if not p["can_close"] else "\033[91m[CLOSE]\033[0m" if p["mem_bytes"] > 300*1024*1024 else "\033[93m[CHECK]\033[0m" if p["mem_bytes"] > 50*1024*1024 else "\033[90m[low]\033[0m"
                    count_str = f" x{p['count']}" if p['count'] > 1 else ""
                    print(f"  [{ts}]  {status} {cat_color}{p['mem']:>10}  {p['label']}{count_str}\033[0m")
                if summary["suggestions"]:
                    closeable = summary["closeable_ram_human"]
                    print(f"  [{ts}] \033[1m{'─' * 58}\033[0m")
                    print(f"  [{ts}] \033[93m You could free ~{closeable} by closing unnecessary apps:\033[0m")
                    for s in summary["suggestions"][:3]:
                        icon = "\033[91m!!!\033[0m" if s["priority"] == "high" else "\033[93m !!\033[0m" if s["priority"] == "medium" else "\033[90m  !\033[0m"
                        print(f"  [{ts}]  {icon} {s['action']} → saves {s['saves']} — {s['reason']}")
                print(f"  [{ts}] \033[1m{'─' * 58}\033[0m")
                print()

        # Clean garbage every 15 minutes
        if ram_counter % 15 == 0 and state["auto_clean_enabled"] and not state["cleaning"]:
            ts = datetime.now().strftime("%H:%M:%S")
            print(f"  [{ts}] Auto-clean triggered...")
            summary = clean_all_garbage()
            print(f"  [{ts}] Cleaned {summary['total_cleaned']} ({summary['items_cleaned']} items)")


# ── Dashboard HTML ──
def get_dashboard_html():
    return '''<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Mac Cleaner</title>
<style>
  :root {
    --bg: #0d1117; --card: #161b22; --border: #30363d; --text: #e6edf3;
    --dim: #8b949e; --green: #3fb950; --yellow: #d29922; --red: #f85149;
    --blue: #58a6ff; --cyan: #39d2c0; --purple: #bc8cff;
  }
  * { margin:0; padding:0; box-sizing:border-box; }
  body { font-family: -apple-system, 'SF Mono', monospace; background:var(--bg); color:var(--text); padding:24px; }
  .header { display:flex; align-items:center; justify-content:space-between; margin-bottom:24px; }
  .header h1 { font-size:20px; font-weight:600; }
  .header .status { font-size:13px; color:var(--dim); }
  .header .status .dot { display:inline-block; width:8px; height:8px; border-radius:50%; background:var(--green); margin-right:6px; animation: pulse 2s infinite; }
  @keyframes pulse { 0%,100%{opacity:1} 50%{opacity:0.4} }
  .grid { display:grid; grid-template-columns:repeat(auto-fit, minmax(200px, 1fr)); gap:16px; margin-bottom:24px; }
  .stat-card { background:var(--card); border:1px solid var(--border); border-radius:12px; padding:16px; }
  .stat-card .label { font-size:11px; text-transform:uppercase; letter-spacing:0.5px; color:var(--dim); margin-bottom:6px; }
  .stat-card .value { font-size:24px; font-weight:700; }
  .stat-card .sub { font-size:12px; color:var(--dim); margin-top:4px; }
  .bar-bg { height:6px; background:var(--border); border-radius:3px; margin-top:8px; overflow:hidden; }
  .bar-fill { height:100%; border-radius:3px; transition: width 0.5s ease; }
  .actions { display:flex; gap:12px; margin-bottom:24px; flex-wrap:wrap; }
  .btn { padding:10px 20px; border-radius:8px; border:1px solid var(--border); background:var(--card); color:var(--text);
         cursor:pointer; font-size:13px; font-family:inherit; transition:all 0.2s; display:flex; align-items:center; gap:8px; }
  .btn:hover { border-color:var(--blue); background:#1c2333; }
  .btn.primary { background:var(--green); color:#000; border-color:var(--green); font-weight:600; }
  .btn.primary:hover { background:#2ea043; }
  .btn.danger { background:var(--red); color:#fff; border-color:var(--red); }
  .btn.danger:hover { background:#da3633; }
  .btn:disabled { opacity:0.5; cursor:not-allowed; }
  .section { background:var(--card); border:1px solid var(--border); border-radius:12px; padding:20px; margin-bottom:16px; }
  .section h2 { font-size:14px; font-weight:600; margin-bottom:12px; display:flex; align-items:center; gap:8px; }
  table { width:100%; border-collapse:collapse; }
  th { text-align:left; font-size:11px; text-transform:uppercase; letter-spacing:0.5px; color:var(--dim); padding:8px 12px; border-bottom:1px solid var(--border); }
  td { padding:8px 12px; font-size:13px; border-bottom:1px solid var(--border); }
  tr:last-child td { border-bottom:none; }
  .size { font-weight:600; font-variant-numeric:tabular-nums; }
  .size.large { color:var(--red); }
  .size.medium { color:var(--yellow); }
  .size.small { color:var(--dim); }
  .cat { display:inline-block; padding:2px 8px; border-radius:4px; font-size:11px; font-weight:500; }
  .cat-xcode { background:#3b2e00; color:var(--yellow); }
  .cat-dev { background:#0d2818; color:var(--green); }
  .cat-ide { background:#1c1040; color:var(--purple); }
  .cat-cache { background:#051d3a; color:var(--blue); }
  .cat-browser { background:#2a1520; color:#f778ba; }
  .cat-system { background:#1a1a2e; color:var(--cyan); }
  .cat-system-data { background:#241b11; color:#ffa657; }
  .cat-app { background:#1a1520; color:#bc8cff; }
  .cat-ai { background:#0d2030; color:#58a6ff; }
  .cat-logs { background:#1a1a1a; color:#8b949e; }
  .history-item { padding:10px 0; border-bottom:1px solid var(--border); font-size:13px; }
  .history-item:last-child { border-bottom:none; }
  .history-time { color:var(--dim); font-size:12px; }
  .toast { position:fixed; bottom:24px; right:24px; background:var(--green); color:#000; padding:12px 20px;
           border-radius:8px; font-size:13px; font-weight:600; opacity:0; transition:opacity 0.3s; z-index:100; }
  .toast.show { opacity:1; }
  .spinner { display:inline-block; width:14px; height:14px; border:2px solid transparent; border-top:2px solid currentColor;
             border-radius:50%; animation:spin 0.6s linear infinite; }
  @keyframes spin { to{transform:rotate(360deg)} }
  .auto-badge { font-size:11px; padding:3px 8px; border-radius:4px; font-weight:500; }
  .auto-badge.on { background:#0d2818; color:var(--green); }
  .auto-badge.off { background:#3b1520; color:var(--red); }
  .countdown { font-size:12px; color:var(--dim); font-variant-numeric:tabular-nums; }
</style>
</head>
<body>

<div class="header">
  <h1>Mac Cleaner</h1>
  <div class="status">
    <span class="dot"></span>Running on localhost:3333
    &nbsp;&middot;&nbsp;
    Auto-clean: <span id="autoStatus" class="auto-badge off">OFF</span>
    &nbsp;
    <span id="countdown" class="countdown"></span>
  </div>
</div>

<div class="grid" id="statsGrid">
  <div class="stat-card">
    <div class="label">Available Memory</div>
    <div class="value" id="freeRam">—</div>
    <div class="bar-bg"><div class="bar-fill" id="ramBar" style="width:0%;background:var(--green)"></div></div>
    <div class="sub" id="ramSub"></div>
    <div class="sub" id="ramDetail" style="margin-top:2px"></div>
  </div>
  <div class="stat-card">
    <div class="label">Disk Available</div>
    <div class="value" id="diskFree">—</div>
    <div class="bar-bg"><div class="bar-fill" id="diskBar" style="width:0%;background:var(--blue)"></div></div>
    <div class="sub" id="diskSub"></div>
  </div>
  <div class="stat-card">
    <div class="label">Garbage Found</div>
    <div class="value" id="garbageTotal">—</div>
    <div class="sub" id="garbageSub"></div>
  </div>
  <div class="stat-card">
    <div class="label">Last Cleanup</div>
    <div class="value" id="lastClean">—</div>
    <div class="sub" id="lastCleanSub"></div>
  </div>
</div>

<div id="alertBanner" style="display:none;background:#3b1520;border:1px solid var(--red);border-radius:12px;padding:16px;margin-bottom:16px">
  <div style="font-weight:700;color:var(--red);margin-bottom:8px" id="alertTitle">⚠ Memory Leak Detected</div>
  <div id="alertBody" style="font-size:13px"></div>
</div>

<div class="actions">
  <button class="btn primary" id="btnClean" onclick="cleanAll()">Clean All Garbage</button>
  <button class="btn" id="btnScan" onclick="scanNow()">Scan Now</button>
  <button class="btn" id="btnDocker" onclick="dockerPrune()">Docker Prune</button>
  <button class="btn" id="btnRam" onclick="purgeRam()">Purge RAM</button>
  <button class="btn" id="btnToggle" onclick="toggleAuto()">Pause Auto-Clean</button>
</div>

<div class="section" id="garbageSection">
  <h2>Garbage Breakdown</h2>
  <table>
    <thead><tr><th>Name</th><th>Category</th><th style="text-align:right">Size</th></tr></thead>
    <tbody id="garbageBody"><tr><td colspan="3" style="color:var(--dim)">Click "Scan Now" to scan...</td></tr></tbody>
  </table>
</div>

<div class="section">
  <h2>RAM Breakdown — Where Your 18 GB Goes</h2>
  <div id="ramSummaryBar" style="display:flex;height:24px;border-radius:6px;overflow:hidden;margin-bottom:12px;gap:2px"></div>
  <div id="ramSummaryText" style="font-size:12px;color:var(--dim);margin-bottom:16px"></div>
  <table>
    <thead><tr><th>Process</th><th>Type</th><th>Status</th><th style="text-align:right">Memory</th><th>Suggestion</th></tr></thead>
    <tbody id="processBody"><tr><td colspan="5" style="color:var(--dim)">Loading...</td></tr></tbody>
  </table>
</div>

<div class="section" id="suggestionsSection" style="display:none;border-color:var(--yellow)">
  <h2 style="color:var(--yellow)">Recommendations to Free RAM</h2>
  <div id="suggestionsBody"></div>
</div>

<div class="section">
  <h2>Cleanup History</h2>
  <div id="historyBody"><div class="history-item" style="color:var(--dim)">No cleanups yet</div></div>
</div>

<div class="toast" id="toast"></div>

<script>
let nextCleanTime = Date.now() + 15 * 60 * 1000;
let nextRamPurge = Date.now() + 3 * 60 * 1000;
let autoEnabled = true;

function toast(msg, dur=3000) {
  const t = document.getElementById('toast');
  t.textContent = msg;
  t.classList.add('show');
  setTimeout(() => t.classList.remove('show'), dur);
}

function sizeClass(bytes) {
  if (bytes > 1024**3) return 'large';
  if (bytes > 100*1024**2) return 'medium';
  return 'small';
}

function catClass(cat) {
  return 'cat-' + (cat || 'cache');
}

function formatTime(iso) {
  if (!iso) return '—';
  const d = new Date(iso);
  return d.toLocaleTimeString([], {hour:'2-digit',minute:'2-digit'});
}

function formatDate(iso) {
  if (!iso) return '';
  const d = new Date(iso);
  const now = new Date();
  if (d.toDateString() === now.toDateString()) return 'Today ' + formatTime(iso);
  return d.toLocaleDateString() + ' ' + formatTime(iso);
}

async function fetchJSON(url, opts) {
  const r = await fetch(url, opts);
  return r.json();
}

async function loadStatus() {
  try {
    const data = await fetchJSON('/api/status');
    autoEnabled = data.auto_clean_enabled;
    document.getElementById('autoStatus').textContent = autoEnabled ? 'ON' : 'OFF';
    document.getElementById('autoStatus').className = 'auto-badge ' + (autoEnabled ? 'on' : 'off');
    document.getElementById('btnToggle').textContent = autoEnabled ? 'Pause Auto-Clean' : 'Resume Auto-Clean';

    // System info
    const info = data.system;
    if (info) {
      const available = info.available_ram || info.free_ram || 0;
      const freeMB = Math.round((info.free_ram || 0) / 1024**2);
      const availableMB = Math.round(available / 1024**2);
      const totalRamGB = (info.total_ram / 1024**3).toFixed(1);
      const ramPct = info.total_ram ? Math.round((1 - available/info.total_ram)*100) : 0;
      // Show MB when under 1GB, GB otherwise
      const availableStr = availableMB < 1024 ? availableMB + ' MB' : (availableMB/1024).toFixed(1) + ' GB';
      document.getElementById('freeRam').textContent = availableStr;
      document.getElementById('freeRam').style.color = availableMB < 1024 ? 'var(--yellow)' : 'var(--green)';
      document.getElementById('ramBar').style.width = ramPct + '%';
      document.getElementById('ramBar').style.background = ramPct > 95 ? 'var(--red)' : ramPct > 85 ? 'var(--yellow)' : 'var(--green)';
      document.getElementById('ramSub').textContent = ramPct + '% pressure footprint of ' + totalRamGB + ' GB; raw free ' + freeMB + ' MB';
      // Breakdown
      const appGB = (info.ram_app / 1024**3).toFixed(1);
      const wiredGB = (info.ram_wired / 1024**3).toFixed(1);
      const compGB = (info.ram_compressed / 1024**3).toFixed(1);
      const cacheGB = (((info.ram_cached||0) + (info.ram_speculative||0) + (info.ram_purgeable||0)) / 1024**3).toFixed(1);
      document.getElementById('ramDetail').textContent = 'App: ' + appGB + 'G | Wired: ' + wiredGB + 'G | Compressed: ' + compGB + 'G | Reusable: ' + cacheGB + 'G';

      document.getElementById('diskFree').textContent = info.disk_available || '—';
      document.getElementById('diskBar').style.width = (info.disk_percent || 0) + '%';
      document.getElementById('diskBar').style.background = info.disk_percent > 90 ? 'var(--red)' : info.disk_percent > 75 ? 'var(--yellow)' : 'var(--blue)';
      document.getElementById('diskSub').textContent = (info.disk_percent||0) + '% used of ' + (info.disk_total||'—');
    }

    if (data.last_clean) {
      document.getElementById('lastClean').textContent = formatTime(data.last_clean);
      document.getElementById('lastCleanSub').textContent = formatDate(data.last_clean);
    }
  } catch(e) { console.error(e); }
}

async function scanNow() {
  const btn = document.getElementById('btnScan');
  btn.disabled = true;
  btn.innerHTML = '<span class="spinner"></span> Scanning...';
  try {
    const data = await fetchJSON('/api/scan');
    renderGarbage(data.items);
    const cleanable = data.items.filter(i => i.cleanable);
    const total = cleanable.reduce((a,b) => a + b.size, 0);
    const reportOnly = data.items.length - cleanable.length;
    document.getElementById('garbageTotal').textContent = cleanable.length > 0 ? formatBytes(total) : '0 B';
    document.getElementById('garbageSub').textContent = cleanable.length + ' cleanable, ' + reportOnly + ' report-only';
    toast('Scan complete — ' + data.items.length + ' items found');
  } catch(e) { toast('Scan failed'); }
  btn.disabled = false;
  btn.innerHTML = 'Scan Now';
}

function formatBytes(b) {
  if (b < 1024) return b + ' B';
  if (b < 1024**2) return (b/1024).toFixed(1) + ' KB';
  if (b < 1024**3) return (b/1024**2).toFixed(1) + ' MB';
  return (b/1024**3).toFixed(2) + ' GB';
}

function renderGarbage(items) {
  const body = document.getElementById('garbageBody');
  if (!items.length) { body.innerHTML = '<tr><td colspan="3" style="color:var(--dim)">System is clean!</td></tr>'; return; }
  body.innerHTML = items.map(i => `
    <tr>
      <td>${i.name}<div style="color:var(--dim);font-size:11px">${i.cleanable ? i.safety : 'report-only'} — ${i.reason || ''}</div></td>
      <td><span class="cat ${catClass(i.category)}">${i.category}</span></td>
      <td style="text-align:right"><span class="size ${sizeClass(i.size)}">${i.size_human}</span></td>
    </tr>
  `).join('');
}

async function cleanAll() {
  const btn = document.getElementById('btnClean');
  btn.disabled = true;
  btn.innerHTML = '<span class="spinner"></span> Cleaning...';
  try {
    const data = await fetchJSON('/api/clean', {method:'POST'});
    toast('Cleaned ' + data.total_cleaned + ' (' + data.items_cleaned + ' items)');
    nextCleanTime = Date.now() + 15*60*1000;
    loadStatus();
    scanNow();
    loadHistory();
  } catch(e) { toast('Clean failed'); }
  btn.disabled = false;
  btn.innerHTML = 'Clean All Garbage';
}

async function dockerPrune() {
  const btn = document.getElementById('btnDocker');
  btn.disabled = true;
  btn.innerHTML = '<span class="spinner"></span> Pruning...';
  try {
    const data = await fetchJSON('/api/docker-prune', {method:'POST'});
    toast(data.success ? 'Docker pruned!' : 'Docker prune failed');
  } catch(e) { toast('Docker prune failed'); }
  btn.disabled = false;
  btn.innerHTML = 'Docker Prune';
}

async function purgeRam() {
  const btn = document.getElementById('btnRam');
  btn.disabled = true;
  btn.innerHTML = '<span class="spinner"></span> Purging...';
  try {
    const data = await fetchJSON('/api/purge-ram', {method:'POST'});
    toast(data.success ? 'RAM purged!' : 'RAM purge needs sudo');
    loadStatus();
  } catch(e) { toast('Failed'); }
  btn.disabled = false;
  btn.innerHTML = 'Purge RAM';
}

async function toggleAuto() {
  try {
    const data = await fetchJSON('/api/toggle-auto', {method:'POST'});
    autoEnabled = data.enabled;
    document.getElementById('autoStatus').textContent = autoEnabled ? 'ON' : 'OFF';
    document.getElementById('autoStatus').className = 'auto-badge ' + (autoEnabled ? 'on' : 'off');
    document.getElementById('btnToggle').textContent = autoEnabled ? 'Pause Auto-Clean' : 'Resume Auto-Clean';
    toast('Auto-clean ' + (autoEnabled ? 'enabled' : 'paused'));
  } catch(e) {}
}

async function loadHistory() {
  try {
    const data = await fetchJSON('/api/history');
    const el = document.getElementById('historyBody');
    if (!data.length) { el.innerHTML = '<div class="history-item" style="color:var(--dim)">No cleanups yet</div>'; return; }
    el.innerHTML = data.slice(0,20).map(h => `
      <div class="history-item">
        <span class="history-time">${formatDate(h.timestamp)}</span>
        &nbsp;&middot;&nbsp;
        Cleaned <strong>${h.total_cleaned}</strong> from ${h.items_cleaned} items
      </div>
    `).join('');
  } catch(e) {}
}

// Countdown timer
setInterval(() => {
  if (!autoEnabled) { document.getElementById('countdown').textContent = ''; return; }
  const remaining = Math.max(0, Math.round((nextCleanTime - Date.now()) / 1000));
  const m = Math.floor(remaining / 60);
  const s = remaining % 60;
  const ramRem = Math.max(0, Math.round((nextRamPurge - Date.now()) / 1000));
  const rm = Math.floor(ramRem / 60);
  const rs = ramRem % 60;
  document.getElementById('countdown').textContent = 'RAM purge: ' + rm + ':' + String(rs).padStart(2,'0') + ' | Garbage clean: ' + m + ':' + String(s).padStart(2,'0');
}, 1000);

const CAT_COLORS = {
  system: '#8b949e', browser: '#f85149', ide: '#d29922', dev: '#58a6ff',
  docker: '#bc8cff', app: '#39d2c0', other: '#6e7681'
};
const CAT_LABELS = {
  system: 'System', browser: 'Browser', ide: 'IDE', dev: 'Dev Tool',
  docker: 'Docker', app: 'App', other: 'Other'
};

async function loadProcesses() {
  try {
    const data = await fetchJSON('/api/top-processes');
    const procs = data.processes || [];
    const suggestions = data.suggestions || [];
    const body = document.getElementById('processBody');

    if (!procs.length) { body.innerHTML = '<tr><td colspan="5" style="color:var(--dim)">No data</td></tr>'; return; }

    // Summary bar
    const totalRam = data.total_ram || 1;
    const barEl = document.getElementById('ramSummaryBar');
    const catTotals = {};
    procs.forEach(p => { catTotals[p.category] = (catTotals[p.category]||0) + p.mem_bytes; });
    barEl.innerHTML = Object.entries(catTotals).map(([cat, bytes]) => {
      const pct = Math.max(2, (bytes/totalRam)*100);
      return `<div style="width:${pct}%;background:${CAT_COLORS[cat]||'#444'};border-radius:3px" title="${CAT_LABELS[cat]||cat}: ${formatBytes(bytes)}"></div>`;
    }).join('');

    const summaryEl = document.getElementById('ramSummaryText');
    summaryEl.textContent = `Available: ${data.available_ram_human || data.free_ram_human} | Raw free: ${data.free_ram_human} | Closeable: ${data.closeable_ram_human} | System (locked): ${data.system_ram_human}`;

    // Process table
    body.innerHTML = procs.map(p => {
      const catColor = CAT_COLORS[p.category] || '#6e7681';
      const catLabel = CAT_LABELS[p.category] || p.category;
      let statusBadge;
      if (!p.can_close) {
        statusBadge = '<span style="color:var(--dim);font-size:11px">LOCKED</span>';
      } else if (p.mem_bytes > 300*1024*1024) {
        statusBadge = '<span style="color:var(--red);font-weight:600;font-size:11px">CLOSE</span>';
      } else if (p.mem_bytes > 50*1024*1024) {
        statusBadge = '<span style="color:var(--yellow);font-size:11px">CHECK</span>';
      } else {
        statusBadge = '<span style="color:var(--dim);font-size:11px">ok</span>';
      }
      const countStr = p.count > 1 ? ' <span style="color:var(--dim);font-size:11px">x' + p.count + '</span>' : '';
      const memColor = p.mem_bytes > 500*1024*1024 ? 'var(--red)' : p.mem_bytes > 200*1024*1024 ? 'var(--yellow)' : 'var(--text)';
      return `<tr>
        <td><strong>${p.label}</strong>${countStr}</td>
        <td><span class="cat" style="background:${catColor}22;color:${catColor}">${catLabel}</span></td>
        <td>${statusBadge}</td>
        <td style="text-align:right"><span style="color:${memColor};font-weight:600;font-variant-numeric:tabular-nums">${p.mem}</span></td>
        <td style="font-size:12px;color:var(--dim)">${p.suggestion}</td>
      </tr>`;
    }).join('');

    // Suggestions
    const sugSection = document.getElementById('suggestionsSection');
    const sugBody = document.getElementById('suggestionsBody');
    if (suggestions.length) {
      sugSection.style.display = 'block';
      sugBody.innerHTML = suggestions.map(s => {
        const icon = s.priority === 'high' ? '🔴' : s.priority === 'medium' ? '🟡' : '🟢';
        return `<div style="padding:8px 0;border-bottom:1px solid var(--border);font-size:13px">
          ${icon} <strong>${s.action}</strong> — saves <span style="color:var(--green);font-weight:600">${s.saves}</span>
          <span style="color:var(--dim);margin-left:8px">${s.reason}</span>
        </div>`;
      }).join('');
    } else {
      sugSection.style.display = 'none';
    }
  } catch(e) { console.error(e); }
}

async function loadAlerts() {
  try {
    const alerts = await fetchJSON('/api/alerts');
    const banner = document.getElementById('alertBanner');
    const body = document.getElementById('alertBody');
    const critical = alerts.filter(a => a.level === 'critical');
    const warnings = alerts.filter(a => a.level === 'warning');

    if (critical.length > 0) {
      banner.style.display = 'block';
      banner.style.background = '#3b1520';
      banner.style.borderColor = 'var(--red)';
      document.getElementById('alertTitle').textContent = '⚠ MEMORY LEAK — Action Required';
      document.getElementById('alertTitle').style.color = 'var(--red)';
      body.innerHTML = critical.map(a =>
        `<div style="margin:6px 0;display:flex;justify-content:space-between;align-items:center">
          <span><strong style="color:var(--red)">${a.label}</strong> is using <strong>${a.mem_gb} GB</strong> — ${a.message}</span>
          <span style="color:var(--dim);font-size:12px">Save work, then restart the app</span>
        </div>`
      ).join('') + (warnings.length ? '<div style="margin-top:8px;color:var(--yellow);font-size:12px">+ ' + warnings.length + ' warnings (2-4 GB range)</div>' : '');
    } else if (warnings.length > 0) {
      banner.style.display = 'block';
      banner.style.background = '#2a1f00';
      banner.style.borderColor = 'var(--yellow)';
      document.getElementById('alertTitle').textContent = '⚡ High Memory Usage';
      document.getElementById('alertTitle').style.color = 'var(--yellow)';
      body.innerHTML = warnings.map(a =>
        `<div style="margin:4px 0"><strong style="color:var(--yellow)">${a.label}</strong> — ${a.mem_gb} GB</div>`
      ).join('');
    } else {
      banner.style.display = 'none';
    }
  } catch(e) {}
}

// Auto-refresh
loadStatus();
scanNow();
loadHistory();
loadProcesses();
loadAlerts();
setInterval(loadStatus, 10000);
setInterval(loadHistory, 60000);
setInterval(loadProcesses, 15000);
setInterval(loadAlerts, 10000);
</script>
</body>
</html>'''


# ── HTTP Handler ──
class CleanerHandler(BaseHTTPRequestHandler):
    def log_message(self, format, *args):
        # Suppress default request logs, only show important stuff
        pass

    def _json_response(self, data, status=200):
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.end_headers()
        self.wfile.write(json.dumps(data).encode())

    def _html_response(self, html):
        self.send_response(200)
        self.send_header("Content-Type", "text/html")
        self.send_header("Cache-Control", "no-cache, no-store, must-revalidate")
        self.end_headers()
        self.wfile.write(html.encode())

    def do_GET(self):
        path = urlparse(self.path).path

        if path == "/" or path == "/dashboard":
            self._html_response(get_dashboard_html())

        elif path == "/api/status":
            info = get_system_info()
            self._json_response({
                "auto_clean_enabled": state["auto_clean_enabled"],
                "last_scan": state["last_scan"],
                "last_clean": state["last_clean"],
                "scanning": state["scanning"],
                "cleaning": state["cleaning"],
                "system": info,
            })

        elif path == "/api/scan":
            items = scan_garbage()
            self._json_response({"items": items, "count": len(items)})

        elif path == "/api/history":
            self._json_response(state["history"][:50])

        elif path == "/api/top-processes":
            self._json_response(get_ram_summary())

        elif path == "/api/alerts":
            self._json_response(state.get("alerts", []))

        else:
            self.send_error(404)

    def do_POST(self):
        path = urlparse(self.path).path

        if path == "/api/clean":
            summary = clean_all_garbage()
            self._json_response(summary)

        elif path == "/api/docker-prune":
            result = docker_prune()
            self._json_response(result)

        elif path == "/api/purge-ram":
            success = purge_ram()
            self._json_response({"success": success})

        elif path == "/api/toggle-auto":
            state["auto_clean_enabled"] = not state["auto_clean_enabled"]
            self._json_response({"enabled": state["auto_clean_enabled"]})

        else:
            self.send_error(404)


def main():
    load_history()

    print(f"""
\033[96m\033[1m
  ╔══════════════════════════════════════════════╗
  ║         MAC CLEANER SERVER v1.0              ║
  ║   Dashboard:  http://localhost:{PORT}          ║
  ║   Auto-clean: disabled by default            ║
  ╚══════════════════════════════════════════════╝
\033[0m""")

    # System info
    info = get_system_info()
    print(f"  Available RAM:  {fmt_size(info.get('available_ram', info['free_ram']))} (raw free {fmt_size(info['free_ram'])})")
    print(f"  Disk available: {info['disk_available']} of {info['disk_total']}")
    print(f"  Disk used:      {info['disk_percent']}%")
    print()

    # Initial scan + immediate cleanup
    print("  Running initial scan...")
    items = scan_garbage()
    cleanable_items = [i for i in items if i.get("cleanable", False)]
    total = sum(i["size"] for i in cleanable_items)
    print(f"  Found {len(cleanable_items)} cleanable garbage items ({fmt_size(total)})")
    system_data_items = [i for i in items if i.get("category") == "system-data"]
    if system_data_items:
        sd_total = sum(i["size"] for i in system_data_items)
        print(f"  System Data discovery: {len(system_data_items)} report-only item(s), {fmt_size(sd_total)} visible")

    if cleanable_items:
        print(f"  \033[93mStartup is scan-only. Use the dashboard button to clean selected known garbage.\033[0m")

    # Purge RAM on startup too
    print("  Purging RAM...")
    if purge_ram():
        print(f"  \033[92mRAM purged\033[0m")
    else:
        print(f"  \033[93mRAM purge needs sudo — run: sudo sh -c 'echo \"{os.getenv('USER')} ALL=(ALL) NOPASSWD: /usr/sbin/purge\" > /etc/sudoers.d/purge'\033[0m")

    print()
    print(f"  \033[92mServer running on http://localhost:{PORT}\033[0m")
    print(f"  \033[2mAuto-clean is OFF by default. Enable it in the dashboard if wanted.\033[0m")
    print(f"  \033[2mPress Ctrl+C to stop\033[0m\n")

    # Start auto-clean thread
    cleaner_thread = threading.Thread(target=auto_clean_loop, daemon=True)
    cleaner_thread.start()

    # Start memory leak watchdog (checks every 60s, alerts without killing)
    watchdog_thread = threading.Thread(target=memory_watchdog, daemon=True)
    watchdog_thread.start()
    print(f"  \033[92mMemory watchdog active\033[0m — alerts on apps using >4 GB")
    print(f"  \033[92mEmergency mode\033[0m — purges RAM and asks you to close apps if available RAM <1 GB\n")

    # Kill anything already on this port
    try:
        out = subprocess.run(["lsof", "-ti", f":{PORT}"], capture_output=True, text=True, timeout=5)
        if out.stdout.strip():
            for pid in out.stdout.strip().split("\n"):
                subprocess.run(["kill", "-9", pid.strip()], capture_output=True, timeout=5)
            time.sleep(0.5)
    except Exception:
        pass

    # Start HTTP server with SO_REUSEADDR
    import socket
    class ReusableHTTPServer(HTTPServer):
        allow_reuse_address = True
        def server_bind(self):
            self.socket.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
            super().server_bind()

    server = ReusableHTTPServer(("127.0.0.1", PORT), CleanerHandler)

    def shutdown(sig, frame):
        print(f"\n  \033[2mShutting down...\033[0m")
        os._exit(0)

    signal.signal(signal.SIGINT, shutdown)
    signal.signal(signal.SIGTERM, shutdown)

    server.serve_forever()


if __name__ == "__main__":
    main()
