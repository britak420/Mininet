# Pre-Go-Live Governance Pause and Anonymous Bootstrap Participation

**Status:** Founder bootstrap decision; temporary operating override

**Activation:** Effective when this exact document becomes canonical on `main`.

**Sunset:** Automatic and irreversible at Mininet Go-Live, defined below.

## 1. Purpose

Mininet is still being built through a temporary GitHub bootstrap while its intended self-governance substrate, Mininet Forge, is not yet canonical. Governance machinery designed for a live, self-governing network must not become a circular dependency that prevents the engineering, independent review, audit, testing, and gate-closing work required to reach that live network.

This decision therefore pauses pre-Go-Live **governance process requirements** and makes fully anonymous bootstrap contribution and review explicitly sufficient. The pause is temporary. It creates no permanent Founder office, no permanent GitHub authority, no identity registry, and no precedent that survives Go-Live.

The purpose is not to lower technical correctness. It is to stop governance procedure from gating the work needed to make real governance possible.

## 2. Definitions

### Pre-Go-Live

The period before Mininet Forge is the canonical governance and development system and the project records an explicit Go-Live activation state.

### Go-Live

Go-Live occurs only when both facts are recorded canonically:

1. `forge_canonical = true`; and
2. `go_live = true` in the bootstrap operating state or its canonical Forge successor.

A calendar date, contributor count, GitHub event, token event, funding event, or founder statement by itself is not Go-Live.

### Governance process requirements

Procedures that allocate or constrain decision authority, including approval floors, reviewer-quorum rules, governance attestations, amendment classification, adversarial-governance review requirements, public reasoning periods, cooling periods, working-group authorization, constitutional-process routing, and procedural prohibitions on unfreezing a rule.

Technical tests, reproducibility checks, security properties, audit scope requirements, release correctness, and the substantive human-freedom protections named in Section 7 are not governance process requirements.

## 3. Decision: governance procedure is paused until Go-Live

During Pre-Go-Live, governance process requirements MUST NOT block:

- ordinary or protocol-critical engineering;
- cryptography engineering and remediation;
- review, testing, benchmarking, or audit work;
- repository integration by the bootstrap custodian;
- completion or closure of an engineering, audit, research, hardware, legal, economic, or other readiness gate when that gate's substantive evidence requirement has actually been met;
- correction, replacement, or unfreezing of a bootstrap governance rule needed to reach Go-Live.

During this period, no proposal is required to obtain a governance quorum, independent-governance approval count, persistent reviewer identity, constitutional amendment classification, adversarial governance review, public reasoning period, or cooling period merely in order to advance toward Go-Live.

The frozen-invariant amendment prohibition in `docs/governance/39_CONSTITUTIONAL_AMENDMENT_PROTOCOL.md` is suspended **as a procedural gate** during Pre-Go-Live. A bootstrap rule may therefore be explicitly unfrozen, corrected, superseded, or retired by a canonical Founder bootstrap decision without first satisfying the future network's amendment machinery.

This does not suspend the substantive protections in Section 7.

## 4. Decision: bootstrap contributors and auditors may be fully anonymous

Until Go-Live, Mininet MUST NOT require a contributor, reviewer, researcher, tester, auditor, or other external specialist to provide or maintain any of the following as a condition of participation, acceptance, compensation eligibility, technical weight, or gate closure:

- legal name;
- public name;
- pseudonym or stable alias;
- persistent DID or other persistent identifier;
- persistent signing key;
- institutional affiliation;
- employer identity;
- nationality or location;
- KYC identity;
- public biography or social account;
- credential whose use would reveal identity;
- continuity between separate pieces of work.

A participant may submit one piece of work fully anonymously and never return. A participant may use an ephemeral channel. A participant may route a report through another person. The project MUST NOT later require retroactive deanonymization of Pre-Go-Live contributors or auditors.

If a hosting platform such as GitHub exposes account or network metadata, Mininet MUST NOT describe that platform limitation as a Mininet identity requirement or claim that the platform itself provides perfect anonymity.

Pseudonymous and public participation remain permitted when voluntarily chosen. They are not required until or after Go-Live unless future Forge governance lawfully creates a narrowly scoped continuity requirement that still does not require legal-name disclosure.

