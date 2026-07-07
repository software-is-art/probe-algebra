//! layout-probe — the second domain, in miniature: a diagram layout engine under
//! metamorphic probe (roadmap candidate 13).
//!
//! One operator vocabulary over three sorts (sources, geometries, renames); two
//! deterministic layered engines differing by a single policy bit (within-rank order:
//! node name vs declaration index); geometry as the OBSERVATION. With that choice the
//! scoped metamorphic relations are ordinary discovery:
//!
//! - "declaration reorder must not move the layout" is the `inert` catalog shape —
//!   held by the stable engine, REFUSED for the eager one (dagre's agent-loop
//!   "layout jumping" pathology, reproduced honestly);
//! - "rename-then-render = render-then-relabel" is the `equivariant map` square —
//!   held by the eager engine, REFUSED for the stable one (the name is load-bearing
//!   for position);
//! - the two engines DECLARE the same laws, so their distance reports are directly
//!   comparable: the pinned pair is the ENGINE SCORECARD, a real tradeoff named by a
//!   missing law rather than debated.
//!
//! Beside the specs: the VISUAL CENSUS (`spec/visual.census`) — the lint layer's
//! constraints EMERGING from the ratified corpus as freeze-stable floors, plus the
//! locality witness (insertion displacement, measured by the real engines). Deliberate
//! deviation, delta-render's precedent: no `boundary-enforce` discipline; the locks
//! are the discipline, drift-gated from the lib so the member's mutation sweep can see
//! them. The ELK/d2 binding stays the downstream adoption; this member validates fit.

pub mod census;
pub mod diagram;
pub mod layout;
pub mod theories;

#[cfg(test)]
mod lock_probes {
    //! The lib-side drift gates — HERE and not only in `tests/`, because the member's
    //! mutation sweep judges lib mutants against lib tests only (the delta-render
    //! lesson): every lock a mutant could silently invalidate re-derives from here.

    use std::path::PathBuf;

    use boundary_spec::discover::engine::Theory;
    use boundary_spec::discover::mutation::MutationReport;
    use boundary_spec::discover::Spec;

    use crate::theories::{EagerLayout, StableLayout};

    fn spec_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("spec")
    }

    fn check_theory<T: Theory>() {
        let locks = [
            Spec::of::<T>().lock_in(&spec_dir()),
            MutationReport::of::<T>().lock_in(&spec_dir()),
        ];
        if let Err(stale) = spec_lock::check(&locks) {
            panic!(
                "lock drifted for: {}. \
                 Run `cargo run -p layout-probe --example freeze` and ratify the diff.",
                stale.join(", ")
            );
        }
    }

    /// Both engines' committed specs and mutation verdicts, fresh from the library side.
    #[test]
    fn every_committed_spec_and_mutation_verdict_is_fresh_from_the_library_side() {
        check_theory::<StableLayout>();
        check_theory::<EagerLayout>();
    }

    /// The visual census (and its embedded locality witness), fresh from the library
    /// side — the emergent constraints cannot rot silently.
    #[test]
    fn the_committed_visual_census_is_fresh_from_the_library_side() {
        let lock = crate::census::lock_in(&spec_dir());
        if let Err(stale) = spec_lock::check(std::slice::from_ref(&lock)) {
            panic!(
                "the visual census drifted: {}. Regenerate \
                 (`cargo run -p layout-probe --example freeze`) and ratify the diff.",
                stale.join(", ")
            );
        }
    }
}
