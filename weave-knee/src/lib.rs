//! weave-knee — the exterior engine's brick one: the weave knee, derived.
//!
//! The founding constraint of the exterior-engine candidate (docs/roadmap.md) is that
//! the unit of abstraction is what fits one agentic mind, and that TWO budgets bind:
//! the token budget bounds the context, the CONCEPT budget bounds synthesis — and the
//! concept budget binds first. This crate is the harness that turns that claim into a
//! measured constant: generate a synthetic corpus of child exteriors (identities,
//! atomic claims, cross-child relations, planted foils), ask a model to weave k of
//! them into one bounded narrative, judge the weave, and read the knee off the curve —
//! the fanout where cross-child RELATION coverage collapses while per-child CLAIM
//! coverage holds. Retrieval degrades gently; synthesis falls off a cliff. The cliff's
//! edge is the constant every other exterior-engine piece inherits.
//!
//! Everything here is deterministic except the model calls: the corpus is a pure
//! function of (fanout, seed); trials are committed evidence under `trials/`; the
//! knee spec is re-derived from committed trials by the drift gate, which never calls
//! a model. Honest frame: a sweep is a sample — the knee is evidence, not certainty.

pub mod corpus;

pub mod prompt;

pub mod record;

pub mod score;

pub mod knee;
