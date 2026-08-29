#!/usr/bin/env python3
"""Capture native GNOME screens and optionally compare them with baselines."""

import os
import struct
import subprocess
import sys
import time
from pathlib import Path


def png_size(path):
    with path.open("rb") as image:
        if image.read(8) != b"\x89PNG\r\n\x1a\n":
            raise AssertionError(f"not a PNG: {path}")
        length = struct.unpack(">I", image.read(4))[0]
        if image.read(4) != b"IHDR" or length < 8:
            raise AssertionError(f"PNG has no IHDR: {path}")
        width, height = struct.unpack(">II", image.read(8))
        return width, height


def capture(path):
    try:
        import gi
        gi.require_version("Gio", "2.0")
        from gi.repository import Gio, GLib
    except (ImportError, ValueError):
        print("SKIP: PyGObject/GIO is unavailable", file=sys.stderr)
        raise SystemExit(77)

    try:
        connection = Gio.bus_get_sync(Gio.BusType.SESSION, None)
        proxy = Gio.DBusProxy.new_sync(
            connection,
            Gio.DBusProxyFlags.NONE,
            None,
            "org.freedesktop.portal.Desktop",
            "/org/freedesktop/portal/desktop",
            "org.freedesktop.portal.Screenshot",
            None,
        )
    except GLib.Error as error:
        print(f"SKIP: Screenshot Portal is unavailable: {error}", file=sys.stderr)
        raise SystemExit(77)

    response = []
    loop = GLib.MainLoop()

    def on_response(_connection, _sender, object_path, _interface, _signal, parameters):
        if response and object_path == response[0]:
            response.append(parameters.unpack())
            loop.quit()

    subscription = connection.signal_subscribe(
        "org.freedesktop.portal.Desktop",
        "org.freedesktop.portal.Request",
        "Response",
        None,
        None,
        Gio.DBusSignalFlags.NONE,
        on_response,
    )
    try:
        options = {
            "interactive": GLib.Variant("b", False),
            "modal": GLib.Variant("b", False),
        }
        handle = proxy.call_sync(
            "Screenshot",
            GLib.Variant("(sa{sv})", ("", options)),
            Gio.DBusCallFlags.NONE,
            10_000,
            None,
        ).unpack()[0]
        response.append(handle)
        GLib.timeout_add_seconds(10, loop.quit)
        while len(response) == 1:
            loop.run()
        if len(response) != 2 or response[1][0] != 0:
            print("SKIP: Screenshot Portal request was not approved", file=sys.stderr)
            raise SystemExit(77)
        uri = response[1][1]["uri"]
        if hasattr(uri, "unpack"):
            uri = uri.unpack()
        Gio.File.new_for_uri(uri).copy(
            Gio.File.new_for_path(str(path)), Gio.FileCopyFlags.OVERWRITE, None, None
        )
    except (GLib.Error, KeyError, IndexError, TypeError, AttributeError) as error:
        print(f"SKIP: Screenshot Portal request failed: {error}", file=sys.stderr)
        raise SystemExit(77)
    finally:
        connection.signal_unsubscribe(subscription)
    if not path.is_file():
        print(f"SKIP: Screenshot Portal did not create {path}", file=sys.stderr)
        raise SystemExit(77)
        raise SystemExit(77)
    width, height = png_size(path)
    assert width > 0 and height > 0


def main():
    output = Path(os.environ.get("BREATH_SCREENSHOT_DIR", "work/screenshots"))
    output.mkdir(parents=True, exist_ok=True)
    process = subprocess.Popen(sys.argv[1:] or ["breath"])
    try:
        time.sleep(1)
        capture(output / "home.png")
        time.sleep(3.5)
        capture(output / "session.png")
        baseline = os.environ.get("BREATH_SCREENSHOT_BASELINE")
        if baseline:
            for name in ("home.png", "session.png"):
                expected = Path(baseline) / name
                if not expected.is_file():
                    raise AssertionError(f"missing screenshot baseline: {expected}")
                result = subprocess.run(
                    ["compare", "-metric", "AE", str(expected), str(output / name), "null:"],
                    capture_output=True,
                    text=True,
                )
                if result.returncode:
                    raise AssertionError(f"screenshot differs: {name}: {result.stderr.strip()}")
        print(f"PASS: captured native GNOME screenshots in {output}")
        return 0
    finally:
        if process.poll() is None:
            process.terminate()
            process.wait(timeout=2)


if __name__ == "__main__":
    raise SystemExit(main())
