# PR #332 audit remediation status — 2026-09-09

**Overall: PARTIAL. Real value / real people: NO-GO.**

This is an engineering evidence inventory against the original F-01–F-24 audit, not approval, independent audit, or production authorization. Historical D-0474–D-0501 statements that all findings are closed overstate the evidence. This living account corrects those statements without rewriting history. PASS below applies only to the named local defect; PARTIAL preserves unmet original acceptance criteria. External gates cannot be closed with internal code or checklists.

## Claim → executor → evidence and remaining gate

| Finding | Status | Enforcement and evidence | Unmet acceptance / production boundary |
| --- | --- | --- | --- |
| F-01 | PARTIAL | `mini-treasury` consuming signing nonces and durable signer reservation/burn journal; restart, binding and secret-redaction tests | Independent cryptographic review and physical crash/rollback campaign; retained external monotonic state |
| F-02 | PASS (local defect) | FROST share verification validates exact participant sets before indexing; missing/substituted participant tests | External FROST composition review remains required |
| F-03 | PASS (local defect) | Canonical scalar decoding rejects aliases, malformed lengths and out-of-range encodings | External cryptographic review remains required |
| F-04 | PARTIAL | CLI sequence allocation locks, checks authenticated store floor, durably reserves author-bound checksummed head before mirror | Whole-state rollback, index loss and physical power-loss recovery evidence |
| F-05 | PARTIAL | Persistent witness v2 commits exact signed receipt/history before acknowledgement; exclusive writer and poisoned-error recovery | Reviewed legacy/key-epoch importer and full fault-injection campaign |
| F-06 | PARTIAL | Witness certification history survives reopen and refuses competing successors through both certification and observation | Mandatory transition continuity in all authority consumers; migration and external history retention |
| F-07 | PARTIAL | Execution rejects shielded bodies without verifier; canonical key/commitment membership, atomic effects, genesis-bound replay; actual QC payment and legitimate re-spend plus Byzantine tests | Evidence gossip/retention, issuance/withdrawal architecture, approved migration and independent crypto review |
| F-08 | PARTIAL | Opaque capacity in storage-fraud; raw observations cannot call weight formula; checked scheduled proofs and duplicate-provider/replica denial | Canonical policy/window anchoring, independent audit evidence, partition/weak-device liveness |
| F-09 | PARTIAL | Signed intake/source link; authenticated journal recovery; shared advance/publish lock and commit-before-publication ordering | Workflow labels are not human/governance approval; broader power-loss and semantic review evidence |
| F-10 | PASS (local defect) | Shared DID signature limits and wire-limit scanner; boundary and scanner mutation tests | Broader protocol composition remains subject to review |
| F-11 | PASS (local defect) | Dependency-report schema/count/exit validation and wrapper fixtures; separate deny gate retained | Current dependencies must continue passing CI |
| F-12 | PARTIAL | Replay process lock/reload, checksummed complete lines, durable time floor, malformed-complete-tail rejection | Full disk-full/truncation/platform campaign and independent rollback checkpoint |
| F-13 | PARTIAL | Opaque ownership evidence from authenticated exact object; verifier-issued holder challenge bound to operation/session; replay/revocation tests | Trusted freshness and canonical object selection remain deployment obligations |
| F-14 | PARTIAL | TCP mesh requires signed admission bound to encrypted session, network and exact fixed validator set; checks expected peer roots and mesh/node consistency | Dynamic membership/KEL freshness and full active-interception/revocation campaign remain open |
| F-15 | PARTIAL | Authenticated send and publication dispatch enforce selected transport before sending; no-send and real TCP onion submission tests | Receipt proves local submission only; traffic-analysis/relay-collusion measurements remain open |
| F-16 | PARTIAL | Bounded manual archive extraction, file/directory barriers, retained previous tree and shared maintenance lease; Linux backup CI and process-death tests | Physical power-loss, full disk-full/platform campaign and reviewed interrupted-restore recovery |
| F-17 | FAIL (production gate) | Founder-guarded phase remains explicit | Independently evidenced succession/custody, founder-absence and hostile-host release exercise |
| F-18 | FAIL (production gate) | Failed F5 economic/resource gates retained | External mechanism review and adversarial population/economic campaigns |
| F-19 | PARTIAL | Personhood and operator-independence limitations retained | Collusion/accessibility/privacy/recovery pilots and external evidence |
| F-20 | PARTIAL | Campaign/identity-bound BLAKE3 reservation records; durable Pending → Authorized → Submitted → Finalized journal; retries reuse exact claim; actual finalized-ledger reconciliation tests | Threshold signing/settlement suite integration, whole-state rollback, comprehensive transition crash campaign |
| F-21 | PARTIAL | Verified source/score metadata sealed against mutation; hostile score ranking and disagreement tests | Independent source/canonical-link truth and query-correlation measurements |
| F-22 | PARTIAL | Snapshot final state bound to verified QC; wrong chunk-tree attack refused on reconstructed state | Interrupted durable resume, multi-peer/suffix composition and measured weak-hardware memory/disk/work |
| F-23 | PARTIAL | Primitive/suite support and limits recorded | Pre-break anchors, compromised-classical-key migration, witness loss and independent recovery architecture review |
| F-24 | PARTIAL | Structural governance checks and this explicit evidence inventory | Human semantic review of the exact resulting change and claimed closures |

