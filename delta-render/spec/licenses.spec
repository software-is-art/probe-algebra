# license registry: operator → classification, DERIVED by parsing the frozen law specs — regenerate via this repo's freeze path and ratify the diff.
#
# The pivot artifact: discovery output consumed as generation input. A
# classification is the PRESENCE of laws in the operator's spec, never a
# declared boolean:
#   linear   ⇔ additive homomorphism over the Z-set group AND zero preserved
#   bilinear ⇔ additive in each argument slot (the distributivity pair, or one
#              distributivity law plus commutativity)
#   neither  ⇔ no license — the generic fallback (Q^Δ = D ∘ Q ∘ I) applies

- filter: LINEAR
      spec/filter.spec: "filter turns plus into plus."
      spec/filter.spec: "filter leaves zero fixed."

- map: LINEAR
      spec/map.spec: "map turns plus into plus."
      spec/map.spec: "map leaves zero fixed."

- sum: LINEAR
      spec/sum.spec: "sum turns plus into plus."
      spec/sum.spec: "sum leaves zero fixed."

- join: BILINEAR
      spec/join.spec: "join distributes over plus."
      spec/join.spec: "join gives the same result in either order."

- scale: BILINEAR
      spec/scale.spec: "scale distributes over plus."
      spec/scale.spec: "scale distributes over plus from the right."

- distinct: NEITHER
      no additivity law in spec/distinct.spec — every delta recomputes (D ∘ Q ∘ I)

- min: NEITHER
      no additivity law in spec/min.spec — every delta recomputes (D ∘ Q ∘ I)
