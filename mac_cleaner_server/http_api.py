from __future__ import annotations

import json
import threading
from http.server import BaseHTTPRequestHandler
from urllib.parse import urlparse

from ai_advisor import fallback_recommendations
from browser_tabs import get_chrome_tabs

from .advisor import disabled_ai_recommendations, get_ai_snapshot, refresh_ai_recommendations, should_refresh_ai
from .chrome import close_requested_chrome_tab, refresh_chrome_tab_recommendations
from .dashboard import get_dashboard_asset, get_dashboard_html
from .memory import purge_ram
from .processes import get_ram_summary
from .state import save_settings, state
from .storage import clean_all_garbage, docker_prune, scan_garbage
from .system import get_system_info

class CleanerHandler(BaseHTTPRequestHandler):
    def log_message(self, format, *args):
        # Suppress default request logs; terminal output is reserved for important status changes.
        pass

    def _json_response(self, data, status=200):
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.end_headers()
        self.wfile.write(json.dumps(data).encode())

    def _html_response(self, html):
        self.send_response(200)
        self.send_header("Content-Type", "text/html")
        self.send_header("Cache-Control", "no-cache, no-store, must-revalidate")
        self.end_headers()
        self.wfile.write(html.encode())

    def _asset_response(self, content, content_type):
        self.send_response(200)
        self.send_header("Content-Type", content_type)
        self.send_header("Cache-Control", "no-cache")
        self.end_headers()
        self.wfile.write(content.encode())

    def _read_json_body(self):
        length = int(self.headers.get("Content-Length", "0") or 0)
        if length <= 0:
            return {}
        try:
            return json.loads(self.rfile.read(length).decode())
        except Exception:
            return {}

    def do_GET(self):
        path = urlparse(self.path).path

        if path in {"/", "/dashboard"}:
            self._html_response(get_dashboard_html())

        elif path.startswith("/assets/"):
            asset = get_dashboard_asset(path.removeprefix("/assets/"))
            if not asset:
                self.send_error(404)
                return
            content, content_type = asset
            self._asset_response(content, content_type)

        elif path == "/api/status":
            info = get_system_info()
            self._json_response({
                "auto_clean_enabled": state["auto_clean_enabled"],
                "ai_enabled": state["ai_enabled"],
                "ai_auto_optimize": state["ai_auto_optimize"],
                "last_ai_optimization": state["last_ai_optimization"],
                "last_scan": state["last_scan"],
                "last_clean": state["last_clean"],
                "scanning": state["scanning"],
                "cleaning": state["cleaning"],
                "system": info,
                "settings": state["settings"],
            })

        elif path == "/api/settings":
            self._json_response(state["settings"])

        elif path == "/api/scan":
            items = scan_garbage()
            if state.get("ai_enabled", False):
                threading.Thread(target=refresh_ai_recommendations, daemon=True).start()
            self._json_response({"items": items, "count": len(items)})

        elif path == "/api/history":
            self._json_response(state["history"][:50])

        elif path == "/api/top-processes":
            self._json_response(get_ram_summary())

        elif path == "/api/alerts":
            self._json_response(state.get("alerts", []))

        elif path == "/api/ai-recommendations":
            result = get_ai_recommendation_response()
            self._json_response(result)

        elif path == "/api/chrome-tabs":
            snapshot = get_chrome_tabs()
            state["chrome_tabs"] = snapshot
            self._json_response(snapshot)

        elif path == "/api/chrome-tab-recommendations":
            result = refresh_chrome_tab_recommendations()
            self._json_response(result)

        else:
            self.send_error(404)

    def do_POST(self):
        path = urlparse(self.path).path

        if path == "/api/clean":
            summary = clean_all_garbage()
            self._json_response(summary)

        elif path == "/api/docker-prune":
            result = docker_prune()
            self._json_response(result)

        elif path == "/api/purge-ram":
            success = purge_ram()
            self._json_response({"success": success})

        elif path == "/api/toggle-auto":
            state["auto_clean_enabled"] = not state["auto_clean_enabled"]
            self._json_response({"enabled": state["auto_clean_enabled"]})

        elif path == "/api/settings":
            settings = save_settings(self._read_json_body())
            self._json_response(settings)

        elif path == "/api/ai-recommendations":
            result = refresh_ai_recommendations(force=True)
            self._json_response(result)

        elif path == "/api/chrome-tab-recommendations":
            result = refresh_chrome_tab_recommendations(force=True)
            self._json_response(result)

        elif path == "/api/chrome-tabs/close":
            payload = self._read_json_body()
            result = close_requested_chrome_tab(payload)
            self._json_response(result, status=200 if result.get("success") else 400)

        else:
            self.send_error(404)


def get_ai_recommendation_response():
    if not state.get("ai_enabled", False):
        result = disabled_ai_recommendations()
        state["ai_recommendations"] = result
    elif should_refresh_ai():
        refresh_ai_recommendations()
        result = state.get("ai_recommendations")
    else:
        result = state.get("ai_recommendations")
    if not result:
        result = fallback_recommendations(get_ai_snapshot(), "AI recommendations are not ready yet.")
        state["ai_recommendations"] = result
    return result
