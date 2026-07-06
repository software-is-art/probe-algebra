# Discovery — the precise model

The reference for the discovery half of the method: what a theory is, how the grid is grown,
what the catalog's data means and how the engine interprets it, what every lock kind pins, and
how the system, world, pipeline, and mutation layers reuse the same move. Every term maps to an
item in `src/discover/`. For the compile-time edge grammar beneath all of this, read
[concepts.md](concepts.md).

## A theory

A domain hands the engine one trait, `engine::Theory`: its **sorts** (value-object kinds), its
**operators** (name, symbol, fixity, typed signature, a possibly-partial evaluator), its
**inhabitants** per sort (the grid seeds), and an **observation** on values. Observation is the
load-bearing choice: grouping terms by observed behaviour IS equality here, which is how a
router with no structural `Eq` is judged (by how it routes a path grid) and how a stateful
store makes expiry visible (its live entries at its own clock).

Three authoring surfaces produce the same impl, in decreasing explicitness:

- `theory!` full form — everything spelled out; used when the observation or grid is a
  deliberate choice worth reading in review.
- `theory!` derived-grid form — no `vars`, no `inhabit`: the grid grows from the value type's
  `Shaped` structure.
- `#[algebra]` on a plain module of operator functions — the whole theory synthesised from
  signatures; multi-sorted (the macro builds the `Value` sum and `sort_of` from the argument
  types).

## The grid

Laws are judged over assignments of inhabitants to variables. Two honesty mechanisms:

- **The shadow algebra** (`shadow_grid`): a boundary whose operators cannot generate values (a
  bare monoid, a lattice with no constant) still gets a grid — grown from the value type's own
  `#[derive(Shaped)]` structure, starting at the canonical inhabitant and closing under
  perturbations. The closure is **partitioned by price**: structural perturbations (which
  constructor shapes exist) are type-level-finite and closed exhaustively FIRST; value
  perturbations spend whatever budget remains. A cap can starve value density, never
  constructor coverage. `grid_gaps` is the library-level audit that the structural closure
  completed.
- **Independent sampling**: a small variable/inhabitant cross-product is enumerated
  exhaustively; a large one is sampled by a coprime-stride mixed-radix decode so no variable's
  value is a function of another's (the over-fitting failure mode a naive stride has).

**Time-indexed values are grids too — the depth-bounded carrier idiom.** `delta-render`'s
`Stream` (a FIXED-DEPTH vector of Z-sets; equality is prefix equality to that depth) worked
as a `Theory` carrier with zero engine changes, and it is the supported way to put histories,
traces, or tick-indexed state on a grid: make the depth a declared constant, make the mint
pad-and-truncate so a wrong-depth value is unconstructible, and hand discovery a handful of
deliberate histories (the impulse, the LATE impulse so delay is visible, the retraction
pair, the ramp) rather than a combinatorial soup. The depth bound is the same concession the
honest frame already declares for grids and term depth — bounded refutation, stated — and it
composes with everything above: the discovered laws (`i` undoes `d`, linearity of all three
stream operators) are exactly the ones the incremental-circuit derivation then leans on.
Keep the grid lean on purpose when a drift gate re-derives the theory inside every
`cargo test`: grid size is an economics decision the theory author owns, not an accident.

## The catalog IS the engine

`ShapeCatalog::inventory()` is the law language — 35 shapes — and `Engine::templates()` is a
generic interpreter over it. There is no second statement of the battery anywhere. Each
`ShapeInfo` stanza carries:

