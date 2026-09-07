# Pull-request history ledger — how to reconstruct every change

**Scope:** GitHub repository history only. Pull requests and issues share GitHub's number namespace, so missing PR numbers do **not** imply missing history. For any row below, the linked PR description, commits, review conversation and merge commit are the primary evidence; this ledger is a navigation/reasoning layer.

**Audit rule:** never infer current correctness from “merged.” A merged PR proves that a change entered the canonical Git history. Later PRs may fix, supersede, narrow or invalidate its claims. Always read forward to the current `main`, `docs/DECISION_LOG.md`, `docs/FAILURE_BOOK.md`, `docs/STATUS.md`, and `docs/ROADMAP_TO_RELEASE.md`.

## Era 1 — core protocol bootstrap

| PR | What changed | Why it mattered | Auditor follow-through |
|---|---|---|---|
| [#1](../../pull/1) | `mini-chain` finality core, equal root vote weight | Establish money/voice separation in consensus | Identity root ≠ unique human; later `mini-consensus` provides networked rounds |
| [#2](../../pull/2) | storage receipts + `mini-net` | First verifiable infrastructure contribution + in-house routing | Later PoST/PoRep and discovery harden this |
| [#3](../../pull/3) | whitepaper reconciliation; UWB/personhood/storage/treasury/value scaffolding | Align implementation with Founder vision and explicitly separate safe bookkeeping from unaudited crypto | Read D-0035 and later superseding decisions |
| [#4](../../pull/4) | stealth-address + ring-signature prototypes | First executable private-value crypto | Still externally unaudited |
| [#5](../../pull/5) | Bulletproofs, FROST, personhood status, possession-over-time | Converted several stubs into real protocol code | External audit remained required; personhood not solved |
| [#6](../../pull/6) | real TCP bearer + multi-process gossip demo | Proved sockets, not only in-process simulation | No auth/encryption in raw TCP layer by design; later Channel wraps it |
| [#7](../../pull/7) | Founder Directives, Failure Book, roadmap, CI security, internal audits, hard external-audit gate | Made honesty/audit policy repository-visible | D-0047 remains a production gate |

## Era 2 — identity, bounty, settlement, custody and external-gate formalization

| PR | What changed | Why it mattered | Auditor follow-through |
|---|---|---|---|
| [#94](../../pull/94) | bounty, addressing, threat model, traceability, recovery and Sybil hardening | First strong system-wide threat/traceability pass | Unresolved Sybil/storage/governance risks intentionally remained |
| [#95](../../pull/95) | settlement, external legitimacy gates, zeroize hardening, dealerless FROST DKG/resharing | Removed the trusted dealer from the intended real ceremony | DKG remains externally unaudited; old shares are not magically revoked |
| [#100](../../pull/100) | finalized execution + real transport interop | Canonical state began to advance behind actual QC verification | Later networked consensus and body binding tighten this |
| [#101](../../pull/101) | PoRep, erasure coding, first `mini` CLI/forge spine | Storage proof and self-hosted development moved from design to code | Later erasure/PoRep defects were found and fixed |

## Era 3 — self-hosted Forge, release and recovery spine

| PR | What changed | Reason / trust boundary |
|---|---|---|
| [#103](../../pull/103) | build provenance + isolated Wasmtime runner | Untrusted build steps must not inherit forge authority |
| [#104](../../pull/104) | TUF-inspired release checks | Rollback/equivocation/freshness/provenance gates before adoption |
| [#105](../../pull/105) | real installer | Explicit owner approval separates verification from installation |
| [#106](../../pull/106) | erasure MDS fix + network sync | External review found a genuine coding-theory defect; also removed shared-filesystem sync assumption |
| [#107](../../pull/107) | Git SHA-256 export + treasury/inflation/human-continuity decisions | Self-hosted code portability and new economic/personhood direction |
| [#108](../../pull/108) | pre-audit simulation/hardware/DTN prep | Turned outside gates into runnable packages; first economic sweep had known model limitation |
| [#109](../../pull/109) | continuous forge→build→release→install→rollback harness | Proved vertical integration, while exposing missing CLI glue |
| [#110](../../pull/110) | persistent installer event log + CLI build/release/provenance/install | Made the spine auditable across process restarts |
| [#111](../../pull/111) | stable JSON CLI output | Machine consumers no longer scrape human text |
| [#112](../../pull/112) | adversarial release/install CLI fixtures | Tested no-quorum, self-attestation, timelock and state-order rejection |
| [#113](../../pull/113) | release reaches independent peer over `mini sync` | Proved full object replication, not only local composition |
| [#115](../../pull/115) | no-GitHub outage demo | Proved GitHub is not a runtime requirement for the development lifecycle |

## Era 4 — networked consensus

| PR | What changed | Reason / remaining risk |
|---|---|---|
| [#114](../../pull/114) | real-socket multi-round Tendermint, view change, signed proposals, bounded mesh | First actual networked consensus | Later equivocation, encryption, state sync and body binding harden it |
| [#116](../../pull/116) | equivocation proof + re-gossip over partial mesh | Honest nodes need accountability and non-full-mesh liveness |
| [#119](../../pull/119) | timestamp/replay/fee edge-case review | Found proposer-controlled context problems; later #124 reconciles parallel work |
| [#120](../../pull/120) | encrypted consensus channels | Removed cleartext link exposure without changing signed consensus semantics |
| [#121](../../pull/121) | independent parallel hardening attempt | Important because its fee-overflow finding was later adopted rather than blindly merging duplicate work |
| [#124](../../pull/124) | deterministic timestamps + fee overflow fix | Reconciled #121 with already-merged D-0085 and tightened logical time |
| [#125](../../pull/125) | KEL freshness pins + equivocation consequence registry | Stale-KEL returning-verifier and dropped-evidence gaps became explicit mitigations |
| [#128](../../pull/128) | killed-transfer resume + multicast discovery | Proved fresh-connection recovery and local serverless discovery |
| [#129](../../pull/129) | PEX | One known peer can yield another dialable peer without a central directory |
| [#130](../../pull/130) | state sync/catch-up | Late node can reach canonical finalized state through QC-verified history |

Later hardening an auditor must read forward includes [#289](../../pull/289) (snapshot/persistent state-sync hardening), [#300](../../pull/300) (body-root binding), [#318](../../pull/318) (encrypted PEX) and [#319](../../pull/319) (validator-authenticated channel). The current roadmap still identifies the shielded-spend chain-validity rule as open.

## Era 5 — governance honesty and canonical values

| PR | What changed | Auditor significance |
|---|---|---|
| [#117](../../pull/117) | governance pack integrated as subordinate tooling/docs | Did not replace constitution/invariants |
| [#118](../../pull/118) | Primary AI Engineer charter + D-0083 Founder exception | **Current centralization red flag:** zero required independent approvals during bootstrap |
| [#123](../../pull/123) | `FullHuman` renamed `EvidenceQualifiedHuman` | Corrected an overclaim: code did not prove a unique human |
| [#126](../../pull/126) | credential taxonomy + custody-separation clause | Separated claim classes and committees in design |
| [#127](../../pull/127) | canonical Founder Directive registry | Reduced competing constitutional-number ambiguity; later D-0352 added Directive 18 |
| [#157](../../pull/157) | machine-readable work claims | Parallel AI/contributor coordination without granting AI approval weight |

The auditor must check current `governance/bootstrap-operating-state.json` and `.github/CODEOWNERS`, not assume D-0083 has sunset merely because it is described as temporary.

## Era 6 — privacy/cost doctrine, object privacy and transport layers

| PR | What changed | Auditor significance |
|---|---|---|
| [#131](../../pull/131) | privacy-cost doctrine + policy vocabulary | Made residual privacy floors explicit rather than pretending spending removes them |
| [#138](../../pull/138) | transport policy router | Fails closed when requested protection exceeds a tier |
| [#139](../../pull/139) | resource-price quoting | Checked arithmetic and deliberate separation from governance |
| [#140](../../pull/140) | human-evidence taxonomy reconciliation | Refused to create a rival “personhood” ladder from incomparable signals |
| [#141](../../pull/141) | encrypted object envelope, capability grants, scoped pseudonyms | Moves metadata inside authenticated ciphertext; no new crypto primitive |
| [#142](../../pull/142) | Sphinx/Loopix research and attack catalog | Design/research only; did not authorize anonymity claims |
| [#143](../../pull/143) | consolidation of conflicting privacy lanes | Same content replayed cleanly onto current main |
| [#145](../../pull/145) | Tier-1 relay/rendezvous protocol | Explicitly no mixnet claim |
| [#146](../../pull/146) | route-policy wiring + live two-hop TCP demo | Demonstrated hop-by-hop relay, explicitly not onion routing |
| [#147](../../pull/147) | pluggable bridge + private-index boundary | External convenience and private lookup seams without making external adapter authority |
| [#150](../../pull/150) | Tor PT process manager | External binaries are digest-pinned byte transformers, not identity/governance authorities |
| [#151](../../pull/151) | PIR + anonymous resource-payment research prep | Correctly deferred new crypto until review instead of inventing it |

Later privacy/value composition to inspect includes [#296](../../pull/296), [#305](../../pull/305), and [#308](../../pull/308), plus the current D-0455/D-0457 private-fee/change and chain-backed-ledger work. The private-value path is still not externally audited.

## Era 7 — post-quantum and KEL witness evolution

| PR | What changed | Auditor significance |
|---|---|---|
| [#148](../../pull/148) | ML-DSA-65 verify-only | Added standardized PQ verification without changing default identity suite |
| [#149](../../pull/149) | KEL witness/freshness design | Explicitly chose no global identity ledger and no witness authority |
| [#176](../../pull/176) | ML-DSA keygen + isolated signing | Continued PQ path; full KEL migration still separate |
| [#180](../../pull/180) | witness receipt types | First implementation slice for first-contact freshness |
| [#187](../../pull/187) | witness state-machine continuation | Builds observation/receipt semantics |
| [#191](../../pull/191) | `KelAssurance` / observation / duplicity registry | Makes assurance explicit instead of binary “verified” wording |
| [#320](../../pull/320) | receipt-collection continuation | Current roadmap still marks gossip/persistence/rotation/authority consumption incomplete |

D-0459 is especially important: it corrected an earlier caller-supplied witness-policy trust flaw by deriving policy from the signed KEL.

## Era 8 — social, intake, search and client surfaces

| PR | What changed | Auditor significance |
|---|---|---|
| [#152](../../pull/152) | free commons/open-search policy | Search/reach design separated from protocol authority |
| [#153](../../pull/153) | intake types/review states | External input starts untrusted/unreviewed by construction |
| [#154](../../pull/154) | social/storage/forge validation hardening | Fixed permissive decoding at trust boundaries |
| [#158](../../pull/158) | filesystem traversal/index hardening | Narrow query performance + symlink/path poison defense |
| [#159](../../pull/159) | intake coordinator | Ingest stores external bytes without granting them signed-project authority |
| [#160](../../pull/160) | MiniSearch shared vocabulary | Ranking/restriction/personalization split into explicit types |
| [#162](../../pull/162) | deterministic crawler planning | Admission/policy before network side effects |
| [#170](../../pull/170) | Windows social/UI beta integration | Explicitly not production-ready; useful real-client integration evidence |
| [#172](../../pull/172) | isolated extractor protocol/host | Treats parsers as hostile process boundary |
| [#179](../../pull/179) | Android foundation | Explicit maturity labels; no fake custody claims before platform adapter exists |
| [#242](../../pull/242) | accepted-review gate before intake linking | Fixed an authority-escalation path in intake semantics |

Search/intake later expand through the #252 and #256–#283 series. Audit current crates and tests rather than treating this table as a completeness proof of search functionality.

## Era 9 — economics, storage and private-value adversarial hardening

The later history is best read as a sequence of adversarial corrections, not a straight “feature-complete” march:

- [#271](../../pull/271) — day-0 monetary kernel / corrected economic foundation.
- [#285](../../pull/285) — F5 anti-collusion simulations. **Do not miss the FAIL results:** colluding genuine-delivery drain and adaptive audit grinding remained exploitable and the mechanism stayed unauthorized for Phase 3.
- [#297](../../pull/297) through [#306](../../pull/306) — storage fraud, capacity, PoRep and related hardening. Several real flaws were found and fixed, including proof-shape/challenge assumptions.
- [#305](../../pull/305) — private payment composition.
- [#308](../../pull/308) — private view/disclosure evolution.

This is why an auditor should regard tests as evidence of known cases, not proof that an unaudited construction is safe.

## Complete-history audit procedure

The table above annotates the architectural dependency chain and all PRs that changed the project's trust model materially. To inspect **every** pull request, including maintenance/fix/CI/documentation PRs not individually summarized here:

1. Open the repository’s GitHub **Pull requests → Closed** view and sort chronologically from oldest to newest.
2. For every merged PR, record: PR number/title, merge commit, authoring/AI disclosure, changed crates/docs, D-number(s), tests claimed, reviewer comments, failures discovered during review, and whether a later PR supersedes it.
3. For every closed-unmerged PR, record why it was abandoned/replaced; parallel PRs such as #121 matter because unique findings can be adopted elsewhere.
4. Search `docs/DECISION_LOG.md` for every D-number named by the PR and then search forward for “supersedes”, “tightens”, “reconciles”, or later entries touching the same mechanism.
5. Search `docs/FAILURE_BOOK.md` and `docs/audits/` for the same subsystem.
6. End at current `main`; never stop at the PR’s own test claims.

### Per-PR worksheet

For each PR, external reviewers should capture:

| Field | Required content |
|---|---|
| PR | number + title + link |
| Canonical outcome | merged / closed-unmerged / superseded |
| Merge revision | exact SHA |
| Domain | identity / personhood / consensus / value / storage / network / governance / forge / client / docs/tooling |
| Authority touched? | exact key, role, vote, custody, release or data authority affected |
| Centralization introduced? | yes/no + mechanism |
| Cryptography touched? | primitive/composition and external-audit status |
| Decision refs | D-number(s), invariant IDs, Founder Directive(s) |
| Claimed tests | commands and named adversarial cases |
| Independent evidence | external review, hardware log, specialist report, or none |
| Later correction | PR/D-number that superseded or fixed it |
| Current verdict | PASS / PARTIAL / FAIL against current main |
| Exact remaining failure | one concrete sentence |

This procedure is intentionally more demanding than copying PR descriptions into a static table: it preserves **all** PR detail—the review conversation, exact diff and later corrections—without freezing a misleading duplicate of GitHub’s canonical history inside another Markdown file.
