//!
//! depend — compatibility as a COMPUTED property, per consumer: the dependence lock.
//!
//! Semver's integer pretends compatibility is a global property of a change. It is not —
//! it is a relation between a change and a consumer: the same release breaks one
//! downstream and is invisible to another, depending on which laws each one leans on.
//! The automatic releases publish the spec-lock diff (the whole truth); this module
//! computes the per-consumer verdict from it.
//!
//! A consumer declares its DEPENDENCES — the laws it actually relies on, by theory and
//! equation, as data in its own repository (freeze the render and the reliance itself
//! is drift-gated). Given the upstream theory locks at two points (the spec text files
//! from any two release tags), [`Dependence::judge`] answers the only question semver
//! ever tried to: did anything I depend on change?
//!
//!   - INTACT — the law holds in both, byte for byte;
//!   - CHANGED — the equation survived but its prose judgment moved, or vice versa
//!     (something about how discovery states the law is different — read the diff);
//!   - GONE — the law is absent from the new lock: the breaking case, named.
//!
//! A dependence naming no law in the BASELINE is a refusal, not a verdict — a consumer
//! cannot rely on a law that was never held, and a typo must never read as "intact".
//! Honest frame: this judges the DECLARED reliances only. A consumer leaning on
//! behaviour it never declared gets what semver always gave it — nothing.
//!
//! Two ways to hold the judgment, one per side of the boundary:
//!
//!   - **old vs new** — the CROSS-REPO consumer's tool: pin two release tags, judge
//!     your declared reliances between their lock texts, read the verdicts.
//!   - **committed vs committed** — the theory OWNER's tool, discovered downstream:
//!     judge the declared reliances with the committed lock on BOTH sides, in the
//!     owner's own suite. INTACT is then trivially the only passing verdict, which is
//!     the point — the protection is the REFUSAL path. The moment a re-bless drops a
//!     law some consumer declared, the baseline no longer holds it and the judgment
//!     refuses by equation, naming the reliance, before the ratification diff lands:
//!     self-judgment makes declared reliances un-droppable.

use std::path::Path;

use super::Spec;

/// One declared reliance: a theory's law, by its rendered equation (the stable key —
/// prose is judgment and may be re-worded; the equation is the law's identity).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Dependence {
    /// The theory, by lock stem (`router`, `date calculus` — the name in the lock header).
    pub theory: String,
    /// The law's equation, exactly as the lock renders it.
    pub equation: String,
}

/// One dependence's verdict between two lock texts.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Standing {
    /// Present in both, identical prose and equation.
    Intact,
    /// The equation is present in the new lock but its prose changed (or the law moved
    /// between the held and undecided bands) — the statement of the law drifted.
    Changed { now: String },
    /// Absent from the new lock — the breaking case.
    Gone,
}

/// The judgment of every declared dependence between an old and a new lock text.
#[derive(Debug)]
pub struct DependenceReport {
    pub theory: String,
    /// `(equation, standing)` per dependence, in declaration order.
    pub verdicts: Vec<(String, Standing)>,
}

impl Dependence {
    /// A reliance on `theory`'s law with this `equation`.
    pub fn on(theory: impl Into<String>, equation: impl Into<String>) -> Dependence {
        Dependence {
            theory: theory.into(),
            equation: equation.into(),
        }
    }

    /// Judge the declared dependences on one theory between two of its frozen lock
    /// texts (`old` is the baseline the consumer built against; `new` is the candidate).
    /// Refuses, by equation, any dependence the BASELINE does not hold — a reliance on
    /// a law that never existed is a declaration bug, not a compatibility verdict.
    pub fn judge(
        theory: &str,
        deps: &[Dependence],
        old: &str,
        new: &str,
    ) -> Result<DependenceReport, String> {
        let old_laws = Spec::parse_lock(old);
        let new_laws = Spec::parse_lock(new);
        let mut verdicts = Vec::new();
        for dep in deps.iter().filter(|d| d.theory == theory) {
            let Some((old_prose, _)) = old_laws.iter().find(|(_, eq)| *eq == dep.equation) else {
                return Err(format!(
                    "dependence on `{}` refused: the baseline lock for `{theory}` holds \
                     no such law — a reliance on a law that never existed is a \
                     declaration bug, never INTACT",
                    dep.equation
                ));
            };
            let standing = match new_laws.iter().find(|(_, eq)| *eq == dep.equation) {
                None => Standing::Gone,
                Some((new_prose, _)) if new_prose == old_prose => Standing::Intact,
                Some((new_prose, _)) => Standing::Changed {
                    now: new_prose.clone(),
                },
            };
            verdicts.push((dep.equation.clone(), standing));
        }
        Ok(DependenceReport {
            theory: theory.to_string(),
            verdicts,
        })
    }
}

