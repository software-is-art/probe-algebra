//! probe-algebra — a boundary-discipline experiment plus a layered probe method.
//!
//! Two constraints under study together:
//!   1. every primitive that means something in the domain is a VALUE OBJECT and
//!      every operation on it a VALUE OPERATOR — never raw primitive arithmetic
//!      at a call site; and
//!   2. a transformation is checked by a LAYERED probe suite, because no single
//!      check is highest-assurance (see `blindspot`).
//!
//! `crate::boundary` defines the grammar: the sealed markers and the generic
//! `Morphism` algebra with three probe flavours — `probe` (residual completeness,
//! structural), `commutes` (commutation, structural), and `coefficient_holds`
//! (quantitative, reference-bearing). Each module's `boundary.rs` is its only
//! public surface; `build.rs` enforces the grammar and the inward "no raw
//! primitive escapes" rule.
//!
//! Modules:
//!   - `ledger`  — lossy worked example (aggregation + residual + mutants);
//!   - `linear`  — lossless transport carrying the decisive coefficient bug;
//!   - `select`  — kill-matrix set-cover selection (the generation+selection loop);
//!   - `synth`   — type-driven degree-of-freedom coverage / operator synthesis.

pub mod boundary;
pub mod ledger;
pub mod linear;
pub mod pipeline;
pub mod select;
pub mod synth;

#[cfg(test)]
mod blindspot;
#[cfg(test)]
mod properties;
#[cfg(test)]
mod tests;