## Validation

The pushed checkpoint `f2acb2b71f4994893eed96c932033aa465b2ec78` passed Linux backup/restore, canonical governance, Android, reproducibility and dependency workflows. Its normal CI failed an unused import (fixed in the follow-up). CodeQL reported 42 high findings flowing from `trusted_head` into CLI output; remediation and a new scan are required. The earlier SHA-256 temporary-buffer annotation was inspected directly; previous guesses about test nonce call-site shapes are not evidence of its cause.

Local focused results include 323 consensus/execution/storage unit/integration tests and two compile-fail tests; four additional real-QC shielded tests; airdrop/treasury payout tests; three native durable-file tests and twelve witness tests. Later edits require the final integration run. Checkpoint `f66bae4` passed the complete GitHub CI test and Clippy job, Android, backup/restore, governance, dependency and reproducibility checks. TCP admission subsequently passed focused all-target/all-feature Clippy and six real-socket tests, including wrong-network, unexpected-peer and removed-membership refusals. The remaining CodeQL findings must be resolved and new-head checks must pass before merge-readiness. Test success does not change the external NO-GO gates above.

## Recovery and compatibility

See [durability details](pr332-durability.md), [consensus/capacity details](pr332-consensus-capacity.md), and [authority/privacy details](pr332-authority-privacy.md). Witness v1 lacks the exact receipt/certification history required by v2 and is rejected. Preserve the original key and records for independently reconciled import; never clear state to bypass startup refusal. Sequence mismatches require reconciliation against retained signed history, not counter reset. Replay corruption must not be erased to bypass refusal.

Legacy airdrop filenames cannot establish campaign binding and are rejected; preserve their records and reconcile prior entitlements/payments before importing. Payout journal progression is local progress until exact canonical finalized payment evidence is checked. Restoring all local checkpoints can erase prior knowledge; these journals do not defeat administrator rollback.

Snapshot v3/state commitment v5 require authorized genesis plus complete shielded evidence. No live-network migration is approved. Restore retains the prior tree at `.restore-previous`; after interruption preserve both trees and reconcile before restarting writers. Do not remove a maintenance lock while a writer holds it. Unsupported durability barriers fail closed. No automatic rollout is authorized by this PR.

## Authority and evidence precedence

The separately verified canonical checkout, not this proposal branch, anchors the active founder-guarded phase and activated GOV-AI-050/D-0084 instruction digests. The canonical constitutional register identifies `FOUNDER_DIRECTIVES.md` under D-0090; `docs/governance/00_GOVERNANCE_INDEX.md` locates applicable policy, phase and delegation records. This audit inventory and AI work cannot amend those authorities or mint missing succession, release or economic approval.

`docs/STATUS.md` is the living implementation account and takes precedence over historical Decision Log implementation-status fields. The Decision Log remains append-only history. Source code and adversarial tests substantiate the particular enforcement claim in this matrix; green structural validators only establish the checks they actually execute. The human review of the exact merge candidate supplies semantic review, not independent cryptographic audit. Merge of incremental hardening does not authorize production deployment, canonicalization, release or owner adoption.

## CodeQL source-trace correction — 2026-09-10

The Rust SARIF analysis `1751037176` for `f66bae4` identifies the same source for all 42 alerts: the `trusted_head` variable access in `mini-cli/src/sequence.rs`. CodeQL's `SensitiveVariableAccess` uses the sensitive-name heuristic, whose secret-name expression includes `trusted`. The value here is a callback computing the largest public, signature-verified object sequence, not a secret. Removing identity capture did not remove that name-derived source. The callback is now `signed_sequence_floor`, with a comment distinguishing integrity from confidentiality. The 12 sequence tests pass unchanged. No query, security gate, logging assertion or alert was suppressed; a fresh hosted scan must confirm resolution.
