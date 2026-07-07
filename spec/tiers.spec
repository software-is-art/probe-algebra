# tier census — the declared partition held against DERIVED evidence: INTERIOR is
# non-pub reachability, BOUNDARY is carrying production edge impls, ALGEBRA is
# the reachable remainder; pure glue stands as declared. KERNEL is a decision,
# recorded and never judged — a privilege cannot be inferred from conduct. A
# DISAGREES row is the
# honest distance between the declared partition and what structure can derive
# today; burn it down or improve the derivation before deleting any marker
# (the ladder: derive alongside, coherence-gate, then delete). Regenerate with
# `BLESS_TIERS=1 cargo build`.
# 46 files: 35 agree, 1 disagree, 10 kernel decisions.

- src/boundary.rs: declared KERNEL — a decision (ratified in build.rs), never derived; evidence for the record: pub-reachable, no production edges
- src/capability.rs: declared KERNEL — a decision (ratified in build.rs), never derived; evidence for the record: pub-reachable, no production edges
- src/discover/agenda.rs: declared ALGEBRA; derived ALGEBRA (pub-reachable, no production edges) — agree
- src/discover/architect.rs: declared ALGEBRA; derived ALGEBRA (pub-reachable, no production edges) — agree
- src/discover/arithmetic.rs: declared ALGEBRA; derived ALGEBRA (pub-reachable, no production edges) — agree
- src/discover/bite.rs: declared ALGEBRA; derived ALGEBRA (pub-reachable, no production edges) — agree
- src/discover/bridge.rs: declared ALGEBRA; derived ALGEBRA (pub-reachable, no production edges) — agree
- src/discover/coherence.rs: declared ALGEBRA; derived ALGEBRA (pub-reachable, no production edges) — agree
- src/discover/cohesion.rs: declared ALGEBRA; derived ALGEBRA (pub-reachable, no production edges) — agree
- src/discover/composition.rs: declared ALGEBRA; derived ALGEBRA (pub-reachable, no production edges) — agree
- src/discover/date.rs: declared ALGEBRA; derived ALGEBRA (pub-reachable, no production edges) — agree
- src/discover/depend.rs: declared ALGEBRA; derived ALGEBRA (pub-reachable, no production edges) — agree
- src/discover/derived.rs: declared ALGEBRA — glue (module declarations and re-exports only, no own substance); the declared tier stands
- src/discover/engine.rs: declared KERNEL — a decision (ratified in build.rs), never derived; evidence for the record: pub-reachable, no production edges
- src/discover/expect.rs: declared KERNEL — a decision (ratified in build.rs), never derived; evidence for the record: pub-reachable, no production edges
- src/discover/freeze.rs: declared ALGEBRA; derived ALGEBRA (pub-reachable, no production edges) — agree
- src/discover/gates.rs: declared ALGEBRA; derived ALGEBRA (pub-reachable, no production edges) — agree
- src/discover/genesis.rs: declared ALGEBRA; derived ALGEBRA (pub-reachable, no production edges) — agree
- src/discover/layering.rs: declared ALGEBRA; derived ALGEBRA (pub-reachable, no production edges) — agree
- src/discover/mod.rs: declared KERNEL — a decision (ratified in build.rs), never derived; evidence for the record: pub-reachable, no production edges
- src/discover/modularize.rs: declared ALGEBRA; derived ALGEBRA (pub-reachable, no production edges) — agree
- src/discover/mutation.rs: declared ALGEBRA; derived ALGEBRA (pub-reachable, no production edges) — agree
- src/discover/protocol.rs: declared ALGEBRA; derived ALGEBRA (pub-reachable, no production edges) — agree
- src/discover/residue.rs: declared ALGEBRA; derived ALGEBRA (pub-reachable, no production edges) — agree
- src/discover/router.rs: declared ALGEBRA; derived ALGEBRA (pub-reachable, no production edges) — agree
- src/discover/scaffold.rs: declared ALGEBRA; derived ALGEBRA (pub-reachable, no production edges) — agree
- src/discover/shape.rs: declared ALGEBRA; derived ALGEBRA (pub-reachable, no production edges) — agree
- src/discover/system.rs: declared ALGEBRA; derived ALGEBRA (pub-reachable, no production edges) — agree
- src/discover/watch.rs: declared ALGEBRA; derived ALGEBRA (pub-reachable, no production edges) — agree
- src/discover/world.rs: declared ALGEBRA; derived ALGEBRA (pub-reachable, no production edges) — agree
- src/gdp.rs: declared KERNEL — a decision (ratified in build.rs), never derived; evidence for the record: pub-reachable, no production edges
- src/harness.rs: declared KERNEL — a decision (ratified in build.rs), never derived; evidence for the record: not pub-reachable
- src/interp/boundary.rs: declared BOUNDARY; derived BOUNDARY (pub-reachable, carries production edges) — agree
- src/interp/internal.rs: declared INTERIOR; derived INTERIOR (not pub-reachable) — agree
- src/interp/mod.rs: declared INTERIOR — glue (module declarations and re-exports only, no own substance); the declared tier stands
- src/kvstore/internal.rs: declared INTERIOR; derived INTERIOR (not pub-reachable) — agree
- src/kvstore/mod.rs: declared INTERIOR — glue (module declarations and re-exports only, no own substance); the declared tier stands
- src/kvstore/probes.rs: declared INTERIOR; derived INTERIOR (not pub-reachable) — agree
- src/kvstore/store.rs: declared BOUNDARY; derived BOUNDARY (pub-reachable, carries production edges) — agree
- src/kvstore/theory.rs: declared ALGEBRA; derived ALGEBRA (pub-reachable, no production edges) — agree
- src/lib.rs: declared KERNEL — a decision (ratified in build.rs), never derived; evidence for the record: pub-reachable, no production edges
- src/main.rs: declared KERNEL — a decision (ratified in build.rs), never derived; evidence for the record: pub-reachable, no production edges
- src/select/boundary.rs: declared BOUNDARY; derived ALGEBRA (pub-reachable, no production edges) — DISAGREES
- src/select/internal.rs: declared INTERIOR; derived INTERIOR (not pub-reachable) — agree
- src/select/mod.rs: declared INTERIOR — glue (module declarations and re-exports only, no own substance); the declared tier stands
- src/tests.rs: declared KERNEL — a decision (ratified in build.rs), never derived; evidence for the record: not pub-reachable
