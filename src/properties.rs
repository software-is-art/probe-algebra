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

use crate::boundary::{
    coefficient_holds, commutes, probe, reconstructs, run, Compose, Construction, Morphism,
};
use crate::ledger::boundary::{
    Account, Aggregate, AggregateDropsAmounts, Balance, Cents, ParseAccount, ParseCents,
    ParseTransaction, Posting, Round, Split, Transaction,
};
use crate::linear::boundary::{Double, Quantity, Scale, UnitResponse};

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

/// Any valid Cents across the full range — exercises the operators near their
/// invariant edges (where checked_add/checked_sub can return None).
fn cents() -> impl Strategy<Value = Cents> {
    (-100_000_000i64..=100_000_000i64).prop_map(|c| Cents::new(c).expect("in range"))
}

/// A Balance built through its operators from a single Cents (provenance: a
/// balance only ever arises from accumulating amounts).
fn balance() -> impl Strategy<Value = Balance> {
    cents().prop_map(|c| Balance::zero().add_cents(c))
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

/// Quantities small enough that doubling-then-scaling stays inside the valid
/// range, so the structural relations are exercised without overflow noise.
fn quantity() -> impl Strategy<Value = Quantity> {
    (-50_000i64..=50_000i64).prop_map(|n| Quantity::new(n).expect("in range"))
}

/// A whitespace run (possibly empty) for padding raw account names.
fn padding() -> impl Strategy<Value = String> {
    prop_oneof![Just(""), Just(" "), Just("  "), Just("\t"), Just(" \t ")].prop_map(str::to_string)
}

/// A RAW (unparsed) account name with arbitrary surrounding padding — the input to
/// the `ParseAccount` construction, before any trimming.
fn padded_name() -> impl Strategy<Value = String> {
    (
        padding(),
        prop_oneof!["Cash", "Revenue", "Tax", "Fees"],
        padding(),
    )
        .prop_map(|(lead, name, trail)| format!("{lead}{name}{trail}"))
}

/// A RAW (unparsed) posting list in arbitrary order — the input to the
/// `ParseTransaction` construction, before canonical sorting.
fn raw_postings() -> impl Strategy<Value = Vec<Posting>> {
    prop::collection::vec(posting(amount_broad()), 1..=8)
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

    /// The SAME bug is caught with NO perturbation operator at all — a plain
    /// round-trip over randomly generated inputs already witnesses the
    /// incompleteness. (Contrast `honest_aggregate_round_trips`, which holds.)
    #[test]
    fn buggy_fails_plain_round_trip(x in tx_substantial()) {
        let (summary, residual) = AggregateDropsAmounts.forward(&x);
        let recovered = AggregateDropsAmounts.backward(&summary, &residual);
        // The bug is not "fails to reconstruct" — it reconstructs SOMETHING, just
        // the WRONG transaction. (Pins that `backward` is not vacuously `None`.)
        prop_assert!(recovered.is_some());
        prop_assert_ne!(recovered.as_ref(), Some(&x));
    }

    // ----- Cents operator laws -----

    /// `split` is loss-free: the two parts always sum back to the original.
    #[test]
    fn cents_split_round_trips(c in cents()) {
        let (half, rest) = c.split();
        prop_assert_eq!(half.checked_add(rest), Some(c));
    }

    /// `split` is BALANCED: the two parts differ by at most one cent. (The
    /// round-trip law alone is invariant under e.g. `/`→`*`, which yields a wildly
    /// unbalanced split that still sums back; this pins the actual halving.)
    #[test]
    fn cents_split_is_balanced(c in cents()) {
        let (half, rest) = c.split();
        prop_assert!((half.get() - rest.get()).abs() <= 1, "unbalanced split: {:?}/{:?}", half, rest);
    }

    /// Negation is an involution.
    #[test]
    fn cents_negate_is_involution(c in cents()) {
        prop_assert_eq!(c.negate().negate(), c);
    }

    /// Negation is the ADDITIVE INVERSE: `c + (-c) == 0`. (Involution alone holds
    /// for the identity, so it cannot tell a deleted negation from a real one.)
    #[test]
    fn cents_negate_is_additive_inverse(c in cents()) {
        prop_assert_eq!(c.negate().checked_add(c), Some(Cents::zero()));
    }

    /// Zero is the additive identity.
    #[test]
    fn cents_zero_is_identity(c in cents()) {
        prop_assert_eq!(c.checked_add(Cents::zero()), Some(c));
    }

    /// Addition is commutative (including when it overflows the range to None).
    #[test]
    fn cents_add_is_commutative(a in cents(), b in cents()) {
        prop_assert_eq!(a.checked_add(b), b.checked_add(a));
    }

    /// add then sub is identity whenever the addition stayed in range.
    #[test]
    fn cents_add_sub_inverse(a in cents(), b in cents()) {
        if let Some(sum) = a.checked_add(b) {
            prop_assert_eq!(sum.checked_sub(b), Some(a));
        }
    }

    // ----- construction (entry-edge) round-trip laws -----

    /// `ParseCents` is a PURE refinement: every admitted integer round-trips through
    /// the `Unit` residual, and the refined value matches the `Cents::new` facade.
    #[test]
    fn parse_cents_round_trips(c in -100_000_000i64..=100_000_000i64) {
        prop_assert_eq!(reconstructs(&ParseCents, &c), Some(true));
        prop_assert_eq!(ParseCents.parse(&c).map(|(v, _)| v), Cents::new(c));
    }

    /// `ParseCents` REJECTS out-of-range integers (no round-trip obligation), exactly
    /// as the `Cents::new` facade does.
    #[test]
    fn parse_cents_rejects_out_of_range(c in prop_oneof![i64::MIN..=-100_000_001i64, 100_000_001i64..=i64::MAX]) {
        prop_assert_eq!(reconstructs(&ParseCents, &c), None);
        prop_assert_eq!(Cents::new(c), None);
    }

    /// `ParseAccount` is a NORMALIZING parse whose residual is COMPLETE: it recovers
    /// the exact padded original for every padded name, and the refined value equals
    /// the trimmed `Account::new` facade.
    #[test]
    fn parse_account_round_trips(raw in padded_name()) {
        prop_assert_eq!(reconstructs(&ParseAccount, &raw), Some(true));
        prop_assert_eq!(ParseAccount.parse(&raw).map(|(v, _)| v), Account::new(&raw));
    }

    /// `ParseTransaction` is a NORMALIZING parse whose permutation residual is
    /// COMPLETE: it restores the exact input ordering for every raw posting list, and
    /// the refined value equals the sorted `Transaction::new` facade.
    #[test]
    fn parse_transaction_round_trips(raw in raw_postings()) {
        prop_assert_eq!(reconstructs(&ParseTransaction, &raw), Some(true));
        prop_assert_eq!(ParseTransaction.parse(&raw).map(|(v, _)| v), Transaction::new(raw.clone()));
    }

    // ----- Balance operator laws -----

    /// `split_dollar` is loss-free, and the remainder is always sub-dollar.
    #[test]
    fn balance_split_dollar_round_trips(b in balance()) {
        let (whole, remainder) = b.split_dollar();
        prop_assert!((0..100).contains(&remainder.get()), "remainder not sub-dollar: {:?}", remainder);
        prop_assert_eq!(whole.add_cents(remainder), b);
    }

    /// Negation is an involution.
    #[test]
    fn balance_negate_is_involution(b in balance()) {
        prop_assert_eq!(b.negate().negate(), b);
    }

    /// Negation is the ADDITIVE INVERSE: `b + (-b) == 0` (kills a deleted `-`,
    /// which involution alone misses).
    #[test]
    fn balance_negate_is_additive_inverse(b in balance()) {
        prop_assert_eq!(b.negate().plus(b), Balance::zero());
    }

    /// Zero is the additive identity.
    #[test]
    fn balance_zero_is_identity(b in balance()) {
        prop_assert_eq!(b.plus(Balance::zero()), b);
    }

    // ----- linear transport: the decisive negative result, over the space -----

    /// BLIND #1 (everywhere): the wrong-coefficient transport round-trips for
    /// every input — round-trip is structurally blind to a wrong constant.
    #[test]
    fn skew_round_trips_everywhere(x in quantity()) {
        let recovered = run(&Scale::skew(), &x).invert(&Scale::skew());
        prop_assert_eq!(recovered.as_ref(), Some(&x));
    }

    /// BLIND #2 (everywhere): the wrong coefficient commutes with doubling for
    /// every input, exactly as the honest one does.
    #[test]
    fn skew_commutes_everywhere(x in quantity()) {
        prop_assert_eq!(commutes(&Scale::honest(), &Double, &x), Some(true));
        prop_assert_eq!(commutes(&Scale::skew(), &Double, &x), Some(true));
    }

    /// CATCH (everywhere): the quantitative probe holds for the honest transport
    /// and FAILS for skew on every input — it is what separates them.
    #[test]
    fn quantitative_separates_honest_and_skew(x in quantity()) {
        let unit_response = UnitResponse::from_reference(Scale::reference_rate());
        prop_assert_eq!(coefficient_holds(&Scale::honest(), &unit_response, &x), Some(true));
        prop_assert_eq!(coefficient_holds(&Scale::skew(), &unit_response, &x), Some(false));
    }
}
