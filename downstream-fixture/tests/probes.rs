//! probes — run the fixture's edge probes.
//!
//! Our build.rs's edge-probe completeness pass proves every edge HAS an `impl Probed`
//! (delete `ParseCredits`'s and the crate stops building); this test is the other half
//! of the contract — the probe actually RUNS in CI, the same split the library's own
//! harness registry uses.

use downstream_fixture::meter::ParseCredits;

use boundary_algebra::boundary::Probed;

/// The entry edge's derived laws hold: admitted raws reconstruct exactly, and admission
/// agrees with the validity rule across a range spanning both saturation points.
#[test]
fn the_entry_edge_is_probed() {
    ParseCredits::probe();
}
