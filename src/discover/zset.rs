/// The number of distinct keys in the bounded universe the theory observes.
const KEYS: usize = 3;

/// The number of ticks in a bounded trace — long enough that integrate/differentiate
/// have interior structure, short enough that the grid stays a grid.
const TICKS: usize = 3;

/// A Z-set: finitely many keys, each with a nonzero signed weight — the one abelian
/// group under the whole incremental program (a delta IS a Z-set; a relation is a
/// Z-set of weight-positive rows). CANONICAL by construction: a zero weight is
/// absence, enforced at every constructor, so structural equality is observational
/// equality and the group laws are discoverable by running the operators.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct ZSet<K: Ord>(std::collections::BTreeMap<K, i64>);

#[crate::mutate]
impl<K: Ord + Clone> ZSet<K> {
    /// The group identity: no keys.
    pub fn zero() -> ZSet<K> {
        ZSet(std::collections::BTreeMap::new())
    }

    /// A Z-set from weighted pairs; zero weights vanish, duplicate keys sum.
    pub fn from_pairs(pairs: &[(K, i64)]) -> ZSet<K> {
        let mut out = ZSet::zero();
        for (k, w) in pairs {
            out.put(k.clone(), *w);
        }
        out
    }

    /// Add `w` to the key's weight, erasing the entry when it reaches zero — the
    /// canonicality invariant lives here and only here.
    pub fn put(&mut self, k: K, w: i64) {
        let next = self.weight(&k) + w;
        if next == 0 {
            self.0.remove(&k);
        } else {
            self.0.insert(k, next);
        }
    }

    /// The key's weight; absence is zero.
    pub fn weight(&self, k: &K) -> i64 {
        self.0.get(k).copied().unwrap_or(0)
    }

    /// Group addition: pointwise weight sum.
    pub fn add(&self, other: &ZSet<K>) -> ZSet<K> {
        let mut out = self.clone();
        for (k, w) in &other.0 {
            out.put(k.clone(), *w);
        }
        out
    }

    /// Group inverse: pointwise negation.
    pub fn neg(&self) -> ZSet<K> {
        ZSet(self.0.iter().map(|(k, w)| (k.clone(), -w)).collect())
    }

    /// The same-key join: pointwise weight PRODUCT — the bilinear operator, and the
    /// reason deltas are cheap (the product rule). Relational join with distinct
    /// key structure arrives with the indexed form; this is its multiplicity core.
    pub fn join(&self, other: &ZSet<K>) -> ZSet<K> {
        let mut out = ZSet::zero();
        for (k, w) in &self.0 {
            let product = w * other.weight(k);
            if product != 0 {
                out.put(k.clone(), product);
            }
        }
        out
    }

    /// The set face of the multiset: positive weight becomes one, everything else
    /// vanishes — the one non-linear operator, where integration pays its way.
    pub fn distinct(&self) -> ZSet<K> {
        ZSet(
            self.0
                .iter()
                .filter(|(_, w)| **w > 0)
                .map(|(k, _)| (k.clone(), 1))
                .collect(),
        )
    }

    /// The entries in key order — the observation the theory folds into its table.
    pub fn entries(&self) -> Vec<(K, i64)> {
        self.0.iter().map(|(k, w)| (k.clone(), *w)).collect()
    }
}

/// A value in the kernel's theory: a Z-set over the bounded key universe, or a
/// bounded trace of them (a stream window, TICKS long).
#[derive(Clone)]
pub enum Flow {
    Zset(ZSet<u8>),
    Window(Vec<ZSet<u8>>),
}

/// The Z-set kernel theory.
pub struct ZKernel;

/// Two sorts: Z-sets and traces of Z-sets.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Sort {
    Zset,
    Window,
}

#[crate::mutate]
fn zpart(v: &Flow) -> ZSet<u8> {
    match v {
        Flow::Zset(z) => z.clone(),
        Flow::Window(t) => t.first().cloned().unwrap_or_else(ZSet::zero),
    }
}

