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
//!
//! The thirteenth ask collapsed the two forms for the pinned consumer: the crate CARRIES
//! its certification data ([`Locks`] — every theory lock, its mutation companion, and the
//! shape catalog, `include_str!`-embedded), so a consumer's committed
//! `upstream-reliances.register` is judged in its own suite against the locks of the exact
//! version it is pinned to ([`Dependence::judge_embedded`]) — declaration lives with its
//! owner, judgment re-runs on every pin bump, nothing crosses repos. A bump to a version
//! that dropped a relied-on law refuses BY NAME, carrying the consumer's why, before a
//! bare compile error explains nothing.

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

#[crate::mutate("dependence")]
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

#[crate::mutate]
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
        judge_reliances(register, |theory| {
            // the theory names a lock FILE, so it must stay inside the spec directory:
            // a name carrying path syntax is refused, never resolved — the register is
            // reviewed data, but the judge does not lean on that.
            if theory.contains('/') || theory.contains('\\') || theory.contains("..") {
                return Err(format!(
                    "`{theory}` is not a theory name — path syntax cannot name a lock"
                ));
            }
            let slug: String = theory
                .chars()
                .map(|c| if c == ' ' { '-' } else { c })
                .collect();
            let lock_path = spec_dir.join(format!("{slug}.spec"));
            std::fs::read_to_string(&lock_path)
                .map_err(|e| format!("no committed lock for `{theory}` ({e})"))
        })
    }

    /// Judge a CONSUMER-SIDE reliance register against the locks this crate carries
    /// ([`Locks`]) — the thirteenth ask's collapse: the cross-repo form becomes the
    /// self-judgment form because the lock travels with the pin. A consumer commits
    /// `upstream-reliances.register` (the same `<theory> | <equation>: <consumer> — <why>`
    /// grammar) and calls this in its own suite; a pin bump re-runs the judgment against
    /// the new version's embedded locks with zero ceremony, and a bump that dropped a
    /// relied-on law refuses by equation and why — before a bare compile error explains
    /// nothing. Returns the held reliances as `(key, justification)` for rendering.
    ///
    /// Capability: Effectful — reads the register (the locks themselves are compiled in).
    pub fn judge_embedded(register: &spec_lock::Register) -> Result<Vec<(String, String)>, String> {
        judge_reliances(register, |theory| {
            Locks::text(theory).map(str::to_string).ok_or_else(|| {
                format!(
                    "this version embeds no lock named `{theory}` — the certification \
                     data a pin carries is its theory locks, their mutation companions, \
                     and the shape catalog"
                )
            })
        })
    }
}

/// The shared judgment core behind [`Dependence::judge_register`] (locks resolved from a
/// spec directory) and [`Dependence::judge_embedded`] (locks resolved from [`Locks`]) —
/// one grammar, one refusal envelope, two resolvers. `lock_text` answers a theory name
/// with its lock text or the refusal detail; every refusal carries the consumer's
/// declared why, so the message names who is broken and what they said they stand on.
#[crate::mutate]
fn judge_reliances(
    register: &spec_lock::Register,
    lock_text: impl Fn(&str) -> Result<String, String>,
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
        let text = lock_text(theory).map_err(|detail| {
            format!(
                "downstream reliance on `{equation}` refused: {detail} — declared by \
                 {justification}"
            )
        })?;
        let deps = vec![Dependence::on(theory, equation)];
        Dependence::judge(theory, &deps, &text, &text)
            .map_err(|inner| format!("{inner} — declared by {justification}"))?;
        held.push((key, justification));
    }
    Ok(held)
}

/// The certification data this crate CARRIES: every committed theory lock, its
/// algebra-mutation companion, and the shape catalog, `include_str!`-embedded byte for
/// byte at compile time and keyed by lock stem. "The version is the certification" as an
/// API — a consumer pinned to a release holds exactly that release's locks, no filesystem
/// archaeology, no release-notes parsing; and because [`Spec::parse_lock`] ships in the
/// same artifact, the lock and the parser that reads it can never skew.
///
/// The roster is DELIBERATE, and census-gated in both directions (see this module's
/// probes): the behaviour locks and their mutation companions ship — the laws are the
/// contract, the ratified survivors are its fine print (the named degrees of freedom a
/// swap could hide in) — and `shapes.spec` rides along as the law-language the equations
/// are written in. Repo-meta locks (gates, tiers, perimeter, substrate, schemata, the
/// censuses) and every hand-authored register stay home: they are this repository's
/// structure and decisions, not the certification a pin carries, and embedding one would
/// invite reliances on interior facts. Consumers judge reliances against this data via
/// [`Dependence::judge_embedded`].
///
/// Honest frame: this is certification DATA, not re-derivation — holding a lock's text is
/// holding what discovery earned at release time, not the ability to re-run it (that
/// takes the engine and the theory, which the crate also ships, but separately).
pub struct Locks;

