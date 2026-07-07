# algebra mutation: fabric — 22 operator-table mutants, all killed — regenerate via this repo's freeze path; ratify the diff.
#
# Every mutant is a perturbed operator table (a VALUE, not a build), judged by
# re-running discovery: KILLED means the named-law set changed — the committed
# spec would go stale against an implementation with this bug. A SURVIVOR means
# the ratified law language cannot tell the mutant from the real thing on this
# grid — a named degree of freedom (the bias-blindness precedent), to be closed
# by a sharper shape or expectation, or ratified here as a free choice.

- killed    `join` evaluates as `mesh`
- killed    `join` evaluates as `reach`
- killed    `grant` evaluates as `mesh`
- killed    `grant` evaluates as `revoke`
- killed    `grant` evaluates as `reach`
- killed    `revoke` evaluates as `mesh`
- killed    `revoke` evaluates as `grant`
- killed    `revoke` evaluates as `reach`
- killed    `reach` evaluates as `mesh`
- killed    `within` evaluates as `true`
- killed    `join` returns its first argument unchanged
- killed    `join` returns its second argument unchanged
- killed    `grant` returns its first argument unchanged
- killed    `revoke` returns its first argument unchanged
- killed    `reach` returns its input unchanged
- killed    `mesh` becomes undefined everywhere
- killed    `join` becomes undefined everywhere
- killed    `grant` becomes undefined everywhere
- killed    `revoke` becomes undefined everywhere
- killed    `reach` becomes undefined everywhere
- killed    `within` becomes undefined everywhere
- killed    `true` becomes undefined everywhere
