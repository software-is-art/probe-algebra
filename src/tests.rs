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
    use crate::boundary::{probe, run, Perturbation};
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

    /// The sample nudge moves the input by exactly one unit.
    #[test]
    fn nudge_moves_the_sample_by_one() {
        let nudged = Perturbation::<Ingest>::perturb(&NudgeSample, &Sample::new(2345).unwrap());
        assert_eq!(nudged, Some(Sample::new(2346).unwrap()));
    }
}

// ===== state: a stateful update is a lossy morphism (residual = prior) =====

mod state {
    use crate::boundary::{probe, run, Compose, Morphism};
    use crate::journal::boundary::{Add, NudgeState, Register, Set, SetForgetsPrior};

    fn reg(n: i64) -> Register {
        Register::new(n).unwrap()
    }

    /// The register accessor reports the real value, and `Add` is invertible on
    /// its own (the composed test can't see this — `Set::backward` discards it).
    #[test]
    fn add_round_trips_and_register_reads_back() {
        assert_eq!(reg(7).get(), 7);
        let carried = run(&Add::by(reg(5)), &reg(7));
        assert_eq!(carried.out(), &reg(12));
        assert_eq!(carried.invert(&Add::by(reg(5))).as_ref(), Some(&reg(7)));
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
        // the bug rewinds to the WRONG value, not to None
        let op = SetForgetsPrior::to(reg(10));
        let (out, residual) = op.forward(&s);
        let recovered = op.backward(&out, &residual);
        assert!(recovered.is_some());
        assert_ne!(recovered, Some(s));
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
    use crate::boundary::{Morphism, Pair, Perturbation};
    use crate::effect::boundary::{
        Clock, Demand, Entry, IgnoresClock, Message, NudgeReading, Stamp, Stamped,
    };
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

    /// `NudgeReading` moves the world reading by exactly one tick (and only the
    /// reading); the over-declared `IgnoresClock` still reconstructs something.
    #[test]
    fn perturbation_and_degenerate_operator_are_pinned() {
        let env = Pair(Message::new(5).unwrap(), Clock::new(100).unwrap());
        let nudged = Perturbation::<Stamp>::perturb(&NudgeReading, &env).unwrap();
        assert_eq!(
            nudged,
            Pair(Message::new(5).unwrap(), Clock::new(101).unwrap())
        );
        assert!(IgnoresClock
            .backward(&Stamped::new(5).unwrap(), &Entry::new(0).unwrap())
            .is_some());
    }
}

// ===== instrumentation: causal-profiling seam over the morphism ===========

mod instrumentation {
    use crate::boundary::{probe, run, Compose, Meter, Morphism, Profiled};
    use crate::ledger::boundary::{Aggregate, Round, Split};
    use std::cell::RefCell;

    /// A test `Meter` that records the labels it sees (a Coz adapter would instead
    /// open coz scopes / mark coz progress points).
    struct RecordingMeter {
        log: RefCell<Vec<String>>,
    }
    impl Meter for RecordingMeter {
        fn measured<R>(&self, label: &'static str, body: impl FnOnce() -> R) -> R {
            self.log.borrow_mut().push(format!("scope:{label}"));
            body()
        }
        fn progress(&self, label: &'static str) {
            self.log.borrow_mut().push(format!("progress:{label}"));
        }
    }

    /// `NoMeter` is transparent: a `Profiled` morphism behaves exactly like the
    /// bare one — round-trips and probes identically. Instrumentation is free and
    /// invisible when off.
    #[test]
    fn no_meter_is_transparent() {
        let x = super::sample();
        let profiled = Profiled::new(Aggregate);
        assert_eq!(run(&profiled, &x).invert(&profiled).as_ref(), Some(&x));
        assert!(probe(&profiled, &Split, &x).unwrap().residual_complete());
    }