impl Dependence {
    /// Judge a DOWNSTREAM-RELIANCE register against this repo's own committed locks —
    /// the owner-side receiving surface that retires open text as the ask channel.
    /// Each entry is `<theory> | <equation>: <consumer> — <why>` (the `Register`
    /// grammar; the key must carry ` | `, and a missing register is honestly empty).
    /// The judgment is the SELF-JUDGMENT form — the committed lock on both sides — so
    /// INTACT is the only passing standing and the protection is the refusal path: a
    /// re-bless that drops a relied-on law refuses BY NAME — equation, consumer, and
    /// why — before the release ships, instead of breaking a pin downstream a day
    /// later. Returns the held reliances as `(key, justification)` for rendering.
    ///
    /// Honest frame: this judges DECLARED reliances only (it cannot know what a
    /// consumer forgot to declare), and surface (API) reliances are the compiler's to
    /// judge — one line each in `downstream-fixture/tests/reliances.rs`.
    ///
    /// Capability: Effectful — reads the register and the committed spec locks.
    pub fn judge_register(
        register: &spec_lock::Register,
        spec_dir: &Path,
    ) -> Result<Vec<(String, String)>, String> {
        let mut held = Vec::new();
        for (key, justification) in register.entries()? {
            let Some((theory, equation)) = key.split_once(" | ") else {
                return Err(format!(
                    "downstream reliance `{key}` refused: the key grammar is \
                     `<theory> | <equation>` — without the ` | ` there is nothing \
                     to judge"
                ));
            };
            let (theory, equation) = (theory.trim(), equation.trim());
            let slug: String = theory
                .chars()
                .map(|c| if c == ' ' { '-' } else { c })
                .collect();
            let lock_path = spec_dir.join(format!("{slug}.spec"));
            let text = std::fs::read_to_string(&lock_path).map_err(|e| {
                format!(
                    "downstream reliance on `{equation}` refused: no committed lock \
                     for `{theory}` ({e}) — declared by {justification}"
                )
            })?;
            let deps = vec![Dependence::on(theory, equation)];
            Dependence::judge(theory, &deps, &text, &text)
                .map_err(|inner| format!("{inner} — declared by {justification}"))?;
            held.push((key, justification));
        }
        Ok(held)
    }
}

impl DependenceReport {
    /// Did every declared dependence survive intact?
    pub fn is_intact(&self) -> bool {
        self.verdicts
            .iter()
            .all(|(_, s)| matches!(s, Standing::Intact))
    }

    /// The report as deterministic text — the consumer's answer to "did anything I
    /// depend on change?", freezable like every other artifact.
    pub fn render(&self) -> String {
        let intact = self
            .verdicts
            .iter()
            .filter(|(_, s)| matches!(s, Standing::Intact))
            .count();
        let mut out = format!(
            "# dependence report: {} — {intact} of {} declared reliances intact.\n",
            self.theory,
            self.verdicts.len()
        );
        for (equation, standing) in &self.verdicts {
            match standing {
                Standing::Intact => {
                    writeln(&mut out, format!("- intact   {equation}"));
                }
                Standing::Changed { now } => {
                    writeln(
                        &mut out,
                        format!("- CHANGED  {equation}\n      now stated: {now}"),
                    );
                }
                Standing::Gone => {
                    writeln(
                        &mut out,
                        format!("- GONE     {equation}\n      the law is no longer held — the breaking case"),
                    );
                }
            }
        }
        out
    }
}

/// Append a line (tiny helper keeping `render` allocation-light and total).
fn writeln(out: &mut String, line: String) {
    out.push_str(&line);
    out.push('\n');
}

#[cfg(test)]
mod probes {
    use super::*;

