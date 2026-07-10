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
verbs cannot express — the second rule's field-report arm, proven on day one), test
authoring, and cross-file moves. Refusals should name their fixing verb.

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
  COMPOSITION stanza (`f(g(x)) = h(x)` — three unary slots) now has its consumer named:
  JOURNAL COMPACTION — squash as algebra. Which verb sequences reduce to a single verb is
  exactly what a `squash` operation must know, and the engine can discover those
  identities the way it found the join rules. Queued, legitimate, waiting on the squash
  brick to pull it in.
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
