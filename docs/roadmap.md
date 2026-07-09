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

1. **The bridge alarm in the agent loop** (`Ticker::hook_line`, now carried by the shipped
   `probe-hook` binary — see the eleventh ask; originally the retired `shape-watch.sh`).
   The mechanism question settled by token economics:
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
its tier and rules) — BUILT, now `probe-hook`'s third voice (`tier_voice`, reading
`spec/tiers.spec`, paid once per file); genesis v2 emits one tiers artifact instead of a
header per file.

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

## The tenth ask: open text stops feeding the rules (BUILT)

The adopter's tenth ask asked to be the last of its kind: nine asks had landed as
hand-carried markdown — plain text as the driving input to a formal system, held by
diligence. The receiving surfaces now exist. A LAW reliance is a line in
`downstream/reliances.register` (`<theory> | <equation>: <consumer> — <why>`),
self-judged against the committed locks on every test run — a re-bless that drops a
relied-on law refuses by name, equation and consumer and why, before the release
ships. A SURFACE reliance is a line in `downstream-fixture/tests/reliances.rs`,
compile-judged — the strongest gate available for an API. Prose keeps only the
justification, the one thing no derivation produces; the antipattern is named in
`docs/experience.md` so the pattern cannot grow back unrecognised. Consumers author
their own lines by PR. Field reports remain welcome for what registers cannot carry:
lessons, corrections of frame, and the eleventh kind of ask nobody has had yet.

## The eleventh ask: the hook is a shipped binary (BUILT)

Filed the moment the tenth ask's antipattern was spotted wearing the consumer's own
`settings.json`: the guard is mutation-tested and register-driven, and every consumer
was fencing it with the same four pieces of unjudged glue — a bash wrapper, inline
JSON-parsing Python, a stale-binary build fallback, hand-authored settings plumbing.
`probe-hook` (a workspace member, `cargo install probe-hook`) is that envelope inside
the boundary: it speaks the hook protocol natively (real JSON parser — paths can carry
escaped quotes), honours `CLAUDE_PROJECT_DIR`, derives voices from the tree and
classes from `spec/agenda.register`, and `probe-hook install` writes or idempotently
merges its own `settings.json` entry, refusing (never clobbering) a file it cannot
parse. Fail-open is a DRILLED property of the library (`respond` returns `Option`;
malformed JSON, missing paths, refused registers are silence), not a `|| exit 0`
convention. The skew floor from the ask's honest frame ships: every voice block
carries the binary's version, and advisory/fail-open means skew degrades to weaker
advice, never a false refusal. Re-execing a repo-local build stays deliberately
unbuilt until skew is observed hurting. It publishes with the other four on every
certification release.

**The antipattern closed on our own tree (follow-up):** the eleventh ask was filed
because the antipattern was wearing THIS repo's `settings.json` — and yet the repo kept
using `shape-watch.sh`, the very bash-wrapper-plus-inline-Python glue the crate exists to
retire. Fixed: `probe-hook::respond` grew the SECOND voice (the shape ticker —
`discover::watch::Ticker`, folded in from what `place_watch --event` did), so the shipped
binary is now the WHOLE edit-time envelope (guard + ticker), not just the guard;
`.claude/settings.json` invokes `probe-hook` (the exact entry `install` writes); and
`shape-watch.sh` is deleted. The repo now dogfoods the guard it ships — the antipattern
gone from the one place it was most visible. Local activation is the shipped consumer flow
(`cargo install --path probe-hook`, wired into the session-start hook for the remote env).

## The perimeter is a lock (BUILT)

The settings page was the last hand-clicked configuration in the loop — the open-text
antipattern one level up: prose recipes translated into a UI, verified by nobody,
drifting silently, re-audited never. Now it is what everything else is. The floor is
DECLARED (`discover::perimeter`), and its required status checks derive from the gate
registry itself, so a renamed or re-cadenced gate moves `spec/perimeter.spec` in the
same diff — a rename can never silently unprotect the default branch. The declaration
renders TWO artifacts: the human-readable floor and `spec/perimeter.ruleset.json`, the
apply-able branch ruleset — the one manual act left is posting it
(`gh api repos/<owner>/<repo>/rulesets -X POST --input spec/perimeter.ruleset.json`).
The write stays human on principle: the perimeter constrains the agents, so it must
not be agent-writable — GitHub's settings page is the platform's kernel register, and
a privilege is ratified, never self-served.

What the machine owns is refusing to let reality rot: the `perimeter (settings
drift)` gate — the pipeline's first WORLD gate, weekly, Effectful-as-a-read — pulls
the live rules back (`.github/perimeter.sh` → `examples/perimeter.rs` →
`Perimeter::judge`, where the probes reach) and holds them to the floor. FLOOR
semantics: extra live protections are not drift (stricter is never a lie); required
approvals are the one exact match (above zero deadlocks a solo maintainer). The
never-applied state is a red gate naming each missing rule — the reminder that never
expires — and a world fact never feeds the countersign: the certified tag must not
wait on a settings page. Casualty of the brick, per its own instruction: CLAUDE.md's
"PRs land by squash merge" line, now a declared, read-back merge-method rule.

## The twelfth ask: encode the infra meaning (BUILT)

The consumer's incident chain had a layer below any repository: a bucket's CORS
allow-list that still named a migrated-away origin, a build command living in a
dashboard text box, a surface running a day on placeholder secrets, store-prefix
meanings encoded as a regex in one file and prose in another. Every one was a law
without a declaration — the seam existed, the law existed, and the refusal came from a
customer's browser weeks later. `discover::infra` extends the declaration discipline
one level down, on the perimeter pattern: `Infra` declares the graph (surfaces with
origins and the build that produces them; stores whose roots carry a meaning class —
`ephemeral(ttl)`, `locked`, `drafts`; presign/read seams; credential NAMES per
surface; authorities; cadences), `coherent()` judges the declaration against itself
(an edge to an undeclared node, a presign outside any declared root: refused by name
before any API is consulted), and `judge()` holds a consumer-extracted `LiveInfra` to
the three API-readable laws — *cors-covers-origins*, *secrets-census*,
*build-command-is-derived* — with the perimeter's floor semantics (extra coverage is
not drift; the build command is the one exact match).

