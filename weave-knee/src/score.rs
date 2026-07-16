use std::collections::BTreeMap;

use crate::corpus::{word_budget, Trial};
use crate::prompt::candidates;
use crate::record::Record;

/// One trial, scored. `mention` is the retrieval floor; `claim_cov` is retrieval
/// under load; `relation_cov` is IDENTITY preservation — every edge carried;
/// `foil_rej` is the calibration (an entailed foil convicts weaver or judge).
/// v2 adds the functor's other two failure modes: `gain` is COMPOSITION — the
/// fraction of planted emergent facts the narrative made explicit (concatenation
/// scores zero by construction); `e_foil_rej` polices gain (a stated emergent foil
/// is an invented composite); `topo_recall`/`topo_prec` are INVERTIBILITY — the
/// ring edges a fresh reader recovered from the narrative alone, and how many of
/// its answers were real.
#[derive(Debug, Clone, PartialEq)]
pub struct Scores {
    pub fanout: usize,
    pub mention: f64,
    pub claim_cov: f64,
    pub relation_cov: f64,
    pub foil_rej: f64,
    pub gain: f64,
    pub e_foil_rej: f64,
    pub topo_recall: f64,
    pub topo_prec: f64,
    pub quality: u8,
    pub words: usize,
    pub within_budget: bool,
    pub prose_only: bool,
}

/// Score a record against its regenerated trial. Every candidate in BOTH censuses
/// must carry a verdict — a judge that skipped one is an incomplete census, refused.
/// Edges are matched exactly (from, to); an empty edge list is a zero, not an error.
pub fn score(t: &Trial, r: &Record) -> Result<Scores, String> {
    let ruled: BTreeMap<&str, bool> = r.verdicts.iter().map(|(id, e)| (id.as_str(), *e)).collect();
    let mut claims = (0usize, 0usize);
    let mut relations = (0usize, 0usize);
    let mut foils = (0usize, 0usize);
    for c in candidates(t) {
        let entailed = *ruled
            .get(c.id.as_str())
            .ok_or_else(|| format!("no verdict for candidate `{}`", c.id))?;
        if !c.truth {
            foils.1 += 1;
            if !entailed {
                foils.0 += 1;
            }
        } else if c.id.starts_with('r') {
            relations.1 += 1;
            if entailed {
                relations.0 += 1;
            }
        } else {
            claims.1 += 1;
            if entailed {
                claims.0 += 1;
            }
        }
    }
    let said: BTreeMap<&str, bool> = r.stated.iter().map(|(id, s)| (id.as_str(), *s)).collect();
    let mut emergents = (0usize, 0usize);
    let mut e_foils = (0usize, 0usize);
    for c in crate::prompt::emergent_candidates(t) {
        let was_stated = *said
            .get(c.id.as_str())
            .ok_or_else(|| format!("no stated ruling for emergent `{}`", c.id))?;
        if c.truth {
            emergents.1 += 1;
            if was_stated {
                emergents.0 += 1;
            }
        } else {
            e_foils.1 += 1;
            if !was_stated {
                e_foils.0 += 1;
            }
        }
    }
    let ring: Vec<(&str, &str)> = t
        .relations
        .iter()
        .map(|rel| {
            (
                t.children[rel.from].lexeme.as_str(),
                t.children[rel.to].lexeme.as_str(),
            )
        })
        .collect();
    let recovered = ring
        .iter()
        .filter(|(from, to)| r.edges.iter().any(|(f, t2)| f == from && t2 == to))
        .count();
    let real = r
        .edges
        .iter()
        .filter(|(f, t2)| ring.iter().any(|(from, to)| f == from && t2 == to))
        .count();
    let mentioned = t
        .children
        .iter()
        .filter(|c| r.narrative.contains(&c.lexeme))
        .count();
    let words = r.narrative.split_whitespace().count();
    let budget = word_budget(t.fanout);
    Ok(Scores {
        fanout: t.fanout,
        mention: mentioned as f64 / t.children.len() as f64,
        claim_cov: claims.0 as f64 / claims.1 as f64,
        relation_cov: relations.0 as f64 / relations.1 as f64,
        foil_rej: foils.0 as f64 / foils.1 as f64,
        gain: emergents.0 as f64 / emergents.1 as f64,
        e_foil_rej: e_foils.0 as f64 / e_foils.1 as f64,
        topo_recall: recovered as f64 / ring.len() as f64,
        topo_prec: if r.edges.is_empty() {
            0.0
        } else {
            real as f64 / r.edges.len() as f64
        },
        quality: r.quality,
        words,
        // a declared tolerance margin: a budget gate that fails on one word is
        // flaky, and flaky locks teach people to ignore locks.
        within_budget: words <= budget + budget / 10,
        prose_only: !r.narrative.lines().any(is_listish),
    })
}

/// A narrative that reverts to bullets or headings is stitching, not weaving; the
/// weave prompt forbids it, so a listish line is a mechanical finding.
fn is_listish(line: &str) -> bool {
    let l = line.trim_start();
    l.starts_with('-') || l.starts_with('*') || l.starts_with('#') || {
        let digits = l.chars().take_while(|c| c.is_ascii_digit()).count();
        digits > 0 && l[digits..].starts_with(". ")
    }
}

