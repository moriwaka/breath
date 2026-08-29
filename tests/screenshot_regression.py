#!/usr/bin/env python3
"""Capture native GNOME screens and optionally compare them with baselines."""

import os
import shutil
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
    if not shutil.which("gnome-screenshot"):
        print(
            "ERROR: gnome-screenshot is required; install it before running "
            "the GNOME screenshot regression test.",
            file=sys.stderr,
        )
        raise SystemExit(1)
    try:
        result = subprocess.run(
            ["gnome-screenshot", "--file", str(path)],
            capture_output=True,
            text=True,
            timeout=10,
        )
    except subprocess.TimeoutExpired:
        print("ERROR: gnome-screenshot timed out", file=sys.stderr)
        raise SystemExit(1)
    if result.returncode:
        print(f"ERROR: GNOME screenshot capture failed: {result.stderr.strip()}", file=sys.stderr)
        raise SystemExit(1)
    if not path.is_file():
        print(f"ERROR: gnome-screenshot did not create {path}", file=sys.stderr)
        raise SystemExit(1)
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
