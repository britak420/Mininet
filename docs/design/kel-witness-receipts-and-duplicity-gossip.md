# KEL witness receipts and duplicity gossip (audit #12 F4, invariant M3)

**Status:** Phase 0 (design), Phase 1 (receipt types, D-0321), Phase 2
(in-memory witness state machine, D-0326), Phase 3's first slice
(`KelAssurance` classification, D-0328), Phase 3's second slice (real
KEL-chain verification wired in front of `WitnessJournal::observe`,
D-0329), a local duplicity-proof registry (`DuplicityRegistry`, D-0330),
a `mini-forge` bridge (`author_assurance`, D-0332), **witness policies
bound to the identity's own signed KEL (D-0459)**, **a receipt collection
protocol (D-0464, Phase 4)**, **a persistent witness journal
(`mini-witness-service`, D-0465, Phase 6)**, **KEL head gossip summaries,
both the pure comparison (D-0466) and real sync-carried transport
(`mini-sync`, D-0467) — closing Phase 5**, **old-policy authorization
for witness-set rotation (D-0468, Phase 7's first slice)**, and
**new-witness readiness threshold for the same rotation (D-0471,
closing §17.2+§17.3 of Phase 7)** shipped. Only wiring `author_assurance`
into a real governance call site — a founder-facing policy call — and a
bounded/incremental KEL re-verify remain open in Phase 3; Phase 7's last
remaining piece (§17.4, unavailable-witness recovery) and Phases 8-10 not
started.

**Full research:** `docs/research/
KEL_WITNESS_RECEIPTS_DUPLICITY_GOSSIP_RESEARCH_20260715.md`
(founder-supplied, 2026-07-15). This document does not reproduce that
report — it records the adopted direction and the phased plan this repo
commits to, and links back for the full threat model, prior-art survey
(KERI witness receipts, Certificate Transparency gossip, Key Transparency,
CoSi), and protocol design.

## The gap this closes

`did_mini::FreshnessPins` (D-0088) already solves the case where a
verifier has *previously seen* an identity: a newly presented conflicting
event is rejected because it contradicts retained state. It does **not**
solve the harder case audit #12's finding F4 named: a verifier meeting an
identity **for the first time** has no prior head to compare against, and
two internally-valid, controller-signed branches can both pass ordinary
KEL verification in isolation. This is the "never seen a fresher log"
gap — SPEC-01 §7, invariant M3's harder half.

## Decision

Adopt the report's recommended architecture: KERI-inspired asynchronous
witness receipts plus proof-carrying gossip, **not** a global identity
ledger, BFT witness consensus, or interactive signature aggregation.

- Establishment events gain a versioned `WitnessPolicy` (witness set +
  threshold + generation).
- Witnesses issue typed `WitnessReceipt`s over a `WitnessReceiptStatement`
  binding identity, sequence, event digest, prior digest, event kind,
  witness-policy generation, and a coarse observation epoch — never a
  generic `sign(bytes)`, per CLAUDE.md's typed-domain rule.
- Enough receipts against the active policy's threshold form a
  `WitnessedEventCertificate`, independently verifiable offline.
- Witnesses keep first-seen monotonic state per identity; a witness that
  signs conflicting receipts, or a controller that signs conflicting
  same-sequence events, produces a compact, self-contained,
  independently-verifiable duplicity proof.
- Peers gossip compact `KelHeadSummary`s during ordinary relevant
  interactions (not a standalone always-on service); disagreements
  trigger targeted evidence retrieval, not full-log flooding.
- A first-contact verifier's acceptance rule and resulting `KelAssurance`
  (`Direct` / `Pinned` / `Witnessed` / `WitnessedRecent` /
  `WitnessedRecentAndGossiped` / `DuplicityDetected`) replaces any
  boolean "is this fresh" claim with an honest, gradable assurance level.

## Why design-only, this PR

The report's own closing recommendation: "the best next engineering
deliverable is a design-only PR, followed by a small receipt/proof type
PR, then an in-memory witness state-machine PR, and only afterward
network gossip. The dangerous mistake would be starting with a witness
daemon or Merkle log before freezing exactly what a receipt means, what
constitutes duplicity, and what a first-contact verifier is allowed to
claim." This repo's own established discipline this session (MN-207/208,
PQ-15) already scopes founder-supplied research to its own recommended
first phase — here that phase is documentation, not code, so that's
exactly what this PR is.

