#!/usr/bin/env python3
"""Fail closed when a proposal changes bootstrap governance while the freeze is active.

The canonical checkout is the trust anchor. A candidate cannot weaken this check by
editing its own freeze record because all frozen paths and transition rules are read
from the canonical record.
"""
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

FREEZE_RECORD = Path("governance/bootstrap-governance-freeze.json")
BOOTSTRAP_STATE = Path("governance/bootstrap-operating-state.json")


def load_object(path: Path, label: str) -> dict:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise ValueError(f"invalid {label}: {exc}") from exc
    if not isinstance(value, dict):
        raise ValueError(f"{label} must be a JSON object")
    return value


def safe_relative(value: object, label: str) -> Path:
    if not isinstance(value, str) or not value:
        raise ValueError(f"{label} must be a non-empty string")
    path = Path(value)
    if path.is_absolute() or ".." in path.parts:
        raise ValueError(f"{label} must be repository-relative and may not contain '..'")
    return path


def same_file(candidate_root: Path, canonical_root: Path, relative: Path) -> bool:
    candidate = candidate_root / relative
    canonical = canonical_root / relative
    if candidate.is_file() != canonical.is_file():
        return False
    if not canonical.is_file():
        return True
    return candidate.read_bytes() == canonical.read_bytes()


def tree_files(root: Path, relative_root: Path) -> dict[str, bytes]:
    base = root / relative_root
    if not base.exists():
        return {}
    if not base.is_dir():
        raise ValueError(
            f"frozen directory root is not a directory: {relative_root.as_posix()}"
        )
    files: dict[str, bytes] = {}
    for path in sorted(base.rglob("*")):
        if path.is_symlink():
            raise ValueError(
                "frozen governance tree contains symbolic link: "
                f"{path.relative_to(root).as_posix()}"
            )
        if path.is_file():
            files[path.relative_to(root).as_posix()] = path.read_bytes()
    return files


def candidate_may_sunset(candidate_root: Path, candidate_record: dict) -> tuple[bool, str]:
    if candidate_record.get("status") != "sunset":
        return False, "candidate freeze record is not in sunset state"
    try:
        state = load_object(
            candidate_root / BOOTSTRAP_STATE,
            "candidate bootstrap operating state",
        )
    except ValueError as exc:
        return False, str(exc)
    if state.get("forge_canonical") is not True:
        return False, "bootstrap governance freeze may sunset only when forge_canonical is true"
    return True, ""


def validate(candidate_root: Path, canonical_root: Path) -> list[str]:
    errors: list[str] = []
    canonical_record_path = canonical_root / FREEZE_RECORD
    if not canonical_record_path.is_file():
        return errors

    try:
        canonical_record = load_object(
            canonical_record_path,
            "canonical bootstrap governance freeze record",
        )
    except ValueError as exc:
        return [str(exc)]

    if canonical_record.get("status") != "active":
        return errors

    exact_paths = canonical_record.get("frozen_exact_paths")
    directory_roots = canonical_record.get("frozen_directory_roots")
    if not isinstance(exact_paths, list) or not isinstance(directory_roots, list):
        return ["canonical bootstrap governance freeze record has invalid frozen path lists"]

    candidate_record_path = candidate_root / FREEZE_RECORD
    try:
        candidate_record = load_object(
            candidate_record_path,
            "candidate bootstrap governance freeze record",
        )
    except ValueError as exc:
        return [str(exc)]

    candidate_status = candidate_record.get("status")
    if candidate_status == "sunset":
        allowed, reason = candidate_may_sunset(candidate_root, candidate_record)
        if allowed:
            return errors
        errors.append(reason)
        return errors
    if candidate_status != "active":
        return [
            "active bootstrap governance freeze cannot be removed, superseded, "
            "or disabled before Forge is canonical"
        ]

    if candidate_record_path.read_bytes() != canonical_record_path.read_bytes():
        errors.append(
            "active bootstrap governance freeze record is immutable until its "
            "Forge-canonical sunset"
        )

    for index, value in enumerate(exact_paths):
        try:
            relative = safe_relative(value, f"frozen_exact_paths[{index}]")
        except ValueError as exc:
            errors.append(str(exc))
            continue
        if not same_file(candidate_root, canonical_root, relative):
            errors.append(
                f"bootstrap governance is frozen; proposal changes {relative.as_posix()}"
            )

    for index, value in enumerate(directory_roots):
        try:
            relative = safe_relative(value, f"frozen_directory_roots[{index}]")
            canonical_tree = tree_files(canonical_root, relative)
            candidate_tree = tree_files(candidate_root, relative)
        except ValueError as exc:
            errors.append(str(exc))
            continue
        if candidate_tree != canonical_tree:
            changed = sorted(set(candidate_tree) ^ set(canonical_tree))
            for path in sorted(set(candidate_tree) & set(canonical_tree)):
                if candidate_tree[path] != canonical_tree[path]:
                    changed.append(path)
            sample = ", ".join(changed[:8]) or relative.as_posix()
            if len(changed) > 8:
                sample += f", ... (+{len(changed) - 8} more)"
            errors.append(
                "bootstrap governance is frozen; proposal changes "
                f"{relative.as_posix()}/ ({sample})"
            )

    return errors


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", default=".")
    parser.add_argument("--canonical-root", required=True)
    args = parser.parse_args()

    errors = validate(Path(args.root).resolve(), Path(args.canonical_root).resolve())
    if errors:
        for error in errors:
            print(f"ERROR: {error}", file=sys.stderr)
        return 1
    print("bootstrap governance freeze: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
