//! probes — the unified probe census: every probe this crate upholds, with the mechanism
//! that proves it SENSITIVE (that it can fail — a green probe that cannot is a lie).
//!
//! The crate's probes were scattered across artifacts with no single roster: a theory's
//! discovered laws (`<theory>.spec`), the world judges (`perimeter`/`infra`/`substrate`),
//! the byte-locks of shape (`qualify`/`tiers`/`<system>.shape.spec`). Each carried its own
//! sensitivity proof somewhere; nothing joined them. This module is that join — one census
//! that names every probe LOCK and the mechanism proving it can fail:
//!
//!   - [`Mechanism::OracleSwap`] — the behavioural probes (discovered laws). Sensitivity is
//!     `discover::mutation`: a perturbed operator table (a VALUE) is judged by the frozen
//!     laws, and a surviving dent is the coordinate no law pins — the enumerated fine print.
//!   - [`Mechanism::LiveDent`] — the world judges (perimeter, infra, substrate). Sensitivity
//!     is `discover::judgment`'s `LiveDent` drill: one field of an applied fixture perturbed,
//!     the verdict must move AND name the fact.
//!   - [`Mechanism::DriftGate`] — the pure byte-locks (surface, tier, shape, seam, catalog).
//!     Sensitivity is only freshness: a change is caught, but there is no active proof the
//!     gate can REFUSE a planted bad fixture. This is disclosed, not hidden.
//!
//! RUNG 1 (BUILT): the census ENUMERATES every frozen probe lock and discloses its
//! mechanism, gated for COMPLETENESS against the committed `spec/` directory so no probe lock
//! can hide (`the_probe_census_covers_every_frozen_lock`).
//!
//! RUNG 1.5 (BUILT): `Mechanism::FireDrill` distinguishes the probes with an INDIVIDUAL
//! fire-drill (the catalog, the seam graph, the placer, and the world lock — each with a
//! plant-a-bad-fixture drill in its own suite, cited at [`STRUCTURAL`]) from the byte-locks
//! that lean on the shared `spec-lock drift gate`. The router files `spec/probes.spec` under
//! its own `Ratification::Probes` class.
//!
//! RUNG 2 (BUILT): NORMATIVE — every probe must carry an INDIVIDUAL active mechanism
//! (oracle-swap / live-dent / fire-drill), or be a ratified line in `spec/probes.register`
//! explaining it leans only on the shared drift gate. `every_probe_has_an_individual_drill_or_is_ratified`
//! holds the drift-gate set to the register by SET DIFFERENCE, so it can only shrink as
//! byte-locks earn individual drills — an un-ratified byte-lock refuses, a stale line refuses.
//!
//! RUNG 3 (BUILT): the `FireDrill` claims are NON-VACUOUS. `every_fire_drill_probe_has_a_live_drill_that_fires`
//! reconciles the census's FireDrill set through a `fire_drill::Battery` that `requires` those
//! keys and drills each by planting the probe's known-bad fixture — so a FireDrill probe with
//! no live drill is UNPROVEN and a drill that stopped firing is VACUOUS. A deleted drill
//! breaks the gate, closing the "a FireDrill claim pinned only by a test that could vanish"
//! gap. Each module owns its drill (`system::tests::seam_sensitivity_drill_fires`,
//! `shape::probes::shape_sensitivity_drill_fires`); catalog and world build from the public API.
//!
//! Not mutated: the census is aggregation whose decisions are pinned by its byte-render
//! probes and the completeness gate — GENERATED against, like `discover::floor` (see
//! `spec/instrumentation.register`).

use std::path::{Path, PathBuf};

use spec_lock::Lock;

use super::all_specs;

/// Whether a probe reads the module's SHAPE (structural) or its CONDUCT (behavioural).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ProbeKind {
    /// A probe over discovered laws — what the module upholds by its conduct.
    Behavioural,
    /// A probe over shape — surface, tier, placement, seam, catalog, world judge.
    Structural,
}

