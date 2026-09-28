from pathlib import Path
import struct
import sys
import tempfile
from unittest import TestCase

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "tools"))
from package_support import TARGETS, check_binary, check_host


class PackageTests(TestCase):
    def test_native_targets_and_architecture_aliases(self):
        for target, (system, arch, _, _) in TARGETS.items():
            check_host(target, system, "AMD64" if arch == "x64" else "aarch64", target)

    def test_mac_cannot_be_labelled_linux_and_rust_target_must_match(self):
        for system, arch, rust in [("Darwin", "x86_64", "x86_64-apple-darwin"),
                                  ("Linux", "aarch64", "x86_64-unknown-linux-gnu"),
                                  ("Linux", "x86_64", "x86_64-unknown-linux-musl")]:
            with self.assertRaises(ValueError):
                check_host("x86_64-unknown-linux-gnu", system, arch, rust)

    def test_binary_headers_for_all_supported_formats(self):
        with tempfile.TemporaryDirectory() as directory:
            binary = Path(directory) / "renderer with spaces"
            for target, (_, _, kind, cpu) in TARGETS.items():
                data = bytearray(128)
                if kind == "ELF":
                    data[:6] = b"\x7fELF\x02\x01"
                    struct.pack_into("<H", data, 18, cpu)
                elif kind == "Mach-O":
                    data[:4] = b"\xcf\xfa\xed\xfe"
                    struct.pack_into("<I", data, 4, cpu)
                else:
                    data[:2] = b"MZ"
                    struct.pack_into("<I", data, 60, 64)
                    data[64:68] = b"PE\x00\x00"
                    struct.pack_into("<H", data, 68, cpu)
                    data[88:90] = b"\x0b\x02"
                binary.write_bytes(data)
                report = check_binary(binary, target)
                self.assertEqual(report["binary_format"], kind)
                self.assertEqual(len(report["renderer_sha256"]), 64)
                for wrong in TARGETS:
                    if wrong != target:
                        with self.assertRaises(ValueError):
                            check_binary(binary, wrong)

    def test_truncated_or_script_files_are_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            binary = Path(directory) / "renderer"
            for data in [b"", b"MZ", b"#!/bin/sh\nexit 0\n", b"MZ" + b"\0" * 100]:
                binary.write_bytes(data)
                with self.assertRaises(ValueError):
                    check_binary(binary, "x86_64-pc-windows-msvc")
