import unittest

import ai_advisor
import browser_tabs


class BrowserTabsTests(unittest.TestCase):
    def tabs(self):
        return [
            browser_tabs.normalize_tab({
                "window_index": 1,
                "tab_index": 1,
                "tab_id": "a",
                "title": "Work Doc",
                "url": "https://docs.google.com/document/d/123?private=1",
                "active": True,
            }),
            browser_tabs.normalize_tab({
                "window_index": 1,
                "tab_index": 2,
                "tab_id": "b",
                "title": "Video",
                "url": "https://www.youtube.com/watch?v=abc",
                "active": False,
            }),
            browser_tabs.normalize_tab({
                "window_index": 1,
                "tab_index": 3,
                "tab_id": "c",
                "title": "Video",
                "url": "https://www.youtube.com/watch?v=abc",
                "active": False,
            }),
            browser_tabs.normalize_tab({
                "window_index": 2,
                "tab_index": 1,
                "tab_id": "d",
                "title": "New Tab",
                "url": "chrome://newtab/",
                "active": False,
            }),
            browser_tabs.normalize_tab({
                "window_index": 2,
                "tab_index": 2,
                "tab_id": "e",
                "title": "YouTube to MP3 Converter - Free Download",
                "url": "https://ytmp3.example/convert",
                "active": False,
            }),
        ]

    def test_normalize_tab_strips_query_from_url(self):
        tab = self.tabs()[0]

        self.assertEqual(tab["domain"], "docs.google.com")
        self.assertNotIn("private=1", tab["url"])

    def test_recommend_tabs_finds_duplicate_and_blank_tabs(self):
        recs = browser_tabs.recommend_tabs(self.tabs())
        reasons = {rec["reason_code"] for rec in recs}

        self.assertIn("duplicate", reasons)
        self.assertIn("empty", reasons)
        self.assertIn("converter", reasons)
        self.assertFalse(any(rec["active"] for rec in recs if "active" in rec))

    def test_converter_tabs_are_review_candidates(self):
        recs = browser_tabs.recommend_tabs(self.tabs())
        converter = [rec for rec in recs if rec["reason_code"] == "converter"]

        self.assertEqual(len(converter), 1)
        self.assertEqual(converter[0]["tab_id"], "e")

    def test_ai_tab_validation_rejects_active_and_unknown_tabs(self):
        snapshot = {"available": True, "tabs": self.tabs(), "recommendations": []}
        recs = ai_advisor.validate_tab_recommendations([
            {"tab_id": "a", "window_index": 1, "tab_index": 1, "priority": "high", "reason": "active"},
            {"tab_id": "missing", "window_index": 9, "tab_index": 9, "priority": "high", "reason": "unknown"},
            {"tab_id": "b", "window_index": 1, "tab_index": 2, "priority": "high", "reason": "duplicate"},
        ], snapshot)

        self.assertEqual(len(recs), 1)
        self.assertEqual(recs[0]["tab_id"], "b")
        self.assertEqual(recs[0]["action"], "close_tab")
        self.assertFalse(recs[0]["executable"])

    def test_find_matching_tab_prefers_tab_id(self):
        snapshot = {"tabs": self.tabs()}
        match = browser_tabs.find_matching_tab(snapshot, {"tab_id": "c", "window_index": 1, "tab_index": 99})

        self.assertIsNotNone(match)
        self.assertEqual(match["tab_index"], 3)

    def test_find_matching_tab_requires_metadata_without_tab_id(self):
        snapshot = {"tabs": self.tabs()}

        stale = browser_tabs.find_matching_tab(snapshot, {
            "window_index": 1,
            "tab_index": 2,
            "title": "Different tab",
            "domain": "youtube.com",
        })
        match = browser_tabs.find_matching_tab(snapshot, {
            "window_index": 1,
            "tab_index": 2,
            "title": "Video",
            "domain": "youtube.com",
            "url": "https://www.youtube.com/watch",
        })

        self.assertIsNone(stale)
        self.assertIsNotNone(match)
        self.assertEqual(match["tab_id"], "b")


if __name__ == "__main__":
    unittest.main()
