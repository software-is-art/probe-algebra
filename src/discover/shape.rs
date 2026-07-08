//!
//! shape — the PLACER: module boundaries derived from net connectivity, continuously.
//!
//! Circuit CAD dissolved the write-behaviour-and-decide-shape-simultaneously friction
//! decades ago: a designer writes the NETLIST (behaviour and connectivity), place-and-route
//! derives the geometry, and design-rule checks gate that the derived layout still matches.
//! Nobody hand-places transistors. The algebraic twin of a net is a SORT: two operators are
//! connected exactly when a sort appears in both signatures — one can produce what the other
//! consumes. So the placer partitions a theory's operators by net connectivity, and the
//! partition is the DERIVED module structure:
//!
//!   - one component  → the module is SETTLED: the declared boundary is the derived boundary;
//!   - several        → the components share NOTHING — not a value, not a wire. That split is
//!     indisputable, and the fix is to move code, never to pin the report.
//!
//! This is deliberately a DIFFERENT instrument from [`super::cohesion`], and the two answer
//! different questions. Cohesion links operators by LAW co-occurrence — the wiring density
//! WITHIN a module — and it reads a protocol's one-way doors as split points, which is why
//! its latent splits are suggestions a human ratifies (this repo keeps three modules whole
//! against its advice). Placement links operators by NETS, and it derives those keep-whole
//! decisions instead of asking for them: `submit`/`approve`/`edit` share the protocol's
//! state sorts, `<` shares `Int` with `+`, `since` shares `Date` with `add` — settled, no
//! ratification spent. Falling back to a pin is kicking the can; the placer's job is to
//! agree with a well-drawn boundary NATIVELY and to flag only the splits nothing connects.
//!
//! CONTINUOUS is the point: [`ShapeReport`] freezes to `spec/<system>.shape.spec` under the
//! same lock discipline as everything else, so the derived shape is recomputed on every test
//! run and re-derived on every freeze — an operator added tomorrow either lands in an
//! existing component (local, invisible in the lock beyond its line) or changes the shape
//! (a diff to ratify). Placement is MONOTONE by construction: a new operator can join a
//! component or bridge two into one, but it can never re-split or reshuffle operators it
//! does not touch — the locality property the layout-probe census demands of any layout
//! engine, held here by union-find's own algebra.
//!
//! One disclosure, in the honest frame: nets are compared by NAME within one theory's sort
//! type, where name equality is identity. ACROSS modules a shared net name is only a
//! coincidence detector — `Duration` in the calendar and `Duration` in the ttl store are two
//! Rust types that happen to agree on a word. The report surfaces such coincidences as SEAM
//! CANDIDATES (declare the seam, or leave the name-sharing standing as coincidence); it
//! never merges modules over them.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use spec_lock::Lock;

use super::engine::{Engine, Theory};
use super::system::System;

/// One derived module: the operators net-connectivity groups together, and the nets they
/// share. Components of a single placement are pairwise net-disjoint by construction.
pub struct Component {
    /// The member operators' symbols, in declaration order.
    pub ops: Vec<&'static str>,
    /// The nets (rendered sort names) the members touch, sorted.
    pub nets: Vec<String>,
}

#[crate::mutate]
impl Component {
    /// The component on one line: `{ ops } over nets { nets }`.
    fn render(&self) -> String {
        format!(
            "{{ {} }} over nets {{ {} }}",
            self.ops.join(", "),
            self.nets.join(", ")
        )
    }
}

/// The placement of one theory (or one hand-written bundle of signatures): its operators
/// partitioned by net connectivity. The agent-facing flow is to write a BIG BUNDLE — one
/// theory, no structure decisions — and read the derived modules off this report.
pub struct Placement {
    pub theory: &'static str,
    /// The derived modules, ordered by each component's first operator.
    pub components: Vec<Component>,
}

