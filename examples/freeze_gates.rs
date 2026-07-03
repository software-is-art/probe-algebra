//! freeze_gates — regenerate the PIPELINE's two locks from the gate registry.
//!
//!     cargo run --example freeze_gates
//!
//! The pipeline is a lock (see `discover::gates`): `spec/gates.spec` carries the ratified
//! inventory (what each gate promises, at what cadence, with what capability), and
//! `.github/workflows/ci.yml` is the DERIVED workflow — never edited by hand. Run this when
//! the registry legitimately changes (a new gate, a toolchain bump, a cadence change);
//! review the diff it produces as the ratification of the new pipeline.

use boundary_spec::discover::gates::GateRegistry;

fn main() {
    let locks = [GateRegistry::registry_lock(), GateRegistry::workflow_lock()];
    spec_lock::bless(&locks).expect("write the pipeline locks");
    for lock in &locks {
        println!("froze {}", lock.path.display());
    }
    println!("the diff to those two files IS the pipeline change — ratify it in review.");
}
