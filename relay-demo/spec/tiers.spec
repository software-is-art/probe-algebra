# the tier partition, DERIVED — the single source the rule dispatch and this lock
# both consume. INTERIOR is non-pub reachability; BOUNDARY is being a DOOR
# (production edge impls, or fronting an interior sibling — the tier-2 relation
# read backwards); ALGEBRA is the reachable remainder; glue takes its
# reachability's tier. KERNEL is a decision, never derived: it is ratified in the
# consumer's own tree (the build.rs allowlist, or a register it parses).
# Regenerate with `BLESS_RELAY_APP_TIERS=1 cargo build`.
# 8 files: 2 boundary, 2 interior, 3 algebra, 1 kernel.
# rule KERNEL: the trusted floor — exempt from the structural rules; a ratified privilege
# rule BOUNDARY: tier 1 — a domain's strict value-object surface; no loose `pub fn`
# rule INTERIOR: tier 2 — the workshop; mutation and raw collections allowed; no loose `pub fn`
# rule ALGEBRA: the discovered-law / report layer; exempt from the inward rule; no loose `pub fn`

- src/gates.rs: ALGEBRA (pub-reachable, no production edges, fronts nothing)
- src/gauge.rs: BOUNDARY (pub-reachable, fronts an interior sibling)
- src/gauge_internal.rs: INTERIOR (not pub-reachable)
- src/lib.rs: KERNEL (registered — a decision, never derived)
- src/mixer.rs: BOUNDARY (pub-reachable, fronts an interior sibling)
- src/mixer_internal.rs: INTERIOR (not pub-reachable)
- src/ops.rs: ALGEBRA (glue — module declarations and re-exports only; tier by reachability)
- src/system.rs: ALGEBRA (pub-reachable, no production edges, fronts nothing)
