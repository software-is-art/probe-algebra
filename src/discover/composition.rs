//!
//! composition — what holds across a PIPELINE of modules, not within one.
//!
//! A program is a graph of algebras joined by seams. A TRANSFORM seam is a conversion `h : A → B`
//! that is a HOMOMORPHISM — it carries the source algebra's structure into the target (the engine
//! discovers each one within a module). But a real program CHAINS them: `A → B → C → …`. The
//! question this answers is the one a single module cannot: does structure SURVIVE the chain?
//!
//! It does, and we discover it by running it. If `h1 : A → B` and `h2 : B → C` are each
//! homomorphisms — `h(x ⊕ y) = h(x) ⊕' h(y)` — then the COMPOSITE `h2∘h1 : A → C` is one too: for
//! all `x, y`, `h2(h1(x ⊕_A y)) = h2(h1(x)) ⊕_C h2(h1(y))`. So along a transform pipeline the
//! OPERATIONS change at every stage (`⊕_A → ⊕_B → ⊕_C`) but the LAW is invariant — the dataflow
//! preserves the algebra end to end. That composite equation is the whole-program spec of the
//! pipeline, and it is verified, not assumed: we evaluate it over the source grid.

use super::engine::{Operator, Theory};

/// A discovered end-to-end law of a transform pipeline: the composite of two conversions is a
/// homomorphism from the source's binary operator to the target's.
#[derive(Debug)]
pub struct PipelineLaw {
    /// The conversions composed, source-first (`["hAB", "hBC"]` ⇒ `hBC∘hAB`).
    pub via: Vec<&'static str>,
    /// The source binary operator (`⊕_A`).
    pub from_op: &'static str,
    /// The target binary operator (`⊕_C`).
    pub to_op: &'static str,
    /// The rendered composite equation.
    pub equation: String,
}

#[crate::mutate]
impl PipelineLaw {
    /// Discover the end-to-end laws of a theory's transform pipelines. The discovery is an
    /// associated function of the LAW it yields — the public surface is the value object, not a
    /// loose function (the no-rats-nest rule: every public callable hangs off a typestate).
    pub fn discover<T: Theory>() -> Vec<Self> {
        pipeline_laws::<T>()
    }

    /// Render the pipeline laws as a readable whole-program spec.
    pub fn render<T: Theory>() -> String {
        let laws = Self::discover::<T>();
        let mut out = format!("module `{}`: ", T::name());
        if laws.is_empty() {
            out.push_str("no transform pipeline — no cross-module composite law.\n");
            return out;
        }
        out.push_str("transform pipeline preserves structure end to end:\n");
        for l in &laws {
            out.push_str(&format!(
                "  {} ∘ {}: {} → {} is a homomorphism — {}\n",
                l.via[1], l.via[0], l.from_op, l.to_op, l.equation
            ));
        }
        out
    }
}

/// Apply a unary operator, threading partiality.
#[crate::mutate]
fn apply1<T: Theory>(op: &Operator<T>, x: &T::Value) -> Option<T::Value> {
    (op.eval)(std::slice::from_ref(x))
}

/// Apply a binary operator.
#[crate::mutate]
fn apply2<T: Theory>(op: &Operator<T>, a: &T::Value, b: &T::Value) -> Option<T::Value> {
    (op.eval)(&[a.clone(), b.clone()])
}

/// Is `op` a homogeneous binary operator on sort `s` (`s × s → s`)?
#[crate::mutate]
fn binary_on<T: Theory>(op: &Operator<T>, s: T::Sort) -> bool {
    op.inputs.len() == 2 && op.inputs[0] == s && op.inputs[1] == s && op.output == s
}

/// Is `op` a unary conversion `from → to`?
#[crate::mutate]
fn unary<T: Theory>(op: &Operator<T>) -> Option<(T::Sort, T::Sort)> {
    (op.inputs.len() == 1).then(|| (op.inputs[0], op.output))
}

