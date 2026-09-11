# Whole-history founder-vision review — 2026-09-07

**Audit scope**

| Field | Value |
|---|---|
| Reviewed at | Canonical checkpoint `main` @ `0ed3a4e523be195d39296eeca16a187fca6a599b`; PR inventory cutoff #326 |
| Workspace size at that commit | Pending measurement from the pinned Cargo manifests; do not infer a count from older audit prose |
| Method | In-progress documentary and source review of every PR in the inventory, including closed-unmerged proposals. Evidence collection is not review completion. No local Rust execution claimed |
| Tool versions | GitHub repository connector; local Python 3.13.5; capture runtime records its actual version |
| Revalidation trigger | Any changed reviewed PR head, canonical security boundary, founder directive, dependency, release gate, or external evidence; later PRs require an appended campaign, not silent rewriting |

**AI-drafted internal research. Not an independent external audit, approval, or production authorization.**

## Scope and ownership

This is a new, complete numbered campaign, not a replacement for PR #321's selected-milestone reconstruction. The observed GitHub inventory contains **174 PRs through #326**. GitHub shares number space between issues and PRs; gaps in PR numbers are not skipped PRs. Later proposals are outside this bounded snapshot and must be listed separately if brought into scope.

Each PR receives its own research entry: intended contribution, founder-directive linkage, mechanism and boundaries, later corrections, remaining risks, proposed implementation changes, and falsifiable acceptance tests. Author-reported results, directly inspected source, locally executed tests, and external research remain separately labeled. Neither a merge nor an AI-authored review establishes independent human approval.

Planned deliverables: reproducible complete inventory; individual PR dossiers; cross-system findings; a prioritized engineering itinerary; comparisons with published cryptography and publicly available code, including actual licenses and new trust assumptions; and a per-directive PASS / PARTIAL / FAIL verdict.

## Reproducible evidence capture

`tools/audit_history_export.py` captures PR metadata, file patches, commits, issue comments, inline comments, and review submissions through the explicit numeric cutoff. It follows every page, checks file/commit counts, refuses a head that moves during capture, and reports missing patches rather than representing them as inspected. It does not execute PR code and marks every exported PR `review_completed: false`.

The accompanying read-only workflow uploads public repository evidence and a pinned tracked-source archive. It does not push, merge, approve, sign a release, access account secrets, or change any branch protection. Its artifact is raw research input, not a passing audit result.

```sh
python3 tools/audit_history_export.py --repo mininet-labs/Mininet \
  --through 326 --expected-count 174 \
  --canonical-sha 0ed3a4e523be195d39296eeca16a187fca6a599b \
  --out /tmp/mininet-history-evidence
```

A network-enabled checkout containing the pinned commit is required. Set a read-only `GITHUB_TOKEN` for the API rate budget; never put it in a command argument or evidence file. Existing capture directories are rejected to preserve prior evidence. Local source execution is not claimed in an environment without Rust or GitHub network access.
