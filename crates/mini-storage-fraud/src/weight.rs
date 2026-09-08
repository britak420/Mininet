//! The private scoring formula. Public callers must use checked provider standing.

use crate::ProvenCapacity;
use mini_spacetime::isqrt;

/// Parameters governing the weight formula. All integer, so weight is
/// exactly reproducible from the same proven-capacity input.
#[derive(Debug, Clone, Copy)]
pub struct ProposerParams {
    /// Per-identity cap on raw capacity counted (in whatever unit the
    /// caller's proof-of-space-time layer measures, e.g. GiB). Capacity
    /// beyond this contributes nothing further — the anti-concentration
    /// floor alongside the concave curve itself.
    pub capacity_cap_units: u64,
    /// Maximum bonus, as a percentage added on top of the base weight, for
    /// geographic/network diversity (e.g. spreading capacity across
    /// multiple distinct regions/network paths rather than one location).
    pub max_diversity_bonus_percent: u32,
    /// Bonus percentage granted per distinct region beyond the first,
    /// before the max cap above is applied.
    pub bonus_percent_per_extra_region: u32,
}

impl ProposerParams {
    /// A starting-point profile: a cap that keeps any single identity from
    /// dominating block production, and a modest diversity bonus. Tunable —
    /// the whitepaper specifies the *shape* (concave, capped, diversity-
    /// bonused), not these exact numbers.
    pub fn default_params() -> Self {
        ProposerParams {
            capacity_cap_units: 1_000_000,
            max_diversity_bonus_percent: 50,
            bonus_percent_per_extra_region: 10,
        }
    }
}

pub(crate) fn proposer_weight(
    capacity: ProvenCapacity,
    regions: u32,
    params: &ProposerParams,
) -> u64 {
    let base = isqrt(capacity.units().min(params.capacity_cap_units));
    let bonus = regions
        .saturating_sub(1)
        .saturating_mul(params.bonus_percent_per_extra_region)
        .min(params.max_diversity_bonus_percent);
    // u128 keeps arbitrary caller policy values from overflowing the formula.
    let result = u128::from(base) + u128::from(base) * u128::from(bonus) / 100;
    u64::try_from(result).unwrap_or(u64::MAX)
}
