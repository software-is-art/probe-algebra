//! Tier: KERNEL — the trusted floor — defines/runs the format, exempt from the structural rules.
//!
//! expect — DECLARED EXPECTATIONS: the module-level unit of top-down construction.
//!
//! Everything else in `discover` works bottom-up: run the operators, keep the laws that hold,
//! freeze them, gate the drift. This module adds the missing top-down half. A theory DECLARES
//! the laws it intends — drawn from the ratified shape catalog (`engine::ShapeCatalog`), named
//! by shape and by the operator symbols the shape ranges over — and [`Distance`] reports how far
//! the implementation is from the declaration. The workflow it exists for:
//!
//!   1. **declare** — write the `expects { ... }` clause (or `#[algebra(..., expects(...))]`)
//!      before the operators earn it: the intended algebra, as a checkable artifact;
//!   2. **red distance** — `Distance::of::<T>()` is not met; [`Distance::render`] names exactly
//!      what is missing ("MISSING: identity(grant, zero), bias_later(renew)") — a report an
//!      agent can work from, not a bare boolean;
//!   3. **implement** — until each declared law is discovered true over the grid, the gate
//!      stays red with the shortfall named;
//!   4. **green** — [`Distance::is_met`] holds: every declared law is discovered. SURPRISES
//!      (discovered, never declared) are PROMPTS, not failures — ratify them into the
//!      declaration or refute the operator that produced them;
//!   5. **the drift gate takes over** — from here the frozen spec discipline of
//!      `docs/ci-discipline.md` (freeze the spec, gate the drift, mutate the diff) keeps the
//!      earned algebra from regressing. Expectations get you TO the lock; the lock keeps you there.
//!
//! The vocabulary is deliberately closed: an expectation can only name a shape the catalog has
//! ratified (`spec/shapes.spec`), so "what may be declared" and "what discovery can find" are
//! the same language — a declaration outside it fails loudly, by name.

use crate::discover::engine::{DiscoveredLaw, Engine, Theory};

/// The declaration vocabulary: the identifier a theory declares a shape by, next to the
/// catalog name the discovery engine tags laws with. One row per `ShapeCatalog::inventory()`
/// entry, in inventory order — the census test below holds the two in lockstep, so a shape
/// cannot be ratified without becoming declarable (nor declared without being ratified).
const VOCABULARY: &[(&str, &str)] = &[
    ("commutative", "commutativity"),
    ("associative", "associativity"),
    ("idempotent", "idempotence"),
    ("bias_later", "bias (right-regular)"),
    ("bias_earlier", "bias (left-regular)"),
    ("identity", "identity"),
    ("annihilation", "annihilation"),
    ("inverse", "inverse"),
    ("distributive", "distributivity"),
    ("distributive_right", "distributivity (right)"),
    ("absorption", "absorption"),
    ("action_identity", "action identity"),
    ("monoid_action", "monoid action"),
    ("irreflexive", "irreflexivity"),
    ("self_application", "self-application"),
    ("involution", "involution"),
    ("fixed_point", "fixed point"),
    ("round_trip", "round-trip"),
    ("homomorphism", "homomorphism"),
    ("nontrivial", "action nontriviality"),
    ("not_constantly", "non-constancy"),
];

/// One declared law: a ratified catalog shape plus the operator symbols it ranges over —
/// `identity` over `[grant, zero]`, `bias_later` over `[renew]`. This is the same identity a
/// [`DiscoveredLaw`] carries (its `shape` tag and its `ops` fingerprint), which is what makes
/// declared and discovered directly comparable.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Expectation {
    /// The catalog shape name — exactly a `ShapeCatalog::inventory()` name.
    pub shape: &'static str,
    /// The distinct operator symbols the shape ranges over, in the law's first-appearance
    /// order (the operator first, then its constant/partner: `[grant, zero]`, `[esc, ++]`).
    /// Owned strings, not `&'static`: a declaration can be PARSED at runtime (genesis reads
    /// `system!` declarations from tokens) as well as written in source.
    pub ops: Vec<String>,
}

