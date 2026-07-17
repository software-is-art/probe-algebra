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

## Candidate: differential-certify — the published artifact as the judge (the bootstrap floor)

The `#[mutate]` retirement (above, "on the table") stalls on ONE residual role: source
mutation of the non-theory PLUMBING — the parser, the engine's structural transforms, and at
the bottom the interpreter that judges the oracles. Two obstructions were named for it: the
plumbing's carrier is structural (ASTs, theories) not a flat grid, and its spec is
EXTENSIONAL (the frozen bytes on the repo's own sample) not a law-set. The first dissolves —
enumerate the STRUCTURED side (bounded ASTs, built as data) and lift with structural laws
(round trip `parse ∘ render = id`, idempotence, the placer's already-named fixed point). What
survives as genuinely irreducible is smaller than "the plumbing": it is the JUDGE. To
oracle-sweep a core function you need a grid and an observation that must not route through
the function under test; for the interpreter, the observation IS the interpreter, so using it
to validate itself is circular — the proof-checker-kernel / self-hosting-compiler floor.

The move that unties the knot: **the last certified release is the independent judge.** A
published release binary carries auto-lift, was built from different source, is frozen, and
already passed certification (mutants-green). So release N's core is judged by release N−1's
binary — "X validates X" becomes "X_{n−1} validates X_n", a regress with a base case, i.e.
INDUCTION. The predecessor's auto-lift ingests the current core's plain-Rust surface, lifts
it, runs discovery + the oracle sweep, and the PREDECESSOR's engine renders the verdict. No
self-reference. This is the repo's existing SECOND-SOURCE pattern (the compiled theory as the
ticker's second source `step_theory`; the crates.io index as the substrate's second source)
applied to the bootstrap floor itself — idiomatic, not a bolt-on.

And the "mutants" get strictly better. Synthetic `#[mutate]` flips carry the equivalent-mutant
problem (is this flip even meaningful?). Here the perturbation under test is the REAL
inter-release diff — every change is one a human actually made, so there are no equivalent
mutants by construction. That is the method's own "generate against the spec boundary, not the
code's neighbourhood," reached from the other side: the boundary crossed is the diff since the
last green.

Proposed gate — `differential-certify`: (1) pull the latest certified release binary; (2)
auto-lift the working-tree core against it; (3) any un-ratified behavioural divergence is a
red check. Divergence routes into the EXISTING discipline: an INTENDED change (new law shape,
fixed bug, deliberate semantics move) makes the predecessor disagree, and that disagreement is
admitted iff it drifts a lock the commit ratifies — "differential divergence → ratify a lock
diff" is the same shape as "mutation survivor → ratify a degree of freedom", so the
ratification machinery already exists. An UNINTENDED divergence is a regression the certified
predecessor catches for free.

Honest limits, none fatal but all load-bearing:

- **Differential, so blind to common-mode faults.** The predecessor is independent w.r.t. the
  DIFF since that release, not w.r.t. shared ancestry. A bug present in both N−1 and N
  (inherited, never noticed) produces no disagreement — the standard N-version correlated-fault
  hole. Only re-baselining against an independently-reasoned implementation reaches it.
- **The floor moves; it does not vanish.** Release 0 was validated by a non-self method
  (`floor`/`relation` are generation-tested, not mutated). The trick AMORTIZES that expensive
  check to the base case and periodic re-baselines; the root is still there. And it inherits
  TRUSTING-TRUST: a compromised release validates its successor into agreeing with the
  compromise. The defense is provenance — the past binary reproducible from frozen source under
  an independent toolchain; the substrate lock is the start of that story, not the whole of it.
- **Auto-lift reach and version skew bound what the predecessor can see.** N−1's scanner lifts
  N's surface only as far as auto-lift reaches (`Shaped`, ≤2 carriers) and as far as N−1
  recognizes N's syntax; the higher-order judge core still needs the hand-built AST-grid-as-data
  lift (the predecessor is its JUDGE, but does not auto-generate it). A large refactor closes
  the adjacent-release window.
- **Per-diff, not whole-code.** It exercises only code that CHANGED; unchanged core "agrees"
  trivially and is covered transitively (checked against its own predecessor when introduced) —
  the same "coverage carries forward as a property" the oracle sweep already runs on, but a
  DIFFERENT guarantee than synthetic mutation's "every line is observed by some probe". It PAIRS
  with the probe census (Rung 3's "a deleted drill breaks the gate" catches a diff that WEAKENS
  coverage of unchanged code — behaviour would not diverge, but the census fires), it does not
  replace it.

The one thing it does not buy, stated so it is not oversold: escape from the base case.
Someone, once, still trusts release 0 by a method that is not "ask release 0." Everything after
that, the published artifact can carry.

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

## The edit-time lock delta: the qualify census re-derived at the edit (BUILT)

Why the spec-lock workflow lets review be coarse: every behavioural change surfaces as a lock
delta, so reviewing the diff of the locks reviews the behaviour — and, because each probe carries
a can-fail proof (a green probe that cannot fail is a lie), the mirror is complete over what it
locks. Agents don't err on purpose; they err where nothing reflects the mistake back. So the
lever is reflection LATENCY: push each mirror as close to the edit as it will go. The coupling
ticker moved the STRUCTURAL reflection (placement) to the edit; this moves the first LOCK-DELTA
reflection there too.

The behavioural locks can't come along — distance, discovered laws, cohesion need `eval`, and
mid-edit the code doesn't compile — which is why the freeze-delta courier exists: it derives those
deltas at the build and couriers them into the next window. But one lock is not behavioural:
`spec/qualify.spec`, the surface census, is a STRUCTURAL property (a module qualifies when its
functions are operator-shaped — bare named value types, no I/O), so its line is text-derivable
from the file alone. `boundary_enforce::qualify_line` / `qualify_census_lines` are that census's
OWN emitters, factored to ONE source (the operator-shape rule is stated once and served to both the
frozen census and the hook — never restated), and the hook's fifth voice re-derives the whole live
census from the tree, diffs it against the committed lock with the same `spec_lock::LockDelta` the
courier uses, and shows the current drift. So the agent learns "this edit made the qualify census
stale" AT the edit, where the intent is freshest, instead of at the red gate the build would
otherwise be first to raise.

Two shape decisions, both made against how it reads to the agent consuming it:

- **A drift LEDGER, not a per-file delta.** It shows the WHOLE current drift — every stale line
  together — so accumulated movements sit in one block, and it goes empty the moment a re-bless
  reconciles the tree. You never lose an outstanding line between the edit that caused it and the
  bless that clears it.
- **Un-scoped but DEDUPED.** The ledger is a repo fact, so it is surfaced on ANY edit, not narrowed
  to a file class (which file you touched should not gate whether you see the drift). Breadth of
  triggering would become breadth of repetition, so the voice persists the last-shown ledger and
  speaks only when the render CHANGES: an accumulated line appears once, on whatever edit first
  surfaces it, then stays quiet until the drift moves again. That is the un-scoped reach without the
  banner-blindness a reprint-every-edit block would train.

NO how-to-bless recipe rides the voice: the delta names `spec/qualify.spec`, whose own header
carries `# … Regenerate with \`BLESS_QUALIFY=1 cargo build\``, so the command is self-documenting at
the named lock and is stable orientation (also CLAUDE.md's one rule) — not news to reprint each
firing. A half-written file contributes nothing to the live census, so a mid-edit save never
invents a movement.

The split is now explicit and on both sides: the STRUCTURAL half of the mirror (placement,
qualification) lives at the edit as a live sense; the BEHAVIOURAL half lives at the build and is
couriered forward. Follow-up worth naming: qualify is the ONLY committed lock a single text edit
can honestly re-derive today. `tiers.spec` is text-derivable but CROSS-file (a file's tier depends
on the dependency graph, not its own text), so an edit-time tier DELTA would need the whole tree,
not one file — deferred until that's worth the walk. Everything behavioural stays couriered by
design, not omission.

## The thirteenth ask: reliances go native — locks ship in the crate (points 1 + 3 BUILT)

From the chattel-cli consumer instance, replacing the fixture lines it almost upstreamed.
The tenth ask built the downstream register so open text stops feeding the rules — but it
left the DATA in the wrong repo: upstream holding a mirror of consumer facts only the
consumer can know is stale, plus a hand-off (or PR access) for every line and every
deletion. The native mechanism already existed one level down in the same stack — the
consumer's own CLI judges ITS clients' registers against the theory lock embedded in the
binary they pin — so probe-algebra now does to itself what its consumer already did.

**Point 1 — the locks ship** (`boundary_spec::Locks`, living in `discover::depend` where
the judgment that consumes them lives): every behaviour lock, its algebra-mutation
companion, and the shape catalog, `include_str!`-embedded byte for byte and keyed by lock
stem. "The version is the certification" becomes an API — a consumer pinned to a tag holds
exactly that tag's certification data, no filesystem archaeology, no release-notes
parsing. The roster is DELIBERATE, not "everything": the mutation locks ride because the
ratified survivors are the guarantee's fine print (the named degrees of freedom a swap
could hide in), and `shapes.spec` rides as the law-language the equations are written in —
while repo-meta locks (gates, tiers, qualify, probes, schemata, perimeter, substrate, the
world lock, the infra exemplar, the seam graph and shape locks) and every hand-authored
register stay home. A shipped lock is a COMMITMENT: embedding a repo-meta artifact would
invite reliances on interior facts and make every interior reorganisation a potential
downstream refusal. The roster is census-gated in both directions, with what ships
DERIVED from the artifacts' own headers (`# discovered spec:` / `# algebra mutation:`,
plus the catalog): a registry theory missing its embed line refuses, an unclassified spec
artifact refuses, a stale home-line refuses, and the embedded bytes are pinned to the
committed files (locally a tautology; in the packaged crate, proof `spec/` shipped
coherently). One free property worth stating: the lock and the parser that reads it
(`Spec::parse_lock`) travel in the same pinned artifact, so grammar skew between lock and
reader is impossible by construction — the release-notes-parsing route could never
promise that.

**Point 3 — reliances live consumer-side** (`Dependence::judge_embedded`): a consumer
commits `upstream-reliances.register` (the existing grammar, unchanged:
`<theory> | <equation, exactly as the lock renders it>: <consumer> — <why>`) and judges
it in its own suite against the pinned crate's embedded locks. The cross-repo form
collapses to the self-judgment form because the lock travels with the pin: a pin bump
re-runs the judgment against the new version's locks with zero ceremony, and a bump to a
version that dropped a relied-on law refuses by equation, carrying the consumer's why,
before a bare compile error explains nothing. Under the hood one shared core
(`judge_reliances`) serves both register judges — one grammar, one refusal envelope, two
lock resolvers (the spec directory for the owner, the embedded roster for the pin) — so
the two forms cannot drift apart.

**Deliberately not in this brick, each with its disposition:**

- *Point 2 (the API surface as a lock)* — its own brick, next. The tenth ask put API
  reliances in the compiled fixture because the compiler is the strongest available
  judge; once judgment moves consumer-side the consumer compiles against the pin anyway,
  so the surface lock's marginal value is the named refusal (why before the compile
  error) and — the bigger prize — upstream's own public surface becoming a ratified,
  review-routed lock, which `qualify.spec` (operator-shape, not pub-surface) does not do.
  #48 just built the exact pattern it needs: a one-source emitter in `boundary-enforce`
  serving both a frozen lock and a live consumer.
- *Point 4 (the consumer census)* — recorded as a candidate, WITH the observation that
  closes its disclosed gap: the census line is an ADDRESS (name + where the register
  lives), and the world-gate pattern already knows what to do with an address — a weekly
  `reliances (consumer drift)` gate pulling each censused consumer's register (one
  anonymous read, the substrate's crates.io move) and judging it against the WORKING
  TREE's locks gives upstream the release-time half: a breaking re-bless learned before
  shipping, not at the consumer's bump. Floor semantics (an unreachable register is a
  disclosed refusal, never silence); a world fact never feeds the countersign.
- *The fixture shrink* — deferred until the consumer's register lands. The four compiled
  surface reliances in `downstream-fixture/tests/reliances.rs` stay until their owner
  re-declares them downstream; then the fixture shrinks to what it uniquely earns, the
  synthetic consumer proving integration compiles.

