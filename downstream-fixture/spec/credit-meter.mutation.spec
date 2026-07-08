# algebra mutation: credit meter — 19 operator-table mutants, all killed — regenerate via this repo's freeze path; ratify the diff.
#
# Every mutant is a perturbed operator table (a VALUE, not a build), judged by
# re-running discovery: KILLED means the named-law set changed — the committed
# spec would go stale against an implementation with this bug. A SURVIVOR means
# the ratified law language cannot tell the mutant from the real thing on this
# grid — a named degree of freedom (the bias-blindness precedent), to be closed
# by a sharper shape or expectation, or ratified here as a free choice.

- killed    `grant` evaluates as `zero`
- killed    `grant` evaluates as `spend`
- killed    `grant` evaluates as `renew`
- killed    `spend` evaluates as `zero`
- killed    `spend` evaluates as `grant`
- killed    `spend` evaluates as `renew`
- killed    `renew` evaluates as `zero`
- killed    `renew` evaluates as `grant`
- killed    `renew` evaluates as `spend`
- killed    `grant` returns its first argument unchanged
- killed    `grant` returns its second argument unchanged
- killed    `spend` returns its first argument unchanged
- killed    `spend` returns its second argument unchanged
- killed    `renew` returns its first argument unchanged
- killed    `renew` returns its second argument unchanged
- killed    `zero` becomes undefined everywhere
- killed    `grant` becomes undefined everywhere
- killed    `spend` becomes undefined everywhere
- killed    `renew` becomes undefined everywhere

# deafness floor: 63 constant-return mutants (every operator × every distinct
# output), judged by re-checking the discovered laws — all killed: every operator's output provably depends on its input.

# dent sweep: 96 one-point mutants (first 16 grid points per operator,
# 2 wrong outputs per point — a resource bound, not a curated list), judged by
# re-checking the discovered laws — all killed: every sampled point is pinned.
