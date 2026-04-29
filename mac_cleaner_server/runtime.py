from __future__ import annotations

import os
import signal
import socket
import subprocess
import sys
import threading
import time
from http.server import HTTPServer

from .advisor import refresh_ai_recommendations, run_ai_auto_memory_optimization
from .background import auto_clean_loop
from .http_api import CleanerHandler
from .memory import memory_watchdog, purge_ram
from .state import PORT, load_history, load_settings, parse_args, state
from .storage import scan_garbage
from .system import fmt_size, get_system_info

def print_startup_banner():
    print(f"""
[96m[1m
  ╔══════════════════════════════════════════════╗
  ║         MAC CLEANER SERVER v1.0              ║
  ║   Dashboard:  http://localhost:{PORT}          ║
  ║   Auto-clean: disabled by default            ║
  ║   AI mode: {'auto RAM' if state["ai_auto_optimize"] else 'advisory' if state["ai_enabled"] else 'disabled'}                         ║
  ╚══════════════════════════════════════════════╝
[0m""")


def print_initial_status():
    info = get_system_info()
    print(f"  Available RAM:  {fmt_size(info.get('available_ram', info['free_ram']))} (raw free {fmt_size(info['free_ram'])})")
    print(f"  Disk available: {info['disk_available']} of {info['disk_total']}")
    print(f"  Disk used:      {info['disk_percent']}%")
    print()


def run_initial_scan():
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
        print("  [93mStartup is scan-only. Use the dashboard button to clean selected known garbage.[0m")


def purge_ram_on_startup():
    print("  Purging RAM...")
    if purge_ram():
        print("  [92mRAM purged[0m")
    else:
        user = os.getenv("USER")
        print(f"  [93mRAM purge needs sudo — run: sudo sh -c 'echo \"{user} ALL=(ALL) NOPASSWD: /usr/sbin/purge\" > /etc/sudoers.d/purge'[0m")


def initialize_ai_mode():
    if state["ai_enabled"]:
        if state["ai_auto_optimize"]:
            print("  [92mAI auto RAM optimization enabled[0m — Gemini advises; mac-cleaner auto-purges RAM safely.")
        else:
            print("  [92mAI advisor enabled[0m — Gemini recommendations are advisory only.")
        result = refresh_ai_recommendations(force=True)
        run_ai_auto_memory_optimization(result.get("recommendations", []), reason="startup")
    else:
        print("  [2mAI disabled. Start with ./start --ai for automatic safe RAM optimization.[0m")


def stop_existing_server_on_port():
    try:
        out = subprocess.run(["lsof", "-ti", f":{PORT}"], capture_output=True, text=True, timeout=5)
        if out.stdout.strip():
            for pid in out.stdout.strip().split("\n"):
                subprocess.run(["kill", "-9", pid.strip()], capture_output=True, timeout=5)
            time.sleep(0.5)
    except Exception:
        pass


class ReusableHTTPServer(HTTPServer):
    allow_reuse_address = True

    def server_bind(self):
        self.socket.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        super().server_bind()


def main(argv=None):
    args = parse_args(sys.argv[1:] if argv is None else argv)
    state["ai_enabled"] = bool((args.ai or args.ai_advisory) and not args.no_ai)
    state["ai_auto_optimize"] = bool(args.ai and not args.no_ai)
    load_settings()
    load_history()

    print_startup_banner()
    print_initial_status()
    run_initial_scan()
    purge_ram_on_startup()

    print()
    print(f"  [92mServer running on http://localhost:{PORT}[0m")
    print("  [2mAuto-clean is OFF by default. Enable it in the dashboard if wanted.[0m")
    initialize_ai_mode()
    print("  [2mPress Ctrl+C to stop[0m\n")

    threading.Thread(target=auto_clean_loop, daemon=True).start()
    threading.Thread(target=memory_watchdog, daemon=True).start()
    print("  [92mMemory watchdog active[0m — alerts on apps using >4 GB")
    print("  [92mEmergency mode[0m — purges RAM and asks you to close apps if available RAM <1 GB\n")

    stop_existing_server_on_port()
    server = ReusableHTTPServer(("127.0.0.1", PORT), CleanerHandler)

    def shutdown(sig, frame):
        print("\n  [2mShutting down...[0m")
        os._exit(0)

    signal.signal(signal.SIGINT, shutdown)
    signal.signal(signal.SIGTERM, shutdown)
    server.serve_forever()
