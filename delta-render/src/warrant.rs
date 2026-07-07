//! warrant — uninterpreted operators with RATIFIED properties: the license judged over
//! sampled interpretations, and every property made to earn its place by removal.
//!
//! Every operator in `ops` has an implementation to probe. A real pipeline's inventory
//! is open: somewhere there is an `enrich` the team wrote last week, with no spec and no
//! grid. Phase 7's answer is to license the SYMBOL from its declared properties alone —
//! sample N interpretations constrained only by those properties (deterministically:
//! splitmix64 from a fixed seed, so the artifact is byte-stable) and judge the circuit
//! law over every one. Survival is evidence the license needs nothing an implementation
//! could smuggle in.
//!
//! The REMOVAL DRILL is the half that keeps the property list honest, and it is a fire
//! drill in its own right: drop one property, re-sample under the remaining constraints
//! (each counter-sample VIOLATES the dropped property, satisfies the rest), and demand
//! the circuit law fail. A property whose removal refutes nothing carried no license
//! weight — DECORATION, flagged in the artifact and never ratified. The demonstration
//! set plants one on purpose (`bounded-fanout`), so the flag's polarity is exercised on
//! every regeneration, not trusted.
//!
//! One subtlety, disclosed rather than smoothed over: FULL additivity implies zero
//! preservation (`f(∅) = f(∅) ⊕ f(∅)`, and a cancellation pair reaches the same point
//! from non-empty inputs), which would make `zero-preserving` decoration by logic alone
//! and its drill counter-sample unsatisfiable. The declared properties are therefore
//! independent constraints — `additive` constrains combination AWAY from the basepoint
//! (both inputs and their sum non-empty), `zero-preserving` pins the basepoint —
//! mirroring how the license classifier already reads them as two separate laws
//! (`license.rs` pins that the homomorphism alone licenses nothing).
//!
//! Honest frame: sampled interpretations are a bounded battery. The drill REFUTES
//! decoration and warrants necessity; it proves neither. And the warrant is not yet a
//! circuit admission — wiring a warranted opaque node into `Registry`/`circuit` is the
//! recorded residual, kept out until a real open-inventory consumer forces its shape.

use std::cell::Cell;
use std::collections::BTreeMap;
use std::path::Path;

use crate::stream::{grid as stream_grid, Stream};
use crate::zset::{Row, ZSet};

/// The opaque symbol under warrant — no inventoried implementation, on purpose.
pub const OPERATOR: &str = "enrich";

/// Interpretations sampled per arm (the full-constraint arm and each removal arm).
pub const SAMPLES: usize = 8;

/// The fixed sampling seed (splitmix64's golden-ratio increment, reused as a seed) —
/// every derivation of the warrant walks the same interpretations, so the rendered
/// artifact is deterministic and lockable.
pub const SEED: u64 = 0x9E37_79B9_7F4A_7C15;

// Violation carriers live outside the generator-image row pool (0..8), so a broken
// property is visible as rows no honest interpretation could mint.
const KICKER: u8 = 250; // additivity violation: minted when an input has ≥ 2 rows
const BASEPOINT: u8 = 251; // zero-preservation violation: the image of the empty set
const PHANTOM: u8 = 252; // determinism violation: weight = the running call count

/// A declared property of the uninterpreted operator — a CONSTRAINT on the sampler,
/// with independent content (see the module doc for why independence is load-bearing).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Property {
    /// `enrich (x plus y) = (enrich x) plus (enrich y)` away from the basepoint (`x`,
    /// `y`, and their sum all non-empty) — how combination passes through; the
    /// basepoint itself is `ZeroPreserving`'s, separately.
    Additive,
    /// `enrich zero = zero` — the basepoint, NOT implied here: additivity constrains
    /// only non-empty pairs.
    ZeroPreserving,
    /// The same input answers identically on every call.
    Deterministic,
    /// One row maps to at most two rows — the planted decoration: true of the honest
    /// sampler, weightless for the license.
    BoundedFanout,
}

impl Property {
    /// Every declared property, in artifact order.
    pub fn all() -> [Property; 4] {
        [
            Property::Additive,
            Property::ZeroPreserving,
            Property::Deterministic,
            Property::BoundedFanout,
        ]
    }

