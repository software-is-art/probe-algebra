//! lifecycle::boundary — a TYPESTATE lifecycle protocol for a ledger entry.
//!
//! This module answers a question the rest of the crate left open: where does
//! the LOGIC live, and how is SEQUENCING made un-screw-up-able? A ledger entry
//! moves `Draft -> Submitted -> Posted`. Each position is a typestate, the entry
//! is the same value object indexed by its position, and each transition is a
//! value operator whose `In`/`Out` types pin the only legal orderings:
//!
//!   - `Submit : Entry<Draft> -> Entry<Submitted>` is a plain `Morphism` — it has
//!     no precondition and loses nothing, so it fits `In -> (Out, Residual)` with
//!     a `Unit` residual, which makes it REVERSIBLE (`backward` returns to Draft)
//!     and probeable like any other morphism.
//!   - `Post : Entry<Submitted> -> Entry<Posted>` is NOT a plain morphism: it has
//!     a PRECONDITION (the entry must balance) that the pure `In -> (Out,
//!     Residual)` shape cannot express. The precondition is carried as a GDP
//!     proof, `Cleared<N>`, minted only by the real check in `Validate::clear` and
//!     branded with the entry's unique name `N`.
//!
//! Two failure classes are made compile errors. ORDER (post a draft, submit a
//! posted entry) is killed by the typestate `In`/`Out` types. The RELATIONAL
//! precondition — "validate entry A, then post entry B" — is killed by the GDP
//! name: a `Cleared<N>` minted for A will not unify with B. Both are pinned by the
//! `tests/compile_fail` suite. The illegal sequence never reaches the test bench.

use core::marker::PhantomData;

use crate::boundary::{sealed, Capability, Morphism, Unit, ValueObject};
use crate::gdp::Named;
use crate::ledger::boundary::{Balance, Transaction};

// ===== typestates: the protocol positions ================================

/// Lifecycle position: a freshly entered, not-yet-submitted entry.
pub struct Draft;
/// Lifecycle position: submitted, awaiting validation and posting.
pub struct Submitted;
/// Lifecycle position: committed to the ledger (terminal).
pub struct Posted;
crate::typestate!(Draft, Submitted, Posted);

// ===== the value object carried through the protocol =====================

/// A ledger entry INDEXED by its lifecycle position `S`. The data never changes —
/// it is always one `Transaction` — but the type records WHERE in the protocol the
/// entry sits, so a transition out of order does not type-check (`Post` wants an
/// `Entry<Submitted>`; an `Entry<Draft>` will not do). `S` is phantom: the index
/// is erased at runtime, so the protocol costs nothing.
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
    /// no constructor for `Submitted` or `Posted`, so an entry can be born only as
    /// a `Draft` — the protocol has a single entrance.
    pub fn draft(tx: Transaction) -> Self {
        Entry(tx, PhantomData)
    }
}

// ===== the transition expressible as a plain Morphism ====================

/// `Draft -> Submitted`. Submission carries no precondition and loses nothing, so
/// it fits the pure `Morphism` shape with a `Unit` residual — which makes it
/// REVERSIBLE: `backward` returns to `Draft`, so the step round-trips and is
/// probeable like every other morphism in the crate. The typestate is what gates
/// ORDER: `Submit::In` is `Entry<Draft>`, so it cannot apply to an entry that is
/// already submitted or posted.
pub struct Submit;
crate::value_operator!(Submit);

impl Morphism for Submit {
    const CAPABILITY: Capability = Capability::Pure;
    type In = Entry<Draft>;
    type Out = Entry<Submitted>;
    type Residual = Unit;

    fn forward(&self, input: &Entry<Draft>) -> (Entry<Submitted>, Unit) {
        (Entry(input.0.clone(), PhantomData), Unit)
    }

    fn backward(&self, out: &Entry<Submitted>, _residual: &Unit) -> Option<Entry<Draft>> {
        Some(Entry(out.0.clone(), PhantomData))
    }
}

// ===== the proof-gated transition: a GDP relational precondition =========

/// A proof that the entry named `N` is BALANCED (double entry holds). It is a GDP
/// "ghost" realized as a value object: zero data, minted ONLY by `Validate::clear`
/// (its field is private to this boundary), and branded with the entry's unique
/// name `N`. Because it is tied to `N`, a proof minted for entry A cannot discharge
/// `Post::commit` on entry B — "validate one, post another" is a COMPILE error, not
/// a runtime slip. A balance is a single-value fact, but WHICH entry it is a fact
/// ABOUT is relational, which is exactly what the name carries and a value object
/// cannot.
pub struct Cleared<N>(PhantomData<N>);

impl<N> Clone for Cleared<N> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<N> Copy for Cleared<N> {}
impl<N> PartialEq for Cleared<N> {
    fn eq(&self, _other: &Self) -> bool {
        // Two clearances of the SAME name are the same fact; there is no data to
        // differ on. (Clearances of different names have different types and never
        // reach this comparison.)
        true
    }
}
impl<N> core::fmt::Debug for Cleared<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("Cleared")
    }
}
impl<N> sealed::Sealed for Cleared<N> {}
impl<N> ValueObject for Cleared<N> {}

/// The validator. `clear` performs the REAL balance check and, only on success,
/// mints a `Cleared<N>` branded with the entry's name — the one place the proof is
/// earned. (A statistical probe must never mint such a proof; this is a total,
/// exact check, per the GDP discipline that a proof is only as true as its mint.)
pub struct Validate;
crate::value_operator!(Validate);

impl Validate {
    /// Mint a clearance proof iff the named, submitted entry's postings sum to
    /// zero. `None` (not a panic, not a defaulted proof) when it does not balance —
    /// the negative case simply yields no proof, so `Post::commit` stays unreachable.
    pub fn clear<N>(&self, entry: &Named<N, Entry<Submitted>>) -> Option<Cleared<N>> {
        let mut total = Balance::zero();
        for posting in entry.value().tx().postings() {
            total = total.add_cents(*posting.amount());
        }
        if total == Balance::zero() {
            Some(Cleared(PhantomData))
        } else {
            None
        }
    }
}

/// The committer. `commit` is the `Submitted -> Posted` transition, but unlike
/// `Submit` it is NOT a plain `Morphism`: it has a PRECONDITION that the pure
/// `In -> (Out, Residual)` shape cannot express. The precondition is supplied as a
/// `Cleared<N>` for the SAME name, so the only route to `Posted` is through a real,
/// matching clearance — the GDP proof is what lifts the transition out of the
/// plain morphism algebra.
pub struct Post;
crate::value_operator!(Post);

impl Post {
    /// Commit a cleared, submitted entry, advancing it to `Posted`. Requires a
    /// `Cleared<N>` for the same name `N`: a proof about another entry will not
    /// unify, so ORDER (typestate) AND the PRECONDITION (proof) are both enforced
    /// at compile time. There is no other constructor of `Entry<Posted>`.
    pub fn commit<N>(
        &self,
        entry: &Named<N, Entry<Submitted>>,
        _proof: &Cleared<N>,
    ) -> Entry<Posted> {
        Entry(entry.value().tx().clone(), PhantomData)
    }
}
