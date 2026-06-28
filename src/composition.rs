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
//!
//! METAMORPHIC RELATIONS under composition (the second §7 question) do NOT compose
//! for free, in two dual ways (see the tests).
//!
//! First, a relation does not LIFT through a stage that does not preserve it:
//! `DoublePostings` (doubling) holds for `Aggregate`, but FAILS for the honest
//! `Aggregate ∘ Round` on sub-dollar input, because rounding is non-linear
//! (round-then-double ≠ double-then-round). Both stages are correct, so naively
//! reusing a stage's relation on the composite is a FALSE POSITIVE; a relation
//! survives only where every intervening stage preserves it.
//!
//! Second, dually, a downstream stage can MASK an upstream bug: `Round` absorbs
//! `AggregateOffsetsTotals`'s sub-dollar offset, so the composite's output is
//! identical to the honest one AND still round-trips — even though the component is
//! wrong (its own commutation probe catches it).
//!
//! Conclusion: you need BOTH per-module and composite-level checks; neither
//! subsumes the other.

use crate::boundary::{Construction, Morphism};
use crate::gdp::{prove_permutation, unpermute, Name, Named, Paired, PermutationOf, Proof, Seed};
use crate::ledger::boundary::{
    AccountSummary, Aggregate, MultiplicityResidual, ParseTransaction, Posting, Transaction,
};

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

/// The same matched-pair seam, GENERALIZED to the ENTRY edge. A `Construction`'s
/// `parse` returns `(refined, residual)` that correspond — the residual reconstructs
/// the raw only WITH its own refined value — yet `reconstruct` takes them separately,
/// so a refined from one parse and a residual from another silently mis-reconstruct.
/// Morphisms get the `Carried`/`run` wrapper that bundles them; constructions had no
/// such wrapper, so this is where output⋈residual coupling reaches the parse edge:
/// `parse_paired` brands the pair with one shared name so they can be stored apart yet
/// only re-paired with each other.
pub fn parse_paired<C: Construction, N: Name>(
    seed: Seed<N>,
    c: &C,
    raw: &C::Raw,
) -> Option<Paired<impl Name, C::Refined, C::Residual>> {
    c.parse(raw)
        .map(|(refined, residual)| seed.new_paired(refined, residual))
}

/// Reconstruct from a branded (refined, residual) pair — only same-parse values,
/// sharing a name, recombine; a mismatched pair will not compile, so no cross-run
/// check is needed.
pub fn reconstruct_paired<C: Construction, N>(
    c: &C,
    refined: &Named<N, C::Refined>,
    residual: &Named<N, C::Residual>,
) -> Option<C::Raw> {
    c.reconstruct(refined.value(), residual.value())
}

// ===== a COUPLING-AWARE construction: the proof makes reconstruct total =====

/// The seed-taking construction the plain `Construction` trait cannot express: its
/// `parse` has no seed, and a GDP name cannot escape a `with_seed` closure, so a
/// branded/proven residual needs a construction that takes the seed itself. Here the
/// sorted `Transaction` and its permutation residual are named TOGETHER and the
/// permutation is PROVEN valid, so reconstruction drops the runtime bounds/bijection
/// checks the plain `ParseTransaction::reconstruct` must keep — it becomes TOTAL.
pub struct CoupledTransaction<N> {
    tx: Named<N, Transaction>,
    order: Named<N, Vec<usize>>,
    proof: Proof<N, PermutationOf>,
}

impl<N> CoupledTransaction<N> {
    /// The canonical (sorted) transaction.
    pub fn transaction(&self) -> &Transaction {
        self.tx.value()
    }

    /// Reconstruct the EXACT original posting order — TOTAL (no `Option`), because the
    /// `PermutationOf` proof guarantees the residual is a valid bijection.
    pub fn reconstruct(&self) -> Vec<Posting> {
        unpermute(&self.order, &self.proof, self.tx.value().postings())
    }
}

