//! journal::boundary — the state module's interface.
//!
//! VALUE OBJECTS   : Register (the state)
//! VALUE OPERATORS : Add, Set (state updates as Morphisms);
//!                   SetForgetsPrior (a buggy Set whose residual is incomplete);
//!                   NudgeState (a perturbation along the carried-state dimension)

use crate::boundary::{Capability, Morphism, Perturbation, Unit};

// ===== value object: the state ===========================================

/// A register holding integer state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Register(i64);
impl Register {
    pub fn new(n: i64) -> Option<Self> {
        if n.abs() <= 1_000_000 {
            Some(Register(n))
        } else {
            None
        }
    }
    pub fn get(&self) -> i64 {
        self.0
    }
    pub fn zero() -> Self {
        Register(0)
    }
    pub fn plus(self, delta: Register) -> Self {
        Register(self.0 + delta.0)
    }
    pub fn minus(self, delta: Register) -> Self {
        Register(self.0 - delta.0)
    }
}
crate::value_object!(Register);

// ===== value operators: state updates ====================================

/// Add a fixed amount to the state. Forgets NOTHING — invertible from its own
/// amount — so its residual is `Unit`. (A read-modify-write that depends on the
/// prior state but does not lose it.)
pub struct Add {
    amount: Register,
}
impl Add {
    pub fn by(amount: Register) -> Self {
        Add { amount }
    }
}
crate::value_operator!(Add);

impl Morphism for Add {
    const CAPABILITY: Capability = Capability::Stateful;

    type In = Register;
    type Out = Register;
    type Residual = Unit;

    fn forward(&self, prior: &Register) -> (Register, Unit) {
        (prior.plus(self.amount), Unit)
    }

    fn backward(&self, next: &Register, _r: &Unit) -> Option<Register> {
        Some(next.minus(self.amount))
    }
}

/// Overwrite the state with a fixed value. This FORGETS the prior, so to invert
/// it the prior must be retained: the residual IS the overwritten prior state.
/// That is the whole claim — a state overwrite is a lossy morphism whose residual
/// is exactly what it forgot.
pub struct Set {
    to: Register,
}
impl Set {
    pub fn to(value: Register) -> Self {
        Set { to: value }
    }
}
crate::value_operator!(Set);

impl Morphism for Set {
    const CAPABILITY: Capability = Capability::Stateful;

    type In = Register;
    type Out = Register;
    type Residual = Register;

    fn forward(&self, prior: &Register) -> (Register, Register) {
        (self.to, *prior) // residual = the prior we overwrote
    }

    fn backward(&self, _next: &Register, prior: &Register) -> Option<Register> {
        Some(*prior)
    }
}

/// A BUGGY `Set`, type-identical to `Set`, whose residual forgets the prior (it
/// records `zero` instead). It cannot rewind, so the probe catches it — the state
/// analogue of `AggregateDropsAmounts`.
pub struct SetForgetsPrior {
    to: Register,
}
impl SetForgetsPrior {
    pub fn to(value: Register) -> Self {
        SetForgetsPrior { to: value }
    }
}
crate::value_operator!(SetForgetsPrior);

impl Morphism for SetForgetsPrior {
    const CAPABILITY: Capability = Capability::Stateful;

    type In = Register;
    type Out = Register;
    type Residual = Register;

    fn forward(&self, _prior: &Register) -> (Register, Register) {
        (self.to, Register::zero()) // BUG: drops the prior
    }

    fn backward(&self, _next: &Register, residual: &Register) -> Option<Register> {
        Some(*residual)
    }
}

/// Perturb the carried state by one unit — moves along the dimension a `Set`
/// loses, so the probe can check the residual records it.
pub struct NudgeState;
crate::value_operator!(NudgeState);

impl<M: Morphism<In = Register>> Perturbation<M> for NudgeState {
    fn perturb(&self, state: &Register) -> Option<Register> {
        Some(state.plus(Register::new(1).expect("one is a valid register value")))
    }
}
