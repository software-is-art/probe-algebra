# incremental circuit: demo — the derivation, node by node, each rule cited to the license that granted it — regenerate via this repo's freeze path and ratify the diff.
#
# The licenses come from spec/licenses.spec (themselves derived from the frozen
# law specs); the generated code is gen/demo_incremental.rs; the end gate
# (I ∘ Q^Δ ∘ D = Q over the stream grid) holds regardless of what any license
# says. A demoted license shows up here as a rule change, named.
#
# sources: s0
# output:  n4

- node 0 `filter(s0)`: LINEAR — the operator is its own incremental form (Q^Δ = Q).
      licensed by spec/filter.spec: "filter turns plus into plus."
      licensed by spec/filter.spec: "filter leaves zero fixed."

- node 1 `map(n0)`: LINEAR — the operator is its own incremental form (Q^Δ = Q).
      licensed by spec/map.spec: "map turns plus into plus."
      licensed by spec/map.spec: "map leaves zero fixed."

- node 2 `join(n1, s0)`: BILINEAR — the three-term delta: Δ(a ⋈ b) = Δa ⋈ delay(I(Δb)) plus delay(I(Δa)) ⋈ Δb plus Δa ⋈ Δb.
      licensed by spec/join.spec: "join distributes over plus."
      licensed by spec/join.spec: "join gives the same result in either order."

- node 3 `distinct(n2)`: NEITHER — generic fallback (Q^Δ = D ∘ Q ∘ I): integrate, recompute, differentiate; correct always, cheap never.
      no license in spec/distinct.spec — every delta recomputes

- node 4 `sum(n3)`: LINEAR — the operator is its own incremental form (Q^Δ = Q).
      licensed by spec/sum.spec: "sum turns plus into plus."
      licensed by spec/sum.spec: "sum leaves zero fixed."
