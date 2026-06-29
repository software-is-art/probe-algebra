# boundary-algebra

**Can a module boundary be specified precisely enough that the tests which validate it
write themselves?**

That is the only question this crate exists to answer. The yardstick has been fixed
throughout: take a module, **mutate its interior code**, and check that every mutation is
caught — *without writing a single test of the interior by hand*. If the boundary
specification is strong enough, the mutants die anyway. If they survive, the specification
was too weak.

**The answer is yes.** The demonstration:

> An expression-language interpreter whose interior — lexer, parser, type checker,
> evaluator — has **zero tests of its own**, and whose entire *positive* behaviour
> (evaluation, parsing, type-checker acceptance) is certified with **zero hand-written
> examples** too: only declared laws and structure-derived probes. Mutating the interior,
> **every viable mutant is killed** — the one equivalent (a relaxed lexer guard whose
> newly-reachable inputs `Ident::new` already rejects) is the single carve-out, documented in
> `.cargo/mutants.toml`. Not one positive test of the interpreter was written by hand.

---

## The fear, and the answer to it

Functional-sounding disciplines scare people because they tend to make a whole codebase
rigid. The defining property here is the opposite:

**Rigidity lives in exactly one place — the module's boundary specification. Everything
inside the module is free.**

The interior may be as imperative, mutable, allocation-happy, and un-functional as you
like. It is ordinary Rust. What you buy by paying for the boundary once is the ability to
**validate the claims in the specification two ways at no further cost**:

| | at **compile time** | at **autotest time** |
|---|---|---|
| how | the type system rejects ill-formed use | derived probes + mutation testing |
| examples | "an unchecked program cannot be evaluated"; "a lossy edge cannot run as pure"; "a cost ceiling is exceeded" | "the parser round-trips"; "the optimizer's constant is right"; "no internal mutation survives" |

You write the *meaning* (the validity rules, the operators, the transformation bodies).
The grammar mints the *proofs*.

---

## A boundary, concretely

A module's boundary is a small **category**: its objects are *value objects* (every domain
primitive wrapped with its validity rule) and its morphisms are *edges* in one of a few
type-distinguished shapes. The interpreter's entire public surface is three edges:

```rust,ignore
// The SPECIFICATION — the only rigid part of the module.
pub struct Parse;   // Construction : String -> Expr      (parse, don't validate)
pub struct Check;   // Branch       : Expr   -> WellTyped + IllTyped
pub struct Eval;    // Guarded      : Expr   -> Value      (needs a WellTyped witness)

// Value objects declare their validity rule and derive their probe surface.
refined! {
    /// A non-negative integer literal.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Shaped)]
    pub struct Int(i64);
    fn new(n: i64) = (n >= 0).then_some(n);   // <- the only content
}
```

From those declarations alone the crate **derives**, with nothing further written by hand:

- input **generators** (`Shaped::inhabitant`) from each type's structure;
- the **degree-of-freedom set** and a **complete probe** (`HasDofs` + `Complete<T>`) — so a
  probe that misses a dimension is not merely *caught*, it is *unrepresentable*;
- a **fused universal probe** (`sensitive_to_all`) that is sensitive to structure, value,
  and (through recursion) semantics in a single operator.

And it **enforces**, at compile time:

- `Eval` cannot be called without the `WellTyped` witness only `Check` can mint — *“well-typed
  programs don't go wrong”* becomes a build error, not a runtime hope
  (`tests/compile_fail/eval_wrong_program`);
- effect ceilings (`run_pure` rejects a `Lossy` edge) and cost budgets (a quadratic pass
  demanded to be linear) are build errors (`tests/compile_fail/`);
