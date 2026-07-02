//! Tier: ALGEBRA — the discovered-law layer: the meter's operators, as a synthesised theory.
//!
//! ops — the MINIMAL authoring path, exercised downstream: `#[algebra]` over three ordinary
//! operator functions synthesises the entire discovery `Theory` (`CreditMeter`) — operator
//! table, sort, `sort_of`, identity observation, and a grid shadow-derived from `Credits`'s
//! `Shaped` structure. Nothing about the algebra is declared here; the laws in
//! `spec/credit-meter.spec` were all DISCOVERED by running these functions.
//!
//! Two consumer facts this file pins down:
//!
//! * `#[algebra]` works downstream ONLY with the `pub use boundary_algebra::discover;` shim at
//!   the consumer's crate root — its expansion hardcodes `crate::discover::…` paths (see
//!   `lib.rs`, "The re-export shim").
//! * A NULLARY constant operator (`pub fn zero() -> Credits`) is unauthorable on this path:
//!   `#[algebra]` requires operators to be PUBLIC functions, but the rats-nest rule refuses a
//!   public zero-argument function (an arity-0 constant is not operator-shaped to the census).
//!   So this theory carries no `zero` constant and its identity laws go undiscovered — a real
//!   gap between the macro and the enforcement, reported as a finding rather than worked
//!   around. (`theory!` domains sidestep it: their eval functions are private.)

use boundary_algebra::algebra;

/// The operator functions ARE the module; the theory (`CreditMeter`, named "credit meter") is
/// what `#[algebra]` reads off their signatures. `(Credits, Credits) -> Credits` ⇒ three binary
/// operators over one sort.
#[algebra(CreditMeter, "credit meter")]
pub mod meter_ops {
    use crate::meter::Credits;

    /// Saturating top-up — discovered: commutative and associative.
    pub fn grant(a: Credits, b: Credits) -> Credits {
        a.grant(b)
    }

    /// Saturating deduction — discovered: NOTHING. Neither commutative nor associative, so the
    /// engine refuses every universal shape and the spec's coverage line names `spend` as the
    /// operator no law speaks for. The silence is meaningful: order of deductions is semantics.
    pub fn spend(a: Credits, b: Credits) -> Credits {
        a.spend(b)
    }

    /// Voucher renewal — discovered: associative, idempotent, and BIASED: the regular-band
    /// sandwich law states that the later operand wins where the two disagree (an
    /// earlier-wins mutant breaks the committed spec).
    pub fn renew(a: Credits, b: Credits) -> Credits {
        a.renew(b)
    }
}
