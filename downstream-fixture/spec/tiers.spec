# the tier partition, DERIVED — the single source the rule dispatch and this lock
# both consume. INTERIOR is non-pub reachability; BOUNDARY is being a DOOR
# (production edge impls, or fronting an interior sibling — the tier-2 relation
# read backwards); ALGEBRA is the reachable remainder; glue takes its
# reachability's tier. KERNEL is a decision, never derived: it is ratified in the
# consumer's own tree (the build.rs allowlist, or a register it parses).
# Regenerate with `BLESS_FIXTURE_TIERS=1 cargo build`.
# 4 files: 1 boundary, 1 interior, 1 algebra, 1 kernel.

- src/internal.rs: INTERIOR (not pub-reachable)
- src/lib.rs: KERNEL (registered — a decision, never derived)
- src/meter.rs: BOUNDARY (pub-reachable, carries production edges)
- src/ops.rs: ALGEBRA (glue — module declarations and re-exports only; tier by reachability)
