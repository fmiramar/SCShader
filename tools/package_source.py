#!/usr/bin/env python3
"""Create/verify a portable working-tree source ZIP, without Git or native binaries.

Includes reviewed source roots even when files are untracked. Generated state is
excluded. Does not overwrite an existing archive/checksum or initialize a repo.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import stat
import zipfile

ROOT = "SCShader-handoff"
PLAN = "SC_Shader_Interface_Agentic_Implementation_Plan.md"
MANIFEST = f"{ROOT}/HANDOFF_MANIFEST.json"
FILES = (
    ".gitignore", "AGENTS.md", "START_HERE.md", "README.md", "CHANGELOG.md",
    "LICENSE", "VERSION", "SCShader.quark", "rust-toolchain.toml",
    "renderer/Cargo.toml", "renderer/Cargo.lock",
)
DIRECTORIES = (
    ".github", "Classes", "HelpSource", "docs", "examples", "licenses",
    "protocol", "renderer/src", "renderer/examples", "renderer/shaders",
    "renderer/tests", "tests", "tools",
)
SKIP_NAMES = {".git", ".DS_Store", "__MACOSX", "__pycache__", "target", "build", "stage", "dist"}
SKIP_SUFFIXES = {".pyc", ".log", ".csv", ".zip", ".so", ".dylib", ".dll", ".exe", ".scx"}
SOURCE_SUFFIXES = {
    ".md", ".rs", ".toml", ".lock", ".sc", ".schelp", ".scd", ".wgsl",
    ".glsl", ".frag", ".ppm", ".json", ".py", ".sh", ".ps1", ".m",
    ".yml", ".yaml", ".txt",
}


def collect_source(project: Path, plan: Path) -> dict[str, Path]:
    entries = {}

    def add(path: Path, name: str):
        if path.is_symlink() or not path.is_file():
            raise ValueError(f"Expected a regular source file: {name}")
        entries[name] = path

    for relative in FILES:
        add(project / relative, f"{ROOT}/SCShader/{relative}")
    for relative in DIRECTORIES:
        source = project / relative
        if source.is_symlink() or not source.is_dir():
            raise ValueError(f"Expected a source directory: {relative}")
        for current, directories, files in os.walk(source):
            directories[:] = sorted(name for name in directories if name not in SKIP_NAMES)
            for name in directories:
                if (Path(current) / name).is_symlink():
                    raise ValueError(f"Symlink in source directory: {relative}/{name}")
            for name in sorted(files):
                path = Path(current) / name
                if name in SKIP_NAMES or name.startswith("._") or path.suffix in SKIP_SUFFIXES:
                    continue
                relative_path = path.relative_to(project).as_posix()
                if path.suffix not in SOURCE_SUFFIXES:
                    raise ValueError(f"Unreviewed source file type: {relative_path}")
                add(path, f"{ROOT}/SCShader/{relative_path}")
    add(plan, f"{ROOT}/{PLAN}")
    return dict(sorted(entries.items()))


def write_entry(archive: zipfile.ZipFile, name: str, data: bytes, mode: int = 0o644):
    # Fixed timestamps/permissions make identical source snapshots reproducible.
    info = zipfile.ZipInfo(name, date_time=(1980, 1, 1, 0, 0, 0))
    info.create_system = 3
    info.external_attr = (stat.S_IFREG | mode) << 16
    info.compress_type = zipfile.ZIP_DEFLATED
    archive.writestr(info, data)


def create_archive(project: Path, plan: Path, output: Path, base_commit: str | None = None):
    checksum = output.with_name(output.name + ".sha256")
    if output.suffix != ".zip":
        raise ValueError("Output must end in .zip")
    if output.exists() or checksum.exists():
        raise ValueError("Archive/checksum already exists; choose a new output name")
    if base_commit and not re.fullmatch(r"[0-9a-f]{40}", base_commit):
        raise ValueError("Base commit must be a full SHA-1 commit identifier")
    entries = collect_source(project, plan)
    version = (project / "VERSION").read_text(encoding="utf-8").strip()
    manifest = {
        "schema_version": 1,
        "kind": "source-development-handoff",
        "version": version,
        "base_commit": base_commit,
        "snapshot": "working tree, including uncommitted/untracked source; not a Git history backup",
        "native_runtime_qualification": False,
        "files": [],
    }
    output.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(output, "x") as archive:
        for name, path in entries.items():
            data = path.read_bytes()
            # Suffix-based normalization is portable across Windows ZIP extraction.
            mode = 0o755 if path.suffix == ".sh" else 0o644
            write_entry(archive, name, data, mode)
            manifest["files"].append({
                "path": name, "size": len(data), "mode": oct(mode),
                "sha256": hashlib.sha256(data).hexdigest(),
            })
        write_entry(archive, MANIFEST, (json.dumps(manifest, indent=2) + "\n").encode("utf-8"))
    verify_archive(output)
    digest = hashlib.sha256(output.read_bytes()).hexdigest()
    with checksum.open("x", encoding="utf-8", newline="\n") as stream:
        stream.write(f"{digest}  {output.name}\n")
    return manifest


def verify_archive(path: Path):
    with zipfile.ZipFile(path) as archive:
        names = archive.namelist()
        if len(set(names)) != len(names):
            raise ValueError("Duplicate ZIP members")
        for info in archive.infolist():
            # zipfile normalizes backslashes on Windows; validate the wire name
            # before that normalization (and before NUL truncation).
            name = info.orig_filename
            parts = PurePosixPath(name).parts
            if (not parts or parts[0] != ROOT or ".." in parts or "\\" in name
                    or ":" in name or "\0" in name or name != PurePosixPath(name).as_posix()):
                raise ValueError("Unsafe ZIP member path")
            if not stat.S_ISREG(info.external_attr >> 16):
                raise ValueError("Non-regular ZIP member")
        if archive.testzip() is not None:
            raise ValueError("ZIP CRC check failed")
        if MANIFEST not in names:
            raise ValueError("Missing handoff manifest")
        manifest = json.loads(archive.read(MANIFEST))
        if manifest["schema_version"] != 1 or manifest["kind"] != "source-development-handoff":
            raise ValueError("Unsupported handoff manifest")
        records = manifest["files"]
        expected = {record["path"] for record in records}
        if len(expected) != len(records) or MANIFEST in expected or set(names) != expected | {MANIFEST}:
            raise ValueError("ZIP inventory does not match manifest")
        for record in records:
            data = archive.read(record["path"])
            mode = stat.S_IMODE(archive.getinfo(record["path"]).external_attr >> 16)
            if (len(data) != record["size"] or hashlib.sha256(data).hexdigest() != record["sha256"]
                    or oct(mode) != record["mode"]):
                raise ValueError(f"Content/permission mismatch: {record['path']}")
        return manifest


def main():
    project = Path(__file__).resolve().parents[1]
    parser = argparse.ArgumentParser(description=__doc__)
    action = parser.add_mutually_exclusive_group(required=True)
    action.add_argument("--output", type=Path)
    action.add_argument("--verify", type=Path)
    parser.add_argument("--plan", type=Path, default=project.parent / PLAN)
    parser.add_argument("--base-commit", help="optional originating commit, not a claim of a clean tree")
    args = parser.parse_args()
    try:
        if args.verify:
            manifest = verify_archive(args.verify)
            print(f"Verified {len(manifest['files'])} source files; SCShader {manifest['version']}")
        else:
            manifest = create_archive(project, args.plan, args.output, args.base_commit)
            print(f"Created {args.output} ({len(manifest['files'])} source files) and SHA-256 checksum")
    except (ValueError, OSError, KeyError, zipfile.BadZipFile) as error:
        parser.exit(1, f"Source handoff failed: {error}\n")


if __name__ == "__main__":
    main()
