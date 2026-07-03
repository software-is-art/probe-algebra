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

## The next brick: candidates

1. **Transform seams end to end in genesis.** The compiled `system!` grammar can already
   discharge a transform seam (`A -- B : transform on V via h in Span;`), but genesis still
   leaves declared transform seams as `tests/seams.rs` meaning holes — it cannot name the
   conversion or the spanning theory. Extend the declaration grammar so a transform seam
   names its conversion op, and have genesis emit the spanning theory's skeleton (the
   conversion as one more `#[algebra]` operator with a `MEANING:` body) plus the compiled
   seam — then the flagship story covers both edge kinds.

2. **The system-level distance.** `Distance` reports declared-vs-discovered per module;
   the analogous report for the graph would compare the DECLARED seams against what
   `CohesionReport` finds latent in the composed operator tables — "you declared one system
   but the algebra decomposes into two", or "these two modules share laws no seam names".
   The pieces (cohesion, coherence, the compiled graph) all exist; the brick is the report.

3. **Equation render unification.** Genesis's target-lock EQUATIONS still restate the
   engine's term-render format by hand (the prose half is unified through the catalog). The
   byte-exact render pin forces sync, but deriving the equation from the shape's canonical
   terms would remove the last restatement.

## Standing follow-ups

- **Publish** when ready: `docs/publishing.md` has the dependency-ordered sequence;
  all four names (`boundary-spec`, `boundary-spec-macros`, `spec-lock`,
  `boundary-enforce`) were verified unclaimed on crates.io on 2026-07-02.
- **Morphism downstream**: the fixture exercises Construction/Branch/Guarded;
  the fourth edge shape is honestly unexercised downstream.
- **MSRV**: unpinned; verify and add `rust-version` after first publish.