Where no API reaches, the register floor: `unjudgeable()` derives the exact key set
(root meanings, authorities, cadences) and `floor()` holds a hand-authored
`<system>.infra.register` to it with the existing set-difference semantics —
declared-but-unjudgeable is disclosed as exactly that, never silently assumed held.
What derives instead of being re-encoded: `ephemeral_prefixes()` hands a probe the
writable-without-consequence roots and their TTLs straight from the declaration; the
TTL half of prefix-discipline is unrepresentable as drift because `Meaning` carries it
by construction. The committed exemplar (`spec/exemplar.infra.spec` +
`.infra.register`, frozen by `freeze_infra`) is the first consumer's shape with the
names washed out — two surfaces, one store with three root meanings, four seams, six
secret names, one authority, two cadences — so the whole path runs on every test run.
The router learned the class (`Ratification::Infra`, named by system). Honest frame:
the judge holds declared facts against readable state — an undeclared seam is
invisible; and the judging gate's own credential is a line in the census it checks,
which is why the floor exists from day one. The extraction (which cloud API, which
fields) is the consumer's, like `examples/perimeter.rs` is ours; only the judgment
lives here, where the probes reach. This is behaviour-as-code rung 2 made concrete:
read-only world probes, judged on the consumer's cadence and credentials.

## Git itself is a lock: the substrate (BUILT)