#[crate::mutate]
fn tpart(v: &Flow) -> Vec<ZSet<u8>> {
    match v {
        Flow::Window(t) => t.clone(),
        Flow::Zset(z) => vec![z.clone(); TICKS],
    }
}

#[crate::mutate]
fn zero(_: &[Flow]) -> Option<Flow> {
    Some(Flow::Zset(ZSet::zero()))
}

#[crate::mutate]
fn add(v: &[Flow]) -> Option<Flow> {
    Some(Flow::Zset(zpart(&v[0]).add(&zpart(&v[1]))))
}

#[crate::mutate]
fn neg(v: &[Flow]) -> Option<Flow> {
    Some(Flow::Zset(zpart(&v[0]).neg()))
}

#[crate::mutate]
fn join(v: &[Flow]) -> Option<Flow> {
    Some(Flow::Zset(zpart(&v[0]).join(&zpart(&v[1]))))
}

#[crate::mutate]
fn distinct(v: &[Flow]) -> Option<Flow> {
    Some(Flow::Zset(zpart(&v[0]).distinct()))
}

/// z⁻¹ — the delay: the window shifts one tick, zero enters at the front.
#[crate::mutate]
fn delay(v: &[Flow]) -> Option<Flow> {
    let t = tpart(&v[0]);
    let mut out = vec![ZSet::zero()];
    out.extend(t.iter().take(TICKS - 1).cloned());
    Some(Flow::Window(out))
}

/// I — the running sum: each tick is the group sum of everything up to it.
#[crate::mutate]
fn integrate(v: &[Flow]) -> Option<Flow> {
    let t = tpart(&v[0]);
    let mut acc = ZSet::zero();
    let mut out = Vec::with_capacity(TICKS);
    for z in &t {
        acc = acc.add(z);
        out.push(acc.clone());
    }
    Some(Flow::Window(out))
}

/// D — the difference: each tick minus the tick before it (the delayed window),
/// which is the group form of "what changed".
#[crate::mutate]
fn differentiate(v: &[Flow]) -> Option<Flow> {
    let t = tpart(&v[0]);
    let mut prev = ZSet::zero();
    let mut out = Vec::with_capacity(TICKS);
    for z in &t {
        out.push(z.add(&prev.neg()));
        prev = z.clone();
    }
    Some(Flow::Window(out))
}

/// The Z-set spread: zero, singletons, negatives, and overlapping multi-key sets,
/// so the grid can refute a false law (a distinct that pretended linearity, an
/// inverse that only held on positives).
#[crate::mutate]
fn zsets() -> Vec<Flow> {
    vec![
        Flow::Zset(ZSet::zero()),
        Flow::Zset(ZSet::from_pairs(&[(0, 1)])),
        Flow::Zset(ZSet::from_pairs(&[(0, -1)])),
        Flow::Zset(ZSet::from_pairs(&[(1, 2)])),
        Flow::Zset(ZSet::from_pairs(&[(0, 1), (1, -2)])),
        Flow::Zset(ZSet::from_pairs(&[(1, 1), (2, 3)])),
    ]
}

/// The trace spread: built from the same Z-sets — constant, impulse, alternating —
/// so the stream laws are judged over windows with real interior differences.
#[crate::mutate]
fn traces() -> Vec<Flow> {
    let a = ZSet::from_pairs(&[(0, 1)]);
    let b = ZSet::from_pairs(&[(1, 2)]);
    let c = ZSet::from_pairs(&[(0, -1), (2, 1)]);
    vec![
        Flow::Window(vec![ZSet::zero(); TICKS]),
        Flow::Window(vec![a.clone(), ZSet::zero(), ZSet::zero()]),
        Flow::Window(vec![ZSet::zero(), b.clone(), ZSet::zero()]),
        Flow::Window(vec![a.clone(), b.clone(), c.clone()]),
        Flow::Window(vec![c, a, b]),
    ]
}

