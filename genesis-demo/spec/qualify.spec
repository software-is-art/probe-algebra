# qualify census — modules that meet the algebra spec by STRUCTURE: their functions are
# operator-shaped (every argument and the return a bare named value type, no primitives, no
# I/O). Boundary-hood is a COMPUTED property here, not the `boundary.rs` file convention — a
# module qualifies wherever it lives. Regenerate with `BLESS_CREDIT_APP_QUALIFY=1 cargo build`.
# 8 files scanned, 5 qualify.

src/billing.rs: QUALIFIES — operators [Credits::charge] over sorts {Credits, Receipt}
src/billing_internal.rs: QUALIFIES — operators [charge] over sorts {Credits, Receipt}
src/meter.rs: QUALIFIES — operators [Credits::grant, Credits::renew] over sorts {Credits}
src/meter_internal.rs: QUALIFIES — operators [grant, renew] over sorts {Credits}
src/ops.rs: QUALIFIES — operators [charge, grant, renew] over sorts {Credits, Receipt}
