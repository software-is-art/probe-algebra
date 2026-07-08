//!
//! protocol — TYPESTATES AS SORTS: the `protocol!` demonstration, and the executable
//! answer to the bag of booleans.
//!
//! Field observation (relayed, and easy to reproduce): agents modelling a workflow
//! gravitate to FLAGS — `in_review: bool, published: bool` on one struct, every
//! operation total, every combination representable, including the incoherent ones
//! (`published && !ever_reviewed`). The classic remedy is "just use typestates," and
//! the classic gap is that typestates come with no story for the LAWS of the protocol
//! — so here is the concrete way of working with them:
//!
//! - **states are SORTS** (`protocol!` generates the sort enum and the state-tagged
//!   value enum — the state is a variant, not a flag conjunction);
//! - **an illegal transition is UNREPRESENTABLE**: no operator carries `Draft →
//!   Published`, so "approve before review" is not a bug to test for — it is a term
//!   that cannot be formed, and [`the_flag_bag_contrast`] pins the difference against
//!   a flags twin where the same call is representable, total, and silently produces
//!   the incoherent state;
//! - **a rejected transition is DEFINEDNESS**: `submit` on an empty draft returns
//!   `None`, and the engine's partial-operator convention judges laws only where the
//!   protocol admits values;
//! - **the protocol's algebra is DISCOVERED like any other**: `spec/doc-flow.spec`
//!   freezes the round-trips the workflow actually satisfies (submit/revise both
//!   ways), and the coverage line records where the spec is silent — there is no law
//!   ABOUT `approve` beyond its signature, because nothing returns from `Published`;
//!   the one-way door is visible as recorded silence.
//!
//! What stays with the compile-time floor, deliberately: the per-value proof binding
//! (`Named<N>` brands, `WellTyped<N>`) — a brand cannot inhabit a grid, and its
//! guarantee is rustc's, pinned by the `compile_fail` suite. Sorts carry the protocol;
//! brands carry the "for THIS value" — the Branch/Guarded seam, restated.

use crate::protocol;

/// The payload every state carries: a document body and its revision count.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Doc {
    pub body: String,
    pub rev: u8,
}

#[crate::mutate]
impl Doc {
    /// Mint a document.
    pub fn of(body: &str, rev: u8) -> Doc {
        Doc {
            body: body.to_string(),
            rev,
        }
    }
}

/// Submission demands substance: an empty draft is REJECTED — definedness, not a flag.
#[crate::mutate]
fn submit(d: &Doc) -> Option<Doc> {
    (!d.body.is_empty()).then(|| d.clone())
}

/// Revision sends a document back to drafting, payload intact (the round-trip's
/// other half).
#[crate::mutate]
fn revise(d: &Doc) -> Option<Doc> {
    Some(d.clone())
}

/// Approval demands at least one edit round — a document nobody touched is refused.
#[crate::mutate]
fn approve(d: &Doc) -> Option<Doc> {
    (d.rev >= 1).then(|| d.clone())
}

/// An edit bumps the revision counter (saturating: the grid stays finite through the
/// closure bound regardless).
#[crate::mutate]
fn edit(d: &Doc) -> Option<Doc> {
    Some(Doc {
        body: d.body.clone(),
        rev: d.rev.saturating_add(1),
    })
}

/// The draft seeds — the only entry point; Review and Published are populated by the
/// CLOSURE, so their inhabitants are exactly what the protocol can reach.
#[crate::mutate]
fn draft_seeds() -> Vec<Doc> {
    vec![Doc::of("", 0), Doc::of("hello", 0), Doc::of("spec", 1)]
}

protocol! {
    DocFlow : "doc flow",
    Sort = DocFlowState, Value = DocFlowValue,
    Payload = Doc, Obs = (String, u8),
    observe = |d: &Doc| (d.body.clone(), d.rev),
    states { Draft, Review, Published }
    seeds { Draft => draft_seeds(), }
    transitions {
        submit  : Draft  => Review    = submit;
        revise  : Review => Draft     = revise;
        approve : Review => Published = approve;
        edit    : Draft  => Draft     = edit;
    }
    expects {
        round_trip(submit, revise);
    }
}

