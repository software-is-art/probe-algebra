//! handler — the world-interface for the effect module (a private sibling, not a
//! boundary). A handler fulfils a `Demand` with a reading and absorbs an
//! `Emission`. `Stamp` is pure relative to whichever handler is supplied.
//!
//! `RecordingHandler` scripts the reading and captures emissions as data — pure,
//! deterministic, probeable. A live handler (real clock + real log sink) would
//! impl the same trait and do the actual I/O; it is the single seam the probe
//! method cannot reach (the program edge), which is why it lives outside the pure
//! value layer.

use crate::boundary::{Morphism, Pair};
use crate::effect::boundary::{Clock, Demand, Entry, Message, Stamp, Stamped};

/// The effect interface: read from the world, write to the world.
pub trait Handler {
    fn read(&self, demand: &Demand) -> Clock;
    fn write(&mut self, emission: &Entry);
}

/// A fake handler: it returns a scripted reading and records what was written.
/// Turns the effect into pure data, so the morphism is fully probeable.
pub struct RecordingHandler {
    scripted: Clock,
    written: Vec<Entry>,
}
impl RecordingHandler {
    pub fn new(scripted: Clock) -> Self {
        RecordingHandler {
            scripted,
            written: Vec::new(),
        }
    }
    /// What the run wrote to the world — captured instead of performed.
    pub fn written(&self) -> &[Entry] {
        &self.written
    }
}
impl Handler for RecordingHandler {
    fn read(&self, _demand: &Demand) -> Clock {
        self.scripted
    }
    fn write(&mut self, emission: &Entry) {
        self.written.push(*emission);
    }
}

/// Run `Stamp` relative to a handler: the handler supplies the reading and
/// absorbs the emission; the operator in the middle is pure.
pub fn run_stamp<H: Handler>(handler: &mut H, demand: &Demand, message: &Message) -> Stamped {
    let reading = handler.read(demand);
    let (out, emission) = Stamp.forward(&Pair(*message, reading));
    handler.write(&emission);
    out
}
