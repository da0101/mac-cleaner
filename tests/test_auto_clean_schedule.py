import unittest

from mac_cleaner_server.http_api import auto_clean_schedule_payload, set_auto_clean_enabled
from mac_cleaner_server.state import clear_auto_clean_schedule, set_auto_clean_schedule, state


class AutoCleanScheduleTests(unittest.TestCase):
    def setUp(self):
        self.previous = {
            "auto_clean_enabled": state["auto_clean_enabled"],
            "next_ram_purge": state.get("next_ram_purge"),
            "next_garbage_clean": state.get("next_garbage_clean"),
            "settings": state["settings"].copy(),
        }

    def tearDown(self):
        state["auto_clean_enabled"] = self.previous["auto_clean_enabled"]
        state["next_ram_purge"] = self.previous["next_ram_purge"]
        state["next_garbage_clean"] = self.previous["next_garbage_clean"]
        state["settings"] = self.previous["settings"]

    def test_set_auto_clean_schedule_uses_configured_intervals(self):
        state["settings"]["ram_purge_interval_seconds"] = 120
        state["settings"]["auto_clean_interval_seconds"] = 600

        set_auto_clean_schedule(now=1000)

        self.assertEqual(state["next_ram_purge"], 1120)
        self.assertEqual(state["next_garbage_clean"], 1600)

    def test_clear_auto_clean_schedule_removes_next_run_times(self):
        set_auto_clean_schedule(now=1000)

        clear_auto_clean_schedule()

        self.assertIsNone(state["next_ram_purge"])
        self.assertIsNone(state["next_garbage_clean"])

    def test_set_auto_clean_enabled_is_idempotent(self):
        state["auto_clean_enabled"] = True
        set_auto_clean_schedule(now=1000)
        first_ram = state["next_ram_purge"]

        payload = set_auto_clean_enabled(True)

        self.assertTrue(payload["enabled"])
        self.assertTrue(payload["auto_clean_enabled"])
        self.assertGreaterEqual(state["next_ram_purge"], first_ram)
        self.assertIsNotNone(state["next_garbage_clean"])

    def test_schedule_payload_repairs_missing_enabled_schedule(self):
        state["auto_clean_enabled"] = True
        state["next_ram_purge"] = None
        state["next_garbage_clean"] = None

        payload = auto_clean_schedule_payload()

        self.assertTrue(payload["enabled"])
        self.assertIsNotNone(payload["next_ram_purge"])
        self.assertIsNotNone(payload["next_garbage_clean"])


if __name__ == "__main__":
    unittest.main()
