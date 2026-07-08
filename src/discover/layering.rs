//!
//! layering — the OTHER selection pressure cohesion left on the table.
//!
//! `cohesion` (v9) answers one pathology: a module whose algebra is DISCONNECTED — several algebras
//! with no laws between them — wants to SPLIT. But a large algebra is not itself a smell (boolean
//! algebra is large and superb); the second pathology is a module that is *connected* but
//! SPRAWLING — held together only through a load-bearing operator, so it is really two tighter
//! sub-algebras hinged at one point. That wants to LAYER, not split.
//!
//! Read it off the same operator-interaction graph, structurally — no threshold. An operator is a
//! HINGE (a graph articulation point) when removing it DISCONNECTS its component: the rest of the
//! algebra only holds together *through* it. A component with no hinge is ATOMIC — one tight layer,
//! keep it. A component with hinges is LAYERED — the hinge is the seam between the sub-algebras it
//! joins, the natural place to introduce a layer. Like cohesion, it is a SUGGESTION a human (or an
//! agent) ratifies, surfaced through the same architect channel.

use std::collections::{BTreeSet, VecDeque};

use super::engine::{Engine, Term, Theory};

/// The layering analysis of one connected component of a theory's algebra.
pub struct ComponentLayering {
    /// The operator symbols in this component.
    pub operators: Vec<&'static str>,
    /// The HINGES — operators whose removal would disconnect the component (articulation points).
    pub hinges: Vec<&'static str>,
}

#[crate::mutate]
impl ComponentLayering {
    /// Atomic — one tight layer, no load-bearing operator to layer at.
    pub fn is_atomic(&self) -> bool {
        self.hinges.is_empty()
    }
}

#[cfg(test)]
impl std::fmt::Debug for ComponentLayering {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{ {} }} hinges {:?}",
            self.operators.join(", "),
            self.hinges
        )
    }
}

/// The layering analysis of a theory's whole discovered algebra.
pub struct LayeringReport {
    pub theory: &'static str,
    pub components: Vec<ComponentLayering>,
}

#[crate::mutate]
impl LayeringReport {
    /// Does any component sprawl — hold together only through a hinge, asking to be layered?
    pub fn wants_layering(&self) -> bool {
        self.components.iter().any(|c| !c.is_atomic())
    }

    /// Analyse a theory's discovered algebra for sprawl. The analysis is an associated
    /// function of its REPORT — the public surface is the value object, not a loose
    /// function (the no-rats-nest rule: every public callable hangs off a typestate).
    pub fn of<T: Theory>() -> Self {
        layering::<T>()
    }

    /// Render the layering report as a readable suggestion.
    pub fn render(&self) -> String {
        let mut out = format!("module `{}`: ", self.theory);
        if !self.wants_layering() {
            out.push_str("every component is atomic — no layering pressure.\n");
            return out;
        }
        out.push_str("some components sprawl — consider layering:\n");
        for (i, c) in self.components.iter().enumerate() {
            if c.is_atomic() {
                out.push_str(&format!(
                    "  component {i}: {{ {} }} — atomic\n",
                    c.operators.join(", ")
                ));
            } else {
                out.push_str(&format!(
                    "  component {i}: {{ {} }} — layered at hinge(s): {} (the algebra holds \
                     together only through them — layer there)\n",
                    c.operators.join(", "),
                    c.hinges.join(", ")
                ));
            }
        }
        out
    }
}

/// Collect the operator indices a term mentions.
#[crate::mutate]
fn ops_in(t: &Term, out: &mut BTreeSet<usize>) {
    if let Term::App(op, args) = t {
        out.insert(*op);
        for a in args {
            ops_in(a, out);
        }
    }
}

/// The operator-interaction graph: an adjacency set per operator, an undirected edge between two
/// operators whenever a discovered law mentions both.
#[crate::mutate]
fn interaction_graph<T: Theory>(n: usize) -> Vec<BTreeSet<usize>> {
    let mut adj = vec![BTreeSet::new(); n];
    for law in Engine::<T>::new().discover().laws {
        let mut ops = BTreeSet::new();
        ops_in(&law.lhs, &mut ops);
        ops_in(&law.rhs, &mut ops);
        let members: Vec<usize> = ops.into_iter().collect();
        // an UNDIRECTED edge between every distinct pair the law mentions (a clique over its
        // operators — the law ties them all together, not just consecutively).
        for &a in &members {
            for &b in &members {
                if a != b {
                    adj[a].insert(b);
                }
            }
        }
    }
    adj
}

