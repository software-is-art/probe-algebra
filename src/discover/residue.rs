//! Tier: ALGEBRA — a discovered-law / report layer (exempt from the inward rule).
//!
//! residue — equivalent mutants as a SIMPLIFICATION signal, not a permanent exception.
//!
//! Mutation is the gate; a surviving mutant points at a missing probe. But a handful survive that NO
//! probe can kill — an EQUIVALENT mutant, behaviourally identical to the original, so there is no
//! observation to distinguish it. Deciding equivalence is undecidable in general, so the residue is
//! ratified by hand in `.cargo/mutants.toml`. The point of this module: an equivalent mutant is not
//! noise to bury — it is a FINDING. A mutation whose value cannot change behaviour means the mutated
//! expression is behaviourally INERT, and that has exactly two causes:
//!
//!   - a REDUNDANT GUARD — the expression is dead/duplicated (a bound checked twice, a guard a later
//!     check re-imposes). The fix is to SIMPLIFY: remove it, and the equivalent mutant is ELIMINATED,
//!     not excluded (as `shadow_grid`'s doubled cap-check and `select`'s guards already were); or
//!   - a FREE CHOICE — the spec genuinely does not constrain this expression (an arbitrary canonical
//!     seed, a deliberately-empty declaration). The fix is to ACCEPT it, or to TIGHTEN the spec.
//!
//! So the residue is surfaced, classified, and — for the redundant kind — pushed toward elimination,
//! the same suggestion-then-ratify loop as cohesion and layering. A DRIFT gate ties this classified
//! list to the carve-outs the mutation gate actually applies, so no exclusion can accumulate
//! undocumented and no documented finding can silently disappear.

/// Why an equivalent mutant cannot be killed — the two causes of behavioural inertness.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Inertia {
    /// Dead/duplicated code — the expression's value cannot affect behaviour because something else
    /// already determines it. SIMPLIFY to eliminate the equivalent mutant.
    RedundantGuard,
    /// The spec does not constrain this expression — any value is equally valid. ACCEPT, or tighten
    /// the spec to pin it.
    FreeChoice,
}

/// One ratified equivalent mutant: the cargo-mutants pattern that names it, why it is inert, and the
/// rationale a human ratified.
#[derive(Clone, Copy)]
pub struct Inert {
    /// The `exclude_re` pattern in `.cargo/mutants.toml` that carves this mutant out (by function, so
    /// it survives line edits).
    pub pattern: &'static str,
    pub kind: Inertia,
    pub note: &'static str,
}

impl Inert {
    /// The action this finding calls for.
    pub fn remedy(&self) -> &'static str {
        match self.kind {
            Inertia::RedundantGuard => {
                "SIMPLIFY — remove the inert expression to eliminate the mutant"
            }
            Inertia::FreeChoice => "ACCEPT — a genuine free choice, or tighten the spec to pin it",
        }
    }
}

/// The ratified equivalent-mutant residue — the behaviourally-inert spots the gate excludes, each
/// classified. The patterns mirror `.cargo/mutants.toml`'s `exclude_re` (kept in lockstep by the
/// drift gate in the tests); the CLASSIFICATION is the irreducible human judgement — why each mutant
/// is inert — surfaced here instead of buried in a config comment.
pub fn residue() -> Vec<Inert> {
    vec![
        Inert {
            pattern: "<impl Shaped for bool>::inhabitant",
            kind: Inertia::FreeChoice,
            note: "the canonical seed is an arbitrary bool (`false` == `Default::default()`), \
                   unobservable by any sensitivity probe",
        },
        Inert {
            pattern: "<impl Declares for ResolvePretendsPure>::declared_sources",
            kind: Inertia::FreeChoice,
            note: "it legitimately declares NOTHING (the under-claim being demonstrated), so an \
                   empty-slice mutant is indistinguishable",
        },
        Inert {
            pattern: "match guard c.is_ascii_alphabetic",
            kind: Inertia::RedundantGuard,
            note:
                "the lexer's first-char guard is re-imposed by `Ident::new`, so relaxing it admits \
                   nothing new — the guard is redundant",
        },
    ]
}

