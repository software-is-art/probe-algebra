//!
//! verbs — THE CHANGE-HISTORY AS A THEORY (the reflog-algebra candidate, built): the
//! bundle's verbs modelled as operators over a miniature bundle-state carrier, handed to
//! the same engine as every other theory, so the FORK/JOIN RULES of the whole system are
//! DISCOVERED, frozen, and mutation-tested instead of designed. A green `commuting maps`
//! instance is a join rule (these two changes merge in either order); a refuted one is a
//! CONFLICT CLASS, named by the engine and kept refused the way the router's
//! non-commutativity is kept refused — load-bearing, not a failure.
//!
//! The miniature, per the stream-carrier lesson (deliberate histories, not soup): TWO
//! named items (`a`, `b`), each absent or present at one of two versions (edit saturates
//! at V1 — the version bound is the grid bound, disclosed), plus a contract flag. The
//! verbs carry the CLI's refusal semantics as total functions: a refused verb returns the
//! state unchanged — which is exactly why every verb comes back IDEMPOTENT (journal
//! replay is safe by law, not by luck).
//!
//! Honest frame, inherited and specific: this miniature proves the algebra's SHAPE — the
//! real journal's items are richer, so the frozen lock is evidence about the DESIGN of
//! fork/join, not a proof about an implementation; the seam between this carrier and a
//! real join verb is a transport seam, judged when that verb exists.

/// The verb algebra's carrier and operators — a plain `#[algebra]` bag, like
/// `modularize::soup`: the theory is synthesized from the functions below, nothing about
/// the algebra is declared, and discovery finds the join rules and the conflicts itself.
#[crate::algebra(VerbAlgebra, "verb algebra")]
pub mod state {
    use crate::Shaped;

    /// One named item's state: absent, or present at a version (V1 is the saturation
    /// bound — the miniature's disclosed edge).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Shaped)]
    pub enum Slot {
        Absent,
        V0,
        V1,
    }

    /// Whether the module carries a declared contract.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Shaped)]
    pub enum Contract {
        Open,
        Declared,
    }

    /// The miniature bundle state: two named items and the contract flag — 18
    /// inhabitants, every verb a total function over them.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Shaped)]
    pub struct BundleState {
        pub a: Slot,
        pub b: Slot,
        pub contract: Contract,
    }

    /// The empty module — the state `bundle add` grows from (birth as the degenerate
    /// case of continuation).
    pub fn empty() -> BundleState {
        BundleState {
            a: Slot::Absent,
            b: Slot::Absent,
            contract: Contract::Open,
        }
    }

    /// `bundle add a`: lands only where `a` is absent — the collision refusal returns
    /// the state unchanged, exactly as the CLI writes nothing.
    pub fn add_a(s: BundleState) -> BundleState {
        match s.a {
            Slot::Absent => BundleState { a: Slot::V0, ..s },
            _ => s,
        }
    }

    /// `bundle add b` — `add_a`'s twin on the other name.
    pub fn add_b(s: BundleState) -> BundleState {
        match s.b {
            Slot::Absent => BundleState { b: Slot::V0, ..s },
            _ => s,
        }
    }

    /// `bundle edit a`: moves a present item to the next version (saturating at the
    /// miniature's bound); editing an absent item refuses — unchanged.
    pub fn edit_a(s: BundleState) -> BundleState {
        match s.a {
            Slot::Absent => s,
            Slot::V0 | Slot::V1 => BundleState { a: Slot::V1, ..s },
        }
    }

    /// `bundle edit b` — `edit_a`'s twin.
    pub fn edit_b(s: BundleState) -> BundleState {
        match s.b {
            Slot::Absent => s,
            Slot::V0 | Slot::V1 => BundleState { b: Slot::V1, ..s },
        }
    }

    /// `bundle collect a`: the sweep — a present item is un-materialized; collecting an
    /// absent item is honestly nothing.
    pub fn collect_a(s: BundleState) -> BundleState {
        BundleState {
            a: Slot::Absent,
            ..s
        }
    }

