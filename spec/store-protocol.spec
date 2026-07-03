# discovered spec: store protocol — a behaviour lock; regenerate via this repo's freeze path and ratify the diff.

- With ++, the grouping of three values doesn't matter.
      ((x ++ y) ++ z) = (x ++ (y ++ z))
- ++ of a value with itself gives that value.
      (x ++ x) = x
- With ++, the later operand wins where the two disagree — re-applying an earlier one cannot overwrite it.
      ((x ++ y) ++ x) = (y ++ x)
- ++ with empty leaves a value unchanged.
      (empty ++ x) = x

# operators in no law (where the spec is silent): none — every operator participates in a law
