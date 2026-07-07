# algebra mutation: bridged-bool — 32 operator-table mutants, all killed — regenerate via this repo's freeze path; ratify the diff.
#
# Every mutant is a perturbed operator table (a VALUE, not a build), judged by
# re-running discovery: KILLED means the named-law set changed — the committed
# spec would go stale against an implementation with this bug. A SURVIVOR means
# the ratified law language cannot tell the mutant from the real thing on this
# grid — a named degree of freedom (the bias-blindness precedent), to be closed
# by a sharper shape or expectation, or ratified here as a free choice.

- killed    `false` evaluates as `true`
- killed    `true` evaluates as `false`
- killed    `not` evaluates as `false`
- killed    `not` evaluates as `true`
- killed    `and` evaluates as `false`
- killed    `and` evaluates as `true`
- killed    `and` evaluates as `not`
- killed    `and` evaluates as `or`
- killed    `and` evaluates as `xor`
- killed    `or` evaluates as `false`
- killed    `or` evaluates as `true`
- killed    `or` evaluates as `not`
- killed    `or` evaluates as `and`
- killed    `or` evaluates as `xor`
- killed    `xor` evaluates as `false`
- killed    `xor` evaluates as `true`
- killed    `xor` evaluates as `not`
- killed    `xor` evaluates as `and`
- killed    `xor` evaluates as `or`
- killed    `not` returns its input unchanged
- killed    `and` returns its first argument unchanged
- killed    `and` returns its second argument unchanged
- killed    `or` returns its first argument unchanged
- killed    `or` returns its second argument unchanged
- killed    `xor` returns its first argument unchanged
- killed    `xor` returns its second argument unchanged
- killed    `false` becomes undefined everywhere
- killed    `true` becomes undefined everywhere
- killed    `not` becomes undefined everywhere
- killed    `and` becomes undefined everywhere
- killed    `or` becomes undefined everywhere
- killed    `xor` becomes undefined everywhere