    /// The artifact token.
    pub fn name(&self) -> &'static str {
        match self {
            Property::Additive => "additive",
            Property::ZeroPreserving => "zero-preserving",
            Property::Deterministic => "deterministic",
            Property::BoundedFanout => "bounded-fanout",
        }
    }

    /// The claim, in the registry's law voice.
    pub fn claim(&self) -> &'static str {
        match self {
            Property::Additive => "enrich turns plus into plus (away from the basepoint)",
            Property::ZeroPreserving => "enrich leaves zero fixed (the basepoint)",
            Property::Deterministic => "enrich answers the same input identically, every call",
            Property::BoundedFanout => "enrich maps one row to at most two rows",
        }
    }

    /// Does an interpretation satisfy THIS property, checked directly (not through the
    /// circuit law)? This is the sampler-honesty oracle the probes hold both arms of:
    /// fully-constrained samples pass every property; a counter-sample fails exactly
    /// the one it was asked to violate.
    pub fn satisfied_by(&self, f: &Interpretation) -> bool {
        let grid = crate::zset::grid();
        let nonempty = |z: &&ZSet| **z != ZSet::empty();
        match self {
            Property::Additive => grid.iter().filter(nonempty).all(|a| {
                grid.iter()
                    .filter(nonempty)
                    .filter(|b| a.plus(b) != ZSet::empty())
                    .all(|b| f.apply(&a.plus(b)) == f.apply(a).plus(&f.apply(b)))
            }),
            Property::ZeroPreserving => f.apply(&ZSet::empty()) == ZSet::empty(),
            Property::Deterministic => grid.iter().all(|z| {
                let first = f.apply(z);
                first == f.apply(z)
            }),
            Property::BoundedFanout => {
                (0..2u8).all(|r| f.apply(&ZSet::of(&[(Row::new(r), 1)])).entries().len() <= 2)
            }
        }
    }
}

// ===== deterministic sampling ==============================================

/// splitmix64 — the whole randomness budget. In-crate on purpose (zero-dep kept), and
/// fixed-seeded so the warrant is a derivation, not a dice roll.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Rng {
        Rng(seed)
    }
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

/// One sampled interpretation of the opaque operator: a generator table extended
/// linearly (`f(z) = Σ weight_r · g(r)`), plus at most one deliberate violation.
pub struct Interpretation {
    /// Generator images: `g(r)` for each row the grids can mint.
    table: BTreeMap<u8, ZSet>,
    /// The one property this sample violates (`None` on the full-constraint arm).
    violate: Option<Property>,
    /// The determinism violation's state: a call counter minting phantom weight.
    calls: Cell<i64>,
}

/// `z` scaled by `w`, pointwise saturating — the linear extension's multiplier.
fn scale_by(z: &ZSet, w: i64) -> ZSet {
    ZSet::of(
        &z.entries()
            .into_iter()
            .map(|(r, wi)| (r, wi.saturating_mul(w)))
            .collect::<Vec<_>>(),
    )
}

impl Interpretation {
    /// Sample one interpretation. Constrained by every declared property except
    /// `violate`, which (when present) is deliberately BROKEN — the removal drill's
    /// counter-sample. Generator images are non-empty by construction (distinct rows,
    /// non-zero weights), so no sample is the trivial constant-zero map and the law
    /// checks are never vacuous.
    fn sample(rng: &mut Rng, violate: Option<Property>) -> Interpretation {
        let fanout = if violate == Some(Property::BoundedFanout) {
            4 // the bound is two — a four-row image breaks exactly this claim
        } else {
            1 + rng.below(2) as usize
        };
        let mut table = BTreeMap::new();
        for r in 0..2u8 {
            let mut image: Vec<(Row, i64)> = Vec::new();
            let mut id = rng.below(3) as u8;
            for _ in 0..fanout {
                let magnitude = 1 + rng.below(3) as i64;
                let weight = if rng.below(2) == 0 {
                    magnitude
                } else {
                    -magnitude
                };
                image.push((Row::new(id), weight));
                id += 1 + rng.below(2) as u8; // strictly increasing ids: no cancellation
            }
            table.insert(r, ZSet::of(&image));
        }
        Interpretation {
            table,
            violate,
            calls: Cell::new(0),
        }
    }

