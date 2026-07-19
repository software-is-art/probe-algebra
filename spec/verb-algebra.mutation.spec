# algebra mutation: verb algebra — 100 operator-table mutants, all killed — regenerate via this repo's freeze path; ratify the diff.
#
# Every mutant is a perturbed operator table (a VALUE, not a build), judged by
# re-running discovery: KILLED means the named-law set changed — the committed
# spec would go stale against an implementation with this bug. A SURVIVOR means
# the ratified law language cannot tell the mutant from the real thing on this
# grid — a named degree of freedom (the bias-blindness precedent), to be closed
# by a sharper shape or expectation, or ratified here as a free choice.

- killed    `add_a` evaluates as `empty`
- killed    `add_a` evaluates as `add_b`
- killed    `add_a` evaluates as `edit_a`
- killed    `add_a` evaluates as `edit_b`
- killed    `add_a` evaluates as `collect_a`
- killed    `add_a` evaluates as `collect_b`
- killed    `add_a` evaluates as `declare`
- killed    `add_a` evaluates as `recast_a`
- killed    `add_a` evaluates as `recast_b`
- killed    `add_b` evaluates as `empty`
- killed    `add_b` evaluates as `add_a`
- killed    `add_b` evaluates as `edit_a`
- killed    `add_b` evaluates as `edit_b`
- killed    `add_b` evaluates as `collect_a`
- killed    `add_b` evaluates as `collect_b`
- killed    `add_b` evaluates as `declare`
- killed    `add_b` evaluates as `recast_a`
- killed    `add_b` evaluates as `recast_b`
- killed    `edit_a` evaluates as `empty`
- killed    `edit_a` evaluates as `add_a`
- killed    `edit_a` evaluates as `add_b`
- killed    `edit_a` evaluates as `edit_b`
- killed    `edit_a` evaluates as `collect_a`
- killed    `edit_a` evaluates as `collect_b`
- killed    `edit_a` evaluates as `declare`
- killed    `edit_a` evaluates as `recast_a`
- killed    `edit_a` evaluates as `recast_b`
- killed    `edit_b` evaluates as `empty`
- killed    `edit_b` evaluates as `add_a`
- killed    `edit_b` evaluates as `add_b`
- killed    `edit_b` evaluates as `edit_a`
- killed    `edit_b` evaluates as `collect_a`
- killed    `edit_b` evaluates as `collect_b`
- killed    `edit_b` evaluates as `declare`
- killed    `edit_b` evaluates as `recast_a`
- killed    `edit_b` evaluates as `recast_b`
- killed    `collect_a` evaluates as `empty`
- killed    `collect_a` evaluates as `add_a`
- killed    `collect_a` evaluates as `add_b`
- killed    `collect_a` evaluates as `edit_a`
- killed    `collect_a` evaluates as `edit_b`
- killed    `collect_a` evaluates as `collect_b`
- killed    `collect_a` evaluates as `declare`
- killed    `collect_a` evaluates as `recast_a`
- killed    `collect_a` evaluates as `recast_b`
- killed    `collect_b` evaluates as `empty`
- killed    `collect_b` evaluates as `add_a`
- killed    `collect_b` evaluates as `add_b`
- killed    `collect_b` evaluates as `edit_a`
- killed    `collect_b` evaluates as `edit_b`
- killed    `collect_b` evaluates as `collect_a`
- killed    `collect_b` evaluates as `declare`
- killed    `collect_b` evaluates as `recast_a`
- killed    `collect_b` evaluates as `recast_b`
- killed    `declare` evaluates as `empty`
- killed    `declare` evaluates as `add_a`
- killed    `declare` evaluates as `add_b`
- killed    `declare` evaluates as `edit_a`
- killed    `declare` evaluates as `edit_b`
- killed    `declare` evaluates as `collect_a`
- killed    `declare` evaluates as `collect_b`
- killed    `declare` evaluates as `recast_a`
- killed    `declare` evaluates as `recast_b`
- killed    `recast_a` evaluates as `empty`
- killed    `recast_a` evaluates as `add_a`
- killed    `recast_a` evaluates as `add_b`
- killed    `recast_a` evaluates as `edit_a`
- killed    `recast_a` evaluates as `edit_b`
- killed    `recast_a` evaluates as `collect_a`
- killed    `recast_a` evaluates as `collect_b`
- killed    `recast_a` evaluates as `declare`
- killed    `recast_a` evaluates as `recast_b`
- killed    `recast_b` evaluates as `empty`
- killed    `recast_b` evaluates as `add_a`
- killed    `recast_b` evaluates as `add_b`
- killed    `recast_b` evaluates as `edit_a`
- killed    `recast_b` evaluates as `edit_b`
- killed    `recast_b` evaluates as `collect_a`
- killed    `recast_b` evaluates as `collect_b`
- killed    `recast_b` evaluates as `declare`
- killed    `recast_b` evaluates as `recast_a`
- killed    `add_a` returns its input unchanged
- killed    `add_b` returns its input unchanged
- killed    `edit_a` returns its input unchanged
- killed    `edit_b` returns its input unchanged
- killed    `collect_a` returns its input unchanged
- killed    `collect_b` returns its input unchanged
- killed    `declare` returns its input unchanged
- killed    `recast_a` returns its input unchanged
- killed    `recast_b` returns its input unchanged
- killed    `empty` becomes undefined everywhere
- killed    `add_a` becomes undefined everywhere
- killed    `add_b` becomes undefined everywhere
- killed    `edit_a` becomes undefined everywhere
- killed    `edit_b` becomes undefined everywhere
- killed    `collect_a` becomes undefined everywhere
- killed    `collect_b` becomes undefined everywhere
- killed    `declare` becomes undefined everywhere
- killed    `recast_a` becomes undefined everywhere
- killed    `recast_b` becomes undefined everywhere

# deafness floor: 216 constant-return mutants (every operator × every distinct
# output), judged by re-checking the discovered laws — all killed: every operator's output provably depends on its input.

# dent sweep: 288 one-point mutants (first 16 grid points per operator,
# 2 wrong outputs per point — a resource bound, not a curated list), judged by
# re-checking the discovered laws — all killed: every sampled point is pinned.
