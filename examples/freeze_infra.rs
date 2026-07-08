//! freeze_infra — regenerate the INFRA graph's lock from its declaration.
//!
//!     cargo run --example freeze_infra
//!
//! The infra graph is a lock (see `discover::infra`): the deployment's surfaces,
//! stores, seams, credential names, authorities, and cadences, declared in one place
//! and frozen as `spec/<system>.infra.spec`. The register floor
//! (`spec/<system>.infra.register`) is HAND-AUTHORED, never generated — writing a line
//! there IS the ratification of a fact no API can judge. Run this when the declaration
//! legitimately changes; review the diff as the ratification.

use boundary_spec::discover::infra::Infra;

fn main() {
    let infra = Infra::exemplar();
    let locks = [infra.lock()];
    spec_lock::bless(&locks).expect("write the infra lock");
    for lock in &locks {
        println!("froze {}", lock.path.display());
    }
    println!("the diff to that file IS the infra change — ratify it in review.");
}
