//! freeze — delta-render's BLESS path: regenerate every committed lock from the live code.
//!
//!     cargo run -p delta-render --example freeze
//!
//! The one sanctioned writer of this crate's `spec/` files. The drift gates
//! (`tests/freeze_gate.rs`) fail whenever a live derivation differs from its committed
//! lock; the fix is never to edit a lock by hand — run this, read the diff, ratify it in
//! review. Idempotent: on unchanged code a second run rewrites identical bytes.

use std::path::PathBuf;

use boundary_spec::discover::mutation::MutationReport;
use boundary_spec::discover::Spec;
use delta_render::circuit::{audit_circuit, circuit_locks, demo_circuit};
use delta_render::license::Registry;
use delta_render::ops::{
    min_retraction_witness, DistinctOp, FilterOp, JoinOp, MapOp, MinOp, ScaleOp, SumOp,
};
use delta_render::stream::StreamCalculus;
use delta_render::zset::ZSetAlgebra;

fn main() {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let spec_dir = crate_root.join("spec");
    let registry = Registry::derive();
    let [gen_lock, derivation_lock] =
        circuit_locks(&demo_circuit(&registry), &registry, &crate_root);
    let [audit_gen_lock, audit_derivation_lock] =
        circuit_locks(&audit_circuit(&registry), &registry, &crate_root);
    let locks = [
        Spec::of::<ZSetAlgebra>().lock_in(&spec_dir),
        MutationReport::of::<ZSetAlgebra>().lock_in(&spec_dir),
        Spec::of::<FilterOp>().lock_in(&spec_dir),
        MutationReport::of::<FilterOp>().lock_in(&spec_dir),
        Spec::of::<MapOp>().lock_in(&spec_dir),
        MutationReport::of::<MapOp>().lock_in(&spec_dir),
        Spec::of::<SumOp>().lock_in(&spec_dir),
        MutationReport::of::<SumOp>().lock_in(&spec_dir),
        Spec::of::<JoinOp>().lock_in(&spec_dir),
        MutationReport::of::<JoinOp>().lock_in(&spec_dir),
        Spec::of::<ScaleOp>().lock_in(&spec_dir),
        MutationReport::of::<ScaleOp>().lock_in(&spec_dir),
        Spec::of::<DistinctOp>().lock_in(&spec_dir),
        MutationReport::of::<DistinctOp>().lock_in(&spec_dir),
        Spec::of::<MinOp>().lock_in(&spec_dir),
        MutationReport::of::<MinOp>().lock_in(&spec_dir),
        Spec::of::<StreamCalculus>().lock_in(&spec_dir),
        MutationReport::of::<StreamCalculus>().lock_in(&spec_dir),
        // the pivot artifact: the registry READS the specs above; its lock makes the
        // read a reviewed diff.
        registry.lock_in(&spec_dir),
        // the RENDER: generated Rust + the plain-language derivation, one derivation,
        // two locks.
        gen_lock,
        derivation_lock,
        audit_gen_lock,
        audit_derivation_lock,
        // the frozen red instance behind `min: NEITHER` — computed, never typed.
        spec_lock::Lock {
            name: "min retraction witness".into(),
            path: spec_dir.join("min.retraction.spec"),
            live: min_retraction_witness(),
        },
    ];
    spec_lock::bless(&locks).expect("write the spec locks");
    for lock in &locks {
        println!("blessed `{}` -> {}", lock.name, lock.path.display());
    }
    println!("ratify the diff: the committed spec files are the behaviour contract.");
}
