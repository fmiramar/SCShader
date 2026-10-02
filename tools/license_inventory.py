#!/usr/bin/env python3
"""Audit cached locked dependencies; never synthesize missing license texts.

An inventory is evidence for notice review, not an SPDX compatibility decision.
Build/fetch the target dependencies first; metadata is always locked and offline.
"""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys

def validate_supplements(document: dict) -> dict:
    """Validate every pinned source, including sources unused by this target."""
    if document.get("schema_version") not in (1, 2):
        raise ValueError("unsupported supplemental notice schema")
    sources = {}
    for source in document["sources"]:
        url, commit = source["url"], source["commit"]
        if not re.fullmatch(r"[0-9a-f]{40}", commit) or not re.fullmatch(
                r"https://raw\.githubusercontent\.com/[^/]+/[^/]+/" + commit + r"/[^?#]+", url):
            raise ValueError("supplement source must use an immutable upstream commit URL")
        if url in sources:
            raise ValueError("duplicate supplemental source")
        raw = source["text"].encode("utf-8")
        if not raw or len(raw) > 2 * 1024 * 1024:
            raise ValueError("empty/oversized supplemental notice")
        if hashlib.sha256(raw).hexdigest() != source["sha256"]:
            raise ValueError("supplemental notice hash mismatch")
        if source["kind"] not in ("license-text", "licensing-context"):
            raise ValueError("unknown supplemental notice kind")
        sources[url] = source
    packages = {}
    for package in document["packages"]:
        key = (package["name"], package["version"])
        if key in packages:
            raise ValueError("duplicate supplemental package")
        if not package["license"] or not package["sources"]:
            raise ValueError("supplement requires a declaration and source")
        if len(set(package["sources"])) != len(package["sources"]):
            raise ValueError("duplicate package source")
        notices = []
        reviews = package.get("source_reviews", {})
        if reviews and document["schema_version"] != 2:
            raise ValueError("source reviews require schema 2")
        if set(reviews) - set(package["sources"]):
            raise ValueError("review does not refer to a package source")
        for url in package["sources"]:
            source = sources.get(url)
            review = reviews.get(url)
            if review is not None and (not review.get("reason") or not review.get("evidence")
                    or not all(re.fullmatch(r"https://github\.com/[^/]+/[^/]+/(?:blob|commit)/[0-9a-f]{40}(?:/[^?#]+)?", item)
                               for item in review["evidence"])):
                raise ValueError("source review requires a reason and immutable evidence")
            if source is None or (source["commit"] != package["vcs_commit"] and not review):
                raise ValueError("supplement source/package revision mismatch")
            notices.append(dict(path="upstream/" + url.rsplit("/", 1)[1],
                                source_url=url, sha256=source["sha256"],
                                kind=source["kind"], text=source["text"], source_review=review))
        packages[key] = dict(package, notices=notices)
    return packages


def is_notice_file(path: Path) -> bool:
    # copying.rs is an Objective-C API, not a copyright notice. Likewise do not
    # mistake LICENSES.json, license.py, etc. for the actual license text.
    return (path.suffix.lower() not in (".rs", ".py", ".js", ".ts", ".c", ".h", ".cpp",
                                       ".json", ".toml", ".yml", ".yaml", ".png", ".svg")
            and re.match(r"^(?:LICENSES?|LICENCES?|NOTICES?|COPYING|COPYRIGHT|UNLICENSE)(?:$|[._-])",
                         path.name, re.IGNORECASE) is not None)


