# probe-algebra

Experiments with constraining bugs by making module boundaries an algebra, and
catching bugs in (especially AI-generated) transformation code by **generating
probe operators from the degrees of freedom of a value object** and checking a
transformation against a *layered* suite.

Two constraints are under study together:

1. **Every primitive that means something in the domain is a value object, and
   every operation on it a value operator.** Money is `Cents` (an amount) and
   `Balance` (a sum); account names are `Account`; a scalar is `Quantity`. The
   strict form of the discipline — *no raw arithmetic at a call site; a raw
   `i64`/`String` appears only inside a value object's own operator or accessor* —
   is held by the domain modules `ledger/` and `linear/` (e.g. `Round` rounds via
   `Balance::split_dollar`, never a bare `%`). The illustrative modules `effect/`
   and `pipeline/` keep arithmetic inside their `Morphism` operator bodies (still
   value operators, the broad principle) rather than minting a value-object
   operator for every cross-type combination — a deliberate, looser bar noted in
   their module docs.
2. **A transformation is checked by a layered probe suite, not a single check** —
   because no single check is highest-assurance (see the blind-spot map below).

This is a `lib` (the algebra + vocabulary) plus a thin `demo` bin.

## The boundary discipline

A module's **`boundary.rs`** is the *only* surface it exposes to other modules,
and it is a **category**: just two things cross the seam — **objects** (the
nouns) and **morphisms** (the verbs).

| Citizen | Role in the category | What it is | Marker (sealed) |
| --- | --- | --- | --- |
| **Value object** | object | immutable, validated, value-equality data | `ValueObject` |
| **Value operator** | morphism | a pure map over value objects — no I/O, no external mutation | `ValueOperator` |
| **Typestate** | object *index* | distinguishes objects (`Entry<Draft>` vs `Entry<Submitted>`) so illegal sequencing fails to compile — not a citizen of its own | `Typestate` |

The morphisms share **one algebra** (residual, `backward`/`reconstruct`, probe,
and a `CAPABILITY` ceiling that composes by static join) in a few
type-distinguished **shapes**:

- **`Morphism`** — a *total* edge between value objects; a *transition* is one
  declared by `transition!`.
- **`Construction`** — the *partial* **entry edge** from a raw primitive *into* a
  value object (`u64 -> Cents` — "parse, don't validate"). The one edge the
  value-object pattern used to leave *outside* the algebra as a native `fn new`;
  modelled as a morphism, it is back inside the probe space.
- **`Branch`** — a total edge into a **coproduct** (`classify`: one input, one of
  two arms), *keeping* the negative witness rather than discarding it as a `None`.
- **`Guarded`** — a *partial* edge admitted by an external **witness** (a
  name-branded proof for the same value); the categorical **sibling** of
  `Construction`, which mints its own. So a guarded transition like `Post` (needs a
  `Cleared` proof) is in the algebra too — every hop of a multistate path is an edge.
  Its output is **brand-coupled** (`type Out<N>`): a `Post`ed entry keeps the brand
  of the submitted entry it came from (`Named::map`), so **provenance flows** through
  the hop — a `Posted` cannot be misattributed to an entry it did not derive from.

The markers are **sealed**, so the set of citizens is *closed*: no module can
invent a new kind, and no external crate can mint one. Non-boundary logic
(algorithms, mutation, primitives) lives in private sibling modules and is
unreachable across the boundary.

## The layered probe suite

A transformation is a `Morphism` `In -> (Out, Residual)`. It is checked by three
layers of increasing cost and decreasing generality — each catches a bug class
the previous one is blind to:

### Layer 1 — type construction (free, always on)
A value object's constructor enforces the structural rules (ranges,
non-emptiness, …). Malformed outputs *cannot be constructed*, so a whole bug
class is gone before any probe runs.

Construction itself is now a **`Construction` morphism** `Raw -> (Refined,
Residual)`, so it joins the probe space instead of sitting outside it as a native
`fn new`. A *pure* refinement (a range check, `ParseCents`) loses nothing, so its
residual is `Unit`; a *normalizing* parse keeps a real residual — `ParseAccount`
keeps the trimmed whitespace, `ParseTransaction` keeps the discarded input
ordering — and **`reconstructs`** (the construction round-trip probe) checks that
residual recovers the exact raw input. A constructor that silently normalizes but
claims a `Unit` residual (`ParseAccountDropsPadding`) is caught, exactly as
`probe` catches an incomplete `Morphism` residual. A construction carries the same
**`CAPABILITY`** ceiling as a `Morphism` (a pure refinement is `Pure`, a
normalizing parse is `Lossy`) and the same perturbation-based completeness probe
(**`construction_probe`**: nudge the normalized-away dimension — the refined value
stays invariant, the residual responds, and the perturbed raw round-trips).
Constructions compose with ordinary morphisms via `Then` (`ParseTransaction` then
`Aggregate` is one edge from `Vec<Posting>` to a summary), which **joins** the
capabilities just as `Compose` does, so the whole primitive-to-output path stays
reconstructible with a computed ceiling — the category closes over construction.

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

