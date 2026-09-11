# Exhaustive Mininet PR-by-PR substantive review

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

## How to read this report

Each of the 174 PRs receives an individual verdict for the contribution and its stated boundaries, not a blanket production-security score. A historical FAIL may have been repaired later; its lineage section names the correction. A PASS on a bounded documentation/test change does not mean the whole project is safe. Concrete examples and acceptance tests are proposals unless VALIDATION_RECORD explicitly records execution.

Source URLs pin the captured PR head (or its base for removed files). The complete changed-file list and raw discussion record are retained in EVIDENCE_MAP.json and the accompanying evidence archive. Generated navigation/lockfile contents are not claimed as line-by-line security reviews. Review discussions were considered as evidence and checked against relevant source, not treated as automatic truth.

**Coverage:** 174/174 dossiers; 21 PASS, 145 PARTIAL, 8 FAIL for their individual scopes.

<a id="pr-0001"></a>

## PR #1: Add mini-chain: finality-verification core of the custom BFT chain

**PARTIAL** | Captured outcome: **merged** | Directives: FD-04, FD-05, FD-08, FD-16.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/1) | [Files changed](https://github.com/mininet-labs/Mininet/pull/1/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/ba9c79976962c0537ad69229fe5da84ef7b10135)

Head `ba9c79976962c0537ad69229fe5da84ef7b10135`; base `63e733d42cfe0b22b248ba7e42a60862147059de`; merge `e3189e53aa02389eae9c42d4e10b17e4440cdd55`. 13 changed files; 2 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This is the first pull request, not the beginning of all source development: its base already contains an identity, object, and Forge foundation. It makes equal-root finality executable in mini-chain instead of treating a whitepaper quorum as a running chain. It contributes the crucial separation between a participant's money and the weight of that participant's vote.

**Mechanism and evidence.** ValidatorSet contains identities rather than stake weights. verify_finality requires more than two thirds of distinct validator roots to sign matching precommits; verify_vote checks the device's delegation and VOTE capability. A root is deduplicated even when several of its devices sign. The validator and vote limits also bound part of the verification workload.

**What remains weaker than the intended claim.** This establishes a finality-certificate predicate under a supplied validator set. It does not establish unique humans, how that set was legitimately selected, network liveness, or persistence against signing twice after restart. The original header does not commit the exact body, and the original vote transcript has less domain separation than later versions. A mathematically valid quorum can still certify an invalid application transition unless execution checks that transition independently.

**Recommended improvement and rationale.** Keep the equal-root tally, but require an explicit network/genesis domain, validator-set epoch, historic key state and application-validity check wherever a certificate becomes authority. Treat proposer scheduling as a separate potential concentration channel. Publish the exact Byzantine and synchrony assumptions rather than calling finality unconditional or instantaneous.

**Concrete example.** With four eligible roots, three correctly delegated roots can certify a block. Three devices belonging to one root must count as one, not three. Four roots controlled by one person still satisfy the software's identity distinction; that is not four humans.

**Acceptance tests to implement.** Exercise duplicate-root devices, wrong phase/height/round, revoked devices, wrong network, a certificate over a substituted body, and transitions between validator sets. A restarted signer must refuse a conflicting vote at an already signed slot. Test both valid and invalid application bodies under otherwise authentic certificates.

**History, supersession and integration.** PR #114 adds a networked state machine; #119 adds a vote-signing domain; #300 binds exact block bodies; #316 adds another accountability representation. None retroactively turns #1 into a personhood proof.

**Source entry points.** [`crates/mini-chain/src/block.rs`](https://github.com/mininet-labs/Mininet/blob/ba9c79976962c0537ad69229fe5da84ef7b10135/crates/mini-chain/src/block.rs); [`crates/mini-chain/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/ba9c79976962c0537ad69229fe5da84ef7b10135/crates/mini-chain/src/error.rs); [`crates/mini-chain/src/finality.rs`](https://github.com/mininet-labs/Mininet/blob/ba9c79976962c0537ad69229fe5da84ef7b10135/crates/mini-chain/src/finality.rs); [`crates/mini-chain/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/ba9c79976962c0537ad69229fe5da84ef7b10135/crates/mini-chain/src/lib.rs); [`crates/mini-chain/src/validator.rs`](https://github.com/mininet-labs/Mininet/blob/ba9c79976962c0537ad69229fe5da84ef7b10135/crates/mini-chain/src/validator.rs); [`crates/mini-chain/src/vote.rs`](https://github.com/mininet-labs/Mininet/blob/ba9c79976962c0537ad69229fe5da84ef7b10135/crates/mini-chain/src/vote.rs); [`crates/mini-chain/tests/finality.rs`](https://github.com/mininet-labs/Mininet/blob/ba9c79976962c0537ad69229fe5da84ef7b10135/crates/mini-chain/tests/finality.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0002"></a>

## PR #2: Add mini-storage and mini-net; CodeQL follow-up; D-0034 founder decisions

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-06, FD-09, FD-15, FD-16.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/2) | [Files changed](https://github.com/mininet-labs/Mininet/pull/2/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/7f65e96d21679466a03b717346489b7d7db33fb6)

Head `7f65e96d21679466a03b717346489b7d7db33fb6`; base `e3189e53aa02389eae9c42d4e10b17e4440cdd55`; merge `1b23552ad99709a1d10ba511b3b1aaa7ec68c48f`. 29 changed files; 6 commits; 0 issue comments, 20 inline comments and 2 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This joins two foundational ideas: infrastructure work should leave checkable evidence, and peers should find and relay content without an indispensable service. mini-storage introduces signed serve receipts; mini-net introduces routing and gossip. The useful contribution is an explicit evidence boundary, not proof that a decentralized storage market is already fair.

**Mechanism and evidence.** verify_serve checks two distinct identity roots, delegated ATTEST devices, the signed receipt transcript, nonces and a freshness policy. Routing tables and the bounded GossipRouter move opaque messages with a seen-message cache. Content digests identify the claimed bytes; ordinary signatures identify the claimants.

**What remains weaker than the intended claim.** A receipt is evidence that these keys signed a statement, not that useful demand occurred, that the parties are independent, or that the bytes remain stored. Freshness needs both an age bound and a future-time bound; an optional clock cannot silently become production verification. Bounded seen caches cannot guarantee a message is forwarded only once for all time. The scanner comments in this PR concern logging and deterministic test inputs, and must not be counted as an independent security audit.

**Recommended improvement and rationale.** Separate verified receipt structure from delivery observation and from settlement entitlement. Make durable replay insertion fallible and atomic where it protects rewards. Give routing a threat model for adversarial peer supply and churn, and treat peer identifiers as routing labels rather than personhood. Never expose the two-party receipt graph merely to make fraud accounting convenient.

**Concrete example.** Alice can run both requester and provider under different roots and sign a valid receipt without creating external demand. Likewise, after the seen cache evicts a message, replaying its bytes may cause re-forwarding; the cache bound is a resource trade, not permanent replay prevention.

**Acceptance tests to implement.** Test a future-dated receipt, missing clock context, disk failure while recording each nonce, the same operator using two roots, and replay after cache eviction. Verify a public-content transfer remains usable without a wallet and that malformed receipt input cannot change canonical balances.

**History, supersession and integration.** #6 supplies real TCP; #154 adds the missing future-skew receipt bound; #94 revisits reward qualification; #284/#285 separate voluntary transfers from subsidy extraction. Storage possession and replication work arrives in #5, #101 and #299-#306.

**Source entry points.** [`crates/mini-crypto/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/7f65e96d21679466a03b717346489b7d7db33fb6/crates/mini-crypto/src/lib.rs); [`crates/mini-crypto/src/random.rs`](https://github.com/mininet-labs/Mininet/blob/7f65e96d21679466a03b717346489b7d7db33fb6/crates/mini-crypto/src/random.rs); [`crates/mini-crypto/tests/crypto.rs`](https://github.com/mininet-labs/Mininet/blob/7f65e96d21679466a03b717346489b7d7db33fb6/crates/mini-crypto/tests/crypto.rs); [`crates/mini-net/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/7f65e96d21679466a03b717346489b7d7db33fb6/crates/mini-net/src/error.rs); [`crates/mini-net/src/gossip.rs`](https://github.com/mininet-labs/Mininet/blob/7f65e96d21679466a03b717346489b7d7db33fb6/crates/mini-net/src/gossip.rs); [`crates/mini-net/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/7f65e96d21679466a03b717346489b7d7db33fb6/crates/mini-net/src/lib.rs); [`crates/mini-net/src/peer.rs`](https://github.com/mininet-labs/Mininet/blob/7f65e96d21679466a03b717346489b7d7db33fb6/crates/mini-net/src/peer.rs); [`crates/mini-net/src/routing.rs`](https://github.com/mininet-labs/Mininet/blob/7f65e96d21679466a03b717346489b7d7db33fb6/crates/mini-net/src/routing.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0003"></a>

## PR #3: Whitepaper reconciliation + full D-0034 batch (net, UWB, uniqueness, spacetime, treasury, value)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-01, FD-08, FD-09, FD-10, FD-14, FD-16.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/3) | [Files changed](https://github.com/mininet-labs/Mininet/pull/3/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/d8e53cb3abcc3cb6164935420f2c95b5ae660d06)

Head `d8e53cb3abcc3cb6164935420f2c95b5ae660d06`; base `1b23552ad99709a1d10ba511b3b1aaa7ec68c48f`; merge `40fd89b3b004ad27d0b2800b81dc5ff787427e1d`. 43 changed files; 5 commits; 0 issue comments, 2 inline comments and 1 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This reconciles the early design with the Rust implementation and records where substantial research is still missing. It adds personhood-fusion, storage-proof, treasury and private-value scaffolding while correcting the project direction, including the exclusion of the abandoned LoRa/radio extension. Its strongest contribution is refusing to manufacture production cryptography behind a complete-looking interface.

**Mechanism and evidence.** Unimplemented cryptographic operations fail closed. The social-graph/personhood layer aggregates evidence; storage and treasury types describe intended inputs and accounting; UWB/presence material describes a future stronger physical signal. The PR also establishes authorship/review boundaries that subsequent decisions explicitly amend.

**What remains weaker than the intended claim.** Combining graph evidence, device observations and age does not make their sources independent or truthful. A UWB numeric field is not a hardware-authenticated ranging transcript. Concave storage weighting can discourage concentration only under assumptions about proven resources and identity splitting. This very broad PR also mixes documentation, economic meaning and prototype APIs, so a single green test summary hides several unrelated assurance levels.

**Recommended improvement and rationale.** Preserve each scaffold's refusal behavior until an actual verifier exists. Split evidence quality, eligibility and authority into different types; document who supplies every graph seed, physical observation and economic fact. For every new signal, specify falsification experiments and privacy leakage before adjusting scores. Attribute changed founder decisions to their own records, not to retroactive edits of old specifications.

**Concrete example.** A farm can submit many plausible device observations and mutually supportive graph edges. A weighted score may rise even though all evidence belongs to one operator. The right response is a qualified-evidence result with named assumptions, not an automatically issued unique-human credential.

**Acceptance tests to implement.** Assert every unsupported crypto path returns an explicit error rather than a plausible placeholder. Model colluding graph seeds, repeated devices and correlated observations. Verify that adding evidence sources cannot increase voting weight and that no UWB placeholder is accepted as measured proximity.

**History, supersession and integration.** #4 and #5 implement previously blocked crypto after explicit authorship decisions; #123 corrects the FullHuman name; #220 turns remaining trust questions into research tracks. The standing external-audit gate is not cancelled by these implementation changes.

**Source entry points.** [`crates/mini-keystone/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/d8e53cb3abcc3cb6164935420f2c95b5ae660d06/crates/mini-keystone/src/lib.rs); [`crates/mini-presence/src/attestation.rs`](https://github.com/mininet-labs/Mininet/blob/d8e53cb3abcc3cb6164935420f2c95b5ae660d06/crates/mini-presence/src/attestation.rs); [`crates/mini-presence/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/d8e53cb3abcc3cb6164935420f2c95b5ae660d06/crates/mini-presence/src/error.rs); [`crates/mini-presence/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/d8e53cb3abcc3cb6164935420f2c95b5ae660d06/crates/mini-presence/src/lib.rs); [`crates/mini-presence/src/ranging.rs`](https://github.com/mininet-labs/Mininet/blob/d8e53cb3abcc3cb6164935420f2c95b5ae660d06/crates/mini-presence/src/ranging.rs); [`crates/mini-presence/src/verify.rs`](https://github.com/mininet-labs/Mininet/blob/d8e53cb3abcc3cb6164935420f2c95b5ae660d06/crates/mini-presence/src/verify.rs); [`crates/mini-presence/tests/presence.rs`](https://github.com/mininet-labs/Mininet/blob/d8e53cb3abcc3cb6164935420f2c95b5ae660d06/crates/mini-presence/tests/presence.rs); [`crates/mini-reward/tests/reward.rs`](https://github.com/mininet-labs/Mininet/blob/d8e53cb3abcc3cb6164935420f2c95b5ae660d06/crates/mini-reward/tests/reward.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0004"></a>

## PR #4: mini-value: real stealth-address and ring-signature prototypes (D-0036 override)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-09, FD-12, FD-14, FD-15.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/4) | [Files changed](https://github.com/mininet-labs/Mininet/pull/4/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/44c40d8228e13a0a6d7d06c3c48e23303e7e4e03)

Head `44c40d8228e13a0a6d7d06c3c48e23303e7e4e03`; base `40fd89b3b004ad27d0b2800b81dc5ff787427e1d`; merge `9554279880bf7069fa607da334e071502da49392`. 11 changed files; 2 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This makes stealth addressing and linkable ring signatures executable, following the founder's explicit AI-authorship override. It advances private payment building blocks, not a private payment system. Keeping spend and view capabilities separate is important for later owner-controlled disclosure.

**Mechanism and evidence.** The Ristretto-based implementation derives one-time output keys, builds a ring challenge chain and emits a key image so repeated use of the same signing key can be detected. It relies on an existing curve-arithmetic library while defining its own transcript, hashing and protocol composition.

**What remains weaker than the intended claim.** An established arithmetic library does not audit the surrounding construction. Ring membership proves knowledge of one key only under the construction's assumptions; it does not prove that the key belongs to a funded canonical output. Nor does the initial composition prove amount conservation, choose statistically plausible decoys or hide network timing. The custom domains and hash choices also prevent assuming Monero or RFC wire compatibility.

**Recommended improvement and rationale.** Keep a normative transcript and canonical encoding beside independently generated vectors. Check non-canonical scalars, identity/small-order points, ring duplication, domain separation and secret erasure. Use other implementations as differential references only after mapping their group, hash-to-point, challenge and serialization conventions exactly. Require a separate production audit of both primitive and composition.

**Concrete example.** A valid ring signature over a newly invented set of keys proves the author knows one of those keys. It does not authorize spending an output absent from the ledger. A verifier must authenticate the output set and its commitments, not just verify the signature.

**Acceptance tests to implement.** Add cross-message and cross-network replay tests, reordered-ring cases, duplicate members, malformed scalars, boundary-size rings, malicious output sets and secret-lifetime checks. Conservation tests belong at the transaction layer and must include attempts to spend invented canonical inputs.

**History, supersession and integration.** #5 adds range proofs; #305 composes payments but initially omits conservation; #307 addresses default decoy selection; #312 adds two-column MLSAG conservation. These are new obligations, not evidence the original prototype was already complete.

**Source entry points.** [`crates/mini-value/src/curve.rs`](https://github.com/mininet-labs/Mininet/blob/44c40d8228e13a0a6d7d06c3c48e23303e7e4e03/crates/mini-value/src/curve.rs); [`crates/mini-value/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/44c40d8228e13a0a6d7d06c3c48e23303e7e4e03/crates/mini-value/src/error.rs); [`crates/mini-value/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/44c40d8228e13a0a6d7d06c3c48e23303e7e4e03/crates/mini-value/src/lib.rs); [`crates/mini-value/src/ring.rs`](https://github.com/mininet-labs/Mininet/blob/44c40d8228e13a0a6d7d06c3c48e23303e7e4e03/crates/mini-value/src/ring.rs); [`crates/mini-value/src/ring_impl.rs`](https://github.com/mininet-labs/Mininet/blob/44c40d8228e13a0a6d7d06c3c48e23303e7e4e03/crates/mini-value/src/ring_impl.rs); [`crates/mini-value/src/stealth.rs`](https://github.com/mininet-labs/Mininet/blob/44c40d8228e13a0a6d7d06c3c48e23303e7e4e03/crates/mini-value/src/stealth.rs); [`crates/mini-value/src/stealth_impl.rs`](https://github.com/mininet-labs/Mininet/blob/44c40d8228e13a0a6d7d06c3c48e23303e7e4e03/crates/mini-value/src/stealth_impl.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0005"></a>

## PR #5: D-0037/D-0038/D-0039/D-0040/D-0041: policy generalization, personhood, storage proof, Bulletproofs, FROST custody

**PARTIAL** | Captured outcome: **merged** | Directives: FD-04, FD-08, FD-09, FD-12, FD-15, FD-16.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/5) | [Files changed](https://github.com/mininet-labs/Mininet/pull/5/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/da98e47c0af1da26968bbf101ce345a0d7e470a5)

Head `da98e47c0af1da26968bbf101ce345a0d7e470a5`; base `9554279880bf7069fa607da334e071502da49392`; merge `e2375cc9a5aa08e37c396d8365201c7c1f1f8011`. 32 changed files; 6 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This substantially widens executable prototypes: Bulletproof range proofs, threshold custody, storage possession and evidence-qualified identity status. It is a historical turning point because security-sensitive behavior now exists as code instead of interfaces. It is also where several later corrections originate.

**Mechanism and evidence.** Range proofs constrain amounts to an allowed interval. FROST creates a threshold signing construction, initially with a trusted dealer. Merkle/PDP challenges check possession claims. Human-evidence classification combines source requirements, age and scores. The founder's AI-authoring decision changes who may draft code, not what constitutes external approval.

**What remains weaker than the intended claim.** A range proof alone is not a balance equation. Threaded threshold participants are not administratively independent custodians, and a trusted dealer has seen the complete secret. Evidence classification was named more strongly than unique-human resistance justified. The later storage challenge-index bug demonstrates why honest round trips are insufficient. Current FROST source also warrants nonce-consumption, participant-set and canonical-scalar hardening (findings F-01 through F-03 in this review).

**Recommended improvement and rationale.** Put each primitive behind a narrow verified-state interface, retain a production-disabled status, and attach explicit misuse tests to the public API. Threshold signing needs one-use nonce state with crash-safe consumption, exact signer-set checks and independently reviewed DKG. Personhood vocabulary should describe evidence rather than pronounce humanity.

**Concrete example.** A range proof for an output of 100 is valid even when the payer owned only 1; only an input/output conservation proof rejects that transaction. Similarly, keeping one Merkle leaf and its path defeats a possession verifier that never checks which leaf was challenged.

**Acceptance tests to implement.** Test inflation despite individually valid range proofs; signing-package substitution; reuse of the same FROST nonce pair; an absent signer with a known verifying share; possession responses for another leaf; and one operator supplying every human-evidence source. No result should be promoted from prototype to production by the test count alone.

**History, supersession and integration.** #95 replaces ordinary trusted-dealer setup with DKG; #123 renames FullHuman; #302 binds storage responses to their challenges; #312 adds transaction conservation. Each correction should be linked from the original assurance claim.

**Source entry points.** [`crates/mini-spacetime/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/da98e47c0af1da26968bbf101ce345a0d7e470a5/crates/mini-spacetime/src/lib.rs); [`crates/mini-spacetime/src/merkle.rs`](https://github.com/mininet-labs/Mininet/blob/da98e47c0af1da26968bbf101ce345a0d7e470a5/crates/mini-spacetime/src/merkle.rs); [`crates/mini-spacetime/src/proof.rs`](https://github.com/mininet-labs/Mininet/blob/da98e47c0af1da26968bbf101ce345a0d7e470a5/crates/mini-spacetime/src/proof.rs); [`crates/mini-spacetime/src/storage_proof.rs`](https://github.com/mininet-labs/Mininet/blob/da98e47c0af1da26968bbf101ce345a0d7e470a5/crates/mini-spacetime/src/storage_proof.rs); [`crates/mini-treasury/src/curve.rs`](https://github.com/mininet-labs/Mininet/blob/da98e47c0af1da26968bbf101ce345a0d7e470a5/crates/mini-treasury/src/curve.rs); [`crates/mini-treasury/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/da98e47c0af1da26968bbf101ce345a0d7e470a5/crates/mini-treasury/src/error.rs); [`crates/mini-treasury/src/frost_keygen.rs`](https://github.com/mininet-labs/Mininet/blob/da98e47c0af1da26968bbf101ce345a0d7e470a5/crates/mini-treasury/src/frost_keygen.rs); [`crates/mini-treasury/src/frost_sign.rs`](https://github.com/mininet-labs/Mininet/blob/da98e47c0af1da26968bbf101ce345a0d7e470a5/crates/mini-treasury/src/frost_sign.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 2 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0006"></a>

## PR #6: D-0042: real TCP transport (mini-bearer) + live multi-process gossip demo (mini-net)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-06, FD-11, FD-14.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/6) | [Files changed](https://github.com/mininet-labs/Mininet/pull/6/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/aefdf7453f434e55d75e825c87a7f8ef86cf84e2)

Head `aefdf7453f434e55d75e825c87a7f8ef86cf84e2`; base `e2375cc9a5aa08e37c396d8365201c7c1f1f8011`; merge `a58d7b7d4117480c040b365275c8ecf3f55022b6`. 15 changed files; 1 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This replaces an in-process transport stand-in with real length-delimited TCP and a multi-process gossip demonstration. It is meaningful progress toward independent nodes because operating-system sockets, disconnects and framing now participate in the tests.

**Mechanism and evidence.** TcpBearer implements the existing Bearer contract, caps frames, maintains framing state and exposes blocking/nonblocking receive behavior. The demonstration runs separate processes instead of merely calling both peers in one function. It does not itself change identity or consensus authority.

**What remains weaker than the intended claim.** TCP reachability is not censorship resistance, confidentiality, authenticated identity, NAT traversal or a mobile lifecycle. A bounded frame length does not by itself bound time spent waiting for a partial frame, the number of connections, or aggregate buffers. The demonstration's simple topology is an operational fixture rather than evidence of a hostile Internet deployment.

**Recommended improvement and rationale.** Keep this adapter small and let an authenticated-encryption layer own cryptographic state. Add total deadlines, bounded queues, connection admission and cancellation semantics at the host boundary. Exercise realistic half-close and backpressure behavior rather than treating localhost delivery as the final transport acceptance test.

**Concrete example.** A peer can send a permitted length prefix and then one byte per minute. The allocation cap works, but a server with no total receive deadline can still exhaust its workers. A slow peer must not block unrelated honest peers or force the client to fall back to cleartext.

**Acceptance tests to implement.** Test partial length prefixes, disconnects at every frame boundary, repeated empty frames, slow readers/writers, cancellation and bounded aggregate connections. Observe actual socket traffic when testing confidentiality in later channel integrations; absence of a plaintext substring is supporting evidence, not a full cryptographic proof.

**History, supersession and integration.** #120 wraps consensus links in Channel; #128-#130 add discovery and catch-up; #289 adds archive-backed state sync. BLE-specific framing and platform integration arrive much later and are not implied by this TCP work.

**Source entry points.** [`crates/mini-bearer/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/aefdf7453f434e55d75e825c87a7f8ef86cf84e2/crates/mini-bearer/src/error.rs); [`crates/mini-bearer/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/aefdf7453f434e55d75e825c87a7f8ef86cf84e2/crates/mini-bearer/src/lib.rs); [`crates/mini-bearer/src/tcp.rs`](https://github.com/mininet-labs/Mininet/blob/aefdf7453f434e55d75e825c87a7f8ef86cf84e2/crates/mini-bearer/src/tcp.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0007"></a>

## PR #7: D-0043–D-0048: Founder Directives, Failure Book, master roadmap, CI security, 4 audits, doc restructuring

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-04, FD-05, FD-10, FD-12, FD-15.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/7) | [Files changed](https://github.com/mininet-labs/Mininet/pull/7/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/5bae81be0bf3469ec580ef47658e9b9fac39e986)

Head `5bae81be0bf3469ec580ef47658e9b9fac39e986`; base `a58d7b7d4117480c040b365275c8ecf3f55022b6`; merge `34997f3ecf8e47ff1527a9e1b386d6a8cdefd82a`. 19 changed files; 5 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This converts founder concerns into a visible Failure Book, explicit invariants, audit documents and tracked work. Most importantly, it records that money cannot be CRDT-merged and that neither AI review nor founder review replaces an external cryptography audit.

**Mechanism and evidence.** The new M1-M3 rules separate signed local claims from canonical settlement; A1/D-0047 records the production audit requirement. CI adds dependency and reproducibility checks, while the roadmap makes known gaps discoverable rather than relying on private conversation.

**What remains weaker than the intended claim.** The presence of a rule or a scanner workflow is not enforcement. The history later shows a dependency scanner that could not install yet produced a tolerated green result. Early audit documents cover a smaller tree and cannot justify current whole-workspace assertions. Roadmap issue counts measure organization, not security closure.

**Recommended improvement and rationale.** Make release admission consume exact, scoped evidence and refuse missing audit artifacts. Distinguish scanner execution failure, verified clean results, advisories awaiting disposition and deliberately accepted risk. Every report needs a reviewed SHA, tool versions, scope and revalidation conditions; every closed issue needs an independently checkable exit criterion.

**Concrete example.** An action configured with continue-on-error can be green after the scanner executable failed to install. A future reader may interpret that as no known vulnerabilities. The result must instead say scanner failed and block whichever release criterion requires a completed scan.

**Acceptance tests to implement.** Fault-inject missing scanner binaries, malformed output, stale advisory databases and genuine findings. Test that a local AcceptedLocal payment cannot be rendered Finalized. Re-run audit scope checks after adding crates and verify a stale report cannot satisfy a release gate for a different source tree.

**History, supersession and integration.** #212 first runs cargo-deny and documents the broken scanner; #299 and #307 repair different scanner-control-flow defects; #310/#311 improve roadmap consistency; #321 prepares external-review navigation. None supplies external sign-off.

**Source entry points.** [`.github/pull_request_template.md`](https://github.com/mininet-labs/Mininet/blob/5bae81be0bf3469ec580ef47658e9b9fac39e986/.github/pull_request_template.md); [`.github/workflows/ci.yml`](https://github.com/mininet-labs/Mininet/blob/5bae81be0bf3469ec580ef47658e9b9fac39e986/.github/workflows/ci.yml); [`CONTRIBUTING.md`](https://github.com/mininet-labs/Mininet/blob/5bae81be0bf3469ec580ef47658e9b9fac39e986/CONTRIBUTING.md); [`README.md`](https://github.com/mininet-labs/Mininet/blob/5bae81be0bf3469ec580ef47658e9b9fac39e986/README.md); [`SECURITY.md`](https://github.com/mininet-labs/Mininet/blob/5bae81be0bf3469ec580ef47658e9b9fac39e986/SECURITY.md); [`docs/BETA_STATUS.md`](https://github.com/mininet-labs/Mininet/blob/5bae81be0bf3469ec580ef47658e9b9fac39e986/docs/BETA_STATUS.md); [`docs/DECISION_LOG.md`](https://github.com/mininet-labs/Mininet/blob/5bae81be0bf3469ec580ef47658e9b9fac39e986/docs/DECISION_LOG.md); [`docs/FAILURE_BOOK.md`](https://github.com/mininet-labs/Mininet/blob/5bae81be0bf3469ec580ef47658e9b9fac39e986/docs/FAILURE_BOOK.md). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0094"></a>

## PR #94: D-0049–D-0054: bounty, addressing, threat model, traceability, review wall, fork legitimacy, identity hardening

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-08, FD-09, FD-12, FD-14, FD-16.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/94) | [Files changed](https://github.com/mininet-labs/Mininet/pull/94/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/0efa2eaae3b1a575c0fca777f496ae95a324c295)

Head `0efa2eaae3b1a575c0fca777f496ae95a324c295`; base `34997f3ecf8e47ff1527a9e1b386d6a8cdefd82a`; merge `0aa8138d2d29484410b13170e2159e541f5a9563`. 32 changed files; 4 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This batch makes contributor compensation and threat traceability more concrete while tightening identity recovery and reward qualification. The important architectural choice is that a bounty payout does not confer Forge approval or political weight.

**Mechanism and evidence.** Bounty claims use a fixed pool ring and stealth payout machinery. Identity recovery works through precommitted key material and threshold checks rather than a custodian resetting an account. Required evidence sources are tightened so a high score from self-controlled signals cannot substitute for the required graph evidence.

**What remains weaker than the intended claim.** Changing a required source closes one score-composition shortcut, not graph-seed capture or unique-human proof. Fixed rings can leak pool membership and become weak when participants disclose their claims. A recovery operation that rotates a key has different semantics from restoring the same process state; later mobile work rightly separates them. The many doctrinal and implementation changes need separate assurance labels.

**Recommended improvement and rationale.** Bind every bounty claim and double-claim rule to an explicit pool, network and claim purpose without creating a global cross-pool identifier. Publish the recovery threat model for lost, compromised and precommitted-next keys separately. Keep eligibility evidence independent from payout value and from review authority.

**Concrete example.** An attacker with ample hardware but no required graph evidence should not become qualified by saturating other scores. Conversely, a legitimate contributor using two devices must not gain two bounty claims or two votes merely because device keys differ.

**Acceptance tests to implement.** Test cross-pool replay, two claims of one grant, incorrect ring substitution, recovery with wrong next-key commitments and threshold combinations. Model a captured graph-seed set instead of testing only isolated fake accounts. Confirm payout success leaves Forge approval counts unchanged.

**History, supersession and integration.** This builds on #4/#5. #101 makes AI-assistance metadata explicit; #206 adds non-rotating Controller restoration; #208 adds project grouping without varying amounts within a pool; later private-payment work must preserve these compartment boundaries.

**Source entry points.** [`crates/did-mini/src/controller.rs`](https://github.com/mininet-labs/Mininet/blob/0efa2eaae3b1a575c0fca777f496ae95a324c295/crates/did-mini/src/controller.rs); [`crates/did-mini/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/0efa2eaae3b1a575c0fca777f496ae95a324c295/crates/did-mini/src/error.rs); [`crates/did-mini/src/kel.rs`](https://github.com/mininet-labs/Mininet/blob/0efa2eaae3b1a575c0fca777f496ae95a324c295/crates/did-mini/src/kel.rs); [`crates/did-mini/tests/recovery.rs`](https://github.com/mininet-labs/Mininet/blob/0efa2eaae3b1a575c0fca777f496ae95a324c295/crates/did-mini/tests/recovery.rs); [`crates/mini-bounty/src/claim.rs`](https://github.com/mininet-labs/Mininet/blob/0efa2eaae3b1a575c0fca777f496ae95a324c295/crates/mini-bounty/src/claim.rs); [`crates/mini-bounty/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/0efa2eaae3b1a575c0fca777f496ae95a324c295/crates/mini-bounty/src/error.rs); [`crates/mini-bounty/src/ledger.rs`](https://github.com/mininet-labs/Mininet/blob/0efa2eaae3b1a575c0fca777f496ae95a324c295/crates/mini-bounty/src/ledger.rs); [`crates/mini-bounty/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/0efa2eaae3b1a575c0fca777f496ae95a324c295/crates/mini-bounty/src/lib.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 2 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0095"></a>

## PR #95: D-0055 → D-0060: mini-settlement, External Legitimacy Gates, README front door, nonce→sequence rename, zeroize hardening, FROST DKG + resharing

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-04, FD-05, FD-09, FD-12, FD-15.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/95) | [Files changed](https://github.com/mininet-labs/Mininet/pull/95/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/c1f08911e00e1df2a259186299797e353f9e9859)

Head `c1f08911e00e1df2a259186299797e353f9e9859`; base `0aa8138d2d29484410b13170e2159e541f5a9563`; merge `0721a84cb95c4bf3dfeb905a697e85fd58c20870`. 50 changed files; 6 commits; 0 issue comments, 0 inline comments and 2 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This is the bridge from ad hoc payment promises to explicit canonical settlement semantics, and from trusted-dealer treasury setup toward distributed key generation. It also assembles specialist audit scope material and records sensitive-code provenance.

**Mechanism and evidence.** mini-settlement exposes a canonical-ledger view and distinct pending/finalized states. FROST DKG and resharing use polynomial commitments, proofs, complaints and a qualified participant set, with unaudited markers and secret erasure. Key resharing preserves the group public key rather than creating a new identity for the funds.

**What remains weaker than the intended claim.** Preserving the group key does not revoke old threshold shares that their holders retained. Complaint handling needs globally consistent qualification and careful control of secret-share revelation. RFC 9591 specifies FROST signing; its section 4 is not a complete DKG specification, so citations must not transfer assurance to Mininet's DKG. Renaming a public sequence formerly called nonce is scanner clarification, not a randomness repair.

**Recommended improvement and rationale.** Write a separate normative DKG/resharing protocol covering reliable authenticated broadcast, participant/session binding, complaint limits, disqualification agreement and abort behavior. Rotate the actual custody key when removing the ability of an old threshold coalition to spend. Keep deterministic settlement ordering and threshold signing as separate reviewed mechanisms.

**Concrete example.** After resharing from custodians A/B/C to D/E/F, a retained old threshold may still sign under the unchanged group key. Calling that rotation would mislead users. Moving funds to a newly governed key is the operation that changes who can authorize future spending.

**Acceptance tests to implement.** Exercise equivocal commitments, inconsistent qualified sets, forged complaints, excessive public share disclosure, insufficient honest participants, replay between DKG sessions and old-threshold signing after resharing. Settlement tests must show conflicting local claims cannot both become canonical, without assuming a fake in-memory view is the production chain.

**History, supersession and integration.** #100 supplies a real chain-backed settlement view. #238 later exposes the mismatch between Mininet's FROST signature representation and PaymentClaim's supported signature suites. That mismatch is not solved by a treasury approval object.

**Source entry points.** [`crates/did-mini/src/kel.rs`](https://github.com/mininet-labs/Mininet/blob/c1f08911e00e1df2a259186299797e353f9e9859/crates/did-mini/src/kel.rs); [`crates/mini-settlement/src/claim.rs`](https://github.com/mininet-labs/Mininet/blob/c1f08911e00e1df2a259186299797e353f9e9859/crates/mini-settlement/src/claim.rs); [`crates/mini-settlement/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/c1f08911e00e1df2a259186299797e353f9e9859/crates/mini-settlement/src/error.rs); [`crates/mini-settlement/src/ledger.rs`](https://github.com/mininet-labs/Mininet/blob/c1f08911e00e1df2a259186299797e353f9e9859/crates/mini-settlement/src/ledger.rs); [`crates/mini-settlement/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/c1f08911e00e1df2a259186299797e353f9e9859/crates/mini-settlement/src/lib.rs); [`crates/mini-settlement/src/reconcile.rs`](https://github.com/mininet-labs/Mininet/blob/c1f08911e00e1df2a259186299797e353f9e9859/crates/mini-settlement/src/reconcile.rs); [`crates/mini-settlement/src/state.rs`](https://github.com/mininet-labs/Mininet/blob/c1f08911e00e1df2a259186299797e353f9e9859/crates/mini-settlement/src/state.rs); [`crates/mini-settlement/src/watcher.rs`](https://github.com/mininet-labs/Mininet/blob/c1f08911e00e1df2a259186299797e353f9e9859/crates/mini-settlement/src/watcher.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 2 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0100"></a>

## PR #100: D-0061/D-0062: mini-execution (closes #40) + real-transport interop for mini-bootstrap/mini-sync (closes #23)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-04, FD-05, FD-06, FD-16.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/100) | [Files changed](https://github.com/mininet-labs/Mininet/pull/100/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/8101c931f4c75623fad8966be77593bb3a87cb98)

Head `8101c931f4c75623fad8966be77593bb3a87cb98`; base `0721a84cb95c4bf3dfeb905a697e85fd58c20870`; merge `3af8d7f53bd0f4992604833f43e5110a3156dcf9`. 24 changed files; 2 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This composes the existing finality verifier with deterministic execution and real TCP bootstrap/synchronization. It makes canonical ordering operational for settlement instead of leaving CanonicalLedgerView implemented only by a test fixture.

**Mechanism and evidence.** LedgerChain applies finalized blocks only after certificate and state-commitment checks. LedgerState records canonical claim ordering, and independent chains fed the same accepted history converge. Bootstrap and ordinary object sync carry portable identity and release-related objects over sockets.

**What remains weaker than the intended claim.** The initial execution ledger records outcomes; it is not yet a funded balance ledger with debit/credit conservation. A certificate over a resulting state is also not necessarily a commitment to the exact body that produced that state. A bootstrap peer supplies bytes, not authority to choose genesis or declare a release legitimate.

**Recommended improvement and rationale.** Bind network identity, the complete application body and the resulting state together. Make the funding and rejection model explicit before claiming a usable currency. Require locally verified genesis/release ancestry rather than accepting whatever the first reachable bootstrap peer advertises. Preserve failure atomicity across both live and persisted state.

**Concrete example.** Two differently ordered or augmented bodies can sometimes lead to the same state. Before exact body commitments, agreement on a state root alone cannot establish which historical body was finalized. Separately, ordering a payment of 1,000 does not prove its payer ever owned 1,000.

**Acceptance tests to implement.** Construct same-state/different-body alternatives, forged genesis seeds, corrupted late sync objects and conflicting claims at one sequence. Show that authentic signatures without funds do not move value once balances are enabled, and that no failed block partially modifies the chain.

**History, supersession and integration.** #273 supplies checked balances, #274 bounded admission, #289 persistence/state sync, #300 exact-body commitments, and #313 shielded ordering. Review each addition against its own maturity rather than projecting the final design backwards.

**Source entry points.** [`crates/mini-bootstrap/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/8101c931f4c75623fad8966be77593bb3a87cb98/crates/mini-bootstrap/src/lib.rs); [`crates/mini-execution/src/body.rs`](https://github.com/mininet-labs/Mininet/blob/8101c931f4c75623fad8966be77593bb3a87cb98/crates/mini-execution/src/body.rs); [`crates/mini-execution/src/chain.rs`](https://github.com/mininet-labs/Mininet/blob/8101c931f4c75623fad8966be77593bb3a87cb98/crates/mini-execution/src/chain.rs); [`crates/mini-execution/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/8101c931f4c75623fad8966be77593bb3a87cb98/crates/mini-execution/src/error.rs); [`crates/mini-execution/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/8101c931f4c75623fad8966be77593bb3a87cb98/crates/mini-execution/src/lib.rs); [`crates/mini-execution/src/state.rs`](https://github.com/mininet-labs/Mininet/blob/8101c931f4c75623fad8966be77593bb3a87cb98/crates/mini-execution/src/state.rs); [`crates/mini-execution/tests/end_to_end.rs`](https://github.com/mininet-labs/Mininet/blob/8101c931f4c75623fad8966be77593bb3a87cb98/crates/mini-execution/tests/end_to_end.rs); [`crates/mini-sync/tests/sync_over_tcp.rs`](https://github.com/mininet-labs/Mininet/blob/8101c931f4c75623fad8966be77593bb3a87cb98/crates/mini-sync/tests/sync_over_tcp.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0101"></a>

## PR #101: Storage/crypto batch (mini-porep, mini-erasure) + self-hosted forge spine Batch 1 (mini-cli)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-03, FD-09, FD-11, FD-12, FD-14, FD-16.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/101) | [Files changed](https://github.com/mininet-labs/Mininet/pull/101/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/13fc8ae3ea067c97bad32867bf5aa11d726e5900)

Head `13fc8ae3ea067c97bad32867bf5aa11d726e5900`; base `3af8d7f53bd0f4992604833f43e5110a3156dcf9`; merge `7745d02feeb22e6af6734516fa62dd3f952461d2`. 43 changed files; 4 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This adds custom replication-proof and erasure-code prototypes and the first practical self-hosted mini command-line workflow. It reduces dependence on hosted development tools while making storage claims testable. The PR also makes AI-assistance declarations explicit rather than letting generated code look like independently authored review.

**Mechanism and evidence.** mini-porep seals data using a replica-specific graph construction; mini-erasure produces recoverable shards; the CLI drives identities, commits, proposals and reviews through existing Forge objects. The implementation-policy decision favors Mininet-owned composition over importing entire network stacks.

**What remains weaker than the intended claim.** The original erasure generator was not MDS for all advertised survivor sets, corrected in #106. Replication sampling later had an unconstrained-final-root gap, corrected in #299. Three CLI homes on one machine demonstrate protocol identities, not independent people. An in-house implementation is forkable but also increases the project's maintenance and audit obligations.

**Recommended improvement and rationale.** Compare the erasure matrix against an independent algebraic reference and test every small survivor subset. Define exactly what PoRep proves under storage/time assumptions, including challenge unpredictability and replica identity binding. Preserve Mininet-owned protocol semantics while allowing narrowly justified, licensed implementations or differential test oracles when they reduce risk.

**Concrete example.** A nominal k-of-n code must reconstruct from every valid k-subset, not just the first k shards. Honest encode/decode tests that always select systematic shards never exercise the broken parity case. Likewise, distinct replica identifiers do not establish distinct operators.

**Acceptance tests to implement.** Exhaustively enumerate small erasure survivor sets, mutate seal roots, challenge final-layer nodes and attempt on-demand reconstruction with reduced storage. Run the CLI from separate processes and stores with incomplete trust material. Prove neither paid storage nor an AI-assistance label changes approval weight.

**History, supersession and integration.** #106 repairs the matrix; #103-#115 build the release/install spine; #299/#302/#306 strengthen storage verification. D-0063 should not be interpreted as a license to invent unreviewed cryptographic primitives.

**Source entry points.** [`crates/mini-cli/src/cli.rs`](https://github.com/mininet-labs/Mininet/blob/13fc8ae3ea067c97bad32867bf5aa11d726e5900/crates/mini-cli/src/cli.rs); [`crates/mini-cli/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/13fc8ae3ea067c97bad32867bf5aa11d726e5900/crates/mini-cli/src/error.rs); [`crates/mini-cli/src/identity.rs`](https://github.com/mininet-labs/Mininet/blob/13fc8ae3ea067c97bad32867bf5aa11d726e5900/crates/mini-cli/src/identity.rs); [`crates/mini-cli/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/13fc8ae3ea067c97bad32867bf5aa11d726e5900/crates/mini-cli/src/lib.rs); [`crates/mini-cli/src/main.rs`](https://github.com/mininet-labs/Mininet/blob/13fc8ae3ea067c97bad32867bf5aa11d726e5900/crates/mini-cli/src/main.rs); [`crates/mini-cli/src/pr.rs`](https://github.com/mininet-labs/Mininet/blob/13fc8ae3ea067c97bad32867bf5aa11d726e5900/crates/mini-cli/src/pr.rs); [`crates/mini-cli/src/project.rs`](https://github.com/mininet-labs/Mininet/blob/13fc8ae3ea067c97bad32867bf5aa11d726e5900/crates/mini-cli/src/project.rs); [`crates/mini-cli/src/repo.rs`](https://github.com/mininet-labs/Mininet/blob/13fc8ae3ea067c97bad32867bf5aa11d726e5900/crates/mini-cli/src/repo.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 2 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0103"></a>

## PR #103: Self-hosted forge spine Batch 2a+2b: build provenance + isolated Wasmtime sandbox (D-0068/D-0069)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-03, FD-06, FD-10, FD-12, FD-14.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/103) | [Files changed](https://github.com/mininet-labs/Mininet/pull/103/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/8e8d49af396bafd3754292d37ea42caf1a38df9b)

Head `8e8d49af396bafd3754292d37ea42caf1a38df9b`; base `7745d02feeb22e6af6734516fa62dd3f952461d2`; merge `23703926d122c4f43e7d5f5fd63779e2acfeabbf`. 47 changed files; 4 commits; 1 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This adds build provenance and an isolated build runner, giving the self-hosted Forge a way to produce and compare artifacts without treating a build worker as a release authority. It is one of the most important supply-chain trust boundaries in the history.

**Mechanism and evidence.** mini-provenance records source, environment, command and output digests and counts distinct contributing roots. mini-pipeline describes bounded jobs; Wasmtime is isolated in a dedicated runner crate and subprocess. Capability declarations restrict filesystem and network access, while native-tool jobs are explicitly weaker than sandboxed jobs.

**What remains weaker than the intended claim.** An attestation proves what a signer claimed, not what an untrusted machine actually executed. Distinct keys are not independent builders. Sandbox assurance depends on the runtime, host imports, filesystem mediation and dependency versions; later Wasmtime advisories demonstrate that this boundary requires sustained maintenance. Determinism is not implied merely by using WebAssembly.

**Recommended improvement and rationale.** Use independently operated rebuilders, exact manifests and artifact hashes. Audit every host capability and resource budget, including clock/randomness exposure and symlink resolution. Retain full upstream advisory disposition for the pinned runtime. A build result must never directly produce release approval or owner adoption.

**Concrete example.** Two containers controlled by the same maintainer can issue two provenance records agreeing on a malicious artifact. The root count is correct but the independence claim is false. A sandbox escape can also corrupt the environment that reports the purportedly isolated result.

**Acceptance tests to implement.** Run capability-denial, traversal/symlink, timeout/fuel, memory-exhaustion and malicious-output-path tests. Rebuild the same source with independently provisioned environments and compare exact bytes. Inject a worker lying about its isolation label; acceptance must not treat that assertion as hardware attestation.

**History, supersession and integration.** #104 consumes provenance optionally in release evaluation; #109 tests full composition; #270 dispatches remote build jobs. #277 and #307 patch Wasmtime vulnerabilities and illustrate the ongoing cost of the sandbox boundary.

**Source entry points.** [`crates/did-mini/src/event.rs`](https://github.com/mininet-labs/Mininet/blob/8e8d49af396bafd3754292d37ea42caf1a38df9b/crates/did-mini/src/event.rs); [`crates/mini-build-runner-wasmtime/src/content_store.rs`](https://github.com/mininet-labs/Mininet/blob/8e8d49af396bafd3754292d37ea42caf1a38df9b/crates/mini-build-runner-wasmtime/src/content_store.rs); [`crates/mini-build-runner-wasmtime/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/8e8d49af396bafd3754292d37ea42caf1a38df9b/crates/mini-build-runner-wasmtime/src/error.rs); [`crates/mini-build-runner-wasmtime/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/8e8d49af396bafd3754292d37ea42caf1a38df9b/crates/mini-build-runner-wasmtime/src/lib.rs); [`crates/mini-build-runner-wasmtime/src/limiter.rs`](https://github.com/mininet-labs/Mininet/blob/8e8d49af396bafd3754292d37ea42caf1a38df9b/crates/mini-build-runner-wasmtime/src/limiter.rs); [`crates/mini-build-runner-wasmtime/src/main.rs`](https://github.com/mininet-labs/Mininet/blob/8e8d49af396bafd3754292d37ea42caf1a38df9b/crates/mini-build-runner-wasmtime/src/main.rs); [`crates/mini-build-runner-wasmtime/src/random.rs`](https://github.com/mininet-labs/Mininet/blob/8e8d49af396bafd3754292d37ea42caf1a38df9b/crates/mini-build-runner-wasmtime/src/random.rs); [`crates/mini-build-runner-wasmtime/src/sandbox.rs`](https://github.com/mininet-labs/Mininet/blob/8e8d49af396bafd3754292d37ea42caf1a38df9b/crates/mini-build-runner-wasmtime/src/sandbox.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 2 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0104"></a>

## PR #104: Ship self-hosted forge spine Batch 3: TUF-adapted release verification (D-0070)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-03, FD-06, FD-09, FD-14.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/104) | [Files changed](https://github.com/mininet-labs/Mininet/pull/104/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/782abc0c3b18a55c6fa4aef87f91d359233ce9c7)

Head `782abc0c3b18a55c6fa4aef87f91d359233ce9c7`; base `23703926d122c4f43e7d5f5fd63779e2acfeabbf`; merge `fe7df5c93392cecf909dbcdfbc6fabb9b438112b`. 16 changed files; 1 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This adapts selected TUF release ideas to Mininet's identity-root and governed-release model: rollback refusal, conflicting-release detection, staleness checks and an additional build-provenance requirement. It adds ways to refuse an update without adding a remote power to install one.

**Mechanism and evidence.** Version/check_no_rollback compares numeric dotted versions; detect_equivocation finds same-project/branch/version releases with different artifact digests. FreshnessPolicy checks caller-supplied last-sync age. evaluate_with_provenance combines independent_agreement with the existing release attestation quorum.

**What remains weaker than the intended claim.** These are selected TUF-inspired mechanisms, not demonstrated conformance to TUF's complete root/targets/snapshot/timestamp model. A local append-only store cannot expose a split view it never receives. A caller-supplied last_synced_ms does not prove fresh canonical release information. Optional provenance protects only callers who request it.

**Recommended improvement and rationale.** Make release policy explicit and versioned, including which checks are mandatory for each artifact class. Persist rollback/freshness pins safely and authenticate the evidence that advances them. Add independently gossiped release checkpoints without creating one mandatory transparency operator. Separate refusing a stale update from remotely disabling already owned software.

**Concrete example.** An eclipsed user can receive a fresh connection to an old, internally consistent release view. Updating last_synced_ms merely because the connection succeeded defeats the intended freshness check. The sync timestamp must reflect verified release-state evidence, not transport activity.

**Acceptance tests to implement.** Test replayed views, equal-version conflicting artifacts, different-length version tuples, clock rollback, forged provenance and caller attempts to omit a required check. Simulate two users receiving conflicting release histories, then exchanging evidence, while both retain the ability to refuse installation.

**History, supersession and integration.** #105 adds actual owner-controlled activation; #110 persists installer events; #269 retrieves exact release evidence. The external TUF comparison in this report identifies additional obligations rather than claiming this PR implemented the whole specification.

**Source entry points.** [`crates/mini-forge/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/782abc0c3b18a55c6fa4aef87f91d359233ce9c7/crates/mini-forge/src/lib.rs); [`crates/mini-forge/src/release.rs`](https://github.com/mininet-labs/Mininet/blob/782abc0c3b18a55c6fa4aef87f91d359233ce9c7/crates/mini-forge/src/release.rs); [`crates/mini-forge/tests/release.rs`](https://github.com/mininet-labs/Mininet/blob/782abc0c3b18a55c6fa4aef87f91d359233ce9c7/crates/mini-forge/tests/release.rs); [`crates/mini-update/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/782abc0c3b18a55c6fa4aef87f91d359233ce9c7/crates/mini-update/src/lib.rs); [`crates/mini-update/tests/update.rs`](https://github.com/mininet-labs/Mininet/blob/782abc0c3b18a55c6fa4aef87f91d359233ce9c7/crates/mini-update/tests/update.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 1 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0105"></a>

## PR #105: Ship self-hosted forge spine Batch 4: real installation, mini-installer (D-0071)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-01, FD-02, FD-03, FD-06, FD-09.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/105) | [Files changed](https://github.com/mininet-labs/Mininet/pull/105/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/0bd1eb20265930a90ff56e5f567aad5308003609)

Head `0bd1eb20265930a90ff56e5f567aad5308003609`; base `fe7df5c93392cecf909dbcdfbc6fabb9b438112b`; merge `50487fbf2cb008070171bb4f256e7abb32975d62`. 17 changed files; 1 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This supplies the real local action behind release adoption: staging, preflight, activation and rollback. It keeps software ownership with the person operating the machine rather than treating a signed release as permission to replace a running program.

**Mechanism and evidence.** Installer::activate requires an OwnerApproval naming the exact release. Staging assembles and checks artifact bytes; preflight verifies them again; activation changes a local current pointer and preserves a previous target. A failed health check rolls back rather than forcing forward progress.

**What remains weaker than the intended claim.** Typed approval is an API discipline, not proof that a human saw and understood a trusted user interface. File paths, pointer changes and state records introduce crash and filesystem attack surfaces. A caller-supplied health result is not an actual supervised process-health test. The original platform assumptions also do not establish Windows support.

**Recommended improvement and rationale.** Tie trusted local confirmation to the exact displayed release and policy digest. Reverify the opened artifact at the activation boundary, not just a pathname inspected earlier. Journal pointer and state transitions with explicit durability semantics and test real process startup, health and rollback. Keep any emergency update voluntary.

**Concrete example.** A release can pass preflight and then have its staged file replaced before activation. Alternatively, a power cut can occur after changing the current pointer but before recording the new state. The installed bytes and recovered state must agree in both cases.

**Acceptance tests to implement.** Test file substitution between preflight and activation, symlink/reparse-point attacks, every crash boundary, first-install rollback and wrong-release approval. Verify that a valid release remains inactive without owner approval and that refusal does not revoke identity, funds or public participation.

**History, supersession and integration.** #109 adds a composed success/failure scenario; #110 adds persistent event history; #171 makes unsupported non-Unix activation fail explicitly. #304 builds machine deployment around this path but must not bypass its consent boundary.

**Source entry points.** [`crates/mini-installer/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/0bd1eb20265930a90ff56e5f567aad5308003609/crates/mini-installer/src/error.rs); [`crates/mini-installer/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/0bd1eb20265930a90ff56e5f567aad5308003609/crates/mini-installer/src/lib.rs); [`crates/mini-installer/tests/installer.rs`](https://github.com/mininet-labs/Mininet/blob/0bd1eb20265930a90ff56e5f567aad5308003609/crates/mini-installer/tests/installer.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0106"></a>

## PR #106: mini-erasure MDS fix + mini-cli network sync (D-0072, Batch 5)

**PASS** | Captured outcome: **merged** | Directives: FD-03, FD-05, FD-06, FD-11, FD-14.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/106) | [Files changed](https://github.com/mininet-labs/Mininet/pull/106/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/44454a8ef40780e1a3fc5a20ff0d073f767425b3)

Head `44454a8ef40780e1a3fc5a20ff0d073f767425b3`; base `50487fbf2cb008070171bb4f256e7abb32975d62`; merge `026f1529f30b8ff77df59737a08f6c6d30937683`. 18 changed files; 3 commits; 1 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This is a substantive correctness repair, not routine cleanup: it replaces an erasure-code generator that could lose reconstructibility for a legal survivor set. It also advances the live TCP development workflow. The historical lesson is that a few successful round trips did not establish the advertised k-of-n guarantee.

**Mechanism and evidence.** The generator is normalized as V multiplied by the inverse of its first k rows, producing a systematic Reed-Solomon matrix rather than concatenating an identity matrix with an unnormalized Vandermonde parity block. A concrete small counterexample and exhaustive small-subset tests expose the earlier defect.

**What remains weaker than the intended claim.** The algebraic correction supports the scoped coding property. It does not prove correct handling of malicious shards without authenticated commitments, independent placement, or availability across correlated failures. A changed encoding also raises a migration question for any artifacts created with the old parity scheme; silently mixing formats is unsafe.

**Recommended improvement and rationale.** Keep an independently implemented finite-field reference and record the exact field polynomial, matrix convention and wire version. Quarantine or regenerate old prototype parity under explicit metadata. Expand real transport tests beyond localhost while retaining the code's clear separation between shard reconstruction and placement authority.

**Concrete example.** The historical 4-data/6-parity configuration has a legal four-shard survivor selection that was rank-deficient under the old generator. A test selecting only the first four systematic shards would pass every time and miss the failure completely.

**Acceptance tests to implement.** Enumerate every survivor subset for small configurations and cross-check random larger cases against a separate reference. Reject duplicate indices, inconsistent shard lengths and forged shard commitments. Test recovery after multiple holders disappear and ensure regeneration preserves the authenticated original content digest.

**History, supersession and integration.** This repairs #101. #253 later supplies placement/repair planning, which still counts DIDs rather than operators. The PASS is for the scoped correction and its direct counterexample, not production storage resilience.

**Source entry points.** [`crates/mini-cli/src/cli.rs`](https://github.com/mininet-labs/Mininet/blob/44454a8ef40780e1a3fc5a20ff0d073f767425b3/crates/mini-cli/src/cli.rs); [`crates/mini-cli/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/44454a8ef40780e1a3fc5a20ff0d073f767425b3/crates/mini-cli/src/error.rs); [`crates/mini-cli/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/44454a8ef40780e1a3fc5a20ff0d073f767425b3/crates/mini-cli/src/lib.rs); [`crates/mini-cli/src/store.rs`](https://github.com/mininet-labs/Mininet/blob/44454a8ef40780e1a3fc5a20ff0d073f767425b3/crates/mini-cli/src/store.rs); [`crates/mini-cli/src/sync.rs`](https://github.com/mininet-labs/Mininet/blob/44454a8ef40780e1a3fc5a20ff0d073f767425b3/crates/mini-cli/src/sync.rs); [`crates/mini-cli/tests/network_sync.rs`](https://github.com/mininet-labs/Mininet/blob/44454a8ef40780e1a3fc5a20ff0d073f767425b3/crates/mini-cli/tests/network_sync.rs); [`crates/mini-erasure/src/matrix.rs`](https://github.com/mininet-labs/Mininet/blob/44454a8ef40780e1a3fc5a20ff0d073f767425b3/crates/mini-erasure/src/matrix.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 1 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0107"></a>

## PR #107: mini-forge git SHA-256 export bridge + treasury/inflation/human-continuity economic decisions (D-0073–D-0075)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-03, FD-04, FD-07, FD-13, FD-16, FD-18.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/107) | [Files changed](https://github.com/mininet-labs/Mininet/pull/107/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/fa82a1caf8180d3491fc13c7cd36589ae0b0a37b)

Head `fa82a1caf8180d3491fc13c7cd36589ae0b0a37b`; base `026f1529f30b8ff77df59737a08f6c6d30937683`; merge `36a8ae2410043c9b3eb3bf9fcba843d52d226489`. 19 changed files; 2 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This combines a Git SHA-256 export bridge with major economic doctrine. It gives Mininet history an external interchange route while separating bridge reserves, issuance envelopes and human-continuity policy instead of letting one vaguely defined treasury mechanism decide all three.

**Mechanism and evidence.** Forge exports blobs, trees and commit chains into a constrained Git SHA-256 representation. The economic decisions distinguish external-asset custody from native MINI issuance and define separate annual envelope components. Presence-conditioned continuity affects vesting behavior rather than buying votes.

**What remains weaker than the intended claim.** Exported Git author metadata is not a did:mini signature, and the constrained mode/metadata subset is not a lossless export of every possible Git repository. Economic caps are policy, not proof of affordability, demand or correct custody. An optional external bridge still exposes its users to custodian and external-chain failure; that risk must not become core authority.

**Recommended improvement and rationale.** Publish a round-trip compatibility table for Git modes, author fields, signatures, object IDs and provenance. Keep external reserve claims explicitly conditional, with independently verifiable backing and exit rules. Evaluate equal-human distribution rather than balance-proportional approximations, and require custody failure to remain confined to the optional asset path.

**Concrete example.** An exported commit may preserve the file bytes while changing its author representation and commit identifier. A holder of a bridged external asset likewise owns a conditional external claim, not a protocol guarantee that a bank, custodian or another chain will always redeem it.

**Acceptance tests to implement.** Round-trip supported Git objects through a real Git implementation and reject unsupported shapes explicitly. Model reserve shortfall, bridge shutdown, unused issuance capacity, late adoption and continuity interruptions. Verify no reserve size, bridge participation or token balance changes validator or governance weight.

**History, supersession and integration.** #108 initially models one distribution incorrectly; #271 corrects that model and implements the monetary kernel. #276 adds the import direction while preserving importer provenance; #218 formalizes the edge-provider doctrine.

**Source entry points.** [`crates/mini-forge/src/git_export.rs`](https://github.com/mininet-labs/Mininet/blob/fa82a1caf8180d3491fc13c7cd36589ae0b0a37b/crates/mini-forge/src/git_export.rs); [`crates/mini-forge/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/fa82a1caf8180d3491fc13c7cd36589ae0b0a37b/crates/mini-forge/src/lib.rs); [`crates/mini-forge/tests/git_export.rs`](https://github.com/mininet-labs/Mininet/blob/fa82a1caf8180d3491fc13c7cd36589ae0b0a37b/crates/mini-forge/tests/git_export.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 1 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0108"></a>

## PR #108: docs: pre-audit validation prep (#47/50/97/98/28/21) + pre-coding planning triage

**PARTIAL** | Captured outcome: **merged** | Directives: FD-04, FD-06, FD-10, FD-11, FD-12, FD-13.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/108) | [Files changed](https://github.com/mininet-labs/Mininet/pull/108/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/d5d8533c831a2f6e504ebd99511dd67240bbcf16)

Head `d5d8533c831a2f6e504ebd99511dd67240bbcf16`; base `36a8ae2410043c9b3eb3bf9fcba843d52d226489`; merge `6a2bf27806502e55979db74bfbc60b0e196ac7b4`. 13 changed files; 2 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This prepares quantitative and physical test material for external review, including an economic sweep and device-test matrices. It contributes reproducible questions and negative results rather than production authorization.

**Mechanism and evidence.** The economic harness records scenario outcomes against declared thresholds; hardware documents describe BLE/UWB and weakest-device experiments. Audit preparation is explicitly separate from an independent specialist completing those experiments. Rejection of stale external planning also keeps the real self-hosted spine from being overwritten by paper proposals.

**What remains weaker than the intended claim.** The original economic model allocates Human Share in proportion to holdings, which is not the founder's equal-per-human rule. Consequently, its concentration results cannot be used as direct evidence against or for the intended monetary design. A written physical itinerary is not physical execution, and a large scenario count does not compensate for the wrong modeled mechanism.

**Recommended improvement and rationale.** Keep the old output as a historical failed model and link the corrected cohort model rather than rewriting history. Precommit success thresholds, label each assumption and separate structural tests from population estimates. Hardware reports need device/OS versions, raw minimized measurements, adversary capability and independent reproduction.

**Concrete example.** A holdings-proportional distribution mechanically gives an existing whale a larger share than a new participant. That observation says little about an equal-per-human distribution. Similarly, a localhost round-trip below a BLE threshold does not measure physical relay resistance.

**Acceptance tests to implement.** Compare model allocation against hand-computed equal-human cases, including one very wealthy and one zero-balance person. Run confidence-interval and sensitivity analysis without tuning thresholds after failures. On hardware, test wall/body blockage, relay attacks and background suspension, preserving participant privacy.

**History, supersession and integration.** #271 supplies the corrected equal-human cohort simulation. #285 is a stronger example of preserving explicit failed mechanism gates. #321's itinerary organizes external completion but does not convert these preparatory documents into completed audits.

**Source entry points.** [`docs/design/human-continuity-proof.md`](https://github.com/mininet-labs/Mininet/blob/d5d8533c831a2f6e504ebd99511dd67240bbcf16/docs/design/human-continuity-proof.md); [`docs/gates/dtn-design-constraints.md`](https://github.com/mininet-labs/Mininet/blob/d5d8533c831a2f6e504ebd99511dd67240bbcf16/docs/gates/dtn-design-constraints.md); [`docs/gates/economic-simulation-spec.md`](https://github.com/mininet-labs/Mininet/blob/d5d8533c831a2f6e504ebd99511dd67240bbcf16/docs/gates/economic-simulation-spec.md); [`docs/gates/hardware-test-log-template.csv`](https://github.com/mininet-labs/Mininet/blob/d5d8533c831a2f6e504ebd99511dd67240bbcf16/docs/gates/hardware-test-log-template.csv); [`docs/gates/hardware-test-protocol.md`](https://github.com/mininet-labs/Mininet/blob/d5d8533c831a2f6e504ebd99511dd67240bbcf16/docs/gates/hardware-test-protocol.md); [`docs/gates/wifi-bearer-test-protocol.md`](https://github.com/mininet-labs/Mininet/blob/d5d8533c831a2f6e504ebd99511dd67240bbcf16/docs/gates/wifi-bearer-test-protocol.md); [`docs/planning/ISSUE_CLOSURE_RULES.md`](https://github.com/mininet-labs/Mininet/blob/d5d8533c831a2f6e504ebd99511dd67240bbcf16/docs/planning/ISSUE_CLOSURE_RULES.md); [`docs/planning/PHASE_DEPENDENCY_GRAPH.md`](https://github.com/mininet-labs/Mininet/blob/d5d8533c831a2f6e504ebd99511dd67240bbcf16/docs/planning/PHASE_DEPENDENCY_GRAPH.md). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0109"></a>

## PR #109: test: self-hosted forge spine end-to-end harness

**PASS** | Captured outcome: **merged** | Directives: FD-02, FD-03, FD-06, FD-12, FD-14.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/109) | [Files changed](https://github.com/mininet-labs/Mininet/pull/109/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/ba2426e7cd369720f53a62e6d41b3cbba8bf0a9a)

Head `ba2426e7cd369720f53a62e6d41b3cbba8bf0a9a`; base `6a2bf27806502e55979db74bfbc60b0e196ac7b4`; merge `7e94feea5c5d75b7b6466da84616ff27fb79edea`. 9 changed files; 1 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This tests the integration that earlier component PRs only promised: a governed source change becomes a sandbox-built artifact, a signed release, an owner-approved installation and a recoverable failed update. Its primary contribution is evidence about composition across boundaries.

**Mechanism and evidence.** The test uses genuine identity/Forge/provenance/release objects and an actual Wasmtime subprocess. The installer follows staging, preflight, activation and health checking; a deliberately broken next release exercises rollback. Some operations are driven through library/CLI dispatch rather than a complete external CLI process for every step.

**What remains weaker than the intended claim.** The test is valuable within its declared environment. It does not establish independent humans, independently administered builders, hostile-network operation, or production release signing. It should not be described as a complete user-facing binary workflow before the later command and script work.

**Recommended improvement and rationale.** Retain this scenario as a compact integration regression, but add fault injection at every durable transition and separate-process command coverage. Label where identities share a filesystem or trust setup so those assumptions cannot quietly become the deployment architecture.

**Concrete example.** A successful build and release verification can coexist with a failed process health check. The expected outcome is continued use of the previous owner-approved release, not accepting the new artifact merely because its signatures and reproducible hash are valid.

**Acceptance tests to implement.** Assert exact artifact bytes, approval binding and previous-release restoration. Interrupt after each stage; restart the installer; alter one provenance or release object; and require failure without a forward activation. Run the same scenario through the compiled command-line interface.

**History, supersession and integration.** #110 exposes and persists more of the path; #111 supplies structured output; #113 carries release evidence over TCP; #115 runs the whole CLI lifecycle. This PASS recognizes integration coverage, not external audit or operational independence.

**Source entry points.** [`crates/mini-cli/src/identity.rs`](https://github.com/mininet-labs/Mininet/blob/ba2426e7cd369720f53a62e6d41b3cbba8bf0a9a/crates/mini-cli/src/identity.rs); [`crates/mini-cli/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/ba2426e7cd369720f53a62e6d41b3cbba8bf0a9a/crates/mini-cli/src/lib.rs); [`crates/mini-cli/tests/common/mod.rs`](https://github.com/mininet-labs/Mininet/blob/ba2426e7cd369720f53a62e6d41b3cbba8bf0a9a/crates/mini-cli/tests/common/mod.rs); [`crates/mini-cli/tests/self_hosted_spine_e2e.rs`](https://github.com/mininet-labs/Mininet/blob/ba2426e7cd369720f53a62e6d41b3cbba8bf0a9a/crates/mini-cli/tests/self_hosted_spine_e2e.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0110"></a>

## PR #110: installer: persisted event log (D-0076) + cli: wire build/release/provenance/installer commands (D-0077)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-03, FD-05, FD-06, FD-09.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/110) | [Files changed](https://github.com/mininet-labs/Mininet/pull/110/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/c454fbd557904805afab0d467dc967e0b6978711)

Head `c454fbd557904805afab0d467dc967e0b6978711`; base `7e94feea5c5d75b7b6466da84616ff27fb79edea`; merge `aaa34ed9c07cab9144f9130fca61c4fdcdfe79d9`. 25 changed files; 2 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This turns the self-hosted release/install spine into usable commands and makes installer progress survive ordinary process boundaries. The event history improves diagnosis and continuation without asking a hosted service what happened.

**Mechanism and evidence.** Typed installer events form a local hash chain, and CLI commands expose build, provenance, release and installer operations. Persisted state can be reconstructed for subsequent commands. The owner explicitly selects the operation rather than the software silently adopting a release.

**What remains weaker than the intended claim.** A hash chain detects inconsistency relative to a retained anchor; an attacker who can replace the whole local history can recompute a different chain. Separate event and filesystem-pointer writes require crash consistency. A command flag reporting health is a caller assertion, not evidence that the actual program started safely. Reloaded metadata must not substitute for rechecking release and artifact authority.

**Recommended improvement and rationale.** Define a single recoverable transaction across journal, staged bytes and activation pointer. Retain independent local or user-controlled anchors where tamper evidence is required. Make real health checks supervised and bounded. Revalidate exact artifact and approval state on restart before continuing an operation.

**Concrete example.** After a crash, a log might say preflight passed while the staged file has changed. Recovery must re-read the actual artifact and verify its digest and governed release rather than trusting the previous event as an authorization token.

**Acceptance tests to implement.** Corrupt a middle event, replace the entire log, cut power around rename/sync boundaries, alter a staged file after restart and provide stale owner approval. Each test should distinguish detected tampering, unavailable evidence and safely recoverable interruption; none should silently activate a different release.

**History, supersession and integration.** This builds on #105/#109. #111/#112 strengthen CLI contracts and adversarial tests. Later filesystem-index and consensus-archive work provides useful durability patterns but does not automatically retrofit the installer.

**Source entry points.** [`crates/mini-cli/src/build.rs`](https://github.com/mininet-labs/Mininet/blob/c454fbd557904805afab0d467dc967e0b6978711/crates/mini-cli/src/build.rs); [`crates/mini-cli/src/cli.rs`](https://github.com/mininet-labs/Mininet/blob/c454fbd557904805afab0d467dc967e0b6978711/crates/mini-cli/src/cli.rs); [`crates/mini-cli/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/c454fbd557904805afab0d467dc967e0b6978711/crates/mini-cli/src/error.rs); [`crates/mini-cli/src/installer.rs`](https://github.com/mininet-labs/Mininet/blob/c454fbd557904805afab0d467dc967e0b6978711/crates/mini-cli/src/installer.rs); [`crates/mini-cli/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/c454fbd557904805afab0d467dc967e0b6978711/crates/mini-cli/src/lib.rs); [`crates/mini-cli/src/provenance.rs`](https://github.com/mininet-labs/Mininet/blob/c454fbd557904805afab0d467dc967e0b6978711/crates/mini-cli/src/provenance.rs); [`crates/mini-cli/src/release.rs`](https://github.com/mininet-labs/Mininet/blob/c454fbd557904805afab0d467dc967e0b6978711/crates/mini-cli/src/release.rs); [`crates/mini-cli/tests/cli_spine_commands.rs`](https://github.com/mininet-labs/Mininet/blob/c454fbd557904805afab0d467dc967e0b6978711/crates/mini-cli/tests/cli_spine_commands.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 1 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0111"></a>

## PR #111: cli: stable --json output for build/release/provenance/installer (#112)

**PASS** | Captured outcome: **merged** | Directives: FD-03, FD-06, FD-10, FD-14.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/111) | [Files changed](https://github.com/mininet-labs/Mininet/pull/111/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/4d6bb686186024c4613ba26fbf8babb0161eb21f)

Head `4d6bb686186024c4613ba26fbf8babb0161eb21f`; base `aaa34ed9c07cab9144f9130fca61c4fdcdfe79d9`; merge `9b50184bacb1ad41d04b2834cd82cbc6c81ed89a`. 18 changed files; 1 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This makes the mini CLI usable as a stable automation interface rather than requiring people and scripts to scrape prose. It supports the founder's succession goal: future tools can compose exact operations and consume clear outcomes.

**Mechanism and evidence.** A versioned JSON success/error envelope covers the new command families. Unsupported combinations are rejected rather than silently ignored. Tests exercise real command dispatch and the emitted output contract; field extraction later removes fragile duplicate digest computations from the outage demonstration.

**What remains weaker than the intended claim.** The scoped output-contract improvement is sound as a direction. A hand-written JSON encoder still needs an independent parser oracle, especially for control characters and Unicode. Stable automation should rely on typed error codes and schema versions, not English wording. JSON output must never leak secret key material or turn a partial operation into success.

**Recommended improvement and rationale.** Keep machine output one complete document on stdout, diagnostics on stderr and exit status aligned with the envelope. Pin the schema and test against a standards-compliant parser outside the hand-written encoder. Bound output lists and make pagination explicit when data can grow.

**Concrete example.** A project label containing a quote, newline or non-ASCII character must remain valid JSON and round-trip exactly. A release verification failure must be a nonzero process exit and an error envelope, not a successful envelope containing an alarming message string.

**Acceptance tests to implement.** Execute the compiled binary, parse every output with an independent JSON library and test escaping, empty results, unknown flags, failure exit codes and interrupted operations. Scan outputs for test-secret values and verify that human-readable diagnostics never corrupt the machine envelope.

**History, supersession and integration.** #115 uses this interface to compose the no-GitHub script. Subsequent command additions should extend the same contract or explicitly refuse --json, as #286 later does, rather than silently producing a different format.

**Source entry points.** [`crates/mini-cli/src/build.rs`](https://github.com/mininet-labs/Mininet/blob/4d6bb686186024c4613ba26fbf8babb0161eb21f/crates/mini-cli/src/build.rs); [`crates/mini-cli/src/cli.rs`](https://github.com/mininet-labs/Mininet/blob/4d6bb686186024c4613ba26fbf8babb0161eb21f/crates/mini-cli/src/cli.rs); [`crates/mini-cli/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/4d6bb686186024c4613ba26fbf8babb0161eb21f/crates/mini-cli/src/error.rs); [`crates/mini-cli/src/installer.rs`](https://github.com/mininet-labs/Mininet/blob/4d6bb686186024c4613ba26fbf8babb0161eb21f/crates/mini-cli/src/installer.rs); [`crates/mini-cli/src/json.rs`](https://github.com/mininet-labs/Mininet/blob/4d6bb686186024c4613ba26fbf8babb0161eb21f/crates/mini-cli/src/json.rs); [`crates/mini-cli/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/4d6bb686186024c4613ba26fbf8babb0161eb21f/crates/mini-cli/src/lib.rs); [`crates/mini-cli/src/main.rs`](https://github.com/mininet-labs/Mininet/blob/4d6bb686186024c4613ba26fbf8babb0161eb21f/crates/mini-cli/src/main.rs); [`crates/mini-cli/src/provenance.rs`](https://github.com/mininet-labs/Mininet/blob/4d6bb686186024c4613ba26fbf8babb0161eb21f/crates/mini-cli/src/provenance.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0112"></a>

## PR #112: release/installer: adversarial CLI fixtures (#113)

**PASS** | Captured outcome: **merged** | Directives: FD-04, FD-06, FD-12, FD-15.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/112) | [Files changed](https://github.com/mininet-labs/Mininet/pull/112/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/5dde5035160b0eaa6f0fc779f4dbbcd4a8e564ec)

Head `5dde5035160b0eaa6f0fc779f4dbbcd4a8e564ec`; base `9b50184bacb1ad41d04b2834cd82cbc6c81ed89a`; merge `81c88122d034b0a123974a465de91b3bd4bf69fc`. 9 changed files; 1 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This is an adversarial-test contribution rather than a new feature. It exercises rejection paths in the self-hosted release/installer flow, reducing the risk that a convincing happy-path demonstration hides bypassable governance checks.

**Mechanism and evidence.** The added tests challenge approval thresholds, author exclusion, duplicate identity roots, artifact/branch binding, cooling conditions and installer state ordering. Positive anchors accompany negative cases so an implementation that refuses everything cannot masquerade as secure.

**What remains weaker than the intended claim.** Test names and green counts do not establish complete adversarial coverage. Several distinct roots can still be controlled by one person, and a forged or stale policy input remains a separate trust-boundary question. Because this PR adds tests without changing production behavior, the relevant evidence is that the tests reach the intended real checks, not that a feature was newly implemented.

**Recommended improvement and rationale.** Preserve the positive/negative pairs and add mutation testing of the exact guards. Challenge policy provenance, historical key rotation, durable-state rollback and correlated root control. Keep the test fixture's administrative independence assumptions visible.

**Concrete example.** Removing the duplicate-root check should make a test fail when one person signs from two devices. Removing author exclusion should make a different test fail. A test that merely expects a generic error before reaching those checks provides no such evidence.

**Acceptance tests to implement.** Mutate one enforcement condition at a time and record which test catches it. Verify each rejection leaves approval, release and installer state unchanged. Add a case with a valid signature under an obsolete key and another with a structurally valid policy that lacks canonical authority.

**History, supersession and integration.** This follows #109-#111 and supports #115. It does not replace the separate external cryptographic review or prove the broader founder-control sunset.

**Source entry points.** [`crates/mini-cli/tests/adversarial_release_install.rs`](https://github.com/mininet-labs/Mininet/blob/5dde5035160b0eaa6f0fc779f4dbbcd4a8e564ec/crates/mini-cli/tests/adversarial_release_install.rs); [`crates/mini-cli/tests/cli_spine_commands.rs`](https://github.com/mininet-labs/Mininet/blob/5dde5035160b0eaa6f0fc779f4dbbcd4a8e564ec/crates/mini-cli/tests/cli_spine_commands.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0113"></a>

## PR #113: sync: prove the full spine reaches a peer over mini sync alone (#114)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-03, FD-06, FD-09, FD-14.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/113) | [Files changed](https://github.com/mininet-labs/Mininet/pull/113/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/a4d00c1c15f66ce4952ed465d83f1f5a7a269d59)

Head `a4d00c1c15f66ce4952ed465d83f1f5a7a269d59`; base `81c88122d034b0a123974a465de91b3bd4bf69fc`; merge `1360a50206e4c2ba381c0fa67393f83f791a1e4d`. 8 changed files; 1 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This removes a shared-filesystem shortcut from the release-transfer demonstration. A second node obtains the necessary objects over a real socket and verifies a release locally, which is essential for surviving disappearance of the development host.

**Mechanism and evidence.** The integration uses independent local stores, ordinary verified object ingestion and the existing encrypted bearer/channel. Release verification and installation happen at the receiver; the serving peer supplies availability rather than signing authority.

**What remains weaker than the intended claim.** The fixture still establishes KEL trust separately and uses a friendly peer. Transferring a whole store may reveal unrelated material and makes cost grow with data the receiver did not request. Successful network delivery is not proof of fresh identity state or absence of an active intermediary.

**Recommended improvement and rationale.** Retrieve a bounded closure of the exact release and its verification evidence, preserving local trust decisions and per-object rejection. Add total transfer budgets, cancellation and recovery without accepting partially verified authority. Avoid requiring a single bootstrap peer or a server-issued account.

**Concrete example.** A node asking for release R should not receive or disclose unrelated private discussion objects merely because both live in the sender's store. A malicious server omitting an approval must leave the release unverified even when the executable bytes are available.

**Acceptance tests to implement.** Test missing approvals, substituted artifact links, oversized closures, truncated streams, unknown author KELs and a receiver restarting mid-transfer. Confirm release acceptance depends on its own verified evidence, not on which address served the bytes.

**History, supersession and integration.** #269 later implements exact bounded release retrieval. #296 adds stronger optional peer authentication and routing composition; those capabilities should not be confused with release authority.

**Source entry points.** [`crates/mini-cli/tests/network_sync_release.rs`](https://github.com/mininet-labs/Mininet/blob/a4d00c1c15f66ce4952ed465d83f1f5a7a269d59/crates/mini-cli/tests/network_sync_release.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0114"></a>

## PR #114: Networked multi-round Tendermint consensus — `mini-consensus` (D-0200–D-0203)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-04, FD-05, FD-06, FD-08, FD-11, FD-15, FD-16.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/114) | [Files changed](https://github.com/mininet-labs/Mininet/pull/114/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/68a679f79b36ee0df02d81c34a3026260f6dabd6)

Head `68a679f79b36ee0df02d81c34a3026260f6dabd6`; base `1360a50206e4c2ba381c0fa67393f83f791a1e4d`; merge `826976c27ee0f57bd5daf36b3866f01c0e800254`. 21 changed files; 1 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This is the major transition from checking finality certificates to running multi-round consensus over real sockets. It adds the state-machine behavior that a paper BFT quorum and a direct function call do not provide: proposals, prevotes, precommits, timeouts, locks and progress between rounds.

**Mechanism and evidence.** The Round/ConsensusNode machinery tracks locked and valid values, proof-of-lock information and round changes. Proposals are signed by the expected proposer with VOTE-capable delegation. TCP mesh buffering avoids making every outbound write a blocking dependency of the whole node.

**What remains weaker than the intended claim.** The implementation is a meaningful prototype, not a formal proof of the advertised Tendermint safety/liveness model. Static membership, local key freshness, persistence of signing/lock state, invalid application bodies and byzantine network topology remain separate obligations. Statements that equivocation is never a safety threat merely because one root is counted once are too broad across rounds and conflicting views.

**Recommended improvement and rationale.** Model the actual state machine, not an idealized protocol with the same name. Bind application validity, exact body, network and validator-set epoch into voting. Persist anti-double-signing state before publication. State synchrony and honest-connectivity assumptions explicitly; bounded buffers need backpressure policies that preserve honest progress.

**Concrete example.** A validator can sign one block, crash before remembering the lock and then sign a conflicting block after restart. Deduplicating roots inside a single certificate does not prevent this cross-history behavior. The signer and consensus journal must retain the relevant decision durably.

**Acceptance tests to implement.** Explore partitions, delayed/reordered/nil votes, conflicting valid-round proposals, malicious proposers, slow peers, buffer saturation and crash/restart schedules. Require no two conflicting finalized blocks under the stated Byzantine threshold, and eventual progress only when the declared synchrony/connectivity conditions hold.

**History, supersession and integration.** #116 adds evidence and re-gossip; #120 encryption; #130 catch-up; #289 archives/state sync; #300 exact-body commitments; #318/#319 optional discovery/authentication; #326 chunked snapshots. Those integrations must be tested together.

**Source entry points.** [`crates/mini-chain/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/68a679f79b36ee0df02d81c34a3026260f6dabd6/crates/mini-chain/src/error.rs); [`crates/mini-chain/src/vote.rs`](https://github.com/mininet-labs/Mininet/blob/68a679f79b36ee0df02d81c34a3026260f6dabd6/crates/mini-chain/src/vote.rs); [`crates/mini-consensus/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/68a679f79b36ee0df02d81c34a3026260f6dabd6/crates/mini-consensus/src/error.rs); [`crates/mini-consensus/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/68a679f79b36ee0df02d81c34a3026260f6dabd6/crates/mini-consensus/src/lib.rs); [`crates/mini-consensus/src/net.rs`](https://github.com/mininet-labs/Mininet/blob/68a679f79b36ee0df02d81c34a3026260f6dabd6/crates/mini-consensus/src/net.rs); [`crates/mini-consensus/src/node.rs`](https://github.com/mininet-labs/Mininet/blob/68a679f79b36ee0df02d81c34a3026260f6dabd6/crates/mini-consensus/src/node.rs); [`crates/mini-consensus/src/round.rs`](https://github.com/mininet-labs/Mininet/blob/68a679f79b36ee0df02d81c34a3026260f6dabd6/crates/mini-consensus/src/round.rs); [`crates/mini-consensus/src/wire.rs`](https://github.com/mininet-labs/Mininet/blob/68a679f79b36ee0df02d81c34a3026260f6dabd6/crates/mini-consensus/src/wire.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 1 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0115"></a>

## PR #115: forge: no-GitHub outage demo (#115)

**PASS** | Captured outcome: **merged** | Directives: FD-02, FD-03, FD-06, FD-07, FD-12.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/115) | [Files changed](https://github.com/mininet-labs/Mininet/pull/115/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/e3169e0906c390719c9aa9eaad80937d91f5e952)

Head `e3169e0906c390719c9aa9eaad80937d91f5e952`; base `f32398288d57174f4541274efcd56c772f62ad1d`; merge `ccdd9f2716b11794165e2006f2fc64cc3360b5e7`. 10 changed files; 1 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This supplies the strongest early end-to-end demonstration of the self-hosting idea: the real mini binary carries identities through Forge proposal, review, merge, release verification, installation and failed-update rollback without a GitHub API dependency.

**Mechanism and evidence.** A narrated shell script uses the JSON interface to pass exact identifiers between commands, establishes explicit KEL trust, and exercises both a healthy release and a deliberately broken one. A subprocess test runs the script itself, preventing documentation-only drift.

**What remains weaker than the intended claim.** The narrow demonstrated property is independence from GitHub at runtime in this scripted environment. It is not a firewall outage drill, an independently operated multi-human governance exercise, a fresh-machine offline build or proof that current repository ownership has decentralized. The explicit trust setup and local fixture identities remain important assumptions.

**Recommended improvement and rationale.** Keep this executable example, then extend the acceptance campaign to unavailable package registries, unavailable DNS/bootstrap providers and disappearance of all founder-held signing material. Separate the no-hosted-API result from the harder governance-succession result. Preserve the human's ability to reject updates even during an outage.

**Concrete example.** A prebuilt mini binary can complete this script without GitHub while a new contributor still cannot rebuild it if required source dependencies disappeared. Both are useful questions, but passing the first must not close the second.

**Acceptance tests to implement.** Run with outbound GitHub blocked; independently reproduce the binary from archived sources; start with empty stores and only documented trust material; lose one reviewer; and interrupt activation. Record exact artifacts and operator boundaries, not merely a completion marker.

**History, supersession and integration.** #249 adds the keystone identity/channel/presence/reward segment; #269/#270 advance native release retrieval and remote build execution. Current founder-guarded repository governance remains a distinct failure under FD-02 despite this scoped PASS.

**Source entry points.** [`crates/mini-cli/tests/no_github_outage_demo.rs`](https://github.com/mininet-labs/Mininet/blob/e3169e0906c390719c9aa9eaad80937d91f5e952/crates/mini-cli/tests/no_github_outage_demo.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0116"></a>

## PR #116: mini-consensus: equivocation evidence + re-gossip over partial meshes (D-0204, D-0205)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-04, FD-05, FD-06, FD-08, FD-15, FD-16.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/116) | [Files changed](https://github.com/mininet-labs/Mininet/pull/116/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/8438969bcabd546d2fd92c94890c08139a8988f1)

Head `8438969bcabd546d2fd92c94890c08139a8988f1`; base `826976c27ee0f57bd5daf36b3866f01c0e800254`; merge `f32398288d57174f4541274efcd56c772f62ad1d`. 13 changed files; 2 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This makes validator misbehavior observable and allows consensus messages to traverse a connected partial topology instead of requiring every validator to connect directly to every other. That reduces one practical barrier to independently operated nodes.

**Mechanism and evidence.** EquivocationEvidence contains two independently verified votes from one root at the same height, round and phase but for different hashes. Round counts the first vote once and surfaces the conflict. The network driver deduplicates and re-gossips messages; a four-node line fixture stalls if forwarding is removed.

**What remains weaker than the intended claim.** The PR's claim that equivocation was never a safety threat is too broad: deduplication inside one local tally does not neutralize Byzantine equivocation across different honest views. Likewise, connectedness of the total graph is insufficient for liveness if the only path between honest nodes goes through a censoring Byzantine relay. Seen-cache eviction also limits the lifetime of forward-once behavior.

**Recommended improvement and rationale.** State the honest-connectivity and eventual-delivery assumptions explicitly. Retain evidence through the network driver and durable storage; define objective evidence separately from any exclusion policy. Make re-gossip budgets resistant to validly signed floods without using wealth as voice weight.

**Concrete example.** In A--B--C, B can drop every message between honest A and C even though the graph is connected. The honest induced communication graph, not a diagram containing a hostile bridge, determines whether messages can reach a quorum.

**Acceptance tests to implement.** Test censoring articulation points, alternate honest routes, reordered conflicting votes, repeated evidence, cache eviction and malformed signatures. Show that evidence cannot accuse two different roots or votes in different rounds, and that losing a relay cannot create conflicting finality.

**History, supersession and integration.** #125 stops the driver from silently discarding evidence. #316 later introduces another equivocation registry/exclusion API; these implementations should converge on one canonical evidence format rather than drift.

**Source entry points.** [`crates/mini-consensus/src/evidence.rs`](https://github.com/mininet-labs/Mininet/blob/8438969bcabd546d2fd92c94890c08139a8988f1/crates/mini-consensus/src/evidence.rs); [`crates/mini-consensus/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/8438969bcabd546d2fd92c94890c08139a8988f1/crates/mini-consensus/src/lib.rs); [`crates/mini-consensus/src/net.rs`](https://github.com/mininet-labs/Mininet/blob/8438969bcabd546d2fd92c94890c08139a8988f1/crates/mini-consensus/src/net.rs); [`crates/mini-consensus/src/node.rs`](https://github.com/mininet-labs/Mininet/blob/8438969bcabd546d2fd92c94890c08139a8988f1/crates/mini-consensus/src/node.rs); [`crates/mini-consensus/src/round.rs`](https://github.com/mininet-labs/Mininet/blob/8438969bcabd546d2fd92c94890c08139a8988f1/crates/mini-consensus/src/round.rs); [`crates/mini-consensus/tests/networked_consensus.rs`](https://github.com/mininet-labs/Mininet/blob/8438969bcabd546d2fd92c94890c08139a8988f1/crates/mini-consensus/tests/networked_consensus.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0117"></a>

## PR #117: governance: integrate founder-supplied Governance Pack v1.0 (D-0082)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-03, FD-07, FD-10, FD-12, FD-14.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/117) | [Files changed](https://github.com/mininet-labs/Mininet/pull/117/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/fb141fd17e5c21829ee003322e0fc814485038d9)

Head `fb141fd17e5c21829ee003322e0fc814485038d9`; base `ccdd9f2716b11794165e2006f2fc64cc3360b5e7`; merge `9bb149f83b14b55637b437ae7988ad3e4affec55`. 89 changed files; 1 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This imports the founder-supplied governance pack as a subordinate process layer and provides a compatibility map instead of silently replacing existing principles. It improves the chance that future contributors can understand development governance without access to private founder conversations.

**Mechanism and evidence.** The PR adds normative process documents, Forge-object schemas, issue templates and a reference governance validator. Live CODEOWNERS and stricter PR-template/ruleset changes remain staged where real teams or decisions do not yet exist. The first policy workflow is advisory, explicitly using continue-on-error.

**What remains weaker than the intended claim.** Parseable schemas and complete-looking governance documents are not deployed authority. Advisory CI cannot enforce the process it describes. The pack is large enough that inconsistent vocabulary or stale supersession rules can become an informal central interpreter role. Copying documents verbatim preserves provenance but does not prove their internal consistency or conformance to the core values.

**Recommended improvement and rationale.** Keep the integration matrix as a live, versioned map of what is specification, implemented validation and activated policy. Add executable conformance cases for authority-bearing transitions before activation. Minimize duplicated normative text and make conflicts fail visibly rather than relying on whoever remembers the founder's intent.

**Concrete example.** A template referencing a security-review team that does not exist cannot provide an approval quorum. Leaving it inert is more honest than activating it and treating an unavailable or founder-controlled team as independent oversight.

**Acceptance tests to implement.** Validate JSON schemas with valid and invalid governance objects, not only JSON parsing. Exercise missing policy, contradictory precedence, stale signatures, duplicate approvers and inactive teams. Check that advisory tooling is never displayed as a production-governance enforcement result.

**History, supersession and integration.** #118 activates specific bootstrap policy and the AI charter; #127 provides stable directive IDs; #218 adds FD-18. This PR is process preparation, not decentralization achieved.

**Source entry points.** [`.github/CODEOWNERS.template`](https://github.com/mininet-labs/Mininet/blob/fb141fd17e5c21829ee003322e0fc814485038d9/.github/CODEOWNERS.template); [`.github/ISSUE_TEMPLATE/audit.yml`](https://github.com/mininet-labs/Mininet/blob/fb141fd17e5c21829ee003322e0fc814485038d9/.github/ISSUE_TEMPLATE/audit.yml); [`.github/ISSUE_TEMPLATE/bounty.yml`](https://github.com/mininet-labs/Mininet/blob/fb141fd17e5c21829ee003322e0fc814485038d9/.github/ISSUE_TEMPLATE/bounty.yml); [`.github/ISSUE_TEMPLATE/bug.yml`](https://github.com/mininet-labs/Mininet/blob/fb141fd17e5c21829ee003322e0fc814485038d9/.github/ISSUE_TEMPLATE/bug.yml); [`.github/ISSUE_TEMPLATE/config.yml`](https://github.com/mininet-labs/Mininet/blob/fb141fd17e5c21829ee003322e0fc814485038d9/.github/ISSUE_TEMPLATE/config.yml); [`.github/ISSUE_TEMPLATE/design.yml`](https://github.com/mininet-labs/Mininet/blob/fb141fd17e5c21829ee003322e0fc814485038d9/.github/ISSUE_TEMPLATE/design.yml); [`.github/ISSUE_TEMPLATE/implementation.yml`](https://github.com/mininet-labs/Mininet/blob/fb141fd17e5c21829ee003322e0fc814485038d9/.github/ISSUE_TEMPLATE/implementation.yml); [`.github/ISSUE_TEMPLATE/research.yml`](https://github.com/mininet-labs/Mininet/blob/fb141fd17e5c21829ee003322e0fc814485038d9/.github/ISSUE_TEMPLATE/research.yml). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 2 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0118"></a>

## PR #118: governance: activate Primary AI Engineer charter v1.1

**FAIL** | Captured outcome: **merged** | Directives: FD-02, FD-03, FD-08, FD-12, FD-16.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/118) | [Files changed](https://github.com/mininet-labs/Mininet/pull/118/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/bf144e7c44062db6e8cce3739590fe462490047a)

Head `bf144e7c44062db6e8cce3739590fe462490047a`; base `9bb149f83b14b55637b437ae7988ad3e4affec55`; merge `16acadbdb5b61b2cbe453e6a8c7d7145a7d904d2`. 51 changed files; 4 commits; 0 issue comments, 2 inline comments and 1 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This is the explicit governance-centralization turning point. It improves resistance to an AI proposal rewriting its own instructions, but also activates the temporary founder-only repository approval exception. Both contributions must be recorded together rather than letting the security tooling obscure the concentration of authority.

**Mechanism and evidence.** D-0084 binds charter, adapter, summary and phase digests. The canonical-base evaluator reads candidate files as data rather than executing their code. D-0083 sets required independent approvals to zero during the founder-guarded window; CODEOWNERS routes sensitive paths to the founder. Calendar and recorded sunset facts are checked.

**What remains weaker than the intended claim.** The temporary exception is transparent and bounded, but one founder account remains the integration control point. Expiry logic cannot independently discover unrecorded appointments, a release claim or a Forge cutover. A digest proves exact text, not the legitimacy of the human who activated it. The CodeQL comments flag the sensitive pull_request_target pattern; the actual mitigation must be traced to canonical-only execution and read-only permissions.

**Recommended improvement and rationale.** End the exception through a recorded, verifiable handoff to independent human maintainers and a tested succession mechanism. Keep model instructions non-authorizing, preserve owner adoption, and require independent exact-head evidence for sensitive changes. Do not automatically extend the exception or describe the repository as institution-independent while it remains active.

**Concrete example.** If the founder account disappears, the code is still copyable, but the current canonical repository integration process has lost its only operator. A fork can continue software development; that alone does not inherit the community's legitimate governance history.

**Acceptance tests to implement.** Run a founder-removal drill, expire the exception, record each earlier sunset trigger and test refusal after expiry. Try nested instruction files, symlinked instructions, changed charter digests and candidate executable payloads. Verify only canonical evaluator code executes and no candidate-controlled cache or credentials can acquire authority.

**History, supersession and integration.** #309 fixes a defect in exception-date scanning. #261 exposes the missing governed path to update protected instructions. The FAIL concerns current central control under FD-02/03/08, not a claim that the anti-injection work has no value.

**Source entry points.** [`.github/CODEOWNERS`](https://github.com/mininet-labs/Mininet/blob/bf144e7c44062db6e8cce3739590fe462490047a/.github/CODEOWNERS); [`.github/CODEOWNERS.template`](https://github.com/mininet-labs/Mininet/blob/bf144e7c44062db6e8cce3739590fe462490047a/.github/CODEOWNERS.template); [`.github/workflows/ci.yml`](https://github.com/mininet-labs/Mininet/blob/bf144e7c44062db6e8cce3739590fe462490047a/.github/workflows/ci.yml); [`.github/workflows/governance-canonical.yml`](https://github.com/mininet-labs/Mininet/blob/bf144e7c44062db6e8cce3739590fe462490047a/.github/workflows/governance-canonical.yml); [`.github/workflows/governance-policy.yml`](https://github.com/mininet-labs/Mininet/blob/bf144e7c44062db6e8cce3739590fe462490047a/.github/workflows/governance-policy.yml); [`AGENTS.md`](https://github.com/mininet-labs/Mininet/blob/bf144e7c44062db6e8cce3739590fe462490047a/AGENTS.md); [`CLAUDE.md`](https://github.com/mininet-labs/Mininet/blob/bf144e7c44062db6e8cce3739590fe462490047a/CLAUDE.md); [`CONTRIBUTING.md`](https://github.com/mininet-labs/Mininet/blob/bf144e7c44062db6e8cce3739590fe462490047a/CONTRIBUTING.md). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 3 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0119"></a>

## PR #119: consensus: edge-case attack review — timestamps, replay, fee manipulation (closes #44)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-04, FD-05, FD-06, FD-13, FD-15.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/119) | [Files changed](https://github.com/mininet-labs/Mininet/pull/119/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/4216a95f63df432b87a64ef8eae26e29c1b1c10c)

Head `4216a95f63df432b87a64ef8eae26e29c1b1c10c`; base `16acadbdb5b61b2cbe453e6a8c7d7145a7d904d2`; merge `27aa6b8e31cf7ab2932d918cedc47516c6b94096`. 14 changed files; 1 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This code-first attack review fixes three genuine gaps in consensus/value handling: unconstrained monotonic block time, undomained vote signatures and acceptance of a zero governed fee rate. It shows why a security review must inspect the actual transcript and arithmetic, not just verify that a signature function exists.

**Mechanism and evidence.** LedgerChain rejects non-increasing timestamps and proposal validation mirrors that early. Vote::transcript gains VOTE_SIGN_DOMAIN. PriceHistory::add_entry rejects a zero price. Negative tests prove legacy undomained signatures and invalid time/rate inputs are rejected.

**What remains weaker than the intended claim.** Monotonicity alone still permits a jump to the maximum timestamp, potentially preventing any later increase. A domain tag separates protocol purposes but is not a deployment identifier. Rejecting zero does not authorize the party supplying rates or bound extreme changes. The parallel review in #121 finds an independent narrowing-cast overflow that this PR missed.

**Recommended improvement and rationale.** Use deterministic logical block time where that is the actual protocol meaning, and keep physical time in a separately governed model. Bind deployment identity to signed consensus context. Make all fee arithmetic checked at both ingestion and quote time; a public PriceEntry can bypass an ingestion-only guard.

**Concrete example.** A proposer can choose u64::MAX and satisfy a simple greater-than-previous test. A later valid block cannot choose anything larger. Separately, a positive u128 fee product can truncate to an unrelated u64 value if converted with as.

**Acceptance tests to implement.** Include maximal timestamps, cross-deployment replay, direct construction of zero-rate entries, overflowing products and unchanged-state assertions after rejection. A signature-field mutation test is useful but must not be mislabeled as proving the distinct cross-protocol domain property.

**History, supersession and integration.** #124 reconciles #121's stronger deterministic timestamp and overflow fixes. Later network binding and exact-body commitment work must retain the signing-domain separation established here.

**Source entry points.** [`crates/mini-chain/src/vote.rs`](https://github.com/mininet-labs/Mininet/blob/4216a95f63df432b87a64ef8eae26e29c1b1c10c/crates/mini-chain/src/vote.rs); [`crates/mini-consensus/src/node.rs`](https://github.com/mininet-labs/Mininet/blob/4216a95f63df432b87a64ef8eae26e29c1b1c10c/crates/mini-consensus/src/node.rs); [`crates/mini-execution/src/chain.rs`](https://github.com/mininet-labs/Mininet/blob/4216a95f63df432b87a64ef8eae26e29c1b1c10c/crates/mini-execution/src/chain.rs); [`crates/mini-execution/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/4216a95f63df432b87a64ef8eae26e29c1b1c10c/crates/mini-execution/src/error.rs); [`crates/mini-execution/tests/end_to_end.rs`](https://github.com/mininet-labs/Mininet/blob/4216a95f63df432b87a64ef8eae26e29c1b1c10c/crates/mini-execution/tests/end_to_end.rs); [`crates/mini-value/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/4216a95f63df432b87a64ef8eae26e29c1b1c10c/crates/mini-value/src/error.rs); [`crates/mini-value/src/fee.rs`](https://github.com/mininet-labs/Mininet/blob/4216a95f63df432b87a64ef8eae26e29c1b1c10c/crates/mini-value/src/fee.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 3 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0120"></a>

## PR #120: consensus: wire mini_bearer::Channel into TcpMesh — links now confidential and tamper-evident

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-06, FD-09, FD-14, FD-15.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/120) | [Files changed](https://github.com/mininet-labs/Mininet/pull/120/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/e50bddf9a064c23c66eff2f1bbee3977a49e04ca)

Head `e50bddf9a064c23c66eff2f1bbee3977a49e04ca`; base `27aa6b8e31cf7ab2932d918cedc47516c6b94096`; merge `4b0f3b18a8accbd3e56c914636f9693b6058953f`. 9 changed files; 1 commits; 1 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This removes cleartext consensus payloads from ordinary TcpMesh links by reusing the existing Channel construction. It also handles a subtle interaction between encryption sequence numbers and outbound queue admission, a real integration issue invisible in isolated cryptographic tests.

**Mechanism and evidence.** The dialer and accepter perform a bounded anonymous ephemeral-X25519 handshake before nonblocking operation. Queue capacity is checked before sealing, because sealing consumes a channel sequence number; discarding a sealed frame would desynchronize the receiver. Votes and proposals retain their own signatures.

**What remains weaker than the intended claim.** Anonymous key agreement protects against passive observation after successful establishment but does not, by itself, authenticate the intended peer against active man-in-the-middle handshakes. Payload signatures protect consensus authenticity, not automatically session confidentiality. A decryption failure strands the link, and no reconnect policy exists in this PR.

**Recommended improvement and rationale.** Narrow on-path protection claims to the actual adversary model. Where peer identity matters, add channel-bound authenticated payload exchange and verify the intended endpoint, role and network; retain anonymous access for roles that do not require identity. Build bounded reconnect/backoff without counter resynchronization or insecure fallback.

**Concrete example.** If sealing increments a send counter and a full queue drops that ciphertext, the next transmitted frame has the wrong expected counter. The PR correctly avoids that failure by checking capacity first. An active intermediary establishing two separate anonymous sessions is a different attack and needs separate authentication.

**Acceptance tests to implement.** Fill queues immediately below and above framing overhead, then verify counter continuity. Inject malformed ciphertext, handshake timeouts, replayed frames, active two-session intermediaries and reconnect storms. Confirm invalid links cannot forge signed votes and never trigger plaintext downgrade.

**History, supersession and integration.** #296 adds optional authenticated transport composition and onion routing; #319 adds a validator-specific channel-bound attestation. Neither makes identity disclosure mandatory for every anonymous network participant.

**Source entry points.** [`crates/mini-consensus/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/e50bddf9a064c23c66eff2f1bbee3977a49e04ca/crates/mini-consensus/src/lib.rs); [`crates/mini-consensus/src/net.rs`](https://github.com/mininet-labs/Mininet/blob/e50bddf9a064c23c66eff2f1bbee3977a49e04ca/crates/mini-consensus/src/net.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0121"></a>

## PR #121: security: harden consensus timestamp, replay, and fee handling

**PASS** | Captured outcome: **closed** | Directives: FD-04, FD-05, FD-06, FD-10, FD-13.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/121) | [Files changed](https://github.com/mininet-labs/Mininet/pull/121/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/8e6802b3b5b0f98aebc60cffae3a2ae6f28a5752)

Head `8e6802b3b5b0f98aebc60cffae3a2ae6f28a5752`; base `16acadbdb5b61b2cbe453e6a8c7d7145a7d904d2`; merge `none`. 6 changed files; 23 commits; 1 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This closed-unmerged proposal materially improves the project despite not being merged itself. It independently identifies the timestamp-max jump and fee truncation defects, and its useful changes are credited and adopted in #124. Omitting it from the history would erase the provenance of a real correction.

**Mechanism and evidence.** The proposal requires timestamp_ms to equal block height, makes fee conversion return Result<u64>, rejects overflow and adds a historical rate-plus-conversion operation. Its review also corrects a test that initially mistook the correct nil prevote for accepting the malicious proposal.

**What remains weaker than the intended claim.** The signed-field replay test checks ordinary signature integrity rather than the separate domain-separation issue already fixed in #119. Closing the branch as superseded is not evidence the valid findings were rejected. Conversely, the proposed code should not be credited as an independently deployed second implementation.

**Recommended improvement and rationale.** Preserve a finding-level disposition: deterministic time and fee overflow adopted in #124; redundant signature-integrity test not adopted; broader economic/oracle and network-domain questions still open. Use such dispositions for every superseded PR rather than losing useful analysis when a branch closes.

**Concrete example.** A rejection test that sees any emitted prevote and calls that acceptance is wrong: consensus deliberately prevotes nil for an invalid proposal. The assertion must distinguish the malicious block hash from the nil value.

**Acceptance tests to implement.** Replay the maximal timestamp and overflowing fee counterexamples against the pre-fix revision and #124. Confirm each fails for the intended reason and that rejected proposals emit the protocol-correct nil response. Verify the adopted implementation, not merely this closed branch.

**History, supersession and integration.** #119 and #121 were parallel reviews of the same area. #124 is the merged reconciliation. This scoped PASS recognizes the contribution and preserved counterexamples, not an additional merged feature or an external audit.

**Source entry points.** [`crates/mini-chain/src/block.rs`](https://github.com/mininet-labs/Mininet/blob/8e6802b3b5b0f98aebc60cffae3a2ae6f28a5752/crates/mini-chain/src/block.rs); [`crates/mini-chain/src/vote.rs`](https://github.com/mininet-labs/Mininet/blob/8e6802b3b5b0f98aebc60cffae3a2ae6f28a5752/crates/mini-chain/src/vote.rs); [`crates/mini-consensus/src/node.rs`](https://github.com/mininet-labs/Mininet/blob/8e6802b3b5b0f98aebc60cffae3a2ae6f28a5752/crates/mini-consensus/src/node.rs); [`crates/mini-value/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/8e6802b3b5b0f98aebc60cffae3a2ae6f28a5752/crates/mini-value/src/error.rs); [`crates/mini-value/src/fee.rs`](https://github.com/mininet-labs/Mininet/blob/8e6802b3b5b0f98aebc60cffae3a2ae6f28a5752/crates/mini-value/src/fee.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0123"></a>

## PR #123: uniqueness: rename HumanStatus::FullHuman to EvidenceQualifiedHuman (D-0086)

**PASS** | Captured outcome: **merged** | Directives: FD-01, FD-08, FD-10, FD-12.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/123) | [Files changed](https://github.com/mininet-labs/Mininet/pull/123/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/d92a4987c38965f9d231aedd8286712f06d896cd)

Head `d92a4987c38965f9d231aedd8286712f06d896cd`; base `4b0f3b18a8accbd3e56c914636f9693b6058953f`; merge `f32f5cb5f8461df4e8f47a691ec614588a81ac1f`. 8 changed files; 1 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This corrects an authority-signaling name: FullHuman becomes EvidenceQualifiedHuman. The change matters because a type or UI label can cause downstream engineers to rely on a guarantee the mechanism never established.

**Mechanism and evidence.** The public status variant, tests and current-state documentation are renamed while qualification behavior remains unchanged. Historical decision and audit documents retain their original terminology, preserving what was actually claimed at the time.

**What remains weaker than the intended claim.** The renamed status is still not a unique-human credential. Retaining Human in the name may remain easy to overread, and downstream UI or policy code can still conflate evidence qualification with eligibility. A naming repair should therefore be credited as honesty, not as progress on the underlying Sybil proof.

**Recommended improvement and rationale.** Use explicit evidence-qualified wording at every authority boundary and keep UniqueHumanCredential absent until its verifier is implemented and independently evaluated. Make conversions to role or voting eligibility explicit and policy-bound. Keep historic names searchable through a migration note, without rewriting the old record.

**Concrete example.** A wallet can display evidence qualified while governance still refuses to treat the same record as proof of one person. An automatic conversion from EvidenceQualifiedHuman to one voting credential would undo the purpose of the rename.

**Acceptance tests to implement.** Test that the rename preserves existing classification behavior, and add structural checks that no public copy claims verified unique humanity from this enum. Search downstream code for implicit conversion into validator, reward or reviewer authority and require a separate verified eligibility input.

**History, supersession and integration.** This corrects terminology introduced in #5. #126 separates credential classes; #143 further reconciles the evidence taxonomy; #220 develops explicit personhood research gates. The scoped PASS is for reducing a misleading claim.

**Source entry points.** [`crates/mini-uniqueness/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/d92a4987c38965f9d231aedd8286712f06d896cd/crates/mini-uniqueness/src/lib.rs); [`crates/mini-uniqueness/src/status.rs`](https://github.com/mininet-labs/Mininet/blob/d92a4987c38965f9d231aedd8286712f06d896cd/crates/mini-uniqueness/src/status.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0124"></a>

## PR #124: consensus/value: deterministic timestamps + fee-overflow fix (D-0087, reconciles PR #121)

**PASS** | Captured outcome: **merged** | Directives: FD-04, FD-05, FD-06, FD-10, FD-13.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/124) | [Files changed](https://github.com/mininet-labs/Mininet/pull/124/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/3b8365044c3598af86a437b112eb9ce3a1837f02)

Head `3b8365044c3598af86a437b112eb9ce3a1837f02`; base `f32f5cb5f8461df4e8f47a691ec614588a81ac1f`; merge `ddb0f16740d7205f5eaa766d518e9a82d18a396a`. 15 changed files; 1 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This reconciles parallel reviews without discarding their useful differences. It closes the actual maximum-timestamp and fee-overflow holes left after #119, while crediting the closed proposal #121 that identified them.

**Mechanism and evidence.** Both authoritative execution and proposal validation require timestamp_ms == height. Checked u128-to-u64 conversion returns FeeOverflow instead of truncating. Direct quote conversion rejects a zero rate even when a caller constructs PriceEntry without going through PriceHistory::add_entry; fee_at composes lookup and checked conversion.

**What remains weaker than the intended claim.** The scoped deterministic/arithmetic corrections are well motivated. The timestamp is now logical block time, not milliseconds of physical elapsed time despite its field name. Downstream vesting, expiry or ranging code must not interpret it as a real wall clock. The rate source itself remains an authorization and economic-policy question.

**Recommended improvement and rationale.** Rename or wrap logical time at the next coordinated format/API change so units cannot be confused. Keep checked arithmetic at the final consumer, not just the producer. Add independent integer-reference vectors and ensure any future price governance cannot convert wealth into political control.

**Concrete example.** At height 10, a proposed timestamp of 11 is rejected even if it is greater than the previous value. That is correct for logical time. It would be incorrect to infer that only ten real milliseconds have elapsed since genesis and accelerate a real-world vesting promise accordingly.

**Acceptance tests to implement.** Check every boundary around u64::MAX and exact conversion limits, direct zero-rate construction, historical lookup transitions and wrong logical time. Assert rejected blocks leave chain state unchanged and that no caller unwraps the new Result to restore a panic path.

**History, supersession and integration.** Adopts #121's useful fixes after #119. #272 later defines deterministic policy-time vesting and must keep its physical-time assumptions explicit. This PASS is for the named corrections, not the completeness of economic governance.

**Source entry points.** [`crates/mini-chain/src/block.rs`](https://github.com/mininet-labs/Mininet/blob/3b8365044c3598af86a437b112eb9ce3a1837f02/crates/mini-chain/src/block.rs); [`crates/mini-consensus/src/node.rs`](https://github.com/mininet-labs/Mininet/blob/3b8365044c3598af86a437b112eb9ce3a1837f02/crates/mini-consensus/src/node.rs); [`crates/mini-execution/src/chain.rs`](https://github.com/mininet-labs/Mininet/blob/3b8365044c3598af86a437b112eb9ce3a1837f02/crates/mini-execution/src/chain.rs); [`crates/mini-execution/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/3b8365044c3598af86a437b112eb9ce3a1837f02/crates/mini-execution/src/error.rs); [`crates/mini-execution/tests/end_to_end.rs`](https://github.com/mininet-labs/Mininet/blob/3b8365044c3598af86a437b112eb9ce3a1837f02/crates/mini-execution/tests/end_to_end.rs); [`crates/mini-value/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/3b8365044c3598af86a437b112eb9ce3a1837f02/crates/mini-value/src/error.rs); [`crates/mini-value/src/fee.rs`](https://github.com/mininet-labs/Mininet/blob/3b8365044c3598af86a437b112eb9ce3a1837f02/crates/mini-value/src/fee.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 1 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0125"></a>

## PR #125: identity/consensus: KEL freshness pin + equivocation-evidence consequence (D-0088)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-05, FD-06, FD-08, FD-15.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/125) | [Files changed](https://github.com/mininet-labs/Mininet/pull/125/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/7c64eba154dee5ac77d7bfa1a2dc32862aea5c23)

Head `7c64eba154dee5ac77d7bfa1a2dc32862aea5c23`; base `ddb0f16740d7205f5eaa766d518e9a82d18a396a`; merge `ff1b4038c171a3ec4ef501f56fcbf5b18ed10c87`. 16 changed files; 1 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This turns two previously documented recommendations into code: remember the newest KEL sequence already seen, and stop dropping verified equivocation evidence at the network driver. It is valuable incremental hardening with explicitly limited scope.

**Mechanism and evidence.** FreshnessPins verifies a KEL and rejects a sequence lower than the highest retained sequence for its SCID. EquivocatorRegistry re-verifies evidence, deduplicates accused roots and is threaded through the live consensus driver. It does not alter the static validator set.

**What remains weaker than the intended claim.** The current FreshnessPins stores only a sequence number, not the event digest or a prefix commitment. An equally long conflicting controller-signed branch is not a sequence regression. The registry is in memory, so the PR body's phrase durably queryable must not be read as disk persistence. First-contact freshness remains unresolved by a local high-water mark.

**Recommended improvement and rationale.** Persist sequence-and-digest/prefix pins before authority decisions and detect conflicting same-sequence or incompatible extending branches. Keep duplicity evidence independently verifiable. Define bounded retention and recovery, and connect first-contact witness assurance only under an explicit reviewed policy rather than declaring a pin equivalent to witness agreement.

**Concrete example.** A verifier pins sequence 5 on branch A. A compromised controller presents a different validly signed branch B ending at sequence 5. Comparing only 5 < 5 does not reject B. A digest/prefix continuity check is needed in addition to monotonicity.

**Acceptance tests to implement.** Test same-sequence forks, higher-sequence branches with incompatible prefixes, restart after pinning, full-capacity retention and corrupted persistent pins. Verify invalid accusations flag nobody, repeated evidence does not multiply penalties, and an honest validator's identity rights are not revoked by a role-specific fault.

**History, supersession and integration.** #149 designs witness receipts/gossip; #180/#187/#191 implement pieces; #314 binds witness policy to the KEL. #316 adds a second accountability path, making consolidation and shared invariants important.

**Source entry points.** [`crates/did-mini/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/7c64eba154dee5ac77d7bfa1a2dc32862aea5c23/crates/did-mini/src/error.rs); [`crates/did-mini/src/freshness.rs`](https://github.com/mininet-labs/Mininet/blob/7c64eba154dee5ac77d7bfa1a2dc32862aea5c23/crates/did-mini/src/freshness.rs); [`crates/did-mini/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/7c64eba154dee5ac77d7bfa1a2dc32862aea5c23/crates/did-mini/src/lib.rs); [`crates/did-mini/tests/recovery.rs`](https://github.com/mininet-labs/Mininet/blob/7c64eba154dee5ac77d7bfa1a2dc32862aea5c23/crates/did-mini/tests/recovery.rs); [`crates/mini-consensus/src/consequence.rs`](https://github.com/mininet-labs/Mininet/blob/7c64eba154dee5ac77d7bfa1a2dc32862aea5c23/crates/mini-consensus/src/consequence.rs); [`crates/mini-consensus/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/7c64eba154dee5ac77d7bfa1a2dc32862aea5c23/crates/mini-consensus/src/lib.rs); [`crates/mini-consensus/src/net.rs`](https://github.com/mininet-labs/Mininet/blob/7c64eba154dee5ac77d7bfa1a2dc32862aea5c23/crates/mini-consensus/src/net.rs); [`crates/mini-consensus/src/node.rs`](https://github.com/mininet-labs/Mininet/blob/7c64eba154dee5ac77d7bfa1a2dc32862aea5c23/crates/mini-consensus/src/node.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 1 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0126"></a>

## PR #126: docs: credential taxonomy, custody-separation clause, docs-supersession finding (D-0089)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-08, FD-09, FD-14, FD-16, FD-18.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/126) | [Files changed](https://github.com/mininet-labs/Mininet/pull/126/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/606ccbb00f6d853597a33782a8c6c56da99e22e6)

Head `606ccbb00f6d853597a33782a8c6c56da99e22e6`; base `ff1b4038c171a3ec4ef501f56fcbf5b18ed10c87`; merge `a417f7cc50073f49c8d71a161520fbb320596fa9`. 8 changed files; 1 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This names distinctions the rest of the project needs to preserve: participant identity, human evidence, role credentials and resource credentials. It also records that treasury and bridge signer committees should be separate, without pretending a document enforces that separation.

**Mechanism and evidence.** The taxonomy maps existing KEL, HumanRecord, capabilities, validator sets and resource receipts into explicit claim classes. UniqueHumanCredential remains unbuilt. The custody-separation clause says general treasury and bridge-vault seats must be disjoint. The supersession sweep correctly confines its findings to documents actually present in GitHub.

**What remains weaker than the intended claim.** A disjoint set of DIDs is not proof of disjoint people or independent operators. Prose-only separation can be bypassed by constructor inputs or out-of-repository appointments. The taxonomy is useful only if consumers do not freely reinterpret a resource receipt or evidence score as role legitimacy.

**Recommended improvement and rationale.** Encode credential purposes and permitted conversions at the verifier boundary. Enforce committee-role separation using the best independently validated membership evidence available, while explicitly retaining residual collusion risk. Maintain a source-scope note when external specifications are unavailable rather than importing assumptions from private documents.

**Concrete example.** One person can control DID A on the treasury committee and DID B on a bridge committee. A simple set-intersection check passes even though the intended separation has failed. Calling that operational independence would repeat the project's personhood confusion.

**Acceptance tests to implement.** Test cross-purpose credential substitution and role grants derived from payment or storage receipts. For custody, test identical-root overlap, linked operator overlap where demonstrable, signer replacement and loss of one committee. Core identity and public publishing must continue if every bridge custodian disappears.

**History, supersession and integration.** The taxonomy follows #123's naming correction. #218/#219 expand the edge/core distinction. Future credential and treasury implementations should cite this classification but still prove their own enforcement.

**Source entry points.** [`README.md`](https://github.com/mininet-labs/Mininet/blob/606ccbb00f6d853597a33782a8c6c56da99e22e6/README.md); [`docs/DECISION_LOG.md`](https://github.com/mininet-labs/Mininet/blob/606ccbb00f6d853597a33782a8c6c56da99e22e6/docs/DECISION_LOG.md); [`docs/STATUS.md`](https://github.com/mininet-labs/Mininet/blob/606ccbb00f6d853597a33782a8c6c56da99e22e6/docs/STATUS.md); [`docs/design/credential-taxonomy.md`](https://github.com/mininet-labs/Mininet/blob/606ccbb00f6d853597a33782a8c6c56da99e22e6/docs/design/credential-taxonomy.md); [`docs/design/treasury-economic-model.md`](https://github.com/mininet-labs/Mininet/blob/606ccbb00f6d853597a33782a8c6c56da99e22e6/docs/design/treasury-economic-model.md). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0127"></a>

## PR #127: docs: canonicalize the 17 Founder Directives; generate constitution registry (D-0090)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-03, FD-07, FD-08, FD-10, FD-12.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/127) | [Files changed](https://github.com/mininet-labs/Mininet/pull/127/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/cb7a23da48e6bf17eef23208a6c37c054c14d3d9)

Head `cb7a23da48e6bf17eef23208a6c37c054c14d3d9`; base `a417f7cc50073f49c8d71a161520fbb320596fa9`; merge `22b47a793e74868ea564dbd5171e43cb3d1ed903`. 9 changed files; 1 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This supplies one stable, digest-bound identity for the founder's principle set after earlier external documents used incompatible counts. It is a significant historical clarification: seventeen canonical directives become the explicit repository reference, later extended to eighteen.

**Mechanism and evidence.** constitution_registry.py parses ordered directive headings and generates stable FD identifiers plus exact text digests. The human-readable distillation is separate from the text digest. D-0090 records the founder's canonicalization decision and states that earlier external principle framings are superseded.

**What remains weaker than the intended claim.** The registry checks correspondence to the selected text, not whether a proposed constitutional change is legitimate. Some surviving documents still refer to an external SPEC-00 as controlling; the exact relationship between principle supersession and other invariant/specification material needs a human-reviewed precedence clarification. This review does not silently decide that constitutional question.

**Recommended improvement and rationale.** Publish one precise authority/predecessor map: which documents and versions are normative, which portions are superseded, and how a legitimate future amendment is recognized. Bind that map to the same reviewed governance record. Keep immutable historical versions and stable directive IDs so future audits can distinguish changed values from changed implementation.

**Concrete example.** A regenerated registry can perfectly match a maliciously edited directive document. The hash confirms the edit's bytes; it does not authorize the edit. An external auditor must validate the governing amendment history as well as the digest.

**Acceptance tests to implement.** Test reordered, missing and duplicated headings, modified text with an old registry, and a regenerated but unauthorized candidate amendment. Check links from each invariant and release policy resolve to the declared normative version, not an uncommitted external document.

**History, supersession and integration.** #118 establishes the bootstrap authority context; #218 adds FD-18 with updated registry generation. The older six- and eleven-principle framings are historical, not alternative lists silently merged into this review.

**Source entry points.** [`README.md`](https://github.com/mininet-labs/Mininet/blob/cb7a23da48e6bf17eef23208a6c37c054c14d3d9/README.md); [`docs/CONSTITUTION_REGISTRY.json`](https://github.com/mininet-labs/Mininet/blob/cb7a23da48e6bf17eef23208a6c37c054c14d3d9/docs/CONSTITUTION_REGISTRY.json); [`docs/DECISION_LOG.md`](https://github.com/mininet-labs/Mininet/blob/cb7a23da48e6bf17eef23208a6c37c054c14d3d9/docs/DECISION_LOG.md); [`docs/FOUNDER_DIRECTIVES.md`](https://github.com/mininet-labs/Mininet/blob/cb7a23da48e6bf17eef23208a6c37c054c14d3d9/docs/FOUNDER_DIRECTIVES.md); [`docs/STATUS.md`](https://github.com/mininet-labs/Mininet/blob/cb7a23da48e6bf17eef23208a6c37c054c14d3d9/docs/STATUS.md); [`tools/constitution_registry.py`](https://github.com/mininet-labs/Mininet/blob/cb7a23da48e6bf17eef23208a6c37c054c14d3d9/tools/constitution_registry.py). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0128"></a>

## PR #128: sync/bearer: real mid-transfer TCP-kill resume test; local peer discovery over UDP multicast (D-0091)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-06, FD-09, FD-11, FD-14.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/128) | [Files changed](https://github.com/mininet-labs/Mininet/pull/128/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/04bc7283b15f3f889c9ae22bed293d8e3dce99fc)

Head `04bc7283b15f3f889c9ae22bed293d8e3dce99fc`; base `22b47a793e74868ea564dbd5171e43cb3d1ed903`; merge `9a704d73925d0634b4e9c7bdd57cf828d92f8a7e`. 9 changed files; 1 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This adds a real interruption/reconnection test and local serverless peer discovery. It moves beyond a resume test that failed before any content arrived, exposing the difference between a protocol restart and an uninterrupted happy path.

**Mechanism and evidence.** The KillSwitchBearer wraps a real TCP connection and interrupts mid-transfer; the failed attempt leaves no newly committed objects, and a fresh connection converges the stores. LocalAnnouncer/LocalScanner exchange identity-free UDP multicast hints. This is a custom discovery datagram, explicitly not full mDNS/DNS-SD.

**What remains weaker than the intended claim.** The sync result demonstrates restart by idempotence, not checkpointed continuation from already accepted chunks. Multicast discovers addresses but neither authenticates them nor hides network presence. An identity-free message still exposes IP-level participation. Hardware Wi-Fi behavior, mobile background restrictions and hostile multicast floods remain untested by the loopback fixtures.

**Recommended improvement and rationale.** Label restart and resumable-progress guarantees separately. Bound discovery responses, socket time and aggregate candidates; authenticate later authority-bearing interactions independently. Allow discovery to be disabled or replaced locally, with no canonical directory becoming mandatory.

**Concrete example.** A transfer stopped after several received batches can safely restart from zero and eventually succeed, yet consume the entire bandwidth again. On a costly weak link this is materially different from resuming the missing suffix. Both need separate measurements and labels.

**Acceptance tests to implement.** Interrupt at each framing and commit boundary; inspect receiver state and bytes retransmitted. Flood malformed multicast hints, supply unroutable addresses and run with multicast unavailable. On actual phones, test interface changes, sleep and battery constraints without treating the discovery hint as personhood evidence.

**History, supersession and integration.** #129 adds PEX; #289 adds persistent consensus snapshots; #326 chunks state transfer but still lacks retry/resume policy. No one of these should close the physical-device gate by title alone.

**Source entry points.** [`crates/mini-bearer/src/discovery.rs`](https://github.com/mininet-labs/Mininet/blob/04bc7283b15f3f889c9ae22bed293d8e3dce99fc/crates/mini-bearer/src/discovery.rs); [`crates/mini-bearer/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/04bc7283b15f3f889c9ae22bed293d8e3dce99fc/crates/mini-bearer/src/lib.rs); [`crates/mini-sync/tests/sync_over_tcp.rs`](https://github.com/mininet-labs/Mininet/blob/04bc7283b15f3f889c9ae22bed293d8e3dce99fc/crates/mini-sync/tests/sync_over_tcp.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0129"></a>

## PR #129: mini-net: peer exchange (PEX) discovery over real TCP (D-0092)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-06, FD-09, FD-11, FD-14.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/129) | [Files changed](https://github.com/mininet-labs/Mininet/pull/129/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/39f3d550e0b9661fad7ec39058cbfa4ed0d6d540)

Head `39f3d550e0b9661fad7ec39058cbfa4ed0d6d540`; base `9a704d73925d0634b4e9c7bdd57cf828d92f8a7e`; merge `808f405e52f35d6ad3c2e9880c0ec831493f9870`. 10 changed files; 1 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This supplies the missing mapping from routing positions to candidate socket addresses and defines peer-exchange messages. It helps a node grow beyond its initially supplied peers without requiring a directory operator.

**Mechanism and evidence.** PeerRecord pairs PeerId with SocketAddr. AddressBook uses first-seen-wins, and bounded PexMessage request/response codecs reject malformed input. build_response/absorb_response are pure logic, exercised over TCP by tests; the production networking adapter is a later integration.

**What remains weaker than the intended claim.** First-seen-wins prevents later redirection but can permanently retain an attacker's first lie or a stale address. The observed source port of an outbound TCP connection is commonly ephemeral, not the peer's listening port. Bounding records per response does not automatically bound total AddressBook growth across repeated responses. Neither a PeerId nor an address proves independent ownership.

**Recommended improvement and rationale.** Use signed, expiring endpoint advertisements when identity binding is needed, with local freshness pins and bounded diverse candidate selection. Separate observed source address from a claimed listening endpoint and verify reachability safely. Add replacement/expiry policy so anti-redirection does not become permanent first-contact capture.

**Concrete example.** Bob connects from source port 53124 while listening on 9000. Recording peer_addr as a dialable endpoint tells Alice to call a transient port after that connection closes. A genuine peer can therefore be undiscoverable without any attacker.

**Acceptance tests to implement.** Test ephemeral-port callers, address rotation, hostile first insertion, repeated bounded responses that grow total state, self-record exclusion and duplicate IDs. Test fallback across independent candidate sources and confirm no PEX response can authorize a validator or a release.

**History, supersession and integration.** #318 supplies a consensus-side encrypted adapter. #296 independently introduces signed advertisements and anti-eclipse selection; consolidate their host-facing use instead of leaving two incompatible discovery stories.

**Source entry points.** [`crates/mini-net/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/39f3d550e0b9661fad7ec39058cbfa4ed0d6d540/crates/mini-net/src/error.rs); [`crates/mini-net/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/39f3d550e0b9661fad7ec39058cbfa4ed0d6d540/crates/mini-net/src/lib.rs); [`crates/mini-net/src/pex.rs`](https://github.com/mininet-labs/Mininet/blob/39f3d550e0b9661fad7ec39058cbfa4ed0d6d540/crates/mini-net/src/pex.rs); [`crates/mini-net/tests/pex_over_tcp.rs`](https://github.com/mininet-labs/Mininet/blob/39f3d550e0b9661fad7ec39058cbfa4ed0d6d540/crates/mini-net/tests/pex_over_tcp.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0130"></a>

## PR #130: mini-consensus: state sync / catch-up over real TCP (D-0093)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-04, FD-05, FD-06, FD-11.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/130) | [Files changed](https://github.com/mininet-labs/Mininet/pull/130/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/3b6d18262c86fd02ec4976dd1b5b5779ce65e432)

Head `3b6d18262c86fd02ec4976dd1b5b5779ce65e432`; base `808f405e52f35d6ad3c2e9880c0ec831493f9870`; merge `e92e59f14025b095f45cf07faebe4edf83d7ab26`. 13 changed files; 2 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This lets a node that missed a height obtain finalized history instead of remaining permanently behind. It is a concrete step from a network demo toward a service that can recover from ordinary outages.

**Mechanism and evidence.** CatchupRequest/Response and FinalizedBlock reuse existing body/header encoders. ConsensusNode retains finalized blocks, serves a bounded slice and applies received blocks through LedgerChain::apply_finalized_block, rechecking every certificate. Public TCP helpers use the existing encrypted Channel.

**What remains weaker than the intended claim.** The initial history is unbounded and in-memory; a restart loses it, and a very old node must replay everything. Static validator sets and current KEL resolution do not prove historic set transitions or long-range recovery. A valid prefix followed by a bad block also requires an explicit all-or-nothing or documented prefix-commit policy.

**Recommended improvement and rationale.** Add authenticated checkpoints, bounded persistent archives and independently verified suffixes without trusting the serving peer. Define atomic adoption, pruning and retry semantics and measure memory/flash cost on weak nodes. Keep local trust anchors explicit; do not replace missing history with a checkpoint server's assertion.

**Concrete example.** A fifth non-validator can catch up by verifying three-of-four certificates; it need not gain a vote to read the chain. If the serving peer inserts a forged block after ten valid ones, the receiver must follow a documented state-update policy and never accept the forged transition.

**Acceptance tests to implement.** Test gaps, duplicate/reordered blocks, invalid late certificates, wrong network, stale key state, restart during adoption and history larger than RAM. Verify a non-validator receiver reaches the same state without acquiring validator membership.

**History, supersession and integration.** #289 adds persistent authenticated snapshots and atomic adoption; #300 closes exact-body substitution; #326 chunks the transfer. Dynamic membership and weak-subjectivity policy remain separate requirements.

**Source entry points.** [`crates/mini-consensus/src/catchup.rs`](https://github.com/mininet-labs/Mininet/blob/3b6d18262c86fd02ec4976dd1b5b5779ce65e432/crates/mini-consensus/src/catchup.rs); [`crates/mini-consensus/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/3b6d18262c86fd02ec4976dd1b5b5779ce65e432/crates/mini-consensus/src/error.rs); [`crates/mini-consensus/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/3b6d18262c86fd02ec4976dd1b5b5779ce65e432/crates/mini-consensus/src/lib.rs); [`crates/mini-consensus/src/net.rs`](https://github.com/mininet-labs/Mininet/blob/3b6d18262c86fd02ec4976dd1b5b5779ce65e432/crates/mini-consensus/src/net.rs); [`crates/mini-consensus/src/node.rs`](https://github.com/mininet-labs/Mininet/blob/3b6d18262c86fd02ec4976dd1b5b5779ce65e432/crates/mini-consensus/src/node.rs); [`crates/mini-consensus/src/wire.rs`](https://github.com/mininet-labs/Mininet/blob/3b6d18262c86fd02ec4976dd1b5b5779ce65e432/crates/mini-consensus/src/wire.rs); [`crates/mini-consensus/tests/networked_consensus.rs`](https://github.com/mininet-labs/Mininet/blob/3b6d18262c86fd02ec4976dd1b5b5779ce65e432/crates/mini-consensus/tests/networked_consensus.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0131"></a>

## PR #131: Adopt founder research V2 (cost doctrine); ship mini-privacy-policy (D-0094)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-01, FD-09, FD-10, FD-11, FD-14, FD-16.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/131) | [Files changed](https://github.com/mininet-labs/Mininet/pull/131/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/e47109de8baaeca2da3f7b88afab106bea24a8c4)

Head `e47109de8baaeca2da3f7b88afab106bea24a8c4`; base `e92e59f14025b095f45cf07faebe4edf83d7ab26`; merge `53dd73f15df3a55ea4efaf6322103d7e37e4b8dc`. 16 changed files; 3 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This introduces explicit privacy-property and residual-risk vocabulary, plus a parallel work plan. It is useful because privacy claims can otherwise collapse many different adversaries into one reassuring label. The economic framing must remain subordinate to the later public-commons entitlement doctrine.

**Mechanism and evidence.** mini-privacy-policy defines requested properties, achieved results, residual floors and Direct/Relayed/Mixed/Burst tiers. Its codec is bounded policy data. expected_cost reproduces research estimates; no relay, mixnet or replication mechanism is implemented by this PR.

**What remains weaker than the intended claim.** A tier enum or a cost curve cannot establish achieved protection. Calling all privacy a priced purchase risks making basic safety conditional on wealth unless minimum rights and resource costs are kept separate. The concurrency plan's disjoint crate footprints still share registries and generated files; later decision collisions show that directory planning alone is not coordination.

**Recommended improvement and rationale.** Make achieved protection an output of actual execution evidence, not a caller-created desired tier. Preserve explicit residual floors and reject unsupported mechanisms. Benchmark costs and disclose uncertainty. Keep free public participation and a non-degrading baseline distinct from optional extra services; reserve decisions atomically or use collision-resistant identifiers.

**Concrete example.** A user selecting Mixed should receive an unsupported/unreviewed result until a reviewed mix executor exists, not a receipt claiming traffic-analysis resistance because the selected enum has a higher price. A zero-wallet user must not lose ordinary speech or identity rights.

**Acceptance tests to implement.** Test every property/tier combination, unknown tags, malformed encodings and unsupported execution paths. Compare declared cost to measured weakest-device resource usage. Exercise concurrent work claims and decision allocation rather than assuming non-overlapping feature files prevent all conflicts.

**History, supersession and integration.** #138 implements route policy; #143 consolidates privacy/evidence lanes; #145/#146 add relay planning; #243-#246 define commons and publication boundaries; #296 supplies a real onion path while Mixed/Burst remain unimplemented.

**Source entry points.** [`crates/mini-privacy-policy/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/e47109de8baaeca2da3f7b88afab106bea24a8c4/crates/mini-privacy-policy/src/error.rs); [`crates/mini-privacy-policy/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/e47109de8baaeca2da3f7b88afab106bea24a8c4/crates/mini-privacy-policy/src/lib.rs); [`crates/mini-privacy-policy/src/tier.rs`](https://github.com/mininet-labs/Mininet/blob/e47109de8baaeca2da3f7b88afab106bea24a8c4/crates/mini-privacy-policy/src/tier.rs); [`crates/mini-privacy-policy/src/vocabulary.rs`](https://github.com/mininet-labs/Mininet/blob/e47109de8baaeca2da3f7b88afab106bea24a8c4/crates/mini-privacy-policy/src/vocabulary.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0138"></a>

## PR #138: mini-transport-policy: TransportRequest policy router (L2, MN-201, D-0301)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-09, FD-10, FD-11, FD-14.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/138) | [Files changed](https://github.com/mininet-labs/Mininet/pull/138/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/216c26c2a66f4f27963a055d95689c3634c28f93)

Head `216c26c2a66f4f27963a055d95689c3634c28f93`; base `e92e59f14025b095f45cf07faebe4edf83d7ab26`; merge `59e4299aa85dcfeea6803b3f719c8154d18b2919`. 20 changed files; 4 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This turns the privacy vocabulary into an explicit route-planning decision and rejects requests whose selected tier is below the policy table's requirement. It is a useful separation between what a user asks for and which mechanisms would be needed.

**Mechanism and evidence.** TransportRequest combines a PrivacyRequest with a payload size class. route consults property_min_tier and mechanisms_for_tier, returning UnsatisfiableProperty on under-provisioning. The crate has no socket or relay executor. The stacked diff includes #131; that inherited work must not be double-counted as a new implementation.

**What remains weaker than the intended claim.** The returned field is named AchievedPrivacy even though no transport ran. Future unknown properties default to Burst rather than Unsupported, so the highest tier can appear to satisfy a property the router does not understand. Some properties also depend on identity, payment or storage mechanisms that a transport tier alone cannot supply.

**Recommended improvement and rationale.** Rename the output PlannedProtection or otherwise make planning and execution receipts different types. Reject unknown properties explicitly. Require runtime evidence from each relevant subsystem before producing an achieved result; price and tier labels must never substitute for that evidence.

**Concrete example.** A request for an unknown future property at Burst should not succeed merely because Burst is the highest enum value. Likewise, choosing a transport that can carry human evidence does not prove the evidence establishes a unique person.

**Acceptance tests to implement.** Test every known property against every tier, explicit unsupported-property behavior, and a runtime that cannot execute a planned mechanism. Verify no API turns a plan into a success receipt without actual execution and no caller silently drops a requested protection to save resources.

**History, supersession and integration.** #145/#146 add relay planning and a live hop-by-hop demonstration; #246 composes publication planning; #296 implements a different real onion path. The missing distinction between planned and achieved protection remains important across those layers.

**Source entry points.** [`crates/mini-privacy-policy/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/216c26c2a66f4f27963a055d95689c3634c28f93/crates/mini-privacy-policy/src/error.rs); [`crates/mini-privacy-policy/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/216c26c2a66f4f27963a055d95689c3634c28f93/crates/mini-privacy-policy/src/lib.rs); [`crates/mini-privacy-policy/src/tier.rs`](https://github.com/mininet-labs/Mininet/blob/216c26c2a66f4f27963a055d95689c3634c28f93/crates/mini-privacy-policy/src/tier.rs); [`crates/mini-privacy-policy/src/vocabulary.rs`](https://github.com/mininet-labs/Mininet/blob/216c26c2a66f4f27963a055d95689c3634c28f93/crates/mini-privacy-policy/src/vocabulary.rs); [`crates/mini-transport-policy/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/216c26c2a66f4f27963a055d95689c3634c28f93/crates/mini-transport-policy/src/error.rs); [`crates/mini-transport-policy/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/216c26c2a66f4f27963a055d95689c3634c28f93/crates/mini-transport-policy/src/lib.rs); [`crates/mini-transport-policy/src/router.rs`](https://github.com/mininet-labs/Mininet/blob/216c26c2a66f4f27963a055d95689c3634c28f93/crates/mini-transport-policy/src/router.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 1 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0139"></a>

## PR #139: mini-resource-pricing: PriceVector/quote engine (L4, MN-601, D-0302)

**PARTIAL** | Captured outcome: **closed** | Directives: FD-04, FD-09, FD-10, FD-11, FD-16.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/139) | [Files changed](https://github.com/mininet-labs/Mininet/pull/139/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/40c894c979b395238ac910246a501e726e04f0b0)

Head `40c894c979b395238ac910246a501e726e04f0b0`; base `e92e59f14025b095f45cf07faebe4edf83d7ab26`; merge `none`. 20 changed files; 4 commits; 1 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This closed-unmerged pricing lane contributes the implementation later consolidated into #143. Its value is checked, deterministic resource quotation that is kept separate from payment execution and political authority.

**Mechanism and evidence.** PriceVector contains bandwidth and storage unit prices. quote applies declared tier multiplier ranges, using u128 intermediates and checked conversion/addition to produce minimum and maximum micro-MINI amounts. It has no ledger-writing, signing or governance path. The closure comment identifies the exact cherry-picked contribution.

**What remains weaker than the intended claim.** The quoted range derives from research estimates, not measured cost or a market-clearing price. Public price fields do not prove governance approval. Whole-megabyte and day inputs leave rounding and sub-unit charging to callers. A Direct quote can have a numerical resource cost while requires_payment is false, which consumers must not reinterpret as a paywall.

**Recommended improvement and rationale.** Use explicit byte/time/amount units and a documented rounding rule at the external interface. Bind accepted commercial quotes to an exact service, provider, expiry and user consent without making those commercial fields governance inputs. Keep the no-payment baseline separate from an estimate of physical resource use.

**Concrete example.** A 1-byte request rounded down to zero megabytes and a 999,999-byte request can both appear free in a naive adapter. A different adapter rounding up charges both for a megabyte. The protocol should not leave that discrepancy hidden in callers.

**Acceptance tests to implement.** Cross-check extreme products and conversions with an independent integer reference. Test sub-unit quantities, min/max ordering, zero payload, zero prices, overflow, expired accepted quotes and a zero-wallet public action. Confirm no quote or payment changes ranking or approval weight.

**History, supersession and integration.** Adopted through #143 rather than merged here. #245 later implements the explicit boundary that never quotes the Direct commons tier. #151 develops privacy-preserving resource-payment research; no token system is supplied by this quote engine.

**Source entry points.** [`crates/mini-privacy-policy/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/40c894c979b395238ac910246a501e726e04f0b0/crates/mini-privacy-policy/src/error.rs); [`crates/mini-privacy-policy/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/40c894c979b395238ac910246a501e726e04f0b0/crates/mini-privacy-policy/src/lib.rs); [`crates/mini-privacy-policy/src/tier.rs`](https://github.com/mininet-labs/Mininet/blob/40c894c979b395238ac910246a501e726e04f0b0/crates/mini-privacy-policy/src/tier.rs); [`crates/mini-privacy-policy/src/vocabulary.rs`](https://github.com/mininet-labs/Mininet/blob/40c894c979b395238ac910246a501e726e04f0b0/crates/mini-privacy-policy/src/vocabulary.rs); [`crates/mini-resource-pricing/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/40c894c979b395238ac910246a501e726e04f0b0/crates/mini-resource-pricing/src/error.rs); [`crates/mini-resource-pricing/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/40c894c979b395238ac910246a501e726e04f0b0/crates/mini-resource-pricing/src/lib.rs); [`crates/mini-resource-pricing/src/quote.rs`](https://github.com/mininet-labs/Mininet/blob/40c894c979b395238ac910246a501e726e04f0b0/crates/mini-resource-pricing/src/quote.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0140"></a>

## PR #140: Human-evidence taxonomy reconciliation: no rival taxonomy (L5, MN-401, D-0303)

**PASS** | Captured outcome: **closed** | Directives: FD-01, FD-08, FD-09, FD-14, FD-16.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/140) | [Files changed](https://github.com/mininet-labs/Mininet/pull/140/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/f4e9fa533e417afe5eabd748bec59f6494b970f6)

Head `f4e9fa533e417afe5eabd748bec59f6494b970f6`; base `e92e59f14025b095f45cf07faebe4edf83d7ab26`; merge `none`. 17 changed files; 4 commits; 1 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This closed-unmerged lane prevents a new research vocabulary from becoming a rival personhood authority ladder. It recognizes that participation, evidence confidence and external provenance are different axes and chooses a documented mapping instead of adding misleading status variants.

**Mechanism and evidence.** The proposal maps confidence labels onto existing HumanStatus values but deliberately gives ActiveParticipant and ExternalUniquenessBacked no promotion-equivalent status. External issuers remain one SignalEvidence source. mini-uniqueness behavior is unchanged; #143 carries the documentation into main.

**What remains weaker than the intended claim.** The sound restraint here should not be described as solving uniqueness. A credential standard can make an issuer's assertion portable and authentic without making the assertion true or the issuer independent. Even the surviving VouchedHuman/EvidenceQualifiedHuman names need clear UI and policy context.

**Recommended improvement and rationale.** Preserve the orthogonal classification in future APIs: evidence provenance, confidence under a named policy, and authority eligibility should remain separate. An external issuer must be replaceable and unable to revoke core participation. Record the threat and error model for every source before changing qualification weights.

**Concrete example.** A government-issued credential may be correctly signed yet unavailable to a stateless person or issued twice to a privileged operator. It can be one scoped piece of evidence; it cannot become the sole gateway to being counted as human on Mininet.

**Acceptance tests to implement.** Test that adding an external source cannot create a new status or bypass required evidence. Examine policy consumers for hidden external-issuer allowlists or identity-provider dependencies. Include people without documents, unusual devices or stable locations in the validation population.

**History, supersession and integration.** This follows #123/#126 and is adopted by #143. #220 later expands the personhood research program. The PASS is for preventing taxonomy/authority inflation, not for certifying any existing human-evidence mechanism.

**Source entry points.** [`crates/mini-privacy-policy/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/f4e9fa533e417afe5eabd748bec59f6494b970f6/crates/mini-privacy-policy/src/error.rs); [`crates/mini-privacy-policy/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/f4e9fa533e417afe5eabd748bec59f6494b970f6/crates/mini-privacy-policy/src/lib.rs); [`crates/mini-privacy-policy/src/tier.rs`](https://github.com/mininet-labs/Mininet/blob/f4e9fa533e417afe5eabd748bec59f6494b970f6/crates/mini-privacy-policy/src/tier.rs); [`crates/mini-privacy-policy/src/vocabulary.rs`](https://github.com/mininet-labs/Mininet/blob/f4e9fa533e417afe5eabd748bec59f6494b970f6/crates/mini-privacy-policy/src/vocabulary.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0141"></a>

## PR #141: ObjectEnvelope v2 + capability grants + scoped pseudonyms (L1, D-0304)

**PARTIAL** | Captured outcome: **closed** | Directives: FD-02, FD-09, FD-12, FD-14, FD-15.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/141) | [Files changed](https://github.com/mininet-labs/Mininet/pull/141/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/9109628ec60b0200ff7b051ab38d927c0c59d3d8)

Head `9109628ec60b0200ff7b051ab38d927c0c59d3d8`; base `e92e59f14025b095f45cf07faebe4edf83d7ab26`; merge `none`. 25 changed files; 4 commits; 1 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This closed-unmerged lane addresses a real architectural privacy leak: encrypting only a payload leaves author, links, sequence and timestamps visible. It also introduces narrowly scoped capabilities and purpose-separated pseudonyms. #143 is the merged carrier of the same work.

**Mechanism and evidence.** ObjectEnvelopeV2 places the private object inside AEAD ciphertext and binds its public routing/retention fields as associated data. CapabilityGrant uses independent rights, exact object scope, token commitment, issuer signatures and holder proof. Scoped pseudonyms reuse the existing pairwise identity mechanism instead of adding a second derivation.

**What remains weaker than the intended claim.** Envelope confidentiality does not solve key distribution, revocation, padding or traffic analysis. CapabilityGrant::validate authenticates the stated issuer but receives no authoritative object-owner/issuer policy; callers must still establish that this issuer may grant rights over that object. Static holder proofs are grant-bound, not fresh session proofs. Current signature-byte caps also need alignment with the later ML-DSA suite.

**Recommended improvement and rationale.** Separate a valid signed grant from an authorized capability. Require an expected issuer/owner policy derived from authenticated object state, explicit revocation/freshness rules and session-bound holder challenges where replay matters. Review metadata at the entire storage/sync/application path, not just the envelope codec. Preserve independent rights: Administer must not silently imply Read.

**Concrete example.** Mallory can sign a perfectly valid grant naming Alice's object and present Mallory's own issuer KEL. The grant's cryptographic checks can pass; an object service must independently reject Mallory as an unauthorized issuer. A token and signature are not ownership.

**Acceptance tests to implement.** Test unauthorized-but-valid issuers, expired/revoked grants, replayed holder proofs across sessions, rights substitution, all public-field tampering and envelope size leakage. After PQ support, round-trip legitimate larger signatures through every object and capability codec.

**History, supersession and integration.** #143 merges this lane; #170 integrates v2 private envelopes into messaging, storage and private sync; #299 fixes some shared decoder-count rules. Those later changes must be checked for byte-size limits as well as count limits.

**Source entry points.** [`crates/mini-objects/src/capability.rs`](https://github.com/mininet-labs/Mininet/blob/9109628ec60b0200ff7b051ab38d927c0c59d3d8/crates/mini-objects/src/capability.rs); [`crates/mini-objects/src/codec.rs`](https://github.com/mininet-labs/Mininet/blob/9109628ec60b0200ff7b051ab38d927c0c59d3d8/crates/mini-objects/src/codec.rs); [`crates/mini-objects/src/envelope_v2.rs`](https://github.com/mininet-labs/Mininet/blob/9109628ec60b0200ff7b051ab38d927c0c59d3d8/crates/mini-objects/src/envelope_v2.rs); [`crates/mini-objects/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/9109628ec60b0200ff7b051ab38d927c0c59d3d8/crates/mini-objects/src/error.rs); [`crates/mini-objects/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/9109628ec60b0200ff7b051ab38d927c0c59d3d8/crates/mini-objects/src/lib.rs); [`crates/mini-objects/src/object.rs`](https://github.com/mininet-labs/Mininet/blob/9109628ec60b0200ff7b051ab38d927c0c59d3d8/crates/mini-objects/src/object.rs); [`crates/mini-objects/src/private_object.rs`](https://github.com/mininet-labs/Mininet/blob/9109628ec60b0200ff7b051ab38d927c0c59d3d8/crates/mini-objects/src/private_object.rs); [`crates/mini-objects/src/pseudonym.rs`](https://github.com/mininet-labs/Mininet/blob/9109628ec60b0200ff7b051ab38d927c0c59d3d8/crates/mini-objects/src/pseudonym.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 2 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0142"></a>

## PR #142: Sphinx-style mix network research report + protocol spec (L3, MN-204, D-0305)

**PARTIAL** | Captured outcome: **closed** | Directives: FD-02, FD-09, FD-10, FD-11, FD-14, FD-15.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/142) | [Files changed](https://github.com/mininet-labs/Mininet/pull/142/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/e73d812f6eae2482d5192fac84dee1b5bac244d1)

Head `e73d812f6eae2482d5192fac84dee1b5bac244d1`; base `e92e59f14025b095f45cf07faebe4edf83d7ab26`; merge `none`. 17 changed files; 4 commits; 1 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This closed-unmerged research lane gives the mixnet idea an adversary model, candidate protocol and explicit simulation backlog. It correctly keeps operational anonymity gated rather than implementing a packet named Sphinx and declaring the problem solved.

**Mechanism and evidence.** The document compares low-latency routing and mixing, names active/timing/route-capture attacks and proposes fixed-size Sphinx-style packets with Loopix-style delays and cover traffic. It records thirteen simulations and an external-review prerequisite. #143 adopts the research document.

**What remains weaker than the intended claim.** The survey's broad historical claims and unverified bibliographic details need primary-source checking before external audit use. Sphinx is a specific packet construction, not a synonym for layered AEAD; changing header blinding, authentication or payload handling can invalidate inherited arguments. Cover-traffic estimates and topology choices are proposals, not measured protection.

**Recommended improvement and rationale.** Write an exact candidate suite and packet/state-machine specification, including replay tags, key compromise, reply blocks, routing selection and cover-traffic scheduling. Compare independently maintained implementations and their actual licenses. Precommit simulation scenarios and privacy/cost metrics, then decide whether the construction meets the weakest-device budget without reducing the promised adversary model.

**Concrete example.** A three-hop onion with immediate forwarding can hide content from intermediate relays yet remain highly correlatable by a global timing observer. Adding a Sphinx label or a higher quoted tier does not supply mixing delays, cover traffic or an adequate anonymity population.

**Acceptance tests to implement.** Simulate sparse traffic, malicious route concentration, correlated clouds/ASes, selective dropping, replay, tagging, mobile sleep and intersection attacks. Report latency, energy and adversary advantage together. Confirm any unsupported Mixed/Burst execution remains unavailable rather than downgraded.

**History, supersession and integration.** Consolidated in #143. #296 explicitly states its real three-hop onion is not Sphinx and makes no global-observer anonymity claim. That distinction must survive product copy and future implementation work.

**Source entry points.** [`crates/mini-privacy-policy/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/e73d812f6eae2482d5192fac84dee1b5bac244d1/crates/mini-privacy-policy/src/error.rs); [`crates/mini-privacy-policy/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/e73d812f6eae2482d5192fac84dee1b5bac244d1/crates/mini-privacy-policy/src/lib.rs); [`crates/mini-privacy-policy/src/tier.rs`](https://github.com/mininet-labs/Mininet/blob/e73d812f6eae2482d5192fac84dee1b5bac244d1/crates/mini-privacy-policy/src/tier.rs); [`crates/mini-privacy-policy/src/vocabulary.rs`](https://github.com/mininet-labs/Mininet/blob/e73d812f6eae2482d5192fac84dee1b5bac244d1/crates/mini-privacy-policy/src/vocabulary.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0143"></a>

## PR #143: Privacy/cost-doctrine lanes L1, L3, L4, L5 (consolidated): object privacy boundary, mixnet research, resource pricing, human-evidence taxonomy

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-08, FD-09, FD-10, FD-12, FD-14, FD-16.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/143) | [Files changed](https://github.com/mininet-labs/Mininet/pull/143/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/e9d539edcae6e73156d77e9bbdac74854b7dfb69)

Head `e9d539edcae6e73156d77e9bbdac74854b7dfb69`; base `59e4299aa85dcfeea6803b3f719c8154d18b2919`; merge `2ce8f3488cc0ce8dc4718382dff6b9fd639eddd6`. 23 changed files; 4 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This is a consolidation and integration PR, not four newly invented features. It carries the substantive work from #139-#142 onto a compatible mainline while preserving the rejected branches' contribution history.

**Mechanism and evidence.** The combined diff introduces the private-envelope/capability/pseudonym code, resource-price engine, evidence-taxonomy reconciliation and Sphinx research. The stated mechanism is cherry-picking the four lane commits and resolving shared decision/status/generated-file conflicts without semantic changes.

**What remains weaker than the intended claim.** Integration has its own risk: a clean merge does not prove every lane's behavior, limits or decision text survived. The private capability boundary is security-sensitive while two other lanes are documentation-only; one whole-PR label can conceal those different obligations. The original lanes' limitations remain, including unimplemented key distribution/revocation and unmeasured mixnet costs.

**Recommended improvement and rationale.** Preserve a per-lane equivalence/disposition record and check the final combined tree, not just each predecessor branch. Re-run cross-crate tests and verify all historical decisions remain present once. Avoid duplicated generated navigation churn obscuring substantive review. Carry every original limitation into the consolidated status narrative.

**Concrete example.** The original lane tests can all pass individually while a conflict resolution drops one rejection path or removes a decision entry. The relevant review artifact is the consolidated exact-head diff plus comparison to each source commit, not the statement no content lost.

**Acceptance tests to implement.** Compare each lane's effective source changes against its adopted commit; inspect conflict resolutions; test envelope/capability, quote and taxonomy boundaries on the final tree. Verify closed PRs #139-#142 link here and are not counted as either rejected ideas or four extra deployed systems.

**History, supersession and integration.** #131/#138 establish shared policy/router prerequisites. #145 onward begins relay/runtime integration. This report keeps separate dossiers for every predecessor and this integration PR so consolidation does not erase accountability.

**Source entry points.** [`crates/mini-objects/src/capability.rs`](https://github.com/mininet-labs/Mininet/blob/e9d539edcae6e73156d77e9bbdac74854b7dfb69/crates/mini-objects/src/capability.rs); [`crates/mini-objects/src/codec.rs`](https://github.com/mininet-labs/Mininet/blob/e9d539edcae6e73156d77e9bbdac74854b7dfb69/crates/mini-objects/src/codec.rs); [`crates/mini-objects/src/envelope_v2.rs`](https://github.com/mininet-labs/Mininet/blob/e9d539edcae6e73156d77e9bbdac74854b7dfb69/crates/mini-objects/src/envelope_v2.rs); [`crates/mini-objects/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/e9d539edcae6e73156d77e9bbdac74854b7dfb69/crates/mini-objects/src/error.rs); [`crates/mini-objects/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/e9d539edcae6e73156d77e9bbdac74854b7dfb69/crates/mini-objects/src/lib.rs); [`crates/mini-objects/src/object.rs`](https://github.com/mininet-labs/Mininet/blob/e9d539edcae6e73156d77e9bbdac74854b7dfb69/crates/mini-objects/src/object.rs); [`crates/mini-objects/src/private_object.rs`](https://github.com/mininet-labs/Mininet/blob/e9d539edcae6e73156d77e9bbdac74854b7dfb69/crates/mini-objects/src/private_object.rs); [`crates/mini-objects/src/pseudonym.rs`](https://github.com/mininet-labs/Mininet/blob/e9d539edcae6e73156d77e9bbdac74854b7dfb69/crates/mini-objects/src/pseudonym.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 1 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0145"></a>

## PR #145: mini-relay: Tier 1 relay + rendezvous protocol (L6, MN-202, D-0306)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-09, FD-11, FD-14, FD-15.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/145) | [Files changed](https://github.com/mininet-labs/Mininet/pull/145/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/72c5d4de5f3548c05df6ea198d3f9842e455ef63)

Head `72c5d4de5f3548c05df6ea198d3f9842e455ef63`; base `2ce8f3488cc0ce8dc4718382dff6b9fd639eddd6`; merge `1d824879d99ce809323db546e2600a1ef3296878`. 17 changed files; 1 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This introduces a relay/mailbox vocabulary without installing a central relay registry. It separates entry, rendezvous and delivery roles and uses per-connection pseudonyms rather than requiring a public human root in the transport role identifier.

**Mechanism and evidence.** RelayEnvelope seals one hop over Channel with role, connection and size class in associated data. MailboxGrant/Token apply typed holder-bound access checks. enforce_role_separation rejects the same relay identity assigned multiple required roles. The crate initially has no live multiprocess relay path.

**What remains weaker than the intended claim.** Different relay identities do not establish different operators, and per-hop encryption does not prevent a relay from seeing the plaintext it forwards. Mailbox issuer authorization and revocation remain separate obligations. DHT privacy is deferred because the repository does not yet have the provider-record PUT/GET layer the proposed restriction would govern.

**Recommended improvement and rationale.** Document exactly what each role learns and make destination encryption independent of hop encryption. Bind mailbox issuer authority to the mailbox's authenticated owner policy, add bounded expiry/revocation semantics and measure state growth. Treat role diversity as a policy assumption to test, not a theorem from DID inequality.

**Concrete example.** One operator can create three role-specific pseudonyms and satisfy identity-distinctness while observing every hop. Alternatively, two honest hops can still expose the message to the entry if the design unwraps and reseals plaintext between them.

**Acceptance tests to implement.** Test same-operator role simulations, missing roles, copied mailbox grants, unauthorized issuers, token leakage without the holder key, replay and state exhaustion. A live test should inspect what each process can decrypt, not only whether the final recipient receives the right bytes.

**History, supersession and integration.** #146 wires route planning and a real hop-by-hop demonstration; #296 later adds a destination-encrypted onion. The two constructions have different privacy guarantees and must not be merged in narrative.

**Source entry points.** [`crates/mini-relay/src/codec.rs`](https://github.com/mininet-labs/Mininet/blob/72c5d4de5f3548c05df6ea198d3f9842e455ef63/crates/mini-relay/src/codec.rs); [`crates/mini-relay/src/connection.rs`](https://github.com/mininet-labs/Mininet/blob/72c5d4de5f3548c05df6ea198d3f9842e455ef63/crates/mini-relay/src/connection.rs); [`crates/mini-relay/src/envelope.rs`](https://github.com/mininet-labs/Mininet/blob/72c5d4de5f3548c05df6ea198d3f9842e455ef63/crates/mini-relay/src/envelope.rs); [`crates/mini-relay/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/72c5d4de5f3548c05df6ea198d3f9842e455ef63/crates/mini-relay/src/error.rs); [`crates/mini-relay/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/72c5d4de5f3548c05df6ea198d3f9842e455ef63/crates/mini-relay/src/lib.rs); [`crates/mini-relay/src/mailbox.rs`](https://github.com/mininet-labs/Mininet/blob/72c5d4de5f3548c05df6ea198d3f9842e455ef63/crates/mini-relay/src/mailbox.rs); [`crates/mini-relay/src/role.rs`](https://github.com/mininet-labs/Mininet/blob/72c5d4de5f3548c05df6ea198d3f9842e455ef63/crates/mini-relay/src/role.rs); [`crates/mini-relay/src/role_separation.rs`](https://github.com/mininet-labs/Mininet/blob/72c5d4de5f3548c05df6ea198d3f9842e455ef63/crates/mini-relay/src/role_separation.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0146"></a>

## PR #146: mini-relay follow-up: route-decision wiring + live TCP demo (D-0307, D-0308)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-06, FD-09, FD-10, FD-14.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/146) | [Files changed](https://github.com/mininet-labs/Mininet/pull/146/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/48dded5892d003cfc9d0b9b30a0b9614a5f19dfb)

Head `48dded5892d003cfc9d0b9b30a0b9614a5f19dfb`; base `1d824879d99ce809323db546e2600a1ef3296878`; merge `a2b3eb57d998087a2a152d0ab1177ca88fdf4bf0`. 11 changed files; 2 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This connects route-policy output to relay role planning and proves two real socket hops work. Its valuable honesty is explicitly describing the demonstration as hop-by-hop store-and-forward rather than onion routing.

**Mechanism and evidence.** roles_for_route_decision accepts a Relayed decision naming OnionRelay and produces Entry/Rendezvous roles. Direct and Mixed/Burst return distinct errors. The live test unwraps a RelayEnvelope at the entry and reseals toward rendezvous over a separately established Channel.

**What remains weaker than the intended claim.** The entry sees forwarded plaintext, so the mechanism name OnionRelay is stronger than the demonstrated transport. A two-hop success test does not establish mailbox pickup, source hiding against colluding relays or traffic-analysis resistance. Role planning returns a plan, not a verified choice of independent operators.

**Recommended improvement and rationale.** Align enum names and achieved-result descriptions with actual forwarding behavior. Require end-to-end destination encryption before carrying sensitive application data. Add the delivery/mailbox leg and bounded retry/cancellation as a coherent end-to-end scenario; keep unimplemented mix tiers fail closed.

**Concrete example.** If the entry can print the plaintext after open and before seal, the system has link confidentiality but not confidentiality from that relay. A correct destination-encrypted design should leave only opaque application ciphertext at both intermediate processes.

**Acceptance tests to implement.** Instrument the test so relays cannot decrypt application content; test wrong-hop AAD, counter desynchronization, disconnects and malformed role plans. Verify Direct/Mixed/Burst cannot accidentally enter the demonstrated Relayed execution path and that failure never silently falls back to direct delivery.

**History, supersession and integration.** This completes specific #145 follow-ups. #296 supplies a stronger onion composition. Historical descriptions must retain that #146 was not the later three-hop design.

**Source entry points.** [`crates/mini-relay/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/48dded5892d003cfc9d0b9b30a0b9614a5f19dfb/crates/mini-relay/src/error.rs); [`crates/mini-relay/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/48dded5892d003cfc9d0b9b30a0b9614a5f19dfb/crates/mini-relay/src/lib.rs); [`crates/mini-relay/src/plan.rs`](https://github.com/mininet-labs/Mininet/blob/48dded5892d003cfc9d0b9b30a0b9614a5f19dfb/crates/mini-relay/src/plan.rs); [`crates/mini-relay/tests/live_relay_over_tcp.rs`](https://github.com/mininet-labs/Mininet/blob/48dded5892d003cfc9d0b9b30a0b9614a5f19dfb/crates/mini-relay/tests/live_relay_over_tcp.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0147"></a>

## PR #147: mini-bridge + mini-private-index: pluggable transports (MN-207) and private-lookup boundary (MN-208)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-09, FD-10, FD-14, FD-18.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/147) | [Files changed](https://github.com/mininet-labs/Mininet/pull/147/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/c36c322d93e21315ffed31ccd01e342bb1e30baa)

Head `c36c322d93e21315ffed31ccd01e342bb1e30baa`; base `a2b3eb57d998087a2a152d0ab1177ca88fdf4bf0`; merge `ce61f03e99a5ea874c2e3e7f565301edd329a13e`. 29 changed files; 7 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This creates replaceable boundaries for circumvention transports and private lookup rather than embedding one provider as an identity authority. It implements a direct adapter and local capability-scoped index while leaving stronger named modes as explicit future work.

**Mechanism and evidence.** BridgeDescriptor is a signed expiring reachability claim; DirectBridgeTransport verifies it before connecting and performs Channel setup. Private lookup labels are HKDF-derived across purpose and epoch domains. LocalIndex enforces record writer/signature/rollback rules; only CapabilityScoped lookup is implemented.

**What remains weaker than the intended claim.** A signed descriptor proves who made a reachability claim, not that the remote endpoint actually controls the signer's key. A rotating opaque label still reveals access patterns to a server that sees requests. Capability tables for unimplemented obfs4/WebTunnel/Snowflake or PIR modes are declarations, not operational properties. Local record integrity does not prove database completeness.

**Recommended improvement and rationale.** Bind the descriptor to a channel-authenticated endpoint when such identity is required. Keep bridge providers optional and replaceable; minimize what the adapter process can access. Treat private lookup as a separate protocol with authenticated database roots, record padding and an explicit access-pattern threat model rather than renaming ordinary encrypted lookup PIR.

**Concrete example.** A server can observe that one client repeatedly asks for the same opaque label even though it cannot read the record. If a descriptor's endpoint is redirected, checking only its stored signature before an anonymous handshake does not authenticate the party now answering there.

**Acceptance tests to implement.** Test expired descriptors, endpoint substitution, writer hijacking, same-sequence different-record updates, repeated-label observations and cross-purpose label reuse. Ensure unsupported transports and PrivatePIR return explicit refusal and that removing every bridge still leaves non-bridge core operation intact.

**History, supersession and integration.** #150 adds process management; #151 defines PIR and anonymous-payment research workloads; #296 reuses this adapter boundary rather than introducing another bridge authority.

**Source entry points.** [`crates/mini-bridge/src/capabilities.rs`](https://github.com/mininet-labs/Mininet/blob/c36c322d93e21315ffed31ccd01e342bb1e30baa/crates/mini-bridge/src/capabilities.rs); [`crates/mini-bridge/src/codec.rs`](https://github.com/mininet-labs/Mininet/blob/c36c322d93e21315ffed31ccd01e342bb1e30baa/crates/mini-bridge/src/codec.rs); [`crates/mini-bridge/src/descriptor.rs`](https://github.com/mininet-labs/Mininet/blob/c36c322d93e21315ffed31ccd01e342bb1e30baa/crates/mini-bridge/src/descriptor.rs); [`crates/mini-bridge/src/direct.rs`](https://github.com/mininet-labs/Mininet/blob/c36c322d93e21315ffed31ccd01e342bb1e30baa/crates/mini-bridge/src/direct.rs); [`crates/mini-bridge/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/c36c322d93e21315ffed31ccd01e342bb1e30baa/crates/mini-bridge/src/error.rs); [`crates/mini-bridge/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/c36c322d93e21315ffed31ccd01e342bb1e30baa/crates/mini-bridge/src/lib.rs); [`crates/mini-bridge/src/transport.rs`](https://github.com/mininet-labs/Mininet/blob/c36c322d93e21315ffed31ccd01e342bb1e30baa/crates/mini-bridge/src/transport.rs); [`crates/mini-bridge/src/transport_id.rs`](https://github.com/mininet-labs/Mininet/blob/c36c322d93e21315ffed31ccd01e342bb1e30baa/crates/mini-bridge/src/transport_id.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 2 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0148"></a>

## PR #148: mini-crypto: SignatureSuite::MlDsa65 verify-only support (D-0095, issue #15 Phase 1)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-06, FD-09, FD-11, FD-13, FD-14.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/148) | [Files changed](https://github.com/mininet-labs/Mininet/pull/148/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/f847abdbd6698f7cf7b7083c5f50e7bdc3f47cf6)

Head `f847abdbd6698f7cf7b7083c5f50e7bdc3f47cf6`; base `82c66857a102e79acefec0cdf394a300a13fffad`; merge `acdc27e69327bc2c6060a669c18b59e7bdd8ab7c`. 11 changed files; 5 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This makes a reserved post-quantum signature suite real for verification, using a published construction rather than writing lattice arithmetic from scratch. It is a concrete crypto-agility step, not a migration of identities or funds to post-quantum safety.

**Mechanism and evidence.** ML-DSA-65 parsing/verification is added through fips204 with a distinct suite tag. Signature and verifying-key serialization become variable-sized vectors. Production signing remains Ed25519 at this stage; tests use the dependency's key generation to exercise real verification.

**What remains weaker than the intended claim.** The larger signatures expose workspace-wide codec assumptions. At the current checkpoint, several mini-objects codecs still cap individual signatures at 256 bytes, below ML-DSA-65's 3,309-byte signatures, despite the primitive supporting them. Correct-length parsing is not proof of a meaningful key, and primitive availability does not create a pre-break ownership anchor or a safe hybrid transition.

**Recommended improvement and rationale.** Test every producer/consumer codec at the largest supported suite sizes and centralize limits by semantic type. Specify hybrid acceptance, downgrade refusal, dormant-anchor binding and pre-break continuity before any identity migration. Review the exact dependency implementation and maintenance path; FIPS standardization is not an audit of this wrapper.

**Concrete example.** A legitimate ML-DSA signature can verify in memory yet fail to decode after an object codec rejects it as larger than 256 bytes. That is a cross-crate compatibility defect even when every primitive-only PQ test passes.

**Acceptance tests to implement.** Use independent FIPS vectors, malformed lengths, cross-suite substitutions and full object/KEL/transport round trips with PQ-sized signatures. Benchmark memory and verification cost on weak devices. Test an attempted post-break recovery with no precommitted unbroken anchor and require explicit refusal rather than first-claim ownership.

**History, supersession and integration.** #176 adds key generation/signing; #237 adds dormant anchor inventory without persistence/activation. #299's signature-count fix does not by itself repair all per-signature byte limits.

**Source entry points.** [`crates/mini-crypto/src/keys.rs`](https://github.com/mininet-labs/Mininet/blob/f847abdbd6698f7cf7b7083c5f50e7bdc3f47cf6/crates/mini-crypto/src/keys.rs); [`crates/mini-crypto/src/suite.rs`](https://github.com/mininet-labs/Mininet/blob/f847abdbd6698f7cf7b7083c5f50e7bdc3f47cf6/crates/mini-crypto/src/suite.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0149"></a>

## PR #149: KEL witness receipts + duplicity gossip: design direction for the "never seen a fresher log" gap (D-0096, audit #12 F4)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-05, FD-08, FD-09, FD-14, FD-15.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/149) | [Files changed](https://github.com/mininet-labs/Mininet/pull/149/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/de306b6235006cb64af0287b7d0af8c4e25cbc56)

Head `de306b6235006cb64af0287b7d0af8c4e25cbc56`; base `ce61f03e99a5ea874c2e3e7f565301edd329a13e`; merge `82c66857a102e79acefec0cdf394a300a13fffad`. 7 changed files; 2 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This gives the first-contact identity-freshness problem an explicit design rather than pretending local sequence pins can discover unseen revocations. The design deliberately avoids a mandatory global identity registry and separates witness observation from controller authority.

**Mechanism and evidence.** Typed witness receipts and threshold certificates attest specific observed events. First-seen witness state refuses incompatible continuations; controller/witness duplicity proofs can be carried by gossip summaries and verified locally. The ten-phase rollout starts with statement semantics before transport or daemons.

**What remains weaker than the intended claim.** Witnesses can still become a de facto indispensable committee if high-assurance identity use requires their availability with no safe rotation/recovery route. A threshold of roots does not demonstrate independent witnesses. Gossip can detect conflicting evidence that reaches a verifier, not prove global absence of a later branch or revocation.

**Recommended improvement and rationale.** Preserve controller-signed authority, explicit assurance levels and privacy-scoped witnessing. Define policy binding, historic witness-key resolution, transition certification, unavailable-witness recovery and evidence retention before granting high-value authority. Separate loss of high assurance from erasure of identity or basic participation.

**Concrete example.** A new verifier presented only an old valid branch cannot infer an unseen newer branch from cryptography alone. Witness receipts improve what evidence it has, but their freshness and independence assumptions must be stated; a certificate is not a universal statement that no other history exists.

**Acceptance tests to implement.** Model captured witness sets, network partitions, stale-but-valid certificates, hidden forks, policy replacement and correlated outages. Verify recovery does not erase evidence and that witnesses cannot author a controller event or acquire custody merely by refusing service.

**History, supersession and integration.** #180/#187/#191 implement the first phases; #314 fixes KEL policy binding; #320-#325 add protocol, persistence, gossip and old-policy transition pieces. The end-to-end authority gate remains a distinct integration obligation.

**Source entry points.** [`docs/DECISION_LOG.md`](https://github.com/mininet-labs/Mininet/blob/de306b6235006cb64af0287b7d0af8c4e25cbc56/docs/DECISION_LOG.md); [`docs/STATUS.md`](https://github.com/mininet-labs/Mininet/blob/de306b6235006cb64af0287b7d0af8c4e25cbc56/docs/STATUS.md); [`docs/design/kel-witness-receipts-and-duplicity-gossip.md`](https://github.com/mininet-labs/Mininet/blob/de306b6235006cb64af0287b7d0af8c4e25cbc56/docs/design/kel-witness-receipts-and-duplicity-gossip.md); [`docs/research/KEL_WITNESS_RECEIPTS_DUPLICITY_GOSSIP_RESEARCH_20260715.md`](https://github.com/mininet-labs/Mininet/blob/de306b6235006cb64af0287b7d0af8c4e25cbc56/docs/research/KEL_WITNESS_RECEIPTS_DUPLICITY_GOSSIP_RESEARCH_20260715.md). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0150"></a>

## PR #150: mini-bridge: generic Tor PT v1 process manager (D-0097, PR2 of bridge-adapter-integration sequence)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-06, FD-09, FD-14, FD-18.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/150) | [Files changed](https://github.com/mininet-labs/Mininet/pull/150/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/cbfbd4426c328824a71201458d37512497895fe5)

Head `cbfbd4426c328824a71201458d37512497895fe5`; base `acdc27e69327bc2c6060a669c18b59e7bdd8ab7c`; merge `22da5b632d532419c8fefdcf81402f9f28de42db`. 12 changed files; 5 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This adds a concrete process boundary for optional censorship-circumvention adapters. It avoids making an external transport binary an identity verifier and uses a real fixture process to exercise startup behavior.

**Mechanism and evidence.** VerifiedExecutable binds an absolute path to a BLAKE3 digest checked before launch. PtProcessManager invokes Command directly without a shell, clears the environment and parses the Tor PT v1 startup exchange under an absolute deadline. Termination waits for OS-confirmed process exit.

**What remains weaker than the intended claim.** A verified path can still have a check-to-exec replacement race unless the executed file is pinned by stronger filesystem/handle semantics. A separate process is not a sandbox: it may retain ambient filesystem/network permissions. The handshake reader's queues and lines need aggregate bounds, and future actual adapter binaries require provenance, licensing and update maintenance.

**Recommended improvement and rationale.** Launch from an immutable owner-approved artifact location or an executable handle where supported, minimize OS privileges and bound line length/count/queue growth. Authenticate Mininet peers above the adapter. Give users a clear way to remove or switch the adapter without losing identity or core data.

**Concrete example.** A fake PT can emit unbounded startup lines quickly while the parent waits for a valid method. A ten-second deadline limits time but not necessarily memory. Another process could replace a writable executable between hashing its path and Command::spawn.

**Acceptance tests to implement.** Test malformed/oversized startup lines, rapid output floods, silent hangs, child exit races, process-tree cleanup and executable substitution. Run a real supported adapter in an isolated integration environment; a fake-PT fixture is not evidence that obfs4/WebTunnel/Snowflake is deployed.

**History, supersession and integration.** Builds on #147. #296 reuses PtProcessManager rather than inventing a parallel authority. The Windows nonexistent-path fixture failures reported later are test portability issues and should be repaired without weakening executable verification.

**Source entry points.** [`crates/mini-bridge/src/bin/fake_pt_fixture.rs`](https://github.com/mininet-labs/Mininet/blob/cbfbd4426c328824a71201458d37512497895fe5/crates/mini-bridge/src/bin/fake_pt_fixture.rs); [`crates/mini-bridge/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/cbfbd4426c328824a71201458d37512497895fe5/crates/mini-bridge/src/error.rs); [`crates/mini-bridge/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/cbfbd4426c328824a71201458d37512497895fe5/crates/mini-bridge/src/lib.rs); [`crates/mini-bridge/src/pt_process.rs`](https://github.com/mininet-labs/Mininet/blob/cbfbd4426c328824a71201458d37512497895fe5/crates/mini-bridge/src/pt_process.rs); [`crates/mini-bridge/tests/pt_process_fixture.rs`](https://github.com/mininet-labs/Mininet/blob/cbfbd4426c328824a71201458d37512497895fe5/crates/mini-bridge/tests/pt_process_fixture.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0151"></a>

## PR #151: mini-private-index/mini-resource-pricing: PIR + anonymous resource-payment research preparation (D-0098, D-0099)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-09, FD-10, FD-11, FD-14, FD-16, FD-18.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/151) | [Files changed](https://github.com/mininet-labs/Mininet/pull/151/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/18f637df409ae782c89a2c60700f68614c475862)

Head `18f637df409ae782c89a2c60700f68614c475862`; base `22da5b632d532419c8fefdcf81402f9f28de42db`; merge `0b19b0ab78a1211cfe6490edce92717fc8402b90`. 9 changed files; 3 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This identifies two privacy leaks that ordinary encryption leaves intact: a lookup server observing which record is fetched, and a payment graph revealing which privacy provider a person used. It sensibly scopes research workloads before choosing cryptography.

**Mechanism and evidence.** The PIR plan uses immutable epoch-versioned equal-size records and keeps whole-index download as a baseline. The candidate portfolio separates non-colluding two-server PIR from computational single-server schemes. The resource-token doctrine separates issuer, wallet, service and redemption roles, with indistinguishable paid/subsidized tokens at spend time.

**What remains weaker than the intended claim.** Neither protocol is implemented. Two-server PIR relies on non-collusion that different keys or cloud accounts do not prove. Blind issuance can still leak through timing, denominations, issuance/redemption correlation or issuer availability. An online issuer or redemption service must not become a mandatory gate over ordinary public activity.

**Recommended improvement and rationale.** Benchmark the exact workload against full download and local caching before adopting a complex scheme. Verify database integrity separately from query privacy. For payments, precommit denomination/expiry/linkability rules and model issuer/provider collusion, withdrawal and offline availability. Keep requester-funded basic operation independent of optional subsidy machinery.

**Concrete example.** Two nominal PIR servers under one operator can combine queries and defeat the non-collusion assumption. A perfectly blind token bought immediately before a uniquely priced service can also be linked statistically even though the blind-signature equation is correct.

**Acceptance tests to implement.** Measure client memory, bandwidth, server work and privacy leakage under sparse use, collusion and churn. Test mismatched database epochs, malicious responses, double redemption, issuer outage and paid/subsidized distinguishers. Report research results without promoting them to an implemented PrivatePIR or anonymous-payment tier.

**History, supersession and integration.** This extends #147/#139. #284/#285 later separate subsidy extraction from voluntary transfers. #294's remote plaintext query remains confidential in transit, not PIR; #305 onward addresses a different private-value layer.

**Source entry points.** [`docs/DECISION_LOG.md`](https://github.com/mininet-labs/Mininet/blob/18f637df409ae782c89a2c60700f68614c475862/docs/DECISION_LOG.md); [`docs/STATUS.md`](https://github.com/mininet-labs/Mininet/blob/18f637df409ae782c89a2c60700f68614c475862/docs/STATUS.md); [`docs/design/mn208-pir-research-and-review-preparation.md`](https://github.com/mininet-labs/Mininet/blob/18f637df409ae782c89a2c60700f68614c475862/docs/design/mn208-pir-research-and-review-preparation.md); [`docs/design/mn602-mn603-anonymous-resource-payment-preparation.md`](https://github.com/mininet-labs/Mininet/blob/18f637df409ae782c89a2c60700f68614c475862/docs/design/mn602-mn603-anonymous-resource-payment-preparation.md); [`docs/research/MN208_PIR_RESEARCH_AND_REVIEW_PREPARATION_20260715.md`](https://github.com/mininet-labs/Mininet/blob/18f637df409ae782c89a2c60700f68614c475862/docs/research/MN208_PIR_RESEARCH_AND_REVIEW_PREPARATION_20260715.md); [`docs/research/MN602_MN603_ANONYMOUS_RESOURCE_PAYMENT_RESEARCH_20260715.md`](https://github.com/mininet-labs/Mininet/blob/18f637df409ae782c89a2c60700f68614c475862/docs/research/MN602_MN603_ANONYMOUS_RESOURCE_PAYMENT_RESEARCH_20260715.md). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0152"></a>

## PR #152: Native Intake / Public Commons / Open Web Search: founder decisions D-0311, D-0312 + legal disclaimer + priority update

**PARTIAL** | Captured outcome: **merged** | Directives: FD-01, FD-02, FD-09, FD-10, FD-11, FD-16, FD-18.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/152) | [Files changed](https://github.com/mininet-labs/Mininet/pull/152/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/c456a35600340b7644a812eb894a66cc8055495e)

Head `c456a35600340b7644a812eb894a66cc8055495e`; base `0b19b0ab78a1211cfe6490edce92717fc8402b90`; merge `640e3572ec30974825a0f38b388d8a5a3748ac93`. 9 changed files; 2 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This clarifies the founder's public-commons and MiniSearch direction: public speech and participation are rights, not purchases, while scarce optional services may be paid. It also distinguishes a proposed documentation architecture from an activated reorganization.

**Mechanism and evidence.** D-0311 defines the free public commons; D-0312 calls for independent, transparent, pluralistic search. The research document and legal disclaimer are committed, and priority guidance is placed in living STATUS after the protected instruction-surface guard rejects an ordinary CLAUDE.md edit.

**What remains weaker than the intended claim.** A policy and disclaimer do not enforce free access or provide legal clearance. Search plurality can still be undermined by provider-supplied scores, discovery concentration, resource exclusion or payment-linked ranking. Moving prose out of a protected instruction file is appropriate only when it remains non-authorizing guidance rather than a back door to change the charter.

**Recommended improvement and rationale.** Turn the commons boundary into real wallet-free application tests and scheduler/resource limits that remain user controlled. Define search provenance, local ranking choice and paid-placement separation. Have qualified counsel review actual deployment roles instead of treating a constitutional disclaimer as immunity.

**Concrete example.** A user with zero MINI should still publish a public comment and read an available public object. A paid relay can sell extra transport service, but payment must not make its customer's post politically authoritative or silently outrank an unpaid post.

**Acceptance tests to implement.** Test real public application flows without constructing a wallet, paid versus unpaid ranking parity, provider removal and revoked local resource consent. Validate that imported research or legal text cannot become activated policy merely by being committed or parsed.

**History, supersession and integration.** #153/#159/#242/#286 build intake; #243-#246 implement commons/publication policy; #256-#294 build search pieces. Each must prove the boundary at its actual consumer, not only cite this doctrine.

**Source entry points.** [`README.md`](https://github.com/mininet-labs/Mininet/blob/c456a35600340b7644a812eb894a66cc8055495e/README.md); [`docs/DECISION_LOG.md`](https://github.com/mininet-labs/Mininet/blob/c456a35600340b7644a812eb894a66cc8055495e/docs/DECISION_LOG.md); [`docs/LEGAL_DISCLAIMER.md`](https://github.com/mininet-labs/Mininet/blob/c456a35600340b7644a812eb894a66cc8055495e/docs/LEGAL_DISCLAIMER.md); [`docs/STATUS.md`](https://github.com/mininet-labs/Mininet/blob/c456a35600340b7644a812eb894a66cc8055495e/docs/STATUS.md); [`docs/design/mininet-canon-documentation-architecture.md`](https://github.com/mininet-labs/Mininet/blob/c456a35600340b7644a812eb894a66cc8055495e/docs/design/mininet-canon-documentation-architecture.md); [`docs/research/MININET_NATIVE_INTAKE_PUBLIC_COMMONS_AND_OPEN_WEB_SEARCH_20260718.md`](https://github.com/mininet-labs/Mininet/blob/c456a35600340b7644a812eb894a66cc8055495e/docs/research/MININET_NATIVE_INTAKE_PUBLIC_COMMONS_AND_OPEN_WEB_SEARCH_20260718.md). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0153"></a>

## PR #153: mini-intake-types: shared Mininet Intake vocabulary (D-0313, Track B1)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-03, FD-05, FD-09, FD-12, FD-14.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/153) | [Files changed](https://github.com/mininet-labs/Mininet/pull/153/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/cc2f99b7b68f28c230a64094f9490d22ebac09db)

Head `cc2f99b7b68f28c230a64094f9490d22ebac09db`; base `640e3572ec30974825a0f38b388d8a5a3748ac93`; merge `6ddaddb69fb9af622d40ce2ea1ca3e93c76ed6c4`. 20 changed files; 1 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This creates the native intake vocabulary and, importantly, starts freshly imported material as unreviewed external data. It supports a project where evidence can be preserved without a parser or AI model promoting itself into governance authority.

**Mechanism and evidence.** IntakeEnvelope has private state fields, explicit review transitions and an authority-promotion method requiring Accepted review before higher classes. Source and derived-representation records keep raw evidence distinct from generated interpretation. The wire codec bounds several collections and rejects truncation/trailing bytes.

**What remains weaker than the intended claim.** The public from_bytes path reconstructs review_state, authority and links directly from the supplied bytes without rechecking the constructor/transition invariants or authenticating an approval. Thus private fields are not, by themselves, proof that decoded Accepted/ReviewedEvidence state was legitimately reached. add_representation/add_warning also need producer-side bounds matching decoder limits.

**Recommended improvement and rationale.** Treat externally decoded envelope status as a claim, never local approval. Separate restoration of trusted local state from import of untrusted envelopes, revalidate cross-field invariants and bind genuine review transitions to the exact source digest and accountable local/governed action. Make collection mutation fallible before producing self-undecodable state.

**Concrete example.** An attacker can encode Accepted and ReviewedEvidence tags into a well-formed envelope without calling advance_review_state. A service that trusts those tags after decoding would launder an external claim into reviewed evidence even though the normal constructor is conservative.

**Acceptance tests to implement.** Craft wire envelopes with every impossible review/authority/link combination and require rejection or reset to untrusted import state. Exceed representation/warning counts through public mutators and verify bounded refusal. Test that changing source bytes invalidates the corresponding review approval rather than retaining authority by filename.

**History, supersession and integration.** #159 adds local intake storage, #242 gates publication links, and #286 rechecks source bytes before signing posts. Those additions do not automatically authenticate serialized review-state claims.

**Source entry points.** [`crates/mini-intake-types/src/authority.rs`](https://github.com/mininet-labs/Mininet/blob/cc2f99b7b68f28c230a64094f9490d22ebac09db/crates/mini-intake-types/src/authority.rs); [`crates/mini-intake-types/src/codec.rs`](https://github.com/mininet-labs/Mininet/blob/cc2f99b7b68f28c230a64094f9490d22ebac09db/crates/mini-intake-types/src/codec.rs); [`crates/mini-intake-types/src/envelope.rs`](https://github.com/mininet-labs/Mininet/blob/cc2f99b7b68f28c230a64094f9490d22ebac09db/crates/mini-intake-types/src/envelope.rs); [`crates/mini-intake-types/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/cc2f99b7b68f28c230a64094f9490d22ebac09db/crates/mini-intake-types/src/error.rs); [`crates/mini-intake-types/src/ids.rs`](https://github.com/mininet-labs/Mininet/blob/cc2f99b7b68f28c230a64094f9490d22ebac09db/crates/mini-intake-types/src/ids.rs); [`crates/mini-intake-types/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/cc2f99b7b68f28c230a64094f9490d22ebac09db/crates/mini-intake-types/src/lib.rs); [`crates/mini-intake-types/src/link.rs`](https://github.com/mininet-labs/Mininet/blob/cc2f99b7b68f28c230a64094f9490d22ebac09db/crates/mini-intake-types/src/link.rs); [`crates/mini-intake-types/src/media.rs`](https://github.com/mininet-labs/Mininet/blob/cc2f99b7b68f28c230a64094f9490d22ebac09db/crates/mini-intake-types/src/media.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0154"></a>

## PR #154: Harden social, storage, and forge validation

**PASS** | Captured outcome: **merged** | Directives: FD-04, FD-05, FD-06, FD-09, FD-15.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/154) | [Files changed](https://github.com/mininet-labs/Mininet/pull/154/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/cad47b5afb7782390a9337ce7c20f955ecc84e67)

Head `cad47b5afb7782390a9337ce7c20f955ecc84e67`; base `6ddaddb69fb9af622d40ce2ea1ca3e93c76ed6c4`; merge `b4b8242c1f8caed8dd5ee9a7e237102a90771977`. 13 changed files; 2 commits; 0 issue comments, 2 inline comments and 1 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This repairs three concrete trust-boundary weaknesses across social decoding, storage receipts and governed Forge commits. It moves validation toward the consumer that actually grants meaning, rather than trusting a type tag or a producer's good behavior.

**Mechanism and evidence.** Social resolution rejects malformed, cross-author, over-limit and trailing objects. Storage verification adds a configurable future-skew bound, closing the one-sided freshness check from #2. Forge governance validates canonical commit/tree structure and current author provenance before accepting a governed branch head.

**What remains weaker than the intended claim.** The scoped corrections are valuable and have adversarial regressions. They do not establish first-contact KEL freshness, independent receipt parties or atomic filesystem behavior. A configurable/optional time context still needs a reviewed production policy; the default five-minute allowance is not a measured physical security bound. Scanner comments on deterministic test nonces are not proof of runtime nonce reuse.

**Recommended improvement and rationale.** Preserve consumer-side checks and extend them across every alternate producer and decoder. Keep a canonical structural validator per object type so desktop, CLI and sync cannot diverge. Track resolved findings explicitly: future-dated storage receipts are not an open historical defect after this PR's bound is applied.

**Concrete example.** A validly signed object tagged COMMIT can still contain two conflicting tree links or malformed payload fields. Governance must reject its structure before making it a branch head. A valid receipt dated far in the future must likewise fail rather than remain fresh indefinitely.

**Acceptance tests to implement.** Test each malformed structure with a valid signature, because signature failure would mask the structural bug. Include exact future-skew boundaries, same-name different-author social objects and a governed commit whose referenced tree is missing. Require rejection before mutation.

**History, supersession and integration.** Hardens #2's receipts and the early social/Forge foundation. #164 further validates ordinary branch targets; #286 later consolidates post validation. The PASS applies to these named fixes, not production readiness of the three subsystems.

**Source entry points.** [`crates/mini-forge/src/governance.rs`](https://github.com/mininet-labs/Mininet/blob/cad47b5afb7782390a9337ce7c20f955ecc84e67/crates/mini-forge/src/governance.rs); [`crates/mini-forge/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/cad47b5afb7782390a9337ce7c20f955ecc84e67/crates/mini-forge/src/lib.rs); [`crates/mini-forge/tests/forge.rs`](https://github.com/mininet-labs/Mininet/blob/cad47b5afb7782390a9337ce7c20f955ecc84e67/crates/mini-forge/tests/forge.rs); [`crates/mini-forge/tests/governance.rs`](https://github.com/mininet-labs/Mininet/blob/cad47b5afb7782390a9337ce7c20f955ecc84e67/crates/mini-forge/tests/governance.rs); [`crates/mini-social/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/cad47b5afb7782390a9337ce7c20f955ecc84e67/crates/mini-social/src/lib.rs); [`crates/mini-social/src/wall.rs`](https://github.com/mininet-labs/Mininet/blob/cad47b5afb7782390a9337ce7c20f955ecc84e67/crates/mini-social/src/wall.rs); [`crates/mini-social/tests/social.rs`](https://github.com/mininet-labs/Mininet/blob/cad47b5afb7782390a9337ce7c20f955ecc84e67/crates/mini-social/tests/social.rs); [`crates/mini-storage/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/cad47b5afb7782390a9337ce7c20f955ecc84e67/crates/mini-storage/src/error.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0156"></a>

## PR #156: ci: narrow the reproducibility check's PR trigger + reduce docs-log merge friction (D-0314)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-03, FD-06, FD-10, FD-14.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/156) | [Files changed](https://github.com/mininet-labs/Mininet/pull/156/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/2988cff1625b138fd1d0a69a3f0f49422b006ff0)

Head `2988cff1625b138fd1d0a69a3f0f49422b006ff0`; base `b4b8242c1f8caed8dd5ee9a7e237102a90771977`; merge `4cf96a1110c5855a9a364feaeb46f39c25f38f52`. 8 changed files; 1 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This tries to reduce contributor friction by separating expensive reproducibility builds and automatically combining append-heavy documentation. The underlying goal is legitimate: routine documentation should not make independent contribution unnecessarily costly.

**Mechanism and evidence.** The reproducibility job moves to a separate workflow, still running on main pushes but using PR paths-ignore for docs/Markdown. .gitattributes selects merge=union for DECISION_LOG and STATUS. The required check name is retained.

**What remains weaker than the intended claim.** The trigger change can leave a required check permanently absent on docs-only PRs, which #185 later observes and fixes. Union merge does not prove semantic compatibility: STATUS is mutable, and concurrent edits or decision-number collisions can be combined into contradictory text without a conflict marker. The assertion that documentation cannot affect any artifact also requires a build-input analysis, not just a filename rule.

**Recommended improvement and rationale.** Always report an explicit required-check conclusion and skip expensive work inside the job under a validated scope rule. Do not use union merge as a substitute for append-only decision checks and status reconciliation. Keep generated navigation disposable and separate from normative history.

**Concrete example.** Two branches can each declare a different current state or reserve the same D-number; union merge can retain both and look clean. Separately, GitHub cannot conclude that a required reproducibility job passed when its path filter prevented it from running at all.

**Acceptance tests to implement.** Test docs-only and mixed-code PR triggers, build-input files under docs, simultaneous status edits, duplicate decision IDs and missing historical entries. Distinguish skipped-by-scope from artifacts-rebuilt-and-compared in check output.

**History, supersession and integration.** #185 repairs the missing-conclusion problem; #261 reduces generated-nav churn; #301 adds decision-history checks. These later corrections are essential context for judging the original optimization.

**Source entry points.** [`.gitattributes`](https://github.com/mininet-labs/Mininet/blob/2988cff1625b138fd1d0a69a3f0f49422b006ff0/.gitattributes); [`.github/workflows/ci.yml`](https://github.com/mininet-labs/Mininet/blob/2988cff1625b138fd1d0a69a3f0f49422b006ff0/.github/workflows/ci.yml); [`.github/workflows/reproducibility.yml`](https://github.com/mininet-labs/Mininet/blob/2988cff1625b138fd1d0a69a3f0f49422b006ff0/.github/workflows/reproducibility.yml); [`docs/DECISION_LOG.md`](https://github.com/mininet-labs/Mininet/blob/2988cff1625b138fd1d0a69a3f0f49422b006ff0/docs/DECISION_LOG.md); [`docs/STATUS.md`](https://github.com/mininet-labs/Mininet/blob/2988cff1625b138fd1d0a69a3f0f49422b006ff0/docs/STATUS.md). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 2 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0157"></a>

## PR #157: coordination: add bootstrap work claims

**PARTIAL** | Captured outcome: **merged** | Directives: FD-03, FD-06, FD-10, FD-12, FD-14.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/157) | [Files changed](https://github.com/mininet-labs/Mininet/pull/157/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/8875cf87bd5ef655244c29ea7ed89991d9f7a692)

Head `8875cf87bd5ef655244c29ea7ed89991d9f7a692`; base `4cf96a1110c5855a9a364feaeb46f39c25f38f52`; merge `34ed4f0e183b7bf5ae3e07b82574736955b2c342`. 7 changed files; 1 commits; 2 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This addresses a real collaboration failure: duplicate AI work had already targeted the same intake crate. Work claims make scope, branch, issue and decision ownership visible before parallel contributors build conflicting implementations.

**Mechanism and evidence.** A machine-readable registry and validator reject duplicate active claims, colliding decision IDs, expired leases and overlapping paths. Claims are coordination records, not payment entitlement, review authority or ownership of the protocol. The original duplicated intake implementation is removed in favor of #153.

**What remains weaker than the intended claim.** The review discussion identifies an important false-positive class: every meaningful PR may append to DECISION_LOG or update shared workspace metadata, so treating all path overlap as exclusive work can block legitimate parallelism. Calendar leases and stale merged claims later repeatedly cause unrelated CI failures. A claim is only as current as the recorded state.

**Recommended improvement and rationale.** Distinguish exclusive feature paths from shared append/registry paths, and represent intentional stacked work explicitly. Reconcile claim state to actual PR outcomes without granting automation merge authority. Reserve decision identifiers atomically or use collision-resistant proposal IDs before assigning final sequential labels.

**Concrete example.** Two developers changing different crates both need DECISION_LOG. A broad overlap rule rejects both even though the feature work is independent, encouraging under-declared claims or bypasses. A good guard should catch duplicated code scope without penalizing ordinary shared bookkeeping.

**Acceptance tests to implement.** Test true code overlap, shared append files, nested directory overlap, stacked branches, expired and already-merged claims, duplicate decision reservations and concurrent claim creation. Verify a work claim cannot confer approval or prevent another participant from forking the software.

**History, supersession and integration.** #308 later adds explicit stacked_on semantics; #301 catches additional decision/history races. The registry remains a temporary coordination adapter, not a permanent institution that assigns work or legitimacy.

**Source entry points.** [`docs/DECISION_LOG.md`](https://github.com/mininet-labs/Mininet/blob/8875cf87bd5ef655244c29ea7ed89991d9f7a692/docs/DECISION_LOG.md); [`docs/governance/51_BOOTSTRAP_WORK_CLAIMS.md`](https://github.com/mininet-labs/Mininet/blob/8875cf87bd5ef655244c29ea7ed89991d9f7a692/docs/governance/51_BOOTSTRAP_WORK_CLAIMS.md); [`governance/work-claims.json`](https://github.com/mininet-labs/Mininet/blob/8875cf87bd5ef655244c29ea7ed89991d9f7a692/governance/work-claims.json); [`governance/work-claims.schema.json`](https://github.com/mininet-labs/Mininet/blob/8875cf87bd5ef655244c29ea7ed89991d9f7a692/governance/work-claims.schema.json); [`tools/check_governance.py`](https://github.com/mininet-labs/Mininet/blob/8875cf87bd5ef655244c29ea7ed89991d9f7a692/tools/check_governance.py); [`tools/test_check_governance.py`](https://github.com/mininet-labs/Mininet/blob/8875cf87bd5ef655244c29ea7ed89991d9f7a692/tools/test_check_governance.py); [`tools/work_claims.py`](https://github.com/mininet-labs/Mininet/blob/8875cf87bd5ef655244c29ea7ed89991d9f7a692/tools/work_claims.py). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0158"></a>

## PR #158: Scale and harden filesystem store traversal

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-06, FD-11, FD-14, FD-15.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/158) | [Files changed](https://github.com/mininet-labs/Mininet/pull/158/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/883a03457fe3b68e1c025ac0b7fc7c58c59d971c)

Head `883a03457fe3b68e1c025ac0b7fc7c58c59d971c`; base `befafb79f24d3d86780f2dc9f68cced1ec3f8d44`; merge `0e321aa221ec2007762cd7f8a7d1d755b99c3f32`. 3 changed files; 4 commits; 1 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This improves filesystem-store query cost and closes persistent path-traversal hazards. It directly serves weaker nodes by avoiding unrelated metadata-tree walks for a narrow author/type/link query.

**Mechanism and evidence.** FsBackend::list_meta_prefix starts at the deepest complete subtree compatible with the prefix. Reads and existence checks validate parent components and reject symlinks, directories and special files rather than blindly following them. Tests include unrelated poisoned subtrees and intermediate symlink escapes.

**What remains weaker than the intended claim.** The PR explicitly does not protect against hostile concurrent filesystem mutation. Checking a pathname with symlink_metadata and later opening it leaves a time-of-check/time-of-use boundary. Narrowing the subtree also does not bound the number of matching rows, and indexes remain nontransactional across multiple metadata writes.

**Recommended improvement and rationale.** Use handle-relative no-follow traversal where supported for adversarial local-filesystem scenarios. Add bounded range/pagination primitives and a recoverable index transaction model, while keeping content-addressed objects authoritative. Evaluate a local transactional storage engine as an implementation option, not as a hosted authority.

**Concrete example.** A prefix query for one author should not fail because another author's unrelated subtree is poisoned. But an attacker able to replace the checked directory with a symlink between validation and open can still exploit a pathname-only check unless the open itself enforces confinement.

**Acceptance tests to implement.** Exercise exact and partial prefixes, large matching subtrees, concurrent replacement races, symlinked parents/final files, special files and interrupted index writes. Measure filesystem reads and latency as unrelated history grows, rather than reporting only result-count correctness.

**History, supersession and integration.** #189/#193 add chronological queries; #287 builds a bounded ordered time index. #289's consensus archive uses no-follow handle opens and supplies a useful pattern, but does not automatically harden every Store path.

**Source entry points.** [`crates/mini-store/src/backend.rs`](https://github.com/mininet-labs/Mininet/blob/883a03457fe3b68e1c025ac0b7fc7c58c59d971c/crates/mini-store/src/backend.rs); [`crates/mini-store/tests/store.rs`](https://github.com/mininet-labs/Mininet/blob/883a03457fe3b68e1c025ac0b7fc7c58c59d971c/crates/mini-store/tests/store.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0159"></a>

## PR #159: mini-intake: trusted intake coordinator (D-0315, Track B2)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-03, FD-05, FD-06, FD-09, FD-12.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/159) | [Files changed](https://github.com/mininet-labs/Mininet/pull/159/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/4be107bd48ac1dea5b53b9f7848cba7cccf5c311)

Head `4be107bd48ac1dea5b53b9f7848cba7cccf5c311`; base `0e321aa221ec2007762cd7f8a7d1d755b99c3f32`; merge `6c66e5727dccd433e1f45e1ae5e6bfbb4fd3abb7`. 13 changed files; 2 commits; 1 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This supplies a real local intake coordinator over the earlier vocabulary. Raw text/Markdown becomes content-addressed evidence without pretending it was already authored or approved inside Mininet.

**Mechanism and evidence.** The coordinator hashes source bytes, stores them through Backend, creates Unreviewed/UntrustedExternal envelopes and deduplicates by content digest. It deliberately avoids signed Object ingestion because external material has no Mininet signature at import time. Supported media and UTF-8 handling are narrowly specified.

**What remains weaker than the intended claim.** Content-addressed lookup keys are not proof that a mutable backend still returns the original bytes. Deduplication should preserve review state only when the exact source and context remain valid. The word trusted describes the coordinator boundary, not the imported document or every decoded envelope field. Filesystem size/path handling and producer-side collection bounds remain important.

**Recommended improvement and rationale.** Recompute digest, length and intake identity at every boundary that signs or promotes fetched source material. Separate immutable source blobs from mutable review records and make updates atomic and provenance-bound. Keep parsers and models outside the authority path and preserve explicit unsupported-media errors.

**Concrete example.** A backend can return different same-length text under an existing source key. If publication signs it without rehashing, the signed post no longer corresponds to the document that was accepted for review. Trusting the key name does not detect that substitution.

**Acceptance tests to implement.** Test same-content deduplication, changed-content reimport, invalid UTF-8, oversized files, corrupt envelope state and source substitution with the same length. Ensure failed ingestion or publication leaves prior accepted evidence intact and creates no governance authority.

**History, supersession and integration.** Builds on #153. #172 adds an isolated-extractor boundary for future formats; #242 gates links; #286 implements the explicit source-byte recheck and crash-safe intake-to-post publication.

**Source entry points.** [`crates/mini-intake/src/coordinator.rs`](https://github.com/mininet-labs/Mininet/blob/4be107bd48ac1dea5b53b9f7848cba7cccf5c311/crates/mini-intake/src/coordinator.rs); [`crates/mini-intake/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/4be107bd48ac1dea5b53b9f7848cba7cccf5c311/crates/mini-intake/src/error.rs); [`crates/mini-intake/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/4be107bd48ac1dea5b53b9f7848cba7cccf5c311/crates/mini-intake/src/lib.rs); [`crates/mini-intake/src/media.rs`](https://github.com/mininet-labs/Mininet/blob/4be107bd48ac1dea5b53b9f7848cba7cccf5c311/crates/mini-intake/src/media.rs); [`crates/mini-intake/tests/intake.rs`](https://github.com/mininet-labs/Mininet/blob/4be107bd48ac1dea5b53b9f7848cba7cccf5c311/crates/mini-intake/tests/intake.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 1 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0160"></a>

## PR #160: mini-web-types: MiniSearch shared vocabulary (D-0316)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-01, FD-09, FD-10, FD-11, FD-14, FD-16.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/160) | [Files changed](https://github.com/mininet-labs/Mininet/pull/160/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/c6cee43f002a7fa50fb543c6d7dff314a1b3dcb7)

Head `c6cee43f002a7fa50fb543c6d7dff314a1b3dcb7`; base `34ed4f0e183b7bf5ae3e07b82574736955b2c342`; merge `befafb79f24d3d86780f2dc9f68cced1ec3f8d44`. 10 changed files; 1 commits; 1 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This defines MiniSearch's shared vocabulary so relevance, availability, restrictions and personalization do not collapse into one opaque score. That is foundational to a pluralistic search system whose users can understand why results appear or disappear.

**Mechanism and evidence.** RankingProfile::public_default selects no personalization. AvailabilityState distinguishes restriction and unavailability reasons; SearchResult refuses a nonzero scored result being converted into a restricted/unavailable output. CanonicalUrl validates already-separated components, and the ranking profile has no payment or governance weight.

**What remains weaker than the intended claim.** These types describe constraints but do not yet implement search or prove that every consumer respects them. Validating separated URL parts is not a full URL parser or SSRF policy. A caller can still supply dishonest availability metadata or ranking signals; their provenance and user-selected policy need explicit treatment.

**Recommended improvement and rationale.** Keep availability filtering visible and distinct from relevance, and attach signed/source-verifiable provenance where appropriate without making one provider the truth authority. Specify canonical URL parsing, normalization and scheme/host/IP admission separately. Test wallet-independent public search through the eventual full pipeline.

**Concrete example.** A document unavailable on one provider should not automatically become globally irrelevant or forbidden. Another provider may hold it, and a user's chosen filter is different from a network-wide restriction. The UI should preserve those distinctions.

**Acceptance tests to implement.** Test public-default no-personalization, paid/unpaid ranking parity, malformed URL components, equivalent URL spellings and restricted-result handling. Later integration tests must prove these rules survive indexing, federated merging and local reranking.

**History, supersession and integration.** #162 adds planning; #256/#257 indexing/ranking; #278 parsing/provenance; #281-#294 federation. This PR's clean type boundary is a prerequisite, not proof that those later systems are complete or neutral.

**Source entry points.** [`crates/mini-web-types/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/c6cee43f002a7fa50fb543c6d7dff314a1b3dcb7/crates/mini-web-types/src/lib.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0162"></a>

## PR #162: mini-crawler: deterministic MiniSearch crawl planning (D-0317)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-06, FD-11, FD-14, FD-15.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/162) | [Files changed](https://github.com/mininet-labs/Mininet/pull/162/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/11cbeb6667687108e73af2ee917abb5edadea61b)

Head `11cbeb6667687108e73af2ee917abb5edadea61b`; base `6c66e5727dccd433e1f45e1ae5e6bfbb4fd3abb7`; merge `1d7fa18f1f3a14d10786817e5b4129350069828a`. 11 changed files; 4 commits; 2 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This builds the deterministic admission/planning half of a crawler before allowing participant devices to fetch arbitrary network content. It is a useful way to make resource and policy decisions testable without giving a parser or remote site ambient authority.

**Mechanism and evidence.** CrawlPlan is FIFO and keyed by canonical URL strings. HTTPS/same-host defaults, depth limits, pending/seen caps and caller-supplied exclusions reject work before network effects. admit_batch returns explicit per-request outcomes and is intentionally non-atomic.

**What remains weaker than the intended claim.** Caller-supplied robots exclusions are not a robots parser or proof that a site permits a request. Same-host URLs can still resolve to private/internal addresses or redirect elsewhere; that must be checked at each actual fetch. Determinism of a queue does not establish politeness, fair distributed scheduling or bounded lifetime resource consumption.

**Recommended improvement and rationale.** Keep admission pure, but make the runtime enforce DNS/IP/redirect and robots policy independently at the moment of use. Specify whether rejected batch elements affect later admission capacity and preserve the documented non-atomic ordering. Add durable leases and budget accounting only with explicit ownership and crash semantics.

**Concrete example.** A permitted HTTPS URL on the initial host can resolve to 127.0.0.1 or redirect to an internal service. Passing the planner is not authorization for that socket connection. The network fetcher must revalidate resolved endpoints and every redirect.

**Acceptance tests to implement.** Test mixed accepted/rejected batches, exact queue limits, duplicate seeds, multiple-host seed sets and deterministic order. At runtime, test DNS rebinding, redirects, robots changes and exhausted budgets while confirming no rejected request makes a network call.

**History, supersession and integration.** Depends on #160. #283 provides the actual bounded fetch runtime; #300 hardens its address policy. Extraction/indexing and orchestration remain separate, explicitly tracked pieces.

**Source entry points.** [`crates/mini-crawler/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/11cbeb6667687108e73af2ee917abb5edadea61b/crates/mini-crawler/src/lib.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 1 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0163"></a>

## PR #163: Forge: add strict release log queries

**PARTIAL** | Captured outcome: **merged** | Directives: FD-03, FD-05, FD-06, FD-15.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/163) | [Files changed](https://github.com/mininet-labs/Mininet/pull/163/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/5e84692c3aa8acaf4509c2ef7802b2f0261b13d7)

Head `5e84692c3aa8acaf4509c2ef7802b2f0261b13d7`; base `1d7fa18f1f3a14d10786817e5b4129350069828a`; merge `ef95048da22b1c15010f331575af79bf5fa22cb8`. 3 changed files; 5 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This adds strict release-log queries so malformed release-shaped objects are not silently omitted from an observer's view. It addresses a real evidence problem: two observers can otherwise construct different apparent histories from the same polluted store.

**Mechanism and evidence.** list_releases_strict and detect_equivocation_strict fail on malformed RELEASE objects claiming the queried project. Compatibility APIs remain permissive. Parsing and evidence reporting become explicit choices rather than accidental consequences of filtering errors.

**What remains weaker than the intended claim.** Strictness can also become an availability attack if any unrelated signer may inject a malformed RELEASE claiming a project and thereby make the entire query fail. These local log-query functions do not themselves establish project-authorized release status or a globally complete log. Existing consumers can still call the old permissive API.

**Recommended improvement and rationale.** Separate authorized release history, untrusted claims and malformed-record diagnostics. Preserve adverse evidence without letting an unauthorized publisher veto all legitimate release queries. Make security-sensitive consumers choose a documented strict verification policy, with bounded work and deterministic error reporting.

**Concrete example.** Mallory publishes a signed but malformed object with a project link naming Alice's project. A strict structural scan can now fail, although Mallory never had release authority for that project. The verifier must distinguish malicious noise from an authorized inconsistent release.

**Acceptance tests to implement.** Test malformed authorized releases, malformed unrelated-author claims, unrelated projects, duplicate versions, split views and large adversarial release sets. Confirm diagnostics do not silently remove genuine evidence and that unauthorized noise cannot authorize or install anything.

**History, supersession and integration.** Extends #104's release history. #164 hardens branch targets and #166/#269 consume governed release verification for retrieval. This is local evidence hygiene, not a global transparency or release-authority proof.

**Source entry points.** [`crates/mini-forge/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/5e84692c3aa8acaf4509c2ef7802b2f0261b13d7/crates/mini-forge/src/lib.rs); [`crates/mini-forge/src/release.rs`](https://github.com/mininet-labs/Mininet/blob/5e84692c3aa8acaf4509c2ef7802b2f0261b13d7/crates/mini-forge/src/release.rs); [`crates/mini-forge/tests/release.rs`](https://github.com/mininet-labs/Mininet/blob/5e84692c3aa8acaf4509c2ef7802b2f0261b13d7/crates/mini-forge/tests/release.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0164"></a>

## PR #164: Forge: validate branch head targets

**PASS** | Captured outcome: **merged** | Directives: FD-03, FD-05, FD-06, FD-15.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/164) | [Files changed](https://github.com/mininet-labs/Mininet/pull/164/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/a42e0f460f3a13036c2791c6515e53a97e710f95)

Head `a42e0f460f3a13036c2791c6515e53a97e710f95`; base `ef95048da22b1c15010f331575af79bf5fa22cb8`; merge `7ebc18ed41c5998a62016ec42fd42e2cb0badea2`. 2 changed files; 5 commits; 1 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This moves branch-target validation to publication time, preventing a signed branch HEAD from pointing at a missing or structurally invalid commit tree. It is a small but important example of putting the check at the authority-bearing consumer.

**Mechanism and evidence.** set_branch validates that the target exists, is a commit and references a well-formed complete tree before writing a signed branch-head object. The test suite challenges missing targets, wrong object types and malformed tree shapes.

**What remains weaker than the intended claim.** The scoped structural repair does not by itself prove the branch change has governance approval, that every file is safe to execute or that current author KEL state is fresh. It should be composed with, not substituted for, governed branch-transition rules. A mutable local backend also requires revalidation when a target is later read.

**Recommended improvement and rationale.** Reuse one canonical commit/tree validator across branch publication, checkout, governance and release construction. Keep structural validity, provenance and branch-change authorization as separately named predicates. Avoid granting a general set_branch caller the power to canonicalize a protected project branch.

**Concrete example.** A blob carrying the COMMIT type tag but no valid tree must not become a branch head merely because it is signed. Conversely, a valid commit by an unknown contributor can be structurally readable without being approved for the canonical branch.

**Acceptance tests to implement.** Test valid unapproved commits separately from malformed ones, missing nested objects, ambiguous links and post-publication backend corruption. Assert rejection creates no signed HEAD object and leaves the previous branch pointer unchanged.

**History, supersession and integration.** Builds on #154's governed commit validation. #276's Git import should pass the same validator; imported provenance must not create an alternate less-strict branch publication path.

**Source entry points.** [`crates/mini-forge/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/a42e0f460f3a13036c2791c6515e53a97e710f95/crates/mini-forge/src/lib.rs); [`crates/mini-forge/tests/forge.rs`](https://github.com/mininet-labs/Mininet/blob/a42e0f460f3a13036c2791c6515e53a97e710f95/crates/mini-forge/tests/forge.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0165"></a>

## PR #165: Make CLI sequence allocation process-safe

**PARTIAL** | Captured outcome: **merged** | Directives: FD-03, FD-05, FD-06, FD-11, FD-15.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/165) | [Files changed](https://github.com/mininet-labs/Mininet/pull/165/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/15064dbc77cb5a943af9a9c1930abe90c71dcec5)

Head `15064dbc77cb5a943af9a9c1930abe90c71dcec5`; base `7ebc18ed41c5998a62016ec42fd42e2cb0badea2`; merge `f845052c91bbd02d7c51abfbac71ff27245cac4c`. 5 changed files; 4 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This closes the ordinary concurrent-process race in per-home author sequence allocation. Multiple CLI invocations should not accidentally create conflicting signed-object sequence numbers while working in one participant's home.

**Mechanism and evidence.** An OS-backed exclusive lock covers the read-modify-write counter operation, and checked_add refuses exhaustion. Persistence, lock reuse and concurrent allocation tests exercise the normal path. The implementation explicitly remains separate from a transaction over all home state.

**What remains weaker than the intended claim.** Current source still parses corrupt counter text with unwrap_or(0) and treats every read error as zero. It writes by truncating the counter file and does not fsync before returning. Corruption, permission/I/O errors or a power cut can therefore reset/reuse sequence numbers even though the concurrent locking case works. This is a confirmed remaining failure, not merely an unmeasured optimization.

**Recommended improvement and rationale.** Allow zero only for a genuinely authorized first initialization. Reject malformed state and unexpected read errors; write the next counter atomically with durability barriers before signing can use it. Consider a recoverable journal tying reserved sequence to exact signed bytes, so retries do not produce conflicting objects.

**Concrete example.** After sequence 100 has been used, replacing the counter contents with an empty string causes the current parser to choose zero and return 1. The OS lock serializes this wrong answer perfectly; locking does not make corrupt state trustworthy.

**Acceptance tests to implement.** Test malformed/empty/overflowing counters, permission errors, truncated writes, crash before/after sync, actual separate-process contention and restart retry. The corruption cases must return an error and leave signed output and counter unchanged, never reset to one.

**History, supersession and integration.** #286 later uses a publish journal to reuse exact signed bytes for intake retries. The same recovery discipline should inform sequence allocation. The narrow corruption/error patch proposed with this review must not be mistaken for full rollback-resistant storage.

**Source entry points.** [`crates/mini-cli/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/15064dbc77cb5a943af9a9c1930abe90c71dcec5/crates/mini-cli/src/lib.rs); [`crates/mini-cli/src/sequence.rs`](https://github.com/mininet-labs/Mininet/blob/15064dbc77cb5a943af9a9c1930abe90c71dcec5/crates/mini-cli/src/sequence.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0166"></a>

## PR #166: Forge: add governed native release fetch

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-03, FD-06, FD-09.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/166) | [Files changed](https://github.com/mininet-labs/Mininet/pull/166/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/54551e4498772a04ddf9c190456ccd8bf778a64c)

Head `54551e4498772a04ddf9c190456ccd8bf778a64c`; base `f845052c91bbd02d7c51abfbac71ff27245cac4c`; merge `9ce4bd9b0be4f3742b121cb030c712a7367a3457`. 2 changed files; 6 commits; 1 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This adds a user-facing command to retrieve an artifact from native Forge release objects only after governed verification. It removes a temptation to treat an arbitrary downloaded executable or hosted release page as sufficient authority.

**Mechanism and evidence.** mini release fetch verifies the complete governed release, assembles the artifact and writes a new output atomically without overwriting an existing file. The scope is CLI retrieval from the available native object store, not a full peer-discovery or automatic-update service.

**What remains weaker than the intended claim.** The name fetch can be mistaken for network acquisition; the later exact peer retrieval path is a different capability. Verification and writing do not install or adopt the release. The PR's own Linux compile failure at a Cow<str> conversion demonstrates why unrelated local Windows limitations must not be used to dismiss actual changed-code failures.

**Recommended improvement and rationale.** Keep retrieval, verification and owner adoption separate in command help and return types. Bind output bytes to the exact verified artifact and retain no-overwrite semantics. Add transport retrieval only through a bounded evidence closure and reverify at the receiving end.

**Concrete example.** A valid release can be fetched to a file while the running program remains unchanged. A user should be able to inspect it, compare independently reproduced bytes and decline installation; successful retrieval must not imply consent.

**Acceptance tests to implement.** Run the actual CLI with valid/invalid release evidence, an existing output path, missing chunks, corrupt assembled bytes and interruption during output creation. Verify failure cannot overwrite a user file or leave a misleading complete artifact.

**History, supersession and integration.** Follows #104/#105 and #163/#164. #269 adds the genuinely networked exact release retrieval operation. The historical Cow<str> compile correction is retained in the PR discussion rather than hidden behind an all-green summary.

**Source entry points.** [`crates/mini-cli/src/cli.rs`](https://github.com/mininet-labs/Mininet/blob/54551e4498772a04ddf9c190456ccd8bf778a64c/crates/mini-cli/src/cli.rs); [`crates/mini-cli/src/release.rs`](https://github.com/mininet-labs/Mininet/blob/54551e4498772a04ddf9c190456ccd8bf778a64c/crates/mini-cli/src/release.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0170"></a>

## PR #170: Windows social UI, People discovery, and route-scoped Inbox beta

**PARTIAL** | Captured outcome: **merged** | Directives: FD-01, FD-02, FD-06, FD-09, FD-11, FD-16.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/170) | [Files changed](https://github.com/mininet-labs/Mininet/pull/170/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/dfc23dd87252e232f79bef3e096026d55baa322e)

Head `dfc23dd87252e232f79bef3e096026d55baa322e`; base `8211d2980840a16d0d7e159e9d09edf1d5d5a5f9`; merge `80e5b82c0f8a522ccaf9dcbdf05791187158defd`. 31 changed files; 13 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This is the first substantial desktop product integration: public profiles and social actions, explicit nearby discovery, route-scoped private inbox and platform-protected identity state. It connects many earlier pure libraries to real user actions and exposes integration failures that unit tests had missed.

**Mechanism and evidence.** Windows DPAPI protects root/device and conversation state at rest. Day-to-day objects are signed by delegated devices, not directly by the root. Opt-in social announcements remain unverified hints until signed profile/KEL ingestion; endpoint selection uses exact DID equality. KelCache hydration restores known remote identity material across sync sessions. Private messaging uses the v2 envelope/store/private-sync path.

**What remains weaker than the intended claim.** DPAPI does not defend against malware or an administrator acting as that user. A route-scoped inbox is not a reviewed prekey/ratchet protocol and lacks forward secrecy and post-compromise recovery. Public discovery deliberately reveals selected name/DID/endpoint during its window. Platform compilation, a rendered window and same-machine tests are not full real-world client acceptance.

**Recommended improvement and rationale.** Keep public-profile consent separate from anonymous/pseudonymous participation and preserve explicit discovery visibility. Add a reviewed session protocol, multi-device key lifecycle and bounded background/reconnect behavior before claiming production private messaging. Audit DPAPI unwrapping, temporary plaintext and UI-triggered signing; no generic action should silently expose the root.

**Concrete example.** Two users with the same display name must never redirect a friend action; the retained verified DID is the binding. After restart, later objects from a known peer should verify from persisted KEL evidence without retransmitting unchanged carriers, but stale evidence must not silently authorize revoked devices.

**Acceptance tests to implement.** Test exact-DID selection under spoofed announcements, visibility expiry, malformed/slow peers, KEL hydration forks, root/device migration, private/public separation, locked-state restoration and restart. Run on independent physical Windows systems and measure what LAN observers and intermediate peers learn.

**History, supersession and integration.** Integrates #143 private envelopes, #154 social validation, #165 sequence work, #171 platform compilation and #172 extractor coexistence. Android #179 onward reuses this foundation but needs its own custody and physical-device evidence.

**Source entry points.** [`crates/mini-bearer/src/discovery.rs`](https://github.com/mininet-labs/Mininet/blob/dfc23dd87252e232f79bef3e096026d55baa322e/crates/mini-bearer/src/discovery.rs); [`crates/mini-cli/src/sync.rs`](https://github.com/mininet-labs/Mininet/blob/dfc23dd87252e232f79bef3e096026d55baa322e/crates/mini-cli/src/sync.rs); [`crates/mini-desktop/src/conversation_state.rs`](https://github.com/mininet-labs/Mininet/blob/dfc23dd87252e232f79bef3e096026d55baa322e/crates/mini-desktop/src/conversation_state.rs); [`crates/mini-desktop/src/main.rs`](https://github.com/mininet-labs/Mininet/blob/dfc23dd87252e232f79bef3e096026d55baa322e/crates/mini-desktop/src/main.rs); [`crates/mini-messaging/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/dfc23dd87252e232f79bef3e096026d55baa322e/crates/mini-messaging/src/lib.rs); [`crates/mini-messaging/tests/messaging.rs`](https://github.com/mininet-labs/Mininet/blob/dfc23dd87252e232f79bef3e096026d55baa322e/crates/mini-messaging/tests/messaging.rs); [`crates/mini-social/src/discovery.rs`](https://github.com/mininet-labs/Mininet/blob/dfc23dd87252e232f79bef3e096026d55baa322e/crates/mini-social/src/discovery.rs); [`crates/mini-social/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/dfc23dd87252e232f79bef3e096026d55baa322e/crates/mini-social/src/lib.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 2 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0171"></a>

## PR #171: fix(mini-installer): compile on non-Unix hosts (D-0318, closes #167)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-01, FD-06, FD-11, FD-14.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/171) | [Files changed](https://github.com/mininet-labs/Mininet/pull/171/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/b6e682a2ec45478137b56d2ffbc574b42ecd101f)

Head `b6e682a2ec45478137b56d2ffbc574b42ecd101f`; base `9ce4bd9b0be4f3742b121cb030c712a7367a3457`; merge `1afe47731bcc13ff350d6a987cbda89d9bed0dd4`. 7 changed files; 3 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Unblocks the shared CLI and downstream crates on non-Unix targets without pretending that a Unix-only activation operation suddenly works on Windows. This matters because platform exclusion can become practical exclusion from running an independent node.

**Mechanism and evidence.** The installer replaces an unconditional Unix symlink import with platform-specific create_symlink implementations. Unix retains the actual pointer swap; the non-Unix implementation returns an explicit Unsupported I/O error. The rest of owner approval and release verification is unchanged.

**What remains weaker than the intended claim.** This is build portability, not Windows installation support. An unsupported activation must leave the old running release intact, and a Linux test result cannot establish the non-Unix branch. The PR correctly does not substitute a silent successful no-op, but the user-facing installation gap remains.

**Recommended improvement and rationale.** Add a Windows CI target that executes refusal and rollback tests, then design an owner-approved Windows activation strategy with explicit filesystem and privilege assumptions. Keep unsupported-platform errors distinct from an invalid release and from a failed health check.

**Concrete example.** A Windows user verifies a release successfully and presses Adopt. Until a Windows activation backend exists, the result should say activation is unsupported, retain the previous version, and never record the new release as running.

**Acceptance tests to implement.** Cross-compile all consuming crates; on Windows exercise unsupported activation with an existing current pointer and assert no state or event-log success is written. On Unix rerun symlink replacement, failed-health rollback and owner-approval mismatch cases.

**History, supersession and integration.** Fixes a portability blocker encountered by #170 and #172; does not complete the platform-specific installer lifecycle or external acceptance gate.

**Source entry points.** [`crates/mini-installer/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/b6e682a2ec45478137b56d2ffbc574b42ecd101f/crates/mini-installer/src/lib.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 2 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0172"></a>

## PR #172: mini-extract: isolated extractor protocol + process host (D-0319, Track B3)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-06, FD-09, FD-11, FD-14.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/172) | [Files changed](https://github.com/mininet-labs/Mininet/pull/172/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/a4abc81ae5b17d019aa88e80f6dbdbe7bce6f8f5)

Head `a4abc81ae5b17d019aa88e80f6dbdbe7bce6f8f5`; base `1afe47731bcc13ff350d6a987cbda89d9bed0dd4`; merge `77ca66734cd05af7a958a58a2d48e11682dba5a7`. 20 changed files; 3 commits; 1 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Creates a real process boundary for untrusted document extraction and a bounded protocol for moving source bytes into a worker. It makes native intake less dependent on a hosted parsing service while retaining original evidence separately from derived text.

**Mechanism and evidence.** mini-extract-protocol supplies framed requests, ResourceLimits and typed outcomes; mini-extract-host drives a child process with pipe reader/writer threads and a deadline. It bounds declared response length before allocation and kills timed-out workers. The initial text worker normalizes supplied bytes rather than fetching remote resources.

**What remains weaker than the intended claim.** A child process is not automatically a security sandbox: the PR does not demonstrate kernel-enforced memory, filesystem, network or descendant-process restrictions. A parser bug or hostile worker still needs capability confinement. The boundary is not yet the complete intake integration. Zero-deadline scheduling was a real defect later fixed by #183.

**Recommended improvement and rationale.** Attach exact source and worker digests to derived representations; launch the worker with an explicit environment, handle and capability allowlist. Enforce resident-memory and process-tree termination outside the worker, and preserve typed refusal instead of converting malformed extraction into accepted evidence.

**Concrete example.** An HTML/PDF worker tries to open the home directory, make an HTTP request, fork a child and emit a huge length prefix. A hardened host must deny those capabilities, kill the entire job, and preserve the original source unchanged for another parser.

**Acceptance tests to implement.** Test split/truncated frames, oversized declarations, stdout/stderr floods, blocked stdin, child grandchildren, deadline expiry, source-digest substitution and cleanup after failure. Measure peak resident memory on the weakest target; successful extraction must grant no review authority.

**History, supersession and integration.** Builds on #159 intake; #183 repairs zero-budget behavior. #252 static HTML extraction and a future reviewed PDF backend should use a genuinely confined worker rather than relabeling this process boundary as complete isolation.

**Source entry points.** [`crates/mini-extract-host/src/extractor.rs`](https://github.com/mininet-labs/Mininet/blob/a4abc81ae5b17d019aa88e80f6dbdbe7bce6f8f5/crates/mini-extract-host/src/extractor.rs); [`crates/mini-extract-host/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/a4abc81ae5b17d019aa88e80f6dbdbe7bce6f8f5/crates/mini-extract-host/src/lib.rs); [`crates/mini-extract-host/src/main.rs`](https://github.com/mininet-labs/Mininet/blob/a4abc81ae5b17d019aa88e80f6dbdbe7bce6f8f5/crates/mini-extract-host/src/main.rs); [`crates/mini-extract-host/tests/host.rs`](https://github.com/mininet-labs/Mininet/blob/a4abc81ae5b17d019aa88e80f6dbdbe7bce6f8f5/crates/mini-extract-host/tests/host.rs); [`crates/mini-extract-protocol/src/codec.rs`](https://github.com/mininet-labs/Mininet/blob/a4abc81ae5b17d019aa88e80f6dbdbe7bce6f8f5/crates/mini-extract-protocol/src/codec.rs); [`crates/mini-extract-protocol/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/a4abc81ae5b17d019aa88e80f6dbdbe7bce6f8f5/crates/mini-extract-protocol/src/error.rs); [`crates/mini-extract-protocol/src/frame.rs`](https://github.com/mininet-labs/Mininet/blob/a4abc81ae5b17d019aa88e80f6dbdbe7bce6f8f5/crates/mini-extract-protocol/src/frame.rs); [`crates/mini-extract-protocol/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/a4abc81ae5b17d019aa88e80f6dbdbe7bce6f8f5/crates/mini-extract-protocol/src/lib.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0174"></a>

## PR #174: ci(reproducibility): build only the artifacts the job actually hashes (D-0320, closes #173)

**PASS** | Captured outcome: **merged** | Directives: FD-10, FD-11, FD-14.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/174) | [Files changed](https://github.com/mininet-labs/Mininet/pull/174/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/0babb1385930227d87bcb1c89185185710adaa0f)

Head `0babb1385930227d87bcb1c89185185710adaa0f`; base `77ca66734cd05af7a958a58a2d48e11682dba5a7`; merge `01cf2f8102c4ab5c41eb26e0d7e8ae41f11bf275`. 6 changed files; 2 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Aligns reproducibility build cost with the artifacts the job actually compares: four example executables, rather than recompiling every target twice while checking only those same four. This is a justified efficiency correction, not evidence that the whole release is reproducible.

**Mechanism and evidence.** The workflow narrows cargo build invocations to bootstrap_live_demo, frost_live_demo, gossip_live_demo and keystone while retaining separate clean passes and byte-hash comparison. No protocol or runtime authority changes.

**What remains weaker than the intended claim.** The scoped PASS is for eliminating work that was not covered by the job assertion. Same-runner repeatability is weaker than independent builders with different administrative control. The remaining paths-ignore/required-check deadlock was explicitly anticipated here and actually corrected in #185.

**Recommended improvement and rationale.** Publish an artifact coverage manifest and expand it deliberately to the real CLI/client release outputs. Distinguish repeatability on one machine, independent environment reproducibility and independently controlled builders. Derive the docs-only optimization from build inputs, not the assumption that all Markdown can never be embedded.

**Concrete example.** A passing keystone example hash must not become a green badge implying the Android APK or release installer was reproduced. Show four named outputs and four measured digests, then add each release artifact with its own recipe.

**Acceptance tests to implement.** Force one output to differ and require the job to fail; verify untouched artifact hashes are still compared. Test docs-only, build-script, embedded-document and source changes against the skip logic, and preserve exact command/toolchain provenance.

**History, supersession and integration.** Follows #162 workflow separation and leads to #185 reliable check completion and #216 Android reproducibility. It does not close the independent release-reproduction gate.

**Source entry points.** [`.github/workflows/reproducibility.yml`](https://github.com/mininet-labs/Mininet/blob/0babb1385930227d87bcb1c89185185710adaa0f/.github/workflows/reproducibility.yml); [`docs/DECISION_LOG.md`](https://github.com/mininet-labs/Mininet/blob/0babb1385930227d87bcb1c89185185710adaa0f/docs/DECISION_LOG.md); [`governance/work-claims.json`](https://github.com/mininet-labs/Mininet/blob/0babb1385930227d87bcb1c89185185710adaa0f/governance/work-claims.json). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 1 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0176"></a>

## PR #176: mini-crypto: ML-DSA-65 key generation + isolated signing (D-0322, Phase 2)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-06, FD-09, FD-13, FD-14.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/176) | [Files changed](https://github.com/mininet-labs/Mininet/pull/176/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/05fc7aff382a375475372a4cdec71c56ab4008c5)

Head `05fc7aff382a375475372a4cdec71c56ab4008c5`; base `01cf2f8102c4ab5c41eb26e0d7e8ae41f11bf275`; merge `8006eba9747bf71d68bf726c73f2e1649c1e81b9`. 10 changed files; 3 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Adds real ML-DSA-65 key generation and signing to the existing verification-only post-quantum suite, giving the long-horizon identity plan an executable primitive rather than a paper migration promise.

**Mechanism and evidence.** mini-crypto adds an ML-DSA key variant, explicit generate_ml_dsa_65 and sign_ml_dsa_65 operations using fips204 and operating-system randomness. Signature and verifying-key suite tags preserve algorithm distinction. New tests check signing, verification and mismatched-suite refusal.

**What remains weaker than the intended claim.** The new primitive does not make KELs, custody, transport or recovery post-quantum. Legacy infallible sign/seed-export operations still have panic paths for the new key variant, so suite agility is not yet uniformly safe at the API boundary. Producing two different keys is not a comprehensive entropy-health test.

**Recommended improvement and rationale.** Use suite-specific key types or fallible generic signing/export APIs and eliminate reachable wrong-suite panics before routing arbitrary suites through common code. Add independently sourced vectors, secret-lifetime review, persistence design and a staged compatibility matrix for every consumer codec.

**Concrete example.** A caller holding an ML-DSA key should receive an explicit UnsupportedExport error from an Ed25519 seed-export operation, not crash an enrollment process. A PQ-signed object must also survive its full wire decoder, not merely verify in memory.

**Acceptance tests to implement.** Test every public key operation against every supported suite, malformed/non-canonical signatures, randomness failure, serialized size limits and cross-language vectors. Benchmark generation, signing, verification and storage on old phones; do not infer those numbers from workstation tests.

**History, supersession and integration.** Extends the earlier PQ verification work and enables #237 dormant anchors. The #209 storage path and fixed small signature-byte limits elsewhere remain separate migration constraints.

**Source entry points.** [`crates/mini-crypto/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/05fc7aff382a375475372a4cdec71c56ab4008c5/crates/mini-crypto/src/error.rs); [`crates/mini-crypto/src/keys.rs`](https://github.com/mininet-labs/Mininet/blob/05fc7aff382a375475372a4cdec71c56ab4008c5/crates/mini-crypto/src/keys.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0179"></a>

## PR #179: Android mobile foundation: UniFFI core and Compose shell

**PARTIAL** | Captured outcome: **merged** | Directives: FD-01, FD-02, FD-06, FD-08, FD-09, FD-11, FD-14.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/179) | [Files changed](https://github.com/mininet-labs/Mininet/pull/179/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/ca4e4e8c20e4ee7b51e873ccb4da21cc65ffd5b6)

Head `ca4e4e8c20e4ee7b51e873ccb4da21cc65ffd5b6`; base `80e5b82c0f8a522ccaf9dcbdf05791187158defd`; merge `0b241f9e2fe931efbf77aea06a39b28a50d308bd`. 29 changed files; 5 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Establishes an Android shell over the Rust semantic core rather than creating a second protocol implementation in Kotlin. The refusal to fake root creation preserves the distinction between a usable screen and an actual self-sovereign identity.

**Mechanism and evidence.** A pinned UniFFI UDL exposes a deterministic stateless reducer and typed snapshots/actions/events to a thin Compose onboarding UI. Revalidated platform capability inputs prevent a caller-constructed snapshot from inventing stronger security readiness. The initial screen stops at RootCreationReady.

**What remains weaker than the intended claim.** Rust reducer tests do not establish Kotlin compilation, device custody, persistence, background behavior or hardware acceptance. The PR discussion records repeated rebases and correction of earlier overstated test evidence; final exact-head evidence matters more than accumulated claims from earlier branches. UniFFI is third-party licensed code, not part of Mininet CC0 by assertion.

**Recommended improvement and rationale.** Keep platform reports as untrusted capability observations, verify the real native-library/binding/APK combination in CI, and display custody and persistence state explicitly. Provide sideloadable owner-controlled releases and avoid mandatory vendor accounts, push services or hardware-attestation authorities.

**Concrete example.** After choosing Create root, an unimplemented secure signer must yield a visible unavailable state; the UI must not display a made-up DID or imply hardware custody. Later software-wrapped custody should be named accurately rather than inheriting the original hardware aspiration.

**Acceptance tests to implement.** Run deterministic replay and forged-snapshot tests plus real Kotlin generation, APK assembly, cold-start, rotation, accessibility and low-memory device tests. Verify the package has no unrequested telemetry or privileged permissions and can operate without Play Services.

**History, supersession and integration.** The long-lived foundation branch was integrated alongside #170-#208; #206/#209 add identity and storage primitives, #217 UI creation, #250 actual Keystore wrapping, #258 LAN/QR. Each is a separate evidence increment, not retroactive proof of this PR.

**Source entry points.** [`crates/mini-ffi/src/bin/uniffi-bindgen.rs`](https://github.com/mininet-labs/Mininet/blob/ca4e4e8c20e4ee7b51e873ccb4da21cc65ffd5b6/crates/mini-ffi/src/bin/uniffi-bindgen.rs); [`crates/mini-ffi/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/ca4e4e8c20e4ee7b51e873ccb4da21cc65ffd5b6/crates/mini-ffi/src/lib.rs); [`crates/mini-ffi/src/mini_ffi.udl`](https://github.com/mininet-labs/Mininet/blob/ca4e4e8c20e4ee7b51e873ccb4da21cc65ffd5b6/crates/mini-ffi/src/mini_ffi.udl). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0180"></a>

## PR #180: did-mini: KEL witness receipt types (D-0321, Phase 1 of D-0096)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-06, FD-08, FD-09, FD-14.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/180) | [Files changed](https://github.com/mininet-labs/Mininet/pull/180/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/1bf1245c8677310bfaff018eb7f8701d61cc9e73)

Head `1bf1245c8677310bfaff018eb7f8701d61cc9e73`; base `8006eba9747bf71d68bf726c73f2e1649c1e81b9`; merge `fe19bd048d0dd89ab6a705000fb1efd2c70388d4`. 10 changed files; 3 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Provides the first concrete receipt/certificate vocabulary for detecting divergent identity histories without a global identity registry. It makes the first-contact freshness research inspectable in code.

**Mechanism and evidence.** WitnessPolicy validates a non-empty distinct witness set and threshold; WitnessedEventCertificate assembles matching identity/sequence/digest/generation receipts and verifies witness membership, uniqueness, signatures and quorum. Versioned bounded codecs reject malformed input. Witnesses attest observations, not control of the identity.

**What remains weaker than the intended claim.** The initial certificate verifies relative to a supplied policy and key resolver. It does not establish that the identity appointed that policy, that the event is a valid KEL head, that observation epochs are trustworthy, or that witness operators are independent. Public struct fields also mean constructor validation alone must never be treated as the only verifier defense.

**Recommended improvement and rationale.** Make the complete trust chain explicit: verified KEL event, identity-declared policy, historically valid witness key, matching receipt and locally defined freshness. Keep certificate verification distinct from an authorization decision and require deterministic revalidation of all policy fields at trust boundaries.

**Concrete example.** Two attacker-operated witnesses can validly sign a receipt for a named event; this proves only those signatures until the recipient independently establishes that the legitimate identity chose those witnesses and signed the event.

**Acceptance tests to implement.** Add adversarial policies constructed without the constructor, duplicate roots, wrong generation, substituted event, unresolved/rotated witness keys and threshold edge cases. Test byte round trips with maximum supported signature suites and independently generated vectors.

**History, supersession and integration.** Implements #149/D-0096 Phase 1. #187 adds state, #191 KEL assurance, #314 binds policy to signed establishment events, and #320-#325 add protocol/persistence/gossip/rotation slices. None alone proves global freshness.

**Source entry points.** [`crates/did-mini/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/1bf1245c8677310bfaff018eb7f8701d61cc9e73/crates/did-mini/src/error.rs); [`crates/did-mini/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/1bf1245c8677310bfaff018eb7f8701d61cc9e73/crates/did-mini/src/lib.rs); [`crates/did-mini/src/witness.rs`](https://github.com/mininet-labs/Mininet/blob/1bf1245c8677310bfaff018eb7f8701d61cc9e73/crates/did-mini/src/witness.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0181"></a>

## PR #181: docs: add the project's first public whitepaper (D-0323)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-01, FD-03, FD-07, FD-10, FD-13, FD-16.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/181) | [Files changed](https://github.com/mininet-labs/Mininet/pull/181/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/992fe74b61c92c3caec0ec1eb8e4f5398b761334)

Head `992fe74b61c92c3caec0ec1eb8e4f5398b761334`; base `624cc414211e70a72ecaa9ac1fc0bfdb377d4069`; merge `874c2f95308d551c9e6ce3bbf8a000bf4fa14ec9`. 7 changed files; 3 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Creates the first public whitepaper actually committed in this repository, making the project understandable without access to private founder conversations or scattered external drafts. That is a substantive transparency contribution.

**Mechanism and evidence.** WHITEPAPER.md describes implemented, prototype and not-yet-built domains, points to canonical directives/invariants/decisions, and labels four additional product/economic directions as proposals. README and STATUS gain entry links rather than a parallel enforcement mechanism.

**What remains weaker than the intended claim.** A public narrative can still overtake its evidence. In particular, proposed identity-weighted discovery must not become a hidden prerequisite for public reach, and a coherent whitepaper does not establish one-human-one-vote or audited cryptography. This is public explanation, not activation of every idea it mentions.

**Recommended improvement and rationale.** Maintain a claim-to-code-to-test map for each public guarantee, with explicit proposal labels and dated maturity. Reconcile authority precedence through a dedicated human-reviewed document instead of letting prose silently amend canonical values. Explain free participation separately from paid resource service.

**Concrete example.** A first-time reader should be able to distinguish an implemented wallet-independent public comment from a proposed private payment and from an externally unvalidated uniqueness mechanism in one table, without translating the word beta into a security promise.

**Acceptance tests to implement.** Check every guarantee link at the pinned revision; add negative examples for unpaid participation, absent personhood, unavailable edge providers and unsafe launch. Human review must examine semantic accuracy; a link checker only establishes that targets exist.

**History, supersession and integration.** Builds on #127 canonicalization and #153 public commons/Search doctrine; later #218 adds Directive 18 and #310 publishes a release-critical path. Historical versions must remain readable rather than rewritten to look prescient.

**Source entry points.** [`README.md`](https://github.com/mininet-labs/Mininet/blob/992fe74b61c92c3caec0ec1eb8e4f5398b761334/README.md); [`WHITEPAPER.md`](https://github.com/mininet-labs/Mininet/blob/992fe74b61c92c3caec0ec1eb8e4f5398b761334/WHITEPAPER.md); [`docs/DECISION_LOG.md`](https://github.com/mininet-labs/Mininet/blob/992fe74b61c92c3caec0ec1eb8e4f5398b761334/docs/DECISION_LOG.md); [`docs/STATUS.md`](https://github.com/mininet-labs/Mininet/blob/992fe74b61c92c3caec0ec1eb8e4f5398b761334/docs/STATUS.md). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0183"></a>

## PR #183: fix(mini-extract-host): make a zero-millisecond deadline a deterministic timeout

**PASS** | Captured outcome: **merged** | Directives: FD-06, FD-10, FD-11.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/183) | [Files changed](https://github.com/mininet-labs/Mininet/pull/183/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/7b9aa75fae96cc90e0b583c3e8f0e9f2c9f2b6d4)

Head `7b9aa75fae96cc90e0b583c3e8f0e9f2c9f2b6d4`; base `fe19bd048d0dd89ab6a705000fb1efd2c70388d4`; merge `6bd8e4ad75acdd86628e3d92026276f324c5256e`. 6 changed files; 1 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Fixes a real flaky deadline test by defining the zero-budget operation rather than weakening the assertion. It protects both reliable CI and an understandable failure contract for extraction.

**Mechanism and evidence.** run_worker detects max_wall_clock_ms == 0, terminates the child and returns a typed Timeout before racing recv_timeout against an already-queued frame. Non-zero deadlines retain their existing behavior.

**What remains weaker than the intended claim.** The scoped PASS covers zero-deadline determinism, not complete worker containment. Thirty repeated passes reported by the author support the regression but do not establish descendant cleanup, a total deadline including startup, or memory isolation. Starting a child only to kill it is also unnecessary work for a known-zero budget.

**Recommended improvement and rationale.** Validate impossible budgets before process creation where API compatibility permits. Track an absolute monotonic job deadline from the start of host work and apply the remaining budget to every blocking phase. Ensure cleanup has bounded behavior independent of worker cooperation.

**Concrete example.** A zero-millisecond request should fail before any extractor starts. A ten-millisecond request must not spend seconds writing blocked stdin and only then begin its ten-millisecond output timer.

**Acceptance tests to implement.** Test zero without spawning, extremely small positive budgets, delayed spawn/write/read, output already queued, worker descendants and kill failure. Assert identical typed outcome and no successful representation after the deadline.

**History, supersession and integration.** Remediates #172/D-0319 directly; it should be cited as a resolved historical race, not carried forward as a current open zero-timeout bug.

**Source entry points.** [`crates/mini-extract-host/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/7b9aa75fae96cc90e0b583c3e8f0e9f2c9f2b6d4/crates/mini-extract-host/src/lib.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0185"></a>

## PR #185: fix(ci): make the reproducibility required check always reach a conclusion

**PASS** | Captured outcome: **merged** | Directives: FD-06, FD-10, FD-14.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/185) | [Files changed](https://github.com/mininet-labs/Mininet/pull/185/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/fa2963f7a58fa0874db7ceefa32d4049e33899b1)

Head `fa2963f7a58fa0874db7ceefa32d4049e33899b1`; base `6bd8e4ad75acdd86628e3d92026276f324c5256e`; merge `624cc414211e70a72ecaa9ac1fc0bfdb377d4069`. 7 changed files; 1 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Repairs the CI liveness failure in which a documentation-only PR could never satisfy a required reproducibility check because the workflow never started. It fixes the mechanism rather than disabling the required protection.

**Mechanism and evidence.** The pull_request trigger runs unconditionally; an in-job diff determines whether build steps can be skipped, while an explicit explanatory step produces a conclusion. Main pushes continue to execute the comparison. Full history checkout enables the base comparison.

**What remains weaker than the intended claim.** A completed skip is not a reproduced binary, and the broad docs/Markdown predicate is safe only while those paths cannot affect the relevant outputs. Shell pipeline failure must not be misclassified as an empty/docs-only diff. The workflow conclusion and the substantive evidence remain different facts.

**Recommended improvement and rationale.** Return an explicit performed/skipped/failed classification with reason and commit pair. Make diff-command errors fail closed and use null-delimited filenames or a checked parser. Test embedded documentation and generated code inputs before treating extensions as a security boundary.

**Concrete example.** A missing base commit should produce a failed scope-detection step, not docs_only=true. A pure prose edit can finish rapidly but the PR summary should state that reproducibility was not rerun.

**Acceptance tests to implement.** Exercise docs-only, mixed source/docs, deleted/renamed files, unusual filenames, unavailable base SHA and generated-source changes. Require a terminal check for every PR and real artifact comparisons for all build-affecting cases.

**History, supersession and integration.** Closes the risk noted by #174 and observed on #181; the same pattern is reused in #214/#216 and should be reviewed there independently rather than assumed correct by copying.

**Source entry points.** [`.github/workflows/reproducibility.yml`](https://github.com/mininet-labs/Mininet/blob/fa2963f7a58fa0874db7ceefa32d4049e33899b1/.github/workflows/reproducibility.yml); [`docs/DECISION_LOG.md`](https://github.com/mininet-labs/Mininet/blob/fa2963f7a58fa0874db7ceefa32d4049e33899b1/docs/DECISION_LOG.md); [`docs/STATUS.md`](https://github.com/mininet-labs/Mininet/blob/fa2963f7a58fa0874db7ceefa32d4049e33899b1/docs/STATUS.md); [`governance/work-claims.json`](https://github.com/mininet-labs/Mininet/blob/fa2963f7a58fa0874db7ceefa32d4049e33899b1/governance/work-claims.json). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0187"></a>

## PR #187: did-mini: in-memory witness state machine + duplicity proofs (D-0326, Phase 2 of KEL witness receipts)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-06, FD-08, FD-14, FD-15.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/187) | [Files changed](https://github.com/mininet-labs/Mininet/pull/187/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/9f2f26d3182668b9aa2220de865497d8fb65a83c)

Head `9f2f26d3182668b9aa2220de865497d8fb65a83c`; base `874c2f95308d551c9e6ce3bbf8a000bf4fa14ec9`; merge `c391b2134bc24b1e9bfc970633aec63bb0989995`. 10 changed files; 3 commits; 0 issue comments, 18 inline comments and 1 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Turns witness receipts into a stateful anti-equivocation mechanism: the witness remembers what it has accepted and can return objective conflicting-event material instead of silently signing every request.

**Mechanism and evidence.** WitnessJournal distinguishes first observation, direct successor, duplicate, stale ancestor, same-sequence conflict and conflicting descendant. It retains actual event/receipt bytes; duplicates return the old receipt without re-signing. ControllerDuplicityProof initially checks structure, while WitnessEquivocationProof verifies conflicting witness signatures.

**What remains weaker than the intended claim.** The raw observe entry point trusts the caller to establish event authorization and policy. Structural pairs of conflicting events are not yet verified accusations. In-memory memory loss can erase anti-equivocation history, and future stateful signing must be crash-safe before receipts leave the process. Many CodeQL comments concern test constants, not automatically exploitable key hardcoding.

**Recommended improvement and rationale.** Admit only a verified KEL/policy result into the signing state machine; expose structural conflict candidates separately from verified fault evidence. Durably reserve each signing slot before returning a receipt and define bounds and recovery without silently forgetting still-relevant slots.

**Concrete example.** After witnessing event A at sequence 12, a crash followed by conflicting B at sequence 12 must not produce a second signature. Conversely, an unsigned fabricated B must not be stored as proof that the controller misbehaved.

**Acceptance tests to implement.** Test invalid controller signatures, wrong predecessor, gapped chains, repeated messages at different local times, restart after every write boundary, independent concurrent processes and invalid evidence submitted to a duplicity registry.

**History, supersession and integration.** Builds on #180. #191 adds observe_verified and a registry; #320 routes declarations through the KEL; #322 persistence and #325 transition certification need additional crash/slot tests.

**Source entry points.** [`crates/did-mini/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/9f2f26d3182668b9aa2220de865497d8fb65a83c/crates/did-mini/src/error.rs); [`crates/did-mini/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/9f2f26d3182668b9aa2220de865497d8fb65a83c/crates/did-mini/src/lib.rs); [`crates/did-mini/src/witness_state.rs`](https://github.com/mininet-labs/Mininet/blob/9f2f26d3182668b9aa2220de865497d8fb65a83c/crates/did-mini/src/witness_state.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0189"></a>

## PR #189: mini-store: chronological index — Store::since/Store::recent (D-0327, Batch 5 forge priority)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-06, FD-10, FD-11, FD-14.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/189) | [Files changed](https://github.com/mininet-labs/Mininet/pull/189/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/5fe8f7bad653be8c9ae7b2caf1c3e0b73f441fc0)

Head `5fe8f7bad653be8c9ae7b2caf1c3e0b73f441fc0`; base `c391b2134bc24b1e9bfc970633aec63bb0989995`; merge `ec3ed65a17d54cc0e1d4e99f0cd43ec346421b5a`. 9 changed files; 1 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Adds chronological object discovery for local feeds and Forge activity without introducing a hosted index. This removes repeated object-body parsing for a common user operation.

**Mechanism and evidence.** Store::insert adds idx/time/<fixed-width timestamp>/<object-id> metadata. Store::since and recent read that index to order results; the object timestamp is the author claim, not a consensus timestamp or receipt time.

**What remains weaker than the intended claim.** The initial implementation still scans the full index before filtering/truncating, so a result limit is not a work limit. A timestamp-only cursor also cannot be a lossless synchronization frontier when new backdated objects arrive. Multiple independent index writes need a crash-consistency strategy.

**Recommended improvement and rationale.** Use a composite timestamp/object-id cursor and a bounded backend range API for browsing; use content reconciliation, not author time, to discover late arrivals. Make secondary indexes reconstructible from canonical objects and test insertion crash boundaries.

**Concrete example.** Two posts with identical timestamps must both appear across pages. A post arriving tomorrow but stamped yesterday must still be discovered through sync even though it falls before today's browsing cursor.

**Acceptance tests to implement.** Test equal timestamps, reverse arrival, hostile future/backdated values, partial index writes, rebuild and fixed-view pagination parity. Measure index reads as well as result count for a million-row store.

**History, supersession and integration.** #193 bounds MemoryBackend recent lookup; #287 adds the genuine filesystem time index and stable pages. Do not attribute #287's complexity bounds to this first version.

**Source entry points.** [`crates/mini-store/src/store.rs`](https://github.com/mininet-labs/Mininet/blob/5fe8f7bad653be8c9ae7b2caf1c3e0b73f441fc0/crates/mini-store/src/store.rs); [`crates/mini-store/tests/store.rs`](https://github.com/mininet-labs/Mininet/blob/5fe8f7bad653be8c9ae7b2caf1c3e0b73f441fc0/crates/mini-store/tests/store.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0191"></a>

## PR #191: did-mini: KEL witness receipts Phase 3 — KelAssurance, chain-verified observe, duplicity registry (D-0328/D-0329/D-0330)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-06, FD-08, FD-09, FD-15.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/191) | [Files changed](https://github.com/mininet-labs/Mininet/pull/191/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/1a58189e931fa85cffe2fd73f0b79c89b6c43313)

Head `1a58189e931fa85cffe2fd73f0b79c89b6c43313`; base `ec3ed65a17d54cc0e1d4e99f0cd43ec346421b5a`; merge `d489941c2d52ec3aff11fe1e38103f0b5dabf017`. 11 changed files; 4 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Introduces graded identity assurance rather than flattening all valid signatures into one boolean, and adds a real KEL-verifying witness path plus local duplicity accumulation.

**Mechanism and evidence.** assess_kel_assurance combines KEL verification/freshness pins, a witness certificate and a known-duplicity signal into Direct, Pinned, Witnessed, WitnessedRecent or DuplicityDetected. observe_verified checks the chain before invoking witness observation. DuplicityRegistry stores supplied proof objects for later lookup.

**What remains weaker than the intended claim.** The original witness policy remained caller-supplied until #314, and a local known_duplicity flag or structurally assembled proof is not authenticated fault evidence by itself. Freshness depends on retained observations and time assumptions. The raw less-checked observe route remains available, so the safe path must be made mandatory where signing matters.

**Recommended improvement and rationale.** Create verified evidence types only after controller/witness historical signature checks; separate an allegation store from an adjudicated cryptographic conflict store. Make current policy provenance and time basis explicit in assurance results. Consumers should receive the evidence and reason, not just an ordinal rank.

**Concrete example.** A network peer sends two fabricated Event structs naming Alice. They may be stored as an untrusted allegation for investigation, but must not cause Alice's governance actions to be rejected as proven duplicity.

**Acceptance tests to implement.** Test forged fault records, unappointed witness policy, equal-height divergent KELs, expired/future epochs and loss of local pins. Require a valid recent certificate to fail when it is about another event, identity or policy generation.

**History, supersession and integration.** Extends #180/#187; #195 exposes assurance to Forge without enforcing it, #314 fixes policy origin, and #322-#325 add incomplete runtime ingredients. Graded labels remain non-authorizing until an explicit consumer policy is adopted.

**Source entry points.** [`crates/did-mini/src/assurance.rs`](https://github.com/mininet-labs/Mininet/blob/1a58189e931fa85cffe2fd73f0b79c89b6c43313/crates/did-mini/src/assurance.rs); [`crates/did-mini/src/duplicity.rs`](https://github.com/mininet-labs/Mininet/blob/1a58189e931fa85cffe2fd73f0b79c89b6c43313/crates/did-mini/src/duplicity.rs); [`crates/did-mini/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/1a58189e931fa85cffe2fd73f0b79c89b6c43313/crates/did-mini/src/lib.rs); [`crates/did-mini/src/witness_state.rs`](https://github.com/mininet-labs/Mininet/blob/1a58189e931fa85cffe2fd73f0b79c89b6c43313/crates/did-mini/src/witness_state.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0193"></a>

## PR #193: mini-store: genuinely bounded "most recent N" backend query (D-0331)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-10, FD-11, FD-14.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/193) | [Files changed](https://github.com/mininet-labs/Mininet/pull/193/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/6080b253a802aed6e060ffbe65806c38204b11a5)

Head `6080b253a802aed6e060ffbe65806c38204b11a5`; base `d489941c2d52ec3aff11fe1e38103f0b5dabf017`; merge `061dbf0a7c5bfd238bb03b40c23dd42f404d93b3`. 9 changed files; 1 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Makes the in-memory recent-object operation genuinely bounded rather than truncating after a full scan. The explicit backend boundary also reveals that the filesystem path still has a different cost.

**Mechanism and evidence.** Backend::list_meta_prefix_last has a compatibility fallback; MemoryBackend overrides it with BTreeMap range/reverse/take. Store::recent delegates to that primitive. The upper-bound sentinel relies on the documented restricted ASCII key alphabet.

**What remains weaker than the intended claim.** The fallback still reads and sorts every filesystem row. A shared API name does not imply shared asymptotic performance, and a future key alphabet expansion could invalidate the sentinel range. The PR correctly names this remaining gap rather than hiding it in the trait default.

**Recommended improvement and rationale.** Document cost guarantees per backend or expose a capability distinguishing bounded from compatibility queries. Centralize the key-alphabet invariant and test prefix ranges at byte boundaries. Migrate interactive filesystem callers to #287's bounded API rather than assuming this change already solved disk scale.

**Concrete example.** A phone asking for 20 recent items should not load a million filesystem records because an in-memory test returned 20 efficiently. Instrument storage reads in the actual backend, not only the length of returned results.

**Acceptance tests to implement.** Check empty/zero/large limits, adjacent prefixes, all permitted key characters and backend result parity. Add I/O-count assertions and physical cold-cache latency tests for the filesystem implementation.

**History, supersession and integration.** Optimizes #189's MemoryBackend branch only; #287 is the later disk-index implementation. Store::since remains an explicitly unbounded compatibility operation.

**Source entry points.** [`crates/mini-store/src/backend.rs`](https://github.com/mininet-labs/Mininet/blob/6080b253a802aed6e060ffbe65806c38204b11a5/crates/mini-store/src/backend.rs); [`crates/mini-store/src/store.rs`](https://github.com/mininet-labs/Mininet/blob/6080b253a802aed6e060ffbe65806c38204b11a5/crates/mini-store/src/store.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0195"></a>

## PR #195: mini-forge: bridge KelAssurance into the identity oracle as author_assurance (D-0332)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-06, FD-08, FD-12, FD-15.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/195) | [Files changed](https://github.com/mininet-labs/Mininet/pull/195/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/c01e170a9a97231e4b82836b48e17be37af2098a)

Head `c01e170a9a97231e4b82836b48e17be37af2098a`; base `061dbf0a7c5bfd238bb03b40c23dd42f404d93b3`; merge `57ebc31aaf901df69d52ea5d10eef83d1c6b390a`. 9 changed files; 2 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Connects the identity assurance vocabulary to Forge authorship, so governance can eventually require stronger evidence without reimplementing KEL verification in every decision path.

**Mechanism and evidence.** author_assurance first reruns author_verified provenance checks, then assesses the author-root KEL with the existing pins, witness evidence and duplicity input. It evaluates the root that counts toward review quorum rather than treating each device as a separate voter.

**What remains weaker than the intended claim.** No propose/approve/merge/resolve_project call is changed to require this result. The bridge is therefore useful machinery but not a protection currently enforced on governance. Silently turning every witness outage into denial of basic publishing would also contradict the founder's independence requirements.

**Recommended improvement and rationale.** Adopt an operation-specific assurance policy through the proper human process, then make high-value approval/release consumers require a verified assurance object. Keep ordinary speech and local work available under weaker explicitly labeled states. Bind the required policy to the exact action/epoch rather than a mutable UI preference.

**Concrete example.** A public draft may be accepted with Direct evidence, while canonical release approval requires recent authenticated witness continuity. When witnesses disappear, drafting continues; release authorization waits rather than inventing freshness.

**Acceptance tests to implement.** Add tests at actual approve/merge/release entry points showing that insufficient, revoked, stale or forged assurance cannot be bypassed. Verify duplicate devices still count one root and a service outage cannot globally disable unrelated public participation.

**History, supersession and integration.** Builds on #191. Later policy binding in #314 improves the evidence underneath but does not itself wire this bridge into authority. R9 must remain open until the complete consumer path is demonstrated.

**Source entry points.** [`crates/mini-forge/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/c01e170a9a97231e4b82836b48e17be37af2098a/crates/mini-forge/src/lib.rs); [`crates/mini-forge/src/oracle.rs`](https://github.com/mininet-labs/Mininet/blob/c01e170a9a97231e4b82836b48e17be37af2098a/crates/mini-forge/src/oracle.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0206"></a>

## PR #206: Android beta slice 1 (D-0334/D-0335) + slice 2 building block (D-0337)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-01, FD-06, FD-08, FD-09, FD-11, FD-14.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/206) | [Files changed](https://github.com/mininet-labs/Mininet/pull/206/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/157cea8ddd9b076b8dc4f6b2802593aae3653415)

Head `157cea8ddd9b076b8dc4f6b2802593aae3653415`; base `0b241f9e2fe931efbf77aea06a39b28a50d308bd`; merge `5494f01fa5f422d5478c8a414e0edc43170f6cc1`. 13 changed files; 6 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Brings genuine root/device creation to the Android Rust boundary and supplies a restore operation that resumes an identity without rotating it on every launch. This is a major step from onboarding mock-up to an actual controller.

**Mechanism and evidence.** RootCore wraps real did-mini controllers, creates/delegates/revokes devices and exposes public identity information. Controller::restore re-verifies the KEL, current key set and next-key commitments before constructing a controller without appending an event. The Keystore adapter design compares hardware-signer and software-wrapped options.

**What remains weaker than the intended claim.** The delivered MVP holds software keys in one process; it is not hardware-isolated signing and initially lacks persistence and UI integration. Restoring cryptographically consistent old state does not detect rollback of the entire local store. The final stacked diff includes persistence work whose separate provenance is #209; do not count it twice as an independent assurance.

**Recommended improvement and rationale.** Keep create, restore and recover as distinct operations with explicit custody labels. Add durable anti-rollback state and user-visible recovery failure, and ensure sensitive operations cannot be invoked through an unrestricted generic UI signing route. Prefer small suite-specific signer interfaces over pretending every hardware keystore implements all Mininet derivations.

**Concrete example.** Reopening the application should resume the same KEL bytes. A restore with a valid historical KEL but superseded local secrets must not silently restore revoked authority; it needs retained freshness evidence or an explicit degraded recovery state.

**Acceptance tests to implement.** Test current/next-key mismatches, old-state replay, partially persisted delegation, root/device separation, failed restore without automatic replacement identity, and key lifetime across FFI. Run the actual UI/platform path, not just two RootCore instances.

**History, supersession and integration.** Builds on #179; #209 adds wrapping callbacks, #210 two-party enrollment, #217 UI calls and #250 platform AES-GCM persistence. None removes the need for independent custody review.

**Source entry points.** [`crates/did-mini/src/controller.rs`](https://github.com/mininet-labs/Mininet/blob/157cea8ddd9b076b8dc4f6b2802593aae3653415/crates/did-mini/src/controller.rs); [`crates/did-mini/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/157cea8ddd9b076b8dc4f6b2802593aae3653415/crates/did-mini/src/error.rs); [`crates/did-mini/tests/restore.rs`](https://github.com/mininet-labs/Mininet/blob/157cea8ddd9b076b8dc4f6b2802593aae3653415/crates/did-mini/tests/restore.rs); [`crates/mini-ffi/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/157cea8ddd9b076b8dc4f6b2802593aae3653415/crates/mini-ffi/src/lib.rs); [`crates/mini-ffi/src/mini_ffi.udl`](https://github.com/mininet-labs/Mininet/blob/157cea8ddd9b076b8dc4f6b2802593aae3653415/crates/mini-ffi/src/mini_ffi.udl). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 2 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0208"></a>

## PR #208: mini-bounty: project-label pools for paying contributors at different rates (D-0336)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-04, FD-09, FD-14, FD-16.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/208) | [Files changed](https://github.com/mininet-labs/Mininet/pull/208/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/cbbea8e119bc4617e1e74c7b28c8dbaf7b39b982)

Head `cbbea8e119bc4617e1e74c7b28c8dbaf7b39b982`; base `57ebc31aaf901df69d52ea5d10eef83d1c6b390a`; merge `8211d2980840a16d0d7e159e9d09edf1d5d5a5f9`. 7 changed files; 1 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Extends the existing bounty mechanism to group differently priced funding rounds without inventing a new usefulness oracle or treasury authority. It demonstrates a valuable refusal to implement an ungrounded external draft as if it were project policy.

**Mechanism and evidence.** BountyPool gains an optional bounded project label. Each pool retains one flat amount_per_grant_micro; different amounts require distinct pools. The label is organizational and does not change claim verification, quorum or governance weight.

**What remains weaker than the intended claim.** Flat amounts reduce one within-pool identifying signal but do not make distinct pools anonymous to one another: a small, uniquely priced pool can still identify a recipient. A project label can itself reveal an association. This is metadata convenience over an unaudited claim construction, not private contributor payroll readiness.

**Recommended improvement and rationale.** Document the observable pool-size, amount and label fingerprint and support consentful minimal labels. Keep contribution assessment separate from payment authorization and from political power. Review anonymity-set population and cross-pool timing before promising unlinkability.

**Concrete example.** A maintainer paid from a one-person premium pool can be recognizable even though every grant in that pool has the same amount. A neutral shared project label does not enlarge that anonymity set.

**Acceptance tests to implement.** Check label byte bounds, unchanged claim/ring behavior, distinct pool IDs, no governance dependency and no amount-dependent approval logic. Add threat experiments correlating small pools, payout timing and public contributor activity.

**History, supersession and integration.** Extends the earlier mini-bounty path under D-0073 rather than replacing treasury design. Private payment composition #305/#312 is a separate mechanism and does not retroactively audit bounty anonymity.

**Source entry points.** [`crates/mini-bounty/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/cbbea8e119bc4617e1e74c7b28c8dbaf7b39b982/crates/mini-bounty/src/lib.rs); [`crates/mini-bounty/src/pool.rs`](https://github.com/mininet-labs/Mininet/blob/cbbea8e119bc4617e1e74c7b28c8dbaf7b39b982/crates/mini-bounty/src/pool.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0209"></a>

## PR #209: mini-ffi: StorageCipher callback + RootCore persist/restore (D-0338)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-06, FD-09, FD-11, FD-14.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/209) | [Files changed](https://github.com/mininet-labs/Mininet/pull/209/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/6394bcd67be28003fac289ce6f86009453379e1a)

Head `6394bcd67be28003fac289ce6f86009453379e1a`; base `be178e1d55543283da3373b8c4b40896e0d77c60`; merge `157cea8ddd9b076b8dc4f6b2802593aae3653415`. 10 changed files; 1 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Provides the real serialization/encryption callback boundary required to preserve a root across app restarts. It also makes an important limitation explicit: wrapping keys with a platform cipher is not the same as keeping all key bytes outside the managed runtime.

**Mechanism and evidence.** RootCore::persist_state encodes KELs and current/next Ed25519 seeds, then passes plaintext to StorageCipher::encrypt. restore calls decrypt, decodes and verifies controllers, and zeroizes the Rust-owned decrypted buffer. Controller exposes an explicitly named storage-key export operation.

**What remains weaker than the intended claim.** The plaintext buffer crosses UniFFI into Kotlin/JVM memory and can have copies Rust cannot erase. The callback may be a test cipher or hostile implementation; the trait alone proves no encryption security. Key exports also enlarge the sensitive API surface. Encryption authenticates bytes but does not prevent rollback to an older authentic ciphertext.

**Recommended improvement and rationale.** State the exact software-wrapped custody guarantee, minimize copies and lifetime across FFI, and keep platform cipher implementations narrow and reviewed. Prefer native-side envelope encryption with a keystore-wrapped data key where it meaningfully reduces managed plaintext, while accounting for all copies and recovery constraints.

**Concrete example.** A compromised StorageCipher callback can copy every root seed even while returning valid AES-GCM ciphertext. Therefore a passing round-trip test cannot justify the phrase private-key bytes never cross FFI.

**Acceptance tests to implement.** Test authenticated decryption failure, malformed plaintext, excessive identity counts, early-return zeroization, callback errors, rollback detection and lifecycle writes. Inspect generated bindings and platform heap behavior; never ship the XOR test cipher as an acceptable fallback.

**History, supersession and integration.** Completes the storage building block anticipated by #206; #250 supplies the real Android implementation and #258 persists additional pairing/replay state. The earlier broad no-key-crossing claim must remain narrowed.

**Source entry points.** [`crates/did-mini/src/controller.rs`](https://github.com/mininet-labs/Mininet/blob/6394bcd67be28003fac289ce6f86009453379e1a/crates/did-mini/src/controller.rs); [`crates/did-mini/tests/restore.rs`](https://github.com/mininet-labs/Mininet/blob/6394bcd67be28003fac289ce6f86009453379e1a/crates/did-mini/tests/restore.rs); [`crates/mini-ffi/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/6394bcd67be28003fac289ce6f86009453379e1a/crates/mini-ffi/src/lib.rs); [`crates/mini-ffi/src/mini_ffi.udl`](https://github.com/mininet-labs/Mininet/blob/6394bcd67be28003fac289ce6f86009453379e1a/crates/mini-ffi/src/mini_ffi.udl). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0210"></a>

## PR #210: mini-ffi: two-party device enrollment/revocation (D-0339)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-06, FD-08, FD-09, FD-14.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/210) | [Files changed](https://github.com/mininet-labs/Mininet/pull/210/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/49e82b52969a454f7762c5545c80dcf732dd2a15)

Head `49e82b52969a454f7762c5545c80dcf732dd2a15`; base `0b241f9e2fe931efbf77aea06a39b28a50d308bd`; merge `be7392fc6fc4fc8affcae096ca66f29580d97b85`. 13 changed files; 7 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Makes a second device create its own secrets while the root holder approves only public enrollment material. This is the right architectural direction for a base device plus multiple personal devices without a custodial account server.

**Mechanism and evidence.** begin_device_enrollment creates a device KEL locally; approve_device_enrollment verifies its claimed delegator and adds delegation to the root KEL; finish_device_enrollment verifies the mutual link before promotion. Revocation can name a delegated device without holding its secret keys.

**What remains weaker than the intended claim.** The tests transfer bytes between in-process objects, not two independently secured phones. The protocol still needs authenticated transport, explicit owner confirmation, persistence of pending/completed states and freshness-aware revocation propagation. A valid request is not evidence that the human intended to enroll that device.

**Recommended improvement and rationale.** Bind enrollment to a single expiring ceremony, a verified root fingerprint and an explicit capability selection. Persist the ceremony safely and prevent duplicate approval from becoming hidden new privileges. Preserve root-only authority without requiring the root secret on every client.

**Concrete example.** A malicious nearby phone submits its own valid device KEL naming Alice's root. Alice must see and approve that exact device and its requested capabilities; cryptographic validity alone must not trigger delegation.

**Acceptance tests to implement.** Test swapped roots/devices, stale approval, duplicate and interrupted ceremonies, revocation after restart, least-privilege capabilities and an active network intermediary. Capture a two-physical-device trace showing no root-secret bytes cross the connection.

**History, supersession and integration.** Uses #206/#209 identity state. #211/#258 provide related LAN/QR pairing surfaces, but social following and device enrollment are different ceremonies and must not silently authorize one another.

**Source entry points.** [`crates/did-mini/src/controller.rs`](https://github.com/mininet-labs/Mininet/blob/49e82b52969a454f7762c5545c80dcf732dd2a15/crates/did-mini/src/controller.rs); [`crates/did-mini/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/49e82b52969a454f7762c5545c80dcf732dd2a15/crates/did-mini/src/error.rs); [`crates/did-mini/tests/restore.rs`](https://github.com/mininet-labs/Mininet/blob/49e82b52969a454f7762c5545c80dcf732dd2a15/crates/did-mini/tests/restore.rs); [`crates/mini-ffi/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/49e82b52969a454f7762c5545c80dcf732dd2a15/crates/mini-ffi/src/lib.rs); [`crates/mini-ffi/src/mini_ffi.udl`](https://github.com/mininet-labs/Mininet/blob/49e82b52969a454f7762c5545c80dcf732dd2a15/crates/mini-ffi/src/mini_ffi.udl). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 2 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0211"></a>

## PR #211: mini-social: signed LAN/QR pairing offer/acceptance protocol (D-0340)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-06, FD-08, FD-09, FD-15.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/211) | [Files changed](https://github.com/mininet-labs/Mininet/pull/211/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/8b54323f8712d61540e49704a083bb2b38fb770d)

Head `8b54323f8712d61540e49704a083bb2b38fb770d`; base `be7392fc6fc4fc8affcae096ca66f29580d97b85`; merge `08343aeab62a004d0f4a34f320fbeed63e720ab6`. 9 changed files; 3 commits; 0 issue comments, 12 inline comments and 2 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Creates signed, bounded LAN/QR social-pairing messages that can be verified without an account directory. It separates a human-visible invitation from unauthenticated local discovery.

**Mechanism and evidence.** Offers carry root/device KELs, endpoint, nonce and expiry; verification checks the delegated POST-capable device and signature. Acceptance binds the offer nonce. PairingNonceLedger is bounded, and the TCP adapter rejects oversized frames and uses finite timeouts.

**What remains weaker than the intended claim.** The embedded KEL proves self-certifying identity, not that it is the person the viewer intended or the newest state of a previously known root. The initial caller supplies endpoint/ceremony inputs; a signature does not make a private-address connection safe by itself. This PR has no camera/UI confirmation or durable replay integration.

**Recommended improvement and rationale.** Make random invitation nonces internal to issuance, show a stable verification fingerprint and require explicit acceptance. Constrain advertised endpoints to the intended local interface class and retain replay evidence for the whole acceptance lifetime. Do not classify mutual follow as personhood or enrollment.

**Concrete example.** A copied screenshot of an invitation must fail after it is consumed or expired. A validly signed QR pointing to a cloud or loopback service should not make the phone dial it under a LAN-only pairing promise.

**Acceptance tests to implement.** Test malicious endpoints, clock rollback, duplicate acceptance, expired invitation, invalid device capability, replay after process death, slow frames and a forged lookalike display name. Physical camera and two-device tests remain separate.

**History, supersession and integration.** #258 later supplies the Android QR/LAN consumer, internally generated nonces and persisted pairing state. #213/#259/#260 concern BLE byte transport, not proof of identity or friendship.

**Source entry points.** [`crates/mini-social/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/8b54323f8712d61540e49704a083bb2b38fb770d/crates/mini-social/src/lib.rs); [`crates/mini-social/src/pairing.rs`](https://github.com/mininet-labs/Mininet/blob/8b54323f8712d61540e49704a083bb2b38fb770d/crates/mini-social/src/pairing.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0212"></a>

## PR #212: deny.toml: install and run cargo-deny for real, verify it clean (D-0341)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-06, FD-10, FD-14.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/212) | [Files changed](https://github.com/mininet-labs/Mininet/pull/212/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/831c69f6c783309da0ec0391ae628b6bc70b8fb3)

Head `831c69f6c783309da0ec0391ae628b6bc70b8fb3`; base `08343aeab62a004d0f4a34f320fbeed63e720ab6`; merge `3b151e9352422f34df14f7fc97f835fbf9c9e2d2`. 6 changed files; 1 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Turns dependency policy from unexecuted configuration into an actually exercised scanner and records why each license/advisory allowance exists. This is material supply-chain progress rather than cosmetic CI.

**Mechanism and evidence.** cargo-deny is run against the resolved lockfile; specific GUI/font licenses and advisories are triaged. The wildcard rule is relaxed because local path dependencies were being flagged alongside unrestricted registry versions. A separate cargo-audit installation failure is documented rather than concealed.

**What remains weaker than the intended claim.** Broad wildcard warning weakens protection for real registry wildcards as well as benign path dependencies. Ignored advisories can outlive the reason for an exception, and calling an issue non-exploitable depends on the actual dependency path and inputs. A scanner succeeding does not establish that the approved licenses make third-party code public domain.

**Recommended improvement and rationale.** Use a path-aware dependency rule that still rejects unrestricted external versions. Give each advisory exception an owner, exact affected path, rationale, expiry and revalidation trigger; fail stale ignores. Preserve third-party license notices and machine-readable dependency provenance.

**Concrete example.** A build-only XML parser may have no hostile input today; moving it into a runtime importer changes that conclusion even with an unchanged version. The exception should be invalidated by that dependency/input change.

**Acceptance tests to implement.** Test a vulnerable dependency, unavailable scanner, invalid report, registry wildcard, local path dependency and expired ignore. Verify policy failure is a real required check and record scanner/advisory-database versions.

**History, supersession and integration.** #299 repairs the scanner-not-running false green; #307 repairs bash error handling and removes obsolete exceptions. #277/#307 show why pinned sandbox dependencies require continued advisory monitoring.

**Source entry points.** [`.github/workflows/ci.yml`](https://github.com/mininet-labs/Mininet/blob/831c69f6c783309da0ec0391ae628b6bc70b8fb3/.github/workflows/ci.yml); [`deny.toml`](https://github.com/mininet-labs/Mininet/blob/831c69f6c783309da0ec0391ae628b6bc70b8fb3/deny.toml); [`docs/DECISION_LOG.md`](https://github.com/mininet-labs/Mininet/blob/831c69f6c783309da0ec0391ae628b6bc70b8fb3/docs/DECISION_LOG.md). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0213"></a>

## PR #213: mini-bearer: MTU-bounded chunking/reassembly for BLE (D-0342)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-06, FD-11, FD-14.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/213) | [Files changed](https://github.com/mininet-labs/Mininet/pull/213/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/01a367feddf319907ce7ac363d0c353b8eca982d)

Head `01a367feddf319907ce7ac363d0c353b8eca982d`; base `3b151e9352422f34df14f7fc97f835fbf9c9e2d2`; merge `64b37d2d7d7dcb18064ad60de263b2eff7926f5e`. 8 changed files; 1 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Adds the missing framing layer for moving ordinary encrypted Mininet frames across small BLE payloads. It reuses the existing bearer abstraction instead of attaching identity authority to a Bluetooth address.

**Mechanism and evidence.** chunk_frame splits a bounded frame with u16 index/count headers; ChunkReassembler enforces sequential indices, stable count and total frame-size bounds. Invalid MTUs, too many chunks and malformed headers receive explicit errors.

**What remains weaker than the intended claim.** This is protocol logic only, not a GATT driver. Returning all chunks at once and permitting a 16-MiB frame can consume substantial memory on a weak phone; small MTUs also impose the u16-count ceiling. Frame sequencing and reconnection behavior are not provided by merely resetting a reassembler.

**Recommended improvement and rationale.** Use a streaming chunk iterator and bounded assembly storage, define per-connection frame identifiers and cancellation/reset semantics, and apply deadlines to partial frames. Derive usable payload length from actual negotiated ATT limits, not an assumed MTU value.

**Concrete example.** At a 20-byte usable payload with a four-byte header, a peer cannot transfer the maximum bearer frame within 65,535 chunks. The sender should reject or negotiate a smaller frame before allocating the full chunk list.

**Acceptance tests to implement.** Test real negotiated payload sizes, duplicates, gaps, mixed frames, changing counts, partial disconnect/reconnect, empty frames and maximum memory. Exercise two physical phones before claiming BLE connectivity.

**History, supersession and integration.** #259 adds a Rust BleRadio-backed Bearer and #260 the FFI callback. Hardware GATT integration and acceptance remain independent of these pure framing tests.

**Source entry points.** [`crates/mini-bearer/src/ble.rs`](https://github.com/mininet-labs/Mininet/blob/01a367feddf319907ce7ac363d0c353b8eca982d/crates/mini-bearer/src/ble.rs); [`crates/mini-bearer/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/01a367feddf319907ce7ac363d0c353b8eca982d/crates/mini-bearer/src/error.rs); [`crates/mini-bearer/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/01a367feddf319907ce7ac363d0c353b8eca982d/crates/mini-bearer/src/lib.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0214"></a>

## PR #214: Add Android CI: real APK assembly on a GitHub-hosted runner (D-0343)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-06, FD-10, FD-11, FD-14.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/214) | [Files changed](https://github.com/mininet-labs/Mininet/pull/214/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/83aaf4d3d9184710d1490f46215eced41d8ea663)

Head `83aaf4d3d9184710d1490f46215eced41d8ea663`; base `64b37d2d7d7dcb18064ad60de263b2eff7926f5e`; merge `c9c4e869fc22d218c89f94a79b01f7fc72f9662a`. 9 changed files; 6 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Establishes an actual Android build on a capable runner after earlier Rust-only work could not compile the shell. This closes a basic evidence gap between source code and an installable package.

**Mechanism and evidence.** A dedicated workflow installs pinned JDK/SDK/NDK/Gradle/cargo-ndk components, builds two Rust Android ABIs and assembles a debug APK. It checks selected installed versions, uploads outputs and failure logs, and uses full-SHA action pins.

**What remains weaker than the intended claim.** Building an APK is not a device security test, release signing approval or reproducibility proof. Version-directory existence is weaker than artifact-digest verification. SDK/runner distribution and Maven resolution remain supply-chain assumptions, and source-controlled pins must be revalidated rather than assumed available forever.

**Recommended improvement and rationale.** Add checksum/dependency verification and a recorded build-input manifest, then test the produced APK on emulator and real hardware. Keep debug signing distinct from owner-approved release signing and provide a non-GitHub recipe usable by independent builders.

**Concrete example.** A package can compile while persistence fails after force-stop or a camera callback returns unusable image resolution. Those are acceptance failures even when the Android CI badge is green.

**Acceptance tests to implement.** Intentionally mismatch a toolchain pin, remove an output artifact, break generated binding signatures and introduce a dependency checksum mismatch; all must fail loudly. Run cold-start, upgrade and offline-sideload tests on the exact APK digest.

**History, supersession and integration.** Builds #179 and exposes defects such as #250's Kotlin constructor mismatch, repaired by #251. #216 adds repeatability comparison, not independent security approval.

**Source entry points.** [`.github/workflows/android-ci.yml`](https://github.com/mininet-labs/Mininet/blob/83aaf4d3d9184710d1490f46215eced41d8ea663/.github/workflows/android-ci.yml); [`app/android/app/build.gradle.kts`](https://github.com/mininet-labs/Mininet/blob/83aaf4d3d9184710d1490f46215eced41d8ea663/app/android/app/build.gradle.kts); [`app/android/app/src/main/java/org/mininet/app/MainActivity.kt`](https://github.com/mininet-labs/Mininet/blob/83aaf4d3d9184710d1490f46215eced41d8ea663/app/android/app/src/main/java/org/mininet/app/MainActivity.kt); [`app/android/scripts/build-rust.sh`](https://github.com/mininet-labs/Mininet/blob/83aaf4d3d9184710d1490f46215eced41d8ea663/app/android/scripts/build-rust.sh); [`docs/DECISION_LOG.md`](https://github.com/mininet-labs/Mininet/blob/83aaf4d3d9184710d1490f46215eced41d8ea663/docs/DECISION_LOG.md); [`docs/mobile/ANDROID_FOUNDATION.md`](https://github.com/mininet-labs/Mininet/blob/83aaf4d3d9184710d1490f46215eced41d8ea663/docs/mobile/ANDROID_FOUNDATION.md). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 1 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0215"></a>

## PR #215: mini-ffi: typed OperationLifecycle state machine for backgroundable exchanges (D-0348)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-01, FD-06, FD-11, FD-14.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/215) | [Files changed](https://github.com/mininet-labs/Mininet/pull/215/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/8abfae44305afd2a578a530f6bb08042c0d4beba)

Head `8abfae44305afd2a578a530f6bb08042c0d4beba`; base `c9c4e869fc22d218c89f94a79b01f7fc72f9662a`; merge `b065c4ac2d54bbc0ab772deee6a696b8a8d74dc2`. 8 changed files; 2 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Makes background interruption a typed state-machine concern rather than an invisible UI event. It gives the platform shell a vocabulary for whether an in-progress operation can safely suspend.

**Mechanism and evidence.** OperationLifecycle tracks Idle/InFlight/checkpoint/suspended/terminal states behind a Mutex. InFlight refuses graceful suspension until a caller reports a checkpoint; terminal states reject further transitions and failure has a typed visible reason.

**What remains weaker than the intended claim.** A caller-reported checkpoint is not proof that any bytes were durably committed. The enum neither prevents Android from killing the process nor enforces rollback of external I/O. Without real pairing/BLE/persistence callers, it can report the right abstract state while the actual operation violates it.

**Recommended improvement and rationale.** Emit checkpoint tokens only from successful durable transactions and bind them to the specific operation/session. Wire the state machine into real platform lifecycle callbacks with bounded cancellation and resume; distinguish safe retry from restart requiring a fresh cryptographic session.

**Concrete example.** The app backgrounds after sending an acceptance but before persisting the consumed nonce. Reporting Checkpointed at that moment is false even if the transition is legal in the enum; resume must not permit replay.

**Acceptance tests to implement.** Kill the actual process before/after every durable boundary; test duplicate resume, checkpoint for another operation, blocked I/O, timeout and terminal cleanup. Verify no success UI precedes the durable state it claims.

**History, supersession and integration.** Composes the #211/#213 communication work and #209 persistence contract. Later #258 must bind these concepts in the product; this PR alone closes no physical background-operation gate.

**Source entry points.** [`crates/mini-ffi/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/8abfae44305afd2a578a530f6bb08042c0d4beba/crates/mini-ffi/src/lib.rs); [`crates/mini-ffi/src/lifecycle.rs`](https://github.com/mininet-labs/Mininet/blob/8abfae44305afd2a578a530f6bb08042c0d4beba/crates/mini-ffi/src/lifecycle.rs); [`crates/mini-ffi/src/mini_ffi.udl`](https://github.com/mininet-labs/Mininet/blob/8abfae44305afd2a578a530f6bb08042c0d4beba/crates/mini-ffi/src/mini_ffi.udl). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0216"></a>

## PR #216: Add android-reproducibility.yml: two-independent-build APK hash check (D-0349)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-06, FD-10, FD-13, FD-14.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/216) | [Files changed](https://github.com/mininet-labs/Mininet/pull/216/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/d4fae46c860773ad95dec95dc3f15fdd6b1f2c53)

Head `d4fae46c860773ad95dec95dc3f15fdd6b1f2c53`; base `b065c4ac2d54bbc0ab772deee6a696b8a8d74dc2`; merge `903300f8c568137c4045fc81d88fd33b9962e485`. 8 changed files; 2 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Extends reproducibility checking to a real Android package built on two separate runner instances. It is stronger than rebuilding twice in one working directory and helps detect hidden build nondeterminism.

**Mechanism and evidence.** Two matrix jobs repeat the pinned Android recipe and hash the complete APK. A compare job downloads both digests and fails on inequality, printing a limited entry-list diagnostic. Environment/command digests are recorded, but no signed BuildProvenance claim is issued.

**What remains weaker than the intended claim.** The two runners still share one administrative provider and recipe. Equal hashes prove repeatability under those conditions, not honest compiler execution or independently controlled build agreement. Entry names/sizes are not a complete byte-level explanation of an APK mismatch. Only one docs-only marker is used by the comparison.

**Recommended improvement and rationale.** Require independently controlled builders for release legitimacy, preserve signed exact-input/output statements and compare both skip markers. Use a structural binary diff tool to diagnose mismatches without weakening the full hash criterion. Reproduce the exact release artifact, not only debug APKs.

**Concrete example.** A compromised shared build action can produce the same malicious APK twice. An independent owner-controlled builder with a separately obtained recipe and toolchain must reproduce the intended bytes before release confidence increases.

**Acceptance tests to implement.** Inject different resource ordering, timestamps, compiler flags and one altered artifact; require mismatch. Test marker disagreement, missing/empty hashes, independent provider removal and offline rebuild from the archived inputs.

**History, supersession and integration.** Builds on #214 and the #185 skip pattern. The current workflow does contain a real comparison step; a truncated source preview must not be misreported as an absent comparison.

**Source entry points.** [`.github/workflows/android-reproducibility.yml`](https://github.com/mininet-labs/Mininet/blob/d4fae46c860773ad95dec95dc3f15fdd6b1f2c53/.github/workflows/android-reproducibility.yml); [`app/android/app/build.gradle.kts`](https://github.com/mininet-labs/Mininet/blob/d4fae46c860773ad95dec95dc3f15fdd6b1f2c53/app/android/app/build.gradle.kts); [`app/android/app/debug.keystore`](https://github.com/mininet-labs/Mininet/blob/d4fae46c860773ad95dec95dc3f15fdd6b1f2c53/app/android/app/debug.keystore); [`docs/DECISION_LOG.md`](https://github.com/mininet-labs/Mininet/blob/d4fae46c860773ad95dec95dc3f15fdd6b1f2c53/docs/DECISION_LOG.md); [`docs/mobile/ANDROID_FOUNDATION.md`](https://github.com/mininet-labs/Mininet/blob/d4fae46c860773ad95dec95dc3f15fdd6b1f2c53/docs/mobile/ANDROID_FOUNDATION.md). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 1 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0217"></a>

## PR #217: Wire Compose UI to real RootCore root/device creation (D-0351)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-01, FD-06, FD-08, FD-11.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/217) | [Files changed](https://github.com/mininet-labs/Mininet/pull/217/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/8b62cf20cb347cef8bc0678a710e98b97d4fc204)

Head `8b62cf20cb347cef8bc0678a710e98b97d4fc204`; base `903300f8c568137c4045fc81d88fd33b9962e485`; merge `d10fb6d1905c74d2a3dd02b753a75100b9f7b08e`. 5 changed files; 1 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Wires the onboarding button to real Rust root/device creation instead of leaving a functioning core behind a placeholder screen. It deliberately tells users the then-current identity is memory-only.

**Mechanism and evidence.** The Compose view model calls generated RootCore.createRoot/createDevice methods and renders the returned DIDs in a RootCreated state. No new key-generation algorithm or identity rule is added.

**What remains weaker than the intended claim.** Correct method names and Rust tests do not establish Kotlin compilation or a safe end-user ceremony. Two sequential creation calls can fail halfway; the UI must distinguish root-created/device-failed from total failure. At this stage closing the app loses identity, so public testing must not create a false expectation of continuity.

**Recommended improvement and rationale.** Make root/device creation a recoverable product transaction, render partial outcomes accurately, and require persistence confirmation before showing an identity as safely reusable. Avoid automatically recreating a root on any restoration problem.

**Concrete example.** A device-creation failure after successful root creation should allow retry with the existing root. Pressing Create again must not silently replace it or leave the user believing no identity was created.

**Acceptance tests to implement.** Compile the exact generated bindings and shell; test double taps, partial failure, rotation, process death and accessibility of the loss warning. Verify no secret key appears in UI state or logs.

**History, supersession and integration.** Connects #179/#206. #250 later adds actual platform-backed persistence and #251 repairs its compilation; this PR should not be credited with durable Android identity.

**Source entry points.** [`app/android/app/src/main/java/org/mininet/app/MainActivity.kt`](https://github.com/mininet-labs/Mininet/blob/8b62cf20cb347cef8bc0678a710e98b97d4fc204/app/android/app/src/main/java/org/mininet/app/MainActivity.kt); [`docs/DECISION_LOG.md`](https://github.com/mininet-labs/Mininet/blob/8b62cf20cb347cef8bc0678a710e98b97d4fc204/docs/DECISION_LOG.md); [`docs/mobile/ANDROID_FOUNDATION.md`](https://github.com/mininet-labs/Mininet/blob/8b62cf20cb347cef8bc0678a710e98b97d4fc204/docs/mobile/ANDROID_FOUNDATION.md). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0218"></a>

## PR #218: Canonize Founder Directive 18: the Edge Provider Doctrine (D-0352)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-03, FD-07, FD-08, FD-16, FD-18.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/218) | [Files changed](https://github.com/mininet-labs/Mininet/pull/218/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/cd70658a872b141d9adfe7f86224fa8a6dd19fb2)

Head `cd70658a872b141d9adfe7f86224fa8a6dd19fb2`; base `d10fb6d1905c74d2a3dd02b753a75100b9f7b08e`; merge `522f452a93a34007b00470bea7f3e23b502a5364`. 14 changed files; 1 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Canonizes the Edge Provider Doctrine: external services may improve convenience but may not become indispensable owners of the core. It makes provider disappearance, substitution and human-controlled exit explicit constitutional concerns.

**Mechanism and evidence.** Directive 18 and nine pending invariants are added to the canonical directive/register documents, with threat-model and contribution checklist updates. The registry expands from seventeen to eighteen digested directives. No Rust enforcement accompanies this doctrine-only PR.

**What remains weaker than the intended claim.** The commitment is strongly aligned but pending invariants are not implemented guarantees. A provider declaration cannot force a bank to return assets or a carrier to preserve connectivity. The clause about losing no more than convenience needs a precise boundary for external custody and outstanding obligations rather than an absolute promise about the outside world.

**Recommended improvement and rationale.** Define core operations and prove they run with all provider adapters disabled. Require explicit custody/freeze/death/exit disclosures and local revocation of each adapter. Separate a human-owned provider choice from a network-wide endorsement or privileged official registry.

**Concrete example.** If every bank adapter disappears, Mininet identity, public publishing, local storage and native settlement verification must remain usable; a claim on an external bank account must be labeled dependent on that bank, not guaranteed native value.

**Acceptance tests to implement.** Implement dependency-graph and runtime provider-removal tests, fake-provider substitution, local disable/offline startup and no-provider-governance-weight checks. Preserve the human approval provenance of the doctrine and exact register digests.

**History, supersession and integration.** #219 begins the vocabulary/state-machine implementation; #220 examines research dependencies. Doctrine ratification does not close INV-18 enforcement rows or legal/custody audits.

**Source entry points.** [`CONTRIBUTING.md`](https://github.com/mininet-labs/Mininet/blob/cd70658a872b141d9adfe7f86224fa8a6dd19fb2/CONTRIBUTING.md); [`README.md`](https://github.com/mininet-labs/Mininet/blob/cd70658a872b141d9adfe7f86224fa8a6dd19fb2/README.md); [`WHITEPAPER.md`](https://github.com/mininet-labs/Mininet/blob/cd70658a872b141d9adfe7f86224fa8a6dd19fb2/WHITEPAPER.md); [`docs/CONSTITUTION_REGISTRY.json`](https://github.com/mininet-labs/Mininet/blob/cd70658a872b141d9adfe7f86224fa8a6dd19fb2/docs/CONSTITUTION_REGISTRY.json); [`docs/DECISION_LOG.md`](https://github.com/mininet-labs/Mininet/blob/cd70658a872b141d9adfe7f86224fa8a6dd19fb2/docs/DECISION_LOG.md); [`docs/FAILURE_BOOK.md`](https://github.com/mininet-labs/Mininet/blob/cd70658a872b141d9adfe7f86224fa8a6dd19fb2/docs/FAILURE_BOOK.md); [`docs/FOUNDER_DIRECTIVES.md`](https://github.com/mininet-labs/Mininet/blob/cd70658a872b141d9adfe7f86224fa8a6dd19fb2/docs/FOUNDER_DIRECTIVES.md); [`docs/INVARIANTS.md`](https://github.com/mininet-labs/Mininet/blob/cd70658a872b141d9adfe7f86224fa8a6dd19fb2/docs/INVARIANTS.md). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0219"></a>

## PR #219: FD-18 Waves 1-2: mini-provider + mini-engagement (D-0400, D-0402)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-04, FD-05, FD-09, FD-16, FD-18.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/219) | [Files changed](https://github.com/mininet-labs/Mininet/pull/219/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/dfeea1d6ce67c44ebcc85b284f241a946134e508)

Head `dfeea1d6ce67c44ebcc85b284f241a946134e508`; base `522f452a93a34007b00470bea7f3e23b502a5364`; merge `a0826b4de4d6b01005a332c304f2ee8d0c6bb18e`. 19 changed files; 3 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Begins the provider layer as replaceable leaf crates rather than a canonical service registry. It also introduces a generic engagement lifecycle, avoiding a separate privileged protocol for each industry.

**Mechanism and evidence.** ProviderDeclaration requires custody, freeze, death and exit fields; LocalProviderPolicy is device-local and not a network off switch. Engagement wraps a PaymentClaim with offered/accepted/milestone/completed/disputed/timed-out bookkeeping. No core crate is intended to depend on provider authority.

**What remains weaker than the intended claim.** A declaration can be signed yet dishonest, and a local timeout cannot return funds that were never canonically escrowed. The early state machine tracks intent, not an enforced refund contract. Structural leaf placement and no governance-weight field are necessary but not sufficient to demonstrate provider independence in an application.

**Recommended improvement and rationale.** Name local engagement state separately from canonical escrow state, require evidence for each irreversible payment transition, and define refund/dispute rights as explicit canonical transactions. Keep provider discovery plural and local; no ranking or engagement certificate may elevate humanness.

**Concrete example.** A courier disappears after an engagement is marked Accepted. Calling timeout may change the local display, but only a valid canonical refund or an unspent requester balance can establish returned value.

**Acceptance tests to implement.** Test disappeared/dishonest providers, disputed milestones, duplicate completion, payment rejection, timeout races and local disable. Add compile-time dependency checks plus a product run with every external provider adapter removed.

**History, supersession and integration.** D-0400/D-0402 were missing from the original decision log and are recorded retroactively by #221. #237 adds canonical completion reads; #275 ties requester-funded creator/seeder payments to the ledger.

**Source entry points.** [`crates/mini-engagement/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/dfeea1d6ce67c44ebcc85b284f241a946134e508/crates/mini-engagement/src/error.rs); [`crates/mini-engagement/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/dfeea1d6ce67c44ebcc85b284f241a946134e508/crates/mini-engagement/src/lib.rs); [`crates/mini-engagement/src/state.rs`](https://github.com/mininet-labs/Mininet/blob/dfeea1d6ce67c44ebcc85b284f241a946134e508/crates/mini-engagement/src/state.rs); [`crates/mini-engagement/src/transitions.rs`](https://github.com/mininet-labs/Mininet/blob/dfeea1d6ce67c44ebcc85b284f241a946134e508/crates/mini-engagement/src/transitions.rs); [`crates/mini-provider/src/declaration.rs`](https://github.com/mininet-labs/Mininet/blob/dfeea1d6ce67c44ebcc85b284f241a946134e508/crates/mini-provider/src/declaration.rs); [`crates/mini-provider/src/discovery.rs`](https://github.com/mininet-labs/Mininet/blob/dfeea1d6ce67c44ebcc85b284f241a946134e508/crates/mini-provider/src/discovery.rs); [`crates/mini-provider/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/dfeea1d6ce67c44ebcc85b284f241a946134e508/crates/mini-provider/src/error.rs); [`crates/mini-provider/src/grant.rs`](https://github.com/mininet-labs/Mininet/blob/dfeea1d6ce67c44ebcc85b284f241a946134e508/crates/mini-provider/src/grant.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0220"></a>

## PR #220: Research proposals: personhood, privacy, PQ recovery, voting, and consensus

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-05, FD-08, FD-09, FD-12, FD-13, FD-15, FD-16, FD-17, FD-18.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/220) | [Files changed](https://github.com/mininet-labs/Mininet/pull/220/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/3a31f0f076108f28e464bcd60e92b4e6077b0f39)

Head `3a31f0f076108f28e464bcd60e92b4e6077b0f39`; base `3db6b174e45b342c8c93d52c7cd69bfd459fa38d`; merge `61ff2af031810050899049559620f8736e22f7aa`. 4 changed files; 6 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Consolidates six difficult trust research areas into explicit, falsifiable workstreams instead of allowing isolated prototypes to become production promises. This gives the founder vision a research discipline as well as an implementation roadmap.

**Mechanism and evidence.** The proposal separates evidence, membership credentials and authority; defines maturity gates for personhood/privacy, engagement attestations, PQ recovery, coercion-resistant voting and long-delay consensus; and decomposes them into fifteen issues with threat models and acceptance criteria.

**What remains weaker than the intended claim.** This is research, not a solved construction or ratified activation. Its impossibility statements must be read under their stated models: lack of an unbroken pre-break anchor prevents cryptographic attribution in that model, not every conceivable recovery policy. Zero knowledge cannot authenticate dishonest sensor inputs, and an issuer threshold does not prove operational independence.

**Recommended improvement and rationale.** Pre-register attack models, false acceptance/rejection thresholds and privacy/resource budgets before selecting constructions. Build adversarial simulators and independent implementations; separate a research result from the governed choice to rely on it. Preserve the option to revise a hypothesis without weakening core equality/privacy requirements.

**Concrete example.** A hundred purchased devices can submit distinct signed liveness traces. A proof that each trace satisfies a predicate still does not establish a hundred independent humans; the pilot must measure this attack rather than count valid proofs.

**Acceptance tests to implement.** Run Sybil farms, issuer collusion, compromised sensors, coerced revoting, lost/PQ-broken keys and planetary partitions. Record failed hypotheses and measured confidence intervals; no maturity upgrade should be granted by an issue closing alone.

**History, supersession and integration.** Drives #237/#239 and future research issues, while exposing #219's missing decision entries fixed in #221. This review preserves its proposal status and does not claim independent validation of its constructions.

**Source entry points.** [`docs/design/frontier-personhood-governance-and-consensus-proposals.md`](https://github.com/mininet-labs/Mininet/blob/3a31f0f076108f28e464bcd60e92b4e6077b0f39/docs/design/frontier-personhood-governance-and-consensus-proposals.md). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0221"></a>

## PR #221: Record D-0400/D-0402 decision-log entries retroactively

**PASS** | Captured outcome: **merged** | Directives: FD-03, FD-05, FD-10, FD-12, FD-14.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/221) | [Files changed](https://github.com/mininet-labs/Mininet/pull/221/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/7f3dbb804bc53d4b32f82a35f21c994e6bbbf4d7)

Head `7f3dbb804bc53d4b32f82a35f21c994e6bbbf4d7`; base `a0826b4de4d6b01005a332c304f2ee8d0c6bb18e`; merge `558fde981aec5c7cd0ef94dba0add73cfa43f67e`. 3 changed files; 1 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Repairs a real traceability gap: code and STATUS referenced D-0400/D-0402 without corresponding decision-log entries. Auditors need the reason and authority record, not just a number in a Cargo description.

**Mechanism and evidence.** Two append-only seven-field entries document already-shipped provider and engagement work from #219. No source behavior is altered, and the entries identify their retrospective nature.

**What remains weaker than the intended claim.** The scoped PASS is for honest historical repair. Retroactive documentation is not evidence that an independent review occurred before the original merge, and a number collision check cannot establish that a decision was authorized. The omission should motivate prevention, not merely an increasingly long log.

**Recommended improvement and rationale.** Validate every newly cited D-number against an existing or same-proposal entry and maintain distinct fields for proposal, acceptance, merge and external approval. Preserve original omissions/corrections in history rather than making the record look contemporaneous.

**Concrete example.** A PR adds D-0500 in STATUS but no entry. CI should reject the dangling reference; a later corrective entry should say when it was actually recorded rather than backdating its approval.

**Acceptance tests to implement.** Test missing references, duplicate numbers, unrelated reused IDs and append-only deletion. Human review checks that the rationale matches the original diff and does not introduce new authority under the guise of documenting old work.

**History, supersession and integration.** Follows the #220 cross-check of #219 and anticipates #301 registry integrity checks. Later status corrections such as #288 should use the same explicit retrospective discipline.

**Source entry points.** [`docs/DECISION_LOG.md`](https://github.com/mininet-labs/Mininet/blob/7f3dbb804bc53d4b32f82a35f21c994e6bbbf4d7/docs/DECISION_LOG.md). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0237"></a>

## PR #237: Frontier Trust: mini-engagement canonical completion + mini-pq-anchor pre-provisioning (closes #226, #231)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-04, FD-05, FD-09, FD-13, FD-18.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/237) | [Files changed](https://github.com/mininet-labs/Mininet/pull/237/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/10b8e510807f6e1b2fc2e01ce2b4bcf4f4a66921)

Head `10b8e510807f6e1b2fc2e01ce2b4bcf4f4a66921`; base `558fde981aec5c7cd0ef94dba0add73cfa43f67e`; merge `ea92c24abb695fb1b39d1ff1aae3c3c01634d212`. 17 changed files; 3 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Closes two concrete gaps identified by the frontier research: engagements can ask the canonical ledger whether their payment finalized, and users can generate a dormant post-quantum anchor before a classical break.

**Mechanism and evidence.** canonical_completion_status combines the existing settlement reconciliation result with local EngagementState and distinguishes FinalizedWithoutLocalCompletion. mini-pq-anchor provisions ML-DSA-65 keys into a per-owner inventory whose status is only Provisioned, not committed or active.

**What remains weaker than the intended claim.** A finalized payment plus a locally marked Completed engagement is not proof of physical work or true escrow semantics. The supplied ledger view is the trust boundary. A volatile, uncommitted PQ key cannot establish pre-break continuity or survive a device failure; the crate correctly does not claim otherwise.

**Recommended improvement and rationale.** Bind service receipts and completion policy to the exact canonical engagement/payment, keeping subjective delivery disputes separate. For PQ anchors add encrypted durable storage, a pre-break signed commitment and independently recoverable continuity evidence before exposing a recoverable status.

**Concrete example.** A dormant PQ key generated after an attacker can forge the old signature cannot distinguish the owner from that attacker. The useful operation is to generate and commit the anchor before the break and retain verifiable history.

**Acceptance tests to implement.** Test finalized-without-local-completion, canonical rejection/conflict, fabricated local completion, stale ledger views, anchor persistence failure, wrong-owner association and exact commitment recovery. Independent review must examine the complete PQ transition, not just key generation.

**History, supersession and integration.** Implements pieces of #220 using #176 and #219. #239 consumes canonical completion for Tier-0 receipts; no personhood, full escrow or emergency migration is activated here.

**Source entry points.** [`crates/mini-engagement/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/10b8e510807f6e1b2fc2e01ce2b4bcf4f4a66921/crates/mini-engagement/src/error.rs); [`crates/mini-engagement/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/10b8e510807f6e1b2fc2e01ce2b4bcf4f4a66921/crates/mini-engagement/src/lib.rs); [`crates/mini-engagement/src/settlement.rs`](https://github.com/mininet-labs/Mininet/blob/10b8e510807f6e1b2fc2e01ce2b4bcf4f4a66921/crates/mini-engagement/src/settlement.rs); [`crates/mini-engagement/src/state.rs`](https://github.com/mininet-labs/Mininet/blob/10b8e510807f6e1b2fc2e01ce2b4bcf4f4a66921/crates/mini-engagement/src/state.rs); [`crates/mini-pq-anchor/src/anchor.rs`](https://github.com/mininet-labs/Mininet/blob/10b8e510807f6e1b2fc2e01ce2b4bcf4f4a66921/crates/mini-pq-anchor/src/anchor.rs); [`crates/mini-pq-anchor/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/10b8e510807f6e1b2fc2e01ce2b4bcf4f4a66921/crates/mini-pq-anchor/src/error.rs); [`crates/mini-pq-anchor/src/inventory.rs`](https://github.com/mininet-labs/Mininet/blob/10b8e510807f6e1b2fc2e01ce2b4bcf4f4a66921/crates/mini-pq-anchor/src/inventory.rs); [`crates/mini-pq-anchor/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/10b8e510807f6e1b2fc2e01ce2b4bcf4f4a66921/crates/mini-pq-anchor/src/lib.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0238"></a>

## PR #238: mini-airdrop + mini-airdrop-treasury: eligibility, signed claims, persisted registry, treasury approval bridge (D-0354/D-0355/D-0356)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-04, FD-05, FD-06, FD-08, FD-09, FD-16.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/238) | [Files changed](https://github.com/mininet-labs/Mininet/pull/238/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/738b86c4f967ba5a47d8764deae7d39aeec72aed)

Head `738b86c4f967ba5a47d8764deae7d39aeec72aed`; base `ea92c24abb695fb1b39d1ff1aae3c3c01634d212`; merge `588143bf9100ccb0a7cf5358fc7cc6fd42ca1679`. 19 changed files; 6 commits; 0 issue comments, 9 inline comments and 1 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Builds a bounded snapshot/claim/treasury-approval pipeline while deliberately stopping short of moving value with an incompatible FROST signature format. That separation prevents an approval object from masquerading as settlement.

**Mechanism and evidence.** mini-airdrop checks campaign match, KEL identity/signature, snapshot membership and a claimed registry. FileClaimedRegistry makes writes fallible and fsyncs appends. mini-airdrop-treasury verifies ordinary signer approvals over campaign and every outcome field, counts distinct authorized roots and returns approval evidence only.

**What remains weaker than the intended claim.** The claim is marked used before any payout finalizes, so a later failure can strand eligibility without a resumable state machine. Registry keys contain identity only, requiring strict campaign isolation. The current file parser stops at any undecodable record, not only a demonstrably truncated last record; it can silently forget a valid suffix. No cross-process lock is present.

**Recommended improvement and rationale.** Use a campaign/snapshot/claim-bound pending-authorized-paid state machine with idempotent canonical settlement and recovery. Distinguish incomplete tail from interior corruption and bound file size before reading; lock the read-check-append transaction. Make verified approval types unforgeable to downstream consumers or always reverify them.

**Concrete example.** An eligible claim is recorded, then the treasury process crashes before paying. On retry the system must resume the same approved payout rather than say AlreadyClaimed and permanently deny the user. A damaged middle record must not make later claims eligible again.

**Acceptance tests to implement.** Inject failures before/after append, approval and finality; run two registry processes concurrently; test campaign reuse, corrupted interior records, exact-tail truncation, signer rotation and copied approval structs. No test may count a distinct DID as a distinct human.

**History, supersession and integration.** #240 tests the cross-crate composition and #241 adds a balance check. FROST-to-settlement format compatibility and real-value external gates remain open.

**Source entry points.** [`crates/mini-airdrop-treasury/src/approval.rs`](https://github.com/mininet-labs/Mininet/blob/738b86c4f967ba5a47d8764deae7d39aeec72aed/crates/mini-airdrop-treasury/src/approval.rs); [`crates/mini-airdrop-treasury/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/738b86c4f967ba5a47d8764deae7d39aeec72aed/crates/mini-airdrop-treasury/src/error.rs); [`crates/mini-airdrop-treasury/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/738b86c4f967ba5a47d8764deae7d39aeec72aed/crates/mini-airdrop-treasury/src/lib.rs); [`crates/mini-airdrop/src/claim.rs`](https://github.com/mininet-labs/Mininet/blob/738b86c4f967ba5a47d8764deae7d39aeec72aed/crates/mini-airdrop/src/claim.rs); [`crates/mini-airdrop/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/738b86c4f967ba5a47d8764deae7d39aeec72aed/crates/mini-airdrop/src/error.rs); [`crates/mini-airdrop/src/file_registry.rs`](https://github.com/mininet-labs/Mininet/blob/738b86c4f967ba5a47d8764deae7d39aeec72aed/crates/mini-airdrop/src/file_registry.rs); [`crates/mini-airdrop/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/738b86c4f967ba5a47d8764deae7d39aeec72aed/crates/mini-airdrop/src/lib.rs); [`crates/mini-airdrop/src/registry.rs`](https://github.com/mininet-labs/Mininet/blob/738b86c4f967ba5a47d8764deae7d39aeec72aed/crates/mini-airdrop/src/registry.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0239"></a>

## PR #239: mini-attest: linkable Tier-0 engagement-proven reviews (D-0404)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-04, FD-05, FD-09, FD-15, FD-16, FD-18.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/239) | [Files changed](https://github.com/mininet-labs/Mininet/pull/239/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/6ac7c8b5bb2b7e19c0ecb369724e5ab557b47bb6)

Head `6ac7c8b5bb2b7e19c0ecb369724e5ab557b47bb6`; base `61ff2af031810050899049559620f8736e22f7aa`; merge `955eefb743e63fca5168bc465480c65a045f7266`. 16 changed files; 5 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Adds an explicitly linkable, opt-in review credential tied to completed paid engagements. It creates evidence for edge service reviews without giving provider interaction any path into humanness or governance.

**Mechanism and evidence.** EngagementCompletionReceiptV1 is provider-signed and content-addressed; verification reruns canonical engagement completion against the supplied ledger. A separately presented zeroized holder token authorizes a pairwise-pseudonym SignedReviewV1, with a local duplicate registry and bounded codecs.

**What remains weaker than the intended claim.** LINKABLE_TIER_0 is the correct label: provider, receipt ID, claim digest, timing and subject commitments remain observable. A genuine payment does not prove useful service, independence or an honest review. Local deduplication does not prevent cross-node duplicate publication, and the ledger view must be authenticated.

**Recommended improvement and rationale.** Keep Tier 0 visibly separate from future selective-disclosure/unlinkable tiers and never feed engagement credentials into personhood. Bind the full receipt to canonical settlement evidence and define privacy-preserving duplicate scope before federation. Review token logging and replay at the service boundary.

**Concrete example.** A provider and customer controlled by one operator can complete a genuine payment and write a glowing review. The receipt proves a qualifying transaction, not independent demand or truth; ranking must not promote it to unquestionable authority.

**Acceptance tests to implement.** Test stolen/replayed holder tokens, altered claim digests, forged provider KELs, stale finality, duplicate submission to independent nodes and cross-context linkage. Assert no human-root DID, token or amount enters public artifacts and no core authority depends on reviews.

**History, supersession and integration.** Builds on #237/#219 and #220's tier taxonomy. The anti-collusion analysis in #284/#285 is relevant to subsidizing such evidence; it does not magically make Tier 0 unlinkable.

**Source entry points.** [`crates/mini-attest/src/codec.rs`](https://github.com/mininet-labs/Mininet/blob/6ac7c8b5bb2b7e19c0ecb369724e5ab557b47bb6/crates/mini-attest/src/codec.rs); [`crates/mini-attest/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/6ac7c8b5bb2b7e19c0ecb369724e5ab557b47bb6/crates/mini-attest/src/error.rs); [`crates/mini-attest/src/holder.rs`](https://github.com/mininet-labs/Mininet/blob/6ac7c8b5bb2b7e19c0ecb369724e5ab557b47bb6/crates/mini-attest/src/holder.rs); [`crates/mini-attest/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/6ac7c8b5bb2b7e19c0ecb369724e5ab557b47bb6/crates/mini-attest/src/lib.rs); [`crates/mini-attest/src/receipt.rs`](https://github.com/mininet-labs/Mininet/blob/6ac7c8b5bb2b7e19c0ecb369724e5ab557b47bb6/crates/mini-attest/src/receipt.rs); [`crates/mini-attest/src/review.rs`](https://github.com/mininet-labs/Mininet/blob/6ac7c8b5bb2b7e19c0ecb369724e5ab557b47bb6/crates/mini-attest/src/review.rs); [`crates/mini-attest/tests/tier0.rs`](https://github.com/mininet-labs/Mininet/blob/6ac7c8b5bb2b7e19c0ecb369724e5ab557b47bb6/crates/mini-attest/tests/tier0.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 2 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0240"></a>

## PR #240: mini-airdrop-treasury: end-to-end integration proof (D-0358)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-04, FD-05, FD-06, FD-14.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/240) | [Files changed](https://github.com/mininet-labs/Mininet/pull/240/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/6189cbc0ff986833d093136a85dcd24247e4f46b)

Head `6189cbc0ff986833d093136a85dcd24247e4f46b`; base `588143bf9100ccb0a7cf5358fc7cc6fd42ca1679`; merge `c54f9276acbd2b15de73c29ce1314ef787d5c672`. 6 changed files; 2 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Adds the missing end-to-end check that snapshot eligibility, claimant signatures, durable claim memory and treasury approvals compose across crate boundaries. This is useful integration evidence, not merely another isolated type test.

**Mechanism and evidence.** The test uses a real SnapshotBuilder, signed ClaimRequest, reopened FileClaimedRegistry and a two-of-three TreasurySignerSet. A second case excludes a non-member before approval and verifies no registry mutation.

**What remains weaker than the intended claim.** The flow ends at TreasuryApprovedPayout and performs no canonical transfer. It therefore cannot establish conservation, payout recovery or atomic claim consumption. A happy-path fsync/reopen test also does not reveal the current interior-corruption and concurrent-reader races in the registry.

**Recommended improvement and rationale.** Extend the scenario through a real canonical payment and a durable resumable payout journal once the signing-suite decision is resolved. Keep approval and paid states separate and add fault injection at each cross-crate handoff.

**Concrete example.** Two valid approvals followed by an insufficient-balance rejection must not be displayed as a paid airdrop. The claim must remain recoverable without authorizing a second independent payout.

**Acceptance tests to implement.** Test crash after mark_claimed, signer withdrawal before finality, canonical rejection, duplicate retries, concurrent claim workers and corrupt replay files. Verify the final recipient balance and supply, not just approval-count equality.

**History, supersession and integration.** Stacked on #238; #241 adds preparatory treasury reconciliation. The test title end-to-end applies to the declared claim-to-approval scope only, not an executed airdrop.

**Source entry points.** [`crates/mini-airdrop-treasury/tests/end_to_end.rs`](https://github.com/mininet-labs/Mininet/blob/6189cbc0ff986833d093136a85dcd24247e4f46b/crates/mini-airdrop-treasury/tests/end_to_end.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0241"></a>

## PR #241: mini-airdrop-treasury: treasury balance reconciliation check (D-0359)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-04, FD-05, FD-10, FD-14.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/241) | [Files changed](https://github.com/mininet-labs/Mininet/pull/241/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/05f3d7d642e87aa69a6638e0df43518ef8ec1582)

Head `05f3d7d642e87aa69a6638e0df43518ef8ec1582`; base `c54f9276acbd2b15de73c29ce1314ef787d5c672`; merge `2fa400c78ef73d7f3684caef918f8908ab166a5d`. 8 changed files; 1 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Adds checked arithmetic to prevent a campaign operator from overlooking allocations larger than the stated treasury balance. It improves preflight bookkeeping without inventing new custody authority.

**Mechanism and evidence.** Total allocation is summed with checked u64 addition and compared to caller-supplied treasury_balance_micro. Overflow and a one-unit shortfall are explicit errors. No signing, claim consumption or payment is performed.

**What remains weaker than the intended claim.** The balance is not read from finalized state and no reservation is made. Two campaigns can each pass against the same funds, then overcommit them together. A positive preflight result is therefore a consistency check over supplied numbers, not proof of solvency or a per-claim safety gate.

**Recommended improvement and rationale.** Bind campaign budgets to an exact finalized treasury snapshot and enforce aggregate reservations or an explicit canonical budget contract. Preserve this pure arithmetic helper but name its output as a preflight estimate, not funded authorization.

**Concrete example.** Two campaigns each allocate 80 MINI from a treasury holding 100. Both individual comparisons pass; a canonical aggregate budget must reject the second reservation or reduce its committed allocation before claims open.

**Acceptance tests to implement.** Test maximum sums, overflow, zero, one-unit deficit, duplicate beneficiaries and multi-campaign contention against a canonical balance adapter. Changing the balance after preflight must invalidate authorization, not the arithmetic result itself.

**History, supersession and integration.** Follows #238/#240 and does not solve their claim-before-payment recovery gap or the FROST/settlement signature mismatch.

**Source entry points.** [`crates/mini-airdrop-treasury/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/05f3d7d642e87aa69a6638e0df43518ef8ec1582/crates/mini-airdrop-treasury/src/error.rs); [`crates/mini-airdrop-treasury/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/05f3d7d642e87aa69a6638e0df43518ef8ec1582/crates/mini-airdrop-treasury/src/lib.rs); [`crates/mini-airdrop-treasury/src/reconciliation.rs`](https://github.com/mininet-labs/Mininet/blob/05f3d7d642e87aa69a6638e0df43518ef8ec1582/crates/mini-airdrop-treasury/src/reconciliation.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0242"></a>

## PR #242: mini-intake-types: gate IntakeLink attachment behind Accepted review (D-0360, Track B PR B5)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-05, FD-09, FD-12, FD-14.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/242) | [Files changed](https://github.com/mininet-labs/Mininet/pull/242/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/d08c49b03dca4d8befc2ed7e71a89082d36790c0)

Head `d08c49b03dca4d8befc2ed7e71a89082d36790c0`; base `2fa400c78ef73d7f3684caef918f8908ab166a5d`; merge `e381a251e9ba631ef0c39763eaf10abd572de7e3`. 7 changed files; 1 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Makes publication links respect the intake review-state boundary instead of allowing newly parsed external material to present itself as accepted project evidence. This is a meaningful local API correction.

**Mechanism and evidence.** IntakeEnvelope::add_link becomes fallible and requires ReviewState::Accepted before attaching Issue/Object/Audit/Research/Profile/Post/Release links. Existing tests are updated to advance through review before linking.

**What remains weaker than the intended claim.** The enum state itself is not an authenticated human approval. Current from_bytes restores review/authority fields directly, so constructor/mutator checks do not protect a consumer that trusts imported serialized state. Gating all links may also make harmless private reference organization depend on review unless the product distinguishes authority-bearing links.

**Recommended improvement and rationale.** Separate untrusted imported metadata from authenticated local review records and validate cross-field consistency on decode. Bind accepted-review evidence to the exact source digest and reviewer authority. Keep private annotation possible without falsely promoting project authority.

**Concrete example.** An attacker supplies an envelope already labeled Accepted/ReviewedEvidence. Decoding that label must not make publish-post treat the source as reviewed; only a verified review record for those bytes can do so.

**Acceptance tests to implement.** Test impossible serialized states, copied acceptance from another source, unsigned review labels, rejected-to-accepted transitions and private non-authorizing annotations. Retain rejection tests at add_link and at every publishing consumer.

**History, supersession and integration.** Hardens #153/#159 intake. #286 adds bounded/idempotent links and a publish bridge, but source digest verification and review authorization are separate checks.

**Source entry points.** [`crates/mini-intake-types/src/envelope.rs`](https://github.com/mininet-labs/Mininet/blob/d08c49b03dca4d8befc2ed7e71a89082d36790c0/crates/mini-intake-types/src/envelope.rs); [`crates/mini-intake-types/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/d08c49b03dca4d8befc2ed7e71a89082d36790c0/crates/mini-intake-types/src/error.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0243"></a>

## PR #243: mini-commons-policy: public-commons entitlement policy (D-0361, Track C1)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-01, FD-08, FD-09, FD-16, FD-17.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/243) | [Files changed](https://github.com/mininet-labs/Mininet/pull/243/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/4f5038cf6e4377353cfb4ae98c94fdf47198e514)

Head `4f5038cf6e4377353cfb4ae98c94fdf47198e514`; base `e381a251e9ba631ef0c39763eaf10abd572de7e3`; merge `3db6b174e45b342c8c93d52c7cd69bfd459fa38d`. 11 changed files; 1 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Encodes public participation as a right independent of wallet balance, not a commercial purchase priced at zero. This is a direct implementation of the founder's free-public-commons distinction.

**Mechanism and evidence.** commons_policy_for accepts WalletStanding but does not consult it; the returned entitlement policy is identical for zero and maximal wealth. The crate is pure policy data with bounded wire encoding and no value/governance dependency.

**What remains weaker than the intended claim.** An unused balance parameter proves this function is wallet-independent, not that publishing, relays or storage schedulers honor the same rule. The crate cannot establish delivery under resource exhaustion or prevent a paid provider from withholding unpaid content.

**Recommended improvement and rationale.** Use the policy at actual public action admission and scheduling boundaries, with explicit resource-fairness limits that do not become wallet requirements. Separate free base participation from optional external service purchases and retain an independently usable unpaid path.

**Concrete example.** A zero-balance user must be able to create a public post without constructing a payment, not merely receive an Entitlement value while the real publisher asks for funds elsewhere.

**Acceptance tests to implement.** Exercise real view/post/comment/reply/react paths without any wallet initialized; then repeat under competing paid traffic and exhausted optional services. Assert identical protocol rights and explicit resource refusal rather than a hidden payment demand.

**History, supersession and integration.** #244 supplies real social-path tests; #245 prices additional services. Neither alone proves the full network scheduling/fairness property.

**Source entry points.** [`crates/mini-commons-policy/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/4f5038cf6e4377353cfb4ae98c94fdf47198e514/crates/mini-commons-policy/src/error.rs); [`crates/mini-commons-policy/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/4f5038cf6e4377353cfb4ae98c94fdf47198e514/crates/mini-commons-policy/src/lib.rs); [`crates/mini-commons-policy/src/policy.rs`](https://github.com/mininet-labs/Mininet/blob/4f5038cf6e4377353cfb4ae98c94fdf47198e514/crates/mini-commons-policy/src/policy.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0244"></a>

## PR #244: mini-commons-policy: contribution budgets + mini-social wallet-independence proof (D-0362, Track C2/C3)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-01, FD-06, FD-09, FD-11, FD-16, FD-17.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/244) | [Files changed](https://github.com/mininet-labs/Mininet/pull/244/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/47e0eab332abd3f0269593a59c6c63f10b01514d)

Head `47e0eab332abd3f0269593a59c6c63f10b01514d`; base `955eefb743e63fca5168bc465480c65a045f7266`; merge `cb16615ef72022ddd6cf2bc274d6a986e9f35515`. 11 changed files; 4 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Adds revocable opt-in contribution budgets and an actual wallet-free social integration test, advancing beyond #243's pure policy object. It protects the user's device from being treated as an involuntary infrastructure resource.

**Mechanism and evidence.** ContributionBudget describes storage, bandwidth, CPU, battery, network and background limits and defaults opted out. The social test publishes profiles/walls/comments and reactions using real identities/store APIs without constructing wallet state.

**What remains weaker than the intended claim.** The budget is data, not an enforcing scheduler, so default opt-out is not yet proof that every background worker obeys it. A same-store social test establishes no wallet dependency in those APIs but not equitable network delivery or survival under paid load.

**Recommended improvement and rationale.** Require a budget capability from the owner-controlled scheduler before every resource job, revoke it promptly on user action and expose measured consumption. Keep imported provider policy from overriding local limits. Test the full public route under no-wallet operation.

**Concrete example.** A user sets zero cellular contribution while watching a public video. Their device may use the data they explicitly request, but must not silently become a paid seeder on cellular because another module reads a default budget differently.

**Acceptance tests to implement.** Test opt-out defaults, runtime revocation, battery/network changes, concurrent job accounting and restart persistence. Run public actions with wallet services absent and verify no paid-tier dependency is introduced by federation.

**History, supersession and integration.** Extends #243; #245/#246 compose pricing/publication policy. The scheduler and physical power/bandwidth measurements remain required engineering work.

**Source entry points.** [`crates/mini-commons-policy/src/budget.rs`](https://github.com/mininet-labs/Mininet/blob/47e0eab332abd3f0269593a59c6c63f10b01514d/crates/mini-commons-policy/src/budget.rs); [`crates/mini-commons-policy/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/47e0eab332abd3f0269593a59c6c63f10b01514d/crates/mini-commons-policy/src/lib.rs); [`crates/mini-commons-policy/tests/social_commons.rs`](https://github.com/mininet-labs/Mininet/blob/47e0eab332abd3f0269593a59c6c63f10b01514d/crates/mini-commons-policy/tests/social_commons.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 2 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0245"></a>

## PR #245: mini-commons-policy: paid-service boundary against mini-resource-pricing (D-0363, Track C4)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-01, FD-09, FD-11, FD-16, FD-18.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/245) | [Files changed](https://github.com/mininet-labs/Mininet/pull/245/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/8a5f7d6929d59873703630285dd0b3f238e25640)

Head `8a5f7d6929d59873703630285dd0b3f238e25640`; base `cb16615ef72022ddd6cf2bc274d6a986e9f35515`; merge `f80995072c48aca330ad3847c2424995231f9bef`. 9 changed files; 1 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Connects public-commons entitlement to optional resource pricing without charging for the base Direct tier. It prevents the entitlement enum itself from becoming a way to discriminate prices by wealth.

**Mechanism and evidence.** service_quote_for returns no quote for Direct and delegates other tiers unchanged to mini-resource-pricing. The entitlement argument does not alter the quote. It introduces no wallet, settlement, governance or ranking authority.

**What remains weaker than the intended claim.** No quote for Direct is narrower than no charge anywhere in the product. Conversely, quoting Mixed/Burst does not prove those transports exist, and a quote cannot be an achieved privacy receipt. Structural privacy should not be presented as guaranteed merely because a stronger service has a price.

**Recommended improvement and rationale.** Distinguish capability availability, requested service, price quote and verified delivery. Refuse execution of unavailable privacy tiers before payment and keep basic publishing usable when every paid provider disappears. Evaluate whether essential protection requires a sustainable free minimum under the founder's privacy-first direction.

**Concrete example.** A user requests Mixed transport, receives a price, but no reviewed mix executor exists. The system must say unavailable and take no money; it must not silently send Direct while showing the purchased tier.

**Acceptance tests to implement.** Test Direct across all entitlement/balance conditions, unavailable tiers, overflow, interrupted payment and provider disappearance. At actual runtime boundaries assert no base action waits for a quote or settlement.

**History, supersession and integration.** Completes the narrow C4 policy crossing over #243/#244. #246 adds publication profiles; #296's real transport still does not implement Mixed/Burst.

**Source entry points.** [`crates/mini-commons-policy/src/boundary.rs`](https://github.com/mininet-labs/Mininet/blob/8a5f7d6929d59873703630285dd0b3f238e25640/crates/mini-commons-policy/src/boundary.rs); [`crates/mini-commons-policy/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/8a5f7d6929d59873703630285dd0b3f238e25640/crates/mini-commons-policy/src/lib.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0246"></a>

## PR #246: mini-publication-policy: publication profile, receipt, source-hiding path (D-0364/D-0365, Track D1-D3)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-01, FD-02, FD-09, FD-11, FD-14, FD-16.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/246) | [Files changed](https://github.com/mininet-labs/Mininet/pull/246/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/3a7f37535b05ed07393cc6b9844f94474be991f3)

Head `3a7f37535b05ed07393cc6b9844f94474be991f3`; base `f80995072c48aca330ad3847c2424995231f9bef`; merge `241a098c28cbb969a8159468102d664085ef1b54`. 13 changed files; 2 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Makes visibility, attribution, transport and persistence independent user choices rather than conflating anonymous authorship with private delivery or durable storage. This is a useful product model for sovereign publishing.

**Mechanism and evidence.** PublicationProfile permits all 72 combinations. Receipt construction composes route and quote functions; source_hiding_publication_path_for selects relay roles and refuses Direct or unimplemented mix paths. An attributed publication can still request network-source hiding.

**What remains weaker than the intended claim.** The named achieved-result receipt is produced from a plan, not a measured transmission or retained replica. A valid combination is not necessarily executable or secure. The early two-role plan predates the destination-encrypted three-hop implementation and must not imply its guarantees.

**Recommended improvement and rationale.** Rename planning outputs to quotes/plans until verified runtime evidence is available; make achieved properties separate types constructed after successful execution. Validate executability and policy intersections while preserving the independence of the user choices themselves.

**Concrete example.** A Public/Anonymous/Durable profile says what the user requests. It does not prove anonymity if the IP was exposed, nor durability if no holder accepted the object. Show requested, attempted and achieved states separately.

**Acceptance tests to implement.** Test all combinations for modeling consistency, then independently test execution refusal, destination encryption, provider failure, persistence receipts and no silent downgrade. Confirm attributed users can still select source-hiding transport.

**History, supersession and integration.** Composes #138/#145/#146 and #245. #253 adds shard placement and #296 actual onion transport; neither makes a planning-only receipt evidence of completed publication.

**Source entry points.** [`crates/mini-publication-policy/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/3a7f37535b05ed07393cc6b9844f94474be991f3/crates/mini-publication-policy/src/error.rs); [`crates/mini-publication-policy/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/3a7f37535b05ed07393cc6b9844f94474be991f3/crates/mini-publication-policy/src/lib.rs); [`crates/mini-publication-policy/src/profile.rs`](https://github.com/mininet-labs/Mininet/blob/3a7f37535b05ed07393cc6b9844f94474be991f3/crates/mini-publication-policy/src/profile.rs); [`crates/mini-publication-policy/src/receipt.rs`](https://github.com/mininet-labs/Mininet/blob/3a7f37535b05ed07393cc6b9844f94474be991f3/crates/mini-publication-policy/src/receipt.rs); [`crates/mini-publication-policy/src/source_hiding.rs`](https://github.com/mininet-labs/Mininet/blob/3a7f37535b05ed07393cc6b9844f94474be991f3/crates/mini-publication-policy/src/source_hiding.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0247"></a>

## PR #247: mini-presence: FileReplayGuard, a durable ReplayGuard backend (D-0366)

**FAIL** | Captured outcome: **merged** | Directives: FD-06, FD-09, FD-11, FD-15.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/247) | [Files changed](https://github.com/mininet-labs/Mininet/pull/247/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/ab14679a02f72403de2030a9c664ee2056b12f3a)

Head `ab14679a02f72403de2030a9c664ee2056b12f3a`; base `241a098c28cbb969a8159468102d664085ef1b54`; merge `ff6404ac1367cdfc3e7fcdf8716c882c1a993fd1`. 8 changed files; 2 commits; 0 issue comments, 12 inline comments and 1 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Adds persistent replay memory, addressing the real problem that a fresh process otherwise forgets accepted presence tokens. The implementation is useful prototype groundwork but does not meet a durable fail-closed replay guarantee.

**Mechanism and evidence.** FileReplayGuard loads a flat log, retains entries by local time, and best-effort appends/fsyncs after updating memory. The infallible ReplayGuard interface returns a freshness boolean while write_failures only exposes a separate counter. There is no cross-process lock.

**What remains weaker than the intended claim.** On append failure the caller can accept evidence that will be replayable after restart. Two processes can independently accept the same token. The parser tolerates an arbitrary malformed final line as truncation, reads all lines without a total cap, and slices purported hex as UTF-8 byte ranges; malformed non-ASCII input needs explicit rejection before slicing.

**Recommended improvement and rationale.** Make check-and-record fallible and atomic, durably commit before reporting fresh, and use an OS-backed lock or transactional backend. Validate canonical ASCII records, distinguish interrupted tail from corruption and derive retention from the full attestation validity window with clock-rollback discipline.

**Concrete example.** The disk is full: an attestation is accepted in memory, the append fails, and the phone restarts. Accepting the same attestation again is a replay failure, regardless of whether a diagnostic counter briefly reported the first write error.

**Acceptance tests to implement.** Inject disk-full/permission/fsync failures, run two independent processes, supply malformed Unicode/overlong records, interrupt every write boundary and roll the clock backward. A failed durable write must never return fresh.

**History, supersession and integration.** #248/#249 wire this guard into demos, increasing the importance of the boundary. They do not repair its infallible/durable mismatch; production reward/personhood reliance remains blocked.

**Source entry points.** [`crates/mini-presence/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/ab14679a02f72403de2030a9c664ee2056b12f3a/crates/mini-presence/src/lib.rs); [`crates/mini-presence/src/persisted.rs`](https://github.com/mininet-labs/Mininet/blob/ab14679a02f72403de2030a9c664ee2056b12f3a/crates/mini-presence/src/persisted.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0248"></a>

## PR #248: mini-presence/mini-keystone: durable replay guard + real range measurement (D-0367/D-0368)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-06, FD-08, FD-11, FD-15.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/248) | [Files changed](https://github.com/mininet-labs/Mininet/pull/248/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/29bd8bcaf9b80377bd44af6a144f2a5956f95b69)

Head `29bd8bcaf9b80377bd44af6a144f2a5956f95b69`; base `ff6404ac1367cdfc3e7fcdf8716c882c1a993fd1`; merge `d6271802d5b014a010378c7895281671fd884cd3`. 12 changed files; 2 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Replaces fabricated RTT samples with actual challenge/response measurements and threads caller-owned replay guards through the keystone demo. It converts two previously simulated ingredients into observable operations.

**Mechanism and evidence.** Fresh encrypted challenges are sent over the established channel, echoed and timed with Instant. run_demo performs the configured samples and signs them into the presence attestation; each participant supplies its own persistent-capable ReplayGuard.

**What remains weaker than the intended claim.** Millisecond software RTT is not a meter-scale distance-bounding protocol or proof of a unique human. A fast network relay may fit a generous BLE threshold, and a colluding measurer can still lie about its own recorded timing. The underlying file guard retains #247's write-failure and concurrency limitations.

**Recommended improvement and rationale.** Call this measured software round-trip evidence and calibrate its evidentiary weight through adversarial relay experiments. Use hardware distance-bounding only under a separately analyzed protocol; never treat vendor hardware attestation as a hidden universal personhood authority. Fix durable replay before value depends on it.

**Concrete example.** A remote pair of devices connected by a low-latency relay can answer within 50 ms. Passing that test does not establish that two humans stood within a few meters of each other.

**Acceptance tests to implement.** Test delayed/forwarded/early responses, challenge substitution, different channels, asymmetric measurement, clock-independent timing and real BLE/UWB relay attacks. Measure false acceptance/rejection across devices, walls and congestion; no happy-path demo can close that gate.

**History, supersession and integration.** Composes #247 and earlier presence primitives; #249 exposes the demo through the CLI. Physical personhood and hardware tests remain outstanding.

**Source entry points.** [`crates/mini-keystone/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/29bd8bcaf9b80377bd44af6a144f2a5956f95b69/crates/mini-keystone/src/lib.rs); [`crates/mini-keystone/tests/keystone.rs`](https://github.com/mininet-labs/Mininet/blob/29bd8bcaf9b80377bd44af6a144f2a5956f95b69/crates/mini-keystone/tests/keystone.rs); [`crates/mini-presence/src/active_range.rs`](https://github.com/mininet-labs/Mininet/blob/29bd8bcaf9b80377bd44af6a144f2a5956f95b69/crates/mini-presence/src/active_range.rs); [`crates/mini-presence/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/29bd8bcaf9b80377bd44af6a144f2a5956f95b69/crates/mini-presence/src/error.rs); [`crates/mini-presence/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/29bd8bcaf9b80377bd44af6a144f2a5956f95b69/crates/mini-presence/src/lib.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0249"></a>

## PR #249: mini-cli: mini keystone run, the standalone CLI harness (D-0369)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-03, FD-06, FD-11, FD-14.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/249) | [Files changed](https://github.com/mininet-labs/Mininet/pull/249/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/0a2380a8998de9eb2d489b698ec39468fd8bebb3)

Head `0a2380a8998de9eb2d489b698ec39468fd8bebb3`; base `d6271802d5b014a010378c7895281671fd884cd3`; merge `8582493e09ea1c81a5956da4cdbc0f697b4acaba`. 14 changed files; 1 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Puts the keystone lifecycle behind the real mini command and expands the no-GitHub demonstration, making independent testers less dependent on developer-only examples or a hosted control plane.

**Mechanism and evidence.** mini keystone run loads two persisted homes, opens file replay guards and calls the existing demo through an in-process bearer pair. The outage script combines this with repository/PR/release/verification/install/rollback operations using the compiled CLI.

**What remains weaker than the intended claim.** Two homes in one process are not two independently controlled devices or a hostile network. The demonstration establishes that GitHub APIs are not needed for that local flow; it does not prove release governance survives founder disappearance, nor that replay durability or proximity evidence is secure.

**Recommended improvement and rationale.** Add a multi-process, independently provisioned, no-network-to-GitHub drill with separate custody and persistent stores. Keep the in-process demo for deterministic debugging but label its shared-process trust boundary clearly in output and documentation.

**Concrete example.** Disconnect GitHub and run Alice's node and Bob's node on different machines with no shared filesystem. They should exchange verifiable objects and independently verify an owner-selected release without any founder-held secret.

**Acceptance tests to implement.** Test repeated runs, distinct-home enforcement, partially initialized homes, replay-log failure and interrupted release/install phases. Capture exact binaries and network traces showing the claimed dependencies are absent.

**History, supersession and integration.** Builds on #115 and #248. It is a useful composition test, not closure of physical BLE, independent governance or external cryptographic review.

**Source entry points.** [`crates/mini-cli/src/cli.rs`](https://github.com/mininet-labs/Mininet/blob/0a2380a8998de9eb2d489b698ec39468fd8bebb3/crates/mini-cli/src/cli.rs); [`crates/mini-cli/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/0a2380a8998de9eb2d489b698ec39468fd8bebb3/crates/mini-cli/src/error.rs); [`crates/mini-cli/src/keystone.rs`](https://github.com/mininet-labs/Mininet/blob/0a2380a8998de9eb2d489b698ec39468fd8bebb3/crates/mini-cli/src/keystone.rs); [`crates/mini-cli/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/0a2380a8998de9eb2d489b698ec39468fd8bebb3/crates/mini-cli/src/lib.rs); [`crates/mini-cli/tests/no_github_outage_demo.rs`](https://github.com/mininet-labs/Mininet/blob/0a2380a8998de9eb2d489b698ec39468fd8bebb3/crates/mini-cli/tests/no_github_outage_demo.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 1 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0250"></a>

## PR #250: Android: real AndroidKeystoreCipher, closes issue #198's Kotlin gap (D-0370)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-01, FD-06, FD-09, FD-11, FD-14.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/250) | [Files changed](https://github.com/mininet-labs/Mininet/pull/250/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/28c4f098abd42ff541b2e8aab7cf1785f37480de)

Head `28c4f098abd42ff541b2e8aab7cf1785f37480de`; base `8582493e09ea1c81a5956da4cdbc0f697b4acaba`; merge `42c685bcbbde3ddfd9f84209d4daff4a33923d51`. 9 changed files; 1 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Finally persists Android identity using a real platform cipher and refuses to mint a replacement root when restoration fails. This directly protects ordinary users from losing identity every time the app closes.

**Mechanism and evidence.** AndroidKeystoreCipher uses a non-exportable AES-GCM wrapping key and stores IV plus authenticated ciphertext. The view model restores root_state.bin and persists after creation. A distinct RestoreFailed UI state separates damaged state from an unavailable Rust core.

**What remains weaker than the intended claim.** The identity signing seeds remain software keys and temporarily enter managed memory through #209; the non-exportable wrapping key does not make signing hardware-isolated. Keystore hardware backing varies. The initial Kotlin implementation did not compile and was repaired in #251, so a pre-fix source claim is not a successful APK.

**Recommended improvement and rationale.** Use atomic ciphertext replacement and platform lifecycle persistence, explicitly bind ciphertext to application/schema context, and design owner-controlled recovery for keystore loss. Show actual custody strength without requiring a central hardware-attestation service or automatically weakening encryption on failure.

**Concrete example.** After a system restore the ciphertext exists but its Keystore key is gone. The app must explain unrecoverable local custody or offer an owner-held backup path; silently creating a new root would erase continuity while looking like success.

**Acceptance tests to implement.** Test cold-start, wrong/missing key, corrupted tag/IV, interrupted writes, key invalidation, app backup/restore, schema upgrades and low-memory termination. Inspect logs and managed plaintext exposure on real devices.

**History, supersession and integration.** Completes #206/#209/#217's platform persistence path; #251 fixes its constructor signature and #258 extends the persisted format for contacts/replay. It does not close the hardware custody audit.

**Source entry points.** [`crates/mini-ffi/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/28c4f098abd42ff541b2e8aab7cf1785f37480de/crates/mini-ffi/src/lib.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0251"></a>

## PR #251: Android: fix AndroidKeystoreCipher.kt compile error (D-0370 follow-up)

**PASS** | Captured outcome: **merged** | Directives: FD-06, FD-10, FD-14.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/251) | [Files changed](https://github.com/mininet-labs/Mininet/pull/251/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/31243413782900b454c06a7602d013647d260f5c)

Head `31243413782900b454c06a7602d013647d260f5c`; base `42c685bcbbde3ddfd9f84209d4daff4a33923d51`; merge `7a7cd6322ef15b723ae965cb3f95b399773bb189`. 1 changed files; 1 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Fixes the actual Kotlin compilation error that prevented the newly added persistence implementation from building. This is a concrete correction to the previous PR's unverified platform code.

**Mechanism and evidence.** Three StorageCipherException.Failed constructions now supply the message argument required by generated UniFFI bindings. The change is limited to platform exception construction and does not alter cryptographic parameters.

**What remains weaker than the intended claim.** The scoped PASS concerns the identified signature mismatch. A message-bearing exception can compile while leaking sensitive platform details into logs or while the persistence behavior remains wrong. Rust-only tests cannot validate this change.

**Recommended improvement and rationale.** Keep generated-binding compilation mandatory before merge of Kotlin/UDL changes and map platform exceptions to safe user-facing error categories. Preserve diagnostic details locally only when they do not expose keys, paths or sensitive account state.

**Concrete example.** A corrupted ciphertext should display a recoverable explanation such as stored identity could not be opened, not an unhandled constructor error or a raw dump of internal state.

**Acceptance tests to implement.** Compile the exact APK against regenerated bindings and exercise all three exception branches. Test missing key and short/corrupt ciphertext on Android; verify errors neither replace the identity nor expose secret bytes.

**History, supersession and integration.** Repairs #250/D-0370. This correction is evidence that CI caught the mismatch, not that merging the prior red Android build was harmless or should become normal practice.

**Source entry points.** [`app/android/app/src/main/java/org/mininet/app/AndroidKeystoreCipher.kt`](https://github.com/mininet-labs/Mininet/blob/31243413782900b454c06a7602d013647d260f5c/app/android/app/src/main/java/org/mininet/app/AndroidKeystoreCipher.kt). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0252"></a>

## PR #252: mini-web-extract: sandboxed-in-principle static HTML extraction (D-0371, Track E4)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-06, FD-09, FD-11, FD-14.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/252) | [Files changed](https://github.com/mininet-labs/Mininet/pull/252/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/1d77a4b800c3225dec735b49e673bd50be5e8c3f)

Head `1d77a4b800c3225dec735b49e673bd50be5e8c3f`; base `7a7cd6322ef15b723ae965cb3f95b399773bb189`; merge `3539f5d40901fe1bffca8881ca7ecfd59a2674ed`. 14 changed files; 2 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Adds a deterministic static HTML extraction stage without executing JavaScript, reducing dependence on a browser engine or hosted parser for the first MiniSearch pipeline.

**Mechanism and evidence.** A bounded hand-written tokenizer extracts visible text, title, headings, links, language/meta/canonical hints and a content digest. Script/style/noscript/template contents are skipped as raw text. The function has no network client and forbids unsafe Rust.

**What remains weaker than the intended claim.** Safe Rust and no JavaScript do not provide process isolation or full HTML-standard correctness. Malformed markup, entities and hidden-text heuristics can influence ranking inputs. Canonical-link and language metadata are publisher assertions, not trustworthy provenance. The title correctly says sandboxed-in-principle, not sandboxed in operation.

**Recommended improvement and rationale.** Run extraction in the hardened #172 worker boundary and compare outputs against a well-tested parser corpus. Document deliberate semantic differences instead of treating a small tokenizer as complete browser equivalence. Preserve raw source/digest and label every inferred field.

**Concrete example.** A page declares a canonical URL belonging to another publisher and hides keyword stuffing in malformed markup. Extraction must not transfer authorship or unquestioned relevance to the claimed URL; it should retain the observation and parsing uncertainty.

**Acceptance tests to implement.** Use adversarial nested/malformed tags, long attributes, Unicode entities, raw-text terminators, hidden containers and pathological inputs. Assert time/memory bounds and differential-test against public HTML test corpora without executing content.

**History, supersession and integration.** Follows #160 crawler planning and #172 worker protocol; #256/#257 consume extracted fields and #283 fetches bytes. A complete sandboxed crawl-to-index pipeline remains separate.

**Source entry points.** [`crates/mini-web-extract/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/1d77a4b800c3225dec735b49e673bd50be5e8c3f/crates/mini-web-extract/src/error.rs); [`crates/mini-web-extract/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/1d77a4b800c3225dec735b49e673bd50be5e8c3f/crates/mini-web-extract/src/lib.rs); [`crates/mini-web-extract/src/limits.rs`](https://github.com/mininet-labs/Mininet/blob/1d77a4b800c3225dec735b49e673bd50be5e8c3f/crates/mini-web-extract/src/limits.rs); [`crates/mini-web-extract/src/parse.rs`](https://github.com/mininet-labs/Mininet/blob/1d77a4b800c3225dec735b49e673bd50be5e8c3f/crates/mini-web-extract/src/parse.rs); [`crates/mini-web-extract/src/types.rs`](https://github.com/mininet-labs/Mininet/blob/1d77a4b800c3225dec735b49e673bd50be5e8c3f/crates/mini-web-extract/src/types.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0253"></a>

## PR #253: mini-replication-policy: suppression-resistant shard placement/repair/retrieval (D-0372, Track D5)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-06, FD-08, FD-11, FD-15.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/253) | [Files changed](https://github.com/mininet-labs/Mininet/pull/253/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/6b8ed87244a6d570d75eddc12d4099c3ce16a33c)

Head `6b8ed87244a6d570d75eddc12d4099c3ce16a33c`; base `3539f5d40901fe1bffca8881ca7ecfd59a2674ed`; merge `e509cb2d60f81f6c2b7e85dd4c8088a38977af95`. 13 changed files; 2 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Provides erasure-shard placement and repair planning that avoids assigning multiple shards to the same listed DID. It gives suppression-resistance policy a concrete interface over the real erasure codec.

**Mechanism and evidence.** plan_placement assigns distinct candidate holders; plan_repair_placement replaces missing holders with fresh distinct DIDs; select_retrieval_set chooses a deterministic subset. Integration tests reconstruct real data after simulated holder loss and repair.

**What remains weaker than the intended claim.** Distinct DIDs are not independent people, machines, networks or failure domains. A warehouse can supply many candidates, so removing one operator may still remove all shards. The policy does not transfer data, prove custody, measure availability or guarantee the chosen retrieval subset will respond.

**Recommended improvement and rationale.** Separate cryptographic replica correctness from operational diversity. Add locally chosen, privacy-minimized fault-domain constraints and multi-peer retrieval/repair with real possession evidence; never install a central operator-certification registry as the shortcut to diversity.

**Concrete example.** A 4-of-7 plan distributed to seven DIDs controlled by one cloud tenant survives no tenant outage. Test that scenario explicitly instead of interpreting DID uniqueness as seven independent holders.

**Acceptance tests to implement.** Test duplicate identities, insufficient candidates, correlated failures, malicious shard bytes, withheld responses, repair churn and privacy leakage from placement metadata. Measure recovery on independent physical nodes and retain the unresolved operator-independence assumption.

**History, supersession and integration.** Builds on #101/#106 erasure coding and #246 publication policy. #297-#306 storage-fraud/capacity work adds evidence but still does not prove independent operators.

**Source entry points.** [`crates/mini-replication-policy/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/6b8ed87244a6d570d75eddc12d4099c3ce16a33c/crates/mini-replication-policy/src/error.rs); [`crates/mini-replication-policy/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/6b8ed87244a6d570d75eddc12d4099c3ce16a33c/crates/mini-replication-policy/src/lib.rs); [`crates/mini-replication-policy/src/placement.rs`](https://github.com/mininet-labs/Mininet/blob/6b8ed87244a6d570d75eddc12d4099c3ce16a33c/crates/mini-replication-policy/src/placement.rs); [`crates/mini-replication-policy/tests/end_to_end.rs`](https://github.com/mininet-labs/Mininet/blob/6b8ed87244a6d570d75eddc12d4099c3ce16a33c/crates/mini-replication-policy/tests/end_to_end.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0256"></a>

## PR #256: feat(mini-lexical-index): deterministic inverted index, MiniSearch Track E5 (D-0405, closes #254)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-06, FD-10, FD-11, FD-14.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/256) | [Files changed](https://github.com/mininet-labs/Mininet/pull/256/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/0e8ab593ee97c6bf15f85051d226dbcbaffe7a34)

Head `0e8ab593ee97c6bf15f85051d226dbcbaffe7a34`; base `5e2bcdd15f2024ea86cf85fbb44a7a9662d6bff0`; merge `1139494c8cc9626b880de6f4a90b2b98192d98b8`. 16 changed files; 4 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Implements a deterministic local inverted index with phrase positions, making search data portable and comparable instead of owned by one hosted search operator.

**Mechanism and evidence.** IndexBuilder freezes field-specific postings into canonical IndexSegment bytes. BLAKE3 identifies the segment; decoding checks sorted terms/documents/positions and rejects dangling references, wrong versions and trailing/truncated bytes. Queries operate on already-held data.

**What remains weaker than the intended claim.** Content addressing proves which bytes were indexed, not that the crawl was complete, the publisher honest or the language analysis adequate. A bounded segment does not establish bounded total index/query resources, and a deterministic tokenizer can still systematically underserve languages or malformed text.

**Recommended improvement and rationale.** Publish tokenizer/version semantics and use a multilingual adversarial corpus. Add streaming segment construction, merge/compaction budgets and query work limits; preserve independently forkable indexes and provenance back to original observations.

**Concrete example.** Two providers indexing identical source documents in different insertion order should produce the same segment. Two providers omitting different documents must not be described as agreeing on web truth merely because each segment has a valid hash.

**Acceptance tests to implement.** Test Unicode normalization choices, non-Latin scripts, phrase boundaries across fields, repeated terms, maximum postings and incremental compaction parity. Measure peak construction/query memory and verify malformed segment rejection before large allocations.

**History, supersession and integration.** Consumes #252 extraction and #160 web types; #257 ranks matches, #281 exchanges segments and #290 carries them over real transport. Those are separate guarantees from deterministic indexing.

**Source entry points.** [`crates/mini-lexical-index/src/codec.rs`](https://github.com/mininet-labs/Mininet/blob/0e8ab593ee97c6bf15f85051d226dbcbaffe7a34/crates/mini-lexical-index/src/codec.rs); [`crates/mini-lexical-index/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/0e8ab593ee97c6bf15f85051d226dbcbaffe7a34/crates/mini-lexical-index/src/error.rs); [`crates/mini-lexical-index/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/0e8ab593ee97c6bf15f85051d226dbcbaffe7a34/crates/mini-lexical-index/src/lib.rs); [`crates/mini-lexical-index/src/segment.rs`](https://github.com/mininet-labs/Mininet/blob/0e8ab593ee97c6bf15f85051d226dbcbaffe7a34/crates/mini-lexical-index/src/segment.rs); [`crates/mini-lexical-index/src/token.rs`](https://github.com/mininet-labs/Mininet/blob/0e8ab593ee97c6bf15f85051d226dbcbaffe7a34/crates/mini-lexical-index/src/token.rs); [`crates/mini-lexical-index/tests/index.rs`](https://github.com/mininet-labs/Mininet/blob/0e8ab593ee97c6bf15f85051d226dbcbaffe7a34/crates/mini-lexical-index/tests/index.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0257"></a>

## PR #257: feat(mini-ranker): transparent deterministic ranker, MiniSearch Track E6 (D-0406, closes #255)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-01, FD-09, FD-10, FD-11, FD-16, FD-17.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/257) | [Files changed](https://github.com/mininet-labs/Mininet/pull/257/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/af0c43ffa9221aa8b8781cd84dfee4fe2a59b25b)

Head `af0c43ffa9221aa8b8781cd84dfee4fe2a59b25b`; base `5e2bcdd15f2024ea86cf85fbb44a7a9662d6bff0`; merge `77561766e5e3b9f94935493e08851feafa9add2e`. 25 changed files; 7 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Adds transparent, forkable local ranking with explicit per-signal explanations, rejecting direct paid-ranking inputs and hidden default personalization. This is central to avoiding a new search monopoly.

**Mechanism and evidence.** rank combines integer lexical, phrase, link, freshness, originality and domain-diversity signals under a RankingProfile. Availability restrictions filter before scoring; ties use canonical identifiers. A greedy diversity loop recomputes scores as results are selected.

**What remains weaker than the intended claim.** No payment argument prevents direct paid bids in this function, not indirect manipulation through purchased links, domains or false metadata. Earliest observed duplicate is not proof of original authorship. Caller-supplied availability can become censorship if treated as universal, and max_results does not bound the initial candidate/postings scan.

**Recommended improvement and rationale.** Label ranking inputs by provenance and uncertainty; preserve user choice of profiles and providers. Bound candidate work and distinguish local/user restrictions from global truth. Add manipulation tests and avoid promoting provider-supplied link/freshness figures to unquestioned organic authority.

**Concrete example.** A wealthy operator buys many domains and supplies inflated inbound-link counts. The ranker remains numerically deterministic while its inputs are gamed; tests must measure that failure rather than infer fairness from the absence of a bid field.

**Acceptance tests to implement.** Test source/arrival-order parity, forged metadata, adversarial duplicate timestamps, domain farms, unavailable-content visibility and worst-case candidate work. Compare complete explanations and user-selected profile changes without network re-query.

**History, supersession and integration.** Builds on #256; #278 adds query/provenance, #282 local rescoring, and #294 remote results introduce a new trust boundary for claimed scores.

**Source entry points.** [`crates/mini-lexical-index/src/codec.rs`](https://github.com/mininet-labs/Mininet/blob/af0c43ffa9221aa8b8781cd84dfee4fe2a59b25b/crates/mini-lexical-index/src/codec.rs); [`crates/mini-lexical-index/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/af0c43ffa9221aa8b8781cd84dfee4fe2a59b25b/crates/mini-lexical-index/src/error.rs); [`crates/mini-lexical-index/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/af0c43ffa9221aa8b8781cd84dfee4fe2a59b25b/crates/mini-lexical-index/src/lib.rs); [`crates/mini-lexical-index/src/segment.rs`](https://github.com/mininet-labs/Mininet/blob/af0c43ffa9221aa8b8781cd84dfee4fe2a59b25b/crates/mini-lexical-index/src/segment.rs); [`crates/mini-lexical-index/src/token.rs`](https://github.com/mininet-labs/Mininet/blob/af0c43ffa9221aa8b8781cd84dfee4fe2a59b25b/crates/mini-lexical-index/src/token.rs); [`crates/mini-lexical-index/tests/index.rs`](https://github.com/mininet-labs/Mininet/blob/af0c43ffa9221aa8b8781cd84dfee4fe2a59b25b/crates/mini-lexical-index/tests/index.rs); [`crates/mini-ranker/src/corpus.rs`](https://github.com/mininet-labs/Mininet/blob/af0c43ffa9221aa8b8781cd84dfee4fe2a59b25b/crates/mini-ranker/src/corpus.rs); [`crates/mini-ranker/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/af0c43ffa9221aa8b8781cd84dfee4fe2a59b25b/crates/mini-ranker/src/error.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 1 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0258"></a>

## PR #258: Android: signed LAN/QR mutual follow for Day 0 (D-0373)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-01, FD-02, FD-06, FD-08, FD-09, FD-11.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/258) | [Files changed](https://github.com/mininet-labs/Mininet/pull/258/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/9748e8b9d7479860115e01a6b13d01a0ae9122da)

Head `9748e8b9d7479860115e01a6b13d01a0ae9122da`; base `fa813cdc24062bd80409c7c885d0000f8d80ad96`; merge `79433e06f89561819a1f996724250abaf3e66bf3`. 15 changed files; 9 commits; 0 issue comments, 1 inline comments and 1 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Delivers the Android product bridge from signed QR invitation through real foreground LAN exchange to persisted mutual follows. It is a genuine user-visible integration rather than another protocol-only slice.

**Mechanism and evidence.** RootCore creates random expiring offers, validates root/device delegation and restricts advertised endpoints to private/link-local addresses. State v2 persists contacts, signed follows, author sequence and consumed nonces under the existing cipher. Kotlin renders/scans QR locally and runs blocking LAN work off the UI thread.

**What remains weaker than the intended claim.** The system camera thumbnail contract may not preserve sufficient QR detail, and the required two-physical-device test is not replaced by loopback or successful compilation. A private address is not automatically a trustworthy peer. Plaintext custody retains #209's limitations, and authentic old state can roll back replay protection without a stronger anchor.

**Recommended improvement and rationale.** Bind the displayed invitation to explicit user consent and the exact DID, test full-resolution camera capture, and make pairing persistence transactional before success is shown. Add anti-rollback/freshness behavior and ensure an untrusted QR cannot access unintended local services.

**Concrete example.** A copied invitation scanned after a successful pairing and restart must be refused. If storage fails after a follow is signed, retry must recover that same operation rather than create another identity, duplicate follow or replay window.

**Acceptance tests to implement.** Run two physical phones through scan/connect/follow/restart/replay, camera-app variants, network changes, force-stop at every boundary, wrong endpoints, expired offers and full storage. Capture actual permissions and verify no tracker or account server is contacted.

**History, supersession and integration.** Integrates #211, #206/#209 and #250/#251. It does not implement device enrollment UI, background BLE exchange or personhood.

**Source entry points.** [`crates/mini-ffi/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/9748e8b9d7479860115e01a6b13d01a0ae9122da/crates/mini-ffi/src/lib.rs); [`crates/mini-ffi/src/mini_ffi.udl`](https://github.com/mininet-labs/Mininet/blob/9748e8b9d7479860115e01a6b13d01a0ae9122da/crates/mini-ffi/src/mini_ffi.udl); [`crates/mini-ffi/src/pairing.rs`](https://github.com/mininet-labs/Mininet/blob/9748e8b9d7479860115e01a6b13d01a0ae9122da/crates/mini-ffi/src/pairing.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0259"></a>

## PR #259: mini-bearer: AndroidBleBearer/BleRadio, Rust-side BLE bearer (D-0374, issue #201)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-06, FD-11, FD-14.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/259) | [Files changed](https://github.com/mininet-labs/Mininet/pull/259/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/882cbbf686ab4a52372e3ae746ec0a3aa6dd082f)

Head `882cbbf686ab4a52372e3ae746ec0a3aa6dd082f`; base `e509cb2d60f81f6c2b7e85dd4c8088a38977af95`; merge `5e2bcdd15f2024ea86cf85fbb44a7a9662d6bff0`. 8 changed files; 2 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Connects BLE framing to the common Bearer trait through a replaceable radio interface. It provides a testable Rust transport adapter without pretending a simulated radio is physical Bluetooth.

**Mechanism and evidence.** AndroidBleBearer<R: BleRadio> uses write_chunk/read_chunk/try_read_chunk to drive the existing chunker and reassembler. Radio failures map to Bearer errors, and MTU/count constraints are checked before sending.

**What remains weaker than the intended claim.** An implementation of BleRadio can block indefinitely or violate the negotiated payload contract. The generic adapter adds no actual GATT lifecycle, discovery, connection security, congestion control or mobile background behavior. Collecting all chunks still inherits the framing memory cost.

**Recommended improvement and rationale.** Define radio deadline/cancellation semantics and streaming backpressure, then provide a platform adapter with explicit connection generation and partial-frame cleanup. Keep BLE addresses and pairing hints outside identity authority.

**Concrete example.** A phone disconnects after half a frame, then reconnects with a smaller MTU. Old partial bytes must not be combined with the new connection or cause an unbounded wait while the UI reports success.

**Acceptance tests to implement.** Test disconnect/reconnect, empty/partial frames, MTU change, blocking radio cancellation, wrong chunk count and maximum memory. Add hardware acceptance over independent Android devices after the real adapter exists.

**History, supersession and integration.** Builds on #213; #260 exports the radio callback through FFI. Neither PR is evidence that a production GATT implementation has been exercised.

**Source entry points.** [`crates/mini-bearer/src/android_ble.rs`](https://github.com/mininet-labs/Mininet/blob/882cbbf686ab4a52372e3ae746ec0a3aa6dd082f/crates/mini-bearer/src/android_ble.rs); [`crates/mini-bearer/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/882cbbf686ab4a52372e3ae746ec0a3aa6dd082f/crates/mini-bearer/src/lib.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0260"></a>

## PR #260: mini-ffi: BleRadio UniFFI callback interface + BleBearerHandle (D-0375, issue #201)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-06, FD-09, FD-11, FD-14.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/260) | [Files changed](https://github.com/mininet-labs/Mininet/pull/260/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/a8f812db179f2009d8f1120ac463cc8393430b99)

Head `a8f812db179f2009d8f1120ac463cc8393430b99`; base `79433e06f89561819a1f996724250abaf3e66bf3`; merge `5cb327123b9c9d4081e26d0dc4e20bb989ccb5e6`. 8 changed files; 4 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Exposes the Rust BLE bearer to a platform-provided radio through UniFFI, closing the language-boundary gap while keeping the platform responsible only for opaque byte transport.

**Mechanism and evidence.** A callback BleRadio is adapted from shared-reference FFI methods to the Rust mutable radio trait. BleBearerHandle owns AndroidBleBearer behind a Mutex and maps errors into a finite FFI-safe enum. Generated scaffolding checks Rust/UDL compatibility.

**What remains weaker than the intended claim.** A mutex-held blocking callback can block cancellation or re-enter the same handle and deadlock. Scaffolding compilation is not proof of generated Kotlin runtime behavior. The PR explicitly has no real GATT implementation or physical exchange.

**Recommended improvement and rationale.** Document and enforce non-reentrant callback/cancellation rules, avoid holding broad locks across unbounded platform calls, and attach per-operation deadlines. Validate generated Kotlin bindings and the actual asynchronous GATT completion semantics rather than treating characteristic submission as delivery.

**Concrete example.** Kotlin write_chunk queues a write but the connection closes before its callback. Rust must receive failure, not infer that the frame was delivered merely because the enqueue call returned.

**Acceptance tests to implement.** Test callback reentrancy, blocked read cancellation, mutex poisoning/error mapping, asynchronous write failure, oversize frames and Kotlin exception propagation. A physical two-phone transfer must exercise the exact generated interface.

**History, supersession and integration.** Extends #259 using the callback pattern from #209. The hardware and background-lifecycle requirements remain open, not completed by a cross-language type match.

**Source entry points.** [`crates/mini-ffi/src/ble.rs`](https://github.com/mininet-labs/Mininet/blob/a8f812db179f2009d8f1120ac463cc8393430b99/crates/mini-ffi/src/ble.rs); [`crates/mini-ffi/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/a8f812db179f2009d8f1120ac463cc8393430b99/crates/mini-ffi/src/lib.rs); [`crates/mini-ffi/src/mini_ffi.udl`](https://github.com/mininet-labs/Mininet/blob/a8f812db179f2009d8f1120ac463cc8393430b99/crates/mini-ffi/src/mini_ffi.udl). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0261"></a>

## PR #261: Record D-0376: defer nav-index regen from per-commit ritual (policy, CLAUDE.md unchanged)

**PASS** | Captured outcome: **merged** | Directives: FD-05, FD-06, FD-10, FD-12, FD-14.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/261) | [Files changed](https://github.com/mininet-labs/Mininet/pull/261/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/75d2b1453c717d17c525d588c9a016d6d9616392)

Head `75d2b1453c717d17c525d588c9a016d6d9616392`; base `77561766e5e3b9f94935493e08851feafa9add2e`; merge `fa813cdc24062bd80409c7c885d0000f8d80ad96`. 1 changed files; 4 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Reduces avoidable merge conflicts from full-tree generated navigation and records the workflow change without silently editing the protected AI instruction surface. The reverted instruction edit is an important part of the history.

**Mechanism and evidence.** D-0376 proposes periodic dedicated navigation refresh rather than regeneration in every ordinary PR. The final diff records the decision only; the earlier CLAUDE.md change was removed after the canonical governance checker refused it.

**What remains weaker than the intended claim.** The scoped PASS is for preserving the instruction trust boundary while recording a real maintenance cost. A policy entry and unchanged old instructions can still leave contributors with contradictory operational guidance. Generated files are not intrinsically meaningless: stale navigation can mislead reviewers if its revision is hidden.

**Recommended improvement and rationale.** Add a legitimate versioned instruction-amendment path and record the generated index's source revision. Prefer deriving navigation at read/build time or updating it in dedicated maintenance commits. Do not give a proposal branch authority to rewrite the validator or instructions judging that proposal.

**Concrete example.** An AI branch wants to remove a security step from its own entry instructions to make CI green. The canonical validator must reject that change; an unrelated docs policy cannot silently authorize it.

**Acceptance tests to implement.** Test protected-surface byte/digest mismatch, authorized amendment through the declared phase, stale generated-index labeling and concurrent non-overlapping PRs. Preserve a visible explanation of which workflow instruction currently controls.

**History, supersession and integration.** Follows repeated conflicts in #179 and peers. Later PRs still regenerated navigation, so the historical policy decision should not be mistaken for uniformly enforced current practice.

**Source entry points.** [`docs/DECISION_LOG.md`](https://github.com/mininet-labs/Mininet/blob/75d2b1453c717d17c525d588c9a016d6d9616392/docs/DECISION_LOG.md). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0265"></a>

## PR #265: docs: prepare Beta contributor front door and review gates (D-0101)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-01, FD-02, FD-08, FD-10, FD-12, FD-18.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/265) | [Files changed](https://github.com/mininet-labs/Mininet/pull/265/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/53a7e3b9594f9274adaf853820c7c3e24e5ca920)

Head `53a7e3b9594f9274adaf853820c7c3e24e5ca920`; base `5cb327123b9c9d4081e26d0dc4e20bb989ccb5e6`; merge `bf3adf6e96fa3b7e4af096a9949155ea41a62603`. 19 changed files; 2 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Creates a contributor front door, structured beta reports and a reusable external-review response template so outsiders can contribute without reconstructing hundreds of decisions. It also scopes native teams without granting GitHub teams protocol authority.

**Mechanism and evidence.** The documentation batch adds contributor/task guides, intake/test issue forms, gate response fields and a D-0101 working-group/routing report. It updates live roadmap/gate references and explicitly distinguishes coordination, approval, release and owner adoption.

**What remains weaker than the intended claim.** Forms can collect unnecessary personal/employer information or create a de facto maintainer gate if treated as mandatory enrollment. A proposed team charter is not an activated delegation; a polished beta guide does not replace physical security or external audit. Historical Windows execution limitations must remain visible.

**Recommended improvement and rationale.** Keep contributor intake voluntary and pseudonymous, minimize identity fields and provide an offline/native submission route. Route by task requirements, not employer prestige or payment. Require exact-scope evidence and named residual risks in external responses without turning one auditor into permanent authority.

**Concrete example.** A contributor using only a pseudonym and an old machine should be able to submit a reproducible bug report without disclosing employer, legal name or buying a token. The report gains weight through evidence, not profile completeness.

**Acceptance tests to implement.** Test the first-time contributor journey with no wallet, no organization membership and no GitHub runtime dependency. Check links, accepted evidence formats, private-data warnings and that AI-generated templates contain no prefilled approval claims.

**History, supersession and integration.** #321 later extends this contributor-facing groundwork with unified audit navigation; #267 implements native coordination objects. This guide itself activates no team or governance right.

**Source entry points.** [`.github/ISSUE_TEMPLATE/beta-test-report.yml`](https://github.com/mininet-labs/Mininet/blob/53a7e3b9594f9274adaf853820c7c3e24e5ca920/.github/ISSUE_TEMPLATE/beta-test-report.yml); [`.github/ISSUE_TEMPLATE/contributor-intake.yml`](https://github.com/mininet-labs/Mininet/blob/53a7e3b9594f9274adaf853820c7c3e24e5ca920/.github/ISSUE_TEMPLATE/contributor-intake.yml); [`CONTRIBUTING.md`](https://github.com/mininet-labs/Mininet/blob/53a7e3b9594f9274adaf853820c7c3e24e5ca920/CONTRIBUTING.md); [`README.md`](https://github.com/mininet-labs/Mininet/blob/53a7e3b9594f9274adaf853820c7c3e24e5ca920/README.md); [`WHITEPAPER.md`](https://github.com/mininet-labs/Mininet/blob/53a7e3b9594f9274adaf853820c7c3e24e5ca920/WHITEPAPER.md); [`docs/AUDITOR_START.md`](https://github.com/mininet-labs/Mininet/blob/53a7e3b9594f9274adaf853820c7c3e24e5ca920/docs/AUDITOR_START.md); [`docs/BETA_CONTRIBUTOR_GUIDE.md`](https://github.com/mininet-labs/Mininet/blob/53a7e3b9594f9274adaf853820c7c3e24e5ca920/docs/BETA_CONTRIBUTOR_GUIDE.md); [`docs/BETA_STATUS.md`](https://github.com/mininet-labs/Mininet/blob/53a7e3b9594f9274adaf853820c7c3e24e5ca920/docs/BETA_STATUS.md). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 2 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0267"></a>

## PR #267: feat: add Forge-native contributor coordination (D-0407)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-03, FD-08, FD-12, FD-16, FD-18.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/267) | [Files changed](https://github.com/mininet-labs/Mininet/pull/267/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/fa459c951557ba9a5da92fc21dbfd9cc795c2fd1)

Head `fa459c951557ba9a5da92fc21dbfd9cc795c2fd1`; base `bf3adf6e96fa3b7e4af096a9949155ea41a62603`; merge `5ad62d0b28085ab3b53388e595ab041bbdc54ece`. 14 changed files; 3 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Implements Forge-native coordination as signed task/charter/claim/handoff objects, reducing dependence on a hosted issue tracker without confusing work organization with governance ownership.

**Mechanism and evidence.** mini team/task CLI commands create and inspect bounded objects, apply expiry and verified-author filtering, and provide deterministic local suggestions. Contributor-selected claims and technical-review handoffs remain distinct from assignment and approval.

**What remains weaker than the intended claim.** A claim does not reserve political authority or prove the claimant is a unique human. Charter-to-Policy delegation and team activation are absent. Signatures authenticate the author's statement, not competence or completion. Local suggestions can become covert ranking if later tied to money or opaque scoring.

**Recommended improvement and rationale.** Keep claims advisory and expiring, publish suggestion criteria and preserve alternate task discovery. Implement delegation only through a separate exact-state governance decision with scope/revocation, never by interpreting a coordination object as authority. Add durable idempotent CLI state and bounded query costs.

**Concrete example.** A contributor claims a task and then disappears. Other contributors must still be free to work, fork or submit a competing solution; the claim cannot become an indefinite exclusive right or a payment entitlement.

**Acceptance tests to implement.** Test expired/conflicting claims, forged authors, identical labels from different roots, restart/idempotence and unauthorized charter-to-policy conversion. Assert that a review handoff never satisfies the approval quorum.

**History, supersession and integration.** D-0407's accepted/merged status is later corrected append-only in #288. Builds on #265 and the Forge spine; it is not the native governance cutover.

**Source entry points.** [`crates/mini-cli/src/cli.rs`](https://github.com/mininet-labs/Mininet/blob/fa459c951557ba9a5da92fc21dbfd9cc795c2fd1/crates/mini-cli/src/cli.rs); [`crates/mini-cli/src/coordination.rs`](https://github.com/mininet-labs/Mininet/blob/fa459c951557ba9a5da92fc21dbfd9cc795c2fd1/crates/mini-cli/src/coordination.rs); [`crates/mini-cli/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/fa459c951557ba9a5da92fc21dbfd9cc795c2fd1/crates/mini-cli/src/lib.rs); [`crates/mini-cli/tests/coordination_commands.rs`](https://github.com/mininet-labs/Mininet/blob/fa459c951557ba9a5da92fc21dbfd9cc795c2fd1/crates/mini-cli/tests/coordination_commands.rs); [`crates/mini-forge/src/coordination.rs`](https://github.com/mininet-labs/Mininet/blob/fa459c951557ba9a5da92fc21dbfd9cc795c2fd1/crates/mini-forge/src/coordination.rs); [`crates/mini-forge/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/fa459c951557ba9a5da92fc21dbfd9cc795c2fd1/crates/mini-forge/src/lib.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0269"></a>

## PR #269: Batch 5: native exact release retrieval

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-03, FD-06, FD-11, FD-14.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/269) | [Files changed](https://github.com/mininet-labs/Mininet/pull/269/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/571d7c105c902372e21c1e927b92ed879bafa3da)

Head `571d7c105c902372e21c1e927b92ed879bafa3da`; base `5ad62d0b28085ab3b53388e595ab041bbdc54ece`; merge `778f3e684a4e978379e3b56d1d0efe2244e2bf14`. 16 changed files; 3 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Adds exact release retrieval without reconciling an entire unrelated object store, a practical step toward a software lifecycle that does not require GitHub or a package-hosting authority.

**Mechanism and evidence.** Release closure selection follows bounded forward content/artifact links and selected reverse release/previous/PR evidence. The transport echoes an exact selection over the encrypted bearer, reuses verified ingestion, then independently verifies the release before writing a new output path.

**What remains weaker than the intended claim.** A serving peer remains an availability source, not release authority. Local KEL trust/freshness is still explicit, single-frame retrieval limits scale, and no automatic discovery/retry/daemon exists. A complete local closure is not proof that newer releases or contradictory governance evidence were not withheld.

**Recommended improvement and rationale.** Add resumable bounded multi-peer retrieval while preserving exact release IDs and independent verification. Separate unknown, unavailable, incomplete and invalid results; never select legitimacy by download count. Verify every output path through race-resistant filesystem handles.

**Concrete example.** One peer supplies a valid old release and omits its successor. The client can verify the chosen artifact but must not report that it is the latest canonical release without appropriate freshness/continuity evidence.

**Acceptance tests to implement.** Test missing evidence, swapped selection, malicious reverse links, oversize closure, dropped connection, stale KEL, path substitution and repeated retrieval into an existing directory. Demonstrate retrieval from independent non-GitHub peers.

**History, supersession and integration.** Extends #104/#110/#115 release workflow; #270 adds remote builds. Activation remains the separate explicit-owner #105/#171 installer path, not a consequence of retrieval.

**Source entry points.** [`crates/mini-cli/src/cli.rs`](https://github.com/mininet-labs/Mininet/blob/571d7c105c902372e21c1e927b92ed879bafa3da/crates/mini-cli/src/cli.rs); [`crates/mini-cli/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/571d7c105c902372e21c1e927b92ed879bafa3da/crates/mini-cli/src/lib.rs); [`crates/mini-cli/src/release.rs`](https://github.com/mininet-labs/Mininet/blob/571d7c105c902372e21c1e927b92ed879bafa3da/crates/mini-cli/src/release.rs); [`crates/mini-cli/src/sync.rs`](https://github.com/mininet-labs/Mininet/blob/571d7c105c902372e21c1e927b92ed879bafa3da/crates/mini-cli/src/sync.rs); [`crates/mini-cli/tests/network_sync_release.rs`](https://github.com/mininet-labs/Mininet/blob/571d7c105c902372e21c1e927b92ed879bafa3da/crates/mini-cli/tests/network_sync_release.rs); [`crates/mini-forge/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/571d7c105c902372e21c1e927b92ed879bafa3da/crates/mini-forge/src/lib.rs); [`crates/mini-forge/src/retrieval.rs`](https://github.com/mininet-labs/Mininet/blob/571d7c105c902372e21c1e927b92ed879bafa3da/crates/mini-forge/src/retrieval.rs); [`crates/mini-sync/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/571d7c105c902372e21c1e927b92ed879bafa3da/crates/mini-sync/src/lib.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0270"></a>

## PR #270: Batch 5: bounded distributed build workers

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-03, FD-06, FD-09, FD-11, FD-14.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/270) | [Files changed](https://github.com/mininet-labs/Mininet/pull/270/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/67428f4c6244c0670ed2b5aa0f96cb5eb1d2052f)

Head `67428f4c6244c0670ed2b5aa0f96cb5eb1d2052f`; base `778f3e684a4e978379e3b56d1d0efe2244e2bf14`; merge `9b40750e7183a6ebaffa6ae53be6dc95ad35b4b8`. 15 changed files; 1 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Makes a second machine execute an exact bounded build job over Mininet transport, removing another reason to rely on a hosted CI operator. The worker remains untrusted rather than becoming a release authority.

**Mechanism and evidence.** Canonical request/response types bind job and artifact digests, capabilities, isolation label and output bounds. The CLI dispatches/serves a one-shot job and keeps Wasmtime in the existing subprocess runner. Output files are checked before writing.

**What remains weaker than the intended claim.** A matching response digest proves returned bytes match the claim, not that the worker performed an honest build. Anonymous channel encryption alone does not identify the intended worker. One-frame one-shot transport lacks scheduling, retry/resume and endpoint authentication; the sandbox inherits the exact Wasmtime version's security.

**Recommended improvement and rationale.** Bind responses to exact source/recipe/toolchain and authenticated optional worker identity, then require independently controlled reproducible builders before release. Add cancellation, total-resource budgets and no ambient secrets/network to the worker. Do not introduce a canonical worker registry or payment-derived review weight.

**Concrete example.** A malicious worker returns a precomputed binary and a truthful hash of that binary. Transport integrity is satisfied; only reproduction and release verification can establish that it corresponds to the reviewed source.

**Acceptance tests to implement.** Test altered job IDs, path traversal/symlinks, omitted/extra artifacts, forged isolation claims, blocked pipes, resource exhaustion and sandbox escape regressions. Run independent builders under different administration and compare exact output digests.

**History, supersession and integration.** Composes #103 sandbox runner and #269 retrieval; #277/#307 later patch the runtime dependency. It does not authorize releases, provenance quorum or owner adoption.

**Source entry points.** [`crates/mini-cli/src/build.rs`](https://github.com/mininet-labs/Mininet/blob/67428f4c6244c0670ed2b5aa0f96cb5eb1d2052f/crates/mini-cli/src/build.rs); [`crates/mini-cli/src/cli.rs`](https://github.com/mininet-labs/Mininet/blob/67428f4c6244c0670ed2b5aa0f96cb5eb1d2052f/crates/mini-cli/src/cli.rs); [`crates/mini-cli/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/67428f4c6244c0670ed2b5aa0f96cb5eb1d2052f/crates/mini-cli/src/lib.rs); [`crates/mini-cli/tests/network_build_workers.rs`](https://github.com/mininet-labs/Mininet/blob/67428f4c6244c0670ed2b5aa0f96cb5eb1d2052f/crates/mini-cli/tests/network_build_workers.rs); [`crates/mini-pipeline-protocol/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/67428f4c6244c0670ed2b5aa0f96cb5eb1d2052f/crates/mini-pipeline-protocol/src/lib.rs); [`crates/mini-pipeline-protocol/src/remote.rs`](https://github.com/mininet-labs/Mininet/blob/67428f4c6244c0670ed2b5aa0f96cb5eb1d2052f/crates/mini-pipeline-protocol/src/remote.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 2 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0271"></a>

## PR #271: Add Day-0 MINI monetary kernel and simulation

**PARTIAL** | Captured outcome: **merged** | Directives: FD-04, FD-05, FD-08, FD-11, FD-13, FD-16, FD-17.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/271) | [Files changed](https://github.com/mininet-labs/Mininet/pull/271/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/8a3573c0a5f1b2e0596c642a8a0f1bfb28ec9050)

Head `8a3573c0a5f1b2e0596c642a8a0f1bfb28ec9050`; base `9b40750e7183a6ebaffa6ae53be6dc95ad35b4b8`; merge `afbe8d0d1d79e4e9eab8f4f257105172e91e878e`. 25 changed files; 1 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Implements the accepted issuance envelope as checked integer arithmetic and corrects the old simulation's balance-proportional approximation of an equal-per-human share. It makes monetary assumptions explicit enough to falsify.

**Mechanism and evidence.** mini-economy uses u128 accounting, annual/epoch caps, vesting metadata, unused-capacity expiry and a constant-space snapshot-root/count Human Share plan. mini-econ-sim models cohorts over two centuries, including adoption, dormancy, concentration and assumed Sybils.

**What remains weaker than the intended claim.** The model assumes the truth of personhood counts, supply and service/treasury evidence; it cannot establish them. Long-run simulation is not a guarantee of price, social legitimacy or fair access. A small fixed test suite does not cover every rounding/overflow and adversarial economic trajectory.

**Recommended improvement and rationale.** Use an independent arbitrary-precision reference model and property tests over epoch lengths, maximum supplies and rounding residues. Pre-register economic assumptions and attack budgets; publish sensitivity rather than a single optimistic trajectory. Keep genesis and human membership activation separately governed.

**Concrete example.** An attacker doubles the number of accepted fake humans. Equal-per-credential issuance remains arithmetically correct while the real humans' distribution is captured. The simulation should display that loss, not label the policy Sybil-resistant.

**Acceptance tests to implement.** Differential-test integer plans against a rational reference, extreme epoch partitions, zero/maximum population and repeated unused capacity. Compare corrected equal-human results with the historical #108 model and retain why its earlier conclusions changed.

**History, supersession and integration.** #272 binds plans to finalized supply/vesting, #273 adds balances and #300 repairs exact-body commitment/wire omissions. External mechanism, personhood and custody review remain release gates.

**Source entry points.** [`crates/mini-econ-sim/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/8a3573c0a5f1b2e0596c642a8a0f1bfb28ec9050/crates/mini-econ-sim/src/lib.rs); [`crates/mini-econ-sim/src/main.rs`](https://github.com/mininet-labs/Mininet/blob/8a3573c0a5f1b2e0596c642a8a0f1bfb28ec9050/crates/mini-econ-sim/src/main.rs); [`crates/mini-econ-sim/tests/simulation.rs`](https://github.com/mininet-labs/Mininet/blob/8a3573c0a5f1b2e0596c642a8a0f1bfb28ec9050/crates/mini-econ-sim/tests/simulation.rs); [`crates/mini-economy/src/amount.rs`](https://github.com/mininet-labs/Mininet/blob/8a3573c0a5f1b2e0596c642a8a0f1bfb28ec9050/crates/mini-economy/src/amount.rs); [`crates/mini-economy/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/8a3573c0a5f1b2e0596c642a8a0f1bfb28ec9050/crates/mini-economy/src/error.rs); [`crates/mini-economy/src/genesis.rs`](https://github.com/mininet-labs/Mininet/blob/8a3573c0a5f1b2e0596c642a8a0f1bfb28ec9050/crates/mini-economy/src/genesis.rs); [`crates/mini-economy/src/issuance.rs`](https://github.com/mininet-labs/Mininet/blob/8a3573c0a5f1b2e0596c642a8a0f1bfb28ec9050/crates/mini-economy/src/issuance.rs); [`crates/mini-economy/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/8a3573c0a5f1b2e0596c642a8a0f1bfb28ec9050/crates/mini-economy/src/lib.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 1 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0272"></a>

## PR #272: Bind MINI issuance and vesting to finalized state

**PARTIAL** | Captured outcome: **merged** | Directives: FD-04, FD-05, FD-06, FD-09, FD-13, FD-16.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/272) | [Files changed](https://github.com/mininet-labs/Mininet/pull/272/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/179c68758a24dbb61279a1e5d58fbf43a9f56660)

Head `179c68758a24dbb61279a1e5d58fbf43a9f56660`; base `afbe8d0d1d79e4e9eab8f4f257105172e91e878e`; merge `66dff1071cca40bc0a459ff5cad8eb5a106ce6cd`. 23 changed files; 1 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Connects monetary plans to finalized execution, so a policy calculation cannot become spendable issuance merely because a proposer presents it. It also avoids publishing a per-person allocation list for aggregate Human Share accounting.

**Mechanism and evidence.** Execution reconstructs the plan from committed opening supply and policy inputs, enforces sequential epochs and at most one plan per block, and advances deterministic cumulative policy-time vesting with checked supply identities. Snapshot roots/counts represent aggregate membership inputs.

**What remains weaker than the intended claim.** Correct accounting does not authenticate the human snapshot or service/treasury evidence. Policy-time determinism also needs a real-world cadence/activation model; chain progress alone can diverge from calendar time. This stage does not yet enforce recipient balances or private claims.

**Recommended improvement and rationale.** Bind each external evidence input to its independently verifiable authorization and define epoch progression independently of proposer wall-clock claims. Add private claim/nullifier rules before distributions are redeemable; preserve deterministic reconstruction and bounded state.

**Concrete example.** A proposer supplies a structurally valid snapshot root naming a fabricated population. Reconstructing its arithmetic should succeed only as bookkeeping, not authorize issuance until snapshot legitimacy is separately proven.

**Acceptance tests to implement.** Test replay/skipped epochs, stale supply, changed nested vesting fields, extreme u128 arithmetic, forged snapshot provenance and restart/state-sync parity. Verify all monetary fields survive wire encoding and affect the finalized body commitment.

**History, supersession and integration.** Builds on #271; #273 adds debit/credit, while #300 later fixes lossless monetary wire and exact-body finality. Those historical gaps must not be hidden by the current stronger implementation.

**Source entry points.** [`crates/mini-economy/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/179c68758a24dbb61279a1e5d58fbf43a9f56660/crates/mini-economy/src/error.rs); [`crates/mini-economy/src/issuance.rs`](https://github.com/mininet-labs/Mininet/blob/179c68758a24dbb61279a1e5d58fbf43a9f56660/crates/mini-economy/src/issuance.rs); [`crates/mini-economy/src/ledger.rs`](https://github.com/mininet-labs/Mininet/blob/179c68758a24dbb61279a1e5d58fbf43a9f56660/crates/mini-economy/src/ledger.rs); [`crates/mini-economy/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/179c68758a24dbb61279a1e5d58fbf43a9f56660/crates/mini-economy/src/lib.rs); [`crates/mini-economy/tests/economy.rs`](https://github.com/mininet-labs/Mininet/blob/179c68758a24dbb61279a1e5d58fbf43a9f56660/crates/mini-economy/tests/economy.rs); [`crates/mini-execution/src/body.rs`](https://github.com/mininet-labs/Mininet/blob/179c68758a24dbb61279a1e5d58fbf43a9f56660/crates/mini-execution/src/body.rs); [`crates/mini-execution/src/chain.rs`](https://github.com/mininet-labs/Mininet/blob/179c68758a24dbb61279a1e5d58fbf43a9f56660/crates/mini-execution/src/chain.rs); [`crates/mini-execution/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/179c68758a24dbb61279a1e5d58fbf43a9f56660/crates/mini-execution/src/error.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 1 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0273"></a>

## PR #273: Enforce Day-0 MINI balances and canonical payment outcomes

**PARTIAL** | Captured outcome: **merged** | Directives: FD-04, FD-05, FD-06, FD-09, FD-16.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/273) | [Files changed](https://github.com/mininet-labs/Mininet/pull/273/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/e92883ec3c91e96d1ca20ab0aafc85b7fd28113d)

Head `e92883ec3c91e96d1ca20ab0aafc85b7fd28113d`; base `66dff1071cca40bc0a459ff5cad8eb5a106ce6cd`; merge `9d7ff8fab5e4b75e6f8c283205b054e9a637a6f7`. 24 changed files; 2 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Adds real transparent balance conservation and canonical payment outcomes, turning settlement ordering into checked ownership accounting for the beta path.

**Mechanism and evidence.** Finalized execution debits/credits with checked arithmetic, validates network-bound signatures, rejects insufficient funds/unsupported payees/stale sequence and commits balance/rejection state. Invalid signatures are not recorded as canonical rejections against an otherwise valid claim digest.

**What remains weaker than the intended claim.** The ledger publicly exposes stable payer/payee, amount, sequence and balance relationships, conflicting with the ultimate structural privacy goal. A safe transparent prototype is not private production money. Genesis allocation and supply/evidence authorization remain external trust boundaries; rejection growth and retry semantics need bounds.

**Recommended improvement and rationale.** Keep the transparent path explicitly beta-only and retire it through a governed migration once consensus-verifiable shielded claims exist. Preserve precise canonical rejection reasons and retry policy; never use malformed signatures to poison an honest digest.

**Concrete example.** A copied legitimate claim with a forged signature must not reserve its digest as rejected before the real signature arrives. Conversely, two valid payments whose combined debit exceeds finalized balance cannot both succeed.

**Acceptance tests to implement.** Test cross-network replay, forged-signature poisoning, aggregate overspend, credit overflow, stale sequence, rejection persistence and wallet label parity. Verify the same finalized block sequence produces identical supply and balances after restart.

**History, supersession and integration.** Builds on #271/#272; #274 adds admission, #300 narrows retry behavior, and #305/#312/#313 form the still-incomplete private replacement. R3 retirement is not complete at the reviewed revision.

**Source entry points.** [`crates/mini-consensus/src/wire.rs`](https://github.com/mininet-labs/Mininet/blob/e92883ec3c91e96d1ca20ab0aafc85b7fd28113d/crates/mini-consensus/src/wire.rs); [`crates/mini-engagement/src/settlement.rs`](https://github.com/mininet-labs/Mininet/blob/e92883ec3c91e96d1ca20ab0aafc85b7fd28113d/crates/mini-engagement/src/settlement.rs); [`crates/mini-execution/src/chain.rs`](https://github.com/mininet-labs/Mininet/blob/e92883ec3c91e96d1ca20ab0aafc85b7fd28113d/crates/mini-execution/src/chain.rs); [`crates/mini-execution/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/e92883ec3c91e96d1ca20ab0aafc85b7fd28113d/crates/mini-execution/src/error.rs); [`crates/mini-execution/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/e92883ec3c91e96d1ca20ab0aafc85b7fd28113d/crates/mini-execution/src/lib.rs); [`crates/mini-execution/src/state.rs`](https://github.com/mininet-labs/Mininet/blob/e92883ec3c91e96d1ca20ab0aafc85b7fd28113d/crates/mini-execution/src/state.rs); [`crates/mini-execution/tests/end_to_end.rs`](https://github.com/mininet-labs/Mininet/blob/e92883ec3c91e96d1ca20ab0aafc85b7fd28113d/crates/mini-execution/tests/end_to_end.rs); [`crates/mini-settlement/src/claim.rs`](https://github.com/mininet-labs/Mininet/blob/e92883ec3c91e96d1ca20ab0aafc85b7fd28113d/crates/mini-settlement/src/claim.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 1 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0274"></a>

## PR #274: Bound Day-0 payment submission and admission

**PARTIAL** | Captured outcome: **merged** | Directives: FD-04, FD-05, FD-06, FD-09, FD-11, FD-15.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/274) | [Files changed](https://github.com/mininet-labs/Mininet/pull/274/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/7c8a0079aa4539801669136e8fd57e35f36442f2)

Head `7c8a0079aa4539801669136e8fd57e35f36442f2`; base `9d7ff8fab5e4b75e6f8c283205b054e9a637a6f7`; merge `3d809c734e15577f9d96a874a338fc05dc980400`. 18 changed files; 1 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Adds bounded payment submission and local admission before network traffic can become an unbounded queue. It protects weak nodes while keeping local acceptance distinct from canonical ownership.

**Mechanism and evidence.** PaymentClaim gets a standalone domain/versioned bounded codec. PaymentAdmissionPool validates signature/network/payee/expiry/outcome, limits total/per-payer claims and bytes, reserves aggregate pending spend locally and sorts candidates deterministically.

**What remains weaker than the intended claim.** A per-payer cap is not Sybil resistance when attackers create many keys. Local reservations do not canonically lock balances, and deterministic ordering can be ground or censored. No authenticated submission service, relay privacy or fee policy is provided. Exact byte accounting and post-finality cleanup are security-critical.

**Recommended improvement and rationale.** Add authenticated/rate-bounded network ingress with per-connection/global budgets that do not demand wealth for basic speech. Keep payment anti-spam distinct from public participation. Revalidate pending claims after finality and preserve original accepted wire size for exact removal accounting.

**Concrete example.** An attacker submits thousands of distinct payer keys, each below the per-payer limit. The global memory/work cap must still bound admission, and rejected malformed claims must not reserve funds or consume permanent replay state.

**Acceptance tests to implement.** Test malformed lengths at every boundary, duplicate/conflicting claims, aggregate overspend, Sybil-shaped floods, expiry/clock rollback, removal accounting and arrival-independent order. Measure CPU per rejected request as well as heap capacity.

**History, supersession and integration.** Extends #273; #300 fixes exact admission accounting and canonical retry handling. The private path needs its own bounded admission model, not reuse of public payer counters as anonymity-breaking rate tags.

**Source entry points.** [`crates/mini-execution/src/admission.rs`](https://github.com/mininet-labs/Mininet/blob/7c8a0079aa4539801669136e8fd57e35f36442f2/crates/mini-execution/src/admission.rs); [`crates/mini-execution/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/7c8a0079aa4539801669136e8fd57e35f36442f2/crates/mini-execution/src/lib.rs); [`crates/mini-execution/src/state.rs`](https://github.com/mininet-labs/Mininet/blob/7c8a0079aa4539801669136e8fd57e35f36442f2/crates/mini-execution/src/state.rs); [`crates/mini-settlement/src/claim.rs`](https://github.com/mininet-labs/Mininet/blob/7c8a0079aa4539801669136e8fd57e35f36442f2/crates/mini-settlement/src/claim.rs); [`crates/mini-settlement/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/7c8a0079aa4539801669136e8fd57e35f36442f2/crates/mini-settlement/src/error.rs); [`crates/mini-settlement/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/7c8a0079aa4539801669136e8fd57e35f36442f2/crates/mini-settlement/src/lib.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 1 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0275"></a>

## PR #275: Contribution and Settlement Coordinator: doctrine + vertical slice 1 (D-0417)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-04, FD-05, FD-09, FD-14, FD-16, FD-18.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/275) | [Files changed](https://github.com/mininet-labs/Mininet/pull/275/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/9b3d0b88456002fa56586a00c52b0cff49739b85)

Head `9b3d0b88456002fa56586a00c52b0cff49739b85`; base `3d809c734e15577f9d96a874a338fc05dc980400`; merge `053fec5aa9b3b7a7870769d8ccdcc5298923cc9e`. 14 changed files; 6 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Connects content publication, seeding, delivery evidence and creator/seeder payments into a concrete three-party lifecycle. It makes the participant-owned economic idea more than disconnected reward primitives.

**Mechanism and evidence.** The leaf mini-contribution crate binds verified delivery evidence to engagements and splits a requester-funded amount deterministically. The Alice/Bob/Carol integration uses real store objects and quorum-finalized PaymentClaims for creator and seeder, with no new issuance.

**What remains weaker than the intended claim.** An honest signed delivery can be collusive; requester-funded self-payment creates volume, not net protocol extraction. The transparent payment path leaks the relationship that the storage view path tries not to record. Canonical payment does not prove service usefulness or refund rights.

**Recommended improvement and rationale.** Preserve the requester-funded baseline without adding personhood/issuer permission to voluntary transfers. Move payouts to the reviewed private path, bind every split to an immutable agreed offer and define retries as one atomic economic intent. Keep any subsidy mechanism separate and finitely budgeted.

**Concrete example.** Bob controls requester and seeder and pays himself from his own balance: no mint attack occurs. The same receipt becomes a drain only when a sponsor or protocol subsidy pays it; those cases require different defenses.

**Acceptance tests to implement.** Test rounding, zero/full shares, swapped content/engagement evidence, partial creator/seeder finality, repeated retries, insufficient funds and privacy across publish/view/pay. A failed leg must not silently report the whole contribution settled.

**History, supersession and integration.** Builds on #219/#237 and #273; #278 initially raises collusion concerns, refined correctly by #284/#285. #305/#312 provide private-payment ingredients but do not migrate this consumer.

**Source entry points.** [`crates/mini-contribution/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/9b3d0b88456002fa56586a00c52b0cff49739b85/crates/mini-contribution/src/error.rs); [`crates/mini-contribution/src/evidence.rs`](https://github.com/mininet-labs/Mininet/blob/9b3d0b88456002fa56586a00c52b0cff49739b85/crates/mini-contribution/src/evidence.rs); [`crates/mini-contribution/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/9b3d0b88456002fa56586a00c52b0cff49739b85/crates/mini-contribution/src/lib.rs); [`crates/mini-contribution/src/offer.rs`](https://github.com/mininet-labs/Mininet/blob/9b3d0b88456002fa56586a00c52b0cff49739b85/crates/mini-contribution/src/offer.rs); [`crates/mini-contribution/src/role.rs`](https://github.com/mininet-labs/Mininet/blob/9b3d0b88456002fa56586a00c52b0cff49739b85/crates/mini-contribution/src/role.rs); [`crates/mini-contribution/src/settle.rs`](https://github.com/mininet-labs/Mininet/blob/9b3d0b88456002fa56586a00c52b0cff49739b85/crates/mini-contribution/src/settle.rs); [`crates/mini-contribution/src/split.rs`](https://github.com/mininet-labs/Mininet/blob/9b3d0b88456002fa56586a00c52b0cff49739b85/crates/mini-contribution/src/split.rs); [`crates/mini-contribution/tests/alice_bob_carol.rs`](https://github.com/mininet-labs/Mininet/blob/9b3d0b88456002fa56586a00c52b0cff49739b85/crates/mini-contribution/tests/alice_bob_carol.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0276"></a>

## PR #276: mini-forge: git SHA-256 import bridge (D-0418)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-03, FD-05, FD-07, FD-14.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/276) | [Files changed](https://github.com/mininet-labs/Mininet/pull/276/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/2cbaab874fe46dcdfd5c5f59c980ca445f09d495)

Head `2cbaab874fe46dcdfd5c5f59c980ca445f09d495`; base `053fec5aa9b3b7a7870769d8ccdcc5298923cc9e`; merge `cd594e5274654e3591fda9f95e54ec846eb5418b`. 8 changed files; 2 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Adds a truthful import direction for Forge history: imported content is signed by the importer, while original Git authorship is retained as provenance rather than forged as a Mininet signature.

**Mechanism and evidence.** The bridge verifies framed SHA-256 Git object IDs, accepts a narrow canonical tree/commit grammar and reconstructs native Forge blobs/trees/commits. Original Git metadata lives in a separately linked provenance object. Unsupported headers, executable modes, symlinks and submodules are rejected.

**What remains weaker than the intended claim.** The restricted grammar cannot import ordinary GitHub SHA-1 repositories or many real development trees. Byte-preserved blobs do not imply preserved commit identities or authorship legitimacy. Re-signing by an importer proves the import statement, not historical author endorsement.

**Recommended improvement and rationale.** Define an explicit interoperable import profile and a non-authorizing treatment of legacy identifiers consistent with the existing SHA-1 prohibition, requiring a human policy decision rather than bypassing it. Preserve executable modes and other necessary metadata only through a reviewed representation that remains safe at checkout.

**Concrete example.** A signed Git commit with an executable build script must not be silently imported as an ordinary file after dropping its signature/header or mode. Either preserve the distinction in a supported profile or reject with an exact explanation.

**Acceptance tests to implement.** Test forged object IDs, malformed lengths, duplicate paths, multiple parents, unsupported modes/headers and byte-for-byte export/import properties. Verify importer identity never appears as proof that the original author approved a native governance action.

**History, supersession and integration.** Extends the earlier Git export spine; #278 explicitly finds the real-GitHub SHA-1 compatibility blocker. It is not yet full GitHub mirror automation or native Forge cutover.

**Source entry points.** [`crates/mini-forge/src/git_export.rs`](https://github.com/mininet-labs/Mininet/blob/2cbaab874fe46dcdfd5c5f59c980ca445f09d495/crates/mini-forge/src/git_export.rs); [`crates/mini-forge/src/git_import.rs`](https://github.com/mininet-labs/Mininet/blob/2cbaab874fe46dcdfd5c5f59c980ca445f09d495/crates/mini-forge/src/git_import.rs); [`crates/mini-forge/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/2cbaab874fe46dcdfd5c5f59c980ca445f09d495/crates/mini-forge/src/lib.rs); [`crates/mini-forge/tests/git_import.rs`](https://github.com/mininet-labs/Mininet/blob/2cbaab874fe46dcdfd5c5f59c980ca445f09d495/crates/mini-forge/tests/git_import.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0277"></a>

## PR #277: mini-build-runner-wasmtime: security bump wasmtime 46.0.1 -&gt; 46.0.2

**PARTIAL** | Captured outcome: **merged** | Directives: FD-06, FD-10, FD-14.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/277) | [Files changed](https://github.com/mininet-labs/Mininet/pull/277/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/e5da777c209f95017f392e7f4b71ba7527b91fdc)

Head `e5da777c209f95017f392e7f4b71ba7527b91fdc`; base `cd594e5274654e3591fda9f95e54ec846eb5418b`; merge `d74c7e62f26c55022c3225d7c3ec0f77b44ae4d6`. 2 changed files; 1 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Updates the exact-pinned build sandbox runtime in response to reported security advisories and reruns the existing adversarial suite. This is necessary maintenance of a critical boundary, not a routine cosmetic dependency bump.

**Mechanism and evidence.** wasmtime/wasmtime-wasi move from 46.0.1 to 46.0.2 within the declared patch line; the repository records advisory identifiers and reports the twelve sandbox criteria and dependency-deny check passing. Capability policy remains unchanged.

**What remains weaker than the intended claim.** This review treats those advisory/version claims as repository history, not independently certified vulnerability coverage. The suite may not reproduce the exact advisory exploit, and a passing patch-level update does not establish that all future sandbox escapes are impossible. A fixed pin still requires active monitoring.

**Recommended improvement and rationale.** Archive primary advisory text and exact fixed ranges, add a regression matching each relevant exploit class and record whether untrusted jobs ran while vulnerable. Keep sandbox runtime updates isolated, reproducible and independently reviewable, with a safe stop policy if no fix is available.

**Concrete example.** If an engine bug can cross capability boundaries, a build job that requests no filesystem access can still be dangerous. The runtime version is part of the security contract and must be recorded in build evidence.

**Acceptance tests to implement.** Rerun the complete sandbox suite, advisory-specific reproducers and independently rebuilt artifacts at the exact new lockfile. Test that an unresolved critical runtime advisory prevents untrusted job execution rather than merely printing a warning.

**History, supersession and integration.** Follows #103 and is inherited by #270 remote workers. #307 later records another runtime update; historical patch success must not be called current blanket sandbox safety.

**Source entry points.** [`crates/mini-build-runner-wasmtime/Cargo.toml`](https://github.com/mininet-labs/Mininet/blob/e5da777c209f95017f392e7f4b71ba7527b91fdc/crates/mini-build-runner-wasmtime/Cargo.toml). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0278"></a>

## PR #278: mini-media (D-0419) + mini-query Track E7/E8 (D-0420) + crypto architecture doctrine (D-0421)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-03, FD-06, FD-09, FD-11, FD-14, FD-16.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/278) | [Files changed](https://github.com/mininet-labs/Mininet/pull/278/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/db76cbade3aca2d931d0dc3db8e1ddd8175de1db)

Head `db76cbade3aca2d931d0dc3db8e1ddd8175de1db`; base `d74c7e62f26c55022c3225d7c3ec0f77b44ae4d6`; merge `6fa7d9525361e624a3e618ecd25bee2e3d6b7e02`. 18 changed files; 3 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Combines three related advances: larger media via nested manifests, user-query/provenance composition for MiniSearch, and an explicit cryptographic strategy favoring established primitives over new inventions.

**Mechanism and evidence.** Superblock composes existing manifests and verifies both part and whole digests with one nesting level. mini-query parses fixed operators and applies filters through ranker availability without changing scoring. The doctrine maps existing research tracks and names anti-collusion useful-contribution work.

**What remains weaker than the intended claim.** A 64-GiB addressable payload is not a 64-GiB-in-memory capability on a phone; assembly needs streaming bounds. Silently dropping malformed filters can broaden a privacy-sensitive query unexpectedly. The doctrine's initial collusion framing must distinguish requester-funded volume from protocol extraction, clarified later in #284.

**Recommended improvement and rationale.** Stream media verification and expose resumable missing-part state. Return parsed/ignored filter diagnostics and require explicit confirmation where a malformed privacy-sensitive filter broadens scope. Use established crypto comparisons as research proposals with license/compatibility review, not automatic authority to vendor or activate them.

**Concrete example.** If before:2026-99-99 is ignored, a user may search all dates while believing the date filter applied. A parse warning should make the actual query visible. For media, verify each part while writing to bounded storage rather than concatenate tens of gigabytes.

**Acceptance tests to implement.** Test forged whole digests, missing/reordered parts, malicious nesting, bounded memory, malformed filters, language/date edges and provenance mismatch. Validate the doctrine creates no new primitive, issuer authority or subsidy authorization.

**History, supersession and integration.** Builds on #256/#257 and #276's compatibility lessons; #281-#294 extend federation, #284/#285 refine anti-collusion, and #305-#313 compose existing private-value primitives.

**Source entry points.** [`crates/mini-media/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/db76cbade3aca2d931d0dc3db8e1ddd8175de1db/crates/mini-media/src/lib.rs); [`crates/mini-media/src/superblock.rs`](https://github.com/mininet-labs/Mininet/blob/db76cbade3aca2d931d0dc3db8e1ddd8175de1db/crates/mini-media/src/superblock.rs); [`crates/mini-media/tests/superblock.rs`](https://github.com/mininet-labs/Mininet/blob/db76cbade3aca2d931d0dc3db8e1ddd8175de1db/crates/mini-media/tests/superblock.rs); [`crates/mini-query/src/context.rs`](https://github.com/mininet-labs/Mininet/blob/db76cbade3aca2d931d0dc3db8e1ddd8175de1db/crates/mini-query/src/context.rs); [`crates/mini-query/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/db76cbade3aca2d931d0dc3db8e1ddd8175de1db/crates/mini-query/src/error.rs); [`crates/mini-query/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/db76cbade3aca2d931d0dc3db8e1ddd8175de1db/crates/mini-query/src/lib.rs); [`crates/mini-query/src/parse.rs`](https://github.com/mininet-labs/Mininet/blob/db76cbade3aca2d931d0dc3db8e1ddd8175de1db/crates/mini-query/src/parse.rs); [`crates/mini-query/src/search.rs`](https://github.com/mininet-labs/Mininet/blob/db76cbade3aca2d931d0dc3db8e1ddd8175de1db/crates/mini-query/src/search.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 2 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0280"></a>

## PR #280: feat(mini-index-exchange): publish + verify content-addressed index segments, Track F2 (D-0422, closes #279)

**PARTIAL** | Captured outcome: **closed** | Directives: FD-02, FD-05, FD-07, FD-14, FD-16.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/280) | [Files changed](https://github.com/mininet-labs/Mininet/pull/280/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/4aa5502973f155d73212948611a201fa2789a3db)

Head `4aa5502973f155d73212948611a201fa2789a3db`; base `6fa7d9525361e624a3e618ecd25bee2e3d6b7e02`; merge `none`. 15 changed files; 1 commits; 1 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This closed-unmerged alternative attempted signed index-segment exchange with both publisher authentication and content-address verification. Its substantive contribution is an independently designed boundary to compare with the implementation that won, not shipped functionality.

**Mechanism and evidence.** SegmentPublication signs a manifest, derives a provider pseudonym from its key and verifies the supplied segment digest and shape. accept_published_segment combines untrusted-byte decoding with both checks. The closure comment identifies #281 as the overlapping merged implementation.

**What remains weaker than the intended claim.** The proposal used the same D-0422 identifier as concurrent work and would have added a duplicate crate and wire surface. A publisher pseudonym derived from a bare key has different rotation/delegation semantics from did-mini provenance. Neither signature scheme proves completeness or relevance of an index.

**Recommended improvement and rationale.** Preserve the useful two-leg acceptance tests in the chosen exchange implementation and document why this alternative was closed. Reserve decision identifiers atomically or use collision-resistant proposal IDs before assigning permanent sequential numbers.

**Concrete example.** A correctly signed manifest with a mismatching segment body must fail, and an unsigned correct body must not be attributed to the claimed provider. Compare those exact properties across the two proposals rather than merging both APIs.

**Acceptance tests to implement.** Test forged signature, correct digest/wrong shape, trailing bytes, key rotation and two providers publishing the same segment. Verify the closed branch created no production dependency or misleading done status.

**History, supersession and integration.** Closed in favor of #281, not merged. Its scope and D-number collision are part of project history and should remain visible rather than skipped because the code did not land.

**Source entry points.** [`crates/mini-index-exchange/src/codec.rs`](https://github.com/mininet-labs/Mininet/blob/4aa5502973f155d73212948611a201fa2789a3db/crates/mini-index-exchange/src/codec.rs); [`crates/mini-index-exchange/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/4aa5502973f155d73212948611a201fa2789a3db/crates/mini-index-exchange/src/error.rs); [`crates/mini-index-exchange/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/4aa5502973f155d73212948611a201fa2789a3db/crates/mini-index-exchange/src/lib.rs); [`crates/mini-index-exchange/src/publication.rs`](https://github.com/mininet-labs/Mininet/blob/4aa5502973f155d73212948611a201fa2789a3db/crates/mini-index-exchange/src/publication.rs); [`crates/mini-index-exchange/tests/exchange.rs`](https://github.com/mininet-labs/Mininet/blob/4aa5502973f155d73212948611a201fa2789a3db/crates/mini-index-exchange/tests/exchange.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 2 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0281"></a>

## PR #281: mini-search-federation: F1/F2 exchange format + F3 federated query merging (D-0422, D-0423)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-05, FD-09, FD-11, FD-14, FD-16.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/281) | [Files changed](https://github.com/mininet-labs/Mininet/pull/281/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/3ab0c92ad1afb71120d68cdec83deaaf72a2cf92)

Head `3ab0c92ad1afb71120d68cdec83deaaf72a2cf92`; base `6fa7d9525361e624a3e618ecd25bee2e3d6b7e02`; merge `2e5c7c8b4cc989f1244472b96cdda2a52e28c19a`. 17 changed files; 2 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Creates the first coherent signed observation/segment exchange and deterministic federation merge, enabling plural local search providers rather than one canonical ranking owner.

**Mechanism and evidence.** F1/F2 wrap crawl observations and canonical index segments in ordinary signed objects. Readers validate type/shape while signature verification remains a separate caller obligation. F3 runs local sources through the existing query path, deduplicates canonical URLs by score/provider tiebreak and sorts deterministically.

**What remains weaker than the intended claim.** The initial sources are local, not a live federation. Higher-score-wins across providers is vulnerable if scores are later accepted without comparable authenticated inputs. A reader that decodes without provenance checking can attribute untrusted metadata. Source count and aggregate work were not yet bounded in this layer.

**Recommended improvement and rationale.** Provide a single verified-ingest facade for network callers and distinguish provider assertions from locally recomputed signals. Require one ranking profile/version or explicitly normalize incomparable results. Bound source/candidate work and preserve competing observations rather than suppressing disagreement by arbitrary trust score.

**Concrete example.** Two providers describe the same URL differently. Selecting the higher score is a merge policy, not proof that that provider is truthful; a hostile provider can otherwise win by declaring maximum relevance.

**Acceptance tests to implement.** Test input-order independence, URL collisions, malformed signed envelopes, forged provider attribution, mismatched profile versions and source-count floods. Verify ordinary unpaid content remains searchable without provider settlement.

**History, supersession and integration.** Supersedes #280's duplicate exchange proposal. #282 adds rescoring, #290 signed corpus metadata/transport and #294 remote result merging; each extends the trust boundary.

**Source entry points.** [`crates/mini-search-federation/src/codec.rs`](https://github.com/mininet-labs/Mininet/blob/3ab0c92ad1afb71120d68cdec83deaaf72a2cf92/crates/mini-search-federation/src/codec.rs); [`crates/mini-search-federation/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/3ab0c92ad1afb71120d68cdec83deaaf72a2cf92/crates/mini-search-federation/src/error.rs); [`crates/mini-search-federation/src/federate.rs`](https://github.com/mininet-labs/Mininet/blob/3ab0c92ad1afb71120d68cdec83deaaf72a2cf92/crates/mini-search-federation/src/federate.rs); [`crates/mini-search-federation/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/3ab0c92ad1afb71120d68cdec83deaaf72a2cf92/crates/mini-search-federation/src/lib.rs); [`crates/mini-search-federation/src/observation.rs`](https://github.com/mininet-labs/Mininet/blob/3ab0c92ad1afb71120d68cdec83deaaf72a2cf92/crates/mini-search-federation/src/observation.rs); [`crates/mini-search-federation/src/segment.rs`](https://github.com/mininet-labs/Mininet/blob/3ab0c92ad1afb71120d68cdec83deaaf72a2cf92/crates/mini-search-federation/src/segment.rs); [`crates/mini-search-federation/tests/federate.rs`](https://github.com/mininet-labs/Mininet/blob/3ab0c92ad1afb71120d68cdec83deaaf72a2cf92/crates/mini-search-federation/tests/federate.rs); [`crates/mini-search-federation/tests/federation.rs`](https://github.com/mininet-labs/Mininet/blob/3ab0c92ad1afb71120d68cdec83deaaf72a2cf92/crates/mini-search-federation/tests/federation.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0282"></a>

## PR #282: mini-search-federation + mini-ranker: local re-ranking under a caller's own profile (Track F4, D-0424)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-01, FD-09, FD-10, FD-11, FD-14, FD-16.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/282) | [Files changed](https://github.com/mininet-labs/Mininet/pull/282/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/c31ba47e468026769d25113df1bf33098e2a3c2b)

Head `c31ba47e468026769d25113df1bf33098e2a3c2b`; base `2e5c7c8b4cc989f1244472b96cdda2a52e28c19a`; merge `8881c232c42fe15208acfa5985bfab74cec04ecc`. 13 changed files; 1 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Lets users change ranking weights locally without revealing another query or asking a provider to approve their preferences. It also centralizes score arithmetic to avoid two subtly different ranking formulas.

**Mechanism and evidence.** mini-ranker::rescore and rank share weighted_average; local_rerank recombines retained per-signal explanations, updates the profile identifier, sorts and truncates. No network call, index fetch or payment occurs.

**What remains weaker than the intended claim.** Rescoring cannot verify that remote explanations were truthful. The diversity component is reused from the previous ordering rather than recomputed, so this is not necessarily equivalent to a fresh diversity-aware ranking pass. A result count limit does not bound the input list or sorting work.

**Recommended improvement and rationale.** Label the operation as reweighting retained explanations and distinguish it from full reranking. Bound input size; preserve provenance of each signal and optionally rerun diversity selection from adequate metadata. Keep personalized profiles local and never transmit them by default.

**Concrete example.** After changing the weights, three results from one domain may carry diversity penalties computed under their old positions. A displayed new profile must not imply every contextual signal was recomputed under the new order.

**Acceptance tests to implement.** Test arithmetic equivalence for fixed signals, genuinely different profiles, adversarial explanations, out-of-range values, old-diversity ordering and large candidate sets. Assert no query/profile bytes leave the device.

**History, supersession and integration.** Extends #257/#281. #294 network responses make signal authenticity especially important; authenticated provider identity does not itself make the explanation true.

**Source entry points.** [`crates/mini-ranker/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/c31ba47e468026769d25113df1bf33098e2a3c2b/crates/mini-ranker/src/lib.rs); [`crates/mini-ranker/src/rank.rs`](https://github.com/mininet-labs/Mininet/blob/c31ba47e468026769d25113df1bf33098e2a3c2b/crates/mini-ranker/src/rank.rs); [`crates/mini-ranker/tests/rank.rs`](https://github.com/mininet-labs/Mininet/blob/c31ba47e468026769d25113df1bf33098e2a3c2b/crates/mini-ranker/tests/rank.rs); [`crates/mini-search-federation/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/c31ba47e468026769d25113df1bf33098e2a3c2b/crates/mini-search-federation/src/error.rs); [`crates/mini-search-federation/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/c31ba47e468026769d25113df1bf33098e2a3c2b/crates/mini-search-federation/src/lib.rs); [`crates/mini-search-federation/src/rerank.rs`](https://github.com/mininet-labs/Mininet/blob/c31ba47e468026769d25113df1bf33098e2a3c2b/crates/mini-search-federation/src/rerank.rs); [`crates/mini-search-federation/tests/rerank.rs`](https://github.com/mininet-labs/Mininet/blob/c31ba47e468026769d25113df1bf33098e2a3c2b/crates/mini-search-federation/tests/rerank.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0283"></a>

## PR #283: Add bounded MiniSearch crawler fetch runtime

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-06, FD-09, FD-11, FD-14, FD-15.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/283) | [Files changed](https://github.com/mininet-labs/Mininet/pull/283/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/b2967a0316b3e7b910b3d94114d1b690de0f6b1e)

Head `b2967a0316b3e7b910b3d94114d1b690de0f6b1e`; base `8881c232c42fe15208acfa5985bfab74cec04ecc`; merge `cb82f607b967b42d57f114b6a9c61d2b3c83a74b`. 19 changed files; 1 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Adds the actual network fetch layer missing from MiniSearch and treats participant-device fetching as an SSRF security boundary rather than a harmless HTTP convenience.

**Mechanism and evidence.** The replaceable reqwest/platform-TLS backend disables automatic redirects/decompression, validates and pins DNS answers per hop, enforces scheme/port/body/time/redirect bounds and requires explicit robots authorization. It returns bounded observations with canonical transcript-derived identifiers.

**What remains weaker than the intended claim.** URL classification must cover every mapped/NAT64/special address form, proxy behavior and rebinding path. Caller-supplied robots authorization is not a fetched/parsed policy. Platform TLS introduces Web-PKI dependence at the optional web edge, not authority over native Mininet identity. Extraction and scheduling remain uncomposed.

**Recommended improvement and rationale.** Use one audited address-policy module with maintained test vectors, explicit proxy disabling, per-hop DNS pinning and conservative redirect rules. Build bounded robots parsing/cache and a real crawl-to-extract-to-index coordinator without letting external web trust become native canonical truth.

**Concrete example.** An apparently public URL redirects to a NAT64 representation of a private IPv4 host. Every hop must be classified before connection, and the chosen socket address must be the exact validated DNS result.

**Acceptance tests to implement.** Test DNS rebinding, redirects across schemes/ports, IPv4-mapped/NAT64 encodings, proxy environment variables, slow bodies, decompression bombs and robots-policy expiry. Run independent SSRF/TLS review before public crawl jobs.

**History, supersession and integration.** Completes a transport slice after #160/#252/#256; #300 later hardens omitted address ranges and NAT64 handling. Historical bypasses fixed there must not be listed as still open unchanged.

**Source entry points.** [`crates/mini-crawler-fetch/src/address.rs`](https://github.com/mininet-labs/Mininet/blob/b2967a0316b3e7b910b3d94114d1b690de0f6b1e/crates/mini-crawler-fetch/src/address.rs); [`crates/mini-crawler-fetch/src/backend.rs`](https://github.com/mininet-labs/Mininet/blob/b2967a0316b3e7b910b3d94114d1b690de0f6b1e/crates/mini-crawler-fetch/src/backend.rs); [`crates/mini-crawler-fetch/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/b2967a0316b3e7b910b3d94114d1b690de0f6b1e/crates/mini-crawler-fetch/src/lib.rs); [`crates/mini-crawler-fetch/src/policy.rs`](https://github.com/mininet-labs/Mininet/blob/b2967a0316b3e7b910b3d94114d1b690de0f6b1e/crates/mini-crawler-fetch/src/policy.rs); [`crates/mini-crawler-fetch/src/runtime.rs`](https://github.com/mininet-labs/Mininet/blob/b2967a0316b3e7b910b3d94114d1b690de0f6b1e/crates/mini-crawler-fetch/src/runtime.rs); [`crates/mini-search-federation/src/observation.rs`](https://github.com/mininet-labs/Mininet/blob/b2967a0316b3e7b910b3d94114d1b690de0f6b1e/crates/mini-search-federation/src/observation.rs); [`crates/mini-search-federation/tests/federation.rs`](https://github.com/mininet-labs/Mininet/blob/b2967a0316b3e7b910b3d94114d1b690de0f6b1e/crates/mini-search-federation/tests/federation.rs); [`crates/mini-web-types/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/b2967a0316b3e7b910b3d94114d1b690de0f6b1e/crates/mini-web-types/src/lib.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 2 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0284"></a>

## PR #284: Track F: object-bound snapshot history (F7, D-0426) + anti-collusion settlement doctrine (F5, D-0427)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-04, FD-05, FD-09, FD-11, FD-16, FD-18.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/284) | [Files changed](https://github.com/mininet-labs/Mininet/pull/284/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/624c74827441a9d16b8eb81a6b4335f4b1b1b954)

Head `624c74827441a9d16b8eb81a6b4335f4b1b1b954`; base `cb82f607b967b42d57f114b6a9c61d2b3c83a74b`; merge `4850a82476f6ba1b2d7386f1689a763a203a32e4`. 14 changed files; 28 commits; 1 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Adds uncertainty-aware local web history and makes the crucial economic distinction between fabricated payment volume and extraction of common funds. The latter prevents anti-collusion machinery from becoming a permission gate on ordinary voluntary exchange.

**Mechanism and evidence.** F7 derives all indexed fields from a canonical object after rechecking its content ID, bounds counts and bytes, and preserves equal-timestamp disagreement/unknown digests. F5 defines requester-funded, sponsor-funded, protocol-subsidized and forbidden authority-bearing settlement classes.

**What remains weaker than the intended claim.** Object integrity is not publisher honesty or canonical time; history is a local view. Delivery proof cannot establish genuine independent demand. The doctrine names responsibility boundaries but selects no production credential, beacon or subsidy mechanism.

**Recommended improvement and rationale.** Keep disagreements visible, authenticate provenance separately and maintain bounded reconstructible history. For subsidies precommit budget, policy family, overlap and audit rules before claims exist, while leaving requester-funded transfers independent of issuers/auditors.

**Concrete example.** A requester and provider controlled by one person can trade their own funds indefinitely without minting value. Paying the same activity from a protocol budget changes the threat: genuine delivery can still drain the entire subsidy.

**Acceptance tests to implement.** Test changed object bytes under stale IDs, equal-time conflicting observations, unknown digests, byte/count bounds and policy-class confusion. Model issuer/auditor disappearance and require ordinary transfers/search to remain available.

**History, supersession and integration.** Extends #281 history and corrects the overly broad collusion framing in #278. #285 implements a falsification model and records failures rather than authorizing production.

**Source entry points.** [`crates/mini-search-federation/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/624c74827441a9d16b8eb81a6b4335f4b1b1b954/crates/mini-search-federation/src/error.rs); [`crates/mini-search-federation/src/history.rs`](https://github.com/mininet-labs/Mininet/blob/624c74827441a9d16b8eb81a6b4335f4b1b1b954/crates/mini-search-federation/src/history.rs); [`crates/mini-search-federation/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/624c74827441a9d16b8eb81a6b4335f4b1b1b954/crates/mini-search-federation/src/lib.rs); [`crates/mini-search-federation/src/observation.rs`](https://github.com/mininet-labs/Mininet/blob/624c74827441a9d16b8eb81a6b4335f4b1b1b954/crates/mini-search-federation/src/observation.rs); [`crates/mini-search-federation/tests/federation.rs`](https://github.com/mininet-labs/Mininet/blob/624c74827441a9d16b8eb81a6b4335f4b1b1b954/crates/mini-search-federation/tests/federation.rs); [`crates/mini-search-federation/tests/history.rs`](https://github.com/mininet-labs/Mininet/blob/624c74827441a9d16b8eb81a6b4335f4b1b1b954/crates/mini-search-federation/tests/history.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 2 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0285"></a>

## PR #285: F5 Phase 2: settlement transcript, adversary/economic model, and falsification gates (D-0428)

**FAIL** | Captured outcome: **merged** | Directives: FD-02, FD-04, FD-05, FD-06, FD-09, FD-11, FD-16, FD-18.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/285) | [Files changed](https://github.com/mininet-labs/Mininet/pull/285/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/31560d07282de1cf09982b2ba12b7cf54a62a2a3)

Head `31560d07282de1cf09982b2ba12b7cf54a62a2a3`; base `4850a82476f6ba1b2d7386f1689a763a203a32e4`; merge `05a7e9faee287dabef517ea34b37f56cdb8687a9`. 11 changed files; 32 commits; 1 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Produces one of the most valuable negative results in the history: a reproducible model showing why the proposed subsidy defenses are not ready. FAIL here is the mechanism's anti-collusion/activation verdict, not a criticism of publishing the failed experiment.

**Mechanism and evidence.** The typed model binds funding source, policy, schema, epoch, event, transcript and replay domains; canonical ownership cannot be rewritten by audit output. Frozen JSONL vectors retain phase3_authorized=false. Tests bind the report to the executable model.

**What remains weaker than the intended claim.** The precommitted failures are decisive: genuine colluders exhaust the modeled budget, known realized audit entropy allows claim-ID grinding outside the sample, and configured replay capacity exceeds the memory ceiling. One friendly honest case cannot estimate population false rejection; threshold identities do not establish independent auditors.

**Recommended improvement and rationale.** Keep production subsidies disabled. Commit claims before unpredictable, bias-resistant audit entropy is realized; analyze withholding/fallback. Redesign or explicitly narrow the genuine-demand subsidy goal rather than pretending delivery integrity detects collusion. Reduce or safely compact replay capacity without evicting live replay keys.

**Concrete example.** For requester-funded payment, a colluding transfer costs the requester. For protocol subsidy, 100 genuine-delivery colluding pairs can consume 100% of the budget against the 10% loss ceiling. More valid delivery proofs do not solve that economic problem.

**Acceptance tests to implement.** Reproduce exact vectors, then test adaptive claim construction, multi-policy duplication, entropy withholding, issuer concentration and configured worst-case memory. Pre-register revised gates before experiments; never lower thresholds after seeing a failure.

**History, supersession and integration.** Implements #284's Phase-2 doctrine. Model unit tests passing and phase3_authorized=false are compatible: the tests correctly reproduce a failed design. No external gate is closed.

**Source entry points.** [`docs/DECISION_LOG.md`](https://github.com/mininet-labs/Mininet/blob/31560d07282de1cf09982b2ba12b7cf54a62a2a3/docs/DECISION_LOG.md); [`docs/STATUS.md`](https://github.com/mininet-labs/Mininet/blob/31560d07282de1cf09982b2ba12b7cf54a62a2a3/docs/STATUS.md); [`docs/design/anti-collusion-content-settlement-preparation.md`](https://github.com/mininet-labs/Mininet/blob/31560d07282de1cf09982b2ba12b7cf54a62a2a3/docs/design/anti-collusion-content-settlement-preparation.md); [`docs/design/f5-phase2-settlement-model.md`](https://github.com/mininet-labs/Mininet/blob/31560d07282de1cf09982b2ba12b7cf54a62a2a3/docs/design/f5-phase2-settlement-model.md); [`tools/f5_phase2_model.py`](https://github.com/mininet-labs/Mininet/blob/31560d07282de1cf09982b2ba12b7cf54a62a2a3/tools/f5_phase2_model.py); [`tools/fixtures/f5_phase2_report.jsonl`](https://github.com/mininet-labs/Mininet/blob/31560d07282de1cf09982b2ba12b7cf54a62a2a3/tools/fixtures/f5_phase2_report.jsonl); [`tools/test_f5_phase2_model.py`](https://github.com/mininet-labs/Mininet/blob/31560d07282de1cf09982b2ba12b7cf54a62a2a3/tools/test_f5_phase2_model.py); [`tools/test_f5_phase2_vectors.py`](https://github.com/mininet-labs/Mininet/blob/31560d07282de1cf09982b2ba12b7cf54a62a2a3/tools/test_f5_phase2_vectors.py). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0286"></a>

## PR #286: mini-social publish_post + mini-intake-social bridge + mini-cli intake wiring (D-0429)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-01, FD-02, FD-05, FD-06, FD-09, FD-14, FD-16.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/286) | [Files changed](https://github.com/mininet-labs/Mininet/pull/286/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/b30ca79dc8cd527679b17aff226fe7132ef25698)

Head `b30ca79dc8cd527679b17aff226fe7132ef25698`; base `05a7e9faee287dabef517ea34b37f56cdb8687a9`; merge `f482813bacf4d017add74bf8703b0c95af45e2a3`. 26 changed files; 4 commits; 3 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Connects native intake to bounded social publishing and unifies previously hand-built POST producers/readers. It also demonstrates why repeated review of integration boundaries matters more than a first green test run.

**Mechanism and evidence.** mini-social uses one Post validator for publish/resolve/feed. read_verified_source_bytes recomputes source digest, length and intake ID. The CLI signs once into a per-intake journal under a publication lock and attaches an idempotent bounded Post link.

**What remains weaker than the intended claim.** The first review's source-substitution and unbounded-producer gaps were fixed. The second review identifies remaining current code paths: load_envelope does not compare decoded ID to requested lookup ID; journal recovery only decodes an Object without verifying its author/type/source binding; cmd_advance does not take the publication lock. Accepted is explicitly only local workflow state.

**Recommended improvement and rationale.** Validate requested-envelope identity at load, verify recovered signed posts against the exact intake/source/current author, and serialize all envelope mutations under one transaction/lock. Bound every envelope mutator consistently with decode and use durable atomic journal replacement before reporting success.

**Concrete example.** A corrupted journal for intake A contains a valid post about B. Retry must reject it, not insert and link B as A's accepted publication. Concurrent advance-to-Rejected must not be overwritten by an old accepted envelope saved after publishing.

**Acceptance tests to implement.** Add wrong-ID lookup, forged/unrelated journal, invalid signature/type, concurrent advance/publish, full disk and process-kill tests. Assert no new signature on legitimate retry and no success after a validation/storage failure.

**History, supersession and integration.** Builds on #242 and #170. The first remediation is real progress; the still-visible second-review failure shapes must not be declared resolved solely because the PR merged.

**Source entry points.** [`crates/mini-cli/src/cli.rs`](https://github.com/mininet-labs/Mininet/blob/b30ca79dc8cd527679b17aff226fe7132ef25698/crates/mini-cli/src/cli.rs); [`crates/mini-cli/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/b30ca79dc8cd527679b17aff226fe7132ef25698/crates/mini-cli/src/error.rs); [`crates/mini-cli/src/intake.rs`](https://github.com/mininet-labs/Mininet/blob/b30ca79dc8cd527679b17aff226fe7132ef25698/crates/mini-cli/src/intake.rs); [`crates/mini-cli/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/b30ca79dc8cd527679b17aff226fe7132ef25698/crates/mini-cli/src/lib.rs); [`crates/mini-cli/tests/intake_commands.rs`](https://github.com/mininet-labs/Mininet/blob/b30ca79dc8cd527679b17aff226fe7132ef25698/crates/mini-cli/tests/intake_commands.rs); [`crates/mini-desktop/src/main.rs`](https://github.com/mininet-labs/Mininet/blob/b30ca79dc8cd527679b17aff226fe7132ef25698/crates/mini-desktop/src/main.rs); [`crates/mini-intake-social/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/b30ca79dc8cd527679b17aff226fe7132ef25698/crates/mini-intake-social/src/error.rs); [`crates/mini-intake-social/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/b30ca79dc8cd527679b17aff226fe7132ef25698/crates/mini-intake-social/src/lib.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 2 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0287"></a>

## PR #287: Forge Batch 5: bounded FsBackend time pages and stable cursors (D-0430)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-03, FD-06, FD-10, FD-11, FD-14.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/287) | [Files changed](https://github.com/mininet-labs/Mininet/pull/287/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/1487669da194b3648344c9fddb80c9b4504bddac)

Head `1487669da194b3648344c9fddb80c9b4504bddac`; base `5c93364e307013891bf934fafe6240b80c97b7de`; merge `05526f81ab18a214dd8a638ffb1452340ef87d85`. 12 changed files; 68 commits; 1 issue comments, 6 inline comments and 2 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Adds a genuinely bounded filesystem chronological query path, reducing pressure to outsource a long-lived local feed/Forge index to a hosted service. The acceleration structure remains disposable rather than authoritative.

**Mechanism and evidence.** An immutable sorted base, bounded append delta, checksummed manifest, cross-process lock and journal support ordered/time-v1. Queries re-read authoritative metadata before returning candidates; composite timestamp/object-ID cursors disambiguate ties. Rebuild/compaction preserve object authority.

**What remains weaker than the intended claim.** The page path is bounded for a fixed view, but rebuild and compaction remain proportional to total history and can hold locks. Old binaries/manual writes can bypass the side index. Checksums detect corruption, not malicious authorship, and non-Unix directory durability remains a stated limit.

**Recommended improvement and rationale.** Migrate interactive callers to the page API, schedule bounded/background compaction with visible resource budgets and measure flash wear/lock pauses. Preserve a deterministic rebuild from authoritative metadata and reject mixed-version writes unless a compatibility protocol is defined.

**Concrete example.** After a million-row index is deleted, the node should rebuild without losing objects; it must not silently freeze the UI indefinitely while pretending every recent-page operation is bounded. A backdated newly synced object still needs content reconciliation.

**Acceptance tests to implement.** Test base/delta duplicate compaction, interrupted manifest advancement, symlink paths, independent process writers, corruption rebuild and mixed-version access. Measure cold-cache page latency and worst-case maintenance pauses on a cheap device.

**History, supersession and integration.** Completes the filesystem optimization deferred by #189/#193. Review fixed parent-directory fsync, budget double counting and compaction equality coverage; those specific historical defects are not open at the final head.

**Source entry points.** [`crates/mini-store/src/backend.rs`](https://github.com/mininet-labs/Mininet/blob/1487669da194b3648344c9fddb80c9b4504bddac/crates/mini-store/src/backend.rs); [`crates/mini-store/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/1487669da194b3648344c9fddb80c9b4504bddac/crates/mini-store/src/lib.rs); [`crates/mini-store/src/store.rs`](https://github.com/mininet-labs/Mininet/blob/1487669da194b3648344c9fddb80c9b4504bddac/crates/mini-store/src/store.rs); [`crates/mini-store/src/time_index.rs`](https://github.com/mininet-labs/Mininet/blob/1487669da194b3648344c9fddb80c9b4504bddac/crates/mini-store/src/time_index.rs); [`crates/mini-store/tests/time_pages.rs`](https://github.com/mininet-labs/Mininet/blob/1487669da194b3648344c9fddb80c9b4504bddac/crates/mini-store/tests/time_pages.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0288"></a>

## PR #288: docs: correct D-0407 status to reflect merged coordination spine (D-0431)

**PASS** | Captured outcome: **merged** | Directives: FD-05, FD-10, FD-12, FD-14.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/288) | [Files changed](https://github.com/mininet-labs/Mininet/pull/288/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/805ba1176fa44f64a704f66e3e3ef992296cd7c7)

Head `805ba1176fa44f64a704f66e3e3ef992296cd7c7`; base `f482813bacf4d017add74bf8703b0c95af45e2a3`; merge `5c93364e307013891bf934fafe6240b80c97b7de`. 5 changed files; 1 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Corrects the accepted/merged status of the native coordination work without rewriting its old proposed decision entry. This preserves both historical provenance and an accurate living status.

**Mechanism and evidence.** D-0431 records the status determination for D-0407 after checking code ancestry; STATUS changes from proposed to shipped while naming unimplemented charter-to-policy and team-delegation work. The older decision text remains unchanged.

**What remains weaker than the intended claim.** The scoped PASS is for traceable status repair. Founder merge during bootstrap is not independent human review, external audit or future governance activation. A shipped coordination object remains non-authorizing even after the decision status changes.

**Recommended improvement and rationale.** Model proposal/acceptance/merge/external-review/activation as separate fields rather than one overloaded status. Add automated ancestry/citation checks where possible and retain human judgment for legitimacy.

**Concrete example.** A merged task-handoff command is shipped code, but it must not become an approval or team delegation because the status line now says shipped. The status record should show both facts.

**Acceptance tests to implement.** Check source ancestry, exact decision references and unchanged old entries; ensure outstanding integration tasks remain open. Test that a status-only correction cannot alter protected authority records.

**History, supersession and integration.** Corrects #267/D-0407, following #221's retrospective documentation discipline. It creates no new runtime behavior or governance power.

**Source entry points.** [`docs/DECISION_LOG.md`](https://github.com/mininet-labs/Mininet/blob/805ba1176fa44f64a704f66e3e3ef992296cd7c7/docs/DECISION_LOG.md); [`docs/STATUS.md`](https://github.com/mininet-labs/Mininet/blob/805ba1176fa44f64a704f66e3e3ef992296cd7c7/docs/STATUS.md). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0289"></a>

## PR #289: Consensus: authenticated snapshots, persistent recovery, and bounded pruning (D-0207)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-04, FD-05, FD-06, FD-11, FD-14.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/289) | [Files changed](https://github.com/mininet-labs/Mininet/pull/289/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/6c3e019b324e6e719e522645997be640c4100790)

Head `6c3e019b324e6e719e522645997be640c4100790`; base `cf71cf3dce7a0b8236cdefd14bcd5fbbe1c26b6b`; merge `3d968d14eef40bb12bb67071a6d9c20576ac18d9`. 26 changed files; 58 commits; 1 issue comments, 10 inline comments and 6 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Builds real authenticated state snapshots, persistent recovery and bounded pruning so a long-offline node can catch up without trusting a checkpoint operator. This is a major vertical integration of consensus and storage.

**Mechanism and evidence.** ConsensusSnapshot binds finalized header/QC/network/state commitment; receivers verify with their own validator/KEL view. State-sync adoption is all-or-nothing on a cloned chain. ConsensusArchive adds locking, preflight plans, replay journaling, no-follow handle reads and snapshot-before-prune ordering.

**What remains weaker than the intended claim.** Static validator membership and old-key/long-range assumptions remain unresolved. A trusted local validator set is not proof of legitimate human membership. The original one-frame transfer limits state size, while local atomic replacement does not by itself establish every filesystem's power-loss guarantees.

**Recommended improvement and rationale.** Add historical validator transitions, explicit long-range recovery rules and resumable multi-peer authenticated state transfer without a central checkpoint. Preserve validate-before-mutate and treat archive files as adversarial. Measure recovery and pruning on weak devices.

**Concrete example.** A malicious peer sends a valid snapshot followed by a bad final suffix block. The entire candidate must be rejected with both live chain and archive unchanged, not partially installed because the first part verified.

**Acceptance tests to implement.** Replay the rejected-journal poisoning, invalid final-state write and symlink-open races fixed during review. Add power-loss, concurrent process, stale validator-set, wrong-network, partial-chunk and maximum-resource tests on the final integrated path.

**History, supersession and integration.** Extends #125/#130; #300 adds exact-body commitment across all formats and #326 chunked transfer. Both are required to understand the current behavior rather than treating this initial snapshot PR as complete sync.

**Source entry points.** [`crates/mini-consensus/src/catchup.rs`](https://github.com/mininet-labs/Mininet/blob/6c3e019b324e6e719e522645997be640c4100790/crates/mini-consensus/src/catchup.rs); [`crates/mini-consensus/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/6c3e019b324e6e719e522645997be640c4100790/crates/mini-consensus/src/error.rs); [`crates/mini-consensus/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/6c3e019b324e6e719e522645997be640c4100790/crates/mini-consensus/src/lib.rs); [`crates/mini-consensus/src/net.rs`](https://github.com/mininet-labs/Mininet/blob/6c3e019b324e6e719e522645997be640c4100790/crates/mini-consensus/src/net.rs); [`crates/mini-consensus/src/node.rs`](https://github.com/mininet-labs/Mininet/blob/6c3e019b324e6e719e522645997be640c4100790/crates/mini-consensus/src/node.rs); [`crates/mini-consensus/src/snapshot.rs`](https://github.com/mininet-labs/Mininet/blob/6c3e019b324e6e719e522645997be640c4100790/crates/mini-consensus/src/snapshot.rs); [`crates/mini-consensus/src/snapshot_sync_tests.rs`](https://github.com/mininet-labs/Mininet/blob/6c3e019b324e6e719e522645997be640c4100790/crates/mini-consensus/src/snapshot_sync_tests.rs); [`crates/mini-consensus/src/state_sync.rs`](https://github.com/mininet-labs/Mininet/blob/6c3e019b324e6e719e522645997be640c4100790/crates/mini-consensus/src/state_sync.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 1 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0290"></a>

## PR #290: feat: mini-search-federation-net -- bounded F1/F2/F2b transport + federate_query wiring (D-0432/D-0433)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-06, FD-09, FD-11, FD-14, FD-16.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/290) | [Files changed](https://github.com/mininet-labs/Mininet/pull/290/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/36a734a4a6b8320d01dbc9b2f729595f17f2bfd5)

Head `36a734a4a6b8320d01dbc9b2f729595f17f2bfd5`; base `05526f81ab18a214dd8a638ffb1452340ef87d85`; merge `186365ed9818364020471b8eab88eb8cbeeb7657`. 24 changed files; 3 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Moves signed search observations/segments/corpus metadata over the existing verified object transport and feeds the result into a real federated query. It closes a meaningful data-plane composition gap.

**Mechanism and evidence.** pull_source advertises IDs, retrieves exact objects through mini-sync and post-checks F1/F2/F2b types and expected provider. F2b carries the corpus/context metadata needed by the existing query function; assembly builds a source only from trusted pulled objects. Source contact count is bounded.

**What remains weaker than the intended claim.** Transport authentication/provenance does not make a crawler's metadata true. ID advertisements expose interest in particular index objects even without query text. Peer discovery, fair scheduling, per-peer fault isolation and aggregate corpus/query resource bounds remain open.

**Recommended improvement and rationale.** Use authenticated expected-provider bindings from the connection, preserve observation uncertainty and isolate one bad peer from otherwise valid sources. Bound total bytes/candidates across sources and provide local query operation without any mandatory provider.

**Concrete example.** A provider signs a correct segment and an unrelated corpus bundle. Assembly must reject the mismatch; a signature alone does not make those two objects a coherent query source.

**Acceptance tests to implement.** Test wrong-provider objects, mixed segment/bundle IDs, partial retrieval, dishonest metadata, source floods and one failed source among honest peers. Capture transport metadata and measure full query memory, not only transfer frame limits.

**History, supersession and integration.** Composes #281 and #269 retrieval; #294 adds remote query text/result transport, while #296 improves peer-bound provenance. Those later changes do not make F1/F2 claims independently truthful.

**Source entry points.** [`crates/mini-search-federation-net/src/assemble.rs`](https://github.com/mininet-labs/Mininet/blob/36a734a4a6b8320d01dbc9b2f729595f17f2bfd5/crates/mini-search-federation-net/src/assemble.rs); [`crates/mini-search-federation-net/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/36a734a4a6b8320d01dbc9b2f729595f17f2bfd5/crates/mini-search-federation-net/src/error.rs); [`crates/mini-search-federation-net/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/36a734a4a6b8320d01dbc9b2f729595f17f2bfd5/crates/mini-search-federation-net/src/lib.rs); [`crates/mini-search-federation-net/src/message.rs`](https://github.com/mininet-labs/Mininet/blob/36a734a4a6b8320d01dbc9b2f729595f17f2bfd5/crates/mini-search-federation-net/src/message.rs); [`crates/mini-search-federation-net/src/multi.rs`](https://github.com/mininet-labs/Mininet/blob/36a734a4a6b8320d01dbc9b2f729595f17f2bfd5/crates/mini-search-federation-net/src/multi.rs); [`crates/mini-search-federation-net/src/session.rs`](https://github.com/mininet-labs/Mininet/blob/36a734a4a6b8320d01dbc9b2f729595f17f2bfd5/crates/mini-search-federation-net/src/session.rs); [`crates/mini-search-federation-net/tests/assemble.rs`](https://github.com/mininet-labs/Mininet/blob/36a734a4a6b8320d01dbc9b2f729595f17f2bfd5/crates/mini-search-federation-net/tests/assemble.rs); [`crates/mini-search-federation-net/tests/federated_query_over_tcp.rs`](https://github.com/mininet-labs/Mininet/blob/36a734a4a6b8320d01dbc9b2f729595f17f2bfd5/crates/mini-search-federation-net/tests/federated_query_over_tcp.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 1 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0292"></a>

## PR #292: Privacy: authenticated peers, anti-eclipse discovery, and onion transport (D-03xx)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-06, FD-09, FD-11, FD-14, FD-15.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/292) | [Files changed](https://github.com/mininet-labs/Mininet/pull/292/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/a01ebba543fa4409ffd3fc31b2b4422da7cb3dac)

Head `a01ebba543fa4409ffd3fc31b2b4422da7cb3dac`; base `bfbabba72b875aecccffc435f9b6eb164afa708f`; merge `e60191c4a0fdc8be42995cc2fb21b9a56e910f44`. 28 changed files; 53 commits; 0 issue comments, 16 inline comments and 2 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Introduces the privacy/transport-security workstream with explicit failure verdicts for censorship and global-observer threats, plus authentication/discovery/onion implementation that later needed convergence and remediation. Its history must not be reduced to its stale scope-only description.

**Mechanism and evidence.** The captured GitHub record says this PR merged, while its description still says work in progress. The diff and review include signed advertisements/session authentication, replay handling and onion/relay work. Review called out public caller-supplied nonces and reuse of the already-existing bridge manager.

**What remains weaker than the intended claim.** Merged metadata is not proof that every promised runtime path was safely integrated. A predictable nonce is not automatically a forgery, but caller-owned freshness/replay inputs and weak retention can invalidate the intended replay contract. Temporary remediation automation and branch divergence complicate provenance.

**Recommended improvement and rationale.** Credit actual code from the pinned diff and follow each defect through #296 rather than use the stale body as truth. Generate freshness entropy inside issuance APIs, retain live replay IDs, bind expected endpoint identity to channels and keep bridge capability separate from identity authority.

**Concrete example.** A future caller copies a fixed test nonce into advertisement issuance. The API should make that impossible for normal issuance, while replay protection still validates signed validity intervals and refuses capacity exhaustion safely.

**Acceptance tests to implement.** Test stale/repeated advertisements, channel transplant, clock rollback, cache saturation, prefix aliases, wrong endpoint and real multi-hop sockets. Audit every temporary workflow permission and verify no branch-writing helper survives in the permanent feature diff.

**History, supersession and integration.** PR #292 is merged, not one of the eight closed-unmerged proposals. #295 is an unmerged remediation trigger; #296 is the integrated replacement/convergence implementation and resolves identified issues by construction.

**Source entry points.** [`crates/mini-relay/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/a01ebba543fa4409ffd3fc31b2b4422da7cb3dac/crates/mini-relay/src/error.rs); [`crates/mini-relay/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/a01ebba543fa4409ffd3fc31b2b4422da7cb3dac/crates/mini-relay/src/lib.rs); [`crates/mini-relay/src/onion.rs`](https://github.com/mininet-labs/Mininet/blob/a01ebba543fa4409ffd3fc31b2b4422da7cb3dac/crates/mini-relay/src/onion.rs); [`crates/mini-relay/tests/onion_tcp.rs`](https://github.com/mininet-labs/Mininet/blob/a01ebba543fa4409ffd3fc31b2b4422da7cb3dac/crates/mini-relay/tests/onion_tcp.rs); [`crates/mini-transport-security/src/advertisement.rs`](https://github.com/mininet-labs/Mininet/blob/a01ebba543fa4409ffd3fc31b2b4422da7cb3dac/crates/mini-transport-security/src/advertisement.rs); [`crates/mini-transport-security/src/auth.rs`](https://github.com/mininet-labs/Mininet/blob/a01ebba543fa4409ffd3fc31b2b4422da7cb3dac/crates/mini-transport-security/src/auth.rs); [`crates/mini-transport-security/src/codec.rs`](https://github.com/mininet-labs/Mininet/blob/a01ebba543fa4409ffd3fc31b2b4422da7cb3dac/crates/mini-transport-security/src/codec.rs); [`crates/mini-transport-security/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/a01ebba543fa4409ffd3fc31b2b4422da7cb3dac/crates/mini-transport-security/src/error.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 1 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0293"></a>

## PR #293: Cold/owner-only storage tiers: sealed-box encryption + CacheTier::ColdArchive (D-0434, closes #34)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-06, FD-09, FD-11, FD-14.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/293) | [Files changed](https://github.com/mininet-labs/Mininet/pull/293/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/e50b41a005f781769d6e7a6f1c3c7845f5244f13)

Head `e50b41a005f781769d6e7a6f1c3c7845f5244f13`; base `186365ed9818364020471b8eab88eb8cbeeb7657`; merge `cf71cf3dce7a0b8236cdefd14bcd5fbbe1c26b6b`. 13 changed files; 2 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Gives encrypted object payloads a real owner-only construction and adds a cold storage tier that is never advertised. It makes private retention a concrete local policy rather than a tag with no encryption behind it.

**Mechanism and evidence.** owner_seal composes fresh X25519 agreement, HKDF-SHA256 and ChaCha20-Poly1305 with an independent owner sealing key. ColdArchive is explicitly selected, not promoted by viewing, and stays below the private advertisement ceiling.

**What remains weaker than the intended claim.** AEAD protects ciphertext integrity under the derived key, but sealed-box encryption alone does not authenticate the sender or bind the recipient key to a KEL/owner policy. Independent keys need backup, rotation and re-sealing; losing them loses access. The cache tier alone does not enforce a complete eviction/replication runtime.

**Recommended improvement and rationale.** Bind owner sealing keys to explicit signed owner policy and object context, with a reviewed key-rotation/re-encryption and owner-held recovery workflow. Keep sender provenance separate from confidentiality. Never advertise encrypted objects through a direct tier-setting bypass.

**Concrete example.** Anyone knowing the public sealing key can encrypt a blob to the owner. Successful decryption means it was encrypted for that key, not that a trusted author sent it. Restoring identity without the separate sealing secret does not restore the archive.

**Acceptance tests to implement.** Test wrong key, altered AAD/ephemeral key, truncated/oversized framing, downgrade tier changes, backup restoration and rotation with old ciphertext. Inspect all advertisement consumers, not only note_view.

**History, supersession and integration.** Builds on object privacy #143/#170; #300 later fixes framing overhead and encrypted cache-advertisement bounds. No mandatory custody service is an acceptable recovery shortcut.

**Source entry points.** [`crates/mini-store/src/cache.rs`](https://github.com/mininet-labs/Mininet/blob/e50b41a005f781769d6e7a6f1c3c7845f5244f13/crates/mini-store/src/cache.rs); [`crates/mini-store/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/e50b41a005f781769d6e7a6f1c3c7845f5244f13/crates/mini-store/src/lib.rs); [`crates/mini-store/src/owner_seal.rs`](https://github.com/mininet-labs/Mininet/blob/e50b41a005f781769d6e7a6f1c3c7845f5244f13/crates/mini-store/src/owner_seal.rs); [`crates/mini-store/tests/cache.rs`](https://github.com/mininet-labs/Mininet/blob/e50b41a005f781769d6e7a6f1c3c7845f5244f13/crates/mini-store/tests/cache.rs); [`crates/mini-store/tests/owner_seal.rs`](https://github.com/mininet-labs/Mininet/blob/e50b41a005f781769d6e7a6f1c3c7845f5244f13/crates/mini-store/tests/owner_seal.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0294"></a>

## PR #294: Track F6 Phase 1+2: bounded, confidential-in-transit remote query transport + F3 merge wiring (D-0435/D-0436, roadmap #175)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-06, FD-09, FD-11, FD-14, FD-16.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/294) | [Files changed](https://github.com/mininet-labs/Mininet/pull/294/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/ae8e961d688af99dbe2ae6dda429953e6f185e19)

Head `ae8e961d688af99dbe2ae6dda429953e6f185e19`; base `3d968d14eef40bb12bb67071a6d9c20576ac18d9`; merge `bfbabba72b875aecccffc435f9b6eb164afa708f`. 15 changed files; 3 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Adds bounded remote query execution and typed merging so a user can obtain a few results without downloading a whole index. It also exposes a new privacy trade: the queried provider sees the exact query.

**Mechanism and evidence.** remote_query/serve_query exchange bounded query/profile/result data over Channel and reuse the local parser/ranker. remote_merge checks score/explanation ranges before merging through the existing deterministic F3 policy. Initial provider tags are caller assertions.

**What remains weaker than the intended claim.** Confidential-in-transit is not private-information retrieval, and anonymous key exchange does not identify an intended peer against an active intermediary. A provider can return plausible but false scores within bounds; numeric validation proves shape, not ranking honesty. One invalid result currently fails the entire merge.

**Recommended improvement and rationale.** Offer a clearly labeled provider-visible mode and a local-index alternative; bind source attribution to authenticated connection identity. Compare reviewed PIR/oblivious retrieval only as a separate research tradeoff with workload/leakage/resource measurements. Preserve per-source faults and locally verifiable provenance.

**Concrete example.** A result carries score 10,000 and a valid provider signature. It is still a provider's assertion; the client must not label it independently verified relevance or infer that the provider never saw the query.

**Acceptance tests to implement.** Test query/profile leakage, active MITM, forged provider tag, maximum valid false scores, malformed one-source results and request/response budgets. Measure traffic correlation and maintain basic search without a paid or canonical provider.

**History, supersession and integration.** Extends #290. #296 adds an authenticated transport/provenance seam; it does not implement PIR or global-observer resistance. F6 completion must be scoped to the delivered transport, not its aspirational private-search name.

**Source entry points.** [`crates/mini-search-federation-net/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/ae8e961d688af99dbe2ae6dda429953e6f185e19/crates/mini-search-federation-net/src/error.rs); [`crates/mini-search-federation-net/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/ae8e961d688af99dbe2ae6dda429953e6f185e19/crates/mini-search-federation-net/src/lib.rs); [`crates/mini-search-federation-net/src/query.rs`](https://github.com/mininet-labs/Mininet/blob/ae8e961d688af99dbe2ae6dda429953e6f185e19/crates/mini-search-federation-net/src/query.rs); [`crates/mini-search-federation-net/src/remote_merge.rs`](https://github.com/mininet-labs/Mininet/blob/ae8e961d688af99dbe2ae6dda429953e6f185e19/crates/mini-search-federation-net/src/remote_merge.rs); [`crates/mini-search-federation-net/tests/query_over_tcp.rs`](https://github.com/mininet-labs/Mininet/blob/ae8e961d688af99dbe2ae6dda429953e6f185e19/crates/mini-search-federation-net/tests/query_over_tcp.rs); [`crates/mini-search-federation/src/federate.rs`](https://github.com/mininet-labs/Mininet/blob/ae8e961d688af99dbe2ae6dda429953e6f185e19/crates/mini-search-federation/src/federate.rs); [`crates/mini-search-federation/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/ae8e961d688af99dbe2ae6dda429953e6f185e19/crates/mini-search-federation/src/lib.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 1 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0295"></a>

## PR #295: CI trigger: validate PR 292 review remediation

**PARTIAL** | Captured outcome: **closed** | Directives: FD-02, FD-06, FD-12, FD-14.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/295) | [Files changed](https://github.com/mininet-labs/Mininet/pull/295/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/643f48de9eb2d4c66274fcbedb665bd85d3f5005)

Head `643f48de9eb2d4c66274fcbedb665bd85d3f5005`; base `3d968d14eef40bb12bb67071a6d9c20576ac18d9`; merge `none`. 31 changed files; 39 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This closed-unmerged PR was a temporary same-repository trigger for #292 remediation. It contributed execution/coordination evidence, not a permanent feature, and must still be reviewed because workflows can act without being merged.

**Mechanism and evidence.** The PR explicitly says do not merge and was closed after its intended remediation. The captured diff contains the trigger/remediation workflow and helper context; its effect must be evaluated through actual workflow permissions, target branch and resulting commits.

**What remains weaker than the intended claim.** Unmerged does not mean harmless or unexecuted: a workflow with write permission can change another branch. The declared self-deletion convention is not proof of cleanup, and untrusted PR data must never be interpolated into privileged commands or used to approve itself.

**Recommended improvement and rationale.** Prefer read-only testing artifacts and authenticated connector/user-approved commits over branch-writing remediation workflows. Where temporary automation is unavoidable, pin target/head, minimize permissions, fail on concurrent head movement and verify removal in the actual destination history.

**Concrete example.** A test-only trigger runs with contents:write and pushes to the feature branch. Auditors must trace that pushed commit even though the trigger PR never merged; otherwise the most consequential action disappears from the history ledger.

**Acceptance tests to implement.** Inspect run logs, exact target/head comparisons, command-input handling and permissions; verify the resulting commit diff and absence of leftover workflows/helpers. Confirm no approvals, secrets or release authority were manufactured.

**History, supersession and integration.** Closed-unmerged auxiliary to #292; #296 documents removing the temporary helpers and integrating the permanent fixes. This dossier records it rather than counting it as a shipped transport feature.

**Source entry points.** [`crates/mini-relay/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/643f48de9eb2d4c66274fcbedb665bd85d3f5005/crates/mini-relay/src/error.rs); [`crates/mini-relay/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/643f48de9eb2d4c66274fcbedb665bd85d3f5005/crates/mini-relay/src/lib.rs); [`crates/mini-relay/src/onion.rs`](https://github.com/mininet-labs/Mininet/blob/643f48de9eb2d4c66274fcbedb665bd85d3f5005/crates/mini-relay/src/onion.rs); [`crates/mini-relay/tests/onion_tcp.rs`](https://github.com/mininet-labs/Mininet/blob/643f48de9eb2d4c66274fcbedb665bd85d3f5005/crates/mini-relay/tests/onion_tcp.rs); [`crates/mini-transport-security/src/advertisement.rs`](https://github.com/mininet-labs/Mininet/blob/643f48de9eb2d4c66274fcbedb665bd85d3f5005/crates/mini-transport-security/src/advertisement.rs); [`crates/mini-transport-security/src/auth.rs`](https://github.com/mininet-labs/Mininet/blob/643f48de9eb2d4c66274fcbedb665bd85d3f5005/crates/mini-transport-security/src/auth.rs); [`crates/mini-transport-security/src/codec.rs`](https://github.com/mininet-labs/Mininet/blob/643f48de9eb2d4c66274fcbedb665bd85d3f5005/crates/mini-transport-security/src/codec.rs); [`crates/mini-transport-security/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/643f48de9eb2d4c66274fcbedb665bd85d3f5005/crates/mini-transport-security/src/error.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 1 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0296"></a>

## PR #296: Privacy transport convergence: authenticated runtime, bridge seam, and onion path (D-0438)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-06, FD-09, FD-11, FD-14, FD-15, FD-16.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/296) | [Files changed](https://github.com/mininet-labs/Mininet/pull/296/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/2445ab0fe07bd9537000adf65253b4055a9621a5)

Head `2445ab0fe07bd9537000adf65253b4055a9621a5`; base `a9f9c96d3bb3dd9419ffce54b9b9abcbe05c3a11`; merge `3b9c85bc5ee40e297d5969618ff3f0e7875943ab`. 32 changed files; 98 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Converges authenticated discovery, optional peer identity and destination-encrypted three-hop onion transport into real-socket paths while retaining anonymous operation and replaceable bridges. This is a substantial integration advance.

**Mechanism and evidence.** Session claims bind delegated identity to the actual channel and advertised endpoint; expiring signed advertisements feed locally seeded prefix-diverse selection. Replay retention fails closed and tracks a time high-water mark. Each onion hop has independent ephemeral X25519/AEAD, with payload encrypted to delivery destination.

**What remains weaker than the intended claim.** Prefix diversity is not operator independence, three-hop onion routing is not Sphinx/mixing or global-observer anonymity, and NAT/censorship/mobile runtime remain open. Mixed/Burst are unreachable because no executor exists; the executable_transport helper has no callers, so its future enforcement is not load-bearing.

**Recommended improvement and rationale.** Introduce one mandatory runtime tier dispatcher that cannot execute unsupported promises, then measure endpoint/timing/volume leakage and adversarial relay selection. Preserve per-hop/destination encryption and explicit peer identity as optional purpose-specific authentication, not a universal public identity handshake.

**Concrete example.** A valid IPv4 peer advertised in native, mapped-IPv6 and NAT64 forms must occupy one diversity bucket, not three. A request for Mixed must return an explicit unimplemented error before any packet is sent.

**Acceptance tests to implement.** Rerun real discovery-to-authentication-to-onion tests with malicious relays, wrong routing keys, replay/cache saturation, rollback clocks, prefix aliases and dropped peers. Verify no relay sees application plaintext or both intended endpoints under the stated model; test traffic analysis separately.

**History, supersession and integration.** Integrates #292 and reuses #150's bridge rather than creating another authority. Review restored lost D-0437 history and removed stale storage-fraud prose; #301 later encodes these registry incidents in checks.

**Source entry points.** [`crates/mini-relay/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/2445ab0fe07bd9537000adf65253b4055a9621a5/crates/mini-relay/src/error.rs); [`crates/mini-relay/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/2445ab0fe07bd9537000adf65253b4055a9621a5/crates/mini-relay/src/lib.rs); [`crates/mini-relay/src/onion.rs`](https://github.com/mininet-labs/Mininet/blob/2445ab0fe07bd9537000adf65253b4055a9621a5/crates/mini-relay/src/onion.rs); [`crates/mini-relay/tests/onion_tcp.rs`](https://github.com/mininet-labs/Mininet/blob/2445ab0fe07bd9537000adf65253b4055a9621a5/crates/mini-relay/tests/onion_tcp.rs); [`crates/mini-search-federation-net/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/2445ab0fe07bd9537000adf65253b4055a9621a5/crates/mini-search-federation-net/src/error.rs); [`crates/mini-search-federation-net/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/2445ab0fe07bd9537000adf65253b4055a9621a5/crates/mini-search-federation-net/src/lib.rs); [`crates/mini-search-federation-net/src/query.rs`](https://github.com/mininet-labs/Mininet/blob/2445ab0fe07bd9537000adf65253b4055a9621a5/crates/mini-search-federation-net/src/query.rs); [`crates/mini-search-federation-net/src/remote_merge.rs`](https://github.com/mininet-labs/Mininet/blob/2445ab0fe07bd9537000adf65253b4055a9621a5/crates/mini-search-federation-net/src/remote_merge.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 2 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0297"></a>

## PR #297: Cross-identity storage-fraud collision evidence: mini-storage-fraud (D-0437, roadmap #42)

**FAIL** | Captured outcome: **merged** | Directives: FD-02, FD-05, FD-06, FD-08, FD-15.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/297) | [Files changed](https://github.com/mininet-labs/Mininet/pull/297/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/7a48a878b9007364a7779f3e03ceec5066e34d61)

Head `7a48a878b9007364a7779f3e03ceec5066e34d61`; base `e60191c4a0fdc8be42995cc2fb21b9a56e910f44`; merge `af45860914edfe16e0fbc483b927bb2b274666d8`. 15 changed files; 1 commits; 1 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Attempts objective storage-fraud evidence across identity roots, but its central inference is unsound. Recording this failed construction and its replacement is essential to understanding why tests and signatures alone did not prove the intended property.

**Mechanism and evidence.** The original claim signs a caller-supplied StorageCommitment; verify_collision accepts two different roots signing the same commitment as fraud evidence. Identity-bound replica-ID derivation exists, but verification does not bind the signed root to an actually audited seal.

**What remains weaker than the intended claim.** An attacker can copy an honest provider's published root, sign it with a valid separate DID and manufacture accepted collision evidence without storing a replica or forging any signature. Duplicate signed bytes do not identify which party, if any, misbehaved. Honest-sealer tests do not test malicious claim construction.

**Recommended improvement and rationale.** Replace rather than cosmetically patch the design: verify registration/seal evidence, derive the storage commitment from the verified seal, reject duplicate admission and classify cross-registry conflicts as unattributed pending further evidence. Never punish an innocent provider based on a copied root.

**Concrete example.** Alice publishes root R. Mallory signs R under her own identity and submits Alice's and her signatures. The old verifier accepts the accusation even though Alice did nothing wrong.

**Acceptance tests to implement.** Construct the copied-root attack through public APIs and require rejection before any reward/exclusion consequence. Test corrupt auditor quorums, stolen receipts, alternate contexts and attribution discipline. Honest distinct seals are only a baseline, not the adversarial proof.

**History, supersession and integration.** Merged historically, then superseded by #299. #298 is the closed-unmerged intermediate replacement. The current tree must not be judged as still running this exact original design.

**Source entry points.** [`crates/mini-storage-fraud/src/codec.rs`](https://github.com/mininet-labs/Mininet/blob/7a48a878b9007364a7779f3e03ceec5066e34d61/crates/mini-storage-fraud/src/codec.rs); [`crates/mini-storage-fraud/src/collision.rs`](https://github.com/mininet-labs/Mininet/blob/7a48a878b9007364a7779f3e03ceec5066e34d61/crates/mini-storage-fraud/src/collision.rs); [`crates/mini-storage-fraud/src/commitment_claim.rs`](https://github.com/mininet-labs/Mininet/blob/7a48a878b9007364a7779f3e03ceec5066e34d61/crates/mini-storage-fraud/src/commitment_claim.rs); [`crates/mini-storage-fraud/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/7a48a878b9007364a7779f3e03ceec5066e34d61/crates/mini-storage-fraud/src/error.rs); [`crates/mini-storage-fraud/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/7a48a878b9007364a7779f3e03ceec5066e34d61/crates/mini-storage-fraud/src/lib.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0298"></a>

## PR #298: Rebuild storage-fraud detection around audited registration; fix three defects in merged code (D-0437, D-0438)

**PARTIAL** | Captured outcome: **closed** | Directives: FD-02, FD-05, FD-06, FD-08, FD-14, FD-15.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/298) | [Files changed](https://github.com/mininet-labs/Mininet/pull/298/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/052d44dbfde62307291e2a91aaed81e68bd662f2)

Head `052d44dbfde62307291e2a91aaed81e68bd662f2`; base `af45860914edfe16e0fbc483b927bb2b274666d8`; merge `28177d3d3c4c5c42845b4c56468debc628ca77a6`. 50 changed files; 1 commits; 2 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** This closed-unmerged remediation identifies the root cause of #297's framing attack and broadens the investigation to decoder consistency and PoRep challenge coverage. Its findings materially shaped the replacement that actually landed.

**Mechanism and evidence.** The proposal introduces typed replica context, identity/device-derived replica IDs, full seal commitments and quorum audit receipts, historical KEL verification, STORE delegation and an unattributed conflict result. It also fixes the missing final-layer sampling condition and inconsistent signature counts/canonicalization.

**What remains weaker than the intended claim.** The original branch rewrote D-0437 after it became merged history and collided with #296's D-0438. Those governance defects require a new superseding decision, not silent replacement. An auditor-root quorum remains a possible single operator and cannot prove physical independence.

**Recommended improvement and rationale.** Preserve the failed historical decision and append a uniquely identified supersession; carry the code and attack tests into one rebased proposal. Separate verified audit sampling from assumptions about auditor honesty, challenge unpredictability and attribution.

**Concrete example.** An audit sampling only intermediate layers can accept a copied arbitrary replica root because the final encoding commitment was never checked. Reserving final-layer challenges addresses that specific gap but does not prove the entire PoRep construction sound.

**Acceptance tests to implement.** Test copied/stolen registration evidence, historical rotations, STORE absence, duplicate auditor roots/seeds and forced final-layer coverage. Run decision-history deletion/collision checks before merge.

**History, supersession and integration.** Closed in favor of #299 after #297 merged and a broader review arrived. Do not count #298's proposal and #299's implementation as two independently validated defenses.

**Source entry points.** [`crates/did-mini/src/delegation.rs`](https://github.com/mininet-labs/Mininet/blob/052d44dbfde62307291e2a91aaed81e68bd662f2/crates/did-mini/src/delegation.rs); [`crates/did-mini/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/052d44dbfde62307291e2a91aaed81e68bd662f2/crates/did-mini/src/error.rs); [`crates/did-mini/src/event.rs`](https://github.com/mininet-labs/Mininet/blob/052d44dbfde62307291e2a91aaed81e68bd662f2/crates/did-mini/src/event.rs); [`crates/did-mini/src/kel.rs`](https://github.com/mininet-labs/Mininet/blob/052d44dbfde62307291e2a91aaed81e68bd662f2/crates/did-mini/src/kel.rs); [`crates/did-mini/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/052d44dbfde62307291e2a91aaed81e68bd662f2/crates/did-mini/src/lib.rs); [`crates/did-mini/src/limits.rs`](https://github.com/mininet-labs/Mininet/blob/052d44dbfde62307291e2a91aaed81e68bd662f2/crates/did-mini/src/limits.rs); [`crates/did-mini/tests/delegation.rs`](https://github.com/mininet-labs/Mininet/blob/052d44dbfde62307291e2a91aaed81e68bd662f2/crates/did-mini/tests/delegation.rs); [`crates/did-mini/tests/historical_verification.rs`](https://github.com/mininet-labs/Mininet/blob/052d44dbfde62307291e2a91aaed81e68bd662f2/crates/did-mini/tests/historical_verification.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 2 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0299"></a>

## PR #299: Rebuild storage-fraud on audited registration; fix five defects in merged code (D-0439, D-0440, D-0441)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-05, FD-06, FD-08, FD-10, FD-14, FD-15.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/299) | [Files changed](https://github.com/mininet-labs/Mininet/pull/299/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/052d44dbfde62307291e2a91aaed81e68bd662f2)

Head `052d44dbfde62307291e2a91aaed81e68bd662f2`; base `af45860914edfe16e0fbc483b927bb2b274666d8`; merge `a9f9c96d3bb3dd9419ffce54b9b9abcbe05c3a11`. 50 changed files; 1 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Replaces an unsound fraud design and repairs several already-merged correctness failures, while preserving the original decision and documenting the unverified provenance of the received external-style review. It is a major corrective turning point.

**Mechanism and evidence.** Audited registration binds full seal/context/device/STORE delegation and historical keys; registry admission rejects duplicate replica roots, with unattributed cross-registry evidence. Shared signature limits/canonical ordering, mandatory final-layer sampling and Merkle proof-shape checks repair distinct verifier gaps. CI distinguishes a missing scanner from ordinary advisory findings.

**What remains weaker than the intended claim.** Quorum roots still do not prove independent auditors/operators, and sampling parameters remain unreviewed. Count fixes do not automatically fix all signature-byte limits or later aliases. The received review was not established as an independent qualified audit; this PR explicitly closes no external gate. Scanner JSON/exit handling still needs stricter success validation.

**Recommended improvement and rationale.** Retain every exploit regression and extend shared codec invariants across all consumer names/suites. Make verified registrations the only capacity/weight input, model corrupt quorums and set reviewed sampling bounds. Validate scanner schema and exit status, not merely JSON parsability.

**Concrete example.** A 32-key identity should sign, encode, decode and verify the same object everywhere. A later MAX_SIGS alias capped at 16 can reintroduce the old defect even while the literal-name scanner reports green.

**Acceptance tests to implement.** Reproduce copied-root framing, forged replica roots, padded proofs, signature permutation/count boundaries and scanner missing/garbage/error-JSON/advisory cases. Independently audit the entire PoRep and registration composition before economic consequences.

**History, supersession and integration.** Supersedes #297 through D-0439 without rewriting it; consolidates #298. #301 adds regression checkers, #302 challenge binding and #306 typed capacity continue the corrective chain.

**Source entry points.** [`crates/did-mini/src/delegation.rs`](https://github.com/mininet-labs/Mininet/blob/052d44dbfde62307291e2a91aaed81e68bd662f2/crates/did-mini/src/delegation.rs); [`crates/did-mini/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/052d44dbfde62307291e2a91aaed81e68bd662f2/crates/did-mini/src/error.rs); [`crates/did-mini/src/event.rs`](https://github.com/mininet-labs/Mininet/blob/052d44dbfde62307291e2a91aaed81e68bd662f2/crates/did-mini/src/event.rs); [`crates/did-mini/src/kel.rs`](https://github.com/mininet-labs/Mininet/blob/052d44dbfde62307291e2a91aaed81e68bd662f2/crates/did-mini/src/kel.rs); [`crates/did-mini/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/052d44dbfde62307291e2a91aaed81e68bd662f2/crates/did-mini/src/lib.rs); [`crates/did-mini/src/limits.rs`](https://github.com/mininet-labs/Mininet/blob/052d44dbfde62307291e2a91aaed81e68bd662f2/crates/did-mini/src/limits.rs); [`crates/did-mini/tests/delegation.rs`](https://github.com/mininet-labs/Mininet/blob/052d44dbfde62307291e2a91aaed81e68bd662f2/crates/did-mini/tests/delegation.rs); [`crates/did-mini/tests/historical_verification.rs`](https://github.com/mininet-labs/Mininet/blob/052d44dbfde62307291e2a91aaed81e68bd662f2/crates/did-mini/tests/historical_verification.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 2 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0300"></a>

## PR #300: release: harden Day-0 boundaries and exact-body finality (D-0442/D-0443)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-04, FD-05, FD-06, FD-09, FD-11, FD-14, FD-15.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/300) | [Files changed](https://github.com/mininet-labs/Mininet/pull/300/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/319035e09c55d47101a73b8a75aa807646d9d295)

Head `319035e09c55d47101a73b8a75aa807646d9d295`; base `3b9c85bc5ee40e297d5969618ff3f0e7875943ab`; merge `82c36607eb7ca6cbe90952a58cd1c70cfdd0e85d`. 47 changed files; 4 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Binds finality to the exact executed block body and repairs multiple day-zero boundaries. It prevents a valid state/QC from being reused to justify a different historical body and makes monetary plans survive wire transport.

**Mechanism and evidence.** Header v2 adds body_root; votes/QCs, execution, proposals, archives, snapshots and catch-up verify it. Message v3 carries bounded complete monetary plans and nested fields. Old outer formats fail closed. Additional fixes bound proposal work, epoch overflow, retry reasons, admission byte accounting, sealing overhead, cache privacy and SSRF address handling.

**What remains weaker than the intended claim.** This is an incompatible prelaunch migration, not a safe inference that old bodies or balances can be reconstructed from insufficient commitments. Static validator legitimacy, private claim validity, dynamic transitions, weak-device sync and external cryptography review remain unresolved. Exact-body finality authenticates agreed bytes, not their underlying economic truth.

**Recommended improvement and rationale.** Keep migration explicit: back up/quarantine old archives, agree on a governed new genesis and forbid same-network rollback after activation. Never fabricate missing old evidence. Add a consensus-verifiable shielded validity boundary and review every body/header/version change as one protocol unit.

**Concrete example.** Two bodies produce the same state but contain different claim histories. A QC for one must not authenticate the other. For v1 history lacking body commitments, the correct response is insufficient evidence, not a guessed body reconstructed from local caches.

**Acceptance tests to implement.** Test same-state/different-body substitution, omitted nested monetary fields, every old version, oversize/truncated plans, late invalid suffix with zero mutation and cross-network rollback. Reproduce full workspace and independent release artifacts at the exact migration head.

**History, supersession and integration.** Hardens #271-#274, #283, #289 and #293 after #299/#296 integration. #313 later adds nullifier ordering without claim validity; this PR's stronger body commitment does not fix that later validity seam.

**Source entry points.** [`crates/mini-chain/src/block.rs`](https://github.com/mininet-labs/Mininet/blob/319035e09c55d47101a73b8a75aa807646d9d295/crates/mini-chain/src/block.rs); [`crates/mini-chain/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/319035e09c55d47101a73b8a75aa807646d9d295/crates/mini-chain/src/lib.rs); [`crates/mini-chain/tests/finality.rs`](https://github.com/mininet-labs/Mininet/blob/319035e09c55d47101a73b8a75aa807646d9d295/crates/mini-chain/tests/finality.rs); [`crates/mini-consensus/src/catchup.rs`](https://github.com/mininet-labs/Mininet/blob/319035e09c55d47101a73b8a75aa807646d9d295/crates/mini-consensus/src/catchup.rs); [`crates/mini-consensus/src/node.rs`](https://github.com/mininet-labs/Mininet/blob/319035e09c55d47101a73b8a75aa807646d9d295/crates/mini-consensus/src/node.rs); [`crates/mini-consensus/src/snapshot.rs`](https://github.com/mininet-labs/Mininet/blob/319035e09c55d47101a73b8a75aa807646d9d295/crates/mini-consensus/src/snapshot.rs); [`crates/mini-consensus/src/snapshot_sync_tests.rs`](https://github.com/mininet-labs/Mininet/blob/319035e09c55d47101a73b8a75aa807646d9d295/crates/mini-consensus/src/snapshot_sync_tests.rs); [`crates/mini-consensus/src/state_sync.rs`](https://github.com/mininet-labs/Mininet/blob/319035e09c55d47101a73b8a75aa807646d9d295/crates/mini-consensus/src/state_sync.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 2 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0301"></a>

## PR #301: Registry integrity checks: decision collisions, lost history, restated wire limits (D-0444)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-05, FD-06, FD-10, FD-12, FD-14.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/301) | [Files changed](https://github.com/mininet-labs/Mininet/pull/301/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/bf564f1469e4c03ce849da13e070a3818101b6f7)

Head `bf564f1469e4c03ce849da13e070a3818101b6f7`; base `82c36607eb7ca6cbe90952a58cd1c70cfdd0e85d`; merge `8e7e29a4f14e62634c145bafb0f94160a6865f84`. 9 changed files; 1 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Encodes three real review incidents as automated checks: lost decision history, colliding decision numbers and inconsistent shared wire limits. This improves governance integrity without letting the checker decide legitimacy.

**Mechanism and evidence.** check_decisions compares against a canonical baseline and fails deletion/new duplicate/claim collisions; in-place edits and pre-existing duplicates warn. check_wire_limits detects selected literal constants below did-mini limits. Replay tests use historical defective commits.

**What remains weaker than the intended claim.** The checks intentionally do not detect semantic rewrites disguised as truth-sync, expression-derived limits or differently named aliases. On push with self-baseline, history comparison can be a no-op. A green structural check must not be represented as append-only semantic integrity or complete codec equivalence.

**Recommended improvement and rationale.** Bind comparisons to an independently obtained previous canonical checkpoint, separate immutable decision payload from mutable status metadata and analyze shared limits through canonical imports or a schema registry. Extend tests to aliases such as MAX_SIGS and byte-length limits, not only the four known literal names.

**Concrete example.** A new validator decoder defines MAX_SIGS=16 while did-mini supports 64. The present name-based checker can miss it; a shared codec/type or exhaustive consumer conformance test should fail.

**Acceptance tests to implement.** Replay known deletion/collision incidents and add renamed constants, computed expressions, semantic decision rewrite, self-baseline and concurrent reservations. Preserve old warnings without allowing them to normalize new violations.

**History, supersession and integration.** Follows #296/#298/#299/#300 review incidents. #319's later signature cap and current object byte caps illustrate why this narrow checker is not a complete guarantee.

**Source entry points.** [`.github/workflows/governance-policy.yml`](https://github.com/mininet-labs/Mininet/blob/bf564f1469e4c03ce849da13e070a3818101b6f7/.github/workflows/governance-policy.yml); [`docs/DECISION_LOG.md`](https://github.com/mininet-labs/Mininet/blob/bf564f1469e4c03ce849da13e070a3818101b6f7/docs/DECISION_LOG.md); [`governance/work-claims.json`](https://github.com/mininet-labs/Mininet/blob/bf564f1469e4c03ce849da13e070a3818101b6f7/governance/work-claims.json); [`tools/check_decisions.py`](https://github.com/mininet-labs/Mininet/blob/bf564f1469e4c03ce849da13e070a3818101b6f7/tools/check_decisions.py); [`tools/check_wire_limits.py`](https://github.com/mininet-labs/Mininet/blob/bf564f1469e4c03ce849da13e070a3818101b6f7/tools/check_wire_limits.py); [`tools/test_registry_checks.py`](https://github.com/mininet-labs/Mininet/blob/bf564f1469e4c03ce849da13e070a3818101b6f7/tools/test_registry_checks.py). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0302"></a>

## PR #302: Storage: proven-not-declared capacity, ongoing possession windows, and a challenge-binding fix in mini-spacetime (D-0445)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-06, FD-08, FD-11, FD-15.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/302) | [Files changed](https://github.com/mininet-labs/Mininet/pull/302/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/5be929dfe2ea8e9417c93e50763f14df86ed823d)

Head `5be929dfe2ea8e9417c93e50763f14df86ed823d`; base `8e7e29a4f14e62634c145bafb0f94160a6865f84`; merge `6b7c6ebcb0ca6793fb8347ad7fdff8515c545cb4`. 14 changed files; 3 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Adds ongoing replica lifecycle and derives capacity from audited seal size, while discovering and fixing a fundamental challenge-binding bug in the existing possession verifier.

**Mechanism and evidence.** verify_storage_challenge now receives the actual challenge and requires challenge, response and Merkle proof leaf indices to match. ReplicaLifecycle moves through degraded/active/suspended/retired windows and credits capacity only after a successful current window; challenges derive from context/window and a caller beacon.

**What remains weaker than the intended claim.** The old bug let one stored leaf/path answer every challenge; its fix is essential but not a proof of replication independence. Window time and beacon unpredictability remain caller assumptions, parameters are uncalibrated and the derived-capacity path was initially optional.

**Recommended improvement and rationale.** Bind windows to authenticated chain/epoch evidence, precommit challenge rules and require unpredictable non-reused entropy after immutable registration. Separate omission/unreachability from objective fraud; retain reversible degradation rather than punishing an offline honest device. Make verification-derived capacity mandatory at every consumer.

**Concrete example.** A prover keeps only leaf 3 and answers a challenge for leaf 7. The repaired verifier rejects it. A prover that can choose the beacon may still arrange that only retained leaves are requested, so that assumption needs a separate defense.

**Acceptance tests to implement.** Reproduce wrong-leaf/proof-index attacks, skipped windows, beacon reuse/grinding, clock rollback, repeated capacity and honest partitions. Calibrate window/grace choices using measured device/network conditions and independent cryptographic review.

**History, supersession and integration.** Continues #299 PoRep hardening; #306 makes a typed capacity input mandatory but still exposes a public construction path from unverified commitments, discussed separately.

**Source entry points.** [`crates/mini-porep/src/challenge.rs`](https://github.com/mininet-labs/Mininet/blob/5be929dfe2ea8e9417c93e50763f14df86ed823d/crates/mini-porep/src/challenge.rs); [`crates/mini-spacetime/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/5be929dfe2ea8e9417c93e50763f14df86ed823d/crates/mini-spacetime/src/lib.rs); [`crates/mini-spacetime/src/storage_proof.rs`](https://github.com/mininet-labs/Mininet/blob/5be929dfe2ea8e9417c93e50763f14df86ed823d/crates/mini-spacetime/src/storage_proof.rs); [`crates/mini-storage-fraud/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/5be929dfe2ea8e9417c93e50763f14df86ed823d/crates/mini-storage-fraud/src/error.rs); [`crates/mini-storage-fraud/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/5be929dfe2ea8e9417c93e50763f14df86ed823d/crates/mini-storage-fraud/src/lib.rs); [`crates/mini-storage-fraud/src/lifecycle.rs`](https://github.com/mininet-labs/Mininet/blob/5be929dfe2ea8e9417c93e50763f14df86ed823d/crates/mini-storage-fraud/src/lifecycle.rs); [`crates/mini-storage-fraud/tests/lifecycle.rs`](https://github.com/mininet-labs/Mininet/blob/5be929dfe2ea8e9417c93e50763f14df86ed823d/crates/mini-storage-fraud/tests/lifecycle.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 1 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0304"></a>

## PR #304: Mininet Node Appliance: full Day 0 lifecycle, arm64, and an operator guide (D-0446)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-01, FD-02, FD-03, FD-06, FD-09, FD-11, FD-18.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/304) | [Files changed](https://github.com/mininet-labs/Mininet/pull/304/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/d33bd00f2cce61fa1c7810c36daff6567e90053c)

Head `d33bd00f2cce61fa1c7810c36daff6567e90053c`; base `f1169ba3914c3fe6e03d1b88ba24b9bc0c7a250c`; merge `21ec9d469ae155e256db1b7cc8e3c979f254d175`. 28 changed files; 4 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Adds an operator-facing appliance lifecycle on Debian rather than inventing an operating system: preflight, install, verification, backup, restore and uninstall for x86-64/ARM64. Local exit and backup are important sovereignty features.

**Mechanism and evidence.** An unprivileged service, explicit owner installation, local passphrase-encrypted archive, bounded journald retention and report-only verification timer avoid a fleet-control service. Uninstall retains identity unless explicitly purged; package-lock generation is a separate target-specific step.

**What remains weaker than the intended claim.** Current backup batch mode places the passphrase in gpg command arguments; restore extracts as root, then deletes the old state before moving the new directory, without a transaction/rollback across that boundary. The firewall takes over the full ruleset. Live-state consistency, archive member safety and real hardware acceptance need more than shell linting.

**Recommended improvement and rationale.** Pass secrets through a protected descriptor rather than argv, take a consistent stopped/snapshotted backup and validate all archive members/types before extraction. Restore to a validated staging directory, preserve the old state until atomic activation succeeds and provide rollback. Limit firewall changes to an explicit owner-approved namespace.

**Concrete example.** A restore crosses filesystems or loses power after rm -rf of the live state but before mv completes. The user can lose their only current identity despite a valid encrypted backup; transaction staging must keep a recoverable old copy until completion.

**Acceptance tests to implement.** Test secret visibility in process listings, traversal/symlink/hardlink archive entries, disk-full/cross-device rename, live backup consistency, interrupted restore and uninstall preservation. Boot independent ARM64/x86-64 hardware and demonstrate no central console or forced repair.

**History, supersession and integration.** D-0446 follows storage/installer primitives; it does not create recovery escrow or production signed images. Two copies of a key create an equivocation risk, not automatic proof of double-signing merely by existing.

**Source entry points.** [`README.md`](https://github.com/mininet-labs/Mininet/blob/d33bd00f2cce61fa1c7810c36daff6567e90053c/README.md); [`deploy/README.md`](https://github.com/mininet-labs/Mininet/blob/d33bd00f2cce61fa1c7810c36daff6567e90053c/deploy/README.md); [`deploy/backup/backup.sh`](https://github.com/mininet-labs/Mininet/blob/d33bd00f2cce61fa1c7810c36daff6567e90053c/deploy/backup/backup.sh); [`deploy/backup/restore.sh`](https://github.com/mininet-labs/Mininet/blob/d33bd00f2cce61fa1c7810c36daff6567e90053c/deploy/backup/restore.sh); [`deploy/image/README.md`](https://github.com/mininet-labs/Mininet/blob/d33bd00f2cce61fa1c7810c36daff6567e90053c/deploy/image/README.md); [`deploy/installer/appliance.conf.example`](https://github.com/mininet-labs/Mininet/blob/d33bd00f2cce61fa1c7810c36daff6567e90053c/deploy/installer/appliance.conf.example); [`deploy/installer/install.sh`](https://github.com/mininet-labs/Mininet/blob/d33bd00f2cce61fa1c7810c36daff6567e90053c/deploy/installer/install.sh); [`deploy/installer/preflight.sh`](https://github.com/mininet-labs/Mininet/blob/d33bd00f2cce61fa1c7810c36daff6567e90053c/deploy/installer/preflight.sh). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 1 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0305"></a>

## PR #305: mini-private-payment: the shielded settlement path (D-0447)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-04, FD-05, FD-09, FD-11, FD-14, FD-16.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/305) | [Files changed](https://github.com/mininet-labs/Mininet/pull/305/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/e2017e03854fdf948bd13bda7b08875b602098ac)

Head `e2017e03854fdf948bd13bda7b08875b602098ac`; base `6b7c6ebcb0ca6793fb8347ad7fdff8515c545cb4`; merge `f1169ba3914c3fe6e03d1b88ba24b9bc0c7a250c`. 28 changed files; 3 commits; 0 issue comments, 3 inline comments and 1 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Composes stealth outputs, ring signatures and range proofs into the first real private-payment object, addressing the transparent payer/payee graph in the existing contribution/settlement path.

**Mechanism and evidence.** PrivatePaymentClaim omits stable payer/sequence, uses a key image for conflicts and an encrypted memo. Split binding/signing transcripts avoid circular memo authentication. mini-value gains range-proof serialization, usable stealth shared-secret access and a verifier-only ring API. Existing settlement states are reused.

**What remains weaker than the intended claim.** This first version proves output range but not value conservation and has no fees/change or chain-backed ledger. Real primitives do not make their composition a complete payment system. Wire tests looking for literal amounts/IDs are useful leak regressions, not proofs of statistical anonymity or network privacy.

**Recommended improvement and rationale.** Keep v1 unusable for real value and require conservation, valid ledger membership, bounded anonymous admission and canonical finality before integration. Analyze memo/transcript composition and recipient spendability independently; do not market an opaque amount as a checked amount.

**Concrete example.** A payer can commit an arbitrary in-range output amount when no input/output balance equation exists. The payment looks confidential while the verifier has no conservation rule to reject inflation.

**Acceptance tests to implement.** Test fabricated funding, arbitrary output amounts, memo substitution, wrong recipient, malformed points/proofs, repeated key images and recipient onward spending. Add independent cryptographic vectors and network-level privacy experiments.

**History, supersession and integration.** #307 changes default decoy selection, #308 disclosure/scan resilience, #312 adds RingCT conservation and rejects v1, and #313 adds ordering. Each closes a different gap; #313 still lacks chain-verifiable claim validity.

**Source entry points.** [`crates/mini-private-payment/src/claim.rs`](https://github.com/mininet-labs/Mininet/blob/e2017e03854fdf948bd13bda7b08875b602098ac/crates/mini-private-payment/src/claim.rs); [`crates/mini-private-payment/src/codec.rs`](https://github.com/mininet-labs/Mininet/blob/e2017e03854fdf948bd13bda7b08875b602098ac/crates/mini-private-payment/src/codec.rs); [`crates/mini-private-payment/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/e2017e03854fdf948bd13bda7b08875b602098ac/crates/mini-private-payment/src/error.rs); [`crates/mini-private-payment/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/e2017e03854fdf948bd13bda7b08875b602098ac/crates/mini-private-payment/src/lib.rs); [`crates/mini-private-payment/src/memo.rs`](https://github.com/mininet-labs/Mininet/blob/e2017e03854fdf948bd13bda7b08875b602098ac/crates/mini-private-payment/src/memo.rs); [`crates/mini-private-payment/src/nullifier.rs`](https://github.com/mininet-labs/Mininet/blob/e2017e03854fdf948bd13bda7b08875b602098ac/crates/mini-private-payment/src/nullifier.rs); [`crates/mini-private-payment/src/reconcile.rs`](https://github.com/mininet-labs/Mininet/blob/e2017e03854fdf948bd13bda7b08875b602098ac/crates/mini-private-payment/src/reconcile.rs); [`crates/mini-private-payment/src/scan.rs`](https://github.com/mininet-labs/Mininet/blob/e2017e03854fdf948bd13bda7b08875b602098ac/crates/mini-private-payment/src/scan.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 2 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0306"></a>

## PR #306: Proven capacity is the only capacity: typed ProvenCapacity, enforced block size (D-0448)

**FAIL** | Captured outcome: **merged** | Directives: FD-02, FD-06, FD-08, FD-11, FD-14, FD-16.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/306) | [Files changed](https://github.com/mininet-labs/Mininet/pull/306/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/65f5ea75dce7cddaed617875e758bc048ec27d3e)

Head `65f5ea75dce7cddaed617875e758bc048ec27d3e`; base `21ec9d469ae155e256db1b7cc8e3c979f254d175`; merge `f4c0fcbf938d186c6f7111938039ddea9b61ad02`. 15 changed files; 3 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Replaces bare numeric capacity parameters with a named ProvenCapacity and enforces declared block size during actual challenges. The objective is correct, but the public type does not prove the verification property its name/documents claim.

**Mechanism and evidence.** StorageCommitment gains block_size_bytes; proof responses must match it. proposer_weight requires ProvenCapacity. However, public ProvenCapacity::from_commitment accepts a public caller-constructible StorageCommitment and computes count times size without checking any proof or current possession window.

**What remains weaker than the intended claim.** A caller can construct an arbitrary large commitment and immediately obtain the supposedly proven type. The type is Copy and addition can double-count one replica; the code now explicitly documents the latter limit. This is a confirmed API trust-boundary failure, not a claim that a deployed live consensus weighting exploit has been demonstrated.

**Recommended improvement and rationale.** Split DeclaredCapacity from VerifiedWindowCapacity. Construct the verified type only inside successful registration/current-window verification, bind replica/network/policy/window and deduplicate a set of verified contributions before summing. Keep size enforcement but stop using an unverified commitment as a proof token.

**Concrete example.** Construct StorageCommitment with an arbitrary root, huge block_count and block_size_bytes, then call ProvenCapacity::from_commitment. No storage response is needed to reach proposer_weight under the public API.

**Acceptance tests to implement.** Add a compile-time/API regression showing unverified commitments cannot create the weight token, and runtime tests for duplicate replicas, stale windows, wrong policy/network, inconsistent block sizes and sums. Trace the actual weighting consumer before claiming production exploitability.

**History, supersession and integration.** Attempts to make #302's defense mandatory; a later verified registry/lifecycle caller may be safe, but it does not repair the generic public constructor. Storage operator independence remains a separate unresolved property.

**Source entry points.** [`crates/mini-porep/src/challenge.rs`](https://github.com/mininet-labs/Mininet/blob/65f5ea75dce7cddaed617875e758bc048ec27d3e/crates/mini-porep/src/challenge.rs); [`crates/mini-spacetime/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/65f5ea75dce7cddaed617875e758bc048ec27d3e/crates/mini-spacetime/src/lib.rs); [`crates/mini-spacetime/src/proof.rs`](https://github.com/mininet-labs/Mininet/blob/65f5ea75dce7cddaed617875e758bc048ec27d3e/crates/mini-spacetime/src/proof.rs); [`crates/mini-spacetime/src/storage_proof.rs`](https://github.com/mininet-labs/Mininet/blob/65f5ea75dce7cddaed617875e758bc048ec27d3e/crates/mini-spacetime/src/storage_proof.rs); [`crates/mini-spacetime/src/weight.rs`](https://github.com/mininet-labs/Mininet/blob/65f5ea75dce7cddaed617875e758bc048ec27d3e/crates/mini-spacetime/src/weight.rs); [`crates/mini-storage-fraud/src/lifecycle.rs`](https://github.com/mininet-labs/Mininet/blob/65f5ea75dce7cddaed617875e758bc048ec27d3e/crates/mini-storage-fraud/src/lifecycle.rs); [`crates/mini-storage-fraud/src/seal.rs`](https://github.com/mininet-labs/Mininet/blob/65f5ea75dce7cddaed617875e758bc048ec27d3e/crates/mini-storage-fraud/src/seal.rs); [`crates/mini-storage-fraud/tests/lifecycle.rs`](https://github.com/mininet-labs/Mininet/blob/65f5ea75dce7cddaed617875e758bc048ec27d3e/crates/mini-storage-fraud/tests/lifecycle.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 1 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0307"></a>

## PR #307: Decoy selection is a protocol rule (D-0449) + Wasmtime sandbox-escape fix and CI repair (D-0450)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-06, FD-09, FD-10, FD-11, FD-14, FD-15.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/307) | [Files changed](https://github.com/mininet-labs/Mininet/pull/307/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/5156bcaae1e7759d2572eba7207d804946a45753)

Head `5156bcaae1e7759d2572eba7207d804946a45753`; base `21ec9d469ae155e256db1b7cc8e3c979f254d175`; merge `932b3608fdb379d61efc8251b1279cd9b49d636d`. 29 changed files; 8 commits; 1 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Moves decoy selection into the default payment builder and standardizes sampling to reduce wallet fingerprint differences. It also records a sandbox-runtime patch and corrects shell error handling in dependency scanning.

**Mechanism and evidence.** build selects rings from a local OutputSet using integer age-bucket weights and fresh entropy rather than accepting an arbitrary ring field. The tunable minimum rises to sixteen above a frozen floor of eight. Golden fixtures are decoupled from tunables. The CI patch clears inherited bash errexit before interpreting scanner results.

**What remains weaker than the intended claim.** The distribution is a judgment, not fitted/validated traffic evidence. A malicious wallet can still encode a poor ring; that can harm other users through decoy elimination and intersection analysis, not only its own user. Local output-set storage is a genuine weak-device burden. Scanner success still requires schema/exit validation beyond clearing errexit.

**Recommended improvement and rationale.** Distinguish consensus-enforceable ring constraints from default wallet policy. Independently analyze decoy distributions, poisoning/intersection and timing with synthetic/adversarial traffic before deployment, then govern measured updates. Preserve private local data access or evaluate a reviewed privacy-preserving retrieval alternative rather than expose targeted queries.

**Concrete example.** An attacker creates and later reveals spends of many outputs used as other users' decoys. Their wallet's bad privacy decisions can shrink other rings; the claim that only that user is harmed is too strong.

**Acceptance tests to implement.** Test rejection sampling, age extremes, poisoned output sets, deterministic cross-platform vectors, malicious custom encoders and memory limits. Reproduce scanner missing/error/advisory branches and advisory-specific sandbox regressions separately.

**History, supersession and integration.** Extends #305; #312 later changes spend proofs while retaining sampling concerns. The reported runtime advisory update is historical repository evidence, not an independent assertion that all sandbox vulnerabilities are closed.

**Source entry points.** [`crates/mini-porep/src/challenge.rs`](https://github.com/mininet-labs/Mininet/blob/5156bcaae1e7759d2572eba7207d804946a45753/crates/mini-porep/src/challenge.rs); [`crates/mini-private-payment/src/claim.rs`](https://github.com/mininet-labs/Mininet/blob/5156bcaae1e7759d2572eba7207d804946a45753/crates/mini-private-payment/src/claim.rs); [`crates/mini-private-payment/src/decoy.rs`](https://github.com/mininet-labs/Mininet/blob/5156bcaae1e7759d2572eba7207d804946a45753/crates/mini-private-payment/src/decoy.rs); [`crates/mini-private-payment/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/5156bcaae1e7759d2572eba7207d804946a45753/crates/mini-private-payment/src/error.rs); [`crates/mini-private-payment/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/5156bcaae1e7759d2572eba7207d804946a45753/crates/mini-private-payment/src/lib.rs); [`crates/mini-private-payment/tests/adversarial.rs`](https://github.com/mininet-labs/Mininet/blob/5156bcaae1e7759d2572eba7207d804946a45753/crates/mini-private-payment/tests/adversarial.rs); [`crates/mini-private-payment/tests/support/mod.rs`](https://github.com/mininet-labs/Mininet/blob/5156bcaae1e7759d2572eba7207d804946a45753/crates/mini-private-payment/tests/support/mod.rs); [`crates/mini-private-payment/tests/unity.rs`](https://github.com/mininet-labs/Mininet/blob/5156bcaae1e7759d2572eba7207d804946a45753/crates/mini-private-payment/tests/unity.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 2 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0308"></a>

## PR #308: Auditability is a disclosure a party makes about itself, not a property of the payment format (D-0450)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-01, FD-04, FD-09, FD-14, FD-16.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/308) | [Files changed](https://github.com/mininet-labs/Mininet/pull/308/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/aef98426a5d200f8768dd449774c0967a4967b95)

Head `aef98426a5d200f8768dd449774c0967a4967b95`; base `f4c0fcbf938d186c6f7111938039ddea9b61ad02`; merge `55959f6cf832f502766474bdd1f2bdb0291257df`. 28 changed files; 9 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Chooses voluntary account disclosure over a permanently transparent payment format, and repairs a scan availability bug where one recognizable but unreadable payment could hide all legitimate income.

**Mechanism and evidence.** ViewKeyDisclosure requires an explicit irreversible-disclosure acknowledgment; private fields discourage accidental construction. audit reuses scan, which now returns readable and unreadable-recognized claims separately. Canonical scalar and collapsed spend/view-key checks constrain disclosure encoding.

**What remains weaker than the intended claim.** A disclosed view key reveals historical recipient information and memo content involving senders who did not consent. A typed phrase is user friction, not third-party consent or a cryptographic policy boundary. The initial path does not open amounts, prove outflows or prove the discloser has no other accounts; transparent payments are not removed here.

**Recommended improvement and rationale.** Prefer scoped per-output disclosures where feasible, separate account-ownership proof from view-key consistency and show precise retroactive exposure before publication. Keep unreadable recognized payments visible without treating them as fraud. Retire transparent production paths only with a complete migration and valid private settlement.

**Concrete example.** A stranger sends one validly addressed output with a memo the recipient cannot open. Scanning should retain all other payments and report that one unreadable entry, not fail the whole wallet. Publishing the view key can expose old sender memos.

**Acceptance tests to implement.** Test malicious memos, mismatched spend/view keys, malformed/non-canonical scalars, replayed disclosures, mixed readable/unreadable histories and irreversible exposure notices. Do not label income visibility as complete account balance auditing.

**History, supersession and integration.** Extends #305/#307; #313 adds amount openings. R3 transparent retirement remains separate, and disclosure does not close cryptographic or counterparty-privacy review.

**Source entry points.** [`crates/mini-private-payment/src/claim.rs`](https://github.com/mininet-labs/Mininet/blob/aef98426a5d200f8768dd449774c0967a4967b95/crates/mini-private-payment/src/claim.rs); [`crates/mini-private-payment/src/decoy.rs`](https://github.com/mininet-labs/Mininet/blob/aef98426a5d200f8768dd449774c0967a4967b95/crates/mini-private-payment/src/decoy.rs); [`crates/mini-private-payment/src/disclosure.rs`](https://github.com/mininet-labs/Mininet/blob/aef98426a5d200f8768dd449774c0967a4967b95/crates/mini-private-payment/src/disclosure.rs); [`crates/mini-private-payment/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/aef98426a5d200f8768dd449774c0967a4967b95/crates/mini-private-payment/src/error.rs); [`crates/mini-private-payment/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/aef98426a5d200f8768dd449774c0967a4967b95/crates/mini-private-payment/src/lib.rs); [`crates/mini-private-payment/src/scan.rs`](https://github.com/mininet-labs/Mininet/blob/aef98426a5d200f8768dd449774c0967a4967b95/crates/mini-private-payment/src/scan.rs); [`crates/mini-private-payment/tests/adversarial.rs`](https://github.com/mininet-labs/Mininet/blob/aef98426a5d200f8768dd449774c0967a4967b95/crates/mini-private-payment/tests/adversarial.rs); [`crates/mini-private-payment/tests/disclosure.rs`](https://github.com/mininet-labs/Mininet/blob/aef98426a5d200f8768dd449774c0967a4967b95/crates/mini-private-payment/tests/disclosure.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 1 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0309"></a>

## PR #309: A governance validator must read data, not commentary: the exceptions scan counted a commented-out example (D-0452)

**PASS** | Captured outcome: **merged** | Directives: FD-05, FD-06, FD-10, FD-12, FD-14.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/309) | [Files changed](https://github.com/mininet-labs/Mininet/pull/309/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/8e903a909d3a813ff1666548f867161f502e09b9)

Head `8e903a909d3a813ff1666548f867161f502e09b9`; base `55959f6cf832f502766474bdd1f2bdb0291257df`; merge `201cd5bf06d44f2629fd8d4f54da96173d7773c2`. 8 changed files; 4 commits; 1 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Fixes a governance validator that mistook a commented template expiry for a live exception and restores deterministic time injection. It also adds tests for the actual sunset guard over founder-only integration.

**Mechanism and evidence.** The scan removes comment lines, uses supplied now instead of ambient date and gives the example a placeholder date. Tests separately cover structural fixtures and the real committed exception expiry so dated test data does not mask a genuinely lapsed exception.

**What remains weaker than the intended claim.** The scoped PASS is for the demonstrated parsing/time bug. Comment stripping plus regex is not a complete YAML parser, and neutralizing fixture dates can hide real expiry if the dedicated live check is later removed. This change does not extend D-0083 or resolve centralized governance.

**Recommended improvement and rationale.** Use a strict small structured format or schema-validated YAML rather than progressively expanding regex semantics. Keep an independent live-state sunset check and an explicit bounded remediation process for canonical-validator failures; never bypass a real expired authority exception to make CI green.

**Concrete example.** A commented expires: date must never revoke authority or fail every PR. An actual expired founder exception must fail at the live policy boundary even while historical structural fixtures remain reproducible.

**Acceptance tests to implement.** Test comments, quoted strings, malformed dates, future/effective boundaries, injected time and the real operating state. Delete the live expiry check in a mutation test and require a protection test to fail.

**History, supersession and integration.** Corrects the calendar failure exposed after #307. #310/#311 track the repair and #317 later records a scheduled run; none changes the underlying founder-governance sunset.

**Source entry points.** [`docs/DECISION_LOG.md`](https://github.com/mininet-labs/Mininet/blob/8e903a909d3a813ff1666548f867161f502e09b9/docs/DECISION_LOG.md); [`governance/exceptions.yml`](https://github.com/mininet-labs/Mininet/blob/8e903a909d3a813ff1666548f867161f502e09b9/governance/exceptions.yml); [`governance/work-claims.json`](https://github.com/mininet-labs/Mininet/blob/8e903a909d3a813ff1666548f867161f502e09b9/governance/work-claims.json); [`tools/check_governance.py`](https://github.com/mininet-labs/Mininet/blob/8e903a909d3a813ff1666548f867161f502e09b9/tools/check_governance.py); [`tools/test_check_governance.py`](https://github.com/mininet-labs/Mininet/blob/8e903a909d3a813ff1666548f867161f502e09b9/tools/test_check_governance.py). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0310"></a>

## PR #310: A published critical path to release, kept honest by a checker (D-0453)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-01, FD-05, FD-10, FD-12, FD-14, FD-17.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/310) | [Files changed](https://github.com/mininet-labs/Mininet/pull/310/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/8dbd84149fc1fe7f6fa477846623997d4835e1b5)

Head `8dbd84149fc1fe7f6fa477846623997d4835e1b5`; base `55959f6cf832f502766474bdd1f2bdb0291257df`; merge `148df7526d3e32402a2b94f6bae8c50b987f5f69`. 14 changed files; 6 commits; 1 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Publishes an ordered release-critical path with explicit engineering and outside gates, giving outsiders an honest way to see how far the project is from real people/value deployment.

**Mechanism and evidence.** ROADMAP_TO_RELEASE lists twenty items with statuses, blockers and closure criteria. check_roadmap validates status vocabulary, decision references, blocker IDs and README/detail counts. Done requires a D-number that exists.

**What remains weaker than the intended claim.** An existing decision citation does not prove the work is complete or externally authorized. The checker verifies internal consistency, not truth. The initial summary omitted completed items and therefore became inaccurate on the first completion, fixed in #311. Phase labels can still pressure premature closure.

**Recommended improvement and rationale.** Bind done to exact evidence type and scope, not merely a number: implementation test, independent review, physical measurement or governance activation. Preserve outside-gate ownership and explicit non-substitution. Show both completed and unresolved work without a misleading percentage of production readiness.

**Concrete example.** A row marked done citing a real decision about a protocol type must not imply that the runtime or physical deployment is complete. The closure evidence should name the tested end-to-end boundary.

**Acceptance tests to implement.** Test malformed/circular blockers, missing citations and count drift; independently review a sample of done rows against code and evidence. Include negative tests where the citation exists but the gate type is wrong.

**History, supersession and integration.** Builds on #99's gate register and the money-layer sequence; #311 fixes counting, #317 closes a narrow CI criterion. R8/R9 must remain open for validity and integration gaps despite completed sub-PRs.

**Source entry points.** [`.github/workflows/governance-policy.yml`](https://github.com/mininet-labs/Mininet/blob/8dbd84149fc1fe7f6fa477846623997d4835e1b5/.github/workflows/governance-policy.yml); [`README.md`](https://github.com/mininet-labs/Mininet/blob/8dbd84149fc1fe7f6fa477846623997d4835e1b5/README.md); [`docs/DECISION_LOG.md`](https://github.com/mininet-labs/Mininet/blob/8dbd84149fc1fe7f6fa477846623997d4835e1b5/docs/DECISION_LOG.md); [`docs/ROADMAP_TO_RELEASE.md`](https://github.com/mininet-labs/Mininet/blob/8dbd84149fc1fe7f6fa477846623997d4835e1b5/docs/ROADMAP_TO_RELEASE.md); [`docs/STATUS.md`](https://github.com/mininet-labs/Mininet/blob/8dbd84149fc1fe7f6fa477846623997d4835e1b5/docs/STATUS.md); [`governance/exceptions.yml`](https://github.com/mininet-labs/Mininet/blob/8dbd84149fc1fe7f6fa477846623997d4835e1b5/governance/exceptions.yml); [`governance/work-claims.json`](https://github.com/mininet-labs/Mininet/blob/8dbd84149fc1fe7f6fa477846623997d4835e1b5/governance/work-claims.json); [`tools/check_governance.py`](https://github.com/mininet-labs/Mininet/blob/8dbd84149fc1fe7f6fa477846623997d4835e1b5/tools/check_governance.py). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0311"></a>

## PR #311: Count completed roadmap items on the front page; R1 done, R2 active (D-0454)

**PASS** | Captured outcome: **merged** | Directives: FD-01, FD-05, FD-10, FD-14.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/311) | [Files changed](https://github.com/mininet-labs/Mininet/pull/311/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/7ca6d1125d3ef5e476eb93869af1d16fc1ac3531)

Head `7ca6d1125d3ef5e476eb93869af1d16fc1ac3531`; base `148df7526d3e32402a2b94f6bae8c50b987f5f69`; merge `7c5153dd62cf802f30a5d0d2d2c206a09a34c2cd`. 8 changed files; 2 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Fixes the roadmap summary the first time completion made its arithmetic wrong and refuses to close CI readiness before the specific scheduled-run evidence exists.

**Mechanism and evidence.** The README/checker count done alongside all open statuses. R1 is closed with the merged validator-fix evidence; R2 becomes active, not done. A new append-only D-0454 records the correction rather than editing D-0453 in place.

**What remains weaker than the intended claim.** The scoped PASS covers truthful counting/status discipline. Arithmetic consistency still does not establish semantic completion, and future status additions need schema-driven totals. The author's own nearly-made append-only edit is a useful incident, not evidence of automatic enforcement.

**Recommended improvement and rationale.** Derive summaries from the canonical row schema and distinguish status evidence from roadmap interpretation. Keep historical decisions immutable and record corrections through explicit supersessions.

**Concrete example.** Twenty rows with one done must still sum to twenty. A green push build cannot satisfy an exit criterion requiring a scheduled build against moving dependencies/advisories.

**Acceptance tests to implement.** Test every status in isolation and mixed totals, unknown new statuses, omitted summary blocks and decision deletion. Verify the cited run type and exact SHA before accepting a status transition.

**History, supersession and integration.** Corrects #310; #317 later provides the scheduled-run evidence for R2. Neither status operation certifies the protocol or external gate readiness.

**Source entry points.** [`README.md`](https://github.com/mininet-labs/Mininet/blob/7ca6d1125d3ef5e476eb93869af1d16fc1ac3531/README.md); [`docs/DECISION_LOG.md`](https://github.com/mininet-labs/Mininet/blob/7ca6d1125d3ef5e476eb93869af1d16fc1ac3531/docs/DECISION_LOG.md); [`docs/ROADMAP_TO_RELEASE.md`](https://github.com/mininet-labs/Mininet/blob/7ca6d1125d3ef5e476eb93869af1d16fc1ac3531/docs/ROADMAP_TO_RELEASE.md); [`governance/work-claims.json`](https://github.com/mininet-labs/Mininet/blob/7ca6d1125d3ef5e476eb93869af1d16fc1ac3531/governance/work-claims.json); [`tools/check_roadmap.py`](https://github.com/mininet-labs/Mininet/blob/7ca6d1125d3ef5e476eb93869af1d16fc1ac3531/tools/check_roadmap.py); [`tools/test_check_roadmap.py`](https://github.com/mininet-labs/Mininet/blob/7ca6d1125d3ef5e476eb93869af1d16fc1ac3531/tools/test_check_roadmap.py). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0312"></a>

## PR #312: D-0455/D-0456: value conservation for the shielded payment path (RingCT balance, fees, change) — closes roadmap R4

**PARTIAL** | Captured outcome: **merged** | Directives: FD-04, FD-05, FD-06, FD-09, FD-11, FD-14, FD-16.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/312) | [Files changed](https://github.com/mininet-labs/Mininet/pull/312/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/225ae79013bda08ccf32bccab0598e5b36bfbb19)

Head `225ae79013bda08ccf32bccab0598e5b36bfbb19`; base `7c5153dd62cf802f30a5d0d2d2c206a09a34c2cd`; merge `5a97e3fdad34c09bd469fa41b03207d51acfe6ed`. 29 changed files; 2 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Adds the missing conservation proof to private payments and enables fees, change and onward spending. This is the difference between hiding an arbitrary number and verifying a confidential transfer.

**Mechanism and evidence.** Two-column MLSAG binds one-time spend-key ownership and a zero-valued commitment difference at the same hidden ring index. Reblinded pseudo-outputs hide the true input commitment; range proofs precede the balance equation. Multi-input/output claims include a public fee, encrypted amount/blinding notes and v2 encoding that rejects unconserved v1.

**What remains weaker than the intended claim.** This is an in-house unaudited implementation of an established construction, not inherited assurance from its paper or another project. It still needs authenticated ledger membership and canonical execution. Fees, input/output counts, decoy distribution and timing leak structure. Debug performance changes preserve assertions but do not measure weak-device production cost.

**Recommended improvement and rationale.** Independently verify transcript domains, generators, scalar/point canonicality, key-image linkage, input membership and balance arithmetic. Use differential/reference vectors where suite-compatible and algebraic/model tests where not. Keep legacy v1 unspendable and migrate every consumer only after the whole validity path exists.

**Concrete example.** A claim with one valid input and an output larger than that input must fail even if every output range proof verifies. Two claims overlapping on only their second input must conflict atomically; checking the first key image is insufficient.

**Acceptance tests to implement.** Test arbitrary encoders, negative/overflow amounts, swapped pseudo-commitments, duplicate inputs, forged ledger members, malformed points, fee tampering, multi-input conflicts and receive-then-spend. Run side-channel/nonce review and worst-case verifier benchmarks independently.

**History, supersession and integration.** Corrects #305's missing balance equation; #313 adds canonical nullifier ordering but not independent claim validity. R4 engineering completion does not close A1/R12 cryptographic audit.

**Source entry points.** [`crates/mini-private-payment/src/claim.rs`](https://github.com/mininet-labs/Mininet/blob/225ae79013bda08ccf32bccab0598e5b36bfbb19/crates/mini-private-payment/src/claim.rs); [`crates/mini-private-payment/src/decoy.rs`](https://github.com/mininet-labs/Mininet/blob/225ae79013bda08ccf32bccab0598e5b36bfbb19/crates/mini-private-payment/src/decoy.rs); [`crates/mini-private-payment/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/225ae79013bda08ccf32bccab0598e5b36bfbb19/crates/mini-private-payment/src/error.rs); [`crates/mini-private-payment/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/225ae79013bda08ccf32bccab0598e5b36bfbb19/crates/mini-private-payment/src/lib.rs); [`crates/mini-private-payment/src/memo.rs`](https://github.com/mininet-labs/Mininet/blob/225ae79013bda08ccf32bccab0598e5b36bfbb19/crates/mini-private-payment/src/memo.rs); [`crates/mini-private-payment/src/nullifier.rs`](https://github.com/mininet-labs/Mininet/blob/225ae79013bda08ccf32bccab0598e5b36bfbb19/crates/mini-private-payment/src/nullifier.rs); [`crates/mini-private-payment/src/reconcile.rs`](https://github.com/mininet-labs/Mininet/blob/225ae79013bda08ccf32bccab0598e5b36bfbb19/crates/mini-private-payment/src/reconcile.rs); [`crates/mini-private-payment/src/scan.rs`](https://github.com/mininet-labs/Mininet/blob/225ae79013bda08ccf32bccab0598e5b36bfbb19/crates/mini-private-payment/src/scan.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 2 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0313"></a>

## PR #313: D-0457/D-0458: finish Phase 2's money layer — chain-backed shielded finality and amount disclosure (closes R5, R6; unblocks R3)

**FAIL** | Captured outcome: **merged** | Directives: FD-04, FD-05, FD-09, FD-14, FD-16.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/313) | [Files changed](https://github.com/mininet-labs/Mininet/pull/313/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/b31f52050c604f5c9338f83cc35a50daa4fe78ad)

Head `b31f52050c604f5c9338f83cc35a50daa4fe78ad`; base `5a97e3fdad34c09bd469fa41b03207d51acfe6ed`; merge `4d00f7d2408c83beb0fe889575eb316236d15196`. 23 changed files; 1 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Adds real canonical ordering for shielded key images and more useful amount disclosure, but leaves a release-blocking validity gap. Ordering arbitrary facts is not enough to protect ownership.

**Mechanism and evidence.** mini-execution records NullifierRecord(key_image, claim_digest) atomically per digest; ChainBackedPrivateLedger reads the finalized map. New snapshot/state/body versions retain nullifiers. AmountDisclosure opens chosen Pedersen commitments, and AuditedIncome reports recognized-but-unopened entries alongside the sum.

**What remains weaker than the intended claim.** The chain does not independently verify the private claim that authorizes a nullifier. A malicious proposer can preempt a visible legitimate key image with a wrong digest and cause honest nodes to finalize the conflict. The claimed crate-dependency interpretation of the voice/value wall must not force validators to trust proposers; verifying validity is not weighting votes by wealth.

**Recommended improvement and rationale.** Introduce a consensus-verifiable neutral validation boundary binding full claim, finalized input set, network/version, conservation and nullifiers before any vote/state mutation. Keep vote eligibility/weight independent of balances. Reconcile the module-policy interpretation through explicit human review rather than accepting an unsafe omission as constitutional necessity.

**Concrete example.** A legitimate spend appears in the mempool. A proposer copies its key image but substitutes another digest into a NullifierRecord. The current ordering layer can burn that claim's canonical opportunity without proving authorization; invalid records must instead be rejected by every validator.

**Acceptance tests to implement.** Construct invalid/missing-proof records, visible-key-image front-running, wrong ledger membership, cross-network replay and partial multi-input conflicts. Require zero live/archive mutation and no QC vote for invalid bodies. For disclosure test omissions, invalid openings, overflow and account completeness boundaries.

**History, supersession and integration.** Builds on #312 and #300. R5 ordering and R6 income-opening functionality exist, but R8 private validity and R3 transparent retirement remain open; external audit must review the composed boundary.

**Source entry points.** [`crates/mini-execution/src/body.rs`](https://github.com/mininet-labs/Mininet/blob/b31f52050c604f5c9338f83cc35a50daa4fe78ad/crates/mini-execution/src/body.rs); [`crates/mini-execution/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/b31f52050c604f5c9338f83cc35a50daa4fe78ad/crates/mini-execution/src/error.rs); [`crates/mini-execution/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/b31f52050c604f5c9338f83cc35a50daa4fe78ad/crates/mini-execution/src/lib.rs); [`crates/mini-execution/src/nullifier.rs`](https://github.com/mininet-labs/Mininet/blob/b31f52050c604f5c9338f83cc35a50daa4fe78ad/crates/mini-execution/src/nullifier.rs); [`crates/mini-execution/src/snapshot.rs`](https://github.com/mininet-labs/Mininet/blob/b31f52050c604f5c9338f83cc35a50daa4fe78ad/crates/mini-execution/src/snapshot.rs); [`crates/mini-execution/src/state.rs`](https://github.com/mininet-labs/Mininet/blob/b31f52050c604f5c9338f83cc35a50daa4fe78ad/crates/mini-execution/src/state.rs); [`crates/mini-execution/tests/shielded_ordering.rs`](https://github.com/mininet-labs/Mininet/blob/b31f52050c604f5c9338f83cc35a50daa4fe78ad/crates/mini-execution/tests/shielded_ordering.rs); [`crates/mini-private-payment/src/amount.rs`](https://github.com/mininet-labs/Mininet/blob/b31f52050c604f5c9338f83cc35a50daa4fe78ad/crates/mini-private-payment/src/amount.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 1 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0314"></a>

## PR #314: D-0459: a witness policy comes from the identity's own signed KEL (roadmap R9)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-06, FD-08, FD-09, FD-14, FD-15.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/314) | [Files changed](https://github.com/mininet-labs/Mininet/pull/314/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/17feade5d8e2a1f73e4545d177235f2c02c717ac)

Head `17feade5d8e2a1f73e4545d177235f2c02c717ac`; base `114fd5cfbb5bb5cb20ad11c5dd3f4f1649b88a7b`; merge `d4fa0501ac4730fd314c400c6bd167585b6c198e`. 14 changed files; 6 commits; 2 issue comments, 1 inline comments and 1 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Closes the specific trust-model flaw where the party presenting a witness certificate also supplied the policy used to judge it. Identity-declared witnesses now come from signed KEL establishment events.

**Mechanism and evidence.** Establishment adds witness_threshold; declared_witness_policy reads the latest signed declaration and retirement clears it. assess_kel_assurance removes caller-supplied policy. Appoint/retire operations validate coherent thresholds/sets. Empty-witness legacy encodings are deliberately preserved and pinned against old-build vectors.

**What remains weaker than the intended claim.** This prevents unauthorized policy substitution but not dishonest appointed witnesses, a compromised controller's legitimate signature, stale witness-key resolution or an unwitnessed policy replacement. The PR body's phrase forged branch the controller never signed is too broad: ordinary KEL signature verification remains required.

**Recommended improvement and rationale.** Preserve byte-level backward-compatibility proofs and add historical witness-key binding, certified policy transitions and operation-specific assurance requirements. Do not let witnesses become recovery owners or indispensable permission services for basic speech.

**Concrete example.** A valid controller-signed branch is accompanied by receipts from an unappointed attacker witness set. The verifier must reject that witness assurance even though those witness signatures are correct; it must also reject a genuinely unsigned KEL regardless of witness approval.

**Acceptance tests to implement.** Test unappointed/retired witnesses, threshold-only changes, reordered sets, invalid controller signatures, old empty-policy wire bytes and stale key resolution. Require real consumer gates before describing the assurance as authority protection.

**History, supersession and integration.** Fixes #180/#191 policy-origin gap; #320 uses it on the signing side and #325 begins transition certification. R9 remains incomplete.

**Source entry points.** [`crates/did-mini/src/assurance.rs`](https://github.com/mininet-labs/Mininet/blob/17feade5d8e2a1f73e4545d177235f2c02c717ac/crates/did-mini/src/assurance.rs); [`crates/did-mini/src/controller.rs`](https://github.com/mininet-labs/Mininet/blob/17feade5d8e2a1f73e4545d177235f2c02c717ac/crates/did-mini/src/controller.rs); [`crates/did-mini/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/17feade5d8e2a1f73e4545d177235f2c02c717ac/crates/did-mini/src/error.rs); [`crates/did-mini/src/event.rs`](https://github.com/mininet-labs/Mininet/blob/17feade5d8e2a1f73e4545d177235f2c02c717ac/crates/did-mini/src/event.rs); [`crates/did-mini/src/kel.rs`](https://github.com/mininet-labs/Mininet/blob/17feade5d8e2a1f73e4545d177235f2c02c717ac/crates/did-mini/src/kel.rs); [`crates/did-mini/tests/witness_policy_binding.rs`](https://github.com/mininet-labs/Mininet/blob/17feade5d8e2a1f73e4545d177235f2c02c717ac/crates/did-mini/tests/witness_policy_binding.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 1 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0316"></a>

## PR #316: D-0460: validator accountability — equivocation proofs and exclusion, never a stake (roadmap R8)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-05, FD-06, FD-08, FD-14, FD-15, FD-16.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/316) | [Files changed](https://github.com/mininet-labs/Mininet/pull/316/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/2b482649bfefa6e8d73964a336057ba4d1d7d882)

Head `2b482649bfefa6e8d73964a336057ba4d1d7d882`; base `4d00f7d2408c83beb0fe889575eb316236d15196`; merge `01ad0ce3e82cc9284946de633bb25b361b40b0d9`. 13 changed files; 1 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Adds a chain-layer equivocation proof and explicit non-economic exclusion operation, preserving equal-root voting without introducing stake deposits or wealth-weighted penalties.

**Mechanism and evidence.** EquivocationProof verifies two differently hashed votes from the same root/height/round/phase and canonicalizes order. A local registry deduplicates proofs; ValidatorSet::excluding returns a non-empty reduced set. Adoption remains a separate governance action.

**What remains weaker than the intended claim.** The repository already had consensus-layer equivocation evidence/gossip from #116, so the claim that no prior mechanism existed overstates novelty. Current verification resolves supplied KELs rather than carrying a complete historical key/membership proof, limiting years-later verification. Exclusion is a policy choice, not the only logically possible non-monetary sanction.

**Recommended improvement and rationale.** Unify the two evidence representations or document exact differences, bind proofs to network/validator epoch and historical key state, and adopt exclusions only through deterministic transition rules with recovery/re-admission policy. Do not confuse a capability-bearing DID with an admitted validator.

**Concrete example.** A validator rotates its device key after double-voting. An offline observer should still verify the historical fault using the correct key/epoch evidence, rather than have the proof disappear because the current KEL head no longer accepts the old signature.

**Acceptance tests to implement.** Test cross-network/epoch mixing, rotated/revoked keys, duplicate same-block votes, mismatched phases and set-transition safety. Run real gossip-to-proof-to-governed-exclusion integration and prove no monetary field enters vote weight.

**History, supersession and integration.** Extends chain primitives from #1 and overlaps #116's consensus evidence. #318/#319 add networking slices but do not complete automatic accountable validator-set evolution.

**Source entry points.** [`crates/mini-chain/src/equivocation.rs`](https://github.com/mininet-labs/Mininet/blob/2b482649bfefa6e8d73964a336057ba4d1d7d882/crates/mini-chain/src/equivocation.rs); [`crates/mini-chain/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/2b482649bfefa6e8d73964a336057ba4d1d7d882/crates/mini-chain/src/error.rs); [`crates/mini-chain/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/2b482649bfefa6e8d73964a336057ba4d1d7d882/crates/mini-chain/src/lib.rs); [`crates/mini-chain/src/vote.rs`](https://github.com/mininet-labs/Mininet/blob/2b482649bfefa6e8d73964a336057ba4d1d7d882/crates/mini-chain/src/vote.rs); [`crates/mini-chain/tests/equivocation.rs`](https://github.com/mininet-labs/Mininet/blob/2b482649bfefa6e8d73964a336057ba4d1d7d882/crates/mini-chain/tests/equivocation.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0317"></a>

## PR #317: D-0461: close roadmap R2 — a scheduled `main` run passed end to end

**PASS** | Captured outcome: **merged** | Directives: FD-01, FD-05, FD-10, FD-14.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/317) | [Files changed](https://github.com/mininet-labs/Mininet/pull/317/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/56829a1370798eef69b9d1295f7c1280ccd24268)

Head `56829a1370798eef69b9d1295f7c1280ccd24268`; base `01ad0ce3e82cc9284946de633bb25b361b40b0d9`; merge `114fd5cfbb5bb5cb20ad11c5dd3f4f1649b88a7b`. 6 changed files; 1 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Records completion of the exact scheduled-main CI criterion rather than using an unrelated green push run. This is a narrowly justified evidence/status update.

**Mechanism and evidence.** D-0461 cites one main head and named workflow runs, distinguishes workflows not triggered on main and retains the non-gating advisory exception. Roadmap counts are updated consistently.

**What remains weaker than the intended claim.** The scoped PASS is for the recorded criterion, not continuous future CI health or release security. This review reads the historical run evidence as repository-recorded evidence unless independently re-fetched; green configured checks do not prove they cover all attacks.

**Recommended improvement and rationale.** Archive run identifiers, event type, exact source/lockfile and tool/advisory database versions. Keep scheduled monitoring active and define what reopens a readiness gate after a dependency or runner change.

**Concrete example.** A new sandbox advisory appears a week after a merge. The previous scheduled green run remains true history, but it must not be used to claim the current tree is still cleared.

**Acceptance tests to implement.** Verify run type/SHA/conclusions and negative cases for missing or skipped jobs. Test that a scanner producing a valid JSON error cannot be counted as a successful scan.

**History, supersession and integration.** Follows #309/#310/#311 and closes only R2 at that moment. It does not remove D-0047, personhood or governance gates.

**Source entry points.** [`README.md`](https://github.com/mininet-labs/Mininet/blob/56829a1370798eef69b9d1295f7c1280ccd24268/README.md); [`docs/DECISION_LOG.md`](https://github.com/mininet-labs/Mininet/blob/56829a1370798eef69b9d1295f7c1280ccd24268/docs/DECISION_LOG.md); [`docs/ROADMAP_TO_RELEASE.md`](https://github.com/mininet-labs/Mininet/blob/56829a1370798eef69b9d1295f7c1280ccd24268/docs/ROADMAP_TO_RELEASE.md); [`governance/work-claims.json`](https://github.com/mininet-labs/Mininet/blob/56829a1370798eef69b9d1295f7c1280ccd24268/governance/work-claims.json). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0318"></a>

## PR #318: D-0462: peer discovery over a real socket (roadmap R8)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-06, FD-09, FD-11, FD-15.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/318) | [Files changed](https://github.com/mininet-labs/Mininet/pull/318/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/8b0b2b92701acf95231e48f4f689dc01f45698a3)

Head `8b0b2b92701acf95231e48f4f689dc01f45698a3`; base `d4fa0501ac4730fd314c400c6bd167585b6c198e`; merge `5ee60a4d3434bf8261345a2dabbd8ed2d25e96d2`. 12 changed files; 2 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Adds a real TCP adapter for peer exchange, allowing a node to learn candidates without a canonical directory service. It is a useful transport capability, not yet autonomous mesh formation.

**Mechanism and evidence.** PEX messages travel through the existing anonymous Channel; bounded responses populate RoutingTable/AddressBook. The observed socket source address is recorded, and connect/read/write timeouts bound individual operations. TcpMesh::establish remains unchanged.

**What remains weaker than the intended claim.** Anonymous ephemeral encryption does not prevent an active intermediary from establishing two sessions and reading/altering PEX. The module's unconditional on-path confidentiality statement is therefore too strong without peer authentication. Observed ephemeral source ports are often not listening endpoints, and first-seen address retention can preserve poisoned/stale hints.

**Recommended improvement and rationale.** Label PEX as unauthenticated hints, optionally authenticate expected peers using the established transport-security layer and verify advertised reachability separately. Add bounded refresh/eviction, diverse candidate selection and an incremental mesh runtime rather than require a single centrally agreed address list.

**Concrete example.** A peer connects from port 49152 but listens on 4000. Recording 49152 as its service address produces a valid-looking unusable address book. A signed, challenge-verified listener advertisement should remain a hint until reachability is tested.

**Acceptance tests to implement.** Test active MITM, poisoned first entry, ephemeral ports, stale hints, total address-book growth, slow-drip frames and diverse discovery under eclipse attempts. Real encrypted payload tests alone do not prove endpoint authenticity.

**History, supersession and integration.** Composes #129 PEX with #120/#289 transport. #319 supplies optional identity authentication but is not automatically wired here; #296 already offers a stronger reusable authenticated discovery path.

**Source entry points.** [`crates/mini-consensus/src/discovery.rs`](https://github.com/mininet-labs/Mininet/blob/8b0b2b92701acf95231e48f4f689dc01f45698a3/crates/mini-consensus/src/discovery.rs); [`crates/mini-consensus/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/8b0b2b92701acf95231e48f4f689dc01f45698a3/crates/mini-consensus/src/error.rs); [`crates/mini-consensus/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/8b0b2b92701acf95231e48f4f689dc01f45698a3/crates/mini-consensus/src/lib.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 1 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0319"></a>

## PR #319: D-0463: validator-authenticated bearer handshake (roadmap R8)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-06, FD-08, FD-09, FD-14, FD-15.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/319) | [Files changed](https://github.com/mininet-labs/Mininet/pull/319/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/ca5bb3b0a24c20ed069ab60713824fd33db75e16)

Head `ca5bb3b0a24c20ed069ab60713824fd33db75e16`; base `5ee60a4d3434bf8261345a2dabbd8ed2d25e96d2`; merge `2721f95db0f8be105d9582d64939d1110ed823b8`. 10 changed files; 2 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Adds optional channel-bound delegated identity authentication for consensus connections without exposing identity in the base anonymous handshake. It can attribute a connection to a verified root/device pair.

**Mechanism and evidence.** ValidatorHandshakeAttestation signs a domain-separated channel binding; verification checks expected binding, root/device KEL identity, delegation, VOTE capability and signature. The TCP examples carry the attestation inside encrypted Channel frames.

**What remains weaker than the intended claim.** VOTE capability proves delegation, not current validator-set membership or epoch admission. No validator set is an input to verify_validator_handshake. The decoder also introduces MAX_SIGS=16 despite did-mini supporting 64 and lacks shared signature-list canonicality enforcement. TcpMesh does not require this handshake.

**Recommended improvement and rationale.** Separate authenticated identity from authorized-validator session; require an explicit membership/epoch check at privileged admission. Use shared canonical signature bounds and domain-bind role/network/session intent where appropriate. Preserve anonymous public transport for uses that do not require identity.

**Concrete example.** A correctly delegated VOTE device belonging to a root absent from the active validator set can authenticate as a root. The caller must reject privileged validator admission even though the identity proof is valid. A legitimate 32-key device must not fail only on wire decode.

**Acceptance tests to implement.** Test non-member and old-epoch roots, revoked keys, channel transplant, role reflection, signature count 16/17/64/65, permutations/duplicates and actual mesh admission. Trace every caller so an optional helper is not mistaken for enforced policy.

**History, supersession and integration.** Builds on #318 and existing channel-binding patterns; #299/#301's shared-limit work was not applied completely here. This is not a demonstrated signature forgery; it is an admission/codec boundary finding.

**Source entry points.** [`crates/mini-consensus/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/ca5bb3b0a24c20ed069ab60713824fd33db75e16/crates/mini-consensus/src/error.rs); [`crates/mini-consensus/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/ca5bb3b0a24c20ed069ab60713824fd33db75e16/crates/mini-consensus/src/lib.rs); [`crates/mini-consensus/src/validator_channel.rs`](https://github.com/mininet-labs/Mininet/blob/ca5bb3b0a24c20ed069ab60713824fd33db75e16/crates/mini-consensus/src/validator_channel.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 1 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0320"></a>

## PR #320: D-0464: witness receipt collection protocol (design doc Phase 4)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-06, FD-08, FD-14, FD-15.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/320) | [Files changed](https://github.com/mininet-labs/Mininet/pull/320/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/17a7f2cc3a31f06bd5db692b31f36fabfe4b6f19)

Head `17a7f2cc3a31f06bd5db692b31f36fabfe4b6f19`; base `2721f95db0f8be105d9582d64939d1110ed823b8`; merge `f3c1b3748579f130ee637edc4fe03b0a2b8a0a4a`. 12 changed files; 2 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Adds typed witness receipt collection operations and applies identity-declared policy on the signing side, avoiding a repeat of the verifier-side policy substitution fixed in #314.

**Mechanism and evidence.** SubmitEventForWitnessingRequest carries only a KEL; handle_submit_for_witnessing calls observe_declared. A fetch request retrieves one witness's already-issued receipt, while requesters assemble multi-witness certificates locally. Codecs reject malformed/trailing data.

**What remains weaker than the intended claim.** These are pure message/handler types, not a socket service. Fetching a receipt is not fetching a missing KEL or automatically resolving gossip disagreement. Durable signing, quotas, historical witness-key binding and rotation readiness remain separate. A field-count regression does not alone prove all future signing callers use the safe handler.

**Recommended improvement and rationale.** Expose only the verified policy-derived signing path to the service adapter, persist before issuing receipts and add explicit bounded KEL/evidence retrieval where needed. Keep certificate assembly local and replaceable; no witness becomes a central certificate server.

**Concrete example.** An identity that never appointed this witness sends a chain-valid KEL. The witness must refuse to sign rather than accept an accompanying attacker policy; a later fetch should return no fabricated receipt.

**Acceptance tests to implement.** Test malformed KELs, unappointed witnesses, replayed identical events, old/new policy transitions, service crashes and bounded missing-evidence retrieval. Add real socket tests before marking collection transport complete.

**History, supersession and integration.** Extends #314/#187; #322 adds persistence and #323/#324 gossip carriers. Those later pieces still need an integrated witness service and authority consumer.

**Source entry points.** [`crates/did-mini/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/17a7f2cc3a31f06bd5db692b31f36fabfe4b6f19/crates/did-mini/src/lib.rs); [`crates/did-mini/src/witness.rs`](https://github.com/mininet-labs/Mininet/blob/17a7f2cc3a31f06bd5db692b31f36fabfe4b6f19/crates/did-mini/src/witness.rs); [`crates/did-mini/src/witness_protocol.rs`](https://github.com/mininet-labs/Mininet/blob/17a7f2cc3a31f06bd5db692b31f36fabfe4b6f19/crates/did-mini/src/witness_protocol.rs); [`crates/did-mini/src/witness_state.rs`](https://github.com/mininet-labs/Mininet/blob/17a7f2cc3a31f06bd5db692b31f36fabfe4b6f19/crates/did-mini/src/witness_state.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 1 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0321"></a>

## PR #321: docs(audit): add project history and external-auditor completion pack

**PARTIAL** | Captured outcome: **merged** | Directives: FD-01, FD-03, FD-05, FD-10, FD-12, FD-14.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/321) | [Files changed](https://github.com/mininet-labs/Mininet/pull/321/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/9d6de970100f9f1f9a9be46f2c2b9021a1c369cf)

Head `9d6de970100f9f1f9a9be46f2c2b9021a1c369cf`; base `52e6eb5c4275b0afcaa4977f64662f94aa842477`; merge `0ed3a4e523be195d39296eeca16a187fca6a599b`. 9 changed files; 17 commits; 1 issue comments, 12 inline comments and 7 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Creates the first unified auditor-facing history/gate pack, including a master directive review and test itinerary. It improves external navigation but is a selected-milestone reconstruction, not the exhaustive PR review now requested.

**Mechanism and evidence.** Eight documentation files index major PR phases, evidence mechanisms, eighteen directives and external gates. Follow-up review added D-0441 scope headers, AI non-substitution wording, audit-index entries, append-only revision conventions and extra identity/supply-chain/founder-account tracks.

**What remains weaker than the intended claim.** The history is pinned to an older revision and contains current-blocker claims that need revalidation as code advances. A grouped PR range or worksheet is not a substantive dossier for every PR. Some verdicts were broader than their mechanisms, particularly voice/value and fork legitimacy.

**Recommended improvement and rationale.** Keep #321 as historical evidence and append this dated exhaustive inventory/dossier campaign rather than rewriting it to imply earlier completeness. Narrow every PASS to its proven scope and attach exact code/PR/test evidence plus concrete closure criteria. Separate independent auditor reports from AI-authored templates.

**Concrete example.** A chapter linking PRs 103-110 to one PR cannot substitute for analyzing each change. This campaign supplies a distinct entry, head, mechanism, failure/improvement and test case for every inventoried PR, including closed alternatives.

**Acceptance tests to implement.** Validate complete PR coverage and source links; recheck current blockers against the pinned later tree. Require human semantic review of claims and never let a coverage checker certify security or external audit completion.

**History, supersession and integration.** Merged before #325/#326. PR #327 is the new exhaustive-review vehicle; its initial scaffold was not the promised report, and this delivery must make actual documents accessible.

**Source entry points.** [`docs/AUDITOR_START.md`](https://github.com/mininet-labs/Mininet/blob/9d6de970100f9f1f9a9be46f2c2b9021a1c369cf/docs/AUDITOR_START.md); [`docs/audits/AUDIT_EVIDENCE_INDEX.md`](https://github.com/mininet-labs/Mininet/blob/9d6de970100f9f1f9a9be46f2c2b9021a1c369cf/docs/audits/AUDIT_EVIDENCE_INDEX.md); [`docs/audits/EXTERNAL_AUDIT_MASTER_REPORT.md`](https://github.com/mininet-labs/Mininet/blob/9d6de970100f9f1f9a9be46f2c2b9021a1c369cf/docs/audits/EXTERNAL_AUDIT_MASTER_REPORT.md); [`docs/audits/PROJECT_BUILD_HISTORY.md`](https://github.com/mininet-labs/Mininet/blob/9d6de970100f9f1f9a9be46f2c2b9021a1c369cf/docs/audits/PROJECT_BUILD_HISTORY.md); [`docs/audits/PR_HISTORY_LEDGER.md`](https://github.com/mininet-labs/Mininet/blob/9d6de970100f9f1f9a9be46f2c2b9021a1c369cf/docs/audits/PR_HISTORY_LEDGER.md); [`docs/audits/README.md`](https://github.com/mininet-labs/Mininet/blob/9d6de970100f9f1f9a9be46f2c2b9021a1c369cf/docs/audits/README.md); [`docs/gates/EXTERNAL_AUDITOR_TEST_ITINERARY.md`](https://github.com/mininet-labs/Mininet/blob/9d6de970100f9f1f9a9be46f2c2b9021a1c369cf/docs/gates/EXTERNAL_AUDITOR_TEST_ITINERARY.md); [`docs/gates/README.md`](https://github.com/mininet-labs/Mininet/blob/9d6de970100f9f1f9a9be46f2c2b9021a1c369cf/docs/gates/README.md). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0322"></a>

## PR #322: D-0465: persistent witness journal (design doc Phase 6)

**FAIL** | Captured outcome: **merged** | Directives: FD-02, FD-06, FD-08, FD-11, FD-14, FD-15.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/322) | [Files changed](https://github.com/mininet-labs/Mininet/pull/322/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/cf0160e2fefd6e141009577648a220611d6fa55a)

Head `cf0160e2fefd6e141009577648a220611d6fa55a`; base `f3c1b3748579f130ee637edc4fe03b0a2b8a0a4a`; merge `1915954e9cac464ee405f273f1184876d9f63033`. 13 changed files; 2 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Adds disk persistence for witness observations, but its current failure ordering breaks the advertised write-before-observable-receipt guarantee. This is a concrete state/persistence defect, separate from the acknowledged lack of fsync.

**Mechanism and evidence.** PersistentWitnessJournal replays stored KEL/epoch inputs through observe_declared and writes accepted observations using temporary-file rename. However, live observe_declared mutates the in-memory journal before persist succeeds; state_for exposes that state, and a retry can return AlreadyAccepted without attempting persistence again.

**What remains weaker than the intended claim.** After an I/O error the process can expose an unpersisted receipt; restart forgets it and can sign a conflicting branch. There is no cross-process lock, no fsync barrier, no per-file read bound and no capacity enforcement during replay. Re-signing on replay also assumes the same witness key, which needs explicit rotation handling.

**Recommended improvement and rationale.** Prepare a candidate state without publishing it, durably commit a bounded canonical record under a lock, then swap live state/return the receipt. On failure preserve old state or poison the service until recovery; persist exact receipt/key epoch rather than assume deterministic signing under an unchanged key forever.

**Concrete example.** Make the .tmp path unwritable. The first observation returns an I/O error after memory has accepted it; a second identical call can return the receipt as AlreadyAccepted. After restart that receipt is absent from disk, defeating the claimed guarantee.

**Acceptance tests to implement.** Inject write/rename/fsync failure and retry in the same process; assert no receipt/state exposure before durable commit. Test two processes, power loss, oversized/corrupt records, excess replay identities and witness-key rotation.

**History, supersession and integration.** Builds on #320/#187. The happy-path restart test proves deterministic replay only under its fixture conditions; R9 must not be closed on that evidence.

**Source entry points.** [`crates/mini-witness-service/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/cf0160e2fefd6e141009577648a220611d6fa55a/crates/mini-witness-service/src/lib.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 1 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0323"></a>

## PR #323: D-0466: KEL head gossip summaries (design doc Phase 5, first slice)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-06, FD-08, FD-11, FD-14, FD-15.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/323) | [Files changed](https://github.com/mininet-labs/Mininet/pull/323/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/a062c7af2d8f4ffc8165b286d5e0a8bbd743d926)

Head `a062c7af2d8f4ffc8165b286d5e0a8bbd743d926`; base `1915954e9cac464ee405f273f1184876d9f63033`; merge `960016cff87950a4d25caa832f3174afa21abb82`. 12 changed files; 2 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Adds compact KEL head hints so peers can detect that they need more evidence without flooding full logs. This is useful discovery of disagreement, not cryptographic proof of it.

**Mechanism and evidence.** KelHeadSummary carries identity, sequence, digest and optional policy generation. compare_head_summaries classifies agreement, same-height disagreement and ahead/behind; mismatched identities are rejected. Summaries can be built from KEL or retained witness state.

**What remains weaker than the intended claim.** The summary is unsigned and caller-constructible. Same-height differing digests are a disagreement hint, not independently verified controller duplicity. An attacker can fabricate huge sequences to trigger fetch work. Existing receipt-submission/fetch operations do not themselves supply every missing chain or ancestry proof.

**Recommended improvement and rationale.** Name the output UnverifiedHeadHint/DisagreementHint and require authenticated bounded targeted evidence before changing trust or issuing accusations. Rate-limit retrieval, bind comparisons to a retained head and define an actual KEL-fetch/ancestry response rather than assume another message means the same thing.

**Concrete example.** A peer claims Alice has sequence 10^18 with a random digest. The node may schedule a bounded evidence request, but must not discard its valid local head or label Alice an equivocator.

**Acceptance tests to implement.** Test arbitrary summaries, huge sequence gaps, policy-generation mismatch, malformed digests, repeated fetch triggers and actual signed-fork verification. Confirm a hint never changes authority by itself.

**History, supersession and integration.** Extends #320; #324 carries hints through sync and #325 addresses a different rotation boundary. This PR's body overstates same-sequence disagreement as real fork evidence; cryptographic verification remains essential.

**Source entry points.** [`crates/did-mini/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/a062c7af2d8f4ffc8165b286d5e0a8bbd743d926/crates/did-mini/src/error.rs); [`crates/did-mini/src/gossip.rs`](https://github.com/mininet-labs/Mininet/blob/a062c7af2d8f4ffc8165b286d5e0a8bbd743d926/crates/did-mini/src/gossip.rs); [`crates/did-mini/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/a062c7af2d8f4ffc8165b286d5e0a8bbd743d926/crates/did-mini/src/lib.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 1 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0324"></a>

## PR #324: D-0467: KEL head gossip summaries ride ordinary sync traffic (design doc Phase 5 closed)

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-06, FD-08, FD-09, FD-11, FD-14.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/324) | [Files changed](https://github.com/mininet-labs/Mininet/pull/324/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/42d8e3ad5cec6065562e1861bc8e7cfd2357fb2c)

Head `42d8e3ad5cec6065562e1861bc8e7cfd2357fb2c`; base `960016cff87950a4d25caa832f3174afa21abb82`; merge `52e6eb5c4275b0afcaa4977f64662f94aa842477`. 11 changed files; 1 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Moves KEL head summaries through the existing object sync protocol rather than inventing another network transport, and distinguishes the relayer's authorship from the subject identity's truth.

**Mechanism and evidence.** gossip_summary_carrier wraps the summary in an ordinary authored object. Existing Ingest provenance checks authenticate the relaying author; compare_gossip_carrier compares decoded hints with KelCache and returns a classification. No self-certifying KEL bypass is added.

**What remains weaker than the intended claim.** The PR title claims Phase 5 closed while automatic evidence retrieval, retention/pruning and wider runtime wiring are explicitly absent. Authenticating a relayer does not authenticate a third-party head claim. Persisting every gossip object indefinitely can burden weak devices and expose an identity-observation graph.

**Recommended improvement and rationale.** Complete a bounded hint-to-evidence-to-verified-update loop and label Phase 5 scope precisely. Add expiry/GC and privacy-aware relevance selection without making gossip availability a global truth authority. Keep known-author accountability distinct from subject authorization.

**Concrete example.** A trusted friend relays a mistaken head for another user. The carrier signature proves who relayed it, not that the subject signed that head; the cache must stay unchanged until the real KEL evidence verifies.

**Acceptance tests to implement.** Test forged subject hints from valid authors, swapped Ahead/Behind semantics, repeated stale objects, quota exhaustion, missing chain retrieval and privacy leakage from carrier metadata. Demonstrate the full disagreement-resolution path over independent nodes.

**History, supersession and integration.** Builds on #323 and existing #130 sync; it is transport carriage plus comparison, not complete automatic duplicity gossip or R9 completion.

**Source entry points.** [`crates/mini-sync/src/gossip.rs`](https://github.com/mininet-labs/Mininet/blob/42d8e3ad5cec6065562e1861bc8e7cfd2357fb2c/crates/mini-sync/src/gossip.rs); [`crates/mini-sync/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/42d8e3ad5cec6065562e1861bc8e7cfd2357fb2c/crates/mini-sync/src/lib.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0325"></a>

## PR #325: D-0468: old-policy authorization for witness-set rotation (design doc Phase 7, first slice)

**FAIL** | Captured outcome: **merged** | Directives: FD-02, FD-06, FD-08, FD-09, FD-14, FD-15.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/325) | [Files changed](https://github.com/mininet-labs/Mininet/pull/325/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/5663d980dcde8a884cf684224269dbde3573cd04)

Head `5663d980dcde8a884cf684224269dbde3573cd04`; base `0ed3a4e523be195d39296eeca16a187fca6a599b`; merge `e45afbe970e33eca58fc5364ca3f728307834462`. 13 changed files; 2 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Begins old-policy authorization for witness-set rotation, but the current certification API can sign conflicting transition candidates from the same retained state. The intended anti-equivocation guarantee is not established.

**Mechanism and evidence.** certify_policy_transition verifies the KEL, direct-successor link and change of witness set/threshold, then signs under the retained old policy. It takes &self and intentionally does not mutate any journal state. verify_policy_transition accepts an old_policy parameter, checks the new event/certificate and delegates threshold verification.

**What remains weaker than the intended claim.** Two controller-signed alternative successors can both obtain receipts because no transition-signing reservation is retained. Not advancing accepted head is sensible for removed witnesses, but not recording any signing decision is unsafe. The public verifier also does not derive/authenticate the supplied old policy from the preceding KEL state, echoing the policy-origin class fixed in #314.

**Recommended improvement and rationale.** Keep accepted state separate from a durable certified-transition slot keyed by identity, predecessor and old-policy generation. Sign at most one conflicting transition per slot; retries return exact prior receipts. Derive old policy from verified history or require an opaque verified-policy token. Add new-witness readiness and explicit unavailable-witness recovery before enforcement.

**Concrete example.** A compromised controller creates two valid rotations from the same parent, appointing different witness sets. Calling the current nonmutating certification function twice can produce two old-policy receipts; honest old witnesses need persistent refusal of the second conflicting choice.

**Acceptance tests to implement.** Construct both branches from one pre-rotation state, request certificates sequentially and concurrently, crash/restart between them and require only one digest to be certified. Test forged old policy, threshold-only changes, retirement, reordered sets and unavailable-witness recovery.

**History, supersession and integration.** Extends #314/#320/#322; merged after #321. No real assurance/authority gate currently requires this certificate, so the immediate finding is an unsafe primitive for future integration, not proven deployed identity takeover.

**Source entry points.** [`crates/did-mini/src/error.rs`](https://github.com/mininet-labs/Mininet/blob/5663d980dcde8a884cf684224269dbde3573cd04/crates/did-mini/src/error.rs); [`crates/did-mini/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/5663d980dcde8a884cf684224269dbde3573cd04/crates/did-mini/src/lib.rs); [`crates/did-mini/src/witness_rotation.rs`](https://github.com/mininet-labs/Mininet/blob/5663d980dcde8a884cf684224269dbde3573cd04/crates/did-mini/src/witness_rotation.rs); [`crates/did-mini/src/witness_state.rs`](https://github.com/mininet-labs/Mininet/blob/5663d980dcde8a884cf684224269dbde3573cd04/crates/did-mini/src/witness_state.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 1 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

<a id="pr-0326"></a>

## PR #326: D-0469: chunked, Merkle-authenticated execution-state transfer over real TCP

**PARTIAL** | Captured outcome: **merged** | Directives: FD-02, FD-04, FD-05, FD-06, FD-11, FD-14.

[PR discussion](https://github.com/mininet-labs/Mininet/pull/326) | [Files changed](https://github.com/mininet-labs/Mininet/pull/326/files) | [Exact head](https://github.com/mininet-labs/Mininet/tree/2d23cd6eda69cbe3356c1093c26192a5aba39d30)

Head `2d23cd6eda69cbe3356c1093c26192a5aba39d30`; base `e45afbe970e33eca58fc5364ca3f728307834462`; merge `684baa103c075299a375d1ed9a3327185c0ba02b`. 13 changed files; 3 commits; 0 issue comments, 0 inline comments and 0 review submissions captured. These counts are not independent approval counts.

**Contribution to the founder's idea.** Adds usable chunked state transfer over real TCP and preserves the final state commitment as authority. It is an important operational step for lossy links, but chunking alone is not resumability or bounded-memory proof.

**Mechanism and evidence.** SnapshotManifest includes header/QC/network and a chunk Merkle root; SnapshotAssembler verifies the QC before accepting chunks, checks chunk proofs and finally reassembles/decodes/recomputes the canonical state commitment. The client installs through the existing atomic state-sync path.

**What remains weaker than the intended claim.** The chunk root is supplied by the serving peer and is not directly committed by the QC; per-chunk proofs authenticate consistency with that manifest, not canonical state until finish. A dishonest server can waste download work before final mismatch. The assembler retains/reassembles the full bounded snapshot, and the one-peer one-pass runtime has no retry/resume or trailing suffix.

**Recommended improvement and rationale.** Keep final state verification mandatory and label preliminary chunk verification accurately. Add persisted bounded assembly, manifest identity binding, multi-peer retry/backoff and suffix catch-up, with quotas against hostile manifests. Consolidate Merkle code only through a neutral reviewed utility if doing so reduces duplicate security surface without inappropriate dependencies.

**Concrete example.** A peer supplies a valid header/QC but a Merkle root for unrelated bytes. Every chunk can verify against its advertised root; finish must still reject the state. A reconnect should eventually reuse verified bounded chunks only under the same manifest, not silently start a new authority context.

**Acceptance tests to implement.** Test arbitrary manifest roots with valid QCs, malformed proof depth/index/length, maximum snapshot allocation, out-of-order/duplicate chunks, power-loss resume, mixed manifests, malicious peers and exact final-state/no-mutation failure. Measure old-phone RAM/flash and complete snapshot-plus-suffix recovery.

**History, supersession and integration.** Extends #289 after #300/#313 format changes and merged at canonical 684baa103c075299a375d1ed9a3327185c0ba02b. It does not resolve dynamic validator history, long-range defense, private-spend validity or full R8 readiness.

**Source entry points.** [`crates/mini-consensus/src/chunked_snapshot.rs`](https://github.com/mininet-labs/Mininet/blob/2d23cd6eda69cbe3356c1093c26192a5aba39d30/crates/mini-consensus/src/chunked_snapshot.rs); [`crates/mini-consensus/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/2d23cd6eda69cbe3356c1093c26192a5aba39d30/crates/mini-consensus/src/lib.rs); [`crates/mini-consensus/src/net.rs`](https://github.com/mininet-labs/Mininet/blob/2d23cd6eda69cbe3356c1093c26192a5aba39d30/crates/mini-consensus/src/net.rs); [`crates/mini-consensus/src/node.rs`](https://github.com/mininet-labs/Mininet/blob/2d23cd6eda69cbe3356c1093c26192a5aba39d30/crates/mini-consensus/src/node.rs); [`crates/mini-consensus/src/snapshot_sync_tests.rs`](https://github.com/mininet-labs/Mininet/blob/2d23cd6eda69cbe3356c1093c26192a5aba39d30/crates/mini-consensus/src/snapshot_sync_tests.rs); [`crates/mini-consensus/src/store.rs`](https://github.com/mininet-labs/Mininet/blob/2d23cd6eda69cbe3356c1093c26192a5aba39d30/crates/mini-consensus/src/store.rs). The complete changed-file inventory is in EVIDENCE_MAP.json.

**Capture limitation.** 1 file(s) lacked an API patch; the exact filenames are recorded in EVIDENCE_MAP.json. Full source history is preserved in the Git bundle; missing API patch bytes are not counted as a reviewed patch.

**Review boundary.** Static PR-specific analysis at the recorded heads plus risk-focused source/test inspection. Historical author test claims were not rerun here. Proposed acceptance tests are not passing results. This dossier grants no approval, custody, release or governance authority.

---

