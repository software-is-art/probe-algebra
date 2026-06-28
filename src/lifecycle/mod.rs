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
    use super::boundary::{
        Amend, Draft, Entry, Post, Posted, Reject, Rejected, Submit, Submitted, Validate, Void,
    };
    use crate::boundary::{run, Branch, Capability, Guarded, Morphism};
    use crate::gdp::{with_seed, Named};
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
                    // `posted` is `Named<N, Entry<Posted>>` — the brand flows through.
                    let posted = Post.commit(&named, &proof);
                    assert_eq!(posted.value().tx(), &balanced());
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
            assert_eq!(rejected.value().tx(), &unbalanced());
            // the cycle: Rejected -> Draft, reversible. `Amend` is a name-free
            // morphism, so the entry steps out of its brand via `value()`.
            let reopened = run(&Amend, rejected.value());
            assert_eq!(reopened.out().tx(), &unbalanced());
            assert_eq!(
                &reopened.invert(&Amend).expect("amend is reversible"),
                rejected.value()
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
            let voided = run(&Void, posted.value());
            assert_eq!(voided.out().tx(), &balanced());
            assert_eq!(
                &voided.invert(&Void).expect("void is reversible"),
                posted.value()
            );
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

    /// Every hop of the path is now an edge of the algebra that declares a capability
    /// — `Submit`/`Void` as `Morphism`, `Validate` as `Branch`, `Post` as `Guarded` —
    /// so the WHOLE multistate lifecycle's ceiling is the static join of its edges,
    /// computed at compile time. An edge that smuggled in an effect would lift this
    /// above `Pure` and the assertion would fail: the category bounds the path.
    #[test]
    fn the_whole_path_capability_is_the_join_of_its_edges() {
        const PATH: Capability = <Submit as Morphism>::CAPABILITY
            .join(<Validate as Branch>::CAPABILITY)
            .join(<Post as Guarded>::CAPABILITY)
            .join(<Void as Morphism>::CAPABILITY);
        assert_eq!(PATH, Capability::Pure);
    }

    /// A multistate hop flowing as ONE categorical edge: `[Post, Reject] ∘ classify`
    /// is `Entry<Submitted> -> Entry<Posted> + Entry<Rejected>` (the copairing of two
    /// `Guarded` edges after a `Branch`). The brand routes the proof to the matching
    /// guard — `Cleared` discharges `Post`, `Flagged` discharges `Reject` — so the
    /// dispatch cannot cross the wires.
    #[test]
    fn classify_then_dispatch_is_one_branching_hop() {
        // Both arms keep the brand `N`: the coproduct is `Named<N, Entry<Posted>> +
        // Named<N, Entry<Rejected>>`, so provenance survives the dispatch.
        fn dispatch<N>(
            entry: &Named<N, Entry<Submitted>>,
        ) -> Result<Named<N, Entry<Posted>>, Named<N, Entry<Rejected>>> {
            match Validate.branch(entry) {
                Ok(cleared) => Ok(Post.guard(entry, &cleared)),
                Err(flagged) => Err(Reject.guard(entry, &flagged)),
            }
        }

        with_seed(|seed| {
            let (s_ok, s_bad) = seed.replicate();

            let (ok, _u) = Submit.forward(&Entry::<Draft>::draft(balanced()));
            match dispatch(&s_ok.new_named(ok)) {
                Ok(posted) => assert_eq!(posted.value().tx(), &balanced()),
                Err(_) => panic!("a balanced entry posts"),
            }

            let (bad, _u) = Submit.forward(&Entry::<Draft>::draft(unbalanced()));
            match dispatch(&s_bad.new_named(bad)) {
                Err(rejected) => assert_eq!(rejected.value().tx(), &unbalanced()),
                Ok(_) => panic!("an unbalanced entry rejects"),
            }
        });
    }

    /// Provenance coupling (case c1): a `Post`ed entry KEEPS the brand of the
    /// submitted entry it came from, so a consumer can demand the posted output and
    /// its origin under ONE name — `same_origin` only type-checks for the matching
    /// brand. A posted entry from a DIFFERENT seed would not unify, so a `Posted`
    /// cannot be misattributed to an entry it did not come from.
    #[test]
    fn posting_carries_provenance_of_its_origin() {
        // The consumer: both arguments must share the brand `N`.
        fn same_origin<N>(
            posted: &Named<N, Entry<Posted>>,
            origin: &Named<N, Entry<Submitted>>,
        ) -> bool {
            posted.value().tx() == origin.value().tx()
        }

        with_seed(|seed| {
            let (sub, _u) = Submit.forward(&Entry::<Draft>::draft(balanced()));
            let named = seed.new_named(sub);
            let cleared = Validate.branch(&named).expect("balanced clears");
            let posted = Post.guard(&named, &cleared); // Named<N, Entry<Posted>>
            assert!(same_origin(&posted, &named));
            // same_origin(&posted, &other_named) — would NOT compile: a posted entry
            // can only be paired with the exact submitted entry it was derived from.
        });
    }
}
