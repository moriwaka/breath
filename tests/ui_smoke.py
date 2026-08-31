#!/usr/bin/env python3
"""AT-SPI smoke test for the installed Breath application."""

import subprocess
import sys
import time
from os import environ

try:
    import pyatspi
except ImportError:
    pyatspi = None


def walk(node):
    if node is None:
        return
    yield node
    for child in node:
        yield from walk(child)


def find_named(root, name, role=None):
    return [
        node
        for node in walk(root)
        if node.name == name and (role is None or node.getRoleName() == role)
    ]


def wait_for_application(timeout=5):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        desktop = pyatspi.Registry.getDesktop(0)
        matches = [
            app for app in desktop if app.name.casefold() == "breath" and app.childCount
        ]
        if matches:
            return matches[0]
        time.sleep(0.1)
    raise AssertionError("Breath application did not appear in AT-SPI")


def invoke(button):
    actions = button.queryAction()
    available = []
    for index in range(actions.nActions):
        name = actions.getName(index)
        available.append(name)
        if name in {"click", "press"}:
            actions.doAction(index)
            return
    raise AssertionError(f"No click/press action for {button.name!r}: {available}")


def main():
    if pyatspi is None:
        print("SKIP: pyatspi is not installed", file=sys.stderr)
        return 77
    try:
        desktop = pyatspi.Registry.getDesktop(0)
        _ = desktop.childCount
    except Exception as exc:
        print(f"SKIP: AT-SPI accessibility bus is unavailable: {exc}", file=sys.stderr)
        return 77

    command = sys.argv[1:] or ["breath"]
    environment = environ.copy()
    environment.update({"LANGUAGE": "ja", "LC_ALL": "C", "LC_MESSAGES": "C", "LANG": "C"})
    if command[0].endswith("/breath") and command[0] != "breath":
        environment["GSETTINGS_SCHEMA_DIR"] = "work/gsettings"
    process = subprocess.Popen(command, env=environment)
    try:
        app = wait_for_application()
        starts = find_named(app, "開始", "button")
        assert starts, "home screen has no accessible start action"
        invoke(starts[0])

        seen = set()
        deadline = time.monotonic() + 1.5
        while time.monotonic() < deadline:
            app = wait_for_application(timeout=1)
            seen.update(node.name for node in walk(app) if node.name == "3")
            if "3" in seen:
                break
            time.sleep(0.1)
        assert "3" in seen, f"countdown start not observed: {seen}"

        countdown_stops = find_named(app, "停止", "button")
        assert countdown_stops, "countdown has no stop button"
        invoke(countdown_stops[0])
        deadline = time.monotonic() + 2
        while time.monotonic() < deadline:
            app = wait_for_application(timeout=1)
            if find_named(app, "開始", "button"):
                break
            time.sleep(0.1)
        assert find_named(app, "開始", "button"), "countdown stop did not return home"

        invoke(find_named(app, "開始", "button")[0])

        deadline = time.monotonic() + 4
        while time.monotonic() < deadline:
            app = wait_for_application(timeout=1)
            if find_named(app, "吸う", "label"):
                break
            time.sleep(0.1)
        assert find_named(app, "吸う", "label"), "session did not start after countdown"
        assert find_named(app, "一時停止", "button"), "session has no pause button"
        stops = find_named(app, "停止", "button")
        assert stops, "session has no stop button"
        invoke(stops[0])
        deadline = time.monotonic() + 2
        while time.monotonic() < deadline:
            app = wait_for_application(timeout=1)
            if find_named(app, "開始", "button"):
                break
            time.sleep(0.1)
        assert find_named(app, "開始", "button"), "stop did not return home"
        print("PASS: home, countdown, and session controls are exposed via AT-SPI")
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
