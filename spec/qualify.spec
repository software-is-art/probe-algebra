# qualify census — modules that meet the algebra spec by STRUCTURE: their functions are
# operator-shaped (every argument and the return a bare named value type, no primitives, no
# I/O). Boundary-hood is a COMPUTED property here, not the `boundary.rs` file convention — a
# module qualifies wherever it lives. Regenerate with `BLESS_QUALIFY=1 cargo build`.
# 53 files scanned, 3 qualify.

src/capability.rs: QUALIFIES — operators [cap_of] over sorts {Capability, Source}
src/discover/derived.rs: QUALIFIES — operators [join, lift, meet, meet_large, meet_small] over sorts {Large, Small, Tri}
src/discover/modularize.rs: QUALIFIES — operators [both, either, peak, rotate] over sorts {Count, Flag, Spin}
