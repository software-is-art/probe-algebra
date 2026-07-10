# discovered spec: stream — a behaviour lock; regenerate via this repo's freeze path and ratify the diff.

- plus gives the same result in either order.
      (s plus t) = (t plus s)
- With plus, the grouping of three values doesn't matter.
      ((s plus t) plus u) = (s plus (t plus u))
- plus with zero leaves a value unchanged.
      (zero plus s) = s
- neg inverts plus — a value plus its own neg gives zero.
      (s plus neg(s)) = zero
- neg twice returns the original value.
      neg(neg(s)) = s
- neg leaves zero fixed.
      neg(zero) = zero
- neg turns plus into plus.
      neg((s plus t)) = (neg(s) plus neg(t))
- delay leaves zero fixed.
      delay(zero) = zero
- delay and neg may be applied in either order.
      delay(neg(s)) = neg(delay(s))
- delay and i may be applied in either order.
      delay(i(s)) = i(delay(s))
- delay turns plus into plus.
      delay((s plus t)) = (delay(s) plus delay(t))
- d leaves zero fixed.
      d(zero) = zero
- d undoes i — the round trip is the identity.
      d(i(s)) = s
- d and neg may be applied in either order.
      d(neg(s)) = neg(d(s))
- d and delay may be applied in either order.
      d(delay(s)) = delay(d(s))
- d and i may be applied in either order.
      d(i(s)) = i(d(s))
- d turns plus into plus.
      d((s plus t)) = (d(s) plus d(t))
- i leaves zero fixed.
      i(zero) = zero
- i undoes d — the round trip is the identity.
      i(d(s)) = s
- i and neg may be applied in either order.
      i(neg(s)) = neg(i(s))
- i turns plus into plus.
      i((s plus t)) = (i(s) plus i(t))

# operators in no law (where the spec is silent): none — every operator participates in a law
