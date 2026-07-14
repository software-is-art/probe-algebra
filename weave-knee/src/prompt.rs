use crate::corpus::{word_budget, Rng, Trial};

/// One judge candidate: a claim or a foil, blinded — the judge sees ids and texts,
/// never which is which.
#[derive(Debug, Clone, PartialEq)]
pub struct Candidate {
    pub id: String,
    pub text: String,
    pub truth: bool,
}

/// The weave prompt: frame, rules, and the child exteriors. The weaver sees
/// exteriors only — never the candidate list, never a foil, never a planted
/// emergent fact. The synthesis rule licenses the semantic step gain is scored
/// on, without naming any target.
pub fn weave(t: &Trial) -> String {
    let budget = word_budget(t.fanout);
    let mut s = String::new();
    s.push_str(&format!(
        "You are writing the exterior narrative of a fictional system called {} for a \
         technical stakeholder. Below are the exteriors of its {} modules. Weave them \
         into ONE cohesive narrative — a single flowing account of how the system works \
         as a whole.\n\nRules:\n- At most {budget} words.\n- Flowing prose only: no \
         bullet lists, no headings, no numbered items.\n- Mention every module by its \
         exact name.\n- State what the exteriors below state and what they jointly \
         imply; invent nothing beyond them.\n- Where two modules' facts combine into \
         something neither states alone — a combined bound, a shared floor, a property \
         of the whole system — state that consequence explicitly, with its number when \
         it has one.\n- Express every relation between modules; the relations are the \
         story.\n- Output only the narrative.\n\nModule exteriors:\n",
        t.system, t.fanout
    ));
    for (i, c) in t.children.iter().enumerate() {
        s.push_str(&format!("\n{} — the {}\n", c.lexeme, c.role));
        for cl in &c.claims {
            s.push_str(&format!("- {}\n", cl.text));
        }
        for r in &t.relations {
            if r.from == i {
                s.push_str(&format!("- relation: {}\n", r.text));
            } else if r.to == i {
                s.push_str(&format!("- relation (incoming): {}\n", r.text));
            }
        }
    }
    s
}

/// Every claim and every foil, blinded and deterministically shuffled so the judge's
/// list carries no positional hint of which statements are true.
pub fn candidates(t: &Trial) -> Vec<Candidate> {
    let mut v = Vec::new();
    for c in &t.children {
        for cl in &c.claims {
            v.push(Candidate {
                id: cl.id.clone(),
                text: cl.text.clone(),
                truth: true,
            });
            v.push(Candidate {
                id: format!("{}F", cl.id),
                text: cl.foil.clone(),
                truth: false,
            });
        }
    }
    for r in &t.relations {
        v.push(Candidate {
            id: r.id.clone(),
            text: r.text.clone(),
            truth: true,
        });
        v.push(Candidate {
            id: format!("{}F", r.id),
            text: r.foil.clone(),
            truth: false,
        });
    }
    let mut rng = Rng::new(t.seed ^ 0x00C0_FFEE ^ ((t.fanout as u64) << 32));
    for i in (1..v.len()).rev() {
        v.swap(i, rng.pick(i + 1));
    }
    v
}

/// The judge prompt: the narrative, the blinded candidates, and the exact output
/// contract `record::parse_judge` reads. Two censuses under two criteria: claims
/// and relations under strict ENTAILMENT, emergent facts under STATED — explicit
/// in the text, where the judge deriving it herself does not count. The system is
/// fictional, so outside knowledge cannot rescue a dropped thread.
pub fn judge(t: &Trial, narrative: &str) -> String {
    let mut s = String::new();
    s.push_str(
        "You are a strict entailment judge. Below is a narrative about a fictional \
         system, then two lists of candidate statements. For each ENTAILMENT candidate, \
         decide whether the narrative ENTAILS it — states it outright or necessarily \
         implies it. For each SYNTHESIS candidate, decide whether the narrative STATES \
         it — says it outright or in direct paraphrase; a statement you could derive \
         yourself but the narrative never makes explicit is NOT-STATED. Judge only from \
         the narrative text; the system is fictional, so outside knowledge cannot help. \
         When in doubt, NOT-ENTAILED and NOT-STATED.\n\nNarrative:\n",
    );
    for line in narrative.lines() {
        s.push_str("> ");
        s.push_str(line);
        s.push('\n');
    }
    s.push_str("\nEntailment candidates:\n");
    for c in candidates(t) {
        s.push_str(&format!("{}: {}\n", c.id, c.text));
    }
    s.push_str("\nSynthesis candidates:\n");
    for c in emergent_candidates(t) {
        s.push_str(&format!("{}: {}\n", c.id, c.text));
    }
    s.push_str(
        "\nOutput exactly one line per entailment candidate, in the order given:\n\
         verdict <id> ENTAILED\nor\nverdict <id> NOT-ENTAILED\nThen exactly one line \
         per synthesis candidate, in the order given:\nstated <id> STATED\nor\nstated \
         <id> NOT-STATED\nThen one final line scoring the narrative as prose (cohesion, \
         one voice, not a stitched list of parts):\nquality <1-5>\nNo other text.\n",
    );
    s
}

