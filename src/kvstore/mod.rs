//!
//! kvstore — a key-value store with TTL expiry: the crate's first STATEFUL domain.
//!
//! Every earlier substrate (the interpreter, the router, the date calculus) is pure: an
//! edge's output is a function of its input and nothing ever *happens*. This module tests
//! whether the discipline transfers off that friendly path — to a domain whose whole point
//! is carried state that DECAYS. The answer the module stakes out: state is just another
//! value object. The `Store` carries its entries AND its own logical clock; time advances
//! only through the explicit `Tick` edge, so every run is deterministic and the domain
//! stays `Stateful` without ever becoming `Effectful` (no real clock, no I/O).
//!
//! Its ONLY public surface is `kvstore::store` (deliberately NOT named `boundary.rs` — see
//! its header): the value objects (`Key`, `Val`, `Ttl`, `Clock`, `Store`, …), the proof
//! tokens (`Live`/`Gone`), and the edges built from them — `ParseKey` (a `Construction`),
//! `Put` and `Tick` (`Stateful` `Morphism`s with load-bearing residuals), and the
//! `Get`/`Read` pair (a `Branch` minting a liveness witness that the `Guarded` read
//! demands, mirroring `Check`/`Eval`). The imperative entry storage and expiry sweep live
//! in the private `internal` module, which — like `interp::internal` — carries ZERO tests
//! and is kept in the mutation sweep, so the sweep measures what the boundary rigour buys.
//!
//! Verification is three-layered, none of it example tables: the per-edge oracle-free
//! probes in `probes` (two-route laws, residual round-trips, an independent recomputation
//! of the sweep), the DISCOVERED algebra in `theory` (the merge monoid and the tick
//! ACTION of durations on stores, found by the generic engine and pinned as a golden
//! spec), and the disclosed hand-written negative base (rejection of malformed keys,
//! expired reads classifying as `Gone`) also in `probes`.

pub mod store;
pub mod theory;

mod internal;

// The per-module probe registry. The file itself carries no `cfg` markers — it is made
// test-only HERE, the same trick `lib.rs` plays with `mod harness`.
#[cfg(test)]
mod probes;
