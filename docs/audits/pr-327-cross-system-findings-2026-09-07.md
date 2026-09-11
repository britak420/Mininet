# PR #327: cross-system findings and review boundaries

**Audit scope**

| Field | Value |
|---|---|
| Reviewed at | Canonical source `main` @ `0ed3a4e523be195d39296eeca16a187fca6a599b`; individual observations below name their narrower source surface |
| Workspace size at that commit | Exact resolved dependency count not measured in this review; no whole-workspace execution coverage is claimed |
| Method | AI-authored source/document comparison and architectural reasoning. This document is not the completed review of all 174 PRs. No Rust execution, exploit reproduction, hardware measurement, or external cryptographic sign-off is claimed |
| Tool versions | GitHub connector; connector version not exposed. No local Rust analysis toolchain used for these findings |
| Revalidation trigger | Changes to the named files, their actual authorization callers, validator membership policy, constitutional authority hierarchy, or underlying release evidence |

**Internal AI research, not an independent external audit.** Every proposed test below is a test still to perform unless an independently linked result establishes otherwise. A successful evidence export is not a successful review.

## 1. What the founder's idea requires from the implementation

The project is not merely a collection of privacy libraries or a chain with equal numerical vote weights. Its canonical Founder Directives require human freedom to survive the disappearance or compromise of its present institutions. That changes what counts as a successful implementation.

A component is not decentralized because its interface accepts an arbitrary peer address. An ordinary participant must have a practical way to obtain alternative peers, authenticate the authority relevant to the action, survive malicious responses, and continue when the original provider disappears. Similarly, a protocol is not human-equal merely because each public key gets one vote. The adversary's ability to manufacture or rent additional eligible identities remains part of the equality claim.

The history should therefore distinguish five levels: a stated principle; an interface expressing that principle; an enforcing implementation; a complete application path that cannot bypass it; and independent evidence under the intended adversary and deployment. Progress at one level must not silently promote the others. This distinction is especially important when early PRs introduce safe type seams and later PRs describe those seams as completed production capabilities.

Primary project sources: `docs/FOUNDER_DIRECTIVES.md`, `docs/INVARIANTS.md`, `docs/DECISION_LOG.md`, and the implementation files named below, all interpreted at the pinned revision rather than an unrecorded moving `main`.

## 2. Validator-channel identity is not validator-set admission

**Verdict: PARTIAL. Confirmed interface boundary; not a demonstrated signature forgery.**

**Location:** `crates/mini-consensus/src/validator_channel.rs`, specifically `verify_validator_handshake` and `recv_validator_handshake`; introduced by PR #319.

The verifier checks the expected channel binding, matches the supplied root and device KELs to the claimed identities, verifies root/device delegation, requires `Capabilities::VOTE`, and verifies the device signatures. These checks establish an authenticated, delegated, vote-capable identity relative to the KELs supplied to the verifier.

The function has no active `ValidatorSet`, height, or membership-epoch argument. It therefore cannot itself establish that the authenticated root belongs to the validator set authorized for a particular consensus height. The distinction matters because an identity root can delegate a capability to its own device without thereby appointing itself to a network's validator committee. The `ValidatorOracle` lookup used by the receiving helper is evidence resolution, not an explicit membership decision in this function.

**Exact failure point:** a caller interpreting the returned `Did` as proof of current validator membership would be relying on a property this function does not check. This is not evidence that every current caller makes that mistake. The call-site audit must establish whether an actual admission bypass exists.

**Long-term solution:** preserve the anonymous transport and its ordinary-user privacy boundary. Separate `AuthenticatedVoteCapableIdentity` from an admission decision such as `AdmittedValidator` that requires the relevant canonical validator set and height/epoch. Perform the admission check only where a validator-only service actually requires it. Do not require globally identifiable validator authentication for unrelated ordinary-user traffic.

**Acceptance tests:** a correctly delegated and correctly signed `VOTE` device whose root is not in the active set must pass identity authentication but fail validator-only admission; an excluded or previous-epoch member must fail admission for the new epoch; an active member must pass; a stale or conflicting membership context must not be accepted through an alternate constructor. Audit every caller before treating the module's name as the actual guarantee.

## 3. Channel transcript scope should be specified, not inferred

**Verdict: PARTIAL. Confirmed signed-field scope; exploitability requires additional analysis.**

**Location:** the same module's `ValidatorHandshakeAttestation::transcript` and `sign_validator_handshake`.

