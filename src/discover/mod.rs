//! discover — the laws WRITE (and READ) themselves, generically, over ANY boundary's algebra.
//!
//! The discovery pipeline lives in `engine`: a domain implements `engine::Theory` (its sorts,
//! operators, inhabitants, and an OBSERVATION on values), and the engine ENUMERATES terms over the
//! signature, groups them by behaviour on a grid, instantiates the universal algebraic shapes over
//! the operators, and keeps the ones that run true — rendered as a non-mathy, ratifiable spec. The
//! arithmetic that was hardcoded through v6 is now just one `Theory` (`arithmetic::Arithmetic`),
//! backed by the real interpreter, so its discovered laws still probe the interpreter's interior.
//!
//! On top of the value algebra the interpreter adds one STRUCTURAL law over a synthetic universal
//! observer `U` — the faithful rendering: `eval` collapses structure (`2+3` and `5` are equal to
//! it), so a transform that computes the right value but mangles the tree is invisible to the
//! equations; the U law (`U` distinguishes every structural/semantic perturbation) closes that.
//!
//! Honest frame unchanged: discovery's oracle is the baseline, so its power is deviation-catching
//! (mutation) plus expectation-checking (ratification); enumeration is depth- and grid-bounded (a
//! resource limit, not a curated list).

pub mod arithmetic;
pub mod engine;

use crate::boundary::sensitive_to_all;
use crate::discover::arithmetic::Arithmetic;
use crate::discover::engine::Engine;
use crate::interp::boundary::{Expr, Ident, Op};

/// A discovered law in rendered form — a plain-language sentence and its symbolic equation. This is
/// the readable, ratifiable contract (and what the freeze records).
pub struct Law {
    pub prose: String,
    pub equation: String,
}

impl Law {
    pub fn prose(&self) -> &str {
        &self.prose
    }
    pub fn equation(&self) -> &str {
        &self.equation
    }
}

/// A theory's discovered spec: the named laws, the count of further (consequence) equalities, and
/// the operators that appear in no named law (where the spec is silent).
pub struct Spec {
    pub laws: Vec<Law>,
    pub consequences: usize,
    pub uncovered_ops: Vec<&'static str>,
}

/// The interpreter's discovered spec: the named value-algebra laws (from the generic engine over
/// the `Arithmetic` theory) plus the structural `U` law. The author supplied only the operators;
/// everything here was found by running them, not declared.
pub fn interpreter_spec() -> Spec {
    let engine = Engine::<Arithmetic>::new();
    let discovered = engine.discover();
    let mut laws: Vec<Law> = discovered
        .laws
        .iter()
        .map(|l| Law {
            prose: l.prose.clone(),
            equation: l.equation.clone(),
        })
        .collect();

    if observer_is_sensitive(render()) {
        laws.push(Law {
            prose: "No two distinct programs look the same — the faithful rendering distinguishes \
                    every structural and semantic difference."
                .to_string(),
            equation: "U(p) = U(q)  ⟹  p = q   (U = faithful render)".to_string(),
        });
    }

    Spec {
        laws,
        consequences: discovered.consequences,
        uncovered_ops: discovered.uncovered_ops,
    }
}

/// The interpreter's discovered laws (named value-algebra laws + the `U` law), for consumers that
/// only need the law list (the example, the harness probe, the freeze).
pub fn discover_laws() -> Vec<Law> {
    interpreter_spec().laws
}

/// Re-probe the interpreter's named value-algebra laws over the engine's grid — a supplementary,
/// oracle-free check that the discovered algebra still holds (the interpreter's per-arm correctness
/// is pinned independently by the harness's `eval_semantics_are_probed`). Returns the first failure.
pub fn replay_interpreter_laws() -> Result<(), String> {
    Engine::<Arithmetic>::new().replay()
}

// ----- the universal observer U: structure + semantics sensitivity --------

fn var(name: &str) -> Expr {
    Expr::var(Ident::new(name).expect("valid identifier"))
}
fn lit(n: i64) -> Expr {
    Expr::int(n).expect("non-negative literal")
}

