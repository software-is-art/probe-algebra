//! bucketize::boundary — the second stage's interface (private to `pipeline`).
//!
//! A LOSSY reduction `Reading -> (Bucket, Leftover)`: it floors a reading to a
//! ten-unit bucket, keeping the sub-ten remainder as the residual. It consumes
//! the sibling stage's `Reading` and produces the PARENT's `Bucket` / `Leftover`
//! value objects — all by private import, none re-exported.

use crate::boundary::{Lossy, Morphism};
use crate::pipeline::boundary::{Bucket, Leftover};
use crate::pipeline::calibrate::boundary::Reading;

/// The bucketing reduction. Lossy in the sub-ten dimension; the residual
/// (`Leftover`) captures it completely, so the reduction round-trips.
pub struct Bucketize;
crate::value_operator!(Bucketize);

impl Morphism for Bucketize {
    type Capability = Lossy;

    type In = Reading;
    type Out = Bucket;
    type Residual = Leftover;

    fn forward(&self, input: &Reading) -> (Bucket, Leftover) {
        let v = input.get();
        (
            Bucket::new(v.div_euclid(10)).expect("bucket index stays in range"),
            Leftover::new(v.rem_euclid(10)).expect("a remainder mod ten is a valid leftover"),
        )
    }

    fn backward(&self, out: &Bucket, r: &Leftover) -> Option<Reading> {
        Reading::new(out.get() * 10 + r.get())
    }
}
