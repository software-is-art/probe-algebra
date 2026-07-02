# boundary-algebra

**Can a module boundary be specified precisely enough that the tests which validate it
write themselves?**

That is the only question this crate exists to answer. The yardstick: take a module,
**mutate its interior**, and check every mutation is caught — *without one hand-written
interior test*. A strong enough boundary kills the mutants anyway; a weak one lets them
survive.

**The answer is yes.** The demonstration is an expression-language interpreter — lexer,
parser, type checker, evaluator — whose interior has **zero tests of its own**, and whose
*positive* behaviour is certified by **derived probes and discovered laws, not example tests**
(the only hand-written examples that remain are a disclosed few that exercise the grammar
plumbing itself — see `src/tests.rs`'s inventory) — and its algebraic laws are **discovered by
running the operators**, not declared, by a generic engine that does the same for a
non-commutative router and a multi-sorted date calculus, rendering as a plain-language spec. Mutate the interior and **every viable mutant dies**; the only survivors are a handful of
documented equivalents (genuine free choices), carved out in `.cargo/mutants.toml`.

```
cargo run --bin demo   # the boundary narrated end-to-end
cargo test             # unit + property + compile-fail suites
cargo mutants          # the real test: do interior mutants survive?
```

---

## The fear, and the answer to it

Functional-sounding disciplines scare people because they make a whole codebase rigid. Here
the defining property is the opposite:

**Rigidity lives in exactly one place — the module's boundary. Everything inside is free.**

The interior may be as imperative, mutable, and allocation-happy as you like — ordinary Rust.
Paying for the boundary once buys validation of the spec's claims two ways, at no further cost:

| | at **compile time** | at **autotest time** |
|---|---|---|
| how | the type system rejects ill-formed use | derived probes + mutation testing |
| e.g. | an unchecked program can't be evaluated; a lossy edge can't run as pure; a cost ceiling is exceeded | the parser round-trips; the optimizer's constant is right; no interior mutant survives |

You write the *meaning* — validity rules, operators, transformation bodies. The grammar mints
the *proofs*.

---

## A boundary, concretely

A boundary is a small **category**: objects are *value objects* (a domain primitive plus its
validity rule), morphisms are *edges* in one of four type-distinguished shapes. The
interpreter's whole public surface is three edges:

```rust,ignore
pub struct Parse;   // Construction : String -> Expr       (parse, don't validate)
pub struct Check;   // Branch       : Expr   -> WellTyped | IllTyped
pub struct Eval;    // Guarded      : Expr   -> Value       (needs a WellTyped witness)

refined! {
    /// A non-negative integer literal.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Shaped)]
    pub struct Int(i64);
    fn new(n: i64) = (n >= 0).then_some(n);   // the predicate is the only content
}
```

From those declarations the crate **derives** — nothing further written by hand — input
generators, each type's degree-of-freedom set, a **complete probe** (so a probe that misses a
dimension is *unrepresentable*, not merely caught), and a **fused universal probe**
(`sensitive_to_all`) sensitive to structure, value, and semantics at once.

And it **enforces**, at compile time, claims that can be statically false:

- `Eval` cannot run without the `WellTyped` witness only `Check` mints — *"well-typed programs
  don't go wrong"* is a build error, not a runtime hope;
- effect ceilings (`run_pure` refuses a `Lossy` edge) and cost budgets (a quadratic pass
  demanded linear) are build errors;