/// The embedded roster, sorted by key. Keys are lock-file stems: a theory's lock under
/// its slug (`date-calculus`), its mutation companion under `<slug>.mutation`, the
/// catalog under `shapes`.
static EMBEDDED: &[(&str, &str)] = &[
    ("bridged-bool", include_str!("../../spec/bridged-bool.spec")),
    (
        "bridged-bool.mutation",
        include_str!("../../spec/bridged-bool.mutation.spec"),
    ),
    (
        "date-calculus",
        include_str!("../../spec/date-calculus.spec"),
    ),
    (
        "date-calculus.mutation",
        include_str!("../../spec/date-calculus.mutation.spec"),
    ),
    ("doc-flow", include_str!("../../spec/doc-flow.spec")),
    (
        "doc-flow.mutation",
        include_str!("../../spec/doc-flow.mutation.spec"),
    ),
    ("fabric", include_str!("../../spec/fabric.spec")),
    (
        "fabric.mutation",
        include_str!("../../spec/fabric.mutation.spec"),
    ),
    (
        "interpreter-arithmetic",
        include_str!("../../spec/interpreter-arithmetic.spec"),
    ),
    (
        "interpreter-arithmetic.mutation",
        include_str!("../../spec/interpreter-arithmetic.mutation.spec"),
    ),
    ("router", include_str!("../../spec/router.spec")),
    (
        "router.mutation",
        include_str!("../../spec/router.mutation.spec"),
    ),
    ("shapes", include_str!("../../spec/shapes.spec")),
    (
        "store-protocol",
        include_str!("../../spec/store-protocol.spec"),
    ),
    (
        "store-protocol.mutation",
        include_str!("../../spec/store-protocol.mutation.spec"),
    ),
    ("ttl-store", include_str!("../../spec/ttl-store.spec")),
    (
        "ttl-store.mutation",
        include_str!("../../spec/ttl-store.mutation.spec"),
    ),
];

#[crate::mutate("locks")]
impl Locks {
    /// Every embedded lock, `(key, text)`, sorted by key — the whole certification
    /// payload, for callers that enumerate rather than look up.
    pub fn all() -> &'static [(&'static str, &'static str)] {
        EMBEDDED
    }

    /// The embedded lock text for `name` — a lock stem (`router`, `ttl-store.mutation`)
    /// or a theory's display name (`date calculus`; spaces slug to hyphens, the same
    /// convention the lock filenames use). `None` is an honest miss, never a fallback.
    pub fn text(name: &str) -> Option<&'static str> {
        let slug: String = name
            .chars()
            .map(|c| if c == ' ' { '-' } else { c })
            .collect();
        EMBEDDED
            .iter()
            .find(|(key, _)| *key == slug)
            .map(|(_, text)| *text)
    }
}