/// How a probe proves it is SENSITIVE — that it can fail. One question, mechanisms by kind.
/// The first three are INDIVIDUAL, active per-probe proofs; [`Mechanism::DriftGate`] is the
/// shared, generic one — normatively ratified in `spec/probes.register` (rung 2).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mechanism {
    /// Behavioural: a perturbed operator table judged by the frozen laws (`discover::mutation`).
    /// Each theory has its own dent/deaf sweep; the `algebra mutation` fire-drill proves the
    /// mechanism itself fires (a lawless theory yields survivors).
    OracleSwap,
    /// Structural world judge: one applied-fixture field perturbed, verdict must move and
    /// name the fact (`discover::judgment`'s `LiveDent`). Individual to each judge.
    LiveDent,
    /// Structural: an INDIVIDUAL fire-drill plants a probe-specific known-bad fixture and
    /// demands refusal (`fire_drill::Battery` — the catalog's `shape data gate` and
    /// `expectation vocabulary` drills).
    FireDrill,
    /// Structural byte-lock with NO individual drill: sensitivity is the SHARED `spec-lock
    /// drift gate` fire-drill (a tampered/missing committed lock is caught), which proves the
    /// drift mechanism fires but not that this lock's own derivation is individually
    /// sensitive. Rung 2 makes each a ratified line in `spec/probes.register`, so the set can
    /// only shrink as byte-locks earn individual drills.
    DriftGate,
}

impl Mechanism {
    /// The lock-text label for this mechanism.
    pub fn label(self) -> &'static str {
        match self {
            Mechanism::OracleSwap => "oracle-swap",
            Mechanism::LiveDent => "live-dent",
            Mechanism::FireDrill => "fire-drill",
            Mechanism::DriftGate => "drift-gate",
        }
    }
}

/// One probe lock: its key, whether it reads shape or conduct, and its sensitivity mechanism.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Probe {
    /// The probe's key — a theory name (behavioural) or a lock name (structural).
    pub key: String,
    pub kind: ProbeKind,
    pub mechanism: Mechanism,
}

/// The externally-mounted theory (`Bridged<0>`), frozen outside `all_specs()` — its lock is
/// `bridged-bool.spec`, so its probe key matches the slug read back to a theory name.
const BRIDGED_THEORY: &str = "bridged bool";

/// Every structural probe lock this crate freezes, with its sensitivity mechanism. Static
/// because a structural lock is a deliberate artifact, not a derived list; the completeness
/// gate (`the_probe_census_covers_every_frozen_lock`) forces a NEW lock to be added here,
/// so the roster can only grow honestly.
const STRUCTURAL: &[(&str, Mechanism)] = &[
    // world judges — the LiveDent drill (discover::judgment) proves each can refuse.
    ("perimeter", Mechanism::LiveDent),
    ("infra", Mechanism::LiveDent),
    ("substrate", Mechanism::LiveDent),
    // INDIVIDUAL fire-drills — each plants a probe-specific known-bad fixture and demands
    // refusal, in the module's own suite:
    //   catalog  — a mis-sorted identity is refused (`shape data gate`) and an unratified
    //              shape name rejected (`expectation vocabulary`), in fire_drill::Battery.
    //   seams    — a non-homomorphic conversion leaves the transform seam UNEARNED, named in
    //              the render (system::probes::a_broken_conversion_leaves_the_transform_seam_unearned).
    //   shape    — a placement disagreeing with the declaration renders "DISAGREES" and is
    //              not settled (shape::probes::a_disagreeing_shape_renders_loud).
    //   world    — a broken vendor's divergence and a mismatched battery are refused and
    //              named (world::probes::reports_over_different_batteries_are_refused).
    ("catalog", Mechanism::FireDrill),
    ("seams", Mechanism::FireDrill),
    ("shape", Mechanism::FireDrill),
    ("world", Mechanism::FireDrill),
    // byte-locks — only the shared `spec-lock drift gate` fire-drill; each ratified in
    // spec/probes.register (rung 2) until it earns an individual drill.
    ("surface", Mechanism::DriftGate),
    ("tiers", Mechanism::DriftGate),
    ("pipeline", Mechanism::DriftGate),
    ("schemata", Mechanism::DriftGate),
];

/// The unified probe census — every probe lock with the mechanism that proves it sensitive.
pub struct ProbeCensus {
    probes: Vec<Probe>,
}

