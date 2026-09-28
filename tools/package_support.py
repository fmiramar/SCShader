#!/usr/bin/env python3
"""Check native target/format and write portable build provenance; no GPU required."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import struct
import subprocess
import sys

TARGETS = {
    "x86_64-apple-darwin": ("Darwin", "x64", "Mach-O", 0x01000007),
    "aarch64-apple-darwin": ("Darwin", "arm64", "Mach-O", 0x0100000C),
    "x86_64-unknown-linux-gnu": ("Linux", "x64", "ELF", 62),
    "aarch64-unknown-linux-gnu": ("Linux", "arm64", "ELF", 183),
    "x86_64-pc-windows-msvc": ("Windows", "x64", "PE", 0x8664),
}


def check_host(target, system, machine, rust_host):
    expected_system, architecture, _, _ = TARGETS[target]
    actual_arch = {"x86_64": "x64", "amd64": "x64", "arm64": "arm64", "aarch64": "arm64"}.get(machine.lower())
    if (system, actual_arch, rust_host) != (expected_system, architecture, target):
        raise ValueError(f"native package requires {target}; host is {system}/{machine}, Rust {rust_host}")


def check_binary(binary: Path, target: str):
    _, _, expected_format, expected_cpu = TARGETS[target]
    with binary.open("rb") as source:
        header = source.read(64)
        kind, cpu = None, None
        if len(header) == 64 and header[:4] == b"\x7fELF" and header[4:6] == b"\x02\x01":
            kind, cpu = "ELF", struct.unpack_from("<H", header, 18)[0]
        elif len(header) == 64 and header[:4] == b"\xcf\xfa\xed\xfe":
            kind, cpu = "Mach-O", struct.unpack_from("<I", header, 4)[0]
        elif len(header) == 64 and header[:2] == b"MZ":
            source.seek(struct.unpack_from("<I", header, 60)[0])
            pe = source.read(26)
            if len(pe) == 26 and pe[:4] == b"PE\x00\x00" and pe[24:26] == b"\x0b\x02":
                kind, cpu = "PE", struct.unpack_from("<H", pe, 4)[0]
        if (kind, cpu) != (expected_format, expected_cpu):
            raise ValueError(f"binary format/architecture does not match {target}: {kind}/{cpu}")
        source.seek(0)
        digest = hashlib.file_digest(source, "sha256").hexdigest()
    return dict(binary_format=kind, renderer_sha256=digest)


def main():
    project = Path(__file__).resolve().parents[1]
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--target", required=True, choices=TARGETS)
    parser.add_argument("--check-host", action="store_true")
    parser.add_argument("--binary", type=Path)
    parser.add_argument("--output-metadata", type=Path)
    args = parser.parse_args()
    if not args.check_host and not args.binary:
        parser.error("specify --check-host or --binary")
    if args.output_metadata and not args.binary:
        parser.error("metadata requires --binary")
    try:
        if args.check_host:
            result = subprocess.run(["rustc", "-vV"], check=True, capture_output=True,
                                    text=True, encoding="utf-8", timeout=30)
            host = next((line.removeprefix("host: ") for line in result.stdout.splitlines()
                         if line.startswith("host: ")), "unknown")
            check_host(args.target, platform.system(), platform.machine(), host)
        if args.binary:
            record = dict(schema_version=1, version=(project / "VERSION").read_text().strip(),
                          target=args.target, architecture=TARGETS[args.target][1],
                          cargo_lock_sha256=hashlib.sha256((project / "renderer/Cargo.lock").read_bytes()).hexdigest(),
                          runtime_validation="not established by packaging",
                          **check_binary(args.binary, args.target))
            if args.output_metadata:
                args.output_metadata.write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")
        print(f"PASS package target/format checks: {args.target}")
        return 0
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        print(f"Package check failed: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
