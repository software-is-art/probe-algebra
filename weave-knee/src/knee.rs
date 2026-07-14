use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::corpus;
use crate::record;
use crate::score::{score, Scores};

/// The declared floor mean relation coverage must hold for a fanout to count as
/// woven. Declared, not derived — moving it is a ratified judgment, and the whole
/// curve is frozen beside the verdict so a moved floor cannot rewrite history.
pub const RELATION_FLOOR: f64 = 0.9;

/// One fanout's averaged scores across its trials.
#[derive(Debug, Clone, PartialEq)]
pub struct CurvePoint {
    pub fanout: usize,
    pub trials: usize,
    pub mention: f64,
    pub claim_cov: f64,
    pub relation_cov: f64,
    pub foil_rej: f64,
    pub gain: f64,
    pub e_foil_rej: f64,
    pub topo_recall: f64,
    pub topo_prec: f64,
    pub quality: f64,
}

/// Average the scored trials per fanout, ascending.
pub fn curve(scores: &[Scores]) -> Vec<CurvePoint> {
    let mut by_fanout: BTreeMap<usize, Vec<&Scores>> = BTreeMap::new();
    for s in scores {
        by_fanout.entry(s.fanout).or_default().push(s);
    }
    by_fanout
        .into_iter()
        .map(|(fanout, group)| {
            let n = group.len() as f64;
            CurvePoint {
                fanout,
                trials: group.len(),
                mention: group.iter().map(|s| s.mention).sum::<f64>() / n,
                claim_cov: group.iter().map(|s| s.claim_cov).sum::<f64>() / n,
                relation_cov: group.iter().map(|s| s.relation_cov).sum::<f64>() / n,
                foil_rej: group.iter().map(|s| s.foil_rej).sum::<f64>() / n,
                gain: group.iter().map(|s| s.gain).sum::<f64>() / n,
                e_foil_rej: group.iter().map(|s| s.e_foil_rej).sum::<f64>() / n,
                topo_recall: group.iter().map(|s| s.topo_recall).sum::<f64>() / n,
                topo_prec: group.iter().map(|s| s.topo_prec).sum::<f64>() / n,
                quality: group.iter().map(|s| f64::from(s.quality)).sum::<f64>() / n,
            }
        })
        .collect()
}

/// Where the sweep put the knee. `At(k)`: k held the floor and the NEXT measured
/// fanout dropped it (conservative on a non-monotonic curve: the first drop ends the
/// held prefix). `NotReached(k)`: the floor held through the largest measured fanout
/// — a LOWER BOUND, keep sweeping. `BelowSweep`: even the smallest fanout dropped it.
#[derive(Debug, Clone, PartialEq)]
pub enum Knee {
    At(usize),
    NotReached(usize),
    BelowSweep,
}

pub fn knee(curve: &[CurvePoint]) -> Knee {
    let mut held = None;
    for point in curve {
        if point.relation_cov + 1e-9 >= RELATION_FLOOR {
            held = Some(point.fanout);
        } else {
            return match held {
                Some(k) => Knee::At(k),
                None => Knee::BelowSweep,
            };
        }
    }
    match held {
        Some(k) => Knee::NotReached(k),
        None => Knee::BelowSweep,
    }
}

/// Render the knee spec from scored trials grouped by (weaver, judge).
pub fn render(groups: &BTreeMap<(String, String), Vec<Scores>>) -> String {
    let mut s = String::new();
    s.push_str(
        "# weave-knee v2 — the functor's three failure modes, derived (regenerate: \
         `cargo run -p weave-knee --example knee -- freeze`)\n",
    );
    s.push_str(&format!(
        "# identity: relation floor {RELATION_FLOOR} — a fanout is WOVEN while mean \
         relation coverage holds this line\n"
    ));
    s.push_str(
        "# composition: gain — planted emergent facts the narrative made explicit; \
         concatenation scores zero\n# invertibility: topo — ring edges a fresh reader \
         recovered from the narrative alone\n",
    );
    if groups.is_empty() {
        s.push_str("\nno trials recorded — run the sweep, then freeze.\n");
        return s;
    }
    for ((weaver, judge), scores) in groups {
        s.push_str(&format!("\n## weaver: {weaver} — judge: {judge}\n"));
        s.push_str(
            "fanout  trials  mention  claims  relations  foil-rej  gain  e-foil  \
             topo-rec  topo-prec  quality\n",
        );
        let c = curve(scores);
        for p in &c {
            s.push_str(&format!(
                "{:>6}  {:>6}  {:>7.3}  {:>6.3}  {:>9.3}  {:>8.3}  {:>4.2}  {:>6.3}  \
                 {:>8.3}  {:>9.3}  {:>7.1}\n",
                p.fanout,
                p.trials,
                p.mention,
                p.claim_cov,
                p.relation_cov,
                p.foil_rej,
                p.gain,
                p.e_foil_rej,
                p.topo_recall,
                p.topo_prec,
                p.quality
            ));
        }
        s.push_str(&match knee(&c) {
            Knee::At(k) => format!(
                "identity knee: {k} — the last fanout holding the floor; the next \
                 measured fanout dropped it\n"
            ),
            Knee::NotReached(k) => format!(
                "identity knee: >= {k} — the floor held through the whole sweep; a \
                 lower bound, keep sweeping\n"
            ),
            Knee::BelowSweep => "identity knee: below the sweep — even the smallest \
                                 fanout dropped the floor\n"
                .to_string(),
        });
        s.push_str(&match gain_knee(&c) {
            Knee::At(k) => format!(
                "gain knee: {k} — the last fanout with positive gain; past it the \
                 weave stops earning its bytes\n"
            ),
            Knee::NotReached(k) => format!(
                "gain knee: >= {k} — gain stayed positive through the whole sweep; a \
                 lower bound, keep sweeping\n"
            ),
            Knee::BelowSweep => "gain knee: below the sweep — no measured fanout \
                                 showed positive gain; the weave never beat \
                                 concatenation\n"
                .to_string(),
        });
    }
    s.push_str(
        "\n# honest frame: a sweep is a sample and the judge is a model — the knees are \
         evidence, never proof.\n",
    );
    s
}

