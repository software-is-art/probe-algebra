# algebra mutation: stable layout — 9 operator-table mutants, 1 SURVIVED — regenerate via this repo's freeze path; ratify the diff.
#
# Every mutant is a perturbed operator table (a VALUE, not a build), judged by
# re-running discovery: KILLED means the named-law set changed — the committed
# spec would go stale against an implementation with this bug. A SURVIVOR means
# the ratified law language cannot tell the mutant from the real thing on this
# grid — a named degree of freedom (the bias-blindness precedent), to be closed
# by a sharper shape or expectation, or ratified here as a free choice.

- killed    `rename` evaluates as `reorder`
- killed    `rename` evaluates as `theme`
- killed    `rename` returns its first argument unchanged
- killed    `relabel` returns its first argument unchanged
- SURVIVED  `render` becomes undefined everywhere
- killed    `reorder` becomes undefined everywhere
- killed    `theme` becomes undefined everywhere
- killed    `rename` becomes undefined everywhere
- killed    `relabel` becomes undefined everywhere

# deafness floor: 30 constant-return mutants (every operator × every distinct
# output), judged by re-checking the discovered laws — 18 SURVIVED.
- SURVIVED  `render` goes deaf: always Placed([])
- SURVIVED  `render` goes deaf: always Placed([("a", (0, 0))])
- SURVIVED  `render` goes deaf: always Placed([("a", (0, 0)), ("b", (4, 0))])
- SURVIVED  `render` goes deaf: always Placed([("a", (0, 0)), ("b", (0, 2)), ("c", (0, 4))])
- SURVIVED  `render` goes deaf: always Placed([("a", (0, 0)), ("b", (0, 2)), ("c", (4, 2)), ("d", (0, 4))])
- SURVIVED  `render` goes deaf: always Placed([("a", (0, 0)), ("b", (4, 0)), ("c", (0, 2))])
- SURVIVED  `rename` goes deaf: always Placed([])
- SURVIVED  `rename` goes deaf: always Placed([("a", (0, 0))])
- SURVIVED  `rename` goes deaf: always Placed([("a", (0, 0)), ("b", (4, 0))])
- SURVIVED  `rename` goes deaf: always Placed([("a", (0, 0)), ("b", (0, 2)), ("c", (0, 4))])
- SURVIVED  `rename` goes deaf: always Placed([("a", (0, 0)), ("b", (0, 2)), ("c", (4, 2)), ("d", (0, 4))])
- SURVIVED  `rename` goes deaf: always Placed([("a", (0, 0)), ("b", (4, 0)), ("c", (0, 2))])
- SURVIVED  `relabel` goes deaf: always Placed([])
- SURVIVED  `relabel` goes deaf: always Placed([("a", (0, 0))])
- SURVIVED  `relabel` goes deaf: always Placed([("a", (0, 0)), ("b", (4, 0))])
- SURVIVED  `relabel` goes deaf: always Placed([("a", (0, 0)), ("b", (0, 2)), ("c", (0, 4))])
- SURVIVED  `relabel` goes deaf: always Placed([("a", (0, 0)), ("b", (0, 2)), ("c", (4, 2)), ("d", (0, 4))])
- SURVIVED  `relabel` goes deaf: always Placed([("a", (0, 0)), ("b", (4, 0)), ("c", (0, 2))])

