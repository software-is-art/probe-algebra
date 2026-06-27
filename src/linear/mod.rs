//! linear — a LOSSLESS-transport domain whose whole purpose is the spec's
//! decisive negative result: a wrong-but-invertible coefficient bug that
//! survives EVERY structural check and dies only to a reference-bearing
//! quantitative probe.
//!
//! Unlike the ledger (lossy; exercises residuals and round-trip), `Scale` is a
//! transport: `Quantity -> Quantity` with the empty residual `Unit`. A transport
//! has nothing to forget, so the residual/round-trip axis is vacuous here and the
//! coefficient is the only thing left to get wrong — which is exactly the
//! situation the quantitative layer was built for.

pub mod boundary;