    /// One wrapper instruments a whole composed dataflow: each stage's `forward`
    /// becomes a scope labelled by its TYPE — the annotation points are determined
    /// by the algebra, not hand-picked. (The usual Coz question, "what do I
    /// annotate?", answered by the morphism boundary.)
    #[test]
    fn metering_records_a_scope_per_stage_labelled_by_type() {
        let x = super::sample();
        let meter = RecordingMeter {
            log: RefCell::new(Vec::new()),
        };
        let pipeline = Compose {
            f: Profiled::metered(Aggregate, &meter),
            g: Profiled::metered(Round, &meter),
        };
        let (_out, _res) = pipeline.forward(&x);

        let log = meter.log.borrow();
        let has = |kind: &str, op: &str| log.iter().any(|e| e.starts_with(kind) && e.contains(op));
        // each stage records a scope AND a per-stage progress point, both labelled
        // by the operator's type — no hand-placed annotations.
        assert!(has("scope:", "Aggregate"));
        assert!(has("scope:", "Round"));
        assert!(has("progress:", "Aggregate"));
        assert!(has("progress:", "Round"));
    }
}

/// Generality check for the `StateMachine` grammar: a SECOND machine over a
/// different carrier and payload reuses `transition!` to get reversible edges with
/// no hand-written `Morphism`. If the abstraction were secretly tied to the ledger
/// `Entry`, this would not compile — proving it before client code relies on it.
mod state_machine {
    use crate::boundary::{run, Morphism};
    use crate::ledger::boundary::Cents;
    use core::marker::PhantomData;

    // states of an unrelated little protocol
    pub struct Lo;
    pub struct Hi;
    crate::typestate!(Lo, Hi);

    // A DIFFERENT carrier (a `Cents` reading indexed by a phantom level) and its
    // descriptor, declared by the SAME grammar macro that `Entry`/`EntryFlow` use —
    // proving `state_machine!` is not tied to the ledger lifecycle. The whole carrier
    // (value-object impls + `StateMachine`) is one line; the module adds only its own
    // entry constructor and accessor.
    crate::state_machine!(GaugeFlow, Gauge, Cents);
    impl Gauge<Lo> {
        fn new(reading: Cents) -> Self {
            Gauge(reading, PhantomData)
        }
    }
    impl<S> Gauge<S> {
        fn reading(&self) -> Cents {
            self.0
        }
    }
    crate::transition!(Raise: GaugeFlow, Lo => Hi);
    crate::transition!(Lower: GaugeFlow, Hi => Lo);

    /// A different carrier (`Gauge` over `Cents`) gets reversible transitions purely
    /// from the grammar: the payload survives the retag, and the move inverts.
    #[test]
    fn a_second_machine_reuses_the_grammar() {
        let lo = Gauge::<Lo>::new(Cents::new(42).unwrap());
        let raised = run(&Raise, &lo);
        assert_eq!(raised.out().reading(), Cents::new(42).unwrap()); // payload preserved
        assert_eq!(raised.invert(&Raise).unwrap(), lo); // reversible
                                                        // `Lower` is the opposite edge; `Raise.forward(&hi)` would not type-check.
        let hi = Raise.forward(&lo).0;
        assert_eq!(Lower.forward(&hi).0, lo);
    }
}

// ===== construction: the smart constructor as the ENTRY morphism ===========

/// Construction is the one edge the value-object pattern left OUTSIDE the probe
/// space — a native `fn new`. Modelled as a `Construction` (the entry morphism
/// `Raw -> (Refined, Residual)`), it is back inside it: the parse/`reconstruct`
/// round-trip is probed exactly like a `Morphism`'s residual, and `new` now routes
/// through the same parse the probe certifies.
mod construction {
    use crate::boundary::{
        construction_probe, reconstructs, Capability, Construction, Morphism, Then, Unit,
    };
    use crate::ledger::boundary::{
        Account, Aggregate, Cents, PadName, ParseAccount, ParseAccountDropsPadding, ParseCents,
        ParseTransaction, Posting, ReorderPostings, Transaction,
    };

    /// Three unbalanced-but-valid postings whose INPUT order differs from canonical
    /// (sorted) order, so the permutation residual is non-trivial.
    fn unsorted() -> Vec<Posting> {
        vec![
            Posting::new(Account::new("Revenue").unwrap(), Cents::new(-100).unwrap()),
            Posting::new(Account::new("Cash").unwrap(), Cents::new(60).unwrap()),
            Posting::new(Account::new("Cash").unwrap(), Cents::new(40).unwrap()),
        ]
    }

