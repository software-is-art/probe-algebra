//! ORDER violation: you cannot submit an entry that is already submitted.
//! `Submit::In` is `Entry<Draft>`, so a second submission has the wrong input type.
#![allow(unused_variables, unused_imports, dead_code)]

use boundary_algebra::boundary::Morphism;
use boundary_algebra::ledger::boundary::{Account, Cents, Posting, Transaction};
use boundary_algebra::lifecycle::boundary::{Draft, Entry, Submit};

fn tx() -> Transaction {
    Transaction::new(vec![
        Posting::new(Account::new("Cash").unwrap(), Cents::new(1).unwrap()),
        Posting::new(Account::new("Revenue").unwrap(), Cents::new(-1).unwrap()),
    ])
    .unwrap()
}

fn main() {
    let draft = Entry::<Draft>::draft(tx());
    let (submitted, _unit) = Submit.forward(&draft);
    // `submitted` is `Entry<Submitted>`; `Submit::forward` wants `&Entry<Draft>`.
    let _again = Submit.forward(&submitted);
}
