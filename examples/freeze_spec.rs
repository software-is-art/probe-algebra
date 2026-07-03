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
//! The ALGEBRA-MUTATION locks (`spec/<theory>.mutation.spec`) freeze here as well: each
//! theory's operator table is perturbed in-process and every mutant judged by re-discovery
//! (see `discover::mutation`) — survivors are RATIFIED degrees of freedom, and a new
//! survivor (or a fixed one) is a reviewed diff to these files.
//!
//! Run `cargo run --example freeze_spec`.

use boundary_spec::discover::arithmetic::Arithmetic;
use boundary_spec::discover::date::Calendar;
use boundary_spec::discover::mutation::MutationReport;
use boundary_spec::discover::router::Router;
use boundary_spec::discover::system::SystemReport;
use boundary_spec::discover::world::{StoreModel, StoreProtocol};
use boundary_spec::discover::{all_specs, BoundarySpec, Spec};
use boundary_spec::kvstore::theory::TtlStore;

fn main() {
    let specs = all_specs();
    let mut locks: Vec<spec_lock::Lock> = specs.iter().map(Spec::lock).collect();
    locks.push(SystemReport::of::<BoundarySpec>().lock());
    // the WORLD lock: the model's ratified beliefs about the demonstration dependency
    // (see `discover::world` — the freeze discipline pointed outward).
    locks.push(StoreModel::beliefs().lock());
    // the ALGEBRA-MUTATION locks: the spec's kill power per theory, survivors ratified.
    locks.push(MutationReport::of::<Arithmetic>().lock());
    locks.push(MutationReport::of::<Router>().lock());
    locks.push(MutationReport::of::<Calendar>().lock());
    locks.push(MutationReport::of::<TtlStore>().lock());
    locks.push(MutationReport::of::<StoreProtocol>().lock());
    spec_lock::bless(&locks).expect("write spec locks");
    for (lock, spec) in locks.iter().zip(&specs) {
        println!("froze {} ({} laws)", lock.path.display(), spec.laws.len());
    }
    for (lock, label) in locks.iter().skip(specs.len()).zip([
        "the seam graph",
        "the world lock",
        "algebra mutation: interpreter arithmetic",
        "algebra mutation: router",
        "algebra mutation: date calculus",
        "algebra mutation: ttl store",
        "algebra mutation: store protocol",
    ]) {
        println!("froze {} ({label})", lock.path.display());
    }
}
