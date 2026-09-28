from pathlib import Path
import copy
import hashlib
import json
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "tools"))
from license_inventory import inventory, notice_text, validate_supplements


class InventoryTests(unittest.TestCase):
    def supplement(self, root, kind="license-text"):
        commit = "a" * 40
        (root / ".cargo_vcs_info.json").write_text(json.dumps({"git": {"sha1": commit}}))
        text = "Exact upstream text, deliberately without a final newline"
        url = f"https://raw.githubusercontent.com/example/project/{commit}/LICENSE"
        return dict(schema_version=1,
                    sources=[dict(url=url, commit=commit, kind=kind, text=text,
                                  sha256=hashlib.sha256(text.encode()).hexdigest())],
                    packages=[dict(name="example", version="1.2.3", license="MIT",
                                   vcs_commit=commit, sources=[url])])

    def fixture(self, root):
        return dict(resolve=dict(root="renderer", nodes=[dict(id="renderer"), dict(id="example")]),
                    packages=[dict(id="example", name="example", version="1.2.3", license="MIT",
                                   license_file=None, manifest_path=str(root / "Cargo.toml"),
                                   source="registry+https://github.com/rust-lang/crates.io-index",
                                   repository="https://example.org/project")])

    def test_collects_exact_text_and_no_machine_paths(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            text = "Copyright Example authors\nPermission text\n"
            (root / "LICENSE-MIT").write_bytes(text.encode("utf-8"))
            report = inventory(self.fixture(root), "x86_64-apple-darwin", b"lock")
            self.assertEqual(report["missing_texts"], [])
            self.assertEqual(report["packages"][0]["notices"][0]["text"], text)
            self.assertNotIn(directory, str(report))
            self.assertIn(text, notice_text(report))
            self.assertIn("NOT REVIEWED", notice_text(report))

    def test_windows_notice_bytes_are_preserved(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            text = "Copyright Example authors\r\nPermission text\r\n"
            (root / "LICENSE-MIT").write_bytes(text.encode("utf-8"))
            report = inventory(self.fixture(root), "x86_64-pc-windows-msvc", b"lock")
            notice = report["packages"][0]["notices"][0]
            self.assertEqual(notice["text"], text)
            self.assertEqual(notice["sha256"], hashlib.sha256(text.encode()).hexdigest())

    def test_does_not_invent_text_from_spdx(self):
        with tempfile.TemporaryDirectory() as directory:
            report = inventory(self.fixture(Path(directory)), "x86_64-apple-darwin", b"lock")
            self.assertEqual(report["missing_texts"], ["example@1.2.3"])
            self.assertIn("MISSING NOTICE TEXT", notice_text(report))

    def test_license_file_cannot_escape_crate_root(self):
        with tempfile.TemporaryDirectory() as directory:
            metadata = self.fixture(Path(directory))
            metadata["packages"][0]["license_file"] = "../LICENSE"
            with self.assertRaisesRegex(ValueError, "escapes"):
                inventory(metadata, "x86_64-apple-darwin", b"lock")

    def test_filters_unresolved_packages_and_requires_resolution(self):
        with tempfile.TemporaryDirectory() as directory:
            metadata = self.fixture(Path(directory))
            metadata["resolve"]["nodes"] = [dict(id="renderer")]
            self.assertEqual(inventory(metadata, "target", b"lock")["packages"], [])
            metadata["resolve"] = None
            with self.assertRaises(ValueError):
                inventory(metadata, "target", b"lock")

    def test_pinned_supplement_preserves_exact_text_and_provenance(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            supplement = self.supplement(root)
            report = inventory(self.fixture(root), "target", b"lock", supplement)
            self.assertEqual(report["missing_texts"], [])
            notice = report["packages"][0]["notices"][0]
            self.assertEqual(notice["text"], supplement["sources"][0]["text"])
            self.assertIn(notice["source_url"], notice_text(report))
            self.assertNotIn(directory, str(report))
            self.assertIn("NOT REVIEWED", notice_text(report))

    def test_context_only_does_not_fill_missing_license(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            report = inventory(self.fixture(root), "target", b"lock",
                               self.supplement(root, "licensing-context"))
            self.assertEqual(report["missing_texts"], ["example@1.2.3"])
            self.assertIn("CONTEXT ONLY", notice_text(report))

    def test_rejects_tampering_mutable_urls_and_duplicate_sources(self):
        with tempfile.TemporaryDirectory() as directory:
            original = self.supplement(Path(directory))
            for mutation in ("text", "sha256", "url", "kind", "duplicate"):
                with self.subTest(mutation=mutation):
                    changed = copy.deepcopy(original)
                    if mutation == "duplicate":
                        changed["sources"].append(changed["sources"][0])
                    else:
                        changed["sources"][0][mutation] = "tampered"
                    with self.assertRaises(ValueError):
                        validate_supplements(changed)

    def test_package_revision_and_declaration_must_match(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            supplement = self.supplement(root)
            for field in ("license", "vcs_commit"):
                changed = copy.deepcopy(supplement)
                changed["packages"][0][field] = "mismatch"
                with self.assertRaises(ValueError):
                    inventory(self.fixture(root), "target", b"lock", changed)
            (root / ".cargo_vcs_info.json").unlink()
            with self.assertRaisesRegex(ValueError, "cached package"):
                inventory(self.fixture(root), "target", b"lock", supplement)

    def test_supplement_does_not_apply_to_other_version(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            supplement = self.supplement(root)
            metadata = self.fixture(root)
            metadata["packages"][0]["version"] = "1.2.4"
            report = inventory(metadata, "target", b"lock", supplement)
            self.assertEqual(report["missing_texts"], ["example@1.2.4"])

    def test_checked_in_supplement_hashes(self):
        path = Path(__file__).resolve().parents[2] / "licenses/upstream-notices.json"
        self.assertTrue(validate_supplements(json.loads(path.read_text(encoding="utf-8"))))


if __name__ == "__main__":
    unittest.main()
