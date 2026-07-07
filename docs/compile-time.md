# The compile-time half — the edge grammar

How the method's compile-time floor works, layered: each section opens plainly and folds its
precise model behind a click. (This page merges the former `concepts.md` — the model — and
`how-it-works.md` — the mechanism; nothing was dropped, the duplicated passages were unified.)
Every term maps to a trait or function in `src/boundary.rs`. The discovery half — theories,
locks, systems, genesis, the pipeline, algebra mutation — is [discovery.md](discovery.md);
the front door is the [README](../README.md); the gentle walk is [the tour](tour.md).

---

## 1. What you write — the one rigid place

A module's boundary is the only rigid thing you author: each domain primitive wrapped with
its validity rule (a bare `i64` is not a value object; `Int` with its `n >= 0` rule is), and
each operation that crosses the boundary declared as one of four typed *edge shapes*. The
body of an edge calls the ordinary interior code; the shape is the contract.

```rust,ignore
refined! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Shaped)]
    pub struct Int(i64);
    fn new(n: i64) = (n >= 0).then_some(n);   // the predicate is the only content
}

impl Guarded for Eval {
    type Capability = Pure;
    type Proof<N>  = WellTyped<N>;          // uncallable without Check's witness
    type Out<N>    = Named<N, Value>;
    fn guard<N>(&self, e: &Named<N, Expr>, _: &WellTyped<N>) -> Named<N, Value> {
        e.map(super::internal::eval)         // the interior, untyped-by-the-grammar
    }
}
```

That is the rigidity, in one file. Nothing below the boundary is constrained in style.

<details>
<summary><b>Full density: the graded category and the four edge shapes</b></summary>

A module's boundary is modelled as a **category**: **objects** are *value objects* — a
domain primitive wrapped with its validity rule (`Int`, `Ident`, `Expr`). **Morphisms** are
*edges* — the operations that cross the boundary, in one of four type-distinguished shapes.
It is *graded*: each edge carries **annotations that compose**, each a monoid (identity +
associative combine). This is a discipline *shaped like* a category, enforced by the type
system and `build.rs` — the shape lets one set of generic probes apply to every edge
uniformly.

All four shapes share one residual algebra; they differ in where partiality and witnesses
sit:

| shape | signature | partial? | witness |
|---|---|---|---|
| `Morphism` | `In -> (Out, Residual)` | total | keeps a residual so `backward` can invert |
| `Construction` | `Raw -> Option<(Refined, Residual)>` | yes (parse) | mints its own (the parse succeeds) |
| `Branch` | `In<N> -> Result<Left<N>, Right<N>>` | total | mints a kept witness of *which* arm |
| `Guarded` | `(In<N>, Proof<N>) -> Out<N>` | yes | *demands* a witness minted elsewhere |

`Construction` is the entry edge — "parse, don't validate" made probeable: keeping the
`Residual` lets the round-trip `reconstruct ∘ parse == id` catch a constructor that silently
normalizes. `Branch` and `Guarded` are siblings: a `Branch` (`Check`) mints the name-branded
proof a `Guarded` edge (`Eval`) consumes, so "you forgot the precondition" is a compile
error and the brand `N` ties the proof to one specific value. `Compose` and `Then` make a
path itself an edge of the same algebra, so its annotations are computed, not re-stated.

</details>

---

## 2. What compile time validates

Claims that can be statically false become build errors — running an unchecked program,
exceeding an effect ceiling or a cost budget, shipping a probe that covers only some of a
type's degrees of freedom. Each rejection is pinned by a `tests/compile_fail` fixture that
must *fail to compile*, so the rejections themselves are regression-tested. The annotation
algebras additionally prove their own laws at compile time — the part mutation testing can
never reach, certified anyway.

<details>
<summary><b>Full density: the rejections, and the gradings' self-proofs</b></summary>

The pinned rejections:

- **"well-typed programs don't go wrong"** — `Eval` demands a `WellTyped<N>` for the *same*
  brand; an unchecked, ill-typed, or wrong-program proof won't type-check
  (`eval_wrong_program`);
- **effect ceilings** — `run_pure` accepts only `AtMost<Pure>`; the `Lossy` folder is
  refused (`run_pure_rejects_lossy`);
- **cost budgets** — mapping an O(n) edge per element is O(n²); demanding it stay linear is
  a type error (`cost_over_budget`);
- **probe completeness** — a probe covering only some degrees of freedom cannot satisfy
  `require_complete` (`incomplete_probe_rejected`).

