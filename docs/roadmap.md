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

## From the field, fifth report (genesis as a greenfield contract generator)

Delivered as a screenshot of another agent's session (a transpiler workitem —
different org, different assistant; the project stays anonymous here, per house
practice for field reports): genesis was pointed at a real declaration and
driven to a compiling generated crate. The verdict — "usable as a greenfield contract
generator, not yet as a clean drop-in"; "the system lock is exactly the kind of small
public-contract review surface we wanted"; "do not vendor the generated crate
wholesale — adapt the generated system.rs/locks pattern, and treat the generator gaps
as follow-up improvements to probe-algebra." Those gaps, verbatim, with dispositions:

1. *Generates a full crate layout, not an existing-module patch.* The migration-mode
   gap — genesis as a PATCHER that emits theory impls, locks, and gates INTO an
   existing tree instead of scaffolding a new one. The largest and realest item.
2. *Value ownership by "first module that mentions it" is surprising.* Make ownership
   declarative (an `owns` clause) and make the inference a stated, locked fact rather
   than a silent heuristic.
3. *Module names can collide with generated theory/value markers.* A generator
   collision must be a NAMED REFUSAL at generation time, never a downstream rustc
   error — the parser-as-gate discipline applied to genesis's own output.
4. *`MEANING:` holes make it scaffolding, not migration.* Fold into (1): migration
   mode should ADOPT an existing function as a meaning (`= path::to::fn`) instead of
   emitting a hole where an implementation already lives.
5. *The algebra is only meaningful with good contract values* (their `u8` stage
   weights "proved mechanics, not [the domain's] semantics"). Partly documentation
   (the carrier IS the contract), partly a possible instrument: a toy-carrier
   advisory in the distance report when a theory's values are bare primitives.

Converging ask from this repo's own operator, same week: genesis should also speak
PROTOCOL — the `protocol!` states-as-sorts form (built; see `discover::protocol`)
as a genesis declaration block, so a blank-slate system can declare a workflow's
states and transitions and get the sort enums, the tagged values, the closure grid,
and the target locks generated with meaning-holes only in the transition bodies.
Together these are the genesis v2 work list.

First strike off that list — **the pipeline gap, closed** (a sixth friction the report
never named because every adopter assumes it: genesis emitted every artifact EXCEPT the
CI that guards them, leaving the adopter to hand-write the one artifact class this repo
argues must never be hand-written). `discover::gates::Pipeline` is the consumer form —
gates declared as data, both locks derived into the consumer's repo via `locks_in`
(the `Spec::lock_in` shape), the two bespoke tiers (green-tag countersign, sharded
sweep) refusing by name rather than rendering wrong YAML — and genesis now emits the
whole loop into every generated crate: `src/gates.rs` (a `Ci::pipeline()` typestate
calling the ONE starter declaration, never a restatement), `examples/freeze_gates.rs`,
`tests/gates.rs`, and the two artifacts pre-rendered so the pipeline locks are fresh
from birth (the declaration fully determines the render — no target-vs-earned gap).
Both demos carry it; the emitted workflow is inert inside this workspace (disclosed in
their manifests) and exactly what the fifth report's standalone adopter needed. A
pleasing verification note: the first draft emitted `pub fn pipeline()` and the demos'
own generated enforcement shim REFUSED it (a loose public function violates the
no-rats-nest rule) — the generated discipline judging its own generator.

## From the field, sixth report (the battery attests its own completeness) — BUILT

From the production adoption (the project stays anonymous, per house practice), written
against `912a476`, after the earlier asks — `drill_with`, the strict `observe` mapping
docs, the action-law shapes — all landed. The gap, in their sentence: "`Battery::requires`
+ UNPROVEN is a census of gates someone remembered to list — so the battery attests its
own completeness, which is the self-attestation shape fire-drill exists to kill, one
level up." How it bit: their surface grew to 21 verdict-bearing commands in four days;
four had no drill, no UNPROVEN entry (never listed), and no recorded reason. Every test
was green.

