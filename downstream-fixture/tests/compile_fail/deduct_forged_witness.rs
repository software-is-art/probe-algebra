//! GUARD violation: the witness cannot be FORGED. `Affordable<N>`'s one field is private
//! to the minting module (`proof_token!` keeps it so), so the only way to obtain one is
//! `CheckFunds::classify` — a consumer cannot construct the proof by hand and skip the
//! funds check. `Deduct` without a real check is therefore unwritable, not just untested.
#![allow(unused_variables, unused_imports, dead_code)]

use core::marker::PhantomData;

use boundary_spec::gdp::with_seed;
use downstream_fixture::meter::{Affordable, Credits, Deduct, Order, Purchase};

fn main() {
    with_seed(|seed| {
        let balance = Credits::new(1).unwrap();
        let purchase = Purchase::of(Credits::new(20).unwrap());
        let order = seed.new_named(Order::new(balance, purchase));
        // ERROR: the token's field is private — only `CheckFunds` mints it.
        let forged = Affordable(PhantomData);
        let _ = Deduct.run(&order, &forged);
    })
}