/// The connected components of the graph (vertices reachable from each other), each as a sorted
/// vertex list, in ascending order of least vertex.
#[crate::mutate]
fn connected_components(adj: &[BTreeSet<usize>]) -> Vec<Vec<usize>> {
    let n = adj.len();
    let mut seen = vec![false; n];
    let mut comps = Vec::new();
    for start in 0..n {
        if seen[start] {
            continue;
        }
        let mut comp = Vec::new();
        let mut q = VecDeque::from([start]);
        seen[start] = true;
        while let Some(u) = q.pop_front() {
            comp.push(u);
            for &v in &adj[u] {
                if !seen[v] {
                    seen[v] = true;
                    q.push_back(v);
                }
            }
        }
        comp.sort_unstable();
        comps.push(comp);
    }
    comps
}

/// The ARTICULATION POINTS (hinges) of the graph — vertices whose removal increases the number of
/// connected components. Tarjan's DFS: a root is a hinge iff it has >1 DFS child; a non-root `u` is
/// a hinge iff some child `v` cannot reach an ancestor of `u` (`low[v] >= disc[u]`).
#[crate::mutate]
fn articulation_points(adj: &[BTreeSet<usize>]) -> BTreeSet<usize> {
    let n = adj.len();
    let mut disc = vec![usize::MAX; n];
    let mut low = vec![0usize; n];
    let mut hinges = BTreeSet::new();
    let mut timer = 0usize;
    for s in 0..n {
        if disc[s] == usize::MAX {
            dfs(
                s,
                usize::MAX,
                adj,
                &mut disc,
                &mut low,
                &mut timer,
                &mut hinges,
            );
        }
    }
    hinges
}

#[allow(clippy::too_many_arguments)]
#[crate::mutate]
fn dfs(
    u: usize,
    parent: usize,
    adj: &[BTreeSet<usize>],
    disc: &mut [usize],
    low: &mut [usize],
    timer: &mut usize,
    hinges: &mut BTreeSet<usize>,
) {
    disc[u] = *timer;
    low[u] = *timer;
    *timer += 1;
    let mut children = 0;
    for &v in &adj[u] {
        if disc[v] == usize::MAX {
            children += 1;
            dfs(v, u, adj, disc, low, timer, hinges);
            low[u] = low[u].min(low[v]);
            // a non-root hinge: a child cannot escape above `u`.
            if parent != usize::MAX && low[v] >= disc[u] {
                hinges.insert(u);
            }
        } else if v != parent {
            low[u] = low[u].min(disc[v]);
        }
    }
    // the root is a hinge iff the DFS branched into more than one subtree.
    if parent == usize::MAX && children > 1 {
        hinges.insert(u);
    }
}

