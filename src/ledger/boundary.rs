//! ledger::boundary — the ledger module's PUBLIC interface.
//!
//! By the boundary discipline this file contains ONLY:
//!   - VALUE OBJECTS   : Cents, Account, Posting, Transaction, AccountSummary, MultiplicityResidual, RoundingResidual
//!   - VALUE OPERATORS : Aggregate, AggregateDropsAmounts, Round (Morphisms); Split, NudgeCents (Perturbations)
//!   - (typestates live in the universal `crate::boundary`)
//!
//! Each value object carries the sealed `ValueObject` marker; each operator the
//! sealed `ValueOperator` marker. The aggregation ALGORITHM is delegated to the
//! private `super::internal` module — it never appears at the boundary.

use std::collections::BTreeMap;

use crate::boundary::{Morphism, Perturbation};

// ===== value objects =====================================================

/// Monetary amount in integer cents, validated to a sane range.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Cents(i64);
impl Cents {
    pub fn new(c: i64) -> Option<Self> {
        if c.abs() <= 100_000_000 {
            Some(Cents(c))
        } else {
            None
        }
    }
    pub fn get(&self) -> i64 {
        self.0
    }
}

/// A non-empty, trimmed account name.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Account(String);
impl Account {
    pub fn new(s: &str) -> Option<Self> {
        let t = s.trim();
        if t.is_empty() {
            None
        } else {
            Some(Account(t.to_string()))
        }
    }
    pub fn get(&self) -> &str {
        &self.0
    }
}

/// A single posting: an amount against an account.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Posting {
    account: Account,
    amount: Cents,
}
impl Posting {
    pub fn new(a: Account, m: Cents) -> Self {
        Posting {
            account: a,
            amount: m,
        }
    }
    pub fn account(&self) -> &Account {
        &self.account
    }
    pub fn amount(&self) -> &Cents {
        &self.amount
    }
}

/// INPUT value object: a transaction = multiset of postings (multiplicity is a
/// real dimension — an account can appear multiple times). Stored canonically
/// sorted so value-equality is stable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    postings: Vec<Posting>,
}
impl Transaction {
    pub fn new(mut p: Vec<Posting>) -> Option<Self> {
        if p.is_empty() {
            return None;
        }
        p.sort();
        Some(Transaction { postings: p })
    }
    pub fn postings(&self) -> &[Posting] {
        &self.postings
    }
    /// Lower the value object to the primitive pairs the internal algorithm eats.
    fn to_pairs(&self) -> Vec<(String, i64)> {
        self.postings
            .iter()
            .map(|p| (p.account.get().to_string(), p.amount.get()))
            .collect()
    }
}

/// OUTPUT value object: per-account totals. Multiplicity is GONE (each account
/// has one total). Constructed only inside this boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountSummary {
    totals: BTreeMap<String, i64>,
}
impl AccountSummary {
    pub fn totals(&self) -> &BTreeMap<String, i64> {
        &self.totals
    }
}

/// RESIDUAL value object for aggregation: the MULTIPLICITY that aggregation
/// collapsed — the original per-account breakdown of each total into its
/// constituent posting amounts (sorted). Summary + this residual reconstructs
/// the transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultiplicityResidual {
    breakdown: BTreeMap<String, Vec<i64>>,
}

/// RESIDUAL value object for rounding: the sub-dollar cents removed from each
/// account total. Rounded total + this residual reconstructs the exact total.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoundingResidual {
    leftover: BTreeMap<String, i64>,
}

crate::value_object!(
    Cents,
    Account,
    Posting,
    Transaction,
    AccountSummary,
    MultiplicityResidual,
    RoundingResidual,
);

// ===== value operators: morphisms ========================================

/// Aggregate a transaction into per-account totals.
/// Lossy in the MULTIPLICITY dimension; the residual captures it COMPLETELY.
pub struct Aggregate;

/// A BUGGY aggregation with the SAME type as `Aggregate`: it records only how
/// MANY postings hit each account, not their amounts. The residual is therefore
/// INCOMPLETE — the probe distinguishes it from `Aggregate` even though they are
/// type-identical morphisms.
pub struct AggregateDropsAmounts;

