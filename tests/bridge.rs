//! bridge — the theory-bridge's drift gates and acceptance pins: the committed prover
//! export (`spec/bridged-bool.export`) is the INPUT; its discovered spec, its algebra-
//! mutation verdict, and its triage (agreements / conjectures) are DERIVED locks,
//! re-derived and held against their files inside every `cargo test`. The one
//! epistemic rule under gate: discovery here SUPPLIES and TRIAGES conjectures — it
//! certifies nothing, and a disagreement with an upstream certificate is a defect
//! with certainty, never a row to ratify.

use std::path::PathBuf;

use boundary_spec::discover::bridge::{Bridged, Export, Triage};
use boundary_spec::discover::mutation::MutationReport;
use boundary_spec::discover::Spec;

fn spec_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("spec")
}

/// Mount the committed export in slot 0 (idempotent — every test calls this).
fn mount_committed() {
    let text = std::fs::read_to_string(spec_dir().join("bridged-bool.export"))
        .expect("the committed export fixture is part of the tree");
    Export::parse(&text)
        .expect("the committed export parses")
        .install::<0>()
        .expect("slot 0 holds the committed fixture");
}

/// Every DERIVED bridge lock is fresh against the committed INPUT: the spec, the
/// mutation verdict, and the triage. A drifted export (the prover re-emitted) or a
/// drifted bridge (parser, table layout, triage prose) reddens here, and the fix is
/// the freeze path, never a hand edit.
#[test]
fn the_committed_bridge_locks_are_fresh() {
    mount_committed();
    let locks = [
        Spec::of::<Bridged<0>>().lock_in(&spec_dir()),
        MutationReport::of::<Bridged<0>>().lock_in(&spec_dir()),
        Triage::of::<Bridged<0>>().lock_in(&spec_dir()),
    ];
    if let Err(stale) = spec_lock::check(&locks) {
        panic!(
            "bridge lock drifted for: {}. \
             Run `cargo run --example freeze_spec` and ratify the diff.",
            stale.join(", ")
        );
    }
}

/// The triage partition, pinned: all four upstream certificates round-trip as
/// agreements, no disagreement exists (`certify` is the gate the freeze path also
/// runs), and the conjecture supply contains the laws worth burning a proof day on —
/// both De Morgan duals, xor's group structure — none of which the prover certified.
#[test]
fn the_triage_partition_is_the_designed_table() {
    mount_committed();
    let t = Triage::of::<Bridged<0>>();
    assert_eq!(
        t.certify(),
        Ok(()),
        "the committed export must not disagree"
    );
    assert_eq!(
        t.agreements,
        vec![
            "commutative(and)",
            "associative(and)",
            "identity(and, true)",
            "commutative(or)"
        ],
        "an upstream certificate stopped round-tripping"
    );
    for supplied in [
        "homomorphism(not, and, or)",
        "homomorphism(not, or, and)",
        "involution(not)",
        "self_inverse(xor, false)",
        "identity(or, false)",
    ] {
        assert!(
            t.conjectures.iter().any(|c| c == supplied),
            "conjecture supply lost {supplied}: {:?}",
            t.conjectures
        );
    }
    // and supply is disjoint from certification: nothing proved renders as a conjecture.
    for a in &t.agreements {
        assert!(
            !t.conjectures.contains(a),
            "{a} is both agreed and conjectured"
        );
    }
}
