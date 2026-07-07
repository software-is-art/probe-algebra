# discovered spec: fabric — a behaviour lock; regenerate via this repo's freeze path and ratify the diff.

- Join gives the same result in either order.
      (f join g) = (g join f)
- With Join, the grouping of three values doesn't matter.
      ((f join g) join h) = (f join (g join h))
- Join of a value with itself gives that value.
      (f join f) = f
- Join with mesh leaves a value unchanged.
      (mesh join f) = f
- Repeated Grant with one parameter settles on the first application.
      grant(grant(f, r), r) = grant(f, r)
- Grant applications commute — the parameter order doesn't matter.
      grant(grant(f, r), s) = grant(grant(f, s), r)
- Grant distributes over Join — acting on a combination is combining the actions.
      grant((f join g), r) = (grant(f, r) join grant(g, r))
- Grant only grows a value — never shrinks it (under Within).
      within(f, grant(f, r)) = true
- Repeated Revoke with one parameter settles on the first application.
      revoke(revoke(f, r), r) = revoke(f, r)
- Revoke applications commute — the parameter order doesn't matter.
      revoke(revoke(f, r), s) = revoke(revoke(f, s), r)
- Revoke distributes over Join — acting on a combination is combining the actions.
      revoke((f join g), r) = (revoke(f, r) join revoke(g, r))
- Revoke only shrinks a value — never grows it (under Within).
      within(revoke(f, r), f) = true
- Reach is a projection — applying it twice is applying it once.
      reach(reach(f)) = reach(f)
- Reach leaves mesh fixed.
      reach(mesh) = mesh
- Reach is subadditive over Join (under Within).
      within(reach((f join g)), (reach(f) join reach(g))) = true
- Reach is monotone under Within.
      within(f, g) = true ⟹ within(reach(f), reach(g)) = true
- Within of a value with itself gives true.
      within(f, f) = true
- Grant actually acts — some parameter moves some value.
      grant(f, r) ≠ f
- Revoke actually acts — some parameter moves some value.
      revoke(f, r) ≠ f
- Within is not constantly true.
      within(f, g) ≠ true

# operators in no law (where the spec is silent): none — every operator participates in a law
