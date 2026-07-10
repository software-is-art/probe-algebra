//! tally — a BUNDLE-BORN module: grown entirely by the continuation verbs (`bundle add`,
//! `bundle declare`, `bundle lift`), never edited in an open file. Every command that
//! built it is recorded in ../../MANIFEST.md; the committed lift beside it is the scan's
//! output, drift-gated in tests/contract.rs.

use boundary_spec::Shaped;

/// A saturating three-level tally — the demo carrier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Shaped)]
pub enum Tally {
    T0,
    T1,
    T2,
}

/// The join: the larger of two tallies — a semilattice merge.
pub fn merge(a: Tally, b: Tally) -> Tally {
    a.max(b)
}

/// The empty tally — merge's identity.
pub fn floor() -> Tally {
    Tally::T0
}

/// One saturating step upward — the ceiling holds.
pub fn bump(t: Tally) -> Tally {
    match t {
        Tally::T0 => Tally::T1,
        Tally::T1 => Tally::T2,
        Tally::T2 => Tally::T2,
    }
}

include!("tally_lift.rs");
