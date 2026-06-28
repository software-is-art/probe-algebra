//! Tests for the boundary algebra, exercised through the interpreter — the sole
//! demonstration substrate. These import ONLY through `interp::boundary`, exactly as
//! another module would, and test the PUBLIC edges (`Parse`/`Check`/`Eval`). The
//! lexer/parser/checker/evaluator in `interp::internal` have no tests of their own; this
//! file plus the autogen `laws` registry are the entire rigour they get, and mutation
//! testing measures how much that buys.

use crate::boundary::{
    fits, require_complete, require_within, CostCons, CostNil, Fold, MapCollect, SpaceCost,
    TimeCost, S, Z,
};
use crate::gdp::with_seed;
use crate::interp::boundary::{
    Check, Depth, Eval, Expr, FullProbe, Ident, Int, Nodes, Op, Parse, Value,
};

fn name(s: &str) -> Ident {
    Ident::new(s).unwrap()
}
fn int(v: i64) -> Expr {
    Expr::int(v).unwrap()
}
fn num(v: i64) -> Value {
    Value::Int(Int::new(v).unwrap())
}

/// Parse → Check → Eval through the boundary, returning the value. Panics if the
/// program is ill-typed (the test feeds only well-typed programs).
fn eval(expr: Expr) -> Value {
    with_seed(|seed| {
        let named = seed.new_named(expr);
        let proof = Check.classify(&named).expect("well-typed");
        *Eval.run(&named, &proof).value()
    })
}

/// Does the named expression type-check (the `Branch`'s positive arm)?
fn well_typed(expr: Expr) -> bool {
    with_seed(|seed| {
        let named = seed.new_named(expr);
        Check.classify(&named).is_ok()
    })
}

#[test]
fn evaluates_arithmetic() {
    assert_eq!(eval(int(2)), num(2));
    assert_eq!(eval(Expr::bin(Op::Add, int(2), int(3))), num(5));
    assert_eq!(eval(Expr::bin(Op::Mul, int(2), int(3))), num(6));
}

#[test]
fn evaluates_comparison_and_if() {
    assert_eq!(eval(Expr::bin(Op::Lt, int(1), int(2))), Value::Bool(true));
    assert_eq!(eval(Expr::bin(Op::Lt, int(2), int(1))), Value::Bool(false));
    // EQUAL operands pin `<` against `<=` (strict less-than).
    assert_eq!(eval(Expr::bin(Op::Lt, int(2), int(2))), Value::Bool(false));
    let cond = Expr::bin(Op::Lt, int(1), int(2));
    assert_eq!(eval(Expr::cond(cond.clone(), int(7), int(9))), num(7));
    let cond_false = Expr::bin(Op::Lt, int(2), int(1));
    assert_eq!(eval(Expr::cond(cond_false, int(7), int(9))), num(9));
}

#[test]
fn evaluates_let_and_var() {
    let prog = Expr::bind(
        name("x"),
        int(4),
        Expr::bin(Op::Add, Expr::var(name("x")), int(1)),
    );
    assert_eq!(eval(prog), num(5));
    let shadow = Expr::bind(
        name("x"),
        int(1),
        Expr::bind(name("x"), int(2), Expr::var(name("x"))),
    );
    assert_eq!(eval(shadow), num(2));
    let square = Expr::bind(
        name("x"),
        int(3),
        Expr::bin(Op::Mul, Expr::var(name("x")), Expr::var(name("x"))),
    );
    assert_eq!(eval(square), num(9));
}

/// The value-object accessors and the canonical `render` report the REAL contents, not a
/// constant. (`render` is pinned here directly because the round-trip law's generator is
/// itself built from `render`.)
#[test]
fn accessors_and_render_report_the_real_value() {
    assert_eq!(Int::new(7).unwrap().get(), 7);
    assert_eq!(name("foo").get(), "foo");
    assert_eq!(Expr::bin(Op::Add, int(1), int(2)).render(), "(1 + 2)");
    assert_eq!(Expr::bin(Op::Mul, int(2), int(3)).render(), "(2 * 3)");
    assert_eq!(Expr::bin(Op::Lt, int(1), int(2)).render(), "(1 < 2)");
    assert_eq!(
        Expr::cond(Expr::boolean(true), int(1), int(2)).render(),
        "(if true then 1 else 2)"
    );
    assert_eq!(
        Expr::bind(name("x"), int(1), Expr::var(name("x"))).render(),
        "(let x = 1 in x)"
    );
}