The signed transcript contains a domain tag and the channel binding. Claimed root and device identifiers are carried in the wire object and are checked against KELs and delegation during verification, but are not directly included in the signed transcript. Channel binding is useful session context; it is not a replacement for documenting which identity, role, network, and direction the authorization statement means.

It would be incorrect to label the absence of those signed fields an immediately proven identity-substitution exploit. Delegation verification and the signing key already constrain the claim. The unanswered review question is whether every allowed key-reuse, identity-recovery, bidirectional-handshake, and multi-deployment case preserves that constraint.

**Exact failure point:** the module's assurance depends on composition assumptions that are not all represented in its statement type. Adding arbitrary nonces without a corresponding verifier rule would not resolve that ambiguity.

**Long-term solution:** write a precise statement definition before a wire-format change. Evaluate a typed transcript binding protocol/network context, the claimed identities, intended role, direction, and the existing fresh channel binding. Bind an epoch only where the caller actually supplies and validates a canonical epoch. Prefer the smallest statement that closes a specified attack, not a collection of fields nobody checks.

**Acceptance tests:** cross-network use, same key material under distinct permissible identity histories, reflection between send/receive roles, replay onto a different channel, revoked delegation, and changed committee membership. Each negative test must first establish that the underlying signatures and non-targeted checks are valid, so a rejection proves the intended boundary rather than an unrelated malformed fixture.

## 4. Encryption, endpoint authentication, and anonymity are different claims

**Verdict: PARTIAL. Required threat-model correction and composition review.**

**Location:** descriptions of `mini_bearer::Channel`, PR #120's consensus integration, PR #318's discovery adapter, PR #319's channel-bound identity layer, and all applications of those interfaces.

The evidence distinguishes an intentionally anonymous transport handshake from later signed, channel-bound identity statements. Consequently, the review must not treat an authenticated-encryption primitive as sufficient evidence that the application is connected to its intended remote identity. Protection of ciphertext within a session and authentication of the entity that established that session are separate properties.

**Exact failure point:** a broad statement that an anonymous encrypted handshake defeats every active on-path attacker needs a demonstrated endpoint-authentication argument. A raw-wire test showing that a plaintext marker is absent proves a useful narrower fact; it does not by itself test an attacker terminating two independent sessions.

**Long-term solution:** give each transport or application path an explicit adversary table: passive observer, active session-interposing attacker, dishonest authenticated peer, compromised relay, and compromised endpoint. Identify whether signed payloads, a pinned service key, an out-of-band invitation, or a channel-bound attestation supplies the missing authentication. Preserve pseudonymous and anonymous ordinary use rather than solving endpoint authentication by forcing everyone to disclose a global human root.

**Acceptance tests:** a controlled two-session interposer; rejection of a substituted endpoint where a particular identity is required; verification that channel-bound statements cannot be transplanted; payload-integrity checks even on a hostile authenticated link; and packet/log inspection showing that the additional identity evidence is not exposed before the intended confidentiality boundary.

## 5. Constitutional meaning and implementation evidence need separate precedence

**Verdict: FAIL for an unambiguous documentation authority hierarchy at the inspected surface.**

**Location:** the opening authority paragraph in `docs/INVARIANTS.md` versus `docs/FOUNDER_DIRECTIVES.md`'s canonical-status section, D-0090, and D-0352.

The invariant document says that SPEC-00 governs disagreements. The Founder Directives' canonical-status section says that the older external six-principle SPEC-00 and eleven-principle framing were superseded and that the eighteen directives are the canonical principle set. A new contributor following these texts can encounter different answers to which source settles a disagreement.

A related issue is that the invariant register retains implementation-status statements describing networked consensus and other subsequently implemented components as pending. Those stale statements do not erase the later code. They show that the purported traceability map needs a deliberately bounded reconciliation.

**Exact failure point:** readers must infer the current authority hierarchy and implementation status from history instead of following one consistent register. Treating executable code as the highest normative authority is not the answer: a bug in code is a violation to fix, not an automatic amendment of the founder's values.

**Long-term solution:** a dedicated, human-reviewed documentation reconciliation should state the constitutional source hierarchy separately from the evidence hierarchy. Preserve historical records. Record supersession explicitly. Update only current implementation-status pointers after inspecting the actual enforcing call sites and tests; do not replace an old `pending` with a blanket PASS because a crate now exists.

