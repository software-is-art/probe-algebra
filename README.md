# probe-algebra

Experiments with constraining bugs by making module boundaries an algebra, and
catching bugs in (especially AI-generated) transformation code by **generating
probe operators from the degrees of freedom of a value object** and checking a
transformation against a *layered* suite.

Two constraints are under study together:

1. **Every primitive that means something in the domain is a value object, and
   every operation on it a value operator** — no raw primitive arithmetic at a
   call site. Money is `Cents` (an amount) and `Balance` (a sum); account names
   are `Account`; a scalar is `Quantity`. The only place a raw `i64`/`String`
   appears is inside a value object's own operator or accessor (the sanctioned
   exit hatch).
2. **A transformation is checked by a layered probe suite, not a single check** —
   because no single check is highest-assurance (see the blind-spot map below).

This is a `lib` (the algebra + vocabulary) plus a thin `demo` bin.

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

## The layered probe suite

A transformation is a `Morphism` `In -> (Out, Residual)`. It is checked by three
layers of increasing cost and decreasing generality — each catches a bug class
the previous one is blind to:

### Layer 1 — type construction (free, always on)
A value object's constructor enforces the structural rules (ranges,
non-emptiness, …). Malformed outputs *cannot be constructed*, so a whole bug
class is gone before any probe runs.

### Layer 2 — structural / variational probes (reference-FREE)
Perturb the input and check the output *responds correctly*, without knowing the
correct output value — so they work on a fully opaque `forward`:

- **`commutes`** (`Metamorphic`): `forward(op(x)) == op_out(forward(x))`. Checks
  *how* the output varies. Its blind spot is the constant: a uniform bug that
  respects the relation survives.
- **`probe`** (residual completeness / round-trip): perturb the lost dimension; a
  *complete* residual leaves the output invariant, makes the residual respond,
  and still round-trips. Catches loss/reconstruction bugs.

### Layer 3 — quantitative / coefficient probes (reference-BEARING)
- **`coefficient_holds`** (`Coefficient`): a known unit step on the input must
  produce a known output delta — the forward map's actual coefficient. This
  *pins values*, catching a right-shape / wrong-constant bug that every
  structural check is blind to. It needs an external correctness criterion (a
  spec or independent reference), and *is* the absolute invariant decomposed
  coefficient-by-coefficient.

### The blind-spot map (proven in `src/blindspot.rs`)

| bug | round-trip | commutation | quantitative |
| --- | --- | --- | --- |
| residual incompleteness (`AggregateDropsAmounts`) | **catches** | blind | n/a |
| non-linear output offset (`AggregateOffsetsTotals`) | blind | **catches** | catches |
| wrong-but-invertible coefficient (`Scale::skew`) | blind | blind | **catches** |

**The decisive negative result:** `Scale::honest` and `Scale::skew` are the *same
type* with different rates. `skew` is wrong but invertible, so it round-trips
**and** commutes with doubling — both structural checks are blind. Only the
reference-bearing quantitative probe separates them. You cannot collapse the
structural and quantitative layers into one.

## The generation + selection loop

The human authors only the value objects and a closed operator set; the checking
is generated and pruned:

- **`src/select.rs`** turns a `relation × mutant` **kill matrix** into a minimal,
  *discriminating* suite by greedy set cover — deferring kill-everything
  relations (high recall, no attribution) in favour of specific ones. A mutant no
  relation kills (`uncoverable`) is the discovery signal that a relation or a
  **degree of freedom** is missing.
- **`src/synth.rs`** reads a value object's degrees of freedom off its type,
  finds one the available checks can't see (a `Transaction`'s *multiplicity*),
  and shows the synthesized coverage: a reaching operator (`Split`) plus a
  *dimension-appropriate* check (the residual, not the totals). Reaching + the
  right check = coverage.

## Mutation testing — the engine

