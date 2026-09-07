# Real-value / real-people release checklist — binary NO-GO gate

This checklist exists to prevent “most things passed” from being mistaken for launch readiness. It is intentionally strict.

## Current verdict

**NO-GO.** As of the repository state reviewed for this pack, the project is not authorized by its own evidence to handle real value or claim production-grade unique-human governance.

A checkbox may be marked complete only by the evidence named beside it. Closing or hiding a GitHub issue is not evidence by itself.

## P0 — must all be complete before real value

- [ ] **Applied cryptography external audit PASS.** Independent cryptographer reviews the current private-value stack against the exact release revision; all Critical/High findings fixed or release blocked.
- [ ] **Shielded-spend chain validity PASS.** Validators independently prove every finalized private-spend key image/nullifier came from a valid private claim; proposer assertion alone is impossible.
- [ ] **Transparent ordinary payment path retired.** Production users are not forced to opt into privacy through a distinguishable second format.
- [ ] **FROST/DKG/custody external audit PASS.** Independent review covers signing, nonces, DKG, complaints/rebuttals, resharing and ceremony failure.
- [ ] **Trusted-dealer custody path unreachable in production.** Demonstrated structurally in the production build/ceremony, not merely labeled prototype.
- [ ] **Legal review PASS for intended launch rails and jurisdictions.** Qualified counsel provides GO or GO-WITH-CONDITIONS that does not require weakening frozen principles.
- [ ] **Independent reproducible release PASS.** Required K independently administered builders reproduce the exact governed release artifact digest.
- [ ] **Dependency/advisory review clean enough for release.** No unresolved Critical/High supply-chain/advisory finding in release dependencies.
- [ ] **Founder-guarded bootstrap sunset.** D-0083 inactive; no sole Founder merge/release path; independent non-Founder human maintainers exist.
- [ ] **No single account/entity can canonicalize a release alone.** Demonstrated by governance/release configuration and a Founder-unavailable drill.

## P0 — must all be complete before real-people governance / Human Share

- [ ] **Unique-human/personhood external validation PASS.** Evidence-qualified identities are shown, under realistic attacks, to satisfy the production uniqueness threshold.
- [ ] **No central verifier/vendor is a required personhood authority.** Any external evidence source is replaceable and cannot unilaterally mint governance identities.
- [ ] **KEL first-contact freshness/duplicity assurance complete for authority paths.** Receipt collection, persistence/gossip/rotation as required, and actual authorization call sites fail closed below assurance threshold.
- [ ] **Hardware presence matrix completed.** T1–T6 and relevant W1–W7 tests run on real devices; trust weights are calibrated to results.
- [ ] **False-positive / false-negative / accessibility review complete.** Unique-human mechanism does not turn weak hardware, disability, geography, or low connectivity into de facto disenfranchisement without a viable alternative path.

## P0 — must all be complete before production storage/provider rewards

- [ ] **Storage proof external/adversarial review PASS.** Capacity, PoRep/PoST, Merkle proofs, challenge randomness and fraud evidence survive independent attack.
- [ ] **Independent-operator diversity solved.** One warehouse/operator controlling many DIDs cannot cheaply satisfy diversity/replication claims intended to decentralize the fabric.
- [ ] **F5 provider anti-collusion FAILs resolved.** Colluding genuine-delivery drain and adaptive audit grinding no longer pass the attacker’s objective.
- [ ] **`phase3_authorized` or equivalent production-enable flag remains false until specialist sign-off.** No bypass.

## P0 — consensus and network release conditions

- [ ] **Independent BFT safety/liveness review PASS.** Round/lock/view-change logic is model/property tested under Byzantine scheduling.
- [ ] **State-sync/catch-up review PASS.** No adversarial peer can cause acceptance of unauthenticated/corrupt canonical state.
- [ ] **Validator authentication/discovery is production-wired.** Not merely available as an optional helper while default construction stays anonymous/unauthenticated.
- [ ] **DoS/resource-bound testing PASS.** Queues, messages, state sync, proof verification and peer churn are bounded on target hardware.
- [ ] **Network partition/rejoin drill PASS.** Honest nodes converge to canonical state without alternative monetary truth.

## P1 — required before broad public production claims

- [ ] Weak-device benchmark suite published for CPU/RAM/storage/battery/bandwidth.
- [ ] Android/iOS/desktop acceptance on supported release targets complete.
- [ ] Real NAT/relay/rendezvous deployment tested under failure and censorship conditions.
- [ ] Search/index/federation scale and abuse behavior measured.
- [ ] Onion/mix metadata-privacy claims independently measured; no unmeasured anonymity number advertised.
- [ ] Long-horizon economic specialist report covers 10/50/100/200-year scenarios, late adopters, dormancy and treasury runway.
- [ ] Edge-removal drills show core still functions when each named convenience provider disappears.

## Release evidence bundle

A real-value release candidate is not complete until one immutable bundle contains:

- release commit SHA and artifact digest;
- constitution/directive registry digest;
- dependency/SBOM/advisory snapshot;
- all independent audit reports and reviewer qualifications;
- finding/disposition matrix;
- hardware raw logs;
- economic simulation inputs/results and specialist report;
- legal opinion summary suitable for repository retention;
- reproducibility attestations from independent builders;
- governance state proving no active single-controller exception;
- exact release/install/rollback evidence;
- named residual risks and next review date.

## Automatic NO-GO conditions

Any one of the following forces NO-GO regardless of other passes:

- active Founder/single-account canonical release authority;
- unresolved Critical/High cryptography/custody/consensus finding;
- unique-human mechanism not independently validated while governance/Human Share is enabled;
- private spend can finalize without validator-verifiable validity;
- trusted-dealer custody reachable in production;
- transparent ordinary payment path remains while “structural privacy” is claimed;
- legal review says the intended launch posture is unlawful unless a frozen invariant is weakened;
- economic/provider mechanism has a known profitable drain/collusion attack and is enabled anyway;
- released binary cannot be independently reproduced from the audited source.

The solution to a failed gate is to fix or narrow the release. It is never to relabel FAIL/PARTIAL as PASS.
