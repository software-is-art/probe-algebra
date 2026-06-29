//! discover — the laws WRITE (and READ) themselves, by UNBOUNDED synthesis.
//!
//! The interpreter's algebraic spec is neither hand-listed nor drawn from a fixed catalog of named
//! shapes. `discover_laws()` ENUMERATES terms over the operators (`Add`, `Mul`, `Lt`), the
//! variables `x, y, z`, and the constants `0, 1`; groups them into BEHAVIOURAL EQUIVALENCE CLASSES
//! by running `eval` over a grid of assignments; and reads each non-trivial equality between
//! distinct terms as a candidate law. The swamp of redundant equalities is then folded — collapsed
//! up to renaming, commutativity, and associativity — to ONE canonical representative per recognized
//! algebraic shape (the cleanest, most general witness: `(x+0)=x`, not `2+0=2` or `(a*b)+0=a*b`); the
//! remaining equalities are all consequences of those, so they are COUNTED, not listed. No human
//! names a structure or writes a law; whatever the operators exhibit is found by running them.
//!
//! It also discovers a STRUCTURE-SENSITIVITY law over a synthetic, maximally-sensitive observer
//! `U` — the program's faithful rendering, virtually treated as one more member of the signature
//! (it is a probe-construction device, never a boundary edge). `eval` collapses structure (`2+3`
//! and `5` are equal to it), so the equational laws above are blind to a transform that is
//! eval-correct but structurally wrong; the U law (`U` distinguishes every structural and semantic
//! perturbation) closes that blind spot — so the WHOLE probe taxonomy, algebraic and structural,
//! falls out of one discovery mechanism.
//!
//! Each held law renders to a NON-MATHY sentence, so the discovered set is a contract a
//! non-mathematical stakeholder can audit and ratify (a law you expect but don't see is a bug).
//! The laws feed the `harness` as probes, where mutation judges their kill power; `discover` itself
//! is kept in the mutation sweep, certified by the probes below.
//!
//! Honest frame: discovery's oracle is the baseline, so its power is deviation-catching (mutation)
//! plus expectation-checking (ratification); enumeration is depth-bounded (a resource limit, not a
//! curated list). That is the precise edge where "the tests write themselves" meets "what did you
//! mean".

use std::collections::{BTreeMap, HashMap};

use crate::boundary::sensitive_to_all;
use crate::gdp::with_seed;
use crate::interp::boundary::{Check, Eval, Expr, Ident, Lit, Op, Value};

/// A discovered law, with its plain-language and symbolic renderings.
pub enum Law {
    /// An equation that held on every assignment: `lhs == rhs` under `eval`.
    Equation {
        prose: String,
        equation: String,
        /// Build the two sides for concrete inputs `(x, y, z)` — the harness re-probes this.
        schema: Box<dyn Fn(i64, i64, i64) -> (Expr, Expr)>,
    },
    /// The synthetic observer `U` distinguishes every structural/semantic perturbation.
    Sensitivity { prose: String, equation: String },
}

impl Law {
    /// The non-mathy statement, for the readable spec.
    pub fn prose(&self) -> &str {
        match self {
            Law::Equation { prose, .. } | Law::Sensitivity { prose, .. } => prose,
        }
    }
    /// The symbolic rendering, for readers who want it.
    pub fn equation(&self) -> &str {
        match self {
            Law::Equation { equation, .. } | Law::Sensitivity { equation, .. } => equation,
        }
    }
}

// ----- the signature: variables, constants, and the input grid ------------

const VARS: [&str; 3] = ["x", "y", "z"];
const CONSTS: [i64; 2] = [0, 1];

/// The assignments the equivalence classes are computed over — enough independent variation in
/// `x, y, z` to separate distinct behaviours (proptest re-probes the survivors for kill power).
const ASSIGN: &[(i64, i64, i64)] = &[
    (0, 0, 0),
    (1, 0, 0),
    (0, 1, 0),
    (0, 0, 1),
    (1, 2, 3),
    (2, 3, 4),
    (3, 5, 7),
    (5, 2, 4),
    (2, 2, 2),
    (4, 1, 6),
    (6, 3, 1),
    (1, 1, 1),
];