// The whole two-sorted `Theory` impl is generated from this block — the kernel type
// (`ZSet`) and the operator functions above are the only authored content.
crate::theory! {
    ZKernel : "zset kernel", Value = Flow, Obs = (u8, Vec<i64>), Sort = Sort,
    sort_of = |v: &Flow| match v {
        Flow::Zset(_) => Sort::Zset,
        Flow::Window(_) => Sort::Window,
    },
    observe = |v: &Flow| match v {
        Flow::Zset(z) => (0u8, (0..KEYS as u8).map(|k| z.weight(&k)).collect()),
        Flow::Window(t) => (
            1u8,
            t.iter()
                .flat_map(|z| (0..KEYS as u8).map(|k| z.weight(&k)).collect::<Vec<_>>())
                .collect(),
        ),
    },
    vars {
        Sort::Zset => &["a", "b", "c"],
        Sort::Window => &["s", "t", "u"],
    }
    inhabit {
        Sort::Zset => zsets(),
        Sort::Window => traces(),
    }
    ops {
        Nullary "Zero"          "zero"          () -> Sort::Zset = zero;
        Infix   "Add"           "+"             (Sort::Zset, Sort::Zset) -> Sort::Zset = add;
        Prefix  "Neg"           "neg"           (Sort::Zset) -> Sort::Zset = neg;
        Infix   "Join"          "join"          (Sort::Zset, Sort::Zset) -> Sort::Zset = join;
        Prefix  "Distinct"      "distinct"      (Sort::Zset) -> Sort::Zset = distinct;
        Prefix  "Delay"         "delay"         (Sort::Window) -> Sort::Window = delay;
        Prefix  "Integrate"     "integrate"     (Sort::Window) -> Sort::Window = integrate;
        Prefix  "Differentiate" "differentiate" (Sort::Window) -> Sort::Window = differentiate;
        Prefix  "Impulse"       "impulse"       (Sort::Zset) -> Sort::Window = impulse;
        Prefix  "Latest"        "latest"        (Sort::Window) -> Sort::Zset = latest;
    }
}

/// A delta entering the stream: zero everywhere except NOW (the window's last
/// tick) — how a transaction's change arrives in the kernel.
#[crate::mutate]
fn impulse(v: &[Flow]) -> Option<Flow> {
    let mut out = vec![ZSet::zero(); TICKS];
    out[TICKS - 1] = zpart(&v[0]);
    Some(Flow::Window(out))
}

/// The stream's current value: the window's last tick — how eduction reads a
/// maintained view out of the kernel.
#[crate::mutate]
fn latest(v: &[Flow]) -> Option<Flow> {
    Some(Flow::Zset(
        tpart(&v[0]).last().cloned().unwrap_or_else(ZSet::zero),
    ))
}

#[cfg(test)]
mod probes {
    use super::*;
    use crate::discover::engine::{Engine, Theory};

    /// The engine discovers the kernel's whole algebra by running it: the abelian
    /// group (with INVERSE — the law that makes differentiation exist), negation as
    /// an Add-homomorphism, join's ring face (commutative, associative, annihilated
    /// by zero, distributing over Add — bilinearity's one-sided witness), distinct
    /// as a projection, and the DBSP theorem itself — I∘D = id and D∘I = id — with
    /// delay commuting past both and the injection/observation round trip closing
    /// the two sorts into one module. This pin is the lib-side twin of
    /// spec/zset-kernel.spec.
    #[test]
    fn the_kernel_algebra_is_discovered() {
        assert_eq!(ZKernel::name(), "zset kernel");
        let d = Engine::<ZKernel>::new().discover();
        let equations: Vec<&str> = d.laws.iter().map(|l| l.equation.as_str()).collect();
        for law in [
            "(a + b) = (b + a)",
            "((a + b) + c) = (a + (b + c))",
            "(zero + a) = a",
            "(a + neg(a)) = zero",
            "neg(neg(a)) = a",
            "neg((a + b)) = (neg(a) + neg(b))",
            "(a join b) = (b join a)",
            "(zero join a) = zero",
            "(a join (b + c)) = ((a join b) + (a join c))",
            "distinct(distinct(a)) = distinct(a)",
            "integrate(differentiate(s)) = s",
            "differentiate(integrate(s)) = s",
            "delay(integrate(s)) = integrate(delay(s))",
            "latest(impulse(a)) = a",
        ] {
            assert!(
                equations.contains(&law),
                "missing law: {law}\nhave: {equations:#?}"
            );
        }
    }

