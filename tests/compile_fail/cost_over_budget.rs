//! COST budget violation: applying an O(n) edge per element bumps its degree on that
//! axis, so `MapCollect<Aggregate, Postings>` is degree 2 (quadratic) in `Postings`.
//! Demanding it stay within a degree-1 (linear) budget on `Postings` cannot be satisfied
//! — `WithinBudget` requires the cost's per-axis degree `<=` the ceiling's, and 2 is not
//! `<= 1`, so the over-budget pipeline is a COMPILE error: the pressure a
//! complexity-blind agent lacks, now per size-axis.
#![allow(unused_variables, unused_imports, dead_code)]

use boundary_algebra::boundary::{require_within, CostCons, CostNil, MapCollect, TimeCost, S, Z};
use boundary_algebra::ledger::boundary::{Aggregate, Postings};

fn main() {
    // postings^2 time demanded to be within postings^1: unsatisfied.
    require_within::<TimeCost, MapCollect<Aggregate, Postings>, CostCons<Postings, S<Z>, CostNil>>();
}
