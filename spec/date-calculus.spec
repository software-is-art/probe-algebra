# discovered spec: date calculus — a behaviour lock; regenerate via this repo's freeze path and ratify the diff.

- Plus gives the same result in either order.
      (p + q) = (q + p)
- With Plus, the grouping of three values doesn't matter.
      ((p + q) + r) = (p + (q + r))
- Plus with zero leaves a value unchanged.
      (zero + p) = p
- Add with zero leaves a value unchanged.
      add(s, zero) = s
- Repeated Add combines its parameters with Plus.
      add(add(s, p), q) = add(s, (p + q))
- Add applications commute — the parameter order doesn't matter.
      add(add(s, p), q) = add(add(s, q), p)
- Diff of a value with itself gives zero.
      diff(s, s) = zero
- since undoes at — the round trip is the identity.
      since(at(p)) = p
- at undoes since — the round trip is the identity.
      at(since(s)) = s
- Add actually acts — some parameter moves some value.
      add(s, p) ≠ s
- Diff is not constantly zero.
      diff(s, t) ≠ zero

# operators in no law (where the spec is silent): none — every operator participates in a law
