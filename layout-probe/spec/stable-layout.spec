# discovered spec: stable layout — a behaviour lock; regenerate via this repo's freeze path and ratify the diff.

- reorder twice returns the original value.
      reorder(reorder(x)) = x
- reorder is a projection — applying it twice is applying it once.
      reorder(reorder(x)) = reorder(x)
- reorder undoes theme — the round trip is the identity.
      reorder(theme(x)) = x
- reorder and theme may be applied in either order.
      reorder(theme(x)) = theme(reorder(x))
- reorder leaves every value unchanged.
      reorder(x) = x
- reorder is equivariant — rename before it becomes rename after it.
      reorder(rename(x, p)) = rename(reorder(x), p)
- theme twice returns the original value.
      theme(theme(x)) = x
- theme is a projection — applying it twice is applying it once.
      theme(theme(x)) = theme(x)
- theme undoes reorder — the round trip is the identity.
      theme(reorder(x)) = x
- theme leaves every value unchanged.
      theme(x) = x
- theme is equivariant — rename before it becomes rename after it.
      theme(rename(x, p)) = rename(theme(x), p)
- rename actually acts — some parameter moves some value.
      rename(x, p) ≠ x
- relabel actually acts — some parameter moves some value.
      relabel(g, p) ≠ g

# operators in no law (where the spec is silent): render