def inventory(metadata: dict, target: str, lock_bytes: bytes, supplements: dict | None = None) -> dict:
    supplements = validate_supplements(supplements) if supplements is not None else {}
    if not metadata.get("resolve") or not metadata["resolve"].get("root"):
        raise ValueError("expected one resolved renderer package")
    root_id = metadata["resolve"]["root"]
    resolved = {node["id"] for node in metadata["resolve"]["nodes"]}
    packages = []
    for package in metadata["packages"]:
        if package["id"] not in resolved or package["id"] == root_id:
            continue
        root = Path(package["manifest_path"]).resolve().parent
        files = {path for path in root.rglob("*")
                 if path.is_file() and is_notice_file(path)}
        if package.get("license_file"):
            files.add(root / package["license_file"])
        notices = []
        for path in sorted(files, key=lambda item: item.relative_to(root).as_posix()):
            # Do not follow a crate's symlinks/license-file outside its source root.
            resolved_path = path.resolve()
            if not resolved_path.is_relative_to(root):
                raise ValueError(f"notice escapes package root: {package['name']}")
            raw = resolved_path.read_bytes()
            if not raw or len(raw) > 2 * 1024 * 1024:
                raise ValueError(f"empty/oversized notice: {package['name']}/{path.name}")
            notices.append(dict(path=path.relative_to(root).as_posix(),
                                sha256=hashlib.sha256(raw).hexdigest(),
                                text=raw.decode("utf-8")))
        vcs = root / ".cargo_vcs_info.json"
        origin = json.loads(vcs.read_text()) if vcs.is_file() else {}
        version = package["version"]
        context = []
        supplemental = supplements.get((package["name"], version))
        if supplemental:
            if (supplemental["vcs_commit"] != origin.get("git", {}).get("sha1")
                    or supplemental["license"] != package.get("license")):
                raise ValueError(f"supplement does not match cached package: {package['name']}@{version}")
            for notice in supplemental["notices"]:
                (notices if notice["kind"] == "license-text" else context).append(notice)
            for excerpt in supplemental.get("file_excerpts", []):
                path = root / excerpt["path"]
                if not path.resolve().is_relative_to(root):
                    raise ValueError("excerpt escapes package root")
                raw = path.read_bytes()
                if hashlib.sha256(raw).hexdigest() != excerpt["file_sha256"]:
                    raise ValueError("excerpt source hash mismatch")
                text = "".join(raw.decode("utf-8").splitlines(keepends=True)
                               [excerpt["start_line"] - 1:excerpt["end_line"]])
                if (excerpt["start_line"] < 1 or not text
                        or hashlib.sha256(text.encode()).hexdigest() != excerpt["sha256"]):
                    raise ValueError("excerpt text hash mismatch")
                notices.append(dict(excerpt, text=text))
        packages.append(dict(name=package["name"], version=version,
                             license=package.get("license"), source=package.get("source"),
                             repository=package.get("repository"),
                             vcs_commit=origin.get("git", {}).get("sha1"),
                             vcs_path=origin.get("path_in_vcs"), notices=notices,
                             licensing_context=context))
    packages.sort(key=lambda package: (package["name"], package["version"]))
    missing_texts = [f"{p['name']}@{p['version']}" for p in packages if not p["notices"]]
    missing_declarations = [f"{p['name']}@{p['version']}" for p in packages
                            if not p["license"] and not p["notices"]]
    return dict(schema_version=2, target=target,
                cargo_lock_sha256=hashlib.sha256(lock_bytes).hexdigest(),
                scope="Resolved target graph, conservatively including build/proc-macro dependencies.",
                review_status="unreviewed; file presence is not proof of complete license obligations",
                missing_texts=missing_texts, missing_declarations=missing_declarations,
                packages=packages)


def apply_distribution_review(report: dict, review: dict) -> None:
    """Fail closed when the lockfile, target graph or any notice/context changes."""
    digest = hashlib.sha256(json.dumps(report["packages"], sort_keys=True,
                                      ensure_ascii=False).encode("utf-8")).hexdigest()
    if (review.get("schema_version") != 1
            or report["cargo_lock_sha256"] != review["cargo_lock_sha256"]
            or digest != review["target_notice_sha256"].get(report["target"])
            or report["missing_texts"] or report["missing_declarations"]):
        raise ValueError("distribution notice review is missing/stale; review the new audit first")
    for package in report["packages"]:
        package["selected_license"] = review["license_choices"][package["license"]]
    report["review_status"] = (
        "SCShader notice-assembly review 2026-10-01; see docs/NOTICE_REVIEW.md for "
        "license choices, recovery provenance and limitations. Not legal certification.")
    report["distribution_review"] = True


