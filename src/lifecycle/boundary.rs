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
//! Four transition SHAPES appear, and only the first is a plain `Morphism`:
//!   - REVERSIBLE (`Submit`, `Amend`, `Void`): data invariant, `Unit` residual, so
//!     `backward` returns to the prior state — phantom transitions that round-trip
//!     and probe like any morphism. These are now DECLARED, not hand-written: the
//!     `StateMachine` descriptor `EntryFlow` plus the grammar's `transition!` macro
//!     reduce each reversible edge to one line (its name and endpoints).
//!   - BRANCHING (`Validate::classify`): one input, one of SEVERAL next states. It
//!     is the GDP total-`classify` lesson AS a transition — the failure case is a
//!     `Flagged` STATE-proof, not a discarded `None`.
//!   - GUARDED (`Post`, `Reject`): each needs a name-branded proof for THIS entry
//!     (`Cleared<N>` / `Flagged<N>`), so you cannot post an unbalanced entry NOR
//!     reject a balanced one, and a proof for entry A will not discharge entry B.
//!
//! The illegal transitions are pinned negatively by `tests/compile_fail`.

use core::marker::PhantomData;

use crate::boundary::{sealed, StateMachine, Typestate, ValueObject};
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

// ===== the value object carried through the protocol =====================

/// A ledger entry INDEXED by its lifecycle position `S`. The data never changes —
/// it is always one `Transaction` — but the type records WHERE in the protocol the
/// entry sits, so a transition out of order does not type-check. `S` is phantom:
/// the index is erased at runtime, so the state machine costs nothing.
pub struct Entry<S>(Transaction, PhantomData<S>);

// Manual impls (no `S: Trait` bounds): the value-object markers delegate to the
// `Transaction` field, and the phantom `S` need implement nothing.
impl<S> Clone for Entry<S> {
    fn clone(&self) -> Self {
        Entry(self.0.clone(), PhantomData)
    }
}
impl<S> PartialEq for Entry<S> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl<S> Eq for Entry<S> {}
impl<S> core::fmt::Debug for Entry<S> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("Entry").field(&self.0).finish()
    }
}
impl<S> sealed::Sealed for Entry<S> {}
impl<S> ValueObject for Entry<S> {}

impl<S> Entry<S> {
    /// The underlying transaction (read-only) — the sanctioned accessor, available
    /// in any state.
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

// ===== the state machine: reversible edges via the grammar ===============

/// The state-machine descriptor for the entry lifecycle: the payload is a
/// `Transaction`, carried as `Entry<S>`. A value-operator family — it supplies the
/// pure retag (`at`) and read (`data`) operations every transition is built from,
/// so the boundary declares its reversible edges with `transition!` instead of
/// hand-writing each `Morphism`. The block of `transition!` declarations below IS
/// the reversible part of the state graph, in code.
pub struct EntryFlow;
crate::value_operator!(EntryFlow);
impl StateMachine for EntryFlow {
    type Data = Transaction;
    type At<S: Typestate> = Entry<S>;

    fn at<S: Typestate>(data: Transaction) -> Entry<S> {
        Entry(data, PhantomData)
    }

    fn data<S: Typestate>(at: &Entry<S>) -> &Transaction {
        &at.0
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

/// A proof that the entry named `N` is BALANCED (double entry holds). A GDP "ghost"
/// realized as a value object: zero data, minted ONLY by `Validate::classify` (its
/// field is private to this boundary), and branded with the entry's unique name
/// `N`. Tied to `N`, a proof for entry A cannot discharge `Post::commit` on B.
pub struct Cleared<N>(PhantomData<N>);
/// A proof that the entry named `N` is UNBALANCED — the NEGATIVE witness, kept (not
/// thrown away as a `None`) so the reject path is a first-class branch. It gates
/// `Reject`, so a balanced entry cannot be rejected.
pub struct Flagged<N>(PhantomData<N>);

impl<N> Clone for Cleared<N> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<N> Copy for Cleared<N> {}
impl<N> PartialEq for Cleared<N> {
    fn eq(&self, _other: &Self) -> bool {
        true // two clearances of the same name are the same fact; no data to differ on.
    }
}
impl<N> core::fmt::Debug for Cleared<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("Cleared")
    }
}
impl<N> sealed::Sealed for Cleared<N> {}
impl<N> ValueObject for Cleared<N> {}

impl<N> Clone for Flagged<N> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<N> Copy for Flagged<N> {}
impl<N> PartialEq for Flagged<N> {
    fn eq(&self, _other: &Self) -> bool {
        true // likewise: a flag of a given name carries no distinguishing data.
    }
}
impl<N> core::fmt::Debug for Flagged<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("Flagged")
    }
}
impl<N> sealed::Sealed for Flagged<N> {}
impl<N> ValueObject for Flagged<N> {}

// ===== the branching + guarded transitions ===============================

/// The classifier — the BRANCH. `classify` performs the REAL balance check and
/// mints the proof for whichever branch holds, branded with the entry's name. The
/// branch carrier is a `Result<Cleared<N>, Flagged<N>>` — a std two-armed sum like
/// `Morphism::backward`'s `Option`, so it needs no boundary citizen of its own —
/// but unlike a bare `Option<Cleared>` it KEEPS the negative witness in the `Err`
/// arm: rejection is a legitimate outcome whose proof is as load-bearing as the
/// clearance's, the paper's total `classify` rather than a `Maybe`. (A statistical
/// probe must never mint such a proof; this is an exact check, per the GDP
/// discipline that a proof is only as true as its mint.)
pub struct Validate;
/// The poster — a GUARDED transition `Submitted -> Posted`. Not a plain `Morphism`:
/// its balance precondition cannot fit `In -> (Out, Residual)`, so it is supplied a
/// `Cleared<N>` for the SAME name.
pub struct Post;
/// The rejecter — a GUARDED transition `Submitted -> Rejected`. Symmetric to
/// `Post`: it needs a `Flagged<N>` for the same name, so a balanced entry (which
/// has no `Flagged`) cannot be rejected.
pub struct Reject;
crate::value_operator!(Validate, Post, Reject);

impl Validate {
    /// Classify a named, submitted entry, minting the proof for the branch that
    /// holds. Both branches carry a proof — the unbalanced case is a `Flagged`
    /// state-proof, never a silent `None`.
    pub fn classify<N>(
        &self,
        entry: &Named<N, Entry<Submitted>>,
    ) -> Result<Cleared<N>, Flagged<N>> {
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

impl Post {
    /// Commit a cleared, submitted entry to `Posted`. Requires a `Cleared<N>` for
    /// the same name `N`: ORDER (typestate) AND the PRECONDITION (proof) are both
    /// enforced at compile time, and there is no other constructor of `Entry<Posted>`.
    pub fn commit<N>(
        &self,
        entry: &Named<N, Entry<Submitted>>,
        _proof: &Cleared<N>,
    ) -> Entry<Posted> {
        Entry(entry.value().tx().clone(), PhantomData)
    }
}

impl Reject {
    /// Move a flagged, submitted entry to `Rejected`. Requires a `Flagged<N>` for
    /// the same name, so only an entry actually found unbalanced can be rejected —
    /// the negative witness is what authorizes the transition.
    pub fn apply<N>(
        &self,
        entry: &Named<N, Entry<Submitted>>,
        _proof: &Flagged<N>,
    ) -> Entry<Rejected> {
        Entry(entry.value().tx().clone(), PhantomData)
    }
}