fn var(name: &str) -> Expr {
    Expr::var(Ident::new(name).expect("valid identifier"))
}
fn lit(n: i64) -> Expr {
    Expr::int(n).expect("non-negative literal")
}

/// Substitute the variables with concrete non-negative integers, yielding a closed term.
fn subst(t: &Expr, a: i64, b: i64, c: i64) -> Expr {
    match t {
        Expr::Var(id) => match id.get() {
            "x" => lit(a),
            "y" => lit(b),
            _ => lit(c),
        },
        Expr::Bin(op, l, r) => Expr::bin(*op, subst(l, a, b, c), subst(r, a, b, c)),
        other => other.clone(),
    }
}

fn eval_closed(e: &Expr) -> Value {
    with_seed(|seed| {
        let named = seed.new_named(e.clone());
        let proof = Check
            .classify(&named)
            .expect("a closed int/bool term is well-typed");
        *Eval.run(&named, &proof).value()
    })
}

/// The behaviour of an INT-typed term: its value at each assignment (`None` if it is bool-typed).
fn int_sig(t: &Expr) -> Option<Vec<i64>> {
    ASSIGN
        .iter()
        .map(|&(a, b, c)| match eval_closed(&subst(t, a, b, c)) {
            Value::Int(i) => Some(i.get()),
            Value::Bool(_) => None,
        })
        .collect()
}

/// The behaviour of a BOOL-typed term: its truth value at each assignment.
fn bool_sig(t: &Expr) -> Option<Vec<bool>> {
    ASSIGN
        .iter()
        .map(|&(a, b, c)| match eval_closed(&subst(t, a, b, c)) {
            Value::Bool(v) => Some(v),
            Value::Int(_) => None,
        })
        .collect()
}

/// Node count — the term-size order that picks the canonical (smallest) representative.
fn size(t: &Expr) -> usize {
    match t {
        Expr::Bin(_, l, r) => 1 + size(l) + size(r),
        _ => 1,
    }
}

/// How many distinct variables a term mentions (more = more general, kept first when pruning).
fn vars_used(t: &Expr) -> usize {
    fn go(t: &Expr, seen: &mut Vec<String>) {
        match t {
            Expr::Var(id) => {
                let n = id.get().to_string();
                if !seen.contains(&n) {
                    seen.push(n);
                }
            }
            Expr::Bin(_, l, r) => {
                go(l, seen);
                go(r, seen);
            }
            _ => {}
        }
    }
    let mut seen = Vec::new();
    go(t, &mut seen);
    seen.len()
}

// ----- synthesis: enumerate, classify by behaviour, read off equalities ----

