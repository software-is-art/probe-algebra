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

        // variables: num_vars per sort.
        let mut vars: Vec<Var<T::Sort>> = Vec::new();
        for &sort in &sorts {
            for ord in 0..T::num_vars() {
                vars.push(Var { sort, ord });
            }
        }

        // the grid: a deterministic spread of assignments. Variable `i` at assignment `k` takes
        // inhabitant `(k * stride_i + ord) mod len` — coprime-ish strides separate the variables.
        let grid_size = T::grid_size();
        let mut grid: Vec<Vec<T::Value>> = Vec::with_capacity(grid_size);
        let inhabitants: Vec<Vec<T::Value>> = vars.iter().map(|v| T::inhabitants(v.sort)).collect();
        for k in 0..grid_size {
            let mut asn: Vec<T::Value> = Vec::with_capacity(vars.len());
            for (i, inh) in inhabitants.iter().enumerate() {
                let len = inh.len().max(1);
                let stride = 1 + (i * 2);
                let idx = (k.wrapping_mul(stride).wrapping_add(vars[i].ord)) % len;
                asn.push(inh[idx % inh.len()].clone());
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

    /// Does a signature carry any information (defined somewhere, and not vacuously identical)?
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

#[cfg(test)]
impl<T: Theory> Engine<T> {
    /// The raw equalities enumeration emits — for probing the discovery machinery itself.
    pub(crate) fn emitted_equalities(&self) -> Vec<(Term, Term)> {
        self.enumerate()
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
        assert_eq!(d.consequences, 1, "consequence count changed");
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
}
