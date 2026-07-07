# The ideal authoring experience — a friction inventory with fresh eyes

Written after converging `genesis-demo/` from blank slate to green as the working agent.
The workflow's defining strength is that every red state carries its own worklist
(`grep MEANING:`, `Distance::render()`, the drift gate's stale names). The frictions below
are the places where that stops being true — where the author supplies transcription,
ceremony, or judgement the declaration already contains. Each pillar is a startable brick;
the last is a research program.

## Pillar 1 — one declaration, two lifecycle stages

Today there are two `system!` dialects that rhyme: the one genesis PARSES from a cfg'd-out
block (values, modules with ops and expects, seams) and the one that COMPILES
(`discover::system` — marker, theory types, seam checks). Genesis emits the second from the
first, and the byte-pins keep them in step — but they are still two artifacts that can only
drift into a test failure, not a compile error.

The ideal: **the declaration file is the same text at both stages.** Genesis reads it on day
zero to derive the tree; the emitted `src/system.rs` carries the ORIGINAL declaration
tokens, and the compiled `system!` macro learns the full grammar — cross-referencing the
theories, values, and seams that now exist in code. A module added to the code but not the
declaration (or vice versa) then fails to COMPILE, with the declaration as the named source
of truth. The registry story ("the graph IS the registry") becomes literal: one block of
tokens is the application's spec at every point in its life.

*Status: BUILT.* The `system!` macro grew a FULL-GRAMMAR arm (`Marker:` followed by the
genesis declaration verbatim): module names resolve to their genesis-conventional theories
via `paste` ident derivation, every declared operator signature compiles into a
`const _: () = { let _: fn(…) -> … = path; }` witness, `transport on V;` discharges by
construction, `transform on V via h;` checks in the genesis-named spanning theory, and ops
modules PUB-re-export their sorts so the declaration can name every type it mentions.
Genesis now emits `src/system.rs` as the ORIGINAL declaration text, spliced verbatim (span
offsets preserve the author's formatting) — `relay-demo/` was regenerated in this form and
converged identically (locks byte-for-byte unchanged), which is the compile-level proof.
Residual v1 honesty: `values` rules and `expects` clauses are carried but not re-checked by
the macro (the rules already generated their artifacts; expectations stay gated by
`Distance` through the `#[algebra]` attribute), and raw-representation drift is caught at
`get()`'s call sites rather than by a declaration witness.

## Pillar 2 — executable validity rules (shrink the holes to pure meaning)

Converging the demo, the holes split cleanly in two:

- **genuine meaning** — the operator interiors (`grant` pools, `renew` replaces, `charge`
  costs). Nothing can generate these; they are the crate's reason to exist.
- **transcription of the rule already declared** — `Credits = i64 where "0..=20"` carries
  the rule as PROSE, and the author then re-derives, by hand: the predicate
  (`(0..=20).contains(&raw)`), the re-entry discipline (`clamp`), and the `Shaped` grid
  stepping toward the rule's edges. Three artifacts, all mechanically implied by eleven
  characters the declaration already contains. Carrying meaning as prose and re-deriving it
  by hand is exactly the transcription step this project exists to delete.

The ideal: **a `where` clause that is tokens, not a string, generates everything it
implies.** `Credits = i64 where 0..=20 saturating;` derives the predicate, the saturating
`mint`, and the edge-seeking `Shaped` impl; `where 0..=20;` (no re-entry policy named)
derives the predicate and grid but leaves `mint` a hole — choosing clamp/reject/panic IS
meaning; the range is not. Prose rules (`where "non-empty"`) stay full holes, unchanged.
The vocabulary can grow honestly (length bounds for strings/collections, non-emptiness,
set membership); anything outside it is prose, and prose is a hole.

Alongside it: **raw types are any `syn::Type`**, not plain paths — `Vec<u8>`,
`Option<Instant>`, tuples. The v1 restriction was parser thrift, not principle. (Fully
generic VALUE OBJECTS — `Meter<T>` — stay deferred: the engine's theory types are
monomorphic by design, and no domain core has demanded them yet.)

## Pillar 3 — the full catalog is declarable

The engine discovers sixteen shapes; the declaration grammar admits seven (the homogeneous
binary rows). The missing nine include exactly the ones that make multi-sorted domains and
seams interesting: `round_trip(at, since)`, `monoid_action(tick, plus)`,
`homomorphism(esc, "++", "++")`, `involution`. The blocker is that each shape's
applicability GATE is prose in `ShapeInfo` — a human-readable sentence, not data genesis
can validate a declaration against.

The ideal: **encode each shape's gate as data** (arity and sort-pattern per parameter slot:
homogeneous-binary, unary-conversion, constant-of-sort, …). Then: declarations validate
against the same gates discovery uses; the `expects` vocabulary becomes the whole ratified
catalog automatically (the lockstep census already ties the two lists); and a transform
seam's homomorphism becomes DECLARABLE — which is the missing piece for genesis to emit
spanning-theory skeletons and compile transform seams end to end (the standing roadmap
brick). One data structure unlocks all three.

*Status: BUILT.* `ShapeGate`/`Slot` sit inside every `ShapeInfo` (`gate_slots`), one generic
checker (`ShapeGate::admit`) serves the engine census and genesis validation, and the
genesis `expects` vocabulary is the whole catalog (minus `irreflexive`, whose `false`
witness no identifier can spell — refused with the alternative named). The declared-law
renderer is held to discovery by a dynamic sync pin: fixtures fire every newly-declarable
shape, every discovered law is declared back, and the emitted target lock must equal the
freeze's live render byte for byte. The remaining payoff — genesis emitting spanning-theory
skeletons so transform seams compile end to end — is now unblocked.

## Pillar 4 (research) — effects as theories: the world lock

The library kept I/O at the edges because effects resist the mental model. The kvstore
already found the crack in that wall: TIME became a value (`Tick` is an explicit edge), and
suddenly a stateful domain was just another discovered algebra. The research program is to
walk the same move outward, in this project's own vocabulary — *derive the spec by running
the thing, freeze it, gate the drift*:

1. **Effects are values.** An external dependency is modeled by a `Command` value object
   (an enum: `Put(k, v)`, `Get(k)`, `Retry`, …) and a TRACE — a sequence of commands —
   which is itself a value with a concatenation algebra.
2. **The mental model is an interpreter, and the interpreter is the observation.** A pure
   `WorldModel: fn(&Trace) -> State` plays the role `observe` plays everywhere else. A
   trace theory's discovered laws are then PROTOCOL laws, found by the existing engine with
   no new machinery: put-then-get returns the put; writes to distinct keys commute;
   retry-after-timeout is idempotent; delete annihilates.
3. **The probe battery derives itself.** `#[derive(Shaped)]` on `Command` gives the
   canonical battery of traces via `shadow_grid` — structure-first, so every command
   constructor appears before value tuning spends the cap. The integration tests for an
   external system are DERIVED from the model's type, the same way probe generators
   already derive from value objects.
4. **The world lock.** You cannot ratify the world, but you can ratify your assumptions
   about it: replay the canonical battery against the REAL dependency (once, in
   integration CI, behind a bless flag — the one deliberately-Effectful gate), record the
   observations, freeze them into `spec/<dep>.world.spec`. From then on two seams are
   drift-gated: model-vs-recorded-world (does our interpreter still predict what reality
   answered?) and core-vs-model (do our domain laws hold over the modeled effects?). When
   a vendor changes behaviour, the replay drifts and the diff NAMES the changed
   assumption — the freeze discipline pointed outward for the first time.
5. **The shell is thin by construction.** With decisions pushed into pure
   `fn decide(state, input) -> Command` operators (probeable, law-bearing), the residual
   effectful adapter is a command executor — the capability audit already knows how to
   hold an edge to "Effectful and nothing else".

What makes this a research effort rather than a feature: nobody has committed a
*drift-gated conformance lock derived from a type-derived trace battery* as the standard
artifact for an external dependency. Every ingredient exists in this repo today — the
engine, `Shaped`, `spec-lock`, the capability lattice, the kvstore precedent. The brick is
the composition.

*Status: FIRST COMPOSITION BUILT* (`discover::world`). `Command`/`Trace` are the effect
values; the pure `StoreModel` interpreter is the observation, and the UNCHANGED engine
discovers the protocol laws — `idempotent(++)` (replay/retry safety), `bias_later(++)`
(last-write-wins), associativity, and the harmless empty batch — declared, met exactly, and
frozen in `spec/store-protocol.spec` like any module's algebra (the theory sits in the
`BoundarySpec` registry). The new artifact is committed too: `spec/store-model.world.spec`
records the model's predicted observation for every trace in the `Shaped`-derived battery,
drift-gated; the conformance gate replays the same battery against a deliberately
INDEPENDENT event-sourced `FakeRemoteStore` (the stand-in vendor) with zero disagreements,
and a first-write-wins vendor (the classic "insert where the contract says upsert" bug) is
refused with the exact diverging trace and both observations NAMED. Remaining research:
point the replay at a genuinely remote dependency behind a bless flag in integration CI,
and grow the trace vocabulary (reads, timeouts, retries as commands).

