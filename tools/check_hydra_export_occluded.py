#!/usr/bin/env python3
"""macOS: export PNG/video while an owned renderer is natively minimized."""
import argparse
import hashlib
import json
from pathlib import Path
import struct
import subprocess
import time
from check_hydra_export import assert_colors, decode
from macos_window import Accessibility, OwnedWindow, WindowUnavailable
from osc_client import ManagedRenderer

BASE = "/scshader/v1/hydra/"


def source(name, values):
    arguments = b"".join(b"\0" + struct.pack(">f", v) for v in values)
    return (b"HYIR\0\1\0\0" + struct.pack(">III", 1, 0, 1)
            + struct.pack(">IBBH", 1, 0, len(values), len(name)) + name.encode()
            + struct.pack(">I", 0) + arguments)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--renderer", required=True, type=Path)
    parser.add_argument("--ffmpeg", required=True, type=Path)
    parser.add_argument("--ffprobe", required=True, type=Path)
    parser.add_argument("--output-dir", required=True, type=Path)
    args = parser.parse_args()
    if args.output_dir.exists():
        parser.error("choose a fresh output directory")
    output = args.output_dir.resolve()
    output.mkdir(parents=True)
    report = dict(ok=False, renderer_sha256=hashlib.sha256(args.renderer.read_bytes()).hexdigest())
    try:
        api = Accessibility()
        with ManagedRenderer(args.renderer, 57221, output / "renderer.log", extra_args=["--backend", "metal"]) as owner:
            native = OwnedWindow(owner.process, api)
            try:
                deadline = time.monotonic() + 5
                while not native.discover():
                    if time.monotonic() > deadline:
                        raise TimeoutError("owned native window unavailable")
                    time.sleep(0.05)
                client = owner.client
                port = client.socket.getsockname()[1]
                client.send(BASE + "capabilities", "iii", (1, 1, port))
                epoch = client.expect(BASE + "capabilities.reply")[1]
                client.send(BASE + "open", "issii", (2, epoch, "export-occluded", 1, port))
                session = client.expect(BASE + "opened")[2]
                sequence = 2
                revisions = [0] * 4

                def request(op, tags="", values=(), reply="ack"):
                    nonlocal sequence
                    sequence += 1
                    client.send(BASE + op, "sii" + tags, (epoch, session, sequence) + values)
                    return client.expect(BASE + reply)

                def commit(slot, name, values):
                    nonlocal sequence
                    sequence += 1
                    data = source(name, values)
                    prefix = (epoch, session, sequence)
                    client.send(BASE + "begin", "siiiii", prefix + (slot, revisions[slot], len(data)))
                    client.expect(BASE + "ack")
                    client.send(BASE + "chunk", "siiib", prefix + (0, data))
                    client.expect(BASE + "ack")
                    client.send(BASE + "commit", "sii", prefix)
                    client.expect(BASE + "committed")
                    revisions[slot] += 1

                def finished():
                    deadline = time.monotonic() + 13
                    while time.monotonic() < deadline:
                        status = request("export-status", reply="export-status.reply")
                        if status[3] in ("finished", "error"):
                            report["last_status"] = status[3:]
                            if status[3] != "finished":
                                raise RuntimeError(str(status[3:]))
                            return status
                        time.sleep(0.03)
                    raise TimeoutError("minimized export did not complete")

                request("resolution", "ii", (320, 180))
                colors = [(1, 0, 0, 1), (0, 1, 0, 1), (0, 0, 1, 1), (1, 1, 0, 1)]
                for index, color in enumerate(colors):
                    commit(index, "solid", color)
                request("present", "i", (4,), "presented")
                native.minimize(True)
                deadline = time.monotonic() + 5
                while not native.snapshot()["minimized"]:
                    if time.monotonic() > deadline:
                        raise TimeoutError("native minimization failed")
                    time.sleep(0.05)
                time.sleep(0.3)
                report["minimized_before"] = True
                client.send("/scshader/v1/status")
                presented_before=client.expect("/scshader/v1/status.reply")[1]
                png = output / "minimized.png"
                request("screenshot", "s", (str(png),))
                status = finished()
                assert status[5:7] == [320, 180] and status[8] == 1 and status[10] == 0, status
                assert_colors(decode(png, args.ffmpeg, 320, 180), 320, 180,
                              [(255, 0, 0, 255), (0, 255, 0, 255), (0, 0, 255, 255), (255, 255, 0, 255)], 0)
                commit(0, "osc", (8, 0.4, 1))
                movie = output / "minimized.mp4"
                request("record-start", "ssi", (str(movie), str(args.ffmpeg), 30))
                time.sleep(1.2)
                request("record-stop")
                status = finished()
                frames = status[8]
                assert frames >= 12 and status[10] == 0, status
                info = json.loads(subprocess.check_output([str(args.ffprobe), "-v", "error", "-count_frames",
                    "-show_entries", "stream=codec_name,width,height,nb_read_frames", "-of", "json", str(movie)], timeout=10))
                stream, = info["streams"]
                assert (stream["codec_name"], stream["width"], stream["height"], int(stream["nb_read_frames"])) == ("h264", 320, 180, frames), stream
                pixels = subprocess.check_output([str(args.ffmpeg), "-v", "error", "-i", str(movie), "-f", "rawvideo", "-pix_fmt", "rgba", "pipe:1"], timeout=10)
                size = 320 * 180 * 4
                assert len(pixels) == frames * size and pixels[:size] != pixels[-size:], "hidden recording did not animate"
                assert native.snapshot()["minimized"], "export restored/raised the window"
                client.send("/scshader/v1/status")
                presented_after=client.expect("/scshader/v1/status.reply")[1]
                assert presented_before==presented_after,"offscreen capture was counted as window presentation"
                native.minimize(False)
                deadline=time.monotonic()+5
                while True:
                    client.send("/scshader/v1/status")
                    restored=client.expect("/scshader/v1/status.reply")[1]
                    try:
                        restored_window=native.snapshot()
                    except WindowUnavailable:
                        restored_window=None  # AppKit temporarily omits AX windows during restore.
                    if restored_window and not restored_window["minimized"] and restored>presented_after:
                        break
                    if time.monotonic()>deadline:
                        raise TimeoutError("window presentation did not resume after restore")
                    time.sleep(0.03)
                report.update(ok=True, frames=frames, minimized_after=True, restored_presentation=True, presented_while_minimized=presented_after-presented_before,png_pixels="pass", animated_video="pass")
                print(json.dumps(report, indent=2))
            finally:
                native.close()
    finally:
        (output / "result.json").write_text(json.dumps(report, indent=2) + "\n")


if __name__ == "__main__":
    main()
