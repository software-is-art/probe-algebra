//! probe-algebra — a boundary-discipline experiment.
//!
//! The constraint under study: every primitive that means something in the
//! domain must be a VALUE OBJECT, and every operation on it a VALUE OPERATOR —
//! never raw primitive arithmetic at a call site. `crate::boundary` defines the
//! grammar (the sealed markers + the generic morphism/probe algebra); each
//! module's `boundary.rs` is its only public surface; `build.rs` enforces both
//! the boundary grammar and the inward "no raw primitive escapes" rule.

pub mod boundary;
pub mod ledger;
pub mod linear;

#[cfg(test)]
mod blindspot;
#[cfg(test)]
mod properties;
#[cfg(test)]
mod tests;
