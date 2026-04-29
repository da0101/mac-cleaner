from __future__ import annotations

import subprocess

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
