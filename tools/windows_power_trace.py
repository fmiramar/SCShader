"""Observe display-power notifications without changing Windows power settings.

Uses PowerSettingRegisterNotification's callback API (Powrprof.dll), with
GUID_SESSION_DISPLAY_STATUS as documented for interactive applications:
https://learn.microsoft.com/en-us/windows/win32/power/power-setting-guids
"""
from __future__ import annotations

import ctypes
from datetime import datetime, timezone
import json
import os
from pathlib import Path
from queue import Empty, SimpleQueue
import threading
import time
import uuid

DISPLAY_GUID = uuid.UUID("2b84c20e-ad23-4ddf-93db-05ffbd7efca5").bytes_le


class PowerSetting(ctypes.Structure):
    _fields_ = [("guid", ctypes.c_ubyte * 16), ("length", ctypes.c_uint32),
                ("value", ctypes.c_uint32)]


def display_state(setting: PowerSetting) -> str:
    if bytes(setting.guid) != DISPLAY_GUID or setting.length != 4 or setting.value not in (0, 1, 2):
        raise ValueError("invalid display-power notification")
    return ("off", "on", "dim")[setting.value]


class WindowsPowerTrace:
    def __init__(self, path: Path):
        if os.name != "nt":
            raise RuntimeError("Windows display-power tracing requires Windows")
        self.path = path
        self.events = SimpleQueue()
        self.initial = threading.Event()
        self.handle = ctypes.c_void_p()
        self.output = None
        self.api = ctypes.WinDLL("powrprof", use_last_error=True)
        callback_type = ctypes.WINFUNCTYPE(ctypes.c_uint32, ctypes.c_void_p,
                                         ctypes.c_uint32, ctypes.c_void_p)

        class Subscription(ctypes.Structure):
            _fields_ = [("callback", callback_type), ("context", ctypes.c_void_p)]

        self.callback = callback_type(self._callback)
        self.subscription = Subscription(self.callback, None)
        self.guid = (ctypes.c_ubyte * 16).from_buffer_copy(DISPLAY_GUID)
        self.api.PowerSettingRegisterNotification.argtypes = [ctypes.c_void_p, ctypes.c_uint32,
                                                              ctypes.c_void_p, ctypes.POINTER(ctypes.c_void_p)]
        self.api.PowerSettingRegisterNotification.restype = ctypes.c_uint32
        self.api.PowerSettingUnregisterNotification.argtypes = [ctypes.c_void_p]
        self.api.PowerSettingUnregisterNotification.restype = ctypes.c_uint32

    def _callback(self, context, kind, address):
        # Callback threads only enqueue owned data; file I/O stays on the driver
        # thread. Never allow a Python exception to cross the native callback.
        try:
            if kind == 0x8013 and address:  # PBT_POWERSETTINGCHANGE
                setting = PowerSetting.from_address(address)
                event = dict(utc=datetime.now(timezone.utc).isoformat(), monotonic=time.monotonic(),
                             event="display_power", state=display_state(setting))
                self.events.put(event)
                self.initial.set()
        except Exception as error:
            self.events.put(dict(event="trace_error", reason=str(error)))
            self.initial.set()
        return 0

    def drain(self):
        error = None
        while True:
            try:
                event = self.events.get_nowait()
            except Empty:
                break
            self.output.write(json.dumps(event) + "\n")
            self.output.flush()
            if event["event"] == "trace_error":
                error = event["reason"]
        if error:
            raise RuntimeError(error)

    def __enter__(self):
        self.output = self.path.open("x", encoding="utf-8")
        try:
            code = self.api.PowerSettingRegisterNotification(ctypes.byref(self.guid), 2,
                       ctypes.byref(self.subscription), ctypes.byref(self.handle))  # DEVICE_NOTIFY_CALLBACK
            if code:
                raise ctypes.WinError(code)
            if not self.initial.wait(3):
                raise RuntimeError("no initial Windows display-power notification")
            self.drain()
            return self
        except BaseException:
            self.__exit__(None, None, None)
            raise

    def __exit__(self, *_):
        try:
            if self.handle.value:
                code = self.api.PowerSettingUnregisterNotification(self.handle)
                self.handle = ctypes.c_void_p()
                if code:
                    raise ctypes.WinError(code)
            self.drain()
        finally:
            if self.output:
                self.output.close()
