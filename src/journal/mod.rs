//! journal — STATE as loss. Tests the claim that a stateful update is a lossy
//! morphism whose residual is the prior state it overwrote.
//!
//! A `Set` forgets the previous value; to invert it you must retain that value,
//! so its residual IS the overwritten prior. An `Add` forgets nothing (it is
//! invertible from its own amount), so its residual is `Unit`. Composing updates
//! threads the state and accumulates the priors into a nested residual — the undo
//! history / event log, built by the same `Compose` the lossy ledger uses.
//!
//! The order of a command sequence is encoded in the Compose nesting, and the
//! retention typestate gives the snapshot-vs-replay tradeoff: discard the
//! residual and `invert` (rewind) is gone at compile time.

pub mod boundary;
