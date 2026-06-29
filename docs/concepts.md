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
  `Discarded`). Its monoid is the one proven **up to isomorphism**, not strict equality: `Pair`
  is a genuine product, so `Pair<Unit, R>` is not *literally* `R` — the unit and associativity
  laws hold up to a canonical iso (`drop_unit_l`/`reassoc_residual` and their inverses), checked
  by the `residual_iso_laws` proptest. An honest finding: the strict-equality technique that
  certifies capability and cost does not reach a free product.
- **capability** — *how much power an edge claims*: the lattice
  `pure ⊂ lossy ⊂ stateful ⊂ effectful`, at the type level (`Effect` / `AtMost` / `Join`)
  with a runtime reflection. A ceiling is *demandable* (`run_pure` accepts only `AtMost<Pure>`),
  and the `capability` module *audits* the declaration against behaviour (below). The `Join`
  table has no runtime body, so mutation can't reach it — instead it is **proven at compile
  time**: `typewit::TypeEq` witnesses certify the algebraic laws (commutativity, identity,
  idempotence, and associativity over all 64 triples), and a `const` assertion certifies the
  type-level table reflects to the same `Capability` the runtime `Capability::join` computes —
  so the two definitions cannot drift. A mistyped cell, or a type↔runtime disagreement, fails
  to build. It also proves the order and the join AGREE — `AtMost` is reflexive and every
  operand is `AtMost` its `Join` (the lub law) — so a `run_pure` ceiling check and a `Compose`
  ceiling computation cannot disagree about what "at most" means.
- **cost** — *time and space complexity*, as an **open keyed map** from named size axes to a
  polynomial degree (`CostCons` / `Lookup` / `WithinBudget`). Time and space diverge at
  iteration (mapping materializes n results; folding streams), so they are two gradings over
  one map. A path over budget on any axis is a compile error. Like `Join`, the map's machinery
  is bodiless type functions the mutation sweep cannot reach, so it too is **proven at compile
  time** — and over the *open* set of Peano degrees these proofs are **total, by structural
  induction**, not a finite sample: each law is a trait whose `Z` impl is the base case and whose
  `S<N>` impl lifts `N`'s witness through the injective successor (`TypeEq::project`), so the
  generic impl bodies prove the law for *every* degree. `Max` is a commutative, associative,
  idempotent semilattice with `Z` as identity; `AppendCost` is an associative monoid with
  `CostNil` as identity — both total. The load-bearing law — `Lookup` distributes over
  `AppendCost` as `Max` (so sequential composition can be plain append, the per-axis max
  recovered at lookup) — is still spot-checked: its total proof needs decidable-equality
  reflection (case-splitting on `NatEq`), and now rests on the total `Max`/`AppendCost` base.
- **provenance** — *a value's journey*: a type-level lineage (`Stamped` / `Step`) that
  records every edge a value crossed, reflectable to a runtime `Provenance`. Lineages compose
  by `AppendLineage` (a bodiless cons-concat), proven a **total** associative monoid with
  `Origin` as identity (the same inductive technique, lifting through `Step<E, _>`); a `#[test]`
  certifies `reflect` is a monoid **homomorphism** into the runtime `Provenance`, so the
  type-level path and the value it reflects to cannot drift.

So **every grading's laws are certified** — the part the mutation gate structurally cannot reach
(a bodiless type table) is closed by the type system instead. Capability's lattice is sealed at
four levels, so its `TypeEq` proofs are *exhaustive by cases*; the cost and provenance algebras
range over *open* type families, so their core laws are *total by induction*; residual, a free
product, is a monoid only *up to isomorphism* (proptest). This is the second tier of the method's
central claim: not only do the probes that validate the interior generate themselves — the algebra
that *shapes* them proves its own laws, for every inhabitant, and `typewit` joins `rustc` and
`cargo-mutants` as a (tiny) trust root.

## The probe taxonomy

No single check is highest-assurance — that is the method's central claim, made executable
as a **blind-spot map**. Each flavour catches a bug class another is blind to:

| probe | catches | blind to |
|---|---|---|
| residual round-trip (`reconstructs`, `probe`) | a dropped/incomplete residual | a wrong-but-invertible value |
| metamorphic commutation (`commutes`) | a non-uniform offset | a uniform (e.g. symmetric) wrong constant |
| quantitative coefficient (`coefficient_holds`) | a wrong constant | (reference-bearing; needs a spec) |
| oracle-free relation (`relation_holds`) | a broken value law (`x+0≠x`) | what no stated relation covers |

The blind-spot map is **derived from real mutation data**, not asserted with hand-rolled
counterexamples. `cargo-mutants` plants the bugs; each mutant's per-mutant log names which
probe(s) caught it, giving a real `[probe × mutant]` kill matrix. `examples/suite_audit` feeds
that matrix to `select` and reports the **minimal attributing suite** — the few probes that, each
catching what the others miss, still kill every killable mutant. That partition *is* the
blind-spot map, in real data. (The earlier hand-rolled counterexamples — a doubling folder, a
residual-forgetful folder — were manual stand-ins for mutants; against the real kill matrix they
added zero kill power, so they were retired.)

Above these sits the **fused universal probe** (`sensitive_to_all`): one operator, derived
from a value object's structure, that is sensitive to structure, value, and (through
recursive perturbation) semantics at once — a map that silently collapses any dimension
fails it. See [how-it-works](how-it-works.md).

## Mutation testing is the meta-test

Probes claim to catch bugs; **mutation testing checks the probes**. `cargo mutants` plants a
bug (negate a condition, swap an operator, return a constant) and asks whether some probe
notices. A surviving mutant is a hole in the specification, not noise — so the interpreter's
interior is deliberately *kept in scope with no tests of its own*, to measure exactly how much
the boundary buys. The whole-crate sweep is a **green gate** (`0 missed`), wired into CI to run
per-PR over the changed lines and as a full sweep on a schedule.

## Self-hosting, and the irreducible base

The method is turned on **every part of its own runtime**. There are two grades of self-host:

- **structural** — a module is re-specified as a boundary (value objects + edges) with a
  private interior that carries *no example tests*: `interp` and `select` (the kill-matrix
  selector that chooses the probes — the method applied to its own kernel).
- **by verification** — crate-level grammar that `build.rs` exempts from the structural rules
  (`gdp`, `capability`) has its example tests replaced with oracle-free property probes and is
  kept in the mutation sweep.

The interpreter's entire *positive* surface — evaluation, parsing, type-checker acceptance —
is certified by the autogen `harness` (declared laws + structure-derived probes) with **zero
hand-written positive examples**: the strongest form of "the tests generate themselves."

What is **irreducible** — what *cannot* be self-hosted, by nature, not for lack of effort:

- the **trust root**: `rustc`, `cargo-mutants`, and `typewit` (whose `TypeEq` witnesses certify
  the grading laws mutation cannot reach). The method is measured by them; it cannot certify them
  (the trusting-trust limit).
- the **grammar** (`boundary.rs`): the probe primitives cannot be defined in terms of
  themselves without circularity. It is the host, kept under the mutation lens but not
  re-specified in itself.
- **rejection tests**: "input X is *rejected*" cannot be derived from the thing under test — you
  cannot generate a counterexample to a property from the property. These stay hand-written, in
  `tests.rs`. (The blind-spot map is no longer in this category: it is *derived* from the real
  mutation kill matrix — `cargo-mutants` plants the bugs, `suite_audit` reads which probe caught
  which — so the hand-rolled counterexamples that used to stand in for mutants were retired.)

So `sensitive_to_all` and the laws make a *weak specification* hard to express; they do not
make *wrong meaning* impossible — the meaning (a validity rule, a declared law) is the one
thing an author still writes, and it is the smallest irreducible input.

## The inward rule

`build.rs` enforces the discipline at compile time, in two tiers:

- **tier 1** — a module's `boundary.rs` may contain only value objects, typestates, and
  value operators: no free functions, no I/O, no public fields.
- **tier 2** — the module interior may do anything *except* return a raw primitive: every
  result is a value object. (`bool` is exempt — a predicate is control, not domain data.)
  Accessors that unwrap to a primitive live at the boundary, the sanctioned exit hatch.

This is why the interpreter's lexer reads characters through a `Source` value object rather
than a bare `&str`: modelling the substrate, not exempting it.
