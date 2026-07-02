//! CAPABILITY under-claim (v4): the capability's STATE FLOOR is read from the INPUT type, so a
//! context that consumes a state-carrying value object (`Bound`, which holds an `Env`) cannot be
//! demanded `Pure` — `<Bound as InputEffect>::Eff` is `Stateful`, which is not `AtMost<Pure>`.
//!
//! This is the bound `run_pure` imposes (`<M::In as InputEffect>::Eff: AtMost<Pure>`), exercised
//! directly: an edge that UNDER-declares `Pure` over a `Bound` input is rejected on STRUCTURE, not
//! on its (lie-able) annotation — the hidden state dependency is a compile error. (The over-claim
//! direction is a negative the type system can't express, so it stays the behavioural audit's job;
//! and the check reads the INPUT type, not the annotation, so a downstream edge cannot lie here.)
#![allow(dead_code)]

use boundary_spec::boundary::{AtMost, InputEffect, Pure};
use boundary_spec::interp::boundary::Bound;

/// The exact floor `run_pure` demands of an edge's input.
fn demand_pure_input<T>()
where
    T: InputEffect,
    <T as InputEffect>::Eff: AtMost<Pure>,
{
}

fn main() {
    // `Bound` carries state, so its input effect is `Stateful` — not `AtMost<Pure>`.
    demand_pure_input::<Bound>();
}
