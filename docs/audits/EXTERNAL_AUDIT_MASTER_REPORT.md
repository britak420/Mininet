# External audit master report — readiness, values, and exact blockers

**Audit scope**

| Field | Value |
|---|---|
| Reviewed at | `main` @ `2721f95db0f8be105d9582d64939d1110ed823b8` |
| Workspace size at that commit | 72 crates; resolved dependency count not independently measured in this GitHub-only review |
| Method | GitHub-only internal reconstruction of current value claims, mechanisms, blockers and external gates. No independent source-code audit, build/test execution, fuzzing, formal verification, hardware testing, legal review or cryptographic analysis was performed for this report |
| Tool versions | GitHub repository connector only; no code-analysis toolchain executed |
| Revalidation trigger | Any merge changing a value-bearing mechanism, authority boundary, current blocker/gate status, governance bootstrap state, or any source evidence supporting a PASS/PARTIAL/FAIL verdict |

**Status:** auditor-facing working report. This does not close any external gate.

**Authorship / non-substitution:** this is an **AI-drafted internal reconstruction** of repository evidence. It raises the review floor but is **not** an independent external audit and must never be cited as a substitute for the cryptography, custody, personhood, hardware, economics, legal, supply-chain or governance reviews it says are still required.

**Method:** every value is marked PASS / PARTIAL / FAIL. Each verdict names the concrete mechanism that supports it and the exact failure point that prevents a stronger verdict. Centralized authority, founder-only control, backdoors, trusted coordinators, and any path from wealth to authority are treated as red flags.

## Value verdicts

### 1. Humanity before technology — PARTIAL
**Mechanism proving it:** `docs/FOUNDER_DIRECTIVES.md` Directive 1, weak-device doctrine, one-human-one-vote intent, human-share design, local-first networking, and explicit refusal to treat AI output as authority.

**Exact failure point:** core rights still depend on an unsolved unique-human proof. The repository correctly says identity roots are not humans. Until that gap closes, the system cannot prove that human equality survives an adversary manufacturing apparently independent identities.

**Long-term fix:** complete and externally validate the unique-human/personhood mechanism before any production governance or Human Share claim.

### 2. No indispensable central authority — FAIL
**Mechanism proving progress:** runtime architecture avoids GitHub dependence; self-hosted forge/release/install exists; no protocol admin key is the design target; PEX/local discovery remove directory-server necessity.

**Exact failure point:** current repository governance remains founder-guarded. `.github/CODEOWNERS` routes critical paths to `@mininet-labs`; `governance/bootstrap-operating-state.json` is still active and records zero independent non-founder maintainers, `forge_canonical=false`, and a zero-approval Founder exception.

**Long-term fix:** sunset D-0083, install independent human maintainers, restore the independent review floor, and move canonical development/release governance to the self-hosted Forge or another protocol-governed path with no single-owner bypass.

### 3. Network outlives its creators — PARTIAL
**Mechanism proving it:** self-hosted forge, deterministic CLI, no-GitHub outage demo, reproducible release path, public CC0 repository, content-addressed storage and recovery logic.

**Exact failure point:** operational legitimacy still depends on Founder bootstrap authority and several external services/procedures not yet replicated by an independent community. The code can outlive the founder; current governance legitimacy has not yet proved that it can.

**Long-term fix:** decentralized maintainer/Forge transition plus external validation of recovery, release and governance processes under founder absence.

### 4. Money never outranks legitimacy — PARTIAL
**Mechanism proving it:** chain execution only advances behind verified finality; deterministic block time; checked fee arithmetic; canonical settlement rules; no balance-weighted vote path.

**Exact failure point:** the private spend path still has a chain-validity gap: a proposer can finalize a shielded-spend key image without the chain independently proving a valid private claim produced it. Real-money crypto has not passed external audit.

**Long-term fix:** add a chain-verifiable validity proof or independently validated verifier model, then complete R12 external cryptography review before real value.

### 5. Canonical truth for money — PARTIAL
**Mechanism proving it:** `mini-execution`, BFT finality, deterministic state roots, body-root binding, M1/M2/M3 settlement discipline and state sync all enforce a single finalized ordering.

**Exact failure point:** the shielded-spend admission issue above means the chain can be canonically consistent about a maliciously inserted nullifier/key image. Canonical ordering is not sufficient if the ordered item itself was never proven valid.

