#!/usr/bin/env python3
"""Check the pinned Hydra compatibility inventory without starting any runtime."""
from __future__ import annotations

import argparse
from collections import Counter
import json
from pathlib import Path
import re
import subprocess

PROJECT = Path(__file__).resolve().parents[1]
REFERENCE = "9d29a9f4fd8f9081b9759943f38db36f05b9a88f"
STATUSES = ("SUPPORTED", "NATIVE ALTERNATIVE", "ADAPTER REQUIRED", "UNSUPPORTED")
CURVES = ("linear", "easeInQuad", "easeOutQuad", "easeInOutQuad", "easeInCubic",
          "easeOutCubic", "easeInOutCubic", "easeInQuart", "easeOutQuart", "easeInOutQuart",
          "easeInQuint", "easeOutQuint", "easeInOutQuint", "sin")
RUNTIME = (
    "s0, s1, s2, s3", "o0, o1, o2, o3", "out(output=o0)", "render(output), render()", "hush()",
    "setFunction(object)", "time=0, speed=1, bpm=30", "fps, stats.fps", "tick(dt), autoLoop",
    "update(dt), afterUpdate(dt)", "setResolution(width,height), width, height",
    "init({src,dynamic}), dynamic", 'initImage(url="", params)', 'initVideo(url="", params)',
    "initCam(index,params)", "initScreen(index=0,params)", "initStream(streamName,params), pb",
    "initCanvas(width=1000,height=1000)", "source.clear()",
    "screencap(), getScreenImage(callback), canvasToImage(callback)",
    "vidRecorder.start(), vidRecorder.stop()", "a.fft, a.vol, a.bins, a.settings",
    "a.setBins(numBins=4)", "a.setCutoff, a.setSmooth, a.setScale", "a.setMax(max), a.max=15",
    "a.beat, a.onBeat, a.detectBeat", "a.show(), a.hide()", "a0…aN(scale=1,offset=0)",
    "mouse.x/y/buttons/mods/enabled", 'loadScript(url=""), eval(code), makeGlobal, canvas',
    "numSources=4, numOutputs=4", "precision, extendTransforms, debug, detectAudio, enableStreamCapture",
)
DYNAMIC = ("array.fast(speed=1)", "array.smooth(smooth=1)", 'array.ease(ease="linear")',
           "array.offset(offset=0.5)", "array.fit(low=0,high=1)", "function(props)") + tuple(f"ease({c})" for c in CURVES)
EXTRA = (
    "createSource(i)", "captureStream", "expression.glsl(), expression.compile(transforms)",
    "expression.addTransform(object)", "source.resize(), source.tick(), source.getTexture()",
    "output.render(), output.tick(), output.resize(), output.getTexture(), output.getCurrent()",
    "audio.tick(), audio.draw()", "synth, s, o, generator, regl, sandbox", "function-valued easing",
)
OPTIONS = {"pb", "width", "height", "numSources", "numOutputs", "makeGlobal", "autoLoop",
           "detectAudio", "enableStreamCapture", "canvas", "precision", "extendTransforms", "debug"}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def table_rows(text):
    section, result = "", {}
    for line in text.splitlines():
        if line.startswith("## "):
            section = line[3:]
        if line.startswith("| `"):
            cells = [c.strip() for c in line.split("|")[1:-1]]
            key = cells[0].strip("`")
            rows = result.setdefault(section, {})
            require(key not in rows, f"duplicate API row: {key}")
            rows[key] = cells
    return result


def local_links(path):
    count = 0
    for target in re.findall(r"\]\(([^)]+)\)", path.read_text(encoding="utf-8")):
        if target.startswith(("https://", "http://", "#")):
            continue
        require((path.parent / target.split("#")[0]).is_file(), f"missing evidence link: {path.name}: {target}")
        count += 1
    return count


