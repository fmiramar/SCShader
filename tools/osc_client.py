"""Small, strict OSC test client and scoped renderer process owner (stdlib only)."""
from __future__ import annotations

import contextlib
import os
import select
import socket
import struct
import subprocess
import tempfile
import time
from pathlib import Path


def osc_string(value: str) -> bytes:
    if "\0" in value:
        raise ValueError("OSC strings cannot contain NUL")
    encoded = value.encode("utf-8") + b"\0"
    return encoded + bytes((-len(encoded)) % 4)


def osc_message(address: str, tags: str = "", values: tuple = ()) -> bytes:
    if len(tags) != len(values):
        raise ValueError("OSC type/value count mismatch")
    result = bytearray(osc_string(address) + osc_string("," + tags))
    formats = {"i": ">i", "h": ">q", "f": ">f", "d": ">d"}
    for tag, value in zip(tags, values):
        if tag in formats:
            result.extend(struct.pack(formats[tag], value))
        elif tag == "s":
            result.extend(osc_string(value))
        elif tag == "b":
            result.extend(struct.pack(">i", len(value)))
            result.extend(value)
            result.extend(bytes((-len(value)) % 4))
        else:
            raise ValueError(f"unsupported OSC type: {tag}")
    return bytes(result)


def osc_bundle(packets: list[bytes], when: float | None = None) -> bytes:
    if when is None:
        tag = 1
    else:
        ntp = when + 2_208_988_800
        seconds = int(ntp)
        tag = (seconds << 32) | int((ntp - seconds) * (1 << 32))
    return b"#bundle\0" + struct.pack(">Q", tag) + b"".join(struct.pack(">i", len(p)) + p for p in packets)


def decode_message(packet: bytes) -> tuple[str, list]:
    offset = 0

    def string() -> str:
        nonlocal offset
        end = packet.find(b"\0", offset)
        if end < 0:
            raise ValueError("unterminated OSC string")
        result = packet[offset:end].decode("utf-8")
        next_offset = (end + 4) & ~3
        if next_offset > len(packet) or any(packet[end:next_offset]):
            raise ValueError("invalid OSC string padding")
        offset = next_offset
        return result

    address, tags = string(), string()
    if not address.startswith("/") or not tags.startswith(","):
        raise ValueError("expected an OSC message")
    values = []
    formats = {"i": ">i", "h": ">q", "f": ">f", "d": ">d"}
    for tag in tags[1:]:
        if tag in formats:
            size = struct.calcsize(formats[tag])
            if offset + size > len(packet):
                raise ValueError("truncated OSC number")
            values.append(struct.unpack_from(formats[tag], packet, offset)[0])
            offset += size
        elif tag == "s":
            values.append(string())
        elif tag == "b":
            if offset + 4 > len(packet):
                raise ValueError("truncated OSC blob size")
            size = struct.unpack_from(">i", packet, offset)[0]
            offset += 4
            if size < 0 or offset + size > len(packet):
                raise ValueError("invalid OSC blob size")
            values.append(packet[offset:offset + size])
            offset += size
            end = (offset + 3) & ~3
            if end > len(packet) or any(packet[offset:end]):
                raise ValueError("invalid OSC blob padding")
            offset = end
        else:
            raise ValueError(f"unsupported OSC reply type: {tag}")
    if offset != len(packet):
        raise ValueError("trailing OSC bytes")
    return address, values


class OscClient:
    def __init__(self, port: int):
        self.address = ("127.0.0.1", port)
        self.socket = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        self.socket.bind(("127.0.0.1", 0))

    def close(self):
        self.socket.close()

    def send(self, address, tags="", values=()):
        self.send_packet(osc_message(address, tags, values))

    def send_packet(self, packet):
        self.socket.sendto(packet, self.address)

    def receive(self, timeout=0.0):
        deadline = time.monotonic() + timeout
        while True:
            if not select.select([self.socket], [], [], max(0.0, deadline - time.monotonic()))[0]:
                return None
            packet, sender = self.socket.recvfrom(65_535)
            if sender == self.address:
                return decode_message(packet)
            if time.monotonic() >= deadline:
                return None

    def expect(self, address, timeout=5.0, predicate=lambda args: True):
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            reply = self.receive(min(0.1, max(0.0, deadline - time.monotonic())))
            if reply is None:
                continue
            path, args = reply
            if path == address and predicate(args):
                return args
            if path == "/scshader/v1/error":
                raise RuntimeError(f"renderer diagnostic: {args}")
        raise TimeoutError(f"no {address} reply within {timeout}s")

    def hello(self, timeout=8.0):
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            try:
                self.send("/scshader/v1/hello", "iis", (1, self.socket.getsockname()[1], "scshader-test"))
                reply = self.expect("/scshader/v1/ready", timeout=0.2)
                if len(reply) != 4 or reply[0] != 1:
                    raise RuntimeError("invalid protocol handshake")
                return reply
            except TimeoutError:
                pass
            except ConnectionResetError as error:
                # Winsock reports ICMP port-unreachable from a hello sent before
                # the child bound its UDP socket. Retry only during startup;
                # ordinary receive errors still fail an established session.
                if os.name != "nt" or error.winerror != 10054:
                    raise
                time.sleep(0.01)
        raise TimeoutError("renderer did not complete handshake")


class ManagedRenderer:
    """Own exactly one child, never attach to or terminate an existing renderer."""
    def __init__(self, executable: Path, port: int, log: Path | None = None, extra_args=()):
        self.executable, self.port, self.log_path = executable, port, log
        self.extra_args = list(extra_args)
        self.process = self.client = self.log = None
        self.ready = False
        self.ready_reply = None

    def __enter__(self):
        try:
            # Refuse an occupied port before launching a child or sending OSC.
            with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as probe:
                probe.bind(("127.0.0.1", self.port))
            self.log = self.log_path.open("w+") if self.log_path else tempfile.TemporaryFile(mode="w+")
            self.process = subprocess.Popen([str(self.executable.resolve()), *self.extra_args, "--listen-port", str(self.port),
                                             "--width", "960", "--height", "540", "--title", "SCShader verification"],
                                            stdout=self.log, stderr=subprocess.STDOUT)
            self.client = OscClient(self.port)
            self.ready_reply = self.client.hello()
            if self.process.poll() is not None:
                raise RuntimeError(f"renderer exited with {self.process.returncode}")
            self.ready = True
            return self
        except BaseException:
            self.__exit__(None, None, None)
            raise

    def __exit__(self, *_):
        if self.process and self.process.poll() is None:
            if self.ready:
                with contextlib.suppress(OSError):
                    self.client.send("/scshader/v1/quit")
            try:
                self.process.wait(timeout=3)
            except subprocess.TimeoutExpired:
                self.process.terminate()
                try:
                    self.process.wait(timeout=3)
                except subprocess.TimeoutExpired:
                    self.process.kill()
                    self.process.wait(timeout=3)
        if self.client:
            self.client.close()
        if self.log:
            self.log.close()