- the **grading algebras prove their own laws.** The parts mutation structurally cannot reach —
  bodiless type tables for capability, cost, and provenance — are certified by `typewit::TypeEq`
  witnesses, and over the *open* type families (Peano degrees, lineages) the core laws are
  **total, by structural induction**, not a sample. (See [concepts](docs/concepts.md#the-gradings).)
- **every production edge carries a probe.** `build.rs` enumerates every concrete edge impl and
  rejects any without an `impl Probed` — so an edge no probe can kill cannot be merged.
- **the partition is total and explicit.** Every source file declares its tier — `KERNEL` (the
  trusted floor: grammar, engine, macros, tooling), `BOUNDARY` (a domain's strict surface),
  `INTERIOR` (the workshop / leaves), or `ALGEBRA` (a discovered-law / report layer) — with a
  `//! Tier:` marker `build.rs` reads to dispatch the right discipline. A file that names no tier is
  a build error, so a new module cannot land silently un-categorized; placement is ratified in the
  diff. This replaces the old path heuristics (the blanket `discover/` exemption is gone). And
  because `KERNEL` is exempt from the structural rules, claiming it is not self-service: kernel
  files are enumerated in an allowlist in `build.rs` itself, so joining the trusted floor is a
  diff to the floor's own gate.
- **effects in an ALGEBRA file are honest, or they don't build.** That layer may touch the world (a
  report tool reads sources, writes a scaffold) — but not *silently*: a function whose body reaches
  `std::fs`/`io`/`process`/`net`/`env` — fully qualified, through a `use` import or alias, or inside
  a macro's tokens — must declare a real `Capability:` (one of the four levels) in its doc, else
  build error. So the fine seam/capability/leaf split *inside* an ALGEBRA file is enforced the same
  way the file-level tier is — the dev tool's `apply` (writes) and `theory_line` (reads) are named
  edges, not hidden dependencies. (Adding the rule immediately flagged two world-reads we'd left
  undeclared.) The residual honesty gap is transitive effects — a helper that does the I/O for you —
  which `build.rs` documents rather than pretends to close.

The interior backing these edges (`interp::internal`) is private, untested, and kept in the
mutation sweep, so the bought correctness is *measured*, not asserted.

---

## The laws discover themselves — for *any* boundary, not just arithmetic

You don't state the algebra, and there's no catalog of shapes to match against. A domain implements
one trait — `Theory` (its sorts, operators, a grid of inhabitants, and an OBSERVATION on values) —
and the generic **engine** enumerates terms over the operators, groups them by how they behave on
the grid, instantiates the universal algebraic shapes over the operators, and keeps the ones that
run true. The spec falls out of the operators' behaviour, not a human's list. The algebra was never
about numbers; numbers were just *legible*. `cargo run --example discovered_spec` discovers three
very different algebras from the **same engine**:

```
interpreter arithmetic    Addition gives the same result in either order.   (x + y) = (y + x)
                          Multiplication distributes over Addition.         (x * (y+z)) = (x*y)+(x*z)
                          A value is never less than itself.                (x < x) = false
router (a monoid)         Or with empty leaves a value unchanged.           (empty or a) = a
                          With Or, the grouping doesn't matter.             ((a or b) or c) = …
                          — and NOT commutativity: overlapping routers route differently each way,
                            so the engine correctly refuses to report it.
date calculus (2 sorts)   Plus with zero leaves a value unchanged.          (zero + p) = p
                          Add with zero leaves a value unchanged.           add(s, zero) = s
                          at undoes since — the round trip is the identity.  at(since(s)) = s
```

The universal shapes the engine tries cover the heterogeneous cases too — monoid **actions**
(date's `add`), **homomorphisms** (boolean De Morgan, `¬(x∧y) = ¬x ∨ ¬y`), absorption,
distributivity — so it finds the laws that are actually there, across sorts. Routers have no
structural `Eq` — they are compared by how they route a path grid, i.e. **observationally**, which
is exactly what the engine groups by. The interpreter adds one **structural** law over a synthetic
**universal observer `U`** (its faithful rendering): `eval` collapses structure (`2+3` and `5` are
equal to it), so a transform that computes the right value but mangles the tree is invisible to the
equations — *no two distinct programs look the same to `U`* closes that. And when a shape doesn't
reach an operator, the **coverage report** names it: where the spec is silent and human attention
belongs.

The discovered spec is **frozen** into a committed file per theory (`spec/*.spec`) — a behaviour
lock. CI re-derives the live spec and fails if it drifts from the committed text, so the committed
file read in a PR diff IS the **ratification**, and an unintended behaviour change is a build error.
The only human acts left are the **validity rule** (`Int ≥ 0`) and ratifying that diff: a law you
expect but don't see is a bug surfaced.

### The grid writes itself, too — from the type, not the operators

A `Theory` needs a **grid** of values to judge laws on, and that grid is the one part that resists
derivation: too few values and a false law survives (over-fitting), and a boundary whose operators
don't *generate* values — a bare monoid, a router whose `or` never leaves its seeds, a lattice with
no constant — can't bootstrap a grid at all. The fix is a **shadow algebra**: synthetic generators
the author never writes and that never enter the spec. Reusing the same `#[derive(Shaped)]` that
mints the probe surface for edges, the grid is grown from the value type's *structure* — start at the
canonical inhabitant, close under its variant/field perturbations — so it is fattened by the type,
not by the operators or by hand. And the assignments the laws are judged on keep pace: a small
variable/inhabitant cross-product is enumerated **exhaustively** (every combination, not a sample);
a large one is sampled by a coprime-stride mixed-radix decode, so every variable varies
independently of every other. A domain then collapses to *just its operators*:

```rust,ignore
theory! {
    Lattice : "tri lattice", Value = Tri, Obs = Tri, Sort = LatticeSort,
    sort_of = |_| LatticeSort::T, observe = |v| *v,
    ops {  Infix "meet" "&" (..) = meet;  Infix "join" "|" (..) = join;  }
}   // no inhabitants, no variables, no constants — the grid comes from Tri's structure
```

`Tri` is a three-element lattice with **no constant operator**, so closing under its boundary would
leave the grid empty; the shadow algebra supplies its three inhabitants, and over them the engine
discovers the whole **distributive-lattice spec** — both operations commutative, associative,
idempotent, with both distributivities and both absorptions — hands-free. The author wrote the value
object and two functions; the grid, the variables, and the ten laws wrote themselves.

---

## Coherence: do two modules *agree*, not just *connect*?

The type system answers "do these modules **connect**?" — type, witness, and grading compatibility,
total at compile time. It does not answer "do they **agree**?". Two modules can be perfectly
connectable and silently *incoherent* — meaning different things about a value type they share. So
`discover::coherence` checks it behaviourally: two same-signature modules are **coherent** iff every
law one discovers also holds under the other's operators. `max`-merge and `gcd`-merge are coherent
(different operators, identical laws — coherence is law-agreement, not operator-equality); `max`-merge
and a first-match merge are **incoherent** (both expose `merge : Key × Key → Key`, so they wire and
type-check, but they disagree on commutativity — the bug class types can't see). This is the
behavioural analog of `gdp`'s proof-carrying seam: gdp carries a value's *proof* across a seam, this
checks its *laws* survive it.

It frames a whole program as a **graph of algebras**: each module is a node (its discovered algebra),
each seam an edge that is either a **transport** (the algebra stays — checked by coherence) or a
**transform** (the algebra changes — checked by the homomorphism law, `h(a op b) = h(a) op' h(b)`,
e.g. `eval`). Dataflow is sound when every transport seam is coherent and every transform seam is a
homomorphism.

---

## Cohesion: when should a module be split?

The smell of a badly-factored module isn't a *large* algebra — boolean algebra is large and superb.
It's a **decomposable** one: secretly several algebras crammed together with no laws connecting them.
So `discover::cohesion` builds the **operator-interaction graph** — operators are nodes, two are
linked whenever a discovered law mentions both — and its weakly-connected components are the **latent
modules**. One dense component → cohesive, keep it; several components → the cut is where it wants to
split, and the seam is classified transport vs transform (so it tells you whether the split needs a
coherence check or a homomorphism). It's a *suggestion*, never a constraint — a modularity signal you
ratify, like an editor quick-fix.

It is self-applicable (`cargo run --example cohesion`):

```
module `interpreter arithmetic`: decomposes into 2 — { 0, 1, +, * } and { false, < }
                                 seam on Int — transport (no law links < to + or *)
module `router`:                 cohesive — one algebra, keep as one module
module `date calculus`:          decomposes into 2 — { zero, +, add, diff } and { since, at }
                                 seam on Date, Duration — transform (since/at convert: a layer line)
```

And the suggestion has an *action*: `discover::scaffold` (`cargo run --example scaffold`) emits the
split — one `theory!` skeleton per component (operators, fixities, and sorts carried faithfully; the
`eval` functions left as move-here markers, since the interior is what moves), plus the seam
obligation. The split is **lossless** by construction — components are defined by law-connectivity,
so every discovered law lives entirely inside one sub-module — and the only obligation is the seam: a
transport seam shares a type (safe by construction), a transform seam emits a homomorphism check so a
bad cut becomes a failing probe, not a silent bug. You (or an agent, naming as it goes) ratify and
apply it — a quick-fix from signal to safe refactor.

---

## Layering: when is a module *sprawling* rather than split?

Cohesion catches one pathology — a module that is secretly *several* (disconnected algebras → split).
But a large algebra is not itself the smell (boolean algebra is large and superb). The other
pathology is a module that is **one connected algebra yet holds together only through a load-bearing
operator** — really two tighter sub-algebras hinged at a point. That wants to **layer**, not split.

`discover::layering` reads it off the same operator-interaction graph, **structurally, with no
threshold**: an operator is a **hinge** when it is a graph *articulation point* — removing it would
disconnect its component, so the rest of the algebra only holds together *through* it. No hinge →
**atomic**, one tight layer, keep it. A hinge → the natural seam to introduce a layer. Run
`cargo run --example layering`:

```
interpreter arithmetic   component { 0, 1, +, * } — layered at hinge: *   (1 reaches the ring only
                         component { false, < }   — atomic                 through multiplication)
router                   every component is atomic — no layering pressure
date calculus            component { zero, +, add, diff } — layered at hinge: zero
```

So the two analyses are complementary selection pressures: **split when disconnected, layer when
sprawling.** Router is tight (no hinge); arithmetic's ring genuinely pivots on `*`. It is the same
kind of suggestion — a structural signal a human (or an agent) ratifies, never a constraint.

---

## Composition: what survives a pipeline of modules

Cohesion and layering look *inside* a module. The last question is the whole-program one: a program is
a graph of algebras joined by seams, and a **transform seam** is a conversion `h : A → B` that is a
homomorphism — it carries the source algebra into the target. A real program **chains** them,
`A → B → C`, and a single module's discovery cannot see whether structure *survives the chain*.

`discover::composition` answers it by running it. If `h1 : A → B` and `h2 : B → C` are each
homomorphisms, the **composite** `h2∘h1 : A → C` is one too — verified over the source grid, not
assumed. So along a transform pipeline the **operation changes at every stage** (`⊕_A → ⊕_B → ⊕_C`)
but the **law is invariant**: the dataflow preserves the algebra end to end. `cargo run --example
composition` discovers it on a three-stage reading pipeline:

```
reading pipeline   report ∘ scale: cR → cP is a homomorphism — cP(h(x), h(y)) = h(cR(x, y))
                   (combine changes at every stage; the composite still preserves it end to end)
interpreter / router / date   no transform pipeline — no cross-module composite law
```

So this is the answer to *"do algebras stay the same or change over modules?"* — they **change** (the
operators differ at each stage) yet the **structure is conserved** along the flow. A non-homomorphic
stage produces no law: the composite is checked, never presumed.

---

## Modularize: reading structure out of an unstructured bag

Cohesion, layering, and composition all *critique a module that already has a shape*. The final turn
points the whole stack at the **pathological input**: one flat file with everything crammed together —
functions from several unrelated algebras dumped in a heap, no structure at all — the bag a real agent
hands you. There is no shape to critique yet; the job is to **propose** one.

`discover::modularize` makes **the algebra itself the selection criterion**. It partitions the
functions by law-connectivity (the same components cohesion reads), **scores** each cluster by how many
discovered laws live inside it, and marks each one's tightness (layering). What falls out is a
**ranking**: the richest shapes first, and at the bottom the **misfits** — functions that cohere with
nothing, which the proposal *refuses to dress up as a module*. `cargo run --example modularize` feeds
it a bag of four functions over three unrelated types — a `max` semilattice, an `and`/`or` lattice, and
a structureless three-cycle:

```
bag `flat soup` → proposed decomposition:
  shape 0: { both, either } — 10 law(s), atomic     (the distributive lattice — richest)
  shape 1: { peak } — 3 law(s), atomic              (the max semilattice)
  misfits (bound by no law — left unstructured): rotate
```

The lattice outranks the semilattice because more laws bind it; `rotate` (a three-cycle that satisfies
no universal shape) is flagged as noise rather than packaged. **Nothing about this decomposition was
written down** — `#[algebra]` synthesised the theory from just the functions, and the structure was
read off the discovered laws. It is the culmination of the stack: cohesion says *this wants to split*,
layering says *this wants to layer*, and modularize says *here is the structure hiding in your
unstructured bag, ranked by how real it is.*

---

## The architect: the suggestion as a dev tool — and the tool is itself an algebra

The cohesion signal and its scaffold only become an *editor experience* when they speak the
editor's protocol, so `discover::architect` packages them as **LSP**: it analyses each registered
theory and, for the decomposable ones, emits a **diagnostic** (a `Hint`, anchored at the `theory!`
declaration) plus a `refactor.extract` **code action** whose `WorkspaceEdit` *creates* the scaffolded
sub-module files. `cargo run --example architect` prints the LSP payload; `-- --apply <dir>` writes
the files. It's a quick-fix like any other — the agent names things as it goes, applies it if it
wants, ignores it if it doesn't.

The dogfood goes one level deeper than "the tool analyses our modules". **The tool's own domain is a
discovered algebra.** An architect run produces a `Report` — the set of flagged modules — and that
`Report` is a join-semilattice: merging two runs is **commutative, idempotent, associative, with the
empty report as identity**. So the architect models its own output as a `theory!`, and the engine
*discovers* those four laws by running them — the same pipeline that probes arithmetic now probes the
report type that decides whether arithmetic should be split. A test asserts the architect's `Report`
theory is itself **cohesive** (one algebra, correctly *not* a candidate for its own refactor) and
that a live run flags the modules we expect. The abstraction validates the tool that wields the
abstraction.

### The imperative shell, in-format too — and where the format *stops*

A dev tool is mostly the unglamorous parts: serialise to a wire format, write files. Those are the
real test of the claim, because we didn't get to design them as clean algebras — and every workload
is mostly made of them. So the rest of the architect is pushed into the format too, and each layer
lands in a different place:

- **The serialiser is a discovered algebra.** The JSON wire-escaping (`esc`/`unesc`) is modelled as
  a string `theory!` over concatenation, and the engine *discovers* its structure: a **homomorphism**
  (`esc(a ++ b) = esc(a) ++ esc(b)` — so the payload escapes piecewise, exactly how it is assembled)
  and a **codec round-trip** (`unesc(esc(s)) = s`). The hand-written escape test is gone; a mutated
  serialiser breaks the round-trip and the law vanishes.
- **The finding it forced.** Demanding the round-trip is what *taught* us something the curated
  domains never could: the escaper had been **dropping** `\r`, which makes it non-invertible — and
  the engine flatly refused the codec law until `\r` was escaped instead. The format caught a latent
  lossiness we'd hand-waved.
- **Where the algebra stops.** The codec *structure* is discovered, but the *encoding* — which
  character maps to which escape — is **not a law**: many invertible homomorphisms exist (the
  identity is one), so the specific arms are a representation **convention** (conformance to the JSON
  standard), an irreducible **leaf** pinned by an oracle. Modelling the serialiser in-format doesn't
  dissolve the human input; it *locates* it precisely.
- **The effect is a declared, bounded capability.** `apply` writes files — the one **`Effectful`**
  edge (`effectful ⊃ stateful ⊃ lossy ⊃ pure`), declared rather than hidden; everything else here is
  `Pure`. Its effect is **confined** to the target root (an edit path that escapes via `..` or an
  absolute path is rejected, not written), so the declared bound is real. The `std::fs::write`
  beneath it is the leaf where the world is finally touched.

So the same module spans the whole partition — **discovered algebra** (the report, the codec), a
**capability-typed effect edge** (`apply`), and the **leaves** the laws bottom out in (the encoding
convention, the syscall) — which is the honest shape of any program: laws where they exist, declared
capabilities where the world is touched, named leaves where derivation stops.

---

## Qualification: boundary-hood is a *computed* property, not a file

So far a module *declares* itself — a `boundary.rs` file, a `//! Tier:` marker, an `#[algebra]`
attribute. The last step removes the declaration from the question itself: **does this module meet
the algebra spec?** is something the build *computes*. `build.rs` scans every module and reports
which ones are **operator-shaped** — every function argument and return a bare named value type, no
raw primitives, no I/O (the shape `#[algebra]` reads) — independent of where the module lives or what
it is named. A module qualifies as a boundary by *structure*; `boundary.rs` is just one place that
happens to.

The census is frozen into [`spec/qualify.spec`](spec/qualify.spec) and drift-gated, so the answer is
ratified in the diff. And running it on the crate's *own* code finds what you'd hope — a module
nobody wrote as an algebra qualifying anyway:

```
src/capability.rs:          QUALIFIES — operators [cap_of] over sorts {Capability, Source}
src/discover/derived.rs:    QUALIFIES — operators [meet, join, lift, …] over sorts {Tri, Small, Large}
src/discover/modularize.rs: QUALIFIES — operators [both, either, peak, rotate] over sorts {Count, Flag, Spin}
```

`capability`'s `cap_of : Source → Capability` is an operator over value objects, so the audit module
*is* a boundary by the same definition the discovery domains are — it was simply never declared one.
That is the answer to "must a boundary be a `boundary.rs`?": no — boundary-hood is a property a module
either has or doesn't, and the build now reads it off the code.

---

## Mutation runs where it pays — and self-hosts

Mutation testing is expensive, so it runs against the **specification**, not the interior:

- it certifies the spec's *derived* probes are strong enough to kill any interior bug;
- once a module passes, an interior-only change cannot weaken that guarantee — the probes come
  from the spec — so the sweep re-runs only when the **boundary** changes.

You pay the mutation cost once, against the abstraction; consumers inherit the assurance for
free and keep CI fast. CI ([`ci.yml`](.github/workflows/ci.yml)) reflects this: fmt + clippy +
test on every push, the mutation gate on each PR's *changed lines*, and a full-crate sweep
(`0 missed`) on the default branch and weekly.

A handful of mutants survive that *no* probe can kill — **equivalent mutants**, behaviourally
identical to the original (deciding equivalence is undecidable, so they're ratified by hand). The
method refuses to let them rot as buried config: `discover::residue` makes the residue a **classified
finding**. A behaviourally-inert expression is either a **redundant guard** — simplify it away and the
mutant is *eliminated*, not excluded (as `shadow_grid`'s doubled cap-check and `select`'s guards
already were) — or a **free choice** the spec doesn't constrain (accept it, or tighten the spec). A
**drift gate** keeps that classified list in lockstep with the carve-outs the gate actually applies,
so an exclusion can't accumulate undocumented. Equivalent mutants become simplification work, surfaced
like any other suggestion (`cargo run --example residue`) — the irreducible human bit is just the
*why* (which kind), not the bookkeeping.

The strongest evidence is that the method is **turned on its own runtime**:

- **`interp`** and **`select`** (the kill-matrix selector that picks the probes) are *structural*
  self-hosts — boundary plus an interior with no tests of its own.
- **`gdp`** (the name/proof vocabulary) and **`capability`** (the declared-vs-behaviour audit)
  are crate-level grammar, self-hosted by *verification* — example tests replaced with
  oracle-free property probes, kept in the sweep.

And `gdp`'s relational proof is load-bearing, not a demo: `select`'s kernel reads its kill
matrix through gdp's `InBounds` proof, which **holds the matrix's borrow** — a proof minted for
one matrix cannot read another, and an unproven read is not writable at all — "make illegal
states unrepresentable" lifted from one value to a *relation between two values*. (A ratified
redesign: a proof keyed only on a phantom brand was not value-unique, since a region brands
many values; the borrow now carries the identity the brand alone could not.)

---

## Reading further

- **[docs/concepts.md](docs/concepts.md)** — the precise model: the boundary as a *graded
  category*, the four edge shapes, the four gradings and how each proves its laws, and the
  probe taxonomy. Every term maps to a trait or function.
- **[docs/how-it-works.md](docs/how-it-works.md)** — the end-to-end mechanism: what you write,
  what compile time and autotest time each give you, and why an under-specified probe is
  unrepresentable.

## Using it

The grammar is domain-agnostic — the interpreter is the lead worked example, the router and date
calculus show the discovery engine generalises, and none of it is the limit:

```toml
[dependencies]
boundary-algebra = { git = "https://github.com/software-is-art/probe-algebra" }
```

Model your boundary as value objects and edges, write the interior in any style, and let the
derived probes and the mutation sweep validate the spec. For a discovery domain specifically, there
is no longer a declaration to write at all: `#[algebra]` reads a module of ordinary operator
**functions** and synthesises the whole `Theory` — each signature gives its arity (→ fixity), its
single value type (→ the sort), its name (→ the symbol); the grid is shadow-derived from the type and
the observation is the value itself. So the agent authors only what it *means*, the value object and
the operators:

```rust,ignore
#[algebra(Lattice, "tri lattice")]
pub mod lattice {
    #[derive(Shaped, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
    pub enum Tri { Lo, Mid, Hi }

    pub fn meet(a: Tri, b: Tri) -> Tri { a.min(b) }   // (Tri, Tri) -> Tri ⇒ binary operator
    pub fn join(a: Tri, b: Tri) -> Tri { a.max(b) }
}
// Engine::<lattice::Lattice>::new().discover() — the whole distributive-lattice spec, hands-free
```

Everything a `theory!` once spelled out — the operator table, `sort_of`, `observe`, `Obs`, the
variables, the grid — is now read off the functions or derived. It is **multi-sorted**: operators may
range over several value types (a `Date`/`Duration` calculus), and the macro synthesises the `Value`
sum and the `sort_of` that tags it from the signatures — the engine then discovers the laws *across*
the sorts (a conversion's homomorphism included). What remains is only the irreducible **meaning**:
the value objects and the operators (the module itself). A *deliberate* deviation — a behavioural
observer (the router, judged by how it routes) or a hand-curated grid (arithmetic's, chosen so the
discovered spec reads cleanly) — is still written out with the explicit `theory!` form, precisely
because it is a choice, not boilerplate.

## Costs, paid openly

- the boundary is **verbose** — that is the single place the rigidity is paid, by design;
- GDP name-branding runs inside a `with_seed` continuation (fine for a program, an imposition
  on a library's callers);
- the type level checks *declared* cost composes within budget; `fits` audits a leaf's declared
  degree empirically, but the type level cannot see a leaf's true cost. Capability is sharper: an
  edge that **under**-declares (claims `Pure` over a state-carrying input) is a compile error,
  inferred from the input type; only **over**-claiming and `Effectful`/I/O — negatives the type
  system can't express — remain the behavioural audit's job.

## License

Dual-licensed under either of [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT), at your
option. Unless you state otherwise, any contribution you submit for inclusion shall be
dual-licensed as above, without additional terms.
