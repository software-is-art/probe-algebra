//! lifecycle — a typestate protocol that shows WHERE logic lives and how
//! sequencing is made un-screw-up-able. See `lifecycle::boundary`. The order of
//! transitions is encoded in the typestate `In`/`Out` types; one transition's
//! relational precondition is encoded as a GDP proof tied to the entry's name.
//! The illegal orderings are pinned negatively by `tests/compile_fail`.

pub mod boundary;

#[cfg(test)]
mod tests {
    use super::boundary::{Cleared, Draft, Entry, Post, Submit, Validate};
    use crate::boundary::{run, Capability, Morphism};
    use crate::gdp::with_seed;
    use crate::ledger::boundary::{Account, Cents, Posting, Transaction};

    fn balanced() -> Transaction {
        Transaction::new(vec![
            Posting::new(Account::new("Cash").unwrap(), Cents::new(10_000).unwrap()),
            Posting::new(
                Account::new("Revenue").unwrap(),
                Cents::new(-10_000).unwrap(),
            ),
        ])
        .unwrap()
    }

    fn unbalanced() -> Transaction {
        Transaction::new(vec![
            Posting::new(Account::new("Cash").unwrap(), Cents::new(10_000).unwrap()),
            Posting::new(Account::new("Fees").unwrap(), Cents::new(5_000).unwrap()),
        ])
        .unwrap()
    }

    /// The submit transition is a reversible morphism: `run` carries the output
    /// with its (`Unit`) residual, and `invert` returns to the `Draft` it came
    /// from. Sequencing forward is free; so is going back.
    #[test]
    fn submit_round_trips() {
        let draft = Entry::<Draft>::draft(balanced());
        let carried = run(&Submit, &draft);
        assert_eq!(carried.out().tx(), &balanced());
        let back = carried.invert(&Submit).expect("submit is reversible");
        assert_eq!(back, draft);
        // submission adds no capability.
        assert_eq!(<Submit as Morphism>::CAPABILITY, Capability::Pure);
    }

    /// The happy path through the whole protocol: draft, submit, name, clear
    /// (mints the proof), commit. Each step is forced into order by its types; the
    /// final `Entry<Posted>` carries the same transaction it started with.
    #[test]
    fn balanced_entry_clears_and_posts() {
        with_seed(|seed| {
            let draft = Entry::<Draft>::draft(balanced());
            let (submitted, _unit) = Submit.forward(&draft);
            let named = seed.new_named(submitted);
            let proof = Validate.clear(&named).expect("a balanced entry clears");
            let posted = Post.commit(&named, &proof);
            assert_eq!(posted.tx(), &balanced());
        });
    }

    /// An unbalanced entry yields NO clearance proof, so `Post::commit` is
    /// unreachable for it: the precondition is unmet and there is simply nothing to
    /// pass. The negative case is a `None`, not a panic.
    #[test]
    fn unbalanced_entry_cannot_be_cleared() {
        with_seed(|seed| {
            let draft = Entry::<Draft>::draft(unbalanced());
            let (submitted, _unit) = Submit.forward(&draft);
            let named = seed.new_named(submitted);
            assert!(Validate.clear(&named).is_none());
        });
    }

    /// The value-object surface of `Entry`: equality tracks the transaction, and a
    /// submitted entry built two ways compares equal while a different transaction
    /// does not.
    #[test]
    fn entry_is_a_value_object() {
        let a = Entry::<Draft>::draft(balanced());
        let b = Entry::<Draft>::draft(balanced());
        let c = Entry::<Draft>::draft(unbalanced());
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_eq!(format!("{a:?}"), format!("Entry({:?})", balanced()));
        assert_eq!(a.clone(), a);
    }

    /// The clearance proof is itself a value object: two proofs for the same named
    /// entry compare equal and Debug-print, exercising the GDP token's surface.
    #[test]
    fn clearance_proof_value_surface() {
        with_seed(|seed| {
            let draft = Entry::<Draft>::draft(balanced());
            let (submitted, _unit) = Submit.forward(&draft);
            let named = seed.new_named(submitted);
            let p1: Cleared<_> = Validate.clear(&named).unwrap();
            let p2 = Validate.clear(&named).unwrap();
            assert_eq!(p1, p2);
            assert_eq!(p1, p1.clone());
            assert_eq!(format!("{p1:?}"), "Cleared");
        });
    }
}
