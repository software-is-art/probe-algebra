# discovered spec: eager layout — a behaviour lock; regenerate via this repo's freeze path and ratify the diff.

- render is equivariant — rename before it becomes relabel after it.
      render(rename(x, p)) = relabel(render(x), p)
- reorder and theme may be applied in either order.
      reorder(theme(x)) = theme(reorder(x))
- reorder is equivariant — rename before it becomes rename after it.
      reorder(rename(x, p)) = rename(reorder(x), p)
- theme twice returns the original value.
      theme(theme(x)) = x
- theme is a projection — applying it twice is applying it once.
      theme(theme(x)) = theme(x)
- theme leaves every value unchanged.
      theme(x) = x
- theme is equivariant — rename before it becomes rename after it.
      theme(rename(x, p)) = rename(theme(x), p)
- rename actually acts — some parameter moves some value.
      rename(x, p) ≠ x
- relabel actually acts — some parameter moves some value.
      relabel(g, p) ≠ g

# operators in no law (where the spec is silent): none — every operator participates in a law
