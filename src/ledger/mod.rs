//! ledger — a domain module.
//!
//! Its ONLY public surface is `ledger::boundary`: value objects, value
//! operators, and the `Morphism` instances built from them. The aggregation
//! algorithm lives in the private `internal` module and is unreachable from
//! other modules — they cannot even name it.

pub mod boundary;
mod internal;