/// Parse a raw posting list into a `CoupledTransaction`: name the sorted transaction
/// and its discarded ordering together, and prove the ordering a valid permutation
/// (it always is, by construction). `None` only for an empty input (no transaction).
pub fn parse_transaction_coupled<N: Name>(
    seed: Seed<N>,
    raw: &[Posting],
) -> Option<CoupledTransaction<impl Name>> {
    let (tx, residual) = ParseTransaction.parse(&raw.to_vec())?;
    let order: Vec<usize> = residual.positions().iter().map(|i| i.get()).collect();
    let len = order.len();
    let (named_tx, named_order) = seed.new_paired(tx, order).split();
    let proof = prove_permutation(&named_order, len)?;
    Some(CoupledTransaction {
        tx: named_tx,
        order: named_order,
        proof,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::boundary::{commutes, probe, Compose};
    use crate::gdp::with_seed;
    use crate::ledger::boundary::{
        Account, AggregateOffsetsTotals, Cents, DoublePostings, Posting, Round, Split,
    };

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
            // name, `reconcile(&summary, &other_residual)` is a type error — the two
            // names do not unify, so a mismatched pair cannot even be expressed.
        });
    }

    /// The seam GENERALIZED to a construction: a `ParseTransaction` refined value and
    /// its permutation residual, branded together, recombine to the exact raw — and
    /// only with their own partner. (Constructions previously had no wrapper coupling
    /// the two halves of `parse`.)
    #[test]
    fn construction_seam_recombines_only_matched_pairs() {
        use crate::ledger::boundary::ParseTransaction;
        with_seed(|seed| {
            // input order differs from canonical, so the residual is non-trivial.
            let raw = vec![
                Posting::new(Account::new("Revenue").unwrap(), Cents::new(-60).unwrap()),
                Posting::new(Account::new("Cash").unwrap(), Cents::new(60).unwrap()),
            ];
            let paired = parse_paired(seed, &ParseTransaction, &raw).expect("non-empty parses");
            let (refined, residual) = paired.split();
            assert_eq!(
                reconstruct_paired(&ParseTransaction, &refined, &residual).as_ref(),
                Some(&raw)
            );
            // A residual from a different parse, under another name, would not unify —
            // the mismatched recombination cannot be expressed.
        });
    }

    /// The coupling-aware construction: with the permutation PROVEN valid, reconstruct
    /// is TOTAL — it returns `Vec<Posting>`, not `Option`, recovering the exact input
    /// order with no runtime bounds/bijection checks.
    #[test]
    fn coupled_transaction_reconstructs_totally() {
        with_seed(|seed| {
            // input order differs from canonical, so the permutation is non-trivial.
            let raw = vec![
                Posting::new(Account::new("Revenue").unwrap(), Cents::new(-60).unwrap()),
                Posting::new(Account::new("Cash").unwrap(), Cents::new(60).unwrap()),
            ];
            let coupled = parse_transaction_coupled(seed, &raw).expect("non-empty input");
            assert_eq!(
                coupled.transaction(),
                &Transaction::new(raw.clone()).unwrap()
            );
            let recovered: Vec<Posting> = coupled.reconstruct(); // total — no Option
            assert_eq!(recovered, raw);
        });
    }

    // ----- metamorphic relations under composition -----

    /// FINDING 1: a relation does NOT lift through a stage that doesn't preserve
    /// it. `DoublePostings` holds for `Aggregate` alone, but FAILS for the honest
    /// `Aggregate ∘ Round` on sub-dollar input — rounding is non-linear. Both
    /// stages are correct, so lifting a stage's relation to the composite is a
    /// false positive.
    #[test]
    fn relation_does_not_survive_a_nonlinear_stage() {
        let subdollar = tx(150); // $1.50 — rounding is not the identity here
        assert_eq!(
            commutes(&Aggregate, &DoublePostings, &subdollar),
            Some(true),
            "the relation holds for the stage alone"
        );
        let composite = Compose {
            f: Aggregate,
            g: Round,
        };
        assert_eq!(
            commutes(&composite, &DoublePostings, &subdollar),
            Some(false),
            "but it does not survive composition through the non-linear Round"
        );
    }

    /// ...and it DOES survive on the sub-domain the stage preserves: whole dollars,
    /// where `Round` is the identity. A relation lifts exactly where every
    /// intervening stage preserves it.
    #[test]
    fn relation_survives_where_the_stage_preserves_it() {
        let whole = tx(10_000); // $100.00 — Round is a no-op
        let composite = Compose {
            f: Aggregate,
            g: Round,
        };
        assert_eq!(commutes(&composite, &DoublePostings, &whole), Some(true));
    }

    /// FINDING 2 (the dual): a downstream stage MASKS an upstream bug, so an
    /// end-to-end check is blind to it. The offset bug is caught at its own
    /// boundary, but `Round` absorbs the sub-dollar offset — the composite's output
    /// equals the honest composite's AND still round-trips. Per-module checking is
    /// therefore necessary; end-to-end is not sufficient.
    #[test]
    fn a_downstream_stage_masks_an_upstream_bug() {
        let t = tx(10_000); // $100.00

        // caught at the module boundary by commutation:
        assert_eq!(
            commutes(&AggregateOffsetsTotals, &DoublePostings, &t),
            Some(false)
        );

        // but masked at the composite — identical output, and it round-trips:
        let honest = Compose {
            f: Aggregate,
            g: Round,
        };
        let buggy = Compose {
            f: AggregateOffsetsTotals,
            g: Round,
        };
        let (honest_out, _) = honest.forward(&t);
        let (buggy_out, buggy_res) = buggy.forward(&t);
        assert_eq!(
            honest_out, buggy_out,
            "Round absorbs the sub-dollar offset — outputs identical"
        );
        assert_eq!(
            buggy.backward(&buggy_out, &buggy_res).as_ref(),
            Some(&t),
            "and the composite still round-trips, so end-to-end is blind to the bug"
        );
    }
}
