# Concepts

The precise model, for readers who want the terminology to check the claims. Nothing here is
load-bearing folklore: every term maps to a trait or function in `src/boundary.rs`.

## The boundary is a graded category

A module's boundary is modelled as a **category**:

- **objects** are *value objects* — every domain primitive wrapped with its validity rule
  (`Int`, `Ident`, `Expr`, …). A bare `i64` is not an object; `Int` is.
- **morphisms** are *edges* — the operations that cross the boundary, in one of four
  type-distinguished shapes (below).

It is a *graded* category: each edge carries **annotations that compose**, and each grading
is a monoid (an identity, an associative combine). This is a discipline *shaped like* a
category, enforced by the type system and `build.rs` — not a machine-checked proof of the
category laws. The value is that the shape lets one set of generic probes apply to every
edge uniformly.

## The four edge shapes

All four share one residual algebra; they differ in where partiality and witnesses sit.

| shape | signature | partial? | witness |
|---|---|---|---|
| `Morphism` | `In -> (Out, Residual)` | total | — keeps a residual so `backward` can invert |
| `Construction` | `Raw -> Option<(Refined, Residual)>` | yes (parse) | mints its own (the parse succeeds) |
| `Branch` | `In<N> -> Result<Left<N>, Right<N>>` | total | mints a kept witness of *which* arm |
| `Guarded` | `(In<N>, Proof<N>) -> Out<N>` | yes | *demands* a witness minted elsewhere |

`Construction` is the entry edge — "parse, don't validate" made probeable: keeping the
`Residual` is what lets the round-trip `reconstruct ∘ parse == id` catch a constructor that
silently normalizes. `Branch` and `Guarded` are siblings: a `Branch` (`Check`) produces the
name-branded proof that a `Guarded` edge (`Eval`) consumes, so "you forgot the precondition"
is a compile error and the brand `N` ties the proof to one specific value.

`Compose` (morphism ∘ morphism) and `Then` (construction ∘ morphism) make a path itself an
edge of the same algebra, so its annotations are computed, not re-stated.

## The gradings

Four annotations ride the edges, each a monoid that `Compose` threads:

- **residual** — *what an edge discards*, at the type level (`Pair` / `Unit`). A `Unit`
  residual marks a lossless edge; a real residual witnesses exactly the collapsed dimension.
  "Discarded residual ⇒ not invertible" is a compile error (`Carried<M, Retained>` vs
  `Discarded`).
- **capability** — *how much power an edge claims*: the lattice
  `pure ⊂ lossy ⊂ stateful ⊂ effectful`, at the type level (`Effect` / `AtMost` / `Join`)
  with a runtime reflection. A ceiling is *demandable* (`run_pure` accepts only `AtMost<Pure>`),
  and the `capability` module *audits* the declaration against behaviour (below).
- **cost** — *time and space complexity*, as an **open keyed map** from named size axes to a
  polynomial degree (`CostCons` / `Lookup` / `WithinBudget`). Time and space diverge at
  iteration (mapping materializes n results; folding streams), so they are two gradings over
  one map. A path over budget on any axis is a compile error.
- **provenance** — *a value's journey*: a type-level lineage (`Stamped` / `Step`) that
  records every edge a value crossed, reflectable to a runtime `Provenance`.

## The probe taxonomy

No single check is highest-assurance — that is the method's central claim, made executable
as a **blind-spot map**. Each flavour catches a bug class another is blind to:

| probe | catches | blind to |
|---|---|---|
| residual round-trip (`reconstructs`, `probe`) | a dropped/incomplete residual | a wrong-but-invertible value |
| metamorphic commutation (`commutes`) | a non-uniform offset | a uniform (e.g. symmetric) wrong constant |
| quantitative coefficient (`coefficient_holds`) | a wrong constant | (reference-bearing; needs a spec) |
| oracle-free relation (`relation_holds`) | a broken value law (`x+0≠x`) | what no stated relation covers |

The **decisive negative result**, re-homed onto the interpreter's constant folder: a folder
that doubles every result keeps a complete residual *and* is symmetric, so it survives both
structural probes — and dies only to the coefficient probe. (`src/tests.rs::blind_spot`.)

Above these sits the **fused universal probe** (`sensitive_to_all`): one operator, derived
from a value object's structure, that is sensitive to structure, value, and (through
recursive perturbation) semantics at once — a map that silently collapses any dimension
fails it. See [how-it-works](how-it-works.md).

## Mutation testing is the meta-test

Probes claim to catch bugs; **mutation testing checks the probes**. `cargo mutants` plants a
bug (negate a condition, swap an operator, return a constant) and asks whether some probe
notices. A surviving mutant is a hole in the specification, not noise — so the suite is
scoped to the algebra surface, and the interpreter's interior is deliberately *kept in
scope with no tests of its own*, to measure exactly how much the boundary buys.

## The inward rule

`build.rs` enforces the discipline at compile time, in two tiers:

- **tier 1** — a module's `boundary.rs` may contain only value objects, typestates, and
  value operators: no free functions, no I/O, no public fields.
- **tier 2** — the module interior may do anything *except* return a raw primitive: every
  result is a value object. (`bool` is exempt — a predicate is control, not domain data.)
  Accessors that unwrap to a primitive live at the boundary, the sanctioned exit hatch.

This is why the interpreter's lexer reads characters through a `Source` value object rather
than a bare `&str`: modelling the substrate, not exempting it.
