# probe-algebra

Experiments with constraining bugs by making module boundaries an algebra.

The constraint under study: **every primitive that means something in the domain
must be a value object, and every operation on it a value operator** — no raw
primitive arithmetic at a call site. Money is `Cents` (an amount) and `Balance`
(a sum), each with its own operators (`split`, `checked_add`, `split_dollar`, …);
account names are `Account`; the only place a raw `i64`/`String` appears is inside
a value object's own operator or its accessor (the sanctioned exit hatch). This
is a `lib` (the algebra + vocabulary) plus a thin `demo` bin.

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

`build.rs` parses the source with `syn` and **fails the build** on two tiers of
violation. Sealed marker traits keep the citizen set closed in the type system;
the build script keeps each file structurally honest.

**Tier 1 — domain boundary files (`src/<module>/boundary.rs`):** the strict
grammar. No free functions, global `static`s, submodules, traits, public fields,
or any `unsafe` / I/O (boundaries are a pure value layer).

**Tier 2 — module-internal files (e.g. `internal.rs`):** the "workshop", where
mutation and raw collections are fine — but the *inward* rule still holds: a
function may not **return a raw primitive** — `String`/`&str` or any numeric
(`i64`, `usize`, `f64`, …) — because every domain primitive must be a value
object with its own operators (`Account` / `Cents` / `Balance`). `bool` is exempt
(a predicate is control, not domain data); accessors that unwrap to a primitive
live at the boundary (tier 1), the sanctioned exit hatch. Files directly under
`src/` (the grammar `boundary.rs`, `main.rs`, tests) are exempt.

```
warning: src/ledger/boundary.rs: free function `helper` — value operators must
         be types implementing ValueOperator; put pure helpers in a private module
warning: src/ledger/internal.rs: `account_label` returns a raw String/str —
         parse-don't-validate: a domain string is a validated value object...
error: boundary discipline enforcement failed: 2 violation(s)
```

### Tier 2 in practice: morphisms reach inward

The ledger's aggregation logic now lives in a private `internal::Aggregation`
that is itself a `Morphism` over value objects — so the **same generic `probe`
tests it directly**, even though it never crosses a boundary (`Aggregate` is a
thin boundary adapter over it). Because the residual keeps value objects
(`Account` / `Cents`) instead of raw primitives, `backward` is *total*: there is
nothing left to re-validate. That is the payoff of pushing the discipline inward
— not blanket newtype-wrapping (a `struct Count(pub usize)` would fail the
value-object test anyway), but extending the algebra's reach and removing a
class of reconstruction failures.

## Property testing

`src/properties.rs` turns the single-sample probe into coverage with `proptest`:
strategies generate transactions (through the public smart constructors only) and
the algebra's laws are checked over hundreds of inputs — honest aggregation always
round-trips, composition round-trips through two lossy stages, the honest residual
is always complete under perturbation, and the count-only residual is *always*
caught. A failing probe is real evidence of an incomplete residual; the property
suite is what lets a *passing* probe stand for "complete across the input space"
rather than "complete for one example".
