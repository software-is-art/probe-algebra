//! freeze — the BLESS path: regenerate `spec/*.spec` from the live, discovered algebra.
//!
//!     cargo run --example freeze
//!
//! This is the ONE sanctioned writer of the lock files. Genesis committed TARGET locks (the
//! DECLARED laws and the declared seam graph); the drift gate stays red until discovery
//! matches them. Once the meaning holes are filled, run this and read the diff against the
//! targets — a clean diff means the code earned exactly what was declared; any other diff is
//! the conversation to have in review. Never edit a lock by hand.
//!
//! The lock list is READ OFF THE GRAPH (`CreditApp::modules()` plus the system lock): the
//! declaration is the registry, so a module cannot silently fall out of the freeze loop.

use std::path::PathBuf;

use boundary_spec::discover::system::{System, SystemReport};
use credit_app::system::CreditApp;

fn main() {
    let spec_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("spec");
    let mut locks: Vec<spec_lock::Lock> = CreditApp::modules()
        .iter()
        .map(|spec| spec.lock_in(&spec_dir))
        .collect();
    locks.push(SystemReport::of::<CreditApp>().lock_in(&spec_dir));
    spec_lock::bless(&locks).expect("write the spec locks");
    for lock in &locks {
        println!("blessed `{}` -> {}", lock.name, lock.path.display());
    }
    println!("now diff spec/ against genesis's targets — that diff is the ratification.");
}
