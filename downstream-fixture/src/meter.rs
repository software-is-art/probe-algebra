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
//!
//! Beyond the entry edge, this file mints the WITNESS-PASSING pair downstream: `CheckFunds`
//! is a `Branch` that classifies a named order into `Affordable<N>` / `Insufficient<N>`
//! (both arms `proof_token!`-minted, both first-class), and `Deduct` is a `Guarded` edge
//! whose `Proof<N>` is `Affordable<N>` — so a deduction whose funds were never checked is a
//! COMPILE error in the consumer's own tree, not a runtime hope. See `tests/compile_fail/`
//! for the negatives pinned as fixtures.

use core::marker::PhantomData;

use boundary_spec::boundary::{
    reconstructs, Branch, Construction, Guarded, Probed, Pure, Shaped, Unit,
};
use boundary_spec::discover::engine::shadow_grid;
use boundary_spec::gdp::{with_seed, Named};

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
boundary_spec::value_object!(Credits);

/// The entry edge: parse a raw `i64` into a valid balance — "parse, don't validate" as a
/// probeable `Construction`, the same shape as the library's own `Parse`. A pure
/// refinement: nothing is normalized away, so the residual is `Unit` and the round trip
/// is exact.
pub struct ParseCredits;
boundary_spec::value_operator!(ParseCredits);

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

// ===== the witness-passing pair: a Branch and a Guarded edge, minted DOWNSTREAM ======
//
// The crown-jewel pattern, run through public API: `CheckFunds` MINTS a name-branded
// witness that only `Deduct` can consume, and only for the SAME order — so "you forgot
// to check the funds" and "you checked a different order" are both compile errors in
// this consumer's tree (pinned in `tests/compile_fail/`). This mirrors the library's
// own `Check`/`Eval` pair exactly; the domain is just money instead of types.

/// A purchase: the amount a caller wants deducted. The same number as a `Credits`
/// balance, but a different ROLE — re-tagging it as its own value object is what keeps
/// `Order::new(balance, purchase)` un-swappable at the call site (a transposed-arguments
/// bug is a type error, not a wrong classification).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Purchase(Credits);

impl Purchase {
    /// Re-tag a (already valid) amount as a purchase. Total: validity was `Credits`'s job.
    pub fn of(amount: Credits) -> Purchase {
        Purchase(amount)
    }

    /// The amount to deduct — the sanctioned exit hatch back to `Credits`.
    pub fn amount(&self) -> Credits {
        self.0
    }
}

/// An order awaiting classification: a balance and the purchase to be deducted from it,
/// bundled into ONE citizen so the `Branch` input is a value object, not a loose tuple
/// (the same shape as the library's `Bound`). What `CheckFunds` classifies and `Deduct`
/// deducts is the SAME named order — the brand ties the two edges to one value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Order {
    balance: Credits,
    purchase: Purchase,
}

impl Order {
    pub fn new(balance: Credits, purchase: Purchase) -> Order {
        Order { balance, purchase }
    }
    pub fn balance(&self) -> Credits {
        self.balance
    }
    pub fn purchase(&self) -> Purchase {
        self.purchase
    }
}

boundary_spec::value_object!(Purchase, Order);

boundary_spec::proof_token!(
    /// A proof that the order named `N` is AFFORDABLE (its purchase does not exceed its
    /// balance — the deduction will not saturate). Branded with the order's name, so a
    /// proof for order A cannot authorize deducting order B; minted ONLY by
    /// `CheckFunds::classify` (the field is private to this module — see
    /// `tests/compile_fail/deduct_forged_witness.rs`). It is the witness `Deduct` demands.
    Affordable
);
boundary_spec::proof_token!(
    /// A proof that the order named `N` is INSUFFICIENT — the NEGATIVE witness, kept
    /// (not discarded as a `None`) so the refusal path is first-class. It discharges
    /// nothing: handing it to `Deduct` is a type error
    /// (`tests/compile_fail/deduct_insufficient_witness.rs`).
    Insufficient
);

/// The classifier: a `Branch` that compares a named order's purchase against its balance
/// and lands in `Affordable<N> + Insufficient<N>`, keeping BOTH witnesses. Pure (a
/// read-only comparison).
pub struct CheckFunds;
/// The deduction: a `Guarded` edge `Order -> Credits` admitted by an `Affordable<N>` for
/// the SAME name. You cannot deduct an order whose funds have not been checked — the
/// witness comes from `CheckFunds`, exactly as the library's `Eval` demands `Check`'s.
pub struct Deduct;
boundary_spec::value_operator!(CheckFunds, Deduct);

impl Branch for CheckFunds {
    type Capability = Pure;
    type In<N> = Named<N, Order>;
    type Left<N> = Affordable<N>;
    type Right<N> = Insufficient<N>;

