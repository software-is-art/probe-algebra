//! probe-algebra — a boundary-discipline experiment plus a layered probe method.
//!
//! Two constraints under study together:
//!   1. every primitive that means something in the domain is a VALUE OBJECT and
//!      every operation on it a VALUE OPERATOR — never raw primitive arithmetic
//!      at a call site; and
//!   2. a transformation is checked by a LAYERED probe suite, because no single
//!      check is highest-assurance (see `blindspot`).
//!
//! `crate::boundary` defines the grammar: a boundary is a CATEGORY of value-object
//! OBJECTS and value-operator MORPHISMS. The morphisms share one algebra in a few
//! type-distinguished shapes — `Morphism` (a total edge between value objects) and
//! `Construction` (the partial ENTRY edge from a raw primitive into a value object,
//! so even "parse, don't validate" construction is probeable) — exercised by four
//! probe flavours: `probe` (residual completeness), `commutes` (commutation),
//! `coefficient_holds` (quantitative, reference-bearing), and `reconstructs` (the
//! construction round-trip). Each module's `boundary.rs` is its only public surface;
//! `build.rs` enforces the grammar and the inward "no raw primitive escapes" rule.
//!
//! Modules:
//!   - `ledger`  — lossy worked example (aggregation + residual + mutants);
//!   - `linear`  — lossless transport carrying the decisive coefficient bug;
//!   - `select`  — kill-matrix set-cover selection (the generation+selection loop);
//!   - `synth`   — type-driven degree-of-freedom coverage / operator synthesis.

pub mod boundary;
pub mod capability;
pub mod composition;
pub mod effect;
pub mod gdp;
pub mod journal;
pub mod ledger;
pub mod lifecycle;
pub mod linear;
pub mod money;
pub mod pipeline;
pub mod select;
pub mod synth;

#[cfg(test)]
mod blindspot;
#[cfg(test)]
mod properties;
#[cfg(test)]
mod tests;
