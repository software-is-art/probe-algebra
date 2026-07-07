//! Tier: ALGEBRA — a discovered-law / report layer (exempt from the inward rule).
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
    /// The pipeline moved (gates.spec / ci.yml): the promises or their execution
    /// changed — re-read what CI now claims.
    Pipeline,
    /// An exception register moved: a justification was added, changed, or resolved.
    Exceptions { register: String },
    /// The world lock moved: the model's beliefs about a dependency were re-ratified.
    World,
    /// A shapes.spec move: the law LANGUAGE itself grew or changed.
    Vocabulary,
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
        let mut agenda = Agenda {
            ratifications: Vec::new(),
            prose: Vec::new(),
            machinery: Vec::new(),
        };
        for path in paths {
            let path = path.as_ref().to_string();
            let class = classify(&path)?;
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

enum Class {
    Ratify(Ratification),
    Prose,
    Machinery,
}

/// One path to its review class. The spec-directory refusal is the router's own
/// completeness gate: an unrecognized `spec/` artifact means a lock class the router
/// was never taught, and misfiling it as machinery would silently drop a ratification.
fn classify(path: &str) -> Result<Class, String> {
    let file = path.rsplit('/').next().unwrap_or(path);
    let theory = |suffix: &str| file.trim_end_matches(suffix).replace('-', " ").to_string();
    if path.contains("spec/") || path.starts_with("spec/") {
        return Ok(Class::Ratify(if file == "qualify.spec" {
            Ratification::Surface
        } else if file == "gates.spec" {
            Ratification::Pipeline
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
    if file.ends_with(".register") {
        return Ok(Class::Ratify(Ratification::Exceptions {
            register: path.to_string(),
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
            ".github/workflows/ci.yml",
            "spec/gates.spec",
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
                Ratification::Pipeline,
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
            "# review agenda — 6 ratification(s) required; 2 file(s) machinery-verified.\n"
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
