# boundary-algebra

**Can a module boundary be specified precisely enough that the tests which validate it
write themselves?**

That is the only question this crate exists to answer. The yardstick: take a module,
**mutate its interior**, and check every mutation is caught — *without one hand-written
interior test*. A strong enough boundary kills the mutants anyway; a weak one lets them
survive.

**The answer is yes.** The demonstration is an expression-language interpreter — lexer,
parser, type checker, evaluator — whose interior has **zero tests of its own**, and whose
entire *positive* behaviour is certified with **zero hand-written examples** — and its algebraic
laws are **discovered by running the operators**, not declared, by a generic engine that does the
same for a non-commutative router and a multi-sorted date calculus, rendering as a plain-language
spec. Mutate the interior and **every viable mutant dies**; the only survivors are a handful of
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
  diff. This replaces the old path heuristics (the blanket `discover/` exemption is gone).
- **effects in an ALGEBRA file are honest, or they don't build.** That layer may touch the world (a
  report tool reads sources, writes a scaffold) — but not *silently*: a function whose body reaches
  `std::fs`/`io`/`process`/`net`/`env` must declare a `Capability:` in its doc, else build error. So
  the fine seam/capability/leaf split *inside* an ALGEBRA file is enforced the same way the file-level
  tier is — the dev tool's `apply` (writes) and `theory_line` (reads) are named edges, not hidden
  dependencies. (Adding the rule immediately flagged two world-reads we'd left undeclared.)

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

## Mutation runs where it pays — and self-hosts

Mutation testing is expensive, so it runs against the **specification**, not the interior:

- it certifies the spec's *derived* probes are strong enough to kill any interior bug;
- once a module passes, an interior-only change cannot weaken that guarantee — the probes come
  from the spec — so the sweep re-runs only when the **boundary** changes.

You pay the mutation cost once, against the abstraction; consumers inherit the assurance for
free and keep CI fast. CI ([`ci.yml`](.github/workflows/ci.yml)) reflects this: fmt + clippy +
test on every push, the mutation gate on each PR's *changed lines*, and a full-crate sweep
(`0 missed`) on the default branch and weekly.

The strongest evidence is that the method is **turned on its own runtime**:

- **`interp`** and **`select`** (the kill-matrix selector that picks the probes) are *structural*
  self-hosts — boundary plus an interior with no tests of its own.
- **`gdp`** (the name/proof vocabulary) and **`capability`** (the declared-vs-behaviour audit)
  are crate-level grammar, self-hosted by *verification* — example tests replaced with
  oracle-free property probes, kept in the sweep.

And `gdp`'s relational proof is load-bearing, not a demo: `select`'s kernel reads its kill
matrix through gdp's `InBounds` proof, so an out-of-range read is a **type error, not a panic** —
"make illegal states unrepresentable" lifted from one value to a *relation between two values*.

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
derived probes and the mutation sweep validate the spec. For a discovery domain specifically, the
whole `Theory` is generated by a `theory!` block — you write only the value objects, the operator
functions, and the declaration; the operator table, `sort_of`, `observe`, and the rest are minted.

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
