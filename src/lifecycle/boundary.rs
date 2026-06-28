//! lifecycle::boundary — a ledger entry's lifecycle as a NON-LINEAR state machine.
//!
//! This module is the architectural reading of the whole crate made explicit: a
//! boundary IS a state machine. Its STATES are value-object types (here, the same
//! `Transaction` phantom-indexed by where it sits), its TRANSITIONS are the value
//! operators, and the type graph makes an illegal transition UNCALLABLE. The
//! `ledger` boundary already works this way over distinct value-object types
//! (`Round: Summary -> Summary` cannot be applied to a `Transaction`); the phantom
//! typestate `Entry<S>` is the tool for the one case a structural type change
//! cannot express — the data shape is INVARIANT but the PERMISSIONS change.
//!
//! The graph, deliberately non-linear to stress how the pattern scales:
//!
//! ```text
//!                      ┌──────────── Amend ◀────────────┐
//!                      ▼                                 │
//!   ▶ Draft ─Submit─▶ Submitted ─classify─▶ Cleared ─Post──▶ Posted ─Void─▶ Voided
//!                          │                                                   │
//!                          └─classify─▶ Flagged ─Reject─▶ Rejected ──Amend─────┘ (cycle)
//! ```
//!
//! Three edge SHAPES appear, and EVERY one is now an edge of the boundary category
//! (each declares a `CAPABILITY`, so the whole path's ceiling is the static join):
//!   - REVERSIBLE `Morphism` (`Submit`, `Amend`, `Void`): data invariant, `Unit`
//!     residual, so `backward` returns to the prior state. DECLARED, not hand-written:
//!     the `StateMachine` descriptor `EntryFlow` plus the grammar's `transition!`
//!     macro reduce each reversible edge to one line (its name and endpoints).
//!   - `Branch` (`Validate`): a total morphism into a COPRODUCT — one input, one of
//!     two next states, KEEPING the negative arm as a `Flagged` STATE-proof rather
//!     than a discarded `None` (the GDP total-`classify`).
//!   - `Guarded` (`Post`, `Reject`): a partial morphism admitted by a name-branded
//!     proof for THIS entry (`Cleared<N>` / `Flagged<N>`) — the sibling of a
//!     `Construction` (which mints its own witness); here the witness comes from the
//!     `Branch`. You cannot post an unbalanced entry nor reject a balanced one, and a
//!     proof for entry A will not discharge entry B.
//!
//! The illegal transitions are pinned negatively by `tests/compile_fail`.

use core::marker::PhantomData;

use crate::boundary::{Branch, Guarded, Pure};
use crate::gdp::Named;
use crate::ledger::boundary::{Balance, Transaction};

// ===== typestates: the protocol positions (the STATES) ===================

/// A freshly entered, not-yet-submitted entry — the only entrance.
pub struct Draft;
/// Submitted, awaiting classification.
pub struct Submitted;
/// Cleared and committed to the ledger.
pub struct Posted;
/// Classified as unbalanced and rejected — awaiting amendment.
pub struct Rejected;
/// A posted entry that has been reversed (terminal).
pub struct Voided;
crate::typestate!(Draft, Submitted, Posted, Rejected, Voided);

// ===== the state machine: carrier + descriptor + reversible edges ========

// The carrier `Entry<S>` (a `Transaction` indexed by its lifecycle position `S`, the
// index erased at runtime) and its descriptor `EntryFlow` are declared in one line by
// the grammar's `state_machine!` — the ~18 lines of value-object boilerplate every
// machine used to hand-write. The data never changes; only the type records WHERE in
// the protocol the entry sits, so a transition out of order does not type-check.
crate::state_machine!(EntryFlow, Entry, Transaction);

impl<S> Entry<S> {
    /// The underlying transaction (read-only) — the sanctioned accessor, available
    /// in any state. (Reads the macro-generated carrier's field directly, so it needs
    /// no `S: Typestate` bound.)
    pub fn tx(&self) -> &Transaction {
        &self.0
    }
}

impl Entry<Draft> {
    /// Enter a new transaction into the protocol at its ONLY start point. There is
    /// no constructor for any other state, so an entry can be born only as a
    /// `Draft` — every other state is reachable only through a transition.
    pub fn draft(tx: Transaction) -> Self {
        Entry(tx, PhantomData)
    }
}

crate::transition!(
    /// `Draft -> Submitted`. No precondition and a `Unit` residual, so it is a
    /// reversible `Morphism`: `backward` returns to `Draft`. The typestate gates
    /// ORDER — `Submit::In` is `Entry<Draft>`, so an already-submitted entry has the
    /// wrong type.
    Submit: EntryFlow, Draft => Submitted
);
crate::transition!(
    /// `Rejected -> Draft` — the CYCLE. The only legal way back to `Draft`, and only
    /// from `Rejected`; reopens a rejected entry for amendment. Reversible.
    Amend: EntryFlow, Rejected => Draft
);
crate::transition!(
    /// `Posted -> Voided` — the REVERSAL. `backward` restores it to `Posted`, so the
    /// undo is the morphism's own inverse.
    Void: EntryFlow, Posted => Voided
);

// ===== the branch proofs (GDP tokens, realized as value objects) =========

// The two branch proofs are GDP tokens realized as value objects, declared by the
// grammar's `proof_token!` (the zero-data, name-branded, fixed-`Debug` boilerplate).
// Their fields stay private to this boundary, so they are minted ONLY here.

