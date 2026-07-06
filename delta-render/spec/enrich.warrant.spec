# uninterpreted-operator warrant: `enrich` — a linear license held on SAMPLED
# evidence; regenerate via `cargo run -p delta-render --example freeze` and ratify
# the diff.
#
# `enrich` has no inventoried implementation — it stands for the open half of a
# real pipeline's operator inventory. Its license is judged over interpretations
# sampled under the declared properties (deterministic: splitmix64, seed
# 0x9e3779b97f4a7c15), and every declared property must earn its place by the REMOVAL
# DRILL: drop it, re-sample under the remaining constraints, and the circuit law
# must fail. A property whose removal refutes nothing is DECORATION — flagged
# below, never ratified.
#
# Honest frame: sampled interpretations are a bounded battery — the drill refutes
# decoration and warrants necessity; it proves neither.
#
# the circuit law (the linear rewrite Q^Δ = Q, end-gate form, tickwise over the
# stream grid): i(enrich(d(s))) = enrich(s)

license: LINEAR, warranted — the circuit law held over all 8 sampled interpretations × 6 stream-grid inhabitants.

ratified properties (each load-bearing under the removal drill):
- additive: enrich turns plus into plus (away from the basepoint).
      removal refuted the law in 8 of 8 counter-samples; first witness: sample #0, stream #5, tick 1: incremental {250:+2} ≠ batch ∅
- zero-preserving: enrich leaves zero fixed (the basepoint).
      removal refuted the law in 8 of 8 counter-samples; first witness: sample #0, stream #0, tick 1: incremental {251:+2} ≠ batch {251:+1}
- deterministic: enrich answers the same input identically, every call.
      removal refuted the law in 8 of 8 counter-samples; first witness: sample #0, stream #0, tick 0: incremental {252:+6} ≠ batch {252:+1}

decoration (declared, drilled, found weightless — flagged, not ratified):
- bounded-fanout: enrich maps one row to at most two rows.
      removal refuted the law in 0 of 8 counter-samples — the license never leaned on it.
