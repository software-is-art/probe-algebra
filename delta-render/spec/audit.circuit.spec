# incremental circuit: audit — the derivation, node by node, each rule cited to the license that granted it — regenerate via this repo's freeze path and ratify the diff.
#
# The licenses come from spec/licenses.spec (themselves derived from the frozen
# law specs); the generated code is gen/audit_incremental.rs; the end gate
# (I ∘ Q^Δ ∘ D = Q over the stream grid) holds regardless of what any license
# says. A demoted license shows up here as a rule change, named.
#
# sources: s0, s1
# output:  n3

- node 0 `join(s0, s1)`: BILINEAR — the three-term delta: Δ(a ⋈ b) = Δa ⋈ delay(I(Δb)) plus delay(I(Δa)) ⋈ Δb plus Δa ⋈ Δb.
      licensed by spec/join.spec: "join distributes over plus."
      licensed by spec/join.spec: "join gives the same result in either order."

- node 1 `filter(n0)`: LINEAR — the operator is its own incremental form (Q^Δ = Q).
      licensed by spec/filter.spec: "filter turns plus into plus."
      licensed by spec/filter.spec: "filter leaves zero fixed."

- node 2 `scale(n1, n0)`: BILINEAR — the three-term delta: Δ(a ⋈ b) = Δa ⋈ delay(I(Δb)) plus delay(I(Δa)) ⋈ Δb plus Δa ⋈ Δb.
      licensed by spec/scale.spec: "scale distributes over plus."
      licensed by spec/scale.spec: "scale distributes over plus from the right."

- node 3 `sum(n2)`: LINEAR — the operator is its own incremental form (Q^Δ = Q).
      licensed by spec/sum.spec: "sum turns plus into plus."
      licensed by spec/sum.spec: "sum leaves zero fixed."
