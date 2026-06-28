//! COST budget violation: applying an O(n) edge per element is O(n^2), so demanding it
//! stay within an O(n) TIME budget cannot be satisfied. `MapCollect(Aggregate)` has
//! `type Time = Quadratic`, and `Quadratic: Within<Linear>` is not implemented, so the
//! over-budget pipeline is a COMPILE error — the pressure a complexity-blind agent
//! lacks: a stage that drifts to quadratic refuses to build under a linear budget.
#![allow(unused_variables, unused_imports, dead_code)]

use boundary_algebra::boundary::{require_within_time, Linear, MapCollect};
use boundary_algebra::ledger::boundary::Aggregate;

fn main() {
    // O(n^2) time demanded to be within O(n): unsatisfied.
    require_within_time::<Linear, _>(&MapCollect(Aggregate));
}