    /// A PURE refinement: `ParseCents` only range-checks, so its residual is `Unit`
    /// and it reconstructs the exact integer. `new` is the same parse, residual dropped.
    #[test]
    fn parse_cents_is_a_pure_refinement() {
        let (refined, residual) = ParseCents.parse(&7).unwrap();
        assert_eq!(residual, Unit);
        assert_eq!(refined, Cents::new(7).unwrap()); // `new` routes through this parse
        assert_eq!(reconstructs(&ParseCents, &7), Some(true));
        // out of range: rejected, no round-trip obligation
        assert_eq!(reconstructs(&ParseCents, &i64::MAX), None);
        assert_eq!(Cents::new(i64::MAX), None);
    }

    /// A NORMALIZING parse: `ParseAccount` trims, and the residual is the leading/
    /// trailing whitespace it removed — so it reconstructs the padded original.
    #[test]
    fn parse_account_residual_recovers_the_padding() {
        let raw = " Cash ".to_string();
        let (refined, residual) = ParseAccount.parse(&raw).unwrap();
        assert_eq!(refined, Account::new("Cash").unwrap());
        assert_eq!(residual.0.get(), " "); // leading
        assert_eq!(residual.1.get(), " "); // trailing
        assert_eq!(
            ParseAccount.reconstruct(&refined, &residual),
            Some(raw.clone())
        );
        assert_eq!(reconstructs(&ParseAccount, &raw), Some(true));
        // a non-padded name still round-trips (empty affixes)...
        assert_eq!(reconstructs(&ParseAccount, &"Cash".to_string()), Some(true));
        // ...and both edges of the guard reject: all-whitespace and the empty string
        // (the `start >= end` boundary — empty is the lone `start == end` case).
        assert_eq!(reconstructs(&ParseAccount, &"   ".to_string()), None);
        assert_eq!(reconstructs(&ParseAccount, &String::new()), None);
    }

    /// The buggy twin claims a `Unit` residual but actually normalizes — so it cannot
    /// rebuild the padding, and the round-trip probe catches it on any padded input.
    /// (The entry-edge analog of `AggregateDropsAmounts`.)
    #[test]
    fn dropping_the_padding_residual_is_caught() {
        // type-identical to a pure refinement, yet it loses the padding:
        assert_eq!(
            reconstructs(&ParseAccountDropsPadding, &" Cash ".to_string()),
            Some(false)
        );
        // on an UNpadded input there is nothing to lose, so it round-trips — exactly
        // why a probe needs an input that exercises the dropped dimension.
        assert_eq!(
            reconstructs(&ParseAccountDropsPadding, &"Cash".to_string()),
            Some(true)
        );
        // it still rejects the empty string (pins its own emptiness guard).
        assert_eq!(
            reconstructs(&ParseAccountDropsPadding, &String::new()),
            None
        );
    }

    /// A NORMALIZING parse: `ParseTransaction` sorts into canonical order, and the
    /// residual is the permutation it discarded — so it restores the exact input Vec.
    #[test]
    fn parse_transaction_residual_recovers_the_order() {
        let raw = unsorted();
        let (refined, residual) = ParseTransaction.parse(&raw).unwrap();
        // the refined value is canonically sorted (so value-equality is stable)...
        assert_eq!(refined, Transaction::new(raw.clone()).unwrap());
        assert_ne!(refined.postings(), raw.as_slice()); // ...and order really changed
        assert_eq!(
            ParseTransaction.reconstruct(&refined, &residual),
            Some(raw.clone())
        );
        assert_eq!(reconstructs(&ParseTransaction, &raw), Some(true));
        // empty is rejected
        assert_eq!(reconstructs(&ParseTransaction, &Vec::new()), None);
    }

