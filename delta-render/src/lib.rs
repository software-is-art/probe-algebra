//! delta-render — specs as LICENSES, DBSP as a rendered lock.
//!
//! A miniature of DBSP-style incremental computation in which the derivation of the
//! incremental program is GENERATED code whose licenses are discovered law specs. Each
//! operator's classification (linear / bilinear / neither) is not declared by hand — it
//! is the presence or absence of specific laws in that operator's frozen spec:
//!
//! - LINEAR ⇔ the spec says the operator is an additive homomorphism over the Z-set
//!   group AND preserves zero (`{op} turns plus into plus.` + `{op} leaves zero
//!   fixed.`);
//! - BILINEAR ⇔ additive in each argument slot (the distributivity pair, or one
//!   distributivity law plus commutativity);
//! - NEITHER ⇔ the licenses above are absent; the generic fallback (`D ∘ Q ∘ I`) still
//!   applies — correct always, cheap never.
//!
//! A render step walks a declared operator DAG, applies the derivation rule each node's
//! classification LICENSES, and emits the incremental circuit as a committed,
//! drift-gated artifact — the same move as `ci.yml`-as-rendered-lock, at a new
//! altitude: discovery output consumed as generation input. An independent end-to-end
//! round-trip law (`I ∘ Q^Δ ∘ D = Q` over the stream grid) gates the product regardless
//! of the licenses, and fire-drill fixtures prove both the classifier and the end gate
//! can fire.
//!
//! Honest frame, inherited: licenses are DISCOVERED — bounded refutation over a
//! deliberate grid — never proofs. The DBSP paper's theorems are the trust root for the
//! rule table itself; the end gate is defense in depth, not a verifier.

pub mod circuit;
pub mod license;
pub mod ops;
pub mod stream;
pub mod zset;

/// The RENDERED incremental circuit — generated code, compiled and tested like any
/// source. Its bytes are a lock: the drift gate re-renders and diffs it inside every
/// `cargo test`, so a hand edit here is caught, never merged quietly.
#[path = "../gen/demo_incremental.rs"]
pub mod demo_incremental;