/// Every emergent fact and its foil, blinded and deterministically shuffled — the
/// gain census the judge scores under the STATED criterion, kept separate from the
/// entailment candidates because the two criteria must never blur.
pub fn emergent_candidates(t: &Trial) -> Vec<Candidate> {
    let mut v = Vec::new();
    for e in crate::corpus::emergents(t) {
        v.push(Candidate {
            id: e.id.clone(),
            text: e.text.clone(),
            truth: true,
        });
        v.push(Candidate {
            id: format!("{}F", e.id),
            text: e.foil.clone(),
            truth: false,
        });
    }
    let mut rng = Rng::new(t.seed ^ 0x00FA_CADE ^ ((t.fanout as u64) << 32));
    for i in (1..v.len()).rev() {
        v.swap(i, rng.pick(i + 1));
    }
    v
}

/// The reconstruction prompt — the functor's return leg. A fresh reader, given ONLY
/// the narrative and the module names in scrambled order, lists every directed
/// relation it can recover; `record::parse_edges` reads the answer.
pub fn reconstruct(t: &Trial, narrative: &str) -> String {
    let mut names: Vec<&str> = t.children.iter().map(|c| c.lexeme.as_str()).collect();
    let mut rng = Rng::new(t.seed ^ 0x0000_D00D ^ ((t.fanout as u64) << 32));
    for i in (1..names.len()).rev() {
        names.swap(i, rng.pick(i + 1));
    }
    let mut s = String::new();
    s.push_str(
        "You are reconstructing the topology of a fictional system from its narrative \
         alone. Below is the narrative, then the names of its modules in scrambled \
         order. Modules are connected by DIRECTED relations (one module hands off to, \
         signals, or reports to another). From the narrative text alone, list every \
         directed relation you can identify.\n\nNarrative:\n",
    );
    for line in narrative.lines() {
        s.push_str("> ");
        s.push_str(line);
        s.push('\n');
    }
    s.push_str("\nModules (scrambled order):\n");
    for n in &names {
        s.push_str(&format!("- {n}\n"));
    }
    s.push_str(
        "\nOutput exactly one line per relation you find, direction preserved:\n\
         edge <from-module> <to-module>\nUse exact module names. No other text.\n",
    );
    s
}

#[cfg(test)]
mod probes {
    use super::*;
    use crate::corpus::{emergents, trial};

    #[test]
    fn the_weaver_sees_every_claim_and_relation_and_no_foil_and_no_emergent() {
        let t = trial(4, 9);
        let w = weave(&t);
        for c in &t.children {
            assert!(w.contains(&c.lexeme));
            for cl in &c.claims {
                assert!(w.contains(&cl.text));
                assert!(!w.contains(&cl.foil));
            }
        }
        for r in &t.relations {
            assert!(w.contains(&r.text));
            assert!(!w.contains(&r.foil));
        }
        for e in emergents(&t) {
            assert!(!w.contains(&e.text), "{} leaked to the weaver", e.id);
            assert!(!w.contains(&e.foil), "{}F leaked to the weaver", e.id);
        }
    }

    #[test]
    fn candidates_pair_every_claim_with_its_foil_and_shuffle_deterministically() {
        let t = trial(5, 3);
        let cands = candidates(&t);
        assert_eq!(cands.len(), 2 * (3 * 5 + 5));
        assert_eq!(cands, candidates(&t));
        let truths = cands.iter().filter(|c| c.truth).count();
        assert_eq!(truths, cands.len() / 2);
        for c in &cands {
            if !c.truth {
                assert!(c.id.ends_with('F'));
            }
        }
    }

    #[test]
    fn emergent_candidates_pair_and_shuffle_the_gain_census() {
        let t = trial(5, 3);
        let cands = emergent_candidates(&t);
        assert_eq!(cands.len(), 2 * emergents(&t).len());
        assert_eq!(cands, emergent_candidates(&t));
        for c in &cands {
            assert!(c.id.starts_with('e'));
            assert_eq!(!c.truth, c.id.ends_with('F'));
        }
    }

    #[test]
    fn the_judge_sees_the_narrative_and_both_censuses() {
        let t = trial(3, 5);
        let j = judge(&t, "A line.\n\nAnother line.");
        assert!(j.contains("> A line."));
        assert!(j.contains("> Another line."));
        for c in candidates(&t) {
            assert!(j.contains(&format!("{}: {}", c.id, c.text)));
        }
        for c in emergent_candidates(&t) {
            assert!(j.contains(&format!("{}: {}", c.id, c.text)));
        }
        assert!(j.contains("stated <id> STATED"));
        assert!(j.contains("quality <1-5>"));
    }

    #[test]
    fn the_reconstructor_sees_every_module_but_no_relation_text() {
        let t = trial(6, 4);
        let p = reconstruct(&t, "A narrative that names nothing.");
        for c in &t.children {
            assert!(p.contains(&c.lexeme));
        }
        for r in &t.relations {
            assert!(!p.contains(&r.text), "{} leaked to the reconstructor", r.id);
        }
        assert!(p.contains("edge <from-module> <to-module>"));
    }
}
