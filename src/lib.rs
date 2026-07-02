//! Tier: KERNEL — the trusted floor — defines/runs the format, exempt from the structural rules.
//!
//! boundary-spec — a boundary-discipline experiment plus a layered probe method.
//!
//! Two constraints under study together:
//!   1. every primitive that means something in the domain is a VALUE OBJECT and
//!      every operation on it a VALUE OPERATOR — never raw primitive arithmetic
//!      at a call site; and
//!   2. a transformation is checked by a LAYERED probe suite, because no single
//!      check is highest-assurance.
//!
//! `crate::boundary` defines the grammar: a boundary is a CATEGORY of value-object
//! OBJECTS and value-operator MORPHISMS. The morphisms share one algebra in a few
//! type-distinguished shapes — `Morphism` (a total edge between value objects),
//! `Construction` (the partial ENTRY edge from a raw primitive, so even "parse, don't
//! validate" construction is probeable), `Branch` (a total edge into a coproduct —
//! the kept-witness `classify`), and `Guarded` (a partial edge admitted by a witness,
//! `Construction`'s sibling). Each declares a `CAPABILITY`, so a whole path's ceiling
//! is the static join of its edges. Probes come in flavours: `probe` (residual
//! completeness), `commutes` (commutation), `coefficient_holds` (quantitative,
//! reference-bearing), and `reconstructs` / `construction_probe` (the entry-edge
//! analogs). A module's public surface is its BOUNDARY-tier file — the tier and the
//! enforced shape, not a filename: `interp`'s is `boundary.rs`, `kvstore`'s is
//! `store.rs`, and both carry the same discipline. `build.rs` enforces the grammar,
//! the inward "no raw primitive escapes" rule, and the NO-RATS-NEST rule: a fully
//! public function anywhere outside the ratified kernel must be attached to a
//! typestate or be operator-shaped, or it does not build.
//!
//! Every part of the runtime is SELF-HOSTED — certified by oracle-free probes with no
//! hand-written example tests, judged by mutation. The interpreter (`interp`) is the lead
//! demonstration substrate: an expression language whose boundary is `Parse` (a
//! `Construction`), `Check` (a `Branch`), and `Eval` (a `Guarded` edge). Its private internals
//! carry zero tests, and its POSITIVE surface — evaluation, parsing, type-checker
//! acceptance — is certified by the autogen `harness` registry (declared laws +
//! structure-derived probes), with only a disclosed few hand-written examples that pin the
//! grammar plumbing itself (see `tests.rs`'s inventory). `kvstore` is the first STATEFUL
//! domain under the same discipline — a TTL store whose merge monoid and tick action the
//! engine discovers, with expiry visible to the observation. `select` is a second
//! structural self-host: the kill-matrix set-cover selector, specified in the discipline with
//! no interior tests. Only the IRREDUCIBLE base is hand-written — the negative tests
//! (rejection of ill-typed / malformed input, the blind-spot map) and the grammar itself —
//! and the trust root (rustc + cargo-mutants) cannot be self-hosted away.
//!
//! `gdp` is the name/proof vocabulary (Ghosts of Departed Proofs): unique type-level names
//! and proofs phrased about a named value. It is load-bearing two ways — `interp`'s
//! `WellTyped`/`IllTyped` are gdp proofs, and `select`'s kernel indexes its matrix entirely
//! through gdp's `InBounds` relational proof (`positions` ⇒ `get`), which HOLDS the matrix's
//! borrow — an unproven or cross-matrix read is not writable at all, so no bounds check
//! survives in the kernel. `capability` is the behavioural audit that reconciles
//! an edge's declared capability with what it actually does (over- and under-claim
//! detection). Both are crate-level grammar (`build.rs` exempts them from the structural
//! rules), self-hosted by replacing their example tests with oracle-free property probes.

// Let this crate refer to itself by its package name, so the proc macros
// (`#[derive(Shaped)]`, `#[algebra]`) can emit `::boundary_spec::…` paths that resolve
// identically here and in a DOWNSTREAM crate — no consumer re-export shim needed.
extern crate self as boundary_spec;

pub mod boundary;
pub mod capability;
pub mod discover;
pub mod gdp;
pub mod interp;
/// The first STATEFUL domain: a TTL key-value store whose clock only moves via an explicit `Tick` edge.
pub mod kvstore;
pub mod select;

/// `#[derive(Shaped)]` — generate a value object's probe surface (the fused universal
/// probe's `inhabitant` + `perturbation_classes`) from its structure. The companion of the
/// `crate::boundary::Shaped` trait, re-exported here so edges write `#[derive(Shaped)]`.
pub use boundary_spec_macros::Shaped;

/// `#[algebra(Marker, "name")]` — generate a WHOLE discovery `Theory` from a module of ordinary
/// operator functions (no `theory!` block). The macro reads each function's signature and emits the
/// operator table, sort, `sort_of`, identity `observe`, and the shadow grid; the agent writes only
/// the value object and the operator functions. See `discover::derived`.
pub use boundary_spec_macros::algebra;

#[cfg(test)]
mod harness;
#[cfg(test)]
mod tests;
