# Roadmap — the next brick

The state as of this writing: the loop is closed at BOTH levels.

**Module level** (both directions): bottom-up, a theory's algebra is discovered, frozen
(`spec/*.spec`), and drift-gated; top-down, a theory declares its laws (`expects { … }` on
`theory!` / `#[algebra]`) and `Distance::of::<T>()` reports exactly what is unmet.

**System level** (the brick just laid): the SEAM GRAPH is a declared, locked, compiled
artifact. `discover::system` is the compiled twin of the grammar `genesis` parses — a
`system!` declaration compiles into a `System` marker whose `modules()` IS the spec registry
(`all_specs()` reads the committed `BoundarySpec` declaration; the graph replaced the
hand-maintained list) and whose seams wire to the existing checkers: transport →
`CoherenceReport::between` (or a by-construction discharge with a compile-time `fn(V) -> V`
witness), transform → the named conversion's discovered homomorphism law plus
`PipelineLaw` composites inside a spanning theory — run, never assumed. The graph freezes
into `spec/<system>.system.spec` under the same spec-lock discipline, so ratification is
hierarchical: interior changes touch no lock, module law changes touch one module lock, only
a re-drawn seam (or a flipped verdict) touches the system lock.

**The flagship demo** (`genesis-demo/`, a CI-tested workspace member): the committed sample
declaration (`examples/genesis_demo.rs`) was genesis'd from blank slate, ONLY its
`MEANING:` holes were filled, and it converged to green at both levels — the expectations
gate (module distances met), the seam obligation (discharged by construction, witnessed at
compile time), and all locks fresh (two module locks, the system lock, the qualify census).
The ratification diff is readable in its `spec/`: the three declared laws came back with
seven discovered surprises (associativity, both distributivities, renew's identity and
annihilation with zero), exactly as the workflow intends. That is the "large application
from scratch" story in miniature, driven by a declaration that fits in one context window.

Also landed with the brick, the flagged unifications: genesis's parse-side `Expect` enum is
gone (a parsed expectation IS a `discover::expect::Expectation`, whose `ops` generalised to
owned strings); genesis's target-lock prose comes from `ShapeCatalog`'s own templates
(`ShapeInfo::instantiate` — one source, no restatement); `shadow_grid` is re-exported from
`boundary`, where its consumers live.

**The partitioned grid** (a small brick laid right after): the perturbation surface is now
split by price. `Shaped::structural_perturbations` (derived: variant swaps, threaded up
through fields) is the type-level-finite half, and `shadow_grid` closes over it EXHAUSTIVELY
before spending any remaining budget on value neighbours — so a cap can starve value
density, never constructor coverage (for recursive types the honest frame returns: phase 1
is exhaustive up to the cap, the bound term enumeration already lives with). `grid_gaps` —
the audit that a grid's structural closure completed — was promoted from the test harness to
the library (`boundary::grid_gaps`), so downstream grids hold it as an invariant instead of
a convention.

**The pipeline is a lock** (the shift-left brick): CI/CD is subsumed into the declaration
discipline. `discover::gates` declares every gate (command, cadence, capability, promise);
`spec/gates.spec` locks the inventory and `.github/workflows/ci.yml` is ITSELF a derived,
drift-gated lock (`cargo run --example freeze_gates` regenerates; never hand-edit), so the
pipeline can no longer drift the way it did (the `--workspace` gap is now a census test
over registry DATA). `cargo run --example gate` executes the every-change gates locally
from the same declaration — CI stops being where verification is defined and keeps only
what cannot shift left: countersigning, effects, and the economics of the expensive
sweeps, all three visible as cadence/capability data.

**The sweep economics** (the brick after that): mutation cost became declared data. The
weekly full sweep is SHARDED (`FULL_SWEEP_SHARDS` matrix jobs — ~1,300 mutants is ~5½
serial hours, past a hosted runner's patience, as the sweep on PR #26's merge proved by
dying at 5h16m); every mutation gate runs its per-mutant suites through nextest's
fail-fast runner (`test_tool = "nextest"`, a census test holds the install lists to it);
and the default branch went INCREMENTAL — a green sweep is a lock over the tree at that
sha (the `mutants-green` tag), so a merge mutates only the diff since it and advances the
tag as CI's countersignature, with the weekly sharded sweep re-certifying from scratch to
backstop the one gap (a test edit weakening kills for unchanged code).