impl Expectation {
    /// Build an expectation from a declaration key (`"commutative"`, `"bias_later"`, ...) or an
    /// exact catalog name (`"bias (right-regular)"`). An unknown name FAILS LOUDLY, listing the
    /// whole vocabulary — a declaration outside the ratified catalog is a spelling of a law the
    /// engine can never discover, so it must never silently count as "missing".
    pub fn of<S: Into<String>>(shape: &'static str, ops: Vec<S>) -> Expectation {
        let canonical = VOCABULARY
            .iter()
            .find(|(key, name)| *key == shape || *name == shape)
            .map(|(_, name)| *name)
            .unwrap_or_else(|| {
                panic!(
                    "expectation names unknown shape {:?} — not in the ratified catalog \
                     (spec/shapes.spec). Declarable shapes: {}",
                    shape,
                    VOCABULARY
                        .iter()
                        .map(|(key, _)| *key)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            });
        Expectation {
            shape: canonical,
            ops: dedup(ops.into_iter().map(S::into).collect()),
        }
    }

    /// The canonical catalog name for a declaration key (or an exact catalog name) — the
    /// NON-PANICKING twin of [`Expectation::of`], for parsers (genesis) that must refuse an
    /// unknown shape gracefully rather than abort.
    pub fn canonical(shape: &str) -> Option<&'static str> {
        VOCABULARY
            .iter()
            .find(|(key, name)| *key == shape || *name == shape)
            .map(|(_, name)| *name)
    }

    /// The declaration keys, in catalog order — what a parser lists when refusing an unknown
    /// shape, so the refusal teaches the whole vocabulary.
    pub fn vocabulary_keys() -> Vec<&'static str> {
        VOCABULARY.iter().map(|(key, _)| *key).collect()
    }

    /// The declaration key for this expectation's shape — the identifier it renders under in a
    /// distance report, so the report reads back in the same vocabulary the agent declares in.
    fn key(&self) -> &'static str {
        VOCABULARY
            .iter()
            .find(|(_, name)| *name == self.shape)
            .map(|(key, _)| *key)
            .unwrap_or(self.shape)
    }

    /// `identity(grant, zero)` — the declaration form, which is also the report form (and
    /// what genesis passes through AS WRITTEN into a generated `expects(...)` attribute).
    pub fn render(&self) -> String {
        format!("{}({})", self.key(), self.ops.join(", "))
    }
}

/// A theory that DECLARES its intended laws. Generated by `theory!`'s optional `expects { ... }`
/// clause and by `#[algebra(..., expects(...))]`; implement it by hand only when neither
/// authoring path fits.
pub trait Expected {
    /// The declared laws, in declaration order.
    fn expectations() -> Vec<Expectation>;
}

/// The DISTANCE between a theory's declaration and its discovered algebra — the three-way
/// gate's verdict: met, missing (declared, not yet earned), and surprising (discovered, never
/// declared). Missing is the red/green axis; surprises are prompts.
pub struct Distance {
    /// The theory's display name, for the report header.
    pub theory: &'static str,
    /// How many laws were declared.
    pub declared: usize,
    /// Declared laws discovery did NOT find true — the implementation has not earned them yet
    /// (or the declaration overshot and should be retracted). Non-empty means the gate is red.
    pub missing: Vec<Expectation>,
    /// Discovered laws no declaration names — true of the operators today, but nobody said
    /// they were intended. Ratify each into the declaration or refute the behaviour; they do
    /// not redden the gate.
    pub surprises: Vec<Expectation>,
}

impl Distance {
    /// Compare a theory's declaration against what discovery actually finds: run the engine,
    /// and match declared `(shape, ops)` pairs against each discovered law's shape tag and
    /// operator fingerprint.
    pub fn of<T: Theory + Expected>() -> Distance {
        let engine = Engine::<T>::new();
        let symbols: Vec<&'static str> = engine
            .signatures()
            .into_iter()
            .map(|(symbol, _, _)| symbol)
            .collect();
        let found: Vec<Expectation> = engine
            .discover()
            .laws
            .iter()
            .map(|law: &DiscoveredLaw| Expectation {
                shape: law.shape,
                ops: law.ops(&symbols).into_iter().map(String::from).collect(),
            })
            .collect();
        let declared = T::expectations();

        let missing: Vec<Expectation> = declared
            .iter()
            .filter(|e| !found.contains(e))
            .cloned()
            .collect();
        let mut surprises: Vec<Expectation> = Vec::new();
        for law in found {
            if !declared.contains(&law) && !surprises.contains(&law) {
                surprises.push(law);
            }
        }

        Distance {
            theory: T::name(),
            declared: declared.len(),
            missing,
            surprises,
        }
    }

    /// The red/green gate: every declared law is discovered true. Surprises do NOT fail this —
    /// they are prompts to ratify or refute, reported by [`Distance::render`].
    pub fn is_met(&self) -> bool {
        self.missing.is_empty()
    }

