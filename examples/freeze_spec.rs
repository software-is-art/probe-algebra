//! freeze_spec — regenerate the committed discovered-spec locks under `spec/`.
//!
//! Discovery is a pure function of the boundary, so its output is frozen into a committed file per
//! theory. Run this when a boundary's algebra legitimately changes; review the diff it produces as
//! the RATIFICATION of the new spec. CI's staleness gate (`freeze::the_committed_specs_are_fresh`)
//! fails if the committed locks are out of date, so an unintended behaviour change is caught.
//!
//! Run `cargo run --example freeze_spec`.

use std::fs;

use boundary_algebra::discover::{all_specs, freeze};

fn main() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/spec");
    fs::create_dir_all(dir).expect("create spec/");
    for spec in all_specs() {
        let path = freeze::lock_path(spec.theory);
        fs::write(&path, freeze::render(&spec)).expect("write spec lock");
        println!("froze {} ({} laws)", path.display(), spec.laws.len());
    }
}
