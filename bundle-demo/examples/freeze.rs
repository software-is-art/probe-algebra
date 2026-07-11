//! freeze — regenerate this member's committed locks (the spec and the mutation verdict
//! for the lifted tally algebra) into `bundle-demo/spec/`, exactly like the root freeze
//! path one altitude down. Run `cargo run -p bundle-demo --example freeze` and ratify
//! the diff.

use boundary_spec::discover::lift::Lifted;
use boundary_spec::discover::mutation::MutationReport;
use boundary_spec::discover::Spec;
use bundle_demo::tally::Tally;

fn main() {
    let spec_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("spec");
    let locks = vec![
        Spec::of::<Lifted<Tally>>().lock_in(&spec_dir),
        MutationReport::of::<Lifted<Tally>>().lock_in(&spec_dir),
    ];
    spec_lock::bless(&locks).expect("write the demo locks");
    for lock in &locks {
        println!("froze {}", lock.path.display());
    }
}