    /// The category claim, made concrete: a construction COMPOSES with a `Morphism`
    /// into a single construction. `ParseTransaction` then `Aggregate` is one edge
    /// from a raw `Vec<Posting>` to an `AccountSummary`, and the PRODUCT residual
    /// (`Pair<PostingOrder, MultiplicityResidual>`) reconstructs the raw end-to-end.
    #[test]
    fn construction_composes_with_a_morphism() {
        let pipeline = Then {
            construct: ParseTransaction,
            then: Aggregate,
        };
        let raw = unsorted();
        let (summary, residual) = pipeline.parse(&raw).unwrap();
        // the output is the morphism's own type, reached straight from the primitive
        assert_eq!(
            summary,
            Aggregate.forward(&Transaction::new(raw.clone()).unwrap()).0
        );
        // and the whole primitive -> summary path inverts through the paired residual
        assert_eq!(pipeline.reconstruct(&summary, &residual), Some(raw.clone()));
        assert_eq!(reconstructs(&pipeline, &raw), Some(true));
    }

    /// Construction now declares a capability just like a `Morphism`: a pure
    /// refinement is `Pure`, a normalizing parse is `Lossy`, and `Then` JOINS them, so
    /// a raw -> summary path's ceiling is computed by the type system.
    #[test]
    fn capability_is_declared_and_joins_through_then() {
        assert_eq!(ParseCents::CAPABILITY, Capability::Pure);
        assert_eq!(ParseAccount::CAPABILITY, Capability::Lossy);
        assert_eq!(ParseTransaction::CAPABILITY, Capability::Lossy);
        // Lossy parse joined with a Lossy morphism is still Lossy (the static join).
        type Pipe = Then<ParseTransaction, Aggregate>;
        assert_eq!(<Pipe as Construction>::CAPABILITY, Capability::Lossy);
    }

    /// The construction COMPLETENESS probe (entry-edge analog of `probe`): perturb the
    /// padding that `ParseAccount` normalizes away — the `Account` stays invariant, the
    /// residual responds, and the perturbed raw round-trips, so the residual is complete.
    #[test]
    fn account_residual_is_complete_under_padding() {
        let raw = "Cash".to_string();
        let pr = construction_probe(&ParseAccount, &PadName, &raw).unwrap();
        assert!(pr.output_invariant, "trimming must normalize the pad away");
        assert!(pr.residual_responds, "the affix residual records the pad");
        assert!(pr.round_trips, "the padded raw reconstructs");
        assert!(pr.residual_complete());
    }

    /// The SAME probe catches the lying parse: a `Unit` residual cannot respond to the
    /// padding nor reconstruct it, so completeness fails — exactly as `probe` flags
    /// `AggregateDropsAmounts`.
    #[test]
    fn dropped_padding_is_caught_by_the_completeness_probe() {
        let raw = "Cash".to_string();
        let pr = construction_probe(&ParseAccountDropsPadding, &PadName, &raw).unwrap();
        assert!(
            !pr.residual_responds,
            "a Unit residual cannot record the pad"
        );
        assert!(!pr.round_trips);
        assert!(!pr.residual_complete());
    }

    /// `ParseTransaction`'s permutation residual is complete under a reordering of the
    /// raw postings: sorting makes the `Transaction` invariant, the permutation
    /// responds, and the reordered raw round-trips.
    #[test]
    fn transaction_residual_is_complete_under_reordering() {
        let raw = unsorted();
        let pr = construction_probe(&ParseTransaction, &ReorderPostings, &raw).unwrap();
        assert!(
            pr.output_invariant,
            "sorting normalizes the reordering away"
        );
        assert!(
            pr.residual_responds,
            "the permutation records the reordering"
        );
        assert!(pr.round_trips);
        assert!(pr.residual_complete());
        // a TWO-posting reorder still applies and is complete (pins the `len < 2`
        // boundary: a single posting yields `None`, a pair does not).
        let pair = raw[..2].to_vec();
        assert!(
            construction_probe(&ParseTransaction, &ReorderPostings, &pair)
                .unwrap()
                .residual_complete()
        );
        // the perturbation does not apply to a single posting (nothing to reorder)
        assert_eq!(
            construction_probe(&ParseTransaction, &ReorderPostings, &raw[..1].to_vec()),
            None
        );
    }
}

// ===== gradings: one monoid pattern at three levels =====
//
// Residual and capability are OPERATOR gradings, composed by `Compose`. Provenance is
// the VALUE's journey, now type-level, accumulated by `stamp_through`.

mod grading {
    use super::sample;
    use crate::boundary::{
        stamp_through, Capability, Compose, Monoid, Morphism, Origin, Provenance, Stamped, Step,
    };
    use crate::ledger::boundary::{AccountSummary, Aggregate, Round};