    /// LINEARITY, the shape the law language cannot yet state across sorts: over the
    /// full inhabitant grid, differentiate, integrate, and delay each commute with
    /// pointwise trace addition, and join's delta expands by the product rule —
    /// Δ(a⋈b) = Δa⋈b + a⋈Δb + Δa⋈Δb. These are the theorems the incremental
    /// executor will lean on; they pin here until the catalog can say them.
    #[test]
    fn the_operators_earn_their_delta_shortcuts() {
        let zs: Vec<ZSet<u8>> = zsets()
            .into_iter()
            .map(|f| match f {
                Flow::Zset(z) => z,
                Flow::Window(_) => unreachable!(),
            })
            .collect();
        let ts: Vec<Vec<ZSet<u8>>> = traces()
            .into_iter()
            .map(|f| match f {
                Flow::Window(t) => t,
                Flow::Zset(_) => unreachable!(),
            })
            .collect();
        let tadd = |s: &[ZSet<u8>], t: &[ZSet<u8>]| -> Vec<ZSet<u8>> {
            s.iter().zip(t).map(|(a, b)| a.add(b)).collect()
        };
        let run = |op: fn(&[Flow]) -> Option<Flow>, t: &[ZSet<u8>]| -> Vec<ZSet<u8>> {
            match op(&[Flow::Window(t.to_vec())]) {
                Some(Flow::Window(out)) => out,
                _ => unreachable!(),
            }
        };
        for s in &ts {
            for t in &ts {
                for op in [differentiate, integrate, delay] {
                    assert_eq!(
                        run(op, &tadd(s, t)),
                        tadd(&run(op, s), &run(op, t)),
                        "a stream operator claimed linearity it does not have"
                    );
                }
            }
        }
        for a in &zs {
            for da in &zs {
                for b in &zs {
                    for db in &zs {
                        let whole = a.add(da).join(&b.add(db));
                        let parts = a
                            .join(b)
                            .add(&da.join(b))
                            .add(&a.join(db))
                            .add(&da.join(db));
                        assert_eq!(whole, parts, "the product rule failed");
                    }
                }
            }
        }
    }

    /// The carrier's canonicality is load-bearing: a zero weight is absence (so
    /// structural equality is observational equality), duplicate keys sum, absence
    /// reads as zero, and entries come out in key order.
    #[test]
    fn the_carrier_is_canonical_by_construction() {
        let z = ZSet::from_pairs(&[(2u8, 1), (0, 2), (2, -1), (1, 0)]);
        assert_eq!(z.entries(), vec![(0, 2)], "zeros vanish, duplicates sum");
        assert_eq!(z.weight(&2), 0, "absence is zero");
        let mut w = ZSet::zero();
        w.put(3, 5);
        w.put(3, -5);
        assert_eq!(w, ZSet::zero(), "a weight returning to zero erases its key");
        assert_eq!(
            ZSet::from_pairs(&[(1u8, -2)]).distinct(),
            ZSet::zero(),
            "distinct drops non-positive weight"
        );
        assert_eq!(
            ZSet::from_pairs(&[(1u8, 3)]).distinct().entries(),
            vec![(1, 1)],
            "distinct caps positive weight at one"
        );
    }
}
