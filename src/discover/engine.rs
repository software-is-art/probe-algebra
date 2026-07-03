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
    /// The ratified catalog shape this law instantiates — EXACTLY a `ShapeCatalog::inventory()`
    /// name, set at the `templates()` push site that minted the law. The tag is what the
    /// declared-expectations layer (`discover::expect`) compares against: prose is for humans,
    /// the tag is the machine-checkable identity of the shape.
    pub shape: &'static str,
    pub prose: String,
    pub equation: String,
    pub lhs: Term,
    pub rhs: Term,
}

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

// ================================================================================================
// THE SHAPE CATALOG — this block and `Engine::templates()` below MUST MOVE TOGETHER.
//
// `templates()` is the battery stated as CODE (shapes that fire); `ShapeCatalog::inventory()` is
// the same battery stated as DATA (shapes that are ratified). Add, remove, or reword a shape in
// one and the census tests fail until the other follows — and until the regenerated
// `spec/shapes.spec` diff is reviewed.
// ================================================================================================

/// One universal algebraic shape, as a ratifiable datum: its name, its schematic equation, the
/// applicability GATE that decides which operators it is tried on, and the prose TEMPLATE a
/// discovered instance renders with (`{op}`/`{other}`/`{via}` are operator names, `{const}` a
/// constant's symbol).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ShapeInfo {
    /// The shape's name ("commutativity", "bias (right-regular)").
    pub name: &'static str,
    /// The schematic equation, over placeholder operators ("(x ⊕ y) = (y ⊕ x)").
    pub schema: &'static str,
    /// When the shape is tried, in prose ("homogeneous binary; skipped when commutative").
    pub gate: &'static str,
    /// The prose template a discovered instance renders with — the exact `format!` skeleton in
    /// `templates()`, with `{...}` holes where operator/constant names are substituted.
    pub template: &'static str,
}

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