    /// `Provenance` obeys the monoid laws: `EMPTY` is the unit, `combine` concatenates.
    /// (It is the runtime reflection target of the type-level lineage.)
    #[test]
    fn provenance_is_a_monoid() {
        assert_eq!(Provenance::EMPTY.steps().len(), 0);
        let p = Provenance::single("a").combine(Provenance::single("b"));
        assert_eq!(p.steps(), &["a", "b"]);
        assert_eq!(
            Provenance::single("a").combine(Provenance::EMPTY),
            Provenance::single("a")
        );
        assert_eq!(
            Provenance::EMPTY.combine(Provenance::single("a")),
            Provenance::single("a")
        );
    }

    /// `Capability` is the same genus at the const level: `Pure` is the unit, `combine`
    /// is the join.
    #[test]
    fn capability_is_a_monoid() {
        assert_eq!(<Capability as Monoid>::EMPTY, Capability::Pure);
        assert_eq!(
            Capability::Lossy.combine(Capability::Pure),
            Capability::Lossy
        );
        assert_eq!(
            Capability::Pure.combine(Capability::Effectful),
            Capability::Effectful
        );
    }

    /// `Compose` threads the two OPERATOR gradings: the const-level capability (a static
    /// join) and the type-level residual (the composite type-checks, so its `Residual`
    /// is `Pair<MultiplicityResidual, RoundingResidual>`).
    #[test]
    fn compose_threads_the_operator_gradings() {
        assert_eq!(
            <Compose<Aggregate, Round> as Morphism>::CAPABILITY,
            Capability::Lossy
        );
    }

    /// PROVENANCE is now type-level on the VALUE: `stamp_through` extends the value's
    /// lineage TYPE by each edge crossed, so the type records the whole journey. It is
    /// reflectable to the runtime `Provenance` (oldest edge first), and a consumer can
    /// DEMAND a specific path — a value with the wrong history will not compile.
    #[test]
    fn stamping_records_the_lineage_in_the_type() {
        let at_origin = Stamped::origin(sample()); // Stamped<Origin, Transaction>
        let summarized = stamp_through(&at_origin, &Aggregate); // + Step<Aggregate, _>
        let rounded = stamp_through(&summarized, &Round); // + Step<Round, _>

        // the value is the real morphism output...
        assert_eq!(rounded.value(), &Round.forward(summarized.value()).0);
        // ...and its TYPE-level lineage reflects to the runtime path, oldest first
        let prov = rounded.lineage();
        let steps = prov.steps();
        assert_eq!(steps.len(), 2);
        assert!(steps[0].contains("Aggregate"), "first: {}", steps[0]);
        assert!(steps[1].contains("Round"), "second: {}", steps[1]);

        // a compile-time provenance contract: `audited` only accepts a value whose TYPE
        // proves it went Aggregate THEN Round — a different history would not unify.
        audited(&rounded);
    }

    /// Demands the exact provenance in the type — `Aggregate` then `Round`.
    fn audited(_x: &Stamped<Step<Round, Step<Aggregate, Origin>>, AccountSummary>) {}

