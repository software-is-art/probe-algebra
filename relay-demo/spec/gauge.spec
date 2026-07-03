# discovered spec: gauge — a behaviour lock; regenerate via this repo's freeze path and ratify the diff.

- fuse gives the same result in either order.
      (x fuse y) = (y fuse x)
- With fuse, the grouping of three values doesn't matter.
      ((x fuse y) fuse z) = (x fuse (y fuse z))
- fuse of a value with itself gives that value.
      (x fuse x) = x

# operators in no law (where the spec is silent): none — every operator participates in a law
