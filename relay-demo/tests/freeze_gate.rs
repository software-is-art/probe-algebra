//! freeze_gate — the DRIFT GATE, as a plain integration test.
//!
//! Re-derive the live discovered spec and hold it against the committed locks. Genesis
//! committed TARGET locks — the DECLARED laws and the declared seam graph, in the exact lock
//! format — so this gate is RED BY DESIGN until the meaning holes are filled and discovery
//! re-earns the declaration. The fix is never to hand-edit a lock: fill the meaning, run
//! `cargo run --example freeze`, and ratify the diff against the targets in review.

use std::path::PathBuf;

use boundary_spec::discover::system::{System, SystemReport};
use relay_app::system::RelayApp;

fn spec_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("spec")
}

/// The committed locks are FRESH: the live discovered algebra (every module in the graph's
/// registry, and the graph itself) matches what was ratified — which, until the first bless,
/// is the DECLARED target.
#[test]
fn the_committed_specs_are_fresh() {
    let spec_dir = spec_dir();
    let mut locks: Vec<spec_lock::Lock> = RelayApp::modules()
        .iter()
        .map(|spec| spec.lock_in(&spec_dir))
        .collect();
    locks.push(SystemReport::of::<RelayApp>().lock_in(&spec_dir));
    if let Err(stale) = spec_lock::check(&locks) {
        panic!(
            "discovered spec differs from the committed lock for: {}. If discovery now matches \
             the declaration, run `cargo run --example freeze` and ratify the (empty) diff; \
             otherwise the meaning does not yet earn the declared laws.",
            stale.join(", ")
        );
    }
}