## Phased plan this repo commits to (see report §29 for full detail)

0. **Design and state audit** — this document.
1. **Receipt types (shipped, D-0321)** — `WitnessPolicy`,
   `WitnessReceiptStatement`, `WitnessReceipt`,
   `WitnessedEventCertificate`; canonical encoding; signature
   verification; no network service. Lives in `did-mini::witness`.
2. **In-memory witness state machine (shipped, D-0326)** — `WitnessJournal`
   in `did-mini::witness_state`: first-seen acceptance, direct-successor
   verification, duplicate idempotence (returns the previously issued
   receipt, never re-signs), stale rejection, same-sequence conflict
   detection (`ControllerDuplicityProof`, built from real controller-
   signed `Event`s), and a standalone `WitnessEquivocationProof::assemble`
   for a third party holding two disagreeing receipts from one witness.
   Trusts the caller that `event` is already chain-valid at its claimed
   position — no signature/pre-rotation/recovery verification, no
   fork-proof construction for the harder "conflicting descendant" case,
   no persistence, no network. Lives in `did-mini::witness_state`.
3. **KEL verification integration** — `KelAssurance` output alongside
   ordinary KEL validity, never replacing it with one boolean.
   **First slice shipped (D-0328):** `did-mini::assurance::
   assess_kel_assurance` classifies `Direct`/`Pinned`/`Witnessed`/
   `WitnessedRecent`/`DuplicityDetected` by composing `Kel::verify`
   (via `FreshnessPins`), a caller-supplied `WitnessedEventCertificate`/
   `WitnessPolicy`, and a caller-supplied `known_duplicity` flag.
   **Second slice shipped (D-0329):** `did_mini::WitnessJournal::
   observe_verified` runs the real `Kel::verify` chain over the entire
   presented KEL before delegating to `observe` — a witness no longer
   has to trust an untrusted peer's bare claim that a presented event is
   chain-valid; `observe` itself is unchanged for callers that establish
   chain validity some other way. **Local duplicity registry shipped
   (D-0330):** `did_mini::DuplicityRegistry` records `ControllerDuplicityProof`/
   `WitnessEquivocationProof` and answers `has_known_duplicity(identity,
   policy)`, so a caller now has a real place to accumulate proofs
   instead of computing `known_duplicity` by hand; `assess_kel_
   assurance`'s own signature is unchanged. **`mini-forge` bridge shipped
   (D-0332):** `mini_forge::author_assurance` composes the oracle's
   existing provenance re-check (`author_verified`) with `assess_kel_
   assurance` over the author-root's KEL — the first consumer of
   `KelAssurance` outside `did-mini` itself. Deliberately **not** wired
   into `propose`/`approve`/`merge`/`resolve_project`'s actual quorum
   gating, which remains purely `author_verified`'s boolean: which
   governance action (if any) should require which minimum assurance
   level is a founder-facing policy call, not something decided
   unilaterally here. **Policy binding shipped (D-0459):** `Establishment`
   now carries `witness_threshold` beside its witness set,
   `Kel::declared_witness_policy` reads the policy back from the most
   recent establishment event, and `assess_kel_assurance` takes it from
   there — `WitnessEvidence::policy` is **gone**. This was not a
   completeness detail. A caller-supplied policy meant whoever handed a
   verifier a certificate also handed it the standard the certificate was
   judged against: an attacker's own witnesses receipt a forged branch, the
   attacker supplies a policy naming them, and the verifier reports
   `WitnessedRecent` for a head the real controller never signed. Every
   assurance level above `Pinned` was decorative against the only attacker
   they exist for. `Controller::appoint_witnesses`/`retire_witnesses` are
   the typed declaration operations, and the threshold is encoded only when
   a witness set is non-empty, so every identity predating D-0459 keeps its
   exact bytes and its SCID — checked against an `origin/main` worktree,
   not assumed.
   Still missing from this phase:
   `WitnessedRecentAndGossiped` (needs Phase 5), a bounded/incremental
   KEL-chain re-verify (today re-verifies the whole chain from inception
   every call), persistence for the duplicity registry (Phase 6), and
   any real call site that actually *gates* an authority decision on a
   `KelAssurance` level or feeds real proofs into `DuplicityRegistry`.