    /// CAPABILITY is now type-level too: a consumer can DEMAND a ceiling as a bound
    /// (`run_pure` ⇒ `M::Capability: AtMost<Pure>`), so the type system rejects a too-
    /// capable edge at the call site. `Scale` is `Pure`, so it satisfies the bound and
    /// runs; a `Lossy` edge would not compile (pinned in `tests/compile_fail`).
    #[test]
    fn an_effect_ceiling_is_demandable_as_a_bound() {
        use crate::boundary::run_pure;
        use crate::linear::boundary::{Quantity, Scale};

        let q = Quantity::new(2).expect("2 is a valid quantity");
        // `Scale: Morphism<Capability = Pure>`, so `AtMost<Pure>` holds and this type-
        // checks; the result is the ordinary forward output.
        let (out, _unit) = run_pure(&Scale::honest(), &q);
        assert_eq!(out, Scale::honest().forward(&q).0);
    }
}

/// interp — boundary-only tests for the cold use case. EVERY test here exercises a
/// PUBLIC boundary edge (`Parse`/`Check`/`Eval`); `interp::internal` has no tests of its
/// own, so these (plus the autogen `Parse` round-trip law in `laws.rs`) are the entire
/// rigour the internals get. Mutation testing of `interp/internal.rs` quantifies how
/// much that buys.
mod interp {
    use crate::gdp::with_seed;
    use crate::interp::boundary::{Check, Eval, Expr, Ident, Int, Op, Parse, Value};

    fn name(s: &str) -> Ident {
        Ident::new(s).unwrap()
    }
    fn int(v: i64) -> Expr {
        Expr::int(v).unwrap()
    }
    fn num(v: i64) -> Value {
        Value::Int(Int::new(v).unwrap())
    }

    /// Parse → Check → Eval through the boundary, returning the value. Panics if the
    /// program is ill-typed (the test feeds only well-typed programs).
    fn eval(expr: Expr) -> Value {
        with_seed(|seed| {
            let named = seed.new_named(expr);
            let proof = Check.classify(&named).expect("well-typed");
            *Eval.run(&named, &proof).value()
        })
    }

    /// Does the named expression type-check (the `Branch`'s positive arm)?
    fn well_typed(expr: Expr) -> bool {
        with_seed(|seed| {
            let named = seed.new_named(expr);
            Check.classify(&named).is_ok()
        })
    }

    #[test]
    fn evaluates_arithmetic() {
        assert_eq!(eval(int(2)), num(2));
        assert_eq!(eval(Expr::bin(Op::Add, int(2), int(3))), num(5));
        assert_eq!(eval(Expr::bin(Op::Mul, int(2), int(3))), num(6));
    }

    #[test]
    fn evaluates_comparison_and_if() {
        assert_eq!(eval(Expr::bin(Op::Lt, int(1), int(2))), Value::Bool(true));
        assert_eq!(eval(Expr::bin(Op::Lt, int(2), int(1))), Value::Bool(false));
        // EQUAL operands pin `<` against `<=` (strict less-than).
        assert_eq!(eval(Expr::bin(Op::Lt, int(2), int(2))), Value::Bool(false));
        let cond = Expr::bin(Op::Lt, int(1), int(2));
        assert_eq!(eval(Expr::cond(cond.clone(), int(7), int(9))), num(7));
        let cond_false = Expr::bin(Op::Lt, int(2), int(1));
        assert_eq!(eval(Expr::cond(cond_false, int(7), int(9))), num(9));
    }

    /// The value-object accessors and the canonical `render` report the REAL contents,
    /// not a constant. (`render` is pinned here directly because the round-trip law's
    /// generator is itself built from `render`, so a `render`-to-constant mutation would
    /// mask itself in that law — these assertions are the independent oracle.)
    #[test]
    fn accessors_and_render_report_the_real_value() {
        assert_eq!(Int::new(7).unwrap().get(), 7);
        assert_eq!(name("foo").get(), "foo");
        assert_eq!(Expr::bin(Op::Add, int(1), int(2)).render(), "(1 + 2)");
        assert_eq!(Expr::bin(Op::Mul, int(2), int(3)).render(), "(2 * 3)");
        assert_eq!(Expr::bin(Op::Lt, int(1), int(2)).render(), "(1 < 2)");
        assert_eq!(
            Expr::cond(Expr::boolean(true), int(1), int(2)).render(),
            "(if true then 1 else 2)"
        );
        assert_eq!(
            Expr::bind(name("x"), int(1), Expr::var(name("x"))).render(),
            "(let x = 1 in x)"
        );
    }

