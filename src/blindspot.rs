//! blindspot.rs — the spec's BLIND-SPOT MAP as executable assertions.
//!
//! Each probe flavour catches a bug class and is blind to another; NO single
//! check is highest-assurance. These tests pin that down:
//!
//!   | bug                     | round-trip | commutation | quantitative |
//!   |-------------------------|------------|-------------|--------------|
//!   | residual incompleteness | CATCHES    | blind       | (n/a)        |
//!   | non-linear output offset | blind     | CATCHES     | (catches)    |
//!   | wrong-but-invertible    | blind      | blind       | CATCHES      |
//!     coefficient (Skew)
//!
//! The decisive negative result: the wrong-coefficient `Skew` survives BOTH
//! structural checks and dies only to the reference-bearing quantitative probe.

use crate::boundary::{coefficient_holds, commutes, probe, run};

// ===== linear transport: the decisive coefficient bug ====================

mod linear {
    use super::*;
    use crate::linear::boundary::{Double, Quantity, Scale, UnitResponse};

    fn q(n: i64) -> Quantity {
        Quantity::new(n).expect("in range")
    }

    /// BLIND #1: the wrong-coefficient transport still round-trips, because it
    /// inverts by its own rate. Round-trip cannot tell a wrong constant apart.
    #[test]
    fn skew_round_trips() {
        let x = q(7);
        let honest = run(&Scale::honest(), &x).invert(&Scale::honest());
        let skew = run(&Scale::skew(), &x).invert(&Scale::skew());
        assert_eq!(honest.as_ref(), Some(&x));
        assert_eq!(
            skew.as_ref(),
            Some(&x),
            "skew is wrong but still invertible"
        );
    }

    /// BLIND #2: the wrong coefficient commutes with doubling just as the honest
    /// one does — linearity holds for ANY rate, so commutation is blind to it.
    #[test]
    fn skew_commutes_with_doubling() {
        let x = q(7);
        assert_eq!(commutes(&Scale::honest(), &Double, &x), Some(true));
        assert_eq!(
            commutes(&Scale::skew(), &Double, &x),
            Some(true),
            "a uniform linear bug respects the scaling relation"
        );
    }

    /// CATCH: only the reference-bearing quantitative probe separates them.
    #[test]
    fn quantitative_probe_catches_skew() {
        let x = q(7);
        let unit_response = UnitResponse::from_reference(Scale::reference_rate());
        assert_eq!(
            coefficient_holds(&Scale::honest(), &unit_response, &x),
            Some(true)
        );
        assert_eq!(
            coefficient_holds(&Scale::skew(), &unit_response, &x),
            Some(false),
            "the quantitative probe pins the coefficient the structural checks missed"
        );
    }
}

// ===== ledger: commutation vs residual are complementary ==================

mod ledger {
    use super::*;
    use crate::ledger::boundary::{
        Account, Aggregate, AggregateDropsAmounts, AggregateOffsetsTotals, Cents, DoublePostings,
        Posting, Split, Transaction,
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

    /// The honest morphism is killed by NO probe (soundness: a probe that flags
    /// the correct implementation is unusable).
    #[test]
    fn honest_survives_every_probe() {
        let x = sample();
        assert_eq!(commutes(&Aggregate, &DoublePostings, &x), Some(true));
        assert!(probe(&Aggregate, &Split, &x).unwrap().residual_complete());
    }

    /// CATCH (commutation) / BLIND (residual): the additive-offset bug breaks
    /// linearity — commutation catches it — but its residual is intact, so the
    /// round-trip rebuilds the input and the residual probe reports complete.
    #[test]
    fn offset_bug_caught_only_by_commutation() {
        let x = sample();
        assert_eq!(
            commutes(&AggregateOffsetsTotals, &DoublePostings, &x),
            Some(false),
            "an additive offset is non-linear; doubling exposes it"
        );
        // blind: the residual probe still passes (output invariant under split,
        // residual responds, round-trip rebuilds from the untouched breakdown).
        assert!(probe(&AggregateOffsetsTotals, &Split, &x)
            .unwrap()
            .residual_complete());
    }

    /// CATCH (residual) / BLIND (commutation): the count-only residual cannot
    /// reconstruct — the residual probe catches it — but its OUTPUT summary is
    /// correct, so it commutes just like the honest morphism.
    #[test]
    fn residual_bug_caught_only_by_residual_probe() {
        let x = sample();
        assert!(!probe(&AggregateDropsAmounts, &Split, &x)
            .unwrap()
            .residual_complete());
        // blind: the output is correct, so commutation cannot see the bug.
        assert_eq!(
            commutes(&AggregateDropsAmounts, &DoublePostings, &x),
            Some(true),
            "the residual bug leaves the output correct, so commutation is blind"
        );
    }

    /// The generation+selection loop end to end: run the candidate relations
    /// against the candidate mutants to build a REAL kill matrix, then let the
    /// selector pick the suite. The two bugs are complementary, so both relations
    /// are selected — neither alone covers both — and nothing is left uncovered.
    #[test]
    fn set_cover_selects_both_complementary_relations() {
        use crate::select::KillMatrix;

        let x = sample();

        // soundness guard: neither relation may flag the honest morphism.
        assert!(probe(&Aggregate, &Split, &x).unwrap().residual_complete());
        assert_eq!(commutes(&Aggregate, &DoublePostings, &x), Some(true));

        // a cell is `true` iff the relation (row) KILLS the mutant (column).
        let residual_kills = |complete: bool| !complete;
        let commute_kills = |c: Option<bool>| c == Some(false);

        // rows = [residual probe, commutation]; cols = [DropsAmounts, OffsetsTotals]
        let matrix = KillMatrix::new(vec![
            vec![
                residual_kills(
                    probe(&AggregateDropsAmounts, &Split, &x)
                        .unwrap()
                        .residual_complete(),
                ),
                residual_kills(
                    probe(&AggregateOffsetsTotals, &Split, &x)
                        .unwrap()
                        .residual_complete(),
                ),
            ],
            vec![
                commute_kills(commutes(&AggregateDropsAmounts, &DoublePostings, &x)),
                commute_kills(commutes(&AggregateOffsetsTotals, &DoublePostings, &x)),
            ],
        ]);

        assert_eq!(matrix.select(), vec![0, 1], "both relations are required");
        assert!(
            matrix.uncoverable().is_empty(),
            "no mutant survives the selected suite"
        );
    }
}