- the **grading algebras prove their own laws** — the parts of capability, cost, and provenance
  that are bodiless type tables (which mutation structurally cannot reach) are certified by
  `typewit::TypeEq` witnesses: a non-commutative join, a `Lookup` that didn't fold the per-axis
  max, a non-associative lineage concat all fail to compile (residual, a free product, is proven
  up to isomorphism by proptest instead). See [docs/concepts.md](docs/concepts.md#the-gradings).

The interior that backs these edges — `interp::internal` — is private, untested, and stays
in the mutation sweep so the bought correctness is *measured*, not asserted.

```
$ cargo run --bin demo      # the boundary narrated end-to-end
$ cargo test                # unit + property + compile-fail suites
$ cargo mutants             # the real test: do interior mutants survive?
```

---

## Declare the law; the probe is generated

You never hand-write a probe's plumbing. For the value frontier you state the *meaning* — an
algebraic law the operator obeys — and the harness mints the probe around it:

```rust,ignore
// The only hand-written part: the meaning itself.
Law::Identity    { op: Op::Add, element: Expr::int(0) }   // x + 0 == x
Law::Commutative { op: Op::Mul }                          // x * y == y * x
Law::Reflexive   { op: Op::Lt,  value: false }            // x < x == false
```

Each `Law` fans out into a first-class relation, run over a derived generator, with the
applicability guard *and the non-vacuity check owned by the harness*. A probe whose guard
never fires — the hand-written-guard antipattern, a "passing" test that never actually
ran — is rejected on an ordinary `cargo test`, no mutation needed. The author cannot weaken
a probe by accident, because the author does not write the part that could be weak.

---

## Where mutation lives — and why consumers don't pay for it

Mutation testing is expensive, so it must run where it pays. Here it is a function of the
**specification**, not the interior:

- it certifies that the spec's *derived* probes are strong enough to kill any interior bug;
- once a module passes, an *interior-only* change cannot weaken that guarantee — the probes
  come from the spec, not the code — so the sweep only needs to re-run when the **boundary**
  changes, never on an interior-only PR.

That is the payoff. You pay the mutation cost once, against the abstraction; consumers of the
abstraction inherit the assurance for free and keep their CI fast.

The strongest evidence for this is that the method is **turned on its own kernel**. The
selector that picks a minimal, *attributing* probe set from a kill matrix (`src/select/`) is
specified in the very discipline it serves: its data are value objects, its interior carries
no example tests, and only its oracle-free probes — judged by the mutation sweep — certify
it, the way a compiler is compiled by itself.

And the gate runs **all the time** ([`ci.yml`](.github/workflows/ci.yml)): fmt + clippy +
test on every push and PR, the mutation gate on each PR's *changed lines* (cheap, per-change),
and a full-crate sweep on the default branch and a weekly schedule. The whole-crate sweep is
green — `0 missed` — because the only three equivalents left are genuine free choices (a
`bool` seed value, a deliberately-empty capability declaration, a guard `Ident::new` already
subsumes), carved out by function in `.cargo/mutants.toml`; everything else is killed or
detected.

---

## How the pieces fit

- **[docs/concepts.md](docs/concepts.md)** — the precise model: the boundary as a *graded
  category*, the four edge shapes (`Morphism` / `Construction` / `Branch` / `Guarded`), the
  gradings (residual, capability, cost, provenance), and the layered probe taxonomy. For
  readers who want the terminology to check the claims.
- **[docs/how-it-works.md](docs/how-it-works.md)** — the end-to-end mechanism: what you
  write, what compile time gives you, what autotest time gives you, and how *full mechanical
  derivation* makes a weak probe impossible to express. The "all the pieces together" tour.

---

## Using it

Add it as a dependency (the grammar is domain-agnostic — the interpreter is one worked
example, not the limit):

```toml
[dependencies]
boundary-algebra = { git = "https://github.com/software-is-art/probe-algebra" }
```

Model your module's boundary as value objects and edges, write the interior in whatever
style suits it, and let the derived probes and the mutation sweep validate the
specification's claims:

```
cargo run --bin demo      # the boundary narrated end-to-end
cargo test                # unit + property + compile-fail suites
cargo mutants             # the real test: do interior mutants survive?
```

## The whole runtime is self-hosted

The method is turned on *every part of its own runtime*, not just the interpreter. Each
runtime module is certified by oracle-free probes with zero hand-written example tests, judged
by mutation:

- **`interp`** and **`select`** are *structural* self-hosts — boundary plus a private interior
  with no interior tests.
- **`gdp`** (the Ghosts-of-Departed-Proofs name/proof vocabulary) and **`capability`** (the
  declared-vs-behavioural capability audit) are crate-level grammar, self-hosted by replacing
  their example tests with oracle-free property probes.

And gdp's relational proof is now *load-bearing*, not a demo: `select`'s kernel reads its kill
matrix entirely through gdp's `InBounds` proof (`positions` ⇒ `at_in_bounds`), so an
out-of-range matrix read is a **type error, not a panic** — a value object enforces a
single-value invariant; this enforces a *relation between two values* (an index belongs to
*that* matrix).

## Scope and costs

The grammar holds the interior-mutation bar across the whole runtime, with the costs paid
openly:

- the boundary is **verbose** — that verbosity is the single place the rigidity is paid, and
  the point is to pay it once;
- GDP name-branding runs inside a `with_seed` continuation (fine for a program, an imposition
  on a library's callers);
- the cost grading checks *declared* complexity composes within budget; `fits` audits a
  leaf's declared degree empirically, but the type level cannot see a leaf's true cost.

## License

Dual-licensed under either of [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT), at your
option. Unless you state otherwise, any contribution you submit for inclusion shall be
dual-licensed as above, without additional terms.