A full-crate run kills **298 of 302** viable mutants (a further one is a timeout —
an infinite loop, effectively caught); the three survivors are provably
**equivalent** mutants, which no test can kill: an empty-source declaration
replaced by another empty (`SecretStamp` genuinely declares none), a monotonic
tie-break in the selector (`-` vs `/` induce the same ordering), and an
always-true demonstration helper. Several survivors in the *first* run revealed
genuinely weak laws — e.g. `Cents::negate` could be *deleted* because
`negate(negate(c)) == c` holds for the identity too — which were then closed by
stronger properties (`src/properties.rs`).

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

**Composition validation** (`src/composition.rs`): the first decisive experiment
is done. An A∘B *interaction* bug — recombining a summary with a residual from a
*different* run — is real and silent, and **invisible to per-module probes** (a
probe only ever tests forward-then-backward on a matched pair, so it cannot even
construct the mismatch). The fix is a relational precondition no value object can
express ("these two came from the same run"), discharged at the seam by a GDP
shared-name brand: `aggregate_paired` brands the summary and residual together,
`reconcile` accepts only same-name values, and crossing brands from two runs is a
**compile error** (verified). The honest limit: this holds in-process inside one
`with_seed`; across persistence the brand cannot travel, so there the runtime
`explains` check returns as the fallback. Still open: whether *metamorphic
relations* (not just preconditions) survive composition.

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

## Carried proofs (a GDP spike)

Value objects enforce *single-value* invariants by construction; they cannot
express a *relational* fact like "this particular transaction balances". `gdp.rs`
spikes [Ghosts of Departed Proofs](https://kataskeue.com/gdp.pdf) (mononym's
technique, hand-rolled, no dependency): a unique type-level **name** brands a
value, and a `Proof<Name, Predicate>` — mintable only by a real check — carries a
relational fact across a seam, *tied to that name* so a proof for transaction A
cannot be used with B (verified: the mismatch is a compile error). Module A's
check thus discharges module B's precondition at the type level.

The name uniqueness is sound (it bottoms out in the GhostCell HRTB +
invariant-lifetime trick); the **proof is only as true as its minting check**, so
proofs are never minted from the statistical probes — those stay runtime. This is
the type-level *carrier* for facts the probe or a boundary check has established,
not a replacement for either.

`gdp.rs` also carries two further GDP forms. A **total `classify`**
(`classify_balance -> Balanced | Unbalanced`, both branded) keeps the negative
witness the paper warns against discarding with `Option`. And an **operation
witness** captures a fact an operation otherwise erases: an operation's
having-occurred is invisible in the type system exactly when it is an *endo-map*
(input and output the same type) — `Round: Summary -> Summary`,
`Scale: Quantity -> Quantity` — whereas a type-changing op
(`Calibrate: Sample -> Reading`) witnesses itself. `round_witnessed` / `scale_witnessed`
brand the output with a `Rounded` / `Scaled` proof, so a consumer can *require*
that the operation ran (e.g. `whole_dollars` needs `Rounded`; `report_scaled`
needs `Scaled`). Together with `Morphism::CAPABILITY` this captures *both*
type-invisible facts of an endo-operation: its capability (a static const) and
its occurrence (a witness). For `Scale` the witness even rejects a `skew`-scaled
value at compile time on the honest path — the provenance complement to the
runtime quantitative probe.

## Interfaces are state machines (the typestate lifecycle)

`src/lifecycle/` makes the architectural reading of the whole crate explicit:
**a boundary IS a state machine.** Its states are value-object types, its
transitions are the operators, and the type graph makes an illegal transition
*uncallable*. The `ledger` boundary already works this way over *distinct*
value-object types (`Round: Summary -> Summary` cannot be applied to a
`Transaction`); the phantom typestate `Entry<S>` is the tool for the one case a
structural type change cannot express — **the data shape is invariant but the
permissions change**. (`Entry<Draft>` and `Entry<Submitted>` are the same
`Transaction`; only the legal next-moves differ.)

The graph is deliberately *non-linear*, because linear flows never stress "just
use typestates" — branches, guards, and cycles do:

```text
                     ┌──────────── Amend ◀────────────┐
                     ▼                                 │
  ▶ Draft ─Submit─▶ Submitted ─classify─▶ Cleared ─Post──▶ Posted ─Void─▶ Voided
                         │                                                   │
                         └─classify─▶ Flagged ─Reject─▶ Rejected ──Amend─────┘
```

Four transition **shapes** fall out, and only the first is a plain `Morphism`:

