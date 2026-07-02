//! COVERAGE violation: a HAND-WRITTEN probe that omits a derived degree of freedom must not
//! be accepted as complete. `Expr`'s DOF set is derived (one `Field<Expr, I>` per variant);
//! the generated `Complete<Expr>` covers them all by construction, but a hand probe that
//! `Covers` only `Field<Expr, 0>` cannot satisfy `CoversAll`, so `require_complete` rejects
//! it at compile time. The normal path is complete-by-construction; this is the backstop for
//! the one thing still hand-writable.
#![allow(unused_variables, unused_imports, dead_code)]

use boundary_spec::boundary::{require_complete, Covers, Field};
use boundary_spec::interp::boundary::Expr;

// A probe that reaches only the first derived dimension.
struct PartialProbe;
impl Covers<Field<Expr, 0>> for PartialProbe {}

fn main() {
    // `Expr` has more than one DOF, so a single-`Field` probe is not `CoversAll`.
    require_complete::<Expr, _>(&PartialProbe);
}
