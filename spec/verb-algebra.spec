# discovered spec: verb algebra — a behaviour lock; regenerate via this repo's freeze path and ratify the diff.

- add_a is a projection — applying it twice is applying it once.
      add_a(add_a(x)) = add_a(x)
- add_a and add_b may be applied in either order.
      add_a(add_b(x)) = add_b(add_a(x))
- add_a and edit_b may be applied in either order.
      add_a(edit_b(x)) = edit_b(add_a(x))
- add_a and collect_b may be applied in either order.
      add_a(collect_b(x)) = collect_b(add_a(x))
- add_a and declare may be applied in either order.
      add_a(declare(x)) = declare(add_a(x))
- add_a and recast_b may be applied in either order.
      add_a(recast_b(x)) = recast_b(add_a(x))
- add_b is a projection — applying it twice is applying it once.
      add_b(add_b(x)) = add_b(x)
- add_b and edit_a may be applied in either order.
      add_b(edit_a(x)) = edit_a(add_b(x))
- add_b and collect_a may be applied in either order.
      add_b(collect_a(x)) = collect_a(add_b(x))
- add_b and declare may be applied in either order.
      add_b(declare(x)) = declare(add_b(x))
- add_b and recast_a may be applied in either order.
      add_b(recast_a(x)) = recast_a(add_b(x))
- edit_a is a projection — applying it twice is applying it once.
      edit_a(edit_a(x)) = edit_a(x)
- edit_a leaves empty fixed.
      edit_a(empty) = empty
- edit_a and edit_b may be applied in either order.
      edit_a(edit_b(x)) = edit_b(edit_a(x))
- edit_a and recast_a may be applied in either order.
      edit_a(recast_a(x)) = recast_a(edit_a(x))
- edit_a and recast_b may be applied in either order.
      edit_a(recast_b(x)) = recast_b(edit_a(x))
- edit_a after collect_a collapses to collect_a.
      edit_a(collect_a(x)) = collect_a(x)
- edit_a after recast_a collapses to recast_a.
      edit_a(recast_a(x)) = recast_a(x)
- edit_b is a projection — applying it twice is applying it once.
      edit_b(edit_b(x)) = edit_b(x)
- edit_b leaves empty fixed.
      edit_b(empty) = empty
- edit_b and recast_a may be applied in either order.
      edit_b(recast_a(x)) = recast_a(edit_b(x))
- edit_b and recast_b may be applied in either order.
      edit_b(recast_b(x)) = recast_b(edit_b(x))
- edit_b after collect_b collapses to collect_b.
      edit_b(collect_b(x)) = collect_b(x)
- edit_b after recast_b collapses to recast_b.
      edit_b(recast_b(x)) = recast_b(x)
- collect_a is a projection — applying it twice is applying it once.
      collect_a(collect_a(x)) = collect_a(x)
- collect_a leaves empty fixed.
      collect_a(empty) = empty
- collect_a and edit_a may be applied in either order.
      collect_a(edit_a(x)) = edit_a(collect_a(x))
- collect_a and edit_b may be applied in either order.
      collect_a(edit_b(x)) = edit_b(collect_a(x))
- collect_a and collect_b may be applied in either order.
      collect_a(collect_b(x)) = collect_b(collect_a(x))
- collect_a and declare may be applied in either order.
      collect_a(declare(x)) = declare(collect_a(x))
- collect_a and recast_b may be applied in either order.
      collect_a(recast_b(x)) = recast_b(collect_a(x))
- collect_a after add_a collapses to collect_a.
      collect_a(add_a(x)) = collect_a(x)
- collect_a after edit_a collapses to collect_a.
      collect_a(edit_a(x)) = collect_a(x)
- collect_b is a projection — applying it twice is applying it once.
      collect_b(collect_b(x)) = collect_b(x)
- collect_b leaves empty fixed.
      collect_b(empty) = empty
- collect_b and edit_a may be applied in either order.
      collect_b(edit_a(x)) = edit_a(collect_b(x))
- collect_b and edit_b may be applied in either order.
      collect_b(edit_b(x)) = edit_b(collect_b(x))
- collect_b and declare may be applied in either order.
      collect_b(declare(x)) = declare(collect_b(x))
- collect_b and recast_a may be applied in either order.
      collect_b(recast_a(x)) = recast_a(collect_b(x))
- collect_b after add_b collapses to collect_b.
      collect_b(add_b(x)) = collect_b(x)
- collect_b after edit_b collapses to collect_b.
      collect_b(edit_b(x)) = collect_b(x)
- declare is a projection — applying it twice is applying it once.
      declare(declare(x)) = declare(x)
- declare and edit_a may be applied in either order.
      declare(edit_a(x)) = edit_a(declare(x))
- declare and edit_b may be applied in either order.
      declare(edit_b(x)) = edit_b(declare(x))
- declare and recast_a may be applied in either order.
      declare(recast_a(x)) = recast_a(declare(x))
- declare and recast_b may be applied in either order.
      declare(recast_b(x)) = recast_b(declare(x))
- recast_a is a projection — applying it twice is applying it once.
      recast_a(recast_a(x)) = recast_a(x)
- recast_a leaves empty fixed.
      recast_a(empty) = empty
- recast_a and recast_b may be applied in either order.
      recast_a(recast_b(x)) = recast_b(recast_a(x))
- recast_a after edit_a collapses to recast_a.
      recast_a(edit_a(x)) = recast_a(x)
- recast_a after collect_a collapses to collect_a.
      recast_a(collect_a(x)) = collect_a(x)
- recast_b is a projection — applying it twice is applying it once.
      recast_b(recast_b(x)) = recast_b(x)
- recast_b leaves empty fixed.
      recast_b(empty) = empty
- recast_b after edit_b collapses to recast_b.
      recast_b(edit_b(x)) = recast_b(x)
- recast_b after collect_b collapses to collect_b.
      recast_b(collect_b(x)) = collect_b(x)

# operators in no law (where the spec is silent): none — every operator participates in a law
