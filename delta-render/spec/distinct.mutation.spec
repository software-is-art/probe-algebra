# algebra mutation: distinct — 15 operator-table mutants, all killed — regenerate via this repo's freeze path; ratify the diff.
#
# Every mutant is a perturbed operator table (a VALUE, not a build), judged by
# re-running discovery: KILLED means the named-law set changed — the committed
# spec would go stale against an implementation with this bug. A SURVIVOR means
# the ratified law language cannot tell the mutant from the real thing on this
# grid — a named degree of freedom (the bias-blindness precedent), to be closed
# by a sharper shape or expectation, or ratified here as a free choice.

- killed    `plus` evaluates as `zero`
- killed    `plus` evaluates as `neg`
- killed    `plus` evaluates as `distinct`
- killed    `neg` evaluates as `zero`
- killed    `neg` evaluates as `distinct`
- killed    `distinct` evaluates as `zero`
- killed    `distinct` evaluates as `neg`
- killed    `plus` returns its first argument unchanged
- killed    `plus` returns its second argument unchanged
- killed    `neg` returns its input unchanged
- killed    `distinct` returns its input unchanged
- killed    `zero` becomes undefined everywhere
- killed    `plus` becomes undefined everywhere
- killed    `neg` becomes undefined everywhere
- killed    `distinct` becomes undefined everywhere
