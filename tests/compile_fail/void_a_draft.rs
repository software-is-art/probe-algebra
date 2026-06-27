//! ORDER violation: you cannot void an entry that was never posted.
//! `Void::forward` wants `&Entry<Posted>`; a draft is in the wrong state.
#![allow(unused_variables, unused_imports, dead_code)]

use probe_algebra::boundary::Morphism;
use probe_algebra::ledger::boundary::{Account, Cents, Posting, Transaction};
use probe_algebra::lifecycle::boundary::{Draft, Entry, Void};

fn tx() -> Transaction {
    Transaction::new(vec![
        Posting::new(Account::new("Cash").unwrap(), Cents::new(1).unwrap()),
        Posting::new(Account::new("Revenue").unwrap(), Cents::new(-1).unwrap()),
    ])
    .unwrap()
}

fn main() {
    let draft = Entry::<Draft>::draft(tx());
    // `Void::In` is `Entry<Posted>`; a `Draft` cannot be voided.
    let _voided = Void.forward(&draft);
}
