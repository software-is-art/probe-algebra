//! cohesion — a selection pressure toward well-factored modules, read off the discovered algebra.
//!
//! A badly-architected module is not one with a LARGE algebra (boolean algebra is large and superb)
//! — it is one with a DECOMPOSABLE algebra: secretly several algebras crammed together with no laws
//! connecting them. So build the OPERATOR-INTERACTION GRAPH — operators are nodes, and two are
//! linked whenever a discovered law mentions both (distributivity links `×`–`+`; De Morgan links
//! `not`–`and`–`or`; the action law links `add`–`plus`; a round trip links `since`–`at`). Its
//! weakly-connected components are the LATENT MODULES:
//!
//!   - one dense component  → cohesive; leave it as one module;
//!   - several components    → the module is several modules; the cut is where it wants to split.
//!
//! Each cut is a SEAM, classified by the A/C distinction: a shared sort with operators on both sides
//! is a TRANSPORT seam (the algebra stays — check with `coherence`); a conversion across the cut
//! (`since : Date → Duration`) is a TRANSFORM seam (the algebra changes — check with the
//! homomorphism law). This is a SUGGESTION, never a constraint — a modularity signal a human (or an
//! agent, naming as it goes) ratifies, like a quick-fix in an editor. It is self-applicable: run it
//! on the date calculus and it proposes splitting duration-arithmetic from epoch-conversion.

use std::collections::BTreeSet;

use super::engine::{Engine, Term, Theory};

/// What kind of seam a split would create.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SeamKind {
    /// The components share a sort with operators on both sides — the algebra stays (coherence).
    Transport,
    /// A conversion operator crosses the cut — the algebra changes (homomorphism).
    Transform,
}

/// A seam between two latent modules.
pub struct Seam {
    /// Indices into `CohesionReport::components`.
    pub left: usize,
    pub right: usize,
    /// The sorts the two components share (rendered).
    pub shared: Vec<String>,
    pub kind: SeamKind,
}

/// The cohesion analysis of a theory's discovered algebra.
pub struct CohesionReport {
    pub theory: &'static str,
    /// The latent modules — operator symbols grouped by interaction component.
    pub components: Vec<Vec<&'static str>>,
    /// The seams between them (empty when the module is cohesive — a single component).
    pub seams: Vec<Seam>,
}

impl CohesionReport {
    /// Is the module cohesive — a single algebra that should stay one module?
    pub fn is_cohesive(&self) -> bool {
        self.components.len() <= 1
    }
}

/// Collect the operator indices a term mentions.
fn ops_in(t: &Term, out: &mut BTreeSet<usize>) {
    if let Term::App(op, args) = t {
        out.insert(*op);
        for a in args {
            ops_in(a, out);
        }
    }
}

/// Union-find: the root of `x`, with path compression.
fn find(parent: &mut [usize], mut x: usize) -> usize {
    while parent[x] != x {
        parent[x] = parent[parent[x]];
        x = parent[x];
    }
    x
}