    /// The report an agent reads — the whole distance in one line, in the declaration
    /// vocabulary:
    ///
    /// ```text
    /// credit meter: 7 of 9 declared laws hold; MISSING: identity(grant, zero), \
    /// bias_later(renew); SURPRISES (discovered, never declared — ratify or refute): \
    /// annihilation(spend, zero)
    /// ```
    pub fn render(&self) -> String {
        let held = self.declared - self.missing.len();
        let mut out = format!(
            "{}: {} of {} declared laws hold",
            self.theory, held, self.declared
        );
        if self.missing.is_empty() && self.surprises.is_empty() {
            out.push_str("; no surprises");
            return out;
        }
        if !self.missing.is_empty() {
            out.push_str("; MISSING: ");
            out.push_str(&render_list(&self.missing));
        }
        if !self.surprises.is_empty() {
            out.push_str("; SURPRISES (discovered, never declared — ratify or refute): ");
            out.push_str(&render_list(&self.surprises));
        }
        out
    }
}

fn render_list(expectations: &[Expectation]) -> String {
    expectations
        .iter()
        .map(Expectation::render)
        .collect::<Vec<_>>()
        .join(", ")
}

/// Distinct symbols in first-appearance order — the same normalisation `DiscoveredLaw::ops`
/// applies, so a declaration like `homomorphism(esc, ++, ++)` (the shape's three parameter
/// slots, two coinciding) compares equal to the discovered law's fingerprint `[esc, ++]`.
fn dedup(ops: Vec<String>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for op in ops {
        if !out.contains(&op) {
            out.push(op);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discover::engine::ShapeCatalog;

    /// The declaration vocabulary and the shape catalog are the SAME set, in the same order —
    /// a shape cannot be ratified without becoming declarable, nor declared without being
    /// ratified. This is the expect-side census, the same lockstep discipline `templates()` and
    /// `ShapeCatalog::inventory()` hold each other to.
    #[test]
    fn the_declaration_vocabulary_is_the_catalog_in_lockstep() {
        let catalog: Vec<&str> = ShapeCatalog::inventory().iter().map(|s| s.name).collect();
        let vocabulary: Vec<&str> = VOCABULARY.iter().map(|(_, name)| *name).collect();
        assert_eq!(
            vocabulary, catalog,
            "VOCABULARY and ShapeCatalog::inventory() diverged — they must move together"
        );
    }

    /// An unknown shape name fails LOUDLY at construction — a declaration the catalog never
    /// ratified must never silently sit in `missing` looking like unfinished work.
    #[test]
    #[should_panic(expected = "unknown shape \"transitive\"")]
    fn an_unratified_shape_name_is_refused_by_name() {
        Expectation::of("transitive", vec!["<"]);
    }

    /// Both spellings construct the same expectation: the declaration key (`bias_later`) and
    /// the exact catalog name (`bias (right-regular)`) — and it renders back in key form.
    #[test]
    fn keys_and_catalog_names_are_interchangeable_and_render_as_keys() {
        let by_key = Expectation::of("bias_later", vec!["renew"]);
        let by_name = Expectation::of("bias (right-regular)", vec!["renew"]);
        assert_eq!(by_key, by_name);
        assert_eq!(by_key.render(), "bias_later(renew)");
    }

    // -- the RED demo: a declaration that deliberately overshoots -----------------------------
    //
    // One tiny theory ({zero, max} over 0..=2), two marker types over the same operators:
    // `Overreach` declares more than the operators deliver (and omits a law they do), so its
    // distance report is the agent-facing artifact this module ships — pinned EXACTLY below.
    // `Earned` declares precisely the discovered algebra: the green gate, met with no surprises.

    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
    struct N;
    #[derive(Clone)]
    struct V(u8);
    struct Overreach;
    struct Earned;
    fn zero_op(_: &[V]) -> Option<V> {
        Some(V(0))
    }
    fn max_op(v: &[V]) -> Option<V> {
        Some(V(v[0].0.max(v[1].0)))
    }

    crate::theory! {
        Overreach : "overreach",
        Value = V,
        Obs = u8,
        Sort = N,
        sort_of = |_: &V| N,
        observe = |v: &V| v.0,
        vars { N => &["x", "y", "z"], }
        inhabit { N => (0..3).map(V).collect(), }
        ops {
            Nullary "zero" "zero" () -> N = zero_op;
            Infix   "max"  "max"  (N, N) -> N = max_op;
        }
        expects {
            commutative(max);
            associative(max);
            identity(max, zero);
            annihilation(max, zero);
            bias_later(max);
        }
    }

    crate::theory! {
        Earned : "earned",
        Value = V,
        Obs = u8,
        Sort = N,
        sort_of = |_: &V| N,
        observe = |v: &V| v.0,
        vars { N => &["x", "y", "z"], }
        inhabit { N => (0..3).map(V).collect(), }
        ops {
            Nullary "zero" "zero" () -> N = zero_op;
            Infix   "max"  "max"  (N, N) -> N = max_op;
        }
        expects {
            commutative(max);
            associative(max);
            idempotent(max);
            identity(max, zero);
        }
    }

    /// THE RED DEMO, with the report text pinned exactly — the rendering IS the product: it is
    /// what an agent reads to know what to implement next. `max` has no annihilator (zero is
    /// its identity, and there is no declared top) and no bias (it is commutative), so those
    /// two declarations are MISSING; its idempotence is real but was never declared, so it is
    /// a SURPRISE — a prompt to ratify, not a failure.
    #[test]
    fn an_overshooting_declaration_reads_back_as_a_named_distance() {
        let distance = Distance::of::<Overreach>();
        assert!(!distance.is_met(), "the overshoot must leave the gate red");
        assert_eq!(
            distance.render(),
            "overreach: 3 of 5 declared laws hold; \
             MISSING: annihilation(max, zero), bias_later(max); \
             SURPRISES (discovered, never declared — ratify or refute): idempotent(max)"
        );
    }

    /// THE GREEN GATE: a declaration that matches the discovered algebra exactly is met, with
    /// no surprises — and says so in the same pinned voice.
    #[test]
    fn an_earned_declaration_is_met_with_no_surprises() {
        let distance = Distance::of::<Earned>();
        assert!(distance.is_met(), "report: {}", distance.render());
        assert_eq!(
            distance.render(),
            "earned: 4 of 4 declared laws hold; no surprises"
        );
    }

    // The remaining quadrant: MET but with surprises. `Undersold` declares one true law and
    // stays silent about the rest, so the gate is green while the report still owes prompts.
    struct Undersold;

    crate::theory! {
        Undersold : "undersold",
        Value = V,
        Obs = u8,
        Sort = N,
        sort_of = |_: &V| N,
        observe = |v: &V| v.0,
        vars { N => &["x", "y", "z"], }
        inhabit { N => (0..3).map(V).collect(), }
        ops {
            Nullary "zero" "zero" () -> N = zero_op;
            Infix   "max"  "max"  (N, N) -> N = max_op;
        }
        expects {
            commutative(max);
        }
    }

    /// A met declaration with surprises must still SAY the surprises — "no surprises" is only
    /// sayable when BOTH lists are empty, never when merely one is (the early-return's `&&` is
    /// the whole claim).
    #[test]
    fn a_met_declaration_with_surprises_still_owes_its_prompts() {
        let distance = Distance::of::<Undersold>();
        assert!(distance.is_met(), "report: {}", distance.render());
        let report = distance.render();
        assert!(
            !report.contains("no surprises"),
            "a green gate must not swallow its surprises: {report}"
        );
        assert!(
            report.starts_with("undersold: 1 of 1 declared laws hold; SURPRISES"),
            "the surprises must be reported in the pinned voice: {report}"
        );
    }

    // The MINIMAL `theory!` form delegates to the derived-grid form; its `expects` clause must
    // survive the delegation. `bool` is `Shaped`, so the whole theory is two lines of meaning.
    struct Fused;
    fn xor_op(v: &[bool]) -> Option<bool> {
        Some(v[0] ^ v[1])
    }

    crate::theory! {
        Fused : "fused",
        Value = bool,
        Sort = N,
        sort_of = |_: &bool| N,
        ops {
            Infix "xor" "xor" (N, N) -> N = xor_op;
        }
        expects {
            commutative(xor);
            associative(xor);
        }
    }

    /// The minimal `theory!` form THREADS `expects` through its delegation — the passthrough is
    /// grammar, so it gets its own pin: drop the forwarded clause and this stops compiling.
    #[test]
    fn the_minimal_theory_form_threads_expects_through() {
        let distance = Distance::of::<Fused>();
        assert!(distance.is_met(), "report: {}", distance.render());
        assert_eq!(
            distance.render(),
            "fused: 2 of 2 declared laws hold; no surprises"
        );
    }
}