    const OLD: &str = "\
# discovered spec: router — a behaviour lock; regenerate via this repo's freeze path and ratify the diff.

- With Or, the grouping of three values doesn't matter.
      ((a or b) or c) = (a or (b or c))
- Or of a value with itself gives that value.
      (a or a) = a
- Or with empty leaves a value unchanged.
      (empty or a) = a

# operators in no law (where the spec is silent): none — every operator participates in a law
";

    // the new lock: idempotence is GONE, identity's prose was re-worded, grouping intact.
    const NEW: &str = "\
# discovered spec: router — a behaviour lock; regenerate via this repo's freeze path and ratify the diff.

- With Or, the grouping of three values doesn't matter.
      ((a or b) or c) = (a or (b or c))
- Empty is the identity for Or.
      (empty or a) = a

# operators in no law (where the spec is silent): none — every operator participates in a law
";

    fn deps() -> Vec<Dependence> {
        vec![
            Dependence::on("router", "((a or b) or c) = (a or (b or c))"),
            Dependence::on("router", "(a or a) = a"),
            Dependence::on("router", "(empty or a) = a"),
        ]
    }

    /// THE SEMVER QUESTION, answered per consumer: between two lock texts, each
    /// declared reliance gets its own verdict — intact, changed (with the new
    /// statement carried), or gone (the breaking case, named). No integer, no claim;
    /// the relation computed.
    #[test]
    fn each_reliance_is_judged_by_name() {
        let report = Dependence::judge("router", &deps(), OLD, NEW).expect("judges");
        assert!(!report.is_intact());
        assert_eq!(
            report.verdicts,
            vec![
                (
                    "((a or b) or c) = (a or (b or c))".to_string(),
                    Standing::Intact
                ),
                ("(a or a) = a".to_string(), Standing::Gone),
                (
                    "(empty or a) = a".to_string(),
                    Standing::Changed {
                        now: "Empty is the identity for Or.".to_string()
                    }
                ),
            ]
        );
        assert_eq!(
            report.render(),
            "# dependence report: router — 1 of 3 declared reliances intact.\n\
             - intact   ((a or b) or c) = (a or (b or c))\n\
             - GONE     (a or a) = a\n\
             \x20     the law is no longer held — the breaking case\n\
             - CHANGED  (empty or a) = a\n\
             \x20     now stated: Empty is the identity for Or.\n"
        );
    }

    /// An unchanged upstream is intact across the board — the common case reads as one
    /// green line, and dependences on OTHER theories are out of scope for this report.
    #[test]
    fn an_unchanged_lock_is_intact_and_scoping_is_by_theory() {
        let mut all = deps();
        all.push(Dependence::on("date calculus", "at(since(a)) = a"));
        let report = Dependence::judge("router", &all, OLD, OLD).expect("judges");
        assert!(report.is_intact());
        assert_eq!(
            report.verdicts.len(),
            3,
            "the calendar reliance is not router's"
        );
        assert!(report
            .render()
            .starts_with("# dependence report: router — 3 of 3 declared reliances intact.\n"));
    }

    /// A dependence the BASELINE never held is a refusal, not a verdict — a typo in a
    /// declared equation must never read as intact.
    #[test]
    fn a_reliance_on_a_law_that_never_existed_refuses() {
        let bogus = vec![Dependence::on("router", "(a or b) = (b or a)")];
        let err = Dependence::judge("router", &bogus, OLD, NEW).unwrap_err();
        assert!(err.contains("refused"), "{err}");
        assert!(err.contains("(a or b) = (b or a)"));
        assert!(err.contains("never existed"));
    }

    /// SELF-JUDGMENT, the theory owner's form: the committed lock on both sides.
    /// Intact is trivially the only passing verdict — the protection is the refusal
    /// path: once a re-bless drops a declared law, the baseline no longer holds it and
    /// the same declaration refuses by equation. Not GONE (a verdict a report could
    /// shrug at) — a refusal, before the ratification diff lands.
    #[test]
    fn self_judgment_makes_declared_reliances_un_droppable() {
        let deps = vec![Dependence::on("router", "(a or a) = a")];
        // today: the committed lock holds the reliance — intact, trivially.
        let held = Dependence::judge("router", &deps, OLD, OLD).expect("judges");
        assert!(held.is_intact());
        // after a re-bless that dropped the law, the SAME declaration refuses:
        let err = Dependence::judge("router", &deps, NEW, NEW).unwrap_err();
        assert!(err.contains("(a or a) = a"), "{err}");
        assert!(err.contains("never existed"), "{err}");
    }

