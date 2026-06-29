//! boundary-algebra — a boundary-discipline experiment plus a layered probe method.
//!
//! Two constraints under study together:
//!   1. every primitive that means something in the domain is a VALUE OBJECT and
//!      every operation on it a VALUE OPERATOR — never raw primitive arithmetic
//!      at a call site; and
//!   2. a transformation is checked by a LAYERED probe suite, because no single
//!      check is highest-assurance.
//!
//! `crate::boundary` defines the grammar: a boundary is a CATEGORY of value-object
//! OBJECTS and value-operator MORPHISMS. The morphisms share one algebra in a few
//! type-distinguished shapes — `Morphism` (a total edge between value objects),
//! `Construction` (the partial ENTRY edge from a raw primitive, so even "parse, don't
//! validate" construction is probeable), `Branch` (a total edge into a coproduct —
//! the kept-witness `classify`), and `Guarded` (a partial edge admitted by a witness,
//! `Construction`'s sibling). Each declares a `CAPABILITY`, so a whole path's ceiling
//! is the static join of its edges. Probes come in flavours: `probe` (residual
//! completeness), `commutes` (commutation), `coefficient_holds` (quantitative,
//! reference-bearing), and `reconstructs` / `construction_probe` (the entry-edge
//! analogs). Each module's `boundary.rs` is its only public surface; `build.rs`
//! enforces the grammar and the inward "no raw primitive escapes" rule.
//!
//! Every part of the runtime is SELF-HOSTED — certified by oracle-free probes with no
//! hand-written example tests, judged by mutation. The interpreter (`interp`) is the lead
//! demonstration substrate: an expression language whose boundary is `Parse` (a
//! `Construction`), `Check` (a `Branch`), and `Eval` (a `Guarded` edge), and whose private
//! internals carry ZERO tests of their own — the boundary rigour plus the autogen `laws`
//! registry are their entire verification. `select` is a second structural self-host: the
//! kill-matrix set-cover selector, specified in the discipline with no interior tests.
//!
//! `gdp` is the name/proof vocabulary (Ghosts of Departed Proofs): unique type-level names
//! and proofs phrased about a named value. It is load-bearing two ways — `interp`'s
//! `WellTyped`/`IllTyped` are gdp proofs, and `select`'s kernel indexes its matrix entirely
//! through gdp's `InBounds` relational proof (`positions`/`at_in_bounds`), so an out-of-range
//! read is a type error, not a panic. `capability` is the behavioural audit that reconciles
//! an edge's declared capability with what it actually does (over- and under-claim
//! detection). Both are crate-level grammar (`build.rs` exempts them from the structural
//! rules), self-hosted by replacing their example tests with oracle-free property probes.

pub mod boundary;
pub mod capability;
pub mod gdp;
pub mod interp;
pub mod select;

/// `#[derive(Shaped)]` — generate a value object's probe surface (the fused universal
/// probe's `inhabitant` + `perturbation_classes`) from its structure. The companion of the
/// `crate::boundary::Shaped` trait, re-exported here so edges write `#[derive(Shaped)]`.
pub use boundary_algebra_macros::Shaped;

#[cfg(test)]
mod laws;
#[cfg(test)]
mod tests;
