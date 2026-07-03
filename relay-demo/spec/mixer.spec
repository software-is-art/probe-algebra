# discovered spec: mixer — a behaviour lock; regenerate via this repo's freeze path and ratify the diff.

- blend gives the same result in either order.
      (x blend y) = (y blend x)
- With blend, the grouping of three values doesn't matter.
      ((x blend y) blend z) = (x blend (y blend z))
- blend of a value with itself gives that value.
      (x blend x) = x

# operators in no law (where the spec is silent): cook
