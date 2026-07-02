//! probes — run the fixture's edge probes, and drive the full edge chain.
//!
//! Our build.rs's edge-probe completeness pass proves every edge HAS an `impl Probed`
//! (delete any of them and the crate stops building); this test is the other half of the
//! contract — the probes actually RUN in CI, the same split the library's own harness
//! registry uses. The chain test then drives all three downstream-minted edges in one
//! seed scope: parse → check → deduct, the whole discipline through public API.

use downstream_fixture::meter::{CheckFunds, Credits, Deduct, Order, ParseCredits, Purchase};

use boundary_algebra::boundary::{Construction, Probed};
use boundary_algebra::gdp::with_seed;

/// The entry edge's derived laws hold: admitted raws reconstruct exactly, and admission
/// agrees with the validity rule across a range spanning both saturation points.
#[test]
fn the_entry_edge_is_probed() {
    ParseCredits::probe();
}

/// The branch's derived law holds: classification agrees with the raw comparison over
/// every grid pair, and both arms fire.
#[test]
fn the_branch_edge_is_probed() {
    CheckFunds::probe();
}

/// The guarded edge's derived law holds: every affordable deduction agrees with `spend`
/// and is exact (the witness guarantees the zero-floor never engages).
#[test]
fn the_guarded_edge_is_probed() {
    Deduct::probe();
}

/// The full chain, through public API alone: raw `i64`s enter through the `Construction`,
/// the named order is classified by the `Branch`, and the `Guarded` deduction spends the
/// witness — no step skippable (the compile-fail suite pins the skips as type errors).
#[test]
fn the_edge_chain_runs_end_to_end() {
    let (balance, _) = ParseCredits.parse(&15).expect("15 is a valid balance");
    let (amount, _) = ParseCredits.parse(&6).expect("6 is a valid amount");

    // The affordable path: check mints the witness, deduct consumes it — exactly.
    let after = with_seed(|seed| {
        let order = seed.new_named(Order::new(balance, Purchase::of(amount)));
        let proof = CheckFunds.classify(&order).expect("6 <= 15 is affordable");
        *Deduct.run(&order, &proof).value()
    });
    assert_eq!(after, Credits::new(9).expect("9 is a valid balance"));

    // The refusal path is first-class: an unaffordable order yields the NEGATIVE witness
    // (an `Insufficient<N>`, not a silent `None`) — and that witness discharges nothing.
    let refused = with_seed(|seed| {
        let order = seed.new_named(Order::new(amount, Purchase::of(balance)));
        CheckFunds.classify(&order).is_err()
    });
    assert!(
        refused,
        "15 from a balance of 6 must classify as insufficient"
    );
}
