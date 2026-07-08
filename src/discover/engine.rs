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
//!      annihilation, idempotence, the regular-band BIAS laws for non-commutative operators,
//!      distributivity, absorption, involution, round-trip, irreflexivity, and the heterogeneous
//!      shapes — monoid ACTION, HOMOMORPHISM) over the actual operators and keeps the ones that
//!      run true over a grid — named laws;
//!   3. counts every other discovered equality as a CONSEQUENCE, and reports which operators appear
//!      in no law (where the spec is silent).
//!
//! Nothing here knows about numbers: a domain implements `Theory` and its algebra discovers itself.

use std::collections::BTreeMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::path::PathBuf;

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
    /// How this theory JUDGES two observations for law purposes. The default is exact
    /// equality — right for every decidable carrier. A metric/setoid carrier (floats,
    /// constructive reals as Cauchy data) overrides with its REGISTERED bars: closer
    /// than the holds-bar judges `Holds`, farther than the refutes-bar judges
    /// `Refuted`, and the band between is `Undecided` — disclosed, never silently
    /// binned. Pair every override with [`Theory::tolerance`], so the bars ship in the
    /// lock text and review ratifies the tolerance along with the laws.
    ///
    /// SCOPE: law judgment only (the template driver, `Engine::check`, witnesses).
    /// Enumeration and canonicalization keep exact equality — a toleranced relation is
    /// not transitive, so it cannot key the term-collision maps; the consequence count
    /// therefore remains an exact-equality fact, disclosed.
    fn judge(a: &Self::Obs, b: &Self::Obs) -> Verdict {
        if a == b {
            Verdict::Holds
        } else {
            Verdict::Refuted
        }
    }
    /// The REGISTERED tolerance, as display text for the lock header ("micro-units: \
    /// exact holds; |Δ| ≤ 2 undecided; else refuted") — `Some` exactly when [`Theory::judge`]
    /// is overridden, so ε is part of the ratified artifact, never ambient.
    fn tolerance() -> Option<&'static str> {
        None
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

/// Which way a law relates its two terms. Equational shapes state a universal EQUALITY
/// (`∀: lhs = rhs`); witness shapes state an INEQUATION (`∃: lhs ≠ rhs`) — the polarity the
/// catalog was missing, found by the algebra-mutation harness: the trivial action satisfies
/// every action EQUATION, the never-true relation satisfies irreflexivity, the unpinned
/// operator satisfies nothing and nobody notices. "This thing actually does something" is
/// only sayable as an inequation.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Polarity {
    /// `lhs = rhs` on EVERY grid assignment — the universal equality every classic shape states.
    Equal,
    /// `lhs ≠ rhs` on SOME grid assignment — a witnessed inequation (grid-bounded like
    /// everything here: the witness refutes triviality, it never proves richness).
    Differs,
}

/// A discovered law: its plain-language and symbolic renderings, plus the two terms it equates
/// (so the spec can be re-probed over a fresh grid, where mutation judges its kill power).
#[derive(Clone)]
pub struct DiscoveredLaw {
    /// The ratified catalog shape this law instantiates — EXACTLY a `ShapeCatalog::inventory()`
    /// name, set at the `templates()` push site that minted the law. The tag is what the
    /// declared-expectations layer (`discover::expect`) compares against: prose is for humans,
    /// the tag is the machine-checkable identity of the shape.
    pub shape: &'static str,
    pub prose: String,
    pub equation: String,
    pub lhs: Term,
    pub rhs: Term,
    /// Equality (`=`, holds everywhere) or witnessed inequation (`≠`, differs somewhere).
    pub polarity: Polarity,
    /// The law's GUARD, where the shape is conditional: `(premise, truth)` terms — an
    /// assignment counts only where they evaluate equal. `None` for unconditional laws.
    /// A guarded law additionally requires its premise SATISFIABLE on the grid (a law
    /// that never fires its guard is vacuous, not true — the fixed-point lesson, guarded).
    pub premise: Option<(Term, Term)>,
}

#[crate::mutate]
impl DiscoveredLaw {
    /// The distinct operator symbols participating in this law, in first-appearance order
    /// (lhs pre-order, then rhs). `symbols` is the theory's symbol table in operator-index
    /// order — `Engine::signatures()` supplies it. Together with `shape` this is the law's
    /// full identity for the declared-expectations gate: `identity` over `[grant, zero]` is
    /// a different fact from `identity` over `[renew, zero]`.
    pub fn ops(&self, symbols: &[&'static str]) -> Vec<&'static str> {
        fn walk(t: &Term, symbols: &[&'static str], out: &mut Vec<&'static str>) {
            if let Term::App(op, args) = t {
                let sym = symbols[*op];
                if !out.contains(&sym) {
                    out.push(sym);
                }
                for a in args {
                    walk(a, symbols, out);
                }
            }
        }
        let mut out = Vec::new();
        walk(&self.lhs, symbols, &mut out);
        walk(&self.rhs, symbols, &mut out);
        if let Some((premise, truth)) = &self.premise {
            walk(premise, symbols, &mut out);
            walk(truth, symbols, &mut out);
        }
        out
    }
}

/// The full result of discovery over a theory.
pub struct Discovered {
    /// The named laws — one canonical representative per recognised shape.
    pub laws: Vec<DiscoveredLaw>,
    /// How many further equalities were found that no shape named (all consequences).
    pub consequences: usize,
    /// Operators (by symbol) that participate in NO named law — where the spec is silent.
    pub uncovered_ops: Vec<&'static str>,
    /// Candidate laws that landed in the DECLARED tolerance's undecided band — neither
    /// held nor refuted, disclosed as `(prose, equation)` so the lock can say so. Always
    /// empty for exact-equality theories (the default [`Theory::judge`]).
    pub undecided: Vec<(String, String)>,
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

/// A three-valued law judgment at one grid assignment: exact carriers only ever say
/// `Holds`/`Refuted`; a toleranced carrier ([`Theory::judge`]) adds the DISCLOSED middle.
/// Without the undecided band, a toleranced gate is a coin flip exactly where metric
/// domains live — near the boundary.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Verdict {
    /// The two observations agree at the declared precision.
    Holds,
    /// The two observations clearly differ — a refutation-strength difference.
    Refuted,
    /// Inside the declared band: neither held nor refuted, and said so.
    Undecided,
}

/// A term's behavioural signature: the observation at each grid assignment (`None` where undefined).
type Sig<T> = Vec<Option<<T as Theory>::Obs>>;

/// The sampling stride for a grid space too large to enumerate: the first integer at or above the
/// golden-ratio point `space · φ⁻¹` (the classic low-discrepancy multiplier) that is coprime to
/// the space, so `k · step (mod space)` visits distinct, well-spread assignments.
#[crate::mutate]
fn coprime_step(space: u128) -> u128 {
    let mut step = (space.saturating_mul(618) / 1000).max(1);
    while gcd(step, space) != 1 {
        step += 1;
    }
    step
}

#[crate::mutate]
fn gcd(a: u128, b: u128) -> u128 {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

/// An operator's evaluator, as the bare `fn` pointer the signature table stores — also the
/// exact granule the algebra-mutation harness (`discover::mutation`) swaps out.
pub(crate) type EvalFn<T> = fn(&[<T as Theory>::Value]) -> Option<<T as Theory>::Value>;

/// An operator's signature, by index: `(symbol, input sorts, output sort)`.
pub type OpSignature<S> = (&'static str, Vec<S>, S);

/// An operator's full declaration: `(name, symbol, fixity, input sorts, output sort)`.
pub type OpDeclaration<S> = (&'static str, &'static str, Fixity, Vec<S>, S);

/// Is `op` a homogeneous binary operator on sort `s` (`s × s -> s`)? Used by every shape that needs
/// a binary on a given sort, so a mutation to this predicate breaks laws across all theories.
#[crate::mutate]
fn is_binary_on<T: Theory>(op: &Operator<T>, s: T::Sort) -> bool {
    op.inputs.len() == 2 && op.inputs[0] == s && op.inputs[1] == s && op.output == s
}

#[crate::mutate]
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

    /// Judge one assignment's pair of observations: `None == None` holds (both sides
    /// undefined together, the partial-operator convention); a definedness mismatch
    /// refutes; two defined values go to the theory's [`Theory::judge`].
    fn judge_at(a: &Option<T::Obs>, b: &Option<T::Obs>) -> Verdict {
        match (a, b) {
            (None, None) => Verdict::Holds,
            (Some(x), Some(y)) => T::judge(x, y),
            _ => Verdict::Refuted,
        }
    }

    /// Judge two whole signatures: `Refuted` if any assignment refutes, else `Undecided`
    /// if any assignment is undecided, else `Holds`.
    fn judge_sigs(a: &Sig<T>, b: &Sig<T>) -> Verdict {
        let mut verdict = Verdict::Holds;
        for (x, y) in a.iter().zip(b.iter()) {
            match Self::judge_at(x, y) {
                Verdict::Holds => {}
                Verdict::Undecided => verdict = Verdict::Undecided,
                Verdict::Refuted => return Verdict::Refuted,
            }
        }
        verdict
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

// ================================================================================================
// THE SHAPE CATALOG IS THE ENGINE. There is no second artifact: `Engine::templates()` is a
// generic interpreter over `ShapeCatalog::inventory()`, so a shape's gate decides where it
// fires, its canonical terms are what discovery runs, its template is the prose a law renders
// with, and its polarity is the judgment. Adding a shape is adding a STANZA OF DATA below —
// ratified through the regenerated `spec/shapes.spec` diff, executable the moment it lands.
// (The old MOVE-TOGETHER discipline, where this block and a hand-written battery restated
// each other under census guard, is dissolved; the censuses remain as regression nets.)
// ================================================================================================

/// One universal algebraic shape, as a ratifiable datum: its name, its schematic equation, the
/// applicability GATE that decides which operators it is tried on (as prose for the lock AND as
/// checkable data), and the prose TEMPLATE a discovered instance renders with
/// (`{op}`/`{other}`/`{via}` are operator names, `{const}` a constant's symbol).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ShapeInfo {
    /// The shape's name ("commutativity", "bias (right-regular)").
    pub name: &'static str,
    /// The schematic equation, over placeholder operators ("(x ⊕ y) = (y ⊕ x)").
    pub schema: &'static str,
    /// When the shape is tried, in prose ("homogeneous binary; skipped when commutative").
    pub gate: &'static str,
    /// The same gate as DATA — the signature pattern a declaration is validated against
    /// (`ShapeGate::admit`), so "what may be declared" and "what discovery tries" are one
    /// artifact. Purely-behavioural conditions (bias's skipped-when-commutative) stay out:
    /// they are discovery's verdict, not a signature's shape.
    pub gate_slots: ShapeGate,
    /// The prose template a discovered instance renders with — the exact `format!` skeleton in
    /// `templates()`, with `{...}` holes where operator/constant names are substituted.
    pub template: &'static str,
    /// The canonical left-hand term, as DATA over slot indices — the single source a concrete
    /// symbolic equation renders from ([`ShapeInfo::equation`]). For the two-sided shapes
    /// (identity, annihilation) this is the left/first-tried variant, the one a declared
    /// expectation's target lock predicts.
    pub lhs: SchemaTerm,
    /// The canonical right-hand term (see `lhs`).
    pub rhs: SchemaTerm,
    /// The placeholder symbol per slot the DISPLAY `schema` string renders with (`⊕`, `act`,
    /// `e`) — the census test holds `schema` against `lhs`/`rhs` rendered through these, so
    /// the displayed string and the term data cannot drift apart.
    pub placeholders: &'static [&'static str],
    /// Equational (`=`, holds on every assignment) or WITNESS (`≠`, differs on some) — see
    /// [`Polarity`]. The witness shapes are the catalog's inequation half.
    pub polarity: Polarity,
    /// The template hole each slot fills, in slot order (`["op", "const"]`) — the single
    /// source for instantiating the prose over a concrete binding, engine- and genesis-side
    /// alike. A constant slot fills its hole with the operator's SYMBOL (`0`, `false`),
    /// every other slot with its NAME ("Addition") — the render `templates()` always used.
    pub holes: &'static [&'static str],
    /// Try the argument-swapped variant of `lhs`'s outer application too, under the same
    /// prose — how identity and annihilation are two-sided (`(e ⊕ x) = x` or `(x ⊕ e) = x`)
    /// without being two shapes.
    pub mirrored: bool,
    /// The shape's behavioural guard — the one applicability condition that is a GRID fact
    /// rather than a signature fact, as data (see [`Guard`]).
    pub guard: Guard,
    /// What the shape requires of its constant slot's symbol (see [`ConstRule`]) — how
    /// irreflexivity and self-application share one signature but split one vocabulary row.
    pub const_rule: ConstRule,
    /// The shape's PREMISE, where the law is conditional (`P ⟹ lhs = rhs`): a schematic
    /// term that must evaluate equal to the shape's CONSTANT slot (the truth reference)
    /// for an assignment to count. Requires a constant slot; judged only for
    /// `Polarity::Equal`. `None` = unconditional (every prior shape).
    pub premise: Option<SchemaTerm>,
}

/// A shape's behavioural applicability guard. Signature facts live in [`ShapeGate`]; this
/// is the residue that only the grid can decide.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Guard {
    /// No behavioural condition.
    None,
    /// Skip when the fire operator (slot 0) is commutative on the grid — a commutative
    /// operator has no bias to state, and the two bias variants exist only where order
    /// matters.
    FireOpNotCommutative,
}

/// A constraint on the SYMBOL bound to a shape's constant slot. `rel(x, x) = false` is
/// irreflexivity exactly when the constant renders as `false`; the same signature with any
/// other constant is a self-application law.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ConstRule {
    /// Any constant qualifies (or the shape has no constant slot).
    Any,
    /// The constant's symbol must be exactly this.
    Named(&'static str),
    /// The constant's symbol must NOT be this.
    NotNamed(&'static str),
}

/// A schematic term: the shape's canonical equation halves as data, over SLOT indices (into
/// `gate_slots.slots` / `placeholders`) and variables named by `(sort variable, ordinal)` —
/// the same sort variables the gate's slots bind. Rendering follows [`Engine`]'s term
/// renderer exactly (infix parenthesised, prefix `f(a, b)`, nullary bare), so a concrete
/// equation derived here is byte-identical to the one a confirming discovery renders.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SchemaTerm {
    /// The `ord`-th variable of sort variable `.0`.
    Var(u8, u8),
    /// Slot `.0`'s operator applied to argument terms (empty for a constant).
    App(u8, &'static [SchemaTerm]),
}

#[crate::mutate]
impl SchemaTerm {
    /// Render over concrete operators: `ops[slot]` is that slot's `(symbol, fixity)`;
    /// `vars[sort_var][ord]` names the variables. Mirrors `Engine::render` case for case.
    pub fn render(&self, ops: &[(&str, Fixity)], vars: &[&[&str]]) -> String {
        match self {
            SchemaTerm::Var(sort, ord) => vars[*sort as usize][*ord as usize].to_string(),
            SchemaTerm::App(slot, args) => {
                let (symbol, fixity) = ops[*slot as usize];
                match fixity {
                    Fixity::Nullary => symbol.to_string(),
                    Fixity::Infix => format!(
                        "({} {} {})",
                        args[0].render(ops, vars),
                        symbol,
                        args[1].render(ops, vars)
                    ),
                    Fixity::Prefix => {
                        let inner: Vec<String> = args.iter().map(|a| a.render(ops, vars)).collect();
                        format!("{}({})", symbol, inner.join(", "))
                    }
                }
            }
        }
    }
}

/// A shape's applicability gate as data: one [`Slot`] per operator the shape ranges over (in
/// the declared/fingerprint order), plus the distinctness constraints the code gate imposes.
/// Sort VARIABLES (the `u8`s inside slots) express how the slots' sorts relate — identity's
/// constant lives on its operator's sort, a homomorphism's binaries live on the conversion's
/// two endpoints — by unification, not by naming concrete sorts.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ShapeGate {
    /// One entry per operator parameter, in declaration order.
    pub slots: &'static [Slot],
    /// Sort-variable pairs that must bind to DIFFERENT sorts (an action's parameter is not
    /// its carrier; a relation's verdict is not its operand).
    pub distinct_sorts: &'static [(u8, u8)],
    /// Slot-index pairs that must be DIFFERENT operators (distributivity's two binaries, a
    /// round trip's two conversions).
    pub distinct_ops: &'static [(u8, u8)],
}

/// One operator slot of a gate, over sort variables.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Slot {
    /// A homogeneous binary on sort `.0` (`s × s → s`).
    Binary(u8),
    /// A nullary constant of sort `.0`.
    Constant(u8),
    /// A unary conversion from sort `.0` to sort `.1` (equal variables ⇒ an endo).
    Unary(u8, u8),
    /// An action of sort `.1` on sort `.0` (`s × t → s`).
    Action(u8, u8),
    /// A relation on sort `.0` with verdict sort `.1` (`s × s → r`).
    Relation(u8, u8),
}

#[crate::mutate]
impl ShapeGate {
    /// Do these operator signatures satisfy the gate? `sigs[i]` is slot `i`'s
    /// `(input sorts, output sort)` and `names[i]` its display name; sorts are any comparable
    /// tokens (the engine checks `T::Sort`s, genesis checks declared value NAMES — one
    /// checker, so the two sides cannot drift). `Err` names the first violated slot or
    /// constraint in the caller's vocabulary.
    pub fn admit<S: PartialEq + Clone>(
        &self,
        sigs: &[(Vec<S>, S)],
        names: &[&str],
    ) -> Result<(), String> {
        self.bind(sigs, names).map(|_| ())
    }

