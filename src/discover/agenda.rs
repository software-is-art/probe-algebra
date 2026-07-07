//!
//! agenda — the review router: the reading list DERIVED from which locks moved.
//!
//! "Review the PR" is the last implied service in the workflow — an unspecified ask to
//! read everything with unspecified judgment. But this repo's diffs classify
//! themselves: every artifact class has a lock, every lock has a specific ratification
//! question, and interior code has none (the gates hold it). So the review agenda is a
//! derivation, not a ritual: given the changed paths, emit exactly the ratifications
//! this change requires and name everything else machinery-verified. Review attention
//! scales with blast radius because the router computes the blast radius.
//!
//! Replacing implied service from a platform with specification, applied to the
//! reviewer: what was "look at all of it, somehow" becomes a checklist with one
//! question per moved lock.
//!
//! Honest frame: the router classifies WHERE judgment is due, never exercises it — the
//! ratification stays human. And an interior code change is "machinery-verified" only
//! as far as the gates reach; the router's floor is the pipeline's floor.

use std::fmt::Write as _;
use std::path::Path;

/// The ratification a moved artifact calls for — one question per lock class.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Ratification {
    /// A theory lock moved: read the law diff — did the boundary's meaning change on
    /// purpose?
    Laws { theory: String },
    /// A mutation spec moved: a degree of freedom appeared or closed — ratify the
    /// survivor set.
    Freedoms { theory: String },
    /// The shape lock moved: the derived module boundaries changed — is the new
    /// placement the intended architecture?
    Boundary,
    /// The system lock moved: a seam was drawn, dropped, or flipped its verdict.
    Seams,
    /// A qualify census moved: the public surface changed — admit the new operators.
    Surface,
    /// The tier lock moved: a file's DERIVED tier changed — its structure carried it
    /// through a door or out of reach — or a kernel decision was made.
    Partition,
    /// The pipeline moved (gates.spec / ci.yml): the promises or their execution
    /// changed — re-read what CI now claims.
    Pipeline,
    /// An exception register moved: a justification was added, changed, or resolved.
    Exceptions { register: String },
    /// The world lock moved: the model's beliefs about a dependency were re-ratified.
    World,
    /// A shapes.spec move: the law LANGUAGE itself grew or changed.
    Vocabulary,
    /// The perimeter moved: the declared repository-settings floor (branch rules,
    /// merge methods, vulnerability reporting) or its apply-able ruleset changed.
    Perimeter,
    /// A CONSUMER-registered lock class moved: the class and its ratification question
    /// are the consumer's own data (see [`Agenda::of_with`]) — the routing machinery is
    /// generic; the class table is not upstream's to own.
    Custom { class: String, question: String },
}

impl Ratification {
    /// The one question this ratification asks the reviewer.
    pub fn question(&self) -> String {
        match self {
            Ratification::Laws { theory } => {
                format!("`{theory}`: the discovered laws moved — is the new meaning intended?")
            }
            Ratification::Freedoms { theory } => format!(
                "`{theory}`: the mutation survivor set moved — ratify the degrees of freedom."
            ),
            Ratification::Boundary => {
                "the derived module boundaries moved — is the new placement the intended \
                 architecture?"
                    .to_string()
            }
            Ratification::Seams => {
                "the seam graph moved — a seam drawn, dropped, or verdict flipped.".to_string()
            }
            Ratification::Surface => {
                "the public surface census moved — admit the new operators.".to_string()
            }
            Ratification::Partition => {
                "the tier partition moved — a file's derived tier changed; is the new \
                 placement intended?"
                    .to_string()
            }
            Ratification::Pipeline => {
                "the pipeline moved — re-read what CI now promises and executes.".to_string()
            }
            Ratification::Exceptions { register } => format!(
                "`{register}`: the exception register moved — justifications are the review."
            ),
            Ratification::World => {
                "the world lock moved — the model's beliefs about a dependency changed.".to_string()
            }
            Ratification::Vocabulary => {
                "the shape catalog moved — the law language itself changed.".to_string()
            }
            Ratification::Perimeter => {
                "the perimeter moved — the declared settings floor changed; re-apply \
                 spec/perimeter.ruleset.json and re-check the live settings against it."
                    .to_string()
            }
            Ratification::Custom { class, question } => format!("`{class}`: {question}"),
        }
    }
}