**Disclosures, all load-bearing:** renderer stability is now a CROSS-REPO contract —
registers key on the equation byte-exact, so a pure render-convention change upstream
reads as GONE in every consumer's suite, indistinguishable from a dropped law until the
consumer reads the diff (part of why `shapes.spec` ships: the law-language is now
certification surface). This protects at ADOPTION time, not release time — the candidate
above is the remedy — and the incident ledger is clean: no upstream change has actually
broken a consumer yet, so consumer register justifications should say they are foresight,
not scar tissue (the ask's own instruction, kept). `Locks` is certification DATA, not
re-derivation — holding the text is holding what discovery earned at release time, not
the ability to re-run it. And the connection worth a line so it is not rediscovered:
embedded locks are the data half of DIFFERENTIAL-CERTIFY — a published artifact carrying
its own certification is exactly what lets release N−1 judge release N — so this brick
lays the induction rail before that candidate's gate.

## Candidate: the bundle is the source — genesis as a continuation process

Named in conversation with the operator, whose reframe is the headline: genesis is not a
starter template, it is a CONTINUATION process. The diagnosis behind it, in the operator's
own accounting of the agent coding experience: finding context is maybe 10% of the battle
(grep and subagents do fine); the other 90% is fighting entropy — a series of disconnected,
task-specific edits IS tech debt, and pulling the accumulated shape back into a context
window to crunch down is exactly what agents are worst at. The wished-for experience: the
agent writes a SNIPPET OF LOGIC, purely additively, and the apparatus assembles the
modularity — with the inverse sweep collecting disconnected logic back up.

The pieces mostly exist, and one property makes the whole idea safe rather than wishful.
Placement is MONOTONE by union-find's own algebra (candidate 14): a new operator joins a
component or bridges two — it can never re-split or reshuffle what it does not touch. That
is precisely the guarantee an additive author needs: a snippet cannot silently reorganise
the rest of the program, so entropy from ADDITION is structurally bounded. Around that
core: the placer derives the partition from signature text, the sixth sense narrates
bridges at the edit, genesis renders whole trees from declarations, auto-lift reads
operators off plain Rust with zero annotation, and cohesion + the spec's silence line
("operators in no law") are already the disconnected-logic detectors — reports today,
awaiting actuation.

The precondition, named by the operator because it is easy to mistake for a side detail:
this works ONLY because the discipline has no primitive obsession. Nets are SORTS, and
sorts exist only where signatures speak the domain's vocabulary — the sixth sense already
refuses to net on ubiquitous types (`String`, `i64`, `Vec`) because with open primitives
everywhere, placement degenerates to soup (everything wires to everything) or silence
(nothing nets at all). Value objects are not a style choice this candidate happens to sit
beside; they are what gives the netlist its nets. Which re-reads the spike below: pushing
the qualify census toward totality is not just measuring the bundle's territory, it is
MANUFACTURING the signal auto-modularisation runs on.

The real inversion, named plainly: FILE LAYOUT BECOMES A VIEW. Today the placer derives
placement as a report and a lock, but the files are still hand-arranged and the lock only
checks agreement. In the continuation form, the module tree is a GENERATED artifact under
the one rule — never hand-edit, regenerate, ratify the diff — rendered from a BUNDLE
(operators + meanings + expectations) the way `ci.yml` is rendered from the gate registry.
Code as a database, files as its placed render: the same authority inversion as every
brick here, pointed at the last hand-maintained artifact class, source layout itself.

Prior art, honestly cited: Unison built the codebase-as-database experience at the
FUNCTION level — content-addressed definitions, names as metadata, edits as append. The
step beyond is AUTO-MODULARISATION: in Unison the namespace is still the human's to
arrange; here the modularity is derived (net connectivity), continuous (re-placed on every
change), and ratified (the shape lock). Placement is the tool's output, not the author's
input.

The rung ladder, round-trip before generation as always:

0. **The type-library surfacing voice — BUILT** (`probe-hook`'s sixth voice,
   `library_voice`, landed the day after the census learned to see impls — the two bricks
   compose: 16 modules' operator families are now the library). On any Rust edit, the
   types the edited file touches (`Ticker::type_vocabulary` — every type ident its
   signatures mention, own or imported) intersect against the committed
   `spec/qualify.spec`, and files elsewhere whose sorts overlap are named with their
   operator families — the existing vocabulary in the window before a twin gets written.
   The census intersection IS the noise filter (ubiquitous types never appear as census
   sorts), the edited file's own line never speaks, and the render dedupes per file,
   re-announcing only when the overlap changes. The anti-duplication sense, attacking the
   context 10% while the assembly 90% gets built.
1. **Round-trip — BUILT** (`discover::bundle`). A module parses into a bundle (each
   top-level item a verbatim segment; operators identified by the sixth sense's net model,
   grouped by the one placer) and renders back in canonical placed order. The pin is real:
   `modularize.rs` — the committed file whose whole subject is proposing modularity —
   round-trips BYTE FOR BYTE (docs, the attributed inline module, the cfg(test) suite all
   verbatim), and was already canonically placed — checked, not assumed. The
   non-vacuousness drill scrambles two components and demands the regroup; refusals are
   named (unparseable text, a non-unique operator name — the dealing is never guessed).
   One design lesson earned on first contact and worth keeping: SEPARATOR TRIVIA BELONGS
   TO THE POSITION, NOT THE ITEM — doc comments are attributes inside an item's span and
   travel with it, but the blank lines between items are the module's furniture, so a
   reorder dresses the dealt items in the original positions' spacing (the first draft
   moved gaps with items and produced jammed output; the drill caught it). Disclosed
   rung-1 limits: only top-level operator fns are re-dealt (impl blocks and inline modules
   ride whole), and identity is the function name.
2. **The continuation verb — LIBRARY FORM BUILT** (`Bundle::add`; the name `genesis add`
   died with the disposition below — the verb is the bundle's). A snippet plus a module
   in, the module re-rendered in canonical placed order out: the added operator lands
   WITH ITS COMPONENT (the placer's dealing, not an append), a helper type lands before
   the trailing `#[cfg(test)]` module (tests stay last), every existing item's bytes
   survive verbatim (addition is monotone — drilled), and the result is a fixed point of
   parse∘render. Refusals named: unparseable snippet, empty snippet, and a NAME COLLISION
   with an existing item — the type-library voice's whisper made a hard stop (an `impl`
   extends a name, which is precisely not a collision). No I/O in the verb; the caller
   writes the file. The binary wrapper is deliberately deferred until real usage names
   its shape — the library call is what an agent's tooling actually invokes.
3. **The inverse sweep, actuated** — cohesion's latent splits, the silence line, and the
   placer's seam candidates feed re-render SUGGESTIONS. Cohesion stays a suggestion by
   design: the machine executes placement, the human keeps ratifying splits — taste
   enters at the freeze, where it always has.

Disclosures, all load-bearing. The algebra does not reach everything: this repo's own
qualify census is 3 of 55 files, so the bundle governs the operator-shaped fraction and
must NAME what it skips (the census move) — the spike below is the measurement of exactly
that territory. Meanings travel opaquely: a snippet is a signature plus a body; placement
is signature-derived and the body just rides along, which is why the behaviour locks are
not decoration on this candidate but the only reason auto-assembly is trustworthy at all.
And review churn must prove out in practice: if files are views, a re-render moves code,
and the promise that hierarchical ratification plus the review router keep those diffs
readable (placement-only reflow = machinery-verified, one agenda line) is the thing rung 1
and 2 must demonstrate, not assume.

### The disposition on genesis: the bundle continues, genesis births, and the declaration is vocabulary

Settled in conversation with the operator before rung 2's design, because rung 2 must be
designed against it. The question — "do we need genesis once we have this?" — splits
genesis along the method's own IS/SHOULD line:

- **Genesis-the-scaffolder dissolves into the bundle.** Scaffolding is rendering a bundle
  with meaning-holes; migration mode (the fifth field report's largest ask, still open on
  the v2 list) is `Bundle::parse` — adopting an existing module IS what rung 1 does. The
  bundle does not just replace the emitter, it delivers the v2 items that were queued for
  it: patch-not-scaffold, adopt-existing-functions-as-meanings, no holes where
  implementations live.
- **Genesis-the-declaration-language survives, as the bundle's INTENTIONAL half.**
  Everything the bundle/auto-lift path produces is DISCOVERED — what the code is. The
  declaration grammar (`expects { }`, seams, protocol blocks, validity) is what the code
  SHOULD be, and no derivation produces it: without declared expectations there is no red
  target lock, no distance, no unmet law driving work; without declared seams there is no
  obligation, only candidates. The end state: A DECLARATION IS A BUNDLE ENTRY — an
  expectation, a seam, a protocol line lives in the same bundle as the operators, written
  additively like everything else, rendering to the distance gates and target locks the
  emitter produces today. The same motion the repo already made one level down when
  genesis's parse-side `Expect` enum dissolved into `discover::expect::Expectation`.
- **Retirement is by the house rule** (the per-diff mutation gate's rule: retire when the
  census closes the territory): genesis's emitter retires when a bundle-born crate
  reproduces the flagship story end to end — expectations declared red, meanings filled,
  green at both levels, locks fresh from birth — with the demos re-founded on the bundle
  as proof. Until then it stands; two CI-tested members lean on it.
- **Naming resolves itself with zero renames.** The continuation tool already has its
  name (the bundle); the declaration grammar must NOT get a name of its own, because it
  must not get a separate existence (one vocabulary — the tenth ask's lesson); and
  "genesis" becomes precisely true by SHRINKING: when the emitter dissolves, what remains
  under the name is the one genuinely genesis-shaped act — minting the crate shell, once,
  at origin. The name stays put and the referent contracts to fit it.

One sentence for the record: the bundle continues, genesis births, and the declaration is
vocabulary, not a tool.

**The declaration entry's first form is BUILT** (`Bundle::declare`, landed the same day as
the disposition): an expectation declares additively into a module's `#[algebra(...)]`
attribute — the `expects` grammar exactly as the macro takes it — and the module comes back
canonically rendered, differing in that one attribute and nothing else (drilled on the
committed `modularize.rs`: exactly one line moves). Parser-as-gate throughout: a shape word
outside the ratified catalog is refused TEACHING the vocabulary (the same validator the
engine panics with, in `Expectation::canonical`'s non-panicking form), a duplicate
declaration is refused, a module with no `#[algebra]` names the fix, and two `#[algebra]`
modules refuse as ambiguous.

**Both disclosed rungs closed the next day.** The ZERO-ANNOTATION channel:
`Liftable::expectations()` (default empty — a plain lift starts with no contract, every
declaration an explicit act), `Lifted<C>` forwarding it as `Expected`, and
`AutoLift::scan_module` growing a declarations parameter that vocabulary-gates at scan
time and BAKES the contract into the generated impl — so
`Distance::of::<Lifted<C>>()` is a red/green gate for an author who wrote only types and
Rust. Both arms drilled: the bool carrier's declared laws judge MET, and an overshooting
declaration (idempotence claimed of an involution) reads UNMET BY NAME — the red target
lock, in the lifted world.

**And the peak got its name, from the operator: THE CLI IS THE INTERFACE TO THE CODE.**
Agents should not write text into open files — and the irony is that agents already drive
everything else through a CLI, including reading and writing the files themselves; the
bundle just makes the CLI speak in judged transactions instead of bytes. First form:
`examples/bundle.rs` — `add`, `declare`, `place`, `check`, `lift` — each verb wrapping the
library form, refusals writing nothing, successes leaving the module canonically placed.
The proof is `bundle-demo/`, THE BUNDLE-BORN MEMBER: its `src` grown entirely through the
verbs (every command in its MANIFEST.md — birth as the degenerate case of continuation, an
`add` onto the empty file), its contract declared through `lift`'s declarations, judged
MET on every test run, its lift committed as a derived artifact held byte-for-byte to the
scan, its module held to the round-trip pin, its locks frozen and sensitivity-swept. And
discovery kept its oldest promise on day one: the four declared laws came back with a
SURPRISE nobody declared — `bump` is a merge-homomorphism — the flagship's
declared-plus-discovered story, reproduced by the continuation process. Retirement
status, honestly: the MODULE level of the criterion is met (grown, declared, judged,
locked, CLI-only); the SYSTEM level (seams, transports, the two-lifecycle red commit)
remains, so the genesis emitter stands for now.

### The peak of the peak: review shifts left — the mech suit

The operator's framing, recorded verbatim in spirit: the entire process that sits in PRs
and code review exists because of POOR LOCAL SENSORY AWARENESS — automating code
interaction through the CLI eliminates everything you could want to review, leaving a
continuous process that supports the agent in perception, eliminates toil, and leaves
only the important things behind. A MECH SUIT for the agent.

This is the pipeline brick's sentence, applied one level up. That brick said: CI stops
being where verification is defined and keeps only what cannot shift left. This says:
REVIEW stops being where judgment happens and keeps only what cannot shift left. Review's
three jobs, taken separately: VERIFICATION (break/duplicate/misplace?) dissolves entirely
— every verb is a judged transaction, so a change that exists has already survived the
collision refusal, the vocabulary gate, the placement, the censuses, and the distance
judgment, by the same judges a reviewer would have to trust anyway; RATIFICATION
concentrates — the review router's first run already routed a whole branch to "one
ratification, seven machinery-verified," and the PR of the future is exactly that agenda,
a stream of lock diffs each asking one signable question; INTENT never shifts left, by
the method's own epistemology — "what did you mean" stays human, the suit only makes it
legible (a declaration is one line, a distance report names the gap, a justification is
the register text no derivation produces).

The suit anatomy, in the parts already built: PERCEPTION is the six voices, the
session-start topography, the distance and cohesion reports; ACTUATION is the verbs;
PROTECTION is refusals that fire before damage instead of findings that arrive after.
And the reason shift-left is SAFE — the load-bearing connection worth stating once —
is the sensitivity program: you only climb into a suit whose sensors you trust, and
every judge that moved left carries a can-fail proof (the probe census, the fire drills,
zero ratified schemata survivors across the compiled population). The suit is
trustworthy because it has been shot at.

Honest bounds, the method's own: the suit covers the algebra-shaped fraction — which is
why the 55/55 spike is load-bearing, not cosmetic (every file it converts is territory
the suit's senses reach; the remainder still gets reviewed the old way); and "eliminates
everything you could want to review" is precisely true of the MACHINE-CHECKABLE class —
review does not disappear, it compresses to the ratification stream and the intent
questions, which is the point.

The rung this names: **the transaction log.** When every change arrives as a verb, the
PR description stops being prose reconstructed from a diff — it IS the log. The verbs
journal themselves; the agenda computes from the journal instead of re-deriving classes
from changed paths; `bundle-demo/MANIFEST.md` is the hand-made prototype of exactly this
artifact, and deriving it is the next brick of the vision.

### The aim, adopted: zero file patching — all CLI

The operator's directive, now a working rule (CLAUDE.md carries the binding form): code
changes go through the verbs wherever the verbs reach, and a change the verbs cannot
express is a FIELD REPORT — the missing verb gets named here before the file gets
patched. Dogfooding is the only way to find the rough edges; the primary user is the
agent, and the gaps are the roadmap's fuel. Scope, stated once: this governs CODE —
prose stays hand-typed (prose is judgment, the house rule already says so), and the
freeze/bless paths already never hand-edit their artifacts.

The named gaps at adoption, each a missing verb: ~~**edit**~~ (CLOSED the same day — the
rule's first working session hit it immediately, as predicted: `Bundle::edit` replaces
one item's text while its SIGNATURE holds, compared token-for-token; a signature move, a
rename, or a kind change is an interface change wearing an edit's clothes, refused by
name with both signatures shown. What an edit may change is the meaning and its prose —
everything a caller cannot observe through the signature — and everything that names the
item re-judges downstream precisely because the signature held. Dogfooded live on
`bundle-demo`'s `bump`, where the planted signature-move refusal fired naming
`fn bump (t : Tally)` against the offered two-argument form), **remove**
(~~**remove**~~ DISSOLVED by the operator's question — see the collection disposition
below: garbage collection replaces deletion, with mark derived and sweep ratified),
~~**collect**~~ (CLOSED — and it is the first verb that implemented its own FROZEN
SPEC: `spec/verb-algebra.spec` had already discovered collect's laws before the verb
existed. `Bundle::collectable` is the MARK — every named private item reached by no
root, the roots each an existing sense (the `pub` boundary, reference by any other
item, a committed law, a declared expectation, a reliance) — and `Bundle::collect` is
the SWEEP, one judged journaled transaction that refuses anything the mark did not
derive, showing the mark set in the refusal. Module-scope, disclosed; the crate-wide
collector needs the tree walk at item grain), **split** (rung
3 — executing a ratified cohesion split), ~~**constrains**~~ (CLOSED next — the
perception verb, and the one the primary user said it would reach for most: everything
that pins a named operator on one page, derived instead of grepped — its placement
component, the committed laws naming it, the declared expectations, the ratified freedoms
at it, and the downstream reliances, with empty sections rendered honestly as findings
("none — every judged mutant of this operator dies"). Read-only: perception writes
nothing and journals nothing; an operator the module does not declare refuses. First
live run on the bundle-born member returned the four declared laws PLUS the discovered
homomorphism — the report already knows more than the author declared. Pleasing
side-effect: the reason census re-classified `bundle.rs` as effectful the moment the
verb grew a filesystem read — the censuses watching the watcher), impl-interior
placement (exercised as the
fallback path the same day: building `edit` itself required an impl-interior change the
verbs cannot express — the second rule's field-report arm, proven on day one),
~~test authoring~~ (CLOSED across two bricks: a `#[cfg(test)] mod probes` is an ADDABLE
item, proven when squash's probe suite went in through `add` — and growing an EXISTING
probes module turned out to already be `edit`'s business: non-Fn/Impl/Trait items edit
under a name + kind hold, exactly right for a test mod, discovered while killing the
sweep's findings), and cross-file moves (SHRUNK the
same day: a module MOUNT is just `add` of a `pub mod` item on `mod.rs`, dissolved before
it was ever named a gap). ~~**The frozen arm**~~ (named and CLOSED the same night — the
verbs' vehicle rebuilt behind the same gate as the tree it operates on, so a
mid-transaction tree blocked the fixing verb; `bundle pin` dissolves it, see the pinned
suit brick). Refusals should name their fixing verb.

~~Newly named from the first sustained night INSIDE the suit — **the show verb**~~
(named in the morning, CLOSED by the afternoon — the editor-reach census's first datum,
gathered by dogfooding instead of survey: every `edit` payload is a whole-item
replacement, so every edit began with an interior read — sed ranges, awk brace-matching,
a python extraction script. The reading, named: "what does item X say right now?" is a
PERCEPTION question, and now the verbs answer it. `bundle show <module.rs> <item>`
returns the item's VERBATIM SEGMENT, cut by the same spans `edit` holds and `replay`
reconstructs — pinned by the round-trip probe: editing an item with its own shown text
is a byte-exact no-op, so `show m.rs x > payload; revise; edit` is an edit cycle that
never opens a file. `bundle show <module.rs>` with no item is the INVENTORY — the
module's table of contents with each item's kind, visibility, operator status, and, for
functions, THE EXACT SIGNATURE THE EDIT HOLD WILL COMPARE, token for token. That last
column is the published-interface brick riding along: an edit payload can now be
authored FROM THE CONTRACT (inventory gives the signature, `constrains` gives the laws,
`trace` gives the conduct) — for a certified operator, the residual reasons to read a
body are exactly the properties the lock language cannot yet say. Refusals teach: an
unknown address lists the addressable roster, which also DISCLOSES the grain honestly —
items inside inline algebra mods address as the mod, inner addressing is a future rung.
Built through the suit except the two Bundle methods themselves — interface GROWTH,
which `edit` refuses by design; the impl-interior fallback arm, exercised as intended.)

Named on its third firing — **grow** (interface growth as a judged verb). The
support-projection brick needed two new methods on `VerdictStore` and one signature
change (`owed` keying per gate support instead of one tree key); `edit` refused by
design ("7 held, 9 offered"), exactly as it refused the show verb's own two methods
and the impl-interior change that built `edit` itself. Three firings is a pattern,
not an incident: the missing verb is roughly `bundle grow <module.rs> <impl> <method
payload>` — ADDITIVE interface change as one judged transaction (a new method, its
probes rider, the qualify-census delta carried in the same judgment), with signature
CHANGES still refused (that is a break, not a growth; the census and the version
bump own it). Until it exists, interface growth is the fallback arm's most-worn
path.

Newly named from brick one of the exterior-engine candidate — **member genesis**. The
knee harness wanted to be a research member (`weave-knee`, the layout-probe /
delta-render shape: version 0.0.0, publish = false) and no verb founds one — but
dogfooding shrank the gap below its prediction: the verbs reach INTO a member's
modules fine (`show` probed on layout-probe before assuming, rather than grepping the
resolver), and `add` turned out to FOUND a module file that does not exist yet,
mounts included, so the entire crate body went in through the suit. The true fallback
footprint was exactly two manifests (the workspace member line, the member's own
`Cargo.toml`), the crate's `//!` doc header (prose anyway), and bare `mkdir` for
`examples/`/`tests/` — `add` refuses to create a parent directory it could safely
imply. Genesis is the adjacent machine — it already scaffolds whole crates — but it
speaks `system!`, and a research member is not a declared algebra at birth. The
missing verb is roughly `bundle found <member> --research`: the two manifests and the
directory skeleton as one judged transaction, with the research-member conventions
(0.0.0, publish = false, the fire-drill dev-dep) as a judged floor instead of a
copied header comment.

Two deferrals from the same conversation, recorded so they are not re-litigated:

- **The incremental turn (DBSP), deferred**: the rhyme is real — the journal is a delta
  stream, replay is integration, squash is delta consolidation, collect is retraction,
  and delta-render's d/i commutations plus the homomorphism shape are the linearity
  condition incrementalization turns on. The disposition when it is time: adopt the
  ALGEBRA, not the engine — incrementalization as a LICENSED optimization (only views
  whose maintenance the lock certifies linear go incremental; the batch path stays as
  the standing second source, `incremental(view, δ) == batch(integrate(journal))` as a
  drift gate), and the verb algebra already knows where the group structure fails (the
  refused conflicts), so Z-set treatment applies to VIEWS over the item set, never to
  the raw verb stream. The trigger to revisit: stage 4 making view-derivation the hot
  loop. Not before — today's measured bottleneck is test execution, which this does not
  touch.
- **The totality claim, named without building**: discovery already EXECUTES every
  operator over every grid tuple on every test run — a panic would fail the suite — so
  "every operator is total on its declared grid" is already load-bearing; it has simply
  never been stated as a lock line. When the spec header format next moves for its own
  reasons, the totality line rides along free. The wider property vocabularies
  (scaling laws for time, allocation counts, sandbox-carrier effect theories — the
  store's "stash is a projection" probe is the prototype: an idempotence law about real
  I/O judged in a scoped world) are the named path for shrinking the read-the-body
  residue; each is a derive-plus-ratify brick waiting for its consumer.

**The disposition on removal — garbage collection, with the house amendment: MARK IS
DERIVED, SWEEP IS RATIFIED.** The operator's question ("is there ever really a need to
remove something? we know how everything hangs together — the unused disconnects and we
eventually discard it automatically") dissolves the `remove` verb, with one correction
the method itself demands. Plain auto-discard would be self-attestation: a runtime GC's
unreachable object is PROVABLY worthless, but an unreferenced item of code can be
seasonal, staged, or vocabulary awaiting its consumer — liveness has an intentional
component the reference graph cannot see, the kernel-register lesson again (intent is
never inferred from conduct). So: the MARK phase is fully derivable and mostly already
built — an item is COLLECTABLE when no root reaches it, and every root is an existing
sense (pub-reachability from the tier derivation, the laws' silence line, declared
expectations, downstream reliances, placement isolation); `collect` is `constrains`
inverted — find everything that nothing pins — rendered as a derived census with
evidence per item. The SWEEP is one judged, journaled verb removing exactly the marked
set: automatic in the sense that matters (zero analysis, one command, machine-authored
diff), ratified in the sense that keeps it honest (the diff is the signature). The
journal gives "eventually" its clock — disconnection age in journal order, generations
in certified releases — so the proposal can escalate on schedule without ever seizing
authority. And in stage 4 the fear of deletion dissolves entirely: the log never
forgets, so collect is UN-MATERIALIZE, never un-exist — precisely why GC is safe in
runtimes. What remains of "remove" is only the BREAKING case, deleting something still
pinned — and that is not collection, it is interface change, which already has its
ceremony: the reliance machinery refusing by name until the consumers migrate.

**The standing question, adopted into CLAUDE.md** — "what else are we doing by hand that
is secretly a derivation plus a signature?" — with the inventory it yields when asked of
the repo today, each entry a named candidate:

- **Commit messages and PR bodies**: narration of a diff is a derivation — the journal
  plus the freeze-delta narration already hold the content; the commit itself is the
  signature. The agenda-from-journal brick subsumes this.
- ~~**The bless loop**~~ (CLOSED the same day it was named — the standing question's
  first kill: `Config::autofix_stale`, default on everywhere except CI. A drifted census
  regenerates INTO THE WORKING TREE and the build fails exactly once — "the regenerated
  text is now in your working tree; the diff is the ratification: commit it, or revert
  it to refuse" — so the gate's authority is untouched while the homework disappears.
  In CI the refusal only reports: a fresh checkout must never mutate itself. Both arms
  drilled in `census_drifts_then_blesses_then_holds`; the bless variables remain for
  explicit regeneration.)
- **Imports in snippets**: `use` lines are name resolution, which is a derivation — a
  `bundle add` payload should not need to carry them; the verb can derive and splice
  them. Real toil today, felt while growing bundle-demo.
- **CLAUDE.md's regenerate table**: artifact → command is registry DATA restated as
  prose — derivable from the gate/freeze declarations and drift-gated like everything
  else, so the table can never lie about a command again.
- **The crate shell** (Cargo.toml, the mod line): mostly derivation (genesis's residue,
  the shell mint); the hand-typed part shrinks to the dependency DECISIONS.
- **Test scaffolds**: the probes are already derived; what stays hand-typed is the
  irreducible base the method has always named — meanings, negative fixtures, and the
  grammar itself.

What stays hand-typed on principle, so the question is never over-applied: register
justifications, roadmap dispositions, declared expectations' CHOICE (which laws to
promise), and every ratification — the signatures themselves. The question hunts
derivations wearing a signature's clothes, never the reverse.

And the standing question asked FORWARD — dissolving questions queued, unanswered on
purpose (each wants its own conversation before it becomes a disposition):

- **Do we need branches?** A branch is a speculative journal SEGMENT — merge is append,
  rebase is replay, conflict is two segments claiming one item. Once the journal is the
  source, the branch ceremony may be the log's view too.
- **Is the issue tracker secretly the red-lock list?** A ticket that describes intended
  behaviour IS a declared expectation not yet met — the target lock committed red was
  always a ticket with a judge attached. TODO comments are the same fact at lower
  altitude: each is either a derivable red gate or a disposition, and hand-tracked
  prose is the worst home for both.
- **Is a bug report secretly a fixture plus a missing law?** A reproduction is a dent
  the spec should have caught — fire-drill's lineage says every incident becomes a
  planted fixture; the dissolving form says it should arrive as one.
- **Does CI need to re-execute what local gates already judged?** Re-execution exists
  because the server does not trust the laptop — but a signed transcript of a local
  gate run is an attestation, and the countersign might verify rather than re-derive.
  (Trust roots are the hard honest part; differential-certify's provenance story is the
  rail.)
- **Is the editor anything but a bundle viewer?** Once files are views and changes are
  verbs, an "editor" is a perception surface with a verb palette — the open-file
  buffer, like the PR, survives only as a way of LOOKING. SHARPENED with the operator
  (who asked the dissolving form: what is a person trying to confirm that we are not
  currently ratifying?): a person reaches for the editor exactly where the question
  they are holding IS NOT YET A LOCK CLASS — the interior is the fallback
  encyclopedia, consulted when derived answers run out. The enumeration: "how does
  this behave on THIS input" ~~is a missing perception verb~~ (CLOSED —
  `discover::trace`: a ground term over a theory's operators, evaluated bottom-up with
  every reduction narrated through the theory's own observation; refusals teach the
  vocabulary, arity mismatches show both counts, and a partial operator's refusal is a
  fact shown rather than an error buried. `bundle trace verbs 'add_a(edit_a(empty))'`
  makes the verb algebra's conflict WATCHABLE — the two orderings traced side by side,
  the divergence visible without opening any file. Variables stay out deliberately:
  quantification is the laws' business, discovery's half of the split); "is this fast / allocation-free" is a missing
  property vocabulary (toleranced BENCHMARK locks — the three-valued judgment's
  shape, pointed at time); "is this safe" is the capability audit and fabric's
  reachability brought down one altitude; "what is here that I can use" is the
  library voice, built; "I don't trust the abstraction" dissolves only by track
  record. The move: census the editor-reaches (the hand-work inventory pointed at
  READING) — each reach names the next verb or the next lock class, and interior
  reads go archaic the same way interior writes did: the answer set grows until the
  fallback goes quiet.
- **Do branches, merges, and the reflog survive the journal?** Sharpened with the
  operator's three observations. (1) The freedom-to-explore that branching grants is
  already here in different clothes: THE RED LOCK IS IN-PLACE BRANCHING — divergence
  as declared intent with a judge attached, converging in public, instead of
  divergence as a parallel tree; and the fear branching protects against shrinks
  when every change is a judged transaction that lands whole or refuses. (2) The
  coordination overhead ("who is changing what file") is an artifact of
  files-as-storage: at verb level, disjoint `add`s COMMUTE by the placer's own
  monotonicity, and the conflict surface collapses to same-item writes plus lock
  ratifications — so FORK/JOIN AT SPEC-LOCK LEVEL is: join = replay both journal
  segments, the judges name what does not commute, the lock diff is the merge
  review. Merge stops being a textual operation performed in ignorance. (3) The
  operator's indirection instinct completes the deletion: with CONTENT-ADDRESSED
  items at add-time (the stage-4 item store, Unison's move), the item universe is a
  grow-only set whose union is trivially conflict-free, and all remaining conflict
  concentrates into the BINDING MAP (name → hash, a ratified decision) and the
  locks — the remove-dissolution's shape again: the mechanical half derives, the
  decision half signs. And the reflog answer, the startling one: git's reflog is
  untyped snapshot pointers; the journal is TYPED — verbs are operators over the
  bundle carrier, segments are terms, and "do these two segments join?" is a
  COMMUTATION LAW. The merge algebra need not be designed: it can be DISCOVERED —
  the engine run over the verb algebra itself, the change-history becoming a
  registry theory whose laws are the fork/join rules. Losing code becomes
  impossible not by hoarding states but because the journal never forgets and
  un-materialize never un-exists.

**Stage 2's first form landed with it: THE JOURNAL.** Every mutating verb appends one
line to `bundle.journal` beside the nearest `Cargo.toml` — `<verb> <module> — <detail>`,
no timestamps, order the only clock — so the change record is derived, never narrated
(`bundle-demo/bundle.journal` opened with the dogfooded edit as its first entry, the
machined successor to MANIFEST.md's hand-written story). The write lands first and a
journal failure is reported, never swallowed. Disclosed: entries carry names, not
payloads — the agenda's source and the reviewer's record, not yet replayable;
tree == replay(journal) is stage 3's business, with the payload store.

And the staged path from here to the operator's horizon — no files on disk or in git:

1. **Verbs-only discipline** (now): every reachable change through the CLI; gaps named.
2. **The transaction log**: verbs journal themselves; the agenda computes from the
   journal; the PR body is derived.
3. **The tree becomes a gated derived artifact**: the journal is committed, and a replay
   gate holds tree == replay(journal) — the `ci.yml` move at tree scale; a hand-edited
   file refuses the same way a hand-edited lock does.
4. **The tree leaves git**: the repository is the journal, the locks, the registers, and
   the shell; the tree materializes at build/publish time like any target artifact. This
   needs the content-addressed item store (the bundle stops borrowing the file's bytes as
   its representation) and judge-version anchoring in journal entries (replay under
   tomorrow's judges may judge differently — the determinism pin, differential-certify's
   rail). rustc and the ecosystem keep getting materialized trees; they just stop being
   the source.

### The verb algebra — the reflog as a theory (BUILT)

Built the session after it was sketched, and the sketch held almost exactly. What landed:

- **The catalog grew its stanza first** — `commuting maps` (`f(g(x)) = g(f(x))`, two
  unary endomorphisms on one sort), with a new GATE datum rather than a guard:
  `ordered_ops`, the symmetric-equation dedup (both orderings state one law, so only the
  canonically-name-ordered binding is admitted — one unordered pair, one lock line).
- **`discover::verbs`** — the miniature (two named slots × three states × the contract
  flag, 18 inhabitants; verbs as total functions carrying the CLI's refusal semantics) —
  and discovery returned 28 laws: EVERY verb a projection (replay safety as law — the
  refusal semantics make re-applying a journal segment harmless by algebra, not luck),
  every disjoint pair a commuting-maps JOIN RULE, `declare` commuting with everything,
  and one finding nobody predicted in the sketch: `collect` ABSORBS `edit` on the same
  name (the state forgets the edit — no conflict; remembering is the journal's job). The
  two conflict classes are exactly the ABSENT pairs — add/edit and add/collect on one
  name — pinned REFUSED the router's way, plus a drill proving the conflict cannot be
  declared away (the wishful-merge distance stays red). Frozen as
  `spec/verb-algebra.spec` + its mutation lock — the sweep reports every operator-table
  mutant killed and every sampled dent pinned, so the fork/join rules of this system
  carry a full can-fail proof. Embedded in `Locks` (a pin now carries its own merge
  semantics), covered in the probe census as oracle-swap.
- **The stanza paid for itself across the workspace the same hour**: delta-render's
  stream calculus gained `d and i may be applied in either order` (the
  differentiate/integrate commutation, sayable at last) plus delay/neg commutations, and
  BOTH layout engines gained `commuting_maps(reorder, theme)` on their scorecards — the
  growth dynamic (hostile domain → stanza → every theory benefits), demonstrated on
  contact.

Two follow-ons from the operator's read of the build:

- **Restraint, calibrated**: the house rule is not "make do" — it is EVERY STANZA NAMES
  ITS CONSUMER (commuting maps named the verbs; inflation/deflation named fabric). The
  COMPOSITION stanza (`f(g(x)) = h(x)` — three unary slots, h free to be f itself, the
  absorption form) named its consumer — JOURNAL COMPACTION, squash as algebra — and then
  LANDED, and it over-delivered the same way commuting maps did. The squash table came
  back discovered (all six: add-then-collect, edit-then-collect, and collect-then-edit
  each collapse to collect, both names — lock lines a future `squash` consults as data,
  the join-verb precedent). And the stanza killed SIX RATIFIED SURVIVORS nobody aimed it
  at: the verb algebra's four edit/collect confusions (invisible while both were merely
  projections with identical partners — the composition laws tell them apart, 64 of 64
  mutants now die) and delta-render's filter-as-zero and map-as-zero confusions, ratified
  freedoms since that crate's first freeze, now dead. The freedom census SHRANK by six
  because the vocabulary grew by one stanza — the bias-blindness hunt, industrialised,
  paying out on theories built months apart. Layout's scorecards gained the composition
  surprises too (`theme(reorder(x)) = reorder(x)` — theme's invisibility, said
  compositionally).
- **The method is not Rust-bound — the substrate interface**: what the machinery requires
  of a language is (1) an item grammar with faithful parse/render (tree-sitter
  everywhere; syn is the Rust binding), (2) a net model — types in signatures as sorts
  (any typed language; the no-primitive-obsession precondition is the real constraint,
  not the language), (3) an EVAL BRIDGE for behavioural discovery — already proven
  cross-language: the theory-bridge consumes Lean's exported operator tables today, so
  any language that can dump finite tables gets spec locks, mutation verdicts, and
  distance with zero new engine machinery — and (4) the language's own compiler as one
  gate among many (rustc's actual role). Rust is the HOST and the first SUBSTRATE;
  layout-probe (geometry), spec-lock's adopters (checklist data), and the verb algebra
  itself (items and names, no syntax) are the existing proofs that the method outruns
  its host. Honest bounds: dynamic languages weaken the nets, and the compiled-mutant
  layer is per-substrate work.

- **The payload language** (the operator's follow-on: "are you writing traits directly
  with edit?"): the audit says NO — no trait has ever gone through the verbs, the
  bundle-born member is fully certified with zero traits/generics/nested modules, and
  the verb grammar treats traits as opaque cargo nothing has missed. The taxonomy
  underneath: language features split into MEANING features (the body's business) and
  COMPOSITION features (modules, visibility, traits, interfaces) — compensations for
  hand-composition, and the apparatus has absorbed them one by one: modules → placement,
  visibility → tiers, traits → declared expectations + the catalog (A TRAIT IS A NAMED
  LAW-SET THE COMPILER CANNOT CHECK; the engine's version carries a can-fail proof),
  semver → locks. What survives of traits is dispatch (a genuine meaning-level need) and
  the HOST's own generics — the host stays Rust. And the body need not be: the theory
  bridge already judges Lean's exported tables, so for enumerable carriers the edit
  payload could be the value TABLE itself — language-free meaning — and for unbounded
  carriers any language with an eval bridge qualifies, per module. The payload language
  has a spec already: THE OPERATOR-SHAPE RULE — which re-reads the 55/55 spike as the
  migration of the codebase INTO the payload language, not a compliance project.

- **Self-hosting** (the operator's question: could probe-algebra be implemented entirely
  with the verbs?) — three rungs, and the first is already behind us. (1)
  SELF-DESCRIBING, done quietly: the locks freeze the freezer, the probes census the
  probes, schemata mutates the mutation machinery, and the verb algebra is the theory of
  the toolchain's own change process. (2) SELF-CONSTRUCTING — every change to this tree
  through the verbs. The audit named ONE gap: the host is mostly impls and traits, which
  the verb grammar treated as cargo — CLOSED the same day: `item_address` gives impls
  their edit address (`impl Type`, `impl Trait for Type`; two blocks sharing one refuse
  as ambiguous), traits join `item_name`, and `edit`'s interface-hold scales up one
  grain — an impl or trait edits under the METHOD-SIGNATURE-SET hold, bodies and docs
  free, surface held, a moved set refused by count. Smoked on real host code:
  `modularize.rs`'s own `impl ProposedModule`, edited through the verb, every byte
  outside the block untouched. What remains of rung 2 is practice, not vocabulary: the
  second rule's coverage growing as the gaps ledger empties. (3) SELF-TRUSTING — the
  verbs building the verbs inherits the compiler-compiling-itself question, and the rail
  is already laid: differential-certify (release N−1 judges N), base case disclosed.
  Self-describing, self-constructing, self-trusting — each rung keeps the honest frame
  of the one below it.

The original sketch follows, kept for the record:

- **The carrier is a small bundle-state value** — a `Shaped` miniature of a module (a
  bounded set of items with names, kinds, and component tags; the stream-carrier lesson:
  deliberate histories, not combinatorial soup). The VERBS are the operators: `add(item)`,
  `edit(name)`, `declare(expectation)`, `collect(name)` — each a total function on the
  carrier (refusals map to the unchanged state or a tagged refusal value; partiality IS in
  the vocabulary).
- **What discovery should find, if the design above is honest**: disjoint `add`s commute;
  `edit`s of distinct names commute; same-name `edit`s do NOT (the non-commutativity is
  the CONFLICT, pinned the way the router's non-commutativity is pinned — a load-bearing
  refusal, not a failure); `add` then `collect` of the same name annihilates; `declare`
  commutes with everything not naming its ops. Every law that comes back green is a
  fork/join rule with a can-fail proof; every refuted law is a conflict class, NAMED.
- **The freeze**: `spec/verbs.spec` — the merge semantics of the entire system as a
  committed, mutation-tested, drift-gated lock. A version-control system whose merge
  rules carry the same evidence discipline as the code they merge. When the join verb
  ships, it consults this lock the way `check` consults the placer — the rules are data,
  never folklore.
- **The honest frame, inherited**: the miniature carrier proves the algebra's SHAPE; the
  real journal's items are richer, so the lock is evidence about the design, not a proof
  about the implementation — the seam between miniature and real join is a transport
  seam like any other, judged when the join verb exists.

### The log and the anchor

And the question the log raises, asked by the operator and answered here so it is not
re-litigated: if the log is the record of change, WHAT IS GIT FOR — do we even need it?
The honest split: git stores STATES, the log stores INTENTS — a diff is a reconstruction
of what happened, the log IS what happened (patch-first systems — darcs, Pijul — are the
prior art; Unison again at the term level). So the log becomes the source of CHANGE, and
git shrinks — the genesis move again, a name contracting to its true referent — into the
ANCHOR: the content-addressed snapshot store, the countersigned certified line, the
distribution surface every other tool speaks. The two cross-check by the second-source
pattern the repo already trusts everywhere: the journal, replayed, must produce the tree
git holds — agreement is a judged fact, divergence a defect detector (and replay is why
the log alone is not enough: a verb replayed under tomorrow's judges may judge
differently, while a snapshot is version-independent — the log needs git's anchor
exactly the way the incremental mutation gate needs `mutants-green`). Code on disk,
likewise, keeps three jobs, all derived-artifact jobs: the build's input, the cache that
keeps verbs fast, and the interop/legibility surface — materialized view #1 of the
bundle, exactly as `ci.yml` is of the gate registry.

### The squash verb — the lock executes (BUILT)

The composition stanza froze the squash table; this brick makes it run. `bundle squash
<bundle.journal>` compacts the change record to its LAW-NORMAL FORM, and the compactor's
entire rule table is READ OFF the discovered laws (`SquashRules::from_spec`, parsing the
engine's machine-grammar equations): same-item composition laws become the collapses
(add-then-collect is one collect), projection laws become the replay-noise filter (a
repeated identical entry is one entry), cross-item commuting-maps laws become MOBILITY
(which verb pairs may slide past each other on different keys, so non-adjacent pairs can
meet). Nothing is hand-authored: change the verb algebra, re-freeze, and the compactor's
behaviour follows — the join-verb design ("the rules are data, never folklore") realised
on the first verb to need it. Conservatism is silence, and the silences are load-bearing,
each pinned: add-then-edit on one key composes to NO single verb, so both lines stay (the
journal keeps the whole story); a verb the algebra does not model (`place`) is opaque and
nothing slides past it; the one hard refusal is a line the journal grammar cannot read —
the record is machine-written, so a strange line means a hand touched it. Squash journals
nothing: it rewrites the record under the record's own warrant, and its trace is the
journal's diff.

The self-hosting note, one rung sharper than collect's: collect CONSULTED committed locks
as evidence; squash EXECUTES the frozen spec of the verbs themselves as its engine. And
the brick was built through the suit end to end: `squash.rs` was born by `add` on a
missing file (module doc hand-typed after — prose is judgment), its probes went in by
`add` too (a `#[cfg(test)] mod probes` is an addable item — the "test authoring" gap
shrank without a new verb), the mount `pub mod squash;` went through `add` on `mod.rs`
itself (the module-mount gap dissolved before it was ever named; the same transaction
re-dealt two operator functions `mod.rs` had never had judged, ratified in this diff),
and the CLI grew its own verb through its own `edit` — `fn run` replaced under the
signature hold. Payload `use` items rode `add` as unnamed cargo, so the imports gap is
narrower than feared; the arm's own imports live block-local inside the new match arm.

**Field report — the frozen arm** (the one genuine rough edge, named for the ledger):
mid-transaction, the tier census correctly derived the not-yet-mounted `squash.rs` as
INTERIOR (not pub-reachable) and the inward rule refused the build — but `cargo run
--example bundle` REBUILDS, so the vehicle of the very fix was blocked by the gate it
was about to satisfy. The prebuilt example binary broke the deadlock. The missing piece:
the verbs' vehicle must not share the build gate with the tree it operates on — a pinned
suit binary (the shipped-hook precedent) or a verbs-exempt profile. Pleasing symmetry on
the way out: the moment the mount landed, the tier census reclassified squash.rs
INTERIOR → ALGEBRA on its own — the derivation watching the tree change under it.

### The pinned suit (BUILT — the frozen arm, closed the same night it was named)

`bundle pin` installs the RUNNING binary at `.suit/bundle` with a provenance line beside
it (version, source path, toolchain), so the verbs stop rebuilding behind the gate of the
tree they operate on. Both incidents that named the gap are dissolved: a mid-transaction
tree can no longer block the verb that would heal it, and a verification poked mid-sweep
no longer taxes the sweep's shared build cache. `.suit/` is ignored — the binary is a
local artifact; the provenance says where it came from. The deeper reason it had to exist
before stage 3: a replayer that must build from the tree it is reconstructing cannot be a
second source for that tree.

### Where the sweep's hour went (accounted, and the candidate it names)

The operator asked where the local sweep's time goes and whether we understand what it
is for. The accounting, from the pipeline's own code: the sweep is ONE build, one
recorded baseline (the coverage map: which tests touch which site), then per site a
fail-fast run of ONLY the covering tests — **when nextest is installed**. On the agent's
box it was not, so the disclosed fallback ran the ENTIRE lib suite per site: ~830 sites ×
(cargo startup + a ~6s suite) across 4 workers oversubscribing 4 cores — the observed
60–90 minutes was the fallback tax plus contention, not the design. CI installs nextest
and runs the same sweep inside the every-change check job on real runners; local
certification was never load-bearing. Fixed at the environment grain (nextest installed;
the pinned-suit lesson again — the vehicle's provisioning is part of the tool).

What it is FOR, restated so the cost has a purpose attached: tests judge the code; the
sweep judges the TESTS — every compiled flip must be detectable by the lib suite or it
is an unpinned degree of freedom, and this is the mutation gate that let the per-diff
source gates retire. But per CHANGE the full sweep re-judges hundreds of sites whose
code and covering tests both stand still — pure recomputation, and the first genuinely
justified consumer of the deferred incremental turn. **Candidate: the changed-scope
sweep** — sites are keyed by function path and the census is committed, so a diff since
`mutants-green` (or since HEAD) names exactly the sites whose guard code changed plus
the sites whose covering tests changed; sweep that subset in the inner loop and leave
the full re-certification to CI and the weekly shards, which already exist for exactly
this division of labour.

**The gates become locks (BUILT — the operator's "make that class of failure
impossible", same day):** the warm-green class named precisely is a VERDICT HELD WITHOUT
ITS INPUT KEY — and every judgment in the repo was already staleness-proof except the
gates themselves, which lived as terminal output and working memory (the bag-of-booleans
pattern at the CI grain). `discover::verdict` closes it by construction: a verdict is
keyed by the content fingerprint of everything the gates judge (relative paths + bytes,
sorted walk, `.git`/`target`/`.suit` disclosed as the skip set), a missing entry FAILS
CLOSED as unjudged, only green is ever recorded (red demands a tree change, and a
changed tree is a new key), and the store's entries sit outside their own scope so
recording cannot stale itself. `bundle gates` runs the owed every-change gates FROM THE
REGISTRY DECLARATION (the same one ci.yml renders from — "green locally" and "green in
CI" stay one claim), records verdicts only if the tree the gates judged is the tree that
exists afterwards (autofix moves the key; the run says so and asks to be re-run), and
`bundle owes` is the ceremony-sequencing toil dissolved: the to-do list a change owes is
DERIVED — the gates without verdicts at the current hash — not remembered. Pinned by the
probes: one moved byte re-opens the debt; identical relative trees share a key (the
portability the countersign below will stand on); the roster is the registry's
every-change set, never restated.

**The sampled countersign (BUILT — the candidate below, finished the next day):** the
sweep now writes a COMMITTED attestation (`attest/sweep.transcript`: tree hash,
toolchain, baseline, and every site's verdict with the covering tests it was judged by),
and the mutation gate's one declared command became `schemata verify` — locally and in
CI, the same claim: a committed attestation matching the checked-out tree and toolchain
is AUDITED by re-judging a random sample of its sites (seed drawn from entropy after the
transcript is fixed, so a false `killed` cannot be placed where the audit won't look —
and it cannot be fabricated from non-covering tests either: tests that never reach the
flipped guard pass under the mutant and the disagreement fires); a missing, foreign, or
disagreeing attestation falls back to the full sweep, which re-attests. The register
still judges the claimed survivor set before any sampling (an unratified survivor is a
red gate, not a disagreement), the census must name exactly the attested population, and
`attest/` is EXCLUDED from the tree hash — the attestation describes the tree, so it
cannot be part of the tree it describes. Timeout portability stays the disclosed edge:
a slow-box detection can read as a fast-box survivor, a FALSE disagreement that costs a
redundant sweep and never a false green. Field report from the first run: the
whole-tree scope makes PROSE edits invalidate the mutation verdict — a docs-only change
re-owes a nine-minute sweep it cannot affect. The named refinement: PER-GATE SCOPES
(fmt reads .rs; the sweep reads src and tests; none of the four reads docs/) — the
verdict key becomes a claim about what each gate actually consumes.

**Candidate behind it (the operator's follow-on): the verdict store with a sampled
countersign.** "Can a local sweep attest so CI skips?" — the waste diagnosis is right
(same tree, same pinned toolchain, a deterministic judgment recomputed), but a straight
skip fails the trust analysis, with same-day evidence: local green lied twice this
session (warm-cache clippy passes that failed cold), and `mutants-green` works precisely
because the countersign comes from the one party the working session cannot mint
signatures for. The version that survives: the local sweep commits its TRANSCRIPT — tree
hash, toolchain, per-site verdicts, content-addressed like the payload store (a judgment
store) — and CI re-judges a RANDOM SAMPLE of sites against it: agreement countersigns
the whole, one divergence triggers the full sweep, the weekly shards stay the
from-scratch backstop. Derivation local, countersign sampled — CI stays the signer at
O(sample) cost. Preconditions, disclosed: normalized timeout semantics (a timeout is a
detection, and timeouts are machine-relative — verdicts must be portable before they are
shareable) and the attestation claiming its ENVIRONMENT (toolchain already locked;
cold-build discipline becomes part of the claim). Sequenced after the changed-scope
sweep, which gets most of the win with none of the new trust surface.

### Stage 3: the payload store and the replay differential (BUILT)

The journal stops being names-only and starts being a SOURCE. `discover::store` adds the
content-addressed payload store (`bundle.payloads/` beside the journal, FNV-1a 64
fingerprints, one blob per distinct payload — stash is a projection, so re-recording
never bloats; a collision REFUSES rather than overwrites); `add`/`edit` stash their
payloads and the journal entry carries ` @<address>`. `bundle replay <journal>` is the
differential: reconstruct each journaled file by re-applying its entries' EFFECTS
(payloads from the store, never re-judging — a verb replayed under tomorrow's judges may
judge differently, and the ratification already happened; `collect` therefore refuses
replay by NAMING the effect/judgment split, the disclosed gap), then judge against the
tree. `tree == replay(journal)` is now measured file by file instead of promised — the
log-and-anchor disposition's second-source cross-check, live. First real run: every
existing entry predates the store, so the bar reads an honest ZERO of nine journaled
files — the metric exists precisely so real work moves it. Squash learned the addresses
the same hour: a trailing ` @<16 hex>` is NOT part of the compaction key, and when a pair
collapses the line whose verb the lock names survives (later over earlier — so
edit-then-edit keeps the LATER payload; the projection law licenses the shape, and
`replay(squash(j)) == replay(j)` is the differential drill that will judge the transport).

Alongside, the first completed FULL sweep since the verb suite landed returned 12
findings — debt from the collect/constrains/trace bricks whose sweeps container-restarts
had killed. Eleven died to new probes (exact-byte splice assertions where contains() let
padding and cfg-detection flips hide; a single-op-component fixture for the constrains
matcher; the freedoms filter's conjunction pinned on the demo lock's natural killed-vs-
SURVIVED split; direct conduct probes for the mutant makers and the doc-flow `edit`);
ONE was ratified — `mutation::never:deaf -> None`, equivalence by definition (`never` IS
the constant-None evaluator), the register's first and only line. And the probe work
surfaced a discovery: TEST MODS ARE ALREADY EDITABLE THROUGH THE VERB — `edit` holds
non-Fn/Impl/Trait items by name + kind, which for a `#[cfg(test)] mod` is exactly right
(bodies free, kind held), so growing an existing probes module needed no new vocabulary.
The test-authoring gap closed by reading the code the verbs already had.

### The name, asked of the whole (disposition)

The operator's question, after the mech-suit exchange: "what do we even call this thing
now? It's grown way beyond my original feedback-loop domain model." Answered here so it
is not re-litigated: KEEP THE NAME — not out of inertia, but because the repo grew INTO
it. Genesis set the precedent of a name contracting to its true referent; this is the
twin case, a name EXPANDING to its true referent. "Probe algebra" began as a metaphor
over a feedback-loop domain model; today it is a description: the theories are algebras
probed by batteries, the shape catalog is an algebra OF law shapes, the change history is
literally an algebra (`spec/verb-algebra.spec`), and as of the squash brick the tooling
EXECUTES frozen algebra as its rule table. The layers keep their working names — the
METHOD is discovery (derive the fact, ratify the decision), the MEDIUM is the bundle
(modules as databases, files as views), the MODALITY is the verbs (the CLI as the
interface to the code), and the EXPERIENCE is the mech suit (the operator's coinage, kept
because it says what it does: the agent supplies intent, the suit carries the load). The
house naming discipline holds at the top: names say what things do, and this thing probes
algebras and is increasingly made of them.

## Spike: from 3 of 55 toward 55 of 55 — the operator-shaped interior

The companion measurement, asked for by the operator in the same conversation, aimed at
the part of the codebase we have been politely walking around: the qualify census says 3
of 55 files carry an algebra, and the other 52 are the plumbing every vision above quietly
excludes. The reason to push anyway, in the operator's framing: 90% of what is hard to get
right in a program is its DOMAIN — and operator-shaping a file is not a formality, it
FORCES the domain modelling (value objects in, primitives out, effects to the edge). Read
that way the qualify census is not a compliance number, it is a domain-modelling progress
bar for the repo's own interior.

Rung 1 is a REASON census, not a refactor — BUILT (`spec/qualify-reasons.spec`, derived by
`boundary-enforce` in the same walk as the qualify census — one rule, two renders — frozen
under `BLESS_REASONS`, drift-gated on every build, covered in the probe census as a
ratified drift-gate probe). The classes are deliberately mechanical (no functions,
impl-attached surface only, primitive signatures, borrowed types, parameterised types,
unshaped types, unit returns, zero-argument constants, effectful bodies); reading them into
the three work classes — (a) value-object debt, (b) missing VOCABULARY (effectful edges,
builder plumbing — the effects-as-theories direction, given a concrete worklist),
(c) principled refusals that become register lines — stays a ratification, as it must.
First numbers, and the headline was visible the moment the lock minted: 56 files, 3
qualify, 53 refuse — and the single biggest named class was IMPL-ATTACHED SURFACE ONLY
(15 files): the no-rats-nest rule pushes every public callable onto a typestate, and the
census read free functions, so the discipline's own crown rule manufactured its largest
blind spot. (Fittingly, `discover::bundle` itself landed in that class on arrival.)

**Brick 2, built the same day — ASSOCIATED FUNCTIONS ARE OPERATORS.** The one rule, one
change: a method is judged exactly like a free function with `self`/`&self`/`Self`
resolved to the impl target (calling convention and spelling, not shape); `&mut self` is
mutation, refused; a generic impl target is not a sort; methods key `Type::method` — the
sixth sense's identity convention, keeping two typestates' `new`s distinct. Two honesty
refinements rode along, both caught by reading the first re-bless before ratifying it:
`Self` must never leak into the census as a sort, and a TYPE PARAMETER is a variable, not
a value type — the walk now refuses `fn id<T>(x: T) -> T` and kin, closing a quirk that
predated the spike. The movement: root census 3 → 16 of 56 (the real algebras were
sitting behind receivers all along — `kvstore/store.rs` alone carries 18 method operators
over 10 sorts, `interp/boundary.rs` nine over seven), and the demo members' actual domain
modules (credit meter, billing, gauge, mixer) all census as the algebras genesis always
knew they were. The impl-attached class is DISSOLVED from the reason vocabulary — not
renamed: the walk sees impls now, so those files report their real signature classes
(a `mutating receivers` class joined for `&mut self`). 40 files still refuse, and the
remainder is now honestly what it looked like it would be: borrowed/parameterised
plumbing — the engine speaks `&`, `Vec`, and `Result` — which is where the vocabulary
work begins.

A milestone landed alongside, found by the same day's full schemata sweep: the last FOUR
ratified schemata survivors now die (the widened suite reaches their guards), so
`spec/schemata.register` is EMPTY — zero ratified survivors across all 706 compiled
mutants. The stale-line drill moved to a fixture register so it outlives the live
register's population. The target restated honestly, the instrumentation-census shape exactly: totality
is "every file qualifies OR carries a ratified reason" — the census owns totality from day
one; what grows is the qualifying fraction. And the tie back to the candidate above is
direct: only operator-shaped code can live in a bundle, so every file this spike converts
is territory the continuation process can govern — the two experiments are one programme
measured from opposite ends.

## Candidate: the exterior engine — every surface a bounded mind consumes

Origin, from a long design conversation (2026-07-13): five years of attempts at
automatic architecture documentation — hierarchical summarisers, RLM pipelines,
LLM-designed UI over an atomic design system, plain-HTML5 sites — all fail the same
way. Each part is locally fine; the whole feels THIN. The diagnosis is this method's
founding one, transposed: models are not bad at prose or pixels, they are structurally
blind to recursively structured cohesion, because no harness ever bounded the weave or
descended a through-line. Thin-ness is what statistical independence of the parts looks
like; cohesion is correlation, and correlation requires shared conditioning. The
blindness is in the harness, not the model.

**The founding constraint.** The unit of abstraction is what fits one agentic mind's
effective context with iteration headroom (~10x drafts). Two budgets, distinct: the
token budget bounds the context; the CONCEPT budget bounds synthesis, and it binds
first — meaningful weaving degrades steeply somewhere around 3–7 threads while
retrieval barely degrades at all. That knee is a DISCOVERABLE CONSTANT per model, not a
design choice: hold a corpus fixed, vary fanout, score the weave (entailment pass rate,
register consistency, judged prose quality), find the knee, freeze it. The first brick
is that harness; every other piece inherits its constant.

**The tree.** Content lives only at leaves. Every non-leaf byte is DERIVED from
children — exteriors are locks under the standard discipline (regenerate, commit, the
diff is ratification). Budget per node; depth free (equal depth was considered and
dropped — balance emerges from the budget, and enforcing it forces padding nodes or
cuts across natural seams). The parent never reads a child's interior. Split when over
budget along the minimum-summary cut — a boundary is good exactly when much interior
compresses to a small exterior, deep modules made mechanical — and merge on underflow,
with hysteresis so the boundary doesn't thrash. The budget is the missing forcing
function for pruning: a full leaf makes deletion the cheap default and splitting the
ratified exception, inverting the economics that make every wiki grow forever. Where
the code side is under this method, pruning is mechanical: each lock class that grows
drains the prose — restatements die, justification remains. Steady state is a
constant-entropy commons of pure WHY on top of a growing lock set.

**The core is structured, not prose.** A node's exterior is: identity (a register
entry), atomic claims (micro-prose IS canonical — a claim's natural serialisation is a
sentence; the woven narrative is NOT — it is a rendering), relations (cross-edges roll
up at the lowest common ancestor, exactly the shape lock's bridge lines), and doors
(children). One-vocabulary, generalised: authoring the same exterior in two genres is
the drift engine reintroduced, and is forbidden. The register is MULTIMODAL — one entry
carries a lexeme, a glyph, a colour, eventually an affordance — and cross-surface
agreement is by PROVENANCE (every rendered string, box, and label carries the id of the
core element it renders), never by similarity judging. Byte-pinning, generalised.

**Genres dissolve.** Narrative, diagram, and UI are not three artifact kinds; they are
mixtures of modal primitives (sentence, label, arrangement, affordance) distinguished
by who controls traversal: the author (narrative), the eye (diagram), the visitor (UI),
the clock (slides, video). Doors are a generated space — traversal controller × modal
mix × consuming mind — and the consuming minds already include the agent (dense plain
text, the mid-tree functional summaries) and CI (byte-locked specs, the strictest mind
in the building). Facts flow up; FRAME flows down: the parent hands each child the
story so far and its chapter's role, the shared conditioning that kills thin-ness. A
parent's claim about a child is that child's thesis, and entailment along each edge is
the battery — refutation, never proof. Doors discipline builders THROUGH the schema
(the memo-culture mechanism, formalised past prose): emphasis descends via the frame;
doors choose stride, not structure (one tree, cut to the strictest regular consumer —
coarser doors render several levels per surface); door-needs grow the schema through
ratification; door-owned content is structurally refused.

**Substrates.** One human door: HTML5 — the DOM is literally a tree of modal regions,
and provenance-carrying render elements are transclusion, Xanadu's missing feature
rebuilt as a gate. The register compiles into CSS custom properties: visual vocabulary
by construction, not by lint. One mechanical battery rides the door for every genre on
it — headless computed-style checks, screenshots judged against claims, accessibility.
Machine minds get plain text. Paged print is deferred (print CSS first; a Typst render
function later only if the board pack's beauty demands it — renders are pure functions
over the core, so doors are additive). On the gate side, a MEASUREMENT INSTRUMENT, not
a second renderer: deterministic text layout in the Rust CLI (the cosmic-text /
HarfBuzz class — metrics from pinned, committed font bytes), with declared tolerance
margins on fit-gates, because a lock that fails on a sub-pixel shaping disagreement is
flaky, and flaky locks teach people to ignore locks. The committed-glyph-table trick
(measure once, ratify the table, pure arithmetic ever after — pretext.js's shape) stays
in the back pocket for client-side layout that must agree with the gates exactly.

**Behaviour is the fourth modality.** Interactivity is where the candidate rejoins the
method instead of extending it: a component's interior behaviour is a THEORY (a pure
`update: (State, Event) → State` — carriers, operations, laws, probed like ttl-store);
its exterior is a PROTOCOL under the same concept budget; state lives at leaves the way
content does; events roll up like edges; async is probed in a synthetic world (the
fabric brick's shape, with DOM carriers); and FEEL is this genre's judgment overlay —
authored, drift-gated, never derived. The checkability ladder runs prose (LLM-judged)
→ diagram (structural) → static UI (computed styles) → behaviour (property tests): the
deeper the interaction, the more mechanical the gate. The collapse it buys: the upper
tree IS the design documentation and the leaves ARE the shipped artifact, so
design/implementation divergence stops being a meeting and becomes a failed lock.

**Prior art, and the shared gap.** RAPTOR builds the recursive summary tree (retrieval
only, no contract); C4 is the drill-down for architecture diagrams; atomic design is
the roll-up for UI — and fails under LLM composition precisely because the component
inventory blows the concept budget, so the design system needs the tree treatment too,
the frame binding a narrowed slice per region; PreTeXt is one-source-many-doors at book
scale. Every field invented the hierarchy by hand; none has budgets, derivation, drift
gates, or provenance. The hierarchy is old; the CONTRACT is the contribution.

**Bricks, in order.** (1) The knee harness — derive the weave constant per model,
freeze it as a lock. (2) The minimal CLI — `check` (budget, no-content-above-leaves,
staleness by leaf hash), `split` (min-summary cut), `summarize`; every verb a judged
transaction, a refusal writes nothing. (3) The routing battery (an agent given only
exteriors must descend to the right leaf) and the leak/vacuity mutation pair: a pure
refactor that moves an exterior convicts it of leaking; a behaviour change that leaves
it fixed convicts it of vacuity — a good boundary's exterior is a FIXED POINT under
interior churn, the placer's criterion applied to prose. (4) The entailment battery per
edge. (5) The HTML door with the mechanical battery. Build it beside this workspace as
a probe-algebra project: every gap the doc domain exposes in the verbs is a field
report, and this is the method's first application whose artifact is not code.

First data (2026-07-13, the same day): brick (1) is built — `weave-knee`, a research
member in the layout-probe shape — and the pilot sweep REFUTED the naive knee. With
relations handed to the weaver pre-formed in the exteriors and a word budget that
scales with fanout, claude-sonnet-5 holds relation coverage at 1.000 through fanout
12: explicit-relation integration does not collapse where the 3–7 intuition said it
would, so `spec/knee.spec` honestly reads `knee: >= 12 — a lower bound, keep
sweeping`. What degrades instead is the judged weave QUALITY (4.0 at fanout 2,
3.0 by 8–12): every fact survives, the through-line thins — the original thin-ness
diagnosis showing up in the one dimension the mechanical scores cannot see. The
harness's next variations are therefore named by the data — and organised by the
transport lens: the weaver IS an approximate transport functor (structured exteriors
→ prose, the one door whose functor can never be code), and functors fail in three
graded ways, so the instrument measures exactly those. IDENTITY preservation (each
module stays itself — mention and claim coverage; the pilot shows this floor holds
through 12, a solved dimension). COMPOSITION preservation (the live one: plant
EMERGENT FACTS, true only of chained behaviour across two or three children with a
semantic step — "nothing Kelkel evicts is ever lost" — and score their coverage; the
k12 weave spontaneously surfaced "the twelve modules form a single ring" even as its
judged quality sagged, so emergence is the variable exactly where the coarse number
flailed). INVERTIBILITY (the return leg: a fresh judge, given ONLY the narrative,
reconstructs the topology — who feeds whom; reconstruction accuracy is
weave-faithfulness with no 1–5 anywhere, and it is brick 3's routing battery arriving
early). The 1–5 rubric split considered and DISSOLVED: functoriality replaces taste
with mechanical dimensions, judgment retreats to pure felicity. And the aim, named:
a weave is worth shipping when the functor is faithful WITH POSITIVE GAIN — it makes
explicit the composites the children only jointly imply (gain, measured by emergent
coverage) while inventing nothing underivable (faithfulness, policed by the foils).
A parent exterior earns its bytes exactly when gain is positive; concatenation is
the zero-gain baseline. The knee, if it exists, lives where gain goes to zero under
fanout — the pilot proves the instrument works and moves the question.

The v2 instrument is built (2026-07-13, later the same day): claims carry their kind
and parameter so composites derive mechanically; `emergents()` plants the ring-closure
fact plus per-edge sums and minima, each with a one-step-wrong foil; the judge rules
two censuses under two criteria (ENTAILED for identity, STATED for gain — a composite
the judge could derive herself but the narrative never makes explicit does not count);
a third model call reconstructs the topology from the narrative alone; and the spec
freezes both knees — the identity knee against the relation floor, the gain knee where
mean gain touches zero. The v1 trials are archived in `weave-knee/trials-v1/` beside
the spec frozen over them: they were measured under the v1 prompt and cannot sit in a
v2 curve without lying about what was asked of the weaver.

The v2 sweep RAN (2026-07-14, the v1 grid mirrored: fanouts 2–12, seeds 1–2,
sonnet-5 weaving and judging; sixteen trials, spec frozen over them). The findings:
the IDENTITY KNEE moved from "held through 12" to "held through 12 under the harder
prompt" — relation coverage 1.000 at every fanout, foil rejection 1.000, and the v2
legs the v1 curve never had both maxed: entailment-foil 1.000 everywhere, topology
reconstruction 1.000/1.000 everywhere — a fresh reader rebuilt the ring from the
narrative alone at every fanout. The naive 3–7 knee stays refuted, now with
invertibility evidence. The GAIN KNEE derived to 2, and that number carries a
methods caveat worth its own line: the rule is "last fanout before gain touches
zero," and gain hit 0.00 at fanout 3 on both seeds — but rebounded to 0.43 at 4,
0.34 at 5, and 0.46 at 8, which two seeds cannot distinguish from sampling noise. A
first-zero rule is brittle at n=2: either the k3 zero is real (a genuine synthesis
dead spot at three strands) or the knee rule needs a noise-robust form (last fanout
with mean gain above a floor across a window). More seeds at 3–4 decide it; the
committed spec honestly carries the twitchy number until then.

Honest frame, kept: exteriors are generated prose judged by generated tests — the
batteries refute, they never prove. A summary that survives routing, mutation, and
entailment is EVIDENCE of a good abstraction, not certainty. That is already the
epistemics the method runs on, and it is a better position than any surface nothing
checks.

## Candidate: warrant tiers — the confidence census

Origin (2026-07-13, the conversation straight after the exterior engine's first
brick): the honest frame says "evidence, not certainty" as if evidence were one
substance. It is not — the library already holds four distinct warrant strengths and
the boilerplate claims the weakest for all of them. Unbundled, "this is a good
abstraction" is four claims: the laws hold, the exterior does not leak, the
incompleteness is bounded, and it serves its purpose. The first three elevate —
separately, by different machinery. The fourth is a signature and never elevates
(intent is never inferred from conduct — the kernel-register lesson is an epistemic
boundary, not a policy).

The ladder, every rung already in-tree: SAMPLED (bounded grids, batteries — the
default the boilerplate describes); EXHAUSTED (a finite carrier's grid IS its domain —
a green Bool law is a decision procedure that ran to completion, and filing it under
"sample" is underclaiming); PROVED (`lean/ProbeBool.lean`'s `proved:` lines,
kernel-certified, statement-bites guarding the statements against vacuity);
DEFINITIONAL (ci.yml cannot drift from the GateRegistry because it is RENDERED from
it — the error class dissolved, nothing left to prove). Orthogonal and already paid
for: the mutation registers are certainty of a different currency — an enumeration of
the remainder, certainty about the uncertainty, the only kind available past Rice.

The brick: a WARRANT TIER on every law line — `sampled(grid)` | `exhausted` |
`proved` | `definitional` — DERIVED, never typed: the engine knows whether the grid
covered the whole carrier, the bridge knows which conjectures carry certificates, the
render layer knows what is generated. Rendered through the one equation source, so
every lock shows how each law is known. Two consumers ride it: the DEMOTION GATE (a
law may never silently lose warrant strength — climbing is a diff to ratify happily,
sliding is a refusal) and the CENSUS (laws per tier, the number the roadmap drains
upward the way qualify drained interior reads). Delta-render's specs-as-licenses gets
its sequel: warrants license moves — only a proved law licenses the optimization,
only a definitional exterior skips the leak battery. Free rider, as its deferral
predicted: the totality line ("every operator is total on its declared grid") rides
along, since this IS the spec header format moving for its own reasons.

The definitional program — the aim behind the top tier: a drift gate is a confession
that two authors exist. Every gate class is either INVERTIBLE (the artifact becomes
the image of a render function; the second author dissolves; the committed diff stays
as the ratification checkpoint — what dissolves is the author, never the signature)
or GENUINELY DUAL-SOURCED, and the reason is a register line. Three things never
climb to definitional: world-facts (measured, not derived — derive the instrument,
commit the reading as evidence; weave-knee is the template), the generators' own
correctness (fire drills and statement-bites live here, the named trusted base), and
purpose (a signature, forever). Steady state: the hand-typed bytes of the repo are
declarations, justifications, and signatures; every other byte is an image.

Honest frame, localized rather than deleted: the certificate is conditional —
guaranteed relative to the declared observables, the trusted kernel, and the ratified
purpose — with every condition named in the artifact instead of ambient in a
paragraph like this one.

## Candidate: the agent's terminal loop — perceive, decide, declare, sign

Origin (2026-07-13, the same conversation, one step further): warrant tiers name how
each fact is known; the definitional program names the aim (every analytic byte an
image). This candidate names where the AGENT lands when both are driven to their
fixed points — the working loop reduces to perceive (derived views) → decide
(judgment) → declare (a verb) → sign (a diff that is pure ratification), and
everything that used to sit between those verbs is an image, an attestation, or a
register line someone ratified. Each stage has a beachhead in-tree, a census that
drives it, and a fixed point that says it is done. The program is self-measuring:
four derived numbers, each drained toward zero-modulo-a-register, exactly the
mutation-survivor shape.

PERCEIVE — drain the fallback encyclopedia. Beachhead: `show`, `constrains`, `trace`,
`owes`, the startup orientation. Driver: the instrument rule's recorded readings —
each cluster of greps names the perception verb that would have answered it (`show`
was born this way), and the exterior engine's agent door supplies the subsystem
grain, budget-sized by the weave-knee constant. Fixed point: interior reads per
session → 0; the residue is exactly the properties the lock language cannot yet say,
each one a lock-language brick.

DECIDE — build nothing; feed it and protect it. The substrate's whole contribution
is calibration in (warrant tiers: spend verification effort only where `sampled`
lives) and contamination out (prose is judgment, stays hand-typed — the standing
danger as everything else mechanises is judgment getting templated too). Done when
deciding is the only stage consuming the concept budget.

DECLARE — close the verb algebra and DERIVE the closure metric. Beachhead: the
bundle verbs and the field-report loop that grows them. The missing piece is the
measure, and it is the standing question pointed at the second rule: "how much of
this commit went through the verbs?" is currently a feeling, but the journal knows
what the verbs carried and git knows what changed, so VERB COVERAGE PER COMMIT is a
derivable ratio — a census line, derived never typed. The gap between journal and
diff IS the field-report list, computed instead of noticed. Fixed point: non-verb
code bytes → 0, remainder ratified in a register.

SIGN — the pure-ratification diff, the one stage with no beachhead and therefore the
new brick. A commit today mixes regenerated images, verb-carried code, and hand
judgment, so ratification attention is diluted by material a machine already vouches
for. The move: every generated hunk arrives ATTESTED — content-keyed by the
derivation that produced it, the keying the gate verdicts already use — so a commit's
reviewable surface reduces to declarations and justifications. "The diff is pure
ratification" then means literally: everything in it is machine-attested (skim) or
judgment (read). Composes with the demotion gate: an attested hunk whose derivation
lost warrant strength is a refusal. Fixed point: unattested generated bytes → 0.

Dependencies, already mostly sequenced: warrant tiers feed perceive's calibration and
sign's demotion rule; the exterior engine feeds perceive's subsystem grain; verb
coverage needs only the journal that exists; the attestation keying exists in the
gate verdicts. No new theory — the method pointed at its own interface, four times.

The constructivity claim, stated so it can be refuted: at the four fixed points the
repo is AS CONSTRUCTIVE AS POSSIBLE. Every analytic byte is an image; synthetic
truth enters only as a derived instrument's committed reading (weave-knee is the
template); the trusted base is enumerated and drilled, never self-attesting; purpose
is a signature. No method exceeds this — a derivation cannot manufacture information
about the world or about intent — so the residue is not a shortfall but the floor,
named. The one frontier that moves a boundary rather than draining a number: the
cubical arm, where proved cross-representation agreements become computing transports
and a whole lock family climbs proved → definitional in one move.

Honest frame: the fixed points are asymptotes approached by ratified steps, not
states a sprint reaches; each census will spend most of its life mid-drain, and the
loop is real the whole way down — every drained line makes the agent's next session
more perceive-decide-declare-sign than the last.

The transport arm, expanded (named a frontier above; the purchases, so they are not
re-derived): a computing transport — constructive univalence's working face — moves
certification from PER FACT to PER BOUNDARY: prove one equivalence between two
representations and every law crosses free, including laws not yet discovered. What
that buys, concretely: the mirror class of locks dissolves (two-authors-plus-a-
comparator becomes inherited agreement); the world's proof corpus becomes consumable
wholesale (one isomorphism to a mathlib structure and its entire theorem library
arrives already-true of the Rust carrier — the structure identity principle as an
elevator where the bridge climbs retail); a refactor ships its own spec (laws,
observables, expectations transport; mutant sites and coverage do not — transport's
boundary lands exactly on the exterior/interior grain the placer already draws);
releases ship their own migrators (a representational version change carries
consumers across, and every reliance re-certifies as an executed program, not a
changelog read); the exterior engine's doors inherit fidelity instead of proving it
(a new door is a functor definition, not a renderer plus a battery); and perception
completes (any fact in any presentation, computed from the thing itself, warranted
definitional). Bounds, kept: nothing sampled climbs, nothing judged climbs,
enumerable or formalisable carriers only — effectful theories transport at their
model. The pragmatic path is the repo's standing lesson pointed at type theory:
never author the trusted base — pin a kernel that exists (Lean today, cubical Agda
`--safe` when transport must compute), bridge through gated agreements, and let the
bites keep the statements honest. A field report from before this repo existed: an
attempted HoTT language with a first-generation agent failed exactly as thin wholes
fail — a kernel is the one artifact whose value is entirely global invariants, and
nothing was checking them. The blindness was in the harness, not the model, there
too.

## Candidate: the freedom budget — no anonymous degrees of freedom (brick 1 BUILT)

Origin (2026-07-14, the conversation that ran from "tests are observational
evidence" through univalence to the CLI): three threads that arrived separately and
turned out to be one claim. The aim is not zero degrees of freedom — it is zero
ANONYMOUS degrees of freedom. Judgment is a degree of freedom with a name and a
signature; a bug is a degree of freedom nothing constrains and nobody owns. So
"eliminate bugs by construction" decomposes: every dimension of an artifact is
either DERIVED (freedom zero — render it, never author it) or DECLARED (freedom
owned — a register line, a signature), and the gate is that nothing sits in
between.

The formal footing, so the intuition has something to stand on: take a spec and
consider the space of implementations satisfying it. Contractible (one inhabitant
up to identity) means the spec forces the implementation — that is what "derived"
means, stated as mathematics, and a uniqueness theorem is its proved-tier form.
Multiple components mean the spec genuinely underdetermines — the components ARE
the decision register, one line per component, and choosing one is the signature.
The paths inside a component — implementations that differ but are equivalent —
are exactly what univalence quotients away: freedom that connects observationally
equivalent programs is not freedom at all, and a language that identifies
equivalents cannot even state the distinctions that don't matter. The placer knew
this first: "a good boundary's exterior is a fixed point under interior churn" is
the statement that the interior is contractible as seen from outside. And the
mutation census gains its true name: an empirical sampler of the component count.
A surviving mutant exhibits two points no probe distinguishes — either they are
equivalent (a path; quotient it, which is why deleting the degree of freedom
beats ratifying it) or they sit in different components (a real decision; the
register line names the one chosen).

The constructive half is already built and running: the bundle CLI. Its verbs are
introduction rules — the reachable tree-states are an inductive type, ill-formed
states not rejected but unreachable, and a refusal writes nothing. The journal is
the proof term (every state carries its derivation; `replay` re-checks it), field
reports are completeness counterexamples (a well-formed state the constructors
cannot reach, named), and `bundle pin` is the de Bruijn criterion arrived at
independently — the kernel is not rebuilt by the tree it is checking, mid-check.
What keeps the property empirical rather than proved is one number, verb coverage:
hand edits still exist, so the reachable set is not yet closed under the
constructors. At coverage 1.0 the drift gates on verb-covered artifacts become
unable to fire BY THEOREM — decoration by proof rather than by rot, the arrow
inversion completing itself — and the residual gates defend against exactly one
thing, bugs in the kernel, which is what a residual gate should do.

The path to the univalent quotient is short because most of it is recognition,
not construction. The shape catalog is already the poorer language — a law can
only mention declared operations; there is no syntax for reaching into a carrier
— so the unstatability property already holds at the layer where theories live,
and what was missing was the proof that it is real rather than accidental. The
interior is exempt by design — representation is where performance judgment
legitimately lives, the placer's exterior/interior split applied to freedom
itself. And univalence proper arrives by pinning (cubical Agda `--safe` when a
transport must compute), never by authoring — the transport arm's standing rule.

Brick 1, the swap drill — BUILT (2026-07-14, `src/kvstore/twin.rs`, cfg(test),
lib-side). The ttl store reimplemented over a genuinely different representation:
a key-ordered map to (value, ABSOLUTE expiry) where the primary keeps a sorted
entry list with a (born, ttl) split — no Entry, no Vec, no relative life stored
anywhere. Same operations, same variable letters, same observation type. The
drill: the discovered law list must be BYTE-IDENTICAL under the carrier swap. It
is — the theory names a true equivalence class; the spec language did not leak
representation. And the drill can fire: a first-write-wins merge (a behaviour
change, not a representation change) moves the list — the bias law names which
side wins and cannot survive the flip. The qualify and tier censuses counted the
twin and their diffs are the ratification.

A reading, recorded (the instrument): building the twin required opening
`kvstore/internal.rs`, because the committed ttl-store spec UNDERDETERMINES the
boundary semantics an equivalent implementation needs — that liveness is strictly
before expiry, that dead entries linger unswept until the next tick, that merge
unions raw entries rather than live ones. The laws held byte-identical anyway,
which is the honest bound stated by the drill itself: identical law lists are
sampled-tier equivalence — the grid may simply be blind to a boundary the spec
never speaks. The reach names the next brick: an OBSERVATION CONTRACT the spec
language can state (the liveness comparison is pinned today by probes, not by any
law), so that the next twin is buildable from the spec alone. When that holds,
the spec is definitionally sufficient for its own carrier swap — which is what
"the exterior is the interface" means, made mechanical.

Bricks, onward. (2) A twin per settled theory — the census is theories-with-twins
over theories, drained like every other census; each green twin is a ratified
"this theory names an equivalence class", each red one a leak found before it
bit. (3) The reconciliation gate: the mutation census against the decision
register, bijectively — every surviving mutant maps to a declared judgment line,
every judgment line has a surviving mutant; a survivor with no line means the
design lied about its freedom, a line with no survivor is a vacuous ratification.
(4) Promote one twin equivalence to proved: state the carrier equivalence in a
pinned univalent checker, transport the law list wholesale, and retire the
sampled comparison to the drill role the fire drills play today.

Honest frame: the swap drill refutes leaks, it never proves their absence — a
bounded grid can miss the observation that separates two carriers, and byte-equal
law lists are evidence the quotient holds, not a theorem that it does. The ladder
from one to the other is warrant tiers, and this entry is its "definitional
program" pointed at the change medium: the same three-step everywhere — recognise
the quotient, gate it cheaply, prove it only where proof pays.

## Candidate: the language constructor — the change medium becomes a generated artifact

Origin (2026-07-14, the conversation that ran from "is there an algebra of CLIs?"
through Lucid and DBSP to "the language for agents"). The recognition that starts
it: everything that made bundle the RIGHT KIND of CLI lives in no verb in
particular. Judged transactions, refusal-writes-nothing, states reachable only
through constructors, the journal as derivation, replay as checker, pin as kernel
separation — none of that is `add`'s code or `edit`'s code; it is the harness.
What is verb-specific is only a signature (name, typed arguments — seven argument
types cover all fifteen verbs) and a judgment. So the freedom budget of "a
morphism-constrained CLI" partitions cleanly: the harness is DERIVED (parser,
usage, dispatch, refusal plumbing, journal, replay, atomicity — identical for
every CLI in the class), and the verb list is DECLARED (each verb a signature
line: `judge: State × Args → Result<Effect, Refusal>`, with the harness owning
the write). The crucial inversion: today "a refusal writes nothing" is true
because the author was disciplined fifteen times; in the generated form the verb
author CANNOT write on refusal — the freedom "when does a verb write" is not
declared and defended, it is deleted, unrepresentable. Bundle is a hand-built
witness that the class is inhabited; the constructor makes membership derived.
There are already two hand-built instances in this repo (bundle and weave-knee's)
— the classic signal that the abstraction is real.

The name, chosen for its triple truth: in type theory, constructors ARE the
introduction rules; this thing constructs languages OUT OF constructors; and
every language it emits admits only constructor-reachable states. Each emitted
instance is a BY-CONSTRUCTION INTERFACE (equivalently: a judged medium; to the
planning literature, an action language; to a mathematician, the declaration is
a presentation — generators and relations — and the harness compiles the
presented object).

The term language (eval). Verbs compose: `bundle eval <program>`, where a program
is a term in the free monad on the verb signature functor and the whole term is
ONE atomic judged transaction — any refusal anywhere and nothing is written. The
term goes in the journal as the proof term; replay re-evaluates it. Three design
commitments made before the language exists, because they cannot be retrofitted:
(1) TOTAL — iteration only as fold over lists the system itself derives, never
general recursion; every term's judgment terminates by construction, or the
proof-term property dies. (2) DAG-shaped, not word-shaped — sequencing forced
only by genuine data or effect dependency, so independence is syntactic:
independent subterms judge in parallel, and squash normalizes a graph (where
commutation is visible) rather than a string (where it must be argued). (3) The
refusal monad's alternative structure makes recovery combinators safe —
atomicity is a property of the transaction boundary, not threatened by handling
a refusal inside the term. Consequence for the roadmap's own fuel: field reports
stop being "missing verb" and become "missing combinator," a far slower-growing
set.

And the emitted languages COMPOSE. Two declarations glue along their shared
argument sorts (the small argument-type vocabulary is the gluing site): verb
signatures compose by sum, carriers by product, and the free monad on a sum of
signatures is the à-la-carte result — so a term may span domains and is still
ONE atomic judged transaction, because atomicity and the refusal boundary are
harness properties, not domain properties. Cross-domain atomicity is the thing
no schema-per-tool ecosystem can say: "edit the code AND move the tracker" as a
single term that lands whole or not at all. The delta world composes for free —
Z-set deltas over disjoint relations commute, so composed domains inherit the
multi-agent merge property unchanged — and the topography already knows this
shape: seams between theories (date calculus ↔ ttl store on Duration) are
exactly what shared sorts between composed domains are. Composition is a
colimit of presentations, performed at the declaration layer; the constructor
needs no new code to support it.

The execution layer, general and built once. Two dualities and one calculus:
the term language is monadic (what a program may do next); the journal is
comonadic — a Lucid stream of tree-states with transaction succession as `fby`,
and the medium's deepest laws are intensional statements over that stream
("a refusal writes nothing" is a `fby` equation). Lucid implementations evaluate
by EDUCTION — demand-driven, with a warehouse of memoized values keyed by
context — and this repo already evaluates that way without saying so: a gate
verdict content-keyed by tree hash IS a warehouse entry, and `owes` is demand.
DBSP supplies the calculus that unifies it: for ANY computation Q over streams
of abelian-group deltas, the incremental version is D∘Q∘I — fully general as a
correctness framework. The efficiency theorems are what's restricted: linear
operators pass deltas through free, bilinear ones (join) cheaply, and the chain
rule propagates the wins through composition. For an opaque Q (rustc, a test
run) the transformation degenerates to integrate-recompute-memoize — which is
exactly the eduction warehouse. So the warehouse is not a second mechanism
beside the delta calculus; it is the calculus's degenerate case. ONE execution
layer: every node is D∘Q∘I; a node EARNS delta propagation by declaring group
structure; everything else gets memoized recompute; hand-authored incremental
maintenance is refused everywhere (it is the canonical anonymous degree of
freedom — a second implementation of the same view, related to the first only
by discipline).

The generality boundary is the abelian-group requirement on deltas, and it is a
fact about representation, not about code: a TEXT file's edit is not a signed
multiset of anything useful, but the verbs already left text — `edit`'s own
signature treats a module as a keyed set of items. The ideal inverts the current
arrangement: the item relation (module, item-name, body) becomes the source of
truth and the `.rs` file a derived rendering, canonical order and all. Then the
Z-set atom boundary lands exactly on the placer's exterior/interior split — the
item body is the atom, its interior the exempt representation freedom — the same
line discovered twice. Payoff: qualify is a projection, tiers a partition, the
instrumentation census a relation, the reconciliation gate a join — censuses
become incremental BY THEOREM, with the wholesale `BLESS_*` recompute demoted to
the gate oracle. Dependency verdict: build the kernel in-house (a few hundred
lines — Z-set, I, D, z⁻¹, lift, join, distinct), not the Feldera crate; this
kernel sits under the judgment layer, and our own code goes under `#[mutate]`
while a dependency's interior is exempt from every instrument we have. The DBSP
laws (I∘D = id, linearity per declared-linear operator, bilinearity of join)
freeze as theory shapes — the execution layer becomes the next settled module in
the topography, with naive recompute as its swap-drill oracle. Precedent:
schemata displaced cargo-mutants for the same reason. Performance is the
adoption property (the temptation to bypass a judged interface is latency), and
the calculus makes the strong form reachable: the judged path becomes the
CHEAPEST path — a raw edit owes wholesale gates against a new tree hash, a
judged edit is a delta that maintains derived facts by theorem — at which point
the temptation inverts. The checkable form is structural ("no census recomputes
wholesale on the transaction path"); wall-clock budgets stay evidence, never
locks.

The agent claim, stated with its bounds. An LLM agent's characteristic failures
are medium failures, and the stack answers the taxonomy point by point: partial
writes → atomic terms; acting on stale beliefs → eduction (perception is derived
against the current state hash or explicitly owed — a stale belief becomes a
state the medium cannot express); transcripts instead of derivations → the
journal as proof term; runaway scripts → totality at the grammar; ambient
authority → the verb boundary as the space of expressible intentions. None of it
constrains the agent's intelligence — it constrains the medium so intelligence
is the only thing left to vary; choosing the term remains the agent's judgment,
signed per transaction. The honest bounds: (1) the language governs only the
formalized region — the world brought inside a declared carrier — and agents
still live at the wild boundary; the claim is that the constructor makes the
judged region cheap to EXTEND, one declaration per domain. (2) It is not "the
language for agents"; it is the language CONSTRUCTOR — instances are
domain-bound, the scheme is what generalizes, and an MCP server is just one more
derived rendering of a declaration (schemas for invocation are what the tool
ecosystem has; judgment, atomicity, journal, and perception discipline are what
it lacks). One property matters most for what comes after: abelian deltas are a
MULTI-AGENT property — independent transactions from concurrent agents merge by
addition, and where two terms do not commute the algebra does not fail, it
DEFINES the conflict. Non-commutation is what a conflict is, named rather than
discovered at merge time.

The limit cases, captured while they are cheap. Downward one layer: an OS is
what a platform must be when userspace is arbitrary machine code — hardware
refuses what the language cannot. With a userspace of judged terms, each OS
organ dissolves or changes role. Isolation moves into the grammar
(cross-domain interference unstatable, not refused) — Singularity proved the
mechanism with software-isolated processes and typed contract channels, and
made the MMU optional defense-in-depth: HARDWARE PROTECTION DEMOTED TO THE
RESIDUAL GATE, defending only against harness-kernel bugs. The filesystem
dissolves into a journal plus a content-addressed payload store — which
`bundle.journal` + `bundle.payloads` already is: a log-structured,
content-addressed FS whose journal is never truncated (ext4 computes the same
derivation and throws it away after recovery). The scheduler becomes the
eduction engine — "what runs" is "which demanded facts are stale," budgets as
refusals. Drivers are declared domains at the wild boundary; legacy software
enters as opaque operators with content-keyed verdicts (rustc is already
treated exactly this way by `gates`; a whole OS in a VM is the same move at
larger grain). Two bounds: Spectre-class timing channels are observations
outside any grammar, so the residual hardware gate earns its keep; and every
language-based OS died of the compatibility moat — the reason this is timely
rather than nostalgic is that AGENTS ARE THE FIRST USERSPACE FOR WHICH THE MOAT
DOES NOT BIND: they need media for action, not shrink-wrapped binaries, and
regenerating software as declarations is the one thing that stopped being
expensive.

Downward again: the metal. Branch prediction and out-of-order execution solve
for bad code — hardware sympathy is an author discipline, and discipline
cannot be socialised, so the CPU compensates. The precise form: OoO execution
IS a dataflow engine — Tomasulo reconstructs at runtime, speculatively, per
instruction window, the dependence DAG the compiler had (SSA is that graph)
and discarded in lowering to a linear ISA. A derivation computed, thrown away,
and re-derived downstream at the cost of most of the die — and of Spectre,
which is what it looks like when hardware acts on unjudged guesses that leak
through an observation vocabulary the ISA never speaks. If terms arrive
DAG-shaped and total, the machine simplifies: no prediction (control flow no
longer hides data flow), no reordering (the linear order was never real),
latency tolerance from parallel slackness — many ready nodes — rather than
speculation. GPUs and Groq's deterministic TSP are the existence proof that
when the software contract changes, hardware simplifies AND wins; Itanium is
the honest failure to keep in frame (a static schedule against dynamic memory
latency loses — the DAG must buy tolerance, not a timetable — and the binary
moat binds, voided again only by an agent userspace). The Om experiment (an
FPGA interpreter, asking how few primitives suffice) is this limit's drill:
the machine whose ISA is the term language, an eduction engine in silicon,
with "how few primitives" as the hardware form of the verb-coverage number.
Hardware sympathy becomes a medium property, not an author discipline — the
freedom-budget move applied to the metal.

Priority (decided 2026-07-14): this trumps the tree-of-minds work — it is
foundational to how one programs with probe-algebra — and tree-of-minds becomes
the first dogfood consumer: many minds, one judged medium, commuting deltas
merged, non-commuting ones surfaced as the decisions they are. Its acceptance
test is the observation-contract test at project scale: buildable from the
declared surface alone; any reach around the verbs is a field report against the
foundation from day one.

Bricks. (1) Found the `cli!` declaration form and its generated harness, through
the verbs. (2) Dogfood: declare bundle's own fifteen verbs, generate, and pin
the first lock — generated usage text byte-identical to the hand-written one
(`spec/cli.spec`); acceptance: both existing journals (root and weave-knee's)
replay under the generated harness — the journal format is declared surface, not
derived interior. (3) The Z-set kernel as a theory-bearing module, laws frozen,
naive-vs-incremental as its oracle gate. (4) The item relation as source of
truth, files as renderings, censuses migrated to derived deltas. (5) eval, on
the generated harness, so every emitted language is born with it.

Honest frame: D∘Q∘I is a theorem; everything downstream of it here is design
warranted by two hand-built instances and one running warehouse. The agent claim
is a hypothesis with a named test (tree-of-minds builds on it without reaching
around the verbs), byte-equal naive-vs-incremental is sampled equivalence like
every drill in this repo, and "the judged path is the cheapest path" is a goal
the structure makes reachable, not a property it already has.

A deferral, recorded so it is not re-litigated (2026-07-15, the substrate
conversation): the LIMIT of this program is a zero-dependency trusted base — the
five-construct kernel, a total-term evaluator, the effect rim (journal, blobs,
spawn, clock), a boundary parser, the hash; ~2–3k auditable lines — with every
toolchain above it (rustc first) permanently FENCED as an opaque operator, never
trusted. Rust is the scaffolding language for bricks 3–5, and the pivot, when it
comes, is a swap drill at substrate scale: the freestanding tissue certified by
differential against the Rust implementation over the committed corpora, the
Rust kernel demoting to oracle exactly as cargo-mutants demoted to schemata.
Two pivot-time problems named now and deliberately deferred: rustc's
REPRODUCIBILITY (pinned-toolchain builds are mostly deterministic; the wound is
provenance — rustc is built by rustc, the trusting-trust shadow; diverse double-
compilation is the known counter), and TOTALITY's edge (operator bodies that
refuse the total term language stay behind the spawn door; where that line lands
is measured, not asserted — the Om question). Neither blocks the bricks; both
gate the pivot.

Brick 3's second half BUILT (2026-07-15, `src/discover/eduction.rs`, through the
verbs, same sitting as the first half). The executor is a Circuit: an operator
DAG acyclic by construction (`wire` refuses an input that names no existing node,
and any wiring after the stream starts — the DAG is declared whole, then the
stream runs), evaluated by EDUCTION — `tick` only records what arrived, `latest`
demands a node, and only the demanded ancestor cone is judged, each node catching
up through the recorded ticks by its operator's delta rule. Linear operators
(add, neg, delay) pass deltas straight through; join pays the product rule from
its two integrated inputs; distinct is the nonlinear rule D∘distinct∘I, and its
recompute goes through the WAREHOUSE: values keyed by the content of the
integrated input they were computed from — never the node, never the tick. The
probes pin the recognition, not just the mechanism: two distinct nodes walking
the same content share verdicts node to node, and a retraction that returns the
content to a judged value returns both to standing evidence — the owes/gates
verdict table is this same discipline in the degenerate case, with rustc and the
test suite as the uninterpreted operators and the tree as the integrated input.
The oracle gate is naive recompute over the same DAG: every node's full value at
every tick from the plain kernel operators, equal to the incremental answer over
a grid of circuits and seeded feed schedules, demanded eagerly and again only
once at the end (catch-up judged against the same oracle). Eduction's economy is
pinned in node-ticks: a tick judges nothing, a repeated demand judges nothing
new, a later demand pays only what is still owed. All sixteen of the module's
schemata sites died to the four probes on the first sweep; the censuses moved
(qualify REFUSES it — mutating receivers, machinery like the engine; tiers says
ALGEBRA) and their diffs ride the commit.

Deliberately deferred from this brick, so it is not re-derived: the Opaque node
(an uninterpreted operator with a content-keyed verdict, the warehouse's real
tenant) waits for the item-relation brick, because its integrated input — the
tree — does not exist inside a `ZSet<K>` circuit; when items are Z-set rows, the
site-keyed sweep's verdicts become exactly a Distinct-shaped view maintained by
this executor, which is the first paying customer already named below. Readings,
recorded per the instrument: before naming the new types the pinned trybuild
stderr was grepped for ident collisions (the Zset/Window lesson applied by hand)
— "will this def name move a pinned diagnostic" is a perception no verb speaks,
though the lock's own census computes the answer; and zset.rs was opened to read
its authoring idiom before writing the sibling module — house style is a
question nothing ratifies. One verb lesson, the verb teaching rather than
lacking: `edit` names a generic impl with its parameters (`impl Circuit` was
refused as a rename; `impl Circuit<K>` held the signature).

Brick 3's first half BUILT (2026-07-15, `src/discover/zset.rs`, through the verbs;
the one hand edit was the `theory!` ops list — macro invocations are not
addressable items, twice observed today with `system!`, a named verb gap). The
kernel entered the repo the way the method wants everything to enter: as a
DISCOVERED THEORY. `ZSet<K>` (canonical: zero weight is absence, so structural
equality is observational equality) with zero/+/neg/join/distinct, and bounded
traces with delay/integrate/differentiate — and the engine, running the
operators, discovered the entire algebra unprompted: the abelian group WITH
INVERSE (the law that makes D exist), neg an Add-homomorphism, join's ring face
(commutative, associative, annihilated by zero, distributing over Add), distinct
a projection, and the DBSP theorem itself — `integrate(differentiate(s)) = s`
and `differentiate(integrate(s)) = s` — as two discovered round-trips, delay
commuting past both. Twenty-one laws, every operator in a law, frozen in
spec/zset-kernel.spec with its 38-mutant algebra-mutation lock beside it.

The instruments each spoke once, and each was right. The PLACER refused the
first declaration — Z ops and trace ops shared no nets — which exposed a real
omission: the kernel's cross-sort operators, `impulse` (a delta enters the
stream at NOW) and `latest` (eduction reads the current value). Adding them
settled the module (8 of 8) and the engine found `latest(impulse(a)) = a` on its
own. COHESION named the kernel the fourth latent split (group / calculus /
bridge pair) — ratified keep-whole beside doc flow: I and D are statements about
the group, and splitting would sever the calculus from the algebra it computes
with. The MUTATION battery ratified 20 degrees of freedom the law language
cannot yet see — constant-zero join satisfies every ring law over an ideal;
distinct is unconstrained at negative weights; latest is pinned only through
impulse — each a sharper-shape brick waiting (`distinct(a + a) = distinct(a)` is
the missing stanza's shape). And a naming lesson for the census: enum VARIANTS
named `Z`/`T` collided with the GDP kernel's type-level `Z` in rustc's
trimmed-path diagnostics, moving pinned trybuild stderr — renamed Zset/Window;
new def names must dodge idents that pinned diagnostics print.

What is NOT yet built, so it is not re-derived: linearity and the product rule
are pinned as grid probes (`the_operators_earn_their_delta_shortcuts`) because
the catalog cannot state cross-sort homomorphisms — that is a shape brick; the
indexed Z-set (real relational join, not the same-key multiplicity core); and
the executor — the operator DAG, eduction, and the warehouse recognized as the
nonlinear rule's memo — which is brick 3's second half, with naive recompute as
its oracle gate.

Brick 1 BUILT, and half of brick 2 with it (2026-07-14, `src/discover/cli.rs`,
through the verbs except one named fallback). The declaration form landed as data
stanzas, the catalog idiom, not yet a macro — `cli!`'s sugar starts paying when
judgment binding arrives, and sugar before then would be syntax with nothing to
sweeten. What exists: `Sort` (the argument-type vocabulary — Module, Item, Payload,
Declaration, Journal, Theory, Term; the predicted "seven types cover all fifteen
verbs" held exactly, with `operator` folding into Item and `theory-name` into
Theory), `Slot` (sort × label × mode — the sort owns its shell-quoting discipline,
the mode owns its brackets), `VerbSpec` (a declared `sibling` renders on the
previous verb's usage row — gates | owes is a presentation judgment, not a derived
fact), and `CliSpec` with the first two harness derivations: `usage()` and
`parse()` (argv → bound invocation or the usage refusal, a refusal binding
nothing). `CliSpec::bundle()` declares the fifteen verbs — the hand-built witness
is now the class's first instance.

The first lock landed stronger than its plan: not generated-vs-hand-written
equality (`spec/cli.spec`), but the hand-written usage text DELETED —
examples/bundle.rs renders its usage from the declaration, so agreement is
definitional and the probe's byte-pin (lib-side, where the root sweep can see it)
is the ratification checkpoint on the declaration itself. The mirror stage was
skipped exactly as the definitional program says to skip it: the second author
dissolved, the signature stayed. Two censuses moved and their diffs ride this
commit (qualify-reasons and tiers count the new file; schemata counts nine new
sites, all killed by the module's three probes).

Field reports from the founding: (a) `add` founds a nested module's FILE but not
its MOUNT — member genesis's "mounts included" held at a crate root, not inside
`discover/mod.rs`; the mount itself went through as a second `add` on mod.rs, so
the verbs reached in two moves where one was predicted. (b) `grow`'s FOURTH
firing: the boundary discipline refused a loose `pub fn bundle_cli` (rightly — it
became `CliSpec::bundle()`, the `GateRegistry::declared` idiom), and moving it
into the impl was interface growth `edit` refuses ("2 held, 3 offered"); the
fallback arm carried it. (c) A discipline interaction worth knowing: an unmounted
file tiers INTERIOR ("not pub-reachable") and inherits the inward rule — the
raw-`String` refusals evaporated when the mount landed and the file tiered
ALGEBRA. The enforcement judged this session's code exactly as it judges hand
code, which is the point. Readings, recorded per the instrument: the `#[mutate]`
label semantics and the tier-rule dispatch were both learned by opening interiors
(boundary-spec-macros, boundary-enforce) — the question they answered, "what will
refuse this payload before I offer it," is a preflight-judgment perception no
verb yet speaks.

Brick 2 BUILT (2026-07-15). `run`'s hand-written arity match is gone: argv flows
through `CliSpec::parse`, a refusal is the derived usage text and binds nothing,
and the arms keep only judgment — the declaration is now the single author of
what argv can say, with one honest residual arm (a verb the declaration speaks
but no judgment consumes refuses with the teaching text — declaration/dispatch
drift, named). The migration was itself a judged transaction: the suit edited its
own source (`bundle edit examples/bundle.rs run`, signature held, payload
stashed), which is the change medium eating its own change. Acceptance held: both
journals' replay differentials are byte-identical to their pre-change baselines
(the one standing divergence — examples/bundle.rs's line-4 edit predating the
payload store — unchanged and already named honestly by the report), refusals
verified uniform across empty argv, unknown verb, missing slot, and stray
argument, and the gates ran green with 926 verdicts carried, zero sites
re-judged. What brick 2 buys: a new verb is now a declaration plus a judgment
arm — parsing, usage, and refusal plumbing arrive derived — which is the cost
model the item-relation brick's perception verbs (the portal instances) assume.

## Candidate: the site-keyed sweep — the last long check dissolves into the delta

Named the day the gate supports landed (2026-07-14), from the same felt pain one
level down. The support projections made prose edits owe nothing; a SOURCE edit
still owes the full schemata sweep, because the attestation's warehouse key is
per-TREE: one changed function makes the transcript "foreign" and voids 866
innocent site verdicts. The key is too coarse — the same disease the supports
just cured at gate grain, recurring at site grain.

The change: key each site's verdict by the CONTENT OF WHAT IT IS EVIDENCE ABOUT —
the site's enclosing item plus its covering test items (the attestation already
records per-site verdicts and per-site covering tests; this is a keying change,
not a new instrument). On the next run, reuse every verdict whose item and
covering tests are byte-identical and re-judge only the moved sites: an ordinary
edit touches one or two items, so the judging phase drops from ~867 site-runs to
single digits — seconds, with the one build left as the compiler's own bill. The
moved-item set is not a diff heuristic: changes go through the verbs at item
granularity, so "what moved since the attested tree" is a JOURNAL QUERY — the
change medium built for judgment turns out to carry exactly the dependency
tracking the incremental sweep needs. This is the dataflow kernel's first paying
customer: site verdicts as a view over the item relation, maintained by deltas,
with the from-scratch sweep demoted to the oracle.

The honesty ladder is already ratified, not new: the retired since-green mutants
gate ran only the diff since the certified tree (incrementality-by-diff was
accepted policy), and the weekly from-scratch shards exist precisely to backstop
incrementality's one gap — a test edit weakening kills for unchanged code, which
is also per-site keying's gap (a test's behaviour can shift through code it
calls without its own text moving; item + covering-test keys cannot see that).
Sampled tier per change, from-scratch tier weekly: the long check survives only
where it should — the weekly backstop in CI, and a cold checkout with no
attestation.

A finding from the day it was named, recorded so the keying change inherits it:
timeout-as-detection is LOAD-SENSITIVE. A QoS-demoted run timed out two sites
whose ratified justifications are definitional equivalences; the false kills
read as stale register lines ("a stale exception is a lie"), the lines were
deleted, and the next full-speed run — correctly — found the survivors
unratified and went red. The doctrine (a timeout is a detection) is right for
CI, where load is uniform; locally it means a verdict can depend on the
machine's mood, and a register line can be deleted on a lie told by the
scheduler. Two consequences: a register deletion earned by a demoted or loaded
run deserves suspicion before ratification (the deleted line's justification
arguing DEFINITIONAL equivalence is the tell — definitional equivalences do not
die), and the site-keyed sweep should key timeout verdicts separately or
re-judge them before treating them as kills, so a slow machine cannot mint
false facts.

BUILT, the day it was named (2026-07-14, through the verbs; one interface-growth
fallback on `SiteVerdict` — which `edit` allowed, teaching that struct fields are
interior while impl method sets are surface). The shape as landed: each site's
transcript row gains a fourth field, the EVIDENCE key — FNV-64 (the verdict
store's fold) over the site's module file and every covering test's module file,
names mixed in. Module grain, not the item grain the candidate sketched: item
grain must wait for total impl addressing, because a missed generic impl would
carry a verdict whose code moved — the one direction this design refuses
everywhere (anything unresolvable keys empty, and an empty key is never carried;
reuse is earned, judgment is the default). `verify` now holds three honesty
tiers: countersign at a matching tree, incremental re-judgment at a foreign
same-toolchain tree (carry at standing evidence, judge the moved), full sweep
otherwise. Elder three-field transcripts parse with empty keys — one honest
transition sweep, then incremental forever. The covering-set gap (a test's
behaviour shifting through code it calls without any keyed module moving) is the
same gap the retired since-green gate ratified, and keeps the same backstop: the
weekly from-scratch shards, which carry nothing.

The timeout doctrine grew a fourth artifact the day's data demanded:
`spec/divergence.register`. Timeouts never carry (the load-sensitivity rule,
kept), but seven flips timed out on three consecutive runs including an idle
machine — and inspection confirmed each is DETECTED BY DIVERGENCE (two union-find
root walks that spin on any root when `!=` flips, the closure's fixed-point exit
deleted, the BFS seen-guard deleted, Euclid's base case corrupted, and the
exhaustive-vs-sampled grid gate flipped into enumerating 2^96 assignments — the
multi-GB worker class, caught by name). Re-proving non-termination at full limit
every run is a treadmill, so a ratified line — mechanism verified before signing
— lets the sweep carry that timeout at standing evidence, and the register is
judged ONE-WAY stale at every sweep: a ratified divergence that stops timing out
goes red and must lose its line.

What the instrument caught while the brick went in, each found by the evidence
keys refusing to stabilise: (1) cargo-nextest was absent on this machine, so
every sweep had been the fallback path — full suite per site, ~50 minutes, no
attestation written; installing it alone was 6x. (2) The coverage recorder keyed
touch files by PROCESS ID with append — a recycled pid appended one test's edges
under another's header, coverage sets flapped ~90 sites between identical trees,
and the sharp direction is a sole coverer clobbered into another test's file:
its site judged by a test that never reaches it, a false survivor. Filter-keyed,
truncate-created files fixed it; back-to-back baselines now differ on zero of
891 sites. (3) The worker pool sized itself by cores when bytes are the binding
constraint — twelve workers put 58 GB of pressure on this 16 GB machine (and an
orphaned by-hand timeout reproduced the documented runaway incident to the
letter); the derived default is now min(cores − 2, GB/8), knob kept. (4) The
name resolver needed the own-file case (an inline `mod tests` in `mod.rs` held
the coverers of 92 sites) and the full `:deaf -> ` suffix (a function honestly
named `deaf_battery` was truncated by the bare `:deaf` cut). Each fix carries a
probe.

The numbers: ~50 min (fallback, per change) → 301 s (full nextest sweep, the
one-time transition) → an ordinary edit now re-judges only the sites whose
module or coverers moved, at 2 workers on this machine — the example-file edit
that moved no site's evidence carried 799 of 891 mid-repair, and with keys fully
minted the steady state landed at 59 s for the COMPLETE gates run (format, lint,
test, mutation — 910 verdicts carried, zero judged). One more leak found on the
way there, the documented runaway incident one layer down: nextest puts each
test in its OWN process group, so `outcome`'s group kill reached cargo and
nextest but not a hung mutant's test binary, which reparented to pid 1 and spun
(ten orphans at ~70% CPU after two timeout judgments). The reap now also kills
our test binaries whose parent has become 1 — precise, because a live worker's
tests still hold their nextest parent.
The remaining fixed costs are the build and one baseline suite run (~35 s
together, warm), which are the dataflow kernel's next targets — site verdicts as
a maintained view over the item relation is this brick's principled form, and
the from-scratch sweep is already demoted to the oracle role that design needs.
Named while retiring `survives` for `outcome` (tri-state; the timeout named
apart): `edit` refused the signature change and the refusal text pointed the
way — "retire the item and add its successor" — which `collect` then carried
out; the mark census kept the item alive until prose stopped mentioning it,
which is `collect` working exactly as declared, and worth knowing: retirement
is uses-then-mentions-then-sweep.

## Idea, logged: semantic zoom — the rendering is a function of altitude

Source: the Pad++ papers (Bederson & Hollan, mid-90s zooming interfaces). Two of
their moves name things this project is already circling, and one is a design
constraint worth adopting before the item-relation brick lands.

First, their throwaway line is our thesis: "instead of showing huge numbers it
might make more sense to show the computations from which the numbers were
derived or a history of interaction with them." Every number this repo shows IS
a derivation — a census total, a verdict count, a settled-module tally — and the
journal already holds the interaction history. `delta()` and `constrains` are
the first renderings that answer with the computation instead of the value; the
idea says that is the general contract, not a feature of two verbs.

Second, SEMANTIC zooming: representation changes with scale non-geometrically —
zoomed out you don't get smaller text, you get a different rendering of the same
object. That is exactly the ladder a bounded mind climbs here already, ad hoc:
topography line → theory spec → item list → item body. The item-relation brick
makes it principled: if a module is a Z-set of (module, item, body), then each
zoom level is a derived rendering of the same relation, and "what does the agent
see at altitude k" becomes a declared function, not a pile of separately
maintained summaries. Pad++ built this for human spatial cognition; our consumer
is an agent with a token budget — the same vast-surface-through-small-aperture
problem, with context window as viewport. This is the exterior-engine candidate
("every surface a bounded mind consumes") given its rendering discipline.

Third, PORTALS: standing views that look anywhere on the surface and re-represent
what they see. Under the dataflow kernel a portal is precisely an incrementally
maintained query over the item relation — pan and zoom are cheap because views
are maintained, not recomputed on look. `constrains <module> <op>` is a
proto-portal that recomputes; the kernel makes portals the cheap default, which
is what would let perception verbs multiply without each one buying its own
walk of the tree.

Nothing to build yet — the brick underneath (item relation as source of truth)
is already queued. The log entry is the constraint to carry into it: design the
renderings as one family indexed by altitude, so the perception verbs that
follow are portal instances, not bespoke reports.

First application, named at logging time (Callum): THIS FILE. The roadmap is the
biggest single surface an agent reads — thousands of lines, consulted at full
altitude every session — and its structure is already implicit zoom (entries with
status, candidates, readings). A zoomed rendering — one line per entry at
altitude zero, expand on demand — is the cheapest honest test of the family,
and the consumer is real: every session opens with exactly this need.

Extended 2026-07-17 (Callum): a candidate viewport, a second consumer, and a
second delta source.

Viewport candidate: the tldraw SDK (the library, not their app). Semantic zoom
is native to it — custom shapes render differently per zoom level, culling and
text LOD are built in — and its store sits on incrementally-maintained signals,
so a Pad++ portal is just a shape subscribed to a maintained query: the same
claim the dataflow kernel makes, in a different medium. Field evidence from an
outside experiment: recursive frame-collapse — a frame folds into its parent
space as a single named node, at every depth — was buildable in an afternoon on
the SDK and made freehand architecture diagrams tidy. That discrete nesting
fits our ladder (topography → theory → items → body) better than continuous
geometric zoom does, since the ladder is discrete anyway. The constraint stands
regardless of viewport: the altitude-indexed rendering family lives on the Rust
side as a declared function; the canvas maps zoom to altitude and displays what
the kernel serves — zoom levels never live in shape code. Two costs, named:
the relation has no geometry, so placement is its own derived view (a layout
function, not hand-arranged positions), and the SDK's license trades a
watermark for a fee.

Second consumer: the human at the glass, beside the agent at the aperture.
Every verb is already a delta source — an edit is a two-row delta, a collect a
set of retractions — so a canvas subscribed to the delta stream shows agent
work live with zero added instrumentation: no invented event schema, the
mechanism IS the schema. And because "what the agent sees at altitude k" is a
declared function, an agent's context viewport is itself renderable as a
portal — watch the aperture move across the surface in one frame while its
writes land in another. Seeing an agent stuck, or sensing how the work is
going well enough to improve the process, falls out of watching deltas and
apertures; nothing extra is built for it.

Second delta source: human gestures, as coordination — not micromanagement,
and not approval gates (judgment stays off the critical path). A gesture on
the canvas compiles to a judged artifact, never open text feeding rules: a
region drawn around modules is a scoped work item, a wireframe sketched in a
frame is a dispatch — an agent picks it up as data and implements it — a note
dropped on a frame is a roadmap candidate. Human intent enters the medium the
same way everything else does, as deltas over the relation, and agents consume
it the same way they consume everything else. Ordering unchanged: all of this
subscribes to the item-relation bricks (relation at repo scale, maintained
views, the delta stream) and can arrive late; none of it can arrive honestly
before them.

## The item relation, brick 1: the tree recognized as data (BUILT)

Built 2026-07-15, the same day brick 2 closed the CLI loop — the two are one
program (the memory of the 07-14 session: item relation as source of truth, eval
as total DAG language, performance as the adoption property). What landed:
`src/discover/items.rs` — `Item` rows as (module, name, body), body the verbatim
segment `show` prints and `edit` holds; `ItemRelation::of_module` the addressable
projection of one module into a `ZSet<Item>`; `of_files` the group sum. The
recognition, not the inversion: files stay authoritative, the relation is derived
from them, and the renderer that makes `.rs` files derived renderings is a later
brick. Five probes pin the load-bearing facts — one span rule shared with `show`
(no second vocabulary), an edit is a TWO-ROW DELTA (retract old body, assert
new — the verbs are delta sources, which is the entire mechanism by which the
kernel will maintain censuses instead of recomputing them), the tree relation is
LINEAR in its files (an untouched module contributes zero to any delta), and
refusals are named. All four new mutation sites killed; qualify, tiers, and
schemata censuses moved with ratified diffs.

Field report, `grow`'s FIFTH firing: the relation's feed (`Bundle::rows`, the
(address, segment) row list) belongs on `Bundle` — same spans, same address
grammar, attached not loose exactly as the enforcement demands — and `edit`
rightly refuses interface growth ("an interface change is not an edit"), so the
method went in by hand, the fallback arm's one touch in this brick. Five firings
is a pattern, not a coincidence: the missing verb has a stable shape (grow an
impl's held signature set by one, judged — a signature ADDITION with the existing
set held, dual to `edit`'s signature-held body swap). It is now the most
field-demanded verb gap in the log.

Next bricks, named so they are not re-derived: (2) the tree walk — `of_tree`
over the workspace's committed modules, the relation at repo scale, with the
walker shared with an existing census rather than a second one; (3) the first
maintained view — feed verb deltas through the eduction circuit to keep a census
(candidate: the qualify census, whose oracle is the existing wholesale scan)
incremental, demoting recompute to the gate oracle; (4) the opaque node — the
executor's deferred tenant gets its integrated input (the tree as Z-set), and
build + baseline-run, the sweep's last fixed costs, become nodes in the DAG.

Brick 2 BUILT the same afternoon: `of_tree` — the relation at repo scale, walked
by `VerdictStore::files`, the NEW public face of the exact walk the gates'
support keys already trust, filtered to the `.rs` subset of the declared
`RustSurface` projection. No second walker, no second opinion about what the
tree is. The self-recognition probe pins it: the tree relation contains the row
that declares the relation, every row a walked `.rs` path at weight 1. All six
new mutation sites killed.

And the brick's real discovery, which rewrites the `grow` story: GROWTH IS
ALREADY AN `add`. The address grammar's own doc said it all along — two impl
blocks for one type are legal Rust and extending an address is not an `add`
collision — so `VerdictStore::files` and `of_tree` both rode in as SIBLING IMPL
BLOCKS through `add`, zero hand edits, judged transactions end to end. Five
`grow` firings, and the fifth (`Bundle::rows`, this morning) was the last one
that will ever fall back: the verb set was complete for growth all along, just
unnoticed. What remains of `grow` is only a PLACEMENT judgment — whether one
type's methods should live in one impl block — which is `place`'s business to
learn someday, not a new transaction. The most field-demanded verb gap in the
log dissolves into an idiom, which is the cheapest kind of brick there is.

Brick 3 BUILT (2026-07-15, the day's third): THE FIRST MAINTAINED VIEW — the
qualify census pair updates at the verb, and the `BLESS_QUALIFY`/`BLESS_REASONS`
ceremony is no longer owed at the edit. The pieces, each in its right home:
`boundary-enforce` grew `reasons_line` (qualify_line's complement — the partition
now reachable from text on both sides), `render_census_from`/`render_reasons_from`
(the format extracted to from-parts renderers, stated once — the wholesale walk
and the maintained update render through the SAME bytes), and `maintain_qualify`
(retract the loc's line from both files, re-judge the source, assert on the side
the rule assigns, re-derive both headers from the line sets). The bundle example's
`commit` now calls it on every writing verb: the crate's census pair is found
beside the journal, the bless env names are READ FROM THE COMMITTED HEADERS
(never configured — a member crate's pair maintains identically), and a module
outside the census's `src/` walk moves nothing. Seven probes pin the view: no-op
is byte-identity, an edit replaces exactly its own line, a partition crossing
moves between the files with both headers re-derived, a new loc grows the roster,
maintenance is idempotent, unparseable is no-signal, `tests.rs` is the walk's own
skip. The acceptance ran at the real tree: a judged no-op edit through the new
binary reproduced both committed censuses byte-for-byte, and the next `cargo
build` — the drift gate, now formally the ORACLE — was green with no bless env
anywhere. This is the dataflow design's claim made concrete at census number one:
the wholesale scan was not replaced, it was DEMOTED to the oracle role, exactly
the naive-recompute pattern the executor's probes already pin. Censuses remaining
for the same treatment: tiers, reasons' member twins, instrumentation, schemata.

Field report from brick 3 (a verb gap, named — then dispositioned): the
DEPENDENCY DECLARATION. Wiring the example to `boundary-enforce` took one hand
line in `Cargo.toml` ([dev-dependencies]) — manifests are outside every verb's
reach, yet a dependency edge moves the build graph, the perimeter's supply
chain, and the qualify walk's scope all at once, and manifests have changed 35
times in this repo's life — not rare.

The disposition, taken the same day (asked "does depend make sense in the
larger picture?"): DON'T BUILD THE VERB — the manifest is secretly a derivation
plus a signature, the standing question's answer one more time. Split it along
the freedom budget's axis: the EDGE SET is derivable from the item relation
(`use boundary_enforce` in a module body witnesses the edge, and WHERE the
references live — examples vs lib — derives the dev/lib placement; today's
hand line is witnessed by exactly two references a machine could have read);
the VERSIONS, FEATURES, and PROFILES are genuine decisions (nobody derives
`1.0.117`, and layout-probe's `opt-level = 2` already carries register-shaped
reasoned prose). So the endgame is the manifest joining ci.yml and the
perimeter ruleset as a RENDERED artifact — edges derived, decisions ratified
in a register, drift-gated against both; unused and missing dependencies
become census violations, not lints. Genesis already emits manifests from
declarations for the demo crates, so the renderer half has an in-tree
precedent. Note also the vocabulary: `depend` is TAKEN — `discover::depend`
is the reliance system, and its shape (a reliance is a register line, judged
against the record) is exactly the shape this disposition lands on: a crate
dependency is a reliance on an API surface, and the repo's precedent for that
species was never "a verb that edits a file." Priority: behind brick 4 and
the census conversions, per the level-up rule — and when its turn comes,
build the derivation, not the editor. The `grow` dissolution is the template:
the best answer to a missing verb is sometimes discovering the change was
never a free decision at all.

Readings from the authoring session, recorded per the instrument (asked "how was
it to author with the CLI," answered honestly). (a) `impl Bundle` is one item, so
reading ONE METHOD meant piping `show` through `sed` twice — a fallback reading
wearing a verb's clothing; a method-grain address (`show`'s inventory face scoped
inside an impl) is the perception half of the rung `edit` already has named. (b)
The blessing sequence is hand-sequenced convention: after a new module, the
author must KNOW to run `BLESS_QUALIFY`/`BLESS_TIERS`/`freeze_gates` — the first
build's two violations went unread and the fix was guessed from lore. `owes`
derives what gates the tree owes; it does not derive what BLESSINGS a change
owes, and that is a derivation wearing a convention's clothing — the standing
question's exact shape. (c) Payload formatting is the author's burden: the
transaction accepts bytes the fmt gate will refuse minutes later; the judged
transaction could carry that judgment at offer time. (d) On the credit side, for
the record: refusals that list the addressable roster convert a wrong guess into
the next correct call with no doc lookup — the property to preserve in every new
verb — and content-keyed verdicts eliminated superstitious re-running entirely;
the frozen-tree window during a gates run is the one honest cost, and it is
exactly what brick 3's maintained views shrink toward nothing.

Brick 4 BUILT (2026-07-15, the day's fourth): THE OPAQUE NODE — the executor's
deferred tenant, seated, with the tree as its integrated input. `Node::Opaque`
is `distinct`'s rule with the recompute handed to an ADMITTED TENANT the
executor never looks inside (D∘f∘I, verdicts warehoused by the integrated
input's content); `admit` is a judged transaction (the name is the operator's
identity in the warehouse, so a spoken name is refused, and both admission and
wiring close when the stream starts); and `carry` is the transcript's carry
spoken in the executor's vocabulary — standing evidence enters before the
stream runs, and a carried verdict answers a demand with the tenant never
invoked. The probes pin the recognitions, not just the mechanism: the oracle
grid runs opaque plans against naive recompute (the tenant's own function over
the fully integrated input); an opaque node whose tenant is distinct's function
agrees with the interpreted `Distinct` at every tick and through catch-up —
interpreting an operator changes what the executor can SAY about it, never what
it computes; the tenant's economy is pinned in INVOCATIONS (at most once per
novel content, shared node to node, zero on a retraction to a judged state);
and the headline probe feeds an opaque census tenant the ITEM RELATION with a
verb-shaped two-row edit delta — keyed by evidence, not by answer: an edit that
leaves the view unmoved still pays once, and the retraction pays nothing.

And the tenants are real: the schemata sweep's two FIXED COSTS are now nodes in
a live circuit — Source(lib cone) → Opaque(build) → Opaque(baseline), the cone
fed as one Z-set (Rust modules at item grain through `of_module`, the
manifest/spec/register remainder at file grain), the baseline keyed by the
build row, which embeds the cone's fingerprint. The LIB CONE is the declared
evidence of both verdicts: the Judged support minus what `cargo test -p
boundary-spec --lib` cannot read (`examples/` less the runner's own text, the
root `tests/`) — a conservative over-approximation in the gate supports' exact
sense, narrowing carries the burden of proof. The transcript grew an `evidence`
line naming the cone (an elder transcript parses with the empty key and carries
neither cost — the same honest transition the site keys made), and because
every site's evidence lives INSIDE the cone, a standing cone carries the WHOLE
verdict set wholesale: population checked against the census, any unratified
timeout forfeits the carry (load-sensitivity, kept), and the run re-attests at
the new tree for the cost of a walk. Acceptance at the real tree: the
transition sweep attested 953 sites and minted the first cone key; an
out-of-cone edit (a judged doc tweak to examples/bundle.rs) then earned the
COMPLETE mutation-gate verdict in 3.7 SECONDS — build carried, baseline
carried, 953 site verdicts carried, 0 judged — and the reverting edit did it
again in 3.6. The ladder so far: ~50 min (fallback) → 301 s (full nextest
sweep) → 59 s (site keys) → under 4 s (the cone standing). What remains when
the cone stands is literally one tree walk and one register comparison; what
remains when it moves is exactly what moved.

Field reports. (a) THE SIBLING-IMPL AMBIGUITY — the grow dissolution's residue,
hit in anger: growth by sibling impl mints a SECOND item at one address, and
`edit`/`show` rightly refuse to guess (`impl Circuit<K>` twice, once admit and
carry rode in as the grow idiom; `impl ItemRelation` twice since the tree-walk
brick). One hand edit fell back through the gap — respelling `admit`'s
parameter to the `Judge<K>` alias the lint gate demanded — because the verbs
could not name WHICH impl to hold. The verb set that dissolved `grow` now owes
the address grammar a discriminator, or the method-grain rung the authoring
readings already named: the same brick seen from two sides. (b) Payload
formatting, reading (c) reconfirmed with a workable idiom: `cargo fmt --check`
named drift in three authored payloads; running rustfmt on the payload FILES
outside the tree and re-offering them whole through `edit` kept the second rule
intact. Offer-time formatting inside the transaction is still the missing half.
(c) On the credit side: `edit` held enum-variant growth (Node) and struct-field
growth (Circuit, Transcript) as interior — the SiteVerdict lesson, now twice
confirmed — so the kernel's entire change went through the verbs except (a)'s
one line. (d) A near-miss the acceptance scenario caught before it shipped: the
first cone included `bundle.journal` and `bundle.payloads/` (Judged admits
them), and the journal moves on EVERY judged transaction — standing evidence
would have been unreachable and the brick silently inert. The change medium's
own ledger records the tree, it does not build it: excluded, on the `attest/`
scope rule's exact reasoning, one shelf down. The lesson generalizes: a
content key's worth is decided by what routinely moves, so every new evidence
cone owes its design one question — "what churns in here that the operator
never reads?"

Brick 5 BUILT (2026-07-16): THE SECOND MAINTAINED VIEW — the tier census updates
at the verb, and `BLESS_TIERS` is no longer owed at the edit. Tiers forced the
family's second shape: qualify was a LOCAL judgment (one file's line derives from
its own source), but a tier reads two global facts — the pub-reachability
fixpoint and the fronting relation — so the qualify recipe (retract, re-judge,
assert) does not transfer as-is. The recognition that makes it maintainable
anyway: THE COMMITTED CENSUS IS ITSELF STANDING EVIDENCE. A non-KERNEL row states
its own reachability (roots by definition, INTERIOR means unreachable, the rest
reachable), and the INTERIOR rows are exactly the frontable set — so a delta that
leaves the module tree's shape alone moves at most its own line, re-derived
TREE-FREE from the same four facts the oracle reads (`assign_tier`, extracted so
oracle and view state the rule once). A structural delta — a mount, an unmount, a
parent whose reachability a KERNEL row masks — is detected by checking every
direct child the census knows against the reachability this source implies, and
falls back to the oracle's own `derive_partition` over the tree: recompute, the
honest answer when standing evidence cannot carry the verdict, and byte-faithful
by construction. KERNEL is the third shape inside the shape: a decision, never
derived — a registered file's line carries verbatim at any verb, the kernel set
is read from the census's own KERNEL rows, and a register edit keeps its bless
ceremony (derivations lose their ceremony; decisions keep theirs). One semantics
adjustment was needed to make the census self-describing: fronting a
KERNEL-registered unreachable file no longer makes a door — the trusted floor is
a ratified privilege, not a workshop being fronted. The change moved ZERO
committed census lines across all four tiers.spec files (verified by the drift
gates), so it is pinned only by its new probe. Nine probes pin the view, headed
by the two that state the design: the local path is exercised with a manifest
naming NO REAL DIRECTORY (standing evidence carries the whole verdict — also
what makes always-recompute mutants killable), and every maintained answer is
judged against the naive-recompute oracle byte for byte, including through a
mount/unmount round trip where the mounted module's line moves without its own
text moving. Acceptance at the real tree: a judged no-op edit through the new
binary reproduced all three committed censuses byte-for-byte. Authoring note:
the whole brick went through the verbs — five `add`s, three `edit`s, zero hand
edits in code — with payloads formatted outside the tree per the standing idiom.
Censuses remaining for the treatment: reasons' member twins, instrumentation
(schemata already carries by cone, brick 4).

Reading from brick 5, recorded per the instrument: the design work consulted
`show` throughout (the verb carried it — item bodies, impl rosters, no raw
opens), but three questions still fell back to grep: "who FEEDS this parameter"
(the kernel allowlist's build.rs wiring), "who CONSUMES this function" (callers
of `maintain_qualify` across crates), and "which TEST FIXTURES does a semantics
change touch" (the kernel/fronting fixtures in rules.rs). All three are the same
missing perception: the reverse edge — uses-of an item across the workspace,
`constrains` read backwards. The census-maintenance work keeps needing it
because a maintained view's first design question is always "who else holds this
fact."

A note for the remaining conversions, recorded while the family is fresh: the
maintained view now has THREE shapes, and the choice is dictated by the view's
dependency structure. (1) The per-row local view (qualify, brick 3): a line
derives from its module alone — retract, re-judge, assert. (2) The
self-describing census (tiers, brick 5): the verdict reads global facts, but the
committed output carries them — maintain locally where standing evidence
suffices, fall back to the oracle's own derivation where it cannot say. (3) The
opaque carry (schemata, brick 4): the computation is a black box — key the
verdict by its evidence cone and carry wholesale. Reasons' member twins are
shape 1 (they ARE qualify); instrumentation looks like 1 or 2 (a register judged
against a file roster). A census that fits none of the three is news about the
census.

## Candidate: the declared agenda — a change ships its bill of materials first

Named from inside brick 5 (2026-07-16, the suit's own operator reporting). The
observation: the verbs' biggest effect on the work was not safety but
DECOMPOSITION PRESSURE. Because `edit` holds signatures and `add` places items,
the change had to exist as a named item roster — five adds, three edits — before
anything touched the tree. A file editor lets design and implementation smear
together across a diff; the verbs force the design artifact to exist first. And
that pressure is not a side effect, it is what funds the delta economy: every
incremental instrument here (evidence keys, carried verdicts, maintained views)
prices a change by its SUPPORT, which only works because the medium refuses
ill-formed deltas. Brick 1's "an edit is a two-row delta" is a fact the verbs
make true.

The gap: the bill of materials is real and nothing holds it. It lived in the
operator's head and scratchpad; the journal records the transactions only after
they land. The candidate is the dual of agenda-from-journal (PR bodies derived
from the record): JOURNAL-TOWARD-AGENDA. Declare the intended transaction set —
which items, which files, add vs edit — as a judged artifact, and the suit
tracks discharge: `owes`, but for a change instead of a tree. Two in-tree
precedents: genesis declares target locks and converges to green (this is the
same pattern one level down — a change declares its roster and converges to
done), and `owes` already derives obligation from declaration. Two named
consumers: review shifts left another rung (judge the nine-line roster before
any payload exists — cheaper than judging 584 lines after), and any multi-agent
split where one mind plans and another discharges.

The honest counterweight, recorded with the candidate: decomposition pressure
taxes exploration. Brick 5 decomposed cleanly because `show` made recon cheap
and the shape was derivable; a change whose items are unknown until tried would
fight an agenda. What the operator actually did — draft freely in a scratchpad,
offer finished payloads whole — reproduces the tier system in the change
process itself: an INTERIOR drafting table, a strict grammar at the boundary.
So the agenda must stay a declaration of intent, revisable by the same judged
means, never a cage; the payload store is already half the drafting table made
first-class.

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
- **Candidate: the ceremony layer is the suit's next frontier** (the operating notes
  from the first merged epic). Inside the tree the agent works suited — verbs, verdicts,
  attestations; at the GitHub layer (PR bodies, merges, releases, CI-watching) it is
  back to buttons and logs, the fallback encyclopedia of the CEREMONY. The evidence:
  PR bodies hand-narrated three times when the journal holds their content (the
  agenda-from-journal candidate, now with a consumer), the release semantics still in
  shell (its two failures were both shell-class), and watch-state dying with every
  container restart while the pinned binary sails through. Also worth naming from the
  same notes: OPERATED failures arrive pre-narrated — the perimeter refused a merge
  method by quoting its own lock; the diagnosis step keeps collapsing into reading —
  and the unkeyed-verdict failure class appeared at a fourth altitude (semver) once the
  first three gave it a name. Fresh evidence from the bricks 1–5 landing (2026-07-16):
  the PR body was hand-narrated a FOURTH time; the whole landing was hand-sequenced
  button ceremony (push rightly refused by the perimeter, then branch, `gh pr create`,
  a watched 6½-minute required check, manual `--squash` merge — repository auto-merge
  is DISABLED, a settings fact worth a deliberate decision either way); and the
  session's only honest wait was exactly here. The sharper framing the day's
  conversation produced: the required check is safety implemented as a GATE IN THE
  PATH, the one architectural pattern everything inside the tree exists to dissolve —
  structure in the medium judges at machine speed off the critical path, while a gate
  queues all throughput behind an attention scheduler. The ceremony layer is not
  merely un-suited; it is the last bottleneck-shaped safety in the system.
- **Candidate: the version bump becomes a derived demand** (named by the release the
  big merge broke twice). The mint's second failure was the honest one: the release
  loop is idempotent by version, `boundary-enforce` grew its API without a bump, so
  the stale 0.1.0 was "already published — skipping" and the root crate's tarball
  verify resolved the OLD one from the live index. The version integer is the last
  hand-asserted compatibility claim in the system — and the machinery to derive the
  demand already exists: each member's qualify census is committed, so "the public
  surface moved since the published version" is a computable fact. The gate shape: a
  publishable crate whose census diff against its published version is non-empty must
  carry a manifest version the index does not know — semver honesty as a lock, not a
  memory. (The SIGPIPE that broke the first mint is fixed in release.sh itself; the
  deeper item — the release ceremony as typed Rust, the last shell carrying
  semantics — stands.)
- **Candidate: the toolchain pin binds CI but not the local cargo** (named by the
  1.97.1 bump, 2026-07-17). `discover::gates::TOOLCHAIN` renders into ci.yml, so the
  pin governs every runner — but a local `cargo test` consults rustup's default, and
  the day's evidence is what that gap costs: dependabot proposed "1.100.0" against
  the rendered YAML (an action-repo tag, not a Rust that exists — the artifact patch
  was refused by the drift gate, as designed, but nothing refused the premise), and
  the bump's regenerated compile-fail `.stderr` files left the un-updated local
  machine failing bare `cargo test` until a by-hand `rustup default`. The fix is one
  more frozen artifact: `freeze_gates` renders `rust-toolchain.toml` from the same
  constant, rustup obeys it in every checkout, and the pin becomes structure in the
  medium instead of a fact about CI. The hand-work line this entry retires: the
  `rustup default` run by hand this session, a derivation (the committed pin) plus a
  signature (nobody's — it should need none).
- **Candidate: the journal as the environment — change locality as the carving's
  empirical judge** (named 2026-07-17, reading an outside harness that saturated an
  interactive benchmark by holding state representation and transition rules in one
  editable program). What that work has and we lack is a fast judge: its next frame
  arrives in milliseconds and cleanly indicts the last revision, so representation
  repairs need no signature — the environment punishes the bad ones. Our next frame
  is the next hundred edits: a wrong module carving is punished by demands that
  should be local deltas smearing across items, a verdict that arrives in days and
  confounded with task difficulty. The ratification signature on representation
  choices is therefore a LATENCY PROXY, not a metaphysical necessity — and the item
  relation is quietly accumulating the dataset that could retire part of it, because
  verbs are typed deltas, so delta size and locality per demand is computable over
  journal history where text churn never was. The promotion path: the cohesion
  report, today a static suggestion from law-connectivity, gains an empirical
  column — this carving's measured cost against the actual change stream — and
  "this module is secretly several" upgrades from taste to evidence. The standing
  question, pointed at its own ratification step: some of what is signed today is a
  derivation whose dataset has not finished arriving. What no amount of journal
  converts: the loss function — which demands ought to exist stays prose.

## The toolchain pin becomes structure in the medium: `rust-toolchain.toml` (BUILT)

Built 2026-07-17, closing the candidate named the same day (#59). The pin is now the
THIRD pipeline lock: `GateRegistry::render_toolchain` renders `rust-toolchain.toml`
from the same `TOOLCHAIN` constant that pins every CI runner (components mirror the
CI install — rustfmt, clippy), `toolchain_lock` rides `freeze_gates` beside the
registry and the workflow, and the freshness probe holds all three byte for byte —
a hand edit to the pin fails inside the very `cargo test` the pin governs. rustup
obeys the committed file in every checkout, so "the toolchain CI tests against" and
"the toolchain a checkout builds with" are one claim, not a memory and a hope.

Acceptance was this container itself: rustup's default here is 1.94.1 — the exact
gap the candidate documented, compile-fail suites red under bare `cargo test` — and
after the mint the same bare command went green with no `rustup default` run by
hand. The hand-work line the candidate flagged is retired by the artifact. The
support census keeps the honesty: `Support::RustSurface` now admits
`rust-toolchain.toml` (the rustfmt that runs rides the pin), so a toolchain bump
can never hold a stale fmt green in the verdict store.

Readings from the build: the suit refused the first attempt — `bundle edit` holds
an impl's method-signature set ("an interface change is not an edit"), so the two
new methods arrived as a sibling impl through `add`, the same grain the item
relation grew by. Residual, named: the consumer `Pipeline::locks_in` (what genesis
emits) still renders only the registry and workflow locks — a generated crate's
checkout does not yet carry its declared pin; the consumer twin of this brick is
one more lock in that list when a genesis crate first trips the same gap.

Judgment addendum, same session: the minted site was judged killed by hand
(`PROBE_MUTANT` × the gates probes — the freshness lock and the pin probe both go
red under the flip). The judging surfaced a reading worth keeping: a site's id is
the WHOLE census line, arrow and flip value included
(`...render_toolchain:deaf -> String::new()`), and the truncated form arms
nothing — silently, a green that means "you flipped nothing". Nothing ratifies
that answer today; it took a macro-interior read (`mutate_body`'s
`format!("{label}:deaf -> {desc}")`) to learn. Two candidate bricks it names:
`schemata judge` (or a new verb) could refuse a `PROBE_MUTANT` that matches no
census site — the deaf-but-armed state is exactly the unkeyed-verdict class,
one altitude down; and the full local `verify` after a support change re-keys
everything (~950 moved sites, serial cargo-test-per-site without nextest) is a
container-hours run — the incremental tier's economics depend on the warm store
the session hook (PR #56) provisions.

## The end state, adopted: correct by construction (docs/endstate.md)

Adopted 2026-07-17: the destination now has its own document. The five asymptotes
— structure as the only memory, the text file gone archaic, no unkeyed green,
evidence become construction, the roadmap deriving itself — with the one
permanent remainder named (the loss function stays human). The honest frame is
reframed by it, not repealed: "green is evidence" is a fact about the CURRENT
instrument's bounded grids and sampled batteries, and the end state closes that
gap like any other rather than enshrining it. Custody, decided deliberately: the
document is reachable through this roadmap, NOT through the every-turn context —
CLAUDE.md carries invariants that bind every action, and a destination is a
hypothesis that new evidence may re-aim; injecting it per-turn would harden it
into a remembered aim outliving its ratification, the exact class the method
kills. Aims live where deliberate reads happen; rules live where every turn does.

Field report, same day (the reading that sharpened the adoption): an outside
system generated roughly eight thousand lines of systems code carrying roughly
ten thousand lines of machine-checked proof — hundreds of definitions and
theorems, end-to-end correctness properties, authored autonomously in about three
minutes. What it taught: the route from sampled refutation to discharged theorem
is not speculative, it is shipping — proof-carrying machine authorship at
interactive speed is a demonstrated capability, so the gap between our batteries
(evidence) and proofs (construction) is an engineering distance, not a research
bet. What it did NOT show, which is exactly this repository's ground: the medium
that keeps a thousand such artifacts honest across time — judged transactions,
drift gates, content-keyed verdicts, the census of what is and is not yet proven.
The two halves are complementary: their demonstration is the discharge step;
our machinery is what makes discharge a STANDING fact about a living tree
instead of a one-shot artifact. The HoTT angle, named for the catalog: a shape's
law is already a typed, slotted equation — a proof obligation in everything but
name; the spec as the type, implementations as inhabitants, equivalence between
implementations as a path. The promotion path when this brick is picked up: one
law of one settled theory, discharged as a proof and its ratified mutation
survivors retired by it — the first green that is not a sample.

Field report amendment (2026-07-18, on reaching the primary source): the earlier
reading undersold it. The outside system is not "generate then check" — the model
is confined to ONE hop (natural language → a formal DSL spec) and everything
downstream is a deterministic compiler emitting the implementation AND its proofs
from the same source, claimed verified down to the floating-point axioms. That is
the second rule at proof altitude: one declaration, many derived artifacts,
nothing restated — their DSL is to their C-plus-proofs what our gate registry is
to ci.yml and the toolchain pin. The near-1:1 proof-to-code ratio that looked
suspicious is the signature of proofs RENDERED rather than searched for. The
sharpened complement: their residual trust sits in the compiler and in the one
stochastic hop — does the spec say what was meant? — and spec-judgment is exactly
this repository's half (derive the spec by running the thing, mutation as
tightness, the ratified diff as the intent check). Spec → artifact with proofs;
artifact → spec with evidence and a signature. Each direction is the other's
missing half, and the promotion path already named (one settled law, discharged)
is where they would meet.

## Candidate sharpened: eliminating the gate in the path — the countersign chases the tip

Sharpened 2026-07-18, extending the ceremony-layer candidate with the mechanism.
The required check re-derives, in minutes and under attention, facts the authoring
container already established; the elimination is three moves plus a floor. (1)
Verdicts become PORTABLE, COUNTERSIGNED facts: the store already keys green by
(gate, support, content) — what CI distrusts is the claim, not the math, so give
verdict records signatures from a trusted runner and the merge check collapses to
a ledger audit. (2) Judgment moves to CHANGE TIME: a branch of judged transactions
is green by induction; the missing piece is support grain — per-file admits
re-keyed the whole store when the support itself moved (measured in the pin PR),
where per-item/per-site keys (the coverage map already holds site→test edges)
would re-judge only what moved. (3) The countersign CHASES THE TIP instead of
gating it — the mutants-green tag already runs this pattern weekly, off the
critical path; generalized to every change, merge lands when the ledger is
complete, a red countersign is a detection, and the substrate's history laws name
the compensation. The floor that remains: the trust root (whose keys countersign
is a ratified decision), the Effectful world gates (event-time by nature), and
the async ratification signatures — none of which belong in the path. Beneath all
three, the endstate's constructed-change program shrinks the owed set itself:
each invariant that migrates into the verb algebra stops being a gate that
re-runs and becomes a refusal that never needed to. Near-term, boring, real: CI
carrying its own verdict store across runs, and the auto-merge settings decision
already flagged above.
