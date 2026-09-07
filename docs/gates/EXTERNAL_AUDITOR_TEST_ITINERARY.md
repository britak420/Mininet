# External auditor test itinerary — from code-complete claims to real-value authorization

**Purpose:** this is the execution order for independent reviewers. It does not replace any existing gate scope; it joins them into one reproducible itinerary so the project cannot close one narrow review and then overclaim overall readiness.

**Release rule:** a reviewer may mark an individual track PASS, PARTIAL or FAIL. **Real value remains NO-GO until every P0 release track below is PASS and the exact audited revision is still the revision being released.** A later security-relevant code change invalidates the relevant sign-off until the reviewer confirms the delta.

## 0. Freeze the exact revision before testing

Before any review starts:

1. Record repository commit SHA, Rust toolchain, `Cargo.lock` digest, OS/architecture, enabled features and test commands.
2. Record the current `docs/CONSTITUTION_REGISTRY.json` digest and active governance/bootstrap state.
3. Export a dependency/SBOM snapshot and current advisory results.
4. Record all exclusions. “Not reviewed” must never become “passed.”
5. Use `docs/gates/EXTERNAL_REVIEW_RESPONSE_TEMPLATE.md` for the report header and disposition.

**PASS evidence:** exact revision + reproducible environment + reviewer identity/competence/independence + scope/exclusions are all stated.

**FAIL condition:** a report says only “reviewed Mininet” without binding itself to an exact revision and defined scope.

---

## Track A — applied cryptography and private value path — P0

**Source scope:** `docs/gates/crypto-audit-scope.md` plus current private-payment implementation, not only the older crate list in that file.

### A1. Primitive correctness
Review:
- stealth-address derivation/recognition/spend-key recovery;
- linkable ring signature challenge chain, key image construction and malleability;
- Bulletproof generator derivation, range proof and IPA algebra;
- confidential commitment/balance equations;
- domain separation and malformed-point/key handling;
- constant-time/secret-dependent behavior where applicable.

Run the existing adversarial suites, then add independent vectors and differential/property tests.

### A2. Composition correctness
Exercise whole private transfers, including:
- one input / one output;
- multiple inputs / multiple outputs;
- change;
- fees;
- duplicate key images;
- ring-member reordering and duplicate members;
- malformed/tampered commitments and proofs;
- replay across transaction/network/context boundaries;
- decoy selection leakage and repeated-use patterns;
- disclosure/view-key behavior.

### A3. Chain-validity rule — mandatory before PASS
Prove that an honest validator can independently reject a private spend whose key image/nullifier did **not** come from a valid private claim. A proposer-supplied key image that merely becomes canonical is not sufficient.

**PASS:** every value-affecting state transition is independently verifiable from public proof material without revealing the protected amount/owner, and no invalid spend can become finalized solely because a proposer named its key image.

**FAIL:** the current R8 failure remains reachable.

### A4. Transparent-path retirement
Verify there is no ordinary production payment format exposing a stable payer identity, cleartext amount and sequence while the private format coexists.

**PASS:** privacy is the protocol default/only ordinary payment path; any explicitly public disclosure is a deliberate higher-level action, not a second settlement format.

**FAIL:** R3 remains open.

### Closure artifact
Independent dated report under `docs/audits/`, findings fixed/accepted with severity and D-number, and a signed statement that the reviewed revision is suitable (or unsuitable) for real-value cryptographic use.

---

## Track B — treasury/FROST/DKG/custody — P0

**Source scope:** `docs/gates/dkg-audit-scope.md` + FROST signing in `mini-treasury`.

Independently attack:
- nonce reuse and partial nonce leakage across sessions;
- rogue-key participation;
- malformed commitments;
- missing/nonresponsive shares;
- sender equivocation;
- false complaints against honest participants;
- complaint/rebuttal ambiguity;
- replay of round-1/round-2 packages across sessions;
- exclude-and-continue safety;
- resharing with malicious/removed old signers;
- group-public-key preservation after resharing;
- old-share retention and the operational deletion problem;
- crash/restart in each round;
- threshold edge cases `1`, `n`, and minimum quorum;
- serialization/canonical transcript/domain separation.