/// The routed review: ratifications due, prose to read, and the machinery-verified
/// remainder.
#[derive(Debug)]
pub struct Agenda {
    /// The ratifications, in path order, deduplicated.
    pub ratifications: Vec<Ratification>,
    /// Documentation and prose changes — read for sense, no lock question.
    pub prose: Vec<String>,
    /// Everything else: code and config the gates hold — listed, not read.
    pub machinery: Vec<String>,
}

impl Agenda {
    /// Route a changed-path list (repo-relative, e.g. `git diff --name-only`) into the
    /// review agenda. Path classification is by the repo's own artifact conventions;
    /// a spec-like path with an unknown suffix is REFUSED by name — a new lock class
    /// must be taught to the router, never silently filed under machinery.
    pub fn of<S: AsRef<str>>(paths: impl IntoIterator<Item = S>) -> Result<Agenda, String> {
        Agenda::of_with(paths, &[])
    }

    /// Route with CONSUMER-TAUGHT lock classes: each entry is `(filename suffix, the
    /// ratification question its move asks)`. The built-in table knows only this
    /// repo's artifact conventions — the routing machinery is generic, so a consumer's
    /// spec directory registers its classes as data (house form: a committed
    /// `spec/agenda.register`, one `suffix: question` line per class, parsed with
    /// `spec_lock::Register` — `examples/review_agenda.rs` wires it). Taught classes
    /// match FIRST: the register is the consumer's own vocabulary, so it outranks
    /// every built-in, including the `.spec` catch-all that would otherwise misfile a
    /// consumer artifact as this repo's law class.
    pub fn of_with<S: AsRef<str>>(
        paths: impl IntoIterator<Item = S>,
        classes: &[(String, String)],
    ) -> Result<Agenda, String> {
        let mut agenda = Agenda {
            ratifications: Vec::new(),
            prose: Vec::new(),
            machinery: Vec::new(),
        };
        for path in paths {
            let path = path.as_ref().to_string();
            let class = classify(&path, classes)?;
            match class {
                Class::Ratify(r) => {
                    if !agenda.ratifications.contains(&r) {
                        agenda.ratifications.push(r);
                    }
                }
                Class::Prose => agenda.prose.push(path),
                Class::Machinery => agenda.machinery.push(path),
            }
        }
        Ok(agenda)
    }

    /// The agenda as readable text — the reviewer's derived reading list.
    pub fn render(&self) -> String {
        let mut out = format!(
            "# review agenda — {} ratification(s) required; {} file(s) machinery-verified.\n",
            self.ratifications.len(),
            self.machinery.len()
        );
        if self.ratifications.is_empty() {
            out.push_str("\nno lock moved: nothing requires ratification.\n");
        } else {
            out.push_str("\nratify:\n");
            for r in &self.ratifications {
                let _ = writeln!(out, "- {}", r.question());
            }
        }
        if !self.prose.is_empty() {
            out.push_str("\nread for sense (prose, no lock question):\n");
            for p in &self.prose {
                let _ = writeln!(out, "- {p}");
            }
        }
        if !self.machinery.is_empty() {
            out.push_str("\nmachinery-verified (the gates hold these — listed, not read):\n");
            for p in &self.machinery {
                let _ = writeln!(out, "- {p}");
            }
        }
        out
    }
}

/// Which of the guard's voices EXIST for the tree being edited. The guard's contract —
/// pre-fire existing refusals, never invent judgments — makes each voice conditional on
/// evidence of the downstream refusal it pre-fires. The generated-lock voice has no
/// switch here: its refusal is a spec-lock freshness gate, which any adopter of the
/// lock mechanics has by construction. The rats-nest voice's refusal lives in
/// `boundary-enforce`'s build shim, which a tree may simply not run — a crate where
/// module-level `pub fn` is the designed shape must get silence, not a warning per
/// public function about a refusal that does not exist for it.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct GuardVoices {
    /// The edited file is REGISTERED kernel: the shim exempts it from every structural
    /// rule, so the guard is silent on it too. Never read from the file — a marker in
    /// the source grants nothing to the build, so it grants nothing here.
    pub kernel_exempt: bool,
    /// The no-rats-nest refusal exists downstream: the tree runs the enforcement shim,
    /// so a loose `pub fn` really will refuse the next build.
    pub rats_nest: bool,
}