/// A raw signature the placer reads: `(symbol, input nets, output net)`. The theory-facing
/// path ([`Placement::of`]) renders these from `Engine::signatures`; the raw form exists so
/// a bundle can be placed from its declaration alone — the placer never runs discovery, so
/// shape is derived before a single law is judged.
pub type NetSignature = (&'static str, Vec<String>, String);

/// Union-find: the root of `x`, with path compression.
#[crate::mutate]
fn find(parent: &mut [usize], mut x: usize) -> usize {
    while parent[x] != x {
        parent[x] = parent[parent[x]];
        x = parent[x];
    }
    x
}

#[crate::mutate]
impl Placement {
    /// Is the boundary settled — one component (or none), nothing indisputably apart?
    pub fn is_settled(&self) -> bool {
        self.components.len() <= 1
    }

    /// Place a theory: signatures in, derived modules out. Associated fn of the report per
    /// the no-rats-nest rule.
    pub fn of<T: Theory>() -> Placement {
        Placement::over(T::name(), Placement::signatures_of::<T>())
    }

    /// A compiled theory's raw net signatures — the rows [`Placement::of`] places,
    /// exposed as the TYPE PATH into everything that consumes signatures: a consumer
    /// that models its theory in code (no `ops { ... }` text to parse) feeds these to
    /// the live ticker (`Ticker::step_theory`) and gets the same layout sense.
    pub fn signatures_of<T: Theory>() -> Vec<NetSignature> {
        Engine::<T>::new()
            .signatures()
            .into_iter()
            .map(|(symbol, inputs, output)| {
                (
                    symbol,
                    inputs.iter().map(|s| format!("{s:?}")).collect(),
                    format!("{output:?}"),
                )
            })
            .collect()
    }

    /// Place raw signatures — the bundle form. Two operators land in one component exactly
    /// when a chain of shared nets connects them.
    pub fn over(theory: &'static str, sigs: Vec<NetSignature>) -> Placement {
        let n = sigs.len();
        let mut parent: Vec<usize> = (0..n).collect();
        // each net remembers the first operator that touched it; later toucher unions in.
        let mut net_rep: BTreeMap<&str, usize> = BTreeMap::new();
        for (i, (_, inputs, output)) in sigs.iter().enumerate() {
            for net in inputs.iter().chain(std::iter::once(output)) {
                match net_rep.get(net.as_str()) {
                    Some(&j) => {
                        let (a, b) = (find(&mut parent, i), find(&mut parent, j));
                        parent[a] = b;
                    }
                    None => {
                        net_rep.insert(net, i);
                    }
                }
            }
        }
        // group by root, components ordered by first member, members in declaration order.
        let roots: Vec<usize> = (0..n).map(|i| find(&mut parent, i)).collect();
        let mut order: Vec<usize> = Vec::new();
        for &r in &roots {
            if !order.contains(&r) {
                order.push(r);
            }
        }
        let components = order
            .iter()
            .map(|&r| {
                let members: Vec<usize> = (0..n).filter(|&i| roots[i] == r).collect();
                let mut nets = BTreeSet::new();
                for &i in &members {
                    nets.extend(sigs[i].1.iter().cloned());
                    nets.insert(sigs[i].2.clone());
                }
                Component {
                    ops: members.iter().map(|&i| sigs[i].0).collect(),
                    nets: nets.into_iter().collect(),
                }
            })
            .collect();
        Placement { theory, components }
    }

    /// The placement as a readable verdict — the bundle author's report.
    pub fn render(&self) -> String {
        let mut out = format!("placement `{}`: ", self.theory);
        if self.is_settled() {
            out.push_str(
                "settled — one component, the declared boundary is the derived boundary.\n",
            );
            for c in &self.components {
                out.push_str(&format!("  {}\n", c.render()));
            }
            return out;
        }
        out.push_str(&format!(
            "places as {} modules — the components share no nets, so the split is indisputable:\n",
            self.components.len()
        ));
        for (i, c) in self.components.iter().enumerate() {
            out.push_str(&format!("  module {}: {}\n", i, c.render()));
        }
        out
    }
}

/// A cross-module net-NAME coincidence with no declared seam: two modules whose sort names
/// overlap, joined by no obligation. A suggestion in the cohesion voice — declare the seam,
/// or leave the shared name standing as coincidence.
pub struct SeamCandidate {
    pub left: &'static str,
    pub right: &'static str,
    /// The coinciding net names, sorted.
    pub nets: Vec<String>,
}

/// The DERIVED shape of a whole system: every registry module's placement, plus the seam
/// candidates the net names volunteer. What the shape lock freezes — so the declared shape
/// is checked against the derived shape on every test run, continuously.
pub struct ShapeReport {
    pub system: &'static str,
    pub placements: Vec<Placement>,
    pub candidates: Vec<SeamCandidate>,
}

#[crate::mutate]
impl ShapeReport {
    /// Compute a system's derived shape: place every registry module, then scan module
    /// pairs for net-name coincidences no declared seam covers.
    pub fn of<S: System>() -> ShapeReport {
        let placements = S::placements();
        // a seam declaration carries no orientation, so BOTH orientations seal the pair —
        // storing the pair twice instead of normalizing it removes a whole degree of
        // freedom (a normalizer applied on the write side and the read side alike is
        // invisible to any consistent mutation of itself).
        let mut sealed: BTreeSet<(&str, &str)> = BTreeSet::new();
        for seam in S::seams() {
            sealed.insert((seam.left, seam.right));
            sealed.insert((seam.right, seam.left));
        }
        let nets: Vec<BTreeSet<&String>> = placements
            .iter()
            .map(|p| p.components.iter().flat_map(|c| &c.nets).collect())
            .collect();
        let mut candidates = Vec::new();
        for i in 0..placements.len() {
            for j in (i + 1)..placements.len() {
                let shared: Vec<String> = nets[i]
                    .intersection(&nets[j])
                    .map(|s| s.to_string())
                    .collect();
                if shared.is_empty()
                    || sealed.contains(&(placements[i].theory, placements[j].theory))
                {
                    continue;
                }
                candidates.push(SeamCandidate {
                    left: placements[i].theory,
                    right: placements[j].theory,
                    nets: shared,
                });
            }
        }
        ShapeReport {
            system: S::name(),
            placements,
            candidates,
        }
    }

    /// Is the declared shape a fixed point of the placer — every module settled?
    pub fn is_settled(&self) -> bool {
        self.placements.iter().all(Placement::is_settled)
    }

    /// The canonical text of the derived shape — deterministic, diffable, the lock body.
    pub fn render(&self) -> String {
        let mut out = format!(
            "# shape spec: {} — the DERIVED shape (operators placed by net connectivity); regenerate via this repo's freeze path and ratify the diff.\n\n",
            self.system
        );
        out.push_str(
            "Two operators share a net when a sort appears in both signatures (the circuit-CAD\n\
             signal: nets, not laws). A settled module is one whose declared boundary the placer\n\
             re-derives; a module placing as several holds operators that share NOTHING — move\n\
             the code, never pin the report.\n\n",
        );
        for p in &self.placements {
            if p.is_settled() {
                for c in &p.components {
                    out.push_str(&format!("- {}: settled — {}\n", p.theory, c.render()));
                }
            } else {
                out.push_str(&format!(
                    "- {}: PLACES AS {} — the components share no nets (an indisputable split):\n",
                    p.theory,
                    p.components.len()
                ));
                for (i, c) in p.components.iter().enumerate() {
                    out.push_str(&format!("      module {}: {}\n", i, c.render()));
                }
            }
        }
        let settled = self.placements.iter().filter(|p| p.is_settled()).count();
        out.push_str(&format!(
            "\nverdict: {settled} of {} modules settled{}\n",
            self.placements.len(),
            if self.is_settled() {
                " — the declared shape is a fixed point of the placer."
            } else {
                " — the derived shape DISAGREES with the declaration."
            }
        ));
        if self.candidates.is_empty() {
            out.push_str(
                "\nseam candidates: none — no cross-module net-name coincidence is undeclared.\n",
            );
            return out;
        }
        out.push_str(
            "\nseam candidates (cross-module net-NAME coincidences no declared seam covers — a\n\
             suggestion: declare the seam, or leave the shared name standing as coincidence):\n",
        );
        for c in &self.candidates {
            out.push_str(&format!(
                "- {} ↔ {} on {}\n",
                c.left,
                c.right,
                c.nets.join(", ")
            ));
        }
        out
    }

    /// This report as a `spec_lock::Lock` in a caller-supplied spec directory — the
    /// consumer-facing form, sibling of `SystemReport::lock_in`. The lock lands at
    /// `spec_dir/<slugified-system>.shape.spec`.
    pub fn lock_in(&self, spec_dir: &Path) -> Lock {
        let slug: String = self
            .system
            .chars()
            .map(|c| if c == ' ' { '-' } else { c })
            .collect();
        Lock {
            name: format!("{} shape", self.system),
            path: spec_dir.join(format!("{slug}.shape.spec")),
            live: self.render(),
        }
    }

    /// This report as a `spec_lock::Lock` in THIS repo's `spec/` directory (downstream
    /// crates use [`ShapeReport::lock_in`] with their own spec directory).
    pub fn lock(&self) -> Lock {
        self.lock_in(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("spec"))
    }
}

#[cfg(test)]
mod probes {
    use super::*;
    use crate::discover::arithmetic::Arithmetic;
    use crate::discover::coherence::{GcdMerge, MaxMerge};
    use crate::discover::cohesion::CohesionReport;
    use crate::discover::date::Calendar;
    use crate::discover::protocol::DocFlow;
    use crate::discover::BoundarySpec;

    fn sig(symbol: &'static str, inputs: &[&str], output: &str) -> NetSignature {
        (
            symbol,
            inputs.iter().map(|s| s.to_string()).collect(),
            output.to_string(),
        )
    }

    /// Three features written as ONE BUNDLE — the agent's no-structure-decisions style —
    /// and the placer derives the three modules from the signatures alone (no discovery
    /// run, no laws consulted). Three features, not two, pin the grouping: with three
    /// parts a flipped membership test cannot masquerade as a swapped order.
    fn bundle() -> Vec<NetSignature> {
        vec![
            sig("zA", &[], "A"),
            sig("mA", &["A", "A"], "A"),
            sig("zB", &[], "B"),
            sig("mB", &["B", "B"], "B"),
            sig("zC", &[], "C"),
            sig("mC", &["C", "C"], "C"),
        ]
    }

    #[test]
    fn a_bundle_of_disjoint_features_places_apart() {
        let p = Placement::over("bundle", bundle());
        assert_eq!(p.components.len(), 3, "three net-disjoint features");
        assert!(!p.is_settled());
        // the degenerate boundary: no operators is settled (nothing is apart).
        assert!(Placement::over("empty", vec![]).is_settled());
        let total: usize = p.components.iter().map(|c| c.ops.len()).sum();
        assert_eq!(total, 6, "each operator in exactly one component");
        assert_eq!(
            p.render(),
            "placement `bundle`: places as 3 modules — the components share no nets, so the split is indisputable:\n\
             \x20 module 0: { zA, mA } over nets { A }\n\
             \x20 module 1: { zB, mB } over nets { B }\n\
             \x20 module 2: { zC, mC } over nets { C }\n"
        );
    }

    /// MONOTONE, the locality property the layout-probe census demands of any layout
    /// engine: a new operator can join a component or bridge two into one, but it can
    /// never re-split or reshuffle operators it does not touch. The conversion `aToB`
    /// merges features A and B; feature C's component is byte-identical.
    #[test]
    fn a_conversion_bridges_and_the_rest_stay_put() {
        let mut sigs = bundle();
        sigs.push(sig("aToB", &["A"], "B"));
        let p = Placement::over("bundle", sigs);
        assert_eq!(p.components.len(), 2, "A and B merge; C stands");
        let merged = &p.components[0];
        assert_eq!(merged.ops, vec!["zA", "mA", "zB", "mB", "aToB"]);
        assert_eq!(merged.nets, vec!["A".to_string(), "B".to_string()]);
        let untouched = &p.components[1];
        assert_eq!(untouched.ops, vec!["zC", "mC"]);
        assert_eq!(untouched.nets, vec!["C".to_string()]);
    }

    /// THE DOGFOOD, and the reason a pin is no longer spent: the three modules this repo
    /// keeps whole AGAINST the cohesion instrument's advice (see
    /// `system::tests::the_repo_distance_names_the_latent_splits`) are all SETTLED to the
    /// placer — nets connect what laws do not. `<` shares `Int` with `+`; `since` shares
    /// `Date` with `add`; the protocol's transitions share its state sorts. Two
    /// instruments, two questions: cohesion reads the wiring density (a suggestion),
    /// placement reads the boundary (a derivation).
    #[test]
    fn nets_place_what_laws_cannot_see() {
        for (split, settled) in [
            (
                CohesionReport::of::<Arithmetic>().components.len(),
                Placement::of::<Arithmetic>(),
            ),
            (
                CohesionReport::of::<Calendar>().components.len(),
                Placement::of::<Calendar>(),
            ),
            (
                CohesionReport::of::<DocFlow>().components.len(),
                Placement::of::<DocFlow>(),
            ),
        ] {
            assert!(
                split > 1,
                "cohesion suggests a split for {}",
                settled.theory
            );
            assert!(
                settled.is_settled(),
                "placement derives keep-whole for {}",
                settled.theory
            );
        }
    }

    /// The settled render carries the boundary-agreement sentence — the one line a bundle
    /// author reads.
    #[test]
    fn a_settled_placement_renders_the_agreement() {
        assert_eq!(
            Placement::of::<DocFlow>().render(),
            "placement `doc flow`: settled — one component, the declared boundary is the derived boundary.\n\
             \x20 { submit, revise, approve, edit } over nets { Draft, Published, Review }\n"
        );
    }

    /// THE WHOLE SYSTEM'S DERIVED SHAPE, byte-pinned: every declared module settled (the
    /// declared shape is a fixed point of the placer — the hand shaping re-derived, not
    /// ratified), and the one cross-module net-name coincidence surfaced as a seam
    /// candidate: `Duration` names a net in both the calendar and the ttl store, and no
    /// seam is declared. A suggestion in the honest frame — the two are different Rust
    /// types that agree on a word; the report will neither merge them nor stay silent.
    #[test]
    fn the_derived_shape_is_the_declared_shape() {
        let report = ShapeReport::of::<BoundarySpec>();
        assert!(report.is_settled());
        let expected = "\
# shape spec: boundary-spec — the DERIVED shape (operators placed by net connectivity); regenerate via this repo's freeze path and ratify the diff.

Two operators share a net when a sort appears in both signatures (the circuit-CAD
signal: nets, not laws). A settled module is one whose declared boundary the placer
re-derives; a module placing as several holds operators that share NOTHING — move
the code, never pin the report.

- interpreter arithmetic: settled — { 0, 1, false, +, *, < } over nets { Bool, Int }
- router: settled — { empty, or } over nets { Router }
- date calculus: settled — { zero, +, add, diff, since, at } over nets { Date, Duration }
- ttl store: settled — { empty, <+, tick, zero, + } over nets { Duration, Store }
- store protocol: settled — { empty, ++ } over nets { P }
- doc flow: settled — { submit, revise, approve, edit } over nets { Draft, Published, Review }
- fabric: settled — { mesh, join, grant, revoke, reach, within, true } over nets { Fabric, Route, Verdict }

verdict: 7 of 7 modules settled — the declared shape is a fixed point of the placer.

seam candidates (cross-module net-NAME coincidences no declared seam covers — a
suggestion: declare the seam, or leave the shared name standing as coincidence):
- date calculus ↔ ttl store on Duration
";
        assert_eq!(report.render(), expected);
    }

    // Two systems over the SAME module pair (the merge theories share the `Key` net):
    // one declares the seam, one does not — the candidate must appear exactly when the
    // coincidence is undeclared.
    struct BareMerges;
    crate::system! {
        BareMerges : "bare merges",
        modules {
            MaxMerge;
            GcdMerge;
        }
    }
    struct SealedMerges;
    crate::system! {
        SealedMerges : "sealed merges",
        modules {
            MaxMerge;
            GcdMerge;
        }
        seams {
            // declared AGAINST registry order on purpose: the seam says Gcd -- Max while
            // the placements pair up as Max/Gcd, so retiring the candidate only works
            // because the seal covers BOTH orientations.
            GcdMerge -- MaxMerge : transport on Key;
        }
    }

    /// An undeclared net coincidence is a CANDIDATE; a declared seam retires it —
    /// regardless of the orientation it was declared in.
    #[test]
    fn a_declared_seam_retires_the_candidate() {
        let bare = ShapeReport::of::<BareMerges>();
        let [candidate] = bare.candidates.as_slice() else {
            panic!("one undeclared coincidence, got {}", bare.candidates.len());
        };
        assert_eq!(
            (candidate.left, candidate.right),
            ("max-merge", "gcd-merge")
        );
        assert_eq!(candidate.nets, vec!["Key".to_string()]);
        assert!(bare.render().contains("- max-merge ↔ gcd-merge on Key"));

        let sealed = ShapeReport::of::<SealedMerges>();
        assert!(sealed.candidates.is_empty(), "the declared seam covers it");
        assert!(sealed.render().contains(
            "seam candidates: none — no cross-module net-name coincidence is undeclared."
        ));
    }

    /// An unsettled module renders LOUD inside the system report — the disagreement
    /// verdict and the component lines a reviewer acts on (by moving code).
    #[test]
    fn a_disagreeing_shape_renders_loud() {
        let report = ShapeReport {
            system: "disagreeing",
            placements: vec![
                Placement::over("bundle", bundle()),
                Placement::of::<DocFlow>(),
            ],
            candidates: vec![],
        };
        assert!(!report.is_settled());
        let text = report.render();
        assert!(text.contains(
            "- bundle: PLACES AS 3 — the components share no nets (an indisputable split):"
        ));
        assert!(text.contains("      module 0: { zA, mA } over nets { A }"));
        assert!(text.contains(
            "verdict: 1 of 2 modules settled — the derived shape DISAGREES with the declaration."
        ));
    }

    /// The lock lands at `<spec_dir>/<slug>.shape.spec` — the `.shape.` infix keeping it
    /// apart from the module and system locks in the same directory.
    #[test]
    fn the_shape_lock_has_its_own_namespace() {
        let lock = ShapeReport::of::<BoundarySpec>().lock_in(Path::new("spec"));
        assert_eq!(lock.name, "boundary-spec shape");
        assert_eq!(
            lock.path,
            Path::new("spec").join("boundary-spec.shape.spec")
        );
        assert_eq!(lock.live, ShapeReport::of::<BoundarySpec>().render());
    }

    /// The committed SHAPE lock is FRESH: the live derived shape matches what was
    /// ratified — the CONTINUOUS half of autoshaping. An operator moved, added, or
    /// re-sorted drifts this file; regenerate with `cargo run --example freeze_spec` and
    /// ratify the diff.
    #[test]
    fn the_committed_shape_spec_is_fresh() {
        let lock = ShapeReport::of::<BoundarySpec>().lock();
        if let Err(stale) = spec_lock::check(std::slice::from_ref(&lock)) {
            panic!(
                "the derived shape drifted from the committed lock for: {}. \
                 Run `cargo run --example freeze_spec` and ratify the diff.",
                stale.join(", ")
            );
        }
    }
}