**Production-specific test:** demonstrate the trusted-dealer prototype path is unreachable in the real production ceremony/build, not merely discouraged by an acknowledgment type.

**PASS:** no party ever holds the whole production secret, malicious participants cannot bias/steal the key or frame honest participants, and the trusted-dealer path is provably excluded from production.

**FAIL:** any single-party key recovery path, nonce-reuse recovery, ambiguous complaint outcome, or production-reachable dealer path.

---

## Track C — unique-human/personhood legitimacy — P0 for real people/governance

This is not a unit-test-only review. The repository itself says an identity root is not a human.

Test the complete adversarial claim:
- one person with many devices/accounts;
- colluding real humans operating fake cohorts;
- account rental/sale;
- social-graph farms with purchased honest edges;
- repeated physical co-presence farms;
- relay/wormhole attacks;
- dormant identities and decay;
- coercion;
- colluding households/venues;
- external credential issuer capture;
- low-connectivity/rural users;
- accessibility and weak-device populations;
- false-positive/false-negative impact.

Run real cohort/field exercises with a documented attacker budget. Measure the cost to manufacture an additional voting-eligible “human,” not merely the score distribution.

**PASS:** an independent specialist can bound false acceptance/duplication at the intended governance/Human Share thresholds under realistic attacks, and the mechanism does not depend on a central verifier or vendor attestation authority.

**FAIL:** governance still counts evidence-qualified identities without a defensible uniqueness bound.

---

## Track D — consensus, execution, state sync and validator networking — P0

Independently model and test:
- Tendermint safety across all round/lock/POLC transitions;
- liveness under crashed/Byzantine proposers;
- equivocation evidence and consequence handling;
- partial connected topologies and churn;
- message delay/reordering/duplication;
- replay and domain separation;
- deterministic body/header/state-root binding;
- state-sync snapshot authenticity, rollback and pruning;
- catch-up from an adversarial peer;
- validator-authenticated channels;
- encrypted peer discovery;
- eclipse/partition attempts;
- bounded queues, memory and CPU DoS;
- restart persistence.

**PASS:** independent model/property testing finds no two-finalized-state safety violation under `<1/3` Byzantine roots, recovery never accepts unauthenticated state, and all value transitions—including private spends—are independently verified by the state machine.

**FAIL:** any conflicting finalization trace, unauthenticated recovery state, or remaining R8 private-spend validity gap.

---

## Track E — identity/KEL witness freshness — P0 for authority-bearing identities

Attack both returning and first-contact verifiers:
- stale but internally valid KEL;
- two conflicting controller branches;
- witness equivocation;
- malicious minority witness set;
- unavailable witnesses;
- witness-set rotation;
- recovery after duplicity;
- receipt replay across generation/identity/event;
- cold start with no cached head;
- partial receipt collection and restart;
- gossip suppression/eclipsing.

**PASS:** authority-bearing call sites fail closed unless the required assurance is present, witness policy derives from the identity’s signed KEL, and no witness becomes an identity controller or global registry.

**FAIL:** witness data exists but no real authorization path consumes its assurance, or first-contact freshness can still be supplied entirely by the claimant.

---

## Track F — storage integrity, capacity and operator independence — P0 for storage economics

Attack:
- Merkle proof shape and wrong-leaf substitution;
- partial storage and regeneration-on-demand;
- repeated challenge grinding;
- duplicate replicas;
- forged capacity proofs;
- fraud-evidence framing;
- erasure-code reconstruction under every tolerated shard-loss pattern;
- one operator controlling many DIDs/replicas;
- correlated geography/network failures;
- churn and self-healing response.

**PASS:** reward-bearing claims prove the actual bytes/capacity/time claimed and the diversity mechanism cannot be satisfied by one operator cheaply masquerading as many independent replicas.

**FAIL:** current “distinct DID” diversity can still be one warehouse/operator.

---

## Track G — hardware/BLE/UWB/Wi-Fi acceptance — P0 for real-device personhood claims

Execute `docs/gates/hardware-test-protocol.md` T1–T6 and the Wi-Fi protocol W1–W7 using the shared CSV log template.

