# probe-algebra

**Derive the spec by running the thing. Freeze it. Gate the drift.**

This is a Rust workspace (root crate `boundary-spec`) built around one discipline: you author
*meaning* — value objects, validity rules, operator bodies — and everything checkable about it
is **derived, committed, and drift-gated byte for byte**. The behaviour spec, the module seam
graph, the CI pipeline, the mutation-testing verdicts: none of them are written by hand, all of
them are locks, and a change to any of them is a diff you consciously ratify in review.

The bet, stated falsifiably: *a module boundary can be specified precisely enough that the
tests validating it write themselves.* The yardstick is mutation — plant bugs in the interior
and count what survives. The interior of the lead example (a full expression-language
interpreter) carries **zero tests of its own**, and every viable mutant dies anyway.

```
cargo run --example gate    # every gate CI runs, locally, from the same declaration
cargo test --workspace      # all suites + every drift gate + in-process mutation
```

---

## The loop, in sixty seconds

A domain is its value objects and its operator functions. Everything else derives:

```rust,ignore
#[algebra(Lattice, "tri lattice")]
pub mod lattice {
    #[derive(Shaped, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
    pub enum Tri { Lo, Mid, Hi }

    pub fn meet(a: Tri, b: Tri) -> Tri { a.min(b) }
    pub fn join(a: Tri, b: Tri) -> Tri { a.max(b) }
}
```

The **discovery engine** runs those operators over a grid grown from the type's own structure
(`#[derive(Shaped)]` — no generators written), instantiates every shape in the ratified law
catalog, and keeps what runs true. For `Tri` that is the whole distributive-lattice spec — ten
laws, hands-free. Each registry theory's result freezes into a committed file under
[`spec/`](spec) — plain language and symbols, e.g. from [`spec/ttl-store.spec`](spec/ttl-store.spec):

```
- With Merge, the later operand wins where the two disagree — re-applying an
  earlier one cannot overwrite it.
      ((s <+ t) <+ s) = (t <+ s)
- Tick actually acts — some parameter moves some value.
      tick(s, p) ≠ s
```

From then on `cargo test` re-derives the live spec and fails if it differs from the committed
text. **The committed diff IS the ratification**: an unintended behaviour change is a red gate
that names the exact law that appeared or vanished, and review happens in plain language, not
by simulating the implementation in your head.

The loop also runs top-down. Declare the algebra you *intend* before the code earns it:

```rust,ignore
expects {
    commutative(join);
    identity(join, bottom);
    nontrivial(tick);          // an INEQUATION: the clock must actually move something
}
```

and `Distance::of::<T>()` reports exactly what is unmet — a red report an agent can work from
("MISSING: identity(join, bottom)"), going green law by law as the implementation earns the
declaration. Expectations get you TO the lock; the lock keeps you there.

---

## One discipline, four altitudes

The same move — declare, derive, freeze, gate — is applied at every level of a program:

| level | declaration | derived artifact | lock |
|---|---|---|---|
| **module** | value objects + operators (+ `expects`) | the discovered law spec | `spec/<theory>.spec` |
| **system** | `system!` — modules + seams | the checked seam graph | `spec/<system>.system.spec` |
| **world** | effects as command values + a pure model | recorded conduct over a derived battery | `spec/<observer>.world.spec` |
| **pipeline** | the gate registry (`discover::gates`) | `.github/workflows/ci.yml` itself | `spec/gates.spec` + the workflow file |

**System level**: a `system!` declaration compiles into a graph whose nodes are module algebras
and whose edges are seams — a *transport* seam (the algebra must survive unchanged; checked by
law-agreement, or discharged at compile time by a `fn(V) -> V` witness) or a *transform* seam (a
named conversion must be a homomorphism in a spanning theory — run, never assumed). Ratification
is hierarchical: interior changes touch no lock, module law changes touch one module lock, only
a re-drawn seam touches the system lock.

**World level**: you cannot ratify the world, but you can ratify your assumptions about it.
Effects are values (`Command`, `Trace`), the mental model is a pure interpreter, and the
protocol laws discovery finds are the operational guarantees — `idempotent(++)` literally IS
retry safety, `bias_later(++)` IS last-write-wins. The recorded conduct freezes into a world
lock; replaying the same battery against the real dependency names the exact trace where the
world left your assumptions.

**Pipeline level**: `ci.yml` is not configuration — it is a **rendered lock** over a declared
gate registry (command, cadence, capability, promise). A hand edit to the YAML fails `cargo
test` inside the very workflow the YAML executes. CI keeps only what cannot shift left:
countersigning, effects, and the economics of the expensive sweeps — all three visible as
registry data.

---

## The law catalog is the engine, not a description of it

