//! ORDER violation: you cannot validate an entry that has not been submitted.
//! `Validate::clear` wants `Named<N, Entry<Submitted>>`; a draft has the wrong state.
#![allow(unused_variables, unused_imports, dead_code)]

use boundary_algebra::gdp::with_seed;
use boundary_algebra::ledger::boundary::{Account, Cents, Posting, Transaction};
use boundary_algebra::lifecycle::boundary::{Draft, Entry, Validate};

fn tx() -> Transaction {
    Transaction::new(vec![
        Posting::new(Account::new("Cash").unwrap(), Cents::new(1).unwrap()),
        Posting::new(Account::new("Revenue").unwrap(), Cents::new(-1).unwrap()),
    ])
    .unwrap()
}

fn main() {
    with_seed(|seed| {
        let draft = Entry::<Draft>::draft(tx());
        let named = seed.new_named(draft);
        // `named` wraps an `Entry<Draft>`; `classify` requires `Entry<Submitted>`.
        let _verdict = Validate.classify(&named);
    });
}