4. **Receipt collection protocol (shipped, D-0464)** — typed request/
   response messages in `did-mini::witness_protocol`:
   `SubmitEventForWitnessingRequest`/`Response` (carries a whole `Kel`, no
   policy field — the policy is read from the KEL's own
   `declared_witness_policy`, per `WitnessJournal::observe_declared`,
   extending D-0459's fix to the signing side) and
   `FetchWitnessReceiptRequest`/`Response` (a single witness's own
   already-issued receipt). No `FetchWitnessCertificate` server operation:
   `WitnessedEventCertificate::assemble` (Phase 1) is already the pure
   aggregator, and a certificate is something a requester builds locally
   from several witnesses' responses, not something one witness can hand
   back without Phase 5's gossip. Pure message types and handler functions
   only — no network transport, matching Phase 1-3's own staging; a real
   socket adapter is separate, later work for whichever crate first runs a
   witness service.
5. **Gossip summaries (shipped, D-0466 + D-0467) — closed.**
   `did_mini::gossip`'s `KelHeadSummary` (a compact, unsigned identity/
   sequence/digest/policy-generation claim) and `compare_head_summaries`
   (a pure function classifying two summaries as `Agrees`/
   `Disagreement`/`Ahead`/`Behind`, D-0466). No new evidence-retrieval
   request type: a `Disagreement` or `Ahead` outcome is resolved with
   Phase 4's already-shipped `SubmitEventForWitnessingRequest`/
   `FetchWitnessReceiptRequest`, the same "no duplicate op" reasoning
   Phase 4 itself already applied. **Real transport (D-0467):**
   `mini_sync::gossip_summary_carrier` wraps a summary as an ordinary
   `mini-objects` object, so it rides the existing MINI/SYNC1
   reconciliation protocol with zero new wire messages — the literal
   "piggybacked on existing sync... traffic" this phase named.
   `compare_gossip_carrier` decodes an ingested carrier and compares it
   against the receiver's own `KelCache` (the same locally-cached "what
   do I believe" state `mini_sync::Ingest` already maintains). Turning a
   `Disagreement`/`Ahead` outcome into an automatic
   `mini_sync::request_retrieval` call remains a host policy choice,
   matching `mini_consensus::discovery::pex_over_tcp`'s own
   never-auto-wired precedent.
6. **Persistent witness service (shipped, D-0465)** — new crate
   `mini-witness-service`'s `PersistentWitnessJournal` gives
   `WitnessJournal` durable, crash-recoverable state by replaying
   `WitnessJournal::observe_declared` (D-0464) over what was durably
   recorded, plus an identity-count quota. No network transport, no
   rotation-aware pruning — those remain Phase 7.