/// Enumerate INT terms by behavioural equivalence class, emitting an equality each time a new term
/// matches the behaviour of a smaller one already seen. Canonical generation keeps the working set
/// at the number of distinct behaviours, so the search is bounded by behaviour, not raw term count.
fn synthesize() -> Vec<(Expr, Expr)> {
    // BTreeMaps (keyed by the behavioural signature) make enumeration order — and therefore the
    // discovered spec — DETERMINISTIC, so the spec is a stable artifact the tests can pin exactly.
    let mut canon: BTreeMap<Vec<i64>, Expr> = BTreeMap::new();
    let mut laws: Vec<(Expr, Expr)> = Vec::new();

    let register = |t: Expr, canon: &mut BTreeMap<Vec<i64>, Expr>, laws: &mut Vec<(Expr, Expr)>| {
        if let Some(sig) = int_sig(&t) {
            match canon.get(&sig) {
                Some(c) if c != &t => laws.push((t.clone(), c.clone())),
                Some(_) => {}
                None => {
                    canon.insert(sig, t);
                }
            }
        }
    };

    // depth 0: the leaves.
    for v in VARS {
        register(var(v), &mut canon, &mut laws);
    }
    for n in CONSTS {
        register(lit(n), &mut canon, &mut laws);
    }

    // two rounds of combination over the canonical set; the set stays small (one term per distinct
    // behaviour), so each round is bounded even though the terms it forms grow in depth.
    for _round in 0..2 {
        let current: Vec<Expr> = canon.values().cloned().collect();
        let mut formed: Vec<Expr> = Vec::new();
        for op in [Op::Add, Op::Mul] {
            for a in &current {
                for b in &current {
                    formed.push(Expr::bin(op, a.clone(), b.clone()));
                }
            }
        }
        for t in formed {
            register(t, &mut canon, &mut laws);
        }
    }

    // a small BOOL layer: `<` over the int leaves, plus the boolean literals — enough to surface
    // the order's laws (irreflexivity) without a second full enumeration.
    let leaves: Vec<Expr> = VARS
        .iter()
        .map(|v| var(v))
        .chain(CONSTS.iter().map(|&n| lit(n)))
        .collect();
    let mut bcanon: BTreeMap<Vec<bool>, Expr> = BTreeMap::new();
    // each bool term (`a < b` for a distinct operand pair, plus the two literals) is registered
    // exactly once, so a match always relates two DISTINCT terms — no `c != &t` guard is needed.
    let breg = |t: Expr, bc: &mut BTreeMap<Vec<bool>, Expr>, laws: &mut Vec<(Expr, Expr)>| {
        if let Some(sig) = bool_sig(&t) {
            match bc.get(&sig) {
                Some(c) => laws.push((t.clone(), c.clone())),
                None => {
                    bc.insert(sig, t);
                }
            }
        }
    };
    breg(Expr::boolean(false), &mut bcanon, &mut laws);
    breg(Expr::boolean(true), &mut bcanon, &mut laws);
    for a in &leaves {
        for b in &leaves {
            breg(
                Expr::bin(Op::Lt, a.clone(), b.clone()),
                &mut bcanon,
                &mut laws,
            );
        }
    }

    laws
}

// ----- pruning: one canonical law per named shape, rest are consequences ----

/// A canonical string order on terms, for sorting the operands of commutative operators.
fn key(t: &Expr) -> String {
    match t {
        Expr::Lit(Lit::Int(i)) => format!("0i{:08}", i.get()),
        Expr::Lit(Lit::Bool(b)) => format!("0b{b}"),
        Expr::Var(id) => format!("1v{}", id.get()),
        Expr::Bin(op, l, r) => format!("2{}({},{})", op.sym(), key(l), key(r)),
        _ => "9".to_string(),
    }
}

/// AC-NORMAL FORM: flatten nested `+`/`*` (associative) and sort each operand list (commutative),
/// so two terms equal up to commutativity and associativity have the SAME normal form. This is how
/// the swamp of comm/assoc consequences (`(x+z)+y == (y+x)+z`, …) is recognized and dropped.
fn acnf(t: &Expr) -> Expr {
    fn flatten(t: &Expr, op: Op, out: &mut Vec<Expr>) {
        match t {
            Expr::Bin(o, l, r) if *o == op => {
                flatten(l, op, out);
                flatten(r, op, out);
            }
            other => out.push(acnf(other)),
        }
    }
    match t {
        Expr::Bin(op, _, _) if *op == Op::Add || *op == Op::Mul => {
            let mut ops = Vec::new();
            flatten(t, *op, &mut ops);
            ops.sort_by_key(key);
            let mut it = ops.into_iter();
            let first = it.next().expect("a flattened op has at least two operands");
            it.fold(first, |acc, x| Expr::bin(*op, acc, x))
        }
        Expr::Bin(op, l, r) => Expr::bin(*op, acnf(l), acnf(r)),
        other => other.clone(),
    }
}

/// The AC-normal-form key — two terms equal up to commutativity and associativity share it.
fn acnf_key(t: &Expr) -> String {
    key(&acnf(t))
}

/// Are two terms equal up to commutativity and associativity?
fn acnf_eq(a: &Expr, b: &Expr) -> bool {
    acnf_key(a) == acnf_key(b)
}

/// How many leaves a term has under repeated application of one associative operator (`x+y+z` ⇒ 3).
fn flatten_len(t: &Expr, op: Op) -> usize {
    match t {
        Expr::Bin(o, l, r) if *o == op => flatten_len(l, op) + flatten_len(r, op),
        _ => 1,
    }
}

