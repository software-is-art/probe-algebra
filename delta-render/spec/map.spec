# discovered spec: map — a behaviour lock; regenerate via this repo's freeze path and ratify the diff.

- plus gives the same result in either order.
      (x plus y) = (y plus x)
- With plus, the grouping of three values doesn't matter.
      ((x plus y) plus z) = (x plus (y plus z))
- plus with zero leaves a value unchanged.
      (zero plus x) = x
- neg inverts plus — a value plus its own neg gives zero.
      (x plus neg(x)) = zero
- neg twice returns the original value.
      neg(neg(x)) = x
- neg leaves zero fixed.
      neg(zero) = zero
- neg turns plus into plus.
      neg((x plus y)) = (neg(x) plus neg(y))
- map is a projection — applying it twice is applying it once.
      map(map(x)) = map(x)
- map leaves zero fixed.
      map(zero) = zero
- map and neg may be applied in either order.
      map(neg(x)) = neg(map(x))
- map turns plus into plus.
      map((x plus y)) = (map(x) plus map(y))

# operators in no law (where the spec is silent): none — every operator participates in a law
