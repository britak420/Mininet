# Cross-system findings and implementation plan

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

## Reading the findings

P0 means no production activation of the affected authority/value path until fixed and independently reviewed. P1 means a concrete engineering prerequisite, not permission to ignore it in production. FAIL applies to the stated property, not every property of the crate. PARTIAL names a narrower primitive or a missing integration guarantee. All fixes below are proposals unless PATCH_STATUS explicitly records implemented and tested work.

## F-01 - FROST signing nonces can be reused through the public API

**FAIL | P0 before custody**

**Location:** [`crates/mini-treasury/src/frost_sign.rs:249`](https://github.com/mininet-labs/Mininet/blob/684baa103c075299a375d1ed9a3327185c0ba02b/crates/mini-treasury/src/frost_sign.rs#L249-L267). Related PRs: [#5](https://github.com/mininet-labs/Mininet/pull/5), [#95](https://github.com/mininet-labs/Mininet/pull/95).

**Mechanism and failure.** round2_sign borrows &SigningNonces. The type is non-Clone and zeroized on drop, but the caller may invoke round 2 again before dropping it. Memory cleanup is not a one-use constraint. The same nonce pair contributes to each response z=d+rho*e+lambda*s*c.

**Concrete example.** With the same d,e and participant coefficient, collect three responses under independent transcripts. If the resulting three coefficient rows are independent, solving the linear system yields d,e and the secret share s. Two arbitrary FROST responses do not by themselves necessarily suffice; the pair has two nonce unknowns.

**Long-term fix.** Consume SigningNonces by value in round2_sign, verify that its derived commitments equal this signer's committed pair, and burn it on every success or error. A durable signer service additionally needs crash-safe reservation/burn records before a response is released; a Rust move alone cannot stop snapshot rollback or a modified signer.

**Acceptance tests to implement.** Add a compile-fail example that attempts a second use; reject a mismatched commitment pair; test error-path consumption. In an isolated test harness demonstrate the three-transcript recovery on the old API, then verify the ordinary API cannot produce that transcript set. Later inject crash/restart between reservation and response.

**Boundary of the finding.** Confirmed API hazard, not evidence that a live treasury key was compromised. No real custody is authorized by the prototype.

## F-02 - FROST share verification can index a missing signing participant

**FAIL | P0 before custody**

**Location:** [`crates/mini-treasury/src/frost_sign.rs:271`](https://github.com/mininet-labs/Mininet/blob/684baa103c075299a375d1ed9a3327185c0ba02b/crates/mini-treasury/src/frost_sign.rs#L271-L289); [`crates/mini-treasury/src/frost_sign.rs:198`](https://github.com/mininet-labs/Mininet/blob/684baa103c075299a375d1ed9a3327185c0ba02b/crates/mini-treasury/src/frost_sign.rs#L198-L216). Related PRs: [#5](https://github.com/mininet-labs/Mininet/pull/5), [#95](https://github.com/mininet-labs/Mininet/pull/95).

**Mechanism and failure.** Verification checks the index exists in the public verification-share map, then indexes the signing package's commitment map. A legitimate group member need not belong to this signing round. aggregate compares map lengths, not exact participant-key equality.

**Concrete example.** A 2-of-3 group builds commitments for {1,2}; the supplied two shares are keyed {1,3}. Participant 3 is a real group member but has no commitment in this package. Length equality does not protect the indexed lookup.

**Long-term fix.** Require exact equality of commitment and share index sets before aggregation. Make the per-signer lookup fallible and return InvalidFrostParticipant rather than indexing maps with attacker-controlled participant IDs. Keep rejection before state mutation or expensive work.

**Acceptance tests to implement.** Test valid group members absent from the current package, equal-sized substituted sets, missing/extra participants and unknown IDs. Wrap the old trigger in a non-production panic-detection test, then require the patched operation to return a typed error without panic.

**Boundary of the finding.** A malformed local/protocol input can cause a panic; network reachability depends on the eventual service caller, not established by the library alone.

## F-03 - FROST scalar wire decoding accepts multiple encodings

**FAIL | P1 canonicalization**

**Location:** [`crates/mini-treasury/src/frost_sign.rs:316`](https://github.com/mininet-labs/Mininet/blob/684baa103c075299a375d1ed9a3327185c0ba02b/crates/mini-treasury/src/frost_sign.rs#L316-L334). Related PRs: [#5](https://github.com/mininet-labs/Mininet/pull/5), [#95](https://github.com/mininet-labs/Mininet/pull/95).

**Mechanism and failure.** Signature::from_bytes reduces the response scalar modulo the group order instead of requiring its canonical encoding. Two byte strings can therefore decode to the same signature. This conflicts with byte-addressed identity, deduplication and transcript expectations.

**Concrete example.** Keep R fixed. Encode z=0 and z=the group order. The old decoder maps both response halves to zero. This example demonstrates decoder aliasing, not a valid signature forgery.

**Long-term fix.** Use canonical scalar decoding, reject out-of-range bytes and preserve strict 64-byte framing. Audit every public scalar decoder by semantic role: reduction may be valid for hash output but not a unique serialized signature. Record the intentional prelaunch acceptance-set narrowing.

**Acceptance tests to implement.** Pin canonical zero and order-minus-one behavior, reject the order and all-ones values, reject trailing/truncated input, and preserve honest signature round trips. Verify formerly accepted aliases fail without changing the bytes emitted by honest signers.

**Boundary of the finding.** Malleability is established at decoding; whether it bypasses a specific deployed replay index requires that caller to key on raw bytes.

## F-04 - A corrupt author sequence file silently becomes a fresh counter

**FAIL | P1 identity continuity**

**Location:** [`crates/mini-cli/src/sequence.rs:23`](https://github.com/mininet-labs/Mininet/blob/684baa103c075299a375d1ed9a3327185c0ba02b/crates/mini-cli/src/sequence.rs#L23-L41). Related PRs: [#101](https://github.com/mininet-labs/Mininet/pull/101), [#171](https://github.com/mininet-labs/Mininet/pull/171).

**Mechanism and failure.** The OS lock excludes concurrent cooperating writers, but read_to_string errors all become zero and parse errors use unwrap_or(0). A truncated, unreadable or malformed existing counter can therefore reuse old author sequence numbers. The subsequent write is not a crash-safe transaction.

**Concrete example.** A home previously issued sequence 50. Its sequence file becomes empty after interrupted replacement. The next call returns 1 rather than refusing a state it cannot trust.

**Long-term fix.** Only NotFound may select the first-use path. Reject malformed/empty/out-of-range data and unexpected read errors without rewriting it. Then replace the file atomically with file and directory durability appropriate to the platform. Full deletion/rollback recovery needs a retained author-head binding; treating NotFound as new alone does not solve that attack.

**Acceptance tests to implement.** Regression-test empty, nonnumeric, overflowing and invalid-UTF-8 files, a directory in place of the file, exhaustion at u64::MAX, first creation, concurrency and preservation of rejected bytes. Add power-loss/rollback tests before claiming durable monotonicity.

**Boundary of the finding.** The proposed narrow parsing patch is distinct from the larger deletion/rollback-resistant sequence design.

## F-05 - Witness state is accepted in memory before durable commitment

**FAIL | P0 before witness authority**

**Location:** [`crates/mini-witness-service/src/lib.rs:197`](https://github.com/mininet-labs/Mininet/blob/684baa103c075299a375d1ed9a3327185c0ba02b/crates/mini-witness-service/src/lib.rs#L197-L215). Related PRs: [#322](https://github.com/mininet-labs/Mininet/pull/322).

**Mechanism and failure.** PersistentWitnessJournal advances the in-memory journal before the persistence call succeeds. On a write error, the accepted state can remain visible; a retry may take AlreadyAccepted and avoid a new write. Returning Err once does not undo the in-memory acceptance. File replacement also lacks a full power-loss durability guarantee.

**Concrete example.** Make the state directory unwritable after open, submit a valid successor, observe an I/O error, restore permissions and retry. A previously accepted receipt must not escape while the durable record still represents the older head.

**Long-term fix.** Prepare on cloned/staged state, persist exact accepted event and receipt with a transactional write, then publish the in-memory state. On uncertain durable outcome, poison the service until recovery. Add cross-process locking, bounded reads and load-time quotas. Persist the original receipt/key epoch rather than assuming re-signing under a later witness key reproduces it.

**Acceptance tests to implement.** Inject errors before write, after write, before rename, after rename and before acknowledgement; test same-process retry and fresh-process reopen. Require every released receipt to have an identical durable record. Test rotated witness keys, corrupt/oversized records and two simultaneous writers.

**Boundary of the finding.** The code is a witness-service primitive, not proof that a production network already trusts its outputs.

## F-06 - Witness transition certification needs its own anti-equivocation state

**FAIL | P0 before high-assurance rotation**

**Location:** [`crates/did-mini/src/witness_rotation.rs:97`](https://github.com/mininet-labs/Mininet/blob/684baa103c075299a375d1ed9a3327185c0ba02b/crates/did-mini/src/witness_rotation.rs#L97-L115); [`crates/did-mini/src/witness_rotation.rs:153`](https://github.com/mininet-labs/Mininet/blob/684baa103c075299a375d1ed9a3327185c0ba02b/crates/did-mini/src/witness_rotation.rs#L153-L171). Related PRs: [#325](https://github.com/mininet-labs/Mininet/pull/325).

**Mechanism and failure.** certify_policy_transition does not mutate or reserve an accepted successor slot. Keeping the old accepted policy is useful when a witness certifies its own retirement, but it also permits certification of two different policy-changing successors of the same retained head. Verification additionally accepts old_policy as an external parameter instead of deriving a verified predecessor policy.

**Concrete example.** A compromised controller signs successor A appointing witness set A and successor B appointing set B, both from the same parent. An honest old witness asked separately can certify both unless it remembers its certification decision.

**Long-term fix.** Retain old accepted state but persist a separate transition commitment keyed by identity, predecessor digest and generation. Reissuing the identical transition is idempotent; a conflicting transition is refused and becomes evidence. Derive the old policy from authenticated history or an opaque verified policy handle. Define new-witness readiness and unavailable-witness recovery without a permanent veto authority.

**Acceptance tests to implement.** Test A then B, B then A, identical retries, crashes between signature and persistence, wrong predecessor policies, threshold-only changes, harmless set reordering and full retirement. Require the real assurance consumer to enforce the transition chain, not merely expose a helper.

**Boundary of the finding.** These checks are not yet wired into a high-value authority decision. This is a dangerous incomplete primitive, not a demonstrated live identity takeover.

## F-07 - Consensus orders shielded key images without proving spend validity

**FAIL | P0 real-value blocker**

**Location:** [`crates/mini-execution/src/lib.rs:72`](https://github.com/mininet-labs/Mininet/blob/684baa103c075299a375d1ed9a3327185c0ba02b/crates/mini-execution/src/lib.rs#L72-L74). Related PRs: [#313](https://github.com/mininet-labs/Mininet/pull/313).

**Mechanism and failure.** The execution side accepts opaque key-image/claim-digest facts without independently verifying the private claim, its input membership and conservation. Canonical agreement about an asserted spend does not make the assertion valid. Keeping vote weight independent of money does not require honest validators to accept unproved state transitions.

**Concrete example.** A proposer sees a pending valid payment's key image and includes that image under a different digest first. Honest execution can then consume the conflict key without a valid corresponding payment. This does not assume the attacker can derive an unseen owner's secret key image.

**Long-term fix.** Introduce a narrow, deterministic validity boundary consumed before voting/execution: full claim digest binding, canonical input-commitment membership, signatures, range/balance proofs, network/version binding and atomic output/nullifier transition. Keep governance weighting in a separate interface that receives no balances. Resolve any repository dependency-policy conflict through explicit human-reviewed architecture rather than a bypass or trusted validity committee.

**Acceptance tests to implement.** Drive a real Byzantine proposal through the same validation path honest validators use. Assert arbitrary/copied key images, substituted digests, invented inputs, imbalance, cross-network proofs and partial multi-input application never reach a QC-backed state. Include a genuine private payment that finalizes and is spent again legitimately.

**Boundary of the finding.** The source itself declares this gap. No production release is safe merely because ordering/snapshot tests pass.

## F-08 - ProvenCapacity can be constructed from unverified declarations

**FAIL | P0 before capacity rewards or weighting**

**Location:** [`crates/mini-spacetime/src/storage_proof.rs:114`](https://github.com/mininet-labs/Mininet/blob/684baa103c075299a375d1ed9a3327185c0ba02b/crates/mini-spacetime/src/storage_proof.rs#L114-L132); [`crates/mini-spacetime/src/storage_proof.rs:123`](https://github.com/mininet-labs/Mininet/blob/684baa103c075299a375d1ed9a3327185c0ba02b/crates/mini-spacetime/src/storage_proof.rs#L123-L141). Related PRs: [#302](https://github.com/mininet-labs/Mininet/pull/302), [#306](https://github.com/mininet-labs/Mininet/pull/306).

**Mechanism and failure.** The numeric constructor was removed, but public StorageCommitment fields still allow a caller to declare large block counts and sizes, then call ProvenCapacity::from_commitment. Arithmetic derivation from a declaration is not verification of possession. Copying and adding the same capacity also requires caller-side deduplication.

**Concrete example.** Construct a large commitment with no replica, derive its typed capacity and pass it to proposer_weight. Separately, add the same valid replica capacity twice. Neither a type name nor saturating arithmetic proves uniqueness.

**Long-term fix.** Split DeclaredCapacity from VerifiedWindowCapacity. Only successful registration and current-window verification may mint the latter; bind it to network, replica, policy and window. Aggregate through a set keyed by unique replica identity, not arbitrary addition. Keep operator independence separate: two valid seals can still belong to one warehouse.

**Acceptance tests to implement.** Compile-fail direct construction of verified capacity; reject expired/replayed windows, duplicate replica totals, forged size metadata and stale registration policy. Measure honest weak-device liveness under partitions. Do not award voice for capacity.

**Boundary of the finding.** No live weighting/reward consumer is established here. The failed claim is the type's sufficiency as proof, not evidence of current inflation.

## F-09 - Intake review labels and crash journals are not authority proofs

**FAIL | P1 before automatic publication**

**Location:** [`crates/mini-intake-types/src/envelope.rs:194`](https://github.com/mininet-labs/Mininet/blob/684baa103c075299a375d1ed9a3327185c0ba02b/crates/mini-intake-types/src/envelope.rs#L194-L212); [`crates/mini-cli/src/intake.rs:84`](https://github.com/mininet-labs/Mininet/blob/684baa103c075299a375d1ed9a3327185c0ba02b/crates/mini-cli/src/intake.rs#L84-L102). Related PRs: [#153](https://github.com/mininet-labs/Mininet/pull/153), [#159](https://github.com/mininet-labs/Mininet/pull/159), [#242](https://github.com/mininet-labs/Mininet/pull/242), [#286](https://github.com/mininet-labs/Mininet/pull/286).

**Mechanism and failure.** Mutator review gates do not make decoded Accepted/authority/link fields authenticated. A stored publication journal can decode an object without proving it is the expected author's canonical post for this intake/source. The first #286 review fixed source-byte rehashing, but later semantic journal and state-transition concerns remain distinct.

**Concrete example.** Replace a pending journal with a different well-formed signed object, or load an envelope with Accepted and inconsistent links. A parser succeeding must not authorize publication or project recognition.

**Long-term fix.** Validate the full envelope state machine on decode, bind load results to the requested intake ID, and authenticate authority transitions separately from data labels. Bind journal object type, author/delegation, canonical payload, source digest and intake link before insertion. Put advance and publish under one per-intake transaction/lock discipline.

**Acceptance tests to implement.** Test forged accepted-state combinations, wrong intake ID, substituted same-length source, wrong-author journal, non-post journal, replayed completed publication and concurrent advance/publish. Require failures to preserve prior state and never sign substituted bytes.

**Boundary of the finding.** A locally parsed review state is not proof of independent human review. Network exploitability depends on how these local artifacts are supplied.

## F-10 - Signature size and count limits have drifted under new names

**FAIL | P1 interoperability and canonical bytes**

**Location:** [`crates/mini-consensus/src/validator_channel.rs:78`](https://github.com/mininet-labs/Mininet/blob/684baa103c075299a375d1ed9a3327185c0ba02b/crates/mini-consensus/src/validator_channel.rs#L78-L96); [`crates/mini-chain/src/vote.rs:24`](https://github.com/mininet-labs/Mininet/blob/684baa103c075299a375d1ed9a3327185c0ba02b/crates/mini-chain/src/vote.rs#L24-L42); [`crates/mini-objects/src/private_object.rs:26`](https://github.com/mininet-labs/Mininet/blob/684baa103c075299a375d1ed9a3327185c0ba02b/crates/mini-objects/src/private_object.rs#L26-L44). Related PRs: [#299](https://github.com/mininet-labs/Mininet/pull/299), [#301](https://github.com/mininet-labs/Mininet/pull/301), [#319](https://github.com/mininet-labs/Mininet/pull/319).

**Mechanism and failure.** #299 fixed several limits and #301 scans selected literal constant names, but MAX_SIGS/MAX_SIGS_PER_VOTE remain 16 where did-mini supports 64. Several object codecs cap signatures at 256 bytes, below the ML-DSA-65 signature size accepted by the cryptographic suite. A name-based lint is not a shared format contract.

**Concrete example.** A larger threshold identity signs a message in memory, then a downstream decoder rejects the valid wire representation. An enabled post-quantum signature may similarly fail a codec that still assumes short classical signatures.

**Long-term fix.** Own limits beside the defining type and reference them everywhere. Test actual encode/decode/verify across every supported signature suite and key count. Separate operational message-budget caps from format validity, and enforce canonical ordered distinct signature indices.

**Acceptance tests to implement.** Cross-crate table-driven tests at 1/16/17/64 keys and each supported suite; rejection at 65 and malformed ordering; whole-frame budget checks. Mutation-test alternative constant names/expressions so the old scanner's blind spot remains visible.

**Boundary of the finding.** Full post-quantum controller integration is not demonstrated; the 256-byte cases are a concrete latent interoperability blocker, not proof that production PQ identities currently exist.

## F-11 - The dependency scanner can accept a parseable error report as success

**FAIL | P1 build assurance**

**Location:** [`.github/workflows/ci.yml:119`](https://github.com/mininet-labs/Mininet/blob/684baa103c075299a375d1ed9a3327185c0ba02b/.github/workflows/ci.yml#L119-L137). Related PRs: [#299](https://github.com/mininet-labs/Mininet/pull/299), [#307](https://github.com/mininet-labs/Mininet/pull/307), [#317](https://github.com/mininet-labs/Mininet/pull/317).

**Mechanism and failure.** The scan step checks that output is nonempty JSON, then defaults a missing vulnerabilities object/count to zero. After set +e, a nonzero exit with parseable error JSON can reach the success message. The separate dependency-deny job is valuable but does not make this scanner status truthful.

**Concrete example.** Stub the scanner to exit 2 with {"error":"database unavailable"}. A JSON parser succeeds, the missing vulnerability count becomes zero and the workflow can report that the scanner ran successfully.

**Long-term fix.** Parse a strict expected report schema and reconcile scanner exit status with report semantics. Distinguish clean scan, advisories, operational failure and malformed report. Keep intentional advisory-warning policy explicit and independently triaged; do not confuse it with scanner-health failure.

**Acceptance tests to implement.** Execute fixtures for clean report, genuine advisories, empty output, malformed JSON, valid-JSON error, inconsistent count/list and unexpected exit codes. Run under the actual bash -e/set +e wrapper. Preserve the separate deny gate.

**Boundary of the finding.** This is a control-flow/schema defect in the checked-in workflow, not a claim that every past clean scan was false.

## F-12 - Replay persistence can fail open across restart and malformed tails

**FAIL | P1 before security-sensitive replay use**

**Location:** [`crates/mini-presence/src/persisted.rs:199`](https://github.com/mininet-labs/Mininet/blob/684baa103c075299a375d1ed9a3327185c0ba02b/crates/mini-presence/src/persisted.rs#L199-L217); [`crates/mini-presence/src/persisted.rs`](https://github.com/mininet-labs/Mininet/blob/684baa103c075299a375d1ed9a3327185c0ba02b/crates/mini-presence/src/persisted.rs). Related PRs: [#247](https://github.com/mininet-labs/Mininet/pull/247), [#248](https://github.com/mininet-labs/Mininet/pull/248), [#249](https://github.com/mininet-labs/Mininet/pull/249).

**Mechanism and failure.** FileReplayGuard follows an infallible boolean interface: a memory acceptance can survive while the append fails, with only a counter recording the failure. A restart forgets that acceptance. Loading tolerates a malformed final line broadly and reads the file eagerly; hexadecimal parsing must not assume arbitrary UTF-8 has ASCII byte boundaries.

**Concrete example.** Disk-full prevents appending a fresh nonce, but the exchange succeeds in memory. Restart and replay the same nonce. A separate malformed Unicode tail can exercise parser assumptions rather than a legitimate incomplete-write recovery.

**Long-term fix.** Make durable acceptance fallible and commit before acknowledging the protected action. Use a bounded, checksummed record format with explicit incomplete-tail detection, cross-process locking and monotonic expiry policy. Validate ASCII hex before byte-pair conversion; quarantine other corruption.

**Acceptance tests to implement.** Disk-full/read-only injection, restart replay, concurrent writers, clock rollback, over-limit logs, invalid hex/Unicode and truncation at every byte. Require only a provably incomplete last record to be recoverable without discarding valid history.

**Boundary of the finding.** A write_failures counter is observability, not enforcement. A changed trait and all consumers need coordinated review.

## F-13 - Capability signature validity is not resource-owner authorization

**PARTIAL | P1 consumer boundary**

**Location:** [`crates/mini-objects/src/capability.rs:249`](https://github.com/mininet-labs/Mininet/blob/684baa103c075299a375d1ed9a3327185c0ba02b/crates/mini-objects/src/capability.rs#L249-L267). Related PRs: [#141](https://github.com/mininet-labs/Mininet/pull/141), [#143](https://github.com/mininet-labs/Mininet/pull/143), [#170](https://github.com/mininet-labs/Mininet/pull/170).

**Mechanism and failure.** Capability validation checks a grant's signature, scope/right/window and holder proof, but an arbitrary signer can sign a grant. The consumer must establish that this issuer is authorized over the requested resource. Static proof material also needs careful request/session replay treatment.

**Concrete example.** Mallory creates a perfectly valid grant signed by Mallory naming Alice's object. A caller that treats validate success as owner authorization would grant access without Alice issuing anything.

**Long-term fix.** Require an authenticated resource policy/owner binding as a validation input, or return VerifiedGrantSignature rather than an authorization-shaped result. Bind holder requests to fresh operation/session context where replay has a consequence. Preserve delegation chains and least privilege.

**Acceptance tests to implement.** Test attacker-issued grants over another owner's resource, wrong audience/right/scope, revoked delegation and replayed holder proof. A valid Alice-issued grant must remain usable by its intended holder.

**Boundary of the finding.** Confirmed integration obligation; no inspected deployed caller exploit is asserted where the caller may already enforce ownership.

## F-14 - Anonymous channel confidentiality and validator admission are different properties

**PARTIAL | P1 authenticated services**

**Location:** [`crates/mini-consensus/src/validator_channel.rs:199`](https://github.com/mininet-labs/Mininet/blob/684baa103c075299a375d1ed9a3327185c0ba02b/crates/mini-consensus/src/validator_channel.rs#L199-L217); [`crates/mini-consensus/src/discovery.rs:94`](https://github.com/mininet-labs/Mininet/blob/684baa103c075299a375d1ed9a3327185c0ba02b/crates/mini-consensus/src/discovery.rs#L94-L112). Related PRs: [#120](https://github.com/mininet-labs/Mininet/pull/120), [#296](https://github.com/mininet-labs/Mininet/pull/296), [#318](https://github.com/mininet-labs/Mininet/pull/318), [#319](https://github.com/mininet-labs/Mininet/pull/319).

**Mechanism and failure.** Anonymous ephemeral key exchange protects a session from passive reading but does not by itself bind an intended peer against an active man-in-the-middle. The validator helper verifies a root's VOTE-capable delegation, not membership of an active validator set or epoch. These narrower properties must not be called authenticated admission.

**Concrete example.** An attacker terminates two anonymous sessions and forwards PEX messages between them. Separately, a valid VOTE-capable root that was never admitted can satisfy the identity helper unless the caller performs membership checking.

**Long-term fix.** Keep anonymous communication available. For privileged services add channel-bound expected-identity proof and a separate current validator-membership/epoch check. Return a typed authenticated-and-authorized session; do not let an identity proof silently stand for membership. Bind network, role and expected peer into the admission contract.

**Acceptance tests to implement.** Test active two-session interception, wrong expected peer, revoked/stale delegation, valid nonmember, obsolete membership epoch, cross-network session and anonymous nonprivileged traffic. No CA, central directory or universal identity disclosure is needed.

**Boundary of the finding.** No signature forgery is alleged. Both deficiencies concern what a caller is licensed to infer from the existing primitive.

## F-15 - Plans and quotes can overstate achieved privacy

**PARTIAL | P1 privacy-facing product claims**

**Location:** [`crates/mini-publication-policy/src/receipt.rs`](https://github.com/mininet-labs/Mininet/blob/684baa103c075299a375d1ed9a3327185c0ba02b/crates/mini-publication-policy/src/receipt.rs); [`crates/mini-transport-security/src/gate.rs`](https://github.com/mininet-labs/Mininet/blob/684baa103c075299a375d1ed9a3327185c0ba02b/crates/mini-transport-security/src/gate.rs). Related PRs: [#131](https://github.com/mininet-labs/Mininet/pull/131), [#138](https://github.com/mininet-labs/Mininet/pull/138), [#145](https://github.com/mininet-labs/Mininet/pull/145), [#146](https://github.com/mininet-labs/Mininet/pull/146), [#246](https://github.com/mininet-labs/Mininet/pull/246), [#296](https://github.com/mininet-labs/Mininet/pull/296).

**Mechanism and failure.** Publication policy composes routing and pricing data. Planning relay roles or selecting a tier does not demonstrate that bytes traversed an authenticated route with the claimed privacy. The Mixed/Burst runtime guarantee currently comes from the absence of an executor, not necessarily a mandatory call through the named gate.

**Concrete example.** A UI receives a role plan and prints "source hidden" even though sending fails or uses a direct fallback. Every policy unit test may pass while the user's IP is exposed.

**Long-term fix.** Separate RequestedProfile, ExecutablePlan and ObservedExecutionReceipt. Only the transport executor may construct achieved outcomes, and downgrade/fallback requires explicit user consent. Route every future tier dispatch through an enforced gate. Define adversary-specific anonymity claims rather than an unqualified private label.

**Acceptance tests to implement.** Test no-send, partial route, bad relay key, wrong destination, fallback attempts and missing mix executor. Capture real traffic in an isolated test network; correlate timing and packet lengths under the stated observer model.

**Boundary of the finding.** A functioning three-hop onion is real progress but is not Sphinx/Loopix or a global-observer anonymity proof.

## F-16 - Appliance backup and restore need a stronger local-secret boundary

**PARTIAL | P1 before deployment to real identities**

**Location:** [`deploy/backup/backup.sh`](https://github.com/mininet-labs/Mininet/blob/684baa103c075299a375d1ed9a3327185c0ba02b/deploy/backup/backup.sh); [`deploy/backup/restore.sh`](https://github.com/mininet-labs/Mininet/blob/684baa103c075299a375d1ed9a3327185c0ba02b/deploy/backup/restore.sh). Related PRs: [#304](https://github.com/mininet-labs/Mininet/pull/304).

**Mechanism and failure.** The deployment tooling handles the user's only recoverable identity state. Passing a passphrase through process arguments broadens exposure. Restore must validate archive paths/types and preserve a previous usable state until the new state is safely committed; a checksum alone is not archive safety or authority.

**Concrete example.** Another local process inspects a command line containing a backup passphrase. An interrupted restore after removal of the old directory leaves neither the old usable home nor a complete replacement.

**Long-term fix.** Use protected descriptor/stdin secret input, authenticated archive handling, explicit member/path validation and staging on the destination filesystem. Quiesce or transactionally snapshot live state. Commit with rollback-safe replacement and verify restored identity before retiring the old state. Copies of the same key are not automatically equivocation; conflicting signed acts are.

**Acceptance tests to implement.** Test malicious archive paths/symlinks, bad passphrase, corruption, disk-full, cross-filesystem staging, power loss and live writes during backup. Inspect process arguments/logs for secrets and test restore without any vendor account.

**Boundary of the finding.** Real hardware/deployment acceptance is still required; lint success cannot establish these properties.

## F-17 - Founder-guarded governance is a declared current central control point

**FAIL | P0 before decentralized production claims**

**Location:** [`governance/bootstrap-operating-state.json`](https://github.com/mininet-labs/Mininet/blob/684baa103c075299a375d1ed9a3327185c0ba02b/governance/bootstrap-operating-state.json); [`.github/CODEOWNERS`](https://github.com/mininet-labs/Mininet/blob/684baa103c075299a375d1ed9a3327185c0ba02b/.github/CODEOWNERS). Related PRs: [#117](https://github.com/mininet-labs/Mininet/pull/117), [#118](https://github.com/mininet-labs/Mininet/pull/118), [#218](https://github.com/mininet-labs/Mininet/pull/218), [#265](https://github.com/mininet-labs/Mininet/pull/265), [#321](https://github.com/mininet-labs/Mininet/pull/321).

**Mechanism and failure.** The pinned operating-state record declares active founder-guarded governance, zero independent non-founder human maintainers and no canonical Forge/production release candidate. Its own last verification timestamp is historical. This is evidence of the declared state, not a fresh personnel or account-control audit.

**Concrete example.** Remove the founder's GitHub account and credentials. Independent participants must still be able to authenticate canonical history, deliberate, approve and distribute a security update without an emergency founder bypass.

**Long-term fix.** Execute a measured succession/removal drill with independent human maintainers, documented conflict-of-interest boundaries, verified rulesets, off-platform evidence mirrors and owner-controlled adoption. Sunset the temporary exception through legitimate governance, never by letting a date or AI assertion manufacture legitimacy.

**Acceptance tests to implement.** Archive actual ruleset/approval evidence, verify independent custody, simulate founder absence and hostile repository hosting, and prove a new exact release can be reviewed and verified without any current single actor.

**Boundary of the finding.** A branch API summary alone does not reveal every repository ruleset. Do not claim the live protection settings were fully audited from that summary.

## F-18 - F5 has deliberately failed economic and resource gates

**FAIL | P0 before subsidized settlement**

**Location:** [`tools/fixtures/f5_phase2_report.jsonl`](https://github.com/mininet-labs/Mininet/blob/684baa103c075299a375d1ed9a3327185c0ba02b/tools/fixtures/f5_phase2_report.jsonl); [`tools/f5_phase2_model.py`](https://github.com/mininet-labs/Mininet/blob/684baa103c075299a375d1ed9a3327185c0ba02b/tools/f5_phase2_model.py). Related PRs: [#284](https://github.com/mininet-labs/Mininet/pull/284), [#285](https://github.com/mininet-labs/Mininet/pull/285).

**Mechanism and failure.** The fixed model records genuine colluding delivery draining 100% of a bounded protocol budget against a 10% ceiling, adaptive audit grinding with zero observed detection against a 95% floor, and configured replay-state estimates above the 8 MiB ceiling. phase3_authorized remains false.

**Concrete example.** One operator controls requester and provider roles, performs actual deliveries to itself and collects every permitted subsidy. Delivery correctness does not establish independent demand or usefulness.

**Long-term fix.** Keep ordinary requester-funded transfers independent of subsidy auditors. For subsidies, redesign or explicitly narrow the funded objective under a new precommitted threat/economic model: immutable claims before bias-resistant audit entropy, privacy-preserving overlap limits, finite budgets and measured worst-case state. Do not lower success thresholds after observing failure.

**Acceptance tests to implement.** Rerun the exact checked-in model/vector; add adaptive withholding, multi-policy overlap and genuine-colluder campaigns. An independent mechanism reviewer must validate assumptions and population false-rejection behavior before any real budget is enabled.

**Boundary of the finding.** Passing tests can correctly reproduce a FAIL result. A model check is not external economic sign-off.

## F-19 - Personhood and operator independence remain empirical trust dependencies

**PARTIAL | P0 before human governance or universal grants**

**Location:** [`docs/INVARIANTS.md`](https://github.com/mininet-labs/Mininet/blob/684baa103c075299a375d1ed9a3327185c0ba02b/docs/INVARIANTS.md); [`docs/design/frontier-personhood-governance-and-consensus-proposals.md`](https://github.com/mininet-labs/Mininet/blob/684baa103c075299a375d1ed9a3327185c0ba02b/docs/design/frontier-personhood-governance-and-consensus-proposals.md). Related PRs: [#123](https://github.com/mininet-labs/Mininet/pull/123), [#140](https://github.com/mininet-labs/Mininet/pull/140), [#143](https://github.com/mininet-labs/Mininet/pull/143), [#220](https://github.com/mininet-labs/Mininet/pull/220), [#253](https://github.com/mininet-labs/Mininet/pull/253), [#299](https://github.com/mininet-labs/Mininet/pull/299).

**Mechanism and failure.** Root uniqueness, valid signatures, physical proximity evidence and replica possession establish different predicates. None automatically proves that two roots belong to different people or two servers have independent control. Zero knowledge hides a proved predicate; it does not make an untrustworthy issuer or sensor truthful.

**Concrete example.** A warehouse maintains many valid roots and distinct seals; a person attends repeated ceremonies under new roots. Every local crypto verification may be correct while global equality fails.

**Long-term fix.** Keep source evidence, policy-qualified credentials and authority separate. Specify measurable false acceptance/rejection, coercion, exclusion, collusion, recovery and concentration limits; compare multiple decentralized enrollment strategies with independently operated pilots. Research is permitted, but no prototype result becomes unique-human authority without its stated activation evidence.

**Acceptance tests to implement.** Adversarial duplicate-enrollment and colluding-issuer pilots, weak-device/accessibility tests, recovery exclusion tests, privacy linkage measurements and attacks on operator-diversity heuristics. Preserve pseudonyms and no institution-issued mandatory identity.

**Boundary of the finding.** This is not a claim that research cannot improve the problem. The submitted evidence does not yet establish the required production property.

## F-20 - Airdrop claim bookkeeping is not atomic successful payment

**PARTIAL | P1 before funded campaigns**

**Location:** [`crates/mini-presence/src/persisted.rs`](https://github.com/mininet-labs/Mininet/blob/684baa103c075299a375d1ed9a3327185c0ba02b/crates/mini-presence/src/persisted.rs); [`crates/mini-airdrop-treasury/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/684baa103c075299a375d1ed9a3327185c0ba02b/crates/mini-airdrop-treasury/src/lib.rs). Related PRs: [#238](https://github.com/mininet-labs/Mininet/pull/238), [#240](https://github.com/mininet-labs/Mininet/pull/240), [#241](https://github.com/mininet-labs/Mininet/pull/241).

**Mechanism and failure.** Claim eligibility, a local claimed registry, treasury approvals and canonical payment are separate steps. Marking a claim before payment can strand an entitlement on failure; an unbounded or permissive registry recovery can forget claims. A public approval-shaped struct must not be accepted as proof merely because it can be constructed.

**Concrete example.** A valid claimant is marked claimed, then signing/submission fails. On retry the system refuses the claim although no funds arrived. Alternatively two writers both pass the initial unclaimed check before either persists.

**Long-term fix.** Use campaign-bound idempotent Pending/Authorized/Submitted/Finalized records with a durable atomic reservation and canonical payment digest. Reconcile retries to the same payment, not a second award. Authenticate approval records at consumption and impose load/write quotas with cross-process exclusion.

**Acceptance tests to implement.** Crash at every transition, simultaneous claims, truncated/malformed middle records, campaign reuse, duplicate payout and canonical rejection/retry. Verify no entitlement disappears without a recorded recoverable state or actual payment.

**Boundary of the finding.** The current approval bridge explicitly stops before moving value. Preserve that honest separation while completing it.

## F-21 - Search pluralism requires honest provenance, not blind score trust

**PARTIAL | P1 before public federation**

**Location:** [`crates/mini-search-federation-net/src/remote_merge.rs`](https://github.com/mininet-labs/Mininet/blob/684baa103c075299a375d1ed9a3327185c0ba02b/crates/mini-search-federation-net/src/remote_merge.rs); [`crates/mini-ranker/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/684baa103c075299a375d1ed9a3327185c0ba02b/crates/mini-ranker/src/lib.rs). Related PRs: [#257](https://github.com/mininet-labs/Mininet/pull/257), [#281](https://github.com/mininet-labs/Mininet/pull/281), [#282](https://github.com/mininet-labs/Mininet/pull/282), [#290](https://github.com/mininet-labs/Mininet/pull/290), [#294](https://github.com/mininet-labs/Mininet/pull/294).

**Mechanism and failure.** A provider can sign false metadata or report a maximum score. Provenance identifies the claim's publisher, not its truth. Local reranking may reuse a diversity score computed under a prior ordering, and remote query transport discloses exact query terms to the selected provider.

**Concrete example.** Two providers return one URL with different scores and self-asserted context. A higher-score-wins merge lets a malicious source dominate unless the consumer distinguishes asserted from recomputed evidence.

**Long-term fix.** Bind provider identity to the authenticated channel/publication, carry score provenance and profile IDs, recompute from locally verified inputs where practical, and retain disagreement. Recompute order-dependent diversity after reranking or label the approximation. Offer local index retrieval or audited private-query options without calling ordinary TLS-like confidentiality PIR.

**Acceptance tests to implement.** Hostile max-score provider, profile mismatch, false canonical URL, duplicate providers, adversarial domains, missing corpus, independent-source failure and query-correlation tests. Confirm payment/bids never become organic ranking authority.

**Boundary of the finding.** A ranker without a payment parameter still needs defenses against purchased links/domains and manipulated source inputs; no perfect ranking truth oracle is proposed.

## F-22 - Chunk Merkle consistency is not an independently finalized chunk root

**PARTIAL | P1 weak-device sync**

**Location:** [`crates/mini-consensus/src/chunked_snapshot.rs:228`](https://github.com/mininet-labs/Mininet/blob/684baa103c075299a375d1ed9a3327185c0ba02b/crates/mini-consensus/src/chunked_snapshot.rs#L228-L246). Related PRs: [#289](https://github.com/mininet-labs/Mininet/pull/289), [#300](https://github.com/mininet-labs/Mininet/pull/300), [#326](https://github.com/mininet-labs/Mininet/pull/326).

**Mechanism and failure.** The QC commits the canonical header/state root, not necessarily the peer's chosen chunk root. Chunk proofs detect inconsistency with that peer manifest; final reconstruction and canonical state commitment comparison remain indispensable. The wired path is one peer/one pass with full reassembly, not resumable multi-peer streaming.

**Concrete example.** A peer supplies a valid finalized header with a different internally consistent chunk tree. All chunk membership checks can pass, but finish must reject the reconstructed state. A dropped connection should not later be marketed as resumable merely because chunks exist.

**Long-term fix.** Preserve the final canonical state check and label per-chunk evidence accurately. Add bounded staging, manifest identity, authenticated resume cursors, retry/backoff and multi-peer sourcing; stream decoding/commitment work under measured memory budgets. Reuse the same exact-body and suffix checks when catching up after snapshot installation.

**Acceptance tests to implement.** Valid-QC/wrong-chunk-tree attack, substituted index, truncation, duplicates, short final chunk, interrupted resume and snapshot-plus-suffix composition. Record peak memory/disk/work on weak hardware, not just maximum chunk size.

**Boundary of the finding.** No current canonical-state forgery is asserted: the inspected final comparison is the protection that must remain.

## F-23 - Post-quantum primitive support is not post-quantum identity recovery

**PARTIAL | P1 migration architecture**

**Location:** [`crates/mini-crypto/src/keys.rs`](https://github.com/mininet-labs/Mininet/blob/684baa103c075299a375d1ed9a3327185c0ba02b/crates/mini-crypto/src/keys.rs); [`crates/mini-pq-anchor/src/lib.rs`](https://github.com/mininet-labs/Mininet/blob/684baa103c075299a375d1ed9a3327185c0ba02b/crates/mini-pq-anchor/src/lib.rs). Related PRs: [#148](https://github.com/mininet-labs/Mininet/pull/148), [#156](https://github.com/mininet-labs/Mininet/pull/156), [#168](https://github.com/mininet-labs/Mininet/pull/168), [#220](https://github.com/mininet-labs/Mininet/pull/220), [#237](https://github.com/mininet-labs/Mininet/pull/237).

**Mechanism and failure.** ML-DSA verification/signing and dormant anchor generation are useful primitives. They do not automatically provide persistent anchors, pre-break KEL commitment, treasury/validator recovery or codecs large enough for their signatures. Legacy convenience APIs may panic when called with a suite they do not support.

**Concrete example.** A device generated a PQ anchor only in memory, then restarted. During a classical-signature break, its unrecorded anchor cannot prove it was the pre-break owner. A fresh claimant cannot acquire authority by arriving first.

**Long-term fix.** Make suite-specific operations explicit and fallible; persist encrypted PQ material with owner-controlled backup, commit anchors before any break, archive last-safe checkpoints independently and define migration classes. Keep no-anchor cases explicit rather than inventing a recovery administrator.

**Acceptance tests to implement.** Cross-suite wire tests, restart/backup/restore, wrong-suite nonpanic rejection, compromised-classical-key migration, unavailable witnesses and no-prebreak-anchor refusal. Review FIPS implementation and protocol composition separately.

**Boundary of the finding.** This report does not claim a future cryptanalytic break has occurred or that standards alone certify Mininet's implementation.

## F-24 - Audit and decision machinery can be structurally consistent but false

**PARTIAL | P1 evidence governance**

**Location:** [`tools/check_decisions.py`](https://github.com/mininet-labs/Mininet/blob/684baa103c075299a375d1ed9a3327185c0ba02b/tools/check_decisions.py); [`tools/check_roadmap.py`](https://github.com/mininet-labs/Mininet/blob/684baa103c075299a375d1ed9a3327185c0ba02b/tools/check_roadmap.py); [`docs/FOUNDER_DIRECTIVES.md`](https://github.com/mininet-labs/Mininet/blob/684baa103c075299a375d1ed9a3327185c0ba02b/docs/FOUNDER_DIRECTIVES.md). Related PRs: [#127](https://github.com/mininet-labs/Mininet/pull/127), [#218](https://github.com/mininet-labs/Mininet/pull/218), [#261](https://github.com/mininet-labs/Mininet/pull/261), [#288](https://github.com/mininet-labs/Mininet/pull/288), [#301](https://github.com/mininet-labs/Mininet/pull/301), [#310](https://github.com/mininet-labs/Mininet/pull/310), [#311](https://github.com/mininet-labs/Mininet/pull/311), [#321](https://github.com/mininet-labs/Mininet/pull/321).

**Mechanism and failure.** Decision/roadmap validators catch missing IDs, counts and some history loss. They cannot prove that a cited decision actually completed the work. Historical instruction surfaces, live status documents and supersession records require precise precedence rather than AI-invented reconciliation.

**Concrete example.** A roadmap item cites a real D-number and says done while the last runtime caller is still unimplemented. All reference/count checks can pass. Two independently verified workflow jobs under one organization likewise are not two independent human approvers.

**Long-term fix.** Maintain immutable historical records plus dated corrections, exact scope and evidence links. Publish a human-reviewed authority/precedence map that distinguishes constitutional principles from implementation status. Add claim-to-executor-to-adversarial-test traceability, and preserve original audit authorship/independence limitations.

**Acceptance tests to implement.** Mutation-test missing/deleted/duplicate decisions and misleading done citations; require human review of semantic closure. Verify the new PR-review coverage checker detects omissions but never presents its green status as proof of depth or security.

**Boundary of the finding.** The earlier claim of an unconditional contradiction between SPEC-00 and founder-directive canonicalization was too broad. Their respective scopes must be reconciled explicitly; this report does not amend either.

## Implementation order and stop conditions

First remove unsafe custody/signing APIs (F-01-F-03), invalid shielded execution (F-07), and witness acknowledgement/transition defects (F-05-F-06). These protect secrets and irreversible authority. Implement them as separately reviewable code changes with failing-before/passing-after tests, not bundled into a documentation merge.

Next make persistence and parser boundaries uniform (F-04, F-09-F-12, F-16, F-20), then connect identity, membership, provenance and actual runtime privacy (F-13-F-15, F-21-F-23). Transactional local databases are a legitimate option to compare; a locally embedded replaceable store is not a central network authority.

In parallel, pursue personhood and mechanism falsification (F-18-F-19), independent governance succession (F-17), and reproducible evidence (F-24). None is closed by adding a helper API, a new type name, a reviewer checkbox, or a date. The project must remain valueless/prototype wherever its stated external gates are unmet.

The finished outcome is not "more crates." It is a small set of operational end-to-end paths whose relevant decisions are locally verifiable, whose failure behavior is tested, whose costs fit the weakest intended participant, and whose continuation does not depend on a founder, vendor, bridge, auditor or hosted repository.