**Long-term fix:** make private-claim validity part of the consensus-verifiable state transition.

### 6. Design for failure — PARTIAL
**Mechanism proving it:** state sync, catch-up, durable snapshots, interrupted-transfer recovery, installer rollback, recovery/event logs, partial mesh gossip, bounded queues and fail-closed parsing.

**Exact failure point:** real-hardware BLE/UWB acceptance, mobile lifecycle, NAT/relay deployment, broad weakest-device resource measurements and adversarial operational drills are incomplete.

**Long-term fix:** execute the published hardware and failure matrices on real devices and publish raw evidence.

### 7. Free forks, earned legitimacy — PASS
**Mechanism proving it:** CC0 code, no owner-exclusive license, self-hosted forge, fork-legitimacy continuity criteria, owner-controlled update installation, no forced update path.

**Failure point:** none in the software freedom mechanism itself. The social claim that one branch remains “legitimate” is necessarily a community/process fact rather than cryptographic ownership.

**Long-term fix:** keep legitimacy evidence public and portable; do not introduce trademark, hosting or maintainer controls that become de facto fork vetoes.

### 8. Human as root of trust — PARTIAL
**Mechanism proving it:** governance/consensus designs count identity roots rather than stake; roles derive from `did:mini` delegation; AI has no approval weight.

**Exact failure point:** unique-human assurance is not solved. In addition, bootstrap repository authority is Founder-rooted today.

**Long-term fix:** externally validate unique-human proof, then remove Founder bootstrap authority.

### 9. Structural privacy — PARTIAL
**Mechanism proving it:** pairwise/scoped pseudonyms, encrypted object envelopes, private payment primitives, onion/relay work, no unmasking backdoor, content-blind channel design.

**Exact failure point:** transparent payment settlement still exists, making privacy optional and therefore itself identifying; private crypto is unaudited; public fee/input/output-count fingerprints remain; mix/PIR protections are incomplete or research-gated.

**Long-term fix:** retire transparent payments, complete the private-spend validity rule, audit the cryptography, and finish the metadata-protection stack with measured anonymity claims only.

### 10. Every optimization names its cost — PASS
**Mechanism proving it:** Failure Book, explicit honest-limit sections, audit gate docs, roadmap rows that distinguish `done`, `active`, `outside`, and recurring correction of overclaims.

**Failure point:** no structural failure found. Individual documents can still go stale, so this remains a maintenance discipline rather than a theorem.

**Long-term fix:** keep tests/checkers binding status claims to code and require reviewers to treat overclaiming as a defect.

### 11. Weakest device matters — PARTIAL
**Mechanism proving it:** Rust mobile foundations, light-client orientation, bounded protocols, local-first transport, weak-device storage goals and Android/Windows work.

**Exact failure point:** weakest-device measurements are not sufficient to prove the complete privacy/storage/consensus/client stack fits the target hardware. Hardware gates remain open.

**Long-term fix:** publish repeatable CPU, RAM, storage, battery and bandwidth benchmarks on old/cheap physical devices across the full acceptance path.

### 12. AI serves humanity, never rules — PARTIAL
**Mechanism proving it:** AI output is explicitly zero approval/quorum weight; governance charter is non-authorizing; repository PRs disclose AI authorship.

**Exact failure point:** because `independent_non_founder_human_maintainers` is currently zero, the practical review path can collapse to Founder + AI evidence even though AI formally has no authority.

**Long-term fix:** independent human reviewer population, especially for crypto/identity/consensus, before production release.

### 13. Think in centuries — PARTIAL
**Mechanism proving it:** crypto agility, ML-DSA work, append-only decisions, recovery, reproducibility, explicit century-scale directives and failure planning.

**Exact failure point:** dependency/supply-chain longevity, long-run economic calibration, post-quantum migration across the full identity/consensus/value stack, and real archival/recovery operations are not externally validated.

**Long-term fix:** external long-horizon review with scheduled re-audit and migration drills.

### 14. Simplicity is security — PARTIAL
**Mechanism proving it:** in-house bounded codecs, typed-domain APIs, fail-closed constructors, deliberate rejection of unnecessary frameworks in core paths.

**Exact failure point:** the repository is now broad and complex. Several security-sensitive constructions are custom, increasing audit burden; some past defects were found only after external or adversarial review.