/// Does the observer `obs` distinguish every structural and semantic perturbation of every sampled
/// program? Parameterized over the observer so the probe is exercised both ways: the faithful
/// rendering passes (it is the synthetic universal observer `U`), a collapsing one (`node_count`)
/// fails — which is what makes the discovered U law load-bearing rather than vacuous.
fn observer_is_sensitive<Y: PartialEq>(obs: impl Fn(&Expr) -> Y + Copy) -> bool {
    sample_programs().iter().all(|e| sensitive_to_all(obs, e))
}

/// The universal observer `U` — the faithful rendering. Named for the call site that gates the law.
fn render() -> impl Fn(&Expr) -> String + Copy {
    |x: &Expr| x.render()
}

/// A deterministic spread of programs that exercises every `Expr` constructor and dimension.
fn sample_programs() -> Vec<Expr> {
    let x = || var("x");
    vec![
        lit(0),
        lit(7),
        Expr::boolean(true),
        x(),
        Expr::bin(Op::Add, lit(2), lit(3)),
        Expr::bin(Op::Mul, x(), lit(4)),
        Expr::bin(Op::Lt, lit(1), x()),
        Expr::cond(Expr::boolean(true), lit(1), lit(2)),
        Expr::bind(
            Ident::new("x").unwrap(),
            lit(5),
            Expr::bin(Op::Add, x(), lit(1)),
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The interpreter's WHOLE discovered spec is exactly this — found by the generic engine running
    /// the operators, deterministically. Pins the engine end to end (against the unmutated
    /// interpreter, since this test runs when the engine itself is mutated): a mutation to
    /// enumeration, template matching, rendering, or coverage changes a law and is killed.
    #[test]
    fn the_interpreter_spec_is_exact() {
        let spec = interpreter_spec();
        let got: Vec<(String, String)> = spec
            .laws
            .iter()
            .map(|l| (l.prose.clone(), l.equation.clone()))
            .collect();
        let expected: Vec<(&str, &str)> = vec![
            (
                "Addition gives the same result in either order.",
                "(x + y) = (y + x)",
            ),
            (
                "With Addition, the grouping of three values doesn't matter.",
                "((x + y) + z) = (x + (y + z))",
            ),
            ("Addition with 0 leaves a value unchanged.", "(0 + x) = x"),
            (
                "Multiplication gives the same result in either order.",
                "(x * y) = (y * x)",
            ),
            (
                "With Multiplication, the grouping of three values doesn't matter.",
                "((x * y) * z) = (x * (y * z))",
            ),
            ("Multiplication by 0 always gives 0.", "(0 * x) = 0"),
            (
                "Multiplication with 1 leaves a value unchanged.",
                "(1 * x) = x",
            ),
            (
                "Multiplication distributes over Addition.",
                "(x * (y + z)) = ((x * y) + (x * z))",
            ),
            ("A value is never less than itself.", "(x < x) = false"),
            (
                "No two distinct programs look the same — the faithful rendering distinguishes \
                 every structural and semantic difference.",
                "U(p) = U(q)  ⟹  p = q   (U = faithful render)",
            ),
        ];
        let expected: Vec<(String, String)> = expected
            .into_iter()
            .map(|(p, e)| (p.to_string(), e.to_string()))
            .collect();
        assert_eq!(got, expected, "the interpreter's discovered spec changed");
        // every operator participates in a law (coverage is complete for arithmetic).
        assert!(
            spec.uncovered_ops.is_empty(),
            "uncovered: {:?}",
            spec.uncovered_ops
        );
    }

    /// The discovered laws actually hold when re-probed (replay), and the U observer is load-bearing:
    /// the faithful render is universally sensitive, a collapsing one (`node_count`) is not.
    #[test]
    fn the_spec_replays_and_u_is_load_bearing() {
        assert_eq!(replay_interpreter_laws(), Ok(()));
        assert!(observer_is_sensitive(render()), "render should be faithful");
        assert!(
            !observer_is_sensitive(|x: &Expr| x.node_count()),
            "a collapsing observer must fail the sensitivity law"
        );
    }

    /// Every discovered law renders readably — non-empty prose ending in a sentence, and an equation.
    #[test]
    fn discovered_laws_render_readably() {
        for law in discover_laws() {
            assert!(law.prose().ends_with('.') || law.prose().ends_with("difference."));
            assert!(!law.equation().is_empty());
        }
    }
}
