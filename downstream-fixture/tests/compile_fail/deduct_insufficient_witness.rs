//! GUARD violation: the NEGATIVE witness discharges nothing. `CheckFunds` keeps both arms
//! first-class — an unaffordable order yields an `Insufficient<N>`, not a silent `None` —
//! but `Deduct`'s `Proof<N>` is `Affordable<N>`, so handing the refusal to the deduction
//! is a type error: "the check ran" is not enough, it must have run in your favour.
#![allow(unused_variables, unused_imports, dead_code)]

use boundary_spec::gdp::with_seed;
use downstream_fixture::meter::{CheckFunds, Credits, Deduct, Order, Purchase};

fn main() {
    with_seed(|seed| {
        let balance = Credits::new(1).unwrap();
        let purchase = Purchase::of(Credits::new(20).unwrap());
        let order = seed.new_named(Order::new(balance, purchase));
        // 20 from a balance of 1: the check refuses, minting `Insufficient<N>`.
        let refusal = CheckFunds.classify(&order).err().unwrap();
        // ERROR: `Deduct` demands `Affordable<N>`, not `Insufficient<N>`.
        let _ = Deduct.run(&order, &refusal);
    })
}
