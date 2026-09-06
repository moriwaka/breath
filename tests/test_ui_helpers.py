#!/usr/bin/env python3
"""Small regression tests for AT-SPI tree helpers."""

import unittest
from subprocess import TimeoutExpired
from unittest.mock import Mock, call, patch

from ui_locale import restore_preset
from ui_smoke import find_named, terminate_process


class Node:
    def __init__(self, name, children=()):
        self.name = name
        self.children = children

    def __iter__(self):
        return iter(self.children)

    def getRoleName(self):
        return "label"


class UiHelperTests(unittest.TestCase):
    def test_find_named_ignores_transient_empty_atspi_nodes(self):
        root = Node("root", [None, Node("Audio warning")])

        self.assertEqual([node.name for node in find_named(root, "Audio warning")], ["Audio warning"])

    def test_terminate_process_kills_a_process_that_does_not_exit(self):
        process = Mock()
        process.poll.return_value = None
        process.wait.side_effect = [TimeoutExpired("breath", 2), None]

        terminate_process(process)

        process.terminate.assert_called_once_with()
        process.kill.assert_called_once_with()
        self.assertEqual(process.wait.call_args_list, [call(timeout=2), call()])

    @patch("ui_locale.subprocess.run")
    def test_restore_preset_reapplies_the_saved_gsettings_value(self, run):
        restore_preset("'deep-calm'")

        run.assert_called_once_with(
            [
                "gsettings",
                "set",
                "io.github.moriwaka.Breath",
                "preset-id",
                "'deep-calm'",
            ],
            check=True,
        )


if __name__ == "__main__":
    unittest.main()