impl GuardVoices {
    /// Derive the voices from the tree itself — a declaration would drift; the
    /// evidence cannot. Kernel-exemption is a register lookup (`spec/kernel.register`,
    /// the same fact the build shim consumes); the rats-nest voice is on exactly when
    /// the tree's `build.rs` names `boundary_enforce`. Both fail open (missing file:
    /// voice off / not exempt) — the guard is advisory, and the build gate owns the
    /// refusals, including a refused register.
    ///
    /// Capability: Effectful — reads `build.rs` and the kernel register under
    /// `manifest`.
    pub fn for_edit(manifest: &Path, edited: &str) -> GuardVoices {
        let kernel_exempt = spec_lock::Register {
            name: "kernel".to_string(),
            path: manifest.join("spec/kernel.register"),
        }
        .entries()
        .unwrap_or_default()
        .iter()
        .any(|(key, _)| edited == key || edited.ends_with(&format!("/{key}")));
        let rats_nest = std::fs::read_to_string(manifest.join("build.rs"))
            .is_ok_and(|source| source.contains("boundary_enforce"));
        GuardVoices {
            kernel_exempt,
            rats_nest,
        }
    }
}

impl Agenda {
    /// The EDIT-TIME guard — the hook's second voice: move EXISTING refusals earlier,
    /// never add judgments. Two pre-fires: the one rule (an edit landing on a generated
    /// lock artifact warns now, instead of the freshness gate refusing a build later),
    /// and the no-rats-nest shim (a module-level `pub fn` warns now, instead of
    /// `build.rs` refusing at compile time). Registers and exports are exempt from the
    /// first — they are hand-authored inputs, not derived artifacts. `None` when the
    /// edit is unremarkable, because silence must stay free.
    ///
    /// `voices` is the CALLER's fact — derive it with [`GuardVoices::for_edit`]: each
    /// voice speaks only where its downstream refusal exists, so the contract holds by
    /// construction rather than by consumer-side filtering. `classes` is the same
    /// consumer-taught table [`Agenda::of_with`] routes with, so an edit landing on a
    /// registered consumer lock gets the never-hand-edit line, not "teach the router".
    pub fn edit_guard(
        path: &str,
        source: &str,
        voices: &GuardVoices,
        classes: &[(String, String)],
    ) -> Option<String> {
        let mut lines = Vec::new();
        if !path.ends_with(".export") && !path.ends_with(".register") {
            match classify(path, classes) {
                Ok(Class::Ratify(_)) => lines.push(format!(
                    "guard: `{path}` is a generated lock artifact — never hand-edit; \
                     regenerate via its freeze path and ratify the diff"
                )),
                Ok(_) => {}
                Err(_) => lines.push(format!(
                    "guard: `{path}` looks like a lock artifact of no known class — if \
                     it is one, teach the review router before it can be misfiled"
                )),
            }
        }
        if path.ends_with(".rs") && voices.rats_nest && !voices.kernel_exempt {
            for name in source.lines().filter_map(|l| {
                l.strip_prefix("pub fn ")
                    .and_then(|rest| rest.split('(').next())
            }) {
                lines.push(format!(
                    "guard: `pub fn {name}` is a loose public function — the \
                     enforcement shim will refuse the build; hang it off a typestate"
                ));
            }
        }
        if lines.is_empty() {
            None
        } else {
            Some(lines.join("\n"))
        }
    }
}

enum Class {
    Ratify(Ratification),
    Prose,
    Machinery,
}