| field | role |
|---|---|
| `name` | the shape tag a `DiscoveredLaw` carries; the identity expectations compare against |
| `schema` | the display equation over placeholders, as `spec/shapes.spec` prints it |
| `gate` / `gate_slots` | applicability as prose AND as checkable slot data (`Binary`/`Constant`/`Unary`/`Action`/`Relation` over sort variables, plus distinctness constraints). `ShapeGate::bind` both admits a binding and returns the sort-variable assignment, so admission and instantiation are one computation — and genesis validates declarations through the same gate, so "what may be declared" and "what discovery tries" cannot drift |
| `template` / `holes` | the prose skeleton and which slot fills which hole (constants render by symbol, operators by name) |
| `lhs` / `rhs` | the canonical terms as `SchemaTerm` data — what discovery actually runs, and the single source of every rendered equation (engine- and genesis-side) |
| `polarity` | `Equal` (`∀: lhs = rhs`) or `Differs` (`∃: lhs ≠ rhs` — a witness shape) |
| `mirrored` | also try the argument-swapped variant under the same prose (identity/annihilation's two-sidedness) |
| `guard` | the one applicability fact only the grid decides (bias fires only on non-commutative operators) |
| `const_rule` | a constraint on the constant slot's symbol (irreflexivity vs self-application split on `false`) |

The driver's emission order is canonical and locked into every spec: the equational band, then
the witness band; within a band, by fire operator (the slot-0 binding), then catalog rank, then
partner bindings in operator order. Genesis's target locks predict exactly this order.

**To add a law shape**: add one stanza to `inventory()`, one row to the declaration vocabulary
in `expect.rs` (a census holds the two in lockstep), regenerate `spec/shapes.spec`, and ratify
the diff. Nothing else — the stanza is executable.

**Witness shapes** are the catalog's inequation half, added because the algebra-mutation
harness proved equations cannot state them: *action nontriviality* (`act(x, p) ≠ x` somewhere)
and *non-constancy* (`rel(x, y) ≠ c` somewhere). `Engine::check` re-probes an equation as
holds-everywhere and a witness law as find-a-witness. Honest frame inverted but intact: a
witness refutes triviality on the grid; it never proves richness.

## Declared expectations and distance

`discover::expect` is the top-down half. An `Expectation` is a catalog shape plus the operator
symbols it ranges over — the same identity a discovered law carries, so declared and discovered
are directly comparable. The vocabulary is deliberately closed to the ratified catalog: an
unknown shape name fails loudly, listing every declarable key.

`Distance::of::<T>()` runs discovery and reports three-ways: **met**, **missing** (declared,
not yet earned — the red/green axis), and **surprises** (discovered, never declared — prompts
to ratify or refute, never failures). The render is the product: one line an agent works from.

## Locks — the complete taxonomy

Every lock is a `spec_lock::Lock { name, path, live }`; `check` is the drift gate, `bless` the
regeneration path, the committed diff the ratification. **A missing lock is stale, never
fresh.**

| lock | pins | regenerate |
|---|---|---|
| `spec/<theory>.spec` | the discovered law set, consequence count, coverage line | `cargo run --example freeze_spec` |
| `spec/<theory>.mutation.spec` | every operator-table mutant's verdict; survivors are ratified degrees of freedom | same |
| `spec/<system>.system.spec` | the module registry + every seam's obligation and verdict | same |
| `spec/<observer>.world.spec` | the model's recorded conduct over the derived trace battery | same |
| `spec/shapes.spec` | the law language itself | `cargo run --example freeze_shapes` |
| `spec/gates.spec` + `.github/workflows/ci.yml` | the pipeline's promises AND its execution | `cargo run --example freeze_gates` |
| `spec/qualify.spec` | which modules are operator-shaped (boundary-hood as a computed property) | `BLESS_QUALIFY=1 cargo build` |

## Systems and seams

`system!` compiles a declaration into a `System`: `modules()` (the registry — this repo's
`all_specs()` reads it, so the graph replaced the hand-maintained list), `seams()`, and
`cohesions()`. Two seam kinds, two checkers:

- **transport** — the algebra must survive the seam unchanged. Checked behaviourally
  (`CoherenceReport::between`: every law one side discovers must hold under the other's
  operators — law-agreement, not operator-equality), or discharged **by construction** with a
  compile-time `fn(V) -> V` witness when both sides share the value type.
- **transform** — a named conversion crosses the seam and must be a homomorphism, checked
  inside a spanning theory (source op, conversion, target op), plus composite laws along
  pipelines (`PipelineLaw`). The verdict is tied to the NAMED conversion, so a healthy
  neighbouring conversion cannot discharge a broken seam.

The macro has two arms: compact (name the modules and seams) and full-grammar (paste the
verbatim genesis declaration — one artifact, two lifecycle stages). `SystemDistance::of::<S>()`
runs every registry module through cohesion and reports declared vs latent modularity — a
report, not a gate, because cohesion is a suggestion.

## Genesis

`genesis` parses a `system!` declaration from tokens (a pure `plan` half and one effectful
`apply` edge, path-confined) and emits a full workspace member: value objects from executable
validity rules (`Rule::Range` generates the predicate, the mint, and the `Shaped` surface),
theory modules with `MEANING:` holes for operator interiors, the compiled `system!` (the
declaration spliced verbatim), distance gates, seam verdict tests, and **target locks** —
rendered through the same catalog terms and templates the engine uses, so a confirming
discovery reproduces them byte for byte (a dynamic sync test holds every shape to the freeze's
actual render). Declared expectations are `discover::expect::Expectation` values directly;
genesis has no parallel vocabulary.

## The world lock

`discover::world` points the freeze discipline outward. A dependency is modelled as command
values and traces; the pure model (`StoreModel`) plays the role observation plays everywhere
else, so the EXISTING engine discovers protocol laws with no new machinery — and those laws
are the operational guarantees (`idempotent` = retry safety, `bias_later` = last-write-wins,
`identity(empty)` = an empty batch is harmless). `WorldReport` records conduct over a battery
derived from the command type's own structure and freezes it; the conformance gate replays the
same battery against the real dependency and names the exact divergent trace. Against a real
remote, the replay is the one deliberately-Effectful gate, run behind a bless flag.

## The pipeline

`discover::gates::GateRegistry` declares every gate: name, promise, exact argv, **cadence**
(every change / per PR diff / default-branch drift since the certified tree / weekly sharded
sweep), and capability. `spec/gates.spec` locks the promises; `ci.yml` is rendered from the
registry and drift-gated, so the pipeline validates itself inside the very `cargo test` it
runs; `cargo run --example gate` executes the every-change gates locally from the same
declaration.

The mutation economics live here as data: PRs mutate changed lines; default-branch pushes
mutate `git diff mutants-green...HEAD` and advance the tag on green (the countersignature —
the pipeline's one effect); the weekly clock re-certifies from scratch, sharded
`FULL_SWEEP_SHARDS` ways; every mutation gate runs per-mutant suites through nextest's
fail-fast runner. Timeouts are detections (`mutants-gate.sh` distinguishes a timeout-only exit
from a real survivor).

## Algebra-level mutation

`discover::mutation` is mutation testing where a mutant is a **value**, not a build: perturb
the operator table (confusion — one operator evaluates as another of prefix-compatible
signature; projection — a binary returns an argument unchanged; partiality — an operator goes
undefined), re-run discovery, and judge: **killed iff the named-law set changes** — lock-drift
semantics applied to a hypothetical implementation. Milliseconds per mutant; the verdict runs
in every `cargo test` and freezes into `spec/<theory>.mutation.spec`.

A survivor means the ratified law language cannot tell the mutant from the real thing on this
grid — a named degree of freedom, not a missing test. The intended response is a catalog
stanza, not a hand-written probe: the founding precedent is the bias-blindness finding (both
merge directions satisfy identical monoid laws → the bias shapes), and the witness shapes
repeated it (four survivors → two stanzas → all four closed by discovery itself, zero
per-theory work). Behaviourally-identical replacements are prefiltered as
equivalent-by-construction. Source-level mutation keeps judging the plumbing the algebra can't
see (parsers, emitters, renderers).

**The consumer economics, measured.** A consumer's theory core needs no source-mutation job
at all: freeze the mutation verdict beside the spec (`MutationReport::of::<T>().lock_in(dir)`
— [`downstream-fixture`](../downstream-fixture) mints and drift-gates
`spec/credit-meter.mutation.spec` this way, through public API, inside an ordinary
`cargo test`), and point cargo-mutants at the plumbing only. The substitution is
evidence-backed, not asserted: a file-scoped sweep over this repo's own five theory operator
files — 90 source mutants, every comparison flipped and every arithmetic operator swapped —
reports **zero missed** (51 caught, 39 unviable), killed by the freshness gates and law pins
alone; those files carry no other tests. Honest boundary: that measurement is this repo's. A
consumer whose operators do work the grid cannot see should run the same file-scoped sweep
once (`cargo mutants -f src/my_theory.rs`) as their own evidence before leaving source
mutation off their core.

## Honest frame

Everything above refutes; nothing proves. Grids and batteries are bounded samples; term
enumeration is depth-bounded; "discovered" means "the bounded grid could not refute it";
"killed" is definite but "survived" is only "indistinguishable here". Reports suggest; locks
gate. Keep the distinction when extending any of this.

## Metric and setoid domains: judging at a declared tolerance

Law judgment compares observations for equality, which assumes the carrier HAS a decidable
equality worth judging. Constructive reals (Cauchy data), float-valued numerics, and other
metric/setoid carriers do not — their honest observable is ε-closeness at a stated
precision. The sanctioned route today is the OBSERVATION HOOK: `observe` maps the carrier
to `Obs`, and every law is judged on `Obs`, so a metric domain declares its working
precision by quantizing there (round to the registered number of digits, or map to a
fixed-point bucket). The tolerance is thereby in the ratified theory declaration — code
review sees it next to the operators — never ambient.

**Now built, the second arm:** [`Theory::judge`] returns a three-valued
[`Verdict`] — holds / refuted / UNDECIDED — and [`Theory::tolerance`] registers the bars
as display text. A candidate law with any undecided assignment (and no refutation) is
neither certified nor refuted: it lands in the lock's DISCLOSED band
(`# undecided at the declared tolerance …`), under a header that carries the registered
bars — so review ratifies ε along with the laws, and a frozen law that drifts into the
band re-checks as a named error, never a silent pass. Scope, disclosed: judgment only —
enumeration and the consequence count keep exact equality, because a toleranced relation
is not transitive and cannot key the term-collision maps.

The quantization route below remains valid for carriers that can keep values off bucket
boundaries; state the hazard before adopting it: **quantized equality is not
ε-closeness.**
Two values within δ ≪ ε of each other can straddle a bucket boundary and compare unequal,
so near the boundary a TRUE law can be refuted by roundoff, and the lock would record the
lie. The quantized route is only honest when the grid keeps values away from bucket
boundaries by construction (residuals small against the bucket width) — a per-domain
obligation the theory author owns and should write down beside the grid. A corpus that
registers its numerics does this per claim: fixed working precision, explicit
classification bars, and a disclosed UNDECIDED band between "holds" and "refuted" rather
than a silent bin. That three-valued judgment — holds / refuted / undecided-at-ε, with ε
part of the lock text so review ratifies the tolerance along with the law — is a roadmap
candidate; without the undecided band, a toleranced grid gate is a coin flip exactly where
metric domains live.
