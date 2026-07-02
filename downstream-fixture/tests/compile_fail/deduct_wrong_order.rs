//! GUARD violation: an `Affordable` proof for one order cannot authorize deducting
//! ANOTHER. `Deduct::run` demands an `Affordable<N>` for the SAME name `N` as the order,
//! so a proof minted for order `a` (named `Na`) will not discharge deduction of `b`
//! (named `Nb`) — even though `b`, on its own, would also have passed the check. This is
//! the two-seeds case from the library's `eval_wrong_program`, reproduced downstream: the
//! precondition is tied to the VALUE it was proven of, not merely to having been checked.
#![allow(unused_variables, unused_imports, dead_code)]

use boundary_spec::gdp::with_seed;
use downstream_fixture::meter::{CheckFunds, Credits, Deduct, Order, Purchase};

fn main() {
    with_seed(|sa| {
        let a = sa.new_named(Order::new(
            Credits::new(10).unwrap(),
            Purchase::of(Credits::new(3).unwrap()),
        ));
        with_seed(|sb| {
            let b = sb.new_named(Order::new(
                Credits::new(5).unwrap(),
                Purchase::of(Credits::new(5).unwrap()),
            ));
            // a's affordability proof, branded `Na`.
            let proof_a = CheckFunds.classify(&a).ok().unwrap();
            // ERROR: `b` is named `Nb`, but the proof is `Affordable<Na>`.
            let _ = Deduct.run(&b, &proof_a);
        })
    })
}
