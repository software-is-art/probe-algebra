# probe-algebra

Experiments with constraining bugs by making module boundaries an algebra.

## The boundary discipline

A module's **`boundary.rs`** is the *only* surface it exposes to other modules,
and it may contain exactly three kinds of citizen:

| Citizen | What it is | Marker (sealed) |
| --- | --- | --- |
| **Value object** | immutable, validated-at-construction, value-equality data | `ValueObject` |
| **Typestate** | a type encoding *where in a protocol* a value sits (compile-time) | `Typestate` |
| **Value operator** | a pure morphism over value objects — no I/O, no external mutation | `ValueOperator` |

The markers are **sealed**, so the set of boundary citizens is *closed*: no
module can invent a fourth kind, and no external crate can mint one. Non-boundary
logic (algorithms, mutation, primitives) lives in private sibling modules and is
unreachable across the boundary.

## The algebra

`src/boundary.rs` defines the grammar once for the whole crate:

- **`Morphism`** — a possibly-lossy map `In -> (Out, Residual)`, where the
  `Residual` value object witnesses *exactly* what `forward` collapsed.
  Retaining the residual restores invertibility (`backward(forward(x)) == x`).
- **`Perturbation`** — a partial value operator `In -> In` that nudges the input
  along one dimension.
- **`probe`** — the generic residual-completeness check. Perturb the lost
  dimension; a *complete* residual leaves the output invariant, makes the
  residual respond, and still round-trips on the perturbed input.
- **`Compose`** / **`Pair`** — loss composes as a value object: `g ∘ f` has
  residual `Pair<R_f, R_g>`, so invertibility flows *through* lossy stages as
  long as the accumulated residual is retained.
- **`Carried<M, Retained|Discarded>`** — the retention typestate. `invert` exists
  only while the residual is `Retained`; discarding it removes invertibility *at
  compile time*.

## The worked example

`src/ledger/` is a domain module exposing only `ledger::boundary`. Aggregating a
`Transaction` (a multiset of postings) into an `AccountSummary` is lossy in the
*multiplicity* dimension; `Aggregate` is one `Morphism` instance whose
`MultiplicityResidual` captures it. `AggregateDropsAmounts` is a *type-identical*
buggy morphism whose residual records only posting counts — the `probe` catches
it (round-trip fails) where the type checker cannot.

```
cargo run     # narrated walk-through
cargo test    # invariants as unit tests + property tests
```

## Enforcement (build tooling)

`build.rs` parses every domain boundary file (`src/<module>/boundary.rs`) with
`syn` and **fails the build** if it contains anything outside the grammar: free
functions, global `static`s, submodules, traits, public fields, or any `unsafe`
/ I/O (boundaries are a pure value layer). The universal grammar file
`src/boundary.rs` is exempt — it *defines* the vocabulary. Sealed marker traits
keep the citizen set closed in the type system; the build script keeps each
boundary file structurally honest. A violation looks like:

```
warning: src/ledger/boundary.rs: free function `helper` — value operators must
         be types implementing ValueOperator; put pure helpers in a private module
error: boundary grammar enforcement failed: 1 violation(s)
```

## Property testing

`src/properties.rs` turns the single-sample probe into coverage with `proptest`:
strategies generate transactions (through the public smart constructors only) and
the algebra's laws are checked over hundreds of inputs — honest aggregation always
round-trips, composition round-trips through two lossy stages, the honest residual
is always complete under perturbation, and the count-only residual is *always*
caught. A failing probe is real evidence of an incomplete residual; the property
suite is what lets a *passing* probe stand for "complete across the input space"
rather than "complete for one example".
