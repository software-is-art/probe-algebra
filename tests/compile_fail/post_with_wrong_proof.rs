//! RELATIONAL violation: you cannot validate one entry and post another with its
//! proof. A `Cleared<N>` is branded with the entry's unique name `N`; a proof
//! minted for entry A will not unify with the name of entry B. This is the GDP
//! seam — the precondition is tied to a specific value, not to the type in general.
#![allow(unused_variables, unused_imports, dead_code)]

use probe_algebra::boundary::Morphism;
use probe_algebra::gdp::with_seed;
use probe_algebra::ledger::boundary::{Account, Cents, Posting, Transaction};
use probe_algebra::lifecycle::boundary::{Draft, Entry, Post, Submit, Validate};

fn tx() -> Transaction {
    Transaction::new(vec![
        Posting::new(Account::new("Cash").unwrap(), Cents::new(1).unwrap()),
        Posting::new(Account::new("Revenue").unwrap(), Cents::new(-1).unwrap()),
    ])
    .unwrap()
}

fn main() {
    with_seed(|seed| {
        let (seed_a, seed_b) = seed.replicate();

        let (submitted_a, _ua) = Submit.forward(&Entry::<Draft>::draft(tx()));
        let named_a = seed_a.new_named(submitted_a);

        let (submitted_b, _ub) = Submit.forward(&Entry::<Draft>::draft(tx()));
        let named_b = seed_b.new_named(submitted_b);

        let proof_a = Validate.classify(&named_a).unwrap();
        // `proof_a` is branded with A's name; committing B with it cannot unify.
        let _posted = Post.commit(&named_b, &proof_a);
    });
}
