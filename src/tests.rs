//! Tests for the boundary algebra. These import ONLY through the boundaries,
//! exactly as another module would.

use crate::boundary::{probe, run, Compose, Morphism};
use crate::ledger::boundary::{
    Account, Aggregate, AggregateDropsAmounts, Cents, NudgeCents, Posting, Round, Split,
    Transaction,
};

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

#[test]
fn value_object_validation_rejects_bad_input() {
    assert!(Account::new("   ").is_none());
    assert!(Cents::new(i64::MAX).is_none());
    assert!(Transaction::new(vec![]).is_none());
}

#[test]
fn honest_aggregate_round_trips() {
    let x = sample();
    let carried = run(&Aggregate, &x);
    assert_eq!(carried.invert(&Aggregate).as_ref(), Some(&x));
}

#[test]
fn honest_residual_is_complete_under_split() {
    let x = sample();
    let pr = probe(&Aggregate, &Split, &x).unwrap();
    assert!(pr.output_invariant, "aggregation is blind to multiplicity");
    assert!(pr.residual_responds, "residual records the new breakdown");
    assert!(
        pr.round_trips,
        "complete residual reconstructs the perturbed input"
    );
    assert!(pr.residual_complete());
}

#[test]
fn incomplete_residual_is_caught_by_probe() {
    let x = sample();
    let pr = probe(&AggregateDropsAmounts, &Split, &x).unwrap();
    // Same morphism type as Aggregate, but the count-only residual cannot
    // reconstruct, so the probe flags it incomplete.
    assert!(!pr.round_trips);
    assert!(!pr.residual_complete());
}

#[test]
fn round_residual_is_complete_under_nudge() {
    let x = sample();
    let summary = run(&Aggregate, &x).out().clone();
    let pr = probe(&Round, &NudgeCents, &summary).unwrap();
    assert!(pr.residual_complete());
}

#[test]
fn composition_round_trips_through_two_lossy_stages() {
    let x = sample();
    let pipeline = Compose {
        f: Aggregate,
        g: Round,
    };
    let (out, res) = pipeline.forward(&x);
    assert_eq!(pipeline.backward(&out, &res).as_ref(), Some(&x));
    // and the composite is itself probeable
    assert!(probe(&pipeline, &Split, &x).unwrap().residual_complete());
}

#[test]
fn discarding_residual_keeps_the_output() {
    let x = sample();
    let carried = run(&Aggregate, &x);
    let expected = carried.out().clone();
    let discarded = carried.discard();
    assert_eq!(discarded.out(), &expected);
    // discarded.invert(&Aggregate) would not compile — invert is not in scope
    // for Carried<_, Discarded>. That is the typestate guarantee.
}
