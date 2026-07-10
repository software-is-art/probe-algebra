# qualify reasons — WHY each module refuses the algebra census: the mechanical blockers,
# per file, derived by the same walk as the qualify census (one rule, two renders). Classes:
# no functions | impl-attached surface only | unit returns | primitive signatures |
# borrowed types | parameterised types | unshaped types | zero-argument constants |
# effectful bodies. The classes are evidence; reading them into value-object debt, missing
# vocabulary, or a principled refusal is the ratification's job. Regenerate with
# `BLESS_REASONS=1 cargo build`.
# 56 files scanned, 3 qualify, 53 refuse.

src/boundary.rs: REFUSES — borrowed types, parameterised types, primitive signatures, unit returns, unshaped types, zero-argument constants
src/discover/agenda.rs: REFUSES — borrowed types, parameterised types
src/discover/architect.rs: REFUSES — borrowed types, effectful bodies, parameterised types, primitive signatures, zero-argument constants
src/discover/arithmetic.rs: REFUSES — borrowed types, parameterised types, primitive signatures
src/discover/bite.rs: REFUSES — impl-attached surface only
src/discover/bridge.rs: REFUSES — borrowed types, parameterised types, zero-argument constants
src/discover/bundle.rs: REFUSES — impl-attached surface only
src/discover/coherence.rs: REFUSES — borrowed types, parameterised types, primitive signatures, zero-argument constants
src/discover/cohesion.rs: REFUSES — borrowed types, primitive signatures, unit returns, zero-argument constants
src/discover/composition.rs: REFUSES — borrowed types, parameterised types, primitive signatures, zero-argument constants
src/discover/date.rs: REFUSES — borrowed types, parameterised types, primitive signatures
src/discover/depend.rs: REFUSES — borrowed types, parameterised types, primitive signatures, unit returns, unshaped types
src/discover/engine.rs: REFUSES — borrowed types, parameterised types, primitive signatures, unit returns, unshaped types
src/discover/expect.rs: REFUSES — borrowed types, parameterised types, primitive signatures
src/discover/fabric.rs: REFUSES — borrowed types, parameterised types, unshaped types, zero-argument constants
src/discover/floor.rs: REFUSES — borrowed types, parameterised types, primitive signatures
src/discover/freeze.rs: REFUSES — borrowed types, primitive signatures
src/discover/gates.rs: REFUSES — borrowed types, primitive signatures
src/discover/genesis.rs: REFUSES — borrowed types, parameterised types, primitive signatures, unshaped types
src/discover/infra.rs: REFUSES — impl-attached surface only
src/discover/judgment.rs: REFUSES — impl-attached surface only
src/discover/layering.rs: REFUSES — borrowed types, parameterised types, primitive signatures, unit returns, zero-argument constants
src/discover/lift.rs: REFUSES — borrowed types, parameterised types, primitive signatures
src/discover/mod.rs: REFUSES — borrowed types, parameterised types, primitive signatures, unshaped types, zero-argument constants
src/discover/mutation.rs: REFUSES — borrowed types, parameterised types, primitive signatures
src/discover/perimeter.rs: REFUSES — impl-attached surface only
src/discover/probes.rs: REFUSES — impl-attached surface only
src/discover/protocol.rs: REFUSES — borrowed types, parameterised types, zero-argument constants
src/discover/relation.rs: REFUSES — impl-attached surface only
src/discover/residue.rs: REFUSES — impl-attached surface only
src/discover/router.rs: REFUSES — borrowed types, parameterised types, zero-argument constants
src/discover/scaffold.rs: REFUSES — borrowed types, parameterised types, primitive signatures, zero-argument constants
src/discover/schemata.rs: REFUSES — impl-attached surface only
src/discover/shape.rs: REFUSES — borrowed types, primitive signatures
src/discover/substrate.rs: REFUSES — impl-attached surface only
src/discover/system.rs: REFUSES — impl-attached surface only
src/discover/watch.rs: REFUSES — borrowed types, parameterised types, primitive signatures, unit returns
src/discover/world.rs: REFUSES — borrowed types, parameterised types
src/gdp.rs: REFUSES — borrowed types, parameterised types, primitive signatures, unshaped types
src/harness.rs: REFUSES — borrowed types, parameterised types, primitive signatures, unit returns, unshaped types, zero-argument constants
src/interp/boundary.rs: REFUSES — impl-attached surface only
src/interp/internal.rs: REFUSES — borrowed types, parameterised types
src/interp/mod.rs: REFUSES — no functions
src/kvstore/internal.rs: REFUSES — borrowed types, parameterised types, unshaped types
src/kvstore/mod.rs: REFUSES — no functions
src/kvstore/probes.rs: REFUSES — borrowed types, parameterised types, primitive signatures, unit returns, unshaped types, zero-argument constants
src/kvstore/store.rs: REFUSES — impl-attached surface only
src/kvstore/theory.rs: REFUSES — borrowed types, parameterised types, primitive signatures, zero-argument constants
src/lib.rs: REFUSES — no functions
src/main.rs: REFUSES — borrowed types, primitive signatures, unit returns, zero-argument constants
src/select/boundary.rs: REFUSES — impl-attached surface only
src/select/internal.rs: REFUSES — borrowed types, parameterised types, primitive signatures
src/select/mod.rs: REFUSES — no functions