#[cfg(test)]
mod probes {
    use super::*;
    use crate::corpus::trial;
    use crate::prompt::emergent_candidates;
    use crate::record::Record;

    fn perfect_record(t: &Trial) -> Record {
        let narrative = t
            .children
            .iter()
            .map(|c| c.lexeme.as_str())
            .collect::<Vec<_>>()
            .join(" weaves into ");
        Record {
            fanout: t.fanout,
            seed: t.seed,
            weaver: "synthetic".to_string(),
            judge: "synthetic".to_string(),
            quality: 5,
            narrative,
            verdicts: candidates(t)
                .iter()
                .map(|c| (c.id.clone(), c.truth))
                .collect(),
            stated: emergent_candidates(t)
                .iter()
                .map(|c| (c.id.clone(), c.truth))
                .collect(),
            edges: t
                .relations
                .iter()
                .map(|r| {
                    (
                        t.children[r.from].lexeme.clone(),
                        t.children[r.to].lexeme.clone(),
                    )
                })
                .collect(),
        }
    }

    #[test]
    fn a_perfect_record_scores_ones_across_the_board() {
        let t = trial(3, 1);
        let s = score(&t, &perfect_record(&t)).expect("score");
        assert_eq!(s.mention, 1.0);
        assert_eq!(s.claim_cov, 1.0);
        assert_eq!(s.relation_cov, 1.0);
        assert_eq!(s.foil_rej, 1.0);
        assert_eq!(s.gain, 1.0);
        assert_eq!(s.e_foil_rej, 1.0);
        assert_eq!(s.topo_recall, 1.0);
        assert_eq!(s.topo_prec, 1.0);
        assert!(s.within_budget);
        assert!(s.prose_only);
    }

    #[test]
    fn a_dropped_relation_moves_only_relation_coverage() {
        let t = trial(3, 1);
        let mut r = perfect_record(&t);
        for v in &mut r.verdicts {
            if v.0 == "r0" {
                v.1 = false;
            }
        }
        let s = score(&t, &r).expect("score");
        assert_eq!(s.claim_cov, 1.0);
        assert!((s.relation_cov - 2.0 / 3.0).abs() < 1e-9);
        assert_eq!(s.foil_rej, 1.0);
        assert_eq!(s.gain, 1.0);
    }

    #[test]
    fn an_unstated_emergent_moves_only_gain_and_a_stated_foil_moves_only_its_rejection() {
        let t = trial(3, 1);
        let mut r = perfect_record(&t);
        for s in &mut r.stated {
            if s.0.starts_with('e') && !s.0.ends_with('F') {
                s.1 = false;
                break;
            }
        }
        let scored = score(&t, &r).expect("score");
        assert!(scored.gain < 1.0);
        assert_eq!(scored.e_foil_rej, 1.0);
        assert_eq!(scored.relation_cov, 1.0);

        let mut r = perfect_record(&t);
        for s in &mut r.stated {
            if s.0.ends_with('F') {
                s.1 = true;
                break;
            }
        }
        let scored = score(&t, &r).expect("score");
        assert_eq!(scored.gain, 1.0);
        assert!(scored.e_foil_rej < 1.0);
    }

    #[test]
    fn topology_scores_split_recall_from_precision() {
        let t = trial(4, 2);
        let mut r = perfect_record(&t);
        r.edges.pop();
        let s = score(&t, &r).expect("score");
        assert!((s.topo_recall - 0.75).abs() < 1e-9);
        assert_eq!(s.topo_prec, 1.0);

        let mut r = perfect_record(&t);
        r.edges.push(("Nowhere".to_string(), "Nothing".to_string()));
        let s = score(&t, &r).expect("score");
        assert_eq!(s.topo_recall, 1.0);
        assert!((s.topo_prec - 0.8).abs() < 1e-9);

        let mut r = perfect_record(&t);
        r.edges.clear();
        let s = score(&t, &r).expect("score");
        assert_eq!(s.topo_recall, 0.0);
        assert_eq!(s.topo_prec, 0.0);
    }

    #[test]
    fn a_missing_verdict_is_refused_not_shrugged() {
        let t = trial(3, 1);
        let mut r = perfect_record(&t);
        r.verdicts.pop();
        assert!(score(&t, &r)
            .unwrap_err()
            .contains("no verdict for candidate"));
        let mut r = perfect_record(&t);
        r.stated.pop();
        assert!(score(&t, &r)
            .unwrap_err()
            .contains("no stated ruling for emergent"));
    }

    #[test]
    fn listish_narratives_are_caught() {
        assert!(is_listish("- a bullet"));
        assert!(is_listish("  * another"));
        assert!(is_listish("2. numbered"));
        assert!(is_listish("# a heading"));
        assert!(!is_listish("Plain prose, 2. is not how it starts."));
        assert!(!is_listish("A sentence from 1876. And on."));
    }
}