7. **Witness rotation and recovery.** **§17.2+§17.3 shipped (D-0468,
   D-0471):** `did_mini::witness_rotation`'s
   `WitnessJournal::certify_policy_transition` lets a witness that already
   holds accepted state for an identity certify, under its own *old*
   retained policy generation, that a specific chain-valid
   direct-successor establishment event legitimately changes that
   identity's witness policy — no new receipt or certificate type, since a
   certification is just an ordinary `WitnessReceiptStatement` naming the
   retiring generation. `verify_policy_transition` checks enough such
   receipts (bundled via Phase 1's existing
   `WitnessedEventCertificate::assemble`) meet the *old* policy's
   threshold, and independently confirms the presented event really is a
   policy change (different witness set or threshold, membership compared
   as a set rather than list order) rather than trusting the certificate's
   mere existence — this was D-0468, §17.2 alone. **D-0471 adds §17.3's
   "new witness readiness threshold":** `verify_witness_rotation`
   AND-composes that same old-policy check with a new-policy one — enough
   *new* witnesses' own ordinary first receipts for the same event, again
   checked via the unchanged `WitnessedEventCertificate::verify`, this
   time against the policy the rotation event itself declares. No new
   receipt type here either: a new witness's first `observe`/
   `observe_declared` call already signs under the new generation, so
   Phase 1-4's existing machinery already produces exactly the statement
   §17.3 asks for — the only new code is the composition, plus treating a
   full witness-policy retirement (no new witness set to prove readiness
   for) as vacuously satisfying the new-policy half. Still open: §17.4's
   unavailable-witness recovery path (deliberately harder — it must work
   *without* old-witness cooperation, the opposite assumption both slices
   make); no wiring into `assess_kel_assurance` — whether a real verifier
   should *require* either assurance level is a founder-facing policy
   call, not decided here.
8. **Public-authority transparency** — per-witness append-only receipt
   logs for governance/release/validator/treasury roots only.
9. **Adversarial network simulation** — forks, witness collusion,
   partitions, eclipse attacks, recovery conflicts.
10. **External review** — gated behind D-0047 before any high-value
    authority decision depends on this layer.

## Hard rules carried forward from the report

- **No global freshness claim, ever.** A witnessed-and-gossiped
  certificate proves a threshold of configured witnesses observed one
  branch within the verifier's gossip horizon — it does not, and must
  never be described as, proof that no other branch exists anywhere.
- **Witnesses never gain authority.** They attest observation; they
  cannot create, rotate, or override identity events. A witness that
  goes unavailable triggers an explicit, non-casual recovery path — not
  a silent threshold reduction.
- **No global identity log.** Public transparency logs are opt-in and
  restricted to public-authority roots (governance/release/validator/
  treasury); private and pairwise identities get scoped gossip, never
  global enumeration.
- **Recovery never erases duplicity evidence.** History stays
  append-only even when lawful recovery overrides a compromised branch.
- **Ordinary independent signatures first.** No BLS, CoSi, threshold
  aggregation, or bespoke consensus in the first version — per Directive
  14 and CLAUDE.md's no-new-cryptography rule, composing `did-mini`'s
  existing typed-signature machinery is sufficient for Phase 1-3.

## What this document originally covered, and what D-0321/D-0326 added

This document was originally Phase 0 only: no new type, `FreshnessPins`
(D-0088) unmodified, no witness state machine, no receipt format, no
gossip protocol. D-0321 (Phase 1) shipped `did-mini::witness`'s four
receipt/certificate types, canonical encoding, and signature/threshold
verification — see that decision-log entry for exactly what it does and
does not cover. D-0326 (Phase 2) shipped `did-mini::witness_state`'s
`WitnessJournal` in-memory state machine plus `ControllerDuplicityProof`/
`WitnessEquivocationProof` — see that entry for exactly what it does and
does not cover. No receipt format wired into real establishment events
(`event.rs`'s `Establishment.witnesses: Vec<Vec<u8>>` field remains its
own pre-existing, differently-shaped placeholder, still unused), no
`KelAssurance`/KEL-verification integration, and no gossip protocol exist
yet — those are Phases 3-5, each its own PR, each scoped no larger than
this session's established discipline for founder-research-driven work.
