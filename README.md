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
                          at undoes since — the round trip is the identity.  at(since(s)) = s
                          add (a heterogeneous action) is reported UNCOVERED — where the spec is silent.
```

Routers have no structural `Eq` — they are compared by how they route a path grid, i.e.
**observationally**, which is exactly what the engine groups by. The interpreter adds one
**structural** law over a synthetic **universal observer `U`** (its faithful rendering): `eval`
collapses structure (`2+3` and `5` are equal to it), so a transform that computes the right value but
mangles the tree is invisible to the equations — *no two distinct programs look the same to `U`*
closes that. And the **coverage report** names operators in no law: where the spec is silent and
human attention belongs.

The discovered spec is **frozen** into a committed file per theory (`spec/*.spec`) — a behaviour
lock. CI re-derives the live spec and fails if it drifts from the committed text, so the committed
file read in a PR diff IS the **ratification**, and an unintended behaviour change is a build error.
The only human acts left are the **validity rule** (`Int ≥ 0`) and ratifying that diff: a law you
expect but don't see is a bug surfaced.

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
derived probes and the mutation sweep validate the spec.

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
