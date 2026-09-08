#!/usr/bin/env python3
"""Tests for dependency_scan_gate.py (PR #327 finding F-11).

Covers exactly the fixtures the finding's own "Acceptance tests to
implement" names: clean report, genuine advisories, empty output,
malformed JSON, valid-JSON error (the concrete example), inconsistent
count/list, and unexpected exit codes.
"""

import json
import unittest

import dependency_scan_gate as gate


def _report(vulns_count=0, vulns_list=None):
    return json.dumps(
        {
            "vulnerabilities": {
                "found": vulns_count > 0,
                "count": vulns_count,
                "list": vulns_list if vulns_list is not None else [],
            }
        }
    )


class CleanReport(unittest.TestCase):
    def test_zero_count_exit_0_is_clean(self):
        result = gate.evaluate(_report(0), 0)
        self.assertEqual(result.verdict, gate.CLEAN)
        self.assertEqual(result.exit_code, 0)


class GenuineAdvisories(unittest.TestCase):
    def test_nonzero_count_exit_1_is_advisories_not_a_failure(self):
        entries = [
            {
                "advisory": {"id": "RUSTSEC-2026-0001", "title": "example flaw"},
                "package": {"name": "example-crate", "version": "1.2.3"},
            }
        ]
        result = gate.evaluate(_report(1, entries), 1)
        self.assertEqual(result.verdict, gate.ADVISORIES)
        self.assertEqual(result.exit_code, 0)
        joined = " ".join(text for _, text in result.lines)
        self.assertIn("RUSTSEC-2026-0001", joined)
        self.assertIn("example-crate", joined)
        self.assertIn("D-0441", joined)


class EmptyOutput(unittest.TestCase):
    def test_empty_string_is_a_scanner_failure(self):
        result = gate.evaluate("", 0)
        self.assertEqual(result.verdict, gate.SCANNER_FAILURE)
        self.assertEqual(result.exit_code, 1)

    def test_whitespace_only_is_a_scanner_failure(self):
        result = gate.evaluate("   \n", 1)
        self.assertEqual(result.verdict, gate.SCANNER_FAILURE)
        self.assertEqual(result.exit_code, 1)


class MalformedJson(unittest.TestCase):
    def test_truncated_json_is_a_hard_failure(self):
        result = gate.evaluate('{"vulnerabilities": {"count": 0', 0)
        self.assertEqual(result.verdict, gate.MALFORMED_REPORT)
        self.assertEqual(result.exit_code, 1)

    def test_non_object_json_is_a_hard_failure(self):
        result = gate.evaluate("[1, 2, 3]", 0)
        self.assertEqual(result.verdict, gate.MALFORMED_REPORT)
        self.assertEqual(result.exit_code, 1)


class ValidJsonError(unittest.TestCase):
    """The exact F-11 concrete example: a well-formed JSON error body with a
    nonzero exit must never be read as a clean scan."""

    def test_a_database_unavailable_error_body_with_exit_2_fails_not_clean(self):
        result = gate.evaluate('{"error": "database unavailable"}', 2)
        self.assertEqual(result.verdict, gate.SCANNER_FAILURE)
        self.assertEqual(result.exit_code, 1)
        joined = " ".join(text for _, text in result.lines)
        self.assertNotIn("No advisories", joined)
        self.assertNotIn("successfully", joined)

    def test_the_same_error_body_at_exit_0_is_still_refused_missing_schema(self):
        # Even if the process happens to exit 0, a body with no real
        # vulnerabilities object must not silently default to zero
        # advisories -- the old bug's exact mechanism.
        result = gate.evaluate('{"error": "database unavailable"}', 0)
        self.assertEqual(result.verdict, gate.MALFORMED_REPORT)
        self.assertEqual(result.exit_code, 1)


class InconsistentCountAndList(unittest.TestCase):
    def test_count_greater_than_list_length_is_rejected(self):
        report = json.dumps({"vulnerabilities": {"count": 3, "list": []}})
        result = gate.evaluate(report, 1)
        self.assertEqual(result.verdict, gate.MALFORMED_REPORT)
        self.assertEqual(result.exit_code, 1)

    def test_count_zero_but_nonempty_list_is_rejected(self):
        report = json.dumps(
            {"vulnerabilities": {"count": 0, "list": [{"advisory": {}}]}}
        )
        result = gate.evaluate(report, 1)
        self.assertEqual(result.verdict, gate.MALFORMED_REPORT)
        self.assertEqual(result.exit_code, 1)

    def test_exit_0_but_report_shows_findings_is_rejected(self):
        entries = [{"advisory": {"id": "X"}, "package": {"name": "y"}}]
        result = gate.evaluate(_report(1, entries), 0)
        self.assertEqual(result.verdict, gate.MALFORMED_REPORT)
        self.assertEqual(result.exit_code, 1)

    def test_exit_1_but_report_shows_zero_findings_is_rejected(self):
        result = gate.evaluate(_report(0), 1)
        self.assertEqual(result.verdict, gate.MALFORMED_REPORT)
        self.assertEqual(result.exit_code, 1)


class UnexpectedExitCodes(unittest.TestCase):
    def test_an_undocumented_exit_code_is_a_scanner_failure_even_with_a_clean_body(
        self,
    ):
        result = gate.evaluate(_report(0), 137)
        self.assertEqual(result.verdict, gate.SCANNER_FAILURE)
        self.assertEqual(result.exit_code, 1)

    def test_a_negative_exit_status_is_still_rejected(self):
        result = gate.evaluate(_report(0), -1)
        self.assertEqual(result.verdict, gate.SCANNER_FAILURE)
        self.assertEqual(result.exit_code, 1)


class MissingVulnerabilitiesKeyEntirely(unittest.TestCase):
    """The exact shape the old code defaulted to zero via `.get(..., {})`."""

    def test_a_top_level_object_with_no_vulnerabilities_key_is_rejected(self):
        result = gate.evaluate('{"ok": true}', 0)
        self.assertEqual(result.verdict, gate.MALFORMED_REPORT)
        self.assertEqual(result.exit_code, 1)


class CliEntryPoint(unittest.TestCase):
    def test_main_returns_the_gate_exit_code(self):
        import tempfile
        import os

        with tempfile.NamedTemporaryFile(
            mode="w", suffix=".json", delete=False
        ) as f:
            f.write(_report(0))
            path = f.name
        try:
            code = gate.main(["--report-file", path, "--exit-status", "0"])
            self.assertEqual(code, 0)
        finally:
            os.unlink(path)

    def test_main_fails_on_a_missing_report_file(self):
        code = gate.main(
            ["--report-file", "/nonexistent/path.json", "--exit-status", "0"]
        )
        self.assertEqual(code, 1)


if __name__ == "__main__":
    unittest.main()
