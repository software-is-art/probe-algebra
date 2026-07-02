# Roadmap — the next brick

The state as of this writing: the module-level loop is closed in both directions.
Bottom-up, a theory's algebra is discovered, frozen (`spec/*.spec`), and drift-gated;
top-down, a theory declares its laws (`expects { … }` on `theory!` / `#[algebra]`),
`Distance::of::<T>()` reports exactly what is unmet, and `genesis` derives a whole
crate layout from one `system!` declaration file — target locks born red, every
meaning hole a greppable `todo!("MEANING: …")`. The consumer story is proven by
`downstream-fixture`, the shape catalog and enforcement rules are themselves
spec-locked, and the four crates are publish-ready (`docs/publishing.md`).

## The next brick: the compiled `system!` graph layer

Make the SEAM GRAPH a declared, locked, compiled artifact — the application-level
spec that composes module specs — and demo blank-slate-to-green on top of it.

A large application's spec is not a big law list; it is a graph of algebras:
nodes are module specs, edges are seam obligations. Exactly two edge kinds exist,
and both already have checkers:

- **transport** — two modules share a value object and must agree on its laws
  (`CoherenceReport::between`, where signatures align; a documented obligation
  stub where they don't yet);
- **transform** — a conversion crosses modules and must be a homomorphism
  (`PipelineLaw::discover` — and composites are checked by running, never assumed,
  so verification scales with modules + seams, not their product).

What to build, in order:

1. **The compiled `system!` form.** `genesis` already parses the grammar from
   tokens (see `discover::genesis`'s module header for the productions); this step
   makes the same declaration COMPILE in a finished codebase: a `System` marker
   whose `modules()` replaces the hand-maintained `all_specs()` registry (the
   graph IS the registry, ratified in the diff like the kernel allowlist), and
   whose seams wire to the existing coherence/composition checks.
2. **The system lock.** Render the graph — modules, seams, each seam's obligation
   and status — to `spec/<system>.system.spec`, drift-gated like every other lock.
   Hierarchical ratification falls out: interior changes touch no lock, module law
   changes touch one module lock, only a re-drawn seam touches the system lock.
   Review attention scales with blast radius.
3. **Unifications** (small, flagged during construction):
   - replace `genesis`'s parse-side `Expect` enum with `discover::expect::Expectation`
     (the mapping is 1:1 by key);
   - derive `genesis`'s target-lock law rendering from `ShapeCatalog::inventory()`
     instead of restating the prose templates (its byte-exact render pin already
     forces sync, but derivation removes the duplication);
   - `shadow_grid` deserves a re-export nearer `boundary` (consumers' `Probed`
     impls legitimately want it; `discover::engine::` reads engine-internal).
4. **The flagship demo.** Genesis a two-module system from blank slate (the
   committed sample `examples/genesis_demo.rs` is the seed), implement the meaning
   holes, and watch it converge to green at BOTH levels — module distances, then
   seam obligations, then all locks fresh. That is the "large application from
   scratch" story in miniature, driven by a declaration that fits in one context
   window.

## Standing follow-ups

- **Publish** when ready: `docs/publishing.md` has the dependency-ordered sequence;
  all four names (`boundary-spec`, `boundary-spec-macros`, `spec-lock`,
  `boundary-enforce`) were verified unclaimed on crates.io on 2026-07-02.
- **Morphism downstream**: the fixture exercises Construction/Branch/Guarded;
  the fourth edge shape is honestly unexercised downstream.
- **MSRV**: unpinned; verify and add `rust-version` after first publish.
