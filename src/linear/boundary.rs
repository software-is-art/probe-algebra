//! linear::boundary — the linear-transport module's PUBLIC interface.
//!
//! By the boundary discipline this file contains ONLY:
//!   - VALUE OBJECTS   : Quantity, Rate
//!   - VALUE OPERATORS : Scale (a Morphism — a lossless transport);
//!     Double (a Metamorphic relation — structural);
//!     UnitResponse (a Coefficient relation — quantitative)
//!   - (typestates and the empty residual `Unit` live in `crate::boundary`)
//!
//! The point of this module is the BLIND-SPOT MAP: `Scale::honest` and
//! `Scale::skew` are the SAME type with different rates. They are mutually
//! indistinguishable to every structural check (both round-trip, both commute
//! with scaling) and are separated only by the quantitative `UnitResponse` probe,
//! which carries the reference coefficient.

use crate::boundary::{Capability, Coefficient, Metamorphic, Morphism, Unit};

// ===== value objects =====================================================

/// A scalar quantity. A pure scalar has no internal dimensionality — nothing to
/// forget — so its constructor only enforces a sane range; its operators are the
/// sanctioned place where raw integer arithmetic happens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Quantity(i64);
impl Quantity {
    pub fn new(n: i64) -> Option<Self> {
        if n.abs() <= 1_000_000 {
            Some(Quantity(n))
        } else {
            None
        }
    }
    pub fn get(&self) -> i64 {
        self.0
    }
    /// The additive identity.
    pub fn zero() -> Self {
        Quantity(0)
    }
    /// The unit step — the input increment the quantitative probe perturbs by.
    pub fn unit() -> Self {
        Quantity(1)
    }
    /// Addition (total within the operating range used by the probes).
    pub fn plus(self, other: Quantity) -> Self {
        Quantity(self.0 + other.0)
    }
    /// The signed difference — used to read an output DELTA for the coefficient.
    pub fn minus(self, other: Quantity) -> Self {
        Quantity(self.0 - other.0)
    }
    /// Multiply by a rate. Raw multiplication is confined to this operator body
    /// (the sanctioned exit hatch); call sites never see a bare `*`.
    pub fn scaled(self, r: Rate) -> Quantity {
        Quantity(self.0 * r.get())
    }
    /// Exact inverse of `scaled` for values produced by it: divide by the rate,
    /// `None` if the value is not an exact multiple (so the map is a bijection
    /// onto its image — invertibility is real, not assumed).
    pub fn unscaled(self, r: Rate) -> Option<Quantity> {
        if self.0 % r.get() == 0 {
            Quantity::new(self.0 / r.get())
        } else {
            None
        }
    }
}

/// A positive scaling rate. Its own value object so a rate cannot be confused
/// with a quantity and so the valid set (1..=1000) is enforced at construction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rate(i64);
impl Rate {
    pub fn new(n: i64) -> Option<Self> {
        if (1..=1000).contains(&n) {
            Some(Rate(n))
        } else {
            None
        }
    }
    pub fn get(&self) -> i64 {
        self.0
    }
}

crate::value_object!(Quantity, Rate);

// ===== value operator: the transport (a Morphism) ========================

/// A lossless linear transport `Quantity -> (Quantity, Unit)`: multiply by a
/// rate. `honest` and `skew` are the SAME type with different rates — `skew` is
/// the wrong-but-invertible coefficient bug. Because each inverts by ITS OWN
/// rate, BOTH round-trip; the bug is a wrong constant, not a broken structure.
pub struct Scale {
    rate: Rate,
}
impl Scale {
    /// The correct transport (the reference coefficient is this rate).
    pub fn honest() -> Self {
        Scale {
            rate: Rate::new(3).expect("3 is a valid rate"),
        }
    }
    /// A wrong-but-INVERTIBLE transport: right shape (linear, bijective), wrong
    /// constant. Survives every structural check; only the quantitative probe
    /// separates it from `honest`.
    pub fn skew() -> Self {
        Scale {
            rate: Rate::new(5).expect("5 is a valid rate"),
        }
    }
    /// The reference coefficient an honest transport must exhibit.
    pub fn reference_rate() -> Rate {
        Rate::new(3).expect("3 is a valid rate")
    }
}
crate::value_operator!(Scale);

impl Morphism for Scale {
    const CAPABILITY: Capability = Capability::Pure;

    type In = Quantity;
    type Out = Quantity;
    type Residual = Unit;

    fn forward(&self, input: &Quantity) -> (Quantity, Unit) {
        (input.scaled(self.rate), Unit)
    }

    fn backward(&self, out: &Quantity, _r: &Unit) -> Option<Quantity> {
        // Divides by ITS OWN rate, so honest and skew BOTH round-trip — this is
        // precisely why round-trip cannot tell a wrong coefficient apart.
        out.unscaled(self.rate)
    }
}

// ===== value operator: a structural relation (Metamorphic) ===============

/// Doubling the input must double the output — the scaling metamorphic relation.
/// Holds for ANY rate (linearity), so it is BLIND to a wrong coefficient: it
/// witnesses the *shape* (linear) without pinning the *constant*.
pub struct Double;
crate::value_operator!(Double);

impl Metamorphic<Scale> for Double {
    fn input_op(&self, x: &Quantity) -> Option<Quantity> {
        Some(x.plus(*x))
    }
    fn output_op(&self, y: &Quantity) -> Quantity {
        y.plus(*y)
    }
}

// ===== value operator: a quantitative relation (Coefficient) =============

/// The unit-response probe: a unit step on the input must produce exactly the
/// reference rate as an output delta. REFERENCE-BEARING — it carries the honest
/// coefficient, so it (and only it) separates `honest` from `skew`.
pub struct UnitResponse {
    expected: Quantity,
}
impl UnitResponse {
    /// Build the probe from the reference rate: a unit step's correct delta is
    /// `unit * rate`.
    pub fn from_reference(r: Rate) -> Self {
        UnitResponse {
            expected: Quantity::unit().scaled(r),
        }
    }
}
crate::value_operator!(UnitResponse);

impl Coefficient<Scale> for UnitResponse {
    type Delta = Quantity;

    fn unit_step(&self, x: &Quantity) -> Option<Quantity> {
        Some(x.plus(Quantity::unit()))
    }
    fn expected_delta(&self) -> Quantity {
        self.expected
    }
    fn observed_delta(&self, before: &Quantity, after: &Quantity) -> Quantity {
        after.minus(*before)
    }
}