/// Analyse a theory's discovered algebra for decomposability.
pub fn cohesion<T: Theory>() -> CohesionReport {
    let engine = Engine::<T>::new();
    let sigs = engine.signatures();
    let n = sigs.len();

    // union operators that co-occur in a discovered law.
    let mut parent: Vec<usize> = (0..n).collect();
    for law in engine.discover().laws {
        let mut ops = BTreeSet::new();
        ops_in(&law.lhs, &mut ops);
        ops_in(&law.rhs, &mut ops);
        let members: Vec<usize> = ops.into_iter().collect();
        for pair in members.windows(2) {
            let (a, b) = (find(&mut parent, pair[0]), find(&mut parent, pair[1]));
            parent[a] = b;
        }
    }

    // group operator indices by component root, in a stable order.
    let mut roots: Vec<usize> = (0..n).map(|i| find(&mut parent, i)).collect();
    let mut order: Vec<usize> = roots.clone();
    order.sort_unstable();
    order.dedup();
    let components: Vec<Vec<usize>> = order
        .iter()
        .map(|&r| (0..n).filter(|&i| roots[i] == r).collect())
        .collect();
    roots.clear();

    // the sorts each component touches (inputs ∪ output of its operators).
    let comp_sorts: Vec<BTreeSet<T::Sort>> = components
        .iter()
        .map(|c| {
            let mut s = BTreeSet::new();
            for &i in c {
                s.extend(sigs[i].1.iter().copied());
                s.insert(sigs[i].2);
            }
            s
        })
        .collect();

    // a seam between any two components that share a sort; Transform if a conversion crosses it.
    let mut seams = Vec::new();
    for i in 0..components.len() {
        for j in (i + 1)..components.len() {
            let shared: BTreeSet<T::Sort> = comp_sorts[i]
                .intersection(&comp_sorts[j])
                .copied()
                .collect();
            if shared.is_empty() {
                continue;
            }
            let conversion = components[i]
                .iter()
                .chain(&components[j])
                .any(|&op| sigs[op].1.len() == 1 && sigs[op].1[0] != sigs[op].2);
            seams.push(Seam {
                left: i,
                right: j,
                shared: shared.iter().map(|s| format!("{s:?}")).collect(),
                kind: if conversion {
                    SeamKind::Transform
                } else {
                    SeamKind::Transport
                },
            });
        }
    }

    CohesionReport {
        theory: T::name(),
        components: components
            .iter()
            .map(|c| c.iter().map(|&i| sigs[i].0).collect())
            .collect(),
        seams,
    }
}

