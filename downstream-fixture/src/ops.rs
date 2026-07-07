//! The discovered-law layer (ALGEBRA by derivation): the meter's operators, as a
//! synthesised theory.
//!
//! ops — the MINIMAL authoring path, exercised downstream: `#[algebra]` over three ordinary
//! operator functions synthesises the entire discovery `Theory` (`CreditMeter`) — operator
//! table, sort, `sort_of`, identity observation, and a grid shadow-derived from `Credits`'s
//! `Shaped` structure. Nothing about the algebra is declared here; the laws in
//! `spec/credit-meter.spec` were all DISCOVERED by running these functions.
//!
//! Two consumer facts this file pinned down as FINDINGS, both since fixed:
//!
//! * `#[algebra]` (and `#[derive(Shaped)]`) once expanded to `crate::…` paths and needed a
//!   re-export shim at the consumer's root; the macros now emit `::boundary_spec::…`
//!   (the library aliases itself), so this module works downstream with no shim.
//! * A NULLARY constant operator was unauthorable on this path — the rats-nest rule
//!   refused any public zero-argument function. The rule now recognises a nullary fn
//!   returning a value type INSIDE an `#[algebra]` module as a CONSTANT operator, so
//!   `zero` below is authorable and the identity laws it unlocks are discovered.

use boundary_spec::algebra;

/// The operator functions ARE the module; the theory (`CreditMeter`, named "credit meter") is
/// what `#[algebra]` reads off their signatures. `(Credits, Credits) -> Credits` ⇒ three binary
/// operators over one sort.
///
/// The `expects(...)` argument is the TOP-DOWN half: the meter's DECLARED algebra, stated
/// before (and now verified against) what discovery finds — all nine laws of
/// `spec/credit-meter.spec`, in the declaration vocabulary. `tests/expectations.rs` holds
/// `Distance::of::<CreditMeter>()` green with no surprises; declaring is free — it changes
/// nothing about discovery or the frozen lock, it only lets the distance be measured.
#[algebra(
    CreditMeter,
    "credit meter",
    expects(
        commutative(grant),
        associative(grant),
        identity(grant, zero),
        identity(spend, zero),
        annihilation(spend, zero),
        self_inverse(spend, zero),
        associative(renew),
        idempotent(renew),
        bias_later(renew),
        identity(renew, zero),
    )
)]
pub mod meter_ops {
    use crate::meter::Credits;

    /// The empty balance — the constant the identity laws need.
    pub fn zero() -> Credits {
        Credits::new(0).expect("zero is a valid balance")
    }

    /// Saturating top-up — discovered: commutative and associative, with `zero` as identity.
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
