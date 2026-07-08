//! freeze_gates — regenerate the PIPELINE's locks from the gate registry, and the
//! PERIMETER's from its declaration.
//!
//!     cargo run --example freeze_gates
//!
//! The pipeline is a lock (see `discover::gates`): `spec/gates.spec` carries the ratified
//! inventory (what each gate promises, at what cadence, with what capability), and
//! `.github/workflows/ci.yml` is the DERIVED workflow — never edited by hand. The
//! perimeter rides the same freeze (see `discover::perimeter`): `spec/perimeter.spec` is
//! the declared settings floor and `spec/perimeter.ruleset.json` the apply-able branch
//! ruleset — both derived (the required checks come from the registry), both drift-gated,
//! the WRITE to the live settings staying human. So does the git substrate (see
//! `discover::substrate`): `spec/substrate.spec` declares the tag meanings and history
//! laws the machinery leans on. Run this when the registry, the perimeter, or the
//! substrate declaration legitimately changes; review the diff as the ratification.

use boundary_spec::discover::gates::GateRegistry;
use boundary_spec::discover::perimeter::Perimeter;
use boundary_spec::discover::schemata::Schemata;
use boundary_spec::discover::substrate::Substrate;

fn main() {
    let perimeter = Perimeter::declared();
    let locks = [
        GateRegistry::registry_lock(),
        GateRegistry::workflow_lock(),
        perimeter.lock(),
        perimeter.ruleset_lock(),
        Substrate::declared().lock(),
        Schemata::lock().expect("the schemata census renders"),
    ];
    spec_lock::bless(&locks).expect("write the pipeline, perimeter, and substrate locks");
    for lock in &locks {
        println!("froze {}", lock.path.display());
    }
    println!(
        "the diff to those files IS the pipeline/perimeter/substrate/schemata change — ratify it in review."
    );
}
