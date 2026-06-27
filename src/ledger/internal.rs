//! internal — PRIVATE implementation detail of the ledger module.
//!
//! Other modules cannot name anything here; this is the module's "workshop"
//! where mutation and raw collections are allowed. The INWARD rule still holds,
//! though: no function returns a raw domain string (parse-don't-validate), and
//! the information-losing aggregation is modelled as a `Morphism` so the SAME
//! generic `probe` reaches it (see the tests) even though it never crosses a
//! boundary.

use std::collections::BTreeMap;

use crate::boundary::Morphism;
use crate::ledger::boundary::{
    Account, AccountSummary, Balance, Cents, MultiplicityResidual, Posting, Transaction,
};

/// Fold postings into per-account balances and the per-account sorted breakdown
/// of the amounts that summed to each total. Returns only VALUE OBJECTS
/// (`Account` / `Balance` / `Cents`) — no raw primitive escapes, and the
/// summation itself goes through `Balance`'s operators.
fn fold(postings: &[Posting]) -> (BTreeMap<Account, Balance>, BTreeMap<Account, Vec<Cents>>) {
    let mut totals: BTreeMap<Account, Balance> = BTreeMap::new();
    let mut breakdown: BTreeMap<Account, Vec<Cents>> = BTreeMap::new();
    for p in postings {
        let running = totals
            .entry(p.account().clone())
            .or_insert_with(Balance::zero);
        *running = running.add_cents(*p.amount());
        breakdown
            .entry(p.account().clone())
            .or_default()
            .push(*p.amount());
    }
    for amounts in breakdown.values_mut() {
        amounts.sort();
    }
    (totals, breakdown)
}

/// The aggregation core, as an INTERNAL morphism. It is never exported from the
/// ledger, yet because it is a `Morphism` over value objects the generic `probe`
/// applies to it directly. And because the residual keeps value objects (not raw
/// primitives), `backward` is total — there is nothing left to re-validate.
pub(super) struct Aggregation;
crate::value_operator!(Aggregation);

impl Morphism for Aggregation {
    type In = Transaction;
    type Out = AccountSummary;
    type Residual = MultiplicityResidual;

    fn forward(&self, input: &Transaction) -> (AccountSummary, MultiplicityResidual) {
        let (totals, breakdown) = fold(input.postings());
        (
            AccountSummary::from_totals(totals),
            MultiplicityResidual::from_breakdown(breakdown),
        )
    }

    fn backward(&self, _out: &AccountSummary, r: &MultiplicityResidual) -> Option<Transaction> {
        let mut postings = Vec::new();
        for (account, amounts) in r.breakdown() {
            for &amount in amounts {
                // value objects in, value objects out — no re-parsing, no failure
                postings.push(Posting::new(account.clone(), amount));
            }
        }
        Transaction::new(postings)
    }
}

#[cfg(test)]
mod tests {
    use super::Aggregation;
    use crate::boundary::{probe, Morphism};
    use crate::ledger::boundary::{Account, Cents, Posting, Split, Transaction};

    fn sample() -> Transaction {
        Transaction::new(vec![
            Posting::new(Account::new("Cash").unwrap(), Cents::new(6000).unwrap()),
            Posting::new(Account::new("Cash").unwrap(), Cents::new(4000).unwrap()),
            Posting::new(
                Account::new("Revenue").unwrap(),
                Cents::new(-10000).unwrap(),
            ),
        ])
        .unwrap()
    }

    /// The SAME generic probe that tests boundary operators also tests this
    /// private, never-exported internal morphism. The algebra reaches inward.
    #[test]
    fn probe_reaches_the_internal_morphism() {
        let x = sample();
        let pr = probe(&Aggregation, &Split, &x).unwrap();
        assert!(pr.residual_complete());
    }

    /// Keeping value objects in the residual makes reconstruction infallible.
    #[test]
    fn internal_backward_is_total() {
        let x = sample();
        let (summary, residual) = Aggregation.forward(&x);
        let recovered = Aggregation.backward(&summary, &residual);
        assert_eq!(recovered.as_ref(), Some(&x));
    }
}
