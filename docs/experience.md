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

## Order of attack

Pillar 2 first (it deletes the most per-crate friction and is self-contained), then
pillar 3 (one data structure, three payoffs), then pillar 1 (grammar unification rides on
3's richer compiled parser), with pillar 4 running as the research track alongside — its
first concrete step is a second stateful fixture: a fake-but-realistic external store
modeled, probed, locked, and drift-gated end to end inside this workspace.
