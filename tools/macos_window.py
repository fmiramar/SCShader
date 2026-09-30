"""PID-scoped macOS Accessibility client for an owned renderer (stdlib only).

Uses Apple's AXUIElement application/window hierarchy, not system-wide UI search.
See docs/MACOS_INTERACTION_CHECKS.md for upstream API references and permissions.
"""
import ctypes as C
import sys


class Pair(C.Structure):
    _fields_ = [("x", C.c_double), ("y", C.c_double)]


class WindowUnavailable(RuntimeError):
    """AppKit may temporarily omit the window during a fullscreen transition."""


class Accessibility:
    def __init__(self):
        if sys.platform != "darwin":
            raise RuntimeError("native macOS Accessibility requires Darwin")
        self.cf = C.CDLL("/System/Library/Frameworks/CoreFoundation.framework/CoreFoundation")
        self.ax = C.CDLL("/System/Library/Frameworks/ApplicationServices.framework/ApplicationServices")
        p, i, b = C.c_void_p, C.c_int, C.c_bool
        signatures = {
            "CFRelease": ([p], None), "CFRetain": ([p], p),
            "CFEqual": ([p, p], b),
            "CFStringCreateWithCString": ([p, C.c_char_p, C.c_uint32], p),
            "CFStringGetCString": ([p, C.c_char_p, C.c_long, C.c_uint32], b),
            "CFArrayGetCount": ([p], C.c_long),
            "CFArrayGetValueAtIndex": ([p, C.c_long], p),
            "CFBooleanGetValue": ([p], b),
        }
        for name, (args, result) in signatures.items():
            fn = getattr(self.cf, name)
            fn.argtypes, fn.restype = args, result
        signatures = {
            "AXIsProcessTrusted": ([], b),
            "AXUIElementCreateApplication": ([i], p),
            "AXUIElementSetMessagingTimeout": ([p, C.c_float], i),
            "AXUIElementCopyAttributeValue": ([p, p, C.POINTER(p)], i),
            "AXUIElementSetAttributeValue": ([p, p, p], i),
            "AXUIElementPerformAction": ([p, p], i),
            "AXUIElementGetPid": ([p, C.POINTER(i)], i),
            "AXValueGetValue": ([p, i, p], b),
            "CGEventCreateKeyboardEvent": ([p, C.c_uint16, b], p),
            "CGEventPostToPid": ([i, p], None),
        }
        for name, (args, result) in signatures.items():
            fn = getattr(self.ax, name)
            fn.argtypes, fn.restype = args, result
        if not self.ax.AXIsProcessTrusted():
            raise PermissionError("Accessibility access is required for native window checks; "
                                  "no permission prompt or settings change was requested")

    @staticmethod
    def require(result, operation):
        if result != 0:
            raise RuntimeError(f"{operation} failed with AXError {result}")

    def application(self, pid):
        element = self.ax.AXUIElementCreateApplication(pid)
        if not element:
            raise RuntimeError("could not create owned application Accessibility reference")
        try:
            self.require(self.ax.AXUIElementSetMessagingTimeout(element, 2.0), "AX timeout")
        except BaseException:
            self.release(element)
            raise
        return element

    def release(self, value):
        if value:
            self.cf.CFRelease(value)

    def equal(self, left, right):
        return self.cf.CFEqual(left, right)

    def pid(self, element):
        result = C.c_int()
        self.require(self.ax.AXUIElementGetPid(element, C.byref(result)), "AX owner lookup")
        return result.value

    def copy(self, element, attribute):
        key = self.cf.CFStringCreateWithCString(None, attribute.encode(), 0x08000100)
        value = C.c_void_p()
        try:
            self.require(self.ax.AXUIElementCopyAttributeValue(element, key, C.byref(value)), attribute)
            return value.value
        finally:
            self.release(key)

    def windows(self, application):
        array = self.copy(application, "AXWindows")
        try:
            return [self.cf.CFRetain(self.cf.CFArrayGetValueAtIndex(array, index))
                    for index in range(self.cf.CFArrayGetCount(array))]
        finally:
            self.release(array)

    def attribute(self, element, name):
        value = self.copy(element, name)
        try:
            if name in ("AXTitle", "AXRole"):
                buffer = C.create_string_buffer(4096)
                if not self.cf.CFStringGetCString(value, buffer, len(buffer), 0x08000100):
                    raise RuntimeError(f"invalid/oversized {name}")
                return buffer.value.decode("utf-8")
            if name in ("AXSize", "AXPosition"):
                pair = Pair()
                if not self.ax.AXValueGetValue(value, 2 if name == "AXSize" else 1, C.byref(pair)):
                    raise RuntimeError(f"invalid {name}")
                return [pair.x, pair.y]
            if name in ("AXMinimized", "AXFullScreen", "AXFocused"):
                return bool(self.cf.CFBooleanGetValue(value))
            raise ValueError(f"unsupported attribute: {name}")
        finally:
            self.release(value)

    def set_minimized(self, element, enabled):
        key = self.cf.CFStringCreateWithCString(None, b"AXMinimized", 0x08000100)
        value = C.c_void_p.in_dll(self.cf, "kCFBooleanTrue" if enabled else "kCFBooleanFalse")
        try:
            self.require(self.ax.AXUIElementSetAttributeValue(element, key, value), "AXMinimized setter")
        finally:
            self.release(key)

    def press_close(self, element, pid):
        button = self.copy(element, "AXCloseButton")
        action = self.cf.CFStringCreateWithCString(None, b"AXPress", 0x08000100)
        try:
            if self.pid(button) != pid:
                raise RuntimeError("close button does not belong to owned process")
            self.require(self.ax.AXUIElementPerformAction(button, action), "AX close button")
        finally:
            self.release(action)
            self.release(button)

    def key(self, pid, keycode, pressed):
        event = self.ax.CGEventCreateKeyboardEvent(None, keycode, pressed)
        if not event:
            raise RuntimeError("could not create test keyboard event")
        try:
            self.ax.CGEventPostToPid(pid, event)
        finally:
            self.release(event)


