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
//! CONSUMER NOTE (a finding this fixture exists to surface): the library's `refined!` /
//! `value_object!` macros and its edge traits (`Morphism`, `Construction`, `Branch`,
//! `Guarded`) are SEALED — `boundary::sealed` is `pub(crate)`, so a downstream crate cannot
//! mint boundary citizens or edges at all. What a consumer CAN do, and what this file
//! demonstrates, is keep the same shape by hand: a newtype over the primitive, a
//! parse-don't-validate smart constructor whose predicate is the only content, operators as
//! methods, and the structural rules held by `boundary-enforce` (which is purely syntactic and
//! needs no marker traits). The discovery side (`Shaped`, `Theory`, `Spec`) is unsealed and
//! fully available.

use boundary_algebra::boundary::Shaped;

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

    /// Deduct `other`, saturating at zero. Deliberately lawless: neither commutative nor
    /// associative, so the engine refuses every shape for it and the spec's coverage line
    /// names it — the silence is the signal.
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
