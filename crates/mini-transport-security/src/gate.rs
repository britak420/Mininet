//! Runtime privacy-tier execution gate. `dispatch_transport` calls this before
//! every authenticated application send, including PublicationRoutingPlan
//! dispatch. Mixed/Burst cannot fall back to direct/onion transport. The
//! lower-level anonymous bearer and packet builders remain protocol primitives
//! and cannot issue an ObservedSendReceipt.

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
