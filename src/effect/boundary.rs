//! effect::boundary — the effect module's interface.
//!
//! VALUE OBJECTS   : Message (pure input), Clock (a fulfilled read / Demand
//!                   result), Stamped (output), Entry (an emission / write),
//!                   Demand (a read request)
//! VALUE OPERATORS : Stamp (a pure Morphism over (Message, Clock)); NudgeReading
//!                   / NudgeMessage (perturbations along the world vs the pure
//!                   input)

use crate::boundary::{Morphism, Pair, Perturbation};

/// A pure caller-supplied input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Message(i64);
impl Message {
    pub fn new(n: i64) -> Option<Self> {
        if n.abs() <= 1_000_000 {
            Some(Message(n))
        } else {
            None
        }
    }
    pub fn get(&self) -> i64 {
        self.0
    }
}

/// A reading from the world (a clock tick) — the value a handler supplies to
/// fulfil a `Demand`. As an INPUT to `Stamp` it is what makes the operator pure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Clock(i64);
impl Clock {
    pub fn new(n: i64) -> Option<Self> {
        if (0..=1_000_000).contains(&n) {
            Some(Clock(n))
        } else {
            None
        }
    }
    pub fn get(&self) -> i64 {
        self.0
    }
}

/// The output: a message stamped with the reading.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stamped(i64);
impl Stamped {
    pub fn new(n: i64) -> Option<Self> {
        if n.abs() <= 2_000_000 {
            Some(Stamped(n))
        } else {
            None
        }
    }
    pub fn get(&self) -> i64 {
        self.0
    }
}

/// An emission: what `Stamp` would WRITE to the world (the tick it logged). As
/// the residual it is the write-side of the effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entry(i64);
impl Entry {
    pub fn new(n: i64) -> Option<Self> {
        if (0..=1_000_000).contains(&n) {
            Some(Entry(n))
        } else {
            None
        }
    }
    pub fn get(&self) -> i64 {
        self.0
    }
}

/// A read request — what the operator declares it needs from the world. A handler
/// fulfils it with a `Clock`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Demand;

crate::value_object!(Message, Clock, Stamped, Entry, Demand);

/// Stamp a message with a clock reading. PURE relative to a handler: the reading
/// arrives as part of the input and the emission leaves as the residual, so this
/// is an ordinary `Morphism` — invertible and probeable.
pub struct Stamp;
crate::value_operator!(Stamp);

impl Morphism for Stamp {
    type In = Pair<Message, Clock>;
    type Out = Stamped;
    type Residual = Entry;

    fn forward(&self, input: &Pair<Message, Clock>) -> (Stamped, Entry) {
        let message = input.0;
        let clock = input.1;
        (
            Stamped::new(message.get() + clock.get()).expect("stamped value in range"),
            Entry::new(clock.get()).expect("the logged tick is a valid entry"),
        )
    }

    fn backward(&self, out: &Stamped, emission: &Entry) -> Option<Pair<Message, Clock>> {
        let clock = Clock::new(emission.get())?;
        let message = Message::new(out.get() - emission.get())?;
        Some(Pair(message, clock))
    }
}

/// An operator that DEMANDS a clock reading but never uses it: declared effectful,
/// actually pure. The capability probe detects this over-declaration — it can move
/// right by dropping the demand.
pub struct IgnoresClock;
crate::value_operator!(IgnoresClock);

impl Morphism for IgnoresClock {
    type In = Pair<Message, Clock>;
    type Out = Stamped;
    type Residual = Entry;

    fn forward(&self, input: &Pair<Message, Clock>) -> (Stamped, Entry) {
        // ignores input.1 (the clock) entirely
        (
            Stamped::new(input.0.get()).expect("message in stamped range"),
            Entry::new(0).expect("zero is a valid entry"),
        )
    }

    fn backward(&self, out: &Stamped, _emission: &Entry) -> Option<Pair<Message, Clock>> {
        Some(Pair(Message::new(out.get())?, Clock::new(0)?))
    }
}

/// Perturb the WORLD reading (the clock) while holding the pure input fixed —
/// used by the capability probe to confirm the output depends on the world.
pub struct NudgeReading;
crate::value_operator!(NudgeReading);

impl<M: Morphism<In = Pair<Message, Clock>>> Perturbation<M> for NudgeReading {
    fn perturb(&self, input: &Pair<Message, Clock>) -> Option<Pair<Message, Clock>> {
        Some(Pair(input.0, Clock::new(input.1.get() + 1)?))
    }
}