/// Analyse a theory's discovered algebra for sprawl — per connected component, which operators are
/// hinges (load-bearing for connectivity). (Private — reached as `LayeringReport::of`.)
#[crate::mutate]
fn layering<T: Theory>() -> LayeringReport {
    let sigs = Engine::<T>::new().signatures();
    let adj = interaction_graph::<T>(sigs.len());
    let hinges = articulation_points(&adj);

    let components = connected_components(&adj)
        .into_iter()
        .map(|comp| ComponentLayering {
            operators: comp.iter().map(|&i| sigs[i].0).collect(),
            hinges: comp
                .iter()
                .filter(|i| hinges.contains(i))
                .map(|&i| sigs[i].0)
                .collect(),
        })
        .collect();

    LayeringReport {
        theory: T::name(),
        components,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discover::arithmetic::Arithmetic;
    use crate::discover::router::Router;

    fn component_with<'a>(r: &'a LayeringReport, op: &str) -> &'a ComponentLayering {
        r.components
            .iter()
            .find(|c| c.operators.contains(&op))
            .expect("operator present")
    }

    // A CHAIN of three `max` algebras on three sorts (A, B, C), bridged by two homomorphisms
    // (`hAB : A → B` and `hBC : B → C`). Each homomorphism law ties its three operators into a
    // triangle — {hAB, maxA, maxB} and {hBC, maxB, maxC} — and the two triangles SHARE `maxB`. So the
    // whole thing is ONE connected component (cohesion would say "cohesive, keep") that nonetheless
    // holds together only through `maxB`: the textbook sprawl cohesion cannot see. Pins the
    // articulation detector against a clean two-block barbell with a single, unambiguous hinge.
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
    enum S3 {
        A,
        B,
        C,
    }
    struct Chain;
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
        Chain : "chain", Value = (u8, i64), Obs = (u8, i64), Sort = S3,
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

    /// The CHAIN is one connected component that LAYERS at a single hinge — `maxB`, the operator both
    /// homomorphisms share. Removing it severs the A-side from the C-side. This is the case cohesion
    /// is blind to (it reports one cohesive component); layering names the seam. Pins the articulation
    /// point detector: exactly one hinge, and it is the shared operator, not either triangle's tips.
    #[test]
    fn the_chain_layers_at_the_shared_operator() {
        let r = LayeringReport::of::<Chain>();
        assert!(r.wants_layering(), "the chain sprawls through maxB");
        assert_eq!(r.components.len(), 1, "one connected component");
        let comp = &r.components[0];
        assert_eq!(comp.operators.len(), 5, "all five operators in it");
        assert_eq!(
            comp.hinges,
            vec!["maxB"],
            "the only hinge is the shared operator"
        );
        assert!(!comp.is_atomic());
    }

    /// The router is ATOMIC — its one component (`or`, `empty`) has no hinge, so there is no internal
    /// layer to peel. No false layering pressure on a genuinely tight module.
    #[test]
    fn the_router_is_atomic() {
        let r = LayeringReport::of::<Router>();
        assert!(!r.wants_layering(), "router is tight: {:?}", r.theory);
        assert!(r.components.iter().all(|c| c.is_atomic()));
        assert!(LayeringReport::of::<Router>()
            .render()
            .contains("no layering pressure"));
    }

    // `min` with a TOP element (identity, `min(2,x)=x`) and a BOTTOM element (annihilator,
    // `min(0,x)=0`) — two leaf constants hanging off one central binary operator. `min` is the first
    // operator, so the articulation DFS roots at it and finds TWO independent arms (top, bot): the
    // root-hinge case (`children > 1`) the chain's interior hinge does not exercise.
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
    enum One {
        N,
    }
    struct MinClamp;
    fn minop(v: &[i64]) -> Option<i64> {
        Some(v[0].min(v[1]))
    }
    fn top(_: &[i64]) -> Option<i64> {
        Some(2)
    }
    fn bot(_: &[i64]) -> Option<i64> {
        Some(0)
    }
    crate::theory! {
        MinClamp : "min clamp", Value = i64, Obs = i64, Sort = One,
        sort_of = |_: &i64| One::N,
        observe = |v: &i64| *v,
        vars { One::N => &["x"], }
        inhabit { One::N => vec![0, 1, 2], }
        ops {
            Infix   "min" "min" (One::N, One::N) -> One::N = minop;
            Nullary "top" "2"   () -> One::N = top;
            Nullary "bot" "0"   () -> One::N = bot;
        }
    }

    /// A central operator with two independent arms is a ROOT hinge: the articulation DFS starts at
    /// `min` and branches into the top arm and the bottom arm, neither reachable from the other except
    /// through `min`. Pins the root case of the detector (`children > 1`) — distinct from the chain's
    /// interior hinge, which exercises the non-root case.
    #[test]
    fn a_central_operator_with_two_arms_is_a_root_hinge() {
        let r = LayeringReport::of::<MinClamp>();
        assert!(r.wants_layering());
        assert_eq!(r.components.len(), 1, "one component: {:?}", r.components);
        assert_eq!(r.components[0].operators.len(), 3, "min, top, bot");
        assert_eq!(
            r.components[0].hinges,
            vec!["min"],
            "the central operator is the hinge"
        );
    }

    /// Arithmetic's ring component (`0`, `1`, `+`, `*`) holds together through `*` — distributivity
    /// links it to `+`, and `1` reaches the algebra only through it — so `*` is the hinge. The
    /// comparison component (`<`, `false`) is atomic. A real, threshold-free finding on a real theory.
    #[test]
    fn arithmetic_ring_hinges_on_multiplication() {
        let r = LayeringReport::of::<Arithmetic>();
        let ring = component_with(&r, "+");
        assert!(ring.operators.contains(&"*") && ring.operators.contains(&"1"));
        assert_eq!(ring.hinges, vec!["*"], "the ring hinges on multiplication");
        let compare = component_with(&r, "<");
        assert!(compare.is_atomic(), "comparison is atomic: {compare:?}");
        // and it renders the hinge as a readable suggestion.
        assert!(LayeringReport::of::<Arithmetic>()
            .render()
            .contains("layered at hinge(s): *"));
    }
}
