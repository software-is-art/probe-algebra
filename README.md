# boundary-algebra

**Can a module boundary be specified precisely enough that the tests which validate it
write themselves?**

That is the only question this crate exists to answer. The yardstick has been fixed
throughout: take a module, **mutate its interior code**, and check that every mutation is
caught — *without writing a single test of the interior by hand*. If the boundary
specification is strong enough, the mutants die anyway. If they survive, the specification
was too weak.

The answer this crate reaches is **yes**, on the one substrate it has been driven against:

> An expression-language interpreter whose interior — lexer, parser, type checker,
> evaluator — has **zero tests of its own**. Mutating that interior produces 73 viable
> mutants; the boundary specification plus its *derived* probes kill **72 of 73**, and the
> lone survivor is provably equivalent (a relaxed lexer guard whose only newly-reachable
> inputs `Ident::new` already rejects). No test of the interior was written by hand.

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
  demanded to be linear) are build errors (`tests/compile_fail/`).

The interior that backs these edges — `interp::internal` — is private, untested, and stays
in the mutation sweep so the bought correctness is *measured*, not asserted.

```
$ cargo run --bin demo      # the boundary narrated end-to-end
$ cargo test                # unit + property + compile-fail suites
$ cargo mutants             # the real test: do interior mutants survive?
```

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

## Status

A **research crate**, not a released library. It has been hardened against one substrate
(the interpreter) to the point where the interior-mutation bar holds; it has not been run
against many. The costs are real and stated plainly in the docs (notably: GDP name-branding
runs inside a `with_seed` continuation, and the boundary is verbose by design — that verbosity
is the single place the rigidity is paid). 100% agent-authored; the verbosity is judged an
acceptable price when the boundary is the only thing that must be gotten right.
