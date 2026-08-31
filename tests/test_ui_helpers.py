#!/usr/bin/env python3
"""Small regression tests for AT-SPI tree helpers."""

import unittest

from ui_smoke import find_named


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


if __name__ == "__main__":
    unittest.main()
