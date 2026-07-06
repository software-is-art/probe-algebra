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
use delta_render::license::Registry;
use delta_render::ops::{DistinctOp, FilterOp, JoinOp, MapOp, MinOp, SumOp};
use delta_render::zset::ZSetAlgebra;

fn main() {
    let spec_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("spec");
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
        Spec::of::<DistinctOp>().lock_in(&spec_dir),
        MutationReport::of::<DistinctOp>().lock_in(&spec_dir),
        Spec::of::<MinOp>().lock_in(&spec_dir),
        MutationReport::of::<MinOp>().lock_in(&spec_dir),
        // the pivot artifact: the registry READS the specs above; its lock makes the
        // read a reviewed diff.
        Registry::derive().lock_in(&spec_dir),
    ];
    spec_lock::bless(&locks).expect("write the spec locks");
    for lock in &locks {
        println!("blessed `{}` -> {}", lock.name, lock.path.display());
    }
    println!("ratify the diff: the committed spec files are the behaviour contract.");
}