impl ProbeCensus {
    /// Derive the census: behavioural probes from the discovered-theory registry
    /// (`all_specs()`, so a new theory appears automatically) plus the externally-mounted
    /// bridged theory, and the structural probe roster. Pure — reads no filesystem; the
    /// completeness cross-check against `spec/` lives in the freshness test.
    pub fn of() -> ProbeCensus {
        let mut probes: Vec<Probe> = all_specs()
            .iter()
            .map(|spec| Probe {
                key: spec.theory.to_string(),
                kind: ProbeKind::Behavioural,
                mechanism: Mechanism::OracleSwap,
            })
            .collect();
        probes.push(Probe {
            key: BRIDGED_THEORY.to_string(),
            kind: ProbeKind::Behavioural,
            mechanism: Mechanism::OracleSwap,
        });
        for (key, mechanism) in STRUCTURAL {
            probes.push(Probe {
                key: key.to_string(),
                kind: ProbeKind::Structural,
                mechanism: *mechanism,
            });
        }
        ProbeCensus { probes }
    }

    /// The probes, in derivation order (behavioural theories, bridged, then structural).
    pub fn probes(&self) -> &[Probe] {
        &self.probes
    }

    /// The canonical census text — deterministic, diffable. Behavioural probes sorted by
    /// key, then structural probes sorted by key, each with its mechanism label.
    pub fn render(&self) -> String {
        let mut out = String::new();
        out.push_str(
            "# probe census: every probe lock this crate upholds, with the mechanism that \
             proves it sensitive.\n",
        );
        out.push_str(
            "# a green probe that cannot fail is a lie; this roster names how each proves it \
             can. Regenerate\n# via `cargo run --example freeze_spec` and ratify the diff.\n",
        );
        out.push_str(
            "# mechanisms: oracle-swap (laws, discover::mutation), live-dent (world judges, \
             discover::judgment),\n#   drift-gate (byte-lock, freshness only — rung 1: \
             disclosed, no active refutation drill; see docs/roadmap.md).\n\n",
        );
        for (heading, kind) in [
            (
                "## behavioural probes (conduct — laws over a grid)",
                ProbeKind::Behavioural,
            ),
            ("## structural probes (shape)", ProbeKind::Structural),
        ] {
            out.push_str(heading);
            out.push('\n');
            let mut rows: Vec<&Probe> = self.probes.iter().filter(|p| p.kind == kind).collect();
            rows.sort_by(|a, b| a.key.cmp(&b.key));
            for probe in rows {
                out.push_str(&format!("- {}: {}\n", probe.key, probe.mechanism.label()));
            }
            out.push('\n');
        }
        out
    }

    /// This census as a `spec_lock::Lock` in a caller-supplied spec directory — the
    /// consumer-facing form (`spec_dir/probes.spec`).
    pub fn lock_in(&self, spec_dir: &Path) -> Lock {
        Lock {
            name: "probes".to_string(),
            path: spec_dir.join("probes.spec"),
            live: self.render(),
        }
    }

    /// This census as a `spec_lock::Lock` in THIS repo's `spec/` directory — for this
    /// repository's own freeze and staleness gate.
    pub fn lock(&self) -> Lock {
        self.lock_in(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("spec"))
    }

    /// The NORMATIVE register (rung 2): the hand-authored ratification of every probe that
    /// leans only on the shared `spec-lock drift gate` (a [`Mechanism::DriftGate`] probe). A
    /// probe with an individual active mechanism needs no line; a drift-gate probe without a
    /// line is an un-ratified finding, and a line for a probe that earned an individual drill
    /// is stale — so the drift-gate set can only shrink honestly (`spec_lock::Register`).
    pub fn register(&self) -> spec_lock::Register {
        spec_lock::Register {
            name: "probes".to_string(),
            path: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("spec")
                .join("probes.register"),
        }
    }

    /// The keys of the probes leaning only on the shared drift gate — the live finding set
    /// the register ratifies.
    pub fn drift_gate_keys(&self) -> Vec<&str> {
        self.probes
            .iter()
            .filter(|p| p.mechanism == Mechanism::DriftGate)
            .map(|p| p.key.as_str())
            .collect()
    }
}

#[cfg(test)]
mod probes_tests {
    use super::*;
    use std::collections::BTreeSet;
    use std::fs;

    /// Bucket a committed `spec/` artifact by whether it backs a BEHAVIOURAL probe (a theory
    /// key), a STRUCTURAL probe (a lock name), or is EXCLUDED (a mutation verdict, a
    /// ratification register, or an apply artifact — none of which is a probe). Kept minimal
    /// and local: the completeness gate cares only which bucket, not the full agenda taxonomy.
    enum Backs {
        Behavioural(String),
        Structural(&'static str),
        Excluded,
    }

