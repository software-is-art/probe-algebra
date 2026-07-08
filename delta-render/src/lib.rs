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
//! rule table itself; the end gate is defense in depth, not a verifier. And the end
//! gate is RELATIVE — batch and incremental share the operator implementations — so
//! the absolute referents are the hand-pinned batch table, the per-operator probes,
//! and (declared, not built) the SQL-emulator oracle slot.
//!
//! Deliberate deviation, ratified here rather than by omission: this crate attaches no
//! `boundary-enforce` build discipline (no tier grammar, no qualify census) — the
//! fire-drill precedent for research members. Its discipline is the locks: every spec,
//! the registry, both rendered circuits, and the mutation verdicts are drift-gated
//! from BOTH the integration tests and the library probes (the latter so the mutation
//! sweeps can see them).

pub mod circuit;
pub mod license;
pub mod ops;
pub mod stream;
pub mod warrant;
pub mod zset;

/// The RENDERED incremental circuit — generated code, compiled and tested like any
/// source. Its bytes are a lock: the drift gate re-renders and diffs it inside every
/// `cargo test`, so a hand edit here is caught, never merged quietly.
#[path = "../gen/demo_incremental.rs"]
pub mod demo_incremental;

/// The audit circuit's rendered form — two sources, a fan-out, and the
/// non-commutative bilinear, all through the same lock discipline.
#[path = "../gen/audit_incremental.rs"]
pub mod audit_incremental;

#[cfg(test)]
mod lock_probes {
    //! The LIB-SIDE twins of `tests/freeze_gate.rs` — duplicated ON PURPOSE: the
    //! mutation sweeps judge library mutants against lib tests only, so every lock a
    //! mutant could silently invalidate must be re-derivable from here. A perturbed
    //! theory (its name, operators, grid, observation, variables, or sampling budget)
    //! discovers a different law set, and these gates catch that as spec drift — the
    //! same freshness semantics the algebra-mutation harness applies in-process.

    use std::path::PathBuf;

    use boundary_spec::discover::engine::Theory;
    use boundary_spec::discover::mutation::MutationReport;
    use boundary_spec::discover::Spec;

    fn spec_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("spec")
    }

    fn check_theory<T>()
    where
        T: Theory + 'static,
        T::Value: 'static,
        T::Obs: std::fmt::Debug + 'static,
    {
        let locks = [
            Spec::of::<T>().lock_in(&spec_dir()),
            MutationReport::of::<T>().lock_in(&spec_dir()),
        ];
        if let Err(stale) = spec_lock::check(&locks) {
            panic!(
                "lock drifted for: {}. \
                 Run `cargo run -p delta-render --example freeze` and ratify the diff.",
                stale.join(", ")
            );
        }
    }

    /// Every theory's committed spec and mutation verdict, fresh from the library side.
    #[test]
    fn every_committed_spec_and_mutation_verdict_is_fresh_from_the_library_side() {
        check_theory::<crate::zset::ZSetAlgebra>();
        check_theory::<crate::ops::FilterOp>();
        check_theory::<crate::ops::MapOp>();
        check_theory::<crate::ops::SumOp>();
        check_theory::<crate::ops::JoinOp>();
        check_theory::<crate::ops::ScaleOp>();
        check_theory::<crate::ops::DistinctOp>();
        check_theory::<crate::ops::MinOp>();
        check_theory::<crate::stream::StreamCalculus>();
    }

    /// The min-retraction witness, fresh from the library side (its values are computed
    /// by the real operators, so a drifted `min` flips this lock too).
    #[test]
    fn the_min_retraction_witness_is_fresh_from_the_library_side() {
        let lock = spec_lock::Lock {
            name: "min retraction witness".into(),
            path: spec_dir().join("min.retraction.spec"),
            live: crate::ops::min_retraction_witness(),
        };
        if let Err(stale) = spec_lock::check(std::slice::from_ref(&lock)) {
            panic!(
                "the min-retraction witness drifted: {}. Regenerate \
                 (`cargo run -p delta-render --example freeze`) and ratify the diff.",
                stale.join(", ")
            );
        }
    }
}