    #[test]
    fn evaluates_let_and_var() {
        // let x = 4 in (x + 1) => 5
        let prog = Expr::bind(
            name("x"),
            int(4),
            Expr::bin(Op::Add, Expr::var(name("x")), int(1)),
        );
        assert_eq!(eval(prog), num(5));
        // shadowing: let x = 1 in (let x = 2 in x) => 2
        let shadow = Expr::bind(
            name("x"),
            int(1),
            Expr::bind(name("x"), int(2), Expr::var(name("x"))),
        );
        assert_eq!(eval(shadow), num(2));
        // let x = 3 in (x * x) => 9
        let square = Expr::bind(
            name("x"),
            int(3),
            Expr::bin(Op::Mul, Expr::var(name("x")), Expr::var(name("x"))),
        );
        assert_eq!(eval(square), num(9));
    }

    #[test]
    fn accepts_well_typed() {
        assert!(well_typed(Expr::bin(Op::Add, int(1), int(2))));
        assert!(well_typed(Expr::bin(Op::Lt, int(1), int(2))));
        let if_ok = Expr::cond(Expr::bin(Op::Lt, int(1), int(2)), int(1), int(2));
        assert!(well_typed(if_ok));
        let let_ok = Expr::bind(
            name("x"),
            int(1),
            Expr::bin(Op::Add, Expr::var(name("x")), int(1)),
        );
        assert!(well_typed(let_ok));
    }

    #[test]
    fn rejects_ill_typed() {
        // a type mismatch in each operand position, each operator, the if condition and
        // its branches, an unbound variable, and a let-bound variable of the wrong type
        assert!(!well_typed(Expr::bin(Op::Add, int(1), Expr::boolean(true))));
        assert!(!well_typed(Expr::bin(Op::Add, Expr::boolean(true), int(1))));
        assert!(!well_typed(Expr::bin(Op::Mul, int(1), Expr::boolean(true))));
        assert!(!well_typed(Expr::bin(Op::Lt, int(1), Expr::boolean(true))));
        assert!(!well_typed(Expr::cond(int(1), int(2), int(3))));
        assert!(!well_typed(Expr::cond(
            Expr::bin(Op::Lt, int(1), int(2)),
            int(1),
            Expr::boolean(true)
        )));
        assert!(!well_typed(Expr::var(name("y"))));
        let bad_let = Expr::bind(
            name("x"),
            Expr::boolean(true),
            Expr::bin(Op::Add, Expr::var(name("x")), int(1)),
        );
        assert!(!well_typed(bad_let));
    }

    #[test]
    fn parses_and_runs_a_program() {
        let e = Parse
            .parse_str("(let x = 5 in (if (x < 10) then (x + 1) else 0))")
            .unwrap();
        assert_eq!(eval(e), num(6));
    }

    #[test]
    fn parse_builds_the_expected_structure() {
        assert_eq!(
            Parse.parse_str("(1 + (2 * 3))").unwrap(),
            Expr::bin(Op::Add, int(1), Expr::bin(Op::Mul, int(2), int(3)))
        );
        assert_eq!(Parse.parse_str("true").unwrap(), Expr::boolean(true));
        assert_eq!(
            Parse.parse_str("(x < y)").unwrap(),
            Expr::bin(Op::Lt, Expr::var(name("x")), Expr::var(name("y")))
        );
    }

    #[test]
    fn rejects_malformed_source() {
        // incomplete, unbalanced, trailing tokens, bad char, empty parens, no operator
        assert!(Parse.parse_str("1 +").is_none());
        assert!(Parse.parse_str("(1 + 2").is_none());
        assert!(Parse.parse_str("(1 + 2) extra").is_none());
        assert!(Parse.parse_str("@").is_none());
        assert!(Parse.parse_str("()").is_none());
        assert!(Parse.parse_str("(1 2 3)").is_none());
    }