Minimum evidence:
- at least 2 Android + 2 iPhone devices where relevant;
- same-room, wall/floor, pocket/bag, outdoor/crowded environments;
- 20+ trials at 3+ measured distances for ranging baseline;
- at least one real relay drill;
- no-UWB fallback;
- UI-thread/battery/performance observations;
- trust weight classified strong/medium/weak/unusable from measured results.

**PASS:** the final personhood weight is calibrated to physical results and no software-RSSI/RTT path is represented as relay-resistant when it is not.

---

## Track H — tokenomics, incentives and anti-collusion — P0

**Source scope:** `docs/gates/economic-simulation-spec.md`, current Rust economic simulation, and F5 anti-collusion work.

Required independent simulation/model review:
- 10/50/100/200-year issuance;
- late adopters and dormant humans;
- Sybil extraction cost vs. reward;
- whale accumulation;
- storage/relay/provider concentration;
- treasury runway;
- liquidity/oracle shocks;
- colluding genuine-delivery providers;
- audit-target grinding/adaptive behavior;
- provider splitting across identities;
- reward-policy gaming;
- validator-role concentration.

**Hard rule:** any mechanism with `phase3_authorized=false` stays disabled until the specialist reproduces the failure, the design changes, and the corrected model passes the agreed thresholds.

**PASS:** independent report states the tested parameter envelope and attack ROI bounds; no unresolved F5 FAIL remains on any production-enabled subsidy/reward path.

---

## Track I — legal launch posture — P0 for real value

Qualified counsel reviews `docs/gates/legal-review-brief.md` against the **actual current rails** (not stale historical assumptions) and intended jurisdictions.

Must cover securities, money transmission, AML/KYC constraints, sanctions, tax, consumer/public communications, custody/treasury characterization and the no-admin/no-seizure constitutional constraint.

**PASS:** written jurisdiction-specific GO or GO-WITH-CONDITIONS opinion whose conditions do not require violating a frozen invariant.

**FAIL:** required operation in a jurisdiction would demand a protocol backdoor/admin seizure/KYC authority. The remedy is launch-scope restriction, not weakening the core.

---

## Track J — reproducible builds and software supply chain — P0 for release

Independently reproduce the actual release artifact on K independently administered builders/hosts.

Verify:
- pinned source revision and lockfile;
- isolated builder policy;
- dependency provenance/advisories/licenses;
- byte-identical or formally explained reproducibility output;
- release transparency/no rollback/equivocation checks;
- owner-approved install and automatic rollback;
- no hidden network fetch during supposedly isolated build steps.

**PASS:** the same release digest is independently reproduced across the required builders and matches the governed release record.

**FAIL:** only same-runner/two-clean-build agreement exists.

---

## Track K — governance decentralization / founder-removal drill — P0 before claiming no owner

The current Founder-guarded GitHub bootstrap is a known central control point. Test its removal, not its intentions.

Required evidence:
1. D-0083 is sunset and the operating-state record is no longer active.
2. Independent non-Founder human maintainers exist.
3. Critical changes require independent human review; AI remains zero-weight.
4. No bypass actor can push/merge/release alone.
5. The canonical Forge/governance path works without the Founder account.
6. Perform a drill in which the Founder account is treated as unavailable/compromised and prove development, review, release and recovery continue.

**PASS:** no single human/account/entity is required to canonicalize protocol development or releases.

**FAIL:** `@mininet-labs` or any successor single account remains the sole critical-path authority.

---

# Final release ceremony

Only after all P0 tracks are PASS:

1. Freeze the release candidate SHA.
2. Re-run all security, integration and hardware suites against that SHA.
3. Reproduce artifacts independently.
4. Attach all external reports with reviewer qualifications and conflict/independence statements.
5. Create one release-gate matrix listing every finding and disposition.
6. Require explicit human release decision under the non-Founder-dependent governance process.
7. Keep every FAIL/PARTIAL visible; no “accepted risk” may be rewritten as PASS.

**Real-value GO criterion:** zero unresolved Critical/High findings in value/custody/consensus/personhood, all production P0 gates PASS, no active single-controller bootstrap exception, and the released binary digest matches the externally reviewed source revision.