impl ShapeCatalog {
    /// The full inventory, in the order `templates()` tries the shapes. KEEP IN STEP with
    /// `templates()` — the census tests hold each side against the other, and the lock holds
    /// both against `spec/shapes.spec`.
    pub fn inventory() -> Vec<ShapeInfo> {
        vec![
            ShapeInfo {
                name: "commutativity",
                schema: "(x ⊕ y) = (y ⊕ x)",
                gate: "homogeneous binary (s × s → s)",
                template: "{op} gives the same result in either order.",
            },
            ShapeInfo {
                name: "associativity",
                schema: "((x ⊕ y) ⊕ z) = (x ⊕ (y ⊕ z))",
                gate: "homogeneous binary (s × s → s)",
                template: "With {op}, the grouping of three values doesn't matter.",
            },
            ShapeInfo {
                name: "idempotence",
                schema: "(x ⊕ x) = x",
                gate: "homogeneous binary (s × s → s)",
                template: "{op} of a value with itself gives that value.",
            },
            ShapeInfo {
                name: "bias (right-regular)",
                schema: "((x ⊕ y) ⊕ x) = (y ⊕ x)",
                gate: "homogeneous binary; skipped when commutative (a commutative operator \
                       has no bias to state); excludes the left-regular variant on the grid",
                template: "With {op}, the later operand wins where the two disagree — \
                           re-applying an earlier one cannot overwrite it.",
            },
            ShapeInfo {
                name: "bias (left-regular)",
                schema: "((x ⊕ y) ⊕ x) = (x ⊕ y)",
                gate: "homogeneous binary; skipped when commutative (a commutative operator \
                       has no bias to state); excludes the right-regular variant on the grid",
                template: "With {op}, the earlier operand wins where the two disagree — \
                           a later one cannot overwrite it.",
            },
            ShapeInfo {
                name: "identity",
                schema: "(e ⊕ x) = x  (or (x ⊕ e) = x)",
                gate: "homogeneous binary plus a constant of its sort; tried on both sides, \
                       deduplicated by prose",
                template: "{op} with {const} leaves a value unchanged.",
            },
            ShapeInfo {
                name: "annihilation",
                schema: "(a ⊕ x) = a  (or (x ⊕ a) = a)",
                gate: "homogeneous binary plus a constant of its sort; tried on both sides, \
                       deduplicated by prose",
                template: "{op} by {const} always gives {const}.",
            },
            ShapeInfo {
                name: "distributivity",
                schema: "(x ⊕ (y ⊗ z)) = ((x ⊕ y) ⊗ (x ⊕ z))",
                gate: "an ordered pair of distinct homogeneous binaries on one sort",
                template: "{op} distributes over {other}.",
            },
            ShapeInfo {
                name: "absorption",
                schema: "(x ⊕ (x ⊗ y)) = x",
                gate: "an ordered pair of distinct homogeneous binaries on one sort",
                template: "{op} absorbs {other}.",
            },
            ShapeInfo {
                name: "action identity",
                schema: "act(x, e) = x",
                gate: "heterogeneous binary s × t → s (an action of t on s) plus a constant \
                       of the parameter sort t",
                template: "{op} with {const} leaves a value unchanged.",
            },
            ShapeInfo {
                name: "monoid action",
                schema: "act(act(x, p), q) = act(x, (p ⊗ q))",
                gate: "heterogeneous binary s × t → s plus a homogeneous binary on the \
                       parameter sort t",
                template: "Repeated {op} combines its parameters with {other}.",
            },
            ShapeInfo {
                name: "irreflexivity",
                schema: "rel(x, x) = false",
                gate: "relation s × s → r (r ≠ s) whose output sort carries a constant \
                       rendering as `false`",
                template: "A value is never {op} itself.",
            },
            ShapeInfo {
                name: "self-application",
                schema: "rel(x, x) = c",
                gate: "relation s × s → r (r ≠ s) plus any other constant of the output sort",
                template: "{op} of a value with itself gives {const}.",
            },
            ShapeInfo {
                name: "involution",
                schema: "u(u(x)) = x",
                gate: "unary s → s",
                template: "{op} twice returns the original value.",
            },
            ShapeInfo {
                name: "round-trip",
                schema: "g(f(x)) = x",
                gate: "a pair of distinct unaries f : s → t and g : t → s",
                template: "{op} undoes {other} — the round trip is the identity.",
            },
            ShapeInfo {
                name: "homomorphism",
                schema: "h((x ⊕ y)) = (h(x) ⊗ h(y))",
                gate: "unary h : s → t plus a homogeneous binary on s and one on t",
                template: "{op} turns {other} into {via}.",
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
    ///
    /// !! MOVE TOGETHER !! This battery and `ShapeCatalog::inventory()` above are the same
    /// catalog stated twice — as code that fires and as data that is locked in
    /// `spec/shapes.spec`. A shape added, removed, or reworded here must be mirrored there
    /// (the census tests enforce both directions — including that every `shape` tag pushed
    /// here names a catalog entry), and the regenerated lock diff ratified.
    fn templates(&self) -> Vec<DiscoveredLaw> {
        let mut out: Vec<DiscoveredLaw> = Vec::new();
        let mut seen_prose: Vec<String> = Vec::new();
        let mut push = |this: &Self, shape: &'static str, prose: String, lhs: Term, rhs: Term| {
            if this.same(&lhs, &rhs) && !seen_prose.contains(&prose) {
                seen_prose.push(prose.clone());
                let equation = format!("{} = {}", this.render(&lhs), this.render(&rhs));
                out.push(DiscoveredLaw {
                    shape,
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
                    "commutativity",
                    format!("{} gives the same result in either order.", f.name),
                    Self::app(fid, vec![x.clone(), y.clone()]),
                    Self::app(fid, vec![y.clone(), x.clone()]),
                );
                // associativity: f(f(x,y),z) = f(x,f(y,z))
                push(
                    self,
                    "associativity",
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
                    "idempotence",
                    format!("{} of a value with itself gives that value.", f.name),
                    Self::app(fid, vec![x.clone(), x.clone()]),
                    x.clone(),
                );

                // BIAS — the regular-band "sandwich" laws, tried only when order matters (a
                // commutative operator has no bias to state): with `(x⊕y)⊕x`, does the re-applied
                // EARLIER operand overwrite the later one? `(x⊕y)⊕x = y⊕x` says no — the later
                // operand wins wherever the two disagree (right-regular / last-write-wins);
                // the mirror `(x⊕y)⊕x = x⊕y` says the earlier wins (left-regular /
                // first-write-wins). This shape exists because a hostile domain taught us the
                // monoid laws are BIAS-BLIND: first- and last-write-wins merges satisfy
                // identical monoid laws, so WHICH side wins was invisible to the discovered
                // spec until this template named it. Given non-commutativity the two variants
                // are mutually exclusive on the grid.
                let commutative = self.same(
                    &Self::app(fid, vec![x.clone(), y.clone()]),
                    &Self::app(fid, vec![y.clone(), x.clone()]),
                );
                if !commutative {
                    let sandwich = Self::app(
                        fid,
                        vec![Self::app(fid, vec![x.clone(), y.clone()]), x.clone()],
                    );
                    push(
                        self,
                        "bias (right-regular)",
                        format!(
                            "With {}, the later operand wins where the two disagree — \
                             re-applying an earlier one cannot overwrite it.",
                            f.name
                        ),
                        sandwich.clone(),
                        Self::app(fid, vec![y.clone(), x.clone()]),
                    );
                    push(
                        self,
                        "bias (left-regular)",
                        format!(
                            "With {}, the earlier operand wins where the two disagree — \
                             a later one cannot overwrite it.",
                            f.name
                        ),
                        sandwich,
                        Self::app(fid, vec![x.clone(), y.clone()]),
                    );
                }

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
                        "identity",
                        format!("{} with {} leaves a value unchanged.", f.name, cs),
                        Self::app(fid, vec![c.clone(), x.clone()]),
                        x.clone(),
                    );
                    push(
                        self,
                        "identity",
                        format!("{} with {} leaves a value unchanged.", f.name, cs),
                        Self::app(fid, vec![x.clone(), c.clone()]),
                        x.clone(),
                    );
                    // annihilation: f(c,x) = c  (or f(x,c) = c)
                    push(
                        self,
                        "annihilation",
                        format!("{} by {} always gives {}.", f.name, cs, cs),
                        Self::app(fid, vec![c.clone(), x.clone()]),
                        c.clone(),
                    );
                    push(
                        self,
                        "annihilation",
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
                            "distributivity",
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
                            "absorption",
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
                            "action identity",
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
                                "monoid action",
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
                        let (shape, prose) = if cs == "false" {
                            (
                                "irreflexivity",
                                format!("A value is never {} itself.", f.name),
                            )
                        } else {
                            (
                                "self-application",
                                format!("{} of a value with itself gives {}.", f.name, cs),
                            )
                        };
                        push(
                            self,
                            shape,
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
                        "involution",
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
                                "round-trip",
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
                                    "homomorphism",
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
            shape: "commutativity",
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
    // boolean sort (irreflexivity), a mod-3 codec (round-trips), and a PARTIAL `half` (defined
    // only on even values — it earns no law and must not fake one).
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
    // identity, monoid action), and a `Gap` relation whose self-application is `zero`.
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
}