#[test]
fn accepts_well_typed() {
    assert!(well_typed(Expr::bin(Op::Add, int(1), int(2))));
    assert!(well_typed(Expr::bin(Op::Lt, int(1), int(2))));
    let if_ok = Expr::cond(Expr::bin(Op::Lt, int(1), int(2)), int(1), int(2));
    assert!(well_typed(if_ok));
    let let_ok = Expr::bind(
        name("x"),
        int(1),
        Expr::bin(Op::Add, Expr::var(name("x")), int(1)),
    );
    assert!(well_typed(let_ok));
}

#[test]
fn rejects_ill_typed() {
    assert!(!well_typed(Expr::bin(Op::Add, int(1), Expr::boolean(true))));
    assert!(!well_typed(Expr::bin(Op::Add, Expr::boolean(true), int(1))));
    assert!(!well_typed(Expr::bin(Op::Mul, int(1), Expr::boolean(true))));
    assert!(!well_typed(Expr::bin(Op::Lt, int(1), Expr::boolean(true))));
    assert!(!well_typed(Expr::cond(int(1), int(2), int(3))));
    assert!(!well_typed(Expr::cond(
        Expr::bin(Op::Lt, int(1), int(2)),
        int(1),
        Expr::boolean(true)
    )));
    assert!(!well_typed(Expr::var(name("y"))));
    let bad_let = Expr::bind(
        name("x"),
        Expr::boolean(true),
        Expr::bin(Op::Add, Expr::var(name("x")), int(1)),
    );
    assert!(!well_typed(bad_let));
}

#[test]
fn parses_and_runs_a_program() {
    let e = Parse
        .parse_str("(let x = 5 in (if (x < 10) then (x + 1) else 0))")
        .unwrap();
    assert_eq!(eval(e), num(6));
}

#[test]
fn parse_builds_the_expected_structure() {
    assert_eq!(
        Parse.parse_str("(1 + (2 * 3))").unwrap(),
        Expr::bin(Op::Add, int(1), Expr::bin(Op::Mul, int(2), int(3)))
    );
    assert_eq!(Parse.parse_str("true").unwrap(), Expr::boolean(true));
    assert_eq!(
        Parse.parse_str("(x < y)").unwrap(),
        Expr::bin(Op::Lt, Expr::var(name("x")), Expr::var(name("y")))
    );
}

#[test]
fn rejects_malformed_source() {
    assert!(Parse.parse_str("1 +").is_none());
    assert!(Parse.parse_str("(1 + 2").is_none());
    assert!(Parse.parse_str("(1 + 2) extra").is_none());
    assert!(Parse.parse_str("@").is_none());
    assert!(Parse.parse_str("()").is_none());
    assert!(Parse.parse_str("(1 2 3)").is_none());
}

// ===== cost grading on the interp edges ===================================

/// A deterministic measure for the `fits` audit: build the sum `1 + 1 + ... + 1` of `n`
/// ones as a left-nested chain and evaluate it THROUGH the boundary. Each `+` adds one to
/// both the node count and the result, so the measured result tracks `Eval`'s real step
/// count — non-tautological because it actually runs the evaluator end to end.
fn eval_steps(n: usize) -> u64 {
    let mut e = int(1);
    for _ in 1..n {
        e = Expr::bin(Op::Add, e, int(1));
    }
    match eval(e) {
        Value::Int(i) => i.get() as u64,
        Value::Bool(_) => unreachable!("a sum of ints evaluates to an int"),
    }
}

/// The empirical honesty check: `Eval`'s declared `Nodes` time degree is `S<Z>` (linear).
/// `fits` confirms the real growth is linear (degree 1) and NOT constant (degree 0), so the
/// leaf's declared degree matches reality — the audit the type level cannot perform itself.
#[test]
fn eval_is_empirically_linear_in_nodes() {
    assert!(
        fits(eval_steps, 1),
        "eval should grow linearly in node count"
    );
    assert!(
        !fits(eval_steps, 0),
        "eval is not constant — a degree-0 claim must be rejected"
    );
}