Four annotations ride the edges, each a monoid `Compose` threads. The parts that are
**bodiless type tables** are unreachable by mutation (which mutates fn bodies), so each is
**proven at compile time** by `typewit::TypeEq` witnesses — a witness compiles only if its
two type arguments are literally one type, so the section compiling *is* the proof:

- **residual** — *what an edge discards* (`Pair` / `Unit`). A `Unit` residual marks a
  lossless edge; "discarded residual ⇒ not invertible" is a compile error
  (`Carried<M, Retained>` vs `Discarded`). Its monoid is the one proven only **up to
  isomorphism**: `Pair` is a genuine product, so `Pair<Unit, R>` is not *literally* `R` —
  the unit/associativity laws hold up to a canonical iso (`drop_unit_l`/`reassoc_residual`
  + inverses), checked by proptest. An honest finding: strict equality doesn't reach a free
  product.
- **capability** — *how much power an edge claims*: the lattice `pure ⊂ lossy ⊂ stateful ⊂
  effectful` (`Effect` / `AtMost` / `Join`), demandable (`run_pure` accepts only
  `AtMost<Pure>`). The lattice is sealed at four levels, so its proofs are **exhaustive by
  cases**: `Join` is a commutative, idempotent semilattice with `Pure` as identity
  (associativity over all 64 triples); a `const` assertion pins the type-level table to the
  runtime `Capability::join`; and `AtMost` agrees with `Join` (reflexive, every operand
  `AtMost` its join) so the ceiling check and the composition can't disagree.
- **cost** — *time and space complexity*, an **open keyed map** from named size axes to a
  polynomial degree (`CostCons` / `Lookup` / `WithinBudget`); time and space diverge at
  iteration (mapping materializes n results, folding streams). A path over budget is a
  compile error. The degrees are an *open* family, so the laws are proven **total, by
  structural induction**: each is a trait whose `Z` impl is the base case and whose `S<N>`
  impl lifts `N`'s witness through the injective successor (`TypeEq::project`). `Max` (a
  semilattice with `Z` identity) and `AppendCost` (a monoid with `CostNil` identity) are
  total this way; the load-bearing `Lookup`-distributes-over-`AppendCost` law is still
  spot-checked (its total proof needs decidable-equality reflection over `NatEq`), resting
  on the total base.
- **provenance** — *a value's journey*: a type-level lineage (`Stamped` / `Step`)
  reflectable to a runtime `Provenance`. Lineages compose by `AppendLineage` (a bodiless
  cons-concat), proven a **total** associative monoid with `Origin` identity (same
  inductive technique, lifting through `Step<E, _>`); a `#[test]` certifies `reflect` is a
  monoid **homomorphism** into `Provenance`, so the type-level path and the value it
  reflects to can't drift.

So **every grading's laws are certified** — sealed ones exhaustively, open ones
inductively, residual up to iso. Not only do the probes generate themselves, the algebra
that *shapes* them proves its own laws, and `typewit` joins `rustc` and `cargo-mutants` in
a small, explicit trust root.

</details>

---

## 3. What autotest time validates

Everything the type system can't see — does the code compute the *right values*? — is
checked by probes, and almost none of them are hand-written: generators and perturbations
derive from the value types' own structure, each edge registered once gets a whole generic
suite, and the value laws are *discovered* by the engine (the [discovery
half](discovery.md)) rather than declared. Different probe flavours exist because no single
check is highest-assurance — each catches a bug class another is blind to, and that
blind-spot map is derived from real mutation data, not asserted.

<details>
<summary><b>Full density: the derived probes, discovery at the boundary, and the probe taxonomy</b></summary>

- **generators, DOFs, and the complete probe are derived** by `#[derive(Shaped)]`:
  `inhabitant` (a seed), `perturbation_classes` (a neighbour-group per dimension),
  `HasDofs` (the DOF set).
- the **fused probe** `sensitive_to_all(map, x)` runs a map against every derived
  perturbation class: a faithful map (`render`) responds to all; a collapsing map
  (`node_count`) is caught. One operator, sensitive to structure, value, and (through
  recursive perturbation) semantics at once.
- the **autogen harness** registers each edge once and gets its whole suite generically —
  the parse round-trip, the capability/residual laws, `ConstFold`'s fold-preserves-value
  law, `Resolve`'s two-route law, and `eval_semantics_are_probed`, which pins every
  evaluator arm oracle-free against an independent computation — so evaluation needs *no
  hand-written examples* (the disclosed exceptions — the grammar-combinator exercises and
  one witness-threading demo — pin the plumbing, not the arithmetic).
