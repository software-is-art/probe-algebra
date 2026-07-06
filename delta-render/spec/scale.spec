# discovered spec: scale — a behaviour lock; regenerate via this repo's freeze path and ratify the diff.

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
- With scale, the grouping of three values doesn't matter.
      ((x scale y) scale z) = (x scale (y scale z))
- scale by zero always gives zero.
      (zero scale x) = zero
- scale distributes over plus.
      (x scale (y plus z)) = ((x scale y) plus (x scale z))
- scale distributes over plus from the right.
      ((y plus z) scale x) = ((y scale x) plus (z scale x))

# operators in no law (where the spec is silent): none — every operator participates in a law
