# Publication validation record - 2026-09-08

## Scope

| Field | Value |
|---|---|
| Reviewed source recorded by the reports | `684baa103c075299a375d1ed9a3327185c0ba02b` |
| Recorded workspace | 74 members; 612 lockfile package records, 538 with external sources |
| Publication method | Exact-byte transfer of four prepared documents; reconstruction of the evidence map from the pinned original capture; structural coverage and navigation validation |
| Local tools executed for publication | Python 3.13.5; markdown-it-py 4.2.0 for exact HTML reconstruction; Python standard library for integrity/coverage checks |
| Revalidation trigger | Any report, map, manifest or checker modification; changed source requires a separate dated research addendum |

This is an **internal publication-integrity record**, not a new protocol audit, an independent external audit, a human approval or a release decision. It adds no governance weight.

## Checks actually completed during publication

**PASS - exact originals.** The four Markdown/HTML source attachments are unchanged. Byte lengths, SHA-256 and Git blob hashes are in [PUBLICATION_MANIFEST.json](PUBLICATION_MANIFEST.json). The final large-file materialization [run 34251797266](https://github.com/mininet-labs/Mininet/actions/runs/34251797266) compared the complete dossier and HTML against those original hashes, created their content-addressed Git blobs and fetched them back for a full byte comparison. Merely creating blobs did not publish them; attachment to this PR's commit is a separate step.

**PASS - original capture and complete map.** [Run 34252008502](https://github.com/mininet-labs/Mininet/actions/runs/34252008502) regenerated [EVIDENCE_MAP.json](EVIDENCE_MAP.json) from the exact 25,100,909-byte original capture archive, verified its archive SHA-256, checked file-count consistency for every PR and matched the locally reproduced map's SHA-256. The map contains 174 PRs and 2,759 changed-file records. It does not claim missing API patch bytes were inspected.

**PASS - local integrity and structural coverage.** Running `python3 verify_publication.py --self-test` on the publication directory produced:

```json
{"changed_file_records": 2759, "dossiers": 174, "files_verified": 5, "integrity_and_coverage": "PASS", "negative_integrity_tests": "PASS (2)", "security_or_external_audit": "NOT_CLAIMED"}
```

The checker verifies that the 174 unique dossier numbers exactly equal the captured PR inventory, their head/base references and outcomes agree, all nine required PR-specific sections exist, all internal HTML links resolve to unique anchors, and the HTML has no remote script dependency. The tests deliberately truncate a dossier and remove the HTML companion; both must fail. Existence of sections is not proof that their substantive conclusions are correct.

## Historical claims are not new results

The unmodified combined HTML includes the original author's historical claims of 112 repository Python tests and F5-vector, wire-limit, roadmap and dossier-coverage checks. The corresponding original `validation/*.log` files and `REVIEW_COVERAGE.json` were not among the four supplied report attachments. This publication does **not** recreate those logs, independently confirm those historical executions, or present them as fresh passes. The newly recorded publication checks above have their own stated scope.

The earlier [narrow patch-validation run 34204762469](https://github.com/mininet-labs/Mininet/actions/runs/34204762469) and [artifact 10047401960](https://github.com/mininet-labs/Mininet/actions/runs/34204762469/artifacts/10047401960) concern an isolated worktree with the proposed FROST/sequence patch applied to its pinned baseline. They are not evidence that the patch is applied to current source or that all 24 findings have been resolved. No new local Rust execution is claimed by this publication record.

Required PR checks are tied to their exact commit and must be read from GitHub. Earlier green runs do not establish a green result for a later publication commit. Human review and external audit gates remain separate from all of these checks.

## Evidence lifetime and authority

Original capture: [run 34200283626 / artifact 10045573430](https://github.com/mininet-labs/Mininet/actions/runs/34200283626/artifacts/10045573430), captured between 2026-09-08 07:37:51 and 07:39:32 UTC. Archive SHA-256: `4dbb5b4f8f6e7911e5f048be3bbb34b8771503d6702a288854e534e6343774cd`. Recorded expiry: **2026-09-22 07:39:32 UTC**. Actions artifacts are temporary, not independent permanent preservation. The committed map records original per-PR capture digests and filenames but does not contain the entire discussion corpus.

Hashes prove correspondence to recorded bytes, not the truth of GitHub, the review's correctness, unique humanity, independent authorship or institutional independence. The temporary helpers are removed from the final publication tree; they were not used to approve, merge, release, change branch protections or update `main`.