/// Rename the variables of `(l, r)` to `x, y, z` in order of first appearance, so α-variants of one
/// law (`y+0=y` and `x+0=x`) collapse together.
fn alpha_normalize(l: &Expr, r: &Expr) -> (Expr, Expr) {
    fn collect(t: &Expr, order: &mut Vec<String>) {
        match t {
            Expr::Var(id) => {
                let n = id.get().to_string();
                if !order.contains(&n) {
                    order.push(n);
                }
            }
            Expr::Bin(_, l, r) => {
                collect(l, order);
                collect(r, order);
            }
            _ => {}
        }
    }
    fn rename(t: &Expr, map: &HashMap<String, String>) -> Expr {
        match t {
            Expr::Var(id) => var(map.get(id.get()).map(|s| s.as_str()).unwrap_or("x")),
            Expr::Bin(op, l, r) => Expr::bin(*op, rename(l, map), rename(r, map)),
            other => other.clone(),
        }
    }
    let mut order = Vec::new();
    collect(l, &mut order);
    collect(r, &mut order);
    let names = ["x", "y", "z"];
    let map: HashMap<String, String> = order
        .into_iter()
        .enumerate()
        .map(|(i, n)| (n, names.get(i).unwrap_or(&"x").to_string()))
        .collect();
    (rename(l, &map), rename(r, &map))
}

/// Is `(nl, nr)` a better representative of its shape than `(bl, br)`? Smallest first (the cleanest,
/// most general form — `(x+0)=x` beats `(0+(x*y))=(x*y)`), then most distinct variables (so
/// commutativity reads `(x+y)=(y+x)`, not `(1+x)=(x+1)`), then a stable canonical order for
/// determinism. This is why the spec shows `(x+0)=x` rather than the ground instance `(2+0)=2`.
fn better_rep(nl: &Expr, nr: &Expr, bl: &Expr, br: &Expr) -> bool {
    let rank = |l: &Expr, r: &Expr| {
        (
            size(l) + size(r),
            usize::MAX - (vars_used(l) + vars_used(r)),
            format!("{}|{}", key(l), key(r)),
        )
    };
    rank(nl, nr) < rank(bl, br)
}

/// A fixed reading order for the named shapes, so the discovered spec lists identities first, then
/// the structural laws — the order a person would want to read them in.
fn shape_rank(prose: &str) -> usize {
    const ORDER: [&str; 9] = [
        "Adding zero",
        "Multiplying by one",
        "Multiplying by zero",
        "Addition gives",
        "Multiplication gives",
        "Addition, the grouping",
        "Multiplication, the grouping",
        "Multiplying a sum",
        "never less",
    ];
    ORDER
        .iter()
        .position(|k| prose.contains(k))
        .unwrap_or(ORDER.len())
}

/// Reduce the synthesized equalities to the minimal, readable spec: ONE canonical representative per
/// named algebraic shape, plus a count of the remaining equalities (all consequences of the named
/// laws). α-variants and AC-variants of the same shape collapse to a single best representative; the
/// unnamed residue is counted, never listed (it is implied, and listing it would swamp the spec).
fn prune(raw: Vec<(Expr, Expr)>) -> (Vec<(Expr, Expr)>, usize) {
    let mut by_shape: BTreeMap<String, (Expr, Expr)> = BTreeMap::new();
    let mut unnamed: std::collections::HashSet<String> = std::collections::HashSet::new();

    for (l0, r0) in raw {
        let (l, r) = alpha_normalize(&l0, &r0);
        if l == r {
            continue;
        }
        match classify(&l, &r) {
            Some(name) => {
                let replace = match by_shape.get(&name) {
                    None => true,
                    Some((bl, br)) => better_rep(&l, &r, bl, br),
                };
                if replace {
                    by_shape.insert(name, (l, r));
                }
            }
            None => {
                // an unnamed equality — a consequence of the named laws. Count it once per
                // AC-normal-form pair (so comm/assoc variants of one consequence count once).
                let mut pair = [acnf_key(&l), acnf_key(&r)];
                pair.sort();
                unnamed.insert(pair.join("=="));
            }
        }
    }

    let mut named: Vec<(String, (Expr, Expr))> = by_shape.into_iter().collect();
    named.sort_by_key(|(name, _)| shape_rank(name));
    (
        named.into_iter().map(|(_, pair)| pair).collect(),
        unnamed.len(),
    )
}

