# qualify census — modules that meet the algebra spec by STRUCTURE: their functions are
# operator-shaped (every argument and the return a bare named value type, no primitives, no
# I/O). Boundary-hood is a COMPUTED property here, not the `boundary.rs` file convention — a
# module qualifies wherever it lives. Regenerate with `BLESS_RELAY_APP_QUALIFY=1 cargo build`.
# 7 files scanned, 3 qualify.

src/gauge_internal.rs: QUALIFIES — operators [fuse] over sorts {Level}
src/mixer_internal.rs: QUALIFIES — operators [blend, cook] over sorts {Level, Signal}
src/ops.rs: QUALIFIES — operators [blend, blend, cook, cook, fuse, fuse] over sorts {Level, Signal}
