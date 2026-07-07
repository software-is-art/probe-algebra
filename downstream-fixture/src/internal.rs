//! The workshop the boundary delegates to — INTERIOR by derivation: not pub-reachable
//! (`mod internal`, never `pub mod`), so the tier-2 inward rule holds here.
//!
//! internal — the credit meter's arithmetic, in ordinary imperative Rust.
//!
//! The interior is FREE: raw `i64` arithmetic, any style. What the tier enforces is the INWARD
//! rule — no function here may RETURN a raw primitive, so every quantity leaves this module as
//! a validated `Credits` (via the boundary's `clamped` constructor), never as a bare number a
//! caller would have to re-validate. The module is private (`mod internal` in `lib.rs`): its
//! only callers are the boundary's operator methods, and its only spec is the discovered
//! algebra those methods freeze — no interior tests, exactly the library's own
//! `kvstore::internal` shape.

use crate::meter::Credits;

/// Saturating top-up: `a + b`, clamped at the ceiling.
pub(crate) fn grant(a: Credits, b: Credits) -> Credits {
    Credits::clamped(a.get().saturating_add(b.get()))
}

/// Saturating deduction: `a - b`, clamped at zero.
pub(crate) fn spend(a: Credits, b: Credits) -> Credits {
    Credits::clamped(a.get().saturating_sub(b.get()))
}

/// Voucher renewal: a non-zero voucher replaces the balance; a zero voucher changes nothing.
pub(crate) fn renew(a: Credits, voucher: Credits) -> Credits {
    if voucher.get() == 0 {
        a
    } else {
        voucher
    }
}
