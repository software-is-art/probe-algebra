# discovered spec: filter — a behaviour lock; regenerate via this repo's freeze path and ratify the diff.

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
- filter is a projection — applying it twice is applying it once.
      filter(filter(x)) = filter(x)
- filter leaves zero fixed.
      filter(zero) = zero
- filter and neg may be applied in either order.
      filter(neg(x)) = neg(filter(x))
- filter turns plus into plus.
      filter((x plus y)) = (filter(x) plus filter(y))

# operators in no law (where the spec is silent): none — every operator participates in a law
