from __future__ import annotations

from datetime import datetime
from typing import Any

from ai_advisor import get_tab_recommendations
from browser_tabs import close_chrome_tab, find_matching_tab, get_chrome_tabs

from .state import setting, state

def refresh_chrome_tab_recommendations(force=False):
    current = state.get("chrome_tab_recommendations")
    if current and not force:
        try:
            generated = datetime.fromisoformat(current.get("generated_at", ""))
            if (datetime.now() - generated).total_seconds() < setting("chrome_tab_recommendation_interval_seconds"):
                return current
        except Exception:
            pass

    snapshot = get_chrome_tabs()
    state["chrome_tabs"] = snapshot
    result = get_tab_recommendations(snapshot) if state.get("ai_enabled", False) else {
        "status": "local",
        "provider": "local-rules",
        "model": None,
        "generated_at": datetime.now().isoformat(),
        "summary": "Local Chrome tab recommendations. Start with --ai for Gemini ranking.",
        "recommendations": snapshot.get("recommendations", []),
    }
    state["chrome_tab_recommendations"] = result
    return result


def close_requested_chrome_tab(request):
    fresh = get_chrome_tabs()
    match = find_matching_tab(fresh, request)
    if not match:
        return {"success": False, "error": "Tab no longer matches the current Chrome snapshot."}
    if match.get("active"):
        return {"success": False, "error": "Refusing to close the active Chrome tab automatically."}

    result = close_chrome_tab(match["window_index"], match["tab_index"])
    state["chrome_tabs"] = get_chrome_tabs()
    state["chrome_tab_recommendations"] = None
    return {
        "success": result.get("success", False),
        "error": result.get("error", ""),
        "closed": {
            "title": match.get("title", ""),
            "domain": match.get("domain", ""),
            "window_index": match.get("window_index"),
            "tab_index": match.get("tab_index"),
        },
    }