    fn bucket(file: &str) -> Backs {
        let theory = |suffix: &str| file.trim_end_matches(suffix).replace('-', " ");
        // excluded: the mutation lock IS a probe's verdict, not a probe; registers are
        // ratification inputs; the ruleset is an apply artifact; the census itself is not
        // its own probe.
        if file.ends_with(".mutation.spec")
            || file.ends_with(".register")
            || file.ends_with(".ruleset.json")
            || file == "probes.spec"
        {
            return Backs::Excluded;
        }
        // structural byte-locks and world judges, keyed to the roster.
        let structural = match file {
            "qualify.spec" => Some("surface"),
            "tiers.spec" => Some("tiers"),
            "shapes.spec" => Some("catalog"),
            "gates.spec" => Some("pipeline"),
            "schemata.spec" => Some("schemata"),
            "perimeter.spec" => Some("perimeter"),
            "substrate.spec" => Some("substrate"),
            _ if file.ends_with(".shape.spec") => Some("shape"),
            _ if file.ends_with(".system.spec") => Some("seams"),
            _ if file.ends_with(".world.spec") => Some("world"),
            _ if file.ends_with(".infra.spec") => Some("infra"),
            _ => None,
        };
        if let Some(key) = structural {
            return Backs::Structural(key);
        }
        // everything else in spec/ is a theory law lock (.spec / .export / .obligations.spec).
        for suffix in [".obligations.spec", ".export", ".spec"] {
            if file.ends_with(suffix) {
                return Backs::Behavioural(theory(suffix));
            }
        }
        Backs::Excluded
    }

    /// COMPLETENESS: every frozen `spec/` artifact that backs a probe is covered by the
    /// census, and every census probe is backed by at least one committed artifact. A new
    /// lock that no probe covers fails here — add it to the roster and ratify. (Reads the
    /// committed spec directory: the honest source of what locks exist.)
    #[test]
    fn the_probe_census_covers_every_frozen_lock() {
        let census = ProbeCensus::of();
        let behavioural: BTreeSet<String> = census
            .probes()
            .iter()
            .filter(|p| p.kind == ProbeKind::Behavioural)
            .map(|p| p.key.clone())
            .collect();
        let structural: BTreeSet<&str> = census
            .probes()
            .iter()
            .filter(|p| p.kind == ProbeKind::Structural)
            .map(|p| p.key.as_str())
            .collect();

        let spec_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("spec");
        let mut seen_behavioural: BTreeSet<String> = BTreeSet::new();
        let mut seen_structural: BTreeSet<&str> = BTreeSet::new();
        for entry in fs::read_dir(&spec_dir).expect("read spec dir") {
            let name = entry.expect("dir entry").file_name();
            let file = name.to_string_lossy().to_string();
            match bucket(&file) {
                Backs::Behavioural(theory) => {
                    assert!(
                        behavioural.contains(&theory),
                        "spec/{file} backs behavioural probe `{theory}` — no probe covers it; \
                         add it to the census and ratify"
                    );
                    seen_behavioural.insert(theory);
                }
                Backs::Structural(key) => {
                    assert!(
                        structural.contains(key),
                        "spec/{file} backs structural probe `{key}` — no probe covers it; add \
                         it to STRUCTURAL and ratify"
                    );
                    seen_structural.insert(key);
                }
                Backs::Excluded => {}
            }
        }
        // reverse: no phantom probes — every census probe is backed by a committed artifact.
        for key in &behavioural {
            assert!(
                seen_behavioural.contains(key),
                "census names behavioural probe `{key}` with no committed lock — stale roster"
            );
        }
        for key in &structural {
            assert!(
                seen_structural.contains(key),
                "census names structural probe `{key}` with no committed lock — stale roster"
            );
        }
    }

    /// The committed `spec/probes.spec` is FRESH: the live census matches what was frozen.
    /// Regenerate with `cargo run --example freeze_spec` and ratify the diff.
    #[test]
    fn the_committed_probe_census_is_fresh() {
        let lock = ProbeCensus::of().lock();
        match spec_lock::check(std::slice::from_ref(&lock)) {
            Ok(_) => {}
            Err(stale) => panic!(
                "the probe census drifted from spec/probes.spec ({}). Run \
                 `cargo run --example freeze_spec` and ratify the diff.",
                stale.join(", ")
            ),
        }
    }