/// Render the cohesion report as a readable suggestion.
pub fn render<T: Theory>() -> String {
    let r = cohesion::<T>();
    let mut out = format!("module `{}`: ", r.theory);
    if r.is_cohesive() {
        out.push_str("cohesive — one algebra, keep as one module.\n");
        return out;
    }
    out.push_str(&format!(
        "decomposes into {} latent modules — consider splitting:\n",
        r.components.len()
    ));
    for (i, c) in r.components.iter().enumerate() {
        out.push_str(&format!("  module {}: {{ {} }}\n", i, c.join(", ")));
    }
    for s in &r.seams {
        let kind = match s.kind {
            SeamKind::Transport => "transport (algebra stays — check coherence)",
            SeamKind::Transform => "transform (algebra changes — check the homomorphism)",
        };
        out.push_str(&format!(
            "  seam {}↔{} on {} — {}\n",
            s.left,
            s.right,
            s.shared.join(", "),
            kind
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discover::arithmetic::Arithmetic;
    use crate::discover::date::Calendar;
    use crate::discover::router::Router;

    fn component_of<'a>(r: &'a CohesionReport, op: &str) -> &'a Vec<&'static str> {
        r.components
            .iter()
            .find(|c| c.contains(&op))
            .expect("operator present")
    }

    // Three independent monoids on three DIFFERENT sorts — no operator crosses sorts, so the algebra
    // has three disconnected components and no seams. Used to pin the partition (with two components
    // a grouping bug merely swaps their order; with three it makes operators appear in two at once).
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
    enum S3 {
        A,
        B,
        C,
    }
    struct Triple;
    fn za(_: &[(u8, i64)]) -> Option<(u8, i64)> {
        Some((0, 0))
    }
    fn zb(_: &[(u8, i64)]) -> Option<(u8, i64)> {
        Some((1, 0))
    }
    fn zc(_: &[(u8, i64)]) -> Option<(u8, i64)> {
        Some((2, 0))
    }
    fn ma(v: &[(u8, i64)]) -> Option<(u8, i64)> {
        Some((0, v[0].1.max(v[1].1)))
    }
    fn mb(v: &[(u8, i64)]) -> Option<(u8, i64)> {
        Some((1, v[0].1.max(v[1].1)))
    }
    fn mc(v: &[(u8, i64)]) -> Option<(u8, i64)> {
        Some((2, v[0].1.max(v[1].1)))
    }
    crate::theory! {
        Triple : "triple", Value = (u8, i64), Obs = (u8, i64), Sort = S3,
        sort_of = |v: &(u8, i64)| match v.0 { 0 => S3::A, 1 => S3::B, _ => S3::C },
        observe = |v: &(u8, i64)| *v,
        vars { S3::A => &["a"], S3::B => &["b"], S3::C => &["c"], }
        inhabit {
            S3::A => vec![(0, 0), (0, 1), (0, 2)],
            S3::B => vec![(1, 0), (1, 1), (1, 2)],
            S3::C => vec![(2, 0), (2, 1), (2, 2)],
        }
        ops {
            Nullary "zA" "zA" () -> S3::A = za;
            Infix   "mA" "mA" (S3::A, S3::A) -> S3::A = ma;
            Nullary "zB" "zB" () -> S3::B = zb;
            Infix   "mB" "mB" (S3::B, S3::B) -> S3::B = mb;
            Nullary "zC" "zC" () -> S3::C = zc;
            Infix   "mC" "mC" (S3::C, S3::C) -> S3::C = mc;
        }
    }

    /// THREE disconnected components, each operator in EXACTLY one, and no seams (disjoint sorts).
    /// Pins the component grouping: a flipped membership test would put each operator in two
    /// components at once (with three parts, a part's complement is not a single other part).
    #[test]
    fn three_independent_monoids_give_three_components() {
        let r = cohesion::<Triple>();
        assert_eq!(r.components.len(), 3, "components: {:?}", r.components);
        let total: usize = r.components.iter().map(|c| c.len()).sum();
        assert_eq!(
            total, 6,
            "each of the 6 operators must appear in exactly one component"
        );
        assert!(
            r.seams.is_empty(),
            "disjoint sorts ⇒ no seam: {:?}",
            r.seams.len()
        );
    }

    /// The router is COHESIVE: `or` and `empty` interact (the identity law links them), so the whole
    /// module is a single algebra — no split suggested.
    #[test]
    fn the_router_is_cohesive() {
        let r = cohesion::<Router>();
        assert!(r.is_cohesive(), "router is one algebra: {:?}", r.components);
        assert!(r.seams.is_empty());
    }

    /// Arithmetic DECOMPOSES into the arithmetic ring and the comparison — no law links `<` to `+`
    /// or `*`. They share the `Int` sort with operators on both sides, so the seam is TRANSPORT.
    #[test]
    fn arithmetic_splits_into_arithmetic_and_comparison() {
        let r = cohesion::<Arithmetic>();
        assert_eq!(r.components.len(), 2, "components: {:?}", r.components);
        // `+`, `*`, `0`, `1` cluster; `<` and `false` form the other cluster.
        let arith = component_of(&r, "+");
        assert!(arith.contains(&"*") && arith.contains(&"0") && arith.contains(&"1"));
        assert!(!arith.contains(&"<"));
        let compare = component_of(&r, "<");
        assert!(compare.contains(&"false"));
        assert_eq!(r.seams.len(), 1);
        assert_eq!(r.seams[0].kind, SeamKind::Transport);
        assert_eq!(r.seams[0].shared, vec!["Int".to_string()]);
    }

    /// The date calculus DECOMPOSES into duration arithmetic and epoch conversion — `since`/`at`
    /// are linked to each other (the round trip) but to nothing else. They convert Date↔Duration,
    /// so the seam is TRANSFORM — the layer line the architecture wants.
    #[test]
    fn date_splits_into_arithmetic_and_conversion() {
        let r = cohesion::<Calendar>();
        assert_eq!(r.components.len(), 2, "components: {:?}", r.components);
        let conversion = component_of(&r, "since");
        assert_eq!(
            conversion.len(),
            2,
            "conversion = {{since, at}}: {conversion:?}"
        );
        assert!(conversion.contains(&"at"));
        let arithmetic = component_of(&r, "+");
        assert!(arithmetic.contains(&"add") && arithmetic.contains(&"diff"));
        assert_eq!(r.seams.len(), 1);
        assert_eq!(r.seams[0].kind, SeamKind::Transform);
    }

    /// The report renders as a readable suggestion — naming the latent modules and the seam kind.
    #[test]
    fn the_report_renders_readably() {
        let text = render::<Calendar>();
        assert!(text.contains("decomposes into 2 latent modules"));
        assert!(text.contains("transform"));
        assert!(render::<Router>().contains("cohesive"));
    }
}
