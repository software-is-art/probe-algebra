# theory-bridge triage: `bridged-bool` — operator tables exported by an external prover, judged
# by discovery as CONJECTURE SUPPLY and cross-check, never certification. Regenerate via
# `cargo run --example freeze_spec` and ratify the diff.
#
# - an AGREEMENT proves nothing new (the kernel's certificate outranks a grid); it records
#   that the export/bridge pipeline round-tripped.
# - a CONJECTURE is a discovered law with no upstream certificate: a proof obligation,
#   grid evidence only — prove or refute it upstream, then move it to `proved:`.
# - a DISAGREEMENT (a proved law the exhaustive carrier refutes) never renders here: it
#   fails the gate — a defect in the export/bridge pipeline, with certainty.

agreements (proved upstream; the grid could not refute them):
- commutative(and)
- associative(and)
- identity(and, true)
- commutative(or)
- homomorphism(not, and, or)
- homomorphism(not, or, and)
- involution(not)
- commutative(xor)
- identity(or, false)
- self_inverse(xor, false)

conjectures (discovered here; unproved upstream — proof obligations):
- idempotent(and)
- annihilation(and, false)
- inverse(and, not, false)
- distributive(and, or)
- distributive(and, xor)
- absorption(and, or)
- associative(or)
- idempotent(or)
- annihilation(or, true)
- inverse(or, not, true)
- distributive(or, and)
- absorption(or, and)
- associative(xor)
- identity(xor, false)
- inverse(xor, not, true)
