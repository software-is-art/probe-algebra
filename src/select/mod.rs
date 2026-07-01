//! Tier: INTERIOR — the workshop / leaves (tier 2 inward rule).
//!
//! select — the kill-matrix selector, brought under the boundary discipline it serves.
//!
//! `select` is part of the method's own kernel: given the kill matrix it picks the minimal,
//! attributable relation suite. This module SELF-HOSTS it — its public surface is
//! `select::boundary` (the `KillMatrix` value object and its operators), the greedy
//! algorithm lives in the private `internal`, and `internal` carries no example tests, so
//! the mutation sweep measures whether the method's own oracle-free probes certify the
//! component that chooses the method's probes.

pub mod boundary;
mod internal;
