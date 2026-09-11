# Mininet: founder vision, whole-history review and production verdict

**Scope and evidence boundary**

| Field | Recorded value |
|---|---|
| Canonical source | `684baa103c075299a375d1ed9a3327185c0ba02b` |
| PR scope | All 174 PRs numbered at or below #326: 166 merged, 8 closed-unmerged. PR numbers also include issues; numeric gaps are not missing PRs. |
| Evidence capture | 2026-09-08 07:37:51-07:39:32 UTC; GitHub run 34200283626, artifact 10045573430 |
| Measured tree | 74 workspace members; 612 lockfile package records, of which 538 carry external sources. Counts are not unique dependency-name counts. |
| Method | Static, risk-focused review of all 174 PRs, their captured discussions and changed-file evidence; targeted current/historical source inspection. Author-reported test results are not reruns by this reviewer. |
| Tools | Git 2.47.3; Python 3.13.5; GitHub repository connector; primary-source web research where explicitly cited. No local Rust toolchain run. |
| Revalidation | Changed PR head, canonical commit, dependency, trust boundary, governance state or audit evidence requires a dated addendum. |

**AI-authored internal engineering research, not an independent external audit or human approval.** Coverage means each PR has an individual substantive analysis. It does not mean every historical line, force-pushed revision, build artifact, or execution path has received a complete security audit. Examples below are attack or acceptance-test designs unless the validation record explicitly records an executed result.

## Overall judgment

**NO: the reviewed tree does not yet establish a free internet for humanity suitable for production trust, real value and verified-human governance.** This judgment is about the whole production claim, not a rejection of the project or of the useful code it already contains.

The implementation materially advances the founder's vision: local identity, owner-chosen updates, peer-to-peer software distribution, bounded storage/query facilities, private payment primitives and explicit adverse-result records all matter. But a founder-independent, human-rooted and structurally private network is not established while current governance remains founder-guarded, private spend validity is incomplete, security-critical state can be acknowledged without durable commitment, and unique-human/operator independence and external gates remain unproved.

**The exhaustive PR-documentary coverage is complete for the fixed inventory: 174 of 174 PRs have individual analyses, improvement recommendations, a concrete example and acceptance-test proposals.** Eight closed-unmerged proposals are included, not discarded. The report distinguishes them from merged code and traces supersession instead of counting the same feature twice. It includes late merge states for #325 and #326. PR #327 is the review vehicle, not recursively part of its own reviewed inventory.

This is not a claim of complete line-by-line auditing of all historic revisions. The source baseline, raw discussion evidence and the exact boundaries of the review are preserved so an external specialist can continue without mistaking this AI review for independent sign-off.

## What the founder's idea demands

The canonical source gives eighteen directives, not a generic blockchain scorecard. Their common purpose is participant sovereignty: people should communicate, preserve content, transact and govern without an owner whose disappearance or capture can end the network. The voice/value wall is central, but cannot be reduced to a Cargo dependency graph. Privacy is central, but cannot be reduced to encrypting a socket. Human equality is central, but cannot be reduced to counting key pairs.

A useful review therefore asks four different questions of each contribution: what new behavior exists; what evidence verifies it; who can still lie, withhold or override it; and what happens when the relevant device, institution or operator disappears. An implementation can advance the idea while its broad product claim remains PARTIAL. A narrow helper may pass its tests while being unsafe as the sole authority-bearing boundary.

## The actual build story

### Before PR #1: the repository was not empty

The parent of the first reviewed PR already contains identity, cryptographic, object/storage, synchronization and Forge-related foundations. The PR inventory is not the same as the entire commit history. This report does not attribute all initial code to PR #1, and it does not invent design deliberations absent from GitHub. Direct commits and pre-repository discussions are outside the per-PR dossier inventory; the Git bundle preserves the available source history for further archaeology.

### #1-#100: executable roots, then the value/finality boundary

#1 introduced equal-root finality verification; #2 added storage receipts and routing; #3 reconciled the wider prototype; #4 and #5 implemented private-value and custody primitives under explicit audit gates; #6 made networking real; #7 and #94 made invariants, adverse findings and review boundaries visible. #95 added settlement and DKG/resharing, and #100 gave settlement a chain-backed execution path. These are concrete contributions, but distinct roots never became proof of distinct humans and signing/receipt validity never became proof of independent parties.