/// One path to its review class. The spec-directory refusal is the router's own
/// completeness gate: an unrecognized `spec/` artifact means a lock class the router
/// was never taught, and misfiling it as machinery would silently drop a ratification.
fn classify(path: &str, classes: &[(String, String)]) -> Result<Class, String> {
    let file = path.rsplit('/').next().unwrap_or(path);
    // consumer-taught classes match FIRST — the register is the consumer's own
    // vocabulary, so it outranks every built-in (a `.spec`-suffixed consumer class
    // would otherwise be swallowed by the law-lock catch-all below).
    for (suffix, question) in classes {
        if file.ends_with(suffix.as_str()) {
            return Ok(Class::Ratify(Ratification::Custom {
                class: suffix.clone(),
                question: question.clone(),
            }));
        }
    }
    let theory = |suffix: &str| file.trim_end_matches(suffix).replace('-', " ").to_string();
    // a register routes as its exception class WHEREVER it lives — `spec/kernel.register`
    // included: it is a hand-authored ratification input whose justifications ARE the
    // review, so it must not fall into the spec-directory unknown-class refusal below.
    if file.ends_with(".register") {
        return Ok(Class::Ratify(Ratification::Exceptions {
            register: path.to_string(),
        }));
    }
    if path.contains("spec/") || path.starts_with("spec/") {
        return Ok(Class::Ratify(if file == "qualify.spec" {
            Ratification::Surface
        } else if file == "gates.spec" {
            Ratification::Pipeline
        } else if file == "tiers.spec" {
            Ratification::Partition
        } else if file == "perimeter.spec" || file == "perimeter.ruleset.json" {
            Ratification::Perimeter
        } else if file == "shapes.spec" {
            Ratification::Vocabulary
        } else if file.ends_with(".mutation.spec") {
            Ratification::Freedoms {
                theory: theory(".mutation.spec"),
            }
        } else if file.ends_with(".shape.spec") {
            Ratification::Boundary
        } else if file.ends_with(".system.spec") {
            Ratification::Seams
        } else if file.ends_with(".world.spec") {
            Ratification::World
        } else if file.ends_with(".census") {
            Ratification::Surface
        } else if file.ends_with(".obligations.spec") {
            Ratification::Laws {
                theory: theory(".obligations.spec"),
            }
        } else if file.ends_with(".export") {
            Ratification::Laws {
                theory: theory(".export"),
            }
        } else if file.ends_with(".spec") {
            Ratification::Laws {
                theory: theory(".spec"),
            }
        } else {
            return Err(format!(
                "`{path}` is a spec-directory artifact of no known lock class — teach \
                 the router its ratification question; misfiling it as machinery would \
                 silently drop a review"
            ));
        }));
    }
    if path.ends_with("ci.yml") {
        return Ok(Class::Ratify(Ratification::Pipeline));
    }
    if path.ends_with(".md") {
        return Ok(Class::Prose);
    }
    Ok(Class::Machinery)
}

#[cfg(test)]
mod probes {
    use super::*;

    /// THE ROUTED REVIEW, end to end: a realistic PR's paths become exactly the
    /// ratification questions its locks pose — one per moved lock class, deduplicated —
    /// with docs routed to prose and interior code to the machinery-verified list.
    #[test]
    fn a_diff_routes_to_its_ratifications() {
        let agenda = Agenda::of([
            "src/discover/router.rs",
            "spec/router.spec",
            "spec/router.mutation.spec",
            "spec/boundary-spec.shape.spec",
            "spec/qualify.spec",
            "spec/tiers.spec",
            ".github/workflows/ci.yml",
            "spec/gates.spec",
            "spec/kernel.register",
            "lean/bites.register",
            "docs/roadmap.md",
            "tests/bridge.rs",
        ])
        .expect("all classes known");
        assert_eq!(
            agenda.ratifications,
            vec![
                Ratification::Laws {
                    theory: "router".to_string()
                },
                Ratification::Freedoms {
                    theory: "router".to_string()
                },
                Ratification::Boundary,
                Ratification::Surface,
                Ratification::Partition,
                Ratification::Pipeline,
                // a register routes as its exception class even INSIDE spec/ — the
                // kernel register is a hand-authored input, never an unknown lock:
                Ratification::Exceptions {
                    register: "spec/kernel.register".to_string()
                },
                Ratification::Exceptions {
                    register: "lean/bites.register".to_string()
                },
            ],
            "one question per moved lock class, ci.yml and gates.spec deduplicated"
        );
        assert_eq!(agenda.prose, vec!["docs/roadmap.md"]);
        assert_eq!(
            agenda.machinery,
            vec!["src/discover/router.rs", "tests/bridge.rs"]
        );
        let text = agenda.render();
        assert!(text.starts_with(
            "# review agenda — 8 ratification(s) required; 2 file(s) machinery-verified.\n"
        ));
        assert!(text.contains("`router`: the discovered laws moved"));
        assert!(text.contains("read for sense (prose, no lock question):\n- docs/roadmap.md\n"));
        assert!(text.contains("machinery-verified (the gates hold these — listed, not read):"));
    }

