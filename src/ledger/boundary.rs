//! ledger::boundary — the ledger module's PUBLIC interface.
//!
//! By the boundary discipline this file contains ONLY (a category of objects and
//! morphisms):
//! - VALUE OBJECTS — the objects: Cents, Account, Posting, Transaction,
//!   AccountSummary, MultiplicityResidual, RoundingResidual, Affix, Index, PostingOrder.
//! - VALUE OPERATORS — the morphisms: Aggregate, AggregateDropsAmounts, Round
//!   (Morphisms); ParseCents, ParseAccount, ParseTransaction (Constructions — the
//!   ENTRY edge from a raw primitive); Split, NudgeCents, PadName, ReorderPostings
//!   (Perturbations).
//! - (typestates — object indices — live in the universal `crate::boundary`)
//!
//! Each value object carries the sealed `ValueObject` marker; each operator the
//! sealed `ValueOperator` marker. The aggregation ALGORITHM is delegated to the
//! private `super::internal` module — it never appears at the boundary.

use std::collections::BTreeMap;

use crate::boundary::{
    Capability, Construction, Metamorphic, Morphism, Pair, Perturbation, RawPerturbation, Unit,
};

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
    /// Smart constructor — the ergonomic facade over the `ParseCents` construction,
    /// which is the single source of truth (and what the round-trip probe certifies).
    pub fn new(c: i64) -> Option<Self> {
        ParseCents.parse(&c).map(|(v, _)| v)
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
    /// Smart constructor — the ergonomic facade over the `ParseAccount` construction.
    /// `ParseAccount` ALSO captures the trimmed padding as its residual; `new` keeps
    /// only the value object and drops that residual.
    pub fn new(s: &str) -> Option<Self> {
        ParseAccount.parse(&s.to_string()).map(|(v, _)| v)
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
    /// Smart constructor — the ergonomic facade over the `ParseTransaction`
    /// construction, which canonically sorts the postings and captures the discarded
    /// input ordering as its residual. `new` keeps only the sorted value object.
    pub fn new(p: Vec<Posting>) -> Option<Self> {
        ParseTransaction.parse(&p).map(|(v, _)| v)
    }
    pub fn postings(&self) -> &[Posting] {
        &self.postings
    }
}

/// OUTPUT value object: per-account totals. Multiplicity is GONE (each account
/// has one total). Constructed only inside this boundary.
///
/// Keyed by `Account`, NOT `String`: downgrading the key to a raw label was the
/// value object dropped to a primitive, which forced later code to re-parse it
/// (`Account::new(label).expect(...)`). Keeping the value object removes the
/// re-parse — and the latent panic — entirely.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountSummary {
    totals: BTreeMap<Account, Balance>,
}
impl AccountSummary {
    /// Module-scoped constructor: the internal aggregation hands back
    /// `Account`-keyed balances, and the summary keeps them as such.
    pub(in crate::ledger) fn from_totals(totals: BTreeMap<Account, Balance>) -> Self {
        Self { totals }
    }
    pub fn totals(&self) -> &BTreeMap<Account, Balance> {
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
    leftover: BTreeMap<Account, Cents>,
}

/// A run of surrounding whitespace that `ParseAccount` trimmed off a name. Empty
/// when the raw had none; otherwise it witnesses EXACTLY what normalization removed,
/// so a leading + trailing pair reconstructs the original padded string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Affix(String);
impl Affix {
    pub fn get(&self) -> &str {
        &self.0
    }
}

/// A position in the original, pre-canonicalization posting order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Index(usize);
impl Index {
    pub fn get(&self) -> usize {
        self.0
    }
}

/// RESIDUAL value object for `ParseTransaction`: the permutation it discarded when it
/// sorted the postings into canonical order. `positions()[k]` is the ORIGINAL index
/// of the k-th sorted posting, so the sorted transaction plus this residual restores
/// the exact input ordering — the construction analog of `MultiplicityResidual`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostingOrder(Vec<Index>);
impl PostingOrder {
    pub fn positions(&self) -> &[Index] {
        &self.0
    }
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
    Affix,
    Index,
    PostingOrder,
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

/// Another BUGGY aggregation, type-identical to `Aggregate`, that adds a fixed
/// one-cent OFFSET to every account total. Its residual (the breakdown) is
/// untouched, so it ROUND-TRIPS (backward rebuilds from the breakdown) and its
/// residual probe passes — yet its OUTPUT is wrong. Only a *commutation* probe
/// (e.g. `DoublePostings`) catches it: an additive offset breaks linearity
/// (`2·(s+c) ≠ 2·s + c`). The complement of `AggregateDropsAmounts`, which the
/// commutation probe is in turn blind to. Together they populate the blind-spot
/// map: no single structural probe catches both.
pub struct AggregateOffsetsTotals;

/// Round each account total down to whole dollars; the residual captures the
/// removed sub-dollar cents. Lossy in the SUB-DOLLAR dimension.
pub struct Round;

crate::value_operator!(
    Aggregate,
    AggregateDropsAmounts,
    AggregateOffsetsTotals,
    Round
);

impl Morphism for Aggregate {
    const CAPABILITY: Capability = Capability::Lossy;

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
    const CAPABILITY: Capability = Capability::Lossy;

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

impl Morphism for AggregateOffsetsTotals {
    const CAPABILITY: Capability = Capability::Lossy;

