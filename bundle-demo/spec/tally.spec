# discovered spec: tally — a behaviour lock; regenerate via this repo's freeze path and ratify the diff.

- merge gives the same result in either order.
      (x merge y) = (y merge x)
- With merge, the grouping of three values doesn't matter.
      ((x merge y) merge z) = (x merge (y merge z))
- merge of a value with itself gives that value.
      (x merge x) = x
- merge with floor leaves a value unchanged.
      (floor merge x) = x
- bump turns merge into merge.
      bump((x merge y)) = (bump(x) merge bump(y))

# operators in no law (where the spec is silent): none — every operator participates in a law
