#!/usr/bin/env python3
"""AT-SPI coverage for the English locale and translated controls."""

import os
import subprocess
import sys
import time

from ui_smoke import find_named, invoke, wait_for_application, walk


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
    environment.update(
        {"LANGUAGE": "en_US:en", "LC_ALL": "C", "LC_MESSAGES": "C", "LANG": "C"}
    )
    if command[0].endswith("/breath") and command[0] != "breath":
        environment["GSETTINGS_SCHEMA_DIR"] = "work/gsettings"
    process = subprocess.Popen(command, env=environment)
    try:
        app = wait_for_application()
        assert find_named(app, "Deep Calm"), "English preset name is missing"
        preferences = find_named(app, "Preferences", "button")
        assert preferences, "English Preferences action is missing"
        invoke(preferences[0])

        deadline = time.monotonic() + 3
        while time.monotonic() < deadline:
            app = wait_for_application(timeout=1)
            names = {node.name for node in walk(app) if node.name}
            if {"Breathing pattern", "Session", "Guidance audio"} <= names:
                break
            time.sleep(0.1)
        names = {node.name for node in walk(app) if node.name}
        assert "Breathing pattern" in names, "English breathing pattern group is missing"
        assert "Session" in names, "English settings group is missing"
        assert "Guidance audio" in names, "English audio control is missing"
        assert "Calm your nervous system slowly  ·  4s / 7s / 8s" in names, (
            "English preset details are missing"
        )
        assert "A foundational yoga breathing practice  ·  7s / 4s / 8s / 4s" in names, (
            "English long-form preset details are missing"
        )
        print("PASS: English locale names and settings controls are exposed via AT-SPI")
        return 0
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
