//! Tier: BOUNDARY — this domain's strict value-object surface (the tier-1 grammar).
//!
//! meter — the credit meter's one value object, `Credits`, and its operator methods.
//!
//! Declaring `Tier: BOUNDARY` subjects this file to the full tier-1 grammar from our own
//! `build.rs`: no free functions, no public fields, no submodules, no re-exports, no I/O, no
//! `unsafe`, and a raw primitive may appear only as the lone field of a newtype wrapper —
//! exactly the discipline the library enforces on its own `kvstore::store`. Boundary-hood is
//! the declared tier plus the enforced shape, not a filename.
//!
//! CONSUMER NOTE: this fixture originally surfaced the finding that the citizen macros and
//! edge traits were sealed shut to downstream crates. That is fixed — `boundary::citizen`
//! is public (only the four-level EFFECT lattice stays truly sealed, because its laws are
//! proven exhaustively), so the `value_object!` / `value_operator!` registrations and the
//! `ParseCredits` Construction below are minted here, downstream, with the library's own
//! macros — and our build.rs's edge-probe completeness pass now has a real obligation to
//! enforce: delete the `impl Probed` below and this crate stops building.

use boundary_algebra::boundary::{reconstructs, Construction, Probed, Pure, Shaped, Unit};

/// The meter's ceiling: a balance is valid iff it lies in `0..=CAP`. The one number the whole
/// domain pivots on — `grant` saturates to it, the validity rule quotes it, the shadow grid
/// reaches it.
const CAP: i64 = 20;

/// A credit balance: a non-negative integer no greater than [`CAP`].
///
/// The validity rule is the only hand-written content; everything downstream of it —
/// the discovery grid, the algebra, the frozen spec — is derived. (In the library's own tree
/// this would be one `refined!` line; the macro is sealed to the library, so the consumer form
/// is the expanded shape: private field, smart constructor.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Credits(i64);

impl Credits {
    /// Parse-don't-validate: `Some` iff `n` is a valid balance (`0..=CAP`).
    pub fn new(n: i64) -> Option<Credits> {
        (0..=CAP).contains(&n).then_some(Credits(n))
    }

    /// The raw amount — the sanctioned exit hatch.
    pub fn get(&self) -> i64 {
        self.0
    }

    /// An arbitrary integer clamped into validity — the interior's constructor, so the
    /// workshop can compute freely in `i64` yet can only ever hand back a VALID balance.
    pub(crate) fn clamped(n: i64) -> Credits {
        Credits(n.clamp(0, CAP))
    }

    /// Top up by `other`, saturating at the ceiling. Discovery finds this commutative and
    /// associative (a semigroup); see the committed spec.
    pub fn grant(self, other: Credits) -> Credits {
        crate::internal::grant(self, other)
    }

    /// Deduct `other`, saturating at zero. Directional by design: the discovered spec says
    /// exactly how far its laws reach — deducting nothing is a no-op, an empty balance
    /// stays empty — and refuses commutativity and associativity, because the order of
    /// deductions is semantics. The refusals are the signal.
    pub fn spend(self, other: Credits) -> Credits {
        crate::internal::spend(self, other)
    }

    /// Renew from a voucher: a non-zero voucher REPLACES the balance, a zero voucher is a
    /// no-op. A non-commutative band — the discovered sandwich law states its bias (the later
    /// operand wins where the two disagree).
    pub fn renew(self, voucher: Credits) -> Credits {
        crate::internal::renew(self, voucher)
    }
}

/// The probe/grid surface, from structure: the canonical inhabitant plus one perturbation
/// class stepping toward the three semantically load-bearing points (up by one, the ceiling,
/// zero). `shadow_grid` closes over these from `Credits(0)`, so the discovery grid covers the
/// whole valid range INCLUDING both saturation points — which is what lets the engine refute
/// the false laws (`spend`'s everything, `renew`'s commutativity) instead of over-fitting.
///
/// A leaf with a smart-constructor invariant impls `Shaped` by hand (the library's `Int` and
/// `Key` do the same); `#[derive(Shaped)]` is for composites — and is in any case unavailable
/// downstream, since its expansion is also spelled `crate::boundary::…`.
impl Shaped for Credits {
    fn inhabitant() -> Self {
        Credits(0)
    }
    fn perturbation_classes(&self) -> Vec<Vec<Self>> {
        let neighbours = [
            Credits::clamped(self.0.saturating_add(1)),
            Credits(CAP),
            Credits(0),
        ];
        vec![neighbours.into_iter().filter(|c| c != self).collect()]
    }
}

// ===== the entry edge: a Construction minted DOWNSTREAM ====================

// Citizen registration, with the library's own macros — public since the citizen/effect
// seal split (`boundary::citizen` is open; only the effect lattice stays sealed).
boundary_algebra::value_object!(Credits);

/// The entry edge: parse a raw `i64` into a valid balance — "parse, don't validate" as a
/// probeable `Construction`, the same shape as the library's own `Parse`. A pure
/// refinement: nothing is normalized away, so the residual is `Unit` and the round trip
/// is exact.
pub struct ParseCredits;
boundary_algebra::value_operator!(ParseCredits);

impl Construction for ParseCredits {
    type Capability = Pure;
    type Raw = i64;
    type Refined = Credits;
    type Residual = Unit;
    fn parse(&self, raw: &i64) -> Option<(Credits, Unit)> {
        Credits::new(*raw).map(|c| (c, Unit))
    }
    fn reconstruct(&self, refined: &Credits, _residual: &Unit) -> Option<i64> {
        Some(refined.get())
    }
}

impl Probed for ParseCredits {
    /// The derived entry-edge laws, oracle-free, over a raw range that spans both ends
    /// of validity: an ADMITTED raw must reconstruct exactly (`reconstructs`), and
    /// admission must agree with the validity rule — so a constructor that silently
    /// normalizes, widens, or narrows the range breaks the probe, not production.
    fn probe() {
        for raw in -3..=25 {
            match reconstructs(&ParseCredits, &raw) {
                Some(ok) => assert!(ok, "admitted raw {raw} must reconstruct exactly"),
                None => assert!(
                    Credits::new(raw).is_none(),
                    "raw {raw} was rejected but is a valid balance"
                ),
            }
            assert_eq!(
                ParseCredits.parse(&raw).is_some(),
                Credits::new(raw).is_some(),
                "admission must agree with the validity rule at {raw}"
            );
        }
    }
}