    /// [`ShapeGate::admit`], keeping its work: on success, the sort each sort VARIABLE
    /// bound to (indexed by variable; `None` where no slot constrained it). The generic
    /// template driver instantiates a shape's canonical terms through this binding, so
    /// admission and instantiation are one computation, never two.
    pub(crate) fn bind<S: PartialEq + Clone>(
        &self,
        sigs: &[(Vec<S>, S)],
        names: &[&str],
    ) -> Result<Vec<Option<S>>, String> {
        if sigs.len() != self.slots.len() || names.len() != self.slots.len() {
            return Err(format!(
                "the shape ranges over {} operator(s), got {}",
                self.slots.len(),
                sigs.len().max(names.len())
            ));
        }
        let mut bound: Vec<Option<S>> = Vec::new();
        let bind = |bound: &mut Vec<Option<S>>, var: u8, sort: &S| -> bool {
            let var = var as usize;
            if bound.len() <= var {
                bound.resize(var + 1, None);
            }
            match &bound[var] {
                Some(existing) => existing == sort,
                None => {
                    bound[var] = Some(sort.clone());
                    true
                }
            }
        };
        for (i, slot) in self.slots.iter().enumerate() {
            let (inputs, output) = &sigs[i];
            let name = names[i];
            let ok = match slot {
                Slot::Binary(s) => {
                    inputs.len() == 2
                        && inputs[0] == *output
                        && inputs[1] == *output
                        && bind(&mut bound, *s, output)
                }
                Slot::Constant(s) => inputs.is_empty() && bind(&mut bound, *s, output),
                Slot::Unary(from, to) => {
                    inputs.len() == 1
                        && bind(&mut bound, *from, &inputs[0])
                        && bind(&mut bound, *to, output)
                }
                Slot::Action(carrier, param) => {
                    inputs.len() == 2
                        && inputs[0] == *output
                        && bind(&mut bound, *carrier, output)
                        && bind(&mut bound, *param, &inputs[1])
                }
                Slot::Relation(operand, verdict) => {
                    inputs.len() == 2
                        && inputs[0] == inputs[1]
                        && bind(&mut bound, *operand, &inputs[0])
                        && bind(&mut bound, *verdict, output)
                }
            };
            if !ok {
                let wanted = match slot {
                    Slot::Binary(_) => "a homogeneous binary operator (s × s → s)",
                    Slot::Constant(_) => "a nullary constant of the matching sort",
                    Slot::Unary(..) => "a unary conversion between the matching sorts",
                    Slot::Action(..) => "an action (s × t → s) on the matching carrier",
                    Slot::Relation(..) => "a relation (s × s → r) on the matching operand sort",
                };
                return Err(format!("`{name}` must be {wanted}"));
            }
        }
        for (a, b) in self.distinct_sorts {
            let (a, b) = (*a as usize, *b as usize);
            if bound.get(a).cloned().flatten() == bound.get(b).cloned().flatten() {
                return Err(format!(
                    "the sorts bound by `{}` must be distinct (an action's parameter is not \
                     its carrier; a relation's verdict is not its operand)",
                    names.join("`, `")
                ));
            }
        }
        for (a, b) in self.distinct_ops {
            if names[*a as usize] == names[*b as usize] {
                return Err(format!(
                    "`{}` and `{}` must be different operators",
                    names[*a as usize], names[*b as usize]
                ));
            }
        }
        Ok(bound)
    }
}

#[crate::mutate]
impl ShapeInfo {
    /// Does a discovered law's prose instantiate this shape's template? The template's literal
    /// fragments (around the `{...}` holes) must appear in the prose IN ORDER — the first as a
    /// prefix, the last as a suffix — with the holes free to match any operator or constant name.
    /// This is the census's matcher: robust to what a domain calls its operators, strict about
    /// the ratified prose skeleton.
    pub fn matches(&self, prose: &str) -> bool {
        // split the template into literal fragments around the `{...}` holes.
        let mut fragments: Vec<&str> = Vec::new();
        let mut rest = self.template;
        while let (Some(open), Some(len)) = (
            rest.find('{'),
            rest.find('{').and_then(|o| rest[o..].find('}')),
        ) {
            fragments.push(&rest[..open]);
            rest = &rest[open + len + 1..];
        }
        fragments.push(rest);

        // no holes at all: a literal template must equal the prose outright.
        let (first, holed) = fragments.split_first().expect("at least one fragment");
        let Some((last, mids)) = holed.split_last() else {
            return prose == *first;
        };
        let Some(mut cursor) = prose.strip_prefix(first) else {
            return false;
        };
        for mid in mids {
            match cursor.find(mid) {
                Some(pos) => cursor = &cursor[pos + mid.len()..],
                None => return false,
            }
        }
        cursor.ends_with(last)
    }

    /// Instantiate the prose template over concrete names: fill each `{hole}` from `subs`
    /// (`[("op", "grant"), ("const", "zero")]`). The catalog is thereby the SINGLE source of
    /// a law's prose — genesis renders its target locks through this, so a declared law's
    /// lock line and a confirming discovery's render cannot drift apart.
    pub fn instantiate(&self, subs: &[(&str, &str)]) -> String {
        let mut out = self.template.to_string();
        for (hole, name) in subs {
            out = out.replace(&format!("{{{hole}}}"), name);
        }
        out
    }