    /// An interior-only diff requires NO ratification — the agenda says so instead of
    /// implying "read it all", because that sentence is the whole point of the router.
    #[test]
    fn an_interior_diff_requires_no_ratification() {
        let agenda = Agenda::of(["src/discover/engine.rs", "fire-drill/src/lib.rs"]).unwrap();
        assert!(agenda.ratifications.is_empty());
        let text = agenda.render();
        assert!(text.contains("no lock moved: nothing requires ratification."));
        assert!(
            !text.contains("read for sense"),
            "an empty prose list renders no prose section"
        );
    }

    /// An unknown spec-directory artifact REFUSES — a new lock class must be taught to
    /// the router, because misfiled-as-machinery is a silently dropped review.
    #[test]
    fn an_unknown_lock_class_refuses() {
        let err = Agenda::of(["spec/router.frobnicate"]).unwrap_err();
        assert!(err.contains("no known lock class"), "{err}");
        assert!(err.contains("spec/router.frobnicate"));
    }

    /// THE CLASS TABLE IS CONSUMER-EXTENSIBLE DATA: a taught suffix routes to its own
    /// question, before every built-in — so a consumer artifact that would otherwise
    /// refuse (unknown class) or misfile (the `.spec` catch-all reading it as a law
    /// lock) gets the consumer's one question instead.
    #[test]
    fn consumer_taught_classes_route_before_the_builtins() {
        let classes = vec![
            (
                "deploy-freshness.lock".to_string(),
                "the deploy-freshness laws moved — do the runbooks still stand?".to_string(),
            ),
            (
                "surface-audit.spec".to_string(),
                "the audit census moved — admit the new commands.".to_string(),
            ),
        ];
        let agenda = Agenda::of_with(
            [
                "spec/deploy-freshness.lock",
                "spec/surface-audit.spec",
                "src/main.rs",
            ],
            &classes,
        )
        .expect("taught classes route");
        assert_eq!(
            agenda.ratifications,
            vec![
                Ratification::Custom {
                    class: "deploy-freshness.lock".to_string(),
                    question: "the deploy-freshness laws moved — do the runbooks still stand?"
                        .to_string()
                },
                Ratification::Custom {
                    class: "surface-audit.spec".to_string(),
                    question: "the audit census moved — admit the new commands.".to_string()
                },
            ]
        );
        assert_eq!(agenda.machinery, vec!["src/main.rs"]);
        // the question renders exactly as the consumer wrote it, named by its class:
        let text = agenda.render();
        assert!(
            text.contains("- `deploy-freshness.lock`: the deploy-freshness laws moved"),
            "{text}"
        );
        // and WITHOUT the register the same paths refuse / misfile — the register is
        // what changes the routing, nothing else:
        assert!(Agenda::of(["spec/deploy-freshness.lock"]).is_err());
        assert_eq!(
            Agenda::of(["spec/surface-audit.spec"])
                .expect("catch-all")
                .ratifications,
            vec![Ratification::Laws {
                theory: "surface audit".to_string()
            }]
        );
        // a taught class reaches the guard too: the edit warns never-hand-edit,
        // instead of "teach the router" about a class that was already taught.
        let voices = GuardVoices {
            kernel_exempt: false,
            rats_nest: true,
        };
        let g = Agenda::edit_guard("spec/deploy-freshness.lock", "", &voices, &classes)
            .expect("a taught lock warns");
        assert!(g.contains("never hand-edit"), "{g}");
    }

