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
//! The interpreter (`interp`) is the sole demonstration substrate: an expression
//! language whose boundary is `Parse` (a `Construction`), `Check` (a `Branch`), and
//! `Eval` (a `Guarded` edge), and whose private internals carry ZERO tests of their own —
//! the boundary rigour plus the autogen `laws` registry are their entire verification,
//! measured by mutation. `gdp` is the name-branding machinery the edges use; `capability`
//! is the behavioural audit that reconciles an edge's declared capability with what it
//! actually does (over- and under-claim detection); `select` is the kill-matrix set-cover
//! meta-tooling.

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
