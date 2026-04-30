import tempfile
import unittest
from pathlib import Path

import scanner


def write_bytes(path: Path, size: int) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(b"x" * size)


class ScannerTests(unittest.TestCase):
    def test_known_cache_item_is_cleanable(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            home = Path(tmpdir)
            write_bytes(home / "Library/Caches/example/blob.bin", 2048)

            items = scanner.scan_known_items(home=home, min_size=1)

            app_cache = next(i for i in items if i["name"] == "App caches")
            self.assertTrue(app_cache["cleanable"])
            self.assertEqual(app_cache["safety"], "safe-cache")
            self.assertEqual(app_cache["category"], "cache")

    def test_system_data_application_support_is_report_only(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            home = Path(tmpdir)
            write_bytes(home / "Library/Application Support/App/data.bin", 2048)

            items = scanner.scan_system_data_items(home=home, min_size=1)

            app_support = next(i for i in items if i["name"] == "System Data: Application Support")
            self.assertFalse(app_support["cleanable"])
            self.assertEqual(app_support["safety"], "never-default-delete")

    def test_pub_cache_is_report_only_development_store(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            home = Path(tmpdir)
            write_bytes(home / ".pub-cache/hosted/pub.dev/example-1.0.0/lib/example.dart", 2048)

            items = scanner.scan_known_items(home=home, min_size=1)

            pub_cache = next(i for i in items if i["name"] == "Dart pub cache")
            self.assertFalse(pub_cache["cleanable"])
            self.assertEqual(pub_cache["safety"], "never-default-delete")
            self.assertIn("package store", pub_cache["reason"])

    def test_clean_scan_item_refuses_pub_cache_even_if_marked_cleanable(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            pub_cache = Path(tmpdir) / ".pub-cache"
            package_file = pub_cache / "hosted/pub.dev/example-1.0.0/lib/example.dart"
            write_bytes(package_file, 2048)
            stale_item = {
                "name": "Dart pub cache",
                "path": str(pub_cache),
                "cleanable": True,
            }

            cleaned = scanner.clean_scan_item(stale_item)

            self.assertEqual(cleaned, 0)
            self.assertTrue(package_file.exists())

    def test_clean_scan_item_refuses_report_only_item(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            path = Path(tmpdir) / "Library/Application Support/App/data.bin"
            write_bytes(path, 2048)
            item = {
                "name": "System Data: Application Support",
                "path": str(path.parent),
                "cleanable": False,
            }

            cleaned = scanner.clean_scan_item(item)

            self.assertEqual(cleaned, 0)
            self.assertTrue(path.exists())

    def test_clean_cache_root_preserves_protected_subdirectories(self):
        old_home = scanner.HOME
        with tempfile.TemporaryDirectory() as tmpdir:
            try:
                scanner.HOME = Path(tmpdir)
                cache_root = scanner.HOME / "Library/Caches"
                write_bytes(cache_root / "ms-playwright/browser.bin", 2048)
                write_bytes(cache_root / "disposable/blob.bin", 2048)
                item = {
                    "name": "App caches",
                    "path": str(cache_root),
                    "cleanable": True,
                }

                cleaned = scanner.clean_scan_item(item)

                self.assertGreater(cleaned, 0)
                self.assertTrue((cache_root / "ms-playwright/browser.bin").exists())
                self.assertFalse((cache_root / "disposable").exists())
            finally:
                scanner.HOME = old_home

    def test_dir_size_uses_allocated_blocks_for_sparse_files(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            sparse = Path(tmpdir) / "sparse.img"
            with sparse.open("wb") as f:
                f.seek(1024 * 1024 * 1024)
                f.write(b"\0")
            if not getattr(sparse.stat(), "st_blocks", 0):
                self.skipTest("filesystem does not expose allocated blocks")

            size = scanner.dir_size(sparse)

            self.assertLess(size, 10 * 1024 * 1024)


if __name__ == "__main__":
    unittest.main()
