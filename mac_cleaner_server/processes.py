from __future__ import annotations

import subprocess
from pathlib import Path

from .system import fmt_size, get_system_info

from .system import fmt_size, get_system_info

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