The perimeter covers GitHub-the-service; nothing covered git-the-data, and the repo
leaned on undeclared git meaning everywhere: the `mutants-green` tag is the
incremental gate's entire baseline, its semantics living as prose and as behaviour
inside `mutants-gate.sh` (the exact gate-defined-where-no-drift-gate-can-see-it shape
the pipeline brick killed); the CalVer release tags carry certification semantics
`release.sh` assumes; and "main is linear, every commit a squash of a gated PR" was a
declared perimeter rule about future merges but an unjudged observation about the
history that exists. `discover::substrate` declares those meanings — each tag or
trailing-`*` family with the prose the scripts otherwise keep (each name owned once:
`mutants-green` IS `gates::GREEN_TAG`, never restated), plus the linearity epoch (the
last pre-discipline merge commit) — and `Substrate::judge` holds a `LiveSubstrate` to
them: a required tag absent, a meaning-carrying tag off the certified line, a merge
commit after the epoch, an unreadable read — each refuses by name. The meanings are
STATE-FREE (what a tag means never changes with whether it exists; presence is the
judge's to report), and the publish-marker law names no tag at all: every version the
crates.io index reports published for the root crate demands its `v<version>` marker
on the certified line — instances derived at judgment time, so a new publish extends
the law with no declaration edit, and `release.sh` mints the marker when it publishes
so the law self-satisfies going forward. Frozen as `spec/substrate.spec` (rides
`freeze_gates` with the other repo-meta locks); read back by the weekly
`substrate (git drift)` world gate (`.github/substrate.sh` → `examples/substrate.rs`,
which asks the declaration for the epoch and the marker crate instead of restating
them). The reads are credential-free — git plumbing against the checkout's own origin
plus one anonymous sparse-index read — and like every world fact never feed the
countersign. One derived instance is red on arrival: `boundary-spec` 0.1.0 is on
crates.io with no `v0.1.0` marker in the repository (the one publish that predates
the minting machinery) — the reminder that never expires, computed instead of
declared; push the tag at the published tree and the gate goes green.

## Mutation as a distance dial: the deafness floor and the dent sweep (BUILT)

A function on a bounded grid IS its value table, so "how different from the real
thing" is a distance, and mutation layers sit at points on that dial.
`discover::mutation` grew two, both in-process, both inside every `cargo test`:
**deafness** (farthest — an operator returns one constant everywhere, one mutant per
distinct output: is its output constrained to depend on its input at all? random
noise derandomised — on enumerable codomains, every constant beats a coin flip) and
the **dent sweep** (nearest — the minimal meaning change: exactly one input tuple
returns a wrong value). The existing table battery (confusion, projection,
partiality) sits between them. The mutants carry captured state a bare `fn` cannot,
so a thread-local surgery + trampoline evaluator plants them without touching the
engine's fn-pointer design; judgment is `Engine::check` over the laws NAMING the
mutated operator (exact, not approximate: a term evaluates only the operators it
names), which is why hundreds of mutants per theory cost tenths of a second.

The feedback changed kind, not just amount: a surviving dent is not a source line, it
is a COORDINATE — the exact input whose output no ratified law constrains. The
committed `.mutation.spec` locks now carry each theory's unpinned-region map: fabric's
`within` verdict is free at specific pairs, doc-flow's `edit` revision counter is
largely unpinned, arithmetic's `<` is unconstrained at `(0, k)` off the law grid —
each a ratified degree of freedom with an address, closable by a sharper shape or
expectation and gated the moment it moves. Honest frame: dent adequacy is per-point —
coordinated multi-point changes that preserve every law are the spec's symmetries
(relational probes pin meaning up to isomorphism, their theoretical ceiling), and the
check-judged layers trade the appearing-law kill direction for every-change economics
(disclosed in the lock headers). Alongside: `boundary-spec` joined layout-probe and
delta-render in the dev-profile opt-level override — measured 8.5× on the mutation
suite (22.8s → 2.7s) — so the widened batteries ride `cargo test` at compute speed.

## Mutant schemata: the compiled population (BUILT)

The rebuild-per-mutant price is gone for the expression flips. `#[mutate]`
(boundary-spec-macros) rewrites each `==`, `!=`, `&&`, `||` in an instrumented
function as `if Schemata::active("<site>") { flipped } else { original }` — the left
operand binds once, so laziness is preserved and left-leaning chains grow linearly —
and every site registers into a link-time slice (`discover::schemata::MUTANT_SITES`)
whose sorted census freezes as `spec/schemata.spec`: instrumenting a function is a
ratified diff, a label collision refuses by name, and the router routes the census to
its own class. The `mutation (schemata)` gate (Pure) builds ONCE and runs the lib
suite once per site with
`PROBE_MUTANT` selecting the flip; survivors are ratified by key in
`spec/schemata.register` or killed with a probe. First full sweep, measured: 23 sites
across the three world judges, the router's classifier, the reliance judge, and the
tag grammar — 23 killed, 0 survivors, 82 seconds end to end (the same population as
rebuilds: ~12+ minutes). Cheap enough that it was promoted from the weekly clock to
EVERY change on arrival — expression mutants now gate PRs, inside the same required
check as fmt/clippy/test. The zero is not luck: the sensitivity drills are the
killers for every judge site — the layers compose.

Vertical integration, same arc: the flip catalog grew the orderings (`<` → `>=` and
kin — the negation, disturbing both edge and direction) and `!`-deletion; every
instrumented function gained DEAFNESS forms read from its return-type syntax
(`Ok(default)`, `Err(default)`, both booleans — the whole-body replacement class,
source mutation's biggest, now compiled behind the same selector); and the sweep's
economics are all derived — nextest fail-fast stops a killed mutant at its first
failing probe, and the timeout prices itself from a timed green baseline (5x + 10s;
a hand-picked constant was the antipattern wearing a number). Second sweep: 38
sites, 38 killed, 0 survivors — including the census catching its own growth (the
router arm taught for `schemata.spec` added a `==` that immediately became site
`classify:9`). The nextest install step in the check job DERIVES from the
declaration: a consumer pipeline with no schemata gate carries no extra step. Honest frame: schemata covers what a
runtime branch can express; whole-function replacements and type-level mutations stay
with the source sweeps, `matches!`/`assert!` interiors are opaque tokens (disclosed in
the macro docs), and one mutant runs per process — flip interactions are out of scope,
as they are for every mutation layer here.

## The sensitivity drill: judges deaf to nothing (BUILT)

The dent idea reached the world-lock judges without table-izing them: a judge's
domain is its LIVE state, so the minimal meaning change is a LIVE DENT — one field of
an applied fixture perturbed, and the verdict must both move and NAME the fact
(`discover::judgment`, `LiveDent::drill`). Each `Live*` struct enumerates its own
dents, and completeness is a compile-time pin: the enumerator opens with a full
destructure naming every field, so a new live field refuses to compile until its dent
is decided. The enumerators encode one reviewable judgment — which perturbations are
refusal-worthy (floor semantics accept widenings; an optional tag family's deletion
claims nothing) — and the drill catches both failure modes by name: a judge DEAF to a
fact (verdict stays green) and a judge that moves without naming it (a refusal nobody
can act on). All three judges pass: perimeter 9 dents, infra 15, substrate 11. Under
the source-level sweeps the drill is itself a probe, so the survivor species that
kept recurring — a membership check that stops distinguishing one element among
present ones — is closed as a CLASS: a mutant that deafens any judge to any fact now
dies here, and a fact added tomorrow arrives with its dent or does not compile.
Alongside, the session-start hook gained a FREEDOM sense next to the topography: the
ratified survivor count read from the committed mutation locks, byte-stable between
ratifications — the agent wakes knowing how many named degrees of freedom are open
and where the addresses are.

## The first retirement, taken: the per-diff source gate is gone

The census went 70 → 371 → 697: cfg-gated dual emission (`--features schemata`
builds carry the branches; every other build compiles the ORIGINAL items byte for
byte) let the engine and the whole discover tree join, impl-level plus free-fn
blanket annotation, all site ids module-path-qualified at expansion (two theories
both had an `add`; the collision refused, as designed). Completeness is now a
CENSUS, not an intention: every top-level fn and impl either carries `#[mutate]`,
is `cfg(test)`/`const`, or its file holds a reasoned line in
`spec/instrumentation.register` — judged two-way, so exemptions can only shrink
honestly. The 697-site sweep (parallel workers over one build, coverage-mapped,
~8 minutes at 4 cores) found six survivors, each ratified with its reason in
`spec/schemata.register`: two equivalences in the mutation harness's own mutant
vocabulary, one conservative-direction freedom, the doc-flow edit freedom seen
through a second lens, and two genesis sites uncovered by the lib suite — the
owed lib-side twins, disclosed. On that basis the `mutation (changed lines)`
per-diff gate is RETIRED from the registry (13 gates; the perimeter's required
checks derive down to the one job — re-apply the ruleset after merge). Still
earning rent: the since-green incremental and the weekly shards (type-level
mutations, statement deletion, the exempted files) and the member-crate
companions. They retire the same way this one did: when the census closes the
territory they cover.

## Retiring the source-level mutator: the enablers (BUILT), the retirement (staged)

The two enablers landed. `#[mutate]` applies to WHOLE impl blocks (labels
`<Type>::<method>` — coverage grows per block, not per function), and the sweep is
COVERAGE-MAPPED: the baseline run doubles as the recording run (`SCHEMATA_RECORD`;
nextest's one-process-per-test makes each touch-file a site→test edge list), so every
mutant runs only the tests whose execution reaches its guard. That selection is
EXACT, not sampled — a flip cannot change behaviour where its guard never executes —
and a site no test reaches is a survivor before any run: unexecuted is unkillable,
disclosed, never assumed. Measured at 70 sites (the judges, the full review router,
the dependence judge and report, the ticker): 70 killed, 0 survivors, 0 uncovered,
~68s end to end with sub-second marginal cost per mutant — the population can grow
an order of magnitude and stay inside the every-change budget, two orders with
process-level fan-out.

The retirement itself is DATA-driven, in stages: (1) keep widening instrumentation
through the cold interior (impl-level attributes make each module a one-line diff,
each ratified in the census lock); (2) the completeness census — "every eligible
interior function carries `#[mutate]` or a register-exempted line" — turns coverage
from intention into a gate; (3) the engine (hot paths) needs CFG-GATED emission
(`--features schemata` builds carry the branches, normal builds carry nothing) — the
one remaining design piece, priced at one extra cached CI build; (4) each source-gate
retires when the sweeps go quiet on the classes schemata covers where it covers them
— the changed-lines PR gate first (its diff is almost always instrumented territory
once the census gate exists), the weekly shards last. Every source-sweep survivor
until then is a signpost naming the next function to instrument.

## Candidate (superseded): retiring the source-level mutator

The four layers exist to be composed, and their composition points at full
elimination of the external mutation tool — aspirationally fast enough to be a LOCAL
check, not a CI economics problem. The pieces, each already proven in miniature:
(1) WIDEN the flip catalog — `<`/`<=`/`>`/`>=`, arithmetic, `!`-deletion are the same
transformation `#[mutate]` already does for `==`/`&&`. (2) WHOLE-FUNCTION deafness as
a schemata form — the macro reads the return TYPE syntax (`Result<A, B>` →
`Ok(A::default())`/`Err(B::default())`, `Option` → `None`, `bool` → both constants,
Shaped values → `inhabitant()`), which is the source tool's biggest mutant class,
compiled behind the same selector. (3) CFG-GATED emission (`--features schemata`
builds carry the branches; normal builds carry literally nothing), so even hot-path
code like the engine can be instrumented without taxing every test run — the sweep
pays one extra build, once. (4) COMPLETENESS DERIVED, not hoped: the qualify machinery
already parses the tree, so "every eligible interior function carries `#[mutate]` or a
register-exempted reason" is a census gate, the same move that closed every other
hand-list. (5) SITE→TEST coverage recorded from one instrumented baseline run — the
one-binary property means the map survives, so each mutant runs only the tests that
reach it (the source tool rebuilds away any such map), and mutants fan out as
parallel processes against one build. Timeouts are already detections (the sweep
script kills at 300s and counts the kill). What the source tool keeps until earned
otherwise: exotic operators, match-arm/statement deletion, and zero-annotation
operation on arbitrary crates — the weekly sweeps stay until they go quiet on the
classes schemata covers, and every weekly survivor is a signpost naming the next
function to instrument.

## Candidate: shape pressure in the hook voice

Field feedback from an agent running the hook day to day: the voices pay their way —
cheap, attached to the real changed files, never a separate review ritual — and the
one addition it would want is SHAPE PRESSURE: a whisper naming which seam to split
next when one file accumulates several operator families; not a hard rule, ranked
coupling evidence. The machinery half-exists: cohesion reports and the placer already compute
clusters and placements; the ticker already narrates shape events on theory edits.
Shape pressure is their composition at edit time — when the edited file's derived
clusters exceed one, the hook whispers the ranked split (which operators cohere,
which seam the placer would draw), advisory like every voice, priced in silence.

## Candidate: the audience is the agent

Adoption of this method is chosen inside agentic sessions, by whichever instance is
advising on stack — not by a programmer who hand-reviews everything (whose bottleneck
the method exists to dissolve, and who is fine without us). That inverts the usual
promotion problem: the reader is a model with a context budget, a taste for verifiable
claims, and no patience for marketing. What would land: a front door sized for one
context window that states the authority inversion once (derive → freeze → ratify the
diff); claims that are CHECKABLE from the tree in seconds (`cargo run --example gate`,
the committed locks, the fire-drill); and the adoption gradient already proven by the
field reports — bricks first (a hook, a register, a judge), worldview later. The
crates.io publish is the first prerequisite (done); the one-page mental-model document
is the open one.

## Candidate: the sweep discovers, the oracle proves sensitivity

The whole candidate in one line: **the sweep discovers the laws, the oracle proves they are
sensitive.** What a consumer is handed is a GUARANTEE, stated the way they would hear it:
swap this module's implementation for any other with the same interface, and a law refuses
the swap — we catch it. The method's honesty bounds that promise exactly, and the bound is
not a footnote but a COMMITTED artifact: the swap is caught wherever it differs from the
real behaviour at a grid coordinate some law pins; it is missed only where the difference
lands entirely in a ratified DEGREE OF FREEDOM (a named dent survivor — the coordinate no
law constrains) or is a SYMMETRY of the spec (a coordinated change every law tolerates —
meaning pinned up to isomorphism, the theoretical ceiling). Both exception classes already
live in `spec/<theory>.mutation.spec`, enumerated and ratified. So the guarantee ships with
its own fine print: green is evidence over the enumerable grid, the survivors ARE the list
of swaps we would miss, and there is nowhere else for one to hide.

Two distinctions keep that guarantee honest. First, DETECTION is not sampled: what catches
a swapped implementation is `check(discovered laws)` run against it over the bounded
term-CLOSURE of the grid (seeds plus every operator composition up to the depth bound —
`surgical_eval` answers on whatever intermediate values `check` feeds it), so any law it
violates anywhere in that closure is caught deterministically, not at coordinates we
happened to try. The dent/deaf SWEEP is a separate, offline thing: evidence that the laws
are worth running — tight enough to catch meaningful deviations, with the gaps enumerated
where they are not. Second, the grid is the JUDGE's observational universe, not a limit of
the oracle: a swap that differs from the real module only OUTSIDE the closure is, by this
repo's observation-function quotient, the SAME behaviour as far as the spec is concerned —
not a coverage gap but the definition of what the spec means, and the reason arbitrary-CODE
oracles add no detection power over perturbed tables (`check` cannot tell apart two
implementations that agree on the closure). The bound widens MONOTONICALLY with the grid —
more inhabitants, more rounds — so the guarantee only ever grows, never silently shrinks.

A different, blunter guarantee is possible and worth NOT conflating: a DIFFERENTIAL mode
that generates oracles as actual code (or fuzzes the real module) over an open input space
and compares against the real implementation directly rather than against the laws. It
catches off-closure differences the spec is blind to, but it has no spec to consult, so it
OVER-reports — flagging every behavioural difference including the ratified degrees of
freedom the spec deliberately left free. It answers "does this differ anywhere?", not "does
this violate the contract?" The spec-check guarantee is the right default (contract
violation, enumerated fine print, monotone in the grid); the differential mode is an opt-in
second net answering a weaker question.

Deriving a consumer's spec is BUILT: discovery runs their code, `spec-lock` freezes the
laws/census, genesis scaffolds the suite, drift is gated. That half needs no new brick.
The half nobody outside this repo can reach is the one the whole method turns on — *are
the probes you have SENSITIVE?* — the mutation-adequacy question. Internally we answer it
by mutating our own code: `discover::schemata` (the compiled population) gates every PR,
`discover::mutation` rides every `cargo test`. Both edit an implementation and ask "did a
probe catch it?", and both inherit mutation testing's tax — a survivor's spec-status is
unknown, so every one needs the equivalent-mutant ratification dance.

The candidate answers the sensitivity question WITHOUT mutating anything, which is what
lets it replace mutation testing on PRs rather than merely speed it up. Do not perturb the
consumer's source (invasive — needs their build, our attributes in their tree, one process
per mutant). GENERATE an alternative behaviour as DATA — a sampled state machine, an
operator table, a synthetic conduct — CONSTRUCTED to violate a named law L (or to conform
to all of them), and check the probes PARTITION it: refuse the constructed-to-violate
oracle, hold on the conforming one. A probe suite that fails to refuse a
constructed-to-violate-L oracle is DEAF to L — named, no litigation, because the oracle
carries its ground-truth label BY CONSTRUCTION. That is the move that dissolves the
equivalent-mutant problem: you generate against the spec boundary, not the code's
neighbourhood, so a behaviour is admitted or forbidden by definition, never an unknown to
ratify. And because the oracle is data judged by the frozen spec (`Engine::check` over a theory
carrier), never code inside the consumer's module, it "could be compiled and run
anywhere" — no per-mutant build, no coverage map, no process fan-out, no attribute. That
is precisely why it can be a PR gate without the schemata machinery.

The core is not a future generalisation — it is BUILT and rides every `cargo test`:
`discover::mutation` already IS this. A mutant there is a perturbed operator TABLE (a
`Deaf` constant, or a one-point `Dent` returning a wrong value at one grid coordinate) —
a behaviour as DATA, labelled by construction — installed over the real evaluator and
judged by `Engine::check` over the laws that name the operator, the real `fn` never
touched. A surviving dent is the insensitive coordinate: the input no ratified law pins.
So "generate a behaviour constructed to violate a law, check the spec partitions it" is
the dent sweep, running today for every declared theory (fabric's twenty-two, the bridge,
the protocol). Its reach is EXACTLY discovery's reach, because it is discovery's own
judgment pointed at generated tables: wherever a carrier is enumerable (`#[derive(Shaped)]`
gives a grid and a function becomes a finite table) it works, and where the carrier is not
enumerable there is no table, no discovery, and no oracle — the same boundary discovery
has always had, not a new one.

That reach reframes the two consumer cases. The NON-Rust encoding — `theory!`,
`protocol!`, `bridge`, `fabric` — is already done, because the declaration bounds the
carrier and the sweep runs on it now. The NATIVE-Rust case is where the one gap sits, and
it is narrow: a plain module's public functions over `Shaped` types already carry
everything the `Theory` trait needs (sorts are the distinct `Shaped` types in the
signatures, `inhabitants` is the grid, `observe` is the value's own observation, operators
are the functions with their `eval`), so the missing brick is an AUTO-LIFT — synthesize the
`Theory` impl from the inferred module surface — after which discovery and the dent / deaf
sweep run with zero declaration and the consumer has written only types and Rust. Minting a
table that fails one NAMED law (the inverse of `check`) is a reporting refinement on top,
not a missing capability: the blanket dent sweep already finds every insensitive
coordinate.

Honest frame, the method's own and unsoftened — but NOT the two-regimes hedge it is tempting
to reach for. There is no separate "implementation neighbourhood" that mutation keeps
covering and the oracle grid does not: the neighbourhood is ALREADY sampled by the probes
that survive the sweep. Making the sweep go green is a one-time selection pressure that
leaves the probe set sensitive; a passing suite CARRIES that coverage as a property, not as
something to redraw every PR. And the only part of the neighbourhood that matters is the
part that crosses the spec boundary — a mutant that does not cross it is an equivalent
mutant, i.e. noise. So the meaningful mutation coverage and the spec-boundary coverage are
the SAME coverage, and the oracle grid samples it directly. The one real caveat is the
shared one: the oracle grid is bounded, so a suite that partitions every sampled oracle is
EVIDENCE of sensitivity over that sample near each law's boundary, never proof it catches
everything (the green-is-evidence discipline discovery and the sweeps already live under).
This removes the coverage-based reason to keep the internal schemata engine standing: its
job was to make the probes sensitive once, and the oracle-partition maintains that against
the spec going forward. What residual role schemata keeps, if any, is a bootstrap or a
periodic audit of the interpreter that judges the oracles — and even that interpreter is
generation-tested, not mutated (`discover::floor` / `discover::relation` are GENERATED
against), so it is not a standing mutation dependency. Retiring schemata is on the table,
not deferred by a coverage argument.

The consumer writes NOTHING new — the zero-annotation posture the rest of the method
already holds: the public surface is auto-inferred (the qualify census), the grid comes
from their types, discovery finds the laws, and the sweep judges generated tables against
them. So the remaining work is one brick and a few genuinely-open questions, no longer the
whole engine:

- **The brick: auto-lift** — synthesize the `Theory` impl by SCANNING the tree the qualify
  census already walks: functions over `Shaped` types → operators, the `Shaped` types →
  sorts and the grid. No macro, no `lift!`, no declaration — the consumer writes types and
  Rust and the surface is read, the same zero-annotation inference the census already does.
  This is the whole distance between "built for our theories" and "a Rust consumer runs it
  on their own PRs having written nothing extra."
- **Open: the enumerability edge** — the honest limit, inherited from discovery: a public
  function over a non-`Shaped` carrier has no table to lift, so auto-lift must name what it
  skips (the census move) rather than silently cover a subset. How far the common Rust
  shapes reach before that edge is the thing to measure first.
- **Open: the entry point** — a library call the consumer wires into their suite, or a
  genesis-emitted gate riding `cargo test` like the internal sweeps.

Not open, contrary to an earlier draft: the oracle representation (a perturbed table) and
minting a violate-the-law oracle (the dent sweep) are BUILT. This is the single change that
most moves the consumer experience — it takes the sensitivity guarantee we keep for
ourselves and makes it a thing a downstream crate runs on its own PRs, with no mutation
tooling in its tree at all.

### The end state: one probe census, every probe sensitivity-proven

The deliverable is not "some autogenerated probes, mutation-tested." It is a UNIFIED PROBE
CENSUS for a crate — every probe the module upholds, structural and behavioural, each
carrying a sensitivity proof — and the degrees of freedom enumerated. Two families, and
the split matters because sensitivity is ONE concept ("can this probe fail? — a green probe
that cannot is a lie") with two mechanisms by probe kind:

- **Structural probes** — what the module upholds by its SHAPE: the public-surface census
  (`qualify.spec`), the tier partition (`tiers.spec`), the placement (`shape.spec`), the
  boundary grammar, the seams. Sensitivity is FIRE-DRILL: plant a known-bad fixture, demand
  the gate refuses.
- **Behavioural probes** — what it upholds by its CONDUCT: the discovered laws
  (`<theory>.spec`). Sensitivity is the ORACLE SWAP: plant a perturbed behaviour, demand a
  law refuses (`<theory>.mutation.spec` is exactly this today).

Fire-drill and the mutation sweep were built separately — one for pipelines, one for
theories — and never said out loud to be the two arms of one thing: the vacuousness hunt.
Both halves already restrict to the FOUND set for performance (discovery keeps only holding
laws; the sweep perturbs only operators a found law names), so the census is a subset of a
fully-enumerable probe space, taken on the real set. Pre-enumerating EVERY probe in this
crate with its sensitivity verdict is the dogfood and the existence proof — the unified
census run on ourselves, the way `downstream-fixture` proves the public-API loop — and it
is the same census a consumer gets pointed at their module, having written nothing. That is
the logical end state: the judges collapsed to data plus one interpreter, the sensitivity
proofs collapsed to one question with two mechanisms, and the whole thing a census a
downstream crate can compute about itself.

**Rung 1 is BUILT** (`discover::probes`, `spec/probes.spec`). The census now exists and
enumerates every frozen probe lock with its sensitivity mechanism: the eight behavioural
theories (the seven in `all_specs()` plus the mounted bridged theory) as `oracle-swap`; the
three world judges (`perimeter`/`infra`/`substrate`) as `live-dent`; and the byte-locks
(`surface`/`tiers`/`shape`/`seams`/`catalog`/`pipeline`/`schemata`/`world`) as `drift-gate`.
It is gated for COMPLETENESS against the committed `spec/` directory — every artifact that
backs a probe must be covered and every named probe must have a committed lock, so a new
lock that no probe covers fails the census, and the roster can only grow honestly (the
tiers-ladder rung-1 move: derive-and-disclose, no coherence to argue with). The disclosure
IS the finding, and reading the fire-drill battery sharpened it: the byte-locks are not
sensitivity-UNPROVEN — they lean on the SHARED `spec-lock drift gate` fire-drill (a
tampered or missing committed lock is caught), which proves the drift mechanism fires but
not that each lock's own derivation is individually sensitive.

**Rungs 1.5 and 2 are BUILT too.** Rung 1.5: `Mechanism::FireDrill` now distinguishes the
probe with an INDIVIDUAL drill (the catalog — the `shape data gate` and `expectation
vocabulary` drills) from the byte-locks on the shared gate, and the review router files
`spec/probes.spec` under its own `Ratification::Probes` class (not the `.spec` catch-all).
Rung 2: the census is NORMATIVE — `every_probe_has_an_individual_drill_or_is_ratified` holds
the drift-gate set to `spec/probes.register` by SET DIFFERENCE, so a byte-lock leaning only
on the shared gate must be a ratified line with a reason (an un-ratified one refuses, a line
for a probe that earned an individual drill is stale), and the drift-gate set can only
shrink as byte-locks earn their own drills.

The register already SHRANK 7 → 4 on first honest review — the tiers-ladder lesson ("was
the derivation missing evidence?") applied to sensitivity. Reading the suites showed the
seam graph, the placer, and the world lock each ALREADY carry a plant-a-bad-fixture drill:
a non-homomorphic conversion leaves the transform seam UNEARNED and names it
(`a_broken_conversion_leaves_the_transform_seam_unearned`), a placement disagreeing with the
declaration renders "DISAGREES" and is unsettled (`a_disagreeing_shape_renders_loud`), a
broken vendor / mismatched battery is refused and named
(`reports_over_different_batteries_are_refused`). All three moved to `Mechanism::FireDrill`
(joining the catalog). Then pipeline and schemata earned individual drills too — a
`Pipeline` declaring a bespoke-tier cadence refuses to render a consumer workflow
(`Pipeline::render_workflow`), and the completeness census names a planted uninstrumented
function (`schemata::uninstrumented`, extracted to a shared `cfg(test)` helper) — both wired
into the rung-3 battery. So only TWO byte-locks now lean on the shared gate: surface and
tiers, whose derivations are mutation-tested cross-crate in `boundary-enforce` (a ratified
reliance, not a debt — an in-crate drill would duplicate coverage that already exists one
crate over). That is the register's honest floor.

RUNG 3 (BUILT): the `FireDrill` claims are now NON-VACUOUS.
`every_fire_drill_probe_has_a_live_drill_that_fires` reconciles the census's FireDrill set
through a `fire_drill::Battery` that `requires` exactly those keys and drills each by
PLANTING the probe's known-bad fixture — so `verdict()` refuses a FireDrill probe with no
live drill (UNPROVEN) and a drill that stopped firing (VACUOUS). A deleted drill breaks the
gate; the "a FireDrill claim pinned only by a test that could vanish" gap is closed. Each
module owns its drill (the seam graph's non-homomorphic conversion, the placer's disagreeing
placement); the catalog and world drills build from the public API.

AUTO-LIFT (BUILT — the headline): the apparatus now points at a consumer's PLAIN RUST
module. `discover::lift` is a generic `Lifted<C>` — a single-carrier `Theory` over any
`Shaped` carrier, one sort, the carrier's `shadow_grid`, identity observation — written
ONCE; the only per-module thing is the operator table (`impl Liftable`: names, arities, thin
wrappers). So a consumer who wrote only ordinary Rust gets `Spec::of::<Lifted<C>>()` (the
probes) and `MutationReport::of::<Lifted<C>>()` (their sensitivity proof), zero declaration.
The worked example runs it end to end: a plain `bool` module (`and`/`or`/`not`/`tru`) lifts,
discovers its algebra (≥3 laws, no uncovered operators), and is sensitivity-swept (deaf
floor + dents, deaf mutants caught). `AutoLift::scan_module` is the build-time half — it
`syn`-scans a module's public functions, infers the single carrier (a second carrier is a
named refusal — multi-sort is out of scope), and generates the `impl Liftable` a consumer's
`build.rs` `include!`s; a reconciliation test ties the scan's output, by (name, arity), to
the runtime-proven table, so the generated table IS the proven table and the only glue left
is the `include!`. And MULTI-SORT is built too: `Lifted2<T>` lifts a module over TWO
`Shaped` carriers — a tagged `Either` value, a `Duo` sort, per-sort grids from each carrier's
`shadow_grid` — so a cross-sort map and a ROUND TRIP are expressible. The worked example
(`bool ⋈ Box<bool>` with `wrap`/`unwrap`/`both`) discovers the cross-sort round trip
`unwrap(wrap(x)) = x` no single-carrier lift can state, and is sensitivity-swept. Scope,
disclosed: a non-`Shaped` carrier has no grid, exactly where discovery itself stops. The
build-time scan (`AutoLift::scan_module`) now emits BOTH the single-carrier `impl Liftable`
AND the two-carrier marker `struct` + `impl Liftable2` (each op's slots tagged by their `Duo`
sort, an `Either`-unwrapping wrapper), reconciled by (name, input sorts, output sort) against
the runtime-proven `BoolBox` table — so for one and two sorts the zero-annotation loop is
closed end to end, the only glue being the consumer's `include!`.

THREE OR MORE distinct carriers is a principled named refusal, not an oversight: arbitrary N
cannot reach the same rigor as one/two without variadic generics — it needs per-N codegen (a
`Lifted3`/`Lifted4`/… ladder, each mechanical but bounded, or a scan that emits a bespoke
`theory!` invocation, which the engine's existing multi-sort theories prove viable but which
a unit test can only parse-check, not run). And the practical pressure is low: N conceptual
sorts modelled as ONE `Shaped` enum are already single-carrier `Lifted`. So the honest end
state is: one and two distinct carriers fully built and proven; three-plus a named refusal
with the two paths (the `LiftedN` ladder, the `theory!`-generating scan) recorded for when a
real consumer needs them. This is the consumer end state the "sweep discovers, oracle proves
sensitivity" arc was aiming at: probes derived from a plain module — single- or two-sorted —
each proven sensitive, with nothing written.

Remaining, disclosed: three-plus DISTINCT carriers (the `LiftedN` ladder or the
`theory!`-generating scan — a principled named refusal until a real consumer needs it, since
N conceptual sorts as one enum are already single-carrier `Lifted`). The byte-lock register
has reached its honest floor — surface and tiers, covered cross-crate.

## Framing: the real domain is stability under containment

Worth stating plainly, because it reframes what the method is FOR. Probe-algebra's real
domain isn't "migration" or "diagrams" or "HTML". It is stability verification over any
DECLARATIVE-MODEL → CONSTRAINT-LAYOUT pipeline where containment makes local edits have
non-local — and sometimes LEGITIMATELY non-local — effects. The hard part of every such
pipeline is the same: a local change ripples through the layout, and you cannot tell a bug
from a correct-but-far-reaching consequence by looking at the diff. That is exactly the
distinction the apparatus already draws — a refuted law is the bug, a surviving dent is the
legitimate degree of freedom — so the method is not analogising to these domains, it is
their native shape.

Three instances make the generality concrete, and their ORACLES are what differ:

- **CALM + ELK** (architecture model → graph layout): a node moves and the whole diagram
  reflows; which reflows are correct is a stability question, not a structural one.
- **DOM + CSS** (document model → box layout): an element's style changes and containment
  propagates; the same non-local, sometimes-legitimate effect.
- **Migration** (schema/model → byte-identical SQL): the THIRD instance, and the reason it
  worked first and cleanly is that its oracle is TRIVIAL — byte-identity is a total,
  decidable equality, so there is no "sometimes-legitimate" band to adjudicate. The layout
  instances have no such luck; their oracle is metamorphic (rename-then-render =
  render-then-relabel) and toleranced (insertion locality within ε), which is precisely the
  vocabulary `layout-probe` built.

So the layout-probe brick is not a second domain bolted on — it is the method meeting its
general case, where the oracle stops being byte-identity and becomes a metamorphic /
toleranced judgment over containment. The consumer guarantee reads the same in all three:
swap the pipeline's implementation, and we catch the swap unless the difference is a
ratified degree of freedom (a legitimate non-local effect) or a spec symmetry. Migration
just happens to be the instance where that fine print is empty.

## The lock narrates its own movement: `delta()` (BUILT)

The build/text boundary, resolved from the emitter's side. Distance, cohesion, and
placement need the compiled theory (running `eval`), so a text edit cannot re-derive them —
but the drift gate already re-derives them on every freeze, and at the compare it holds BOTH
sides: the committed text and the freshly derived live text (`spec-lock`'s `check`, the line
`committed == lock.live`). The gate collapsed that to a bool and threw the difference away.

`spec_lock::Lock::delta` keeps it. For a lock whose lines ARE recommendations — the shape's
placement verdict (`7 of 7 settled` → `6 of 7`) and seam candidates, the tier assignments —
the committed→live line diff IS the updated recommendation, and it is produced by the same
run that emits the lock, from the two sides that run holds at that instant. Nothing watches a
file write; nothing reconstructs a diff from git after the fact. The multiset line diff
(`LockDelta::between`, blank lines dropped) is the same set-diff honesty as `RegisterDrift`,
one altitude down: an in-place line cancels, a moved line shows as one removal and one
addition.

The wiring closes the loop without moving the derivation off the build:

- **`examples/freeze_spec`** captures `spec_lock::deltas(&locks)` BEFORE `bless` overwrites
  the committed side, prints the movement for whoever ran the freeze, and writes the rendered
  narration to `target/probe-hook/freeze-delta` (empty on no movement — a stale delta never
  lingers).
- **probe-hook** gains a FOURTH voice, the freeze-delta courier — the only one it does not
  compute. It reads that file, inserts the movement ONCE into the next context window, and
  clears it. The mechanism is native to `delta()`, run at freeze time; the hook is only the
  wire. This is the honest resolution of "should the hook re-derive cohesion" — no: the hook
  cannot afford `eval`, so the recommendation movement is derived where it is cheap (the build
  that emits the locks) and couriered to where it is read.

Why this shape and not a session-start `git diff` of the shape lock: a git diff would
RECONSTRUCT downstream a delta the emitter already had in hand and deleted. Surfacing it from
`delta()` keeps one derivation, at the build, in the grain of the one rule — the committed
diff is the ratification, and now the run that produces that diff also narrates it.

### The tier voice reads its rules from the lock (dogfooding caught the last copy)

Exercising the installed hook surfaced an oversight against our own "derive everything from
the locks" claim: `tiers.spec` carried the DATA (which file is which tier) but the tier voice
recited the MEANINGS ("KERNEL — the trusted floor, exempt from the structural rules") from a
`match` in `probe-hook`'s own source. So the lock never went stale, but the reader did — an
older binary against a fresh lock would recite rules that no longer matched what the enforcer
forbids. Fixed by rendering a `# rule <TIER>:` legend into `tiers.spec` (`boundary-enforce`
owns what a tier forbids, so it owns the legend), and having the hook READ the matching line
instead of holding a copy. Proven live: reword the legend in the lock and the hook echoes the
new words; a lock without the legend names the tier and points at regeneration rather than
guessing. The remaining reader-skew is the linked `boundary-spec` (the guard and ticker are
called at probe-hook's own pin, not the consumer's) — the honest fix there is re-execing the
repo-pinned build, the same nicer form the crate docs already flag, deferred until skew is
seen hurting.

## The sixth sense: the coupling ticker on any Rust edit (BUILT)

The build/text split said structure is text-derivable and behaviour needs the build. The
coupling recommendation (split / cohesion) is STRUCTURE — placement is "two operators share a
net when a sort appears in both signatures," pure signature text — so it never needed the
build, and it is most useful at the moment of the edit, when intent is freshest ("this
function I just wrote couples Order and Invoice — intended?"). The ticker already proved the
text path for theories (`parse_ops`, no compile); this widens it to a THIRD front,
`parse_rust_sigs`: an ordinary module nets on its OWN declared types (structs, enums — the
plain-Rust analog of a theory's sorts), so its functions place into clusters and an edit that
first spans two of them BRIDGES them.

The noise answer is the whole design: ubiquitous types (`String`, `Result`, `Vec`, generics,
references) are NEVER nets — netting on them would wire every function to every other and drown
the sense. A function is an operator only when it mentions one of the module's own types (Self
resolved to the impl target). The existing noise policy carries over unchanged: an edit inside
one cluster is silence, only a bridge or a new net-disjoint component speaks. Both fronts share
one core (`hook_line_signatures`), so theory and plain Rust get the identical live sense, and
the hook's `shape_voice` picks the front by content (`ops {` → sorts, else own types).

What stays on the build, correctly: distance, discovered laws, cohesion-as-behaviour — they
need `eval`, and mid-edit the code does not even compile, so a behavioural verdict then would
be noise about a half-written function. The courier (above) delivers those. So the edit hook
now carries the STRUCTURAL half as a sixth sense, and the build carries the BEHAVIOURAL half as
a narrated delta — the split the build/text boundary always implied, finally on both sides.

Follow-up worth naming: the plain-Rust net model is a first cut. It nets on the module's own
types by name; it does not yet see cross-module coupling (a function wiring THIS module's type
to an imported one), and name-collision across modules is possible. The theory front has the
compiled second source to cross-check against (`step_theory`); the plain-Rust front has only
text. A build-time cross-check (parsed placement vs a compiled reachability view) is the honest
next tightening, deferred until the text model is seen misleading.

## Standing follow-ups

- ~~**Tag `v0.1.0`**~~ — superseded by AUTOMATIC RELEASES (the `release (certified
  tree)` gate, the registry's first Effectful entry): every countersign that advances
  `mutants-green` publishes the certified tree as a CalVer-tagged GitHub release, with
  notes derived from the commits and the ratified spec-lock diff. No semver: a version
  integer is a hand-asserted compatibility claim nobody checks — the lock diff is the
  same information, uncompressed and verifiable. Consumers pin a release tag and read
  exactly which laws moved between any two of them.
- ~~**Publish** when ready~~ — PUBLISHED 2026-07-07: all four crates
  (`spec-lock`, `boundary-spec-macros`, `boundary-enforce`, `boundary-spec`) live
  on crates.io at 0.1.0, in the runbook's dependency order. The root crate's
  registry verify build resolved the three from the LIVE index and ran every
  enforcement pass over the packaged tree — the real consumer path, green at
  publish time. Still owed: a `v0.1.0` tag at the published tree (the publish
  session's push scope was branch-only, so tags could not leave it) —
  `git tag v0.1.0 <sha of "Publish-readiness: version the spec-lock
  build-dependency"> && git push origin v0.1.0`; the tag sits outside
  `release.sh`'s `v2*` CalVer glob, so the two tag families cannot collide.
- **Morphism downstream**: the fixture exercises Construction/Branch/Guarded;
  the fourth edge shape is honestly unexercised downstream.
- **MSRV**: unpinned; verify and add `rust-version` after first publish.
