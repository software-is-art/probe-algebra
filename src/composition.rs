//! composition — the §7 frontier: an A∘B INTERACTION bug invisible to per-module
//! probes, and a GDP seam contract that closes it.
//!
//! `Aggregate` produces `(summary, residual)` that CORRESPOND — the residual is a
//! breakdown of the summary. Any downstream that recombines a summary with a
//! residual is correct only if they came from the SAME run. That is a RELATION
//! between two values: no single value object can express it, and a per-module
//! probe never sees it — the probe always tests forward-then-backward on a
//! matched pair, so it cannot construct, let alone catch, a mismatched one.
//! A mismatched recombination is therefore a silent INTERACTION bug.
//!
//! `explains` shows the bug is real. The GDP seam (`aggregate_paired` + `reconcile`)
//! brands the summary and residual with one shared name, so only same-run values
//! recombine — a mismatch is a COMPILE error, and the "explains" precondition is
//! discharged by provenance with no runtime check.
//!
//! Scope (the honest limit): this holds for IN-PROCESS recombination inside one
//! `with_seed`. Across a persistence/serialization boundary the brand cannot
//! travel, so there you fall back to the runtime `explains` check — exactly where
//! a carried compile-time proof would be unsound anyway.

use crate::boundary::Morphism;
use crate::gdp::{Name, Named, Paired, Seed};
use crate::ledger::boundary::{AccountSummary, Aggregate, MultiplicityResidual, Transaction};

/// The RELATION, via public API only: does `residual` actually explain `summary`?
/// Reconstruct the transaction from the residual, re-aggregate it, and compare to
/// the summary. (`Aggregate::backward` rebuilds from the residual alone, so a
/// mismatched summary shows up as a re-aggregation that differs from it.)
pub fn explains(summary: &AccountSummary, residual: &MultiplicityResidual) -> bool {
    match Aggregate.backward(summary, residual) {
        Some(tx) => &Aggregate.forward(&tx).0 == summary,
        None => false,
    }
}

/// Run `Aggregate`, branding the summary and its residual with ONE shared name.
/// They can be split apart and routed to different stores, yet only re-paired with
/// each other.
pub fn aggregate_paired<N: Name>(
    seed: Seed<N>,
    tx: &Transaction,
) -> Paired<impl Name, AccountSummary, MultiplicityResidual> {
    let (summary, residual) = Aggregate.forward(tx);
    seed.new_paired(summary, residual)
}

/// Recombine — accepts a summary and residual ONLY when they share a name (i.e.
/// came from the same `aggregate_paired`). The "explains" precondition is
/// discharged by provenance: no runtime check here, and a mismatch will not
/// compile.
pub fn reconcile<N>(
    summary: &Named<N, AccountSummary>,
    residual: &Named<N, MultiplicityResidual>,
) -> Option<Transaction> {
    Aggregate.backward(summary.value(), residual.value())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::boundary::probe;
    use crate::gdp::with_seed;
    use crate::ledger::boundary::{Account, Cents, Posting, Split};

    fn tx(cash: i64) -> Transaction {
        Transaction::new(vec![Posting::new(
            Account::new("Cash").unwrap(),
            Cents::new(cash).unwrap(),
        )])
        .unwrap()
    }

    /// A matched pair is consistent, and the module itself is correct in isolation
    /// — its probe passes. The bug below is purely at the seam.
    #[test]
    fn matched_pair_explains_and_the_module_is_correct() {
        let t = tx(100);
        let (summary, residual) = Aggregate.forward(&t);
        assert!(explains(&summary, &residual));
        assert!(probe(&Aggregate, &Split, &t).unwrap().residual_complete());
    }

    /// THE INTERACTION BUG: a summary from one run recombined with a residual from
    /// another is silently inconsistent — and no per-module probe ever builds this
    /// pair, so none can catch it.
    #[test]
    fn mismatched_pair_is_a_silent_interaction_bug() {
        let (summary1, _r1) = Aggregate.forward(&tx(100));
        let (_s2, residual2) = Aggregate.forward(&tx(200));
        assert!(
            !explains(&summary1, &residual2),
            "a residual from run 2 must not explain a summary from run 1"
        );
    }

    /// The GDP seam: branded values recombine correctly, and only with their
    /// partner. Crossing brands from two runs does NOT compile (see the note).
    #[test]
    fn gdp_seam_recombines_only_matched_pairs() {
        with_seed(|seed| {
            let t = tx(100);
            let paired = aggregate_paired(seed, &t);
            let (summary, residual) = paired.split();
            assert_eq!(reconcile(&summary, &residual).as_ref(), Some(&t));

            // Negative (compile-checked by hand): with a second run under another
            // name, `reconcile(&summary, &other.right())` is a type error — the two
            // names do not unify, so a mismatched pair cannot even be expressed.
        });
    }
}
