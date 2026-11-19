#!/usr/bin/env python3
"""Opt-in installed/source PNG + MP4 export check with real FFmpeg decode."""
from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import re
import subprocess
import sys

PROJECT = Path(__file__).resolve().parents[1]


def decode(path: Path, ffmpeg: Path, width: int, height: int) -> bytes:
    result = subprocess.run([str(ffmpeg), "-v", "error", "-i", str(path), "-frames:v", "1",
                             "-f", "rawvideo", "-pix_fmt", "rgba", "pipe:1"],
                            capture_output=True, check=True, timeout=15)
    if len(result.stdout) != width * height * 4:
        raise ValueError(f"unexpected decoded dimensions: {path.name}")
    return result.stdout


def assert_colors(pixels: bytes, width: int, height: int, colors, tolerance: int):
    positions = [(width // 4, height // 4), (width // 4, height * 3 // 4),
                 (width * 3 // 4, height // 4), (width * 3 // 4, height * 3 // 4)]
    for (x, y), color in zip(positions, colors):
        start = (y * width + x) * 4
        actual = tuple(pixels[start:start + 4])
        if any(abs(a - b) > tolerance for a, b in zip(actual, color)):
            raise ValueError(f"orientation/channel mismatch at {(x, y)}: {actual}, expected {color}")


def verify(files: Path, ffmpeg: Path, ffprobe: Path, expected_frames: int) -> dict:
    colors = [(255, 0, 0, 255), (0, 255, 0, 255), (0, 0, 255, 255), (255, 255, 0, 255)]
    png = files / "quadrants.png"
    if png.read_bytes()[:8] != b"\x89PNG\r\n\x1a\n":
        raise ValueError("invalid PNG signature")
    assert_colors(decode(png, ffmpeg, 319, 181), 319, 181, colors, 0)
    assert_colors(decode(files / "single.png", ffmpeg, 320, 180), 320, 180,
                  [(0, 0, 255, 255)] * 4, 0)
    movie = files / "short.mp4"
    result = subprocess.run([str(ffprobe), "-v", "error", "-count_frames", "-select_streams", "v:0",
                             "-show_entries", "stream=codec_name,width,height,r_frame_rate,nb_read_frames:format=duration",
                             "-of", "json", str(movie)], capture_output=True, check=True, timeout=15)
    info = json.loads(result.stdout)
    stream, = info["streams"]
    if (stream["codec_name"], stream["width"], stream["height"], stream["r_frame_rate"]) != ("h264", 320, 180, "30/1"):
        raise ValueError(f"unexpected video format: {stream}")
    frames = int(stream["nb_read_frames"])
    duration = float(info["format"]["duration"])
    if frames != expected_frames or frames < 6 or abs(duration - frames / 30) > 0.04:
        raise ValueError(f"video frame count/duration mismatch: {info}")
    assert_colors(decode(movie, ffmpeg, 320, 180), 320, 180, colors, 12)
    recovered = files / "after-error.png"
    if recovered.exists():
        assert_colors(decode(recovered, ffmpeg, 320, 180), 320, 180, colors, 0)
    return dict(ok=True, after_encoder_failure=recovered.exists(), png_dimensions=[319, 181], single_dimensions=[320, 180],
                video_dimensions=[320, 180], codec="h264", fps=30, frames=frames,
                duration=duration, orientation_and_channels="pass")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--ffmpeg", required=True, type=Path)
    parser.add_argument("--ffprobe", required=True, type=Path)
    parser.add_argument("--renderer", type=Path)
    parser.add_argument("--sclang-config", type=Path)
    parser.add_argument("--output-dir", required=True, type=Path)
    args = parser.parse_args()
    if args.output_dir.exists():
        parser.error("output directory must be fresh")
    for executable in (args.ffmpeg, args.ffprobe):
        if not executable.is_absolute() or not executable.is_file():
            parser.error("FFmpeg/ffprobe must be absolute executable paths")
    output = args.output_dir.resolve()
    output.mkdir(parents=True)
    environment = os.environ.copy()
    environment["SCSHADER_FFMPEG"] = str(args.ffmpeg)
    environment["SCSHADER_EXPORT_TEST_DIR"] = str(output / "files")
    if os.name == "posix" and " " not in sys.executable:
        stalled = output / "stalled-encoder"
        stalled.write_text(f"#!{sys.executable}\nimport time\ntime.sleep(30)\n")
        stalled.chmod(0o700)
        environment["SCSHADER_EXPORT_STALLED_ENCODER"] = str(stalled)
    command = [sys.executable, str(PROJECT / "tools/run_sc_checks.py"), "hydra_export_smoke", "hydra_export_example_smoke",
               "--output-dir", str(output / "sc")]
    if args.renderer:
        command += ["--renderer", str(args.renderer.resolve())]
    if args.sclang_config:
        command += ["--sclang-config", str(args.sclang_config.resolve())]
    subprocess.run(command, cwd=PROJECT, env=environment, check=True, timeout=90)
    log = (output / "sc/hydra_export_smoke.log").read_text()
    match = re.search(r"EXPORT_RECORDED_FRAMES (\d+)", log)
    if not match:
        raise ValueError("missing SC written-frame result")
    report = verify(output / "files", args.ffmpeg, args.ffprobe, int(match[1]))
    example = output / "files/example"
    quadrants, = example.glob("*-quadrants.png")
    single, = example.glob("*-single.png")
    movie, = example.glob("*-quadrants.mp4")
    decode(quadrants,args.ffmpeg,640,360)
    decode(single,args.ffmpeg,320,180)
    info=json.loads(subprocess.check_output([str(args.ffprobe),"-v","error","-count_frames","-show_entries","stream=codec_name,width,height,nb_read_frames","-of","json",str(movie)],timeout=15))
    stream,=info["streams"]
    if (stream["codec_name"],stream["width"],stream["height"])!=("h264",640,360) or int(stream["nb_read_frames"])<6:
        raise ValueError("queued example movie is invalid")
    decode(movie,args.ffmpeg,640,360)
    report["queued_example"]="two valid PNGs and H.264 MP4, in order"
    (output / "result.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(report, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
