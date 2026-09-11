#!/usr/bin/env python3
"""Reconciles cargo-audit's process exit status with its JSON report content
so the dependency-audit CI job (`.github/workflows/ci.yml`) can never read a
scanner operational failure as a clean or successful scan (PR #327 finding
F-11).

Before this module existed, the job's own inline python (a `bash -e`/`set
+e` heredoc, untestable outside a real CI run) checked only that
`audit-report.json` was non-empty and parseable, then defaulted a missing
`vulnerabilities` object to a zero count. `cargo-audit` exiting nonzero with
an unrelated, well-formed JSON error body (`{"error": "database
unavailable"}`) satisfied both checks: the missing count became 0, and the
job printed "Scanner ran successfully." That control-flow bug -- content
that merely parses being trusted as a truthful outcome -- is what this
module closes, by treating the captured exit status as load-bearing input
to the verdict, not a value read and then ignored.

Standalone and imported by `test_dependency_scan_gate.py` so the exact
decision logic the workflow runs is exercised directly, not only embedded
in YAML where the only way to test it is a real GitHub Actions run.
"""

from __future__ import annotations

import argparse
import json
import sys
from dataclasses import dataclass, field

CLEAN = "clean"
ADVISORIES = "advisories"
SCANNER_FAILURE = "scanner_failure"
MALFORMED_REPORT = "malformed_report"

# cargo-audit's documented exit-code contract: 0 = ran to completion, no
# vulnerabilities; 1 = ran to completion, vulnerabilities found (the report
# carries a real `vulnerabilities` object with a non-empty list). Any other
# code means the tool did not complete a real scan -- database fetch
# failure, argument error, panic -- regardless of what its stdout happens
# to contain.
_CLEAN_EXIT = 0
_ADVISORIES_EXIT = 1


@dataclass
class GateResult:
    verdict: str
    exit_code: int
    # Each entry is (level, text); level is "error", "warning", or "notice".
    lines: list = field(default_factory=list)


def evaluate(report_text: str, exit_status: int) -> GateResult:
    """Decide the dependency-audit job's outcome from cargo-audit's raw
    stdout and its process exit status."""
    if not report_text or not report_text.strip():
        return GateResult(
            SCANNER_FAILURE,
            1,
            [
                (
                    "error",
                    f"cargo-audit produced no report (exit {exit_status}). "
                    "This is a scanner failure, not an advisory finding -- "
                    "failing the build.",
                )
            ],
        )

    try:
        report = json.loads(report_text)
    except (json.JSONDecodeError, ValueError):
        return GateResult(
            MALFORMED_REPORT,
            1,
            [
                (
                    "error",
                    f"cargo-audit produced an unparseable report (exit {exit_status}). "
                    "Treating a scanner malfunction as a hard failure.",
                )
            ],
        )

    if not isinstance(report, dict):
        return GateResult(
            MALFORMED_REPORT,
            1,
            [
                (
                    "error",
                    f"cargo-audit's report was not a JSON object (exit {exit_status}). "
                    "Treating a scanner malfunction as a hard failure.",
                )
            ],
        )

    if exit_status not in (_CLEAN_EXIT, _ADVISORIES_EXIT):
        # Any exit status cargo-audit does not document as a real-scan
        # outcome is an operational failure. An {"error": ...} body (or any
        # other stray JSON) must never read as "zero advisories" just
        # because it happens to parse -- this is the exact F-11 attack.
        return GateResult(
            SCANNER_FAILURE,
            1,
            [
                (
                    "error",
                    f"cargo-audit exited {exit_status}, which is not a documented "
                    "clean-scan (0) or advisories-found (1) outcome -- treating "
                    "this as an operational failure regardless of report content.",
                )
            ],
        )

    vulnerabilities = report.get("vulnerabilities")
    schema_ok = (
        isinstance(vulnerabilities, dict)
        and isinstance(vulnerabilities.get("count"), int)
        and isinstance(vulnerabilities.get("list"), list)
        and vulnerabilities.get("count") == len(vulnerabilities.get("list", []))
    )
    if not schema_ok:
        return GateResult(
            MALFORMED_REPORT,
            1,
            [
                (
                    "error",
                    f"cargo-audit exited {exit_status} but its report does not carry "
                    "a well-formed vulnerabilities object (count/list present and "
                    "consistent) -- refusing to treat a schema mismatch as zero "
                    "advisories.",
                )
            ],
        )

    count = vulnerabilities["count"]

    if exit_status == _CLEAN_EXIT and count != 0:
        return GateResult(
            MALFORMED_REPORT,
            1,
            [
                (
                    "error",
                    f"cargo-audit exited 0 (clean) but its report lists {count} "
                    "advisory finding(s) -- exit status and report content "
                    "disagree, refusing to trust either.",
                )
            ],
        )
    if exit_status == _ADVISORIES_EXIT and count == 0:
        return GateResult(
            MALFORMED_REPORT,
            1,
            [
                (
                    "error",
                    "cargo-audit exited 1 (advisories found) but its report lists "
                    "zero -- exit status and report content disagree, refusing to "
                    "trust either.",
                )
            ],
        )

    if count == 0:
        return GateResult(
            CLEAN, 0, [("notice", "No advisories against the current Cargo.lock.")]
        )

    lines = [("warning", f"cargo-audit reported {count} advisory finding(s).")]
    for entry in vulnerabilities["list"]:
        advisory = entry.get("advisory", {}) if isinstance(entry, dict) else {}
        package = entry.get("package", {}) if isinstance(entry, dict) else {}
        lines.append(
            (
                "warning",
                f"{advisory.get('id')} in {package.get('name')} "
                f"{package.get('version')}: {advisory.get('title')}",
            )
        )
    lines.append(("notice", "Advisories do not fail this job yet -- see D-0441."))
    return GateResult(ADVISORIES, 0, lines)


def main(argv: list | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--report-file", required=True, help="path to cargo-audit's --json output"
    )
    parser.add_argument(
        "--exit-status",
        required=True,
        type=int,
        help="cargo-audit's own process exit status",
    )
    args = parser.parse_args(argv)

    try:
        with open(args.report_file, "r", encoding="utf-8") as f:
            report_text = f.read()
    except OSError:
        report_text = ""

    result = evaluate(report_text, args.exit_status)
    for level, text in result.lines:
        if level in ("error", "warning"):
            print(f"::{level}::{text}")
        else:
            print(text)
    if result.exit_code == 0:
        print("Scanner ran successfully; that is what this job gates on.")
    return result.exit_code


if __name__ == "__main__":
    sys.exit(main())
