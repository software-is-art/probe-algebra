//! Tier: KERNEL — the crate's trusted floor: the module roster and the macro re-export shim.
//!
//! # downstream-fixture — the consumer tutorial, as a crate that must keep compiling
//!
//! This crate is the EXISTENCE PROOF that a consumer of `boundary-algebra` can run the whole
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
//! Chosen so the discovered spec's SILENCES carry information: `grant` is a commutative
//! semigroup, `renew` is a non-commutative band whose BIAS the sandwich law states ("the later
//! operand wins"), and `spend` satisfies no universal shape at all — the engine refuses laws for
//! it, and the lock's coverage line names it as the place human attention belongs.
//!
//! ## The pieces, and why each exists
//!
//! * **`build.rs`** — the enforcement shim: attaches `boundary-enforce` with THIS crate's own
//!   kernel allowlist and its own drift-gated `spec/qualify.spec`. From then on every source
//!   file must declare a `//! Tier:` marker and carry that tier's rules, exactly as in the
//!   library's own tree.
//! * **[`meter`]** (`Tier: BOUNDARY`) — the strict value-object surface: `Credits`, its validity
//!   rule, and the operator methods. The tier-1 grammar (no free functions, no public fields,
//!   no I/O, primitives only as lone newtype fields) is enforced on it at build time.
//! * **`internal`** (`Tier: INTERIOR`, private) — the workshop the boundary delegates to. The
//!   inward rule holds here: no function returns a raw primitive, so a domain quantity cannot
//!   leak out un-typed.
//! * **[`ops`]** (`Tier: ALGEBRA`) — the minimal authoring path: `#[algebra]` over three
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
//! ## The re-export shim below (read this before copying)
//!
//! The `#[algebra]` proc-macro expands to paths spelled `crate::discover::engine::…` — hardcoded
//! against the library's own module tree, since a proc-macro has no `$crate`. In a consumer
//! crate `crate::` is the CONSUMER, so the expansion only resolves if the consumer re-exports
//! the library's `discover` module at its own crate root:
//!
//! ```ignore
//! pub use boundary_algebra::discover;   // required by #[algebra]'s expansion
//! ```
//!
//! That is what the `pub use` below is — a wart this fixture documents rather than hides (the
//! declarative macros `theory!` / `refined!` use `$crate` and need no shim; only the
//! proc-macros carry this obligation). If you would rather not re-export, hand-write the
//! `theory!` block instead — it is the same authoring surface, one explicit step down.

/// The macro shim: `#[algebra]`'s generated code resolves `crate::discover::…` through this
/// re-export. See the crate docs ("The re-export shim") for why a consumer needs it.
pub use boundary_algebra::discover;

pub mod meter;
pub mod ops;

mod internal;
