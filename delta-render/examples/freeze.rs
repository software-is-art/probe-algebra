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
use delta_render::zset::ZSetAlgebra;

fn main() {
    let spec_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("spec");
    let locks = [
        Spec::of::<ZSetAlgebra>().lock_in(&spec_dir),
        MutationReport::of::<ZSetAlgebra>().lock_in(&spec_dir),
    ];
    spec_lock::bless(&locks).expect("write the spec locks");
    for lock in &locks {
        println!("blessed `{}` -> {}", lock.name, lock.path.display());
    }
    println!("ratify the diff: the committed spec files are the behaviour contract.");
}