    type In = Transaction;
    type Out = AccountSummary;
    type Residual = MultiplicityResidual;

    fn forward(&self, input: &Transaction) -> (AccountSummary, MultiplicityResidual) {
        let (summary, residual) = super::internal::Aggregation.forward(input);
        // BUG: every total is offset by one cent. The residual is left correct,
        // so the round-trip (which rebuilds from the residual) still holds — the
        // damage is only in the OUTPUT, where a commutation probe must catch it.
        let mut totals = BTreeMap::new();
        for (account, balance) in summary.totals() {
            totals.insert(
                account.clone(),
                balance.add_cents(Cents::new(1).expect("one cent is valid")),
            );
        }
        (AccountSummary::from_totals(totals), residual)
    }

    fn backward(&self, out: &AccountSummary, r: &MultiplicityResidual) -> Option<Transaction> {
        super::internal::Aggregation.backward(out, r)
    }
}

impl Morphism for Round {
    const CAPABILITY: Capability = Capability::Lossy;

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

/// A METAMORPHIC RELATION (structural, reference-free): duplicating every
/// posting must double every account total. Holds for any honest aggregation by
/// linearity, so it witnesses the SHAPE without pinning constants. It catches a
/// non-linear output bug (`AggregateOffsetsTotals`) but is BLIND to a
/// residual-only bug (`AggregateDropsAmounts`), whose output is already correct.
pub struct DoublePostings;
crate::value_operator!(DoublePostings);

impl<M: Morphism<In = Transaction, Out = AccountSummary>> Metamorphic<M> for DoublePostings {
    fn input_op(&self, x: &Transaction) -> Option<Transaction> {
        let mut out = Vec::new();
        for p in x.postings() {
            out.push(p.clone());
            out.push(p.clone());
        }
        Transaction::new(out)
    }

