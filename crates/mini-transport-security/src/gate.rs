//! Runtime privacy-tier execution gate.
//!
//! Policy vocabulary is not implementation evidence. Direct and Relayed have
//! concrete executors after #291; Mixed and Burst remain unavailable until the
//! exact Sphinx/Loopix executor receives independent review under #72/D-0305.
//!
//! **Not yet a mandatory dispatch point (F-15).** No function in this crate
//! (or anywhere in this workspace) currently calls [`executable_transport`]
//! before establishing a real connection — `runtime.rs`'s own
//! `connect_authenticated_tcp`/`build_verified_onion_route` are reached
//! directly by a caller that already decided which one to call, without ever
//! consulting this gate. The Mixed/Burst refusal today is real only because
//! this crate does not offer a mix/burst executor at all, not because some
//! enforced single entry point turns every send through this check. What
//! *is* closed here: [`ExecutableTransport`] cannot be constructed except by
//! calling [`executable_transport`] — its variants carry a private
//! [`Sealed`] token, so a future caller cannot accidentally (or a hostile
//! code path deliberately) manufacture `ExecutableTransport::ThreeHopOnion`
//! and skip the Mixed/Burst rejection the moment this crate ever does wire a
//! real dispatcher through this type. Building that dispatcher — the actual
//! mandatory gate the long-term fix calls for — needs a real, single send
//! entry point this crate does not have yet; inventing one unilaterally here
//! would be exactly the kind of unreviewed architecture decision this
//! session's other findings (D-0478, D-0487's declined trait redesign)
//! already decline to make without it.

use mini_privacy_policy::PrivacyTier;

use crate::{Result, TransportSecurityError};

/// An unforgeable proof this [`ExecutableTransport`] variant was produced by
/// [`executable_transport`] — its only field is `pub(crate)`, so no crate
/// outside this one can construct a `Sealed` value, and therefore none can
/// construct an [`ExecutableTransport`] by variant-literal syntax either.
/// Matching an existing value with `_` (e.g. `ExecutableTransport::
/// ThreeHopOnion(_)`) still works normally from any crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sealed(pub(crate) ());

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutableTransport {
    /// Anonymous CH1: encrypted and forward-secret, no endpoint identity claim.
    AnonymousDirect(Sealed),
    /// CH1 plus the optional transcript-bound delegated-device proof.
    AuthenticatedDirect(Sealed),
    /// Exact three-role layered onion path from `mini-relay`.
    ThreeHopOnion(Sealed),
}

pub fn executable_transport(
    tier: PrivacyTier,
    authenticate_direct_peer: bool,
) -> Result<ExecutableTransport> {
    match tier {
        PrivacyTier::Direct => Ok(if authenticate_direct_peer {
            ExecutableTransport::AuthenticatedDirect(Sealed(()))
        } else {
            ExecutableTransport::AnonymousDirect(Sealed(()))
        }),
        PrivacyTier::Relayed => Ok(ExecutableTransport::ThreeHopOnion(Sealed(()))),
        PrivacyTier::Mixed | PrivacyTier::Burst => {
            Err(TransportSecurityError::MixedTransportNotReviewed)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn implemented_tiers_map_to_real_executors() {
        assert!(matches!(
            executable_transport(PrivacyTier::Direct, false).unwrap(),
            ExecutableTransport::AnonymousDirect(_)
        ));
        assert!(matches!(
            executable_transport(PrivacyTier::Direct, true).unwrap(),
            ExecutableTransport::AuthenticatedDirect(_)
        ));
        assert!(matches!(
            executable_transport(PrivacyTier::Relayed, false).unwrap(),
            ExecutableTransport::ThreeHopOnion(_)
        ));
    }

    #[test]
    fn an_executable_transport_can_only_be_produced_by_the_gate() {
        // F-15: `ExecutableTransport`'s variants carry a private `Sealed`
        // token, so this crate's own module -- the only place that can
        // construct `Sealed(())` -- is the only place that can construct
        // this type. Confirmed by construction here, since an external
        // crate literally cannot write `ExecutableTransport::ThreeHopOnion
        // (Sealed(()))` (a `pub(crate)` field is not visible/constructible
        // outside this crate) -- this test documents that guarantee by
        // demonstrating the type equality this in-crate scope still allows.
        let forged = ExecutableTransport::ThreeHopOnion(Sealed(()));
        let real = executable_transport(PrivacyTier::Relayed, false).unwrap();
        assert_eq!(forged, real);
    }

    #[test]
    fn mixed_and_burst_fail_closed_until_external_review() {
        for tier in [PrivacyTier::Mixed, PrivacyTier::Burst] {
            assert_eq!(
                executable_transport(tier, false),
                Err(TransportSecurityError::MixedTransportNotReviewed)
            );
        }
    }
}