- **value laws are discovered, not declared**: a domain implements
  `discover::engine::Theory` (sorts, operators, a grid of inhabitants, an OBSERVATION on
  values) and the engine enumerates terms, groups them by behaviour on the grid,
  instantiates the catalog's shapes, and keeps what runs true — reporting operators in no
  law (**where the spec is silent**). The same engine discovers the interpreter's
  arithmetic, a non-commutative **router** monoid (routers compared *observationally* — by
  how they route a path grid — so commutativity is correctly omitted), and a multi-sorted
  **date calculus** with a partial operator. The interpreter adds one **structural** law
  over a synthetic **universal observer `U`** (the faithful rendering): `eval` collapses
  structure, so *no two distinct programs look the same to `U`* closes the blind spot the
  equations leave. Specs freeze into `spec/*.spec`; the committed diff is the ratification.
  (The generic `Relation` runner with its **non-vacuity guard** remains for metamorphic
  relations that only sometimes apply; `relation_laws` fails if a probe fired zero times.)
- **`cargo mutants` judges all of it** — the interior carries no tests; the sweep reports
  how many of its mutants the boundary kills.

The probe taxonomy, as a blind-spot map:

| probe | catches | blind to |
|---|---|---|
| residual round-trip (`reconstructs`, `probe`) | a dropped/incomplete residual | a wrong-but-invertible value |
| metamorphic commutation (`commutes`) | a non-uniform offset | a uniform (symmetric) wrong constant |
| quantitative coefficient (`coefficient_holds`) | a wrong constant | (reference-bearing; needs a spec) |
| oracle-free relation (`relation_holds`) | a broken value law (`x+0≠x`) | what no stated relation covers |

The map is **derived from real mutation data**: `cargo-mutants` plants the bugs; each
mutant's log names which probe caught it, giving a real `[probe × mutant]` kill matrix.
`examples/suite_audit` feeds it to `select` and reports the **minimal attributing suite** —
the few probes that, each catching what the others miss, still kill every killable mutant.
That partition *is* the blind-spot map. (Earlier hand-rolled counterexamples were manual
stand-ins for mutants; against the real matrix they added zero kill power, so they were
retired.)

</details>

---

## 4. Why a weak probe is impossible, not merely caught

The degrees of freedom a probe must cover are *derived* from the type, the complete probe
is *generated* over them, and a hand-written partial probe is rejected at compile time —
so "someone forgot to test a field" is unrepresentable rather than merely detectable. The
same closure applies to the edge set: `build.rs` enumerates every concrete edge in the
source and fails the build if one carries no probe.

<details>
<summary><b>Full density: the derivation chain</b></summary>

1. `#[derive(Shaped)]` emits `HasDofs` with one `Field<T, I>` per variant/field — the DOF
   set is **derived**, no marker to forget;
2. `Complete<T>` covers every `Field<T, I>` by a blanket impl — the complete probe is
   **generated**, so `assert_complete::<T>()` holds by construction;
3. the fused probe perturbs along every derived class — so the *runtime* probe is complete
   too.

The only thing left hand-writable is a *partial* `Covers` set, which `require_complete`
rejects at compile time. For the **edge set**: `assert_all_probed::<Edges>()` makes "every
listed edge carries a probe" a compile-time bound, and `build.rs` closes the open-world
residue an in-language check can't — it **enumerates every concrete edge impl in the
source** and rejects any without an `impl Probed` (a build error). A counterexample fixture
is `#[cfg(test)]`, so it is skipped: it is not a spec edge. The type system can't enumerate
impls; the build step can, so edge-completeness is total.

</details>

---

## 5. The interior is free — and that freedom is audited

Below the boundary lives ordinary imperative Rust — `HashMap`s, mutable cursors, recursion
— with **zero tests of its own**. Its correctness is a consequence of the boundary,
*measured* by the mutation sweep: every viable mutant dies. And the interior can't lie
about how much power it uses: hiding a state dependency behind a `Pure` claim is a compile
error (the dependency is visible in the input type), while claiming more than you use is
caught by a behavioural audit.

<details>
<summary><b>Full density: under-claiming vs over-claiming</b></summary>

