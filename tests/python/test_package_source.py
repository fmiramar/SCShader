from pathlib import Path
import hashlib
import os
import sys
import tempfile
from unittest import TestCase
import zipfile

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "tools"))
from package_source import (DIRECTORIES, FILES, PLAN, ROOT, collect_source,
                            create_archive, verify_archive, write_entry)


class SourcePackageTests(TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory(prefix="scshader-source-test-")
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.project = self.root / "SCShader"
        for directory in DIRECTORIES:
            (self.project / directory).mkdir(parents=True, exist_ok=True)
        for name in FILES:
            path = self.project / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("0.0.16\n" if name == "VERSION" else "fixture\n", encoding="utf-8")
        self.plan = self.root / PLAN
        self.plan.write_text("# Original plan\n", encoding="utf-8")
        self.output = self.root / "handoff.zip"

    def test_untracked_source_hidden_workflow_and_permissions_without_git(self):
        shader = self.project / "renderer/shaders/new.wgsl"
        shader.write_text("// current untracked source\n", encoding="utf-8")
        (self.project / ".github/ci.yml").write_text("name: test\n", encoding="utf-8")
        (self.project / "tools/package.sh").write_text("#!/bin/sh\n", encoding="utf-8")
        result = create_archive(self.project, self.plan, self.output, "a" * 40)
        self.assertEqual(verify_archive(self.output), result)
        names = {record["path"]: record for record in result["files"]}
        self.assertIn(f"{ROOT}/SCShader/renderer/shaders/new.wgsl", names)
        self.assertIn(f"{ROOT}/SCShader/.github/ci.yml", names)
        self.assertIn(f"{ROOT}/{PLAN}", names)
        self.assertEqual(names[f"{ROOT}/SCShader/tools/package.sh"]["mode"], "0o755")
        checksum = self.output.with_name(self.output.name + ".sha256").read_text()
        self.assertTrue(checksum.startswith(hashlib.sha256(self.output.read_bytes()).hexdigest()))

    def test_generated_files_and_git_are_not_included(self):
        for name in ["renderer/target/private.rs", "build/evidence.json", ".git/config",
                     "tools/__pycache__/cache.pyc", "docs/example.log", "tools/.DS_Store",
                     "docs/._STATUS.md", "tools/__MACOSX/metadata.txt"]:
            path = self.project / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("excluded", encoding="utf-8")
        result = create_archive(self.project, self.plan, self.output)
        self.assertEqual(len(result["files"]), len(FILES) + 1)

    def test_unreviewed_file_type_rejected(self):
        (self.project / "tools/.env").write_text("secret fixture", encoding="utf-8")
        with self.assertRaisesRegex(ValueError, "Unreviewed"):
            collect_source(self.project, self.plan)

    def test_symlink_rejected(self):
        link = self.project / "docs/linked.md"
        try:
            os.symlink(self.plan, link)
        except OSError:
            self.skipTest("Symlink creation requires privileges on this host")
        with self.assertRaisesRegex(ValueError, "regular source"):
            collect_source(self.project, self.plan)

    def test_reproducible_and_no_overwrite(self):
        create_archive(self.project, self.plan, self.output)
        with self.assertRaisesRegex(ValueError, "already exists"):
            create_archive(self.project, self.plan, self.output)
        second = self.root / "second.zip"
        create_archive(self.project, self.plan, second)
        self.assertEqual(self.output.read_bytes(), second.read_bytes())

    def test_content_mismatch_and_extra_member_rejected(self):
        create_archive(self.project, self.plan, self.output)
        for change in ["content", "extra"]:
            altered = self.root / f"{change}.zip"
            with zipfile.ZipFile(self.output) as source, zipfile.ZipFile(altered, "x") as dest:
                for info in source.infolist():
                    data = source.read(info)
                    if change == "content" and info.filename.endswith("/VERSION"):
                        data = b"changed\n"
                    dest.writestr(info, data)
                if change == "extra":
                    write_entry(dest, f"{ROOT}/extra.md", b"extra")
            with self.assertRaises(ValueError):
                verify_archive(altered)

    def test_unsafe_path_rejected_before_manifest_access(self):
        for index, name in enumerate([f"{ROOT}/../escape", "/absolute", f"{ROOT}/bad\\path",
                                       f"{ROOT}/C:drive", f"{ROOT}//duplicate-separator"]):
            path = self.root / f"unsafe-{index}.zip"
            with zipfile.ZipFile(path, "x") as archive:
                # ZipInfo's constructor normalizes backslashes on Windows.
                # Write the literal malformed member to exercise the verifier.
                info = zipfile.ZipInfo("fixture")
                info.filename = name
                info.external_attr = 0o100644 << 16
                archive.writestr(info, b"fixture")
            with self.assertRaisesRegex(ValueError, "Unsafe"):
                verify_archive(path)

    def test_missing_manifest_is_a_validation_error(self):
        with zipfile.ZipFile(self.output, "x") as archive:
            write_entry(archive, f"{ROOT}/readme.md", b"fixture")
        with self.assertRaisesRegex(ValueError, "manifest"):
            verify_archive(self.output)

    def test_missing_required_source_rejected(self):
        (self.project / "renderer/Cargo.lock").unlink()
        with self.assertRaisesRegex(ValueError, "regular source"):
            create_archive(self.project, self.plan, self.output)
