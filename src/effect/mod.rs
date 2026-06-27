//! effect — EFFECT internalized as a pure morphism relative to a handler.
//!
//! A raw effect touches a world the morphism does not own, so it is neither pure
//! nor loss. But you can internalize the *interface* to that world: declare what
//! it READS as part of the input (a fulfilled `Demand` — a `Clock` reading) and
//! what it WRITES as the residual (an `Emission` — an `Entry`). Then the operator
//! `Stamp` is a perfectly pure `Morphism` over `(Message, Clock) -> (Stamped,
//! Entry)`, and the probe applies to it.
//!
//! The `handler` decides where the reading comes from and where the emission
//! goes. A `RecordingHandler` scripts the reading and captures the emission as
//! data — keeping everything pure and probeable. A live handler (real clock, real
//! log) would do the actual I/O; that is the single seam this method cannot probe
//! — the program edge — and the boundary discipline already forbids I/O here, so
//! it must live outside.

pub mod boundary;
pub mod handler;