**Mutation at the algebra level** (`discover::mutation`, the research brick): for anything
that is a THEORY, a mutant does not have to be a build — it is a VALUE, a perturbed
operator table (confusion, projection, partiality), judged by re-running discovery: killed
iff the named-law set changes, the freshness gate's own lock-drift semantics applied to a
hypothetical implementation. Milliseconds per mutant, so the whole verdict lives in `cargo
test` on every change. It earned its keep on first contact: four survivors across the five
registry theories, all of one deep kind — equational laws cannot state INEQUATIONS. The
trivial action (`add`/`tick` returning the carrier unchanged satisfies every action law),
the never-true relation (`<` pinned to constant-false satisfies irreflexivity), and the
unpinned operator (`diff` as constant-zero appears in no named law). Each is ratified in
`spec/<theory>.mutation.spec` under the standard lock discipline — the bias-blindness hunt,
industrialised. Source-level mutation keeps the plumbing (genesis, architect, renderers);
the theory-carrying core is now judged in-process.

**Witness shapes** (the brick that closed the loop): the catalog gained its INEQUATION half.
Laws now carry a [`Polarity`] — the classic shapes state `∀: lhs = rhs`, the two new WITNESS
shapes state `∃: lhs ≠ rhs`: "action nontriviality" (`act(x, p) ≠ x` somewhere — the action
actually acts) and "non-constancy" (`rel(x, y) ≠ c` somewhere — the relation escapes the
constant). Discovery emits them in a second pass (equations first, witnesses last), `check`
re-probes them as exists-a-witness, both are declarable (`nontrivial(tick)`,
`not_constantly("<", false)`), and genesis derives their target-lock lines from the same
catalog terms. The payoff proved the "autogenerate the fixes" conjecture: no per-theory work
was done — the vocabulary was extended, discovery found the closing inequations itself on the
next freeze (`add(s, p) ≠ s`, `tick(s, p) ≠ s`, `(x < y) ≠ false`, `diff(s, t) ≠ zero`), and
all four algebra-mutation survivors died, pinned by a zero-survivors census. The flagship
demo's `billing` module told the same story unprompted: `charge` sat in "operators in no law"
— the spec's own silence line — and the witness pass replaced that silence with
`(x charge x) ≠ x`. Honest frame inverted but intact: a witness refutes triviality on the
grid; it never proves richness.

**The catalog IS the engine** (the de-restatement brick): the last code/data duality in the
kernel is gone. `Engine::templates()` was the battery stated as CODE while
`ShapeCatalog::inventory()` stated it as DATA, census-guarded to move together; now
`templates()` is a ~150-line generic interpreter over the inventory — a shape's `gate_slots`
decide where it fires (and bind its sort variables, via `ShapeGate::bind`, so admission and
instantiation are one computation), its canonical `SchemaTerm`s are what discovery runs, its
`template`/`holes` render the prose, its `polarity` is the judgment, and the three residues
that were only code are now fields (`mirrored` for two-sided identity/annihilation, `guard`
for bias's skipped-when-commutative, `const_rule` for irreflexivity vs self-application).
Adding a shape is adding a STANZA OF DATA; `spec/shapes.spec` is executable the moment it
lands. Genesis's fire model simplified with it (fire is uniformly slot 0 — the round-trip
special case dissolved). The byte pins earned their keep on the way: they caught a real
sort-variable inversion in round-trip's slot descriptors that the display census could not
see. The canonical emission order (band → fire op → catalog rank → partner order) moved
exactly two committed lock lines in the whole workspace — both pure reorders, ratified — and
the demos did not change at all. Net −109 lines.

