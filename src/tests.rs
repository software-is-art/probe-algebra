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

// ===== state: a stateful update is a lossy morphism (residual = prior) =====

mod state {
    use crate::boundary::{probe, run, Compose, Morphism};
    use crate::journal::boundary::{Add, NudgeState, Register, Set, SetForgetsPrior};

    fn reg(n: i64) -> Register {
        Register::new(n).unwrap()
    }

    /// A `Set` overwrites the state; its residual is exactly the prior it forgot,
    /// so retaining it rewinds the update.
    #[test]
    fn set_residual_is_the_overwritten_prior() {
        let prior = reg(42);
        let carried = run(&Set::to(reg(10)), &prior);
        assert_eq!(carried.out(), &reg(10), "the new state is the set value");
        assert_eq!(
            carried.residual(),
            &prior,
            "the residual is the prior state"
        );
        assert_eq!(carried.invert(&Set::to(reg(10))).as_ref(), Some(&prior));
    }

    /// The forgetful `Set` cannot rewind — the probe catches it where the honest
    /// `Set` is complete. The state analogue of the count-only residual bug.
    #[test]
    fn forgetful_set_is_caught_by_the_probe() {
        let s = reg(42);
        assert!(probe(&Set::to(reg(10)), &NudgeState, &s)
            .unwrap()
            .residual_complete());
        let buggy = probe(&SetForgetsPrior::to(reg(10)), &NudgeState, &s).unwrap();
        assert!(!buggy.residual_responds, "the residual ignores the prior");
        assert!(!buggy.round_trips, "so it cannot rewind");
        assert!(!buggy.residual_complete());
    }

    /// Composing updates threads the state and accumulates the priors into a
    /// nested residual — the undo history, built by the same `Compose`.
    #[test]
    fn composed_updates_build_an_undo_history() {
        let start = reg(7);
        // Set to 10, then Add 5: state 7 -> 10 -> 15.
        let history = Compose {
            f: Set::to(reg(10)),
            g: Add::by(reg(5)),
        };
        let (out, residual) = history.forward(&start);
        assert_eq!(out, reg(15));
        // residual is Pair<prior-of-Set = 7, Unit-of-Add>; it rewinds to the start.
        assert_eq!(residual.0, start);
        assert_eq!(history.backward(&out, &residual).as_ref(), Some(&start));
    }

    /// Discarding the state residual removes the ability to rewind — at compile
    /// time. `discarded.invert(..)` would not compile (the state is unrecoverable).
    #[test]
    fn discarding_the_prior_removes_rewind() {
        let prior = reg(42);
        let carried = run(&Set::to(reg(10)), &prior);
        let discarded = carried.discard();
        assert_eq!(discarded.out(), &reg(10)); // current state still readable
    }
}

// ===== effect: a morphism made pure relative to a handler ==================

mod effect {
    use crate::boundary::{Morphism, Pair};
    use crate::effect::boundary::{Clock, Demand, Entry, Message, Stamp, Stamped};
    use crate::effect::handler::{run_stamp, RecordingHandler};

    /// Pure relative to a handler: the recording handler scripts the reading and
    /// captures the emission; the operator round-trips like any pure morphism.
    #[test]
    fn pure_relative_to_a_handler_round_trips() {
        let mut handler = RecordingHandler::new(Clock::new(100).unwrap());
        let out = run_stamp(&mut handler, &Demand, &Message::new(5).unwrap());
        assert_eq!(out, Stamped::new(105).unwrap());
        // the WRITE is captured as data, not performed
        assert_eq!(handler.written(), &[Entry::new(100).unwrap()]);
        // the pure morphism inverts: (output, emission) -> (message, reading)
        let recovered = Stamp.backward(&out, &Entry::new(100).unwrap());
        assert_eq!(
            recovered,
            Some(Pair(Message::new(5).unwrap(), Clock::new(100).unwrap()))
        );
    }

    /// The output depends on the WORLD reading — the defining mark of a read
    /// effect. Two scripted clocks give two outputs; each handler is deterministic.
    #[test]
    fn output_responds_to_the_world_reading() {
        let msg = Message::new(5).unwrap();
        let mut early = RecordingHandler::new(Clock::new(100).unwrap());
        let mut late = RecordingHandler::new(Clock::new(200).unwrap());
        let a = run_stamp(&mut early, &Demand, &msg);
        let b = run_stamp(&mut late, &Demand, &msg);
        assert_ne!(a, b, "the output reflects the world reading");
        assert_eq!(a, Stamped::new(105).unwrap());
        assert_eq!(b, Stamped::new(205).unwrap());
        // deterministic relative to a fixed handler (unlike a live clock)
        let mut again = RecordingHandler::new(Clock::new(100).unwrap());
        assert_eq!(run_stamp(&mut again, &Demand, &msg), a);
    }
}