crate::proof_token!(
    /// A proof that the entry named `N` is BALANCED (double entry holds). Branded with
    /// the entry's unique name `N`, so a proof for entry A cannot discharge
    /// `Post::commit` on B; minted only by `Validate::classify`.
    Cleared
);
crate::proof_token!(
    /// A proof that the entry named `N` is UNBALANCED — the NEGATIVE witness, kept (not
    /// thrown away as a `None`) so the reject path is a first-class branch. It gates
    /// `Reject`, so a balanced entry cannot be rejected.
    Flagged
);

// ===== the branching + guarded transitions ===============================

/// The classifier — a `Branch` edge (`Submitted -> Cleared + Flagged`). It performs
/// the REAL balance check and mints the proof for whichever arm holds, branded with
/// the entry's name. The coproduct carrier is `Result<Cleared<N>, Flagged<N>>` — a
/// std two-armed sum, so it needs no boundary citizen of its own — but unlike a bare
/// `Option<Cleared>` it KEEPS the negative witness in the `Err` arm: rejection is a
/// legitimate outcome whose proof is as load-bearing as the clearance's, the paper's
/// total `classify` rather than a `Maybe`. (A statistical probe must never mint such a
/// proof; this is an exact check, per the GDP discipline that a proof is only as true
/// as its mint.)
pub struct Validate;
/// The poster — a `Guarded` edge `Submitted -> Posted`. Its balance precondition is
/// supplied as a `Cleared<N>` for the SAME name (the sibling of a `Construction`'s
/// self-minted witness; here the witness comes from the `Validate` branch).
pub struct Post;
/// The rejecter — a `Guarded` edge `Submitted -> Rejected`. Symmetric to `Post`: it
/// needs a `Flagged<N>` for the same name, so a balanced entry (which has no
/// `Flagged`) cannot be rejected.
pub struct Reject;
crate::value_operator!(Validate, Post, Reject);

impl Branch for Validate {
    // A read-only classification: no loss, no state, no effect.
    type Capability = Pure;
    type In<N> = Named<N, Entry<Submitted>>;
    type Left<N> = Cleared<N>;
    type Right<N> = Flagged<N>;

    fn branch<N>(&self, entry: &Named<N, Entry<Submitted>>) -> Result<Cleared<N>, Flagged<N>> {
        let mut total = Balance::zero();
        for posting in entry.value().tx().postings() {
            total = total.add_cents(*posting.amount());
        }
        if total == Balance::zero() {
            Ok(Cleared(PhantomData))
        } else {
            Err(Flagged(PhantomData))
        }
    }
}

impl Validate {
    /// Classify a named, submitted entry — the ergonomic name for the `Branch` edge.
    /// Both arms carry a proof; the unbalanced case is a `Flagged` state-proof, never
    /// a silent `None`.
    pub fn classify<N>(
        &self,
        entry: &Named<N, Entry<Submitted>>,
    ) -> Result<Cleared<N>, Flagged<N>> {
        self.branch(entry)
    }
}

impl Guarded for Post {
    // A payload-preserving retag gated by a proof — pure.
    type Capability = Pure;
    type In<N> = Named<N, Entry<Submitted>>;
    type Proof<N> = Cleared<N>;
    // The posted entry KEEPS the submitted entry's brand `N` — provenance flows
    // through the edge, so a `Named<N, Entry<Posted>>` is provably the one named `N`.
    type Out<N> = Named<N, Entry<Posted>>;

    fn guard<N>(
        &self,
        entry: &Named<N, Entry<Submitted>>,
        _proof: &Cleared<N>,
    ) -> Named<N, Entry<Posted>> {
        // `Named::map` retags the payload while inheriting the name (the coupling).
        entry.map(|e| Entry(e.tx().clone(), PhantomData))
    }
}

impl Post {
    /// Commit a cleared, submitted entry to `Posted` — the ergonomic name for the
    /// `Guarded` edge. Requires a `Cleared<N>` for the same name `N`: ORDER (typestate)
    /// AND the PRECONDITION (proof) are both enforced at compile time, and there is no
    /// other constructor of `Entry<Posted>`. The result keeps the brand `N`.
    pub fn commit<N>(
        &self,
        entry: &Named<N, Entry<Submitted>>,
        proof: &Cleared<N>,
    ) -> Named<N, Entry<Posted>> {
        self.guard(entry, proof)
    }
}

impl Guarded for Reject {
    type Capability = Pure;
    type In<N> = Named<N, Entry<Submitted>>;
    type Proof<N> = Flagged<N>;
    type Out<N> = Named<N, Entry<Rejected>>;

    fn guard<N>(
        &self,
        entry: &Named<N, Entry<Submitted>>,
        _proof: &Flagged<N>,
    ) -> Named<N, Entry<Rejected>> {
        entry.map(|e| Entry(e.tx().clone(), PhantomData))
    }
}

impl Reject {
    /// Move a flagged, submitted entry to `Rejected` — the ergonomic name for the
    /// `Guarded` edge. Requires a `Flagged<N>` for the same name, so only an entry
    /// actually found unbalanced can be rejected — the negative witness authorizes it.
    /// The result keeps the brand `N`.
    pub fn apply<N>(
        &self,
        entry: &Named<N, Entry<Submitted>>,
        proof: &Flagged<N>,
    ) -> Named<N, Entry<Rejected>> {
        self.guard(entry, proof)
    }
}