**Specs as licenses** (the `delta-render` brick, from a thrown design): discovery output
consumed as GENERATION input — a new polarity for the whole method. The crate is a miniature
of DBSP-style incremental computation in which each operator's derivation rule is LICENSED by
the presence of laws in its frozen spec, never by a declared boolean: linear ⇔ the additive
homomorphism plus the zero fixed point; bilinear ⇔ additive in each slot (the catalog gained
three data stanzas for the vocabulary — `inverse`, `fixed point`, `distributivity (right)` —
and the maximal logic theory promptly discovered the two classical Boolean complements the
catalog had been silent about). The chain of derived artifacts: eight exhaustively-judged
theories freeze their specs (including `scale`, the non-commutative bilinear whose license
IS the discovered distributivity pair — both slot laws, no commutativity shortcut);
`spec/licenses.spec` is derived by PARSING those specs' text, each row citing the law lines
that granted it; the render walks a validated circuit DAG (unlicensed operators are
unconstructible) and emits generated Rust — compiled, tested, drift-gated: the `ci.yml`
move at a new altitude — plus a plain-language derivation artifact, from ONE derivation,
for TWO circuits (the single-source demo chain, and an audit circuit with two sources and
a fan-out). The end gate —
`I ∘ Q^Δ ∘ D = Q` over the stream grid — trusts no license, and the fire drill proves both
gates can fire: the almost-linear operator (drops retractions) is DENIED over the honest grid
and, when forged via a pruned insert-only grid, caught by the end law instead; a forged
`distinct → linear` registry fires the gate; the all-fallback floor passes it; and
`spec/min.retraction.spec` freezes WHY min under deletion is hard, with values computed by
the real operators. Two incidental findings while building it: the new `fixed point` stanza
was the first equational shape with no variable on either side, exposing (and closing) a
vacuous-truth hole — the `meaningful` filter now guards both polarities; and a per-package
`opt-level` override collapsed the drift-gate economics from ~95s to ~6s because `Engine<T>`
monomorphizes into the consumer crate. Deliberately absent from v1, recorded as candidates
below: recursion/fixpoint circuits, SQL/NULL semantics (the end law's batch oracle is the
declared seam), and phase 7's property-constrained interpretation sampling (since BUILT —
candidate 6).

## The next brick: candidates

The authoring-experience frictions found while converging the demo — and the I/O research
program (effects as theories, the world lock) — are inventoried with their bricks in
[experience.md](experience.md) — all four pillars now carry BUILT status there, each with
its recorded residuals. The items below predate that inventory:

1. **Transform seams end to end in genesis — DONE.** A declared
   `a -- b : transform on V via h;` (the conversion is a regular declared unary operator, so
   its interior is an ordinary meaning hole) now generates: the seam's SPANNING theory in
   `src/ops.rs` (source op, conversion, target op, `expects(homomorphism(h, from, to))`),
   the compiled seam in `src/system.rs`, a distance gate in `tests/expectations.rs`, a
   verdict test in `tests/seams.rs` (replacing the hole), and the PRESERVED stanza in the
   system target lock. A via-less transform keeps the old hole, with the fix named. The
   flagship story now covers both edge kinds, and `relay-demo/` is the CI-tested proof:
   generated from `examples/genesis_relay.rs`, ONLY its three operator interiors and the
   probes stub were filled (the structured validity rules generated every value-object
   artifact), and it converged green at both levels — with the system lock fresh on the
   FIRST freeze, the preserved transform stanza matching its target byte for byte.

2. **The system-level distance — DONE.** `System::cohesions()` (macro-generated for both
   `system!` forms) runs every registry module through the cohesion analysis, and
   `SystemDistance::of::<S>()` renders the verdict in the distance voice — a REPORT, not a
   gate, because cohesion is a suggestion. It earns its keep immediately: run on this
   repo's own graph it names three declared modules that are secretly several (the
   interpreter splits arithmetic from comparison, the calendar splits duration arithmetic
   from epoch conversion, the TTL store splits the merge monoid from the clock action),
   each with its suggested seam kind — byte-pinned as a deliberate keep-whole decision.
   Follow-up: genesis could emit a distance report example per generated crate.

