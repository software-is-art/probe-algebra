//! The crate's trusted floor: the module roster and the macro re-export shim. KERNEL is
//! a REGISTRATION in this crate's own build.rs, never an assertion in a file.
//!
//! # downstream-fixture — the consumer tutorial, as a crate that must keep compiling
//!
//! This crate is the EXISTENCE PROOF that a consumer of `boundary-spec` can run the whole
//! discipline — tier partition, boundary grammar, discovered laws, frozen spec, drift gate —
//! through PUBLIC API alone. It is deliberately tiny; copy its shape into your own repo piece by
//! piece. Every piece below names why it exists; the discipline itself is documented in the
//! library's `docs/ci-discipline.md` (see especially its "upgrade contract" section for what a
//! library upgrade may and may not do to your committed lock).
//!
//! ## The domain (small on purpose): a saturating credit meter
//!
//! One value object, `Credits` — a balance validated into `0..=20` — and three operators:
//!
//! * `grant`  — saturating top-up, capped at the ceiling;
//! * `spend`  — saturating deduction, floored at zero;
//! * `renew`  — replace the balance with a voucher, except that a zero voucher is a no-op.
//!
//! Chosen so the discovered spec's REFUSALS carry information: `grant` is a commutative
//! monoid (`zero` identity), `renew` is a non-commutative band whose BIAS the sandwich law
//! states ("the later operand wins"), and `spend` is directional — the spec grants it only
//! its two `zero` laws and refuses commutativity and associativity, because the order of
//! deductions is semantics.
//!
//! ## The pieces, and why each exists
//!
//! * **`build.rs`** — the enforcement shim: attaches `boundary-enforce` with THIS crate's own
//!   kernel allowlist and its own drift-gated `spec/qualify.spec` and `spec/tiers.spec`. Every
//!   source file's tier is DERIVED from structure (reachability, doors, glue) and carries that
//!   tier's rules, exactly as in the library's own tree.
//! * **[`meter`]** (BOUNDARY — derived: pub-reachable, carries production edges) — the strict
//!   value-object surface: `Credits`, its validity
//!   rule, and the operator methods. The tier-1 grammar (no free functions, no public fields,
//!   no I/O, primitives only as lone newtype fields) is enforced on it at build time.
//! * **`internal`** (INTERIOR — derived: not pub-reachable) — the workshop the boundary
//!   delegates to. The
//!   inward rule holds here: no function returns a raw primitive, so a domain quantity cannot
//!   leak out un-typed.
//! * **[`ops`]** (ALGEBRA — derived: the reachable remainder) — the minimal authoring path:
//!   `#[algebra]` over three
//!   ordinary operator functions synthesises the whole discovery `Theory` (operator table,
//!   sort, shadow-derived grid). Nothing about the algebra is declared; it is discovered by
//!   running these functions.
//! * **`spec/credit-meter.spec`** — the committed behaviour lock, written by
//!   `Spec::of::<CreditMeter>().lock_in(<our spec dir>)` + `spec_lock::bless`.
//! * **`tests/freeze_gate.rs`** — the drift gate: a plain test re-derives the live spec and
//!   fails if it differs from the committed file. The fix is never to hand-edit the lock —
//!   regenerate and ratify the diff in review.
//! * **`examples/freeze.rs`** — the bless path (`cargo run -p downstream-fixture --example
//!   freeze`): the ONE sanctioned writer of the lock file. Idempotent — run it twice, the
//!   second run changes nothing.
//!
//! ## No shim required (a finding this fixture surfaced, now fixed)
//!
//! The `#[algebra]` and `#[derive(Shaped)]` proc-macros once expanded to `crate::…` paths and
//! forced every consumer to re-export the library's modules at its own root. The library now
//! aliases itself (`extern crate self as boundary_spec;`) and the macros emit
//! `::boundary_spec::…`, which resolves identically in the library and in any consumer that
//! depends on it under its package name — so this crate uses `#[algebra]` directly, with no
//! re-export. (If you rename the dependency in your Cargo.toml, restore the alias with
//! `extern crate boundary_spec as <your-name>;` — or just don't rename it.)
//!
//! ## Edges work downstream too
//!
//! [`meter`] mints three of the four edge shapes, through public API alone:
//!
//! * **`ParseCredits`** — a `Construction`, the entry edge: "parse, don't validate" with a
//!   certified round trip, registered with the library's own `value_object!` /
//!   `value_operator!` macros (public since the citizen/effect seal split).
//! * **`CheckFunds`** — a `Branch`, the classifier: a named order lands in
//!   `Affordable<N> + Insufficient<N>`, both arms `proof_token!`-minted witnesses (the macro
//!   is `$crate`-hygienic, so it mints downstream unchanged), the refusal first-class.
//! * **`Deduct`** — a `Guarded` edge whose `Proof<N>` is `Affordable<N>`: the deduction is
//!   UNCALLABLE without `CheckFunds`' witness for the SAME brand, so "you forgot the funds
//!   check" is a compile error in this consumer's tree. `tests/compile_fail.rs` (trybuild)
//!   pins the negatives: the witness cannot be forged, the wrong arm does not discharge,
//!   and a proof for one order cannot deduct another.
//!
//! Every edge carries its `impl Probed`, which our build.rs's edge-probe completeness pass
//! enforces exactly as the library's does — delete one and this crate stops building.
//! Honestly incomplete: the fourth shape, `Morphism`, is not exercised here (the meter has
//! no lossy/stateful transformation to model); the library's `ConstFold` / `Resolve` remain
//! its reference instances. Only the four-level effect lattice remains sealed, because its
//! laws are proven exhaustively over exactly those four levels.

pub mod meter;
pub mod ops;

mod internal;
