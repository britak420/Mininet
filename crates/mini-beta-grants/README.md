# mini-beta-grants

Campaign-scoped multi-party acceptance evidence for **Beta MINI only**.

This crate exists because `mini-beta::BetaMiniLedger` deliberately does not decide who may issue a shared-network grant. Filling that gap with one Founder key, GitHub bot, faucet server, or treasury operator would create a trusted issuer. `mini-beta-grants` instead requires a short-lived campaign policy and multiple distinct operational authorizer DIDs before the underlying Beta ledger may apply a grant.

## What it proves

Given one exact campaign, policy, grant and immutable approval set, independent nodes can deterministically check:

- policy author/campaign/epoch/time consistency;
- a minimum three-member authorization set;
- testing and participation thresholds of at least two;
- unique DID counting (one DID cannot multiply its own weight);
- an exact testing-grant amount;
- explicit participation reward bands;
- same-campaign contribution evidence for participation rewards; and
- exact policy/grant binding for every approval.

The wrapper `SharedBetaLedger` calls `BetaMiniLedger::apply_grant` only after threshold acceptance succeeds, preserving the accounting core's separate supply, campaign-cap, wrong-epoch and one-contribution/one-award checks.

## What it does **not** prove

A `did:mini` is not proof of a unique human. The current campaign record authority chooses the temporary policy membership, so several listed DIDs could still be controlled by one actor. This crate removes **unilateral grant issuance**, not the remaining Pre-Go-Live bootstrap selection dependency.

That distinction is intentional. The long-term fix is Forge-native governance/personhood and the one-way bootstrap-authority shutdown tracked by #338, not pretending multiple keys automatically mean multiple independent humans.

This crate also provides no rollback-free distributed finality. It can detect signed authorizer equivocation and produces deterministic validation once peers hold the same objects, but canonical conflict resolution for durable shared execution belongs to #337/#338.

## Hard walls

This crate must not depend on production value, settlement, treasury, chain/consensus, personhood/economy, Forge governance, or GitHub APIs. Its dependency-wall test enforces the expected runtime dependency list.

Beta MINI accepted here:

- is resettable test value;
- has no production-MINI conversion promise;
- does not increase governance/review/release/personhood authority;
- does not make authorizer membership a political role; and
- cannot be made valid by wealth, balance, employer, GitHub account, reward history or contribution count.

See `docs/design/decentralized-beta-grant-acceptance.md` for the full threat model and acceptance plan.
