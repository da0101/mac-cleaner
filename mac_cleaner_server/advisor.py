from __future__ import annotations

import time
from datetime import datetime
from typing import Any

from ai_advisor import build_snapshot, fallback_recommendations, get_ai_recommendations

from .memory import LOW_AVAILABLE_RAM, purge_ram
from .processes import get_ram_summary, get_top_memory_processes
from .state import setting, state
from .system import fmt_size, get_system_info

def get_ai_snapshot():
    """Build the structured snapshot consumed by the advisory model."""
    system = get_system_info()
    ram = get_ram_summary()
    storage = state.get("last_storage_items") or []
    return build_snapshot(system, ram, storage)


def refresh_ai_recommendations(force=False):
    """Refresh cached AI recommendations. Never executes recommendations."""
    if not state.get("ai_enabled", False):
        result = disabled_ai_recommendations()
        state["ai_recommendations"] = result
        return result

    if state.get("ai_refreshing") and not force:
        return state.get("ai_recommendations") or fallback_recommendations(get_ai_snapshot(), "AI refresh already running.")

    state["ai_refreshing"] = True
    try:
        snapshot = get_ai_snapshot()
        result = get_ai_recommendations(snapshot)
        state["ai_recommendations"] = result
        return result
    finally:
        state["ai_refreshing"] = False


def should_refresh_ai():
    """Return True when a new background recommendation pass is useful."""
    if not state.get("ai_enabled", False):
        return False

    current = state.get("ai_recommendations")
    if not current:
        return True

    try:
        generated = datetime.fromisoformat(current.get("generated_at", ""))
        if (datetime.now() - generated).total_seconds() > setting("ai_recommendation_interval_seconds"):
            return True
    except Exception:
        return True

    info = get_system_info()
    available = info.get("available_ram", info.get("free_ram", 0))
    if 0 < available < LOW_AVAILABLE_RAM:
        return True

    for proc in get_top_memory_processes(8):
        if proc.get("can_close") and proc.get("mem_bytes", 0) > 700 * 1024 * 1024:
            return True
    return False


def disabled_ai_recommendations():
    return {
        "status": "disabled",
        "provider": "disabled",
        "model": None,
        "generated_at": datetime.now().isoformat(),
        "summary": "Background AI is disabled. Restart with ./start --ai to enable Gemini recommendations.",
        "recommendations": [{
            "id": "ai-rec-1",
            "action": "do_nothing",
            "target_type": "none",
            "target_id": "",
            "target_label": "AI disabled",
            "priority": "low",
            "risk": "safe",
            "expected_savings_bytes": 0,
            "expected_savings": "0 B",
            "reason": "Run ./start --ai when you want Gemini to advise in the dashboard.",
            "executable": False,
        }],
    }

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
