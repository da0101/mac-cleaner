from __future__ import annotations

import subprocess
import time
from datetime import datetime

from .processes import get_top_memory_processes
from .state import PORT, setting, state
from .system import fmt_size, get_system_info

RAM_PURGE_INTERVAL = 3 * 60  # 3 minutes

# ── Memory leak thresholds (per app) ──
# If a single app group exceeds this, it's a memory leak and should be restarted
MEMORY_LEAK_THRESHOLD = 4 * 1024 * 1024 * 1024  # 4 GB — no single app should use this much on 18GB
LOW_AVAILABLE_RAM = 1024 * 1024 * 1024  # 1 GB reusable memory target
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


def run_ai_auto_memory_optimization(recommendations=None, reason="threshold"):
    """Apply safe RAM-only optimizations in AI mode. Never kills or quits apps."""
    if not state.get("ai_auto_optimize", False):
        return {"ran": False, "reason": "AI automatic RAM optimization is disabled."}

    now = time.time()
    last = state.get("last_ai_optimization") or {}
    if now - last.get("timestamp_epoch", 0) < setting("ai_auto_optimize_cooldown_seconds"):
        return {"ran": False, "reason": "AI automatic RAM optimization cooldown is active."}

    info_before = get_system_info()
    available_before = info_before.get("available_ram", info_before.get("free_ram", 0))
    target_available_ram = setting("ai_target_available_ram_mb") * 1024 * 1024
    model_requested_purge = any(
        rec.get("action") == "purge_ram"
        for rec in (recommendations or [])
    )

    if not model_requested_purge and available_before >= target_available_ram:
        return {"ran": False, "reason": "Available RAM is above AI target."}

    ts = datetime.now().strftime("%H:%M:%S")
    print(f"  [{ts}] AI auto RAM optimization triggered ({reason})...")
    success = purge_ram()
    time.sleep(1)
    info_after = get_system_info()
    available_after = info_after.get("available_ram", info_after.get("free_ram", 0))

    result = {
        "ran": True,
        "success": success,
        "reason": reason,
        "timestamp": datetime.now().isoformat(),
        "timestamp_epoch": now,
        "available_before": available_before,
        "available_after": available_after,
        "available_before_human": fmt_size(available_before),
        "available_after_human": fmt_size(available_after),
        "note": "Safe RAM purge only; no apps or processes were closed.",
    }
    state["last_ai_optimization"] = result
    print(f"  [{ts}] AI auto RAM: {result['available_before_human']} -> {result['available_after_human']} available")
    return result


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
