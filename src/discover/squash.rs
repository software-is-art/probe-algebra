//! squash — journal compaction as law: the verb whose implementation is a CONSUMER of
//! the verb algebra's own frozen spec. The rule table is not authored, it is READ OFF
//! the discovered laws: composition laws are the collapses, projection laws are the
//! replay-noise filter, cross-item commuting-maps laws are mobility. A collapse the lock
//! does not state never happens — the compactor is exactly as smart as the algebra, no
//! more, which is the whole warrant for letting it rewrite the record.
//!
//! Honest frame, inherited: the miniature proves the algebra's SHAPE — its `_a`/`_b`
//! items transport to the journal's `module — detail` keys by base verb name, a seam the
//! probes judge on real journal fixtures, not a proof about every journal.

use std::collections::{BTreeMap, BTreeSet};

use crate::discover::Spec;

/// The compactor's rule table, DERIVED from a discovered spec — never hand-authored.
/// Three law shapes feed it: same-item COMPOSITION laws become collapses (`collect_a
/// after add_a collapses to collect_a` → add-then-collect is one collect), PROJECTION
/// laws become the replay-noise filter (a repeated identical entry is one entry), and
/// cross-item COMMUTING-MAPS laws become mobility (which verb pairs may slide past each
/// other on different keys). The transport seam, disclosed: the miniature names items by
/// `_a`/`_b` suffix, the real journal by its `module — detail` key — base verb names
/// carry across, and the probes below judge the seam on real journal fixtures.
pub struct SquashRules {
    /// `(first, then)` → the one verb the ordered same-key pair equals.
    collapses: BTreeMap<(String, String), String>,
    /// Verbs the lock states are projections — replaying one is applying it once.
    projections: BTreeSet<String>,
    /// Unordered verb pairs frozen to commute across DIFFERENT keys.
    commutes: BTreeSet<(String, String)>,
}

#[crate::mutate("squash")]
impl SquashRules {
    /// Read the rule table off a spec's law equations (the engine's machine grammar:
    /// `f(g(x)) = h(x)`, `f(f(x)) = f(x)`, `f(g(x)) = g(f(x))`); every other law shape
    /// carries no journal rule and is skipped. Same-item composition only becomes a
    /// collapse; same-item commutation is deliberately NOT mobility — same-key entries
    /// collapse or stay put, they never leapfrog.
    pub fn from_spec(spec: &Spec) -> SquashRules {
        let mut rules = SquashRules {
            collapses: BTreeMap::new(),
            projections: BTreeSet::new(),
            commutes: BTreeSet::new(),
        };
        for law in &spec.laws {
            let Some((lhs, rhs)) = law.equation().split_once('=') else {
                continue;
            };
            let (Some(l), Some(r)) = (Self::nest(lhs), Self::nest(rhs)) else {
                continue;
            };
            match (l.as_slice(), r.as_slice()) {
                // f(f(x)) = f(x): a projection — replay noise filters to one entry.
                ([f, g], [h]) if f == g && g == h => {
                    rules.projections.insert(Self::split_verb(f).0);
                }
                // f(g(x)) = h(x), one shared item: g-then-f collapses to h.
                ([f, g], [h]) => {
                    let (f_base, f_item) = Self::split_verb(f);
                    let (g_base, g_item) = Self::split_verb(g);
                    let (h_base, h_item) = Self::split_verb(h);
                    if f_item == g_item && g_item == h_item && !f_item.is_empty() {
                        rules.collapses.insert((g_base, f_base), h_base);
                    }
                }
                // f(g(x)) = g(f(x)), items apart: the pair slides on different keys.
                ([f, g], [g2, f2]) if f == f2 && g == g2 => {
                    let (f_base, f_item) = Self::split_verb(f);
                    let (g_base, g_item) = Self::split_verb(g);
                    if f_item != g_item {
                        rules.commutes.insert(Self::unordered(&f_base, &g_base));
                    }
                }
                _ => {}
            }
        }
        rules
    }

