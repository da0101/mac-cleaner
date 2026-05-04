from __future__ import annotations

import time
from datetime import datetime

from .advisor import refresh_ai_recommendations, run_ai_auto_memory_optimization, should_refresh_ai
from .memory import purge_ram
from .processes import CATEGORY_COLORS, get_ram_summary
from .state import setting, state
from .storage import clean_all_garbage
from .system import fmt_size, get_system_info

def auto_clean_loop():
    """Background thread: clean garbage every 15 min, purge RAM every 3 min."""
    while True:
        time.sleep(60)  # Check every minute
        now = time.time()

        if should_refresh_ai():
            ts = datetime.now().strftime("%H:%M:%S")
            print(f"  [{ts}] AI advisor refresh triggered...")
            result = refresh_ai_recommendations()
            recommendations = result.get("recommendations", [])
            print(f"  [{ts}] AI advisor: {result.get('summary', 'recommendations updated')}")
            run_ai_auto_memory_optimization(recommendations, reason="ai-advisor")
        else:
            run_ai_auto_memory_optimization(reason="ram-threshold")

        if state["auto_clean_enabled"]:
            if state.get("next_ram_purge") is None:
                state["next_ram_purge"] = now + setting("ram_purge_interval_seconds")
            if state.get("next_garbage_clean") is None:
                state["next_garbage_clean"] = now + setting("auto_clean_interval_seconds")

        # Purge RAM on configured cadence
        if state["auto_clean_enabled"] and now >= float(state.get("next_ram_purge") or 0):
            state["next_ram_purge"] = now + setting("ram_purge_interval_seconds")
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

        # Clean garbage on configured cadence
        if state["auto_clean_enabled"] and now >= float(state.get("next_garbage_clean") or 0) and not state["cleaning"]:
            state["next_garbage_clean"] = now + setting("auto_clean_interval_seconds")
            ts = datetime.now().strftime("%H:%M:%S")
            print(f"  [{ts}] Auto-clean triggered...")
            summary = clean_all_garbage()
            print(f"  [{ts}] Cleaned {summary['total_cleaned']} ({summary['items_cleaned']} items)")
