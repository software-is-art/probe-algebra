//! PRECONDITION violation: you cannot forge a clearance to skip validation.
//! `Cleared`'s field is private to the boundary, so the only `Cleared<N>` in
//! existence are the ones `Validate::clear` mints after a real balance check.
#![allow(unused_variables, unused_imports, dead_code)]

use core::marker::PhantomData;

use probe_algebra::lifecycle::boundary::Cleared;

struct Name;

fn main() {
    // The tuple-struct constructor is private (its field is) — a clearance proof
    // cannot be conjured, only earned.
    let _forged: Cleared<Name> = Cleared(PhantomData);
}