**Acceptance tests:** every constitutional source reference resolves to a versioned, available document or is clearly marked historical; every supersession has an explicit recorded basis; generated checks detect dangling decision and enforcement references; and a reviewer can walk directive to invariant to real verification function to negative test without consulting an unrecorded founder explanation.

## 6. The voice/value wall includes eligibility and operating costs

**Verdict: PARTIAL for the full Founder Directive 16 claim.**

**Location:** `mini-chain` equal-root voting, `mini-forge` quorum policy, `mini-uniqueness` eligibility, resource and reward policies, and FD-16's explicit prohibition of indirect wealth-to-rule paths.

A validator type with no stake or balance field is strong evidence against direct token-weighted voting. The history should credit that achievement. It is not complete evidence that wealth cannot acquire additional eligible roots, purchase access to the social graph, subsidize a dominant share of required infrastructure, or control practical reviewer availability.

**Exact failure point:** a direct-weighting PASS must not be reused as a PASS for the stronger end-to-end anti-capture claim. The register itself warns that a verified DID root is not a verified distinct human. That warning remains part of interpreting quorum results.

**Long-term solution:** retain the structural prohibition on money as a vote-count input while evaluating eligibility and participation separately. Measure duplicate-identity acquisition, credential rental, reviewer capture, and resource-cost exclusion under explicit attacker budgets. Avoid a remedy that introduces a central identity issuer or vendor attestation monopoly: that would replace one route to power with another.

**Acceptance tests:** vary an actor's wealth without changing legitimate human membership and establish unchanged formal voting weight; then separately measure whether wealth changes the practical number of admitted identities or access to indispensable roles. Publish failed adversarial cases rather than averaging them away. Ordinary pseudonyms and public profiles must not each manufacture additional human entitlements.

## 7. An external library is not automatically an external authority

**Verdict: PARTIAL pending exact implementation, license, and integration review.**

The founder's in-house Rust direction should be interpreted carefully during the comparison study. Owning a protocol's implementation and avoiding an indispensable service do not require reimplementing every group operation, codec, or operating-system primitive. Conversely, a mature library does not automatically make an unreviewed composition safe.

The comparison dossiers must evaluate at least: exact construction and parameters; implementation language; wire compatibility; audit scope and dates; secret-handling behavior; side-channel assumptions; maintenance and vendoring options; reproducible source availability; license of the exact source version; and whether an external online authority becomes necessary.

**Exact failure point:** describing all publicly downloadable code as public domain would create a provenance error. Likewise, recommending a different FROST ciphersuite or amount-proof system as a drop-in replacement without addressing transcripts, encodings, and stored commitments would create a protocol-migration error.

**Long-term solution:** compare reference implementations first as independent test oracles and sources of test vectors. Propose production adoption only after identifying compatibility and governance consequences. Keep third-party notices and source provenance intact; Mininet's own CC0 declaration does not erase upstream licenses. A reproducible, locally vendorable library can be replaceable even if it originated elsewhere. A mandatory remote verification service remains a trust dependency regardless of its source license.

**Acceptance tests:** differential positive and negative vectors for genuinely compatible constructions; explicit expected non-interoperability for different suites; dependency-graph isolation; offline builds from pinned source; retention of applicable license notices; and a removal/replacement drill that does not require an upstream account or service to authorize users.

## 8. Completion criteria for the requested exhaustive review

This document does not mark any uninspected PR complete. PR #327's finished campaign needs one independently checkable record for every observed PR in its declared 174-PR inventory, including closed-unmerged and superseded proposals. Each record must bind its actual head/outcome, substantive changed files, relevant discussions, founder intent, contribution, implementation limits, subsequent corrections, and PR-specific engineering recommendations with acceptance tests.

A coverage checker can prove that every inventory identifier has a dossier. It cannot prove that the dossier contains competent analysis. Both checks are needed. Boilerplate generated from titles, an archive of PR bodies, or a table linking elsewhere is not the deep review the user requested.

Open proposal behavior must remain separate from shipped behavior. Historical test claims must remain separate from tests actually rerun for this campaign. When later code closes a historical defect, the review must credit that correction while retaining the history of the defect and its cause.

**Overall judgment for the inspected boundaries:** the evidence does not establish a production free internet for humanity. The required direction is clear: remove single-party authority, preserve user-chosen identity privacy, make authority decisions explicit and context-bound, audit the whole execution path rather than its vocabulary, and require evidence matching the strength of every claim. This is a bounded intermediate finding, not completion of the 174-PR campaign.
