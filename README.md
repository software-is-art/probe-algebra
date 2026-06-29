# boundary-algebra

**Can a module boundary be specified precisely enough that the tests which validate it
write themselves?**

That is the only question this crate exists to answer. The yardstick: take a module,
**mutate its interior**, and check every mutation is caught — *without one hand-written
interior test*. A strong enough boundary kills the mutants anyway; a weak one lets them
survive.

**The answer is yes.** The demonstration is an expression-language interpreter — lexer,
parser, type checker, evaluator — whose interior has **zero tests of its own**, and whose
entire *positive* behaviour is certified with **zero hand-written examples**: only declared
laws and structure-derived probes. Mutate the interior and **every viable mutant dies**; the
only survivors are a handful of documented equivalents (genuine free choices), carved out in
`.cargo/mutants.toml`.

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

## Declare the law; the probe is generated

You never hand-write a probe's plumbing. For the value frontier you state the *meaning* — an
algebraic law — and the harness mints the probe around it:

```rust,ignore
Law::Identity    { op: Op::Add, element: Expr::int(0) }   // x + 0 == x
Law::Commutative { op: Op::Mul }                          // x * y == y * x
Law::Reflexive   { op: Op::Lt,  value: false }            // x < x == false
```

Each `Law` fans out into a first-class relation over a derived generator, with the
applicability guard *and the non-vacuity check owned by the harness*: a probe whose guard never
fires — a "passing" test that never ran — is rejected on an ordinary `cargo test`. The author
cannot weaken a probe by accident, because the author never writes the part that could be weak.

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

The grammar is domain-agnostic — the interpreter is one worked example, not the limit:

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
  degree empirically, but the type level cannot see a leaf's true cost — and, likewise, a
  declared *capability* is audited against behaviour at runtime, not inferred from it (yet).

## License

Dual-licensed under either of [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT), at your
option. Unless you state otherwise, any contribution you submit for inclusion shall be
dual-licensed as above, without additional terms.
