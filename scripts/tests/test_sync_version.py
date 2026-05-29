import importlib.util
import plistlib
import tempfile
import unittest
from pathlib import Path


SCRIPT_PATH = Path(__file__).resolve().parents[1] / "sync-version.py"
spec = importlib.util.spec_from_file_location("sync_version", SCRIPT_PATH)
sync_version = importlib.util.module_from_spec(spec)
spec.loader.exec_module(sync_version)


class SyncVersionTests(unittest.TestCase):
    def write_fixture(self, directory: Path, cargo_version: str, plist_version: str):
        cargo = directory / "Cargo.toml"
        plist = directory / "Info.plist"
        cargo.write_text(f'[package]\nname = "shuttle-rs"\nversion = "{cargo_version}"\n', encoding="utf-8")
        with plist.open("wb") as plist_file:
            plistlib.dump(
                {
                    "CFBundleName": "Shuttle",
                    "CFBundleShortVersionString": plist_version,
                    "CFBundleVersion": plist_version,
                },
                plist_file,
                sort_keys=False,
            )
        return cargo, plist

    def test_sync_updates_plist_versions(self):
        with tempfile.TemporaryDirectory() as tmp:
            cargo, plist = self.write_fixture(Path(tmp), "1.2.3", "1.0.0")

            exit_code = sync_version.main(["--cargo-toml", str(cargo), "--plist", str(plist)])

            self.assertEqual(exit_code, 0)
            with plist.open("rb") as plist_file:
                values = plistlib.load(plist_file)
            self.assertEqual(values["CFBundleShortVersionString"], "1.2.3")
            self.assertEqual(values["CFBundleVersion"], "1.2.3")

    def test_check_fails_when_versions_differ(self):
        with tempfile.TemporaryDirectory() as tmp:
            cargo, plist = self.write_fixture(Path(tmp), "1.2.3", "1.0.0")

            exit_code = sync_version.main(["--check", "--cargo-toml", str(cargo), "--plist", str(plist)])

            self.assertEqual(exit_code, 1)

    def test_check_passes_when_versions_match(self):
        with tempfile.TemporaryDirectory() as tmp:
            cargo, plist = self.write_fixture(Path(tmp), "1.2.3", "1.2.3")

            exit_code = sync_version.main(["--check", "--cargo-toml", str(cargo), "--plist", str(plist)])

            self.assertEqual(exit_code, 0)


if __name__ == "__main__":
    unittest.main()
