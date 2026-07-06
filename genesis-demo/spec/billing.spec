# discovered spec: billing — a behaviour lock; regenerate via this repo's freeze path and ratify the diff.

- charge applications commute — the parameter order doesn't matter.
      ((x charge x) charge y) = ((x charge y) charge x)
- charge actually acts — some parameter moves some value.
      (x charge x) ≠ x

# operators in no law (where the spec is silent): none — every operator participates in a law
