//! CAPABILITY violation: `run_pure` accepts only effect-free edges (`AtMost<Pure>`), but
//! `ConstFold` is declared `Lossy` — it collapses constant subexpressions, a real loss. So
//! running it through the pure runner is a COMPILE error: the capability ceiling is an
//! enforced contract, not a comment. The honest residual-keeping form is `run` / `Carried`,
//! or `run_within::<Lossy, _>`.
#![allow(unused_variables, unused_imports, dead_code)]

use boundary_spec::boundary::run_pure;
use boundary_spec::interp::boundary::{ConstFold, Expr};

fn main() {
    let e = Expr::int(2).unwrap();
    // ConstFold is Lossy, so it does not satisfy `AtMost<Pure>`.
    let _ = run_pure(&ConstFold, &e);
}