/// Round each account total down to whole dollars; the residual captures the
/// removed sub-dollar cents. Lossy in the SUB-DOLLAR dimension.
pub struct Round;

crate::value_operator!(Aggregate, AggregateDropsAmounts, Round);

impl Morphism for Aggregate {
    type In = Transaction;
    type Out = AccountSummary;
    type Residual = MultiplicityResidual;

    fn forward(&self, input: &Transaction) -> (AccountSummary, MultiplicityResidual) {
        let (totals, breakdown) = super::internal::fold(&input.to_pairs());
        (
            AccountSummary { totals },
            MultiplicityResidual { breakdown },
        )
    }

    fn backward(&self, _out: &AccountSummary, r: &MultiplicityResidual) -> Option<Transaction> {
        let mut postings = Vec::new();
        for (acct, amts) in &r.breakdown {
            let a = Account::new(acct)?;
            for &amt in amts {
                postings.push(Posting::new(a.clone(), Cents::new(amt)?));
            }
        }
        Transaction::new(postings)
    }
}

impl Morphism for AggregateDropsAmounts {
    type In = Transaction;
    type Out = AccountSummary;
    type Residual = MultiplicityResidual;

    fn forward(&self, input: &Transaction) -> (AccountSummary, MultiplicityResidual) {
        let (totals, breakdown) = super::internal::fold(&input.to_pairs());
        // BUG: keep only the COUNT of postings per account, not their amounts.
        let lossy = breakdown
            .into_iter()
            .map(|(k, amts)| (k, vec![amts.len() as i64]))
            .collect();
        (
            AccountSummary { totals },
            MultiplicityResidual { breakdown: lossy },
        )
    }

    fn backward(&self, out: &AccountSummary, r: &MultiplicityResidual) -> Option<Transaction> {
        // Same reconstruction logic as Aggregate — it just has too little to work
        // with, so the round-trip will not match.
        Aggregate.backward(out, r)
    }
}

impl Morphism for Round {
    type In = AccountSummary;
    type Out = AccountSummary;
    type Residual = RoundingResidual;

    fn forward(&self, input: &AccountSummary) -> (AccountSummary, RoundingResidual) {
        let mut rounded = BTreeMap::new();
        let mut leftover = BTreeMap::new();
        for (acct, &total) in &input.totals {
            let cents = total.rem_euclid(100);
            rounded.insert(acct.clone(), total - cents);
            leftover.insert(acct.clone(), cents);
        }
        (
            AccountSummary { totals: rounded },
            RoundingResidual { leftover },
        )
    }

    fn backward(&self, out: &AccountSummary, r: &RoundingResidual) -> Option<AccountSummary> {
        let mut totals = BTreeMap::new();
        for (acct, &rounded) in &out.totals {
            let cents = *r.leftover.get(acct)?;
            totals.insert(acct.clone(), rounded + cents);
        }
        Some(AccountSummary { totals })
    }
}

// ===== value operators: perturbations ====================================

/// Split the first posting into two of the same account — perturbs the
/// MULTIPLICITY dimension. Works for any morphism whose input is a Transaction.
pub struct Split;
crate::value_operator!(Split);

impl<M: Morphism<In = Transaction>> Perturbation<M> for Split {
    fn perturb(&self, input: &Transaction) -> Option<Transaction> {
        let ps = input.postings();
        let first = ps.first()?;
        let a = first.amount().get();
        let half = a / 2;
        let rest = a - half;
        let mut out = Vec::new();
        if half != 0 {
            out.push(Posting::new(first.account().clone(), Cents::new(half)?));
        }
        out.push(Posting::new(first.account().clone(), Cents::new(rest)?));
        out.extend(ps[1..].iter().cloned());
        Transaction::new(out)
    }
}

/// Nudge the first account's total by one cent — perturbs the SUB-DOLLAR
/// dimension. Works for any morphism whose input is an AccountSummary.
pub struct NudgeCents;
crate::value_operator!(NudgeCents);

impl<M: Morphism<In = AccountSummary>> Perturbation<M> for NudgeCents {
    fn perturb(&self, input: &AccountSummary) -> Option<AccountSummary> {
        let mut totals = input.totals.clone();
        let (k, v) = totals.iter_mut().next()?;
        let _ = k;
        *v += 1;
        Some(AccountSummary { totals })
    }
}