    /// The shape's canonical symbolic equation over concrete operators — `lhs`/`rhs` rendered
    /// through `ops` (one `(symbol, fixity)` per slot, in slot order) and `vars` (variable
    /// letters per sort variable), joined by the polarity's connective (`=` for equations,
    /// `≠` for witness shapes). The catalog is thereby the SINGLE source of a law's
    /// equation, exactly as `instantiate` makes it the single source of the prose — genesis's
    /// target locks derive their equations here instead of restating the render format.
    pub fn equation(&self, ops: &[(&str, Fixity)], vars: &[&[&str]]) -> String {
        let connective = match self.polarity {
            Polarity::Equal => "=",
            Polarity::Differs => "≠",
        };
        let body = format!(
            "{} {connective} {}",
            self.lhs.render(ops, vars),
            self.rhs.render(ops, vars)
        );
        match &self.premise {
            None => body,
            // the guard renders against the shape's constant slot — the truth reference.
            Some(premise) => {
                let truth = self
                    .gate_slots
                    .slots
                    .iter()
                    .position(|s| matches!(s, Slot::Constant(_)))
                    .expect("a conditional shape carries a constant slot");
                format!("{} = {} ⟹ {body}", premise.render(ops, vars), ops[truth].0)
            }
        }
    }
}

/// The engine's shape catalog — the library's LAW-LANGUAGE surface, as a value object.
///
/// Every law any theory's discovered spec can ever state is an instance of one shape here: the
/// catalog is the vocabulary discovery speaks in. Until now that vocabulary existed only as code
/// (`templates()`), so adding a shape silently changed what EVERY downstream consumer's discovered
/// spec contains — the regular-band BIAS laws are the motivating example: the day they landed,
/// two frozen specs (router, ttl store) changed underneath their consumers with no artifact
/// naming the cause. The catalog closes that: `inventory()` states the battery as data,
/// `lock()` freezes its deterministic rendering into `spec/shapes.spec`, and the drift gate makes
/// template evolution a RATIFIED, VERSIONED event — a new or changed shape is a reviewed diff to
/// the committed catalog (regenerate with `cargo run --example freeze_shapes`), not a silent
/// side effect of an engine edit. Associated fns per the no-rats-nest rule: every public callable
/// hangs off a typestate.
pub struct ShapeCatalog;

#[crate::mutate]
impl ShapeCatalog {
    /// The full inventory — the order IS the order discovery tries (and therefore renders)
    /// the shapes within each polarity band, and `spec/shapes.spec` locks it. This is the
    /// engine's whole battery: `templates()` interprets these stanzas, nothing else.
    pub fn inventory() -> Vec<ShapeInfo> {
        // the data-gate shorthand (prose gates render into the lock; these validate).
        const fn open(slots: &'static [Slot]) -> ShapeGate {
            ShapeGate {
                slots,
                distinct_sorts: &[],
                distinct_ops: &[],
            }
        }
        const BINARY: ShapeGate = open(&[Slot::Binary(0)]);
        const WITH_CONSTANT: ShapeGate = open(&[Slot::Binary(0), Slot::Constant(0)]);
        const BINARY_PAIR: ShapeGate = ShapeGate {
            slots: &[Slot::Binary(0), Slot::Binary(0)],
            distinct_sorts: &[],
            distinct_ops: &[(0, 1)],
        };
        const HETERO: &[(u8, u8)] = &[(0, 1)];

        // the canonical-term shorthand: variables of the carrier sort (X, Y, Z), of the
        // action-parameter sort (P, Q), and slot 1's constant (C — identity's `e`,
        // annihilation's `a`, a relation's verdict constant).
        use SchemaTerm::{App, Var};
        const X: SchemaTerm = Var(0, 0);
        const Y: SchemaTerm = Var(0, 1);
        const Z: SchemaTerm = Var(0, 2);
        const P: SchemaTerm = Var(1, 0);
        const Q: SchemaTerm = Var(1, 1);
        const C: SchemaTerm = App(1, &[]);

        vec![
            ShapeInfo {
                name: "commutativity",
                schema: "(x ⊕ y) = (y ⊕ x)",
                gate: "homogeneous binary (s × s → s)",
                gate_slots: BINARY,
                template: "{op} gives the same result in either order.",
                lhs: App(0, &[X, Y]),
                rhs: App(0, &[Y, X]),
                placeholders: &["⊕"],
                polarity: Polarity::Equal,
                holes: &["op"],
                mirrored: false,
                guard: Guard::None,
                const_rule: ConstRule::Any,
                premise: None,
            },
            ShapeInfo {
                name: "associativity",
                schema: "((x ⊕ y) ⊕ z) = (x ⊕ (y ⊕ z))",
                gate: "homogeneous binary (s × s → s)",
                gate_slots: BINARY,
                template: "With {op}, the grouping of three values doesn't matter.",
                lhs: App(0, &[App(0, &[X, Y]), Z]),
                rhs: App(0, &[X, App(0, &[Y, Z])]),
                placeholders: &["⊕"],
                polarity: Polarity::Equal,
                holes: &["op"],
                mirrored: false,
                guard: Guard::None,
                const_rule: ConstRule::Any,
                premise: None,
            },
            ShapeInfo {
                name: "idempotence",
                schema: "(x ⊕ x) = x",
                gate: "homogeneous binary (s × s → s)",
                gate_slots: BINARY,
                template: "{op} of a value with itself gives that value.",
                lhs: App(0, &[X, X]),
                rhs: X,
                placeholders: &["⊕"],
                polarity: Polarity::Equal,
                holes: &["op"],
                mirrored: false,
                guard: Guard::None,
                const_rule: ConstRule::Any,
                premise: None,
            },
            ShapeInfo {
                name: "bias (right-regular)",
                schema: "((x ⊕ y) ⊕ x) = (y ⊕ x)",
                gate: "homogeneous binary; skipped when commutative (a commutative operator \
                       has no bias to state); excludes the left-regular variant on the grid",
                gate_slots: BINARY,
                template: "With {op}, the later operand wins where the two disagree — \
                           re-applying an earlier one cannot overwrite it.",
                lhs: App(0, &[App(0, &[X, Y]), X]),
                rhs: App(0, &[Y, X]),
                placeholders: &["⊕"],
                polarity: Polarity::Equal,
                holes: &["op"],
                mirrored: false,
                guard: Guard::FireOpNotCommutative,
                const_rule: ConstRule::Any,
                premise: None,
            },
            ShapeInfo {
                name: "bias (left-regular)",
                schema: "((x ⊕ y) ⊕ x) = (x ⊕ y)",
                gate: "homogeneous binary; skipped when commutative (a commutative operator \
                       has no bias to state); excludes the right-regular variant on the grid",
                gate_slots: BINARY,
                template: "With {op}, the earlier operand wins where the two disagree — \
                           a later one cannot overwrite it.",
                lhs: App(0, &[App(0, &[X, Y]), X]),
                rhs: App(0, &[X, Y]),
                placeholders: &["⊕"],
                polarity: Polarity::Equal,
                holes: &["op"],
                mirrored: false,
                guard: Guard::FireOpNotCommutative,
                const_rule: ConstRule::Any,
                premise: None,
            },
            ShapeInfo {
                name: "identity",
                schema: "(e ⊕ x) = x  (or (x ⊕ e) = x)",
                gate: "homogeneous binary plus a constant of its sort; tried on both sides, \
                       deduplicated by prose",
                gate_slots: WITH_CONSTANT,
                template: "{op} with {const} leaves a value unchanged.",
                lhs: App(0, &[C, X]),
                rhs: X,
                placeholders: &["⊕", "e"],
                polarity: Polarity::Equal,
                holes: &["op", "const"],
                mirrored: true,
                guard: Guard::None,
                const_rule: ConstRule::Any,
                premise: None,
            },
            ShapeInfo {
                name: "annihilation",
                schema: "(a ⊕ x) = a  (or (x ⊕ a) = a)",
                gate: "homogeneous binary plus a constant of its sort; tried on both sides, \
                       deduplicated by prose",
                gate_slots: WITH_CONSTANT,
                template: "{op} by {const} always gives {const}.",
                lhs: App(0, &[C, X]),
                rhs: C,
                placeholders: &["⊕", "a"],
                polarity: Polarity::Equal,
                holes: &["op", "const"],
                mirrored: true,
                guard: Guard::None,
                const_rule: ConstRule::Any,
                premise: None,
            },
            ShapeInfo {
                name: "inverse",
                schema: "(x ⊕ inv(x)) = e",
                gate: "homogeneous binary plus a unary endo and a constant, all on one \
                       sort; tried on both sides, deduplicated by prose",
                gate_slots: open(&[Slot::Binary(0), Slot::Unary(0, 0), Slot::Constant(0)]),
                template: "{other} inverts {op} — a value {op} its own {other} gives {const}.",
                lhs: App(0, &[X, App(1, &[X])]),
                rhs: App(2, &[]),
                placeholders: &["⊕", "inv", "e"],
                polarity: Polarity::Equal,
                holes: &["op", "other", "const"],
                mirrored: true,
                guard: Guard::None,
                const_rule: ConstRule::Any,
                premise: None,
            },
            ShapeInfo {
                name: "self-inverse",
                schema: "(x ⊕ x) = e",
                gate: "homogeneous binary plus a constant of its sort — every element its \
                       own inverse (the Boolean-group law `inverse` cannot say, because \
                       the inverting map is the identity and identity is not an operator)",
                gate_slots: WITH_CONSTANT,
                template: "{op} of a value with itself gives {const} — every element is \
                           its own inverse.",
                lhs: App(0, &[X, X]),
                rhs: C,
                placeholders: &["⊕", "e"],
                polarity: Polarity::Equal,
                holes: &["op", "const"],
                mirrored: false,
                guard: Guard::None,
                const_rule: ConstRule::Any,
                premise: None,
            },
            ShapeInfo {
                name: "distributivity",
                schema: "(x ⊕ (y ⊗ z)) = ((x ⊕ y) ⊗ (x ⊕ z))",
                gate: "an ordered pair of distinct homogeneous binaries on one sort",
                gate_slots: BINARY_PAIR,
                template: "{op} distributes over {other}.",
                lhs: App(0, &[X, App(1, &[Y, Z])]),
                rhs: App(1, &[App(0, &[X, Y]), App(0, &[X, Z])]),
                placeholders: &["⊕", "⊗"],
                polarity: Polarity::Equal,
                holes: &["op", "other"],
                mirrored: false,
                guard: Guard::None,
                const_rule: ConstRule::Any,
                premise: None,
            },
            ShapeInfo {
                name: "distributivity (right)",
                schema: "((y ⊗ z) ⊕ x) = ((y ⊕ x) ⊗ (z ⊕ x))",
                gate: "an ordered pair of distinct homogeneous binaries on one sort; \
                       skipped when the first is commutative (the left-slot law already \
                       says it) — the other slot's distributivity, so each argument \
                       position carries its own additivity law",
                gate_slots: BINARY_PAIR,
                template: "{op} distributes over {other} from the right.",
                lhs: App(0, &[App(1, &[Y, Z]), X]),
                rhs: App(1, &[App(0, &[Y, X]), App(0, &[Z, X])]),
                placeholders: &["⊕", "⊗"],
                polarity: Polarity::Equal,
                holes: &["op", "other"],
                mirrored: false,
                guard: Guard::FireOpNotCommutative,
                const_rule: ConstRule::Any,
                premise: None,
            },
            ShapeInfo {
                name: "absorption",
                schema: "(x ⊕ (x ⊗ y)) = x",
                gate: "an ordered pair of distinct homogeneous binaries on one sort",
                gate_slots: BINARY_PAIR,
                template: "{op} absorbs {other}.",
                lhs: App(0, &[X, App(1, &[X, Y])]),
                rhs: X,
                placeholders: &["⊕", "⊗"],
                polarity: Polarity::Equal,
                holes: &["op", "other"],
                mirrored: false,
                guard: Guard::None,
                const_rule: ConstRule::Any,
                premise: None,
            },
            ShapeInfo {
                name: "action identity",
                schema: "act(x, e) = x",
                gate: "heterogeneous binary s × t → s (an action of t on s) plus a constant \
                       of the parameter sort t",
                gate_slots: ShapeGate {
                    slots: &[Slot::Action(0, 1), Slot::Constant(1)],
                    distinct_sorts: HETERO,
                    distinct_ops: &[],
                },
                template: "{op} with {const} leaves a value unchanged.",
                lhs: App(0, &[X, C]),
                rhs: X,
                placeholders: &["act", "e"],
                polarity: Polarity::Equal,
                holes: &["op", "const"],
                mirrored: false,
                guard: Guard::None,
                const_rule: ConstRule::Any,
                premise: None,
            },
            ShapeInfo {
                name: "monoid action",
                schema: "act(act(x, p), q) = act(x, (p ⊗ q))",
                gate: "heterogeneous binary s × t → s plus a homogeneous binary on the \
                       parameter sort t",
                gate_slots: ShapeGate {
                    slots: &[Slot::Action(0, 1), Slot::Binary(1)],
                    distinct_sorts: HETERO,
                    distinct_ops: &[],
                },
                template: "Repeated {op} combines its parameters with {other}.",
                lhs: App(0, &[App(0, &[X, P]), Q]),
                rhs: App(0, &[X, App(1, &[P, Q])]),
                placeholders: &["act", "⊗"],
                polarity: Polarity::Equal,
                holes: &["op", "other"],
                mirrored: false,
                guard: Guard::None,
                const_rule: ConstRule::Any,
                premise: None,
            },
            ShapeInfo {
                name: "action idempotence",
                schema: "act(act(x, p), p) = act(x, p)",
                gate: "heterogeneous binary s × t → s (an action of t on s)",
                gate_slots: ShapeGate {
                    slots: &[Slot::Action(0, 1)],
                    distinct_sorts: HETERO,
                    distinct_ops: &[],
                },
                template: "Repeated {op} with one parameter settles on the first \
                           application.",
                lhs: App(0, &[App(0, &[X, P]), P]),
                rhs: App(0, &[X, P]),
                placeholders: &["act"],
                polarity: Polarity::Equal,
                holes: &["op"],
                mirrored: false,
                guard: Guard::None,
                const_rule: ConstRule::Any,
                premise: None,
            },
            ShapeInfo {
                name: "action commutation",
                schema: "act(act(x, p), q) = act(act(x, q), p)",
                gate: "heterogeneous binary s × t → s (an action of t on s)",
                gate_slots: ShapeGate {
                    slots: &[Slot::Action(0, 1)],
                    distinct_sorts: HETERO,
                    distinct_ops: &[],
                },
                template: "{op} applications commute — the parameter order doesn't matter.",
                lhs: App(0, &[App(0, &[X, P]), Q]),
                rhs: App(0, &[App(0, &[X, Q]), P]),
                placeholders: &["act"],
                polarity: Polarity::Equal,
                holes: &["op"],
                mirrored: false,
                guard: Guard::None,
                const_rule: ConstRule::Any,
                premise: None,
            },
            ShapeInfo {
                name: "action equivariance",
                schema: "act((x ⊕ y), p) = (act(x, p) ⊕ act(y, p))",
                gate: "heterogeneous binary s × t → s (an action of t on s) plus a \
                       homogeneous binary on the carrier sort s",
                gate_slots: ShapeGate {
                    slots: &[Slot::Action(0, 1), Slot::Binary(0)],
                    distinct_sorts: HETERO,
                    distinct_ops: &[],
                },
                template: "{op} distributes over {other} — acting on a combination is \
                           combining the actions.",
                lhs: App(0, &[App(1, &[X, Y]), P]),
                rhs: App(1, &[App(0, &[X, P]), App(0, &[Y, P])]),
                placeholders: &["act", "⊕"],
                polarity: Polarity::Equal,
                holes: &["op", "other"],
                mirrored: false,
                guard: Guard::None,
                const_rule: ConstRule::Any,
                premise: None,
            },
            ShapeInfo {
                name: "action fixed point",
                schema: "act(c, p) = c",
                gate: "heterogeneous binary s × t → s (an action of t on s) plus a \
                       constant of the carrier sort s",
                gate_slots: ShapeGate {
                    slots: &[Slot::Action(0, 1), Slot::Constant(0)],
                    distinct_sorts: HETERO,
                    distinct_ops: &[],
                },
                template: "{op} leaves {const} fixed — no parameter moves it.",
                lhs: App(0, &[C, P]),
                rhs: C,
                placeholders: &["act", "c"],
                polarity: Polarity::Equal,
                holes: &["op", "const"],
                mirrored: false,
                guard: Guard::None,
                const_rule: ConstRule::Any,
                premise: None,
            },
            // -- the ORDERED-ACTION pair: an action judged against an order on its
            // carrier. Born from the fabric theory's first mutation sweep: `grant` and
            // `revoke` carried identical law sets (idempotent, commuting, equivariant,
            // nontrivial), so confusing them SURVIVED — the equational language could
            // not say which direction an action moves a value. These two stanzas say
            // exactly that, and nothing else.
            ShapeInfo {
                name: "action inflation",
                schema: "le(x, act(x, p)) = true  (x ≤ act(x, p) — the action only grows)",
                gate: "heterogeneous binary s × t → s (an action of t on s) plus an \
                       order relation s × s → r (r ≠ s) whose output sort carries a \
                       constant rendering as `true`",
                gate_slots: ShapeGate {
                    slots: &[Slot::Action(0, 1), Slot::Relation(0, 2), Slot::Constant(2)],
                    distinct_sorts: HETERO,
                    distinct_ops: &[],
                },
                template: "{op} only grows a value — never shrinks it (under {via}).",
                lhs: App(1, &[X, App(0, &[X, P])]),
                rhs: App(2, &[]),
                placeholders: &["act", "le", "true"],
                polarity: Polarity::Equal,
                holes: &["op", "via", "const"],
                mirrored: false,
                guard: Guard::None,
                const_rule: ConstRule::Named("true"),
                premise: None,
            },
            ShapeInfo {
                name: "action deflation",
                schema: "le(act(x, p), x) = true  (act(x, p) ≤ x — the action only shrinks)",
                gate: "heterogeneous binary s × t → s (an action of t on s) plus an \
                       order relation s × s → r (r ≠ s) whose output sort carries a \
                       constant rendering as `true`",
                gate_slots: ShapeGate {
                    slots: &[Slot::Action(0, 1), Slot::Relation(0, 2), Slot::Constant(2)],
                    distinct_sorts: HETERO,
                    distinct_ops: &[],
                },
                template: "{op} only shrinks a value — never grows it (under {via}).",
                lhs: App(1, &[App(0, &[X, P]), X]),
                rhs: App(2, &[]),
                placeholders: &["act", "le", "true"],
                polarity: Polarity::Equal,
                holes: &["op", "via", "const"],
                mirrored: false,
                guard: Guard::None,
                const_rule: ConstRule::Named("true"),
                premise: None,
            },
            ShapeInfo {
                name: "symmetry",
                schema: "rel(x, y) = rel(y, x)",
                gate: "relation s × s → r (r ≠ s) — a symmetric distance says so here; an \
                       order refuses it",
                gate_slots: ShapeGate {
                    slots: &[Slot::Relation(0, 1)],
                    distinct_sorts: HETERO,
                    distinct_ops: &[],
                },
                template: "{op} is symmetric — the arguments' order doesn't matter.",
                lhs: App(0, &[X, Y]),
                rhs: App(0, &[Y, X]),
                placeholders: &["rel"],
                polarity: Polarity::Equal,
                holes: &["op"],
                mirrored: false,
                guard: Guard::None,
                const_rule: ConstRule::Any,
                premise: None,
            },
            ShapeInfo {
                name: "irreflexivity",
                schema: "rel(x, x) = false",
                gate: "relation s × s → r (r ≠ s) whose output sort carries a constant \
                       rendering as `false`",
                gate_slots: ShapeGate {
                    slots: &[Slot::Relation(0, 1), Slot::Constant(1)],
                    distinct_sorts: HETERO,
                    distinct_ops: &[],
                },
                template: "A value is never {op} itself.",
                lhs: App(0, &[X, X]),
                rhs: C,
                placeholders: &["rel", "false"],
                polarity: Polarity::Equal,
                holes: &["op", "const"],
                mirrored: false,
                guard: Guard::None,
                const_rule: ConstRule::Named("false"),
                premise: None,
            },
            ShapeInfo {
                name: "self-application",
                schema: "rel(x, x) = c",
                gate: "relation s × s → r (r ≠ s) plus any other constant of the output sort",
                gate_slots: ShapeGate {
                    slots: &[Slot::Relation(0, 1), Slot::Constant(1)],
                    distinct_sorts: HETERO,
                    distinct_ops: &[],
                },
                template: "{op} of a value with itself gives {const}.",
                lhs: App(0, &[X, X]),
                rhs: C,
                placeholders: &["rel", "c"],
                polarity: Polarity::Equal,
                holes: &["op", "const"],
                mirrored: false,
                guard: Guard::None,
                const_rule: ConstRule::NotNamed("false"),
                premise: None,
            },
            ShapeInfo {
                name: "involution",
                schema: "u(u(x)) = x",
                gate: "unary s → s",
                gate_slots: open(&[Slot::Unary(0, 0)]),
                template: "{op} twice returns the original value.",
                lhs: App(0, &[App(0, &[X])]),
                rhs: X,
                placeholders: &["u"],
                polarity: Polarity::Equal,
                holes: &["op"],
                mirrored: false,
                guard: Guard::None,
                const_rule: ConstRule::Any,
                premise: None,
            },
            ShapeInfo {
                name: "projection",
                schema: "u(u(x)) = u(x)",
                gate: "unary s → s",
                gate_slots: open(&[Slot::Unary(0, 0)]),
                template: "{op} is a projection — applying it twice is applying it once.",
                lhs: App(0, &[App(0, &[X])]),
                rhs: App(0, &[X]),
                placeholders: &["u"],
                polarity: Polarity::Equal,
                holes: &["op"],
                mirrored: false,
                guard: Guard::None,
                const_rule: ConstRule::Any,
                premise: None,
            },
            ShapeInfo {
                name: "fixed point",
                schema: "u(c) = c",
                gate: "unary endo s → s plus a constant of its sort",
                gate_slots: open(&[Slot::Unary(0, 0), Slot::Constant(0)]),
                template: "{op} leaves {const} fixed.",
                lhs: App(0, &[C]),
                rhs: C,
                placeholders: &["u", "c"],
                polarity: Polarity::Equal,
                holes: &["op", "const"],
                mirrored: false,
                guard: Guard::None,
                const_rule: ConstRule::Any,
                premise: None,
            },
            ShapeInfo {
                name: "round-trip",
                schema: "g(f(x)) = x",
                gate: "a pair of distinct unaries f : s → t and g : t → s",
                // sort VARIABLE 0 is the carrier (x's sort, g's output): slot 0 is the
                // outer g : 1 → 0, slot 1 the inner f : 0 → 1, so the canonical terms'
                // `Var(0, 0)` is the value the trip returns to.
                gate_slots: ShapeGate {
                    slots: &[Slot::Unary(1, 0), Slot::Unary(0, 1)],
                    distinct_sorts: &[],
                    distinct_ops: &[(0, 1)],
                },
                template: "{op} undoes {other} — the round trip is the identity.",
                lhs: App(0, &[App(1, &[X])]),
                rhs: X,
                placeholders: &["g", "f"],
                polarity: Polarity::Equal,
                holes: &["op", "other"],
                mirrored: false,
                guard: Guard::None,
                const_rule: ConstRule::Any,
                premise: None,
            },
            ShapeInfo {
                name: "homomorphism",
                schema: "h((x ⊕ y)) = (h(x) ⊗ h(y))",
                gate: "unary h : s → t plus a homogeneous binary on s and one on t",
                gate_slots: open(&[Slot::Unary(0, 1), Slot::Binary(0), Slot::Binary(1)]),
                template: "{op} turns {other} into {via}.",
                lhs: App(0, &[App(1, &[X, Y])]),
                rhs: App(2, &[App(0, &[X]), App(0, &[Y])]),
                placeholders: &["h", "⊕", "⊗"],
                polarity: Polarity::Equal,
                holes: &["op", "other", "via"],
                mirrored: false,
                guard: Guard::None,
                const_rule: ConstRule::Any,
                premise: None,
            },
            // -- the WITNESS shapes: the catalog's inequation half (∃, not ∀). Found by
            // the algebra-mutation harness: its four survivors were all statements no
            // equation can make — the trivial action, the never-true relation, the
            // unpinned operator. Same honest frame inverted: a witness REFUTES
            // triviality on the grid, it never proves richness.
            ShapeInfo {
                name: "action nontriviality",
                schema: "act(x, p) ≠ x  (for some x, p)",
                gate: "heterogeneous binary s × t → s (an action of t on s); a witness \
                       shape — holds when some parameter moves some value",
                gate_slots: ShapeGate {
                    slots: &[Slot::Action(0, 1)],
                    distinct_sorts: HETERO,
                    distinct_ops: &[],
                },
                template: "{op} actually acts — some parameter moves some value.",
                lhs: App(0, &[X, P]),
                rhs: X,
                placeholders: &["act"],
                polarity: Polarity::Differs,
                holes: &["op"],
                mirrored: false,
                guard: Guard::None,
                const_rule: ConstRule::Any,
                premise: None,
            },
            ShapeInfo {
                name: "non-constancy",
                schema: "rel(x, y) ≠ c  (for some x, y)",
                gate: "relation s × s → r (r ≠ s) plus a constant of the output sort; a \
                       witness shape — holds when the relation escapes the constant \
                       somewhere",
                gate_slots: ShapeGate {
                    slots: &[Slot::Relation(0, 1), Slot::Constant(1)],
                    distinct_sorts: HETERO,
                    distinct_ops: &[],
                },
                template: "{op} is not constantly {const}.",
                lhs: App(0, &[X, Y]),
                rhs: C,
                placeholders: &["rel", "c"],
                polarity: Polarity::Differs,
                holes: &["op", "const"],
                mirrored: false,
                guard: Guard::None,
                const_rule: ConstRule::Any,
                premise: None,
            },
            // -- the ORDERED-RELATION shapes: ∀-inequalities, stated as equations over a
            // theory's own declared order (`le(lhs, rhs) = true`). No new polarity: a
            // bounded grid refutes a ∀-inequality exactly as it refutes an equation, and
            // the order is an ordinary discovered relation, so the driver is untouched.
            // (Field-note origin: metric/interval mathematics, whose bread and butter —
            // triangle inequality, subadditive enclosures, monotone operators — the
            // catalog previously could not say.)
            ShapeInfo {
                name: "subadditivity",
                schema: "le(f((x ⊕ y)), (f(x) ⊕ f(y))) = true  (f(x ⊕ y) ≤ f(x) ⊕ f(y))",
                gate: "unary endo s → s, a homogeneous binary on s, and an order relation \
                       s × s → r (r ≠ s) whose output sort carries a constant rendering \
                       as `true`",
                gate_slots: ShapeGate {
                    slots: &[
                        Slot::Unary(0, 0),
                        Slot::Binary(0),
                        Slot::Relation(0, 1),
                        Slot::Constant(1),
                    ],
                    distinct_sorts: HETERO,
                    distinct_ops: &[],
                },
                template: "{op} is subadditive over {other} (under {via}).",
                lhs: App(
                    2,
                    &[
                        App(0, &[App(1, &[X, Y])]),
                        App(1, &[App(0, &[X]), App(0, &[Y])]),
                    ],
                ),
                rhs: App(3, &[]),
                placeholders: &["f", "⊕", "le", "true"],
                polarity: Polarity::Equal,
                holes: &["op", "other", "via", "const"],
                mirrored: false,
                guard: Guard::None,
                const_rule: ConstRule::Named("true"),
                premise: None,
            },
            ShapeInfo {
                name: "triangle inequality",
                schema: "le(d(x, z), (d(x, y) ⊕ d(y, z))) = true  (d(x, z) ≤ d(x, y) ⊕ d(y, z))",
                gate: "a distance d : s × s → t, a homogeneous binary on t, and an order \
                       relation t × t → r (r ≠ t) whose output sort carries a constant \
                       rendering as `true`; d and the binary must be different operators",
                gate_slots: ShapeGate {
                    slots: &[
                        Slot::Relation(0, 1),
                        Slot::Binary(1),
                        Slot::Relation(1, 2),
                        Slot::Constant(2),
                    ],
                    // d's output MAY share its operand sort (an integer-valued distance
                    // on integers); only the order's verdict sort must be foreign.
                    distinct_sorts: &[(1, 2)],
                    distinct_ops: &[(0, 1)],
                },
                template: "{op} satisfies the triangle inequality with {other} (under {via}).",
                lhs: App(
                    2,
                    &[App(0, &[X, Z]), App(1, &[App(0, &[X, Y]), App(0, &[Y, Z])])],
                ),
                rhs: App(3, &[]),
                placeholders: &["d", "⊕", "le", "true"],
                polarity: Polarity::Equal,
                holes: &["op", "other", "via", "const"],
                mirrored: false,
                guard: Guard::None,
                const_rule: ConstRule::Named("true"),
                premise: None,
            },
            ShapeInfo {
                name: "monotonicity (join form)",
                schema: "le(f(x), f((x ⊕ y))) = true  (f(x) ≤ f(x ⊕ y) — monotone in the ⊕-order)",
                gate: "unary endo s → s, a homogeneous binary on s (read as the domain's \
                       join), and an order relation s × s → r (r ≠ s) whose output sort \
                       carries a constant rendering as `true` — the unconditional form of \
                       `∀ x ≤ y: f(x) ≤ f(y)` for join-induced orders; the guarded \
                       general form stays a roadmap candidate (conditional laws)",
                gate_slots: ShapeGate {
                    slots: &[
                        Slot::Unary(0, 0),
                        Slot::Binary(0),
                        Slot::Relation(0, 1),
                        Slot::Constant(1),
                    ],
                    distinct_sorts: HETERO,
                    distinct_ops: &[],
                },
                template: "{op} is monotone in the {other}-order (under {via}).",
                lhs: App(2, &[App(0, &[X]), App(0, &[App(1, &[X, Y])])]),
                rhs: App(3, &[]),
                placeholders: &["f", "⊕", "le", "true"],
                polarity: Polarity::Equal,
                holes: &["op", "other", "via", "const"],
                mirrored: false,
                guard: Guard::None,
                const_rule: ConstRule::Named("true"),
                premise: None,
            },
            // -- the GUARDED shapes: conditional laws, `P ⟹ lhs = rhs`. The premise is a
            // schematic term judged against the shape's constant slot; assignments where
            // it does not fire are skipped, and a law whose premise never fires is
            // VACUOUS, not true — antisymmetry of a strict order stays silent because
            // `lt(x,y) ∧ lt(y,x)` is satisfiable nowhere.
            ShapeInfo {
                name: "monotonicity (guarded)",
                schema: "le(x, y) = true ⟹ le(f(x), f(y)) = true  (∀ x ≤ y: f(x) ≤ f(y))",
                gate: "unary endo s → s plus an order relation s × s → r (r ≠ s) whose \
                       output sort carries a constant rendering as `true` — the general \
                       form of monotonicity, judged only where the premise fires",
                gate_slots: ShapeGate {
                    slots: &[Slot::Unary(0, 0), Slot::Relation(0, 1), Slot::Constant(1)],
                    distinct_sorts: HETERO,
                    distinct_ops: &[],
                },
                template: "{op} is monotone under {other}.",
                lhs: App(1, &[App(0, &[X]), App(0, &[Y])]),
                rhs: App(2, &[]),
                placeholders: &["f", "le", "true"],
                polarity: Polarity::Equal,
                holes: &["op", "other", "const"],
                mirrored: false,
                guard: Guard::None,
                const_rule: ConstRule::Named("true"),
                premise: Some(App(1, &[X, Y])),
            },
            ShapeInfo {
                name: "transitivity",
                schema: "(le(x, y) ∧ le(y, z)) = true ⟹ le(x, z) = true",
                gate: "a relation s × s → r (r ≠ s), a homogeneous binary on r (read as \
                       the verdict sort's and), and a constant of r rendering as `true` — \
                       judged only where both premise links fire",
                gate_slots: ShapeGate {
                    slots: &[Slot::Relation(0, 1), Slot::Binary(1), Slot::Constant(1)],
                    distinct_sorts: HETERO,
                    distinct_ops: &[],
                },
                template: "{op} is transitive (chained through {other}).",
                lhs: App(0, &[X, Z]),
                rhs: App(2, &[]),
                placeholders: &["le", "∧", "true"],
                polarity: Polarity::Equal,
                holes: &["op", "other", "const"],
                mirrored: false,
                guard: Guard::None,
                const_rule: ConstRule::Named("true"),
                premise: Some(App(1, &[App(0, &[X, Y]), App(0, &[Y, Z])])),
            },
            ShapeInfo {
                name: "antisymmetry",
                schema: "(le(x, y) ∧ le(y, x)) = true ⟹ x = y",
                gate: "a relation s × s → r (r ≠ s), a homogeneous binary on r (read as \
                       the verdict sort's and), and a constant of r rendering as `true`; \
                       the conclusion is CARRIER equality — and a strict order, whose \
                       premise is satisfiable nowhere, correctly earns nothing",
                gate_slots: ShapeGate {
                    slots: &[Slot::Relation(0, 1), Slot::Binary(1), Slot::Constant(1)],
                    distinct_sorts: HETERO,
                    distinct_ops: &[],
                },
                template: "{op} is antisymmetric — mutual relation forces equality.",
                lhs: X,
                rhs: Y,
                placeholders: &["le", "∧", "true"],
                polarity: Polarity::Equal,
                holes: &["op", "other", "const"],
                mirrored: false,
                guard: Guard::None,
                const_rule: ConstRule::Named("true"),
                premise: Some(App(1, &[App(0, &[X, Y]), App(0, &[Y, X])])),
            },
            ShapeInfo {
                name: "totality",
                schema: "(le(x, y) ⊕ le(y, x)) = true  (every pair relates, one way or the other)",
                gate: "a relation s × s → r (r ≠ s), a homogeneous binary on r (read as \
                       the verdict sort's or), and a constant of r rendering as `true` — \
                       a total order says yes somewhere on every pair; a strict order \
                       refuses this on the diagonal",
                gate_slots: ShapeGate {
                    slots: &[Slot::Relation(0, 1), Slot::Binary(1), Slot::Constant(1)],
                    distinct_sorts: HETERO,
                    distinct_ops: &[],
                },
                template: "{op} is total under {other} — every pair relates one way or \
                           the other.",
                lhs: App(1, &[App(0, &[X, Y]), App(0, &[Y, X])]),
                rhs: App(2, &[]),
                placeholders: &["le", "⊕", "true"],
                polarity: Polarity::Equal,
                holes: &["op", "other", "const"],
                mirrored: false,
                guard: Guard::None,
                const_rule: ConstRule::Named("true"),
                premise: None,
            },
            // -- the LAYOUT words (from scoping the second-domain adoption: a diagram
            // renderer under metamorphic probe). `inert` is the stability law an agent
            // loop leans on — an operator that is observationally a NO-OP (declaration
            // reorder must not move a layout); `equivariant map` is the commuting
            // square (rename-then-render = render-then-relabel), which the catalog
            // could not previously say: `action equivariance` distributes an action
            // over a binary, but no shape carried a unary map INTERTWINING two actions.
            ShapeInfo {
                name: "inert",
                schema: "u(x) = x",
                gate: "unary endo s → s — an operator the observation cannot see: a \
                       normalization already normal, a reorder a stable layout ignores",
                gate_slots: open(&[Slot::Unary(0, 0)]),
                template: "{op} leaves every value unchanged.",
                lhs: App(0, &[X]),
                rhs: X,
                placeholders: &["u"],
                polarity: Polarity::Equal,
                holes: &["op"],
                mirrored: false,
                guard: Guard::None,
                const_rule: ConstRule::Any,
                premise: None,
            },
            ShapeInfo {
                name: "equivariant map",
                schema: "f(act(x, p)) = act2(f(x), p)",
                gate: "a unary f : s → t plus an action of u on s and an action of u on \
                       t — the commuting square: acting before f is acting after it \
                       (rename-then-render = render-then-relabel)",
                gate_slots: ShapeGate {
                    slots: &[Slot::Unary(0, 1), Slot::Action(0, 2), Slot::Action(1, 2)],
                    distinct_sorts: &[(0, 2), (1, 2)],
                    distinct_ops: &[],
                },
                template: "{op} is equivariant — {other} before it becomes {via} after it.",
                // the param rides sort variable 2 here (0 and 1 are the two
                // carriers), so `P`'s usual Var(1, 0) does not apply.
                lhs: App(0, &[App(1, &[X, Var(2, 0)])]),
                rhs: App(2, &[App(0, &[X]), Var(2, 0)]),
                placeholders: &["f", "act", "act2"],
                polarity: Polarity::Equal,
                holes: &["op", "other", "via"],
                mirrored: false,
                guard: Guard::None,
                const_rule: ConstRule::Any,
                premise: None,
            },
        ]
    }