    /// Run the interpretation. The honest path is the linear extension of the
    /// generator table; each violation perturbs exactly its own property's content.
    pub fn apply(&self, z: &ZSet) -> ZSet {
        if self.violate == Some(Property::ZeroPreserving) && *z == ZSet::empty() {
            // the broken basepoint: the empty set maps somewhere visible.
            return ZSet::of(&[(Row::new(BASEPOINT), 1)]);
        }
        let mut out = ZSet::empty();
        for (r, w) in z.entries() {
            if let Some(g) = self.table.get(&r.get()) {
                out = out.plus(&scale_by(g, w));
            }
        }
        if self.violate == Some(Property::Additive) && z.entries().len() >= 2 {
            // the nonlinearity: multi-row inputs mint a kicker their parts don't.
            out = out.plus(&ZSet::of(&[(Row::new(KICKER), 1)]));
        }
        if self.violate == Some(Property::Deterministic) {
            // the broken promise: every call answers a little differently.
            self.calls.set(self.calls.get() + 1);
            out = out.plus(&ZSet::of(&[(Row::new(PHANTOM), self.calls.get())]));
        }
        out
    }

    /// The generator table in artifact prose: `g(0) = {2:+2}; g(1) = {0:-1 3:+1}` — the
    /// DISCLOSURE that makes the sampler lock-visible. Rendered into the warrant for
    /// sample #0 of every arm, so any change to the randomness stream, the sampling
    /// arithmetic, or the per-arm seed derivation moves committed bytes: the mutation
    /// sweep found twelve sampler mutants the artifact was invariant to, and this line
    /// is their grave.
    pub fn table_render(&self) -> String {
        self.table
            .iter()
            .map(|(r, g)| format!("g({r}) = {}", show(g)))
            .collect::<Vec<_>>()
            .join("; ")
    }
}

// ===== the judged law and the artifact =====================================

/// A Z-set in witness prose: `{0:+1 1:-2}`, `∅` for empty — the artifact's value voice.
fn show(z: &ZSet) -> String {
    if *z == ZSet::empty() {
        return "∅".to_string();
    }
    let inner: Vec<String> = z
        .entries()
        .iter()
        .map(|(r, w)| format!("{}:{:+}", r.get(), w))
        .collect();
    format!("{{{}}}", inner.join(" "))
}

/// The circuit law the warrant judges: the LINEAR rewrite `Q^Δ = Q` in end-gate form,
/// `i(enrich(d(s))) = enrich(s)` tickwise over the whole stream grid. `Ok` is survival;
/// `Err` carries the first witness (stream, tick, both sides).
pub fn circuit_law(f: &Interpretation) -> Result<(), String> {
    for (n, s) in stream_grid().iter().enumerate() {
        let batch = Stream::of(&s.ticks().iter().map(|z| f.apply(z)).collect::<Vec<_>>());
        let incremental = Stream::of(
            &s.differentiate()
                .ticks()
                .iter()
                .map(|z| f.apply(z))
                .collect::<Vec<_>>(),
        )
        .integrate();
        if incremental != batch {
            let t = (0..crate::stream::DEPTH)
                .find(|t| incremental.at(*t) != batch.at(*t))
                .expect("the streams differ, so some tick does");
            return Err(format!(
                "stream #{n}, tick {t}: incremental {} ≠ batch {}",
                show(&incremental.at(t)),
                show(&batch.at(t))
            ));
        }
    }
    Ok(())
}

/// One removal arm's verdict: the property, how many counter-samples refuted the law,
/// and the first witness when any did.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct DrillRow {
    pub property: Property,
    pub refuted: usize,
    pub witness: Option<String>,
    /// Counter-sample #0's generator table, disclosed — the sampler made lock-visible.
    pub sample0: String,
}

impl DrillRow {
    /// A property is LOAD-BEARING when its removal refuted the law somewhere;
    /// refuting nowhere is the decoration finding.
    pub fn load_bearing(&self) -> bool {
        self.refuted > 0
    }
}

/// The whole warrant: the full-constraint verdict plus every removal arm, rendered as
/// `spec/enrich.warrant.spec`.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Warrant {
    /// Did the circuit law hold over every fully-constrained sample?
    pub held: bool,
    /// Sample #0's generator table (fully constrained), disclosed — see
    /// [`Interpretation::table_render`] for why disclosure is load-bearing.
    pub sample0: String,
    /// The full arm's story (the survival sentence, or the first refutation).
    pub held_detail: String,
    /// One row per declared property, in `Property::all()` order.
    pub rows: Vec<DrillRow>,
}

