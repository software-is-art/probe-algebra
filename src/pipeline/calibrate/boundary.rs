//! calibrate::boundary — the first stage's interface (private to `pipeline`).
//!
//! A lossless transport `Sample -> (Reading, Unit)`: it shifts a raw sample onto
//! a calibrated reading. `Reading` is THIS stage's intermediate value object; it
//! is the type the parent must NOT leak. `Sample` is the parent's input type,
//! imported here privately (an import, not a re-export).

use crate::boundary::{Morphism, Unit};
use crate::pipeline::boundary::Sample;

/// The calibrated reading — the intermediate that flows between the two stages
/// and is hidden from everything outside `pipeline`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reading(i64);
impl Reading {
    pub fn new(n: i64) -> Option<Self> {
        if n.abs() <= 2_000_000 {
            Some(Reading(n))
        } else {
            None
        }
    }
    pub fn get(&self) -> i64 {
        self.0
    }
}
crate::value_object!(Reading);

/// The calibration transport. Lossless (residual `Unit`) and invertible.
pub struct Calibrate;
crate::value_operator!(Calibrate);

impl Morphism for Calibrate {
    type In = Sample;
    type Out = Reading;
    type Residual = Unit;

    fn forward(&self, input: &Sample) -> (Reading, Unit) {
        (
            Reading::new(input.get() + 1000).expect("calibrated sample stays in range"),
            Unit,
        )
    }

    fn backward(&self, out: &Reading, _r: &Unit) -> Option<Sample> {
        Sample::new(out.get() - 1000)
    }
}
