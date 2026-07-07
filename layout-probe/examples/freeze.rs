//! freeze — layout-probe's BLESS path: regenerate every committed lock from the live code.
//!
//!     cargo run -p layout-probe --example freeze
//!
//! The one sanctioned writer of this crate's `spec/` files; the drift gates fail
//! whenever a live derivation differs from its committed lock, and the fix is never a
//! hand edit — run this, read the diff, ratify it in review.

use std::path::PathBuf;

use boundary_spec::discover::mutation::MutationReport;
use boundary_spec::discover::Spec;
use layout_probe::census;
use layout_probe::theories::{EagerLayout, StableLayout};

fn main() {
    let spec_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("spec");
    let locks = [
        Spec::of::<StableLayout>().lock_in(&spec_dir),
        MutationReport::of::<StableLayout>().lock_in(&spec_dir),
        Spec::of::<EagerLayout>().lock_in(&spec_dir),
        MutationReport::of::<EagerLayout>().lock_in(&spec_dir),
        // the emergent constraints: floors and the locality witness, derived never typed.
        census::lock_in(&spec_dir),
    ];
    spec_lock::bless(&locks).expect("write the spec locks");
    for lock in &locks {
        println!("blessed `{}` -> {}", lock.name, lock.path.display());
    }
    println!("ratify the diff: the committed spec files are the behaviour contract.");
}
