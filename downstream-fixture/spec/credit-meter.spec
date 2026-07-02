# discovered spec: credit meter — a behaviour lock; regenerate via this repo's freeze path and ratify the diff.

- grant gives the same result in either order.
      (x grant y) = (y grant x)
- With grant, the grouping of three values doesn't matter.
      ((x grant y) grant z) = (x grant (y grant z))
- grant with zero leaves a value unchanged.
      (zero grant x) = x
- spend with zero leaves a value unchanged.
      (x spend zero) = x
- spend by zero always gives zero.
      (zero spend x) = zero
- With renew, the grouping of three values doesn't matter.
      ((x renew y) renew z) = (x renew (y renew z))
- renew of a value with itself gives that value.
      (x renew x) = x
- With renew, the later operand wins where the two disagree — re-applying an earlier one cannot overwrite it.
      ((x renew y) renew x) = (y renew x)
- renew with zero leaves a value unchanged.
      (zero renew x) = x

# operators in no law (where the spec is silent): none — every operator participates in a law
