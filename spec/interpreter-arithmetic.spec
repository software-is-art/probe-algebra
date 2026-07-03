# discovered spec: interpreter arithmetic — a behaviour lock; regenerate via this repo's freeze path and ratify the diff.

- Addition gives the same result in either order.
      (x + y) = (y + x)
- With Addition, the grouping of three values doesn't matter.
      ((x + y) + z) = (x + (y + z))
- Addition with 0 leaves a value unchanged.
      (0 + x) = x
- Multiplication gives the same result in either order.
      (x * y) = (y * x)
- With Multiplication, the grouping of three values doesn't matter.
      ((x * y) * z) = (x * (y * z))
- Multiplication with 1 leaves a value unchanged.
      (1 * x) = x
- Multiplication by 0 always gives 0.
      (0 * x) = 0
- Multiplication distributes over Addition.
      (x * (y + z)) = ((x * y) + (x * z))
- A value is never less than itself.
      (x < x) = false
- less than is not constantly false.
      (x < y) ≠ false
- No two distinct programs look the same — the faithful rendering distinguishes every structural and semantic difference.
      U(p) = U(q)  ⟹  p = q   (U = faithful render)

# operators in no law (where the spec is silent): none — every operator participates in a law