    /// THE EDIT-TIME GUARD pre-fires exactly the two refusals that already exist
    /// downstream, and stays silent otherwise. A generated lock warns; a hand-authored
    /// register or export does not; a module-level `pub fn` warns (indented assoc fns
    /// do not); prose and interior code are silence. Absolute paths classify the same
    /// as repo-relative — the Edit tool hands the hook absolute paths.
    #[test]
    fn the_edit_guard_prefires_only_existing_refusals() {
        // the enforced tree's voices: the shim runs, the file is not kernel.
        let v = GuardVoices {
            kernel_exempt: false,
            rats_nest: true,
        };
        let g = Agenda::edit_guard("spec/router.spec", "", &v, &[]).expect("a lock warns");
        assert!(g.contains("never hand-edit"), "{g}");
        assert!(Agenda::edit_guard("/home/u/repo/spec/router.spec", "", &v, &[]).is_some());
        assert_eq!(Agenda::edit_guard("lean/bites.register", "", &v, &[]), None);
        assert_eq!(
            Agenda::edit_guard("spec/bridged-bool.export", "", &v, &[]),
            None
        );
        assert_eq!(Agenda::edit_guard("docs/roadmap.md", "", &v, &[]), None);

        let loose = "pub fn pipeline() -> Pipeline {\n    todo!()\n}\n";
        let g = Agenda::edit_guard("src/gates.rs", loose, &v, &[]).expect("a loose fn warns");
        assert!(
            g.contains("`pub fn pipeline` is a loose public function"),
            "{g}"
        );
        let hung = "impl Ci {\n    pub fn pipeline() -> Pipeline { todo!() }\n}\n";
        assert_eq!(Agenda::edit_guard("src/gates.rs", hung, &v, &[]), None);
        // a REGISTERED kernel file is exempt from the shim, so the guard is silent on
        // it — and only the register grants that: a `Tier: KERNEL` marker in the
        // source is dead syntax.
        let exempt = GuardVoices {
            kernel_exempt: true,
            rats_nest: true,
        };
        assert_eq!(
            Agenda::edit_guard("src/engine.rs", loose, &exempt, &[]),
            None
        );
        let marker = "//! Tier: KERNEL — self-asserted; grants nothing.\npub fn shadow_grid() {}\n";
        assert!(Agenda::edit_guard("src/engine.rs", marker, &v, &[]).is_some());
        // and where the shim does not run, the structural voice does not exist — a
        // crate whose designed shape IS module-level `pub fn` gets silence, not a
        // warning per public function (the guard pre-fires refusals; for that tree
        // this one isn't).
        let unenforced = GuardVoices {
            kernel_exempt: false,
            rats_nest: false,
        };
        assert_eq!(
            Agenda::edit_guard("src/gates.rs", loose, &unenforced, &[]),
            None
        );

        let unknown =
            Agenda::edit_guard("spec/new.frobnicate", "", &v, &[]).expect("unknown warns");
        assert!(unknown.contains("no known class"), "{unknown}");
    }

    /// THE VOICES ARE DERIVED, not declared: kernel-exemption is a register lookup and
    /// the rats-nest voice is evidence the shim runs — so the guard's contract ("only
    /// existing refusals") holds by construction on any tree, including one that never
    /// heard of the shim.
    #[test]
    fn the_guard_voices_derive_from_the_tree() {
        let root = std::env::temp_dir().join(format!("agenda-voices-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("spec")).unwrap();

        // an ordinary crate: no shim, no register — both voices off.
        let plain = GuardVoices::for_edit(&root, "src/lib.rs");
        assert_eq!(
            plain,
            GuardVoices {
                kernel_exempt: false,
                rats_nest: false
            }
        );

        // an enforced tree: build.rs attaches the shim, the register names a file.
        std::fs::write(
            root.join("build.rs"),
            "use boundary_enforce::{Config, Enforcement};\nfn main() {}\n",
        )
        .unwrap();
        std::fs::write(
            root.join("spec/kernel.register"),
            "src/engine.rs: defines and runs the format\n",
        )
        .unwrap();
        let registered = GuardVoices::for_edit(&root, "/abs/checkout/src/engine.rs");
        assert_eq!(
            registered,
            GuardVoices {
                kernel_exempt: true,
                rats_nest: true
            }
        );
        // an unregistered file in the same tree: the shim's voice, no exemption.
        let ordinary = GuardVoices::for_edit(&root, "src/gates.rs");
        assert_eq!(
            ordinary,
            GuardVoices {
                kernel_exempt: false,
                rats_nest: true
            }
        );

        // this repo's own tree derives both voices live (the register holds engine.rs).
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
        let live = GuardVoices::for_edit(manifest, "src/discover/engine.rs");
        assert_eq!(
            live,
            GuardVoices {
                kernel_exempt: true,
                rats_nest: true
            }
        );
    }

    /// Member crates' locks route like the root's — the path prefix carries the crate,
    /// the suffix carries the class (layout-probe's shape lock and visual census both
    /// classify).
    #[test]
    fn member_lock_paths_classify_by_suffix() {
        let agenda = Agenda::of([
            "layout-probe/spec/stable-layout.spec",
            "layout-probe/spec/visual.census",
            "genesis-demo/spec/credit-app.system.spec",
        ])
        .unwrap();
        assert_eq!(
            agenda.ratifications,
            vec![
                Ratification::Laws {
                    theory: "stable layout".to_string()
                },
                Ratification::Surface,
                Ratification::Seams,
            ]
        );
    }
}
