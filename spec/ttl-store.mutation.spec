# algebra mutation: ttl store — 13 operator-table mutants, all killed — regenerate via this repo's freeze path; ratify the diff.
#
# Every mutant is a perturbed operator table (a VALUE, not a build), judged by
# re-running discovery: KILLED means the named-law set changed — the committed
# spec would go stale against an implementation with this bug. A SURVIVOR means
# the ratified law language cannot tell the mutant from the real thing on this
# grid — a named degree of freedom (the bias-blindness precedent), to be closed
# by a sharper shape or expectation, or ratified here as a free choice.

- killed    `<+` evaluates as `empty`
- killed    `tick` evaluates as `empty`
- killed    `+` evaluates as `zero`
- killed    `<+` returns its first argument unchanged
- killed    `<+` returns its second argument unchanged
- killed    `tick` returns its first argument unchanged
- killed    `+` returns its first argument unchanged
- killed    `+` returns its second argument unchanged
- killed    `empty` becomes undefined everywhere
- killed    `<+` becomes undefined everywhere
- killed    `tick` becomes undefined everywhere
- killed    `zero` becomes undefined everywhere
- killed    `+` becomes undefined everywhere

# deafness floor: 14 constant-return mutants (every operator × every distinct
# output), judged by re-checking the discovered laws — all killed: every operator's output provably depends on its input.

# dent sweep: 96 one-point mutants (first 16 grid points per operator,
# 2 wrong outputs per point — a resource bound, not a curated list), judged by
# re-checking the discovered laws — 9 SURVIVED — each an UNPINNED COORDINATE, the exact input a missing probe would constrain.
- SURVIVED  `<+` dented at [(0, Snapshot([(Key("a"), Val(1), Ttl(2))]), Ttl(0)), (0, Snapshot([]), Ttl(0))]: (0, Snapshot([(Key("a"), Val(1), Ttl(2))]), Ttl(0)) -> (0, Snapshot([]), Ttl(0))
- SURVIVED  `<+` dented at [(0, Snapshot([(Key("a"), Val(1), Ttl(2))]), Ttl(0)), (0, Snapshot([(Key("a"), Val(2), Ttl(5))]), Ttl(0))]: (0, Snapshot([(Key("a"), Val(2), Ttl(5))]), Ttl(0)) -> (0, Snapshot([(Key("a"), Val(1), Ttl(2))]), Ttl(0))
- SURVIVED  `<+` dented at [(0, Snapshot([(Key("a"), Val(2), Ttl(5))]), Ttl(0)), (0, Snapshot([(Key("a"), Val(1), Ttl(2))]), Ttl(0))]: (0, Snapshot([(Key("a"), Val(1), Ttl(2))]), Ttl(0)) -> (0, Snapshot([(Key("a"), Val(2), Ttl(5))]), Ttl(0))
- SURVIVED  `tick` dented at [(0, Snapshot([(Key("a"), Val(1), Ttl(2))]), Ttl(0)), (1, Snapshot([]), Ttl(1))]: (0, Snapshot([(Key("a"), Val(1), Ttl(1))]), Ttl(0)) -> (0, Snapshot([]), Ttl(0))
- SURVIVED  `tick` dented at [(0, Snapshot([(Key("a"), Val(2), Ttl(5))]), Ttl(0)), (1, Snapshot([]), Ttl(5))]: (0, Snapshot([]), Ttl(0)) -> (0, Snapshot([(Key("a"), Val(1), Ttl(2))]), Ttl(0))
- SURVIVED  `tick` dented at [(0, Snapshot([(Key("b"), Val(3), Ttl(3))]), Ttl(0)), (1, Snapshot([]), Ttl(1))]: (0, Snapshot([(Key("b"), Val(3), Ttl(2))]), Ttl(0)) -> (0, Snapshot([]), Ttl(0))
- SURVIVED  `tick` dented at [(0, Snapshot([(Key("b"), Val(3), Ttl(3))]), Ttl(0)), (1, Snapshot([]), Ttl(1))]: (0, Snapshot([(Key("b"), Val(3), Ttl(2))]), Ttl(0)) -> (0, Snapshot([(Key("a"), Val(1), Ttl(2))]), Ttl(0))
- SURVIVED  `tick` dented at [(0, Snapshot([(Key("b"), Val(3), Ttl(3))]), Ttl(0)), (1, Snapshot([]), Ttl(2))]: (0, Snapshot([(Key("b"), Val(3), Ttl(1))]), Ttl(0)) -> (0, Snapshot([]), Ttl(0))
- SURVIVED  `tick` dented at [(0, Snapshot([(Key("b"), Val(3), Ttl(3))]), Ttl(0)), (1, Snapshot([]), Ttl(5))]: (0, Snapshot([]), Ttl(0)) -> (0, Snapshot([(Key("a"), Val(1), Ttl(2))]), Ttl(0))