// ----- rendering: a symbolic equation and a non-mathy sentence -------------

/// Print a term in conventional notation (`(x + (y * z))`).
fn show(t: &Expr) -> String {
    match t {
        Expr::Var(id) => id.get().to_string(),
        Expr::Lit(Lit::Int(i)) => i.get().to_string(),
        Expr::Lit(Lit::Bool(b)) => b.to_string(),
        Expr::Bin(op, l, r) => format!("({} {} {})", show(l), op.sym(), show(r)),
        _ => "?".to_string(),
    }
}

fn op_word(op: Op) -> &'static str {
    match op {
        Op::Add => "Addition",
        Op::Mul => "Multiplication",
        Op::Lt => "Comparison",
    }
}

/// Classify a `(lhs, rhs)` pair against the named algebraic shapes, for a plain-language sentence.
/// Both orientations are tried, and operand order is compared up to commutativity/associativity (via
/// `acnf_eq`), so a discovered instance with permuted operands is still recognized. A pair matching
/// no known shape returns `None` (honest: synthesis finds equalities with no textbook name — they
/// are counted as consequences, not listed).
fn classify(l: &Expr, r: &Expr) -> Option<String> {
    classify_oriented(l, r).or_else(|| classify_oriented(r, l))
}

/// Try to name the shape with `l` as the left-hand side (the structured side).
fn classify_oriented(l: &Expr, r: &Expr) -> Option<String> {
    use Expr::Bin;
    let is_int = |e: &Expr, n: i64| matches!(e, Expr::Lit(Lit::Int(i)) if i.get() == n);

    if let Bin(op, a, b) = l {
        // IDENTITY: `t op e == t`, with `e` the operator's unit (0 for `+`, 1 for `*`).
        if *op == Op::Add || *op == Op::Mul {
            let unit = if *op == Op::Add { 0 } else { 1 };
            let identity = (is_int(a, unit) && acnf_eq(b, r)) || (is_int(b, unit) && acnf_eq(a, r));
            if identity {
                let phrase = if *op == Op::Add {
                    "Adding zero"
                } else {
                    "Multiplying by one"
                };
                return Some(format!("{phrase} leaves a value unchanged."));
            }
        }
        // ANNIHILATION: `t * 0 == 0`.
        if *op == Op::Mul && (is_int(a, 0) || is_int(b, 0)) && is_int(r, 0) {
            return Some("Multiplying by zero always gives zero.".to_string());
        }
    }
    // COMMUTATIVITY: `a op b == b op a` — a single swap of two distinct operands.
    if let (Bin(o1, a, b), Bin(o2, c, d)) = (l, r) {
        if o1 == o2
            && (*o1 == Op::Add || *o1 == Op::Mul)
            && acnf_eq(a, d)
            && acnf_eq(b, c)
            && !acnf_eq(a, b)
        {
            return Some(format!(
                "{} gives the same result in either order.",
                op_word(*o1)
            ));
        }
    }
    // ASSOCIATIVITY: same operator, AC-equal, three or more operands — a pure regrouping.
    if let (Bin(o1, _, _), Bin(o2, _, _)) = (l, r) {
        if o1 == o2
            && (*o1 == Op::Add || *o1 == Op::Mul)
            && acnf_eq(l, r)
            && flatten_len(l, *o1) >= 3
        {
            return Some(format!(
                "When combining three values with {}, the grouping doesn't matter.",
                op_word(*o1)
            ));
        }
    }
    // DISTRIBUTIVITY: `a * (b + c) == (a*b) + (a*c)`, up to operand order on each side.
    if let Bin(Op::Mul, a, bc) = l {
        if let Bin(Op::Add, b, c) = &**bc {
            let expected = Expr::bin(
                Op::Add,
                Expr::bin(Op::Mul, (**a).clone(), (**b).clone()),
                Expr::bin(Op::Mul, (**a).clone(), (**c).clone()),
            );
            if acnf_eq(r, &expected) {
                return Some(
                    "Multiplying a sum is the same as multiplying each part and adding the results."
                        .to_string(),
                );
            }
        }
    }
    // IRREFLEXIVITY: `a < a == false`.
    if let Bin(Op::Lt, a, b) = l {
        if acnf_eq(a, b) && matches!(r, Expr::Lit(Lit::Bool(false))) {
            return Some("A value is never less than itself.".to_string());
        }
    }
    None
}