def check(matrix, reference=None):
    text = matrix.read_text(encoding="utf-8")
    require(REFERENCE in text, "matrix lost its pinned reference")
    tables = table_rows(text)
    registry = (PROJECT / "renderer/src/hydra/registry.rs").read_text(encoding="utf-8")
    categories = ("src", "coord", "color", "combine", "combineCoord")
    native = {name: categories[int(kind)] for name, kind in re.findall(r'name:\s*"(\w+)",\s*category:\s*(\d)', registry)}
    require(len(native) == 52, "native registry changed; review the pinned inventory")
    expected = {"Shader operations": set(native), "Runtime and service APIs": set(RUNTIME),
                "Arrays and dynamic parameters": set(DYNAMIC), "Additional inspected source surfaces": set(EXTRA)}
    counts = Counter()
    for section, keys in expected.items():
        rows = tables.get(section, {})
        require(set(rows) == keys, f"API coverage mismatch in {section}: missing={sorted(keys-set(rows))}, extra={sorted(set(rows)-keys)}")
        for name, cells in rows.items():
            status = cells[-1].removeprefix("A / ").split(":", 1)[0]
            require(status in STATUSES, f"unclassified/placeholder status for {name}: {status}")
            require(re.search(r"\[[^\]]+\]\([^)]+\)", cells[-1]), f"API lacks an evidence link: {name}")
            if section == "Shader operations":
                require(cells[1] == native[name], f"native category differs for {name}")
            counts[status] += 1
    compatibility = PROJECT / "docs/hydra/COMPATIBILITY.md"
    require(compatibility.is_file(), "missing compatibility deliverable")
    links = local_links(matrix) + local_links(compatibility)
    result = dict(ok=True, pinned_hydra_commit=REFERENCE, rows=sum(counts.values()),
                  sections={section: len(keys) for section, keys in expected.items()},
                  statuses=dict(sorted(counts.items())), local_evidence_links=links,
                  pinned_source_checked=False, browser_gpu_parity="NOT RUN", runtime_changed=False)
    if reference:
        commit = subprocess.check_output(["git", "-C", str(reference), "rev-parse", "HEAD"], text=True, encoding="utf-8").strip()
        require(commit == REFERENCE, "reference checkout is not the pinned commit")
        files = ["src/glsl/glsl-functions.js", "src/hydra-synth.js", "src/hydra-source.js",
                 "src/glsl-source.js", "src/lib/audio.js", "src/lib/easing-functions.js", "package.json"]
        # Read the committed objects, so uncommitted reference edits cannot change evidence.
        def source(path):
            return subprocess.check_output(["git", "-C", str(reference), "show", f"{REFERENCE}:{path}"], text=True, encoding="utf-8")
        sources = {path: source(path) for path in files}
        declarations = sources[files[0]].split("export default", 1)[1]
        upstream = dict(re.findall(r"\bname:\s*['\"](\w+)['\"]\s*,\s*type:\s*['\"](src|coord|color|combine|combineCoord)['\"]", declarations))
        require(upstream == native, "pinned/native shader names or categories differ")
        curves = set(re.findall(r"^  (\w+):\s*function", sources["src/lib/easing-functions.js"], re.M))
        require(curves == set(CURVES), "pinned easing inventory changed")
        constructor = sources["src/hydra-synth.js"].split("constructor ({", 1)[1].split("} = {})", 1)[0]
        options = set(re.findall(r"^\s*(\w+)\s*(?:=|,)", constructor, re.M))
        require(options == OPTIONS, f"constructor inventory mismatch: {options ^ OPTIONS}")
        option_table = compatibility.read_text(encoding="utf-8").split("### Constructor options are not interchangeable", 1)[1].split("## Tier C", 1)[0]
        documented = set()
        for line in option_table.splitlines():
            if line.startswith("| ") and any(f"| {status} |" in line for status in STATUSES):
                documented.update(n.strip() for n in line.split("|")[1].split(","))
        require(documented == OPTIONS, "constructor options not individually classified")
        method_sets = {
            "src/hydra-synth.js": {"eval", "getScreenImage", "hush", "loadScript", "setResolution", "canvasToImage", "createSource", "tick"},
            "src/hydra-source.js": {"init", "initCam", "initVideo", "initImage", "initStream", "initScreen", "initCanvas", "resize", "clear", "tick", "getTexture"},
            "src/lib/audio.js": {"detectBeat", "tick", "setCutoff", "setSmooth", "setBins", "setScale", "setMax", "hide", "show", "draw"},
        }
        for path, methods in method_sets.items():
            found = {n for n in re.findall(r"^  ([A-Za-z]\w*)\s*\(", sources[path], re.M) if n != "constructor"}
            require(found == methods, f"public source surface changed: {path}: {found ^ methods}")
        expr = set(re.findall(r"GlslSource\.prototype\.(\w+)\s*=", sources["src/glsl-source.js"]))
        require(expr == {"addTransform", "out", "glsl", "compile"}, "expression helpers changed")
        require("this.pb.initSource(streamName)" in sources["src/hydra-source.js"]
                and "got video" in sources["src/hydra-source.js"], "patch-bay delegation changed")
        require(set(json.loads(sources["package.json"])["dependencies"]) == {"meyda", "raf-loop", "regl"}, "review changed upstream runtime dependencies")
        result.update(pinned_source_checked=True, constructor_options=len(options), source_surfaces="matched")
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--matrix", type=Path, default=PROJECT / "docs/hydra/HYDRA_API_MATRIX.md")
    parser.add_argument("--reference", type=Path, help="optional local pinned Hydra checkout; no fetch/JS execution")
    args = parser.parse_args()
    print(json.dumps(check(args.matrix.resolve(), args.reference.resolve() if args.reference else None), indent=2))


if __name__ == "__main__":
    main()
