//!
//! judgment — THE SENSITIVITY DRILL: a world judge proven deaf to nothing, by
//! perturbing its live state one fact at a time.
//!
//! The world locks (perimeter, infra, substrate) judge a `Live*` struct — field reads
//! of state the tree cannot contain — and refuse departures by name. Their probes were
//! hand-written per departure, and the source-level sweeps kept finding the gaps
//! between them: an `==` flipped to `!=` inside a membership check survives when no
//! probe stages EXACTLY ONE missing element among present ones. Every one of those
//! survivors was the same species: a fact the judge could go silently deaf to.
//!
//! This module is the dent sweep (`discover::mutation`) aimed at judges. The domain of
//! a judge is its live state, so the minimal meaning change is a LIVE DENT: one field
//! perturbed, everything else the applied fixture. [`LiveDent::drill`] runs the whole
//! battery — the applied fixture must hold, every dent must MOVE the verdict, and the
//! verdict must NAME the dented fact — so a judge that stops distinguishing a fact
//! fails the drill by the fact's name, not by a source line.
//!
//! Each `Live*` struct enumerates its own dents (`dents()`), and completeness is a
//! COMPILE-TIME pin, not a convention: the enumerator opens with a full destructuring
//! `let Self { .. } = self` naming every field, so adding a field to the live state
//! refuses to compile until the dent list learns what perturbing it means. The one
//! judgment the enumerator encodes is which perturbations are REFUSAL-WORTHY — floor
//! semantics accept widenings (an extra origin, an extra secret), so a dent list is
//! the refusal-worthy perturbations only, and that choice is reviewable data.
//!
//! Honest frame: a live dent perturbs the judge's INPUT — it proves the judge
//! distinguishes states, not that the extraction reads the world faithfully (the
//! extraction examples carry their own parse probes). And under the source-level
//! sweeps this drill is itself a probe: a mutant that deafens a judge to any fact now
//! dies here, which is the point — the class that kept surviving is closed as a
//! class, not survivor by survivor.

/// One live-state dent: a single perturbed fact, what was perturbed, and the key the
/// refusing verdict must name.
pub struct LiveDent<L> {
    /// What was perturbed, for the drill's own report.
    pub what: String,
    /// The dented live state — the applied fixture with exactly one fact wrong.
    pub live: L,
    /// A string the verdict's violations must contain — the fact, named.
    pub must_name: String,
}

impl<L> LiveDent<L> {
    /// Drill a judge against its dent battery. `Ok` carries one held line per dent
    /// the judge distinguished and named; `Err` carries every failure: an applied
    /// fixture that does not hold, a dent the judge is DEAF to (verdict stayed
    /// green), or a verdict that moved without naming the dented fact (a refusal
    /// nobody can act on).
    pub fn drill(
        judge: impl Fn(&L) -> Result<Vec<String>, Vec<String>>,
        applied: &L,
        dents: Vec<LiveDent<L>>,
    ) -> Result<Vec<String>, Vec<String>> {
        let mut held = Vec::new();
        let mut refusals = Vec::new();
        if let Err(violations) = judge(applied) {
            refusals.push(format!(
                "the applied fixture does not hold — every dent verdict below is \
                 meaningless until it does: {}",
                violations.join("; ")
            ));
        }
        for dent in dents {
            match judge(&dent.live) {
                Ok(_) => refusals.push(format!(
                    "the judge is DEAF to {} — the verdict stayed green with the fact \
                     wrong",
                    dent.what
                )),
                Err(violations) if violations.iter().any(|v| v.contains(&dent.must_name)) => {
                    held.push(format!("sensitive to {}", dent.what));
                }
                Err(violations) => refusals.push(format!(
                    "the verdict moved on {} but never named `{}` — a refusal nobody \
                     can act on: {}",
                    dent.what,
                    dent.must_name,
                    violations.join("; ")
                )),
            }
        }
        if refusals.is_empty() {
            Ok(held)
        } else {
            Err(refusals)
        }
    }
}

#[cfg(test)]
mod probes {
    use super::*;

    /// The applied state for a toy judge: one bool fact, one list fact.
    #[derive(Clone)]
    struct ToyLive {
        locked: bool,
        names: Vec<String>,
    }

    fn toy_judge(live: &ToyLive) -> Result<Vec<String>, Vec<String>> {
        let mut held = Vec::new();
        let mut violations = Vec::new();
        if live.locked {
            held.push("locked".to_string());
        } else {
            violations.push("the door is UNLOCKED".to_string());
        }
        for name in ["a", "b"] {
            if live.names.iter().any(|n| n == name) {
                held.push(format!("name `{name}` present"));
            } else {
                violations.push(format!("name `{name}` is missing"));
            }
        }
        if violations.is_empty() {
            Ok(held)
        } else {
            Err(violations)
        }
    }

    fn applied() -> ToyLive {
        ToyLive {
            locked: true,
            names: vec!["a".to_string(), "b".to_string()],
        }
    }

    fn dents() -> Vec<LiveDent<ToyLive>> {
        let base = applied();
        let mut out = vec![LiveDent {
            what: "locked flipped".to_string(),
            live: ToyLive {
                locked: false,
                ..base.clone()
            },
            must_name: "UNLOCKED".to_string(),
        }];
        for name in &base.names {
            let mut live = base.clone();
            live.names.retain(|n| n != name);
            out.push(LiveDent {
                what: format!("name `{name}` removed"),
                live,
                must_name: format!("`{name}` is missing"),
            });
        }
        out
    }

    /// A sensitive judge passes the whole battery, and the held lines name each
    /// distinguished fact.
    #[test]
    fn a_sensitive_judge_holds() {
        let held = LiveDent::drill(toy_judge, &applied(), dents()).expect("sensitive");
        assert_eq!(held.len(), 3);
        assert!(held.iter().any(|h| h == "sensitive to name `b` removed"));
    }

    /// Every failure mode refuses by name, ONE ARM AT A TIME: a deaf judge (green on
    /// a dent), a mute judge (moves without naming the fact), and a fixture that does
    /// not hold — each its own refusal.
    #[test]
    fn each_failure_mode_refuses_by_name() {
        // deaf: a judge that only reads `locked` never notices a name dent.
        let deaf = |live: &ToyLive| {
            if live.locked {
                Ok(vec!["locked".to_string()])
            } else {
                Err(vec!["the door is UNLOCKED".to_string()])
            }
        };
        let refusals = LiveDent::drill(deaf, &applied(), dents()).unwrap_err();
        assert!(refusals
            .iter()
            .any(|r| r.contains("DEAF to name `a` removed")));
        assert!(
            !refusals.iter().any(|r| r.contains("locked flipped")),
            "the fact it distinguishes must not be accused: {refusals:#?}"
        );

        // mute: the verdict moves but the violation names nothing actionable.
        let mute = |live: &ToyLive| {
            if live.locked && live.names.len() == 2 {
                Ok(vec![])
            } else {
                Err(vec!["something is off".to_string()])
            }
        };
        let refusals = LiveDent::drill(mute, &applied(), dents()).unwrap_err();
        assert!(
            refusals
                .iter()
                .any(|r| r.contains("never named ``a` is missing`")
                    && r.contains("nobody can act on"))
        );

        // a broken fixture is refused before any dent verdict is trusted.
        let mut broken = applied();
        broken.locked = false;
        let refusals = LiveDent::drill(toy_judge, &broken, dents()).unwrap_err();
        assert!(refusals
            .iter()
            .any(|r| r.contains("applied fixture does not hold")));
    }
}
