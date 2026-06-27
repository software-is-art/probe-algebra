//! lifecycle — a ledger entry's lifecycle as a non-linear state machine, the
//! architectural reading of the crate made explicit: a boundary is a state
//! machine (states = value-object types, transitions = operators, the type graph
//! makes illegal transitions uncallable). See `lifecycle::boundary`. Reversible
//! transitions are plain morphisms; the classify branch keeps the negative
//! witness as a state-proof; the guarded transitions need a name-branded proof.
//! The illegal sequences are pinned negatively by `tests/compile_fail`.

pub mod boundary;

#[cfg(test)]
mod tests {
    use super::boundary::{Amend, Draft, Entry, Post, Reject, Submit, Validate, Void};
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

    /// Submit is a reversible morphism: `run` carries the output with its (`Unit`)
    /// residual, and `invert` returns to the `Draft` it came from. Forward
    /// sequencing is free; so is going back.
    #[test]
    fn submit_round_trips() {
        let draft = Entry::<Draft>::draft(balanced());
        let carried = run(&Submit, &draft);
        assert_eq!(carried.out().tx(), &balanced());
        let back = carried.invert(&Submit).expect("submit is reversible");
        assert_eq!(back, draft);
        assert_eq!(<Submit as Morphism>::CAPABILITY, Capability::Pure);
    }

    /// The cleared path: draft, submit, name, classify → Cleared, commit. Each step
    /// is forced into order by its types; the final `Entry<Posted>` carries the same
    /// transaction it started with.
    #[test]
    fn balanced_entry_clears_and_posts() {
        with_seed(|seed| {
            let draft = Entry::<Draft>::draft(balanced());
            let (submitted, _unit) = Submit.forward(&draft);
            let named = seed.new_named(submitted);
            match Validate.classify(&named) {
                Ok(proof) => {
                    let posted = Post.commit(&named, &proof);
                    assert_eq!(posted.tx(), &balanced());
                }
                Err(_) => panic!("a balanced entry clears"),
            }
        });
    }

    /// The flagged path AND the cycle: an unbalanced entry classifies to `Flagged`,
    /// is `Reject`ed into `Entry<Rejected>`, then `Amend`ed back to `Entry<Draft>` —
    /// the only legal route back to the start. Amend is reversible too.
    #[test]
    fn unbalanced_entry_is_rejected_then_amended() {
        with_seed(|seed| {
            let draft = Entry::<Draft>::draft(unbalanced());
            let (submitted, _unit) = Submit.forward(&draft);
            let named = seed.new_named(submitted);
            let rejected = match Validate.classify(&named) {
                Err(proof) => Reject.apply(&named, &proof),
                Ok(_) => panic!("the sample does not balance"),
            };
            assert_eq!(rejected.tx(), &unbalanced());
            // the cycle: Rejected -> Draft, reversible.
            let reopened = run(&Amend, &rejected);
            assert_eq!(reopened.out().tx(), &unbalanced());
            assert_eq!(
                reopened.invert(&Amend).expect("amend is reversible"),
                rejected
            );
        });
    }

    /// The reversal: a posted entry can be `Void`ed, and the void round-trips back
    /// to `Posted` via the morphism's own `backward`.
    #[test]
    fn posted_entry_can_be_voided_and_restored() {
        with_seed(|seed| {
            let draft = Entry::<Draft>::draft(balanced());
            let (submitted, _unit) = Submit.forward(&draft);
            let named = seed.new_named(submitted);
            let posted = match Validate.classify(&named) {
                Ok(proof) => Post.commit(&named, &proof),
                Err(_) => panic!("a balanced entry clears"),
            };
            let voided = run(&Void, &posted);
            assert_eq!(voided.out().tx(), &balanced());
            assert_eq!(voided.invert(&Void).expect("void is reversible"), posted);
        });
    }

    /// The value-object surface of `Entry`: equality tracks the transaction.
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

    /// The proofs are value objects: same-name proofs compare equal, clone, and
    /// Debug-print. `classify` is total — a balanced entry takes the `Ok`/`Cleared`
    /// arm, an unbalanced one the `Err`/`Flagged` arm, and both carry a usable proof.
    #[test]
    fn proofs_carry_a_value_surface() {
        with_seed(|seed| {
            let (seed_ok, seed_bad) = seed.replicate();

            let (ok, _u1) = Submit.forward(&Entry::<Draft>::draft(balanced()));
            let named_ok = seed_ok.new_named(ok);
            let c1 = Validate.classify(&named_ok).expect("balanced clears");
            let c2 = Validate.classify(&named_ok).expect("balanced clears");
            assert_eq!(c1, c2);
            assert_eq!(c1, c1.clone());
            assert_eq!(format!("{c1:?}"), "Cleared");

            let (bad, _u2) = Submit.forward(&Entry::<Draft>::draft(unbalanced()));
            let named_bad = seed_bad.new_named(bad);
            let f1 = Validate
                .classify(&named_bad)
                .expect_err("unbalanced is flagged");
            let f2 = Validate
                .classify(&named_bad)
                .expect_err("unbalanced is flagged");
            assert_eq!(f1, f2);
            assert_eq!(f1, f1.clone());
            assert_eq!(format!("{f1:?}"), "Flagged");
        });
    }
}