Mutation testing certifies the suite and *discovers* missing checks. Wired with
[`cargo-mutants`](https://mutants.rs):

```
cargo mutants        # plant bugs; every survivor is a missing relation/DOF
```

The suite kills **109 of 111** viable mutants; the two survivors are provably
equivalent mutants (a monotonic tie-break in the selector; an always-true
demonstration helper). Several survivors in the first run revealed genuinely weak
laws — e.g. `Cents::negate` could be *deleted* because `negate(negate(c)) == c`
holds for the identity too — which were then closed by stronger properties
(`src/properties.rs`).

## The worked example

`src/ledger/` aggregates a `Transaction` (a multiset of postings) into an
`AccountSummary` — lossy in the *multiplicity* dimension, captured by
`MultiplicityResidual`. `src/linear/` is a lossless transport carrying the
coefficient bug. Run the narrated walk-through:

```
cargo run     # the layered suite + the blind-spot map, narrated
cargo test    # unit + property tests (the probes over the input space)
cargo mutants # the engine
```

## Loss, residuals, and composition

A lossy transition is a lossless one that **discards its residual**. Retaining
the residual (`Carried<M, Retained>`) restores invertibility; discarding it
(`Carried<M, Discarded>`) removes `invert` *at compile time* (the retention
typestate). Loss **composes** as a value object: `Compose`/`Pair` give `g ∘ f`
the residual `Pair<R_f, R_g>`, so invertibility flows through lossy stages while
the accumulated residual is kept — the basis for migration data-validation
(*is the accumulated residual within the acceptable-loss set?*).

**Composition validation is the open frontier** (single-transform is validated
here; a bug living only in `A∘B`'s interaction is the next experiment).

## The capability chain (least power)

The same residual machinery places every module on one chain of increasing
capability and *decreasing* verifiability:

```text
effect  ⊃  state  ⊃  pure-with-loss  ⊃  pure
```

- **pure** — `Out = f(In)`; every probe applies. Max assurance.
- **+loss** — invertible only if the residual is retained (`src/ledger/`).
- **+state** — a state update is a lossy morphism whose residual is the prior it
  overwrote; composing updates accumulates the priors into the undo history
  (`src/journal/`). Replayable, not undoable.
- **+effect** — touches a world it does not own; made pure *relative to a handler*
  by treating the read as input and the write as residual (`src/effect/`). The
  live (I/O) handler is the one seam the method cannot probe — the program edge.

The rule: keep each operator as far RIGHT (as close to pure) as its behaviour
allows. Each `Morphism` declares a `CAPABILITY` ceiling in the grammar, and
**`Compose` joins the stages' ceilings at compile time** — a path's capability is
a *type fact*, so a `const` assertion that a pipeline stays "at most Lossy" is a
BUILD error if any stage is promoted to `Stateful`/`Effectful` (see the const in
`src/capability.rs`). `src/capability.rs` also makes the claim *measurable* — it
perturbs each declared capability *source* and observes whether the
output/residual respond.

An operator carries its capability claim (`Declares`), and the `Audit` reconciles
the claim with the probed behaviour per source, catching BOTH error directions —
each a real cost:

- **over-claiming** (declared but unused) → capability-slop: needless guards,
  over-testing, false barriers to composition. Move right by dropping it.
- **under-claiming** (used but undeclared) → a hidden dependency, a latent bug —
  e.g. a declared-pure operator that secretly reads the world, so a call believed
  deterministic is not.

The audit verifies a *declared* channel (it cannot infer a channel's role from
the type), so it catches taking the wrong amount of power once the channels are
named — the same perturb-and-observe machinery as every other probe.

## Enforcement (build tooling)

`build.rs` parses the source with `syn` and **fails the build** on two tiers.

**Tier 1 — boundary files (`src/<module>/boundary.rs`):** the strict grammar. No
free functions, global `static`s, submodules, traits, public fields, any
`unsafe` / I/O, **or `pub use` re-exports** — a boundary must *define* its
citizens, not forward a child's. (Re-export would let a parent's surface silently
become its whole subtree, destroying "one place to look"; a parent narrows by
defining an operator that delegates inward — see `src/pipeline/`.)

**Tier 2 — module-internal files (e.g. `internal.rs`):** the "workshop", where
mutation and raw collections are fine — but a function may not **return a raw
primitive** (`String`/`&str` or any numeric), because every domain primitive must
be a value object. `bool` is exempt (a predicate is control, not domain data).
Files directly under `src/` (the grammar `boundary.rs`, the tooling `select.rs` /
`synth.rs`, `main.rs`, tests) are exempt — they are crate-level vocabulary, not a
module interior.

## Map of the crate

| Path | Role |
| --- | --- |
| `src/boundary.rs` | the grammar: sealed markers + `Morphism` / `probe` / `commutes` / `coefficient_holds` / `Compose` / retention typestate |
| `src/ledger/` | lossy worked example: aggregation, its residual, and the complementary mutants |
| `src/linear/` | lossless transport: the decisive coefficient bug (`Scale::skew`) |
| `src/journal/` | state as loss: a state overwrite's residual is the prior it forgot |
| `src/effect/` | effect as a pure morphism relative to a handler (read = input, write = residual) |
| `src/pipeline/` | nested module: a parent boundary composing two private child boundaries into one narrowed operator |
| `src/capability.rs` | capability probe: classify a morphism on the chain and flag over-declaration |
| `src/select.rs` | kill-matrix set-cover selection |
| `src/synth.rs` | type-driven DOF coverage / operator synthesis |
| `src/blindspot.rs` | the blind-spot map as tests |
| `src/properties.rs` | the probes + operator laws under `proptest` |
| `build.rs` | two-tier boundary enforcement |