Every law any spec can state is an instance of one of the **18 shapes** in
`ShapeCatalog::inventory()` — and that inventory is not documentation of the engine, it *is*
the engine: discovery is a generic interpreter over the catalog's data. A shape carries its
applicability gate (as checkable slot data), its canonical terms, its prose template, and its
polarity; adding a law shape to the language is **adding one stanza of data**, ratified through
the regenerated `spec/shapes.spec` diff and executable the moment it lands.

The catalog speaks two polarities:

- **equations** (`∀: lhs = rhs`) — commutativity, associativity, identity, distributivity,
  the write-bias laws, monoid actions, homomorphisms, round-trips, …
- **witness inequations** (`∃: lhs ≠ rhs`) — *action nontriviality* ("the clock actually
  moves something") and *non-constancy* ("the relation is true somewhere"). These exist
  because equations provably cannot say "this thing actually does something": a TTL store
  whose `tick` never advances satisfies every action equation vacuously. Now it contradicts
  the spec.

The bias laws and the witness shapes were both *found*, not designed — see the mutation story
below. That is the intended dynamic: hostile domains find blind spots, blind spots become
catalog stanzas, and one stanza fixes every theory at once.

---

## Mutation judges everything — at two speeds

Mutation testing is the method's own meta-test: probes claim to catch bugs; mutation checks
the probes. It runs at two levels with very different economics:

**Source level** (`cargo mutants`, ~15s per mutant — a mutant is a build). The economics are
declared data in the gate registry: PRs mutate only their changed lines; the default branch
mutates only the diff since the last fully-certified tree (the `mutants-green` tag, advanced on
green — CI's countersignature made durable); the weekly clock re-certifies everything from
scratch, sharded across parallel runners. Timeouts are detections, not survivors; the few
genuine equivalents are classified findings, ratified by name in `.cargo/mutants.toml`.

**Algebra level** (`discover::mutation`, milliseconds per mutant — a mutant is a *value*). For
anything that is a theory, the harness perturbs the operator table in process — one operator
evaluates as another, a binary returns an argument unchanged, an operator goes undefined — and
judges each mutant by re-running discovery: **killed iff the law set changes**, the freshness
gate's own semantics applied to a hypothetical implementation. The whole verdict runs inside
every `cargo test`, so this half of mutation testing is fully shifted left. Verdicts freeze
into `spec/<theory>.mutation.spec`; a survivor is not a missing test but a **named degree of
freedom the spec cannot see**.

That last sentence is not hypothetical — it is how the catalog grows. The harness's first run
found four survivors, all one lesson (equational laws cannot state inequations: the trivial
action, the never-true relation, the operator no law names). The witness shapes were added as
catalog data, discovery found the closing inequations *itself* on the next freeze, and all four
survivors died — zero per-theory work. The flagship demo confirmed it unprompted: its `billing`
module's spec had carried the line *"operators in no law: charge"*, and the witness pass
replaced that recorded silence with `(x charge x) ≠ x`.

---

## From a blank slate: genesis

`genesis` runs the loop from nothing. One declaration — value objects with executable validity
rules, operators, seams, declared expectations — generates a whole workspace member: theory
impls, value-object constructors, the compiled `system!` graph, distance gates, seam verdict
tests, and **target locks** (the spec files discovery *should* reproduce, byte for byte, once
the meaning holes are filled). The authoring surface that remains is exactly the irreducible
part: `MEANING:` holes for operator interiors.

Two CI-tested workspace members are the proof, each converged from blank slate to green at both
levels with only their meaning holes filled: [`genesis-demo/`](genesis-demo) (two modules, a
by-construction transport seam) and [`relay-demo/`](relay-demo) (a transform seam through a
named conversion, checked as a homomorphism in its spanning theory — the system lock was fresh
on the *first* freeze). Their `src/system.rs` is the verbatim declaration: one artifact, two
lifecycle stages.

---

## Structure is reported, never imposed

The engine's law graph doubles as a modularity instrument. All of these are **suggestions a
reviewer ratifies, never constraints**:

- **cohesion** (`cargo run --example cohesion`) — a module is smelly not when its algebra is
  large but when it is *decomposable*: the operator-interaction graph's components are the
  latent modules, and each proposed seam is classified transport vs transform.
- **layering** (`… --example layering`) — a connected algebra that holds together only through
  one operator (a graph articulation point) wants to layer, not split.
- **composition** (`… --example composition`) — homomorphisms compose, so a transform pipeline
  is checked end to end: operators change at every stage, the law survives the chain.
- **modularize** (`… --example modularize`) — pointed at an unstructured bag of functions, it
  proposes modules ranked by how many laws bind them, and refuses to dress up the misfits.
- **system distance** (`SystemDistance::of`) — the declared module graph vs the latent one, in
  the distance voice; this repo's own registry names three declared modules that are secretly
  several, byte-pinned as a deliberate keep-whole decision.
- **the architect** (`… --example architect`) — the cohesion signal as an LSP diagnostic plus a
  `refactor.extract` code action that writes the scaffolded split. The tool's own report type
  is a discovered join-semilattice; the abstraction validates the tool that wields it.

---

## The compile-time floor

Beneath discovery sits the boundary grammar, where claims that can be statically false are
build errors: evaluation is uncallable without the type-checker's witness (proof-carrying,
name-branded per value); effect ceilings and cost budgets reject at compile time; every source
file declares its tier (`KERNEL` / `BOUNDARY` / `INTERIOR` / `ALGEBRA`) and `build.rs` dispatches
the matching discipline; every concrete edge must carry a probe or the build fails; public
functions must attach to a typestate or be operator-shaped (the rats-nest rule); and
boundary-hood itself is *computed* — a census (`spec/qualify.spec`, drift-gated) reports which
modules are operator-shaped regardless of what they are named. The full model lives in
[docs/concepts.md](docs/concepts.md) and [docs/how-it-works.md](docs/how-it-works.md).

---

## What it feels like to work here

The drift workflow is the whole experience. You change something; `cargo test` goes red naming
the exact stale artifact and its regeneration command; you regenerate; the diff that lands in
your commit is the ratification a reviewer reads. **Never hand-edit a generated artifact** — a
missing lock is stale, never fresh.

| artifact | regenerate with |
|---|---|
| `spec/<theory>.spec`, `.system.spec`, `.world.spec`, `.mutation.spec` | `cargo run --example freeze_spec` |
| `spec/shapes.spec` (the law catalog) | `cargo run --example freeze_shapes` |
| `spec/gates.spec` + `.github/workflows/ci.yml` | `cargo run --example freeze_gates` |
| `spec/qualify.spec` (public-surface census) | `BLESS_QUALIFY=1 cargo build` |

`cargo run --example gate` runs every every-change gate from the same declaration CI executes —
green locally and green in CI are the same claim. `cargo run --example discovered_spec` prints
the live specs; each analysis above has its own example binary.

---

## The honest frame

Discovery **refutes, it never proves**: grids are bounded, term enumeration is depth-bounded,
batteries are samples. A discovered law is one the bounded grid could not refute; a witness law
refutes triviality without proving richness; a mutation survivor is "indistinguishable on this
grid", never an equivalence proof. Reports (distance, cohesion, layering) *suggest*; locks
*gate*. What remains irreducibly human (or agent): the validity rules, the operator meanings,
ratifying each diff, rejection tests, and the trust root — `rustc`, `cargo-mutants`, `typewit`.

## Reading further

- [docs/discovery.md](docs/discovery.md) — the discovery half, precisely: theories, the grid,
  the catalog's fields and driver semantics, expectations, every lock kind, seams and
  `system!`, genesis, the world lock, the gate registry, algebra mutation.
- [docs/concepts.md](docs/concepts.md) — the compile-time half: the graded category, the four
  edge shapes, the gradings and their self-proofs, the probe taxonomy.
- [docs/how-it-works.md](docs/how-it-works.md) — the edge grammar end to end: what you write,
  what compile time and autotest time each validate.
- [docs/ci-discipline.md](docs/ci-discipline.md) — the extractable pattern (deterministic spec
  → frozen file → drift gate → diff-scoped mutation) and the standalone
  [`spec-lock`](spec-lock) crate that carries it to any project.
- [docs/roadmap.md](docs/roadmap.md) — the brick chronicle: what's built, what's next, and the
  findings that redirected the method.
- [docs/experience.md](docs/experience.md) — the authoring-experience program and its
  residuals.
- [CLAUDE.md](CLAUDE.md) — the working discipline, distilled for agent sessions, with a
  standing invitation: if you see an idea the method is missing, say so unprompted.

## Using it

```toml
[dependencies]
boundary-spec = { git = "https://github.com/software-is-art/probe-algebra" }
```

Model the domain as value objects and operators (`#[algebra]` for the hands-free path,
`theory!` where an observation or grid is a deliberate choice), declare what you intend with
`expects`, freeze with `Spec::of::<T>().lock_in(your_spec_dir)`, and gate the drift with
`spec_lock::check` in a test. Freeze the mutation verdict beside it
(`MutationReport::of::<T>().lock_in(...)`) and your theory core's mutation testing runs
in-process, in milliseconds, with no CI job — reserve cargo-mutants for your plumbing
([`downstream-fixture`](downstream-fixture) is the copyable proof; the substitution is
measured in [docs/discovery.md](docs/discovery.md#algebra-level-mutation)). The interior
stays ordinary Rust — rigidity lives at the boundary, paid once.

## License

Dual-licensed under either of [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT), at your
option. Unless you state otherwise, any contribution you submit for inclusion shall be
dual-licensed as above, without additional terms.