impl Warrant {
    /// Derive the warrant: the full-constraint arm, then one removal arm per declared
    /// property. Each arm walks its own splitmix64 stream (seed ⊕ arm index), so adding
    /// a property leaves the other arms' bytes alone.
    pub fn derive() -> Warrant {
        let mut rng = Rng::new(SEED);
        let mut held = true;
        let mut held_detail = format!(
            "the circuit law held over all {SAMPLES} sampled interpretations × {} \
             stream-grid inhabitants",
            stream_grid().len()
        );
        let mut sample0 = String::new();
        for i in 0..SAMPLES {
            let f = Interpretation::sample(&mut rng, None);
            if i == 0 {
                sample0 = f.table_render();
            }
            if let Err(w) = circuit_law(&f) {
                held = false;
                held_detail = format!("sample #{i} REFUTED the law: {w}");
                break;
            }
        }
        let rows = Property::all()
            .iter()
            .enumerate()
            .map(|(arm, p)| {
                let mut rng = Rng::new(SEED ^ (arm as u64 + 1));
                let mut refuted = 0;
                let mut witness = None;
                let mut sample0 = String::new();
                for i in 0..SAMPLES {
                    let f = Interpretation::sample(&mut rng, Some(*p));
                    if i == 0 {
                        sample0 = f.table_render();
                    }
                    if let Err(w) = circuit_law(&f) {
                        refuted += 1;
                        witness.get_or_insert(format!("first witness: sample #{i}, {w}"));
                    }
                }
                DrillRow {
                    property: *p,
                    refuted,
                    witness,
                    sample0,
                }
            })
            .collect();
        Warrant {
            held,
            sample0,
            held_detail,
            rows,
        }
    }

    /// The warrant as deterministic text — `spec/enrich.warrant.spec`'s whole content.
    pub fn render(&self) -> String {
        let mut out = format!(
            "# uninterpreted-operator warrant: `{OPERATOR}` — a linear license held on SAMPLED\n\
             # evidence; regenerate via `cargo run -p delta-render --example freeze` and ratify\n\
             # the diff.\n\
             #\n\
             # `{OPERATOR}` has no inventoried implementation — it stands for the open half of a\n\
             # real pipeline's operator inventory. Its license is judged over interpretations\n\
             # sampled under the declared properties (deterministic: splitmix64, seed\n\
             # {SEED:#018x}), and every declared property must earn its place by the REMOVAL\n\
             # DRILL: drop it, re-sample under the remaining constraints, and the circuit law\n\
             # must fail. A property whose removal refutes nothing is DECORATION — flagged\n\
             # below, never ratified.\n\
             #\n\
             # Honest frame: sampled interpretations are a bounded battery — the drill refutes\n\
             # decoration and warrants necessity; it proves neither.\n\
             #\n\
             # the circuit law (the linear rewrite Q^Δ = Q, end-gate form, tickwise over the\n\
             # stream grid): i({OPERATOR}(d(s))) = {OPERATOR}(s)\n\n"
        );
        out.push_str(&format!(
            "license: {} — {}.\n      sample #0 (fully constrained): {}\n",
            if self.held {
                "LINEAR, warranted"
            } else {
                "REFUSED"
            },
            self.held_detail,
            self.sample0
        ));
        out.push_str("\nratified properties (each load-bearing under the removal drill):\n");
        for row in self.rows.iter().filter(|r| r.load_bearing()) {
            out.push_str(&format!(
                "- {}: {}.\n      removal refuted the law in {} of {SAMPLES} counter-samples; {}\n",
                row.property.name(),
                row.property.claim(),
                row.refuted,
                row.witness.as_deref().expect("a load-bearing row has one")
            ));
            out.push_str(&format!("      counter-sample #0: {}\n", row.sample0));
        }
        out.push_str(
            "\ndecoration (declared, drilled, found weightless — flagged, not ratified):\n",
        );
        for row in self.rows.iter().filter(|r| !r.load_bearing()) {
            out.push_str(&format!(
                "- {}: {}.\n      removal refuted the law in 0 of {SAMPLES} counter-samples — \
                 the license never leaned on it.\n      counter-sample #0: {}\n",
                row.property.name(),
                row.property.claim(),
                row.sample0,
            ));
        }
        out
    }

