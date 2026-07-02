//! Tier: KERNEL — the trusted floor — defines/runs the format, exempt from the structural rules.
//!
//! engine — generic, signature-parameterized law discovery.
//!
//! The arithmetic discovery in v5/v6 was hardcoded to `Int`/`Bool` with `Add`/`Mul`/`Lt`. This is
//! the same pipeline made generic over a `Theory`: a multi-sorted signature of operators, the value
//! objects' inhabitants, and an OBSERVATION (how to fingerprint a value's behaviour). The engine
//!
//!   1. ENUMERATES terms over the operators and per-sort variables, canonically (one term per
//!      behavioural class), so the working set is bounded by behaviour, not raw term count;
//!   2. instantiates the UNIVERSAL ALGEBRAIC SHAPES (identity, commutativity, associativity,
//!      annihilation, idempotence, distributivity, absorption, involution, round-trip, irreflexivity,
//!      and the heterogeneous shapes — monoid ACTION, HOMOMORPHISM) over the actual operators and
//!      keeps the ones that run true over a grid — named laws;
//!   3. counts every other discovered equality as a CONSEQUENCE, and reports which operators appear
//!      in no law (where the spec is silent).
//!
//! Nothing here knows about numbers: a domain implements `Theory` and its algebra discovers itself.

use std::collections::BTreeMap;
use std::fmt::Debug;
use std::hash::Hash;

/// How an operator renders in a symbolic equation.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Fixity {
    /// `a + b`
    Infix,
    /// `f(a, b)`
    Prefix,
    /// `c` (a constant — arity 0)
    Nullary,
}

/// One operator in a theory's signature: a typed, possibly-partial function over values.
pub struct Operator<T: Theory> {
    /// Human name for prose ("Addition").
    pub name: &'static str,
    /// Symbol for equations (`+`, `add`).
    pub symbol: &'static str,
    pub fixity: Fixity,
    pub inputs: Vec<T::Sort>,
    pub output: T::Sort,
    /// The (possibly partial) evaluator — `None` where the operator is undefined on its inputs.
    pub eval: fn(&[T::Value]) -> Option<T::Value>,
}

/// A domain's algebra: the sorts, the operators, the inhabitants, and how a value is observed.
pub trait Theory: Sized {
    /// The value-object sorts (e.g. `Int`, `Bool`).
    type Sort: Copy + Eq + Ord + Hash + Debug;
    /// A runtime value, tagged by sort.
    type Value: Clone;
    /// A behavioural fingerprint — grouping terms by this IS observational equality. For a
    /// first-order value it is the value itself; for a function-valued one, its behaviour on a grid.
    type Obs: Clone + Eq + Ord + Hash;

    /// The theory's display name.
    fn name() -> &'static str;
    /// The signature: every operator (including arity-0 constants).
    fn operators() -> Vec<Operator<Self>>;
    /// Seed inhabitants for a sort — the grid the laws are judged over (from `#[derive(Shaped)]`).
    fn inhabitants(sort: Self::Sort) -> Vec<Self::Value>;
    /// The sort of a value.
    fn sort_of(value: &Self::Value) -> Self::Sort;
    /// Observe a value — the universal observer `U`, specialised to this sort.
    fn observe(value: &Self::Value) -> Self::Obs;

    /// Variable letters per sort, for rendering and enumeration (first `num_vars` are used).
    fn sort_vars(_sort: Self::Sort) -> &'static [&'static str] {
        &["x", "y", "z"]
    }
    /// How many variables per sort to enumerate over.
    fn num_vars() -> usize {
        3
    }
    /// Rounds of operator combination (the term-depth bound).
    fn rounds() -> usize {
        2
    }
    /// How many grid assignments to sample (a resource limit, not a curated list).
    fn grid_size() -> usize {
        24
    }
}

/// A term over the signature: a variable, or an operator applied to argument terms. Variables and
/// operators are referenced by index into the engine's tables, so the type is sort-agnostic.
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Term {
    Var(usize),
    App(usize, Vec<Term>),
}

/// A discovered law: its plain-language and symbolic renderings, plus the two terms it equates
/// (so the spec can be re-probed over a fresh grid, where mutation judges its kill power).
pub struct DiscoveredLaw {
    pub prose: String,
    pub equation: String,
    pub lhs: Term,
    pub rhs: Term,
}

/// The full result of discovery over a theory.
pub struct Discovered {
    /// The named laws — one canonical representative per recognised shape.
    pub laws: Vec<DiscoveredLaw>,
    /// How many further equalities were found that no shape named (all consequences).
    pub consequences: usize,
    /// Operators (by symbol) that participate in NO named law — where the spec is silent.
    pub uncovered_ops: Vec<&'static str>,
}

#[derive(Clone, Copy)]
struct Var<S> {
    sort: S,
    ord: usize,
}

/// The engine: builds the variable set and the grid, then runs enumeration and the template battery.
pub struct Engine<T: Theory> {
    ops: Vec<Operator<T>>,
    vars: Vec<Var<T::Sort>>,
    /// Each assignment maps a variable id (index into `vars`) to a value.
    grid: Vec<Vec<T::Value>>,
}

/// A term's behavioural signature: the observation at each grid assignment (`None` where undefined).
type Sig<T> = Vec<Option<<T as Theory>::Obs>>;

/// The sampling stride for a grid space too large to enumerate: the first integer at or above the
/// golden-ratio point `space · φ⁻¹` (the classic low-discrepancy multiplier) that is coprime to
/// the space, so `k · step (mod space)` visits distinct, well-spread assignments.
fn coprime_step(space: u128) -> u128 {
    let mut step = (space.saturating_mul(618) / 1000).max(1);
    while gcd(step, space) != 1 {
        step += 1;
    }
    step
}

