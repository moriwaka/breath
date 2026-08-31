#!/usr/bin/env python3
"""Small regression tests for screenshot session preparation."""

import unittest
from unittest.mock import patch

from screenshot_regression import start_session


class ScreenshotRegressionTests(unittest.TestCase):
    @patch("screenshot_regression.time.sleep")
    @patch("screenshot_regression.invoke")
    @patch("screenshot_regression.find_named")
    @patch("screenshot_regression.wait_for_application")
    def test_start_session_invokes_start_and_waits_for_inhale(
        self, wait_for_application, find_named, invoke, _sleep
    ):
        home = object()
        session = object()
        start = object()
        wait_for_application.side_effect = [home, session]
        find_named.side_effect = [[start], [object()]]

        start_session()

        invoke.assert_called_once_with(start)
        self.assertEqual(find_named.call_args_list[0].args, (home, "開始", "button"))
        self.assertEqual(find_named.call_args_list[1].args, (session, "吸う", "label"))


if __name__ == "__main__":
    unittest.main()
