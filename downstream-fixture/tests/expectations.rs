//! expectations — the consumer's DECLARED-EXPECTATIONS gate, green.
//!
//! `ops.rs` declares the meter's intended algebra in its `#[algebra(..., expects(...))]`
//! attribute — nine laws, the full content of `spec/credit-meter.spec` in the declaration
//! vocabulary — and this test holds the DISTANCE between declared and discovered at zero:
//! every declared law is found true (the gate is met) and nothing discovered goes undeclared
//! (no surprises). This is the top-down loop's terminal state: declare → red distance →
//! implement → GREEN → the freeze/drift gate (`tests/freeze_gate.rs`) takes over. From here a
//! regression reads as a red gate that NAMES the lost law, and a new law reads as a SURPRISE
//! prompting ratification — never as silence.

use boundary_spec::discover::expect::Distance;
use downstream_fixture::ops::meter_ops::CreditMeter;

/// The declaration is EARNED, exactly: nine of nine declared laws hold and discovery found
/// nothing the declaration doesn't name. The rendered report is pinned because the report is
/// the artifact — it is what an agent reads at every state of the gate, so its green wording
/// is part of the product too.
#[test]
fn the_declared_expectations_are_met_with_no_surprises() {
    let distance = Distance::of::<CreditMeter>();
    assert!(distance.is_met(), "report: {}", distance.render());
    assert!(
        distance.surprises.is_empty(),
        "undeclared laws were discovered — ratify them into `expects(...)` or refute the \
         operator: {}",
        distance.render()
    );
    assert_eq!(
        distance.render(),
        "credit meter: 10 of 10 declared laws hold; no surprises"
    );
}