/// The redundant-guard residue — the entries that should be SIMPLIFIED AWAY, shrinking the carve-out
/// list toward only genuine free choices.
pub fn simplifiable() -> Vec<Inert> {
    residue()
        .into_iter()
        .filter(|i| i.kind == Inertia::RedundantGuard)
        .collect()
}

/// Render the residue as a readable report — the redundant ones first, as work to do.
pub fn render() -> String {
    let r = residue();
    let mut out = format!(
        "equivalent-mutant residue — {} ratified, behaviourally inert:\n",
        r.len()
    );
    for i in r.iter().filter(|i| i.kind == Inertia::RedundantGuard) {
        out.push_str(&format!(
            "  [redundant] {} — {}\n      {}\n",
            i.pattern,
            i.note,
            i.remedy()
        ));
    }
    for i in r.iter().filter(|i| i.kind == Inertia::FreeChoice) {
        out.push_str(&format!(
            "  [free choice] {} — {}\n      {}\n",
            i.pattern,
            i.note,
            i.remedy()
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    /// DRIFT GATE: every carve-out the mutation gate actually applies (`.cargo/mutants.toml`'s
    /// `exclude_re`) is a ratified, classified finding here, and vice versa. So an exclusion cannot
    /// accumulate undocumented, and a documented finding cannot silently vanish from the gate. (Reads
    /// the config; `#[cfg(test)]`, so exempt from the ALGEBRA capability rule.)
    #[test]
    fn the_residue_matches_the_gate_carveouts() {
        let toml = std::fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR")).join(".cargo/mutants.toml"),
        )
        .expect("read mutants.toml");
        // extract the quoted strings inside the `exclude_re = [ ... ]` array.
        let start = toml.find("exclude_re").expect("exclude_re present");
        let arr = &toml[start..];
        let end = arr.find(']').expect("array closes");
        let carveouts: Vec<&str> = arr[..end]
            .match_indices('"')
            .collect::<Vec<_>>()
            .chunks(2)
            .filter_map(|pair| match pair {
                [(a, _), (b, _)] => Some(&arr[a + 1..*b]),
                _ => None,
            })
            .collect();

        let mut declared: Vec<&str> = residue().iter().map(|i| i.pattern).collect();
        let mut gated = carveouts.clone();
        declared.sort_unstable();
        gated.sort_unstable();
        assert_eq!(
            declared, gated,
            "residue() and .cargo/mutants.toml exclude_re have drifted"
        );
    }

    /// The classification is the point: the lexer guard is the REDUNDANT one (simplifiable), the other
    /// two are free choices. Pins the kinds against a flip.
    #[test]
    fn equivalents_are_classified_redundant_vs_free() {
        let simp = simplifiable();
        assert_eq!(simp.len(), 1, "exactly one redundant guard");
        assert_eq!(simp[0].pattern, "match guard c.is_ascii_alphabetic");
        assert!(simp[0].remedy().contains("SIMPLIFY"));
        // the free choices are not offered up for simplification.
        let free: Vec<_> = residue()
            .into_iter()
            .filter(|i| i.kind == Inertia::FreeChoice)
            .collect();
        assert_eq!(free.len(), 2);
        assert!(free.iter().all(|i| i.remedy().contains("ACCEPT")));
    }

    /// The report leads with the redundant (actionable) finding and names EVERY entry — the redundant
    /// guard and both free choices, each under its own heading.
    #[test]
    fn the_report_surfaces_every_finding() {
        let text = render();
        assert!(text.contains("[redundant]") && text.contains("is_ascii_alphabetic"));
        // both free choices appear (so the free-choice section actually lists them, not a mislabel).
        assert!(text.contains("[free choice]"));
        assert!(
            text.contains("inhabitant") && text.contains("declared_sources"),
            "every free choice must be named: {text}"
        );
        // the redundant one is listed before the free choices (work first).
        let red = text.find("[redundant]").unwrap();
        let free = text.find("[free choice]").unwrap();
        assert!(red < free, "redundant findings lead");
    }
}
