//! freeze_spec — regenerate the committed discovered-spec locks under `spec/`.
//!
//! Discovery is a pure function of the boundary, so its output is frozen into a committed file per
//! theory. Run this when a boundary's algebra legitimately changes; review the diff it produces as
//! the RATIFICATION of the new spec. CI's staleness gate (`freeze::the_committed_specs_are_fresh`)
//! fails if the committed locks are out of date, so an unintended behaviour change is caught.
//! The write itself is `spec_lock::bless` — the generic regeneration path; `Spec::lock` supplies
//! this repo's artifacts (path + rendered text). A downstream crate writes the same loop over its
//! own theories with `Spec::of::<MyTheory>().lock_in(spec_dir)`, rooting the locks in ITS repo.
//!
//! The SYSTEM lock (`spec/boundary-spec.system.spec`) freezes here too: the compiled
//! `system!` graph — the module registry plus every seam's obligation and status — under the
//! same discipline (see `discover::system`).
//!
//! Run `cargo run --example freeze_spec`.

use boundary_spec::discover::system::SystemReport;
use boundary_spec::discover::{all_specs, BoundarySpec, Spec};

fn main() {
    let specs = all_specs();
    let mut locks: Vec<spec_lock::Lock> = specs.iter().map(Spec::lock).collect();
    locks.push(SystemReport::of::<BoundarySpec>().lock());
    spec_lock::bless(&locks).expect("write spec locks");
    for (lock, spec) in locks.iter().zip(&specs) {
        println!("froze {} ({} laws)", lock.path.display(), spec.laws.len());
    }
    println!(
        "froze {} (the seam graph)",
        locks.last().expect("system lock").path.display()
    );
}
