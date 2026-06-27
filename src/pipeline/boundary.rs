//! pipeline::boundary — the PARENT module's only public surface.
//!
//! This is the test of recursion. The parent composes two private child stages
//! (`calibrate` then `bucketize`) but exposes exactly ONE operator, `Ingest`,
//! whose `Morphism` signature is `Sample -> (Bucket, Pair<Unit, Leftover>)` —
//! every type in it is owned HERE or by the grammar. The child intermediate
//! `Reading` is consumed internally and never surfaces. The parent re-exports
//! nothing (the build forbids `pub use`); it DEFINES `Sample` / `Bucket` /
//! `Leftover` and an operator that delegates inward.
//!
//! Because `probe` / `run` are altitude-agnostic generics, the parent's composite
//! is probed for residual completeness exactly as a leaf operator is.

use crate::boundary::{Compose, Morphism, Pair, Perturbation, Unit};
use crate::pipeline::bucketize::boundary::Bucketize;
use crate::pipeline::calibrate::boundary::Calibrate;

// ===== value objects (the parent's owned public surface) ==================

/// INPUT: a raw sample reading.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sample(i64);
impl Sample {
    pub fn new(n: i64) -> Option<Self> {
        if n.abs() <= 1_000_000 {
            Some(Sample(n))
        } else {
            None
        }
    }
    pub fn get(&self) -> i64 {
        self.0
    }
}

/// OUTPUT: the ten-unit bucket index a sample fell into.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bucket(i64);
impl Bucket {
    pub fn new(n: i64) -> Option<Self> {
        if n.abs() <= 200_000 {
            Some(Bucket(n))
        } else {
            None
        }
    }
    pub fn get(&self) -> i64 {
        self.0
    }
}

/// RESIDUAL piece: the sub-ten remainder bucketing discarded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Leftover(i64);
impl Leftover {
    pub fn new(n: i64) -> Option<Self> {
        if (0..10).contains(&n) {
            Some(Leftover(n))
        } else {
            None
        }
    }
    pub fn get(&self) -> i64 {
        self.0
    }
}

crate::value_object!(Sample, Bucket, Leftover);

// ===== value operator: the one narrowed surface ==========================

/// Ingest a sample end to end: calibrate, then bucketize. The composite is lossy
/// only in the sub-ten dimension; the composite residual `Pair<Unit, Leftover>`
/// retains it, so `Ingest` round-trips. The intermediate `Reading` lives entirely
/// inside `forward` / `backward` and never appears in the signature.
pub struct Ingest;
crate::value_operator!(Ingest);

impl Morphism for Ingest {
    type In = Sample;
    type Out = Bucket;
    type Residual = Pair<Unit, Leftover>;

    fn forward(&self, input: &Sample) -> (Bucket, Pair<Unit, Leftover>) {
        Compose {
            f: Calibrate,
            g: Bucketize,
        }
        .forward(input)
    }

    fn backward(&self, out: &Bucket, r: &Pair<Unit, Leftover>) -> Option<Sample> {
        Compose {
            f: Calibrate,
            g: Bucketize,
        }
        .backward(out, r)
    }
}

// ===== value operator: a perturbation along the lost dimension ============

/// Nudge a sample by one unit — perturbs the sub-ten dimension that bucketing
/// loses. (Like the ledger's perturbations, it must stay within the lost
/// dimension; callers pick a sample whose bucket does not flip.)
pub struct NudgeSample;
crate::value_operator!(NudgeSample);

impl<M: Morphism<In = Sample>> Perturbation<M> for NudgeSample {
    fn perturb(&self, input: &Sample) -> Option<Sample> {
        Sample::new(input.get() + 1)
    }
}