/// Re-derive the knee lock from the committed trials: parse every `.trial` under
/// `trials_dir`, regenerate each trial's corpus from (fanout, seed), score, render.
/// Never calls a model — the gate stays mechanical.
pub fn lock_in(spec_dir: &Path, trials_dir: &Path) -> Result<spec_lock::Lock, String> {
    let mut groups: BTreeMap<(String, String), Vec<Scores>> = BTreeMap::new();
    for path in trial_files(trials_dir)? {
        let text =
            std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let rec = record::parse(&text).map_err(|e| format!("{}: {e}", path.display()))?;
        let t = corpus::trial(rec.fanout, rec.seed);
        let scored = score(&t, &rec).map_err(|e| format!("{}: {e}", path.display()))?;
        groups
            .entry((rec.weaver.clone(), rec.judge.clone()))
            .or_default()
            .push(scored);
    }
    Ok(spec_lock::Lock {
        name: "weave knee".into(),
        path: spec_dir.join("knee.spec"),
        live: render(&groups),
    })
}

fn trial_files(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    if !dir.exists() {
        return Ok(files);
    }
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let entries = std::fs::read_dir(&d)
            .map_err(|e| format!("{}: {e}", d.display()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("{}: {e}", d.display()))?;
        for e in entries {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().is_some_and(|x| x == "trial") {
                files.push(p);
            }
        }
    }
    files.sort();
    Ok(files)
}

/// The gain knee: the longest ascending prefix of fanouts whose mean gain stays
/// STRICTLY positive — the weave still states composites concatenation would not.
/// Where gain touches zero, the parent exterior has stopped earning its bytes;
/// that fanout, not the relation floor, is where the functor dies under load.
pub fn gain_knee(curve: &[CurvePoint]) -> Knee {
    let mut held = None;
    for point in curve {
        if point.gain > 0.0 {
            held = Some(point.fanout);
        } else {
            return match held {
                Some(k) => Knee::At(k),
                None => Knee::BelowSweep,
            };
        }
    }
    match held {
        Some(k) => Knee::NotReached(k),
        None => Knee::BelowSweep,
    }
}

#[cfg(test)]
mod probes {
    use super::*;

    fn point(fanout: usize, relation_cov: f64, gain: f64) -> CurvePoint {
        CurvePoint {
            fanout,
            trials: 2,
            mention: 1.0,
            claim_cov: 1.0,
            relation_cov,
            foil_rej: 1.0,
            gain,
            e_foil_rej: 1.0,
            topo_recall: 1.0,
            topo_prec: 1.0,
            quality: 4.0,
        }
    }

    #[test]
    fn the_knee_is_the_last_fanout_holding_the_floor() {
        let c = vec![point(2, 1.0, 0.5), point(4, 0.95, 0.5), point(6, 0.7, 0.5)];
        assert_eq!(knee(&c), Knee::At(4));
    }

    #[test]
    fn a_sweep_the_floor_survives_yields_a_lower_bound() {
        let c = vec![point(2, 1.0, 0.5), point(4, 0.92, 0.5)];
        assert_eq!(knee(&c), Knee::NotReached(4));
    }

    #[test]
    fn a_sweep_that_never_holds_is_below_the_sweep() {
        let c = vec![point(2, 0.5, 0.5), point(4, 0.4, 0.5)];
        assert_eq!(knee(&c), Knee::BelowSweep);
    }

    #[test]
    fn a_non_monotonic_curve_ends_at_the_first_drop() {
        let c = vec![point(2, 1.0, 0.5), point(4, 0.5, 0.5), point(6, 1.0, 0.5)];
        assert_eq!(knee(&c), Knee::At(2));
    }

    #[test]
    fn the_gain_knee_dies_where_gain_touches_zero() {
        let c = vec![point(2, 1.0, 0.8), point(4, 1.0, 0.25), point(6, 1.0, 0.0)];
        assert_eq!(gain_knee(&c), Knee::At(4));
        let c = vec![point(2, 1.0, 0.8), point(4, 1.0, 0.25)];
        assert_eq!(gain_knee(&c), Knee::NotReached(4));
        let c = vec![point(2, 1.0, 0.0)];
        assert_eq!(gain_knee(&c), Knee::BelowSweep);
    }

    #[test]
    fn an_empty_trial_set_renders_honestly() {
        let rendered = render(&BTreeMap::new());
        assert!(rendered.contains("no trials recorded"));
    }
}
