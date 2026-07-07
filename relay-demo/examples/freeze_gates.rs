//! freeze_gates — regenerate the pipeline locks from this crate's gate declaration.
//!
//!     cargo run --example freeze_gates
//!
//! The ONE sanctioned writer of `spec/gates.spec` and `.github/workflows/ci.yml`. Both are
//! renders of `src/gates.rs`'s declaration; the committed diff is the ratification. Never
//! edit either by hand.

use std::path::PathBuf;

use relay_app::gates::Ci;

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let locks = Ci::pipeline()
        .locks_in(&root)
        .expect("the declared pipeline renders");
    spec_lock::bless(&locks).expect("write the pipeline locks");
    for lock in &locks {
        println!("blessed `{}` -> {}", lock.name, lock.path.display());
    }
}
