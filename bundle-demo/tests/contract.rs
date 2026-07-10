//! The bundle-born member's whole verdict, in four gates: the declared contract is MET,
//! the committed lift IS the scan's output (derived, never transcribed), the module IS
//! canonically placed (the round-trip pin, on the file the verbs grew), and the frozen
//! locks are fresh. Together they close the loop MANIFEST.md narrates.

use boundary_spec::discover::bundle::Bundle;
use boundary_spec::discover::expect::Distance;
use boundary_spec::discover::lift::{AutoLift, Lifted};
use boundary_spec::discover::mutation::MutationReport;
use boundary_spec::discover::Spec;
use bundle_demo::tally::Tally;

/// The declarations, ONE source: the same list every `bundle lift` in MANIFEST.md passed —
/// the drift gate below holds the committed lift to exactly these.
const DECLARATIONS: &[&str] = &[
    "commutative(merge)",
    "associative(merge)",
    "idempotent(merge)",
    "identity(merge, floor)",
];

/// THE CONTRACT GATE: every declared law is found by discovery — the SHOULD half, judged
/// by the same engine that derives the IS half, on every test run. This is the
/// zero-annotation red/green gate: an implementation edit that loses a declared law
/// refuses HERE, naming the law.
#[test]
fn the_declared_contract_is_met() {
    let d = Distance::of::<Lifted<Tally>>();
    assert_eq!(d.declared, DECLARATIONS.len());
    assert!(
        d.missing.is_empty(),
        "declared laws unmet: {:?}",
        d.missing.iter().map(|e| e.render()).collect::<Vec<_>>()
    );
}

/// THE DERIVATION GATE: the committed `tally_lift.rs` is byte-for-byte what `bundle lift`
/// generates from the committed module and the declarations above — the lift is a derived
/// artifact under the one rule, and a module edit that should change it cannot land
/// without regenerating it.
#[test]
fn the_committed_lift_is_the_scans_output() {
    let source = include_str!("../src/tally.rs");
    let generated = AutoLift::scan_module(source, "tally", DECLARATIONS).expect("the module scans");
    assert_eq!(
        generated,
        include_str!("../src/tally_lift.rs"),
        "tally_lift.rs drifted — regenerate: cargo run --example bundle -- lift \
         bundle-demo/src/tally.rs tally <declarations>  > bundle-demo/src/tally_lift.rs"
    );
}

/// THE PLACEMENT PIN: the module the verbs grew is canonically placed and round-trips
/// byte for byte — the continuation process left no disorder for a human to tidy.
#[test]
fn the_grown_module_is_canonically_placed() {
    let source = include_str!("../src/tally.rs");
    let bundle = Bundle::parse(source).expect("parses");
    assert!(bundle.is_canonical());
    assert_eq!(bundle.render(), source);
}

/// THE LOCK GATE: the committed spec and mutation locks are fresh — what discovery finds
/// today is what was ratified (regenerate via `cargo run -p bundle-demo --example freeze`).
/// And the probes are not vacuous: the sweep catches a deaf operator.
#[test]
fn the_frozen_locks_are_fresh_and_sensitive() {
    let spec_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("spec");
    let locks = vec![
        Spec::of::<Lifted<Tally>>().lock_in(&spec_dir),
        MutationReport::of::<Lifted<Tally>>().lock_in(&spec_dir),
    ];
    if let Err(stale) = spec_lock::check(&locks) {
        panic!(
            "stale lock(s): {} — run `cargo run -p bundle-demo --example freeze` and \
             ratify the diff",
            stale.join(", ")
        );
    }
    let report = MutationReport::of::<Lifted<Tally>>();
    assert!(
        report.deaf.iter().any(|(_, killed)| *killed),
        "the discovered laws catch a deaf operator"
    );
}
