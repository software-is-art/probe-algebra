# discovered spec: zset kernel — a behaviour lock; regenerate via this repo's freeze path and ratify the diff.

- Add gives the same result in either order.
      (a + b) = (b + a)
- With Add, the grouping of three values doesn't matter.
      ((a + b) + c) = (a + (b + c))
- Add with zero leaves a value unchanged.
      (zero + a) = a
- Neg inverts Add — a value Add its own Neg gives zero.
      (a + neg(a)) = zero
- Neg twice returns the original value.
      neg(neg(a)) = a
- Neg leaves zero fixed.
      neg(zero) = zero
- Neg turns Add into Add.
      neg((a + b)) = (neg(a) + neg(b))
- Join gives the same result in either order.
      (a join b) = (b join a)
- With Join, the grouping of three values doesn't matter.
      ((a join b) join c) = (a join (b join c))
- Join by zero always gives zero.
      (zero join a) = zero
- Join distributes over Add.
      (a join (b + c)) = ((a join b) + (a join c))
- Distinct is a projection — applying it twice is applying it once.
      distinct(distinct(a)) = distinct(a)
- Distinct leaves zero fixed.
      distinct(zero) = zero
- Delay and Integrate may be applied in either order.
      delay(integrate(s)) = integrate(delay(s))
- Delay and Differentiate may be applied in either order.
      delay(differentiate(s)) = differentiate(delay(s))
- Integrate undoes Differentiate — the round trip is the identity.
      integrate(differentiate(s)) = s
- Differentiate undoes Integrate — the round trip is the identity.
      differentiate(integrate(s)) = s
- Differentiate and Integrate may be applied in either order.
      differentiate(integrate(s)) = integrate(differentiate(s))
- Latest undoes Impulse — the round trip is the identity.
      latest(impulse(a)) = a

# operators in no law (where the spec is silent): none — every operator participates in a law