## 5. Anonymous evidence can close a gate

A gate is closed by the quality and completeness of the evidence required by its substantive scope, not by the reviewer's public identity.

For an audit or expert-review gate, a fully anonymous report MAY close the gate when the closure record binds, as applicable:

- the exact code, commit, release, object, protocol, or specification reviewed;
- the scope and exclusions;
- the methods used;
- the questions examined;
- reproducible evidence, tests, attacks, proofs, or analysis sufficient for the claim being made;
- findings and severities;
- corrections or explicit residual risks; and
- the exact disposition of every material finding.

The report MAY state `reviewer: anonymous`. No stable pseudonym, signature, employment record, academic credential, or legal identity is required.

Where anonymity prevents verification of a reviewer's biography, employment conflicts, or prior authorship, the project MUST state that limitation rather than inventing assurance. The technical artifact stands or falls on inspectable evidence.

An anonymous reviewer receives no governance power, release authority, treasury authority, identity privilege, or continuing role merely because their work closes a gate.

### A1 / external cryptography audit gate

During Pre-Go-Live, A1 and the corresponding external cryptography review gates are **unfrozen as governance invariants**. Their substantive security purpose remains: value-bearing cryptography must receive serious review and material findings must be resolved or explicitly dispositioned before the relevant production claim.

The gate-closing mechanism, however, is evidence-based rather than identity- or governance-based. A fully anonymous cryptography audit may close the gate when it satisfies the scope and finding-disposition requirements above. `credentialed academic`, public firm identity, legal name, persistent pseudonym, or persistent signing key MUST NOT be interpreted as mandatory.

## 6. Bootstrap authority while governance is paused

Until Go-Live, the Founder bootstrap custodian remains the mechanical canonical-integration authority for the GitHub mainline. This is a temporary central control point and therefore a deliberate bootstrap liability, not a model for the live network.

The bootstrap custodian MAY:

- merge engineering work after the applicable technical checks;
- record that a readiness gate is complete when its substantive evidence is present;
- accept anonymous audit and review material;
- record or integrate bootstrap decisions required to reach Go-Live; and
- correct bootstrap governance machinery that would otherwise deadlock progress.

The bootstrap custodian MUST NOT use this temporary authority to create a permanent Founder role or to violate Section 7.

AI systems MAY engineer, test, model, review, draft, and supply evidence. AI output does not independently become canonical authority and does not itself create a Founder decision. Canonical integration remains a human bootstrap act until Go-Live.

## 7. Substantive protections that are NOT paused

This decision pauses governance procedure, not Mininet's reason for existing. The following protections remain binding throughout Pre-Go-Live:

- money, balance, stake, employer status, contribution volume, or hardware ownership MUST NOT purchase political authority;
- no permanent owner, administrator, recovery master, platform authority, or Founder key may control the network;
- no kill switch;
- no hidden or unilateral unmasking path;
- no forced software update or forced owner adoption;
- owners retain voluntary adoption and possession of their own keys and data;
- free forking and exit remain possible;
- AI evidence does not become self-authorizing political power;
- technical claims must not exceed the evidence actually produced;
- canonical settlement correctness and other safety-critical protocol properties remain engineering requirements.

A change that violates one of these protections is not legitimized by calling it a bootstrap governance exception.

## 8. Supersession and precedence during Pre-Go-Live

For the Pre-Go-Live period only, this decision supersedes conflicting bootstrap governance procedure, including:

- D-0083's limits to the extent D-0083 says the temporary Founder bootstrap exception may not affect governance-process gates, external-audit gate procedure, or frozen-governance procedure;
- the statements in `docs/governance/08_FOUNDER_BOOTSTRAP_AND_HANDOFF.md` that independent review, amendment gates, external audit governance requirements, and frozen-rule procedure always remain applicable during bootstrap;
- `docs/governance/39_CONSTITUTIONAL_AMENDMENT_PROTOCOL.md` to the extent it requires classification, adversarial-governance review, public reasoning, exact-state governance approval, cooling, or an already-defined unfreezing process before a Pre-Go-Live bootstrap rule can change;
- `governance/policy.yml` approval floors and constitutional-process requirements insofar as they would block Pre-Go-Live repository progress or gate closure; and
- `governance/exceptions.yml` calendar expiry and maintainer-count sunset conditions for the Founder bootstrap integration path.

