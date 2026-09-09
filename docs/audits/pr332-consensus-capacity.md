# PR #332: consensus validity and storage capacity follow-up

Status: engineering work in progress. F-07 and F-08 remain PARTIAL against the full original audit. This is not production approval, external cryptographic review, governance approval, or a claim that all acceptance tests passed.

## F-07 changes

Every shielded body requires an injected verifier in execution, proposal construction, proposal validation, live commitment, catch-up and state-sync suffix replay. Missing or invalid evidence rejects the whole candidate before installing state. Malformed records reject the entire body; a later valid record cannot resurrect their group.

The verifier now returns opaque ring-member, output and fee effects after checking the full claim proof and exact nullifier group. Execution independently requires every key/commitment ring pair to exist in canonical state and prevents duplicate output keys. Output creation, nullifier consumption and public-fee accounting occur on one candidate state. No value or private-payment dependency was introduced into the consensus crates, and validator votes remain equal per identity root.

An explicit funded shielded genesis is verified against public zero-blinding amount commitments. This is a network configuration surface, not a mint transaction or authority to establish canonical genesis. Shielded pool value participates in supply conservation; fees leave that pool and enter unallocated circulating supply.

Snapshots carry genesis allocations, the canonical output map, and the ordered successful claim digests. Installation binds the configured genesis commitment and replays shielded effects from genesis using independent evidence. Snapshot version is now v3 and state commitment v5: old state cannot silently acquire new validity guarantees. Prelaunch state must be rebuilt from the authorized genesis and complete evidence. There is no approved live-network migration.

## F-08 changes

Low-level declared/possession measurements are `mini_spacetime::ObservedCapacity`. The public lower-layer proposer weighting function was removed. `mini_storage_fraud::ProvenCapacity` has private construction and private aggregation; only checked provider standing reaches the internal scoring formula. Compile-fail examples exercise direct forged construction and the removed low-level weight route.

A proving window now requires all scheduled Merkle responses to verify against the registered replica, with exact response count. A caller can no longer mark a window proven by assertion. Replayed or backward windows are rejected. Provider standing rejects duplicate replica tracking (preventing replay of an older active lifecycle) and mixing provider identity roots. The lifecycle pins its registration-time window policy: weaker challenge counts are rejected and later grace substitution cannot preserve expired capacity. These checks do not prove auditor/operator independence or physical replication uniqueness. Canonical storage-policy state, network-bound finalized window/beacon selection and fresh auditor authority are not yet available here; merely taking a chain argument would not establish these missing facts. F-08 therefore remains PARTIAL. The full `cargo test -p mini-storage-fraud` suite passed after policy pinning, including all 22 lifecycle integration tests and both compile-fail boundary examples.

## Validation and remaining work at checkpoint

Before the canonical-output extension, focused native tests passed: mini-consensus 110; mini-execution 32 unit + 12 end-to-end + 18 ordering; mini-porep 33. The next executable was blocked by Windows Application Control. Those results do not validate the later architecture.

The extended architecture passed the six-crate focused suite: 323 unit/integration tests plus 2 compile-fail documentation tests; 1 illustrative documentation example is ignored. The funded real-claim test exercises canonical membership, execution and snapshot replay. After that run, the explicit genesis/archive node constructors and additional integration-test dependencies were added; `cargo check --tests` passed for execution, consensus, shielded verification and storage fraud, but the focused runtime suite was not repeated after that last constructor change. Validation is recorded in `.validation/consensus-capacity-check.log` and `.validation/consensus-capacity-tests.log`; these local logs are not authority or approval.

Four additional real-cryptography integration tests passed: a genuine payment finalizes under a real quorum certificate and its recipient spends the new output; invented inputs, rebound commitments, copied key images, substituted digests and partial groups cannot enter quorum-backed state. These run through actual node voting and recovery, not a boolean verification fixture.

Canonical window/policy anchoring and weak-device partition evidence remain incomplete for F-08. Evidence gossip, bounded evidence retention, audited shielded issuance/deposit and withdrawal architecture, approved migration, and external cryptography/security review remain production gates. Internal tests are not independent human review.