    /// THE RECEIVING SURFACE for downstream asks, live: every reliance declared in
    /// `downstream/reliances.register` HOLDS in this repo's committed locks. A re-bless
    /// that drops a relied-on law fails HERE, naming the equation, the consumer, and
    /// the why — before the release ships. An empty register passes honestly: the
    /// surface exists; consumers author their own lines by PR.
    #[test]
    fn the_downstream_reliance_register_holds() {
        let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let register = spec_lock::Register {
            name: "downstream reliances".to_string(),
            path: manifest.join("downstream/reliances.register"),
        };
        let held = Dependence::judge_register(&register, &manifest.join("spec"))
            .expect("every declared downstream reliance holds in the committed locks");
        // render the held set so a passing run still shows what is being defended.
        for (key, justification) in &held {
            println!("held: {key} ({justification})");
        }
    }

    /// The register judge's refusal paths, on fixtures: a held reliance passes and is
    /// returned; a dropped law refuses naming equation AND consumer; a key without the
    /// ` | ` grammar refuses; a missing register is honestly empty.
    #[test]
    fn the_register_judge_refuses_by_name() {
        let root = std::env::temp_dir().join(format!("depend-register-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("spec")).unwrap();
        std::fs::write(root.join("spec/router.spec"), OLD).unwrap();
        let register = |name: &str, text: &str| {
            let path = root.join(name);
            std::fs::write(&path, text).unwrap();
            spec_lock::Register {
                name: "downstream reliances".to_string(),
                path,
            }
        };

        // a held reliance passes, and the held set carries the justification through:
        let good = register(
            "good.register",
            "router | (a or a) = a: a production consumer — the retry loop stands on idempotence\n",
        );
        let held = Dependence::judge_register(&good, &root.join("spec")).expect("holds");
        assert_eq!(held.len(), 1);
        assert!(held[0].1.contains("retry loop"));

        // a reliance the lock no longer holds refuses, naming equation and consumer:
        let dropped = register(
            "dropped.register",
            "router | (a or b) = (b or a): a production consumer — assumed commutativity\n",
        );
        let err = Dependence::judge_register(&dropped, &root.join("spec")).unwrap_err();
        assert!(err.contains("(a or b) = (b or a)"), "{err}");
        assert!(err.contains("a production consumer"), "{err}");

        // a key without the ` | ` grammar has nothing to judge — refused, not skipped:
        let malformed = register("malformed.register", "router idempotence: someone — why\n");
        let err = Dependence::judge_register(&malformed, &root.join("spec")).unwrap_err();
        assert!(err.contains("` | `"), "{err}");

        // a reliance on a theory with no committed lock refuses by path:
        let unknown = register(
            "unknown.register",
            "ghost theory | (x ⊕ y) = (y ⊕ x): someone — relies on a lock that never froze\n",
        );
        let err = Dependence::judge_register(&unknown, &root.join("spec")).unwrap_err();
        assert!(err.contains("no committed lock"), "{err}");

        // a missing register is the empty register — no declared reliances, honestly none:
        let absent = spec_lock::Register {
            name: "downstream reliances".to_string(),
            path: root.join("absent.register"),
        };
        assert_eq!(
            Dependence::judge_register(&absent, &root.join("spec")).expect("empty"),
            vec![]
        );
    }

    /// The judgment reads REAL frozen locks: every law in this repo's committed router
    /// spec judges intact against itself — the parser and the freeze render agree.
    #[test]
    fn the_committed_locks_judge_intact_against_themselves() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("spec/router.spec");
        let text = std::fs::read_to_string(path).expect("committed lock");
        let laws = Spec::parse_lock(&text);
        assert!(!laws.is_empty(), "the committed lock parses");
        let deps: Vec<Dependence> = laws
            .iter()
            .map(|(_, eq)| Dependence::on("router", eq.clone()))
            .collect();
        let report = Dependence::judge("router", &deps, &text, &text).expect("judges");
        assert!(report.is_intact());
        assert_eq!(report.verdicts.len(), laws.len());
    }
}
