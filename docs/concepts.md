# Concepts

The precise model, for readers who want the terminology to check the claims. Every term maps
to a trait or function in `src/boundary.rs`.

## The boundary is a graded category

A module's boundary is modelled as a **category**: **objects** are *value objects* — a domain
primitive wrapped with its validity rule (`Int`, `Ident`, `Expr`); a bare `i64` is not an
object, `Int` is. **Morphisms** are *edges* — the operations that cross the boundary, in one of
four type-distinguished shapes.

It is *graded*: each edge carries **annotations that compose**, each a monoid (identity +
associative combine). This is a discipline *shaped like* a category, enforced by the type
system and `build.rs` — the shape lets one set of generic probes apply to every edge uniformly.

## The four edge shapes

All four share one residual algebra; they differ in where partiality and witnesses sit.

| shape | signature | partial? | witness |
|---|---|---|---|
| `Morphism` | `In -> (Out, Residual)` | total | keeps a residual so `backward` can invert |
| `Construction` | `Raw -> Option<(Refined, Residual)>` | yes (parse) | mints its own (the parse succeeds) |
| `Branch` | `In<N> -> Result<Left<N>, Right<N>>` | total | mints a kept witness of *which* arm |
| `Guarded` | `(In<N>, Proof<N>) -> Out<N>` | yes | *demands* a witness minted elsewhere |

`Construction` is the entry edge — "parse, don't validate" made probeable: keeping the
`Residual` lets the round-trip `reconstruct ∘ parse == id` catch a constructor that silently
normalizes. `Branch` and `Guarded` are siblings: a `Branch` (`Check`) mints the name-branded
proof a `Guarded` edge (`Eval`) consumes, so "you forgot the precondition" is a compile error
and the brand `N` ties the proof to one specific value. `Compose` and `Then` make a path itself
an edge of the same algebra, so its annotations are computed, not re-stated.

## The gradings

Four annotations ride the edges, each a monoid `Compose` threads. The parts that are **bodiless
type tables** are unreachable by mutation (which mutates fn bodies), so each is **proven at
compile time** by `typewit::TypeEq` witnesses — a witness compiles only if its two type
arguments are literally one type, so the section compiling *is* the proof:

- **residual** — *what an edge discards* (`Pair` / `Unit`). A `Unit` residual marks a lossless
  edge; "discarded residual ⇒ not invertible" is a compile error (`Carried<M, Retained>` vs
  `Discarded`). Its monoid is the one proven only **up to isomorphism**: `Pair` is a genuine
  product, so `Pair<Unit, R>` is not *literally* `R` — the unit/associativity laws hold up to a
  canonical iso (`drop_unit_l`/`reassoc_residual` + inverses), checked by proptest. An honest
  finding: strict equality doesn't reach a free product.
- **capability** — *how much power an edge claims*: the lattice `pure ⊂ lossy ⊂ stateful ⊂
  effectful` (`Effect` / `AtMost` / `Join`), demandable (`run_pure` accepts only `AtMost<Pure>`).
  The lattice is sealed at four levels, so its proofs are **exhaustive by cases**: `Join` is a
  commutative, idempotent semilattice with `Pure` as identity (associativity over all 64 triples);
  a `const` assertion pins the type-level table to the runtime `Capability::join`; and `AtMost`
  agrees with `Join` (reflexive, every operand `AtMost` its join) so the ceiling check and the
  composition can't disagree. A declared capability could still *lie*, in two directions, and the
  two are split: **under-claiming** (declare `Pure`, secretly read state) is now **inferred from
  structure** — `InputEffect` reads a STATE FLOOR off the input type (`Bound` carries an `Env` ⇒
  `Stateful`), and `run_pure` rejects an edge whose input grants more than the ceiling whatever its
  annotation says, so the dangerous hidden dependency is a compile error; **over-claiming** (declare
  more than you use) is a *negative* the type system can't express, so it stays the `capability`
  module's behavioural audit (perturb a source, watch the output) — as does `Effectful`/I/O, which
  types can't see at all. Inference and audit are complementary, with a sharp boundary between them.
- **cost** — *time and space complexity*, an **open keyed map** from named size axes to a
  polynomial degree (`CostCons` / `Lookup` / `WithinBudget`); time and space diverge at
  iteration (mapping materializes n results, folding streams). A path over budget is a compile
  error. The degrees are an *open* family, so the laws are proven **total, by structural
  induction**: each is a trait whose `Z` impl is the base case and whose `S<N>` impl lifts `N`'s
  witness through the injective successor (`TypeEq::project`). `Max` (a semilattice with `Z`
  identity) and `AppendCost` (a monoid with `CostNil` identity) are total this way; the
  load-bearing `Lookup`-distributes-over-`AppendCost` law is still spot-checked (its total proof
  needs decidable-equality reflection over `NatEq`), now resting on the total base.
- **provenance** — *a value's journey*: a type-level lineage (`Stamped` / `Step`) reflectable to
  a runtime `Provenance`. Lineages compose by `AppendLineage` (a bodiless cons-concat), proven a
  **total** associative monoid with `Origin` identity (same inductive technique, lifting through
  `Step<E, _>`); a `#[test]` certifies `reflect` is a monoid **homomorphism** into `Provenance`,
  so the type-level path and the value it reflects to can't drift.