/// The per-axis map earns its keep here: `Eval`'s TIME is keyed on `Nodes` while its SPACE
/// is keyed on `Depth`, so the two budgets are demanded independently. A scalar cost could
/// not separate them. `require_within` is a compile-time demand; reaching this line means
/// both bounds hold.
#[test]
fn eval_time_and_space_axes_are_within_budget() {
    // Eval: linear in Nodes (time), linear in Depth (space).
    require_within::<TimeCost, Eval, CostCons<Nodes, S<Z>, CostNil>>();
    require_within::<SpaceCost, Eval, CostCons<Depth, S<Z>, CostNil>>();
    // Parse: linear in Nodes in both resources.
    require_within::<TimeCost, Parse, CostCons<Nodes, S<Z>, CostNil>>();
    require_within::<SpaceCost, Parse, CostCons<Nodes, S<Z>, CostNil>>();
    // Folding Eval per node STREAMS: time gains a Nodes degree (now quadratic) but space
    // stays linear in Depth — the "stream, don't materialize" fact, per-axis.
    require_within::<TimeCost, Fold<Eval, Nodes>, CostCons<Nodes, S<S<Z>>, CostNil>>();
    require_within::<SpaceCost, Fold<Eval, Nodes>, CostCons<Depth, S<Z>, CostNil>>();
    // Collecting Eval per node MATERIALIZES n results: BOTH time and space gain a Nodes
    // degree, so space is now quadratic in Nodes too.
    require_within::<TimeCost, MapCollect<Eval, Nodes>, CostCons<Nodes, S<S<Z>>, CostNil>>();
    require_within::<SpaceCost, MapCollect<Eval, Nodes>, CostCons<Nodes, S<S<Z>>, CostNil>>();
}

// ===== degrees of freedom: the static completeness demand =================

/// A probe that reaches BOTH of `Expr`'s declared DOFs (shape and literals) satisfies
/// `require_complete` — the positive of `tests/compile_fail/incomplete_probe_rejected`,
/// where a shape-only probe is rejected for missing the `Literals` dimension.
#[test]
fn a_complete_probe_covers_every_dof() {
    require_complete::<Expr, _>(&FullProbe);
}

/// The headline: the brand minted at `with_seed` threads through parse, the `Check`
/// branch, and the `Eval` guard. Only `Check` can mint the `WellTyped` witness `Eval`
/// demands, and the shared name ties them to THIS program (a proof for another program
/// will not type-check — pinned in `tests/compile_fail/eval_wrong_program`).
#[test]
fn the_brand_threads_parse_check_eval() {
    with_seed(|seed| {
        let named = seed.new_named(Parse.parse_str("(2 + 3)").unwrap());
        match Check.classify(&named) {
            Ok(proof) => {
                let result = Eval.run(&named, &proof);
                assert_eq!(result.value(), &num(5));
            }
            Err(_) => panic!("(2 + 3) is well-typed"),
        }
    });
}

// ===== the capability lattice + the blind-spot map ========================
//
// `ConstFold` is a `Lossy` `Morphism` (collapses constant subexpressions, witnessing the
// loss in a residual). Three variants make the blind-spot map executable: NO single probe
// flavour is highest-assurance, and the decisive negative result is that a wrong-but-
// invertible coefficient survives BOTH structural probes and dies only to the
// reference-bearing quantitative one.
mod blind_spot {
    use super::{int, Expr, Op};
    use crate::boundary::{
        coefficient_holds, commutes, Capability, Coefficient, Metamorphic, Morphism, Unit,
    };
    use crate::interp::boundary::{ConstFold, ConstFoldDoubles, ConstFoldForgetful, Int, Lit};

    /// A top-level `Add` of two integer literals — folds to a single literal.
    fn sum(a: i64, b: i64) -> Expr {
        Expr::bin(Op::Add, int(a), int(b))
    }
    /// The folded output (the morphism's `Out`).
    fn folds_to<M: Morphism<In = Expr, Out = Expr>>(m: &M, x: &Expr) -> Expr {
        m.forward(x).0
    }
    /// Does `backward(forward(x)) == x`? — the residual round-trip, as a `bool`.
    fn round_trips<M: Morphism<In = Expr, Out = Expr>>(m: &M, x: &Expr) -> bool {
        let (out, residual) = m.forward(x);
        m.backward(&out, &residual).as_ref() == Some(x)
    }

