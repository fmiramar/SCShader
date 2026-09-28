#!/usr/bin/env python3
"""Native GPU restart/readback check; requires --features gpu-test-hooks (not shipped).

Destroys only our child renderer's logical wgpu device. Does not reset the host GPU.
An actual driver hang/reset remains a separate hardware/backend qualification.
"""
import argparse
from pathlib import Path
import shutil
import struct
import tempfile
import time

from osc_client import ManagedRenderer, osc_bundle, osc_message


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--renderer", type=Path, required=True)
    parser.add_argument("--port", type=int, default=57168)
    parser.add_argument("--log", type=Path)
    args = parser.parse_args()
    project = Path(__file__).resolve().parents[1]
    with tempfile.TemporaryDirectory(prefix="scshader-recovery-") as temporary:
        temporary = Path(temporary)
        shader = temporary / "recovery.wgsl"
        image = temporary / "source.ppm"
        shutil.copyfile(project / "tests/shaders/recovery.wgsl", shader)
        image.write_bytes(b"P6\n1 1\n255\n" + bytes([64, 128, 192]))
        with ManagedRenderer(args.renderer, args.port, args.log) as owner:
            client = owner.client

            def status():
                client.send("/scshader/v1/status")
                return client.expect("/scshader/v1/status.reply")

            def frames_after(previous, count=3):
                end = time.monotonic() + 5
                while time.monotonic() < end:
                    state = status()
                    if state[1] >= previous + count:
                        return state
                    time.sleep(0.02)
                raise TimeoutError("frame counter did not progress")

            def pixel():
                client.send("/scshader/v1/test/pixel")
                return client.expect("/scshader/v1/test/pixel.reply")[0]

            def value(resource, name, tag="f"):
                client.send("/scshader/v1/uniform/get", "isi", (resource, name, 1))
                reply = client.expect("/scshader/v1/uniform/value")
                return struct.unpack(">" + tag, reply[4])

            def ping():
                client.send("/scshader/v1/ping", "id", (1, time.monotonic()))
                return client.expect("/scshader/v1/pong")[2]

            assert status()[14:16] == [0, 0], "overlay/recovery should default off/zero"
            for resource in [7, 8, 9]:
                client.send("/scshader/v1/shader/create", "iss", (resource, str(shader), "wgsl"))
                client.expect("/scshader/v1/shader/created")
                client.send("/scshader/v1/uniform/f", "isf", (resource, "gain", 0.5))
                client.send("/scshader/v1/uniform/set", "issb", (resource, "tint", "vec3",
                    struct.pack(">fff", 0.02, 0.03, 0.04)))
            client.send("/scshader/v1/texture/create", "is", (20, str(image)))
            assert status()[6] == 1
            client.send("/scshader/v1/shader/texture", "isi", (9, "source", 20))
            client.send("/scshader/v1/buffer/create", "ii", (21, 128))
            assert status()[7] == 1
            client.send("/scshader/v1/buffer/write", "iib", (21, 0, struct.pack(">f", 0.125)))
            client.send("/scshader/v1/shader/buffer", "isi", (9, "spectrum", 21))
            client.send("/scshader/v1/graph/set", "ii", (7, 8))
            baseline = frames_after(status()[1])
            reference = pixel()
            assert any(reference[:-1]), "fixture must produce visible output"
            client.send("/scshader/v1/texture/create", "is", (22, str(temporary / "missing.png")))
            assert client.expect("/scshader/v1/error")[3] == "E_TEXTURE_LOAD"
            assert status()[6] == 2, "failed image load did not create a placeholder"
            client.send("/scshader/v1/shader/texture", "isi", (9, "source", 22))
            frames_after(status()[1])
            assert pixel() != reference, "diagnostic placeholder did not render"
            client.send("/scshader/v1/texture/free", "i", (22,))
            client.send("/scshader/v1/shader/texture", "isi", (9, "source", 20))
            frames_after(status()[1])
            assert pixel() == reference
            print("PASS missing-image diagnostic placeholder and rebind", flush=True)
            baseline = status()
            client.send("/scshader/v1/diagnostics/overlay", "i", (1,))
            overlay_state = frames_after(baseline[1])
            assert overlay_state[14] == 1 and overlay_state[13] > baseline[13]
            assert pixel() == reference, "overlay contaminated feedback history"
            print("PASS live overlay and feedback isolation", flush=True)

            client.send("/scshader/v1/test/surface-loss")
            surface_state = frames_after(overlay_state[1])
            assert surface_state[15] == 0 and pixel() == reference
            print("PASS presentation surface recreation without a device restart", flush=True)

            # Failed source edits and removed images must not affect cached recovery.
            shader.write_text("invalid WGSL edit\n")
            image.unlink()
            errors = []
            deadline = time.monotonic() + 5
            while len(errors) < 3 and time.monotonic() < deadline:
                reply = client.receive(0.2)
                if reply and reply[0] == "/scshader/v1/error":
                    assert reply[1][3] == "E_SHADER_COMPILE", reply
                    errors.append(reply[1][2])
            assert sorted(errors) == [7, 8, 9], errors
            client.send("/scshader/v1/uniform/glide", "issbds", (9, "gain", "float",
                struct.pack(">f", 1.0), 20.0, "linear"))
            ramp_before = value(9, "gain")[0]
            before = status()
            clock_before = ping()
            client.send_packet(osc_bundle([osc_message("/scshader/v1/timing/marker", "i", (42,))], time.time() + 4))
            client.send("/scshader/v1/test/device-loss")
            assert client.expect("/scshader/v1/error")[3] == "E_GPU_DEVICE_LOST"
            recovered = client.expect("/scshader/v1/gpu/recovered", timeout=15)
            assert recovered[0] == 1 and "history cleared" in recovered[1], recovered
            after = frames_after(before[1])
            assert after[5:9] == before[5:9] == [3, 1, 1, 1], (before, after)
            assert after[14:17] == [1, 1, 0], after
            assert ping() > clock_before
            assert pixel() == reference, "GPU resources/graph/typed controls changed across recovery"
            assert value(7, "gain") == (0.5,)
            assert value(8, "tint", "fff") == struct.unpack(">fff", struct.pack(">fff", 0.02, 0.03, 0.04))
            assert ramp_before < value(9, "gain")[0] < 1.0, "active ramp restarted or disappeared"
            marker = client.expect("/scshader/v1/timing/marker.reply", timeout=6)
            assert marker[0] == 42, marker
            print("PASS device recovery: exact GPU pixels, graph, cached images/data/source, controls, ramp, clock, schedule", flush=True)
            client.send("/scshader/v1/test/device-loss")
            assert client.expect("/scshader/v1/error")[3] == "E_GPU_FATAL"
            assert owner.process.wait(timeout=5) == 70
            print("PASS second device loss exits with code 70", flush=True)


if __name__ == "__main__":
    main()
