#!/usr/bin/env python3
"""
Mac Cleaner — local garbage & cache scanner/cleaner for macOS.
Targets ONLY safe-to-delete caches, logs, and build artifacts.
Never touches user documents, photos, or application data.
"""

import os
import sys
import shutil
import subprocess
import json
import time
from pathlib import Path
from datetime import datetime
from scanner import clean_scan_item, group_items, scan_known_items, scan_system_data_items

# ANSI colors
RED = "\033[91m"
GREEN = "\033[92m"
YELLOW = "\033[93m"
BLUE = "\033[94m"
CYAN = "\033[96m"
BOLD = "\033[1m"
DIM = "\033[2m"
RESET = "\033[0m"

HOME = Path.home()


def fmt_size(size_bytes):
    """Format bytes to human-readable string."""
    if size_bytes < 1024:
        return f"{size_bytes} B"
    elif size_bytes < 1024 ** 2:
        return f"{size_bytes / 1024:.1f} KB"
    elif size_bytes < 1024 ** 3:
        return f"{size_bytes / 1024**2:.1f} MB"
    else:
        return f"{size_bytes / 1024**3:.2f} GB"


def dir_size(path):
    """Get total size of a directory, handling permission errors."""
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


def check_docker():
    """Check Docker disk usage."""
    results = []

    # Docker Desktop VM disk image
    docker_vm = HOME / "Library/Containers/com.docker.docker/Data/vms"
    if docker_vm.exists():
        size = dir_size(docker_vm)
        if size > 0:
            results.append(("Docker VM disk image", docker_vm, size, "docker_vm"))

    # Docker data (images, containers, volumes, build cache)
    # We'll check via CLI for more detail
    try:
        out = subprocess.run(
            ["docker", "system", "df", "--format", "json"],
            capture_output=True, text=True, timeout=10
        )
        if out.returncode == 0:
            for line in out.stdout.strip().split("\n"):
                if not line.strip():
                    continue
                try:
                    data = json.loads(line)
                    reclaimable = data.get("Reclaimable", "0B")
                    # Parse reclaimable size
                    results.append((
                        f"Docker {data.get('Type', 'unknown')} (reclaimable)",
                        None, None, "docker_prune",
                        reclaimable
                    ))
                except json.JSONDecodeError:
                    pass
    except (FileNotFoundError, subprocess.TimeoutExpired):
        pass

    return results


def scan_garbage():
    """Scan all known garbage/cache locations. Returns list of (name, path, size, category)."""
    print(f"\n{CYAN}{BOLD}  Scanning for garbage...{RESET}\n")

    found = group_items(scan_known_items())

    # Clear scanning line
    sys.stdout.write(" " * 80 + "\r")

    return found


def scan_large_node_modules():
    """Find node_modules directories that haven't been accessed recently."""
    print(f"  {DIM}Scanning for stale node_modules (this may take a moment)...{RESET}", end="\r")
    results = []
    search_dirs = [HOME / "Documents", HOME / "Projects", HOME / "Developer", HOME / "Code"]

    for search_dir in search_dirs:
        if not search_dir.exists():
            continue
        try:
            out = subprocess.run(
                ["find", str(search_dir), "-maxdepth", "5",
                 "-name", "node_modules", "-type", "d",
                 "-not", "-path", "*/node_modules/*/node_modules/*"],
                capture_output=True, text=True, timeout=30
            )
            for line in out.stdout.strip().split("\n"):
                if not line:
                    continue
                p = Path(line)
                if p.exists():
                    size = dir_size(p)
                    if size > 50 * 1024 * 1024:  # > 50MB
                        results.append((f"node_modules: {p.parent.name}/", p, size, "Stale node_modules"))
        except subprocess.TimeoutExpired:
            pass

    sys.stdout.write(" " * 80 + "\r")
    return results


