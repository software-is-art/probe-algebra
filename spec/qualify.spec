# qualify census — modules that meet the algebra spec by STRUCTURE: their functions are
# operator-shaped (every argument and the return a bare named value type, no primitives, no
# I/O). Boundary-hood is a COMPUTED property here, not the `boundary.rs` file convention — a
# module qualifies wherever it lives. Regenerate with `BLESS_QUALIFY=1 cargo build`.
# 60 files scanned, 17 qualify.

src/boundary.rs: QUALIFIES — operators [Capability::combine, Capability::join, Provenance::combine] over sorts {Capability, Provenance}
src/capability.rs: QUALIFIES — operators [Audit::declared, Audit::observed, cap_of] over sorts {Audit, Capability, Source}
src/discover/derived.rs: QUALIFIES — operators [join, lift, meet, meet_large, meet_small] over sorts {Large, Small, Tri}
src/discover/freeze.rs: QUALIFIES — operators [Spec::lock] over sorts {Lock, Spec}
src/discover/infra.rs: QUALIFIES — operators [Infra::lock, Infra::register, Infra::requirements] over sorts {Floor, Infra, Lock, Register}
src/discover/modularize.rs: QUALIFIES — operators [both, either, peak, rotate] over sorts {Count, Flag, Spin}
src/discover/mutation.rs: QUALIFIES — operators [MutationReport::lock] over sorts {Lock, MutationReport}
src/discover/perimeter.rs: QUALIFIES — operators [Perimeter::floor, Perimeter::lock, Perimeter::ruleset_lock] over sorts {Floor, Lock, Perimeter}
src/discover/probes.rs: QUALIFIES — operators [ProbeCensus::lock, ProbeCensus::register] over sorts {Lock, ProbeCensus, Register}
src/discover/shape.rs: QUALIFIES — operators [ShapeReport::lock] over sorts {Lock, ShapeReport}
src/discover/substrate.rs: QUALIFIES — operators [Substrate::lock] over sorts {Lock, Substrate}
src/discover/system.rs: QUALIFIES — operators [SystemReport::lock] over sorts {Lock, SystemReport}
src/discover/verbs.rs: QUALIFIES — operators [add_a, add_b, collect_a, collect_b, declare, edit_a, edit_b] over sorts {BundleState}
src/discover/world.rs: QUALIFIES — operators [FakeRemoteStore::snapshot, WorldReport::lock] over sorts {FakeRemoteStore, Lock, State, WorldReport}
src/interp/boundary.rs: QUALIFIES — operators [Bound::new, Env::bind, Expr::bin, Expr::bind, Expr::cond, Expr::var, Int::plus, Int::times, Pos::next] over sorts {Bound, Env, Expr, Ident, Int, Op, Pos}
src/kvstore/store.rs: QUALIFIES — operators [Advance::by, Advance::new, Clock::advanced, Clock::until, Entry::expires_at, Entry::new, Entry::remaining_at, Entry::ttl, Entry::val, Lookup::new, Store::clock, Store::put, Store::tick, Store::view, Ttl::plus, Write::new, Write::ttl, Write::val] over sorts {Advance, Clock, Entry, Key, Lookup, Snapshot, Store, Ttl, Val, Write}
src/select/boundary.rs: QUALIFIES — operators [KillMatrix::select, KillMatrix::uncoverable] over sorts {Cover, KillMatrix}