The ask, built as specified in `fire_drill::Census`: the battery reconciled against a
consumer-derived surface enumeration (a clap tree, a route table — the same walk a
surface lock freezes; the crate never knows what a CLI is), with exemption as a
first-class frozen object (`CensusEntry::Drilled` names covering gates,
`CensusEntry::Exempt` carries a ratified reason). `verdict()` fails by name on their
four refusals — UNREGISTERED, STALE, UNKNOWN-GATE, EMPTY-REASON — plus two added in the
same spirit and disclosed here: EMPTY-CLAIM (a Drilled entry citing no gates is an
exemption in drill's clothing) and DUPLICATE (one entry per element, the register
discipline). `render()` is deterministic and spec-lockable, problems shown loudly.
One deliberate deviation from their design point 3: gate names are validated against
the battery's in-memory required list rather than the frozen spec text — the battery's
own freshness gate holds those equal, so the frozen cross-reference comes one hop
through an existing lock instead of a second parser. Their field observation is now the
type's docs: writing the exemption reason is the review, and the drill is often less
work than the excuse (their register turned four would-be shadow gates into drills the
day it landed). fire-drill also gained its own weekly member sweep
(`mutation (fire-drill plumbing)`) — the crate had none, a pre-existing gap this growth
made worth closing.

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

