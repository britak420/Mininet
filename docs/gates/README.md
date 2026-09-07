# External legitimacy gates

Scope packages for the issues tracked in
[#99](../../issues/99) — work where the
blocker is not "nobody wrote the code," it's that closing it genuinely
requires outside authority (a cryptography auditor, legal counsel, a
tokenomics specialist), real hardware, or a founder decision on open
research with no known construction. More Rust cannot close these.

Each file below is a **handoff package**: what needs review, the exact
questions the outside party must answer, and the hard constraints the
review must respect (never weaken the Constitution to satisfy a reviewer's
convenience). Engineering's job stops at preparing the package — checking
a box on #99 requires the named outside action to have actually happened.

## Unified completion pack

Two files now sit above the individual scopes without replacing them:

- [`EXTERNAL_AUDITOR_TEST_ITINERARY.md`](EXTERNAL_AUDITOR_TEST_ITINERARY.md) — ordered execution plan across cryptography, DKG/custody, personhood, consensus, KEL freshness, storage, hardware, economics, legal, reproducibility and governance decentralization, with PASS/FAIL evidence for each track.
- [`REAL_VALUE_RELEASE_CHECKLIST.md`](REAL_VALUE_RELEASE_CHECKLIST.md) — binary NO-GO checklist for real value and real-people governance. A closed issue is never sufficient evidence by itself.

The matching repository-history and evidence reconstruction lives in [`../audits/PROJECT_BUILD_HISTORY.md`](../audits/PROJECT_BUILD_HISTORY.md), [`../audits/PR_HISTORY_LEDGER.md`](../audits/PR_HISTORY_LEDGER.md), [`../audits/AUDIT_EVIDENCE_INDEX.md`](../audits/AUDIT_EVIDENCE_INDEX.md), and [`../audits/EXTERNAL_AUDIT_MASTER_REPORT.md`](../audits/EXTERNAL_AUDIT_MASTER_REPORT.md).

| File | Gates | Founder action needed |
|---|---|---|
| [`crypto-audit-scope.md`](crypto-audit-scope.md) | [#72](../../issues/72) | Engage a cryptography auditor |
| [`dkg-audit-scope.md`](dkg-audit-scope.md) | [#93](../../issues/93) | Engage a cryptography auditor (same pool as #72, separate scope) |
| [`legal-review-brief.md`](legal-review-brief.md) | [#96](../../issues/96) | Engage counsel |
| [`personhood-signal-b-decision.md`](personhood-signal-b-decision.md) | [#21](../../issues/21) | Decide: deprioritize / fund research / accept TEE interim |
| [`hardware-test-protocol.md`](hardware-test-protocol.md) | [#97](../../issues/97) | Find a mobile engineer + real devices |
| [`economic-simulation-spec.md`](economic-simulation-spec.md) | [#47](../../issues/47), [#50](../../issues/50) | Engage a mechanism-design/tokenomics specialist |
| [`dtn-design-constraints.md`](dtn-design-constraints.md) | [#28](../../issues/28) | Find DTN/satellite-networking domain expertise |
| [`EXTERNAL_REVIEW_RESPONSE_TEMPLATE.md`](EXTERNAL_REVIEW_RESPONSE_TEMPLATE.md) | [#262](../../issues/262), all listed external gates | Record scope, evidence, findings, disposition, and residual limits |

This directory is append-only in spirit, same as `docs/audits/` and
`docs/design/` — when a gate closes, the package stays as the record of
what was asked and answered; a superseding review gets its own dated file
rather than an edit in place.