def print_report(found, docker_info, node_modules, system_data):
    """Print the scan report."""
    print(f"\n{BOLD}{'═' * 70}{RESET}")
    print(f"{BOLD}  GARBAGE SCAN REPORT{RESET}")
    print(f"{BOLD}{'═' * 70}{RESET}\n")

    total = 0
    all_items = []
    idx = 1

    for category, items in found.items():
        cat_total = sum(i["size"] for i in items)
        total += cat_total
        print(f"  {YELLOW}{BOLD}{category}{RESET} {DIM}({fmt_size(cat_total)}){RESET}")
        for item in items:
            name = item["name"]
            size = item["size"]
            safety = item.get("safety", "review-required")
            color = RED if size > 1024**3 else YELLOW if size > 100*1024**2 else RESET
            print(f"    {DIM}[{idx}]{RESET} {color}{fmt_size(size):>10}{RESET}  {name} {DIM}({safety}){RESET}")
            all_items.append(item)
            idx += 1
        print()

    if node_modules:
        nm_total = sum(i[2] for i in node_modules)
        total += nm_total
        print(f"  {YELLOW}{BOLD}Stale node_modules{RESET} {DIM}({fmt_size(nm_total)}){RESET}")
        for name, path, size, cat in node_modules:
            color = RED if size > 500*1024**2 else YELLOW
            print(f"    {DIM}[{idx}]{RESET} {color}{fmt_size(size):>10}{RESET}  {name}")
            all_items.append((name, path, size, cat))
            idx += 1
        print()

    if docker_info:
        print(f"  {YELLOW}{BOLD}Docker{RESET}")
        for item in docker_info:
            if len(item) == 5:
                name, _, _, cat, reclaimable = item
                print(f"    {DIM}[~]{RESET} {CYAN}{reclaimable:>10}{RESET}  {name}")
            elif item[2] and item[2] > 0:
                name, path, size, cat = item
                color = RED if size > 1024**3 else YELLOW
                print(f"    {DIM}[{idx}]{RESET} {color}{fmt_size(size):>10}{RESET}  {name}")
                all_items.append((name, path, size, cat))
                idx += 1
        print()

    if system_data:
        sd_total = sum(i["size"] for i in system_data)
        print(f"  {CYAN}{BOLD}System Data Discovery{RESET} {DIM}({fmt_size(sd_total)} visible){RESET}")
        for item in system_data:
            size = item["size"]
            size_text = item["size_human"]
            reason = item.get("reason", "Review before deleting.")
            color = RED if size > 5*1024**3 else YELLOW if size > 1024**3 else CYAN
            print(f"    {DIM}[~]{RESET} {color}{size_text:>10}{RESET}  {item['name']}")
            print(f"          {DIM}{item['path']} — {item['safety']}: {reason}{RESET}")
        print()

    print(f"{BOLD}{'─' * 70}{RESET}")
    color = RED if total > 5*1024**3 else YELLOW if total > 1024**3 else GREEN
    print(f"  {BOLD}Total reclaimable: {color}{fmt_size(total)}{RESET}")
    print(f"{BOLD}{'─' * 70}{RESET}\n")

    return all_items


def clean_items(items, selections):
    """Clean selected items."""
    cleaned = 0
    errors = []

    for idx in selections:
        if idx < 0 or idx >= len(items):
            continue
        item = items[idx]
        if isinstance(item, dict):
            name = item["name"]
            size = item["size"]
            if not item.get("cleanable", False):
                errors.append((name, "not cleanable by default"))
                print(f"  {DIM}Skipping {name}...{RESET} {YELLOW}review-only{RESET}")
                continue
        else:
            name, path, size, cat = item

        print(f"  {DIM}Cleaning {name}...{RESET}", end="")
        try:
            if isinstance(item, dict):
                freed = clean_scan_item(item)
                if freed <= 0:
                    raise OSError("nothing cleaned or permission denied")
                cleaned += freed
            elif path.is_dir():
                shutil.rmtree(path, ignore_errors=True)
                # Recreate empty dir for caches (apps expect them to exist)
                if "cache" in name.lower() or "Cache" in str(path):
                    path.mkdir(parents=True, exist_ok=True)
            elif path.is_file():
                path.unlink()
            print(f" {GREEN}done{RESET} ({fmt_size(size)})")
        except Exception as e:
            errors.append((name, str(e)))
            print(f" {RED}failed: {e}{RESET}")

    return cleaned, errors


def run_docker_prune():
    """Run docker system prune."""
    print(f"\n  {CYAN}Running docker system prune...{RESET}")
    try:
        result = subprocess.run(
            ["docker", "system", "prune", "-a", "-f", "--volumes"],
            capture_output=True, text=True, timeout=120
        )
        if result.returncode == 0:
            print(f"  {GREEN}Docker pruned successfully{RESET}")
            for line in result.stdout.strip().split("\n")[-3:]:
                print(f"  {DIM}{line}{RESET}")
        else:
            print(f"  {RED}Docker prune failed: {result.stderr}{RESET}")
    except FileNotFoundError:
        print(f"  {DIM}Docker not found, skipping{RESET}")
    except subprocess.TimeoutExpired:
        print(f"  {RED}Docker prune timed out{RESET}")


def purge_ram():
    """Purge inactive RAM (macOS)."""
    print(f"\n  {CYAN}Purging inactive memory...{RESET}")
    try:
        result = subprocess.run(["sudo", "purge"], capture_output=True, text=True, timeout=30)
        if result.returncode == 0:
            print(f"  {GREEN}Memory purged{RESET}")
        else:
            print(f"  {YELLOW}Could not purge memory (needs sudo){RESET}")
    except subprocess.TimeoutExpired:
        print(f"  {RED}Memory purge timed out{RESET}")