/// Discover the end-to-end laws of a theory's transform PIPELINES: for every pair of unary
/// conversions that compose across a sort (`h1 : s → t`, `h2 : t → u`) and end at a DIFFERENT sort
/// than they start (`s ≠ u` — a genuine transform, not a round trip back home), verify that the
/// composite `h2∘h1` carries a binary operator on `s` to one on `u` as a homomorphism, over the
/// source grid. Each that holds is a whole-program law of that pipeline. (Private — reached as
/// `PipelineLaw::discover`.)
#[crate::mutate]
fn pipeline_laws<T: Theory>() -> Vec<PipelineLaw> {
    let ops = T::operators();
    let mut laws = Vec::new();

    for h1 in &ops {
        let Some((s, t)) = unary(h1) else { continue };
        for h2 in &ops {
            let Some((t2, u)) = unary(h2) else { continue };
            // the conversions must chain (h1's output feeds h2) and the pipeline must TRANSFORM —
            // end somewhere other than where it began (a round trip `s → t → s` is not a pipeline).
            if t2 != t || u == s {
                continue;
            }
            let grid = T::inhabitants(s);
            if grid.is_empty() {
                continue;
            }
            let comp = |x: &T::Value| apply1(h1, x).and_then(|hx| apply1(h2, &hx));

            for p in ops.iter().filter(|p| binary_on(p, s)) {
                for r in ops.iter().filter(|r| binary_on(r, u)) {
                    let holds = grid.iter().all(|x| {
                        grid.iter().all(|y| {
                            let lhs = apply2(p, x, y).as_ref().and_then(&comp);
                            let rhs = match (comp(x), comp(y)) {
                                (Some(cx), Some(cy)) => apply2(r, &cx, &cy),
                                _ => None,
                            };
                            match (lhs, rhs) {
                                (Some(l), Some(rr)) => T::observe(&l) == T::observe(&rr),
                                _ => false,
                            }
                        })
                    });
                    if holds {
                        laws.push(PipelineLaw {
                            via: vec![h1.symbol, h2.symbol],
                            from_op: p.symbol,
                            to_op: r.symbol,
                            equation: format!(
                                "{to}(h(x), h(y)) = h({from}(x, y))   where h = {h2}∘{h1}",
                                to = r.symbol,
                                from = p.symbol,
                                h2 = h2.symbol,
                                h1 = h1.symbol,
                            ),
                        });
                    }
                }
            }
        }
    }
    laws
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discover::arithmetic::Arithmetic;
    use crate::discover::router::Router;

    // A three-stage pipeline: a value flows A → B → C, combined by `max` at each stage, with the
    // stage conversions `hAB` and `hBC` (relabelings that preserve magnitude). Each conversion is a
    // homomorphism over `max`; the question is whether the COMPOSITE A → C still is. (This is the
    // same shape v14's `Chain` used to demonstrate layering — here it demonstrates that the transform
    // pipeline's structure survives composition.)
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
    enum S3 {
        A,
        B,
        C,
    }
    struct Stages;
    fn maxa(v: &[(u8, i64)]) -> Option<(u8, i64)> {
        Some((0, v[0].1.max(v[1].1)))
    }
    fn maxb(v: &[(u8, i64)]) -> Option<(u8, i64)> {
        Some((1, v[0].1.max(v[1].1)))
    }
    fn maxc(v: &[(u8, i64)]) -> Option<(u8, i64)> {
        Some((2, v[0].1.max(v[1].1)))
    }
    fn hab(v: &[(u8, i64)]) -> Option<(u8, i64)> {
        Some((1, v[0].1))
    }
    fn hbc(v: &[(u8, i64)]) -> Option<(u8, i64)> {
        Some((2, v[0].1))
    }
    crate::theory! {
        Stages : "staged pipeline", Value = (u8, i64), Obs = (u8, i64), Sort = S3,
        sort_of = |v: &(u8, i64)| match v.0 { 0 => S3::A, 1 => S3::B, _ => S3::C },
        observe = |v: &(u8, i64)| *v,
        vars { S3::A => &["a"], S3::B => &["b"], S3::C => &["c"], }
        inhabit {
            S3::A => vec![(0, 0), (0, 1), (0, 2)],
            S3::B => vec![(1, 0), (1, 1), (1, 2)],
            S3::C => vec![(2, 0), (2, 1), (2, 2)],
        }
        ops {
            Infix  "maxA" "maxA" (S3::A, S3::A) -> S3::A = maxa;
            Infix  "maxB" "maxB" (S3::B, S3::B) -> S3::B = maxb;
            Infix  "maxC" "maxC" (S3::C, S3::C) -> S3::C = maxc;
            Prefix "hAB"  "hAB"  (S3::A) -> S3::B = hab;
            Prefix "hBC"  "hBC"  (S3::B) -> S3::C = hbc;
        }
    }

    // A BROKEN stage: `hBC` here NEGATES the magnitude, which does not commute with `max`
    // (`-max(a,b) = min(-a,-b) ≠ max(-a,-b)`), so it is NOT a homomorphism over `max` — and the
    // composite must therefore NOT be reported. Pins that the verifier actually checks the law rather
    // than assuming any two conversions compose. (A constant collapse would NOT work as a
    // counterexample: a constant map onto `max`'s fixpoint genuinely IS a homomorphism.)
    struct Broken;
    fn hbc_broken(v: &[(u8, i64)]) -> Option<(u8, i64)> {
        Some((2, -v[0].1))
    }
    crate::theory! {
        Broken : "broken pipeline", Value = (u8, i64), Obs = (u8, i64), Sort = S3,
        sort_of = |v: &(u8, i64)| match v.0 { 0 => S3::A, 1 => S3::B, _ => S3::C },
        observe = |v: &(u8, i64)| *v,
        vars { S3::A => &["a"], S3::B => &["b"], S3::C => &["c"], }
        inhabit {
            S3::A => vec![(0, 0), (0, 1), (0, 2)],
            S3::B => vec![(1, 0), (1, 1), (1, 2)],
            S3::C => vec![(2, 0), (2, 1), (2, 2)],
        }
        ops {
            Infix  "maxA" "maxA" (S3::A, S3::A) -> S3::A = maxa;
            Infix  "maxB" "maxB" (S3::B, S3::B) -> S3::B = maxb;
            Infix  "maxC" "maxC" (S3::C, S3::C) -> S3::C = maxc;
            Prefix "hAB"  "hAB"  (S3::A) -> S3::B = hab;
            Prefix "hBC"  "hBC"  (S3::B) -> S3::C = hbc_broken;
        }
    }

    /// The composite of the two stage conversions is itself a homomorphism: the A → C pipeline
    /// preserves the `max` algebra end to end, even though the operator changes at every stage. This
    /// is the whole-program law a single module's discovery cannot see — and it is VERIFIED over the
    /// source grid, not assumed.
    #[test]
    fn a_transform_pipeline_preserves_structure_end_to_end() {
        let laws = PipelineLaw::discover::<Stages>();
        assert_eq!(laws.len(), 1, "exactly the A→C composite: {laws:?}");
        let l = &laws[0];
        assert_eq!(l.via, vec!["hAB", "hBC"], "composed source-first");
        assert_eq!(l.from_op, "maxA");
        assert_eq!(l.to_op, "maxC");
        assert!(l.equation.contains("hBC∘hAB"));
        assert!(PipelineLaw::render::<Stages>().contains("preserves structure end to end"));
    }

    /// When a stage is NOT a homomorphism, the composite is not reported — the law is checked, not
    /// presumed. (`hBC` collapses to a constant, so `max` is not preserved through it.)
    #[test]
    fn a_broken_stage_yields_no_pipeline_law() {
        assert!(
            PipelineLaw::discover::<Broken>().is_empty(),
            "a non-homomorphic stage must not produce a composite law"
        );
        assert!(PipelineLaw::render::<Broken>().contains("no transform pipeline"));
    }

    /// Theories with no chained conversions have no pipeline law — arithmetic and the router have no
    /// unary transforms at all, so there is nothing to compose. No false positives.
    #[test]
    fn theories_without_pipelines_report_none() {
        assert!(PipelineLaw::discover::<Arithmetic>().is_empty());
        assert!(PipelineLaw::discover::<Router>().is_empty());
        assert!(PipelineLaw::render::<Router>().contains("no transform pipeline"));
    }

    /// `binary_on` requires ALL of: two inputs, both of the sort, output of the sort. Pins every
    /// conjunct — in particular a UNARY conversion is not binary even at its own output sort (which a
    /// too-permissive `||` would wrongly admit, then mis-use as a pipeline operator).
    #[test]
    fn binary_on_is_strict() {
        let ops = <Stages as Theory>::operators();
        let max_a = ops.iter().find(|o| o.symbol == "maxA").unwrap();
        let h_ab = ops.iter().find(|o| o.symbol == "hAB").unwrap();
        assert!(binary_on(max_a, S3::A), "maxA is binary on A");
        assert!(
            !binary_on(max_a, S3::B),
            "maxA is not binary on B (wrong sort)"
        );
        assert!(
            !binary_on(h_ab, S3::A),
            "hAB is unary, not binary on its input"
        );
        assert!(
            !binary_on(h_ab, S3::B),
            "hAB is unary, not binary on its output"
        );
    }
}