fn gcd(a: u128, b: u128) -> u128 {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

/// An operator's signature, by index: `(symbol, input sorts, output sort)`.
pub type OpSignature<S> = (&'static str, Vec<S>, S);

/// An operator's full declaration: `(name, symbol, fixity, input sorts, output sort)`.
pub type OpDeclaration<S> = (&'static str, &'static str, Fixity, Vec<S>, S);

/// Is `op` a homogeneous binary operator on sort `s` (`s × s -> s`)? Used by every shape that needs
/// a binary on a given sort, so a mutation to this predicate breaks laws across all theories.
fn is_binary_on<T: Theory>(op: &Operator<T>, s: T::Sort) -> bool {
    op.inputs.len() == 2 && op.inputs[0] == s && op.inputs[1] == s && op.output == s
}

impl<T: Theory> Engine<T> {
    /// Build the engine for a theory: collect the sorts that appear in the signature, mint `num_vars`
    /// variables per sort, and lay out a deterministic spread of grid assignments.
    pub fn new() -> Self {
        let ops = T::operators();

        // sorts that appear anywhere in the signature, in a stable order.
        let mut sorts: Vec<T::Sort> = Vec::new();
        for op in &ops {
            for s in op.inputs.iter().chain(std::iter::once(&op.output)) {
                if !sorts.contains(s) {
                    sorts.push(*s);
                }
            }
        }

        // variables: num_vars per sort. A sort with NO inhabitants mints no variables — there is
        // nothing to assign one to, so its laws can only come from constant-built terms (the old
        // code would instead have hit a divide-by-zero constructing the grid).
        let mut vars: Vec<Var<T::Sort>> = Vec::new();
        for &sort in &sorts {
            if T::inhabitants(sort).is_empty() {
                continue;
            }
            for ord in 0..T::num_vars() {
                vars.push(Var { sort, ord });
            }
        }

        // the grid: assignments drawn from the cross-product of the variables' inhabitant sets.
        // A small space is enumerated EXHAUSTIVELY — the laws are judged on every combination, not
        // a sample. A larger one is sampled by decoding `k · step (mod space)` in mixed radix (one
        // digit per variable; `step` coprime to the space, so distinct `k` pick distinct
        // assignments). The decode gives every variable its own digit, so no variable's value is a
        // function of another's. The previous odd-stride spread aliased variables of
        // same-cardinality sorts — two boolean variables were always equal or complementary, and a
        // three-element sort pinned its middle variable constant — which is exactly the
        // over-fitting the grid exists to refute.
        let inhabitants: Vec<Vec<T::Value>> = vars.iter().map(|v| T::inhabitants(v.sort)).collect();
        // clamped so `k * step` below cannot overflow u128; the decode stays valid either way.
        let space: u128 = inhabitants
            .iter()
            .fold(1u128, |acc, inh| acc.saturating_mul(inh.len() as u128))
            .min(1 << 96);
        let exhaustive_cap = (T::grid_size() as u128).max(64);
        let picks: Vec<u128> = if space <= exhaustive_cap {
            (0..space).collect()
        } else {
            let step = coprime_step(space);
            (0..T::grid_size() as u128)
                .map(|k| (k * step) % space)
                .collect()
        };
        let mut grid: Vec<Vec<T::Value>> = Vec::with_capacity(picks.len());
        for pick in picks {
            let mut rest = pick;
            let mut asn: Vec<T::Value> = Vec::with_capacity(vars.len());
            for inh in &inhabitants {
                let len = inh.len() as u128;
                asn.push(inh[(rest % len) as usize].clone());
                rest /= len;
            }
            grid.push(asn);
        }

        Engine { ops, vars, grid }
    }

    fn sort_of_term(&self, t: &Term) -> T::Sort {
        match t {
            Term::Var(i) => self.vars[*i].sort,
            Term::App(op, _) => self.ops[*op].output,
        }
    }

    fn eval(&self, t: &Term, asn: &[T::Value]) -> Option<T::Value> {
        match t {
            Term::Var(i) => Some(asn[*i].clone()),
            Term::App(op, args) => {
                let mut vals = Vec::with_capacity(args.len());
                for a in args {
                    vals.push(self.eval(a, asn)?);
                }
                (self.ops[*op].eval)(&vals)
            }
        }
    }

    /// The behavioural signature of a term over the grid (`None` per assignment where undefined).
    fn signature(&self, t: &Term) -> Sig<T> {
        self.grid
            .iter()
            .map(|asn| self.eval(t, asn).map(|v| T::observe(&v)))
            .collect()
    }

    /// Does a signature carry any information — is the term defined on at least one assignment?
    /// A term undefined everywhere is noise: two all-undefined constants would otherwise share
    /// the empty behaviour and collide into a bogus equality.
    fn meaningful(sig: &Sig<T>) -> bool {
        sig.iter().any(|o| o.is_some())
    }

    // -- enumeration: canonical generation, emitting an equality per behavioural collision --------

    /// Enumerate terms canonically and return every equality between two distinct terms that share a
    /// behaviour. Used to COUNT consequences and to surface novelties the templates don't name.
    fn enumerate(&self) -> Vec<(Term, Term)> {
        let mut canon: BTreeMap<(T::Sort, Sig<T>), Term> = BTreeMap::new();
        let mut equalities: Vec<(Term, Term)> = Vec::new();

        let register = |this: &Self,
                        canon: &mut BTreeMap<(T::Sort, Sig<T>), Term>,
                        eqs: &mut Vec<(Term, Term)>,
                        t: Term| {
            let sig = this.signature(&t);
            if !Self::meaningful(&sig) {
                return;
            }
            let key = (this.sort_of_term(&t), sig);
            match canon.get(&key) {
                Some(c) if *c != t => eqs.push((t.clone(), c.clone())),
                Some(_) => {}
                None => {
                    canon.insert(key, t);
                }
            }
        };

        // seed: variables and nullary operators (constants).
        for i in 0..self.vars.len() {
            register(self, &mut canon, &mut equalities, Term::Var(i));
        }
        for (oid, op) in self.ops.iter().enumerate() {
            if op.inputs.is_empty() {
                register(self, &mut canon, &mut equalities, Term::App(oid, vec![]));
            }
        }

        // rounds of combination over the canonical set (kept small — one term per behaviour).
        for _round in 0..T::rounds() {
            let current: Vec<Term> = canon.values().cloned().collect();
            let by_sort: BTreeMap<T::Sort, Vec<Term>> = {
                let mut m: BTreeMap<T::Sort, Vec<Term>> = BTreeMap::new();
                for t in &current {
                    m.entry(self.sort_of_term(t)).or_default().push(t.clone());
                }
                m
            };
            let mut formed: Vec<Term> = Vec::new();
            for (oid, op) in self.ops.iter().enumerate() {
                if op.inputs.is_empty() {
                    continue;
                }
                let empty = Vec::new();
                let choices: Vec<&Vec<Term>> = op
                    .inputs
                    .iter()
                    .map(|s| by_sort.get(s).unwrap_or(&empty))
                    .collect();
                if choices.iter().any(|c| c.is_empty()) {
                    continue;
                }
                let mut combo: Vec<usize> = vec![0; op.inputs.len()];
                'gen: loop {
                    let args: Vec<Term> = combo
                        .iter()
                        .enumerate()
                        .map(|(i, &j)| choices[i][j].clone())
                        .collect();
                    formed.push(Term::App(oid, args));
                    // odometer over the choice indices.
                    let mut d = combo.len();
                    loop {
                        if d == 0 {
                            break 'gen;
                        }
                        d -= 1;
                        combo[d] += 1;
                        if combo[d] < choices[d].len() {
                            break;
                        }
                        combo[d] = 0;
                    }
                }
            }
            for t in formed {
                register(self, &mut canon, &mut equalities, t);
            }
        }

        equalities
    }

    // -- rendering ------------------------------------------------------------------------------

    fn render(&self, t: &Term) -> String {
        match t {
            Term::Var(i) => {
                let v = self.vars[*i];
                T::sort_vars(v.sort)
                    .get(v.ord)
                    .copied()
                    .unwrap_or("?")
                    .to_string()
            }
            Term::App(op, args) => {
                let o = &self.ops[*op];
                match o.fixity {
                    Fixity::Nullary => o.symbol.to_string(),
                    // infix operators are binary by construction.
                    Fixity::Infix => {
                        format!(
                            "({} {} {})",
                            self.render(&args[0]),
                            o.symbol,
                            self.render(&args[1])
                        )
                    }
                    Fixity::Prefix => {
                        let inner: Vec<String> = args.iter().map(|a| self.render(a)).collect();
                        format!("{}({})", o.symbol, inner.join(", "))
                    }
                }
            }
        }
    }
}

// -- the template battery: universal algebraic shapes instantiated over the operators -----------

impl<T: Theory> Engine<T> {
    /// The id of the variable for `(sort, ord)`, if it exists.
    fn var(&self, sort: T::Sort, ord: usize) -> Option<Term> {
        self.vars
            .iter()
            .position(|v| v.sort == sort && v.ord == ord)
            .map(Term::Var)
    }

    /// Every nullary operator (constant), as `(op id, the constant term)`.
    fn constants(&self) -> Vec<(usize, Term)> {
        self.ops
            .iter()
            .enumerate()
            .filter(|(_, op)| op.inputs.is_empty())
            .map(|(oid, _)| (oid, Term::App(oid, vec![])))
            .collect()
    }

    fn same(&self, a: &Term, b: &Term) -> bool {
        self.signature(a) == self.signature(b)
    }

    fn app(op: usize, args: Vec<Term>) -> Term {
        Term::App(op, args)
    }

    /// Instantiate the universal algebraic shapes over the operators; keep those that run true.
    /// One law per recognised shape-instance, deduplicated by its prose (so left/right variants of
    /// one symmetric law collapse).
    fn templates(&self) -> Vec<DiscoveredLaw> {
        let mut out: Vec<DiscoveredLaw> = Vec::new();
        let mut seen_prose: Vec<String> = Vec::new();
        let mut push = |this: &Self, prose: String, lhs: Term, rhs: Term| {
            if this.same(&lhs, &rhs) && !seen_prose.contains(&prose) {
                seen_prose.push(prose.clone());
                let equation = format!("{} = {}", this.render(&lhs), this.render(&rhs));
                out.push(DiscoveredLaw {
                    prose,
                    equation,
                    lhs,
                    rhs,
                });
            }
        };

        let ops = &self.ops;
        for (fid, f) in ops.iter().enumerate() {
            // homogeneous binary `f : s × s -> s`
            if is_binary_on(f, f.output) {
                let s = f.output;
                let (Some(x), Some(y), Some(z)) = (self.var(s, 0), self.var(s, 1), self.var(s, 2))
                else {
                    continue;
                };

                // commutativity: f(x,y) = f(y,x)
                push(
                    self,
                    format!("{} gives the same result in either order.", f.name),
                    Self::app(fid, vec![x.clone(), y.clone()]),
                    Self::app(fid, vec![y.clone(), x.clone()]),
                );
                // associativity: f(f(x,y),z) = f(x,f(y,z))
                push(
                    self,
                    format!(
                        "With {}, the grouping of three values doesn't matter.",
                        f.name
                    ),
                    Self::app(
                        fid,
                        vec![Self::app(fid, vec![x.clone(), y.clone()]), z.clone()],
                    ),
                    Self::app(
                        fid,
                        vec![x.clone(), Self::app(fid, vec![y.clone(), z.clone()])],
                    ),
                );
                // idempotence: f(x,x) = x
                push(
                    self,
                    format!("{} of a value with itself gives that value.", f.name),
                    Self::app(fid, vec![x.clone(), x.clone()]),
                    x.clone(),
                );

                // constant-bearing shapes
                for (_, c) in self
                    .constants()
                    .into_iter()
                    .filter(|(cid, _)| ops[*cid].output == s)
                {
                    let cs = self.render(&c);
                    // identity: f(c,x) = x  (or f(x,c) = x)
                    push(
                        self,
                        format!("{} with {} leaves a value unchanged.", f.name, cs),
                        Self::app(fid, vec![c.clone(), x.clone()]),
                        x.clone(),
                    );
                    push(
                        self,
                        format!("{} with {} leaves a value unchanged.", f.name, cs),
                        Self::app(fid, vec![x.clone(), c.clone()]),
                        x.clone(),
                    );
                    // annihilation: f(c,x) = c  (or f(x,c) = c)
                    push(
                        self,
                        format!("{} by {} always gives {}.", f.name, cs, cs),
                        Self::app(fid, vec![c.clone(), x.clone()]),
                        c.clone(),
                    );
                    push(
                        self,
                        format!("{} by {} always gives {}.", f.name, cs, cs),
                        Self::app(fid, vec![x.clone(), c.clone()]),
                        c.clone(),
                    );
                }

                // a second homogeneous binary `g` of the same sort: distributivity and absorption.
                for (gid, g) in ops.iter().enumerate() {
                    if gid != fid && is_binary_on(g, s) {
                        // distributivity: f(x, g(y,z)) = g(f(x,y), f(x,z))
                        push(
                            self,
                            format!("{} distributes over {}.", f.name, g.name),
                            Self::app(
                                fid,
                                vec![x.clone(), Self::app(gid, vec![y.clone(), z.clone()])],
                            ),
                            Self::app(
                                gid,
                                vec![
                                    Self::app(fid, vec![x.clone(), y.clone()]),
                                    Self::app(fid, vec![x.clone(), z.clone()]),
                                ],
                            ),
                        );
                        // absorption: f(x, g(x,y)) = x  (e.g. `x ∧ (x ∨ y) = x`)
                        push(
                            self,
                            format!("{} absorbs {}.", f.name, g.name),
                            Self::app(
                                fid,
                                vec![x.clone(), Self::app(gid, vec![x.clone(), y.clone()])],
                            ),
                            x.clone(),
                        );
                    }
                }
            }

            // heterogeneous binary `f : s × t -> s` (an ACTION of `t` on `s`, `t ≠ s`)
            if f.inputs.len() == 2 && f.inputs[0] == f.output && f.inputs[1] != f.output {
                let s = f.output;
                let t = f.inputs[1];
                if let (Some(x), Some(p), Some(q)) =
                    (self.var(s, 0), self.var(t, 0), self.var(t, 1))
                {
                    // action identity: f(x, c) = x for a constant `c : t` (`add(d, zero) = d`)
                    for (_, c) in self
                        .constants()
                        .into_iter()
                        .filter(|(cid, _)| ops[*cid].output == t)
                    {
                        let cs = self.render(&c);
                        push(
                            self,
                            format!("{} with {} leaves a value unchanged.", f.name, cs),
                            Self::app(fid, vec![x.clone(), c.clone()]),
                            x.clone(),
                        );
                    }
                    // action compatibility: f(f(x,p),q) = f(x, g(p,q)) for a binary `g : t × t -> t`
                    // (`add(add(d,p),q) = add(d, plus(p,q))`)
                    for (gid, g) in ops.iter().enumerate() {
                        if is_binary_on(g, t) {
                            push(
                                self,
                                format!(
                                    "Repeated {} combines its parameters with {}.",
                                    f.name, g.name
                                ),
                                Self::app(
                                    fid,
                                    vec![Self::app(fid, vec![x.clone(), p.clone()]), q.clone()],
                                ),
                                Self::app(
                                    fid,
                                    vec![x.clone(), Self::app(gid, vec![p.clone(), q.clone()])],
                                ),
                            );
                        }
                    }
                }
            }

            // relation `p : s × s -> r`, with `r` carrying a `false` constant → irreflexivity
            if f.inputs.len() == 2 && f.inputs[0] == f.inputs[1] && f.output != f.inputs[0] {
                let s = f.inputs[0];
                let r = f.output;
                if let Some(x) = self.var(s, 0) {
                    for (_, c) in self
                        .constants()
                        .into_iter()
                        .filter(|(cid, _)| ops[*cid].output == r)
                    {
                        // a relation collapsing to `false` is irreflexivity; any other constant is a
                        // self-application law (`diff(x, x) = zero`).
                        let cs = self.render(&c);
                        let prose = if cs == "false" {
                            format!("A value is never {} itself.", f.name)
                        } else {
                            format!("{} of a value with itself gives {}.", f.name, cs)
                        };
                        push(
                            self,
                            prose,
                            Self::app(fid, vec![x.clone(), x.clone()]),
                            c.clone(),
                        );
                    }
                }
            }

            // unary `u : s -> s` → involution
            if f.inputs.len() == 1 && f.output == f.inputs[0] {
                let s = f.output;
                if let Some(x) = self.var(s, 0) {
                    push(
                        self,
                        format!("{} twice returns the original value.", f.name),
                        Self::app(fid, vec![Self::app(fid, vec![x.clone()])]),
                        x.clone(),
                    );
                }
            }

            // round-trip: g(f(x)) = x for f : s -> t and g : t -> s
            if f.inputs.len() == 1 {
                let s = f.inputs[0];
                let t = f.output;
                if let Some(x) = self.var(s, 0) {
                    for (gid, g) in ops.iter().enumerate() {
                        if gid != fid && g.inputs.len() == 1 && g.inputs[0] == t && g.output == s {
                            push(
                                self,
                                format!(
                                    "{} undoes {} — the round trip is the identity.",
                                    g.name, f.name
                                ),
                                Self::app(gid, vec![Self::app(fid, vec![x.clone()])]),
                                x.clone(),
                            );
                        }
                    }
                }

                // homomorphism: `h(p(x,y)) = q(h(x), h(y))` — `h = f` turns a binary `p` on its input
                // sort into a binary `q` on its output sort (e.g. De Morgan: `¬(x∧y) = ¬x ∨ ¬y`).
                if let (Some(xs), Some(ys)) = (self.var(s, 0), self.var(s, 1)) {
                    for (pid, p) in ops.iter().enumerate() {
                        if !is_binary_on(p, s) {
                            continue;
                        }
                        for (qid, q) in ops.iter().enumerate() {
                            if is_binary_on(q, t) {
                                push(
                                    self,
                                    format!("{} turns {} into {}.", f.name, p.name, q.name),
                                    Self::app(
                                        fid,
                                        vec![Self::app(pid, vec![xs.clone(), ys.clone()])],
                                    ),
                                    Self::app(
                                        qid,
                                        vec![
                                            Self::app(fid, vec![xs.clone()]),
                                            Self::app(fid, vec![ys.clone()]),
                                        ],
                                    ),
                                );
                            }
                        }
                    }
                }
            }
        }

        out
    }

    /// Run discovery: the named template laws, the count of further (consequence) equalities, and
    /// the operators that appear in no named law.
    pub fn discover(&self) -> Discovered {
        let laws = self.templates();

        // signatures the named laws already account for (per sort), to separate consequences.
        let named_sigs: std::collections::BTreeSet<(T::Sort, Sig<T>)> = laws
            .iter()
            .map(|l| (self.sort_of_term(&l.lhs), self.signature(&l.lhs)))
            .collect();
        let mut consequence_sigs: std::collections::BTreeSet<(T::Sort, Sig<T>)> =
            std::collections::BTreeSet::new();
        for (a, _) in self.enumerate() {
            let key = (self.sort_of_term(&a), self.signature(&a));
            if !named_sigs.contains(&key) {
                consequence_sigs.insert(key);
            }
        }

        // coverage: which operator symbols never appear in a named law.
        let mut used: std::collections::BTreeSet<usize> = std::collections::BTreeSet::new();
        fn collect(t: &Term, used: &mut std::collections::BTreeSet<usize>) {
            if let Term::App(op, args) = t {
                used.insert(*op);
                for a in args {
                    collect(a, used);
                }
            }
        }
        for l in &laws {
            collect(&l.lhs, &mut used);
            collect(&l.rhs, &mut used);
        }
        let uncovered_ops: Vec<&'static str> = self
            .ops
            .iter()
            .enumerate()
            .filter(|(i, _)| !used.contains(i))
            .map(|(_, op)| op.symbol)
            .collect();

        Discovered {
            laws,
            consequences: consequence_sigs.len(),
            uncovered_ops,
        }
    }

    /// Check that the GIVEN laws hold when evaluated over the grid against the current operators,
    /// returning the first failure. Unlike re-deriving (which is tautological), feeding FROZEN laws
    /// here is the mutation-judged probe: a mutant that breaks a frozen law is caught.
    pub fn check(&self, laws: &[DiscoveredLaw]) -> Result<(), String> {
        for l in laws {
            for asn in &self.grid {
                let lhs = self.eval(&l.lhs, asn).map(|v| T::observe(&v));
                let rhs = self.eval(&l.rhs, asn).map(|v| T::observe(&v));
                if lhs != rhs {
                    return Err(format!("discovered law failed: {}", l.equation));
                }
            }
        }
        Ok(())
    }

    /// The signature, by operator index: `(symbol, input sorts, output sort)`. For the cohesion
    /// analysis, which reads which operators interact (share a law) and what sorts they touch.
    pub fn signatures(&self) -> Vec<OpSignature<T::Sort>> {
        self.ops
            .iter()
            .map(|o| (o.symbol, o.inputs.clone(), o.output))
            .collect()
    }

    /// The full declaration of each operator — `(name, symbol, fixity, inputs, output)` — for the
    /// scaffolder, which regenerates a `theory!` block for each split-out sub-module.
    pub fn declarations(&self) -> Vec<OpDeclaration<T::Sort>> {
        self.ops
            .iter()
            .map(|o| (o.name, o.symbol, o.fixity, o.inputs.clone(), o.output))
            .collect()
    }
}

impl<T: Theory> Default for Engine<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Build a grid for a value type from its STRUCTURE — the "shadow algebra".
///
/// The boundary operators define the LAWS, but they may not generate enough values to judge them on:
/// a thin grid over-fits (too few values to refute a false law), and a boundary with no
/// value-producing operator — a bare monoid, a router whose `or` never leaves its seeds — cannot
/// bootstrap a grid at all. So the grid is grown not from the boundary operators but from the value
/// type's OWN `Shaped` surface — a shadow algebra of synthetic generators the user never writes and
/// that never enter the discovered spec: start at the canonical `inhabitant`, close under its
/// structural PERTURBATIONS (variant swaps, field neighbours), bounded by `cap`. Deterministic — the
/// derived perturbation order is fixed. This is `#[derive(Shaped)]` (already minting the probe
/// surface for edges) reused to fatten the discovery grid.
pub fn shadow_grid<V: crate::boundary::Shaped>(cap: usize) -> Vec<V> {
    let mut grid: Vec<V> = ::std::vec![V::inhabitant()];
    let mut i = 0;
    // the `cap` bound lives in the inner break (the only place new values are added); the outer loop
    // just walks the frontier. `i` indexes into `grid`, so `i <= grid.len()` would read past the end.
    while i < grid.len() {
        for n in grid[i].all_perturbations() {
            if grid.len() >= cap {
                break;
            }
            if !grid.contains(&n) {
                grid.push(n);
            }
        }
        i += 1;
    }
    grid
}

#[cfg(test)]
impl<T: Theory> Engine<T> {
    /// The raw equalities enumeration emits — for probing the discovery machinery itself.
    pub(crate) fn emitted_equalities(&self) -> Vec<(Term, Term)> {
        self.enumerate()
    }

    /// The grid assignments (one `Vec<Value>` per assignment, columns aligned with `var_sorts`) —
    /// so tests can put the sampler itself on trial instead of taking its spread on faith.
    pub(crate) fn grid_assignments(&self) -> &[Vec<T::Value>] {
        &self.grid
    }

    /// The sort of each variable, in grid column order — the key for reading `grid_assignments`.
    pub(crate) fn var_sorts(&self) -> Vec<T::Sort> {
        self.vars.iter().map(|v| v.sort).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A self-contained boolean algebra, to pin the engine independent of any domain.
    #[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash, Debug)]
    struct Bit;
    struct Bits;
    fn b(v: &[bool], f: impl Fn(bool, bool) -> bool) -> Option<bool> {
        Some(f(v[0], v[1]))
    }
    fn and(v: &[Value2]) -> Option<Value2> {
        b(&[v[0].0, v[1].0], |a, c| a && c).map(Value2)
    }
    fn or(v: &[Value2]) -> Option<Value2> {
        b(&[v[0].0, v[1].0], |a, c| a || c).map(Value2)
    }
    fn not(v: &[Value2]) -> Option<Value2> {
        Some(Value2(!v[0].0))
    }
    fn ff(_: &[Value2]) -> Option<Value2> {
        Some(Value2(false))
    }
    fn tt(_: &[Value2]) -> Option<Value2> {
        Some(Value2(true))
    }
    #[derive(Clone)]
    struct Value2(bool);

    impl Theory for Bits {
        type Sort = Bit;
        type Value = Value2;
        type Obs = bool;
        fn name() -> &'static str {
            "bits"
        }
        fn operators() -> Vec<Operator<Self>> {
            use Fixity::{Infix, Nullary, Prefix};
            vec![
                Operator {
                    name: "False",
                    symbol: "F",
                    fixity: Nullary,
                    inputs: vec![],
                    output: Bit,
                    eval: ff,
                },
                Operator {
                    name: "True",
                    symbol: "T",
                    fixity: Nullary,
                    inputs: vec![],
                    output: Bit,
                    eval: tt,
                },
                Operator {
                    name: "And",
                    symbol: "&",
                    fixity: Infix,
                    inputs: vec![Bit, Bit],
                    output: Bit,
                    eval: and,
                },
                Operator {
                    name: "Or",
                    symbol: "|",
                    fixity: Infix,
                    inputs: vec![Bit, Bit],
                    output: Bit,
                    eval: or,
                },
                Operator {
                    name: "Not",
                    symbol: "~",
                    fixity: Prefix,
                    inputs: vec![Bit],
                    output: Bit,
                    eval: not,
                },
            ]
        }
        fn inhabitants(_: Self::Sort) -> Vec<Self::Value> {
            vec![Value2(false), Value2(true)]
        }
        fn sort_of(_: &Self::Value) -> Self::Sort {
            Bit
        }
        fn observe(v: &Self::Value) -> Self::Obs {
            v.0
        }
    }

    /// The engine, exercised end to end on a boolean algebra it knows nothing about: it discovers
    /// the FULL boolean law set (both distributivities, involution) by running the operators. Pins
    /// enumeration, the template battery, rendering, and coverage against mutation.
    #[test]
    fn the_engine_discovers_boolean_algebra() {
        let e = Engine::<Bits>::new();
        let d = e.discover();
        let got: Vec<(String, String)> = d
            .laws
            .iter()
            .map(|l| (l.prose.clone(), l.equation.clone()))
            .collect();
        // the COMPLETE boolean algebra — including both absorption laws and both De Morgan laws —
        // discovered by running the operators, nothing hand-listed.
        let expected: Vec<(&str, &str)> = vec![
            (
                "And gives the same result in either order.",
                "(x & y) = (y & x)",
            ),
            (
                "With And, the grouping of three values doesn't matter.",
                "((x & y) & z) = (x & (y & z))",
            ),
            (
                "And of a value with itself gives that value.",
                "(x & x) = x",
            ),
            ("And by F always gives F.", "(F & x) = F"),
            ("And with T leaves a value unchanged.", "(T & x) = x"),
            (
                "And distributes over Or.",
                "(x & (y | z)) = ((x & y) | (x & z))",
            ),
            ("And absorbs Or.", "(x & (x | y)) = x"),
            (
                "Or gives the same result in either order.",
                "(x | y) = (y | x)",
            ),
            (
                "With Or, the grouping of three values doesn't matter.",
                "((x | y) | z) = (x | (y | z))",
            ),
            ("Or of a value with itself gives that value.", "(x | x) = x"),
            ("Or with F leaves a value unchanged.", "(F | x) = x"),
            ("Or by T always gives T.", "(T | x) = T"),
            (
                "Or distributes over And.",
                "(x | (y & z)) = ((x | y) & (x | z))",
            ),
            ("Or absorbs And.", "(x | (x & y)) = x"),
            ("Not twice returns the original value.", "~(~(x)) = x"),
            ("Not turns And into Or.", "~((x & y)) = (~(x) | ~(y))"),
            ("Not turns Or into And.", "~((x | y)) = (~(x) & ~(y))"),
        ];
        let expected: Vec<(String, String)> = expected
            .into_iter()
            .map(|(p, q)| (p.to_string(), q.to_string()))
            .collect();
        assert_eq!(got, expected, "the discovered boolean algebra changed");
        assert_eq!(d.consequences, 41, "consequence count changed");
        assert!(
            d.uncovered_ops.is_empty(),
            "uncovered: {:?}",
            d.uncovered_ops
        );
    }

    /// `check` has real kill power: it passes the genuine laws and REJECTS a false one (`x & y` is
    /// not `x | y`). Pins `check` against an always-`Ok` mutant and a flipped comparison.
    #[test]
    fn check_accepts_real_laws_and_rejects_a_false_one() {
        let e = Engine::<Bits>::new();
        let d = e.discover();
        assert_eq!(e.check(&d.laws), Ok(()));
        // ops: [F=0, T=1, And=2, Or=3, Not=4]; vars: x=0, y=1, z=2.
        let bogus = DiscoveredLaw {
            prose: "bogus".into(),
            equation: "(x & y) = (x | y)".into(),
            lhs: Term::App(2, vec![Term::Var(0), Term::Var(1)]),
            rhs: Term::App(3, vec![Term::Var(0), Term::Var(1)]),
        };
        assert!(e.check(&[bogus]).is_err(), "check must reject a false law");
    }

    /// Enumeration never emits a reflexive equality (`t = t`): every emitted equality relates two
    /// DISTINCT terms with the same behaviour. Pins the `*c != t` guard against being dropped.
    #[test]
    fn enumeration_emits_no_reflexive_equality() {
        let e = Engine::<Bits>::new();
        assert!(
            e.enumerate().iter().all(|(a, b)| a != b),
            "a reflexive t = t equality was emitted"
        );
    }

    // A second theory with PARTIAL operators (all-undefined constants) and a ROUND-TRIP pair, to
    // pin the `meaningful` filter and round-trip detection that total theories can't reach.
    #[derive(Clone)]
    struct M3(u8);
    struct Codec;
    fn enc(v: &[M3]) -> Option<M3> {
        Some(M3((v[0].0 + 1) % 3))
    }
    fn dec(v: &[M3]) -> Option<M3> {
        Some(M3((v[0].0 + 2) % 3))
    }
    fn bottom(_: &[M3]) -> Option<M3> {
        None
    }

    impl Theory for Codec {
        type Sort = Bit;
        type Value = M3;
        type Obs = u8;
        fn name() -> &'static str {
            "codec"
        }
        fn operators() -> Vec<Operator<Self>> {
            use Fixity::{Nullary, Prefix};
            vec![
                // two all-undefined constants: filtered by `meaningful`, and they would COLLIDE
                // (both have the empty behaviour) if the filter were removed.
                Operator {
                    name: "Bottom1",
                    symbol: "b1",
                    fixity: Nullary,
                    inputs: vec![],
                    output: Bit,
                    eval: bottom,
                },
                Operator {
                    name: "Bottom2",
                    symbol: "b2",
                    fixity: Nullary,
                    inputs: vec![],
                    output: Bit,
                    eval: bottom,
                },
                Operator {
                    name: "encode",
                    symbol: "enc",
                    fixity: Prefix,
                    inputs: vec![Bit],
                    output: Bit,
                    eval: enc,
                },
                Operator {
                    name: "decode",
                    symbol: "dec",
                    fixity: Prefix,
                    inputs: vec![Bit],
                    output: Bit,
                    eval: dec,
                },
            ]
        }
        fn inhabitants(_: Self::Sort) -> Vec<Self::Value> {
            vec![M3(0), M3(1), M3(2)]
        }
        fn sort_of(_: &Self::Value) -> Self::Sort {
            Bit
        }
        fn observe(v: &Self::Value) -> Self::Obs {
            v.0
        }
    }

    /// The engine finds the ROUND-TRIP laws (`dec` undoes `enc`, and vice versa), filters the
    /// all-undefined constants via `meaningful`, and reports them as uncovered. Pins round-trip
    /// detection and the `meaningful` filter (without it the two bottoms collide).
    #[test]
    fn the_engine_discovers_round_trips_and_filters_undefined() {
        let e = Engine::<Codec>::new();
        let d = e.discover();
        let proses: Vec<&str> = d.laws.iter().map(|l| l.prose.as_str()).collect();
        assert!(proses.contains(&"decode undoes encode — the round trip is the identity."));
        assert!(proses.contains(&"encode undoes decode — the round trip is the identity."));
        // the all-undefined constants are in no law (and `meaningful` kept them from colliding —
        // without the filter the two bottoms collide and this count rises to 9).
        assert_eq!(d.uncovered_ops, vec!["b1", "b2"]);
        assert_eq!(
            d.consequences, 8,
            "an undefined-constant collision leaked in"
        );
    }

    /// `shadow_grid` grows a value type's grid from its STRUCTURE, not from any operator. For `bool`
    /// (which is `Shaped`: `inhabitant` = false, its one perturbation = negation) it produces exactly
    /// {false, true} from the type alone — the synthetic generation that fattens a boundary whose own
    /// operators could not. The `cap` is a hard bound.
    #[test]
    fn shadow_grid_grows_from_the_type_structure() {
        assert_eq!(
            shadow_grid::<bool>(8),
            vec![false, true],
            "bool's shadow grid is its two inhabitants, from the type alone"
        );
        assert_eq!(shadow_grid::<bool>(1).len(), 1, "cap bounds the grid");
    }

    // -- adversarial self-tests: the grid on trial ------------------------------------------------
    //
    // Every theory above bears only laws that are universally TRUE, so no test so far could tell a
    // grid that judges laws from one that rubber-stamps them. The theories below carry KNOWN-FALSE
    // laws: the engine passes only by refusing them, which is the one thing a degenerate grid
    // cannot do.

    // A theory whose operators are chosen for the laws they DON'T have: left projection
    // (associative and idempotent, but not commutative) and truncated subtraction (monus — neither
    // commutative, nor associative, nor idempotent). Over Q4 = {0,1,2,3} with three variables the
    // space is 4³ = 64, at the exhaustive cap, so refusal here means the full cross-product
    // refuted each false shape.
    #[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash, Debug)]
    struct Q4;
    struct Skew;
    #[derive(Clone)]
    struct V4(u8);
    fn proj(v: &[V4]) -> Option<V4> {
        Some(V4(v[0].0))
    }
    fn monus(v: &[V4]) -> Option<V4> {
        Some(V4(v[0].0.saturating_sub(v[1].0)))
    }

    impl Theory for Skew {
        type Sort = Q4;
        type Value = V4;
        type Obs = u8;
        fn name() -> &'static str {
            "skew"
        }
        fn operators() -> Vec<Operator<Self>> {
            use Fixity::{Infix, Prefix};
            vec![
                Operator {
                    name: "Left projection",
                    symbol: "proj",
                    fixity: Prefix,
                    inputs: vec![Q4, Q4],
                    output: Q4,
                    eval: proj,
                },
                Operator {
                    name: "Monus",
                    symbol: "-.",
                    fixity: Infix,
                    inputs: vec![Q4, Q4],
                    output: Q4,
                    eval: monus,
                },
            ]
        }
        fn inhabitants(_: Self::Sort) -> Vec<Self::Value> {
            (0..4).map(V4).collect()
        }
        fn sort_of(_: &Self::Value) -> Self::Sort {
            Q4
        }
        fn observe(v: &Self::Value) -> Self::Obs {
            v.0
        }
    }

    /// FALSE laws are REFUSED — and the refusal is honest because the same run still finds the
    /// TRUE ones. Left projection is associative and idempotent but not commutative; monus is
    /// none of the three. A grid degenerate enough to alias variables (or a `same` mutated toward
    /// `true`) certifies at least one of the false shapes and fails here; a grid mutated toward
    /// emptiness certifies ALL shapes and also fails here — the positive and negative assertions
    /// pin the sampler from both sides.
    #[test]
    fn the_grid_refutes_false_laws_while_naming_true_ones() {
        let e = Engine::<Skew>::new();
        let d = e.discover();
        let proses: Vec<&str> = d.laws.iter().map(|l| l.prose.as_str()).collect();

        // the true laws: found (a refusal test that finds nothing proves nothing).
        assert!(
            proses.contains(&"With Left projection, the grouping of three values doesn't matter."),
            "projection's associativity is real and must be discovered; got {proses:?}"
        );
        assert!(
            proses.contains(&"Left projection of a value with itself gives that value."),
            "projection's idempotence is real and must be discovered; got {proses:?}"
        );

        // the false laws: refused.
        for false_law in [
            "Left projection gives the same result in either order.",
            "Monus gives the same result in either order.",
            "With Monus, the grouping of three values doesn't matter.",
            "Monus of a value with itself gives that value.",
        ] {
            assert!(
                !proses.contains(&false_law),
                "a FALSE law was certified: {false_law:?} — the grid failed to refute it"
            );
        }
    }

    // The regression sort for the sampler fix: Lo/Mid/Hi with a binary op commutative on every
    // pair INVOLVING Mid but not on (Lo, Hi) — the exact blind spot of a sample that pins the
    // middle variable to Mid.
    #[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash, Debug)]
    struct T3s;
    struct Tilted;
    #[derive(Clone)]
    struct T3v(u8); // 0 = Lo, 1 = Mid, 2 = Hi
    fn tilt(v: &[T3v]) -> Option<T3v> {
        let (a, b) = (v[0].0, v[1].0);
        // max — i.e. symmetric — everywhere except {Lo, Hi}, where the FIRST argument wins:
        // tilt(Lo,Hi) = Lo but tilt(Hi,Lo) = Hi. Commutative on every pair involving Mid (and on
        // the diagonal), non-commutative only at the (Lo,Hi) corner.
        Some(T3v(if a.min(b) == 0 && a.max(b) == 2 {
            a
        } else {
            a.max(b)
        }))
    }

    impl Theory for Tilted {
        type Sort = T3s;
        type Value = T3v;
        type Obs = u8;
        fn name() -> &'static str {
            "tilted"
        }
        fn operators() -> Vec<Operator<Self>> {
            vec![Operator {
                name: "Tilt",
                symbol: "><",
                fixity: Fixity::Infix,
                inputs: vec![T3s, T3s],
                output: T3s,
                eval: tilt,
            }]
        }
        fn inhabitants(_: Self::Sort) -> Vec<Self::Value> {
            (0..3).map(T3v).collect()
        }
        fn sort_of(_: &Self::Value) -> Self::Sort {
            T3s
        }
        fn observe(v: &Self::Value) -> Self::Obs {
            v.0
        }
    }

    /// THE REGRESSION PIN for the grid sampler. The pre-fix sampler assigned variable `i` (ord
    /// `ord_i`, minted in ord order per sort) the inhabitant at `(k·(1 + 2i) + ord_i) mod n` for
    /// assignment `k` — which, on a 3-element sort, pins the middle variable to `(3k + 1) mod 3 =
    /// 1` FOREVER: `y` is the constant Mid, and commutativity of `x >< y` is only ever judged on
    /// pairs involving Mid. `Tilt` is built to be commutative on exactly those pairs and false on
    /// (Lo, Hi), so:
    ///
    ///   (a) reconstructing the OLD sample inline, the false commutativity HOLDS on every one of
    ///       its assignments — the old sampler would have certified it as a law;
    ///   (b) the current engine refuses it (while still finding Tilt's real idempotence), and
    ///       `check` on the frozen false law returns `Err`.
    ///
    /// Reverting `Engine::new`'s sampler to the aliasing spread makes (b) fail while (a) still
    /// passes: this test is the fix's load-bearing wall.
    #[test]
    fn the_sampler_fix_is_load_bearing_against_the_old_degenerate_sample() {
        let inh = Tilted::inhabitants(T3s);
        let f = |a: &T3v, b: &T3v| tilt(&[a.clone(), b.clone()]).unwrap().0;

        // the law is genuinely false: one witness pair breaks it.
        assert_ne!(
            f(&inh[0], &inh[2]),
            f(&inh[2], &inh[0]),
            "Tilt must actually be non-commutative for this test to mean anything"
        );

        // (a) the OLD sample, reconstructed inline: index = (k·(1 + 2i) + ord_i) mod 3 for
        // variable i over k in 0..grid_size, one sort so ord_i = i. On every assignment it
        // produces, the false commutativity HOLDS — the old grid would have certified it.
        for k in 0..Tilted::grid_size() {
            let idx = |i: usize| (k * (1 + 2 * i) + i) % 3;
            assert_eq!(
                idx(1),
                1,
                "the old sample pinned y to Mid — that IS the bug"
            );
            let (x, y) = (&inh[idx(0)], &inh[idx(1)]);
            assert_eq!(
                f(x, y),
                f(y, x),
                "the old sample was degenerate precisely because it never refuted this law; \
                 if it refutes it now, the reconstruction no longer matches the old formula"
            );
        }

        // (b) the CURRENT engine refuses the false law on a grid that reaches the (Lo, Hi) corner
        // (3³ = 27 assignments, under the exhaustive cap — the full cross-product)...
        let e = Engine::<Tilted>::new();
        let d = e.discover();
        let proses: Vec<&str> = d.laws.iter().map(|l| l.prose.as_str()).collect();
        assert!(
            !proses.contains(&"Tilt gives the same result in either order."),
            "the grid certified the false commutativity the old sampler certified — sampler regressed"
        );
        // ...while still finding the real law, so the refusal isn't an artifact of finding nothing.
        assert!(
            proses.contains(&"Tilt of a value with itself gives that value."),
            "Tilt's idempotence is real and must be discovered; got {proses:?}"
        );

        // and `check` on the frozen false law is the same probe from the other door: ops = [Tilt],
        // vars x = 0, y = 1.
        let frozen = DiscoveredLaw {
            prose: "Tilt gives the same result in either order.".into(),
            equation: "(x >< y) = (y >< x)".into(),
            lhs: Term::App(0, vec![Term::Var(0), Term::Var(1)]),
            rhs: Term::App(0, vec![Term::Var(1), Term::Var(0)]),
        };
        assert!(
            e.check(&[frozen]).is_err(),
            "check certified a false commutativity — the grid never reaches (Lo, Hi)"
        );
    }

    // A two-sort theory (one 2-element sort, one 3-element sort, three variables each) to probe
    // variable independence in the sampled regime: 2³·3³ = 216 > the exhaustive cap, so this grid
    // comes from the mixed-radix stride sampler.
    #[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash, Debug)]
    enum MSort {
        B2,
        T3,
    }
    struct Mixed;
    #[derive(Clone)]
    enum MVal {
        B(bool),
        T(u8),
    }
    fn mxor(v: &[MVal]) -> Option<MVal> {
        match (&v[0], &v[1]) {
            (MVal::B(a), MVal::B(b)) => Some(MVal::B(a ^ b)),
            _ => None,
        }
    }
    fn mmax(v: &[MVal]) -> Option<MVal> {
        match (&v[0], &v[1]) {
            (MVal::T(a), MVal::T(b)) => Some(MVal::T(*a.max(b))),
            _ => None,
        }
    }

    impl Theory for Mixed {
        type Sort = MSort;
        type Value = MVal;
        type Obs = u8;
        fn name() -> &'static str {
            "mixed"
        }
        fn operators() -> Vec<Operator<Self>> {
            vec![
                Operator {
                    name: "Xor",
                    symbol: "^",
                    fixity: Fixity::Infix,
                    inputs: vec![MSort::B2, MSort::B2],
                    output: MSort::B2,
                    eval: mxor,
                },
                Operator {
                    name: "Max",
                    symbol: "max",
                    fixity: Fixity::Prefix,
                    inputs: vec![MSort::T3, MSort::T3],
                    output: MSort::T3,
                    eval: mmax,
                },
            ]
        }
        fn inhabitants(s: Self::Sort) -> Vec<Self::Value> {
            match s {
                MSort::B2 => vec![MVal::B(false), MVal::B(true)],
                MSort::T3 => (0..3).map(MVal::T).collect(),
            }
        }
        fn sort_of(v: &Self::Value) -> Self::Sort {
            match v {
                MVal::B(_) => MSort::B2,
                MVal::T(_) => MSort::T3,
            }
        }
        fn observe(v: &Self::Value) -> Self::Obs {
            match v {
                MVal::B(b) => *b as u8,
                MVal::T(n) => *n,
            }
        }
    }

    /// VARIABLE INDEPENDENCE, read straight off the grid: for every pair of distinct variables
    /// there is an assignment where they take DIFFERENT inhabitants of their sorts and one where
    /// they take the SAME (by position in the sort's inhabitant list — derived from the grid and
    /// `inhabitants`, no hand-picked expectations). The old stride made each variable an affine
    /// function of `k`, so same-cardinality variables were locked together: on a 2-element sort
    /// x ≡ z on every assignment (the "differ" half never happens), and on a 3-element sort the
    /// middle variable was pinned constant (failing "differ" against everything it happened to
    /// start equal to, and "agree" against the rest). Any sampler whose variables are functions
    /// of each other fails this test.
    #[test]
    fn every_variable_pair_both_agrees_and_differs_on_the_grid() {
        let e = Engine::<Mixed>::new();
        let sorts = e.var_sorts();
        assert_eq!(sorts.len(), 6, "three variables per sort, two sorts");

        // each assignment as inhabitant COORDINATES: variable -> index into its sort's inhabitants.
        let coords: Vec<Vec<usize>> = e
            .grid_assignments()
            .iter()
            .map(|asn| {
                asn.iter()
                    .zip(&sorts)
                    .map(|(v, &s)| {
                        Mixed::inhabitants(s)
                            .iter()
                            .position(|i| Mixed::observe(i) == Mixed::observe(v))
                            .expect("grid value must be one of its sort's inhabitants")
                    })
                    .collect()
            })
            .collect();
        assert!(!coords.is_empty(), "the grid must not be empty");

        for i in 0..sorts.len() {
            for j in (i + 1)..sorts.len() {
                assert!(
                    coords.iter().any(|c| c[i] != c[j]),
                    "variables {i} ({:?}) and {j} ({:?}) NEVER differ — they are aliased, \
                     and any law relating them is judged on a diagonal slice only",
                    sorts[i],
                    sorts[j]
                );
                assert!(
                    coords.iter().any(|c| c[i] == c[j]),
                    "variables {i} ({:?}) and {j} ({:?}) never agree — the diagonal (where \
                     idempotence-like laws live) is unsampled",
                    sorts[i],
                    sorts[j]
                );
            }
        }
    }
}
