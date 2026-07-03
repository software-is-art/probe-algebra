# world lock: store model — recorded conduct over the derived trace battery; regenerate via this repo's freeze path and ratify the diff.
#
# You cannot ratify the world, but you can ratify your ASSUMPTIONS about it: each
# row is a canonical trace (derived from the command type's own structure) and the
# observation the model predicts. The conformance gate replays the same battery
# against the real dependency and holds it to this text — a divergence names the
# exact trace where the world left the assumptions. Grid-bounded, like all
# discovery: the battery refutes conformance, it never proves it.

- Trace([])
      -> {}

- Trace([Put(A, V0)])
      -> {A: V0}

- Trace([Del(A)])
      -> {}

- Trace([Put(B, V0)])
      -> {B: V0}

- Trace([Put(A, V1)])
      -> {A: V1}

- Trace([Del(B)])
      -> {}

- Trace([Put(B, V1)])
      -> {B: V1}

- Trace([Put(A, V0), Put(A, V0)])
      -> {A: V0}

- Trace([Put(A, V0), Del(A)])
      -> {}

- Trace([Put(A, V0), Put(B, V0)])
      -> {A: V0, B: V0}

- Trace([Put(A, V0), Put(A, V1)])
      -> {A: V1}

- Trace([Put(A, V0), Del(B)])
      -> {A: V0}

- Trace([Put(A, V0), Put(B, V1)])
      -> {A: V0, B: V1}

- Trace([Del(A), Put(A, V0)])
      -> {A: V0}

- Trace([Del(A), Del(A)])
      -> {}

- Trace([Del(A), Put(B, V0)])
      -> {B: V0}

- Trace([Del(A), Put(A, V1)])
      -> {A: V1}

- Trace([Del(A), Del(B)])
      -> {}

- Trace([Del(A), Put(B, V1)])
      -> {B: V1}

- Trace([Put(B, V0), Put(A, V0)])
      -> {A: V0, B: V0}

- Trace([Put(B, V0), Del(A)])
      -> {B: V0}

- Trace([Put(B, V0), Put(B, V0)])
      -> {B: V0}

- Trace([Put(B, V0), Put(A, V1)])
      -> {A: V1, B: V0}

- Trace([Put(B, V0), Del(B)])
      -> {}
