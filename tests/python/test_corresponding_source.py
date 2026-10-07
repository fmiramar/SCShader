from pathlib import Path
import hashlib
import json
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch
import zipfile

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "tools"))
from package_source import DIRECTORIES, FILES, write_entry
from package_corresponding_source import ROOT, create, vendor_sources, verify


class CorrespondingSourceTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory(prefix="scshader-cs-test-")
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.project = self.root / "project"
        for name in DIRECTORIES:
            (self.project / name).mkdir(parents=True, exist_ok=True)
        for name in FILES:
            path = self.project / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("0.0.18\n" if name == "VERSION" else "fixture\n", encoding="utf-8")
        self.binary = self.root / "renderer.exe"
        (self.project / "tools/build.sh").write_text("#!/bin/sh\n", encoding="utf-8")
        self.binary.write_bytes(b"test binary")
        self.output = self.root / "source.zip"

    def vendor(self, project, destination):
        package = destination / "example-1.0"
        package.mkdir(parents=True)
        (package / "source.rs").write_bytes(b"// dependency source\n")
        (package / "LICENSE").write_bytes(b"Exact license\r\n")
        return '[source.vendored-sources]\ndirectory = "vendor"\n'

    def create(self):
        with patch("package_corresponding_source.vendor_sources", side_effect=self.vendor):
            return create(self.project, self.output, self.binary, "x86_64-pc-windows-msvc")

    def test_complete_source_no_external_plan_and_binary_identity(self):
        manifest = self.create()
        self.assertEqual(manifest, verify(self.output))
        self.assertEqual(manifest["renderer_sha256"], hashlib.sha256(b"test binary").hexdigest())
        with zipfile.ZipFile(self.output) as archive:
            self.assertEqual(archive.read(f"{ROOT}/vendor/example-1.0/LICENSE"), b"Exact license\r\n")
            self.assertIn(f"{ROOT}/.cargo/config.toml", archive.namelist())
            self.assertIn(f"{ROOT}/COPYING", archive.namelist())
            self.assertIn(f"{ROOT}/tools/build.sh", archive.namelist())
            self.assertEqual(archive.getinfo(f"{ROOT}/tools/build.sh").external_attr >> 16 & 0o777, 0o755)
            self.assertNotIn("renderer.exe", "\n".join(archive.namelist()))
            self.assertNotIn(str(self.root), archive.read(f"{ROOT}/SOURCE_MANIFEST.json").decode())
        self.assertTrue(self.output.with_name(self.output.name + ".sha256").is_file())

    def test_windows_performance_profile_is_included(self):
        profile = self.project / "tools/wpt/SCShaderScheduler.wprp"
        profile.parent.mkdir(parents=True, exist_ok=True)
        profile.write_bytes(b"<WindowsPerformanceRecorderProfile />\n")
        self.create()
        with zipfile.ZipFile(self.output) as archive:
            self.assertEqual(
                archive.read(f"{ROOT}/tools/wpt/SCShaderScheduler.wprp"),
                b"<WindowsPerformanceRecorderProfile />\n",
            )

    def test_existing_outputs_are_retained(self):
        self.create()
        original = self.output.read_bytes()
        with self.assertRaisesRegex(ValueError, "new .zip"):
            self.create()
        self.assertEqual(original, self.output.read_bytes())

    def test_source_change_during_vendor_fails(self):
        def changing(project, destination):
            config = self.vendor(project, destination)
            (project / "renderer/Cargo.lock").write_text("changed")
            return config
        with patch("package_corresponding_source.vendor_sources", side_effect=changing):
            with self.assertRaisesRegex(ValueError, "changed during"):
                create(self.project, self.output, self.binary, "target")
        self.assertFalse(self.output.with_name(self.output.name + ".sha256").exists())

    def test_vendor_configuration_is_portable_and_offline(self):
        destination = self.root / "vendor"
        config = '[source.crates-io]\nreplace-with = "vendored-sources"\n[source.vendored-sources]\ndirectory = ' + json.dumps(str(destination))
        with patch("package_corresponding_source.subprocess.run", return_value=subprocess.CompletedProcess([], 0, config)) as run:
            result = vendor_sources(self.project, destination)
            self.assertIn('directory = "vendor"', result)
            self.assertNotIn(str(self.root), result)
            self.assertIn("--offline", run.call_args.args[0])
            self.assertIn("--locked", run.call_args.args[0])
        with patch("package_corresponding_source.subprocess.run", return_value=subprocess.CompletedProcess([], 0, config + '\n[source.other]\ndirectory="unexpected"')):
            with self.assertRaisesRegex(ValueError, "unexpected"):
                vendor_sources(self.project, destination)

    def test_tampered_content_and_extra_paths_fail_verification(self):
        self.create()
        for change in ("content", "extra", "traversal"):
            altered = self.root / f"{change}.zip"
            with zipfile.ZipFile(self.output) as source, zipfile.ZipFile(altered, "x") as dest:
                for info in source.infolist():
                    data = source.read(info)
                    if change == "content" and info.filename.endswith("/COPYING"):
                        data = b"changed"
                    dest.writestr(info, data)
                if change != "content":
                    write_entry(dest, f"{ROOT}/" + ("extra.txt" if change == "extra" else "../outside"), b"extra")
            with self.assertRaises(ValueError):
                verify(altered)


if __name__ == "__main__":
    unittest.main()