    fn branch<N>(&self, order: &Named<N, Order>) -> Result<Affordable<N>, Insufficient<N>> {
        let order = order.value();
        if order.purchase().amount().get() <= order.balance().get() {
            Ok(Affordable(PhantomData))
        } else {
            Err(Insufficient(PhantomData))
        }
    }
}

impl CheckFunds {
    /// Classify a named order — the ergonomic name for the `Branch` edge. Both arms carry
    /// a proof; an unaffordable order is an `Insufficient` witness, never a silent `None`.
    pub fn classify<N>(&self, order: &Named<N, Order>) -> Result<Affordable<N>, Insufficient<N>> {
        self.branch(order)
    }
}

impl Guarded for Deduct {
    type Capability = Pure;
    type In<N> = Named<N, Order>;
    type Proof<N> = Affordable<N>;
    // Deduction KEEPS the order's brand `N`: the resulting balance is provably the
    // remainder of the order that was checked, not some other.
    type Out<N> = Named<N, Credits>;

    fn guard<N>(&self, order: &Named<N, Order>, _proof: &Affordable<N>) -> Named<N, Credits> {
        order.map(|o| o.balance().spend(o.purchase().amount()))
    }
}

impl Deduct {
    /// Deduct a checked, named order — the ergonomic name for the `Guarded` edge.
    /// Requires an `Affordable<N>` for the same name, so an unchecked or insufficient
    /// order will not type-check, and there is no other way to reach the new balance.
    pub fn run<N>(&self, order: &Named<N, Order>, proof: &Affordable<N>) -> Named<N, Credits> {
        self.guard(order, proof)
    }
}

impl Probed for CheckFunds {
    /// The branch's derived law, oracle-free, over every pair drawn from the
    /// `Shaped`-derived grid (which closes over the whole valid range, both saturation
    /// points included): classification must agree with an INDEPENDENT raw comparison —
    /// `Ok` iff `purchase <= balance` — so a classifier that flips the comparison,
    /// drops the boundary case (`purchase == balance` IS affordable), or collapses into
    /// one arm dies here, not in production. Totality (every pair lands in exactly one
    /// arm) is the `Result`'s own guarantee; the probe pins WHICH arm, and the counters
    /// guard that BOTH arms actually fire (a degenerate grid cannot go green vacuously).
    fn probe() {
        let grid = shadow_grid::<Credits>(64);
        let (mut ok, mut err) = (0u32, 0u32);
        for &balance in &grid {
            for &amount in &grid {
                let affordable = with_seed(|seed| {
                    let order = seed.new_named(Order::new(balance, Purchase::of(amount)));
                    CheckFunds.classify(&order).is_ok()
                });
                assert_eq!(
                    affordable,
                    amount.get() <= balance.get(),
                    "classification must agree with the raw comparison at \
                     balance {} / purchase {}",
                    balance.get(),
                    amount.get()
                );
                if affordable {
                    ok += 1
                } else {
                    err += 1
                }
            }
        }
        assert!(ok > 0 && err > 0, "both arms must fire: a vacuous probe");
    }
}

impl Probed for Deduct {
    /// The guarded edge's derived law, oracle-free, over every AFFORDABLE pair in the
    /// grid: with the witness in hand the deduction is EXACT — the result agrees with
    /// the `spend` operator AND with raw subtraction, because affordability is precisely
    /// the condition under which `spend`'s zero-floor never engages. A deduction that
    /// swaps its operands, saturates early, or grants instead of spending dies here.
    /// Every witness comes from `CheckFunds`, never minted by hand — a proof is only as
    /// true as its mint — so the probe also drives the branch→guard chain end to end.
    fn probe() {
        let grid = shadow_grid::<Credits>(64);
        let mut affordable = 0u32;
        for &balance in &grid {
            for &amount in &grid {
                with_seed(|seed| {
                    let order = seed.new_named(Order::new(balance, Purchase::of(amount)));
                    let Ok(proof) = CheckFunds.classify(&order) else {
                        return; // insufficient: nothing to deduct; the count guards vacuity
                    };
                    affordable += 1;
                    let after = Deduct.run(&order, &proof);
                    assert_eq!(
                        *after.value(),
                        balance.spend(amount),
                        "deduction must agree with the spend operator at \
                         balance {} / purchase {}",
                        balance.get(),
                        amount.get()
                    );
                    assert_eq!(
                        after.value().get(),
                        balance.get() - amount.get(),
                        "an affordable deduction is exact — the zero-floor must not engage"
                    );
                });
            }
        }
        assert!(
            affordable > 0,
            "no affordable pair in the grid: a vacuous probe"
        );
    }
}
