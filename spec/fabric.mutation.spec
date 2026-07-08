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

# deafness floor: 26 constant-return mutants (every operator × every distinct
# output), judged by re-checking the discovered laws — 5 SURVIVED.
- SURVIVED  `grant` goes deaf: always (0, [(0, 1), (1, 2)], [])
- SURVIVED  `revoke` goes deaf: always (0, [], [])
- SURVIVED  `revoke` goes deaf: always (0, [(0, 1)], [(0, 1)])
- SURVIVED  `revoke` goes deaf: always (0, [], [(1, 2)])
- SURVIVED  `reach` goes deaf: always (0, [], [])

# dent sweep: 124 one-point mutants (first 16 grid points per operator,
# 2 wrong outputs per point — a resource bound, not a curated list), judged by
# re-checking the discovered laws — 16 SURVIVED — each an UNPINNED COORDINATE, the exact input a missing probe would constrain.
- SURVIVED  `grant` dented at [(0, [], []), (1, [(0, 2)], [])]: (0, [(0, 2)], []) -> (0, [], [])
- SURVIVED  `grant` dented at [(0, [], []), (1, [(1, 0)], [])]: (0, [(1, 0)], []) -> (0, [], [])
- SURVIVED  `revoke` dented at [(0, [], []), (1, [(0, 2)], [])]: (0, [], [(0, 2)]) -> (0, [], [])
- SURVIVED  `revoke` dented at [(0, [], []), (1, [(1, 0)], [])]: (0, [], [(1, 0)]) -> (0, [], [])
- SURVIVED  `reach` dented at [(0, [(0, 1)], [(0, 1)])]: (0, [], [(0, 1)]) -> (0, [], [])
- SURVIVED  `reach` dented at [(0, [], [(1, 2)])]: (0, [], [(1, 2)]) -> (0, [], [])
- SURVIVED  `within` dented at [(0, [], []), (0, [(1, 2)], [])]: (2, [(1, 1)], []) -> (2, [], [])
- SURVIVED  `within` dented at [(0, [], []), (0, [(0, 1), (1, 2)], [])]: (2, [(1, 1)], []) -> (2, [], [])
- SURVIVED  `within` dented at [(0, [], []), (0, [(0, 1)], [(0, 1)])]: (2, [(1, 1)], []) -> (2, [], [])
- SURVIVED  `within` dented at [(0, [], []), (0, [], [(1, 2)])]: (2, [(1, 1)], []) -> (2, [], [])
- SURVIVED  `within` dented at [(0, [(0, 1)], []), (0, [], [])]: (2, [], []) -> (2, [(1, 1)], [])
- SURVIVED  `within` dented at [(0, [(0, 1)], []), (0, [(1, 2)], [])]: (2, [], []) -> (2, [(1, 1)], [])
- SURVIVED  `within` dented at [(0, [(0, 1)], []), (0, [(0, 1)], [(0, 1)])]: (2, [], []) -> (2, [(1, 1)], [])
- SURVIVED  `within` dented at [(0, [(0, 1)], []), (0, [], [(1, 2)])]: (2, [], []) -> (2, [(1, 1)], [])
- SURVIVED  `within` dented at [(0, [(1, 2)], []), (0, [], [])]: (2, [], []) -> (2, [(1, 1)], [])
- SURVIVED  `within` dented at [(0, [(1, 2)], []), (0, [(0, 1)], [])]: (2, [], []) -> (2, [(1, 1)], [])