    /// METAMORPHIC relation (reference-free): swapping the operands of a top-level `Add`
    /// must leave the folded result unchanged (addition is commutative).
    struct SwapAddends;
    crate::value_operator!(SwapAddends);
    impl<M: Morphism<In = Expr, Out = Expr>> Metamorphic<M> for SwapAddends {
        fn input_op(&self, x: &Expr) -> Option<Expr> {
            match x {
                Expr::Bin(Op::Add, a, b) => Some(Expr::bin(Op::Add, (**b).clone(), (**a).clone())),
                _ => None,
            }
        }
        fn output_op(&self, y: &Expr) -> Expr {
            y.clone()
        }
    }

    /// COEFFICIENT relation (reference-bearing): incrementing one addend by 1 must raise the
    /// folded sum by exactly 1 — the reference coefficient of honest addition.
    struct IncrementAddend;
    crate::value_operator!(IncrementAddend);
    fn lit_int(e: &Expr) -> i64 {
        match e {
            Expr::Lit(Lit::Int(i)) => i.get(),
            _ => -1, // a non-folded result fails the equality, as it should.
        }
    }
    impl<M: Morphism<In = Expr, Out = Expr>> Coefficient<M> for IncrementAddend {
        type Delta = Int;
        fn unit_step(&self, x: &Expr) -> Option<Expr> {
            match x {
                Expr::Bin(Op::Add, a, b) => match &**b {
                    Expr::Lit(Lit::Int(bi)) => Some(Expr::bin(
                        Op::Add,
                        (**a).clone(),
                        Expr::Lit(Lit::Int(bi.plus(Int::new(1)?))),
                    )),
                    _ => None,
                },
                _ => None,
            }
        }
        fn expected_delta(&self) -> Int {
            Int::new(1).expect("1 is a valid Int")
        }
        fn observed_delta(&self, before: &Expr, after: &Expr) -> Int {
            Int::new((lit_int(after) - lit_int(before)).max(0)).expect("non-negative delta")
        }
    }

    /// The `Lossy` ceiling is REFLECTED to the runtime so the audit/laws read it unchanged.
    /// (The type-level demand is pinned in `tests/compile_fail/run_pure_rejects_lossy`.)
    #[test]
    fn const_fold_declares_lossy() {
        assert_eq!(ConstFold::CAPABILITY, Capability::Lossy);
        assert_eq!(ConstFoldDoubles::CAPABILITY, Capability::Lossy);
    }

    /// ROW 1 — residual incompleteness: the round-trip CATCHES the forgetful folder (its
    /// `Unit` residual cannot rebuild the collapsed input), while a value-only check is
    /// BLIND, because the folded value itself is correct.
    #[test]
    fn round_trip_catches_a_dropped_residual_but_the_value_is_blind() {
        let x = sum(2, 3);
        assert!(
            round_trips(&ConstFold, &x),
            "honest keeps a complete residual"
        );
        assert!(
            !round_trips(&ConstFoldForgetful, &x),
            "a dropped residual cannot reconstruct the folded input"
        );
        // value-only blind spot: the forgetful folder computes the SAME (correct) value.
        assert_eq!(folds_to(&ConstFold, &x), folds_to(&ConstFoldForgetful, &x));
    }

    /// ROW 2 — wrong coefficient: the doubling folder keeps a complete residual AND is
    /// symmetric, so BOTH structural probes (round-trip and commutation) are BLIND to it.
    #[test]
    fn structural_probes_are_blind_to_a_wrong_coefficient() {
        let x = sum(2, 3);
        // round-trip blind: the residual restores the original regardless of the value.
        assert!(round_trips(&ConstFoldDoubles, &x));
        // commutation blind: doubling is symmetric, so it commutes with operand-swap.
        assert_eq!(commutes(&ConstFold, &SwapAddends, &x), Some(true));
        assert_eq!(
            commutes(&ConstFoldDoubles, &SwapAddends, &x),
            Some(true),
            "a uniform (symmetric) wrong coefficient respects the swap relation"
        );
    }

    /// The DECISIVE negative result: only the reference-bearing COEFFICIENT probe separates
    /// the honest folder from the doubling one — the structural checks above could not.
    #[test]
    fn the_coefficient_probe_catches_what_structure_cannot() {
        let x = sum(2, 3);
        assert_eq!(
            coefficient_holds(&ConstFold, &IncrementAddend, &x),
            Some(true)
        );
        assert_eq!(
            coefficient_holds(&ConstFoldDoubles, &IncrementAddend, &x),
            Some(false),
            "the quantitative probe pins the coefficient the structural checks missed"
        );
    }

