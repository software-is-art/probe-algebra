//! GUARD violation: a `WellTyped` proof for one program cannot authorize evaluating
//! ANOTHER. `Eval::run` demands a `WellTyped<N>` for the SAME name `N` as the expression,
//! so a proof minted for program `a` (named `Na`) will not discharge evaluation of `b`
//! (named `Nb`). This is "well-typed programs don't go wrong" made un-bypassable: you
//! cannot evaluate an expression except with ITS OWN type-correctness witness.
#![allow(unused_variables, unused_imports, dead_code)]

use probe_algebra::gdp::with_seed;
use probe_algebra::interp::boundary::{Check, Eval, Expr};

fn main() {
    with_seed(|sa| {
        let a = sa.new_named(Expr::int(1).unwrap());
        with_seed(|sb| {
            let b = sb.new_named(Expr::int(2).unwrap());
            // a's well-typedness proof, branded `Na`.
            let proof_a = Check.classify(&a).ok().unwrap();
            // ERROR: `b` is named `Nb`, but the proof is `WellTyped<Na>`.
            let _ = Eval.run(&b, &proof_a);
        })
    })
}