13. **Second-domain validation: layout engines under metamorphic probe — BUILT IN
    MINIATURE** (`layout-probe/`; the ELK/d2 binding in the diagram tool's repo remains
    the downstream adoption this member de-risks). What the build proved, over two
    deterministic layered engines differing by one policy bit (within-rank order: node
    name vs declaration index): the scoped laws needed TWO new catalog stanzas —
    `inert` (u(x) = x) and `equivariant map` (the commuting square f(act(x,p)) =
    act2(f(x),p), which `action equivariance` could not say) — and with geometry as
    the observation, discovery split the engines along a REAL tradeoff neither wins:
    the stable engine holds `inert(reorder)` and loses rename-equivariance (its
    missing law shows as coverage silence: "operators in no law: render"), the eager
    engine holds the square and loses reorder-inertness (dagre's layout-jumping
    pathology, named by an absent law). Both declare the same three laws, so the two
    pinned distance reports ARE the engine scorecard. The visual census emerged as
    designed (floors 4/2 derived never typed; freeze-stable rows only), the locality
    witness measured the same tradeoff in pixels (an early-sorting insertion shifts a
    whole rank under stable, nothing under eager), and the fire drills prove refusal
    is earned: a jittering engine never reads as stable, and a grown corpus reddens
    the census. Weekly member sweep (`--lib`, the delta-render economics) feeds the
    countersign. Original scoping below, kept as the downstream build's brief. The domain: diagram source → layout engine →
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
    than per-`cargo test`.

    **The lint layer, refined — the constraints EMERGE.** The first fence ("plain
    predicates, no algebra") was right about the predicates and too crude about the
    thresholds. Measuring contrast is plain code; *4.5:1* and *8px* need not be typed
    by anyone. The honest mechanism is the third artifact of the design, the **visual
    census**: derive, from the ratified diagram corpus, the regularities it actually
    exhibits — the contrast floor, the spacing residues mod each candidate grid, the
    alignment deviations — as one deterministic text artifact, frozen with `spec-lock`
    (the tiny crate, not the engine: this is the qualify-census move pointed at
    pixels). The epistemics matter and are the method's own: a derived floor is
    DESCRIPTIVE of the corpus; **the freeze is the only normative act** — committing
    the census is what turns "what our diagrams happen to do" into "what our diagrams
    must do". From then on a new diagram that erodes a floor is a red gate naming the
    movement ("contrast floor 4.7 → 3.9 — ratify or fix") instead of silent decay,
    a diagram that TIGHTENS a floor is a visible diff too (standards can be raised by
    example), and deliberate outliers are `Register` entries with justifications. So
    the reconciliation: predicates plain, thresholds emergent, standards ratified —
    the 200-line lint script survives, it just stops carrying opinions and starts
    carrying derivations. The strong version — the LAW ENGINE inferring threshold
    constants itself (discovered bars filling Constant slots, tightest-that-holds) —
    is recorded as catalog research, not v1: the census form needs none of it.

    **Stretch goal: how the constraints shape aesthetics.** Once the census exists,
    taste stops being ambient and starts leaving a record — four executable forms,
    in ascending ambition:

    - *Taste as chronicle.* The only place aesthetics enters the system is
      ratification, and every ratification is now a census diff — so the diff history
      IS the house style's chronicle: when the spacing grid tightened, when a floor
      was deliberately traded away, who signed it. An aesthetic you can `git log`.
    - *Taste as inheritance.* A new agent (or designer) dropped into the project
      inherits the frozen census and the metamorphic laws — the taste FLOOR transfers
      without the taster: they cannot silently produce work below the look, and every
      way their work pushes on the look arrives as a reviewable diff.
    - *The removal drill, pointed at taste* (the warrant's move, verbatim): drop one
      frozen constraint, regenerate or re-sample layouts under the remaining set, and
      ask whether preference — human ratification, or a pinned preference model —
      actually degrades. A constraint whose removal changes nothing anyone can see is
      DECORATION, flagged not frozen; the load-bearing residue is, operationally, the
      aesthetic. "Which rules make it look like US" becomes an experiment, not a
      debate.
    - *Style as the quotient.* What the census pins is the house look; the visual
      degrees of freedom it CANNOT see (the mutation-survivor concept, aimed at
      pixels) are where individual voice lives. Choosing which surviving DOFs to pin
      next is the design act, made explicit — and the three-valued band marks the
      taste boundary honestly: near-threshold work is DISCLOSED for human eyes,
      never auto-judged at the edge.

    Honest limit, stated as the method always states it: the judge here is human, and
    stays human. The apparatus makes taste legible, transmissible, and measurable for
    load-bearing-ness; it can never derive it. This is the "what did you mean"
    boundary wearing its other face — "what do you like" — and the freeze remains the
    only place the answer enters.

    What this domain uniquely buys the METHOD: millisecond feedback with visible
    ground truth — the fastest refutation loop any consumer has offered yet. Open
    questions for the build session: engine order (ELK first, exhaust it before
    anything proprietary; Graphviz as the free DAG baseline), observation
    canonicalization (quantized positions vs topology-only, possibly BOTH as two
    observers over one carrier), whether locality needs its own stanza or
    premise-guarded monotonicity over a declared distance order suffices, and which
    census statistics are STABLE enough to freeze (floors and residue histograms
    survive corpus growth; means do not — a census row must be a fact a new diagram
    can only move visibly, never smear).

14. **Continuous autoshaping: the placer — BUILT** (`discover::shape`,
    `spec/boundary-spec.shape.spec`). The friction it dissolves, named by this repo's
    operator: LLMs writing code and considering shape at the same time degrade at
    both — the useful move is the circuit-CAD one, where the author writes the
    NETLIST (behaviour in one big bundle) and the tool derives the placement.
    The algebraic twin of a net is a SORT: the placer partitions operators by
    net connectivity (a sort in both signatures — one produces what the other
    consumes), read off declarations alone, never discovery, so shape is derived
    before a single law is judged. Two instruments now, two questions: cohesion
    links by LAW co-occurrence and reads wiring density (a suggestion a human
    ratifies); placement links by NETS and reads the boundary (a derivation).
    The dogfood was the design constraint — "if we can get this to work where
    we've manually pinned things we've got something really good; falling back
    to pinning kicks the can" — and it landed on the first run: all six declared
    modules place SETTLED, including the three cohesion wants split and this repo
    keeps whole by hand-ratified pin (arithmetic's `<` shares `Int` with `+`,
    the calendar's `since` shares `Date` with `add`, the protocol's transitions
    share its state sorts). The keep-wholes are now derived, not spent; the pin
    survives only as the wiring observation it always was. CONTINUOUS because the
    shape is a lock like everything else: `ShapeReport` freezes per system,
    re-judged every `cargo test`, and placement is MONOTONE by union-find's own
    algebra — a new operator joins a component or bridges two, it can never
    re-split or reshuffle what it does not touch (the layout-probe locality
    property, holding for the placer itself). First-run bonus finding: the report
    surfaces cross-module net-NAME coincidences no declared seam covers as SEAM
    CANDIDATES — and this repo has exactly one, `Duration` named by both the
    calendar and the ttl store (two Rust types agreeing on a word; declare the
    seam or leave the coincidence standing — rendered, never merged). Follow-ups:
    the splits placement CANNOT see (a genuinely shared net carrying two features
    — the rail-sort problem, circuit CAD's power-plane exclusion) stay cohesion's
    territory, disclosed; genesis v2 should close the loop by EMITTING a bundle's
    derived modules as the scaffolded tree (write the netlist, get the placed
    crate), which is the autoshaping half of the v2 work list below.

## Three bricks from the release conversation (the ceremony hunt, continued)

The observation that named them: every arbitrary ceremony in software is a derivable
fact wearing a ritual's clothes — and the operator's phrase for the programme,
"replacing implied service from a platform with specification". All three BUILT:

1. **The bridge alarm in the agent loop** (`Ticker::hook_line`,
   `.claude/hooks/shape-watch.sh`). The mechanism question settled by token economics:
   LSP is the human surface (pull-based, verbose, free for eyes); the lock is the CI
   surface (late); the agent surface is a PostToolUse hook, because it prices feedback
   right — zero tokens when silent, one line when the shape moved, no instruction
   anywhere. The noise policy is the design: joined edits are free silence, a seed that
   opens a second component announces its extraction, a bridge always speaks. The hook
   informs, never acts.

2. **The dependence lock** (`discover::depend`). Compatibility is a relation between a
   change and a consumer, not a global property of the change — semver's integer
   pretends otherwise. A consumer declares the laws it relies on (equation as identity);
   `Dependence::judge` answers over any two frozen lock texts: INTACT, CHANGED (new
   statement carried), GONE (the breaking case, named). A reliance the baseline never
   held refuses. With automatic releases publishing the lock diff, the full loop is:
   pin two release tags, judge your reliances, read the verdicts.

3. **The review router** (`discover::agenda`, `examples/review_agenda`). "Review the
   PR" was the last implied service in the workflow. The diff classifies itself: one
   ratification question per moved lock class (laws, freedoms, boundary, seams, surface,
   pipeline, exceptions, world, vocabulary), prose read for sense, interior code named
   machinery-verified. An unknown spec-directory artifact refuses — misfiled-as-machinery
   would be a silently dropped review. First run routed its own branch: one ratification
   (the qualify census), seven files machinery-verified.

## Tiers as a lock — derive three, Register the fourth (BUILT: the ladder ran to the top)

The tier annotations (`//! Tier: <...>` on every file) are the last per-file ceremony:
N scattered hand-maintained declarations of a partition that is mostly derivable.
The four tiers are not the same kind of thing — BOUNDARY, INTERIOR, and ALGEBRA are
FACTS (qualify logic, import-graph reachability, remainder) and belong in a derived,
frozen `spec/tiers.spec`; KERNEL is a DECISION (a privilege can never be inferred from
conduct — that is self-attestation) and belongs in a `kernel.register`
(`spec_lock::Register`: every exemption carries a ratified justification, an
unjustified key refuses, a stale entry is a lie — strictly better than
marker-plus-allowlist, because the reason becomes reviewable text). The reader-service
the markers provide moves to the edit hook (one injected line on first edit of a file:
its tier and rules); genesis v2 emits one tiers artifact instead of a header per file.

Step one landed with the candidate: `boundary-enforce` computes the tier census in the
same walk as the qualify census (`Config::tiers_spec`, `BLESS_TIERS`), frozen to
`spec/tiers.spec`; the review router routes it as its own ratification class. The first
derivation (operator-shape as the BOUNDARY signal) froze at 28 agree / 8 disagree, and
the disagreements taught the derivation: operator-shape is QUALIFY's fact, not a tier
claim — the repo's real boundary mark is carrying production edge impls, and pure glue
(module declarations and re-exports) has no evidence to judge, so it stands as
declared. Derivation v2 froze at 35 agree / 1 disagree
(`select/boundary.rs`: tier-1 strictness, no production edges), and the operator asked
the right question — are we just missing evidence? Yes: the FRONTING relation, the tier
system's own semantics read backwards. INTERIOR is not merely unreachable, it is
fronted; the front is the file that delegates into it; doors are boundaries. A
pub-reachable file referencing an interior sibling by path derives BOUNDARY. Derivation
v3 froze at 46 files: 36 agree, 0 DISAGREE, 10 kernel decisions — the partition was
fully derivable, and steps two and three followed in one motion: `derive_partition` is
now the single source both the rule dispatch and the lock consume (they cannot
diverge — same rows), KERNEL comes only from `spec/kernel.register` (ten entries, each
with its justification; `build.rs` parses it via `spec_lock::Register`, a stale entry
is a violation), and every `//! Tier:` marker is deleted. `spec/tiers.spec` no longer
records coherence — there is no declared column left to disagree with; it IS the
partition, one line per file with the evidence that placed it. The edit hook's kernel
exemption reads the register too, so a marker in a file grants nothing anywhere.

The ladder held at every rung, never skipping step one: (1) derive and freeze
alongside the markers with a coherence gate — declared vs derived, disagreement is a
build error; (2) once quiet, flip enforcement to the derivation; (3) delete the
markers. The lesson worth keeping: when the coherence gate disagrees, ask whether the
declaration is wrong or the derivation is missing EVIDENCE — both v2 and v3 came from
disagreements that were really undiscovered relations.

## The ninth ask: convention out of the code (BUILT)

A production adopter took the agent-senses release same-hour and reported the two
places v1 baked THIS repo's conventions into shipped code — both the same lesson the
tiers ladder just taught internally: turn the convention into data or derivation.

1. **The guard's voices are derived, not assumed** (`GuardVoices::for_edit`). The
   loose-`pub fn` voice pre-fires a refusal that only exists where the enforcement
   shim runs; on an ordinary binary crate it was a warning per public function about
   a refusal that does not exist — the guard violating its own contract. Now each
   voice is switched by evidence: kernel-exemption is a register lookup, the
   rats-nest voice is on exactly when the tree's `build.rs` attaches the shim. The
   contract ("pre-fire existing refusals, never invent judgments") holds by
   construction on any tree, including one that never heard of the shim.

2. **The router's class table is consumer data** (`Agenda::of_with`,
   `Ratification::Custom`, `spec/agenda.register`). `classify()` knew only this
   repo's artifacts, so a consumer's lock classes all read "teach the review
   router" — an instruction only upstream could follow. Teaching is now a committed
   register (one `suffix: question` line per class, `spec_lock::Register` grammar);
   taught classes match before every built-in, and the guard consumes the same
   table, so a registered consumer lock gets never-hand-edit instead of
   teach-the-router.

3. **The ticker's second source** (`Ticker::step_theory`,
   `Placement::signatures_of`). The place ticker keyed on `ops { ... }` stanza text;
   a consumer that models its theory through the library API had nothing to parse
   and read the silence as non-adoption. The engine's signature table now feeds the
   same source-agnostic core, so code-modeled theories get the identical live
   layout sense — text and type are two fronts on one ticker.

And a pattern discovered downstream, now documented on `discover::depend`:
**self-judgment** — `judge(committed, committed)` in the theory owner's own suite,
where INTACT is trivially the only verdict and the protection is the refusal path: a
re-bless that drops a law any consumer declared refuses by equation before the
ratification diff lands. Old-vs-new is the cross-repo consumer's tool; self-judgment
is the owner's tool for making declared reliances un-droppable.

## Candidate: behaviour as code — the world lock grows eyes

Terraform is two machines in one trenchcoat: a TRUTH machine (state file, plan,
drift detection) and an ACTUATOR (apply, convergence, dependency-ordered mutation).
The candidate is to take the eyes and leave the arm — the method covers the truth
half better than the incumbents, and grafting on an actuator would break the house
rule that makes locks trustworthy (locks gate, they never act; the one Effectful
gate acts only on an already-certified tree).

The reframe that makes it a candidate rather than a metaphor: **the spec of
infrastructure is its behaviour, not its physicality.** A request from A reaches B;
a credential minted with scope S cannot read T; the queue drains under load N —
these are observations through the boundary, and resource attributes (instance
types, ARNs, module trees) are interior representation. Two worlds that behave
identically ARE the same world; that is the observation-function quotient this repo
already stands on, pointed at clouds. Today's tools diff physicality (`plan` is
field-by-field structural equality against a state cache); nobody diffs behaviour,
and nobody has an algebra of it.

