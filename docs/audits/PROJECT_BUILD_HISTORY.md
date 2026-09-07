# Mininet build history — auditor reconstruction

**Purpose:** give an external reviewer a single chronological account of how the repository reached its current state, why major choices were made, what evidence landed with each phase, what was later superseded, and what remains unproved.

**Evidence rule:** this document is reconstructed from the GitHub repository, merged pull requests, `docs/DECISION_LOG.md`, `docs/FAILURE_BOOK.md`, `docs/STATUS.md`, `docs/ROADMAP_TO_RELEASE.md`, and the gate packages under `docs/gates/`. It is an index, not a substitute for the linked source. A test passing, a Founder review, or an AI review never counts as independent external sign-off.

## 1. Foundational phase — identity, equal-weight finality, storage receipts, networking

The first merged implementation established the repository's permanent direction: a custom Rust stack, self-sovereign `did:mini` identities, equal-weight finality over distinct identity roots, local storage proofs, and Mininet-owned networking primitives rather than stake-weighted or vendor-owned control planes.

- [PR #1](../../pull/1) created `mini-chain`, with finality defined by `>2/3` distinct validator roots and no balance/stake weight field. **Reason:** preserve the voice/value wall. **Limit that remains:** a distinct identity root is not yet proof of a distinct human.
- [PR #2](../../pull/2) added storage serve receipts and `mini-net` Kademlia-style routing/gossip. **Reason:** make infrastructure contribution verifiable without introducing a central service. **Limit:** a receipt proves an event, not long-term unique storage or independent operator ownership.
- [PR #3](../../pull/3) reconciled the early whitepaper with the Rust implementation, added UWB/presence direction, personhood fusion scaffolding, storage proof scaffolding, treasury/value scaffolding, and explicit fail-closed crypto seams. It also recorded the production external-audit requirement for the security-critical domains.

This phase already established a pattern that remains important to auditors: deterministic/bookkeeping halves could ship before the research-grade cryptographic halves, but the latter could not be represented as production-safe merely because tests existed.

## 2. Cryptography prototype phase — privacy, Bulletproofs, FROST, DKG

- [PR #4](../../pull/4) implemented the first real stealth-address and linkable ring-signature prototypes after an explicit Founder override allowing AI authorship. The override did **not** eliminate the external-audit production gate.
- [PR #5](../../pull/5) added Bulletproof range proofs, the first FROST custody implementation, a personhood evidence-status model, and proof-of-possession-over-time logic. This was the first point where several security-sensitive constructions existed as executable code.
- [PR #7](../../pull/7) made the audit boundary explicit in repository governance: D-0047 states that production use of value, treasury, consensus, and personhood-sensitive cryptography requires external review. The same PR added the Failure Book, threat/invariant audits, roadmap issues, and dependency/reproducibility checks.
- [PR #95](../../pull/95) replaced the trusted-dealer FROST bootstrap path with a Pedersen/Feldman DKG and resharing flow, with complaint/rebuttal handling and explicit unaudited markers.

**Auditor conclusion for this phase:** executable cryptography exists, but its history itself says external review is a release gate. No internal review later in the history silently cancels that requirement.

## 3. Canonical settlement and self-hosted forge spine

- [PR #100](../../pull/100) introduced canonical settlement execution backed by finalized chain state and proved real TCP bootstrap/sync composition.
- [PR #101](../../pull/101) added `mini-porep`, erasure coding, and the first self-hosted `mini` CLI. It also recorded the concern that implementation breadth was running ahead of vertical integration.
- [PRs #103–#110](../../pull/103) built the self-hosted development spine in stages: build provenance, an isolated Wasmtime build runner, TUF-inspired release verification, explicit-owner installation, an end-to-end forge/release/install/rollback test, a persistent installer event log, and CLI access to the complete path.
- [PR #115](../../pull/115) added a no-GitHub outage demonstration, proving the developer lifecycle can complete without GitHub being a runtime dependency.

**Reason:** a project claiming no owner and no off switch cannot depend on GitHub for its long-term software lifecycle. **Residual limit:** repository governance during bootstrap is still centralized on GitHub today; the runtime forge architecture and the current repository authority are separate facts and must not be conflated.

## 4. Networked consensus and recovery

- [PR #114](../../pull/114) added multi-round Tendermint-style consensus over real sockets, including view change and signed proposals.
- [PR #116](../../pull/116) added equivocation evidence and re-gossip across partial connected topologies.
- [PR #120](../../pull/120) encrypted consensus links with the existing `mini_bearer::Channel` construction.
- [PRs #124–#130](../../pull/124) hardened deterministic timestamps/fee arithmetic, added KEL freshness pins, local discovery/PEX, and state sync/catch-up.
- [PR #289](../../pull/289) materially strengthened state sync with authenticated snapshots, persistent recovery and bounded pruning.
- [PR #300](../../pull/300) bound finality to exact block-body roots, removing ambiguity between a finalized header and the actual executed body.
- [PRs #318–#319](../../pull/318) later added encrypted PEX discovery and an opt-in validator-authenticated channel bound to a delegated `VOTE` key.

**Exact remaining failure point:** `docs/ROADMAP_TO_RELEASE.md` still marks consensus R8 active because the canonical chain can finalize a shielded-spend key image on proposer say-so without independently verifying that a valid private claim produced it. The discovery/authentication/catch-up primitives are also not automatically wired into the default mesh constructor.

## 5. Identity freshness and witness assurance

- [PR #125](../../pull/125) added `FreshnessPins`, which closes stale-KEL replay only for a verifier that has already seen a fresher log.
- [PR #149](../../pull/149) fixed the first-contact problem at the design level: witness receipts, duplicity evidence and gossip, without introducing a global identity registry.
- [PRs #180, #187, #191](../../pull/180) implemented receipt types, witness state, duplicity proofs and `KelAssurance`.
- D-0459 later corrected a critical trust-model flaw: witness policy may not be caller-supplied; it must come from the identity's own signed KEL.
- [PR #320](../../pull/320) adds the next receipt-collection protocol slice.

**Exact remaining failure point:** R9 remains active. Receipt collection transport, gossip, persistence, rotation, and a real authority decision that actually gates on an assurance level are not all complete.

## 6. Personhood — the project’s primary unresolved security dependency

The repository deliberately corrected its language over time:

- Early code counted verified identity roots, not humans.
- [PR #123](../../pull/123) renamed `FullHuman` to `EvidenceQualifiedHuman` because the original name exceeded what the evidence proved.
- [PRs #140 and #220](../../pull/140) kept the evidence taxonomy separate from a unique-human credential and documented the research gap instead of inventing a false solution.

The current repository explicitly states that one-human-one-vote, Human Share distribution and personhood-rooted validator formation cannot be represented as production-secure until unique-human resistance is independently demonstrated. This is not ordinary engineering debt; it is a release-legitimacy dependency.

## 7. Storage — possession became real; operator independence did not

The progression was:

1. signed storage serve receipts (#2),
2. possession-over-time (#5),
3. `mini-porep` + erasure coding (#101),
4. externally-discovered MDS correction (#106),
5. replica/capacity/fraud hardening (#297–#306).

Important corrections happened because adversarial review found real defects: an unsound collision-evidence design was abandoned/rebuilt; the PoRep sampling path was tightened; Merkle proof shape was corrected; a challenge-verification bug that let one path answer unrelated leaves was fixed; `ProvenCapacity` became mandatory.

**Exact remaining failure point:** proof of capacity/possession does not prove independent human/operator control of replicas. A warehouse controlling many DIDs can still satisfy distinct-DID placement. That is why the top of `docs/INVARIANTS.md` still warns that replication uniqueness / operator diversity is unresolved.

## 8. Privacy and value path

Privacy moved from prototypes to composition:

- ring signatures + stealth addressing (#4),
- Bulletproofs (#5),
- object-envelope privacy/capabilities (#141/#143),
- relay/privacy policy (#131/#145/#146),
- private index/bridge seams (#147/#150/#151),
- three-hop onion transport and anti-eclipse composition (#296),
- private payment composition (#305),
- private view-key/disclosure work (#308),
- multi-output private fees/change and chain-backed private ledger (D-0455/D-0457).

The project then discovered a protocol-level privacy problem rather than treating privacy as opt-in UX: `docs/ROADMAP_TO_RELEASE.md` R3 says the still-existing transparent payment path exposes stable payer key, amount and sequence. If transparent and private formats coexist, choosing privacy is itself a signal.

**Exact remaining failure points:** R3 is ready but not done; the transparent format remains. The private construction remains externally unaudited. Public fee and input/output counts remain fingerprints. The chain-verifiable shielded-spend validity rule is still open under R8.

## 9. Economics and anti-collusion

The economic design was repeatedly corrected rather than frozen around early assumptions:

- D-0073/D-0074/D-0075 separated bridge/reserve policy, issuance ceilings and human-continuity evidence.
- The first tokenomics sweep (#108) was explicitly recognized as using a wrong balance-proportional approximation for a per-human distribution and therefore over-reporting some concentration effects.
- [PR #271](../../pull/271) later introduced the day-0 monetary kernel and corrected simulation framing.
- [PR #285](../../pull/285) is especially important: the F5 anti-collusion model recorded explicit **FAIL** outcomes, including colluding genuine-delivery drain and adaptive audit grinding, and left `phase3_authorized=false`.

**Exact remaining failure point:** protocol-subsidized provider settlement must not be enabled until the F5 failures are resolved and independently reviewed. Passing ordinary unit tests cannot convert an explicitly failed mechanism model into a production-safe mechanism.

## 10. Search, social, intake and client surfaces

The application layer was built as separate typed, auditable slices:

- social/public account and wall mechanics were hardened in #154 and expanded into Windows social/UI work in #170;
- MiniSearch types/crawler/extract/index/rank/federation arrived across #160, #162, #172, #252, #256–#283;
- native intake uses explicit review/authority states (#153/#159/#242), so imported bytes do not gain project authority merely by being parsed;
- Android foundation, Keystore custody, enrollment, LAN/QR and BLE seams arrived across #179, #206, #209–#216 and later device work.

**Exact remaining failure points:** real two-device hardware acceptance is not complete; BLE/UWB requires the external hardware matrix; Internet-wide discovery/NAT/relay operations, production messaging/ratchets/calls, full search runtime and some weakest-device measurements remain outside the demonstrated beta slices.

## 11. Governance and the centralization red flag

The repository has strong structural rules: AI has zero approval weight; money has no vote-weight path; constitutional/instruction surfaces are protected; reproducible build/release evidence exists; a future self-hosted forge is implemented.

But GitHub bootstrap authority is currently centralized:

- D-0083 is still `status: active` in `governance/bootstrap-operating-state.json`, expiring 2026-10-12 unless an earlier truthful sunset trigger is recorded.
- `.github/CODEOWNERS` routes all protected/security-critical paths to `@mininet-labs` and says the active ruleset requires zero independent approvals during the exception.
- the operating-state file records `independent_non_founder_human_maintainers: 0`, `production_release_candidate: false`, and `forge_canonical: false`.

This is an **automatic red flag** under Mininet's own Directive 2. The repository does not hide it, but it is still a current central control point and must be removed before Mininet can claim that its development governance has escaped founder control.

## 12. Current release picture

The current roadmap correctly separates engineering progress from outside legitimacy:

- CI baseline and many protocol primitives are real and tested.
- The private value path is substantially more complete than it was.
- Consensus, identity witness assurance, storage, forge/release and clients have real implementations rather than paper-only designs.

But “ready for real value and real people” is **not** established. External cryptography review, legal review, unique-human validation, hardware acceptance, tokenomics/mechanism review, storage operator-independence, private-spend chain validity, the transparent-payment retirement, and bootstrap-governance decentralization remain open.

The next documents in this pack convert those facts into a reviewer-facing evidence index, a gate closure matrix and an executable external-audit itinerary.
