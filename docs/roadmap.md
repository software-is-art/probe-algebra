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

## From the field (the first production adoption)

A production Rust CLI adopted `spec-lock` as a pinned git dependency — four agent sessions in
one day, five substrates (a command surface, checklist data, coverage manifests, a
reconciliation policy, an accepted-findings baseline), every re-bless in the same commit as
its cause, the discipline transferred from the README alone. Its field notes seeded two
candidate bricks:

4. **Negative-fixture batteries — mutation testing for PIPELINES.** The witness-inequation
   idea (`∃`: prove the gate ACTS) kept being needed at the process level: the adoption hit
   three separate self-attestation failures in one system — a coverage manifest asserted by
   the session that produced it, a "pass" stamp byte-identical whether the work happened or
   not, a data tree that would have passed reconciliation with zero items checked. The
   generalization: a standing battery of KNOWN-BAD fixtures proving each gate can still fire
   — including gates over pure prose. Nobody does this; the field notes show why someone
   should. Plausibly extractable in the spec-lock spirit: small, substrate-free, adoptable in
   minutes. This is the witness shapes' lesson ("a spec that says nothing kills nothing" —
   the mute-theory fixture) applied to processes instead of algebras.

5. **The exception-register pattern, first-class.** The adoption's highest-leverage use was
   freezing an accepted-findings baseline (documented in
   [ci-discipline.md](ci-discipline.md)). The convention (deterministic keyed serialization →
   byte equality IS set equality) covers it with spec-lock as-is; if field use keeps wanting
   set-diff rendering ("2 new findings, 1 resolved" instead of a byte diff), a tiny keyed
   helper alongside `Lock` is the shape — resist anything larger.

## Standing follow-ups

- **Tag `v0.1.0`** — consumers currently pin by bare rev; a tag makes downstream manifests
  read as versioned intent. No crates.io needed for this half-step.
- **Publish** when ready: `docs/publishing.md` has the dependency-ordered sequence;
  all four names (`boundary-spec`, `boundary-spec-macros`, `spec-lock`,
  `boundary-enforce`) were verified unclaimed on crates.io on 2026-07-02.
- **Morphism downstream**: the fixture exercises Construction/Branch/Guarded;
  the fourth edge shape is honestly unexercised downstream.
- **MSRV**: unpinned; verify and add `rust-version` after first publish.