#[crate::mutate("dependence_report")]
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
#[crate::mutate]
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

        // a theory name carrying path syntax is refused, never resolved — the lock
        // lookup must not be steerable outside the spec directory. Each syntax REFUSES
        // ALONE (slash, backslash, dot-dot): the guard is a disjunction, and a probe
        // that only ever combines them would let `||` quietly become `&&`.
        for bad in [
            "../outside | (a or a) = a: someone — a path is not a theory\n",
            "up..sneak | (a or a) = a: someone — dot-dot alone must refuse\n",
            "sub/theory | (a or a) = a: someone — a slash alone must refuse\n",
            "sub\\theory | (a or a) = a: someone — a backslash alone must refuse\n",
        ] {
            let traversal = register("traversal.register", bad);
            let err = Dependence::judge_register(&traversal, &root.join("spec")).unwrap_err();
            assert!(err.contains("path syntax cannot name a lock"), "{err}");
        }

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

    /// THE ROSTER, complete for the registry: every theory the spec registry declares
    /// ships its lock and its mutation companion, the mounted bridge ships both of its
    /// own, and the shape catalog — the law-language every embedded equation is written
    /// in — rides along. A theory added to the registry without its `include_str!` lines
    /// fails HERE, so the crate cannot publish a version whose certification data is
    /// missing a theory.
    #[test]
    fn the_embedded_locks_cover_the_registry() {
        for spec in crate::discover::all_specs() {
            assert!(
                Locks::text(spec.theory).is_some(),
                "`{}` is in the registry but its lock is not embedded — add its \
                 include_str! line to EMBEDDED",
                spec.theory
            );
            assert!(
                Locks::text(&format!("{}.mutation", spec.theory)).is_some(),
                "`{}` ships without its mutation companion — the degrees of freedom \
                 are the guarantee's fine print and travel with it",
                spec.theory
            );
        }
        // the bridged theory is mounted, not registry-declared — its locks ship too:
        assert!(Locks::text("bridged-bool").is_some());
        assert!(Locks::text("bridged-bool.mutation").is_some());
        assert!(Locks::text("shapes").is_some());
    }

    /// THE SHIP/HOME CENSUS over the committed spec directory, judged both ways. What
    /// ships is DERIVED from the artifacts themselves (a behaviour lock's or mutation
    /// lock's own header, plus the catalog); registers stay home as a CLASS
    /// (hand-authored decisions are not certification data); everything else must carry
    /// a reasoned HOME line below — an unclassified artifact refuses, and a HOME line
    /// whose artifact is gone is stale, a lie to delete. So the embedded roster and the
    /// spec directory can only move together, ratified.
    #[test]
    fn every_spec_artifact_ships_or_stays_home_by_decision() {
        const HOME: &[(&str, &str)] = &[
            (
                "boundary-spec.shape.spec",
                "this repo's own derived shape — a consumer freezes its own via ShapeReport::lock_in",
            ),
            (
                "boundary-spec.system.spec",
                "this repo's own seam graph — structure, not certification data",
            ),
            (
                "bridged-bool.export",
                "the prover's input emission — data the bridge consumes, not a lock it certifies",
            ),
            (
                "bridged-bool.obligations.spec",
                "the proof-obligation triage — the prover loop's worklist, not a consumer contract",
            ),
            (
                "exemplar.infra.spec",
                "the first infra consumer's shape, names washed — an exemplar, not this crate's conduct",
            ),
            ("gates.spec", "repo-meta: this repo's pipeline declaration"),
            (
                "perimeter.ruleset.json",
                "repo-meta: the platform render of the perimeter floor",
            ),
            ("perimeter.spec", "repo-meta: this repo's settings floor"),
            ("probes.spec", "repo-meta: this repo's probe roster"),
            (
                "qualify-reasons.spec",
                "repo-meta: this repo's domain-modelling worklist (the qualify census's complement)",
            ),
            ("qualify.spec", "repo-meta: this repo's surface census"),
            (
                "schemata.spec",
                "repo-meta: this repo's compiled-mutant census",
            ),
            (
                "store-model.world.spec",
                "this repo's ratified beliefs about ITS demonstration dependency",
            ),
            ("substrate.spec", "repo-meta: this repo's git meaning"),
            ("tiers.spec", "repo-meta: this repo's tier partition"),
        ];
        let spec_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("spec");
        let mut ships: Vec<String> = Vec::new();
        for entry in std::fs::read_dir(&spec_dir).expect("spec directory") {
            let name = entry
                .expect("dir entry")
                .file_name()
                .into_string()
                .expect("utf-8 file name");
            if name.ends_with(".register") {
                continue; // registers stay home as a class: decisions, never data.
            }
            let text = std::fs::read_to_string(spec_dir.join(&name)).expect("spec artifact");
            let first = text.lines().next().unwrap_or_default();
            if name == "shapes.spec"
                || first.starts_with("# discovered spec:")
                || first.starts_with("# algebra mutation:")
            {
                let stem = name
                    .strip_suffix(".mutation.spec")
                    .map(|s| format!("{s}.mutation"))
                    .unwrap_or_else(|| {
                        name.strip_suffix(".spec")
                            .expect("a .spec file")
                            .to_string()
                    });
                ships.push(stem);
            } else {
                assert!(
                    HOME.iter().any(|(home, _)| *home == name),
                    "`spec/{name}` is neither certification data (embed it in Locks) nor \
                     a reasoned HOME line — classify it"
                );
            }
        }
        for (home, _) in HOME {
            assert!(
                spec_dir.join(home).exists(),
                "stale HOME line `{home}` — the artifact is gone; a stale exception is a lie"
            );
        }
        ships.sort();
        let embedded: Vec<&str> = Locks::all().iter().map(|(key, _)| *key).collect();
        assert_eq!(
            embedded, ships,
            "the embedded roster and the committed certification artifacts must move together"
        );
    }

    /// The embedded texts ARE the committed bytes. Locally this is `include_str!`
    /// tautology; in the PACKAGED crate it proves `spec/` shipped coherently — the
    /// files a consumer's tooling might read agree with the constants their pin
    /// carries.
    #[test]
    fn the_embedded_locks_are_the_committed_bytes() {
        let spec_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("spec");
        for (key, text) in Locks::all() {
            let file = key
                .strip_suffix(".mutation")
                .map(|s| format!("{s}.mutation.spec"))
                .unwrap_or_else(|| format!("{key}.spec"));
            let committed = std::fs::read_to_string(spec_dir.join(&file)).expect("committed lock");
            assert_eq!(*text, committed, "`{key}` diverges from `spec/{file}`");
        }
    }

    /// Lookup speaks the theory vocabulary: a display name with spaces resolves to the
    /// same lock as its slug, and a miss is `None` — never a fallback.
    #[test]
    fn lock_lookup_speaks_the_theory_vocabulary() {
        assert_eq!(Locks::text("date calculus"), Locks::text("date-calculus"));
        assert!(Locks::text("date calculus")
            .expect("embedded")
            .starts_with("# discovered spec: date calculus"));
        assert!(Locks::text("interpreter arithmetic.mutation").is_some());
        assert_eq!(Locks::text("ghost theory"), None);
    }

    /// THE COLLAPSE, live: a consumer-side register judged against the locks the pin
    /// carries — a held reliance passes with its why; a law this version does not hold
    /// refuses naming equation and consumer; a theory the roster does not carry refuses
    /// naming what a pin does carry. No spec directory anywhere: the certification data
    /// is the crate's own.
    #[test]
    fn the_embedded_judgment_collapses_the_cross_repo_form() {
        let root = std::env::temp_dir().join(format!("depend-embedded-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let register = |name: &str, text: &str| {
            let path = root.join(name);
            std::fs::write(&path, text).unwrap();
            spec_lock::Register {
                name: "upstream reliances".to_string(),
                path,
            }
        };

        // a real law, read from the embedded lock itself — the consumer's authoring flow:
        let (_, equation) = Spec::parse_lock(Locks::text("router").expect("embedded"))
            .into_iter()
            .next()
            .expect("the router lock holds laws");
        let good = register(
            "good.register",
            &format!("router | {equation}: a pinned consumer — its retry loop stands on this\n"),
        );
        let held = Dependence::judge_embedded(&good).expect("the pinned version holds it");
        assert_eq!(held.len(), 1);
        assert!(held[0].1.contains("retry loop"));

        // a law this version does not hold refuses, naming equation and consumer:
        let dropped = register(
            "dropped.register",
            "router | (a ⊗ b) = (b ⊗ a): a pinned consumer — assumed a law that never froze\n",
        );
        let err = Dependence::judge_embedded(&dropped).unwrap_err();
        assert!(err.contains("(a ⊗ b) = (b ⊗ a)"), "{err}");
        assert!(err.contains("a pinned consumer"), "{err}");

        // a theory outside the roster refuses naming what a pin does carry:
        let unknown = register(
            "unknown.register",
            "ghost theory | (x ⊕ y) = (y ⊕ x): someone — relies on a lock no pin carries\n",
        );
        let err = Dependence::judge_embedded(&unknown).unwrap_err();
        assert!(err.contains("embeds no lock"), "{err}");
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