    /// The warrant as a lock in this crate's `spec/` directory.
    pub fn lock_in(&self, spec_dir: &Path) -> spec_lock::Lock {
        spec_lock::Lock {
            name: "enrich warrant".into(),
            path: spec_dir.join("enrich.warrant.spec"),
            live: self.render(),
        }
    }
}

#[cfg(test)]
mod probes {
    use super::*;

    /// SAMPLER HONESTY, both arms — the pin that keeps the whole artifact from going
    /// vacuous: a fully-constrained sample satisfies every declared property under the
    /// DIRECT check (not the circuit law), and each counter-sample fails exactly the
    /// property it was asked to violate while keeping the untangled ones.
    #[test]
    fn the_sampler_is_honest_about_every_property() {
        let mut rng = Rng::new(SEED);
        for _ in 0..SAMPLES {
            let f = Interpretation::sample(&mut rng, None);
            for p in Property::all() {
                assert!(
                    p.satisfied_by(&f),
                    "a fully-constrained sample must satisfy `{}`",
                    p.name()
                );
            }
        }
        for (arm, p) in Property::all().iter().enumerate() {
            let mut rng = Rng::new(SEED ^ (arm as u64 + 1));
            let f = Interpretation::sample(&mut rng, Some(*p));
            assert!(
                !p.satisfied_by(&f),
                "a counter-sample must violate `{}` under the direct check",
                p.name()
            );
        }
        // and the violations stay untangled where independence is claimed: the broken
        // basepoint keeps (non-empty) additivity; the broken additivity keeps the
        // basepoint and the fan-out bound.
        let mut rng = Rng::new(SEED ^ 2);
        let zero_broken = Interpretation::sample(&mut rng, Some(Property::ZeroPreserving));
        assert!(Property::Additive.satisfied_by(&zero_broken));
        let mut rng = Rng::new(SEED ^ 1);
        let add_broken = Interpretation::sample(&mut rng, Some(Property::Additive));
        assert!(Property::ZeroPreserving.satisfied_by(&add_broken));
        assert!(Property::BoundedFanout.satisfied_by(&add_broken));
    }

    /// NON-TRIVIALITY: no sample is the constant-zero map — some grid row maps to a
    /// non-empty image, so "the law held" is never survival-by-annihilation.
    #[test]
    fn no_sample_is_the_trivial_interpretation() {
        let mut rng = Rng::new(SEED);
        for _ in 0..SAMPLES {
            let f = Interpretation::sample(&mut rng, None);
            assert!(
                (0..2u8).any(|r| f.apply(&ZSet::of(&[(Row::new(r), 1)])) != ZSet::empty()),
                "a sampled interpretation must move something"
            );
        }
    }

    /// THE ACCEPTANCE PARTITION, pinned lib-side (the mutation sweeps only see lib
    /// tests): the license is warranted, the three real constraints are load-bearing
    /// with EVERY counter-sample refuting, and the planted decoration refutes nowhere.
    #[test]
    fn the_partition_is_exactly_three_load_bearing_and_the_planted_decoration() {
        let w = Warrant::derive();
        assert!(w.held, "{}", w.held_detail);
        let verdicts: Vec<(&str, usize)> = w
            .rows
            .iter()
            .map(|r| (r.property.name(), r.refuted))
            .collect();
        assert_eq!(
            verdicts,
            vec![
                ("additive", SAMPLES),
                ("zero-preserving", SAMPLES),
                ("deterministic", SAMPLES),
                ("bounded-fanout", 0),
            ],
            "the removal drill's partition moved — a property changed its weight"
        );
    }

    /// The derivation is DETERMINISTIC end to end — same seed, same interpretations,
    /// same bytes — which is what makes the warrant lockable at all.
    #[test]
    fn the_warrant_derives_the_same_bytes_every_time() {
        assert_eq!(Warrant::derive().render(), Warrant::derive().render());
    }

    /// The committed warrant is FRESH, lib-side.
    #[test]
    fn the_committed_warrant_is_fresh_from_the_library_side() {
        let spec_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("spec");
        let lock = Warrant::derive().lock_in(&spec_dir);
        if let Err(stale) = spec_lock::check(std::slice::from_ref(&lock)) {
            panic!(
                "the enrich warrant drifted: {}. Regenerate \
                 (`cargo run -p delta-render --example freeze`) and ratify the diff.",
                stale.join(", ")
            );
        }
    }
}