The security mapping is the strongest leg, because its miniature is already BUILT.
Security posture today is asserted by configuration convention — platform features
plus module layering, the tier markers of operations. The tiers-as-a-lock ladder
translates line for line:

- attack surface = DERIVED reachability (pub-reachable ≙ internet-reachable; the
  fronting relation ≙ a bastion — a door is a door);
- the qualify census ≙ the surface census: what is exposed, as a computed,
  ratified, drift-gated fact;
- the kernel register ≙ privilege: never inferred from conduct, every exemption
  (break-glass role, admin path) a justified line in a committed register, a stale
  entry a lie;
- and security claims are REFUTATION-SHAPED — "no path from the public net to the
  db port" is a law a probe can only fail to refute. Green-is-evidence-not-certainty
  is the honest frame security has always needed and compliance checkboxes have
  always faked.

The full loop would be genesis's two-lifecycle story at world scale: a `system!`
declaration whose modules are services, seams are network edges, and expects are
reachability/deny/flow laws; the target lock committed RED on purpose; the existing
actuator (terraform or anything) makes the world; Effectful probes earn the
declaration green; the world lock freezes what was earned and gates the drift.
"Behaviour as code" is the honest name — the declaration is a law set, not a
machine list.

The overlay analogy sharpens the product shape: Tailscale is to network switches
what this is to physical infrastructure — an overlay whose LAWS are the product.
Tailscale does not configure switches into compliance; it simulates a flat,
identity-addressed network on top of hostile physicality, programs are written
against the overlay, and the switches become interior. Likewise here: program
against a virtual world whose behavioural laws are discovered and locked, and the
physical cloud is an INTERIOR that must front it. Fronting is already the
vocabulary — the virtual layer is the door, physicality the workshop behind it.

