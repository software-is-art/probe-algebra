# qualify reasons — WHY each module refuses the algebra census: the mechanical blockers,
# per file, derived by the same walk as the qualify census (one rule, two renders; free
# functions AND impl methods, the receiver resolved to the typestate). Classes:
# no functions | unit returns | primitive signatures | borrowed types | parameterised
# types | unshaped types | zero-argument constants | effectful bodies | mutating
# receivers. The classes are evidence; reading them into value-object debt, missing
# vocabulary, or a principled refusal is the ratification's job. Regenerate with
# `BLESS_REASONS=1 cargo build`.
# 68 files scanned, 18 qualify, 50 refuse.

src/discover/agenda.rs: REFUSES — borrowed types, effectful bodies, parameterised types, primitive signatures, unshaped types
src/discover/architect.rs: REFUSES — borrowed types, effectful bodies, parameterised types, primitive signatures, zero-argument constants
src/discover/arithmetic.rs: REFUSES — borrowed types, parameterised types, primitive signatures
src/discover/attest.rs: REFUSES — borrowed types, effectful bodies, mutating receivers, parameterised types, primitive signatures
src/discover/bite.rs: REFUSES — borrowed types, effectful bodies, parameterised types, unshaped types, zero-argument constants
src/discover/bridge.rs: REFUSES — borrowed types, parameterised types, primitive signatures, zero-argument constants
src/discover/bundle.rs: REFUSES — borrowed types, effectful bodies, parameterised types, primitive signatures, unit returns
src/discover/cli.rs: REFUSES — borrowed types, parameterised types, primitive signatures, zero-argument constants
src/discover/coherence.rs: REFUSES — borrowed types, parameterised types, primitive signatures, zero-argument constants
src/discover/cohesion.rs: REFUSES — borrowed types, primitive signatures, unit returns, zero-argument constants
src/discover/composition.rs: REFUSES — borrowed types, parameterised types, primitive signatures, zero-argument constants
src/discover/date.rs: REFUSES — borrowed types, parameterised types, primitive signatures
src/discover/depend.rs: REFUSES — borrowed types, effectful bodies, parameterised types, primitive signatures, unit returns, unshaped types, zero-argument constants
src/discover/eduction.rs: REFUSES — borrowed types, mutating receivers, parameterised types, primitive signatures, unit returns, unshaped types, zero-argument constants
src/discover/engine.rs: REFUSES — borrowed types, parameterised types, primitive signatures, unit returns, unshaped types, zero-argument constants
src/discover/envelope.rs: REFUSES — borrowed types, parameterised types, primitive signatures, unshaped types
src/discover/expect.rs: REFUSES — borrowed types, parameterised types, primitive signatures, zero-argument constants
src/discover/fabric.rs: REFUSES — borrowed types, parameterised types, unshaped types, zero-argument constants
src/discover/floor.rs: REFUSES — borrowed types, parameterised types, primitive signatures, unshaped types
src/discover/gates.rs: REFUSES — borrowed types, parameterised types, primitive signatures, zero-argument constants
src/discover/genesis.rs: REFUSES — borrowed types, parameterised types, primitive signatures, unshaped types
src/discover/items.rs: REFUSES — borrowed types, effectful bodies, parameterised types
src/discover/judgment.rs: REFUSES — borrowed types, parameterised types, unshaped types
src/discover/layering.rs: REFUSES — borrowed types, parameterised types, primitive signatures, unit returns, zero-argument constants
src/discover/lift.rs: REFUSES — borrowed types, parameterised types, primitive signatures, unit returns, zero-argument constants
src/discover/mod.rs: REFUSES — borrowed types, parameterised types, primitive signatures, unshaped types, zero-argument constants
src/discover/protocol.rs: REFUSES — borrowed types, parameterised types, primitive signatures, zero-argument constants
src/discover/relation.rs: REFUSES — borrowed types, parameterised types, primitive signatures, unshaped types, zero-argument constants
src/discover/residue.rs: REFUSES — borrowed types, parameterised types, primitive signatures, zero-argument constants
src/discover/router.rs: REFUSES — borrowed types, parameterised types, zero-argument constants
src/discover/scaffold.rs: REFUSES — borrowed types, parameterised types, primitive signatures, zero-argument constants
src/discover/schemata.rs: REFUSES — borrowed types, effectful bodies, parameterised types, primitive signatures, unit returns, unshaped types, zero-argument constants
src/discover/squash.rs: REFUSES — borrowed types, parameterised types, primitive signatures, unshaped types
src/discover/store.rs: REFUSES — borrowed types, effectful bodies, parameterised types, primitive signatures
src/discover/trace.rs: REFUSES — borrowed types, parameterised types, primitive signatures
src/discover/verdict.rs: REFUSES — borrowed types, effectful bodies, parameterised types, primitive signatures
src/discover/watch.rs: REFUSES — borrowed types, mutating receivers, parameterised types, primitive signatures, unit returns, unshaped types, zero-argument constants
src/discover/zset.rs: REFUSES — borrowed types, mutating receivers, parameterised types, primitive signatures, unit returns, zero-argument constants
src/gdp.rs: REFUSES — borrowed types, parameterised types, primitive signatures, unshaped types
src/harness.rs: REFUSES — borrowed types, parameterised types, primitive signatures, unit returns, unshaped types, zero-argument constants
src/interp/internal.rs: REFUSES — borrowed types, mutating receivers, parameterised types, primitive signatures
src/interp/mod.rs: REFUSES — no functions
src/kvstore/internal.rs: REFUSES — borrowed types, parameterised types, unshaped types
src/kvstore/mod.rs: REFUSES — no functions
src/kvstore/probes.rs: REFUSES — borrowed types, parameterised types, primitive signatures, unit returns, unshaped types, zero-argument constants
src/kvstore/theory.rs: REFUSES — borrowed types, parameterised types, primitive signatures, zero-argument constants
src/lib.rs: REFUSES — no functions
src/main.rs: REFUSES — borrowed types, primitive signatures, unit returns, zero-argument constants
src/select/internal.rs: REFUSES — borrowed types, parameterised types, primitive signatures
src/select/mod.rs: REFUSES — no functions