    /// `bundle collect b` — `collect_a`'s twin.
    pub fn collect_b(s: BundleState) -> BundleState {
        BundleState {
            b: Slot::Absent,
            ..s
        }
    }

    /// `bundle declare`: the contract flag rises — monotone, item-blind, and therefore
    /// expected to commute with everything (the declaration is the SHOULD half; it
    /// touches no item).
    pub fn declare(s: BundleState) -> BundleState {
        BundleState {
            contract: Contract::Declared,
            ..s
        }
    }
}

#[cfg(test)]
mod probes {
    use super::state::VerbAlgebra;
    use crate::discover::expect::Expectation;
    use crate::discover::Spec;

    /// THE HEADLINE: the fork/join rules of the verb set are DISCOVERED. Every cross-name
    /// pair commutes (disjoint changes merge in either order), `declare` commutes with
    /// everything (the contract touches no item), `collect` absorbs `edit` on the same
    /// name (an edit before a sweep is no conflict — the state forgets it; the journal
    /// still remembers) — and the two SAME-NAME conflicts are exactly the ones the
    /// engine REFUSES to state: add/edit (you cannot edit what is not yet added) and
    /// add/collect (a birth and a burial do not reorder). The conflict classes of the
    /// version-control model, found by the law engine, with a can-fail proof.
    #[test]
    fn the_join_rules_are_discovered_and_the_conflicts_are_refused() {
        let spec = Spec::of::<VerbAlgebra>();
        let equations: Vec<&str> = spec.laws.iter().map(|l| l.equation()).collect();

        // the join rules: disjoint names commute, in canonical name order.
        for (f, g) in [
            ("add_a", "add_b"),
            ("add_a", "edit_b"),
            ("add_b", "edit_a"),
            ("add_a", "collect_b"),
            ("add_b", "collect_a"),
            ("edit_a", "edit_b"),
            ("collect_a", "collect_b"),
            ("collect_a", "declare"),
            ("add_a", "declare"),
        ] {
            assert!(
                equations.contains(&format!("{f}({g}(x)) = {g}({f}(x))").as_str())
                    || equations.contains(&format!("{g}({f}(x)) = {f}({g}(x))").as_str()),
                "`{f}`/`{g}` should be a discovered join rule; laws: {equations:#?}"
            );
        }

        // the CONFLICT CLASSES: same-name add/edit and add/collect must NOT commute —
        // the refusal is load-bearing (the router precedent): if the engine ever starts
        // asserting these, the conflict semantics of the whole design have silently
        // changed, and this pin fires.
        for (f, g) in [("add_a", "edit_a"), ("add_a", "collect_a")] {
            assert!(
                !equations.contains(&format!("{f}({g}(x)) = {g}({f}(x))").as_str())
                    && !equations.contains(&format!("{g}({f}(x)) = {f}({g}(x))").as_str()),
                "`{f}`/`{g}` is a conflict class — it must stay refused"
            );
        }

        // collect ABSORBS edit on the same name: the state forgets the edit, so the pair
        // commutes — the journal is what remembers, which is its whole job.
        assert!(
            equations.contains(&"collect_a(edit_a(x)) = edit_a(collect_a(x))")
                || equations.contains(&"edit_a(collect_a(x)) = collect_a(edit_a(x))"),
            "collect/edit same-name commute (the absorption finding); laws: {equations:#?}"
        );
    }

