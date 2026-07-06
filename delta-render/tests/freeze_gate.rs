//! freeze_gate — delta-render's DRIFT GATES: every committed lock is re-derived and held
//! against its file inside every `cargo test`. A mismatch is never fixed by editing the
//! lock: run `cargo run -p delta-render --example freeze` and ratify the diff.

use std::path::PathBuf;

use boundary_spec::discover::expect::Distance;
use boundary_spec::discover::mutation::MutationReport;
use boundary_spec::discover::Spec;
use delta_render::zset::ZSetAlgebra;

fn spec_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("spec")
}

/// The Z-set group's committed spec is FRESH — the live discovered algebra still matches
/// what was ratified.
#[test]
fn the_committed_zset_spec_is_fresh() {
    let lock = Spec::of::<ZSetAlgebra>().lock_in(&spec_dir());
    if let Err(stale) = spec_lock::check(std::slice::from_ref(&lock)) {
        panic!(
            "discovered spec drifted from the committed lock for: {}. \
             Run `cargo run -p delta-render --example freeze` and ratify the diff.",
            stale.join(", ")
        );
    }
}

/// The algebra-mutation verdict is fresh: the Z-set operator table is perturbed
/// in-process and every mutant judged by re-discovery — a survivor names a degree of
/// freedom the spec leaves open, ratified in the lock or closed by a sharper law.
#[test]
fn the_committed_zset_mutation_verdict_is_fresh() {
    let lock = MutationReport::of::<ZSetAlgebra>().lock_in(&spec_dir());
    if let Err(stale) = spec_lock::check(std::slice::from_ref(&lock)) {
        panic!(
            "the algebra-mutation verdict drifted for: {}. \
             Run `cargo run -p delta-render --example freeze` and ratify the diff.",
            stale.join(", ")
        );
    }
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
