#!/usr/bin/env python3
"""AT-SPI coverage for the English locale and translated controls."""

import os
import subprocess
import sys
import time

from ui_smoke import find_named, wait_for_application, walk


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
        {
            "LANGUAGE": "en_US:en",
            "LC_ALL": "C",
            "LC_MESSAGES": "C",
            "LANG": "C",
        }
    )
    if command[0].endswith("/breath") and command[0] != "breath":
        environment["GSETTINGS_SCHEMA_DIR"] = "work/gsettings"
    process = subprocess.Popen(command, env=environment)
    try:
        app = wait_for_application()
        deadline = time.monotonic() + 3
        while time.monotonic() < deadline:
            app = wait_for_application(timeout=1)
            names = {node.name for node in walk(app) if node.name}
            if {"Breathing pattern", "Session", "Guidance audio"} <= names and any(
                name in names
                for name in (
                    "Deep Calm",
                    "Awake",
                    "Coherent Breathing",
                    "Extended Exhale",
                    "Pranayama",
                    "Square Breathing",
                    "Ujjayi",
                )
            ):
                break
            time.sleep(0.1)
        names = {node.name for node in walk(app) if node.name}
        initial_preset = next(
            (
                name
                for name in (
                    "Deep Calm",
                    "Awake",
                    "Coherent Breathing",
                    "Extended Exhale",
                    "Pranayama",
                    "Square Breathing",
                    "Ujjayi",
                )
                if name in names
            ),
            None,
        )
        assert initial_preset, "English preset name is missing"
        assert "Breathing pattern" in names, "English breathing pattern group is missing"
        assert "Session" in names, "English session group is missing"
        assert "Guidance audio" in names, "English audio control is missing"

        target_preset = "Awake" if initial_preset != "Awake" else "Deep Calm"
        target_key = {"Deep Calm": "deep-calm", "Awake": "awake"}[target_preset]
        subprocess.run(
            [
                "gsettings",
                "set",
                "io.github.moriwaka.Breath",
                "preset-id",
                target_key,
            ],
            check=True,
        )

        deadline = time.monotonic() + 3
        while time.monotonic() < deadline:
            app = wait_for_application(timeout=1)
            if find_named(app, target_preset) and find_named(
                app,
                {
                    "Deep Calm": "Calm your nervous system slowly  ·  4s / 7s / 8s",
                    "Awake": "For morning energy and focus  ·  6s / 2s",
                }[target_preset],
            ):
                break
            time.sleep(0.1)
        assert find_named(app, target_preset), "Home screen did not update the selected preset"
        assert find_named(
            app,
            {
                "Deep Calm": "Calm your nervous system slowly  ·  4s / 7s / 8s",
                "Awake": "For morning energy and focus  ·  6s / 2s",
            }[target_preset],
        ), "Home screen did not update the selected preset details"
        print("PASS: English locale names and inline settings controls are exposed via AT-SPI")
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