**Long-term fix:** continue deleting redundant paths, converge to one private payment path, one canonical authority model, and prefer audited standard implementations where custom code adds no Mininet-specific value.

### 15. Honest majority, malicious minority — PARTIAL
**Mechanism proving it:** BFT thresholding, equivocation proof, bounded gossip, capability checks, witness receipts, fail-closed parsing and adversarial tests.

**Exact failure point:** personhood/Sybil resistance is the master dependency; F5 anti-collusion simulations also report unresolved malicious-provider drain/grinding.

**Long-term fix:** resolve personhood and F5 before high-value production operation.

### 16. Voice/value wall — PASS
**Mechanism proving it:** no balance/stake field in governance/finality weighting; separate crate/dependency boundaries; equal-unit vote counting; resource pricing kept outside governance; tests/reviewer checks explicitly inspect the wall.

**Failure point:** no direct code path found in the current GitHub material that lets wealth buy governance weight. Indirect social influence can never be eliminated by software, but it is not protocol vote weight.

**Long-term fix:** keep dependency-level checks and governance-code audits mandatory for every economic feature.

### 17. Future child test — PARTIAL
**Mechanism proving it:** slow/fair distribution intent, CC0, no owner key, privacy, weak-device and anti-whale policies.

**Exact failure point:** long-term tokenomics and unique-human distribution are not yet independently validated; premature real-value launch could permanently encode an unfair allocation.

**Long-term fix:** do not start irreversible real-value distribution until the personhood and economic gates close.

### 18. Edge convenient, core independent — PARTIAL
**Mechanism proving it:** pluggable transports, bridge adapters, external PT process isolation, no runtime GitHub dependence, bridge/twin framed as convenience rather than truth.

**Exact failure point:** some real-world usability still depends on unproven edge operations (mobile app distribution, relay/NAT infrastructure, external liquidity/custody, hardware/platform services). The architecture generally keeps these optional, but that claim needs deployment proof.

**Long-term fix:** demonstrate core operation with every named edge provider removed and document equivalent replacement paths.

## External legitimacy gates

| Gate | Current verdict | Evidence required to close | Exact blocker |
|---|---|---|---|
| Applied cryptography: value/private payments | **FAIL** | Independent cryptographer report against exact commit/release | Unaudited ring/stealth/Bulletproof/private-payment composition; chain-validity gap remains |
| Treasury/FROST DKG/custody | **FAIL** | Independent DKG/FROST/custody audit; ceremony and failure tests | DKG/FROST are tested but unaudited; real custody not authorized |
| Personhood / unique human | **FAIL** | Independent research/security validation with adversarial Sybil results | Evidence-qualified identity is not unique-human proof |
| Hardware BLE/UWB / device acceptance | **FAIL** | T1–T6/W1–W7 physical device logs and reviewer signoff | Not demonstrated on required real hardware matrix |
| Tokenomics/mechanism design | **FAIL** | Independent mechanism-design review over corrected models and F5 | F5 explicitly contains failing collusion/grinding scenarios |
| Legal review for real value | **FAIL** | Qualified counsel opinion for intended jurisdictions and rails | Repository says this cannot be closed by code |
| Reproducibility | **PARTIAL** | Independent K-builder/cross-machine build agreement on release artifacts | Current same-machine/scheduled CI is useful but not full independent reproducibility |
| Governance decentralization | **FAIL** | D-0083 sunset + independent maintainers + non-Founder canonical governance | Founder-guarded bootstrap is active |
| Storage operator independence | **FAIL** | Mechanism/tests proving replica placement is not one operator under many DIDs | Current proofs establish bytes/capacity, not independent control |

## Overall judgment

**NO — in its current state, Mininet is not yet ready to claim it serves a free Internet for humanity in production.**

The architecture is substantially aligned with that goal, and several values already have strong structural enforcement. But the project still has a live Founder control point, no production-grade unique-human proof, unaudited value/custody cryptography, unresolved private-spend consensus validity, unresolved storage-operator independence, open hardware/economic/legal gates, and a transparent-payment path that weakens structural privacy.

The correct engineering posture is not to soften those blockers. Close them, publish the external evidence against exact revisions, then rerun this report. The target state is clear: **no single authority, no wealth-to-voice path, no privacy downgrade path, no real value before independent cryptographic/economic/legal review, and no human-equality claim before personhood actually proves it.**