def notice_text(report: dict) -> str:
    lines = [("SCShader third-party notices" if report.get("distribution_review")
              else "SCShader dependency notice audit — NOT REVIEWED"),
             f"Target: {report['target']}", f"Cargo.lock SHA-256: {report['cargo_lock_sha256']}",
             report["scope"], report["review_status"],
             f"Packages without license text (including pinned supplements): {len(report['missing_texts'])}", ""]
    for package in report["packages"]:
        lines.extend(["=" * 72, f"{package['name']} {package['version']}",
                      f"Declared license: {package['license'] or '(not declared)'}",
                      f"Repository: {package['repository'] or '(not declared)'}", ""])
        if package.get("selected_license"):
            lines.append(f"SCShader license selection: {package['selected_license']}")
        if not package["notices"]:
            lines.append("MISSING NOTICE TEXT — retrieve and review the exact upstream revision.")
        for notice in package["notices"]:
            if notice.get("source_url"):
                lines.append(f"Pinned upstream source: {notice['source_url']}")
            if notice.get("source_review"):
                lines.append(f"SCShader source-recovery decision: {notice['source_review']['reason']}")
                lines.extend(notice["source_review"]["evidence"])
            lines.extend([f"--- {notice['path']} (SHA-256 {notice['sha256']}) ---", notice["text"], ""])
        for notice in package.get("licensing_context", []):
            lines.extend(["CONTEXT ONLY — not a full license text",
                          f"Pinned upstream source: {notice['source_url']}",
                          f"--- {notice['path']} (SHA-256 {notice['sha256']}) ---", notice["text"], ""])
    return "\n".join(lines) + "\n"


def main() -> int:
    project = Path(__file__).resolve().parents[1]
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--target", required=True, help="explicit Rust target triple")
    parser.add_argument("--output-dir", type=Path, required=True, help="new, non-existing audit directory")
    parser.add_argument("--require-texts", action="store_true", help="exit nonzero for missing texts/declarations")
    parser.add_argument("--distribution", action="store_true", help="require the checked-in notice-assembly review")
    parser.add_argument("--supplements", type=Path, default=project / "licenses/upstream-notices.json",
                        help="pinned upstream notice collection (validated offline)")
    args = parser.parse_args()
    if not re.fullmatch(r"[A-Za-z0-9_-]+", args.target):
        parser.error("invalid target triple")
    if args.output_dir.exists():
        parser.error("output directory exists; use a fresh audit directory")
    try:
        result = subprocess.run(["cargo", "metadata", "--manifest-path", str(project / "renderer/Cargo.toml"),
                                 "--locked", "--offline", "--format-version", "1", "--filter-platform", args.target],
                                check=True, capture_output=True, text=True, timeout=120)
        supplements = json.loads(args.supplements.read_text(encoding="utf-8"))
        report = inventory(json.loads(result.stdout), args.target,
                           (project / "renderer/Cargo.lock").read_bytes(), supplements)
        if args.distribution:
            apply_distribution_review(report, json.loads(
                (project / "licenses/review.json").read_text(encoding="utf-8")))
        args.output_dir.mkdir(parents=True)
        (args.output_dir / "inventory.json").write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
        (args.output_dir / "notices-audit.txt").write_text(notice_text(report), encoding="utf-8")
        if args.distribution:
            (args.output_dir / "THIRD_PARTY_NOTICES.txt").write_text(notice_text(report), encoding="utf-8")
        status = "notice-assembly review matched" if args.distribution else "review still required"
        print(f"Audited {len(report['packages'])} packages for {args.target}; "
              f"{len(report['missing_texts'])} without license text; {status}.")
        for package in report["missing_texts"]:
            print(f"MISSING {package}")
        return int(args.require_texts and bool(report["missing_texts"] or report["missing_declarations"]))
    except (OSError, ValueError, KeyError, TypeError, subprocess.SubprocessError) as error:
        print(f"Audit failed: {error}", file=sys.stderr)
        if isinstance(error, subprocess.CalledProcessError):
            print(error.stderr, file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