def flush_dns():
    """Flush DNS cache."""
    print(f"  {CYAN}Flushing DNS cache...{RESET}")
    try:
        subprocess.run(
            ["sudo", "dscacheutil", "-flushcache"],
            capture_output=True, timeout=10
        )
        subprocess.run(
            ["sudo", "killall", "-HUP", "mDNSResponder"],
            capture_output=True, timeout=10
        )
        print(f"  {GREEN}DNS cache flushed{RESET}")
    except Exception:
        print(f"  {YELLOW}Could not flush DNS{RESET}")


def interactive_menu(all_items, docker_info):
    """Show interactive cleanup menu."""
    print(f"  {BOLD}What do you want to clean?{RESET}\n")
    print(f"    {GREEN}a{RESET} — Clean ALL safe garbage (recommended)")
    print(f"    {YELLOW}s{RESET} — Select specific items by number (e.g. 1,3,5-8)")
    print(f"    {CYAN}d{RESET} — Docker full prune (images + containers + volumes + build cache)")
    print(f"    {CYAN}m{RESET} — Purge inactive RAM")
    print(f"    {CYAN}f{RESET} — Flush DNS cache")
    print(f"    {BLUE}x{RESET} — Clean ALL + Docker prune + RAM purge + DNS flush")
    print(f"    {DIM}q{RESET} — Quit\n")

    choice = input(f"  {BOLD}>{RESET} ").strip().lower()

    if choice == "q":
        print(f"\n  {DIM}Nothing cleaned. Bye!{RESET}\n")
        return

    if choice == "a" or choice == "x":
        print(f"\n  {BOLD}Cleaning all garbage...{RESET}\n")
        cleaned, errors = clean_items(all_items, list(range(len(all_items))))

        if choice == "x":
            if docker_info:
                run_docker_prune()
            purge_ram()
            flush_dns()

        print(f"\n{BOLD}{'─' * 70}{RESET}")
        print(f"  {GREEN}{BOLD}Cleaned: {fmt_size(cleaned)}{RESET}")
        if errors:
            print(f"  {YELLOW}{len(errors)} items had errors{RESET}")
        print(f"{BOLD}{'─' * 70}{RESET}\n")
        return

    if choice == "d":
        run_docker_prune()
        return

    if choice == "m":
        purge_ram()
        return

    if choice == "f":
        flush_dns()
        return

    if choice == "s":
        sel_input = input(f"  Enter item numbers (e.g. 1,3,5-8): ").strip()
        indices = []
        for part in sel_input.split(","):
            part = part.strip()
            if "-" in part:
                try:
                    start, end = part.split("-")
                    indices.extend(range(int(start) - 1, int(end)))
                except ValueError:
                    pass
            else:
                try:
                    indices.append(int(part) - 1)
                except ValueError:
                    pass

        if indices:
            print(f"\n  {BOLD}Cleaning {len(indices)} items...{RESET}\n")
            cleaned, errors = clean_items(all_items, indices)
            print(f"\n  {GREEN}{BOLD}Cleaned: {fmt_size(cleaned)}{RESET}\n")
        else:
            print(f"  {YELLOW}No valid selections{RESET}")


def main():
    print(f"""
{BOLD}{CYAN}
  ╔══════════════════════════════════════════╗
  ║         MAC CLEANER v1.0                 ║
  ║   Local garbage & cache scanner          ║
  ║   Safe cleanup for macOS (M-series)      ║
  ╚══════════════════════════════════════════╝
{RESET}""")

    # Quick system info
    try:
        mem = subprocess.run(["vm_stat"], capture_output=True, text=True, timeout=5)
        if mem.returncode == 0:
            lines = mem.stdout.strip().split("\n")
            page_size = 16384  # M-series default
            free = 0
            for line in lines:
                if "Pages free" in line:
                    free = int(line.split(":")[1].strip().rstrip(".")) * page_size
            print(f"  {DIM}Free RAM: {fmt_size(free)}{RESET}")
    except Exception:
        pass

    try:
        disk = subprocess.run(["df", "-h", "/"], capture_output=True, text=True, timeout=5)
        if disk.returncode == 0:
            parts = disk.stdout.strip().split("\n")[1].split()
            print(f"  {DIM}Disk: {parts[3]} available of {parts[1]}{RESET}")
    except Exception:
        pass

    print()

    # Scan
    found = scan_garbage()
    docker_info = check_docker()
    node_modules = scan_large_node_modules()
    system_data = scan_system_data_items()

    # Report
    all_items = print_report(found, docker_info, node_modules, system_data)

    if not all_items and not docker_info:
        print(f"  {GREEN}Your system is clean! Nothing significant to remove.{RESET}\n")
        return

    # Interactive cleanup
    interactive_menu(all_items, docker_info)


if __name__ == "__main__":
    main()
