# algebra mutation: stream — 33 operator-table mutants, 1 SURVIVED — regenerate via this repo's freeze path; ratify the diff.
#
# Every mutant is a perturbed operator table (a VALUE, not a build), judged by
# re-running discovery: KILLED means the named-law set changed — the committed
# spec would go stale against an implementation with this bug. A SURVIVOR means
# the ratified law language cannot tell the mutant from the real thing on this
# grid — a named degree of freedom (the bias-blindness precedent), to be closed
# by a sharper shape or expectation, or ratified here as a free choice.

- killed    `plus` evaluates as `zero`
- killed    `plus` evaluates as `neg`
- killed    `plus` evaluates as `delay`
- killed    `plus` evaluates as `d`
- killed    `plus` evaluates as `i`
- killed    `neg` evaluates as `zero`
- killed    `neg` evaluates as `delay`
- killed    `neg` evaluates as `d`
- killed    `neg` evaluates as `i`
- SURVIVED  `delay` evaluates as `zero`
- killed    `delay` evaluates as `neg`
- killed    `delay` evaluates as `d`
- killed    `delay` evaluates as `i`
- killed    `d` evaluates as `zero`
- killed    `d` evaluates as `neg`
- killed    `d` evaluates as `delay`
- killed    `d` evaluates as `i`
- killed    `i` evaluates as `zero`
- killed    `i` evaluates as `neg`
- killed    `i` evaluates as `delay`
- killed    `i` evaluates as `d`
- killed    `plus` returns its first argument unchanged
- killed    `plus` returns its second argument unchanged
- killed    `neg` returns its input unchanged
- killed    `delay` returns its input unchanged
- killed    `d` returns its input unchanged
- killed    `i` returns its input unchanged
- killed    `zero` becomes undefined everywhere
- killed    `plus` becomes undefined everywhere
- killed    `neg` becomes undefined everywhere
- killed    `delay` becomes undefined everywhere
- killed    `d` becomes undefined everywhere
- killed    `i` becomes undefined everywhere