    /// Compact a journal to its LAW-NORMAL FORM: the nearest same-key pair whose
    /// in-between entries all commute past the later verb collapses to the verb the
    /// lock names, repeated to a fixed point. Conservatism is silence, not error: a
    /// pair the table is silent on keeps both lines (add-then-edit composes to no
    /// single verb — the journal keeps the whole story), and a verb the algebra does
    /// not model (`place`) is opaque — nothing slides past it. The one hard refusal is
    /// a line the journal grammar cannot read: the record is machine-written, so a
    /// strange line means a hand touched it.
    pub fn compact(&self, journal: &str) -> Result<String, String> {
        let mut entries = Vec::new();
        for (n, line) in journal.lines().enumerate() {
            let parsed = line
                .split_once(' ')
                .filter(|(verb, key)| !verb.is_empty() && key.contains(" — "));
            match parsed {
                Some((verb, key)) => entries.push((verb.to_string(), key.to_string())),
                None => {
                    return Err(format!(
                        "bundle squash: line {} is not `<verb> <module> — <detail>` — \
                         the journal is machine-written, so a line the grammar cannot \
                         read means a hand touched it: `{line}`",
                        n + 1
                    ))
                }
            }
        }
        let mut changed = true;
        while changed {
            changed = false;
            'scan: for i in 0..entries.len() {
                for j in i + 1..entries.len() {
                    if entries[j].1 != entries[i].1 {
                        continue;
                    }
                    let slides = entries[i + 1..j]
                        .iter()
                        .all(|e| self.mobile(&e.0, &entries[j].0));
                    if slides {
                        if let Some(one) = self.collapse(&entries[i].0, &entries[j].0) {
                            entries[i].0 = one;
                            entries.remove(j);
                            changed = true;
                            break 'scan;
                        }
                    }
                    // the nearest same-key partner is judged; farther ones may not
                    // leapfrog it.
                    break;
                }
            }
        }
        Ok(entries
            .into_iter()
            .map(|(verb, key)| format!("{verb} {key}\n"))
            .collect())
    }

    /// May two entries on DIFFERENT keys slide past each other? Only when the lock
    /// froze this verb pair as cross-item commuting; an unknown verb is opaque.
    pub fn mobile(&self, a: &str, b: &str) -> bool {
        self.commutes.contains(&Self::unordered(a, b))
    }

    /// The single verb `first`-then-`then` on ONE key equals — `None` is honest
    /// silence, and silence means both lines stay.
    pub fn collapse(&self, first: &str, then: &str) -> Option<String> {
        if first == then && self.projections.contains(first) {
            return Some(first.to_string());
        }
        self.collapses
            .get(&(first.to_string(), then.to_string()))
            .cloned()
    }

    /// `add_a` → `("add", "a")`; a name without the miniature's one-letter item suffix
    /// is item-blind (`declare` → `("declare", "")`).
    fn split_verb(op: &str) -> (String, String) {
        match op.rsplit_once('_') {
            Some((base, item)) if item.len() == 1 => (base.to_string(), item.to_string()),
            _ => (op.to_string(), String::new()),
        }
    }

    fn unordered(a: &str, b: &str) -> (String, String) {
        if a <= b {
            (a.to_string(), b.to_string())
        } else {
            (b.to_string(), a.to_string())
        }
    }

    /// `f(g(x))` → `["f", "g"]` when the term is a pure nest over one variable;
    /// anything else (a second variable, an infix symbol) carries no journal rule.
    fn nest(term: &str) -> Option<Vec<String>> {
        let mut ops = Vec::new();
        let mut t = term.trim();
        while let Some((head, rest)) = t.split_once('(') {
            let head = head.trim();
            if head.is_empty() || !head.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                return None;
            }
            ops.push(head.to_string());
            t = rest.trim_end().strip_suffix(')')?;
        }
        let t = t.trim();
        (!ops.is_empty()
            && !t.is_empty()
            && t.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'))
        .then_some(ops)
    }
}

