# discovered spec: ttl store — a behaviour lock; regenerate via this repo's freeze path and ratify the diff.

- With Merge, the grouping of three values doesn't matter.
      ((s <+ t) <+ u) = (s <+ (t <+ u))
- Merge of a value with itself gives that value.
      (s <+ s) = s
- With Merge, the later operand wins where the two disagree — re-applying an earlier one cannot overwrite it.
      ((s <+ t) <+ s) = (t <+ s)
- Merge with empty leaves a value unchanged.
      (empty <+ s) = s
- Tick with zero leaves a value unchanged.
      tick(s, zero) = s
- Repeated Tick combines its parameters with Plus.
      tick(tick(s, p), q) = tick(s, (p + q))
- Tick applications commute — the parameter order doesn't matter.
      tick(tick(s, p), q) = tick(tick(s, q), p)
- Tick leaves empty fixed — no parameter moves it.
      tick(empty, p) = empty
- Plus gives the same result in either order.
      (p + q) = (q + p)
- With Plus, the grouping of three values doesn't matter.
      ((p + q) + r) = (p + (q + r))
- Plus with zero leaves a value unchanged.
      (zero + p) = p
- Tick actually acts — some parameter moves some value.
      tick(s, p) ≠ s

# operators in no law (where the spec is silent): none — every operator participates in a law