So **every grading's laws are certified** — sealed ones exhaustively, open ones inductively,
residual up to iso. This is the second tier of the claim: not only do the probes generate
themselves, the algebra that *shapes* them proves its own laws, and `typewit` joins `rustc` and
`cargo-mutants` in a small, explicit trust root.

## The probe taxonomy

No single check is highest-assurance — the method's central claim, made executable as a
**blind-spot map**. Each flavour catches a bug class another is blind to:

| probe | catches | blind to |
|---|---|---|
| residual round-trip (`reconstructs`, `probe`) | a dropped/incomplete residual | a wrong-but-invertible value |
| metamorphic commutation (`commutes`) | a non-uniform offset | a uniform (symmetric) wrong constant |
| quantitative coefficient (`coefficient_holds`) | a wrong constant | (reference-bearing; needs a spec) |
| oracle-free relation (`relation_holds`) | a broken value law (`x+0≠x`) | what no stated relation covers |

The map is **derived from real mutation data**, not asserted with hand-rolled counterexamples.
`cargo-mutants` plants the bugs; each mutant's log names which probe caught it, giving a real
`[probe × mutant]` kill matrix. `examples/suite_audit` feeds it to `select` and reports the
**minimal attributing suite** — the few probes that, each catching what the others miss, still
kill every killable mutant. That partition *is* the blind-spot map. (Earlier hand-rolled
counterexamples were manual stand-ins for mutants; against the real matrix they added zero kill
power, so they were retired.)

Above these sits the **fused universal probe** (`sensitive_to_all`): one operator, derived from
a value object's structure, sensitive to structure, value, and (through recursive perturbation)
semantics at once — a map that silently collapses any dimension fails it.

## Mutation testing is the meta-test

Probes claim to catch bugs; **mutation checks the probes**. `cargo mutants` plants a bug and
asks whether some probe notices; a survivor is a hole in the spec, not noise. So the interior is
deliberately kept in scope with no tests of its own, to measure exactly what the boundary buys.
The whole-crate sweep is a **green gate** (`0 missed`).

## Self-hosting, and the irreducible base

The method is turned on **every part of its own runtime**, two ways: *structurally* — a module
re-specified as a boundary with an interior carrying no example tests (`interp`, and `select`,
the selector applied to its own kernel); and *by verification* — crate-level grammar `build.rs`
exempts from the structural rules (`gdp`, `capability`), its example tests replaced with
oracle-free property probes and kept in the sweep. The interpreter's entire *positive* surface
is certified by the autogen `harness` with **zero hand-written positive examples**.

What is **irreducible** — by nature, not for lack of effort:

- the **trust root**: `rustc`, `cargo-mutants`, `typewit`. The method is measured by them; it
  cannot certify them.
- the **grammar** (`boundary.rs`): the probe primitives can't be defined in terms of themselves
  without circularity — the host, kept under the mutation lens but not re-specified in itself.
- **rejection tests**: "input X is *rejected*" can't be derived from the thing under test (you
  cannot generate a counterexample to a property from the property). These stay hand-written.
- the **meaning** itself — but it has shrunk. The algebraic laws are not authored, not matched against
  a catalog, and not even arithmetic-specific: a domain implements one trait — `engine::Theory` (its
  sorts, operators, inhabitants, and an OBSERVATION on values) — and the generic engine ENUMERATES
  terms, groups them by behaviour, instantiates the universal algebraic shapes over the operators, and
  keeps the ones that run true, counting the rest as consequences and reporting operators in no law.
  The same engine discovers the interpreter's arithmetic, a non-commutative **router** monoid (routers
  compared *observationally* — by how they route a path grid — so commutativity is correctly omitted),
  and a multi-sorted **date calculus**; the interpreter adds a structural law over a synthetic
  **universal observer `U`** (the faithful rendering): *no two distinct programs look the same to
  `U`*. The whole law set falls out of the operators' behaviour and renders as a readable, non-mathy
  report, then is **frozen** into a committed file per theory (`spec/*.spec`) — the staleness gate
  fails on drift, so the committed file in a PR diff IS the ratification. What an author still supplies
  is the **validity rule** (`Int ≥ 0`) and ratifying that diff — recognizing whether it is the algebra
  they meant (a law you expect but don't see is a bug). Discovery's reference frame is the baseline,
  so it catches *deviations* (mutation) and *surprises* (ratification); it cannot conjure a law the
  operators don't exhibit, and enumeration is depth- and grid-bounded (a resource limit, not a curated
  list). That is the precise edge where "the tests
  write themselves" ends and "what did you mean" begins.

## The inward rule

`build.rs` enforces the discipline at compile time, in two tiers:

- **tier 1** — a module's `boundary.rs` holds only value objects, typestates, and value
  operators: no free functions, no I/O, no submodules, no public fields.
- **tier 2** — the interior may do anything *except* return a raw primitive: every result is a
  value object (`bool` exempt — a predicate is control, not domain data). Accessors that unwrap
  to a primitive live at the boundary, the sanctioned exit hatch.

It also enforces **edge-completeness by enumeration**: every concrete edge impl in the source
must have an `impl Probed`, or the build fails (counterexample fixtures are `#[cfg(test)]`, so
they are skipped). This is why the lexer reads characters through a `Source` value object, not a
bare `&str`: modelling the substrate, not exempting it.