    /// The catalog's canonical text — deterministic, human-readable, diffable: one shape per
    /// stanza (name, schema, gate, prose template). This is what `spec/shapes.spec` locks.
    pub fn render() -> String {
        let mut out = String::from(
            "# shape catalog: the universal algebraic shapes the discovery engine instantiates \
             — generated by `cargo run --example freeze_shapes`; ratify changes.\n\
             #\n\
             # This is the engine's LAW-LANGUAGE: every law in every theory's discovered spec is\n\
             # an instance of one stanza below. Adding or changing a stanza changes what EVERY\n\
             # consumer's discovered spec can say, so it lands as a reviewed diff to this file —\n\
             # never as a silent engine edit.\n",
        );
        for shape in Self::inventory() {
            out.push_str(&format!(
                "\n- {}\n      schema:   {}\n      gate:     {}\n      template: {}\n",
                shape.name, shape.schema, shape.gate, shape.template
            ));
        }
        out
    }

    /// The catalog as a `spec_lock::Lock` on `spec/shapes.spec` — same discipline as the theory
    /// specs: `spec_lock::check` is the drift gate, `spec_lock::bless` (via
    /// `examples/freeze_shapes.rs`) the regeneration path, the committed diff the ratification.
    pub fn lock() -> spec_lock::Lock {
        spec_lock::Lock {
            name: "shape catalog".to_string(),
            path: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("spec")
                .join("shapes.spec"),
            live: Self::render(),
        }
    }
}

#[crate::mutate]
impl<T: Theory> Engine<T> {
    /// The id of the variable for `(sort, ord)`, if it exists.
    fn var(&self, sort: T::Sort, ord: usize) -> Option<Term> {
        self.vars
            .iter()
            .position(|v| v.sort == sort && v.ord == ord)
            .map(Term::Var)
    }

    fn same(&self, a: &Term, b: &Term) -> bool {
        self.signature(a) == self.signature(b)
    }

    /// Instantiate every catalog shape over the operators; keep those that run true.
    ///
    /// THE CATALOG IS THE ENGINE: this is a generic interpreter over
    /// [`ShapeCatalog::inventory()`] — the shape's `gate_slots` decide which operator
    /// bindings are tried (and bind the sort variables), its `lhs`/`rhs` terms instantiate
    /// through that binding, its `template`/`holes` render the prose, its `polarity` decides
    /// the judgment, and `mirrored`/`guard`/`const_rule` carry the three residues that used
    /// to live only in code. Adding a shape is adding a STANZA OF DATA — there is no second
    /// artifact to keep in step, and the ratified catalog (`spec/shapes.spec`) is executable.
    ///
    /// Emission order is canonical and matches what genesis's target locks predict: the
    /// equational band first, then the witness band; within a band, by FIRE operator (the
    /// slot-0 binding), then catalog order, then partner bindings in operator-index order.
    /// One law per confirmed instance, deduplicated by prose (a two-sided shape's mirrored
    /// variant collapses onto the same line).
    fn templates(&self) -> (Vec<DiscoveredLaw>, Vec<(String, String)>) {
        let mut out: Vec<DiscoveredLaw> = Vec::new();
        let mut seen_prose: Vec<String> = Vec::new();
        let mut undecided: Vec<(String, String)> = Vec::new();
        let inventory = ShapeCatalog::inventory();
        for band in [Polarity::Equal, Polarity::Differs] {
            for fire in 0..self.ops.len() {
                for shape in inventory.iter().filter(|s| s.polarity == band) {
                    self.instantiate_shape(shape, fire, &mut seen_prose, &mut out, &mut undecided);
                }
            }
        }
        // a candidate that eventually HELD (another fire order or partner) is not
        // undecided — the law wins.
        undecided.retain(|(p, _)| !out.iter().any(|l| l.prose == *p));
        (out, undecided)
    }