# dent sweep: 84 one-point mutants (first 16 grid points per operator,
# 2 wrong outputs per point — a resource bound, not a curated list), judged by
# re-checking the discovered laws — 60 SURVIVED — each an UNPINNED COORDINATE, the exact input a missing probe would constrain.
- SURVIVED  `render` dented at [Placed([])]: Placed([]) -> Placed([("a", (0, 0))])
- SURVIVED  `render` dented at [Placed([])]: Placed([]) -> Placed([("a", (0, 0)), ("b", (4, 0))])
- SURVIVED  `render` dented at [Placed([("a", (0, 0))])]: Placed([("a", (0, 0))]) -> Placed([])
- SURVIVED  `render` dented at [Placed([("a", (0, 0))])]: Placed([("a", (0, 0))]) -> Placed([("a", (0, 0)), ("b", (4, 0))])
- SURVIVED  `render` dented at [Placed([("a", (0, 0)), ("b", (4, 0))])]: Placed([("a", (0, 0)), ("b", (4, 0))]) -> Placed([])
- SURVIVED  `render` dented at [Placed([("a", (0, 0)), ("b", (4, 0))])]: Placed([("a", (0, 0)), ("b", (4, 0))]) -> Placed([("a", (0, 0))])
- SURVIVED  `render` dented at [Placed([("a", (0, 0)), ("b", (0, 2)), ("c", (0, 4))])]: Placed([("a", (0, 0)), ("b", (0, 2)), ("c", (0, 4))]) -> Placed([])
- SURVIVED  `render` dented at [Placed([("a", (0, 0)), ("b", (0, 2)), ("c", (0, 4))])]: Placed([("a", (0, 0)), ("b", (0, 2)), ("c", (0, 4))]) -> Placed([("a", (0, 0))])
- SURVIVED  `render` dented at [Placed([("a", (0, 0)), ("b", (0, 2)), ("c", (4, 2)), ("d", (0, 4))])]: Placed([("a", (0, 0)), ("b", (0, 2)), ("c", (4, 2)), ("d", (0, 4))]) -> Placed([])
- SURVIVED  `render` dented at [Placed([("a", (0, 0)), ("b", (0, 2)), ("c", (4, 2)), ("d", (0, 4))])]: Placed([("a", (0, 0)), ("b", (0, 2)), ("c", (4, 2)), ("d", (0, 4))]) -> Placed([("a", (0, 0))])
- SURVIVED  `render` dented at [Placed([("a", (0, 0)), ("b", (4, 0)), ("c", (0, 2))])]: Placed([("a", (0, 0)), ("b", (4, 0)), ("c", (0, 2))]) -> Placed([])
- SURVIVED  `render` dented at [Placed([("a", (0, 0)), ("b", (4, 0)), ("c", (0, 2))])]: Placed([("a", (0, 0)), ("b", (4, 0)), ("c", (0, 2))]) -> Placed([("a", (0, 0))])
- SURVIVED  `rename` dented at [Placed([]), Swap("a<->b")]: Placed([]) -> Placed([("a", (0, 0))])
- SURVIVED  `rename` dented at [Placed([]), Swap("a<->b")]: Placed([]) -> Placed([("a", (0, 0)), ("b", (4, 0))])
- SURVIVED  `rename` dented at [Placed([]), Swap("b<->c")]: Placed([]) -> Placed([("a", (0, 0))])
- SURVIVED  `rename` dented at [Placed([]), Swap("b<->c")]: Placed([]) -> Placed([("a", (0, 0)), ("b", (4, 0))])
- SURVIVED  `rename` dented at [Placed([("a", (0, 0))]), Swap("a<->b")]: Placed([("b", (0, 0))]) -> Placed([])
- SURVIVED  `rename` dented at [Placed([("a", (0, 0))]), Swap("a<->b")]: Placed([("b", (0, 0))]) -> Placed([("a", (0, 0))])
- SURVIVED  `rename` dented at [Placed([("a", (0, 0))]), Swap("b<->c")]: Placed([("a", (0, 0))]) -> Placed([])
- SURVIVED  `rename` dented at [Placed([("a", (0, 0))]), Swap("b<->c")]: Placed([("a", (0, 0))]) -> Placed([("a", (0, 0)), ("b", (4, 0))])
- SURVIVED  `rename` dented at [Placed([("a", (0, 0)), ("b", (4, 0))]), Swap("a<->b")]: Placed([("a", (0, 0)), ("b", (4, 0))]) -> Placed([])
- SURVIVED  `rename` dented at [Placed([("a", (0, 0)), ("b", (4, 0))]), Swap("a<->b")]: Placed([("a", (0, 0)), ("b", (4, 0))]) -> Placed([("a", (0, 0))])
- SURVIVED  `rename` dented at [Placed([("a", (0, 0)), ("b", (4, 0))]), Swap("b<->c")]: Placed([("a", (0, 0)), ("c", (4, 0))]) -> Placed([])
- SURVIVED  `rename` dented at [Placed([("a", (0, 0)), ("b", (4, 0))]), Swap("b<->c")]: Placed([("a", (0, 0)), ("c", (4, 0))]) -> Placed([("a", (0, 0))])
- SURVIVED  `rename` dented at [Placed([("a", (0, 0)), ("b", (0, 2)), ("c", (0, 4))]), Swap("a<->b")]: Placed([("a", (0, 2)), ("b", (0, 0)), ("c", (0, 4))]) -> Placed([])
- SURVIVED  `rename` dented at [Placed([("a", (0, 0)), ("b", (0, 2)), ("c", (0, 4))]), Swap("a<->b")]: Placed([("a", (0, 2)), ("b", (0, 0)), ("c", (0, 4))]) -> Placed([("a", (0, 0))])
- SURVIVED  `rename` dented at [Placed([("a", (0, 0)), ("b", (0, 2)), ("c", (0, 4))]), Swap("b<->c")]: Placed([("a", (0, 0)), ("b", (0, 4)), ("c", (0, 2))]) -> Placed([])
- SURVIVED  `rename` dented at [Placed([("a", (0, 0)), ("b", (0, 2)), ("c", (0, 4))]), Swap("b<->c")]: Placed([("a", (0, 0)), ("b", (0, 4)), ("c", (0, 2))]) -> Placed([("a", (0, 0))])
- SURVIVED  `rename` dented at [Placed([("a", (0, 0)), ("b", (0, 2)), ("c", (4, 2)), ("d", (0, 4))]), Swap("a<->b")]: Placed([("a", (0, 2)), ("b", (0, 0)), ("c", (4, 2)), ("d", (0, 4))]) -> Placed([])
- SURVIVED  `rename` dented at [Placed([("a", (0, 0)), ("b", (0, 2)), ("c", (4, 2)), ("d", (0, 4))]), Swap("a<->b")]: Placed([("a", (0, 2)), ("b", (0, 0)), ("c", (4, 2)), ("d", (0, 4))]) -> Placed([("a", (0, 0))])
- SURVIVED  `rename` dented at [Placed([("a", (0, 0)), ("b", (0, 2)), ("c", (4, 2)), ("d", (0, 4))]), Swap("b<->c")]: Placed([("a", (0, 0)), ("b", (0, 2)), ("c", (4, 2)), ("d", (0, 4))]) -> Placed([])
- SURVIVED  `rename` dented at [Placed([("a", (0, 0)), ("b", (0, 2)), ("c", (4, 2)), ("d", (0, 4))]), Swap("b<->c")]: Placed([("a", (0, 0)), ("b", (0, 2)), ("c", (4, 2)), ("d", (0, 4))]) -> Placed([("a", (0, 0))])
- SURVIVED  `rename` dented at [Placed([("a", (0, 0)), ("b", (4, 0)), ("c", (0, 2))]), Swap("a<->b")]: Placed([("a", (0, 0)), ("b", (4, 0)), ("c", (0, 2))]) -> Placed([])
- SURVIVED  `rename` dented at [Placed([("a", (0, 0)), ("b", (4, 0)), ("c", (0, 2))]), Swap("a<->b")]: Placed([("a", (0, 0)), ("b", (4, 0)), ("c", (0, 2))]) -> Placed([("a", (0, 0))])
- SURVIVED  `rename` dented at [Placed([("a", (0, 0)), ("b", (4, 0)), ("c", (0, 2))]), Swap("b<->c")]: Placed([("a", (0, 0)), ("b", (0, 2)), ("c", (4, 0))]) -> Placed([])
- SURVIVED  `rename` dented at [Placed([("a", (0, 0)), ("b", (4, 0)), ("c", (0, 2))]), Swap("b<->c")]: Placed([("a", (0, 0)), ("b", (0, 2)), ("c", (4, 0))]) -> Placed([("a", (0, 0))])
- SURVIVED  `relabel` dented at [Placed([]), Swap("a<->b")]: Placed([]) -> Placed([("a", (0, 0))])
- SURVIVED  `relabel` dented at [Placed([]), Swap("a<->b")]: Placed([]) -> Placed([("a", (0, 0)), ("b", (4, 0))])
- SURVIVED  `relabel` dented at [Placed([]), Swap("b<->c")]: Placed([]) -> Placed([("a", (0, 0))])
- SURVIVED  `relabel` dented at [Placed([]), Swap("b<->c")]: Placed([]) -> Placed([("a", (0, 0)), ("b", (4, 0))])
- SURVIVED  `relabel` dented at [Placed([("a", (0, 0))]), Swap("a<->b")]: Placed([("b", (0, 0))]) -> Placed([])
- SURVIVED  `relabel` dented at [Placed([("a", (0, 0))]), Swap("a<->b")]: Placed([("b", (0, 0))]) -> Placed([("a", (0, 0))])
- SURVIVED  `relabel` dented at [Placed([("a", (0, 0))]), Swap("b<->c")]: Placed([("a", (0, 0))]) -> Placed([])
- SURVIVED  `relabel` dented at [Placed([("a", (0, 0))]), Swap("b<->c")]: Placed([("a", (0, 0))]) -> Placed([("a", (0, 0)), ("b", (4, 0))])
- SURVIVED  `relabel` dented at [Placed([("a", (0, 0)), ("b", (4, 0))]), Swap("a<->b")]: Placed([("a", (4, 0)), ("b", (0, 0))]) -> Placed([])
- SURVIVED  `relabel` dented at [Placed([("a", (0, 0)), ("b", (4, 0))]), Swap("a<->b")]: Placed([("a", (4, 0)), ("b", (0, 0))]) -> Placed([("a", (0, 0))])
- SURVIVED  `relabel` dented at [Placed([("a", (0, 0)), ("b", (4, 0))]), Swap("b<->c")]: Placed([("a", (0, 0)), ("c", (4, 0))]) -> Placed([])
- SURVIVED  `relabel` dented at [Placed([("a", (0, 0)), ("b", (4, 0))]), Swap("b<->c")]: Placed([("a", (0, 0)), ("c", (4, 0))]) -> Placed([("a", (0, 0))])
- SURVIVED  `relabel` dented at [Placed([("a", (0, 0)), ("b", (0, 2)), ("c", (0, 4))]), Swap("a<->b")]: Placed([("a", (0, 2)), ("b", (0, 0)), ("c", (0, 4))]) -> Placed([])
- SURVIVED  `relabel` dented at [Placed([("a", (0, 0)), ("b", (0, 2)), ("c", (0, 4))]), Swap("a<->b")]: Placed([("a", (0, 2)), ("b", (0, 0)), ("c", (0, 4))]) -> Placed([("a", (0, 0))])
- SURVIVED  `relabel` dented at [Placed([("a", (0, 0)), ("b", (0, 2)), ("c", (0, 4))]), Swap("b<->c")]: Placed([("a", (0, 0)), ("b", (0, 4)), ("c", (0, 2))]) -> Placed([])
- SURVIVED  `relabel` dented at [Placed([("a", (0, 0)), ("b", (0, 2)), ("c", (0, 4))]), Swap("b<->c")]: Placed([("a", (0, 0)), ("b", (0, 4)), ("c", (0, 2))]) -> Placed([("a", (0, 0))])
- SURVIVED  `relabel` dented at [Placed([("a", (0, 0)), ("b", (0, 2)), ("c", (4, 2)), ("d", (0, 4))]), Swap("a<->b")]: Placed([("a", (0, 2)), ("b", (0, 0)), ("c", (4, 2)), ("d", (0, 4))]) -> Placed([])
- SURVIVED  `relabel` dented at [Placed([("a", (0, 0)), ("b", (0, 2)), ("c", (4, 2)), ("d", (0, 4))]), Swap("a<->b")]: Placed([("a", (0, 2)), ("b", (0, 0)), ("c", (4, 2)), ("d", (0, 4))]) -> Placed([("a", (0, 0))])
- SURVIVED  `relabel` dented at [Placed([("a", (0, 0)), ("b", (0, 2)), ("c", (4, 2)), ("d", (0, 4))]), Swap("b<->c")]: Placed([("a", (0, 0)), ("b", (4, 2)), ("c", (0, 2)), ("d", (0, 4))]) -> Placed([])
- SURVIVED  `relabel` dented at [Placed([("a", (0, 0)), ("b", (0, 2)), ("c", (4, 2)), ("d", (0, 4))]), Swap("b<->c")]: Placed([("a", (0, 0)), ("b", (4, 2)), ("c", (0, 2)), ("d", (0, 4))]) -> Placed([("a", (0, 0))])
- SURVIVED  `relabel` dented at [Placed([("a", (0, 0)), ("b", (4, 0)), ("c", (0, 2))]), Swap("a<->b")]: Placed([("a", (4, 0)), ("b", (0, 0)), ("c", (0, 2))]) -> Placed([])
- SURVIVED  `relabel` dented at [Placed([("a", (0, 0)), ("b", (4, 0)), ("c", (0, 2))]), Swap("a<->b")]: Placed([("a", (4, 0)), ("b", (0, 0)), ("c", (0, 2))]) -> Placed([("a", (0, 0))])
- SURVIVED  `relabel` dented at [Placed([("a", (0, 0)), ("b", (4, 0)), ("c", (0, 2))]), Swap("b<->c")]: Placed([("a", (0, 0)), ("b", (0, 2)), ("c", (4, 0))]) -> Placed([])
- SURVIVED  `relabel` dented at [Placed([("a", (0, 0)), ("b", (4, 0)), ("c", (0, 2))]), Swap("b<->c")]: Placed([("a", (0, 0)), ("b", (0, 2)), ("c", (4, 0))]) -> Placed([("a", (0, 0))])