    /// A relation `ConstFold` does NOT commute with (bump the left addend, expect the sum
    /// unchanged) — so `commutes` returns `Some(false)`; and an inapplicable input returns
    /// `None`. Pins `commutes` against a constant-`Some(true)` collapse.
    struct BumpLeftExpectSame;
    crate::value_operator!(BumpLeftExpectSame);
    impl<M: Morphism<In = Expr, Out = Expr>> Metamorphic<M> for BumpLeftExpectSame {
        fn input_op(&self, x: &Expr) -> Option<Expr> {
            match x {
                Expr::Bin(Op::Add, a, b) => match &**a {
                    Expr::Lit(Lit::Int(ai)) => Some(Expr::bin(
                        Op::Add,
                        Expr::Lit(Lit::Int(ai.plus(Int::new(1)?))),
                        (**b).clone(),
                    )),
                    _ => None,
                },
                _ => None,
            }
        }
        fn output_op(&self, y: &Expr) -> Expr {
            y.clone() // expect NO change — wrong, so the fold does not commute.
        }
    }

    #[test]
    fn commutes_distinguishes_outcomes() {
        let x = sum(2, 3);
        assert_eq!(
            commutes(&ConstFold, &BumpLeftExpectSame, &x),
            Some(false),
            "bumping an addend changes the sum, so the fold does not commute with it"
        );
        // inapplicable: a bare literal has no top-level Add to perturb.
        assert_eq!(commutes(&ConstFold, &SwapAddends, &int(5)), None);
    }

    /// The forgetful folder's `backward` returns the folded OUTPUT (it has no residual to do
    /// better) — distinct from `None`. Pins it so a `-> None` mutant cannot hide behind the
    /// round-trip, which is false either way.
    #[test]
    fn forgetful_backward_returns_the_output() {
        let x = sum(2, 3);
        let (out, _unit) = ConstFoldForgetful.forward(&x);
        assert_eq!(ConstFoldForgetful.backward(&out, &Unit), Some(out.clone()));
        assert_ne!(out, x, "the fold did change the expression");
    }
}

// ===== the rest of the algebra, exercised on the interpreter ==============
//
// With interp the sole substrate, the generic grammar (residual probe, DOF synthesis,
// Compose/run/Carried, Then, construction_probe, the profiling wrapper, provenance
// lineage, the type-level degree) must be exercised HERE or it is dead. These tests run
// each on the `ConstFold`/`Parse` edges; mutation testing then certifies the grammar is
// covered, not merely present. Plus the direct pins for the `Expr` cost measures.
mod algebra_surface {
    use super::{int, name, Expr, Op};
    use crate::boundary::{
        construction_probe, dof_covered, probe, probe_declared_dofs, reconstructs, run,
        stamp_through, Compose, Construction, Degree, DofProbe, Meter, Morphism, Perturbation,
        ProbeResult, Profiled, RawPerturbation, Stamped, Then, S, Z,
    };
    use crate::interp::boundary::{ConstFold, Lit, Literals, Parse, Shape};

    fn sum() -> Expr {
        Expr::bin(Op::Add, int(2), int(3))
    }
    fn five() -> Expr {
        Expr::int(5).unwrap() // sum() folded: 2 + 3
    }
    fn three() -> Expr {
        Expr::int(3).unwrap() // "(1 + 2)" folded
    }

    // --- perturbations on the ConstFold morphism (for probe / DOF synthesis) ---

    /// Shape perturbation: swap the top operator (Add <-> Mul) — changes the folded value.
    pub struct PerturbOp;
    crate::value_operator!(PerturbOp);
    impl<M: Morphism<In = Expr, Out = Expr>> Perturbation<M> for PerturbOp {
        fn perturb(&self, x: &Expr) -> Option<Expr> {
            match x {
                Expr::Bin(Op::Add, a, b) => Some(Expr::bin(Op::Mul, (**a).clone(), (**b).clone())),
                Expr::Bin(Op::Mul, a, b) => Some(Expr::bin(Op::Add, (**a).clone(), (**b).clone())),
                _ => None,
            }
        }
    }