    /// Every confirmed instance of one shape fired on one operator: enumerate the partner
    /// bindings, admit them through the shape's data gate, concretize the canonical terms,
    /// judge by polarity, and push (deduplicated by prose).
    fn instantiate_shape(
        &self,
        shape: &ShapeInfo,
        fire: usize,
        seen_prose: &mut Vec<String>,
        out: &mut Vec<DiscoveredLaw>,
        undecided: &mut Vec<(String, String)>,
    ) {
        // the behavioural guard — the one applicability fact only the grid decides. The
        // signature check comes first: the guard PROBES the fire operator on the grid, so
        // it must not run on an operator the shape's gate would refuse anyway.
        match shape.guard {
            Guard::None => {}
            Guard::FireOpNotCommutative => {
                let s = self.ops[fire].output;
                if !is_binary_on(&self.ops[fire], s) {
                    return;
                }
                let (Some(x), Some(y)) = (self.var(s, 0), self.var(s, 1)) else {
                    return;
                };
                let commutative = self.same(
                    &Term::App(fire, vec![x.clone(), y.clone()]),
                    &Term::App(fire, vec![y, x]),
                );
                if commutative {
                    return;
                }
            }
        }

        // every operator tuple with slot 0 = the fire operator, remaining slots in
        // operator-index order (lexicographic — the canonical partner order).
        let n = self.ops.len();
        let rest = shape.gate_slots.slots.len() - 1;
        let combos = n.pow(rest as u32);
        for code in 0..combos {
            let mut tuple = vec![fire];
            for digit in (0..rest).rev() {
                tuple.push((code / n.pow(digit as u32)) % n);
            }

            // the data gate admits (and binds the sort variables) or the tuple is skipped.
            let sigs: Vec<(Vec<T::Sort>, T::Sort)> = tuple
                .iter()
                .map(|&i| (self.ops[i].inputs.clone(), self.ops[i].output))
                .collect();
            let names: Vec<&str> = tuple.iter().map(|&i| self.ops[i].symbol).collect();
            let Ok(bound) = shape.gate_slots.bind(&sigs, &names) else {
                continue;
            };

            // the constant-symbol residue (irreflexivity vs self-application).
            let constant_ok = tuple
                .iter()
                .zip(shape.gate_slots.slots)
                .filter(|(_, slot)| matches!(slot, Slot::Constant(_)))
                .all(|(&i, _)| match shape.const_rule {
                    ConstRule::Any => true,
                    ConstRule::Named(s) => self.ops[i].symbol == s,
                    ConstRule::NotNamed(s) => self.ops[i].symbol != s,
                });
            if !constant_ok {
                continue;
            }

            // prose: the template over the binding — constants by symbol, operators by name.
            let subs: Vec<(&str, &str)> = shape
                .holes
                .iter()
                .zip(&tuple)
                .zip(shape.gate_slots.slots)
                .map(|((hole, &i), slot)| {
                    let name = if matches!(slot, Slot::Constant(_)) {
                        self.ops[i].symbol
                    } else {
                        self.ops[i].name
                    };
                    (*hole, name)
                })
                .collect();
            let prose = shape.instantiate(&subs);
            if seen_prose.contains(&prose) {
                continue;
            }

            // concretize the canonical terms; a shape whose variables the theory cannot
            // mint (an uninhabited sort) simply never fires.
            let (Some(lhs), Some(rhs)) = (
                self.concretize(&shape.lhs, &tuple, &bound),
                self.concretize(&shape.rhs, &tuple, &bound),
            ) else {
                continue;
            };

            // judge — trying the mirrored (argument-swapped) variant under the same prose
            // where the shape is two-sided.
            let mut variants = vec![lhs];
            if shape.mirrored {
                if let Some(Term::App(op, args)) = variants.first().cloned() {
                    if args.len() == 2 {
                        variants.push(Term::App(op, vec![args[1].clone(), args[0].clone()]));
                    }
                }
            }
            // the GUARD, where the shape is conditional: the premise term and its truth
            // reference (the shape's constant slot), concretized through the same binding.
            let guard = match &shape.premise {
                None => None,
                Some(premise) => {
                    let truth_slot = shape
                        .gate_slots
                        .slots
                        .iter()
                        .position(|s| matches!(s, Slot::Constant(_)))
                        .expect("a conditional shape carries a constant slot");
                    let (Some(p), Some(t)) = (
                        self.concretize(premise, &tuple, &bound),
                        Some(Term::App(tuple[truth_slot], vec![])),
                    ) else {
                        continue;
                    };
                    Some((p, t))
                }
            };

            let prose_undecided = prose.clone();
            let mut undecided_here: Option<String> = None;
            for lhs in variants {
                let verdict = match (&guard, shape.polarity) {
                    // meaningfulness guards BOTH polarities: two all-undefined sides agree
                    // on nothing yet compare equal, which "fixed point" (the first shape
                    // with no variable in either term) can reach where every older shape
                    // kept a defined variable in play.
                    (None, Polarity::Equal) => {
                        let (a, b) = (self.signature(&lhs), self.signature(&rhs));
                        if Self::meaningful(&a) {
                            Self::judge_sigs(&a, &b)
                        } else {
                            Verdict::Refuted
                        }
                    }
                    // a WITNESS needs a refutation-strength difference: a difference
                    // that is merely undecided at the declared tolerance witnesses
                    // nothing (and reports as undecided instead of firing).
                    (None, Polarity::Differs) => {
                        let (a, b) = (self.signature(&lhs), self.signature(&rhs));
                        if !(Self::meaningful(&a) && Self::meaningful(&b)) {
                            Verdict::Refuted
                        } else {
                            match Self::judge_sigs(&a, &b) {
                                Verdict::Refuted => Verdict::Holds,
                                Verdict::Holds => Verdict::Refuted,
                                Verdict::Undecided => Verdict::Undecided,
                            }
                        }
                    }
                    // a GUARDED equation: judged only where the premise evaluates equal
                    // to the truth reference, and the premise must be SATISFIABLE with a
                    // defined lhs somewhere — an unsatisfiable guard manufactures no law
                    // (antisymmetry of a strict order stays silent, correctly). A premise
                    // that is itself UNDECIDED anywhere taints the law to undecided.
                    (Some((premise, truth)), Polarity::Equal) => {
                        let (p, t) = (self.signature(premise), self.signature(truth));
                        let (a, b) = (self.signature(&lhs), self.signature(&rhs));
                        let mut grounded = false;
                        let mut verdict = Verdict::Holds;
                        for k in 0..p.len() {
                            let satisfied = match (&p[k], &t[k]) {
                                (Some(pv), Some(tv)) => match T::judge(pv, tv) {
                                    Verdict::Holds => true,
                                    Verdict::Refuted => false,
                                    Verdict::Undecided => {
                                        verdict = Verdict::Undecided;
                                        false
                                    }
                                },
                                _ => false,
                            };
                            if !satisfied {
                                continue;
                            }
                            grounded = grounded || a[k].is_some();
                            match Self::judge_at(&a[k], &b[k]) {
                                Verdict::Holds => {}
                                Verdict::Undecided => verdict = Verdict::Undecided,
                                Verdict::Refuted => {
                                    verdict = Verdict::Refuted;
                                    break;
                                }
                            }
                        }
                        if !grounded && verdict != Verdict::Refuted {
                            Verdict::Refuted
                        } else {
                            verdict
                        }
                    }
                    // no witness shape carries a premise (the catalog has none; the
                    // combination is unratified until a stanza needs it).
                    (Some(_), Polarity::Differs) => Verdict::Refuted,
                };
                if verdict != Verdict::Holds {
                    if verdict == Verdict::Undecided {
                        let connective = match shape.polarity {
                            Polarity::Equal => "=",
                            Polarity::Differs => "≠",
                        };
                        undecided_here = Some(format!(
                            "{} {connective} {}",
                            self.render(&lhs),
                            self.render(&rhs)
                        ));
                    }
                    continue;
                }
                undecided_here = None;
                seen_prose.push(prose.clone());
                let connective = match shape.polarity {
                    Polarity::Equal => "=",
                    Polarity::Differs => "≠",
                };
                let body = format!("{} {connective} {}", self.render(&lhs), self.render(&rhs));
                let equation = match &guard {
                    None => body,
                    Some((premise, truth)) => {
                        format!("{} = {} ⟹ {body}", self.render(premise), self.render(truth))
                    }
                };
                out.push(DiscoveredLaw {
                    shape: shape.name,
                    prose,
                    equation,
                    lhs,
                    rhs,
                    polarity: shape.polarity,
                    premise: guard.clone(),
                });
                break;
            }
            if let Some(equation) = undecided_here {
                if !undecided.iter().any(|(p, _)| *p == prose_undecided) {
                    undecided.push((prose_undecided.clone(), equation));
                }
            }
        }
    }

    /// A schematic term over a concrete binding: slots become the tuple's operators, sort
    /// variables' variables become the theory's (`None` when the theory minted no such
    /// variable — the instance simply cannot fire).
    fn concretize(
        &self,
        term: &SchemaTerm,
        tuple: &[usize],
        bound: &[Option<T::Sort>],
    ) -> Option<Term> {
        match term {
            SchemaTerm::Var(sv, ord) => {
                let sort = (*bound.get(*sv as usize)?)?;
                self.var(sort, *ord as usize)
            }
            SchemaTerm::App(slot, args) => {
                let mut concrete = Vec::with_capacity(args.len());
                for arg in *args {
                    concrete.push(self.concretize(arg, tuple, bound)?);
                }
                Some(Term::App(tuple[*slot as usize], concrete))
            }
        }
    }

    /// Run discovery: the named template laws, the count of further (consequence) equalities, and
    /// the operators that appear in no named law.
    pub fn discover(&self) -> Discovered {
        let (laws, undecided) = self.templates();

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
            undecided,
        }
    }

