import tempfile
import unittest
from pathlib import Path

from mac_cleaner_server import state as server_state
from mac_cleaner_server.dashboard import get_dashboard_asset, get_dashboard_html


class SettingsTests(unittest.TestCase):
    def test_sanitize_settings_clamps_intervals_and_accepts_booleans(self):
        settings = server_state.sanitize_settings({
            "theme": "light",
            "ai_recommendation_interval_seconds": 1,
            "chrome_tab_recommendation_interval_seconds": 999999,
            "show_ai_recommendations": False,
        })

        self.assertEqual(settings["theme"], "light")
        self.assertEqual(settings["ai_recommendation_interval_seconds"], 60)
        self.assertEqual(settings["chrome_tab_recommendation_interval_seconds"], 3600)
        self.assertFalse(settings["show_ai_recommendations"])

    def test_sanitize_settings_rejects_unknown_theme(self):
        settings = server_state.sanitize_settings({"theme": "purple"})

        self.assertEqual(settings["theme"], server_state.DEFAULT_SETTINGS["theme"])

    def test_save_settings_writes_sanitized_json(self):
        old_file = server_state.SETTINGS_FILE
        old_settings = server_state.state["settings"]
        with tempfile.TemporaryDirectory() as tmpdir:
            try:
                server_state.SETTINGS_FILE = Path(tmpdir) / "settings.json"

                saved = server_state.save_settings({"ai_auto_optimize_cooldown_seconds": 10})
                loaded = server_state.load_settings()

                self.assertEqual(saved["ai_auto_optimize_cooldown_seconds"], 60)
                self.assertEqual(loaded["ai_auto_optimize_cooldown_seconds"], 60)
                self.assertTrue(server_state.SETTINGS_FILE.exists())
            finally:
                server_state.SETTINGS_FILE = old_file
                server_state.state["settings"] = old_settings

    def test_dashboard_html_references_split_assets(self):
        html = get_dashboard_html()

        self.assertIn("/assets/dashboard.css", html)
        self.assertIn("/assets/dashboard_common.js", html)
        self.assertIsNotNone(get_dashboard_asset("dashboard.css"))
        self.assertIsNone(get_dashboard_asset("../settings.json"))


if __name__ == "__main__":
    unittest.main()
