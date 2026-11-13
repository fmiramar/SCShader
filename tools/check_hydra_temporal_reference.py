#!/usr/bin/env python3
"""Trace pinned Hydra output callbacks on macOS JavaScriptCore (no browser GPU).

The upstream source is read from a local Git object and never vendored here.
regl's package is fetched from npm and verified against Hydra's pinned lockfile.
The small recorder models regl's source-verified framebuffer-before-uniform order;
it does not execute regl/WebGL and is not browser pixel parity evidence.
"""
import argparse
import base64
import hashlib
import io
import json
from pathlib import Path
import subprocess
import tarfile
import urllib.request

PIN = "9d29a9f4fd8f9081b9759943f38db36f05b9a88f"
TRACE = r"""
function run() {
    function check(ok, why) { if (!ok) throw Error(why); }
    // Recorder only: execute dynamic callbacks in regl's verified draw order.
    function regl(config) {
        return function(props) {
            const destination = config.framebuffer();
            const input = config.uniforms.tex();
            check(input !== destination, "src aliased attachment");
            check(config.uniforms.prevBuffer() === destination,
                  "upstream prev alias changed; revisit native exception");
            destination.value = input.value + props.step;
        };
    }
    regl.buffer = x => x; regl.prop = x => x; regl.texture = x => x;
    regl.framebuffer = x => ({value: 0});
    function fresh() {
        return [0,1,2,3].map(i => new Output({regl: regl, precision: "highp", width: 8, height: 8}));
    }
    const o = fresh();
    const sources = [0, 0, 3, 3];
    o.forEach((out,i) => out.render([{frag: "recorder", uniforms: {tex: () => o[sources[i]].getTexture()}}]));
    const tags = [];
    for (let n=1; n<=8; n++) {
        o.forEach((out,i) => out.tick({step: [8,0,0,16][i]}));
        const row = o.map(out => out.getCurrent().value);
        const expected = [n*8, (n-1)*8, Math.max(0,n-2)*16, n*16];
        check(JSON.stringify(row) === JSON.stringify(expected), "frame tags " + n);
        tags.push(row);
    }
    const cycle = fresh();
    cycle[0].render([{frag: "recorder", uniforms: {tex: () => cycle[1].getTexture()}}]);
    cycle[1].render([{frag: "recorder", uniforms: {tex: () => cycle[0].getTexture()}}]);
    const a=[0,0], b=[0,0], cycles=[];
    for (let n=2; n<10; n++) {
        a.push(b[n-2]+8); b.push(a[n-1]+16);
        cycle[0].tick({step:8}); cycle[1].tick({step:16});
        const row = cycle.slice(0,2).map(out => out.getCurrent().value);
        check(JSON.stringify(row) === JSON.stringify([a[n],b[n]]), "cycle " + n);
        check(cycle[2].getTexture().value === 0, "unassigned");
        cycles.push(row);
    }
    return JSON.stringify({frame_tags: tags, cycles: cycles, upstream_prev_alias: true});
}
"""


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--reference", type=Path, default=Path("build/hydra-reference"))
    parser.add_argument("--output-dir", type=Path, required=True)
    args = parser.parse_args()
    args.output_dir.mkdir(parents=True, exist_ok=False)

    def source(path):
        return subprocess.check_output(["git", "--no-replace-objects", "-C", str(args.reference), "show", f"{PIN}:{path}"], text=True)

    lock = json.loads(source("package-lock.json"))
    regl = lock["packages"]["node_modules/regl"]
    assert regl["version"] == "1.7.0"
    with urllib.request.urlopen(regl["resolved"], timeout=30) as response:
        archive = response.read()
    algorithm, expected = regl["integrity"].split("-", 1)
    assert algorithm == "sha512"
    assert base64.b64encode(hashlib.sha512(archive).digest()).decode() == expected
    with tarfile.open(fileobj=io.BytesIO(archive), mode="r:gz") as package:
        core = package.extractfile("package/lib/core.js").read().decode()
    draw = core[core.index("function emitDrawProc"):core.index("function emitDrawProc") + 1400]
    assert draw.index("emitPollFramebuffer") < draw.index("emitDrawBody")
    body = core[core.index("function emitDrawBody"):core.index("function emitDrawProc")]
    assert "emitUniforms" in body
    synth = source("src/hydra-synth.js")
    assert synth.index("this.o[i].tick(") < synth.index("tex0: this.o[0].getCurrent()", synth.index("this.o[i].tick("))
    assert "typedArg.value = () => (x.getTexture())" in source("src/format-arguments.js")
    output = source("src/output.js")
    script = output.replace("export default Output", "") + TRACE
    result = subprocess.run(["/usr/bin/osascript", "-l", "JavaScript", "-"], input=script,
                            text=True, capture_output=True, timeout=20, check=True)
    record = {"pin": PIN, "regl_integrity": regl["integrity"],
              "output_sha256": hashlib.sha256(output.encode()).hexdigest(),
              "scope": "source-verified callback recorder; no regl/WebGL execution",
              "trace": json.loads(result.stdout)}
    (args.output_dir / "trace.json").write_text(json.dumps(record, indent=2) + "\n")
    print(json.dumps(record, indent=2))
    print("SCSHADER_HYDRA_TEMPORAL_REFERENCE_OK")


if __name__ == "__main__":
    main()
