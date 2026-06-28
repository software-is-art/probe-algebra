//! interp — a tiny expression-language interpreter: the crate's "cold" use case.
//!
//! Its ONLY public surface is `interp::boundary`: the value objects (`Expr`, `Value`,
//! …), the proof tokens (`WellTyped`/`IllTyped`), and the three boundary edges built
//! from them — `Parse` (a `Construction`), `Check` (a `Branch`), and `Eval` (a
//! `Guarded` edge that is uncallable without `Check`'s witness). The lexer, parser, type
//! checker, and evaluator live in the private `internal` module.
//!
//! This module exists to test two things the rest of the crate only asserts:
//!   1. that the grammar survives contact with a domain it was NOT designed around; and
//!   2. that the boundary rigour BUYS internal freedom — `internal.rs` has zero tests of
//!      its own, yet stays in the mutation sweep so the bought correctness is measurable.

pub mod boundary;
mod internal;
