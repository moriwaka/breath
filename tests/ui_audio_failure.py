#!/usr/bin/env python3
"""AT-SPI coverage for the user-visible audio playback failure warning."""

import os
import subprocess
import sys
import time

from ui_smoke import find_named, invoke, wait_for_application


def main():
    if not any(os.environ.get(key) for key in ("DISPLAY", "WAYLAND_DISPLAY")):
        print("SKIP: no graphical display is available", file=sys.stderr)
        return 77
    bus_check = subprocess.run(
        [
            "gdbus",
            "call",
            "--session",
            "--dest",
            "org.a11y.Bus",
            "--object-path",
            "/org/a11y/bus",
            "--method",
            "org.freedesktop.DBus.Peer.Ping",
        ],
        capture_output=True,
        text=True,
    )
    if bus_check.returncode:
        print("SKIP: AT-SPI accessibility bus is unavailable", file=sys.stderr)
        return 77
    try:
        import pyatspi
    except ImportError:
        print("SKIP: pyatspi is not installed", file=sys.stderr)
        return 77

    try:
        desktop = pyatspi.Registry.getDesktop(0)
        _ = desktop.childCount
    except Exception as exc:
        print(f"SKIP: AT-SPI accessibility bus is unavailable: {exc}", file=sys.stderr)
        return 77

    command = sys.argv[1:] or ["breath"]
    environment = os.environ.copy()
    environment["BREATH_AUDIO_DIR"] = os.path.join("work", "missing-audio")
    if command[0].endswith("/breath") and command[0] != "breath":
        environment["GSETTINGS_SCHEMA_DIR"] = "work/gsettings"
    process = subprocess.Popen(command, env=environment)
    try:
        app = wait_for_application()
        preset = find_named(app, "4-7-8 深い落ち着き", "button")
        if not preset:
            preset = find_named(app, "4-7-8 Deep Calm", "button")
        assert preset, "preset action is missing"
        invoke(preset[0])

        warning = (
            "音声を再生できません。音声ファイルまたはGStreamerのデコーダーを確認してください."
        )
        deadline = time.monotonic() + 8
        while time.monotonic() < deadline:
            app = wait_for_application(timeout=1)
            if find_named(app, warning, "label"):
                print("PASS: audio playback failure warning is exposed via AT-SPI")
                return 0
            time.sleep(0.1)
        raise AssertionError("audio playback failure warning was not exposed")
    finally:
        if process.poll() is None:
            process.terminate()
            try:
                process.wait(timeout=2)
            except subprocess.TimeoutExpired:
                process.kill()
        pyatspi.Registry.stop()


if __name__ == "__main__":
    raise SystemExit(main())