#[cfg(test)]
mod probes {
    use super::*;
    use crate::discover::engine::{Engine, Theory};
    use crate::discover::expect::Distance;

    /// THE BAG-OF-BOOLEANS CONTRAST, executable. The flags twin models the same
    /// workflow as one struct with two booleans and TOTAL operations — and the
    /// incoherent history is representable, runs without complaint, and lands in a
    /// state the protocol version cannot even spell.
    #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
    struct FlagDoc {
        body: String,
        rev: u8,
        in_review: bool,
        published: bool,
    }

    fn flag_approve(d: &FlagDoc) -> FlagDoc {
        FlagDoc {
            published: true, // total: nothing checks in_review — the silent bug.
            ..d.clone()
        }
    }

    #[test]
    fn the_flag_bag_contrast() {
        // FLAGS: approving a fresh draft is representable, total, and incoherent —
        // published without ever being reviewed, and no type said no.
        let fresh = FlagDoc {
            body: "hello".into(),
            rev: 0,
            in_review: false,
            published: true == false, // false, spelled to make the reviewer wince
        };
        let broken = flag_approve(&fresh);
        assert!(
            broken.published && !broken.in_review,
            "the incoherent state, reached"
        );

        // PROTOCOL: the same call is UNREPRESENTABLE — no operator carries
        // Draft → Published, a fact of the signature table, not of a test.
        let signatures = Engine::<DocFlow>::new().signatures();
        assert!(
            !signatures
                .iter()
                .any(|(_, inputs, output)| inputs == &vec![DocFlowState::Draft]
                    && *output == DocFlowState::Published),
            "no operator may connect Draft directly to Published"
        );
        // and `approve` demands the Review sort by type: its one input IS Review.
        let approve_sig = signatures
            .iter()
            .find(|(symbol, _, _)| *symbol == "approve")
            .expect("approve is declared");
        assert_eq!(approve_sig.1, vec![DocFlowState::Review]);
    }

    /// A rejected transition is DEFINEDNESS, not a flag: the empty draft cannot enter
    /// review, the unedited document cannot be approved — `None`, judged by the
    /// engine's partial-operator convention, never a boolean somebody forgot to check.
    #[test]
    fn rejection_is_definedness() {
        assert_eq!(submit(&Doc::of("", 0)), None);
        assert_eq!(approve(&Doc::of("hello", 0)), None);
        assert!(submit(&Doc::of("hello", 0)).is_some());
        assert!(approve(&Doc::of("spec", 1)).is_some());
    }

    /// REACHABILITY IS THE GRID: Review and Published have no seeds — everything in
    /// them arrived through the protocol, so their inhabitants are exactly the
    /// reachable set (and every Review inhabitant has a non-empty body, because
    /// `submit` is the only door).
    #[test]
    fn the_closure_populates_exactly_the_reachable_states() {
        let review = DocFlow::inhabitants(DocFlowState::Review);
        assert!(!review.is_empty(), "Review is reachable");
        assert!(review.iter().all(|v| match v {
            DocFlowValue::Review(d) => !d.body.is_empty(),
            _ => false,
        }));
        let published = DocFlow::inhabitants(DocFlowState::Published);
        assert!(!published.is_empty(), "Published is reachable");
        assert!(published.iter().all(|v| match v {
            DocFlowValue::Published(d) => d.rev >= 1,
            _ => false,
        }));
    }

    /// The protocol's DISTANCE, green and exact: the declared round-trip holds and
    /// nothing else surprises (the catalog canonicalizes the submit/revise pair to
    /// one law) — pinned so the protocol's algebra cannot drift into or out of laws
    /// silently.
    #[test]
    fn the_declared_protocol_is_met() {
        assert_eq!(
            Distance::of::<DocFlow>().render(),
            "doc flow: 1 of 1 declared laws hold; no surprises"
        );
    }
}
