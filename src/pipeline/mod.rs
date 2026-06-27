//! pipeline — a NESTED module: a parent boundary built by composing two PRIVATE
//! child boundaries. It exists to test whether the discipline RECURSES — whether
//! "one place to look" and narrowing survive a module-of-modules.
//!
//! The children (`calibrate`, `bucketize`) are declared PRIVATE here, so nothing
//! outside `pipeline` can name them or their citizens. The only public surface is
//! `pipeline::boundary`, which exposes ONE operator (`Ingest`) whose `Morphism`
//! signature mentions only parent-owned types and grammar types — the child's
//! intermediate `Reading` appears NOWHERE in it. That is narrowing enforced by
//! Rust visibility: the parent re-exports nothing (the build forbids it) and
//! instead DEFINES a citizen that delegates inward.

pub mod boundary;
mod bucketize;
mod calibrate;