class OwnedWindow:
    """Keep a retained AX reference; recheck child, PID, identity and title before use."""
    def __init__(self, process, api):
        self.process, self.api = process, api
        self.application = api.application(process.pid)
        self.window = None

    def discover(self):
        self.check_process()
        windows = list(self.api.windows(self.application))
        try:
            matches = [window for window in windows
                       if self.api.pid(window) == self.process.pid
                       and self.api.attribute(window, "AXRole") == "AXWindow"
                       and self.api.attribute(window, "AXTitle") == "SCShader verification"]
            if len(matches) > 1:
                raise RuntimeError("ambiguous owned renderer windows")
            if not matches:
                return False
            if self.window is None:
                self.window = matches[0]
                windows.remove(self.window)  # Transfer the retained reference to this owner.
            elif not self.api.equal(matches[0], self.window):
                raise RuntimeError("owned window identity changed")
            return True
        finally:
            for window in windows:
                self.api.release(window)

    def check_process(self):
        if self.process.poll() is not None:
            raise RuntimeError("owned renderer has exited")

    def snapshot(self):
        if not self.discover():
            raise WindowUnavailable("owned window unavailable or changed title")
        return {name: self.api.attribute(self.window, attribute) for name, attribute in (
            ("size", "AXSize"), ("position", "AXPosition"), ("minimized", "AXMinimized"),
            ("fullscreen", "AXFullScreen"), ("focused", "AXFocused"))}

    def minimize(self, enabled):
        self.snapshot()  # Validate ownership before every native mutation.
        self.api.set_minimized(self.window, enabled)

    def close_window(self):
        self.snapshot()
        self.api.press_close(self.window, self.process.pid)

    def key(self, keycode, pressed):
        if not self.snapshot()["focused"]:
            raise RuntimeError("owned window is not focused for keyboard check")
        self.api.key(self.process.pid, keycode, pressed)

    def close(self):
        self.api.release(self.window)
        self.api.release(self.application)
        self.window = self.application = None