- **Reversible** (`Submit`, `Amend`, `Void`): data invariant, `Unit` residual, so
  `backward` returns to the prior state — phantom transitions that round-trip and
  probe like any morphism. (`Amend` is the *cycle* `Rejected → Draft`; `Void` the
  *reversal* `Posted → Voided`.) These are **lifted into the grammar**: a machine
  implements `StateMachine` once (`EntryFlow`: payload `Transaction`, carrier
  `Entry<S>`), then each reversible edge is one line of the `transition!` macro —
  its name and endpoints. Writing the macro *is* adding the edge; an undeclared
  edge has no operator, so the legal graph stays exactly the set of declarations.
  (A second machine in `src/tests.rs` reuses the same grammar over a different
  carrier, proving it isn't `Entry`-specific.)
- **Branching** (`Validate::classify`): one input, one of several next states. It
  is the GDP total-`classify` lesson *as a transition* — the failure case is a
  `Flagged<N>` state-proof carried in the `Err` arm, not a discarded `None`.
- **Guarded** (`Post`, `Reject`): each needs a name-branded proof for *this* entry
  (`Cleared<N>` / `Flagged<N>`), so you cannot post an unbalanced entry **nor**
  reject a balanced one, and a proof for entry A will not discharge entry B. The
  precondition can't fit `In -> (Out, Residual)`, so the GDP brand is what lifts
  these transitions out of the morphism algebra.

The illegal transitions are pinned **negatively** by a `trybuild` compile-fail
suite (`tests/compile_fail/`) — the typestate analog of a perturbation probe, one
fixture per illegal move: submit twice (order), validate before submit (order),
void a draft (order), forge a clearance (precondition, `E0423`), post with a
`Flagged` proof (typed guard, `E0308`), and post an entry with another's proof
(relational, `E0308`). A green run means each illegal transition is still a
*compile* error, not a runtime slip — so the typestate gives ordering for free,
GDP supplies the relational/guard preconditions, and the residual keeps the
reversible steps probeable: the features built around the `Morphism` reappear
here because a typestate transition *is* a morphism.

## Enforcement (build tooling)

`build.rs` parses the source with `syn` and **fails the build** on two tiers.

**Tier 1 — boundary files (`src/<module>/boundary.rs`):** the strict grammar. No
free functions, global `static`s, submodules, traits, public fields, any
`unsafe` / I/O, **or `pub use` re-exports** — a boundary must *define* its
citizens, not forward a child's. (Re-export would let a parent's surface silently
become its whole subtree, destroying "one place to look"; a parent narrows by
defining an operator that delegates inward — see `src/pipeline/`.) A value
object's field **may not downgrade a value object to a raw primitive**: a
primitive may appear only as the lone field of a newtype wrapper (`Cents(i64)`,
`Account(String)`), never nested in a composite (`BTreeMap<String, _>` is a build
error — use `BTreeMap<Account, _>`). This makes the downgrade that forces
re-parsing impossible.

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
| `src/boundary.rs` | the grammar (a boundary as a **category**): sealed markers + edge shapes `Morphism` / `Construction` (entry edge) / `Branch` (coproduct) / `Guarded` (witnessed) + probes `probe` / `commutes` / `coefficient_holds` / `reconstructs` / `construction_probe` / `Compose` + `Then` composition / retention typestate / `Qty` tagged primitives / `StateMachine`+`transition!` (a boundary as a state graph) / `Meter`+`Profiled` instrumentation seam |
| `src/ledger/` | lossy worked example: aggregation, its residual, and the complementary mutants; **constructions** (`ParseCents`/`ParseAccount`/`ParseTransaction`) bring the smart constructor into the probe space |
| `src/linear/` | lossless transport: the decisive coefficient bug (`Scale::skew`) |
| `src/journal/` | state as loss: a state overwrite's residual is the prior it forgot |
| `src/effect/` | effect as a pure morphism relative to a handler (read = input, write = residual) |
| `src/pipeline/` | nested module: a parent boundary composing two private child boundaries into one narrowed operator |
| `src/money/` + `boundary::Qty` | tagged-primitive substrate: one operator set per *kind* — a full-domain concept costs a tag (near-zero boilerplate), a partitioning one adds only a validity rule |
| `src/capability.rs` | capability probe: classify a morphism on the chain and flag over-declaration |
| `src/gdp.rs` | Ghosts-of-Departed-Proofs spike: unique type-level names carry a relational fact (balance) across a seam; brand-preserving `Named::map` (provenance through an edge) and the `InBounds` lookup coupling (a fresh-named index proven in bounds of its map — check-free indexing) |
| `src/composition.rs` | composition validation: an interaction bug invisible to per-module probes, closed by a GDP shared-name seam contract — output⋈residual coupling for both morphisms (`aggregate_paired`/`reconcile`) and constructions (`parse_paired`/`reconstruct_paired`) |
| `src/lifecycle/` | "interfaces are state machines": a non-linear typestate lifecycle (`Draft/Submitted/Posted/Rejected/Voided`) with reversible, branching, and guarded transitions; illegal moves pinned by `tests/compile_fail/` |
| `src/select.rs` | kill-matrix set-cover selection |
| `src/synth.rs` | type-driven DOF coverage / operator synthesis |
| `src/blindspot.rs` | the blind-spot map as tests |
| `src/properties.rs` | the probes + operator laws under `proptest` |
| `build.rs` | two-tier boundary enforcement |
