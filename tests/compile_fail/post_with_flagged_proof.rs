//! GUARD violation: a `Flagged` proof cannot discharge `Post`. The two guarded
//! transitions take DIFFERENT proofs — `Post` wants `Cleared<N>`, `Reject` wants
//! `Flagged<N>` — so the negative witness of an unbalanced entry cannot be used to
//! post it. The guard is typed, not just present.
#![allow(unused_variables, unused_imports, dead_code)]

use boundary_algebra::boundary::Morphism;
use boundary_algebra::gdp::with_seed;
use boundary_algebra::ledger::boundary::{Account, Cents, Posting, Transaction};
use boundary_algebra::lifecycle::boundary::{Draft, Entry, Post, Submit, Validate};

fn unbalanced() -> Transaction {
    Transaction::new(vec![
        Posting::new(Account::new("Cash").unwrap(), Cents::new(10_000).unwrap()),
        Posting::new(Account::new("Fees").unwrap(), Cents::new(5_000).unwrap()),
    ])
    .unwrap()
}

fn main() {
    with_seed(|seed| {
        let (submitted, _unit) = Submit.forward(&Entry::<Draft>::draft(unbalanced()));
        let named = seed.new_named(submitted);
        let flagged = Validate.classify(&named).unwrap_err();
        // `flagged` is a `Flagged<N>`; `Post::commit` requires `&Cleared<N>`.
        let _posted = Post.commit(&named, &flagged);
    });
}