    fn output_op(&self, y: &AccountSummary) -> AccountSummary {
        let mut totals = BTreeMap::new();
        for (account, balance) in y.totals() {
            totals.insert(account.clone(), balance.plus(*balance));
        }
        AccountSummary::from_totals(totals)
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

// ===== value operators: constructions (the entry edge) ===================

/// Parse a raw `i64` into `Cents`. A PURE REFINEMENT: it only range-checks, losing
/// nothing, so its residual is `Unit` and `reconstruct` returns the exact integer.
pub struct ParseCents;

/// Parse a raw `String` into an `Account`, trimming surrounding whitespace. A
/// NORMALIZING parse: the trimmed padding is a real lost dimension, so the residual
/// is the leading/trailing `Affix` pair that `reconstruct` re-attaches.
pub struct ParseAccount;

/// A BUGGY parse, type-shaped like a PURE refinement (`Unit` residual) though it
/// actually normalizes: it trims the name but keeps no witness, so it cannot rebuild
/// the padding it removed. `reconstructs` catches it on any padded input — the
/// entry-edge analog of `AggregateDropsAmounts`.
pub struct ParseAccountDropsPadding;

/// Parse a raw `Vec<Posting>` into a `Transaction`, sorting into canonical order. A
/// NORMALIZING parse: sorting discards the input ordering, so the residual is the
/// `PostingOrder` permutation that `reconstruct` un-applies.
pub struct ParseTransaction;

crate::value_operator!(
    ParseCents,
    ParseAccount,
    ParseAccountDropsPadding,
    ParseTransaction
);

impl Construction for ParseCents {
    const CAPABILITY: Capability = Capability::Pure;

    type Raw = i64;
    type Refined = Cents;
    type Residual = Unit;

    fn parse(&self, raw: &i64) -> Option<(Cents, Unit)> {
        if raw.abs() <= 100_000_000 {
            Some((Cents(*raw), Unit))
        } else {
            None
        }
    }

    fn reconstruct(&self, refined: &Cents, _residual: &Unit) -> Option<i64> {
        Some(refined.0)
    }
}

impl Construction for ParseAccount {
    // Trimming collapses the surrounding whitespace — a real lost dimension — so the
    // honest ceiling is `Lossy` (the residual restores it), exactly like `Aggregate`.
    const CAPABILITY: Capability = Capability::Lossy;

    type Raw = String;
    type Refined = Account;
    type Residual = Pair<Affix, Affix>;

    fn parse(&self, raw: &String) -> Option<(Account, Pair<Affix, Affix>)> {
        // Byte lengths of the trimmed-away runs are valid char boundaries, since
        // `trim_start`/`trim_end` return suffixes/prefixes of `raw`.
        let start = raw.len() - raw.trim_start().len();
        let end = raw.trim_end().len();
        if start >= end {
            return None; // empty or all-whitespace: no name to refine
        }
        let leading = raw[..start].to_string();
        let core = raw[start..end].to_string();
        let trailing = raw[end..].to_string();
        Some((Account(core), Pair(Affix(leading), Affix(trailing))))
    }

    fn reconstruct(&self, refined: &Account, residual: &Pair<Affix, Affix>) -> Option<String> {
        Some(format!(
            "{}{}{}",
            residual.0.get(),
            refined.get(),
            residual.1.get()
        ))
    }
}

impl Construction for ParseAccountDropsPadding {
    // It CLAIMS to be a pure refinement (`Unit` residual, `Pure`) though it actually
    // trims — the lie `construction_probe` exposes, the entry-edge twin of
    // `AggregateDropsAmounts`'s incomplete residual.
    const CAPABILITY: Capability = Capability::Pure;

    type Raw = String;
    type Refined = Account;
    type Residual = Unit;

    fn parse(&self, raw: &String) -> Option<(Account, Unit)> {
        let core = raw.trim();
        if core.is_empty() {
            None
        } else {
            Some((Account(core.to_string()), Unit))
        }
    }

    fn reconstruct(&self, refined: &Account, _residual: &Unit) -> Option<String> {
        // BUG: with no residual, the best it can do is the trimmed name — the padding
        // is gone, so a padded input cannot round-trip.
        Some(refined.get().to_string())
    }
}

impl Construction for ParseTransaction {
    // Sorting discards the input ordering (the residual restores it) — `Lossy`.
    const CAPABILITY: Capability = Capability::Lossy;

    type Raw = Vec<Posting>;
    type Refined = Transaction;
    type Residual = PostingOrder;

    fn parse(&self, raw: &Vec<Posting>) -> Option<(Transaction, PostingOrder)> {
        if raw.is_empty() {
            return None;
        }
        // Pair each posting with its original index, then sort by posting (stably).
        let mut indexed: Vec<(usize, Posting)> = raw.iter().cloned().enumerate().collect();
        indexed.sort_by(|a, b| a.1.cmp(&b.1));
        let postings: Vec<Posting> = indexed.iter().map(|(_, p)| p.clone()).collect();
        let order: Vec<Index> = indexed.iter().map(|(i, _)| Index(*i)).collect();
        Some((Transaction { postings }, PostingOrder(order)))
    }

    fn reconstruct(&self, refined: &Transaction, residual: &PostingOrder) -> Option<Vec<Posting>> {
        let sorted = refined.postings();
        let order = residual.positions();
        if sorted.len() != order.len() {
            return None;
        }
        // `order[k]` is the original index of `sorted[k]`; place each back home.
        let mut out: Vec<Option<Posting>> = vec![None; sorted.len()];
        for (k, idx) in order.iter().enumerate() {
            let pos = idx.get();
            if pos >= out.len() {
                return None;
            }
            out[pos] = Some(sorted[k].clone());
        }
        out.into_iter().collect()
    }
}

// ===== value operators: raw perturbations (probe the construction edge) ===

/// Prepend a space to a raw name — perturbs the LEADING-PADDING dimension that
/// `ParseAccount` normalizes away. A complete residual leaves the `Account` invariant
/// and records the extra space.
pub struct PadName;
crate::value_operator!(PadName);

impl RawPerturbation<ParseAccount> for PadName {
    fn perturb(&self, raw: &String) -> Option<String> {
        Some(format!(" {raw}"))
    }
}

/// The SAME perturbation pointed at the lying parse, so one probe catches the
/// `Unit`-residual `ParseAccountDropsPadding` exactly as `Split` catches
/// `AggregateDropsAmounts`.
impl RawPerturbation<ParseAccountDropsPadding> for PadName {
    fn perturb(&self, raw: &String) -> Option<String> {
        Some(format!(" {raw}"))
    }
}

/// Rotate the postings by one — perturbs the ORDERING dimension that
/// `ParseTransaction` normalizes away by sorting. `None` for a single posting (no
/// reordering is possible), like `Split` on a degenerate input.
pub struct ReorderPostings;
crate::value_operator!(ReorderPostings);

impl RawPerturbation<ParseTransaction> for ReorderPostings {
    fn perturb(&self, raw: &Vec<Posting>) -> Option<Vec<Posting>> {
        if raw.len() < 2 {
            return None;
        }
        let mut out = raw.clone();
        out.rotate_left(1);
        Some(out)
    }
}
