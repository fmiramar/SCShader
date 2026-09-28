#!/usr/bin/env python3
"""Bounded native-renderer scheduling/packet stress with explicit recovery checks."""
import argparse
from pathlib import Path
import struct
import time

from osc_client import ManagedRenderer, osc_message, osc_bundle


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--renderer", type=Path, required=True)
    parser.add_argument("--port", type=int, default=57166)
    args = parser.parse_args()
    with ManagedRenderer(args.renderer, args.port) as renderer:
        client = renderer.client
        diagnostics = []

        def drain(seconds=0):
            end = time.monotonic() + seconds
            while True:
                reply = client.receive(max(0, min(0.01, end - time.monotonic())))
                if reply and reply[0] == "/scshader/v1/error":
                    diagnostics.append(reply[1])
                if time.monotonic() >= end:
                    break

        def status():
            drain(0.05)
            client.send("/scshader/v1/status")
            return client.expect("/scshader/v1/status.reply")

        def clear():
            drain(0.05)
            client.send("/scshader/v1/schedule/clear")
            count = client.expect("/scshader/v1/schedule/cleared")[0]
            state = status()
            assert state[4] == 0 and state[9] == 0, state
            return count

        def amount(request_id):
            client.send("/scshader/v1/uniform/get", "isi", (1, "amount", request_id))
            reply = client.expect("/scshader/v1/uniform/value", predicate=lambda values: values[2] == request_id)
            return struct.unpack(">f", reply[4])[0]

        baseline = status()
        update = osc_message("/scshader/v1/uniform/f", "isf", (1, "amount", 0.75))
        # Fill by command count, allowing the render thread to consume each burst.
        packet = osc_bundle([update] * 64, time.time() + 30)
        for _ in range(160):
            client.send_packet(packet)
            drain(0.01)
        state = status()
        assert 0 < state[4] <= 8192 and state[9] <= 16 * 1024 * 1024, state
        assert state[10] > 0 and state[11] == 0, state
        assert diagnostics and all(reply[3] == "E_SCHEDULE_LIMIT" for reply in diagnostics), diagnostics
        assert clear() > 0
        print("PASS command bound and queue clearing", flush=True)

        rejected = state[10]
        diagnostics.clear()
        # Stay below macOS's default UDP datagram ceiling (9216 bytes).
        client.send("/scshader/v1/buffer/create", "ii", (990, 2048))
        write = osc_message("/scshader/v1/buffer/write", "iib", (990, 0, bytes(2048 * 4)))
        packet = osc_bundle([write], time.time() + 120)
        for _ in range(2400):
            client.send_packet(packet)
            drain(0.01)
        state = status()
        assert 0 < state[9] <= 16 * 1024 * 1024 and state[4] < 8192, state
        assert state[10] > rejected and state[11] == 0, state
        assert diagnostics and all(reply[3] == "E_SCHEDULE_LIMIT" for reply in diagnostics), diagnostics
        clear()
        client.send("/scshader/v1/buffer/free", "i", (990,))
        print("PASS payload-byte bound and recovery", flush=True)

        client.send("/scshader/v1/uniform/f", "isf", (1, "amount", 0.25))
        client.send_packet(osc_bundle([update, osc_message("/scshader/v1/unknown")]))
        assert client.expect("/scshader/v1/error")[3] == "E_PROTOCOL"
        assert amount(10) == 0.25, "invalid bundle partially applied"
        nested = osc_bundle([osc_bundle([update])], time.time() + 0.3)
        client.send_packet(nested)
        time.sleep(0.05)
        assert amount(11) == 0.25, "nested immediate escaped parent timestamp"
        time.sleep(0.35)
        assert amount(12) == 0.75
        print("PASS atomic rejection and nested timing", flush=True)

        deeply_nested = update
        for _ in range(20):
            deeply_nested = osc_bundle([deeply_nested])
        for packet in [deeply_nested, osc_bundle([update], time.time() + 3605)]:
            client.send_packet(packet)
            assert client.expect("/scshader/v1/error")[3] == "E_PROTOCOL"
        final = status()
        assert final[1] > baseline[1] and final[4] == 0 and final[9] == 0, final
        client.send("/scshader/v1/ping", "id", (99, 0.0))
        assert client.expect("/scshader/v1/pong")[0] == 99
        print("PASS malformed/deep/far-future rejection; renderer still drawing and responsive", flush=True)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
