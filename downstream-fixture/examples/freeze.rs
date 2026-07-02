//! freeze — the consumer's BLESS path: regenerate `spec/credit-meter.spec` from the live code.
//!
//!     cargo run -p downstream-fixture --example freeze
//!
//! This is the ONE sanctioned writer of the lock file. The drift gate
//! (`tests/freeze_gate.rs`) fails whenever the live discovered spec differs from the committed
//! file, and the fix is never to edit the file by hand: run this, read the diff it produces,
//! and ratify that diff in review — the committed diff IS the ratification. Idempotent: on
//! unchanged code a second run rewrites identical bytes and `git status` stays clean.
//!
//! Consumer notes: `Spec::of::<CreditMeter>()` derives the spec through the public engine;
//! `Spec::lock_in` points the lock at THIS repository's `spec/` directory (never use
//! `Spec::lock`, whose path is baked into the library's own checkout); `spec_lock::bless` owns
//! the write. See `docs/ci-discipline.md` in the library for the four-move discipline.

use std::path::PathBuf;

use boundary_spec::discover::Spec;
use downstream_fixture::ops::meter_ops::CreditMeter;

fn main() {
    let spec_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("spec");
    let lock = Spec::of::<CreditMeter>().lock_in(&spec_dir);
    spec_lock::bless(std::slice::from_ref(&lock)).expect("write the spec lock");
    println!("blessed `{}` -> {}", lock.name, lock.path.display());
    println!("ratify the diff: the committed spec file is the behaviour contract.");
}
