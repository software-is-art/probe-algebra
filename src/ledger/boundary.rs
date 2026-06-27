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
///
/// `Cents` carries its OWN value operators, so amount arithmetic stays in the
/// typed domain instead of being lowered to raw `i64` and re-validated. The
/// operators uphold the range invariant: total maps return `Cents`, partial ones
/// (which can leave the valid range) return `Option<Cents>`.
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

    /// The additive identity.
    pub fn zero() -> Self {
        Cents(0)
    }

    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }

    /// Negation — total, since the valid range is symmetric.
    pub fn negate(self) -> Self {
        Cents(-self.0)
    }

    /// Range-checked addition (partial: `None` if the sum leaves the valid range).
    pub fn checked_add(self, other: Cents) -> Option<Cents> {
        Cents::new(self.0 + other.0)
    }

    /// Range-checked subtraction (partial).
    pub fn checked_sub(self, other: Cents) -> Option<Cents> {
        Cents::new(self.0 - other.0)
    }

    /// Split into two parts that sum back to the original (`half + rest == self`).
    /// Total and loss-free: `split` followed by `checked_add` round-trips.
    pub fn split(self) -> (Cents, Cents) {
        let half = self.0 / 2;
        (Cents(half), Cents(self.0 - half))
    }
}

/// A running BALANCE — a sum of cent amounts. Distinct from `Cents`: a sum can
/// exceed any single amount, so it is its own value object with its own
/// operators. It is built ONLY through those operators (from `Cents`), never
/// from a raw integer, so a balance's provenance is guaranteed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Balance(i64);
impl Balance {
    /// The empty balance — the only way to start one without a `Cents`.
    pub fn zero() -> Self {
        Balance(0)
    }
    /// Accessor: the sanctioned exit hatch back to a raw integer (e.g. display).
    pub fn get(&self) -> i64 {
        self.0
    }
    /// Accumulate a posting amount.
    pub fn add_cents(self, c: Cents) -> Self {
        Balance(self.0 + c.get())
    }
    /// Combine two balances.
    pub fn plus(self, other: Balance) -> Self {
        Balance(self.0 + other.0)
    }
    pub fn negate(self) -> Self {
        Balance(-self.0)
    }
    /// Split at the dollar boundary into (whole dollars, sub-dollar remainder).
    /// Total and loss-free: `whole.add_cents(remainder) == self`.
    pub fn split_dollar(self) -> (Balance, Cents) {
        let remainder = self.0.rem_euclid(100);
        (Balance(self.0 - remainder), Cents(remainder))
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
}

/// OUTPUT value object: per-account totals. Multiplicity is GONE (each account
/// has one total). Constructed only inside this boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountSummary {
    totals: BTreeMap<String, Balance>,
}
impl AccountSummary {
    /// Module-scoped constructor: the internal aggregation hands back
    /// `Account`-keyed balances; the summary stores plain labels for its view.
    pub(in crate::ledger) fn from_totals(totals: BTreeMap<Account, Balance>) -> Self {
        Self {
            totals: totals
                .into_iter()
                .map(|(a, b)| (a.get().to_string(), b))
                .collect(),
        }
    }
    pub fn totals(&self) -> &BTreeMap<String, Balance> {
        &self.totals
    }
}

/// RESIDUAL value object for aggregation: the MULTIPLICITY that aggregation
/// collapsed — the original per-account breakdown of each total into its
/// constituent posting amounts (sorted). Summary + this residual reconstructs
/// the transaction. The breakdown keeps VALUE OBJECTS (`Account` / `Cents`), so
/// reconstruction needs no re-validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultiplicityResidual {
    breakdown: BTreeMap<Account, Vec<Cents>>,
}
impl MultiplicityResidual {
    pub(in crate::ledger) fn from_breakdown(breakdown: BTreeMap<Account, Vec<Cents>>) -> Self {
        Self { breakdown }
    }
    pub(in crate::ledger) fn breakdown(&self) -> &BTreeMap<Account, Vec<Cents>> {
        &self.breakdown
    }
}

/// RESIDUAL value object for rounding: the sub-dollar cents removed from each
/// account total. Rounded total + this residual reconstructs the exact total.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoundingResidual {
    leftover: BTreeMap<String, Cents>,
}

crate::value_object!(
    Cents,
    Balance,
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

    // The boundary operator is a thin adapter over the internal morphism, which
    // holds the actual (probeable) aggregation logic.
    fn forward(&self, input: &Transaction) -> (AccountSummary, MultiplicityResidual) {
        super::internal::Aggregation.forward(input)
    }

    fn backward(&self, out: &AccountSummary, r: &MultiplicityResidual) -> Option<Transaction> {
        super::internal::Aggregation.backward(out, r)
    }
}

impl Morphism for AggregateDropsAmounts {
    type In = Transaction;
    type Out = AccountSummary;
    type Residual = MultiplicityResidual;

    fn forward(&self, input: &Transaction) -> (AccountSummary, MultiplicityResidual) {
        let (summary, full) = super::internal::Aggregation.forward(input);
        // BUG: keep only the COUNT of postings per account, not their amounts.
        let lossy = full
            .breakdown()
            .iter()
            .map(|(account, amounts)| {
                let count = Cents::new(amounts.len() as i64).expect("posting count fits in Cents");
                (account.clone(), vec![count])
            })
            .collect();
        (summary, MultiplicityResidual::from_breakdown(lossy))
    }

    fn backward(&self, out: &AccountSummary, r: &MultiplicityResidual) -> Option<Transaction> {
        // Same reconstruction logic — it just has too little to work with, so the
        // round-trip will not match.
        super::internal::Aggregation.backward(out, r)
    }
}

impl Morphism for Round {
    type In = AccountSummary;
    type Out = AccountSummary;
    type Residual = RoundingResidual;

    fn forward(&self, input: &AccountSummary) -> (AccountSummary, RoundingResidual) {
        let mut rounded = BTreeMap::new();
        let mut leftover = BTreeMap::new();
        for (acct, balance) in &input.totals {
            // split at the dollar boundary using Balance's own operator
            let (whole, remainder) = balance.split_dollar();
            rounded.insert(acct.clone(), whole);
            leftover.insert(acct.clone(), remainder);
        }
        (
            AccountSummary { totals: rounded },
            RoundingResidual { leftover },
        )
    }

    fn backward(&self, out: &AccountSummary, r: &RoundingResidual) -> Option<AccountSummary> {
        let mut totals = BTreeMap::new();
        for (acct, rounded) in &out.totals {
            let remainder = *r.leftover.get(acct)?;
            totals.insert(acct.clone(), rounded.add_cents(remainder));
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
        // split the amount using Cents' own operator — no lowering to i64, no
        // re-validation: split() is total and its parts are valid by construction.
        let (half, rest) = first.amount().split();
        let mut out = Vec::new();
        if !half.is_zero() {
            out.push(Posting::new(first.account().clone(), half));
        }
        out.push(Posting::new(first.account().clone(), rest));
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
        let (_acct, balance) = totals.iter_mut().next()?;
        *balance = balance.add_cents(Cents::new(1).expect("one cent is valid"));
        Some(AccountSummary { totals })
    }
}
