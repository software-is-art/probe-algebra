/-
ProbeBool — the bridged Boolean fragment, formalised: this corpus IS the upstream
prover behind `spec/bridged-bool.export`.

The loop it closes: `discover::bridge` runs discovery over the exported tables as
CONJECTURE SUPPLY; the conjectures worth certifying get proved HERE; each proof is
annotated `-- certifies: <claim>` and its claim moves into the export's `proved:`
lines, shrinking the obligations artifact. The correspondence is gated from the Rust
side (`discover::bite` probes hold tables ↔ export and certifies ↔ proved:, no Lean
needed), and the proofs re-check in CI.

STATEMENT BITES (the mutation gate this corpus is judged by): everything above the
`-- ===== THEOREMS` marker is the DEFINITIONS region. The bite harness flips one
result literal at a time — a definition mutant, never a proof mutant — and demands
the theorems fail to re-check. A surviving bite means no theorem depended on that
degree of freedom: the vacuous-statement finding a kernel cannot make about itself.
Survivors are ratified by key in `lean/bites.register`, with justifications.

One finding is PLANTED: `bnand` is defined and deliberately untheoremed, so its four
bites survive on every run — the register carries them as the debt they are, and the
harness's survivor arm is exercised on every regeneration instead of trusted.

Core Lean only: no imports, no lake, no mathlib — `lean ProbeBool.lean` is the whole
re-check. The nullary exported operators (`false`, `true`) are Bool's own literals.
-/

namespace ProbeBool

-- ===== DEFINITIONS (the statement-bite region: mutants flip result literals below,
-- ===== and only below — the theorems after the marker are never touched) =====

def bnot : Bool → Bool
  | true => false
  | false => true

def band : Bool → Bool → Bool
  | true, true => true
  | true, false => false
  | false, true => false
  | false, false => false

def bor : Bool → Bool → Bool
  | true, true => true
  | true, false => true
  | false, true => true
  | false, false => false

def bxor : Bool → Bool → Bool
  | true, true => false
  | true, false => true
  | false, true => true
  | false, false => false

-- the planted vacuous-statement finding: defined, exported nowhere, theoremed nowhere.
def bnand : Bool → Bool → Bool
  | true, true => false
  | true, false => true
  | false, true => true
  | false, false => true

-- ===== THEOREMS (each `certifies:` annotation is one `proved:` line in
-- ===== spec/bridged-bool.export — the bijection is gated; a theorem about an
-- ===== exported operator must export its certificate, or it hides one) =====

-- certifies: commutative and
theorem and_comm : ∀ x y, band x y = band y x := by decide

-- certifies: associative and
theorem and_assoc : ∀ x y z, band (band x y) z = band x (band y z) := by decide

-- certifies: identity and true
theorem and_id : ∀ x, band true x = x := by decide

-- certifies: commutative or
theorem or_comm : ∀ x y, bor x y = bor y x := by decide

-- certifies: homomorphism not and or
theorem demorgan_and : ∀ x y, bnot (band x y) = bor (bnot x) (bnot y) := by decide

-- certifies: homomorphism not or and
theorem demorgan_or : ∀ x y, bnot (bor x y) = band (bnot x) (bnot y) := by decide

-- certifies: involution not
theorem not_involution : ∀ x, bnot (bnot x) = x := by decide

-- certifies: commutative xor
theorem xor_comm : ∀ x y, bxor x y = bxor y x := by decide

-- certifies: identity or false
theorem or_id : ∀ x, bor false x = x := by decide

-- certifies: self_inverse xor false
theorem xor_self : ∀ x, bxor x x = false := by decide

end ProbeBool
