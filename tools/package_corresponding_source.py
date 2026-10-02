#!/usr/bin/env python3
"""Bundle the current project and all locked Cargo sources beside a binary ZIP.

No credentials, Git history, toolchain, SDK, or local build outputs are copied.
The output is a source snapshot, not a claim of bit-for-bit reproducible builds.
"""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile
import tomllib
import zipfile

from package_source import ROOT as HANDOFF_ROOT, collect_source, verify_archive, write_entry

ROOT = "SCShader-source"
KIND = "corresponding-source"
MANIFEST = "SOURCE_MANIFEST.json"


def sha256(path: Path) -> str:
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def vendor_sources(project: Path, destination: Path) -> str:
    result = subprocess.run(["cargo", "vendor", "--locked", "--offline", "--versioned-dirs",
                             "--manifest-path", str(project / "renderer/Cargo.toml"), str(destination)],
                            cwd=project, check=True, capture_output=True, text=True)
    # Cargo emits the absolute temporary path. Store only a portable relative one.
    config = tomllib.loads(result.stdout)
    if (set(config) != {"source"} or set(config["source"]) != {"crates-io", "vendored-sources"}
            or config["source"]["crates-io"] != {"replace-with": "vendored-sources"}
            or Path(config["source"]["vendored-sources"]["directory"]).resolve() != destination.resolve()):
        raise ValueError("unexpected Cargo vendor configuration; review new dependency sources")
    return ('[source.crates-io]\nreplace-with = "vendored-sources"\n\n'
            '[source.vendored-sources]\ndirectory = "vendor"\n')


def verify(path: Path) -> dict:
    return verify_archive(path, ROOT, KIND, MANIFEST)


def create(project: Path, output: Path, binary: Path, target: str) -> dict:
    checksum = output.with_name(output.name + ".sha256")
    if output.suffix != ".zip" or output.exists() or checksum.exists():
        raise ValueError("choose a new .zip archive/checksum path")
    entries = {name.removeprefix(f"{HANDOFF_ROOT}/SCShader/"): path
               for name, path in collect_source(project, None).items()}
    # Snapshot before vendoring; detect concurrent source changes before success.
    original = {name: sha256(path) for name, path in entries.items()}
    manifest = dict(schema_version=1, kind=KIND,
                    version=(project / "VERSION").read_text().strip(), target=target,
                    renderer_sha256=sha256(binary), cargo_lock_sha256=original["renderer/Cargo.lock"],
                    snapshot="working-tree source used by the packaging invocation; no Git history",
                    files=[])
    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="scshader-vendor-") as temporary:
        vendor = Path(temporary) / "vendor"
        config = vendor_sources(project, vendor)
        for path in sorted(vendor.rglob("*")):
            if path.is_symlink():
                raise ValueError("symlink in vendored sources")
            if path.is_file():
                entries["vendor/" + path.relative_to(vendor).as_posix()] = path
        with zipfile.ZipFile(output, "x") as archive:
            def add(name, data, mode=0o644):
                member = f"{ROOT}/{name}"
                write_entry(archive, member, data, mode)
                manifest["files"].append(dict(path=member, size=len(data), mode=oct(mode),
                                              sha256=hashlib.sha256(data).hexdigest()))

            for name, path in sorted(entries.items()):
                data = path.read_bytes()
                if name in original and hashlib.sha256(data).hexdigest() != original[name]:
                    raise ValueError("project changed during source packaging")
                mode = 0o755 if path.suffix == ".sh" or path.stat().st_mode & 0o111 else 0o644
                add(name, data, mode)
            add(".cargo/config.toml", config.encode("utf-8"))
            add("SOURCE_README.txt", (
                "SCShader matching source\n\n"
                "This archive accompanies the binary identified in SOURCE_MANIFEST.json.\n"
                "It includes the project, build scripts, Cargo.lock, and all locked Cargo\n"
                "dependency sources (including other targets), with original license files.\n"
                "See COPYING, LICENSE, docs/NOTICE_REVIEW.md and docs/RELEASE.md.\n\n"
                "Install the Rust version in rust-toolchain.toml and the native compiler/SDK\n"
                "documented in docs/PLATFORMS.md. These external system tools are not bundled.\n"
                "From THIS directory (not renderer/), build without a crates.io download:\n"
                f"  cargo build --manifest-path renderer/Cargo.toml --release --locked --offline --target {target}\n"
                "Rustup/toolchain/target installation may itself require a network connection.\n"
                "The source is portable; this does not assert byte-identical executables or\n"
                "runtime qualification on another machine. Do not discard the vendor folder.\n"
            ).encode("utf-8"))
            write_entry(archive, f"{ROOT}/{MANIFEST}",
                        (json.dumps(manifest, indent=2) + "\n").encode("utf-8"))
        for name, digest in original.items():
            if sha256(entries[name]) != digest:
                raise ValueError("project changed during source packaging")
    verify(output)
    with checksum.open("x", encoding="utf-8", newline="\n") as stream:
        stream.write(f"{sha256(output)}  {output.name}\n")
    return manifest


def attach(project: Path, extension: Path, output: Path, binary: Path, target: str) -> None:
    # The Rust standard library is statically linked but outside Cargo.lock.
    sysroot = Path(subprocess.check_output(["rustc", "--print", "sysroot"], cwd=project, text=True).strip())
    rust_version = subprocess.check_output(["rustc", "--version"], cwd=project, text=True).strip()
    pinned = tomllib.loads((project / "rust-toolchain.toml").read_text())["toolchain"]["channel"]
    if rust_version.split()[1] != pinned:
        raise ValueError("source packaging requires the pinned Rust toolchain")
    notice_dir = extension / "dependency-audit"
    rust_docs = sysroot / "share/doc/rust"
    rust_notices = [rust_docs / "COPYRIGHT-library.html", *sorted((rust_docs / "licenses").glob("*.txt"))]
    if len(rust_notices) < 3:
        raise ValueError("Rust standard-library license texts are missing")
    for source in rust_notices:
        destination = notice_dir / "rust" / source.relative_to(rust_docs)
        destination.parent.mkdir(parents=True, exist_ok=True)
        with destination.open("xb") as stream:
            stream.write(source.read_bytes())
    manifest = create(project, output, binary, target)
    pointer = dict(schema_version=1, archive=output.name, sha256=sha256(output),
                   renderer_sha256=manifest["renderer_sha256"],
                   cargo_lock_sha256=manifest["cargo_lock_sha256"], rustc=rust_version)
    with (extension / "corresponding-source.json").open("x", encoding="utf-8", newline="\n") as stream:
        stream.write(json.dumps(pointer, indent=2) + "\n")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--verify", type=Path)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--extension", type=Path)
    parser.add_argument("--binary", type=Path)
    parser.add_argument("--target")
    args = parser.parse_args()
    try:
        if args.verify:
            manifest = verify(args.verify)
            print(f"Verified {len(manifest['files'])} corresponding-source files")
        else:
            if not all((args.output, args.extension, args.binary, args.target)):
                parser.error("building requires --output, --extension, --binary and --target")
            attach(Path(__file__).resolve().parents[1], args.extension, args.output, args.binary, args.target)
            print(f"Created corresponding source {args.output}")
    except (OSError, ValueError, KeyError, subprocess.SubprocessError, zipfile.BadZipFile) as error:
        parser.exit(1, f"Corresponding source failed: {error}\n")


if __name__ == "__main__":
    main()