    /// THE SQUASH TABLE, discovered (the composition stanza's named consumer): which verb
    /// sequences collapse to a single verb — journal compaction as law. Add-then-collect,
    /// edit-then-collect, and collect-then-edit all collapse to collect; a future `squash`
    /// consults these lock lines as data, the join-verb precedent. And the stanza closed a
    /// sensitivity gap on arrival: the four ratified edit/collect confusion survivors —
    /// invisible while both were merely projections with identical partners — die under
    /// the composition laws, so the verb algebra's mutation lock reads ALL KILLED.
    #[test]
    fn the_squash_table_is_discovered() {
        let spec = Spec::of::<VerbAlgebra>();
        let equations: Vec<&str> = spec.laws.iter().map(|l| l.equation()).collect();
        for squash in [
            "collect_a(add_a(x)) = collect_a(x)",
            "collect_a(edit_a(x)) = collect_a(x)",
            "edit_a(collect_a(x)) = collect_a(x)",
            "collect_b(add_b(x)) = collect_b(x)",
            "collect_b(edit_b(x)) = collect_b(x)",
            "edit_b(collect_b(x)) = collect_b(x)",
        ] {
            assert!(
                equations.contains(&squash),
                "`{squash}` should be a discovered squash rule; laws: {equations:#?}"
            );
        }
    }

    /// REPLAY SAFETY IS A LAW: every verb is idempotent — the refusal semantics (a
    /// refused verb returns the state unchanged) make re-applying a journal segment
    /// harmless by algebra, not by luck. Discovery states it as projection laws.
    #[test]
    fn every_verb_is_a_projection_so_replay_is_safe() {
        let spec = Spec::of::<VerbAlgebra>();
        let proses: Vec<&str> = spec.laws.iter().map(|l| l.prose()).collect();
        for verb in [
            "add_a",
            "add_b",
            "edit_a",
            "edit_b",
            "collect_a",
            "collect_b",
            "declare",
        ] {
            assert!(
                proses.contains(
                    &format!("{verb} is a projection — applying it twice is applying it once.")
                        .as_str()
                ),
                "`{verb}` must be idempotent for replay safety; laws: {proses:#?}"
            );
        }
    }

    /// The distance voice works here too: DECLARE the two conflict classes as commuting
    /// and the gap is named — the red lock pointed at the merge semantics themselves
    /// (a design change that made add/edit commute would show as this distance closing,
    /// which is exactly the review that change deserves).
    #[test]
    fn declaring_a_conflict_away_reads_as_distance() {
        use crate::discover::expect::{Distance, Expected};
        struct WishfulMerge;
        impl crate::discover::engine::Theory for WishfulMerge {
            type Sort = <VerbAlgebra as crate::discover::engine::Theory>::Sort;
            type Value = <VerbAlgebra as crate::discover::engine::Theory>::Value;
            type Obs = <VerbAlgebra as crate::discover::engine::Theory>::Obs;
            fn name() -> &'static str {
                "wishful merge"
            }
            fn operators() -> Vec<crate::discover::engine::Operator<Self>> {
                <VerbAlgebra as crate::discover::engine::Theory>::operators()
                    .into_iter()
                    .map(|op| crate::discover::engine::Operator {
                        name: op.name,
                        symbol: op.symbol,
                        fixity: op.fixity,
                        inputs: op.inputs,
                        output: op.output,
                        eval: op.eval,
                    })
                    .collect()
            }
            fn inhabitants(
                sort: Self::Sort,
            ) -> Vec<<VerbAlgebra as crate::discover::engine::Theory>::Value> {
                <VerbAlgebra as crate::discover::engine::Theory>::inhabitants(sort)
            }
            fn sort_of(v: &Self::Value) -> Self::Sort {
                <VerbAlgebra as crate::discover::engine::Theory>::sort_of(v)
            }
            fn observe(v: &Self::Value) -> Self::Obs {
                <VerbAlgebra as crate::discover::engine::Theory>::observe(v)
            }
        }
        impl Expected for WishfulMerge {
            fn expectations() -> Vec<Expectation> {
                vec![Expectation::of("commuting maps", vec!["add_a", "edit_a"])]
            }
        }
        let d = Distance::of::<WishfulMerge>();
        assert_eq!(d.missing.len(), 1, "the conflict cannot be declared away");
        assert!(d.missing[0].render().contains("add_a"));
    }
}