    /// Check that the GIVEN laws hold when evaluated over the grid against the current operators,
    /// returning the first failure — each per its polarity: an equation must hold on EVERY
    /// assignment, a witness law must find SOME assignment where the terms differ. Unlike
    /// re-deriving (which is tautological), feeding FROZEN laws here is the mutation-judged
    /// probe: a mutant that breaks a frozen law is caught.
    pub fn check(&self, laws: &[DiscoveredLaw]) -> Result<(), String> {
        for l in laws {
            let mut witnessed = false;
            let mut grounded = l.premise.is_none();
            for asn in &self.grid {
                // a guarded law is judged only where its premise fires — and it must
                // fire with a defined lhs SOMEWHERE, or the law has lost its ground.
                if let Some((premise, truth)) = &l.premise {
                    let p = self.eval(premise, asn).map(|v| T::observe(&v));
                    let t = self.eval(truth, asn).map(|v| T::observe(&v));
                    if p.is_none() || Self::judge_at(&p, &t) != Verdict::Holds {
                        continue;
                    }
                }
                let lhs = self.eval(&l.lhs, asn).map(|v| T::observe(&v));
                let rhs = self.eval(&l.rhs, asn).map(|v| T::observe(&v));
                grounded = grounded || lhs.is_some();
                match (l.polarity, Self::judge_at(&lhs, &rhs)) {
                    (Polarity::Equal, Verdict::Refuted) => {
                        return Err(format!("discovered law failed: {}", l.equation));
                    }
                    // a frozen law drifting into the band is DRIFT, named as such —
                    // never a silent pass.
                    (Polarity::Equal, Verdict::Undecided) => {
                        return Err(format!(
                            "law became undecided at the declared tolerance: {}",
                            l.equation
                        ));
                    }
                    (Polarity::Equal, Verdict::Holds) => {}
                    // a witness needs a refutation-STRENGTH difference; an undecided
                    // difference witnesses nothing.
                    (Polarity::Differs, Verdict::Refuted) => {
                        witnessed = true;
                        break;
                    }
                    (Polarity::Differs, _) => {}
                }
            }
            if l.polarity == Polarity::Differs && !witnessed {
                return Err(format!("witness law lost its witness: {}", l.equation));
            }
            if !grounded {
                return Err(format!("guarded law lost its ground: {}", l.equation));
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

    /// The operator evaluators, in operator-index order — the surface the algebra-mutation
    /// harness (`discover::mutation`) perturbs.
    pub(crate) fn evals(&self) -> Vec<EvalFn<T>> {
        self.ops.iter().map(|o| o.eval).collect()
    }

    /// This engine with its evaluators REPLACED — same signature table, same variables, same
    /// grid: an operator-table mutant, ready to re-run discovery. The mutation harness's one
    /// hook into the engine.
    pub(crate) fn with_evals(&self, evals: &[EvalFn<T>]) -> Engine<T> {
        Engine {
            ops: self
                .ops
                .iter()
                .zip(evals)
                .map(|(o, &eval)| Operator {
                    name: o.name,
                    symbol: o.symbol,
                    fixity: o.fixity,
                    inputs: o.inputs.clone(),
                    output: o.output,
                    eval,
                })
                .collect(),
            vars: self.vars.clone(),
            grid: self.grid.clone(),
        }
    }
}

#[crate::mutate]
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
/// PERTURBATIONS, bounded by `cap`. Deterministic — the derived perturbation order is fixed. This
/// is `#[derive(Shaped)]` (already minting the probe surface for edges) reused to fatten the
/// discovery grid.
///
/// The closure is PARTITIONED, because the perturbation surface has two differently-priced
/// halves. STRUCTURE (which constructor shapes exist — variant swaps, here or threaded up from
/// any field) is type-level-finite for a non-recursive type, so it is closed over FIRST and
/// exhaustively: the cap can starve value density, never constructor coverage. VALUES (leaf
/// quantities tuned toward a validity rule's edges) are open-ended, so they get whatever budget
/// remains. For a RECURSIVE type the structural space is itself unbounded and the honest frame
/// returns — phase 1 is then exhaustive only up to the cap (depth-bounded structure), the same
/// bound term enumeration already lives with. `grid_gaps` is the audit that a grid's structural
/// closure actually completed.
#[crate::mutate]
pub fn shadow_grid<V: crate::boundary::Shaped>(cap: usize) -> Vec<V> {
    let mut grid: Vec<V> = ::std::vec![V::inhabitant()];
    // PHASE 1 — structure, exhaustively: close under structural perturbations alone, so every
    // reachable constructor shape is in the grid before any budget is spent on value tuning.
    close(&mut grid, cap, V::structural_perturbations);
    // PHASE 2 — values, with the remainder: continue the closure under the FULL surface (value
    // neighbours of every phase-1 member included; re-walking members is cheap dedup).
    close(&mut grid, cap, V::all_perturbations);
    grid
}

/// One closure pass: walk the frontier, admitting unseen `neighbours` until `cap`. The `cap`
/// bound lives in the inner break (the only place new values are added); the outer loop just
/// walks the frontier. `i` indexes into `grid`, so `i <= grid.len()` would read past the end.
#[crate::mutate]
fn close<V: PartialEq>(grid: &mut Vec<V>, cap: usize, neighbours: impl Fn(&V) -> Vec<V>) {
    let mut i = 0;
    while i < grid.len() {
        for n in neighbours(&grid[i]) {
            if grid.len() >= cap {
                break;
            }
            if !grid.contains(&n) {
                grid.push(n);
            }
        }
        i += 1;
    }
}

/// The grid's structural AUDIT: every constructor reachable in ONE perturbation step from the
/// grid that the grid itself fails to exhibit, by discriminant. Empty iff the closure completed
/// (the cap was generous enough) — which `shadow_grid`'s structure-first partition guarantees
/// whenever the structural space fits the cap at all. Promoted from this repo's test harness to
/// the library so every downstream `#[derive(Shaped)]` grid can be held to it as an invariant
/// (`assert!(grid_gaps(&grid).is_empty())`) instead of by convention. Discriminants, never a
/// hand-written variant list — a hand list would just move the gap.
#[crate::mutate]
pub fn grid_gaps<V: crate::boundary::Shaped>(grid: &[V]) -> Vec<core::mem::Discriminant<V>> {
    let exhibited: Vec<_> = grid.iter().map(core::mem::discriminant).collect();
    let mut gaps = Vec::new();
    for v in grid {
        for n in v.all_perturbations() {
            let d = core::mem::discriminant(&n);
            if !exhibited.contains(&d) && !gaps.contains(&d) {
                gaps.push(d);
            }
        }
    }
    gaps
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
            ("And with T leaves a value unchanged.", "(T & x) = x"),
            ("And by F always gives F.", "(F & x) = F"),
            (
                "Not inverts And — a value And its own Not gives F.",
                "(x & ~(x)) = F",
            ),
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
                "Not inverts Or — a value Or its own Not gives T.",
                "(x | ~(x)) = T",
            ),
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
            shape: "commutativity",
            prose: "bogus".into(),
            equation: "(x & y) = (x | y)".into(),
            lhs: Term::App(2, vec![Term::Var(0), Term::Var(1)]),
            rhs: Term::App(3, vec![Term::Var(0), Term::Var(1)]),
            polarity: Polarity::Equal,
            premise: None,
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

    // -- the PARTITIONED closure: structure exhaustively first, values with the remainder --------
    //
    // `Fat` is a value-rich leaf (four value neighbours per step, no structural degree of
    // freedom); `Choice` is a two-variant enum; `Gauge` composes them. Under the OLD single-pass
    // closure a tight cap spends the whole budget on `Fat`'s value chain and `Choice::Off` never
    // appears — the over-fit hazard the partition exists to kill.

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    struct Fat(u8);
    impl crate::boundary::Shaped for Fat {
        fn inhabitant() -> Self {
            Fat(0)
        }
        fn perturbation_classes(&self) -> Vec<Vec<Self>> {
            vec![(1..=4).map(|step| Fat(self.0.wrapping_add(step))).collect()]
        }
    }

    #[derive(Clone, PartialEq, Eq, Hash, Debug, crate::Shaped)]
    enum Choice {
        On,
        Off,
    }

    #[derive(Clone, PartialEq, Eq, Hash, Debug, crate::Shaped)]
    struct Gauge {
        fat: Fat,
        choice: Choice,
    }

    /// STRUCTURE CANNOT BE STARVED: with a cap of 3, the grid admits the `choice` variant swap
    /// (phase 1, structural, threaded up through the struct field) BEFORE any of `Fat`'s value
    /// neighbours — pinned exactly, so removing either phase, swapping their order, or breaking
    /// the derive's field threading all fail here. The old single-pass closure yields
    /// `[{0,On}, {1,On}, {2,On}]` under the same cap: `Off` never appears.
    #[test]
    fn shadow_grid_closes_structure_before_spending_on_values() {
        let gauge = |fat: u8, choice: Choice| Gauge {
            fat: Fat(fat),
            choice,
        };
        assert_eq!(
            shadow_grid::<Gauge>(3),
            vec![
                gauge(0, Choice::On),
                gauge(0, Choice::Off),
                gauge(1, Choice::On)
            ],
            "phase 1 admits the variant swap; phase 2 spends the remaining budget on values"
        );
        // with a generous cap the value half fills in behind the (already complete) structure.
        let full = shadow_grid::<Gauge>(12);
        assert!(full
            .iter()
            .any(|g| g.fat == Fat(2) && g.choice == Choice::Off));
        assert!(
            grid_gaps(&full).is_empty(),
            "the closure is constructor-complete"
        );
    }

    /// The structural surface itself, pinned per shape: an enum's own variant swap, the swap
    /// THREADED through an enum's field and a struct's field, the leaf default (none), bool's
    /// negation (a bool IS a two-variant sum), and `Box` transparency.
    #[test]
    fn structural_perturbations_are_the_variant_swaps_threaded_up() {
        use crate::boundary::Shaped;

        #[derive(Clone, PartialEq, Eq, Hash, Debug, crate::Shaped)]
        enum Wrap {
            Carry(Choice),
            Empty,
        }

        assert_eq!(Choice::On.structural_perturbations(), vec![Choice::Off]);
        assert_eq!(
            Wrap::Carry(Choice::On).structural_perturbations(),
            vec![Wrap::Empty, Wrap::Carry(Choice::Off)],
            "the swap at this level, then the swap below threaded up"
        );
        assert_eq!(
            Gauge {
                fat: Fat(7),
                choice: Choice::On
            }
            .structural_perturbations(),
            vec![Gauge {
                fat: Fat(7),
                choice: Choice::Off
            }],
            "a struct threads its fields' swaps; the value-only field contributes none"
        );
        assert_eq!(
            Fat(0).structural_perturbations(),
            vec![],
            "a leaf defaults to none"
        );
        assert_eq!(false.structural_perturbations(), vec![true]);
        assert_eq!(true.structural_perturbations(), vec![false]);
        let boxed: Box<Choice> = Box::new(Choice::On);
        assert_eq!(
            boxed.structural_perturbations(),
            vec![Box::new(Choice::Off)],
            "a box is transparent to the structural surface too"
        );
    }

    /// `grid_gaps` (promoted from the harness to the library) has kill power of its own: a grid
    /// missing a one-step-reachable constructor is reported by discriminant, and a completed
    /// closure is gap-free.
    #[test]
    fn grid_gaps_reports_the_missing_constructor() {
        let gaps = grid_gaps(&[Choice::On]);
        assert_eq!(gaps, vec![core::mem::discriminant(&Choice::Off)]);
        assert!(grid_gaps(&shadow_grid::<Choice>(8)).is_empty());
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

    /// `check` judges each polarity in BOTH directions — the negative cases the committed
    /// theories never supply (their witness laws all hold, and all have points of equality,
    /// so a comparison flipped inside the witness arm was invisible to every existing gate:
    /// a mutant caught by the changed-lines dogfood sweep). Two constants make the sharpest
    /// probes: `T ≠ F` differs on EVERY assignment (a mutant demanding equality finds none),
    /// and `T ≠ T` differs on NONE (a lost witness must be an error, never a vacuous pass).
    #[test]
    fn check_judges_witness_laws_in_both_directions() {
        let e = Engine::<Bits>::new();
        let op = |sym: &str| {
            e.signatures()
                .iter()
                .position(|(s, _, _)| *s == sym)
                .expect("a Bits operator")
        };
        let witness = |lhs: Term, rhs: Term, equation: &str| DiscoveredLaw {
            shape: "non-constancy",
            prose: format!("probe: {equation}"),
            equation: equation.to_string(),
            lhs,
            rhs,
            polarity: Polarity::Differs,
            premise: None,
        };

        // a witness that holds everywhere: the two constants never agree.
        let held = witness(
            Term::App(op("T"), vec![]),
            Term::App(op("F"), vec![]),
            "T ≠ F",
        );
        assert_eq!(e.check(&[held]), Ok(()));

        // a witness with no witness: identical terms must FAIL the check, by name.
        let lost = witness(
            Term::App(op("T"), vec![]),
            Term::App(op("T"), vec![]),
            "T ≠ T",
        );
        assert_eq!(
            e.check(&[lost]),
            Err("witness law lost its witness: T ≠ T".to_string())
        );
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
            shape: "commutativity",
            prose: "Tilt gives the same result in either order.".into(),
            equation: "(x >< y) = (y >< x)".into(),
            lhs: Term::App(0, vec![Term::Var(0), Term::Var(1)]),
            rhs: Term::App(0, vec![Term::Var(1), Term::Var(0)]),
            polarity: Polarity::Equal,
            premise: None,
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

    // -- the shape-catalog census: the battery and the catalog hold each other -------------------
    //
    // Two SYNTHETIC MAXIMAL theories, built so that every catalog shape fires at least once
    // across them. They are not domains anyone ships — they are the catalog's exercise yard:
    // a boolean algebra with a relation and a partial codec (`MaxLogic`), and a non-commutative
    // selection algebra with an action and a distance relation (`MaxSelect`). A catalog entry no
    // maximal theory can exhibit is stale; a template that fires with no catalog entry is
    // unratified — the census tests below assert both directions.

    // MaxLogic: booleans (commutativity, associativity, idempotence, identity, annihilation,
    // distributivity, absorption, involution, De Morgan homomorphism), a `<` relation into the
    // boolean sort (irreflexivity), a mod-3 codec (round-trips), a PARTIAL `half` (defined
    // only on even values — it earns no law and must not fake one), and the ordered-relation
    // ingredients on N: `cap` (min with 1), `plus`, `dist` (abs diff), and `le` — the
    // subadditivity, triangle, and monotonicity stanzas fire here, judged against `true`.
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
    enum LSort {
        B,
        N,
    }
    struct MaxLogic;
    #[derive(Clone)]
    enum LVal {
        B(bool),
        N(u8),
    }
    fn l_bool(v: &LVal) -> bool {
        match v {
            LVal::B(b) => *b,
            LVal::N(_) => unreachable!("sort-checked by the engine"),
        }
    }
    fn l_nat(v: &LVal) -> u8 {
        match v {
            LVal::N(n) => *n,
            LVal::B(_) => unreachable!("sort-checked by the engine"),
        }
    }
    fn l_false(_: &[LVal]) -> Option<LVal> {
        Some(LVal::B(false))
    }
    fn l_true(_: &[LVal]) -> Option<LVal> {
        Some(LVal::B(true))
    }
    fn l_and(v: &[LVal]) -> Option<LVal> {
        Some(LVal::B(l_bool(&v[0]) && l_bool(&v[1])))
    }
    fn l_or(v: &[LVal]) -> Option<LVal> {
        Some(LVal::B(l_bool(&v[0]) || l_bool(&v[1])))
    }
    fn l_not(v: &[LVal]) -> Option<LVal> {
        Some(LVal::B(!l_bool(&v[0])))
    }
    fn l_xor(v: &[LVal]) -> Option<LVal> {
        Some(LVal::B(l_bool(&v[0]) ^ l_bool(&v[1])))
    }
    fn l_lt(v: &[LVal]) -> Option<LVal> {
        Some(LVal::B(l_nat(&v[0]) < l_nat(&v[1])))
    }
    fn l_enc(v: &[LVal]) -> Option<LVal> {
        Some(LVal::N((l_nat(&v[0]) + 1) % 3))
    }
    fn l_dec(v: &[LVal]) -> Option<LVal> {
        Some(LVal::N((l_nat(&v[0]) + 2) % 3))
    }
    fn l_half(v: &[LVal]) -> Option<LVal> {
        let n = l_nat(&v[0]);
        n.is_multiple_of(2).then_some(LVal::N(n / 2))
    }
    fn l_cap(v: &[LVal]) -> Option<LVal> {
        Some(LVal::N(l_nat(&v[0]).min(1)))
    }
    fn l_plus(v: &[LVal]) -> Option<LVal> {
        Some(LVal::N(l_nat(&v[0]) + l_nat(&v[1])))
    }
    fn l_dist(v: &[LVal]) -> Option<LVal> {
        Some(LVal::N(l_nat(&v[0]).abs_diff(l_nat(&v[1]))))
    }
    fn l_le(v: &[LVal]) -> Option<LVal> {
        Some(LVal::B(l_nat(&v[0]) <= l_nat(&v[1])))
    }

    impl Theory for MaxLogic {
        type Sort = LSort;
        type Value = LVal;
        type Obs = u8;
        fn name() -> &'static str {
            "maximal logic"
        }
        fn operators() -> Vec<Operator<Self>> {
            use Fixity::{Infix, Nullary, Prefix};
            use LSort::{B, N};
            vec![
                Operator {
                    name: "false",
                    symbol: "false",
                    fixity: Nullary,
                    inputs: vec![],
                    output: B,
                    eval: l_false,
                },
                Operator {
                    name: "true",
                    symbol: "true",
                    fixity: Nullary,
                    inputs: vec![],
                    output: B,
                    eval: l_true,
                },
                Operator {
                    name: "And",
                    symbol: "&",
                    fixity: Infix,
                    inputs: vec![B, B],
                    output: B,
                    eval: l_and,
                },
                Operator {
                    name: "Or",
                    symbol: "|",
                    fixity: Infix,
                    inputs: vec![B, B],
                    output: B,
                    eval: l_or,
                },
                Operator {
                    name: "Not",
                    symbol: "~",
                    fixity: Prefix,
                    inputs: vec![B],
                    output: B,
                    eval: l_not,
                },
                Operator {
                    name: "Xor",
                    symbol: "^",
                    fixity: Infix,
                    inputs: vec![B, B],
                    output: B,
                    eval: l_xor,
                },
                Operator {
                    name: "less-than",
                    symbol: "<",
                    fixity: Infix,
                    inputs: vec![N, N],
                    output: B,
                    eval: l_lt,
                },
                Operator {
                    name: "encode",
                    symbol: "enc",
                    fixity: Prefix,
                    inputs: vec![N],
                    output: N,
                    eval: l_enc,
                },
                Operator {
                    name: "decode",
                    symbol: "dec",
                    fixity: Prefix,
                    inputs: vec![N],
                    output: N,
                    eval: l_dec,
                },
                Operator {
                    name: "halve",
                    symbol: "half",
                    fixity: Prefix,
                    inputs: vec![N],
                    output: N,
                    eval: l_half,
                },
                Operator {
                    name: "cap",
                    symbol: "cap",
                    fixity: Prefix,
                    inputs: vec![N],
                    output: N,
                    eval: l_cap,
                },
                Operator {
                    name: "plus",
                    symbol: "plus",
                    fixity: Infix,
                    inputs: vec![N, N],
                    output: N,
                    eval: l_plus,
                },
                Operator {
                    name: "dist",
                    symbol: "dist",
                    fixity: Prefix,
                    inputs: vec![N, N],
                    output: N,
                    eval: l_dist,
                },
                Operator {
                    name: "le",
                    symbol: "le",
                    fixity: Prefix,
                    inputs: vec![N, N],
                    output: B,
                    eval: l_le,
                },
            ]
        }
        fn inhabitants(sort: Self::Sort) -> Vec<Self::Value> {
            match sort {
                LSort::B => vec![LVal::B(false), LVal::B(true)],
                LSort::N => (0..3).map(LVal::N).collect(),
            }
        }
        fn sort_of(v: &Self::Value) -> Self::Sort {
            match v {
                LVal::B(_) => LSort::B,
                LVal::N(_) => LSort::N,
            }
        }
        fn observe(v: &Self::Value) -> Self::Obs {
            match v {
                LVal::B(b) => *b as u8,
                LVal::N(n) => 10 + *n,
            }
        }
        fn grid_size() -> usize {
            216 // = 2³·3³, the whole space: the maximal laws are judged exhaustively.
        }
    }

    // MaxSelect: two projections (`First` left-regular, `Last` right-regular — the bias laws), a
    // duration monoid (`Plus`/`zero`), a mod-3 `Shift` action of durations on values (action
    // identity, monoid action), a `Gap` relation whose self-application is `zero`, and a
    // doubling endo `Twice` on durations (fixed point at `zero`; the additive homomorphism —
    // the very instance the license derivation reads as LINEARITY), and a `Clamp` action
    // (min with the duration) — idempotent where `Shift` is not, so both action stanzas fire.
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
    enum SSort {
        V,
        D,
    }
    struct MaxSelect;
    #[derive(Clone)]
    enum SVal {
        V(u8),
        D(u8),
    }
    fn s_v(v: &SVal) -> u8 {
        match v {
            SVal::V(n) => *n,
            SVal::D(_) => unreachable!("sort-checked by the engine"),
        }
    }
    fn s_d(v: &SVal) -> u8 {
        match v {
            SVal::D(n) => *n,
            SVal::V(_) => unreachable!("sort-checked by the engine"),
        }
    }
    fn s_first(v: &[SVal]) -> Option<SVal> {
        Some(SVal::V(s_v(&v[0])))
    }
    fn s_last(v: &[SVal]) -> Option<SVal> {
        Some(SVal::V(s_v(&v[1])))
    }
    fn s_zero(_: &[SVal]) -> Option<SVal> {
        Some(SVal::D(0))
    }
    fn s_plus(v: &[SVal]) -> Option<SVal> {
        Some(SVal::D(s_d(&v[0]) + s_d(&v[1])))
    }
    fn s_shift(v: &[SVal]) -> Option<SVal> {
        Some(SVal::V((s_v(&v[0]) + s_d(&v[1])) % 3))
    }
    fn s_gap(v: &[SVal]) -> Option<SVal> {
        Some(SVal::D(s_v(&v[0]).abs_diff(s_v(&v[1]))))
    }
    fn s_twice(v: &[SVal]) -> Option<SVal> {
        Some(SVal::D(s_d(&v[0]) * 2))
    }
    fn s_clamp(v: &[SVal]) -> Option<SVal> {
        Some(SVal::V(s_v(&v[0]).min(s_d(&v[1]))))
    }
    fn s_floor(_: &[SVal]) -> Option<SVal> {
        Some(SVal::V(0))
    }
    fn s_same(v: &[SVal]) -> Option<SVal> {
        Some(SVal::V(s_v(&v[0]))) // the observational no-op: `inert` must fire, and the
                                  // identity intertwines every action with itself, so
                                  // `equivariant map` fires through it too.
    }

    impl Theory for MaxSelect {
        type Sort = SSort;
        type Value = SVal;
        type Obs = u8;
        fn name() -> &'static str {
            "maximal selection"
        }
        fn operators() -> Vec<Operator<Self>> {
            use Fixity::{Infix, Nullary, Prefix};
            use SSort::{D, V};
            vec![
                Operator {
                    name: "First",
                    symbol: "fst",
                    fixity: Prefix,
                    inputs: vec![V, V],
                    output: V,
                    eval: s_first,
                },
                Operator {
                    name: "Last",
                    symbol: "lst",
                    fixity: Prefix,
                    inputs: vec![V, V],
                    output: V,
                    eval: s_last,
                },
                Operator {
                    name: "zero",
                    symbol: "zero",
                    fixity: Nullary,
                    inputs: vec![],
                    output: D,
                    eval: s_zero,
                },
                Operator {
                    name: "Plus",
                    symbol: "+",
                    fixity: Infix,
                    inputs: vec![D, D],
                    output: D,
                    eval: s_plus,
                },
                Operator {
                    name: "Shift",
                    symbol: "shift",
                    fixity: Prefix,
                    inputs: vec![V, D],
                    output: V,
                    eval: s_shift,
                },
                Operator {
                    name: "Gap",
                    symbol: "gap",
                    fixity: Prefix,
                    inputs: vec![V, V],
                    output: D,
                    eval: s_gap,
                },
                Operator {
                    name: "Twice",
                    symbol: "dbl",
                    fixity: Prefix,
                    inputs: vec![D],
                    output: D,
                    eval: s_twice,
                },
                Operator {
                    name: "Clamp",
                    symbol: "clamp",
                    fixity: Prefix,
                    inputs: vec![V, D],
                    output: V,
                    eval: s_clamp,
                },
                Operator {
                    name: "floor",
                    symbol: "floor",
                    fixity: Nullary,
                    inputs: vec![],
                    output: V,
                    eval: s_floor,
                },
                Operator {
                    name: "Same",
                    symbol: "same",
                    fixity: Prefix,
                    inputs: vec![V],
                    output: V,
                    eval: s_same,
                },
            ]
        }
        fn inhabitants(sort: Self::Sort) -> Vec<Self::Value> {
            match sort {
                SSort::V => (0..3).map(SVal::V).collect(),
                SSort::D => (0..3).map(SVal::D).collect(),
            }
        }
        fn sort_of(v: &Self::Value) -> Self::Sort {
            match v {
                SVal::V(_) => SSort::V,
                SVal::D(_) => SSort::D,
            }
        }
        fn observe(v: &Self::Value) -> Self::Obs {
            match v {
                SVal::V(n) => *n,
                SVal::D(n) => 100 + *n,
            }
        }
        fn grid_size() -> usize {
            729 // = 3⁶, the whole space: bias and action are judged exhaustively.
        }
    }

    /// The discovered laws of a theory, engine-level (no per-domain addenda — the census is
    /// about what the ENGINE's battery can say).
    fn discovered_laws<T: Theory>() -> Vec<DiscoveredLaw> {
        Engine::<T>::new().discover().laws
    }

    /// The discovered prose of a theory — the census matcher's view of `discovered_laws`.
    fn discovered_prose<T: Theory>() -> Vec<String> {
        discovered_laws::<T>()
            .into_iter()
            .map(|l| l.prose)
            .collect()
    }

    /// CENSUS, direction one: every catalog shape FIRES at least once across the maximal
    /// theories. A shape that never fires is stale — either its `templates()` twin was removed
    /// or reworded without the catalog following, or the maximal theories lost the ingredient
    /// that exhibits it.
    #[test]
    fn every_catalog_shape_fires_across_the_maximal_theories() {
        let mut proses = discovered_prose::<MaxLogic>();
        proses.extend(discovered_prose::<MaxSelect>());
        // the ordered-action pair (inflation/deflation) is exhibited by the fabric
        // registry theory (grant/revoke under within): the maximal fixtures carry no
        // order relation, and growing their tuned exhaustive grids to add one would
        // perturb every other shape they exist to pin — so the census reads the
        // committed exhibitor for these two.
        proses.extend(discovered_prose::<crate::discover::fabric::Fabric>());
        for shape in ShapeCatalog::inventory() {
            assert!(
                proses.iter().any(|p| shape.matches(p)),
                "catalog shape {:?} (template {:?}) fired on NO law of the maximal theories — \
                 a stale catalog entry: align it with `templates()` (they must move together), \
                 re-bless spec/shapes.spec, and ratify the diff. Discovered: {proses:#?}",
                shape.name,
                shape.template
            );
        }
    }

    /// CENSUS, direction two: every law discovered across EVERY registered theory in the crate
    /// (plus the maximal ones) IS a ratified shape — checked at two strengths. The law's
    /// `shape` TAG (set at its `templates()` push site) must name a catalog entry, and that
    /// SAME entry's prose template must match the law's prose. The tag check is stronger than
    /// prose matching alone: a push site tagged with a misspelled or unratified name fails
    /// here even if its prose happens to fit some template, and a tag/prose mismatch (the tag
    /// says one shape, the sentence reads as another) is caught as the lockstep violation it is.
    #[test]
    fn every_discovered_law_in_the_crate_is_a_ratified_shape() {
        use crate::discover::architect::{EscapeCodec, Reports};
        use crate::discover::arithmetic::Arithmetic;
        use crate::discover::coherence::{FirstMerge, GcdMerge, MaxMerge};
        use crate::discover::date::Calendar;
        use crate::discover::derived::{lattice::Lattice, tiers::Tiers};
        use crate::discover::router::Router;
        use crate::kvstore::theory::TtlStore;

        let all: Vec<(&str, Vec<DiscoveredLaw>)> = vec![
            ("interpreter arithmetic", discovered_laws::<Arithmetic>()),
            ("router", discovered_laws::<Router>()),
            ("date calculus", discovered_laws::<Calendar>()),
            ("ttl store", discovered_laws::<TtlStore>()),
            ("tri lattice", discovered_laws::<Lattice>()),
            ("graded lattices", discovered_laws::<Tiers>()),
            ("architect report", discovered_laws::<Reports>()),
            ("lsp escape codec", discovered_laws::<EscapeCodec>()),
            ("max-merge", discovered_laws::<MaxMerge>()),
            ("gcd-merge", discovered_laws::<GcdMerge>()),
            ("first-merge", discovered_laws::<FirstMerge>()),
            ("maximal logic", discovered_laws::<MaxLogic>()),
            ("maximal selection", discovered_laws::<MaxSelect>()),
        ];
        let inventory = ShapeCatalog::inventory();
        for (theory, laws) in all {
            for law in laws {
                let entry = inventory.iter().find(|s| s.name == law.shape);
                let Some(entry) = entry else {
                    panic!(
                        "law {:?} (theory {theory:?}) is tagged with shape {:?}, which names \
                         NO catalog entry — an unratified tag escaped `templates()`: add the \
                         shape to `ShapeCatalog::inventory()` (they must move together), \
                         re-bless spec/shapes.spec, and ratify the diff",
                        law.prose, law.shape
                    );
                };
                assert!(
                    entry.matches(&law.prose),
                    "law {:?} (theory {theory:?}) is tagged {:?} but does not instantiate \
                     that shape's prose template {:?} — the tag and the prose at its \
                     `templates()` push site have come apart",
                    law.prose,
                    law.shape,
                    entry.template
                );
            }
        }
    }

    /// The template matcher itself has kill power: it accepts an instance of its own skeleton
    /// and refuses prose from a different shape, a truncation, and a prefix-extended sentence —
    /// so the census cannot be satisfied by a matcher mutated toward `true`.
    #[test]
    fn the_shape_matcher_accepts_instances_and_refuses_impostors() {
        let inv = ShapeCatalog::inventory();
        let shape = |n: &str| *inv.iter().find(|s| s.name == n).expect("shape by name");

        let comm = shape("commutativity");
        assert!(comm.matches("And gives the same result in either order."));
        assert!(!comm.matches("And absorbs Or."), "a different shape");
        assert!(
            !comm.matches("And gives the same result"),
            "a truncated skeleton is not an instance"
        );
        assert!(
            !comm.matches("Oddly, And gives the same result in either order. Mostly."),
            "the first fragment is a PREFIX and the last a SUFFIX — padding is refused"
        );

        // identity vs annihilation share their opening; the tails keep them apart.
        let identity = shape("identity");
        assert!(identity.matches("Multiplication with 1 leaves a value unchanged."));
        assert!(!identity.matches("Multiplication by 0 always gives 0."));

        // a hole spans multi-word operator names, so the census is robust to naming.
        let irrefl = shape("irreflexivity");
        assert!(irrefl.matches("A value is never less than itself."));
    }

    /// `instantiate` round-trips through `matches`: filling a template's holes yields prose
    /// the same shape recognises as its own instance — exactly the guarantee genesis's target
    /// locks lean on (a declared law's line IS what a confirming discovery renders).
    #[test]
    fn instantiate_yields_prose_the_shape_recognises() {
        let inv = ShapeCatalog::inventory();
        let shape = |n: &str| *inv.iter().find(|s| s.name == n).expect("shape by name");

        let comm = shape("commutativity");
        assert_eq!(
            comm.instantiate(&[("op", "grant")]),
            "grant gives the same result in either order."
        );
        assert!(comm.matches(&comm.instantiate(&[("op", "grant")])));

        // a two-hole template fills BOTH holes (and `{const}` everywhere it appears).
        let annihilation = shape("annihilation");
        assert_eq!(
            annihilation.instantiate(&[("op", "Multiplication"), ("const", "0")]),
            "Multiplication by 0 always gives 0."
        );

        // an unnamed hole is left intact — instantiate substitutes, it never invents.
        assert_eq!(
            annihilation.instantiate(&[("op", "spend")]),
            "spend by {const} always gives {const}."
        );
    }

    /// The displayed `schema` string IS the canonical terms rendered over the shape's
    /// placeholder symbols — the string and the term data cannot drift apart, so the data
    /// genesis derives its equations from is exactly what `spec/shapes.spec` ratifies.
    /// (Identity and annihilation append an `(or …)` note for the mirrored variant the
    /// engine also tries; the canonical render is then the schema's strict prefix.)
    #[test]
    fn the_schema_string_is_the_canonical_terms_rendered() {
        for shape in ShapeCatalog::inventory() {
            assert_eq!(
                shape.gate_slots.slots.len(),
                shape.placeholders.len(),
                "`{}` needs one placeholder per slot",
                shape.name
            );
            // the display convention: binaries render infix (`x ⊕ y`), everything else
            // applied (`act(x, e)`, `rel(x, x)`, `u(x)`), constants bare.
            let ops: Vec<(&str, Fixity)> = shape
                .gate_slots
                .slots
                .iter()
                .zip(shape.placeholders)
                .map(|(slot, sym)| {
                    let fixity = match slot {
                        Slot::Binary(_) => Fixity::Infix,
                        Slot::Constant(_) => Fixity::Nullary,
                        Slot::Unary(..) | Slot::Action(..) | Slot::Relation(..) => Fixity::Prefix,
                    };
                    (*sym, fixity)
                })
                .collect();
            // three sort-variable rows: carrier, partner/verdict, and (for the
            // equivariant square) a second-partner param row — the display names may
            // repeat across rows because no stanza renders variables of two non-carrier
            // sorts in one equation.
            let rendered = shape.equation(&ops, &[&["x", "y", "z"], &["p", "q"], &["p", "q"]]);
            if shape.schema == rendered {
                continue;
            }
            // schemas may carry a parenthesised note after the canonical render: the
            // mirrored variant of a two-sided shape ("(or ...)") or a witness shape's
            // quantifier ("(for some ...)").
            let note = shape.schema.strip_prefix(&rendered);
            assert!(
                note.is_some_and(|n| n.starts_with("  (")),
                "`{}`: schema {:?} is not its canonical terms rendered ({:?})",
                shape.name,
                shape.schema,
                rendered
            );
        }
    }

    /// The DATA gate admits every law the CODE gate fires — the two sides of `ShapeInfo`
    /// cannot drift: across all four registry theories, every discovered law's fingerprint
    /// signatures satisfy its shape's `gate_slots`. (Fingerprints shorter than the slot list
    /// — coincident slots, dedup'd, e.g. a self-homomorphism — are exempt: the fingerprint
    /// cannot say which slots coincided.)
    #[test]
    fn every_discovered_law_satisfies_its_shapes_data_gate() {
        fn check<T: Theory>() {
            let engine = Engine::<T>::new();
            let sigs = engine.signatures();
            let symbols: Vec<&'static str> = sigs.iter().map(|(s, _, _)| *s).collect();
            let inventory = ShapeCatalog::inventory();
            for law in engine.discover().laws {
                let shape = inventory
                    .iter()
                    .find(|s| s.name == law.shape)
                    .expect("every law's shape is ratified");
                let ops = law.ops(&symbols);
                if ops.len() != shape.gate_slots.slots.len() {
                    continue;
                }
                let gate_sigs: Vec<(Vec<T::Sort>, T::Sort)> = ops
                    .iter()
                    .map(|sym| {
                        let (_, inputs, output) =
                            sigs.iter().find(|(s, _, _)| s == sym).expect("known op");
                        (inputs.clone(), *output)
                    })
                    .collect();
                if let Err(why) = shape.gate_slots.admit(&gate_sigs, &ops) {
                    panic!(
                        "law `{}` ({}) violates its shape's data gate: {why} — \
                         templates() and gate_slots have come apart",
                        law.prose, law.shape
                    );
                }
            }
        }
        check::<crate::discover::arithmetic::Arithmetic>();
        check::<crate::discover::router::Router>();
        check::<crate::discover::date::Calendar>();
        check::<crate::kvstore::theory::TtlStore>();
    }

    /// The gate REFUSES what its shape cannot range over, naming the fault — the checker's
    /// own kill power: wrong slot kind, unbound-sort mismatch, a required distinction
    /// collapsed (same operator twice; an action whose parameter is its carrier).
    #[test]
    fn the_data_gate_refuses_misdeclarations_by_name() {
        let inv = ShapeCatalog::inventory();
        let gate = |n: &str| inv.iter().find(|s| s.name == n).expect("shape").gate_slots;
        // sorts as plain tokens — the checker is generic over them.
        let binary = (vec!["s", "s"], "s");
        let unary_st = (vec!["s"], "t");
        let constant_t = (Vec::<&str>::new(), "t");

        let err = gate("commutativity")
            .admit(std::slice::from_ref(&unary_st), &["esc"])
            .expect_err("a unary is not a binary");
        assert!(err.contains("`esc` must be a homogeneous binary"));

        let err = gate("identity")
            .admit(&[binary.clone(), constant_t.clone()], &["pool", "unit"])
            .expect_err("the constant's sort must match the binary's");
        assert!(err.contains("`unit` must be a nullary constant"));

        let err = gate("distributivity")
            .admit(&[binary.clone(), binary.clone()], &["pool", "pool"])
            .expect_err("distributivity needs two DIFFERENT binaries");
        assert!(err.contains("must be different operators"));

        let err = gate("action identity")
            .admit(&[(vec!["s", "s"], "s"), (vec![], "s")], &["bump", "unit"])
            .expect_err("an action's parameter sort must differ from its carrier");
        assert!(err.contains("must be distinct"));

        // and the happy paths admit.
        assert_eq!(
            gate("homomorphism").admit(
                &[unary_st, (vec!["s", "s"], "s"), (vec!["t", "t"], "t")],
                &["esc", "cat", "glue"],
            ),
            Ok(())
        );
        assert_eq!(
            gate("round-trip").admit(&[(vec!["t"], "s"), (vec!["s"], "t")], &["unesc", "esc"],),
            Ok(())
        );
    }

    /// The gate counts SIGS and NAMES independently — a caller handing the right number of
    /// signatures but the wrong number of names (or vice versa) is refused, never indexed
    /// out of step (the length check's `||` joins two separate claims).
    #[test]
    fn the_gate_refuses_mismatched_sig_and_name_counts() {
        let inv = ShapeCatalog::inventory();
        let gate = inv
            .iter()
            .find(|s| s.name == "commutativity")
            .expect("shape")
            .gate_slots;
        let binary = (vec!["s", "s"], "s");
        assert!(
            gate.admit(std::slice::from_ref(&binary), &[]).is_err(),
            "one sig, zero names must be refused"
        );
        assert!(
            gate.admit::<&str>(&[], &["pool"]).is_err(),
            "zero sigs, one name must be refused"
        );
    }

    /// A template with NO holes matches exactly its own prose and nothing else — the literal
    /// branch of the census matcher, exercised through a catalog shape with its template
    /// overwritten (the fields are data; that is the point).
    #[test]
    fn a_hole_less_template_matches_only_its_exact_prose() {
        let mut shape = ShapeCatalog::inventory()[0];
        shape.template = "the whole prose, verbatim";
        assert!(shape.matches("the whole prose, verbatim"));
        assert!(!shape.matches("the whole prose, verbatim, plus drift"));
        assert!(!shape.matches(""));
    }

    /// The sampling stride is pinned at the golden point: for a space of 10⁶ the raw stride
    /// 618000 shares factors with the space, so the walk lands on 618001 — stepping DOWN
    /// (or mis-scaling) from the golden point changes this value and degrades the spread.
    #[test]
    fn the_sampling_stride_walks_up_from_the_golden_point() {
        assert_eq!(coprime_step(1_000_000), 618_001);
        // degenerate space: the stride floors at 1 and 1 is coprime to everything.
        assert_eq!(coprime_step(1), 1);
    }

    /// THE VACUITY RULE, pinned from both sides: the guarded shapes fire where the
    /// premise has ground (`le` is antisymmetric and transitive; `less-than` is
    /// transitive), and stay SILENT where it never fires — `less-than` is antisymmetric
    /// only vacuously (`lt(x,y) ∧ lt(y,x)` is satisfiable nowhere), and a vacuous truth
    /// is not a law. This is the fixed-point lesson, guarded.
    #[test]
    fn the_vacuity_rule_keeps_strict_orders_silent() {
        let proses = discovered_prose::<MaxLogic>();
        assert!(proses
            .iter()
            .any(|p| p == "le is antisymmetric — mutual relation forces equality."));
        assert!(proses
            .iter()
            .any(|p| p == "le is transitive (chained through And)."));
        assert!(proses
            .iter()
            .any(|p| p == "less-than is transitive (chained through And)."));
        assert!(
            !proses
                .iter()
                .any(|p| p.contains("less-than is antisymmetric")),
            "a strict order's antisymmetry is vacuous — it must NOT be discovered"
        );
        assert!(proses.iter().any(|p| p == "cap is monotone under le."));
    }

    /// `check` on a frozen guarded law demands the premise stays SATISFIABLE: a law
    /// whose guard lost all ground (here: antisymmetry guarded by a strict order's
    /// mutual relation, unsatisfiable by construction) is an error naming the loss,
    /// never a vacuous pass.
    #[test]
    fn a_guarded_law_that_lost_its_ground_is_named() {
        let e = Engine::<MaxLogic>::new();
        // operator indices in MaxLogic: find lt (name "less-than"), And, true.
        let ops = MaxLogic::operators();
        let idx = |n: &str| ops.iter().position(|o| o.name == n).expect("op");
        let (lt, and, tru) = (idx("less-than"), idx("And"), idx("true"));
        let x = e.var(LSort::N, 0).expect("var x");
        let y = e.var(LSort::N, 1).expect("var y");
        let groundless = DiscoveredLaw {
            shape: "antisymmetry",
            prose: "less-than is antisymmetric — mutual relation forces equality.".into(),
            equation: "(lt(x, y) ∧ lt(y, x)) = true ⟹ x = y".into(),
            lhs: x.clone(),
            rhs: y.clone(),
            polarity: Polarity::Equal,
            premise: Some((
                Term::App(
                    and,
                    vec![
                        Term::App(lt, vec![x.clone(), y.clone()]),
                        Term::App(lt, vec![y, x]),
                    ],
                ),
                Term::App(tru, vec![]),
            )),
        };
        let err = e
            .check(std::slice::from_ref(&groundless))
            .expect_err("an unsatisfiable guard must not read as a passing law");
        assert!(err.contains("lost its ground"), "{err}");
    }

    // -- the TOLERANCED theory: a metric carrier judged at registered bars ------------
    //
    // `blend` is integer averaging — commutative EXACTLY, idempotent exactly, but its
    // associativity carries ±1 of integer-division noise: at exact equality it would be
    // REFUTED, at a sloppy tolerance it would be certified; at the REGISTERED bars it is
    // UNDECIDED, and the spec says so instead of flipping a coin at the boundary.
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
    struct Micro;
    struct NoisyGauge;
    fn blend(v: &[i64]) -> Option<i64> {
        Some((v[0] + v[1]) / 2)
    }

    impl Theory for NoisyGauge {
        type Sort = Micro;
        type Value = i64;
        type Obs = i64;
        fn name() -> &'static str {
            "noisy gauge"
        }
        fn operators() -> Vec<Operator<Self>> {
            vec![Operator {
                name: "blend",
                symbol: "blend",
                fixity: Fixity::Infix,
                inputs: vec![Micro, Micro],
                output: Micro,
                eval: blend,
            }]
        }
        fn inhabitants(_: Micro) -> Vec<i64> {
            vec![0, 1, 2, 3]
        }
        fn sort_of(_: &i64) -> Micro {
            Micro
        }
        fn observe(v: &i64) -> i64 {
            *v
        }
        fn sort_vars(_: Micro) -> &'static [&'static str] {
            &["x", "y", "z"]
        }
        fn grid_size() -> usize {
            64 // = 4³, the whole space: the bars are judged exhaustively.
        }
        fn judge(a: &i64, b: &i64) -> Verdict {
            match (a - b).abs() {
                0 => Verdict::Holds,
                1 => Verdict::Undecided,
                _ => Verdict::Refuted,
            }
        }
        fn tolerance() -> Option<&'static str> {
            Some("micro-units: exact ⇒ holds; |Δ| = 1 ⇒ undecided; else refuted")
        }
    }

    /// The three-valued judgment, end to end: exact laws are laws, boundary noise is
    /// UNDECIDED (neither certified nor refuted), and the lock text carries both the
    /// registered bars and the disclosed band — ε is ratified with the laws.
    #[test]
    fn a_toleranced_theory_disccloses_its_undecided_band() {
        let d = Engine::<NoisyGauge>::new().discover();
        let laws: Vec<&str> = d.laws.iter().map(|l| l.prose.as_str()).collect();
        assert!(laws.contains(&"blend gives the same result in either order."));
        assert!(laws.contains(&"blend of a value with itself gives that value."));
        assert!(
            !laws.iter().any(|p| p.contains("grouping")),
            "noisy associativity must NOT be certified: {laws:?}"
        );
        assert_eq!(
            d.undecided
                .iter()
                .map(|(p, _)| p.as_str())
                .collect::<Vec<_>>(),
            vec!["With blend, the grouping of three values doesn't matter."],
            "the ±1 associativity noise lands in the DISCLOSED band"
        );

        // and the lock text carries the whole story.
        let live = crate::discover::Spec::of::<NoisyGauge>()
            .lock_in(std::path::Path::new("spec"))
            .live;
        assert!(live.contains("# tolerance (registered with the theory): micro-units:"));
        assert!(live.contains("|Δ| = 1 ⇒ undecided; else refuted"));
        assert!(live.contains(
            "# undecided at the declared tolerance (disclosed — neither held nor refuted):"
        ));
        assert!(live.contains("\n- With blend, the grouping of three values doesn't matter."));
    }

    /// `check` on a frozen law that DRIFTS into the band is an error naming the drift —
    /// never a silent pass, never a spurious hard failure.
    #[test]
    fn a_frozen_law_drifting_into_the_band_is_named() {
        let e = Engine::<NoisyGauge>::new();
        let x = e.var(Micro, 0).expect("x");
        let y = e.var(Micro, 1).expect("y");
        let z = e.var(Micro, 2).expect("z");
        // associativity, frozen as if it had once been certified.
        let frozen = DiscoveredLaw {
            shape: "associativity",
            prose: "With blend, the grouping of three values doesn't matter.".into(),
            equation: "((x blend y) blend z) = (x blend (y blend z))".into(),
            lhs: Term::App(0, vec![Term::App(0, vec![x.clone(), y.clone()]), z.clone()]),
            rhs: Term::App(0, vec![x, Term::App(0, vec![y, z])]),
            polarity: Polarity::Equal,
            premise: None,
        };
        let err = e
            .check(std::slice::from_ref(&frozen))
            .expect_err("band drift must be named");
        assert!(
            err.contains("became undecided at the declared tolerance"),
            "{err}"
        );
    }

    /// THE LOCK: the catalog's deterministic rendering matches the committed, ratified
    /// `spec/shapes.spec` — the same spec-lock discipline as the theory specs, applied to the
    /// law-language itself.
    #[test]
    fn the_committed_shape_catalog_is_fresh() {
        let lock = ShapeCatalog::lock();
        if spec_lock::check(std::slice::from_ref(&lock)).is_err() {
            panic!(
                "the shape catalog changed — a new/changed universal shape alters every \
                 consumer's discovered spec; ratify spec/shapes.spec and note it in the \
                 release contract. Regenerate with `cargo run --example freeze_shapes` and \
                 put the diff through review."
            );
        }
    }

    /// The partial-operator convention at `judge_at`, pinned point-blank: two undefined
    /// observations agree (both sides undefined TOGETHER is the convention that lets a
    /// partial operator's laws hold where it is honestly silent), a definedness
    /// mismatch refutes, and two defined values defer to the theory's judge.
    #[test]
    fn judge_at_holds_the_partiality_convention() {
        assert_eq!(
            Engine::<NoisyGauge>::judge_at(&None, &None),
            Verdict::Holds,
            "undefined-together must HOLD, never refute"
        );
        assert_eq!(
            Engine::<NoisyGauge>::judge_at(&None, &Some(0)),
            Verdict::Refuted
        );
        assert_eq!(
            Engine::<NoisyGauge>::judge_at(&Some(1), &Some(1)),
            Verdict::Holds
        );
        assert_eq!(
            Engine::<NoisyGauge>::judge_at(&Some(0), &Some(3)),
            Verdict::Refuted
        );
    }

    // ===== the one-sided-meaningless witness refusal ========================
    // `lt` is a relation that is UNDEFINED EVERYWHERE; `no` is a defined constant.
    // A witness shape comparing them has one meaningful side and one meaningless
    // side, and must be REFUSED — a relation with no defined instance witnesses
    // nothing. (The refusal requires BOTH sides meaningful, not either.)
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
    enum GhostSort {
        V,
        B,
    }
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
    enum GhostVal {
        V(i64),
        B(bool),
    }
    struct GhostRelation;
    fn ghost_lt(_: &[GhostVal]) -> Option<GhostVal> {
        None // undefined everywhere — the ghost
    }
    fn ghost_no(_: &[GhostVal]) -> Option<GhostVal> {
        Some(GhostVal::B(false))
    }
    impl Theory for GhostRelation {
        type Sort = GhostSort;
        type Value = GhostVal;
        type Obs = GhostVal;
        fn name() -> &'static str {
            "ghost relation"
        }
        fn operators() -> Vec<Operator<Self>> {
            vec![
                Operator {
                    name: "lt",
                    symbol: "lt",
                    fixity: Fixity::Infix,
                    inputs: vec![GhostSort::V, GhostSort::V],
                    output: GhostSort::B,
                    eval: ghost_lt,
                },
                Operator {
                    name: "no",
                    symbol: "false",
                    fixity: Fixity::Nullary,
                    inputs: vec![],
                    output: GhostSort::B,
                    eval: ghost_no,
                },
            ]
        }
        fn inhabitants(sort: GhostSort) -> Vec<GhostVal> {
            match sort {
                GhostSort::V => vec![GhostVal::V(0), GhostVal::V(1)],
                GhostSort::B => vec![GhostVal::B(false), GhostVal::B(true)],
            }
        }
        fn sort_of(v: &GhostVal) -> GhostSort {
            match v {
                GhostVal::V(_) => GhostSort::V,
                GhostVal::B(_) => GhostSort::B,
            }
        }
        fn observe(v: &GhostVal) -> GhostVal {
            *v
        }
        fn sort_vars(sort: GhostSort) -> &'static [&'static str] {
            match sort {
                GhostSort::V => &["x", "y", "z"],
                GhostSort::B => &["p", "q", "r"],
            }
        }
    }

    /// An everywhere-undefined relation earns NO witness: `lt(x, y) ≠ false` has a
    /// meaningless left side and a meaningful right side, and the witness driver must
    /// refuse it rather than let the definedness mismatch masquerade as a difference.
    #[test]
    fn a_ghost_relation_witnesses_nothing() {
        let d = Engine::<GhostRelation>::new().discover();
        assert!(
            !d.laws.iter().any(|l| l.equation.contains("lt")),
            "an undefined-everywhere relation must appear in NO law: {:?}",
            d.laws.iter().map(|l| l.prose.as_str()).collect::<Vec<_>>()
        );
        // and the silence is recorded where it belongs: coverage, not a witness.
        assert!(d.uncovered_ops.contains(&"lt"));
    }

    // ===== the undecided band's dedup =======================================
    // `blend` with a constant at 2, on a grid where the constant's identity law is
    // UNDECIDED in BOTH mirrored orientations — the disclosure must carry the law
    // once, not once per orientation.
    struct NoisyPair;
    fn two(_: &[i64]) -> Option<i64> {
        Some(2)
    }
    impl Theory for NoisyPair {
        type Sort = Micro;
        type Value = i64;
        type Obs = i64;
        fn name() -> &'static str {
            "noisy pair"
        }
        fn operators() -> Vec<Operator<Self>> {
            vec![
                Operator {
                    name: "blend",
                    symbol: "blend",
                    fixity: Fixity::Infix,
                    inputs: vec![Micro, Micro],
                    output: Micro,
                    eval: blend,
                },
                Operator {
                    name: "two",
                    symbol: "2",
                    fixity: Fixity::Nullary,
                    inputs: vec![],
                    output: Micro,
                    eval: two,
                },
            ]
        }
        fn inhabitants(_: Micro) -> Vec<i64> {
            vec![1, 2, 3]
        }
        fn sort_of(_: &i64) -> Micro {
            Micro
        }
        fn observe(v: &i64) -> i64 {
            *v
        }
        fn sort_vars(_: Micro) -> &'static [&'static str] {
            &["x", "y", "z"]
        }
        fn grid_size() -> usize {
            27
        }
        fn judge(a: &i64, b: &i64) -> Verdict {
            match (a - b).abs() {
                0 => Verdict::Holds,
                1 => Verdict::Undecided,
                _ => Verdict::Refuted,
            }
        }
        fn tolerance() -> Option<&'static str> {
            Some("micro-units: exact ⇒ holds; |Δ| = 1 ⇒ undecided; else refuted")
        }
    }

    /// A law undecided under BOTH of a mirrored shape's orientations is disclosed
    /// ONCE — the band is a set of laws, not a log of attempts.
    #[test]
    fn an_undecided_mirrored_law_is_disclosed_once() {
        let d = Engine::<NoisyPair>::new().discover();
        let prose: Vec<&str> = d.undecided.iter().map(|(p, _)| p.as_str()).collect();
        assert!(
            prose.contains(&"blend with 2 leaves a value unchanged."),
            "the mirrored identity must land in the band: {prose:?}"
        );
        let mut deduped = prose.clone();
        deduped.sort_unstable();
        deduped.dedup();
        assert_eq!(
            deduped.len(),
            prose.len(),
            "each undecided law is disclosed exactly once: {prose:?}"
        );
    }
}
