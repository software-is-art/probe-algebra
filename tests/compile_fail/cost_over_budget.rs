//! COST budget violation: collecting an edge per element bumps its degree on that axis,
//! so `MapCollect<Eval, Nodes>` is degree 2 (quadratic) in `Nodes`. Demanding it stay
//! within a degree-1 (linear) budget on `Nodes` cannot be satisfied — `WithinBudget`
//! requires the cost's per-axis degree `<=` the ceiling's, and 2 is not `<= 1`, so the
//! over-budget pipeline is a COMPILE error: the pressure a complexity-blind agent lacks,
//! now per size-axis.
#![allow(unused_variables, unused_imports, dead_code)]

use boundary_spec::boundary::{require_within, CostCons, CostNil, MapCollect, TimeCost, S, Z};
use boundary_spec::interp::boundary::{Eval, Nodes};

fn main() {
    // nodes^2 time demanded to be within nodes^1: unsatisfied.
    require_within::<TimeCost, MapCollect<Eval, Nodes>, CostCons<Nodes, S<Z>, CostNil>>();
}
