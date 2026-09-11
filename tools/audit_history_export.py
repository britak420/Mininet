#!/usr/bin/env python3
"""Export public PR evidence without interpreting it as review or approval.

Read-only GitHub REST calls; stdlib only. Fail on missing pages or duplicate PRs.
Never executes PR contents. GITHUB_TOKEN is used only as an HTTPS API credential.
The export is a point-in-time observation, not a cryptographic attestation of
GitHub's completeness, authors' identities, or independent human review.
"""
from __future__ import annotations
import argparse
from concurrent.futures import ThreadPoolExecutor
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import time
import tomllib
from urllib.error import HTTPError
from urllib.request import Request, urlopen

API = "https://api.github.com"
MAX_RESPONSE = 32 * 1024 * 1024
MAX_PAGES = 100


def get_json(path: str):
    if not path.startswith("/repos/") or ".." in path:
        raise ValueError("only repository REST paths are accepted")
    headers = {"Accept": "application/vnd.github+json", "X-GitHub-Api-Version": "2022-11-28", "User-Agent": "mininet-read-only-history-export"}
    token = os.environ.get("GITHUB_TOKEN")
    if token:
        headers["Authorization"] = "Bearer " + token
    for attempt in range(4):
        try:
            with urlopen(Request(API + path, headers=headers), timeout=45) as response:
                data = response.read(MAX_RESPONSE + 1)
            if len(data) > MAX_RESPONSE:
                raise ValueError("response exceeds export limit")
            return json.loads(data)
        except HTTPError as exc:
            if exc.code not in (429, 500, 502, 503, 504) or attempt == 3:
                raise RuntimeError(f"GET failed ({exc.code}): {path}") from None
            time.sleep(2 ** attempt)
    raise RuntimeError("retry exhausted")


def pages(path: str) -> list:
    result = []
    separator = "&" if "?" in path else "?"
    for page in range(1, MAX_PAGES + 1):
        items = get_json(f"{path}{separator}per_page=100&page={page}")
        if not isinstance(items, list):
            raise ValueError(f"expected a list: {path}")
        result.extend(items)
        if len(items) < 100:
            return result
    raise ValueError(f"pagination limit reached: {path}")


def write_json(path: Path, value) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def collect_one(repo: str, number: int, output: Path) -> dict:
    root = f"/repos/{repo}"
    info = get_json(f"{root}/pulls/{number}")
    files = pages(f"{root}/pulls/{number}/files")
    comments = pages(f"{root}/issues/{number}/comments")
    inline = pages(f"{root}/pulls/{number}/comments")
    reviews = pages(f"{root}/pulls/{number}/reviews")
    commits = pages(f"{root}/pulls/{number}/commits")
    if len(files) != info["changed_files"]:
        raise ValueError(f"PR {number}: file count mismatch/API cap or concurrent update")
    if len(commits) != info["commits"]:
        raise ValueError(f"PR {number}: commit count mismatch/API cap or concurrent update")
    again = get_json(f"{root}/pulls/{number}")
    if again["head"]["sha"] != info["head"]["sha"]:
        raise ValueError(f"PR {number}: head moved during capture; retry export")
    outcome = "merged" if info["merged"] else info["state"]
    row = {"number": number, "title": info["title"], "url": info["html_url"], "outcome": outcome,
           "created_at": info["created_at"], "updated_at": info["updated_at"], "merged_at": info["merged_at"],
           "base_sha": info["base"]["sha"], "head_sha": info["head"]["sha"], "merge_commit_sha": info["merge_commit_sha"],
           "changed_files": len(files), "commits": len(commits), "issue_comments": len(comments), "inline_comments": len(inline), "reviews": len(reviews),
           "files_without_patch": [f["filename"] for f in files if "patch" not in f],
           "evidence_file": f"pulls/{number:04d}.json", "review_completed": False}
    write_json(output / row["evidence_file"], {"info": info, "files": files, "comments": comments, "inline_comments": inline, "reviews": reviews, "commits": commits})
    return row


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", required=True)
    parser.add_argument("--through", type=int, required=True)
    parser.add_argument("--expected-count", type=int, required=True)
    parser.add_argument("--canonical-sha", required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    if not re.fullmatch(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+", args.repo):
        parser.error("invalid repository name")
    if not re.fullmatch(r"[0-9a-f]{40}", args.canonical_sha):
        parser.error("canonical SHA must be full lowercase hex")
    if args.out.exists():
        parser.error("output directory must not already exist; preserve previous captures")
    started = datetime.now(timezone.utc).isoformat()
    pulls = pages(f"/repos/{args.repo}/pulls?state=all&sort=created&direction=asc")
    numbers = sorted(p["number"] for p in pulls if p["number"] <= args.through)
    if len(set(numbers)) != len(numbers) or len(numbers) != args.expected_count:
        raise ValueError(f"inventory mismatch: {len(numbers)} entries, expected {args.expected_count}")
    args.out.mkdir(parents=True)
    with ThreadPoolExecutor(max_workers=4) as pool:
        rows = sorted(pool.map(lambda n: collect_one(args.repo, n, args.out), numbers), key=lambda r: r["number"])
    commit = get_json(f"/repos/{args.repo}/git/commits/{args.canonical_sha}")
    cargo = tomllib.loads(subprocess.check_output(["git", "show", args.canonical_sha + ":Cargo.toml"], text=True))
    lock = tomllib.loads(subprocess.check_output(["git", "show", args.canonical_sha + ":Cargo.lock"], text=True))
    packages = lock.get("package", [])
    members = cargo["workspace"]["members"]
    if any("*" in m for m in members):
        raise ValueError("expand workspace globs before claiming a measured crate count")
    manifest = {"schema_version": 1, "repository": args.repo, "through_pr": args.through, "expected_pr_count": args.expected_count,
                "capture_started_at": started, "capture_finished_at": datetime.now(timezone.utc).isoformat(),
                "canonical_sha": args.canonical_sha, "canonical_tree_sha": commit["tree"]["sha"],
                "workspace_members": members, "workspace_member_count": len(members), "lockfile_package_count": len(packages),
                "external_lockfile_package_count": sum("source" in p for p in packages),
                "python_version": sys.version, "pull_requests": rows,
                "limits": ["Export is evidence collection, not completion of human or AI review.", "Missing file patches are explicitly listed; inspect full blobs/diffs separately.", "PR descriptions/comments and review state can change after capture.", "GitHub is the source of inventory; independent mirrors are a separate requirement."]}
    write_json(args.out / "inventory.json", manifest)
    with (args.out / "canonical-source.tar").open("wb") as target:
        subprocess.run(["git", "archive", "--format=tar", args.canonical_sha], stdout=target, check=True)
    hashes = {}
    for path in sorted(args.out.rglob("*")):
        if path.is_file():
            hashes[str(path.relative_to(args.out))] = hashlib.sha256(path.read_bytes()).hexdigest()
    write_json(args.out / "SHA256SUMS.json", hashes)
    print(json.dumps({"captured_prs": len(rows), "workspace_members": len(members), "external_lockfile_packages": manifest["external_lockfile_package_count"], "canonical_sha": args.canonical_sha}))


if __name__ == "__main__":
    main()