3. **Equation render unification — DONE.** Every shape's canonical equation is now catalog
   DATA: `ShapeInfo` carries schematic lhs/rhs terms (`SchemaTerm`, over slot indices and
   sort-variable ordinals) plus the placeholder symbols its display `schema` string renders
   with, and `ShapeInfo::equation` renders a concrete equation exactly the way the engine
   renders a discovered law's terms. Genesis's `law()` derives its equation there — the
   16-arm hand-written `format!` match is gone — and a census test holds the displayed
   schema string to the same term data, so `spec/shapes.spec`'s schemas, genesis's
   target-lock equations, and the engine's render conventions cannot drift apart. The
   dynamic sync pin (`the_target_lock_reproduces_discovery_byte_for_byte`) remains the
   end-to-end net over the freeze's actual render.

## From the field, day zero (fire-drill consumed in production)

The production CLI adoption consumed fire-drill untagged, same-day — and the crate's
motivating claim ("a rubber stamp passes every positive test") was validated in their
system THE SAME MORNING, by an incident that happened before they'd read it: two committed
regression fixtures both carried the degenerate case (a zero-valued pool), so a 95-test
green suite could not see that two implementations of one identity disagreed — it surfaced
when the first real non-degenerate job hit the validator. The vacuousness failure class
exactly, one level down: not a gate that stopped firing, a fixture set that never pressed
the button. Their six-drill battery froze both incident shapes as permanent drills, register
spec-locked. Three findings folded back in: the CENSUS is the sleeper feature (the drills
catch rot; `requires` + UNPROVEN prevents gates being born rotten — now leading the README),
the strict outcome mapping is where consumers quietly cheat (`Fired` only when the verdict
NAMES the planted defect; harness errors panic — now a documented example),
and `Battery::drill_with` makes declaration order be execution order in one expression.
Their line for the chronicle: "the loop is closed and it is short" — seven days from idea to
a production consumer, a foreign-domain theory lock, the pattern reinvented locally in a
client repo, and an incident-extracted module consumed back by the system whose report
motivated it.

## From the field, third assessment (a registered-numerics research corpus)

A ~1021-artifact corpus attacking a mathematical conjecture — 250 kernel-checked proofs
(cubical Agda `--safe`, Lean), ~700 Julia/Python numerics whose entire warrant is a
hand-rolled procedural discipline (pre-registered gates, sha-pinned inputs, cross-artifact
string-identity, acceptance by independent rerun) — assessed the method as a feasibility
test. Its verdict, kept verbatim in the chronicle because it is the honest positioning of
the whole discipline: **the method strengthens the WARRANT, not the theorems.** For
kernel-checked stones, discovery is strictly weaker than what already stands; for the
~700 numerics stones there is no kernel underneath — the gate/rerun discipline IS the
warrant — which makes fire-drill's question ("can every one of these
positive-tested-only gates still fire?") the sharpest available upgrade to the corpus's
credibility, and registered-numerics pipelines possibly fire-drill's highest-leverage
domain. Three mechanism gaps became bricks: the CROSS-LOCK (spec-lock's check-only,
sha-pinned anchor to a foreign ratified baseline — their most load-bearing integrity
move, previously hand-rolled per stone with the chain topology living in prose), the
ordered-relation stanzas (subadditivity, triangle, monotonicity — the ∀-inequality
polarity, stated as equations over a declared order), and the toleranced-judgment
candidate above. Their planned adoption: a fire-drill battery over the standing guard
classes, and spec-lock over the corpus-level censuses — including a verbatim-copy
provenance registry, which is the exception-register pattern almost exactly.

## From the field (the first production adoption)

A production Rust CLI adopted `spec-lock` as a pinned git dependency — four agent sessions in
one day, five substrates (a command surface, checklist data, coverage manifests, a
reconciliation policy, an accepted-findings baseline), every re-bless in the same commit as
its cause, the discipline transferred from the README alone. Its field notes seeded two
candidate bricks:

4. **Negative-fixture batteries — mutation testing for PIPELINES — BUILT** (`fire-drill/`).
   The witness-inequation idea (`∃`: prove the gate ACTS) at the process level, seeded by
   three self-attestation failures the adoption hit in one system (a coverage manifest
   asserted by the session that produced it, a "pass" stamp byte-identical whether the work
   happened or not, a reconciliation that would have passed on zero items). The extraction is
   spec-lock-shaped: zero dependencies, substrate-free (`Battery`/`Drill`/`Outcome` — the
   adopter runs their gate over their known-bad fixture however they run it and hands over
   the outcome), two failure modes each named in its own vocabulary (VACUOUS: the gate passed
   a planted bad fixture; UNPROVEN: a required gate carries no fixture at all — the census
   half, so a gate cannot join a pipeline without proof it can fail), and a deterministic
   `render()` for spec-lock composition so removing a drill is a reviewed diff. Dogfooded in
   `tests/fire_drill.rs`: the repo's own battery plants a tampered lock, a missing lock, a
   lawless theory, an over-claiming declaration, a mis-sorted shape binding, and an
   unratified vocabulary word — six known-bad fixtures across five of the discipline's gates,
   every one fired on. Honest frame inherited: a drill refutes vacuousness for ITS fixture
   only; the battery proves the alarm rings when pressed, never that it hears everything.

5. **The exception-register pattern, first-class — BUILT** (`spec_lock::Register`, with
   candidate 10 as its first consumer). Exactly the resisted-to shape: a register is
   HAND-AUTHORED, never generated — writing a key IS the ratification, the justification
   is the one thing no derivation can produce, and the format enforces it (a bare key or
   an empty justification is a parse refusal, not an entry). The tooling only reads and
   diffs: `Register::check` renders drift as SET DIFFERENCE — "1 new finding(s) — ratify
   with a justification or fix", "1 resolved — delete the line(s) (a stale exception is a
   lie)" — never a byte diff. A missing register is honestly the empty register (the one
   place missing-is-not-stale: a declaration's absence declares "no exceptions"; there is
   nothing to regenerate). First consumer: `lean/bites.register`, the statement-bite
   survivors baseline.

6. **Uninterpreted operators with ratified properties — BUILT** (`delta-render/src/warrant.rs`,
   `spec/enrich.warrant.spec`). The opaque symbol `enrich` — no inventoried
   implementation, standing for the open half of a real pipeline's inventory — holds a
   linear license on SAMPLED evidence: the circuit law (`i(enrich(d(s))) = enrich(s)`)
   judged over interpretations sampled under the declared properties, deterministically
   (splitmix64, fixed seed), so the warrant is a derivation, not a dice roll. The REMOVAL
   drill keeps the property list honest: drop one property, re-sample under the rest (each
   counter-sample violates exactly the dropped one), demand refutation — and a property
   whose removal refutes nothing is DECORATION, flagged in the artifact and never ratified.
   The demonstration plants one on purpose (`bounded-fanout`), so the flag's polarity is
   exercised on every regeneration; `tests/gates.rs` registers each arm as a fire-drill
   gate. One subtlety earned its own disclosure: full additivity implies zero preservation
   (a cancellation pair reaches the basepoint from non-empty inputs), so the ratified
   properties are declared as INDEPENDENT constraints — additivity away from the
   basepoint, zero-preservation as the basepoint — mirroring how the license classifier
   already reads them as two separate laws. Recorded residual: the warrant is not yet a
   circuit admission (no warranted opaque node in `Registry`/`circuit`) — kept out until a
   real open-inventory consumer forces its shape.