## Order of attack

Pillar 2 first (it deletes the most per-crate friction and is self-contained), then
pillar 3 (one data structure, three payoffs), then pillar 1 (grammar unification rides on
3's richer compiled parser), with pillar 4 running as the research track alongside — its
first concrete step is a second stateful fixture: a fake-but-realistic external store
modeled, probed, locked, and drift-gated end to end inside this workspace.

## The workbench protocol (authoring with the autorouter)

Writing behaviour and deciding shape are two problems, and doing them simultaneously
degrades both — for agents especially. The placer (`discover::shape`) and its editor
half (`Architect::place`) dissolve the interleaving. The protocol:

1. **Declare a workbench** when new work starts: one bundle `theory!`, no structure
   decisions. Append operators to it as the behaviour comes.
2. **Let the placer watch.** `Architect::place::<Workbench>(file, out_dir)` is cheap
   enough for every keystroke (signatures only, no discovery). While new operators
   share nets with existing ones, it is silent.
3. **Extract when it fires.** A net-disjoint component is an indisputable, lossless,
   seamless split — the finding's action is `isPreferred` and writes the scaffolded
   module files; move the operator evals in and name the module. The workbench shrinks
   back to one component and writing continues.
4. **Extract before merge.** A workbench never ships with two components: assert
   `Placement::of::<Workbench>().is_settled()` in the crate's tests while the
   workbench lives, and retire the bundle (fold it into its final module) when the
   work is done. The shape lock holds the boundary from then on.

No standing workbench file exists in this repo between features — a placeholder theory
would be dead weight for the mutation gates to chew on. The workbench is declared when
work starts and retired when it ends; the fixtures in `architect.rs` pin the machinery
meanwhile. Dogfood commitment: the next algebra brick in this repository gets written
through this protocol, and what it teaches goes back into this section.