And SIMULATION-FIRST dissolves the engine objection that would otherwise park this
for years. The engine assumes cheap, pure, replayable evaluation; a world probe is
none of those; but a SYNTHETIC world is all three — the TTL store's move at world
scale. The store made time a value (its own logical clock, advanced only through
the Tick edge) so there was no ambient now; a world value object (services, edges,
policies as Shaped data) makes there be no ambient cloud. You cannot make
us-east-1 a value — but you can make A WORLD a value and demand us-east-1 front
it. Grids of worlds are enumerable in-process, so reachability/deny/flow laws are
discoverable and mutation-testable TODAY, by the existing engine, with zero new
machinery: the interior is synthetic initially, and the tier-2 rule is precisely
that the interior is free to swap — replacing synthetic semantics with real
read-only probes later moves no boundary and restates no law. Sim-vs-real then
becomes a TRANSPORT seam (`CoherenceReport::between` the model and the probed
world: one observable, two paths), so "does reality still match the simulation" is
a seam verdict with a lock, and the expensive probes are spot-check countersigns
of laws the simulation earned cheaply. Evidence discipline unchanged: a green sim
law says nothing about reality until the seam countersigns it.

Step one is BUILT: `discover::fabric`, the synthetic world as a registry theory
like any other. A `Fabric` is grants plus standing denies over a small node
universe; `mesh`/`join`/`grant`/`revoke`/`reach`/`within` (and the `true` constant)
are its operators; twenty named laws froze into `spec/fabric.spec` — the join
semilattice, grant/revoke as well-behaved actions WITH their directions (grant only
grows delivery, revoke only shrinks it), reach as a projection fixed at `mesh` and
monotone under `within`, and the witness inequations. The load-bearing REFUSAL is
pinned like the router's non-commutativity: reach is NOT monotone in the
join-order, because a merge carries the other side's denies — "adding
infrastructure only adds connectivity" is false, and the engine keeps saying so.
Discovery also corrected the author once (predicted-refused subadditivity HOLDS:
`within` compares closed deliveries, so bridging is quotiented away — the
observation-function lesson, taught by the machine), and the first mutation sweep
GREW the law language: grant confused with revoke survived in both directions
(identical law sets — the vocabulary could not say which way an action moves a
value), so the `action inflation`/`action deflation` stanzas joined the catalog,
discovery found the direction laws itself on the next run, and all 22 mutants die.
Infrastructure behaviour is a discoverable algebra, today, with zero new machinery.

Remaining rungs: (2) read-only world probes as first-class edges with declared
cost, countersigning the simulation through the transport seam; (3) the world lock
generalised from library dependencies to any probed surface. Build nothing that
mutates: the arm stays someone else's.

## Standing follow-ups

- ~~**Tag `v0.1.0`**~~ — superseded by AUTOMATIC RELEASES (the `release (certified
  tree)` gate, the registry's first Effectful entry): every countersign that advances
  `mutants-green` publishes the certified tree as a CalVer-tagged GitHub release, with
  notes derived from the commits and the ratified spec-lock diff. No semver: a version
  integer is a hand-asserted compatibility claim nobody checks — the lock diff is the
  same information, uncompressed and verifiable. Consumers pin a release tag and read
  exactly which laws moved between any two of them.
- **Publish** when ready: `docs/publishing.md` has the dependency-ordered sequence;
  all four names (`boundary-spec`, `boundary-spec-macros`, `spec-lock`,
  `boundary-enforce`) were verified unclaimed on crates.io on 2026-07-02.
- **Morphism downstream**: the fixture exercises Construction/Branch/Guarded;
  the fourth edge shape is honestly unexercised downstream.
- **MSRV**: unpinned; verify and add `rust-version` after first publish.