#[cfg(test)]
mod probes {
    use super::SquashRules;
    use crate::discover::verbs::state::VerbAlgebra;
    use crate::discover::Spec;

    fn rules() -> SquashRules {
        SquashRules::from_spec(&Spec::of::<VerbAlgebra>())
    }

    /// THE HEADLINE: the rule table is READ OFF the frozen algebra, never authored —
    /// the composition stanza's squash table, the projection laws, and the cross-item
    /// commutations arrive as equations and come out as rules. The collect precedent
    /// completed: a verb implemented BY the frozen spec of the verbs themselves.
    #[test]
    fn the_rule_table_is_derived_not_authored() {
        let r = rules();
        for (first, then, one) in [
            ("add", "collect", "collect"),
            ("edit", "collect", "collect"),
            ("collect", "edit", "collect"),
        ] {
            assert_eq!(
                r.collapses.get(&(first.to_string(), then.to_string())),
                Some(&one.to_string()),
                "`{first}` then `{then}` should collapse to `{one}`"
            );
        }
        for verb in ["add", "edit", "collect", "declare"] {
            assert!(r.projections.contains(verb), "`{verb}` replays safely");
        }
        assert!(r.mobile("edit", "collect"), "cross-key edit/collect slides");
        assert!(
            r.mobile("add", "declare"),
            "declare is item-blind — it slides"
        );
        assert!(
            !r.mobile("place", "collect"),
            "`place` is outside the algebra — opaque"
        );
    }

    /// The working case: an add and a collect on the same key meet across an unrelated
    /// edit (the lock licenses the slide) and collapse to the one collect the frozen
    /// table names.
    #[test]
    fn the_journal_compacts_to_law_normal_form() {
        let journal = "add src/m.rs — fn x\nedit src/m.rs — fn y\ncollect src/m.rs — fn x\n";
        assert_eq!(
            rules().compact(journal).unwrap(),
            "collect src/m.rs — fn x\nedit src/m.rs — fn y\n"
        );
    }

    /// Silence is load-bearing: add-then-edit on one key composes to NO single verb (no
    /// verb makes a twice-touched item from an absent one), so the table is silent and
    /// the journal keeps the whole story — squash never invents a collapse.
    #[test]
    fn a_pair_the_lock_is_silent_on_keeps_both_lines() {
        let journal = "add src/m.rs — fn x\nedit src/m.rs — fn x\n";
        assert_eq!(rules().compact(journal).unwrap(), journal);
    }

    /// A verb the algebra does not model is OPAQUE: nothing slides past `place`, so the
    /// same-key pair around it stays apart — mobility only where the lock speaks.
    #[test]
    fn an_unmodelled_verb_is_opaque() {
        let journal = "add src/m.rs — fn x\n\
                       place src/m.rs — re-placed canonically\n\
                       collect src/m.rs — fn x\n";
        assert_eq!(rules().compact(journal).unwrap(), journal);
    }

    /// Replay noise: the projection laws make a repeated identical entry one entry —
    /// the same laws that make journal replay safe make its record deduplicate.
    #[test]
    fn replay_noise_is_one_entry() {
        let journal = "edit src/m.rs — fn bump\nedit src/m.rs — fn bump\n";
        assert_eq!(
            rules().compact(journal).unwrap(),
            "edit src/m.rs — fn bump\n"
        );
    }

    /// The one hard refusal: the journal is machine-written, so a line the grammar
    /// cannot read means a hand touched the record — named by line, nothing written.
    #[test]
    fn a_hand_touched_line_refuses() {
        let refusal = rules().compact("add src/m.rs\n").unwrap_err();
        assert!(refusal.contains("line 1"), "{refusal}");
    }
}
