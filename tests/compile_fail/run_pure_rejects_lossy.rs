//! CAPABILITY ceiling violation: `run_pure` demands `M::Capability: AtMost<Pure>`,
//! but `Aggregate` is a `Lossy` edge, so the bound is unsatisfied. The wrong EFFECT
//! shape is a compile error at the call site — the LSP push-back a coding agent gets
//! before any test runs, the capability twin of the typestate/provenance contracts.
#![allow(unused_variables, unused_imports, dead_code)]

use probe_algebra::boundary::run_pure;
use probe_algebra::ledger::boundary::{Account, Aggregate, Cents, Posting, Transaction};

fn tx() -> Transaction {
    Transaction::new(vec![
        Posting::new(Account::new("Cash").unwrap(), Cents::new(1).unwrap()),
        Posting::new(Account::new("Revenue").unwrap(), Cents::new(-1).unwrap()),
    ])
    .unwrap()
}

fn main() {
    // `Aggregate::Capability` is `Lossy`, which does NOT implement `AtMost<Pure>`.
    let _ = run_pure(&Aggregate, &tx());
}