This decision does not erase those records. They remain preserved history and become relevant again only as specified in Section 9 or as incorporated into Forge governance.

Where another bootstrap document conflicts with this decision about whether governance procedure can block Pre-Go-Live progress, this decision controls after canonical activation.

## 9. Automatic sunset at Go-Live

At Go-Live:

1. this procedural pause ends automatically and may not be extended by Founder instruction;
2. Mininet Forge becomes the canonical venue for governance authority;
3. the Founder bootstrap integration exception ends;
4. GitHub becomes a mirror or non-authoritative adapter according to the Forge transition state;
5. the governance and amendment rules adopted by the live Forge community apply prospectively;
6. no Pre-Go-Live contributor or auditor is required to reveal an identity, adopt a pseudonym, create a persistent key, or establish continuity retroactively; and
7. a gate validly closed under this decision remains part of the historical evidence set and does not automatically reopen merely because governance has activated. Forge may reopen a gate only because of a substantive new finding, changed code, changed scope, or an explicit new safety requirement applied prospectively.

The Founder has no unilateral power under this document after Go-Live.

## 10. Engineering implementation required by this decision

Canonical implementation SHOULD make the pause machine-readable and fail clearly rather than relying on prose. The implementation work is:

1. replace the D-0083 calendar-based bootstrap profile with a Pre-Go-Live profile whose sunset is the explicit Go-Live/Forge-canonical transition;
2. add an explicit `go_live` state bit or equivalent canonical Forge transition object;
3. update governance validation so Pre-Go-Live governance quorum, amendment-process, cooling-period, and procedural frozen-domain checks do not gate candidate integration, while the Section 7 prohibitions and ordinary technical/security checks remain enforced;
4. update `governance/exceptions.yml` so the Founder integration path does not expire merely by date or contributor count before Go-Live;
5. update external-review templates to allow `reviewer: anonymous` with no persistent key or pseudonym requirement;
6. update cryptography, DKG, legal, economic, hardware, and other gate documents so identity, public credentials, or pseudonymous continuity are not prerequisites for gate closure;
7. update gate indexes and closure matrices so a completed anonymous review can be recorded as complete without misrepresenting what identity or independence evidence exists;
8. preserve exact reviewed-state, findings, dispositions, tests, and residual-risk evidence even when the human source is fully anonymous; and
9. encode Go-Live as the one-way transition that disables this bootstrap exception and activates Forge governance.

Until those machine changes are merged, this document records the intended operating decision but software that still implements older rules may continue to fail closed. Engineers should update those enforcement points rather than mislabeling the old failures as substantive gate failures.

## 11. Security and legitimacy analysis

### Benefit

The project can finish its own governance substrate without requiring that unfinished governance substrate to authorize every step needed to build it. Experts can contribute without accepting identity exposure, institutional dependence, or a permanent reputation handle. Technical gates remain evidence-bearing instead of becoming status checks on who the reviewer is.

### Cost

The Founder remains a centralized canonical-integration point during bootstrap. Fully anonymous review also makes biography, institutional competence, and conflicts of interest harder or impossible to verify.

### Mitigation

The centralization has one hard sunset: Go-Live. It grants no participant political weight, no permanent Founder authority, no forced adoption, and no hidden control path. Anonymous review must bind inspectable technical evidence and must state identity-derived assurance as unknown instead of pretending it was verified.

### Failure point

If Go-Live can be indefinitely avoided while the Founder continues to control canonical integration, this temporary pause becomes de facto permanent centralized governance. The long-term solution is therefore not another GitHub governance layer. It is to complete Forge, make it canonical, execute the one-way Go-Live transition, and remove Founder bootstrap authority.

## 12. Overall judgment

**PASS for the Pre-Go-Live bootstrap only.** The pause serves Mininet's free-Internet purpose because it removes premature governance bureaucracy and identity pressure while preserving substantive human-freedom protections and technical evidence requirements.

**FAIL if retained after Go-Live.** Permanent Founder integration authority would contradict Mininet's anti-centralization purpose. The exact required end state is Forge-canonical governance with no unilateral Founder authority and no retroactive deanonymization of early contributors or auditors.