### #101-#130: a network that can build and recover itself

PoRep/erasure work and the CLI were followed by provenance and isolated builds (#103), release verification (#104), explicit-owner installation (#105), the MDS correction and network sync (#106), the Git export/economic direction (#107), integration tests (#109-#113), real multi-round consensus (#114), and the no-GitHub lifecycle (#115). Subsequent PRs added evidence/gossip, governance bootstrap, timestamp/replay/fee hardening, identity freshness, discovery and catch-up. The recurring lesson is that composition tests uncover assumptions that unit tests of isolated primitives do not.

### #131-#195: privacy policy, applications and witnessed identity

Privacy/cost doctrine was split into explicit types and then consolidated rather than treating every abandoned lane as independently shipped. Relay, bridge, private-index and post-quantum work narrowed dependencies; native intake and public commons separated evidence from authority; desktop/social and search foundations made user-facing paths possible. Witness receipts/state/assurance and local chronological indexing arrived in stages. The danger in this phase is completion language getting ahead of callers: a policy, receipt type or assurance helper is not yet the runtime that enforces it.

### #206-#270: devices, edge services and usable contribution workflows

The Android sequence distinguishes Rust state, FFI, encrypted storage, Kotlin compilation and real hardware rather than collapsing them into one milestone. Edge/provider and engagement work protects replaceability, while airdrop and treasury-approval work deliberately stops short of pretending authorization is settled payment. Public commons, protected publishing, extraction/index/ranking and native teams make the network useful. Their next requirement is not more vocabulary alone: it is persistent, bounded, authenticated orchestration with real failure/recovery tests.

### #271-#306: economics and transport meet adversarial reality

The monetary kernel and finality-bound supply/balances corrected earlier economic modeling assumptions. Contribution settlement, search federation, exact release retrieval, bounded workers and chronological filesystem pages extended the real system. F5 recorded failed anti-collusion/resource gates instead of granting itself success. Transport and storage reviews exposed copied-root framing, missing challenge binding, shape ambiguity and exact-body gaps; #299, #300 and #302 repaired important concrete defects. #306 strengthened capacity accounting but did not make a caller-constructible commitment equivalent to verified capacity or independent operation.

### #307-#326: privacy composition and assurance are closer, not finished

Decoy/disclosure work, private conservation (#312), chain-backed ordering (#313), KEL policy binding (#314), validator accountability/discovery/authentication, witness protocol/persistence/gossip/rotation and chunked state transfer all add meaningful capability. However, the money chain still lacks mandatory private-spend validity; the witness service still has durability/certification gaps; anonymous transport is not automatically intended-peer authentication; chunk consistency is not a replacement for final state verification. These precise seams are the highest-value next engineering work.

## Verdict on each canonical directive

Each verdict covers the whole directive, not only its easiest invariant. Source links pin the inspected tree. A PARTIAL is not a production authorization.

### FD-01: Humanity Before Technology - PARTIAL

**Mechanism and failure point.** Wallet-independent commons, owner-controlled updates and explicit prototype limits prioritize people; the complete weak-device and recovery journey is not proven. Evidence: [`crates/mini-commons-policy/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/684baa103c075299a375d1ed9a3327185c0ba02b/crates/mini-commons-policy/src/lib.rs). Detailed findings: F-04, F-16, F-19.

**Best long-term improvement.** Run zero-balance, pseudonymous, low-resource onboarding/recovery journeys. No mandatory wallet, institution-issued identity or data-harvesting enrollment.

### FD-02: Assume Every Central Authority Will Eventually Fail - FAIL

**Mechanism and failure point.** The recorded founder-guarded GitHub exception is active. Self-hosted Forge and peer retrieval reduce platform dependence but do not themselves remove current governance dependence. Evidence: [`governance/bootstrap-operating-state.json`](https://github.com/mininet-labs/Mininet/blob/684baa103c075299a375d1ed9a3327185c0ba02b/governance/bootstrap-operating-state.json). Detailed findings: F-17.

**Best long-term improvement.** Independent human succession, verified rulesets and a founder/GitHub removal drill, with no permanent emergency administrator.

### FD-03: The Network Must Outlive Its Creators - PARTIAL

**Mechanism and failure point.** The no-GitHub demonstration and exact peer release retrieval are real mechanisms. Durable succession and independent long-term canonical history are not established by a same-operator demo. Evidence: [`tools/no_github_outage_demo.sh`](https://github.com/mininet-labs/Mininet/blob/684baa103c075299a375d1ed9a3327185c0ba02b/tools/no_github_outage_demo.sh). Detailed findings: F-17, F-22, F-24.

**Best long-term improvement.** Archive source, review and activation evidence through independently operated stores; prove release and recovery after removal of all present privileged accounts.

### FD-04: Money Is Never Worth More Than Legitimacy - FAIL

**Mechanism and failure point.** The private execution boundary can order unproved nullifier assertions, while custody remains externally gated. Correct ordering is not valid ownership transfer. Evidence: [`crates/mini-execution/src/nullifier.rs`](https://github.com/mininet-labs/Mininet/blob/684baa103c075299a375d1ed9a3327185c0ba02b/crates/mini-execution/src/nullifier.rs). Detailed findings: F-01-F-03, F-07, F-18.

**Best long-term improvement.** Make spend validity mandatory before voting and execution, remove nonce hazards, and independently review the exact composition before any real value.

### FD-05: Canonical Truth Is Sacred - FAIL

**Mechanism and failure point.** Exact-body finality and atomic multi-input ordering are important. They still cannot establish lawful private ownership when the chain accepts key images without claim-validity evidence. Evidence: [`crates/mini-execution/src/nullifier.rs`](https://github.com/mininet-labs/Mininet/blob/684baa103c075299a375d1ed9a3327185c0ba02b/crates/mini-execution/src/nullifier.rs). Detailed findings: F-07, F-22.

**Best long-term improvement.** Verify the complete transition under deterministic rules, preserve exact body/state commitments and fail closed under partitions rather than invent local finality.

### FD-06: Design for Failure, Not Success - PARTIAL

**Mechanism and failure point.** Consensus archives and local indexes contain meaningful recovery mechanisms. Replay, witness, sequence and publication state still have distinct error/restart gaps. Evidence: [`crates/mini-witness-service/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/684baa103c075299a375d1ed9a3327185c0ba02b/crates/mini-witness-service/src/lib.rs). Detailed findings: F-04-F-06, F-09, F-12, F-16.

**Best long-term improvement.** Use transactional prepare/persist/publish boundaries with fault injection and observable recovery. State a power-loss guarantee separately from process-kill atomicity.

### FD-07: Forks Must Remain Free, But Legitimacy Must Be Earned - PARTIAL

**Mechanism and failure point.** CC0 and owner-controlled adoption support copying and refusal. The whole directive also requires legitimate community/history continuity, not yet established for unique humans. Evidence: [`LICENSE`](https://github.com/mininet-labs/Mininet/blob/684baa103c075299a375d1ed9a3327185c0ba02b/LICENSE). Detailed findings: F-17, F-19, F-24.

**Best long-term improvement.** Preserve free source and local adoption while making continuity evidence portable and independently verifiable. Never let a trademark, download majority or repository owner define legitimacy.

### FD-08: The Human Is the Root of Trust - PARTIAL

**Mechanism and failure point.** DID roots, mutual delegation and evidence taxonomy distinguish important predicates. The repository explicitly does not prove unique people or independent operator control. Evidence: [`docs/INVARIANTS.md`](https://github.com/mininet-labs/Mininet/blob/684baa103c075299a375d1ed9a3327185c0ba02b/docs/INVARIANTS.md). Detailed findings: F-19.

**Best long-term improvement.** Validate risk-bounded personhood through privacy-preserving adversarial research and independent pilots; forbid replacing people with an institution-owned identity oracle.

### FD-09: Privacy Is Structural - FAIL

**Mechanism and failure point.** Private payment composition and owner sealing are real improvements. Transparent ordinary payment paths remain, planned protection can be confused with achieved protection, and identity/transport/disclosure composition has unclosed boundaries. Evidence: [`crates/mini-private-payment/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/684baa103c075299a375d1ed9a3327185c0ba02b/crates/mini-private-payment/src/lib.rs). Detailed findings: F-07, F-10, F-13-F-16, F-21.

**Best long-term improvement.** Retire privacy-downgrade formats under an explicit migration, enforce actual execution guarantees, and independently test transaction plus network plus disclosure leakage.

### FD-10: Every Optimization Has a Cost - PARTIAL

**Mechanism and failure point.** The Failure Book and several explicit non-claims are unusually useful. Cost disclosure is not uniform: bounded output can still hide unbounded scan/reassembly or incomplete durability. Evidence: [`docs/FAILURE_BOOK.md`](https://github.com/mininet-labs/Mininet/blob/684baa103c075299a375d1ed9a3327185c0ba02b/docs/FAILURE_BOOK.md). Detailed findings: F-05, F-11, F-22, F-24.

**Best long-term improvement.** Tie every claimed bound and optimization to measured worst-case input/work/state budgets and a precise loss of guarantee; keep failing results visible.

### FD-11: The Weakest Device Matters Most - PARTIAL

**Mechanism and failure point.** Bounded frames, time-index pages, chunk transfer and ARM deployment support the goal. They do not supply real old-phone memory, energy, flash-wear, latency and recovery evidence. Evidence: [`crates/mini-consensus/src/chunked_snapshot.rs`](https://github.com/mininet-labs/Mininet/blob/684baa103c075299a375d1ed9a3327185c0ba02b/crates/mini-consensus/src/chunked_snapshot.rs). Detailed findings: F-08, F-12, F-22.

**Best long-term improvement.** Measure exact end-to-end workloads on representative low-resource devices. Stream and checkpoint large work without shifting authority to a cloud helper.

### FD-12: AI Serves Humanity - PARTIAL

**Mechanism and failure point.** The activated charter says AI evidence has zero approval weight, and canonical instruction checks restrict proposal self-authorization. Independent human/external review is not supplied by the AI-authored corpus itself. Evidence: [`AGENTS.md`](https://github.com/mininet-labs/Mininet/blob/684baa103c075299a375d1ed9a3327185c0ba02b/AGENTS.md). Detailed findings: F-17, F-24.

**Best long-term improvement.** Keep evidence, review, approval, release and adoption separately recorded. Require independent humans for authority and independent specialists for the stated external gates.

### FD-13: Think in Centuries - PARTIAL

**Mechanism and failure point.** Long-horizon economic simulation, crypto agility and explicit migration proposals are foundations. Persistent PQ anchors, long-range history, governance succession and century-scale access are not demonstrated. Evidence: [`docs/design/frontier-personhood-governance-and-consensus-proposals.md`](https://github.com/mininet-labs/Mininet/blob/684baa103c075299a375d1ed9a3327185c0ba02b/docs/design/frontier-personhood-governance-and-consensus-proposals.md). Detailed findings: F-17-F-19, F-22-F-23.

**Best long-term improvement.** Use versioned protocol archives, independently reproducible implementations, precommitted migration/recovery anchors and long-horizon falsification rather than promises of permanent value.

### FD-14: Simplicity Is Security - PARTIAL

**Mechanism and failure point.** The pure-core/adapter separation is useful, but repeated codecs, Merkle implementations, journals and hand-built crypto enlarge the trusted surface. Merely saying no new primitive does not prove safe composition. Evidence: [`Cargo.toml`](https://github.com/mininet-labs/Mininet/blob/684baa103c075299a375d1ed9a3327185c0ba02b/Cargo.toml). Detailed findings: F-01-F-13, F-24.

**Best long-term improvement.** Compare audited replaceable libraries and shared neutral types against in-house maintenance cost. Remove duplicate authority paths; preserve explicit trust and license boundaries.

### FD-15: Build for the Honest Majority, Defend Against the Malicious Minority - PARTIAL

**Mechanism and failure point.** Many adversarial tests and historical corrections are real. Malformed-input panics, forged-capacity construction, collusion and Sybil assumptions still prevent the broad protection claim. Evidence: [`crates/mini-treasury/src/frost_sign.rs`](https://github.com/mininet-labs/Mininet/blob/684baa103c075299a375d1ed9a3327185c0ba02b/crates/mini-treasury/src/frost_sign.rs). Detailed findings: F-02, F-08, F-18-F-19.

**Best long-term improvement.** Construct attacks against real caller paths, retain regression tests and combine protocol verification with independently calibrated economic/personhood limits.

### FD-16: Preserve the Voice/Value Wall at All Costs - PARTIAL

**Mechanism and failure point.** Validator and Forge quorum interfaces count roots rather than stake. That is the narrow structural protection, not proof that wealth cannot buy additional roots, operator concentration or indispensable infrastructure. Evidence: [`crates/mini-chain/src/validator.rs`](https://github.com/mininet-labs/Mininet/blob/684baa103c075299a375d1ed9a3327185c0ba02b/crates/mini-chain/src/validator.rs). Detailed findings: F-07-F-08, F-17-F-19.

**Best long-term improvement.** Keep vote-weight inputs independent of balances and service revenues. Permit deterministic validity verification through an explicitly reviewed boundary; prevent resource ownership becoming eligibility or political authority.

### FD-17: Every Decision Must Survive the Future Child Test - PARTIAL

**Mechanism and failure point.** No preferential governance weight and privacy-first doctrine protect future participants in principle. Unproved personhood, permanent metadata and irreversible launch mistakes would burden later people who never consented. Evidence: [`docs/FOUNDER_DIRECTIVES.md`](https://github.com/mininet-labs/Mininet/blob/684baa103c075299a375d1ed9a3327185c0ba02b/docs/FOUNDER_DIRECTIVES.md). Detailed findings: F-07, F-09, F-17-F-19, F-23.

**Best long-term improvement.** Delay irreversible activation until its evidence exists; preserve late-adopter equality, private participation, migration, recovery and local refusal without inherited institutional privileges.

### FD-18: The Edge May Be Convenient; The Core May Never Depend On It - PARTIAL

**Mechanism and failure point.** Provider declarations, local off-switches and edge leaf crates keep services replaceable in the design. Real refunds, substitution and failure of all providers are not yet an end-to-end demonstrated lifecycle. Evidence: [`crates/mini-provider/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/684baa103c075299a375d1ed9a3327185c0ba02b/crates/mini-provider/src/lib.rs). Detailed findings: F-13, F-16-F-17, F-20-F-21.

**Best long-term improvement.** Test provider disappearance against actual escrow/payment/identity state, with user-controlled substitution. No provider, bank, court, auditor or hosted search service may acquire core authority.

## The most important architectural correction

The project should reject false equivalences before adding features. A signature is not authorization. A valid authorized identity is not a currently admitted validator. An authenticated observation is not independent demand. A capacity declaration is not possession. Possession is not operator diversity. A finalized assertion is not necessarily a valid spend. A successfully parsed review label is not human approval. Two jobs are not two independent maintainers. A role plan is not achieved anonymity.

The long-term solution is to make each transition between these concepts explicit, typed where helpful, checked at the actual authority boundary and demonstrated by adversarial end-to-end tests. Keep the people-facing freedoms constant while improving the implementation beneath them.

## What must not be weakened to ship

Do not introduce stake to cure validator accountability; a centralized personhood issuer to make enrollment easy; a trusted validity signer to avoid private-proof verification; a global activity graph to make anti-collusion simple; a recovery administrator to hide missing owner backups; a mandatory relay/search provider to hide weak-device costs; or a forced-update key to make migration convenient.

Each would turn an engineering gap into precisely the institutional dependency the project exists to avoid. Delaying a release is preferable to granting those powers and hoping future governance removes them.

## Scope corrections to earlier reports

The review does not repeat the earlier broad assertion that the founder-directive canonicalization necessarily contradicts SPEC-00. Their scope and precedence need a precise human-reviewed map. It also does not call closed PR #292 unmerged: its captured metadata says merged even though the old body still describes a scope-only draft. Conversely, #139-#142 and #298 are preserved as closed-unmerged history, with later consolidations credited appropriately.

#297's framing flaw, #302's wrong-leaf verification and #300's exact-body corrections are historical incidents with later remedies; they are not represented as unchanged current defects. New findings identify what remains in the pinned tree, separately from the fact that earlier repairs succeeded.
