//! freeze_gate — delta-render's DRIFT GATES: every committed lock is re-derived and held
//! against its file inside every `cargo test`. A mismatch is never fixed by editing the
//! lock: run `cargo run -p delta-render --example freeze` and ratify the diff.

use std::path::PathBuf;

use boundary_spec::discover::engine::Theory;
use boundary_spec::discover::expect::Distance;
use boundary_spec::discover::mutation::MutationReport;
use boundary_spec::discover::Spec;
use delta_render::license::{Classification, Registry};
use delta_render::ops::{DistinctOp, FilterOp, JoinOp, MapOp, MinOp, SumOp};
use delta_render::zset::ZSetAlgebra;

fn spec_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("spec")
}

fn check_theory<T: Theory>() {
    let locks = [
        Spec::of::<T>().lock_in(&spec_dir()),
        MutationReport::of::<T>().lock_in(&spec_dir()),
    ];
    if let Err(stale) = spec_lock::check(&locks) {
        panic!(
            "lock drifted for: {}. \
             Run `cargo run -p delta-render --example freeze` and ratify the diff.",
            stale.join(", ")
        );
    }
}

/// Every theory's committed spec AND its algebra-mutation verdict are FRESH — the
/// carrier group and all six lifted operators, re-derived inside this test.
#[test]
fn every_committed_spec_and_mutation_verdict_is_fresh() {
    check_theory::<ZSetAlgebra>();
    check_theory::<FilterOp>();
    check_theory::<MapOp>();
    check_theory::<SumOp>();
    check_theory::<JoinOp>();
    check_theory::<DistinctOp>();
    check_theory::<MinOp>();
}

/// The LICENSE REGISTRY is fresh: re-derive the classifications by re-reading the live
/// spec renders and hold the table against `spec/licenses.spec`. This is the pivot
/// artifact's gate — change any operator's body so its laws shift and this lock (plus
/// the operator's own spec lock) reddens in the same `cargo test` run.
#[test]
fn the_committed_license_registry_is_fresh() {
    let lock = Registry::derive().lock_in(&spec_dir());
    if let Err(stale) = spec_lock::check(std::slice::from_ref(&lock)) {
        panic!(
            "the license registry drifted for: {}. An operator's laws changed — its \
             classification is a derived fact, so regenerate \
             (`cargo run -p delta-render --example freeze`) and ratify BOTH diffs: the \
             operator's spec and the registry row it flips.",
            stale.join(", ")
        );
    }
}

/// The classification table, pinned — the design's acceptance row by row: the three
/// linear operators, the bilinear join, and the two deliberate negatives that must fall
/// to the generic fallback.
#[test]
fn the_classifications_are_exactly_the_designed_table() {
    let r = Registry::derive();
    let table: Vec<(&str, Classification)> = r
        .licenses
        .iter()
        .map(|l| (l.operator.as_str(), l.classification))
        .collect();
    assert_eq!(
        table,
        vec![
            ("filter", Classification::Linear),
            ("map", Classification::Linear),
            ("sum", Classification::Linear),
            ("join", Classification::Bilinear),
            ("distinct", Classification::Neither),
            ("min", Classification::Neither),
        ],
        "an operator's license flipped — read the spec diff to see which law moved"
    );
}

/// The DECLARED Abelian group is met with no surprises: everything discovery finds was
/// declared, everything declared is found. The inverse law is the one the whole design
/// leans on — deltas need retractions — so the distance gate holding green IS the
/// license's precondition.
#[test]
fn the_declared_group_is_met_with_no_surprises() {
    let distance = Distance::of::<ZSetAlgebra>();
    assert!(distance.is_met(), "report: {}", distance.render());
    assert_eq!(
        distance.render(),
        "zset: 7 of 7 declared laws hold; no surprises"
    );
}