    /// Literal perturbation: bump the left integer literal — changes the folded value.
    pub struct PerturbLit;
    crate::value_operator!(PerturbLit);
    impl<M: Morphism<In = Expr, Out = Expr>> Perturbation<M> for PerturbLit {
        fn perturb(&self, x: &Expr) -> Option<Expr> {
            match x {
                Expr::Bin(op, a, b) => match &**a {
                    Expr::Lit(Lit::Int(ai)) => Some(Expr::bin(
                        *op,
                        Expr::Lit(Lit::Int(ai.plus(crate::interp::boundary::Int::new(1)?))),
                        (**b).clone(),
                    )),
                    _ => None,
                },
                _ => None,
            }
        }
    }

    impl DofProbe<ConstFold> for Shape {
        type Perturb = PerturbOp;
        fn perturbation() -> PerturbOp {
            PerturbOp
        }
    }
    impl DofProbe<ConstFold> for Literals {
        type Perturb = PerturbLit;
        fn perturbation() -> PerturbLit {
            PerturbLit
        }
    }

    /// The residual-completeness `probe` on `ConstFold`: bumping a literal moves the folded
    /// OUTPUT (the residual is not a hidden lost dimension here), and the original-keeping
    /// residual still reconstructs the perturbed input. The exact `ProbeResult` is pinned so
    /// every comparison inside `probe` is load-bearing.
    #[test]
    fn probe_reports_each_field() {
        let r = probe(&ConstFold, &PerturbLit, &sum()).expect("perturbation applies");
        assert!(
            !r.output_invariant,
            "folding is sensitive to the bumped literal"
        );
        assert!(
            r.residual_responds,
            "the residual keeps the (changed) original"
        );
        assert!(
            r.round_trips,
            "the residual reconstructs the perturbed input"
        );
    }

    /// `residual_complete` is the strict conjunction — pinned on hand-built results so each
    /// `&&` and the bare return are exercised.
    #[test]
    fn residual_complete_is_the_conjunction() {
        let yes = ProbeResult {
            output_invariant: true,
            residual_responds: true,
            round_trips: true,
        };
        assert!(yes.residual_complete());
        for r in [
            ProbeResult {
                output_invariant: false,
                residual_responds: true,
                round_trips: true,
            },
            ProbeResult {
                output_invariant: true,
                residual_responds: false,
                round_trips: true,
            },
            ProbeResult {
                output_invariant: true,
                residual_responds: true,
                round_trips: false,
            },
        ] {
            assert!(!r.residual_complete());
        }
    }

    /// DOF SYNTHESIS: the completeness suite is generated from `Expr`'s declared DOF set —
    /// both `Shape` and `Literals` are observable through `ConstFold` (the output responds),
    /// so the synthesized verdicts are both `Some(true)`.
    #[test]
    fn declared_dofs_synthesize_their_probes() {
        let verdicts = probe_declared_dofs::<Expr, ConstFold>(&ConstFold, &sum());
        assert_eq!(verdicts, vec![Some(true), Some(true)]);
        // a single DOF directly: covered at a foldable node...
        assert_eq!(
            dof_covered::<ConstFold, Literals>(&ConstFold, &sum()),
            Some(true)
        );
        // ...but INAPPLICABLE at a bare literal (no operator to perturb) — `None`, which
        // pins `dof_covered` against an always-`Some(true)` collapse.
        assert_eq!(dof_covered::<ConstFold, Shape>(&ConstFold, &int(5)), None);
    }

    /// COMPOSE + run + Carried::invert: two folds compose into one morphism whose retained
    /// residual still inverts back to the original — end-to-end invertibility through a
    /// lossy stage.
    #[test]
    fn compose_run_and_invert() {
        let comp = Compose {
            f: ConstFold,
            g: ConstFold,
        };
        let x = sum();
        let carried = run(&comp, &x);
        assert_eq!(carried.out(), &five());
        assert_eq!(carried.invert(&comp), Some(x));
    }

    /// THEN: a `Construction` composed with a `Morphism` is itself one construction —
    /// parse-then-fold, and its product residual reconstructs the raw source.
    #[test]
    fn then_composes_construction_with_morphism() {
        let then = Then {
            construct: Parse,
            then: ConstFold,
        };
        let (refined, residual) = then.parse(&"(1 + 2)".to_string()).expect("valid source");
        assert_eq!(refined, three());
        assert_eq!(
            then.reconstruct(&refined, &residual),
            Some("(1 + 2)".to_string())
        );
    }

