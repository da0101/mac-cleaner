from __future__ import annotations

from pathlib import Path

ASSET_DIR = Path(__file__).with_name("assets")
ASSET_TYPES = {
    "dashboard.html": "text/html",
    "dashboard.css": "text/css",
    "dashboard_common.js": "application/javascript",
    "dashboard_actions.js": "application/javascript",
    "dashboard_processes.js": "application/javascript",
    "dashboard_boot.js": "application/javascript",
}


def get_dashboard_html() -> str:
    return read_dashboard_asset("dashboard.html")[0]


def get_dashboard_asset(name: str) -> tuple[str, str] | None:
    if name not in ASSET_TYPES or name == "dashboard.html":
        return None
    return read_dashboard_asset(name)


def read_dashboard_asset(name: str) -> tuple[str, str]:
    if name not in ASSET_TYPES:
        raise ValueError(f"Unknown dashboard asset: {name}")
    path = ASSET_DIR / name
    return path.read_text(), ASSET_TYPES[name]
