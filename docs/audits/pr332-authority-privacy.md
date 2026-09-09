# PR 332 authority and privacy checkpoint

Engineering proposal. This checkpoint does not close the original audit or
authorize production use, merge, governance promotion, or real-value operation.

## Implemented boundaries

- F-13: `AuthenticatedObjectOwner` is minted by re-deriving an object's content
  address and checking its device signature and root delegation. Validation
  binds the owner proof to the exact requested object. A raw grant issuer DID
  cannot substitute for ownership. Holder proofs also require a random,
  verifier-issued, single-use `CapabilityRequest`; its context should commit to
  the exact operation, session, and audience. The verifier consumes the request
  on successful validation. Fresh KEL selection remains a session obligation.
- F-09: published intake posts sign both intake ID and source digest. Recovery
  rejects ordinary same-author/same-text posts lacking these bindings and wrong
  links. Envelope decode rejects impossible review/authority/link combinations,
  duplicate links, and mismatched representation provenance. Superseded records
  preserve valid historical links. Load, deduplication, and save check the
  requested/derived intake ID. The CLI transaction/durability work is described
  separately by its implementation owner.
- F-15: authenticated application sends pass through `dispatch_transport`.
  Dispatch enforces requested tier, matching direct peer or verified onion
  entry, exact route roles, and implemented transport properties. Mixed/Burst,
  tier fallback, and storage/personhood claims fail before application send.
  `PublicationRoutingPlan::dispatch` preserves the original requested properties.
  Only successful local bearer submission creates `ObservedSendReceipt`.
- F-21: score/provider fields cannot be mutated while retaining a local origin.
  Remote scores cannot determine URL order or displace locally computed results.
  Competing remote claims remain available with their profiles, observations,
  index references, titles, and scores; repeated identical claims deduplicate.
  Reweighting explicitly labels reuse of the original diversity signal.

## Limits and compatibility

Intake Accepted/authority fields remain local workflow labels, not independently
authenticated human or governance review. F-09 therefore remains PARTIAL for
public authority. Legacy pending publications without signed intake/source links
fail closed and need explicit operator recovery; published ordinary posts still
decode normally. Capability holder proofs now require the new request challenge.

A send receipt proves local submission only. An onion receipt does not prove
forwarding by every relay, destination receipt, anonymity under relay collusion,
or traffic-correlation resistance. Low-level anonymous CH1 and packet builders
remain available as protocol primitives and cannot create this receipt. F-15's
external observer-model validation remains open.

Remote profile/source identifiers are locators, not proof of metadata truth or
canonical-link correctness. Local scoring recomputes a formula over held inputs;
it does not establish that all input claims are true. F-21 remains PARTIAL before
public federation and query privacy remains limited to the selected transport.

## Validation at checkpoint

Adversarial tests were added for wrong owner/object, mutated objects, revoked
delegation, request replay/context mismatch, forged envelope combinations,
substituted intake IDs, signed source-link substitution, hostile maximum scores,
disagreement preservation, and no-send transport refusals. The focused nine-package Cargo suite completed successfully. Additional compile-fail and TCP transport tests are included in the final workspace validation. Full workspace Clippy passed on the follow-up before the subsequent validator-admission integration. CI and current-head CodeQL must still confirm the pushed state; these local runs do not establish production readiness.
