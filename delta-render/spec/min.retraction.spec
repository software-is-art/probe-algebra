# the min retraction — why `min` earns no license, frozen as a WITNESS — regenerate via this repo's freeze path and ratify the diff.
#
# Additivity is what a delta needs: f(state plus delta) = f(state) plus f(delta).
# For min, ONE retraction refutes it: deleting the current minimum uncovers the
# next, and no delta short of recomputation says which row that is. The instance
# below is computed by the real operators, not typed:
#
#   state = {r0: +1, r1: +1}   delta = {r0: -1}
#   min(state plus delta)       = {r1: +1}
#   min(state) plus min(delta)  = {r0: +1}
#   the two routes DISAGREE — the incremental one still names the deleted row.
#
# This is the red instance behind `min: NEITHER` in spec/licenses.spec; the
# generic fallback (D ∘ Q ∘ I) is what correctness costs here.
