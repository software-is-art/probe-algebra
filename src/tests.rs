//! Tests for the boundary algebra. These import ONLY through the boundaries,
//! exactly as another module would.

use crate::boundary::{probe, run, Compose, Morphism};
use crate::ledger::boundary::{
    Account, Aggregate, AggregateDropsAmounts, Balance, Cents, NudgeCents, Posting, Round, Split,
    Transaction,
};
use crate::linear::boundary::Quantity;

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
fn accessors_return_the_wrapped_value() {
    // The sanctioned exit hatches must report the real contents, not a constant.
    assert_eq!(Cents::new(7).unwrap().get(), 7);
    assert_eq!(Balance::zero().add_cents(Cents::new(5).unwrap()).get(), 5);
    assert_eq!(Account::new("Cash").unwrap().get(), "Cash");
    assert_eq!(Quantity::new(7).unwrap().get(), 7);
    // `is_zero` must track the value, not a fixed answer.
    assert!(Cents::zero().is_zero());
    assert!(!Cents::new(5).unwrap().is_zero());
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

// ===== nesting: a parent boundary composed of private child boundaries =====

mod nesting {
    use crate::boundary::{probe, run};
    use crate::pipeline::boundary::{Bucket, Ingest, NudgeSample, Sample};

    // The child intermediate is UNREACHABLE from here: naming
    // `crate::pipeline::calibrate::boundary::Reading` does not compile, because
    // `calibrate` is a private child of `pipeline`. Only `pipeline::boundary` is
    // public — that is "one place to look", recursing.

    /// The parent operator round-trips through TWO nested child stages while
    /// exposing only its own types; the intermediate `Reading` never surfaces.
    #[test]
    fn nested_pipeline_round_trips() {
        let x = Sample::new(2345).unwrap();
        let carried = run(&Ingest, &x);
        // the output is the parent's own Bucket type (2345 -> reading 3345 -> 334)
        assert_eq!(carried.out(), &Bucket::new(334).unwrap());
        assert_eq!(carried.invert(&Ingest).as_ref(), Some(&x));
    }

    /// The SAME altitude-agnostic `probe` that tests leaf operators tests the
    /// parent composite: it perturbs the sub-ten dimension and checks the
    /// COMPOSITE residual `Pair<Unit, Leftover>` is complete.
    #[test]
    fn parent_probe_sees_the_composite_residual() {
        // 2345 -> reading 3345, leftover 5; +1 stays in bucket 334.
        let x = Sample::new(2345).unwrap();
        let pr = probe(&Ingest, &NudgeSample, &x).unwrap();
        assert!(
            pr.output_invariant,
            "the bucket must not flip under a sub-ten nudge"
        );
        assert!(
            pr.residual_responds,
            "the composite residual records the nudge"
        );
        assert!(
            pr.round_trips,
            "the composite residual reconstructs the sample"
        );
        assert!(pr.residual_complete());
    }
}
