//! properties.rs — the probe/residual algebra under property-based testing.
//!
//! These turn the single-sample probe into coverage: every law is checked over
//! many generated transactions. Strategies build value objects ONLY through
//! their public smart constructors, exactly as another module would.
//!
//! Two amount regimes:
//!   - `tx_broad`        : any valid amount (incl. 0 and ±1) — for round-trip laws.
//!   - `tx_substantial`  : |amount| in [1000, 1_000_000] — guarantees the Split
//!     perturbation actually moves multiplicity and that a count-only residual
//!     cannot masquerade as complete, so the completeness laws are exercised
//!     without degenerate inputs muddying them.

use proptest::prelude::*;

use crate::boundary::{probe, run, Compose, Morphism};
use crate::ledger::boundary::{
    Account, Aggregate, AggregateDropsAmounts, Cents, Posting, Round, Split, Transaction,
};

// ===== strategies ========================================================

/// A small account space so multiplicity collisions (several postings to one
/// account) are common — that is the dimension aggregation loses.
fn account() -> impl Strategy<Value = Account> {
    prop_oneof![Just("Cash"), Just("Revenue"), Just("Tax"), Just("Fees"),]
        .prop_map(|s| Account::new(s).expect("literal account name is valid"))
}

fn amount_broad() -> impl Strategy<Value = i64> {
    -1_000_000i64..=1_000_000i64
}

fn amount_substantial() -> impl Strategy<Value = i64> {
    prop_oneof![1_000i64..=1_000_000i64, -1_000_000i64..=-1_000i64]
}

fn posting(amount: impl Strategy<Value = i64>) -> impl Strategy<Value = Posting> {
    (account(), amount).prop_map(|(a, m)| Posting::new(a, Cents::new(m).expect("amount in range")))
}

fn transaction(amount: impl Strategy<Value = i64>) -> impl Strategy<Value = Transaction> {
    prop::collection::vec(posting(amount), 1..=8)
        .prop_map(|p| Transaction::new(p).expect("non-empty postings"))
}

fn tx_broad() -> impl Strategy<Value = Transaction> {
    transaction(amount_broad())
}

fn tx_substantial() -> impl Strategy<Value = Transaction> {
    transaction(amount_substantial())
}

// ===== laws ==============================================================

proptest! {
    /// An honest residual makes the lossy map invertible for EVERY input.
    #[test]
    fn honest_aggregate_round_trips(x in tx_broad()) {
        let carried = run(&Aggregate, &x);
        let recovered = carried.invert(&Aggregate);
        prop_assert_eq!(recovered.as_ref(), Some(&x));
    }

    /// Rounding is invertible given its residual, for every summary.
    #[test]
    fn round_round_trips(x in tx_broad()) {
        let summary = run(&Aggregate, &x).out().clone();
        let (rounded, residual) = Round.forward(&summary);
        let recovered = Round.backward(&rounded, &residual);
        prop_assert_eq!(recovered.as_ref(), Some(&summary));
    }

    /// Composition: invertibility flows through TWO lossy stages when the paired
    /// residual is retained.
    #[test]
    fn compose_round_trips(x in tx_broad()) {
        let pipeline = Compose { f: Aggregate, g: Round };
        let (out, res) = pipeline.forward(&x);
        let recovered = pipeline.backward(&out, &res);
        prop_assert_eq!(recovered.as_ref(), Some(&x));
    }

    /// The honest residual is COMPLETE under multiplicity perturbation, for
    /// every non-degenerate input: output invariant, residual responds, and the
    /// perturbed input round-trips.
    #[test]
    fn honest_probe_is_complete(x in tx_substantial()) {
        let pr = probe(&Aggregate, &Split, &x).unwrap();
        prop_assert!(pr.output_invariant, "summary moved under split: {:?}", x);
        prop_assert!(pr.residual_responds, "residual ignored the split: {:?}", x);
        prop_assert!(pr.round_trips, "honest residual failed to reconstruct: {:?}", x);
        prop_assert!(pr.residual_complete());
    }

    /// The probe CATCHES the count-only residual on every non-degenerate input:
    /// it cannot reconstruct, so it is never reported complete.
    #[test]
    fn buggy_probe_is_caught(x in tx_substantial()) {
        let pr = probe(&AggregateDropsAmounts, &Split, &x).unwrap();
        prop_assert!(!pr.round_trips, "count-only residual unexpectedly round-tripped: {:?}", x);
        prop_assert!(!pr.residual_complete());
    }
}