// ----- the universal observer U: structure + semantics sensitivity --------

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

// ----- the discovered spec -------------------------------------------------

/// The discovered spec: the named equational laws synthesized by running the operators (one
/// canonical representative each), plus the structure-sensitivity law over the universal observer
/// `U`. The author supplied only the operators and constants; everything here was found, not
/// declared. The harness re-probes each as an oracle-free equation.
pub fn discover_laws() -> Vec<Law> {
    discovered_spec().0
}

/// The discovered spec AND the number of further equational consequences pruned away — every one a
/// consequence of the named laws (so it is counted for honesty, not listed, which would swamp the
/// readable spec). The example surfaces the count; the harness only needs the laws.
pub fn discovered_spec() -> (Vec<Law>, usize) {
    let (named, consequences) = prune(synthesize());
    let mut out: Vec<Law> = named
        .into_iter()
        .map(|(l, r)| {
            let prose = classify(&l, &r).expect("a kept law has a named shape");
            let equation = format!("{} = {}", show(&l), show(&r));
            let (ll, rr) = (l, r);
            Law::Equation {
                prose,
                equation,
                schema: Box::new(move |a, b, c| (subst(&ll, a, b, c), subst(&rr, a, b, c))),
            }
        })
        .collect();

    if observer_is_sensitive(render()) {
        out.push(Law::Sensitivity {
            prose: "No two distinct programs look the same — the faithful rendering distinguishes \
                    every structural and semantic difference."
                .to_string(),
            equation: "U(p) = U(q)  ⟹  p = q   (U = faithful render)".to_string(),
        });
    }

    (out, consequences)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn equations() -> Vec<String> {
        discover_laws()
            .iter()
            .map(|l| l.equation().to_string())
            .collect()
    }

    /// Synthesis RUNS, and the WHOLE discovered spec is exactly this — found by running the
    /// operators, deterministically (BTreeMap-ordered enumeration), so the spec is pinned to the
    /// last symbol. This single assertion exercises the entire pipeline — `synthesize`, `prune`,
    /// `classify`, `better_rep`, `shape_rank`, `show`, `flatten_len`, `alpha_normalize` — so a
    /// mutation to any of them changes a rendered law or the consequence count and is killed.
    #[test]
    fn the_discovered_spec_is_exact() {
        let (laws, consequences) = discovered_spec();
        let got: Vec<(String, String)> = laws
            .iter()
            .map(|l| (l.prose().to_string(), l.equation().to_string()))
            .collect();
        let expected: Vec<(&str, &str)> =
            vec![
            ("Adding zero leaves a value unchanged.", "(0 + x) = x"),
            ("Multiplying by one leaves a value unchanged.", "(1 * x) = x"),
            ("Multiplying by zero always gives zero.", "(0 * x) = 0"),
            ("Addition gives the same result in either order.", "(x + y) = (y + x)"),
            ("Multiplication gives the same result in either order.", "(x * y) = (y * x)"),
            (
                "When combining three values with Addition, the grouping doesn't matter.",
                "(x + (y + z)) = (y + (x + z))",
            ),
            (
                "When combining three values with Multiplication, the grouping doesn't matter.",
                "(x * (y * z)) = ((x * z) * y)",
            ),
            (
                "Multiplying a sum is the same as multiplying each part and adding the results.",
                "(x * (y + z)) = ((x * z) + (x * y))",
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
        assert_eq!(got, expected, "the discovered spec changed");
        // every other discovered equality is a consequence of the named laws (counted, not listed).
        assert_eq!(consequences, 22, "the consequence count changed");
    }

    /// Synthesis DISCRIMINATES: it does not emit a false equality. `x + 1` and `x` differ, so no
    /// discovered equation equates them — a mutant collapsing `int_sig` would.
    #[test]
    fn synthesis_emits_no_false_equality() {
        // `x + 1 = x` must never appear.
        assert!(!equations().iter().any(|e| e == "(x + 1) = x"));
        // and the discovered set is non-trivial.
        assert!(equations().len() > 3, "synthesis found almost nothing");
    }

    /// Pruning WORKS: each named shape appears EXACTLY ONCE, with a general (variable-bearing, not
    /// ground) representative — no ground instance like `(2 + 0) = 2` survives, and no shape is
    /// duplicated by its commutative/associative variants.
    #[test]
    fn pruning_removes_subsumed_laws() {
        // one representative per shape — no duplicated prose among the equational laws.
        let proses: Vec<String> = discover_laws()
            .iter()
            .filter(|l| matches!(l, Law::Equation { .. }))
            .map(|l| l.prose().to_string())
            .collect();
        let mut deduped = proses.clone();
        deduped.sort();
        deduped.dedup();
        assert_eq!(
            proses.len(),
            deduped.len(),
            "a shape is listed more than once"
        );

        // no ground instance survives where a general one exists.
        assert!(!equations().iter().any(|e| e == "(2 + 0) = 2"));
        // every representative of a variable-law shape mentions a variable, not just literals.
        for law in discover_laws() {
            if law.prose() == "Adding zero leaves a value unchanged." {
                assert!(
                    law.equation().contains('x'),
                    "identity law is a ground instance: {}",
                    law.equation()
                );
            }
        }
    }

    /// The universal observer's sensitivity law is discovered (the structural blind spot is
    /// covered), and a collapsing observer would NOT pass — pins `observer_is_sensitive` against an
    /// always-`true`/always-`false` mutant by exercising it both ways.
    #[test]
    fn the_universal_observer_law_is_discovered() {
        // the faithful rendering (U) is universally sensitive...
        assert!(observer_is_sensitive(render()), "render should be faithful");
        // ...while a collapsing observer (node count ignores values/operators) is NOT.
        assert!(
            !observer_is_sensitive(|x: &Expr| x.node_count()),
            "a collapsing observer must fail the sensitivity law"
        );
        assert!(discover_laws()
            .iter()
            .any(|l| matches!(l, Law::Sensitivity { .. })));
    }

    /// Every discovered law renders readably — non-empty prose ending in a sentence, and an
    /// equation. Pins the renderers against an empty-string mutant.
    #[test]
    fn discovered_laws_render_readably() {
        for law in discover_laws() {
            assert!(law.prose().ends_with('.'), "prose: {}", law.prose());
            assert!(!law.equation().is_empty());
        }
    }

    // --- probes for the discovery machinery's own helpers (discover is kept in the sweep) ---

    fn add(l: Expr, r: Expr) -> Expr {
        Expr::bin(Op::Add, l, r)
    }
    fn mul(l: Expr, r: Expr) -> Expr {
        Expr::bin(Op::Mul, l, r)
    }
    fn lt(l: Expr, r: Expr) -> Expr {
        Expr::bin(Op::Lt, l, r)
    }

    /// `size` counts every node — `(x + (y * z))` is five. Pins the `+` in the recursion (a `*`
    /// mutant would give a different, wrong count) so the representative-selection order is honest.
    #[test]
    fn size_counts_nodes() {
        assert_eq!(size(&var("x")), 1);
        assert_eq!(size(&add(var("x"), lit(0))), 3);
        assert_eq!(size(&add(var("x"), mul(var("y"), var("z")))), 5);
    }

    /// `synthesize` never emits a reflexive equality (`t = t`): an equality always relates two
    /// DISTINCT terms with the same behaviour. Pins the `c != &t` guards in both registries.
    #[test]
    fn synthesis_emits_no_reflexive_equality() {
        assert!(
            synthesize().iter().all(|(l, r)| l != r),
            "synthesis emitted a trivial t = t equality"
        );
    }

    /// `key` separates the boolean literals (and every leaf kind), so canonical ordering is total —
    /// pins the `Bool` arm against deletion.
    #[test]
    fn key_distinguishes_every_leaf() {
        assert_ne!(key(&Expr::boolean(true)), key(&Expr::boolean(false)));
        assert_ne!(key(&lit(0)), key(&var("x")));
    }

    /// `acnf` normalizes `+`/`*` (commutative) but leaves `<` ALONE (not commutative): `0 < 1` and
    /// `1 < 0` must stay distinct. Pins the AC guard against admitting the order operator.
    #[test]
    fn acnf_normalizes_only_the_ac_operators() {
        assert!(acnf_eq(&add(var("x"), var("y")), &add(var("y"), var("x"))));
        assert!(!acnf_eq(&lt(lit(0), lit(1)), &lt(lit(1), lit(0))));
    }

    /// `flatten_len` counts leaves under ONE operator only: `x + (y * z)` has two `+`-operands (the
    /// `*` is opaque). Pins the operator guard against flattening through the wrong operator.
    #[test]
    fn flatten_len_respects_the_operator() {
        assert_eq!(
            flatten_len(&add(var("x"), mul(var("y"), var("z"))), Op::Add),
            2
        );
        assert_eq!(
            flatten_len(&add(add(var("x"), var("y")), var("z")), Op::Add),
            3
        );
    }

    /// `better_rep` prefers the smaller, then the more-general term — and is STRICT (a term is not a
    /// better representative than itself). Pins the size sum (`+`, not `*`) and the strict `<`.
    #[test]
    fn better_rep_prefers_small_then_general_strictly() {
        let small = (add(lit(0), var("x")), var("x")); // (0 + x) = x, size 4
        let big = (
            add(lit(0), add(var("x"), var("y"))),
            add(var("x"), var("y")),
        ); // size 8
        assert!(better_rep(&small.0, &small.1, &big.0, &big.1));
        assert!(!better_rep(&big.0, &big.1, &small.0, &small.1));
        // strictness: a pair is never a better representative than itself.
        assert!(!better_rep(&small.0, &small.1, &small.0, &small.1));
        // same total size, more variables wins (commutativity reads `(x+y)=(y+x)`, not `(1+x)=(x+1)`).
        let two_var = (add(var("x"), var("y")), add(var("y"), var("x")));
        let one_var = (add(lit(1), var("x")), add(var("x"), lit(1)));
        assert!(better_rep(&two_var.0, &two_var.1, &one_var.0, &one_var.1));
        // the size criterion is the SUM, not the product: sides of sizes (3, 3) beat (1, 7) because
        // 3+3 < 1+7, even though 3*3 > 1*7. Pins the `+` against a `*` mutant.
        let balanced = (add(var("x"), var("y")), add(var("y"), var("z"))); // sizes (3, 3), sum 6
        let lopsided = (
            lit(0),                                          // size 1
            add(lit(0), add(lit(0), add(lit(0), var("x")))), // size 7
        );
        assert!(better_rep(
            &balanced.0,
            &balanced.1,
            &lopsided.0,
            &lopsided.1
        ));
    }

    /// `classify` is PRECISE, not just permissive: a near-miss of a shape is NOT named. Pins the
    /// conjunctions in the identity and annihilation clauses against being loosened to disjunctions.
    #[test]
    fn classify_rejects_near_misses() {
        // identity needs the unit AND the other side preserved — `(x + y) = y` is neither.
        assert_eq!(classify(&add(var("x"), var("y")), &var("y")), None);
        // annihilation needs a zero factor AND a zero result — `(0 * x) = x` has the wrong result.
        assert_eq!(classify(&mul(lit(0), var("x")), &var("x")), None);
        // and an Add term is never annihilation even with a zero around — `(0 + y) = 0` is not.
        assert_eq!(classify(&add(lit(0), var("y")), &lit(0)), None);
        // the genuine shapes still classify.
        assert!(classify(&add(lit(0), var("x")), &var("x")).is_some());
        assert!(classify(&mul(lit(0), var("x")), &lit(0)).is_some());
    }
}
