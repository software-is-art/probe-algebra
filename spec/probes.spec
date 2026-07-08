# probe census: every probe lock this crate upholds, with the mechanism that proves it sensitive.
# a green probe that cannot fail is a lie; this roster names how each proves it can. Regenerate
# via `cargo run --example freeze_spec` and ratify the diff.
# mechanisms: oracle-swap (laws, discover::mutation), live-dent (world judges, discover::judgment),
#   drift-gate (byte-lock, freshness only — rung 1: disclosed, no active refutation drill; see docs/roadmap.md).

## behavioural probes (conduct — laws over a grid)
- bridged bool: oracle-swap
- date calculus: oracle-swap
- doc flow: oracle-swap
- fabric: oracle-swap
- interpreter arithmetic: oracle-swap
- router: oracle-swap
- store protocol: oracle-swap
- ttl store: oracle-swap

## structural probes (shape)
- catalog: fire-drill
- infra: live-dent
- perimeter: live-dent
- pipeline: drift-gate
- schemata: drift-gate
- seams: fire-drill
- shape: fire-drill
- substrate: live-dent
- surface: drift-gate
- tiers: drift-gate
- world: fire-drill

