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
cargo test    # the algebra's invariants as tests
```
