# qualify census — modules that meet the algebra spec by STRUCTURE: their functions are
# operator-shaped (every argument and the return a bare named value type, no primitives, no
# I/O). Boundary-hood is a COMPUTED property here, not the `boundary.rs` file convention — a
# module qualifies wherever it lives. Regenerate with `BLESS_FIXTURE_QUALIFY=1 cargo build`.
# 4 files scanned, 2 qualify.

src/internal.rs: QUALIFIES — operators [grant, renew, spend] over sorts {Credits}
src/ops.rs: QUALIFIES — operators [grant, renew, spend] over sorts {Credits}