    /// The headline: the brand minted at `with_seed` threads through parse, the `Check`
    /// branch, and the `Eval` guard. Only `Check` can mint the `WellTyped` witness `Eval`
    /// demands, and the shared name ties them to THIS program. (A proof for another
    /// program will not type-check — pinned in `tests/compile_fail/eval_wrong_program`.)
    #[test]
    fn the_brand_threads_parse_check_eval() {
        with_seed(|seed| {
            let named = seed.new_named(Parse.parse_str("(2 + 3)").unwrap());
            match Check.classify(&named) {
                Ok(proof) => {
                    let result = Eval.run(&named, &proof);
                    assert_eq!(result.value(), &num(5));
                }
                Err(_) => panic!("(2 + 3) is well-typed"),
            }
        });
    }
}

/// cost — the time/space budget grading. The type level checks costs COMPOSE within
/// budget (sequential = max, iteration = multiply, collect-vs-fold splits space); `fits`
/// audits a declared class against measured growth.
mod cost {
    use crate::boundary::{
        fits, require_within_space, require_within_time, BigO, Compose, Costed, Fold, Linear,
        MapCollect, Quadratic,
    };
    use crate::ledger::boundary::{Aggregate, Round};

    /// Sequential composition takes the MAX on both axes: aggregate (O(n)/O(n)) then
    /// round (O(n)/O(1)) is O(n) time and O(n) space, and stays within an O(n) budget.
    #[test]
    fn compose_takes_the_sequential_max() {
        assert_eq!(<Compose<Aggregate, Round> as Costed>::TIME, BigO::Linear);
        assert_eq!(<Compose<Aggregate, Round> as Costed>::SPACE, BigO::Linear);
        let pipeline = Compose {
            f: Aggregate,
            g: Round,
        };
        require_within_time::<Linear, _>(&pipeline);
        require_within_space::<Linear, _>(&pipeline);
    }

    /// Iteration multiplies time on both combinators, but space splits: `MapCollect`
    /// materializes n results (quadratic space) while `Fold` streams (space stays linear).
    /// This is the type-level "stream, don't materialize" an agent can be held to.
    #[test]
    fn iteration_multiplies_time_collect_vs_fold_splits_space() {
        assert_eq!(<MapCollect<Aggregate> as Costed>::TIME, BigO::Quadratic);
        assert_eq!(<MapCollect<Aggregate> as Costed>::SPACE, BigO::Quadratic);
        assert_eq!(<Fold<Aggregate> as Costed>::TIME, BigO::Quadratic);
        assert_eq!(<Fold<Aggregate> as Costed>::SPACE, BigO::Linear);
        require_within_time::<Quadratic, _>(&MapCollect(Aggregate));
        // the fold keeps space within linear; the collect would NOT compile here
        // (pinned in tests/compile_fail/cost_over_budget).
        require_within_space::<Linear, _>(&Fold(Aggregate));
    }

    /// The honesty audit: `fits` measures work growth and catches a declared class that
    /// does not match reality — a `Linear`-declared edge that is secretly quadratic.
    #[test]
    fn fits_audits_declared_against_measured_growth() {
        let constant = |_n: usize| 1u64;
        let linear = |n: usize| n as u64;
        let quadratic = |n: usize| (n as u64) * (n as u64);
        let cubic = |n: usize| (n as u64).pow(3);
        let exponential = |n: usize| 1u64 << n.min(40); // capped to avoid overflow
                                                        // each class fits its own declared bound...
        assert!(fits(constant, BigO::Constant));
        assert!(fits(linear, BigO::Linear));
        assert!(fits(quadratic, BigO::Quadratic));
        assert!(fits(cubic, BigO::Cubic));
        assert!(fits(exponential, BigO::Exponential));
        // ...and a class declared too LOW is caught (the honesty check at every rung):
        assert!(
            !fits(linear, BigO::Constant),
            "a linear edge is not constant"
        );
        assert!(
            !fits(quadratic, BigO::Linear),
            "a quadratic edge is not linear"
        );
        assert!(
            !fits(cubic, BigO::Quadratic),
            "a cubic edge is not quadratic"
        );
    }
}
