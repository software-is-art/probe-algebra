# qualify census — modules that meet the algebra spec by STRUCTURE: their functions are
# operator-shaped (every argument and the return a bare named value type, no primitives, no
# I/O). Boundary-hood is a COMPUTED property here, not the `boundary.rs` file convention — a
# module qualifies wherever it lives. Regenerate with `BLESS_FIXTURE_QUALIFY=1 cargo build`.
# 4 files scanned, 3 qualify.

src/internal.rs: QUALIFIES — operators [grant, renew, spend] over sorts {Credits}
src/meter.rs: QUALIFIES — operators [Credits::grant, Credits::renew, Credits::spend, Order::balance, Order::new, Order::purchase, Purchase::amount, Purchase::of] over sorts {Credits, Order, Purchase}
src/ops.rs: QUALIFIES — operators [grant, renew, spend] over sorts {Credits}
