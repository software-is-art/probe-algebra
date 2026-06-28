//! COVERAGE violation: a probe that omits a real degree of freedom must not be
//! accepted as complete. `Expr` declares `Shape` AND `Literals`, but `ShapeOnlyProbe`
//! only `Covers<Shape>` — so `require_complete` cannot prove `CoversAll` and the
//! incomplete probe is a compile error, not a runtime gap. This is the static twin of a
//! runtime coverage check: the LSP push-back a coding agent gets for an incomplete probe
//! before any test runs.
#![allow(unused_variables, unused_imports, dead_code)]

use boundary_algebra::boundary::require_complete;
use boundary_algebra::interp::boundary::{Expr, ShapeOnlyProbe};

fn main() {
    // `ShapeOnlyProbe` never varies a literal, so it does not `CoversAll` the DOFs
    // `Expr` declares.
    require_complete::<Expr, _>(&ShapeOnlyProbe);
}