**Under-claiming** (declare `Pure`, secretly read state) — the dangerous case — is caught
**structurally**: the capability's state floor is inferred from the input type (`Bound`
carries an `Env` ⇒ `InputEffect = Stateful`), so `run_pure` rejects an edge whose input
grants more than the ceiling, whatever its annotation says
(`tests/compile_fail/under_claimed_capability`). **Over-claiming** (declare `Stateful`,
ignore state → slop) is a *negative* the type system can't express — "this body does *not*
read state" — so it stays the `capability` module's behavioural audit: perturb a declared
source, watch whether the output moves. `Effectful`/I/O stays the audit's too, invisible to
types. Inference handles what flows through a type; the audit handles what doesn't.

</details>

---

## 6. The inward rule

`build.rs` enforces the discipline's shape: a boundary file holds only value objects,
typestates, and value operators; the interior may do anything *except* leak a raw primitive
outward. Structure is checked at build time, so drifting out of the discipline is a red
build, not a review catch.

<details>
<summary><b>Full density: the two tiers</b></summary>

- **tier 1** — a module's `boundary.rs` holds only value objects, typestates, and value
  operators: no free functions, no I/O, no submodules, no public fields.
- **tier 2** — the interior may do anything *except* return a raw primitive: every result
  is a value object (`bool` exempt — a predicate is control, not domain data). Accessors
  that unwrap to a primitive live at the boundary, the sanctioned exit hatch.

This is why the lexer reads characters through a `Source` value object, not a bare `&str`:
modelling the substrate, not exempting it. Boundary-hood itself is *computed* — the qualify
census (`spec/qualify.spec`, drift-gated) reports which modules are operator-shaped
regardless of what they are named.

</details>

---

## 7. Self-hosting, and the irreducible base

The discipline runs on every part of its own runtime — some modules re-specified as
boundaries with untested interiors, others verified with oracle-free property probes kept
under the mutation lens. What remains irreducible is short and explicit: the trust root
(`rustc`, `cargo-mutants`, `typewit`), the grammar itself, hand-written rejection tests,
and the meaning — the validity rules and the human act of ratifying each diff.

<details>
<summary><b>Full density: the self-hosts, what it unlocks, and the precise edge of "irreducible"</b></summary>

The method is turned on its own runtime two ways: *structurally* — a module re-specified as
a boundary with an interior carrying no example tests (`interp`, and `select`, the
kill-matrix selector applied to its own kernel); and *by verification* — crate-level
grammar exemptions (`gdp`, `capability`) whose example tests were replaced with oracle-free
property probes (`permute ∘ unpermute == id`, the liveness rule, the audit's
claim/behaviour reconciliation), kept in the sweep. `gdp` is load-bearing: `select` reads
its matrix through gdp's `InBounds` proof, which *holds the matrix's borrow* — a proof
minted for one matrix cannot read another, the same proof-carrying that makes `Eval`
uncallable without `Check`'s witness, lifted to a relation between two values. (An honest
redesign lives here: a proof keyed only on a phantom brand was not value-unique — a region
brands many values — so the proofs now carry the borrow; the brand carries provenance, the
borrow carries identity.)

Once the abstraction certifies itself:

- **consumers stop paying for mutation** — it validates the abstraction once; a module
  built on the discipline inherits the guarantee and runs only the cheap derived probes per
  PR;
- **a new verified module is nearly free** — declaring value objects and laws is the whole
  authoring cost; the harness mints the probes;
- **the selector closes the loop on real data** (`cargo run --example suite_audit`): it
  reads `cargo-mutants`' own output into a real kill matrix and runs `select` over it — the
  minimal attributing suite, and survivors as *missing-relation* signals — the method
  optimizing its own tests.

What is **irreducible** — by nature, not for lack of effort:

- the **trust root**: `rustc`, `cargo-mutants`, `typewit`. The method is measured by them;
  it cannot certify them.
- the **grammar** (`boundary.rs`): the probe primitives can't be defined in terms of
  themselves without circularity — the host, kept under the mutation lens but not
  re-specified in itself.
- **rejection tests**: "input X is *rejected*" can't be derived from the thing under test
  (you cannot generate a counterexample to a property from the property). These stay
  hand-written.
- the **meaning** itself — but it has shrunk. The laws fall out of the operators' behaviour
  via discovery (see §3 and [discovery.md](discovery.md)); what an author still supplies is
  the **validity rule** (`Int ≥ 0`) and ratifying each spec diff — recognizing whether it
  is the algebra they meant (a law you expect but don't see is a bug). Discovery's
  reference frame is the baseline, so it catches *deviations* (mutation) and *surprises*
  (ratification); it cannot conjure a law the operators don't exhibit, and enumeration is
  depth- and grid-bounded. That is the precise edge where "the tests write themselves" ends
  and "what did you mean" begins.

</details>
