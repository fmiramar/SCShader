#!/usr/bin/env python3
"""Short owned-child Hydra history reset check; gpu-test-hooks only, never shipped."""
import argparse
from pathlib import Path
import struct
import time
from osc_client import ManagedRenderer

BASE = "/scshader/v1/hydra/"


def ir(feedback):
    def node(i, name, category, receiver, arguments):
        return (struct.pack(">IBBH", i, category, len(arguments), len(name)) + name.encode()
                + struct.pack(">I", receiver) + b"".join(arguments))
    def scalar(value):
        return b"\0" + struct.pack(">f", value)
    nodes = [node(1, "prev", 0, 0, []),
             node(2, "solid", 0, 0, [scalar(v) for v in [8/255, 0, 0, 0]]),
             node(3, "add", 3, 1, [b"\2" + struct.pack(">I", 2), scalar(1)])] if feedback else [
                 node(1, "solid", 0, 0, [scalar(0)] * 4)]
    return b"HYIR\0\1\0\0" + struct.pack(">III", len(nodes), 0, len(nodes)) + b"".join(nodes)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--renderer", type=Path, required=True)
    parser.add_argument("--log", type=Path, required=True)
    parser.add_argument("--port", type=int, default=57195)
    args = parser.parse_args()
    with ManagedRenderer(args.renderer, args.port, args.log, extra_args=["--backend", "metal"]) as owner:
        client = owner.client
        reply_port = client.socket.getsockname()[1]
        client.send(BASE + "capabilities", "iii", (1, 1, reply_port))
        capabilities = client.expect(BASE + "capabilities.reply")
        epoch = capabilities[1]
        assert "outputs-temporal-v1" in capabilities[-1]
        client.send(BASE + "open", "issii", (2, epoch, "hydra-recovery", 1, reply_port))
        session = client.expect(BASE + "opened")[2]
        sequence = 2
        revisions = [0] * 4

        def request(op, tags="", values=(), reply="ack"):
            nonlocal sequence
            sequence += 1
            client.send(BASE + op, "sii" + tags, (epoch, session, sequence) + values)
            return client.expect(BASE + reply)

        def commit(slot, data):
            nonlocal sequence
            sequence += 1
            prefix = (epoch, session, sequence)
            client.send(BASE + "begin", "siiiii", prefix + (slot, revisions[slot], len(data)))
            client.expect(BASE + "ack")
            client.send(BASE + "chunk", "siiib", prefix + (0, data))
            client.expect(BASE + "ack")
            client.send(BASE + "commit", "sii", prefix)
            result = client.expect(BASE + "committed")
            revisions[slot] += 1
            assert result[3:] == [slot, revisions[slot]], result

        def status():
            client.send("/scshader/v1/status")
            return client.expect("/scshader/v1/status.reply")

        def frames(count=3):
            initial = status()[1]
            end = time.monotonic() + 5
            while time.monotonic() < end:
                value = status()
                if value[1] >= initial + count:
                    return value
                time.sleep(0.02)
            raise TimeoutError("frames did not advance")

        def pixel(slot):
            client.send("/scshader/v1/test/hydra/pixel", "i", (slot,))
            return client.expect("/scshader/v1/test/pixel.reply")[0]

        def present(mode):
            request("present", "i", (mode,), "presented")

        for slot in range(4):
            commit(slot, ir(True))
        present(4)
        frames(8)
        present(-1)  # Freeze histories, so exact reset assertions have no frame race.
        before = [pixel(i) for i in range(4)]
        assert all(p[0] >= 64 and p[1:] == bytes(3) for p in before), before
        frames()
        assert [pixel(i) for i in range(4)] == before
        client.send("/scshader/v1/test/surface-loss")
        assert frames()[15] == 0
        assert [pixel(i) for i in range(4)] == before
        print("PASS hidden Hydra histories freeze; surface reacquisition preserves them", flush=True)

        baseline = status()
        client.send("/scshader/v1/test/device-loss")
        assert client.expect("/scshader/v1/error")[3] == "E_GPU_DEVICE_LOST"
        assert client.expect("/scshader/v1/gpu/recovered", timeout=10)[0] == 1
        after = frames()
        assert after[15] == 1 and after[1] > baseline[1]
        assert [pixel(i) for i in range(4)] == [bytes(4)] * 4
        state = request("status", "i", (0,), "status.reply")
        assert state[3] == -1 and struct.unpack(">4i", state[4]) == tuple(revisions), state
        present(4)
        frames()
        assert all(pixel(i)[0] > 0 for i in range(4)), "programs were not restored"
        print("PASS one logical-device restart clears all histories and restores programs/revisions/mode", flush=True)

        present(-1)
        client.send("/scshader/v1/window/resize", "ii", (320, 240))
        frames()
        assert [pixel(i) for i in range(4)] == [bytes(4)] * 4
        present(4)
        frames()
        assert all(pixel(i)[0] > 0 for i in range(4))
        for slot in range(4):
            commit(slot, ir(False))
        present(0)
        frames()
        assert [pixel(i) for i in range(4)] == [bytes(4)] * 4
        print("PASS resize resets history; hush replacements keep every output clear", flush=True)
        client.send("/scshader/v1/test/device-loss")
        assert client.expect("/scshader/v1/error")[3] == "E_GPU_FATAL"
        assert owner.process.wait(timeout=5) == 70
        print("SCSHADER_HYDRA_RECOVERY_OK (second loss exits 70)", flush=True)


if __name__ == "__main__":
    main()