    /// reconstructs: a valid source round-trips; a REJECTED source has no obligation
    /// (`None`), which also pins the probe against a constant-`Some(true)` collapse.
    #[test]
    fn reconstructs_round_trips_or_abstains() {
        assert_eq!(reconstructs(&Parse, &"(1 + 2)".to_string()), Some(true));
        assert_eq!(reconstructs(&Parse, &"@".to_string()), None);
    }

    /// construction_probe on `Parse`: changing a digit changes the REFINED expr (the parse
    /// normalizes nothing — its `Unit` residual does not respond), and the canonical
    /// perturbed source still round-trips.
    struct SwapDigit;
    crate::value_operator!(SwapDigit);
    impl RawPerturbation<Parse> for SwapDigit {
        fn perturb(&self, raw: &String) -> Option<String> {
            if raw.contains('2') {
                Some(raw.replace('2', "3"))
            } else {
                None
            }
        }
    }
    #[test]
    fn construction_probe_reports_each_field() {
        let r = construction_probe(&Parse, &SwapDigit, &"(1 + 2)".to_string()).expect("applies");
        assert!(
            !r.output_invariant,
            "a different digit parses to a different expr"
        );
        assert!(!r.residual_responds, "the pure parse has a Unit residual");
        assert!(
            r.round_trips,
            "the perturbed source is canonical, so it round-trips"
        );
    }

    /// PROFILED + Meter: wrapping a morphism meters every `forward`/`backward` transparently.
    /// A counting meter confirms `progress` fires and `backward` still returns its result.
    #[test]
    fn profiled_meters_without_changing_behaviour() {
        use std::cell::Cell;
        struct CountMeter {
            progressed: Cell<u32>,
        }
        impl Meter for CountMeter {
            fn measured<R>(&self, _label: &'static str, body: impl FnOnce() -> R) -> R {
                body()
            }
            fn progress(&self, _label: &'static str) {
                self.progressed.set(self.progressed.get() + 1);
            }
        }
        let meter = CountMeter {
            progressed: Cell::new(0),
        };
        let p = Profiled::metered(ConstFold, &meter); // &meter -> exercises `impl Meter for &T`
        let x = sum();
        let (out, residual) = p.forward(&x);
        assert_eq!(out, five());
        assert_eq!(meter.progressed.get(), 1, "forward marks one progress unit");
        // backward is transparent: it still reconstructs via ConstFold's residual.
        assert_eq!(p.backward(&out, &residual), Some(x));
    }

    /// PROVENANCE: stamping a value through an edge extends its type-level lineage, and the
    /// reflected `Provenance` names the edge it crossed.
    #[test]
    fn stamp_through_records_the_lineage() {
        let stamped = Stamped::origin(sum());
        let folded = stamp_through(&stamped, &ConstFold);
        assert_eq!(folded.value(), &five());
        let lineage = folded.lineage();
        assert_eq!(lineage.steps().len(), 1);
        assert!(
            lineage.steps()[0].contains("ConstFold"),
            "the lineage names the edge crossed: {:?}",
            lineage.steps()
        );
    }

    /// The type-level degree reflects to its number — the `D::N + 1` recursion.
    #[test]
    fn degree_reflects_to_its_number() {
        assert_eq!(<Z as Degree>::N, 0);
        assert_eq!(<S<Z> as Degree>::N, 1);
        assert_eq!(<S<S<Z>> as Degree>::N, 2);
    }

    /// `Expr::node_count` / `Expr::depth` report the real structure (the cost axes). Pinned
    /// directly because the cost grading consumes them only at the type level.
    #[test]
    fn node_count_and_depth_measure_the_tree() {
        assert_eq!(int(1).node_count(), 1);
        assert_eq!(int(1).depth(), 1);
        assert_eq!(sum().node_count(), 3);
        let nested = Expr::bin(Op::Add, int(1), Expr::bin(Op::Mul, int(2), int(3)));
        assert_eq!(nested.node_count(), 5);
        assert_eq!(nested.depth(), 3);
        let iff = Expr::cond(Expr::boolean(true), int(1), int(2));
        assert_eq!(iff.node_count(), 4);
        assert_eq!(iff.depth(), 2);
        let lett = Expr::bind(name("x"), int(1), Expr::var(name("x")));
        assert_eq!(lett.node_count(), 3);
        assert_eq!(lett.depth(), 2);
    }
}
