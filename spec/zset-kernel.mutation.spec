# algebra mutation: zset kernel — 38 operator-table mutants, all killed — regenerate via this repo's freeze path; ratify the diff.
#
# Every mutant is a perturbed operator table (a VALUE, not a build), judged by
# re-running discovery: KILLED means the named-law set changed — the committed
# spec would go stale against an implementation with this bug. A SURVIVOR means
# the ratified law language cannot tell the mutant from the real thing on this
# grid — a named degree of freedom (the bias-blindness precedent), to be closed
# by a sharper shape or expectation, or ratified here as a free choice.

- killed    `+` evaluates as `zero`
- killed    `+` evaluates as `neg`
- killed    `+` evaluates as `join`
- killed    `+` evaluates as `distinct`
- killed    `neg` evaluates as `zero`
- killed    `neg` evaluates as `distinct`
- killed    `join` evaluates as `zero`
- killed    `join` evaluates as `+`
- killed    `join` evaluates as `neg`
- killed    `join` evaluates as `distinct`
- killed    `distinct` evaluates as `zero`
- killed    `distinct` evaluates as `neg`
- killed    `delay` evaluates as `integrate`
- killed    `delay` evaluates as `differentiate`
- killed    `integrate` evaluates as `delay`
- killed    `integrate` evaluates as `differentiate`
- killed    `differentiate` evaluates as `delay`
- killed    `differentiate` evaluates as `integrate`
- killed    `latest` evaluates as `zero`
- killed    `+` returns its first argument unchanged
- killed    `+` returns its second argument unchanged
- killed    `neg` returns its input unchanged
- killed    `join` returns its first argument unchanged
- killed    `join` returns its second argument unchanged
- killed    `distinct` returns its input unchanged
- killed    `delay` returns its input unchanged
- killed    `integrate` returns its input unchanged
- killed    `differentiate` returns its input unchanged
- killed    `zero` becomes undefined everywhere
- killed    `+` becomes undefined everywhere
- killed    `neg` becomes undefined everywhere
- killed    `join` becomes undefined everywhere
- killed    `distinct` becomes undefined everywhere
- killed    `delay` becomes undefined everywhere
- killed    `integrate` becomes undefined everywhere
- killed    `differentiate` becomes undefined everywhere
- killed    `impulse` becomes undefined everywhere
- killed    `latest` becomes undefined everywhere

# deafness floor: 50 constant-return mutants (every operator × every distinct
# output), judged by re-checking the discovered laws — 3 SURVIVED.
- SURVIVED  `join` goes deaf: always (0, [0, 0, 0])
- SURVIVED  `distinct` goes deaf: always (0, [0, 0, 0])
- SURVIVED  `delay` goes deaf: always (1, [0, 0, 0, 0, 0, 0, 0, 0, 0])

# dent sweep: 140 one-point mutants (first 16 grid points per operator,
# 2 wrong outputs per point — a resource bound, not a curated list), judged by
# re-checking the discovered laws — 17 SURVIVED — each an UNPINNED COORDINATE, the exact input a missing probe would constrain.
- SURVIVED  `distinct` dented at [(0, [-1, 0, 0])]: (0, [0, 0, 0]) -> (0, [1, 0, 0])
- SURVIVED  `distinct` dented at [(0, [-1, 0, 0])]: (0, [0, 0, 0]) -> (0, [-1, 0, 0])
- SURVIVED  `distinct` dented at [(0, [0, 2, 0])]: (0, [0, 1, 0]) -> (0, [0, 0, 0])
- SURVIVED  `distinct` dented at [(0, [0, 2, 0])]: (0, [0, 1, 0]) -> (0, [1, 0, 0])
- SURVIVED  `distinct` dented at [(0, [1, -2, 0])]: (0, [1, 0, 0]) -> (0, [0, 0, 0])
- SURVIVED  `distinct` dented at [(0, [0, 1, 3])]: (0, [0, 1, 1]) -> (0, [0, 0, 0])
- SURVIVED  `distinct` dented at [(0, [0, 1, 3])]: (0, [0, 1, 1]) -> (0, [1, 0, 0])
- SURVIVED  `impulse` dented at [(0, [0, 0, 0])]: (1, [0, 0, 0, 0, 0, 0, 0, 0, 0]) -> (1, [1, 0, 0, 0, 0, 0, 0, 0, 0])
- SURVIVED  `impulse` dented at [(0, [0, 0, 0])]: (1, [0, 0, 0, 0, 0, 0, 0, 0, 0]) -> (1, [0, 0, 0, 0, 2, 0, 0, 0, 0])
- SURVIVED  `latest` dented at [(1, [1, 0, 0, 0, 0, 0, 0, 0, 0])]: (0, [0, 0, 0]) -> (0, [1, 0, 0])
- SURVIVED  `latest` dented at [(1, [1, 0, 0, 0, 0, 0, 0, 0, 0])]: (0, [0, 0, 0]) -> (0, [-1, 0, 0])
- SURVIVED  `latest` dented at [(1, [0, 0, 0, 0, 2, 0, 0, 0, 0])]: (0, [0, 0, 0]) -> (0, [1, 0, 0])
- SURVIVED  `latest` dented at [(1, [0, 0, 0, 0, 2, 0, 0, 0, 0])]: (0, [0, 0, 0]) -> (0, [-1, 0, 0])
- SURVIVED  `latest` dented at [(1, [1, 0, 0, 0, 2, 0, -1, 0, 1])]: (0, [-1, 0, 1]) -> (0, [0, 0, 0])
- SURVIVED  `latest` dented at [(1, [1, 0, 0, 0, 2, 0, -1, 0, 1])]: (0, [-1, 0, 1]) -> (0, [1, 0, 0])
- SURVIVED  `latest` dented at [(1, [-1, 0, 1, 1, 0, 0, 0, 2, 0])]: (0, [0, 2, 0]) -> (0, [0, 0, 0])
- SURVIVED  `latest` dented at [(1, [-1, 0, 1, 1, 0, 0, 0, 2, 0])]: (0, [0, 2, 0]) -> (0, [1, 0, 0])
