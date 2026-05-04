from __future__ import annotations

import argparse
import json
import time
from pathlib import Path

HOME = Path.home()
PROJECT_DIR = Path(__file__).resolve().parent.parent
PORT = 3333
LOG_FILE = PROJECT_DIR / "cleanup_history.json"
SETTINGS_FILE = PROJECT_DIR / "settings.json"

DEFAULT_SETTINGS = {
    "theme": "dark",
    "ai_recommendation_interval_seconds": 300,
    "chrome_tab_recommendation_interval_seconds": 300,
    "ai_auto_optimize_cooldown_seconds": 300,
    "auto_clean_interval_seconds": 900,
    "ram_purge_interval_seconds": 300,
    "ai_target_available_ram_mb": 3072,
    "dashboard_status_interval_seconds": 10,
    "dashboard_process_interval_seconds": 15,
    "dashboard_alert_interval_seconds": 10,
    "dashboard_history_interval_seconds": 60,
    "show_ai_recommendations": True,
    "show_chrome_tab_optimizer": True,
    "show_garbage_breakdown": True,
    "show_ram_breakdown": True,
    "show_rule_recommendations": True,
    "show_cleanup_history": True,
}

SETTING_LIMITS = {
    "ai_recommendation_interval_seconds": (60, 3600),
    "chrome_tab_recommendation_interval_seconds": (60, 3600),
    "ai_auto_optimize_cooldown_seconds": (60, 1800),
    "auto_clean_interval_seconds": (300, 86400),
    "ram_purge_interval_seconds": (60, 3600),
    "ai_target_available_ram_mb": (512, 16384),
    "dashboard_status_interval_seconds": (5, 300),
    "dashboard_process_interval_seconds": (5, 300),
    "dashboard_alert_interval_seconds": (5, 300),
    "dashboard_history_interval_seconds": (10, 600),
}

# ── Shared state ──
state = {
    "last_scan": None,
    "last_clean": None,
    "history": [],
    "auto_clean_enabled": False,
    "scanning": False,
    "cleaning": False,
    "last_storage_items": [],
    "ai_recommendations": None,
    "ai_refreshing": False,
    "ai_enabled": False,
    "ai_auto_optimize": False,
    "last_ai_optimization": None,
    "next_ram_purge": None,
    "next_garbage_clean": None,
    "chrome_tabs": None,
    "chrome_tab_recommendations": None,
    "settings": DEFAULT_SETTINGS.copy(),
}


def parse_args(argv):
    parser = argparse.ArgumentParser(description="Run the local Mac Cleaner dashboard.")
    ai_group = parser.add_mutually_exclusive_group()
    ai_group.add_argument("--ai", action="store_true", help="Enable Gemini recommendations and automatic safe RAM optimization.")
    ai_group.add_argument("--ai-advisory", action="store_true", help="Enable Gemini recommendations without automatic RAM optimization.")
    ai_group.add_argument("--no-ai", action="store_true", help="Disable background AI recommendations.")
    return parser.parse_args(argv)


def sanitize_settings(raw):
    settings = DEFAULT_SETTINGS.copy()
    if not isinstance(raw, dict):
        return settings

    for key, default in DEFAULT_SETTINGS.items():
        if isinstance(default, bool):
            settings[key] = bool(raw.get(key, default))
            continue
        if key == "theme":
            theme = str(raw.get(key, default))
            settings[key] = theme if theme in {"dark", "light", "system"} else default
            continue
        value = raw.get(key, default)
        try:
            value = int(value)
        except (TypeError, ValueError):
            value = default
        minimum, maximum = SETTING_LIMITS[key]
        settings[key] = max(minimum, min(maximum, value))
    return settings


def load_settings():
    if not SETTINGS_FILE.exists():
        state["settings"] = DEFAULT_SETTINGS.copy()
        return state["settings"]
    try:
        with open(SETTINGS_FILE) as f:
            state["settings"] = sanitize_settings(json.load(f))
    except Exception:
        state["settings"] = DEFAULT_SETTINGS.copy()
    return state["settings"]


def save_settings(settings):
    state["settings"] = sanitize_settings(settings)
    with open(SETTINGS_FILE, "w") as f:
        json.dump(state["settings"], f, indent=2)
        f.write("\n")
    return state["settings"]


def setting(name):
    return state.get("settings", DEFAULT_SETTINGS).get(name, DEFAULT_SETTINGS[name])


def set_auto_clean_schedule(now=None):
    now = time.time() if now is None else now
    state["next_ram_purge"] = now + setting("ram_purge_interval_seconds")
    state["next_garbage_clean"] = now + setting("auto_clean_interval_seconds")


def clear_auto_clean_schedule():
    state["next_ram_purge"] = None
    state["next_garbage_clean"] = None


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
