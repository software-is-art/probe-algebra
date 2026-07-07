# the tier partition, DERIVED — the single source the rule dispatch and this lock
# both consume. INTERIOR is non-pub reachability; BOUNDARY is being a DOOR
# (production edge impls, or fronting an interior sibling — the tier-2 relation
# read backwards); ALGEBRA is the reachable remainder; glue takes its
# reachability's tier. KERNEL is a decision, never derived: it is ratified in the
# consumer's own tree (the build.rs allowlist, or a register it parses).
# Regenerate with `BLESS_CREDIT_APP_TIERS=1 cargo build`.
# 8 files: 2 boundary, 2 interior, 3 algebra, 1 kernel.

- src/billing.rs: BOUNDARY (pub-reachable, fronts an interior sibling)
- src/billing_internal.rs: INTERIOR (not pub-reachable)
- src/gates.rs: ALGEBRA (pub-reachable, no production edges, fronts nothing)
- src/lib.rs: KERNEL (registered — a decision, never derived)
- src/meter.rs: BOUNDARY (pub-reachable, fronts an interior sibling)
- src/meter_internal.rs: INTERIOR (not pub-reachable)
- src/ops.rs: ALGEBRA (glue — module declarations and re-exports only; tier by reachability)
- src/system.rs: ALGEBRA (pub-reachable, no production edges, fronts nothing)
