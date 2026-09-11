# Complete review publication - 2026-09-08

**All four prepared review documents are published in this directory.** The historical review covers every one of the 174 PRs through #326: 166 merged and 8 closed-unmerged. Its source baseline is `684baa103c075299a375d1ed9a3327185c0ba02b`; this publication does not silently extend the review to later code or PRs.

## Read the review

| Document | Contents |
|---|---|
| [Master review](MASTER_REVIEW.md) | Development history, founder vision, all 18 directive verdicts, mechanisms, failure locations and long-term improvements. |
| [All 174 individual PR reviews](ALL_174_PR_REVIEWS.md) | Every PR's contribution, evidence, limitations, recommended improvements, concrete example, acceptance-test proposals and later corrections. |
| [Findings and improvements](FINDINGS_AND_IMPROVEMENTS.md) | All 24 cross-system findings, source locations, failure mechanisms, explained remedies, examples, test proposals and implementation order. |
| [Complete offline HTML report](EXHAUSTIVE_REVIEW.html) | Combined reports, searchable PR navigation, cryptography/public-code reuse comparisons, auditor itinerary and historical validation section. Save the raw HTML and open it locally; this is a repository file, not a hosted site. |

The four reports preserve their supplied attachment bytes exactly. Their terminology, conclusions and historical limits have not been rewritten during publication. Later corrections belong in dated addenda, not silent changes to the originals.

## Traceability and reproducible checks

[EVIDENCE_MAP.json](EVIDENCE_MAP.json) maps all 174 dossiers to their captured heads, outcomes and all 2,759 changed-file records. It records absent API patches rather than pretending they were reviewed. Its `review_completed: false` fields are retained from the evidence exporter: collecting a record was never approval or completion of substantive review. The dossier is the separate research output.

[PUBLICATION_MANIFEST.json](PUBLICATION_MANIFEST.json) records byte lengths, SHA-256 digests and Git blob hashes for all four originals and the evidence map. [VALIDATION_RECORD.md](VALIDATION_RECORD.md) distinguishes actual publication checks from historical test claims and proposed acceptance tests.

Run the standard-library-only publication checker from a trusted checkout of the published commit:

```sh
python3 docs/audits/pr-history-2026-09-08/verify_publication.py --self-test
```

It checks exact bytes, the 174-PR inventory, captured heads/base references, required dossier sections and internal HTML navigation. Its two negative tests require truncated and missing companions to fail. These are integrity/coverage checks, not proof of security, correct research, independent humans or legitimate approval. The manifest and checker do not authenticate an untrusted checkout by themselves.

To regenerate the evidence map without executing or extracting repository code:

```sh
python3 docs/audits/pr-history-2026-09-08/build_evidence_map.py \
  /path/to/mininet-pr-history-through-326-revalidated.zip /tmp/EVIDENCE_MAP.json
cmp /tmp/EVIDENCE_MAP.json docs/audits/pr-history-2026-09-08/EVIDENCE_MAP.json
```

The generator requires the exact captured archive's size and SHA-256. It refuses to overwrite prior output.

## Original evidence retention

The [raw evidence archive](https://github.com/mininet-labs/Mininet/actions/runs/34200283626/artifacts/10045573430) contains the original inventory, PR discussions/reviews/file patches, canonical-source archive and Git bundle. Its SHA-256 is `4dbb5b4f8f6e7911e5f048be3bbb34b8771503d6702a288854e534e6343774cd`; its recorded expiry is **2026-09-22 07:39:32 UTC**. It is temporary Actions storage, not a permanent independent archive. Preserve independently controlled copies before expiry. The committed evidence map survives but is not a substitute for the full raw discussions and Git objects.

The initial 2026-09-07 capture used `0ed3a4e523be195d39296eeca16a187fca6a599b`; this publication uses the reports' later 2026-09-08 capture. Both scopes remain explicit.

## Recommendations are not implemented fixes

The [narrow FROST/sequence patch](patches/0001-frost-and-sequence-hardening.patch) remains a proposed patch file, not a change applied to production crates or `main`. Publication does not implement every finding, grant approval weight, change frozen invariants or close an external gate. Temporary report-transfer and patch-validation workflows have been removed from this publication tree; their historical run records remain separate evidence.

**AI-authored internal research is not an independent external audit, human approval or production authorization.** The report's production judgment remains NO-GO for the pinned source. Publishing a complete corpus does not change that judgment.