7. **Depth-bounded stream carriers as a grid idiom — DONE** (documented in
   [discovery.md](discovery.md)'s grid section): `delta-render`'s `Stream` (fixed-depth
   vector, prefix equality) worked as a Theory carrier with zero engine changes — recorded
   as the supported way to put time-indexed values on a grid (declared depth constant,
   pad-and-truncate mint, deliberate histories over combinatorial soup), alongside the grid
   and term-depth bounds the honest frame already declares.

8. **Toleranced three-valued judgment — BUILT.** `Theory::judge` returns
   holds / refuted / UNDECIDED and `Theory::tolerance` registers the bars; an undecided
   candidate is disclosed in the lock under the registered-ε header instead of being
   coin-flipped at the boundary, and a frozen law drifting into the band re-checks as a
   named error. The demonstration is integer averaging: commutativity certified exactly,
   its ±1-noise associativity landing in the band, pinned. Scope disclosed: judgment
   only — enumeration keeps exact equality (a toleranced relation is not transitive).

9. **Conditional / guarded laws — BUILT.** `ShapeInfo` carries `premise:
   Option<SchemaTerm>`, judged against the shape's constant slot: an assignment counts
   only where the premise fires, and a law whose premise never fires is VACUOUS, not
   true — the fixed-point lesson, guarded. Three stanzas prove the family: guarded
   monotonicity (`∀ x ≤ y: f(x) ≤ f(y)`), transitivity, and antisymmetry (whose
   conclusion is carrier equality) — and the vacuity rule earned its census on day one:
   `le` is antisymmetric, `less-than` is NOT (its mutual premise is satisfiable nowhere,
   and the pin holds the silence). Frozen guarded laws re-check with the same semantics
   plus a "lost its ground" arm, so a mutant that empties a premise is named, never
   passed. The guard rides `DiscoveredLaw`, renders as `premise = true ⟹ lhs = rhs`
   through the one equation source, and `transitive` — the word two tests used as the
   canonical UNKNOWN shape — became declarable, which the census caught in both places.

10. **Statement-bite mutation — BUILT, with our own corpus as the consumer**
    (`discover::bite`, `lean/ProbeBool.lean`, `lean/bites.register`, weekly gate
    `.github/statement-bite.sh`). The proof-corpus consumer it was waiting for turned
    out to be us: `lean/ProbeBool.lean` formalises the bridged Boolean fragment, making
    it the REAL upstream prover behind `spec/bridged-bool.export` — six of the bridge's
    conjectures (both De Morgan duals among them) were proved there and ratified into
    `proved:` lines, so conjecture supply → upstream proof → certificate is now one
    executed loop (agreements 4 → 10, obligations 21 → 15). The bites: flip one result
    literal in the DEFINITIONS region (never a pattern — a syntax wound is not a
    statement bite; never a proof) and demand the theorems fail to re-check under the
    Lean kernel. A survivor is the vacuous-statement finding the kernel cannot make
    about itself, ratified by key in `lean/bites.register` — candidate 5's register,
    consumed. One survivor set is PLANTED (`bnand`: defined, deliberately untheoremed,
    four surviving bites) so the survivor arm is exercised on every run. Honesty split:
    the Lean kernel (weekly CI gate; the countersign now needs it) is the substrate
    authority, while a Rust MIRROR of the ten theorems re-judges every bite inside
    every `cargo test`, pinning the expected survivor set toolchain-free — plus
    cell-for-cell corpus-tables ↔ export-tables agreement and a `proved:` ↔
    `-- certifies:` bijection, so the Lean file, the export, and the register cannot
    drift apart silently. Remaining out-of-repo half, disclosed: richer per-language
    mutant families (quantifier scope, hypothesis deletion) beyond result-literal
    flips.

11. **Theory-bridge — BUILT** (`discover::bridge`, `spec/bridged-bool.export` →
    `.spec` / `.mutation.spec` / `.obligations.spec`). `Theory` from exported data: a
    prover emits finite operator tables (a table IS an eval function) in a small line
    format whose every malformed line is a named, fire-drilled refusal;
    `Export::install` mounts it in a compile-time slot and `Bridged<SLOT>` is a full
    citizen of the apparatus — spec lock, algebra-mutation verdict, distance — with
    zero new engine machinery (the slot table is the price of keeping `Operator.eval`
    a plain fn pointer). The prover's certificates ride along as `proved:` lines in
    the `expects` vocabulary, and `Triage` re-reads the distance with the prover's
    epistemics: agreements cross-check the pipeline, conjectures are proof
    obligations (the demonstration Boolean fragment yields 21, both De Morgan duals
    among them, from four certificates). The carrier is judged EXHAUSTIVELY (v1 caps
    exports at 8 elements / 8 operators to keep that true), so a refutation is a
    fact, not a sample; absence of a conjecture remains no evidence, and agreement
    never certifies.

12. **The disagreement detector — BUILT** (falls out of 11, and did): `Triage::certify`
    fails — by law name, with the certainty prose — whenever an upstream-proved law is
    refuted over the exhaustive carrier: a defect SOMEWHERE in the export/bridge
    pipeline, unconditionally (differential testing for the untrusted half of a proof
    corpus, same polarity as delta-render's end gate). A disagreement never renders in
    the obligations artifact and the freeze path refuses to freeze it — it is a defect
    to fix upstream, never a row to ratify; the drill export (`proved: commutative
    implies`) pins the conviction end to end.

13. **Second-domain validation: layout engines under metamorphic probe** (scoped; builds
    DOWNSTREAM, in the diagram tool's repo, pinning this library — the second production
    adoption, not a workspace member). The domain: diagram source → layout engine →
    geometry, with an agent loop editing the source. The operators live on SOURCE
    (add node, add edge, rename, reorder declarations, group into container, toggle
    theme); the OBSERVATION is canonicalized geometry — which makes the proposed
    metamorphic relations mostly existing catalog vocabulary, not new machinery:

    - *rename-then-render = render-then-relabel* is the `action_equivariance` stanza,
      verbatim: `f(act(x, p)) = act'(f(x), p)` with `f` = render.
    - *declaration-reorder invariance* is not even a new law — with geometry as the
      observation, a stable engine makes `reorder` OBSERVATIONALLY IDENTITY, and
      discovery finds (or refuses) that on its own. Declare it (`expects`) and the
      DISTANCE REPORT becomes an engine scorecard: dagre expected red on exactly this
      row (the known agent-loop "layout jumping" pathology, named as a missing law),
      ELK expected green. Refutation as product insight.
    - *insertion locality* (a new node must not move geometrically distant nodes beyond
      ε) is a TOLERANCED law — the three-valued judgment built for the trace-logic
      corpus, pointed at pixels: register the metric bars, and near-threshold layouts
      land in the DISCLOSED band instead of coin-flipping (float coordinates are the
      quantization hazard discovery.md already documents). Likely ONE new catalog
      stanza (toleranced action locality, premise-guarded by distance) — the intended
      growth dynamic: hostile domain → stanza → every theory benefits.
    - *theme toggle is geometry-invariant* is an action fixed point on the projected
      observation.

    The rest of the apparatus maps one-to-one: the layout engine is an EXTERNAL
    dependency, so its conduct over a derived source battery freezes into a WORLD
    lock — replay after an engine upgrade names exactly which fixtures moved (upgrade
    drift, currently discovered by squinting at diagrams); known-unstable fixtures are
    exception-register entries with justifications; and a fire drill plants a
    deliberately unstable engine stub (seeded jitter) to prove the stability gates can
    fire. Economics, disclosed up front: an observation is a process spawn (~50–200ms),
    so the grid must be DELIBERATE (the stream-idiom lesson — a handful of designed
    fixtures, not a soup), renders memoized, and the gate probably PR-cadence rather
    than per-`cargo test`. Scope fence, agreed with the field assessment that prompted
    this entry: the lint layer (contrast thresholds, spacing-grid conformance,
    alignment) is plain predicates over rendered output — no algebra earns rent there,
    and wrapping it here would be the internal-consistency trap. What this domain
    uniquely buys the METHOD: millisecond feedback with visible ground truth — the
    fastest refutation loop any consumer has offered yet. Open questions for the
    build session: engine order (ELK first, exhaust it before anything proprietary;
    Graphviz as the free DAG baseline), observation canonicalization (quantized
    positions vs topology-only, possibly BOTH as two observers over one carrier), and
    whether locality needs its own stanza or premise-guarded monotonicity over a
    declared distance order suffices.

## Standing follow-ups

- **Tag `v0.1.0`** — consumers currently pin by bare rev; a tag makes downstream manifests
  read as versioned intent. No crates.io needed for this half-step.
- **Publish** when ready: `docs/publishing.md` has the dependency-ordered sequence;
  all four names (`boundary-spec`, `boundary-spec-macros`, `spec-lock`,
  `boundary-enforce`) were verified unclaimed on crates.io on 2026-07-02.
- **Morphism downstream**: the fixture exercises Construction/Branch/Guarded;
  the fourth edge shape is honestly unexercised downstream.
- **MSRV**: unpinned; verify and add `rust-version` after first publish.
