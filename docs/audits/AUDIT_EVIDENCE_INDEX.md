# Audit evidence index — claim → mechanism → evidence → remaining gate

**Audit scope**

| Field | Value |
|---|---|
| Reviewed at | `main` @ `2721f95db0f8be105d9582d64939d1110ed823b8` |
| Workspace size at that commit | 72 crates; resolved dependency count not independently measured in this GitHub-only review |
| Method | GitHub-only evidence mapping across current code/document references, merged PRs, invariants, decisions, status/roadmap and gate packages. No local build/test/fuzz/static-analysis run was performed for this document |
| Tool versions | GitHub repository connector only; no code-analysis toolchain executed |
| Revalidation trigger | Any merge changing a mapped mechanism, invariant, gate status, or the current release roadmap; any external review that changes a FAIL/PARTIAL disposition |

This file is the external reviewer’s map from a Mininet claim to the concrete place where it is implemented or tested. It deliberately separates **repository evidence** from **external legitimacy evidence**. The former can show what code does; it cannot self-certify that novel cryptography, personhood, economics, hardware, or legal posture is safe.

| Domain / claim | Concrete mechanism | Repository evidence to inspect | Current verdict | External/remaining gate |
|---|---|---|---|---|
| Money never buys governance | Equal-unit root counting; no balance/stake field in governance/finality paths; voice/value dependency separation | `docs/INVARIANTS.md`; `mini-chain`; governance/forge reviewer checks; D-0008/D-0045/D-0051 | **PASS** for direct protocol weighting | Re-audit every future economic/governance dependency edge |
| One human, one vote | Personhood evidence feeds an intended unique-human credential; roots counted once | `mini-uniqueness`; personhood docs; D-0038/D-0054/D-0075; PR #123 terminology correction | **FAIL** as a production human claim | Independent unique-human/personhood validation |
| No owner/admin/off switch in protocol | Owner-approved install; no protocol owner key; free forks; self-hosted forge | `mini-installer`; `mini-update`; fork legitimacy design; no-GitHub demo | **PARTIAL** | Current GitHub Founder bootstrap authority must sunset |
| Repository governance decentralized | Protected paths, work claims, governance validator, AI zero-weight | `.github/CODEOWNERS`; `governance/bootstrap-operating-state.json`; governance tooling | **FAIL** | D-0083 active; zero independent non-Founder maintainers |
| Canonical monetary truth | Finality QC → execution; canonical sequence rules; body/state roots | `mini-chain`; `mini-consensus`; `mini-execution`; `mini-settlement`; D-0200+; PR #300 | **PARTIAL** | Shielded-spend validity still not independently proven on-chain |
| Consensus safety | Tendermint locking/view-change, signed proposals, equivocation evidence, re-gossip | `mini-consensus`; networked consensus tests; PRs #114/#116/#120/#124/#289/#300 | **PARTIAL** | Independent formal/model review + remaining private-spend validity rule |
| Consensus confidentiality | `mini_bearer::Channel` on consensus links; validator-authenticated optional channel | PR #120; D-0206; PR #319 | **PARTIAL** | Default wiring / deployment / DoS review |
| State sync integrity | Finalized-history catch-up, snapshot authentication, persistent recovery, pruning | PR #130; PR #289; state-sync tests | **PARTIAL** | Independent adversarial state-sync review under real deployment |
| Self-sovereign identity | KERI-style KEL, pre-rotation, delegated devices, recovery | `did-mini`; recovery/delegation tests; D-0005/D-0053 | **PARTIAL** | First-contact freshness/witness assurance not fully end-to-end |
| KEL freshness | `FreshnessPins`, witness receipts/state, duplicity proof, KEL-derived witness policy | PRs #125/#180/#187/#191; D-0459 | **PARTIAL** | Receipt collection/gossip/persistence/rotation + authority call-site gating |
| Structural privacy / pseudonyms | Pairwise/scoped pseudonyms; encrypted ObjectEnvelopeV2; private routing layers | `did-mini`; `mini-objects`; PR #141/#143; relay/onion work | **PARTIAL** | Transparent payments remain; external crypto audit; measured metadata privacy |
| Private value | Stealth addresses, ring signatures, Bulletproofs, private payment composition, change/fees | `mini-value`; private-payment code; PRs #4/#5/#305; D-0455/D-0457 | **FAIL** for real value | External crypto audit + chain-verifiable private-claim validity + retire transparent path |
| Double-spend resistance | Linkable key images/nullifiers + canonical finalization | `mini-value`; execution/private-ledger tests | **PARTIAL** | Chain must prove nullifier/key-image came from a valid claim |
| Treasury custody | FROST signing, Pedersen/Feldman DKG, complaint/rebuttal, resharing | `mini-treasury`; `frost_dkg.rs`; `frost_reshare.rs`; PR #95 | **FAIL** for real value | Independent DKG/FROST audit; production dealer path exclusion; ceremony review |
| Storage possession | Serve receipts, PoST/PDP, PoRep, capacity proof | `mini-storage`; `mini-spacetime`; `mini-porep`; later #297–#306 hardening | **PARTIAL** | Independent proof review; operator-independence unresolved |
| Erasure durability | Systematic Reed-Solomon; reconstruction/repair | `mini-erasure`; PR #106 MDS correction | **PARTIAL** | Operational churn/correlation/weak-device validation |
| Independent replicas/operators | Distinct placement/reward identities | storage/reward policy | **FAIL** | One operator can still control many DIDs; requires independent-control mechanism |
| Local/offline networking | TCP bearer, multicast discovery, PEX, interrupted-sync resume | PRs #6/#100/#128/#129; `mini-bearer`; `mini-net`; `mini-sync` | **PARTIAL** | Hardware BLE/Wi-Fi acceptance, NAT/relay deployment, peer-auth default wiring |
| Onion/relay privacy | Relay roles, route policy, live multi-hop work, three-hop onion path | `mini-relay`; `mini-privacy-policy`; PRs #145/#146/#296 | **PARTIAL** | External anonymity review, traffic-analysis simulation, production deployment |
| BLE/UWB presence | Software RTT + `RangingSource`; hardware-ranged weighting seam | `mini-presence`; hardware gate protocol | **FAIL** as strong real-world proof | T1–T6 physical-device test matrix |
| Search independence | Typed public-web vocabulary, crawler/extract/index/rank/federation slices | `mini-web-types`; `mini-crawler`; `mini-extract-*`; search PR series | **PARTIAL** | Full runtime/scale/privacy/censorship-resistance validation |
| Intake cannot self-authorize | ReviewState/AuthorityClass gating; accepted-only linking | `mini-intake-types`; `mini-intake`; PR #242 | **PASS** for current intake authority boundary | Re-audit new parser/import paths |
| Self-hosted development | CLI repo/PR/review, provenance, sandboxed build, release verification, installer/rollback | PRs #101/#103–#115; self-hosted-spine tests | **PASS** as a demonstrated local/runtime path | Governance must actually migrate off Founder-guarded GitHub bootstrap |
| Reproducible releases | Clean rebuild comparison, provenance agreement, release transparency | CI; `mini-provenance`; release/installer path | **PARTIAL** | K independently administered builders on exact release artifact |
| AI never authorizes | AI contributions are evidence only; charter explicitly non-authorizing | governance pack/charter; PR metadata; D-0084 | **PASS** formally | Need independent human population so practical process is not Founder + AI only |
| Economic anti-whale shape | Issuance ceilings, Human Share floor, voice/value wall, simulations | D-0074; `mini-econ-sim`; economic gate docs | **PARTIAL** | Specialist calibration; current F5 failures block provider subsidy activation |
| Provider anti-collusion | F5 model / mechanism sweeps | PR #285 and related simulation results | **FAIL** | Colluding genuine-delivery drain and adaptive audit grinding unresolved |
| Legal launch posture | Technical legal brief exists | `docs/gates/legal-review-brief.md` | **FAIL** for real value | Qualified counsel GO/GO-WITH-CONDITIONS for actual rails/jurisdictions |
| Weak-device equality | Mobile/desktop foundations, bounded protocols, weak-device doctrine | Android/Windows client PRs; hardware gate docs | **PARTIAL** | Real old/cheap-device CPU/RAM/battery/bandwidth acceptance data |
| Edge-provider independence | Pluggable transport/bridge seams; no runtime GitHub requirement | `mini-bridge`; PT process manager; no-GitHub demo; Directive 18 | **PARTIAL** | Demonstrate production operation after removal of each named external convenience provider |

## Evidence precedence

When evidence conflicts, use this order:

1. current executable code and adversarial tests;
2. current `docs/INVARIANTS.md` and canonical Founder Directives;
3. current `docs/DECISION_LOG.md` entry that supersedes older decisions;
4. living `docs/STATUS.md` / `docs/ROADMAP_TO_RELEASE.md`;
5. point-in-time internal audit reports;
6. historical PR descriptions and superseded design notes.

A historical PASS is not a current PASS if later code changed the reviewed surface. A merged PR is evidence that code entered `main`; it is not external security certification.

## Reviewer output requirement

For each row reviewed, the external report should record:

- exact commit/release digest;
- files/crates actually reviewed;
- tests/vectors independently executed;
- findings with severity and exploit/precondition;
- PASS / PARTIAL / FAIL;
- exact residual risk;
- required fix and owner;
- whether a later delta requires re-review.

Use `docs/gates/EXTERNAL_AUDITOR_TEST_ITINERARY.md` for the full execution order and `docs/gates/REAL_VALUE_RELEASE_CHECKLIST.md` for the binary launch gate.