    /// The render is grouped, sorted, and mechanism-labelled — byte pins on the shape so a
    /// silent reformat cannot pass.
    #[test]
    fn the_render_groups_and_labels() {
        let text = ProbeCensus::of().render();
        assert!(text.contains("## behavioural probes (conduct — laws over a grid)"));
        assert!(text.contains("## structural probes (shape)"));
        assert!(text.contains("- perimeter: live-dent"));
        assert!(text.contains("- surface: drift-gate"));
        assert!(text.contains("- catalog: fire-drill"));
        assert!(text.contains("- seams: fire-drill"));
        assert!(text.contains("- world: fire-drill"));
        // behavioural section lists a theory as oracle-swap.
        assert!(text.contains(": oracle-swap"));
    }

    /// NORMATIVE (rung 2): every probe carries an INDIVIDUAL active mechanism, or its reliance
    /// on the shared `spec-lock drift gate` is ratified in `spec/probes.register`. Held by set
    /// difference — an un-ratified drift-gate probe is a new finding, a ratified line for a
    /// probe that earned an individual drill is stale. The drift-gate set can only shrink.
    #[test]
    fn every_probe_has_an_individual_drill_or_is_ratified() {
        let census = ProbeCensus::of();
        if let Err(drift) = census.register().check(census.drift_gate_keys()) {
            panic!(
                "the probe register drifted from the drift-gate probe set: {drift}\n\
                 (a byte-lock leaning only on the shared spec-lock drift gate must be ratified \
                 in spec/probes.register with the reason; give it an individual drill to drop \
                 the line)"
            );
        }
    }

    /// RUNG 3: every `FireDrill` probe has a LIVE drill that actually FIRES — the census
    /// classification is not vacuous. Reconciled through `fire_drill::Battery`: the battery
    /// `requires` exactly the census's FireDrill keys and drills each by planting the probe's
    /// known-bad fixture, so `verdict()` refuses a FireDrill probe with no drill (UNPROVEN)
    /// and a drill that stopped firing (VACUOUS). A deleted drill therefore breaks this gate,
    /// closing the "a FireDrill claim pinned only by a test that could vanish" gap.
    #[test]
    fn every_fire_drill_probe_has_a_live_drill_that_fires() {
        use fire_drill::{Battery, Outcome};
        fn fired(refused: bool) -> Outcome {
            if refused {
                Outcome::Fired
            } else {
                Outcome::Passed
            }
        }
        let census = ProbeCensus::of();
        let keys: Vec<String> = census
            .probes()
            .iter()
            .filter(|p| p.mechanism == Mechanism::FireDrill)
            .map(|p| p.key.clone())
            .collect();

        // catalog — a mis-sorted identity declaration is refused by the shape-data gate.
        let identity = crate::discover::engine::ShapeCatalog::inventory()
            .into_iter()
            .find(|s| s.name == "identity")
            .expect("identity is ratified")
            .gate_slots;
        let missorted: Vec<(Vec<&str>, &str)> = vec![
            (vec!["credits", "credits"], "credits"),
            (vec![], "vouchers"),
        ];
        let catalog = identity.admit(&missorted, &["merge", "empty"]).is_err();

        // world — a battery of a different size is refused, never row-matched by luck.
        let world = !crate::discover::world::StoreModel::beliefs()
            .disagreements(&crate::discover::world::WorldReport::record(
                "small",
                4,
                crate::discover::world::StoreModel::replay,
            ))
            .is_empty();

        let battery = Battery::named("probe sensitivity")
            .requires(keys)
            .drill(
                "catalog",
                "a mis-sorted identity declaration",
                fired(catalog),
            )
            .drill(
                "seams",
                "a non-homomorphic transform conversion",
                fired(crate::discover::system::tests::seam_sensitivity_drill_fires()),
            )
            .drill(
                "shape",
                "a placement disagreeing with the declaration",
                fired(crate::discover::shape::probes::shape_sensitivity_drill_fires()),
            )
            .drill("world", "a mismatched-size world battery", fired(world));

        if let Err(why) = battery.verdict() {
            panic!(
                "the probe-sensitivity battery is not sound: {why}\n(a FireDrill probe with \
                 no drill is UNPROVEN; a drill that no longer fires is VACUOUS — reconcile \
                 discover::probes STRUCTURAL against the live drills)"
            );
        }
    }
}
