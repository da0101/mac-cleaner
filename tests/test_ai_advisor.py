import unittest

import ai_advisor


class AiAdvisorTests(unittest.TestCase):
    def snapshot(self):
        return {
            "system": {"available_ram": 2 * 1024**3, "free_ram": 100 * 1024**2, "total_ram": 18 * 1024**3},
            "ram": {
                "processes": [
                    {
                        "label": "VS Code",
                        "category": "ide",
                        "can_close": False,
                        "memory_bytes": 900 * 1024**2,
                    },
                    {
                        "label": "Chrome",
                        "category": "browser",
                        "can_close": True,
                        "memory_bytes": 800 * 1024**2,
                    },
                ],
            },
            "storage": [
                {
                    "name": "App caches",
                    "cleanable": True,
                    "safety": "safe-cache",
                    "size_bytes": 500 * 1024**2,
                },
                {
                    "name": "System Data: Application Support",
                    "cleanable": False,
                    "safety": "never-default-delete",
                    "size_bytes": 5 * 1024**3,
                },
            ],
        }

    def test_protected_process_close_is_downgraded_to_review(self):
        payload = {
            "recommendations": [{
                "action": "close_app",
                "target_type": "process",
                "target_id": "VS Code",
                "target_label": "VS Code",
                "priority": "high",
                "risk": "safe",
                "expected_savings_bytes": 900 * 1024**2,
                "reason": "Large app",
            }]
        }

        rec = ai_advisor.validate_recommendations(payload, self.snapshot())[0]

        self.assertEqual(rec["action"], "review_app")
        self.assertEqual(rec["risk"], "danger")
        self.assertFalse(rec["executable"])
        self.assertIn("Protected", rec["reason"])

    def test_report_only_storage_clean_is_downgraded(self):
        payload = {
            "recommendations": [{
                "action": "clean_storage",
                "target_type": "storage",
                "target_id": "System Data: Application Support",
                "target_label": "System Data: Application Support",
                "priority": "high",
                "risk": "safe",
                "expected_savings_bytes": 5 * 1024**3,
                "reason": "Large",
            }]
        }

        rec = ai_advisor.validate_recommendations(payload, self.snapshot())[0]

        self.assertEqual(rec["action"], "review_app")
        self.assertEqual(rec["risk"], "danger")
        self.assertIn("report-only", rec["reason"])

    def test_closeable_process_still_requires_local_confirmation(self):
        payload = {
            "recommendations": [{
                "action": "close_app",
                "target_type": "process",
                "target_id": "Chrome",
                "target_label": "Chrome",
                "priority": "high",
                "risk": "safe",
                "expected_savings_bytes": 800 * 1024**2,
                "reason": "Unused browser",
            }]
        }

        rec = ai_advisor.validate_recommendations(payload, self.snapshot())[0]

        self.assertEqual(rec["action"], "close_app")
        self.assertEqual(rec["risk"], "review")
        self.assertFalse(rec["executable"])
        self.assertEqual(rec["expected_savings"], "800.0 MB")

    def test_protected_review_app_cannot_be_marked_safe(self):
        payload = {
            "recommendations": [{
                "action": "review_app",
                "target_type": "process",
                "target_id": "VS Code",
                "target_label": "VS Code",
                "priority": "medium",
                "risk": "safe",
                "expected_savings_bytes": 1,
                "reason": "Maybe close",
            }]
        }

        rec = ai_advisor.validate_recommendations(payload, self.snapshot())[0]

        self.assertEqual(rec["action"], "review_app")
        self.assertEqual(rec["risk"], "danger")
        self.assertIn("Protected", rec["reason"])

    def test_fallback_recommends_safe_cache_and_reviewable_process(self):
        result = ai_advisor.fallback_recommendations(self.snapshot(), "test fallback")
        actions = {rec["action"] for rec in result["recommendations"]}
        labels = {rec["target_label"] for rec in result["recommendations"]}

        self.assertIn("review_app", actions)
        self.assertIn("clean_storage", actions)
        self.assertIn("Chrome", labels)
        self.assertIn("App caches", labels)


if __name__ == "__main__":
    unittest.main()
