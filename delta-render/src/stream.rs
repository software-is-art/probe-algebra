//! stream — depth-bounded streams of Z-sets: `delay`, differentiation, integration.
//!
//! A [`Stream`] is a FIXED-DEPTH vector of Z-sets (`DEPTH` = 5): slot `t` is what
//! arrived at tick `t`. Equality is prefix equality to that depth — a deliberate
//! bounded-refutation concession, consistent with the engine's honest frame (grids are
//! bounded, batteries are samples); v1 builds no lazy/coinductive carrier.
//!
//! Streams inherit the group pointwise (`plus`/`neg`/`zero`), and add the three
//! operators the incremental story runs on:
//!
//! - `delay` (z⁻¹): prepend the group zero, truncate — one tick of memory;
//! - `d` (differentiation): `s − delay(s)` — what CHANGED each tick;
//! - `i` (integration): prefix sums — the state a change history accumulates to.
//!
//! The load-bearing laws are discovered, not asserted: `i undoes d` AND `d undoes i`
//! (the round-trip pair), and all three operators are LINEAR (additive homomorphism +
//! zero fixed point) — which is exactly why `Q^Δ = D ∘ Q ∘ I` is correct for ANY `Q`
//! and why linear operators commute with the whole apparatus. `spec/stream.spec` is
//! the lock.

use crate::zset::{Row, ZSet};

/// The declared depth bound: every stream is judged to exactly this many ticks.
pub const DEPTH: usize = 5;

/// A depth-[`DEPTH`] stream of Z-sets. Canonical by construction: always exactly
/// `DEPTH` slots (constructors pad with the group zero and truncate).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Stream(Vec<ZSet>);

impl Stream {
    /// The zero stream — the group identity (all slots empty).
    pub fn zero() -> Stream {
        Stream(vec![ZSet::empty(); DEPTH])
    }

    /// Build from up to [`DEPTH`] ticks: shorter inputs pad with zero, longer truncate —
    /// the one mint, so a wrong-depth stream is unconstructible.
    pub fn of(ticks: &[ZSet]) -> Stream {
        let mut v: Vec<ZSet> = ticks.iter().take(DEPTH).cloned().collect();
        v.resize(DEPTH, ZSet::empty());
        Stream(v)
    }

    /// Pointwise group sum.
    pub fn plus(&self, other: &Stream) -> Stream {
        Stream(
            self.0
                .iter()
                .zip(&other.0)
                .map(|(a, b)| a.plus(b))
                .collect(),
        )
    }

    /// Pointwise group inverse.
    pub fn neg(&self) -> Stream {
        Stream(self.0.iter().map(ZSet::neg).collect())
    }

    /// z⁻¹ — one tick of memory: prepend zero, truncate to depth.
    pub fn delay(&self) -> Stream {
        let mut v = vec![ZSet::empty()];
        v.extend(self.0.iter().take(DEPTH - 1).cloned());
        Stream(v)
    }

    /// Differentiation: `s − delay(s)` — slot `t` becomes what CHANGED at tick `t`.
    pub fn differentiate(&self) -> Stream {
        self.plus(&self.delay().neg())
    }

    /// Integration: prefix sums — slot `t` becomes everything that arrived up to `t`.
    pub fn integrate(&self) -> Stream {
        let mut acc = ZSet::empty();
        Stream(
            self.0
                .iter()
                .map(|z| {
                    acc = acc.plus(z);
                    acc.clone()
                })
                .collect(),
        )
    }

    /// The tick at `t` (zero beyond the depth — absence is the group zero).
    pub fn at(&self, t: usize) -> ZSet {
        self.0.get(t).cloned().unwrap_or_else(ZSet::empty)
    }

    /// The ticks, exactly [`DEPTH`] of them — the observation the engine fingerprints,
    /// and the sanctioned exit hatch.
    pub fn ticks(&self) -> Vec<ZSet> {
        self.0.clone()
    }
}

// ===== the theory: the stream calculus, judged exhaustively ================

/// The stream calculus's marker — `spec/stream.spec` is its frozen law set.
pub struct StreamCalculus;

/// The one sort.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct St;

fn op_zero(_: &[Stream]) -> Option<Stream> {
    Some(Stream::zero())
}
fn op_plus(v: &[Stream]) -> Option<Stream> {
    Some(v[0].plus(&v[1]))
}
fn op_neg(v: &[Stream]) -> Option<Stream> {
    Some(v[0].neg())
}
fn op_delay(v: &[Stream]) -> Option<Stream> {
    Some(v[0].delay())
}
fn op_d(v: &[Stream]) -> Option<Stream> {
    Some(v[0].differentiate())
}
fn op_i(v: &[Stream]) -> Option<Stream> {
    Some(v[0].integrate())
}

/// The deliberate stream grid: the zero stream, an impulse at the first tick and a LATE
/// one (delay must be visible), a retraction history (+1 then −1 — the inverse in
/// time), a ramp (integration must pile it up), and a two-row mixed history. Six
/// inhabitants, judged exhaustively — the grid is kept lean on purpose: the drift gate
/// re-derives this theory (and its whole mutation verdict) inside every `cargo test`,
/// so grid size is a declared economics decision, not an accident.
///
/// Public on purpose: the END GATE (`I ∘ Q^Δ ∘ D = Q`) is judged over exactly this
/// grid, so the gate and the discovered laws share one space.
pub fn grid() -> Vec<Stream> {
    let a = Row::new(0);
    let b = Row::new(1);
    let z = |pairs: &[(Row, i64)]| ZSet::of(pairs);
    let e = ZSet::empty();
    vec![
        Stream::zero(),
        Stream::of(&[z(&[(a, 1)])]),
        Stream::of(&[e.clone(), e.clone(), z(&[(a, 1)])]),
        Stream::of(&[z(&[(a, 1)]), z(&[(a, -1)])]),
        Stream::of(&[z(&[(a, 1)]), z(&[(a, 2)]), z(&[(a, 3)])]),
        Stream::of(&[z(&[(a, 1), (b, 1)]), e.clone(), z(&[(b, -2)])]),
    ]
}

impl boundary_spec::discover::engine::Theory for StreamCalculus {
    type Sort = St;
    type Value = Stream;
    type Obs = Vec<Vec<(Row, i64)>>;

    fn name() -> &'static str {
        "stream"
    }
    fn operators() -> Vec<boundary_spec::discover::engine::Operator<Self>> {
        use boundary_spec::discover::engine::{Fixity, Operator};
        vec![
            Operator {
                name: "zero",
                symbol: "zero",
                fixity: Fixity::Nullary,
                inputs: vec![],
                output: St,
                eval: op_zero,
            },
            Operator {
                name: "plus",
                symbol: "plus",
                fixity: Fixity::Infix,
                inputs: vec![St, St],
                output: St,
                eval: op_plus,
            },
            Operator {
                name: "neg",
                symbol: "neg",
                fixity: Fixity::Prefix,
                inputs: vec![St],
                output: St,
                eval: op_neg,
            },
            Operator {
                name: "delay",
                symbol: "delay",
                fixity: Fixity::Prefix,
                inputs: vec![St],
                output: St,
                eval: op_delay,
            },
            Operator {
                name: "d",
                symbol: "d",
                fixity: Fixity::Prefix,
                inputs: vec![St],
                output: St,
                eval: op_d,
            },
            Operator {
                name: "i",
                symbol: "i",
                fixity: Fixity::Prefix,
                inputs: vec![St],
                output: St,
                eval: op_i,
            },
        ]
    }
    fn inhabitants(_: St) -> Vec<Stream> {
        grid()
    }
    fn sort_of(_: &Stream) -> St {
        St
    }
    fn observe(v: &Stream) -> Vec<Vec<(Row, i64)>> {
        v.ticks().iter().map(ZSet::entries).collect()
    }
    fn sort_vars(_: St) -> &'static [&'static str] {
        &["s", "t", "u"]
    }
    fn grid_size() -> usize {
        216 // = 6³, the whole space: the calculus is judged exhaustively.
    }
}

#[cfg(test)]
mod probes {
    use super::*;

    fn z(pairs: &[(u8, i64)]) -> ZSet {
        ZSet::of(
            &pairs
                .iter()
                .map(|(r, w)| (Row::new(*r), *w))
                .collect::<Vec<_>>(),
        )
    }

    /// The mint pads and truncates to the declared depth — the wrong-depth stream is
    /// unconstructible, and `at` reads the group zero beyond the horizon.
    #[test]
    fn the_depth_bound_is_the_mints_invariant() {
        let short = Stream::of(&[z(&[(0, 1)])]);
        assert_eq!(short.ticks().len(), DEPTH);
        assert_eq!(short.at(0), z(&[(0, 1)]));
        assert_eq!(short.at(DEPTH + 3), ZSet::empty());
        let long: Vec<ZSet> = (0..DEPTH as i64 + 2).map(|n| z(&[(0, n + 1)])).collect();
        assert_eq!(Stream::of(&long).ticks().len(), DEPTH);
    }

    /// The three operators tell their stories on an impulse and a constant: delay
    /// shifts, differentiation isolates the change, integration accumulates — two
    /// distinct fixtures each, so no constant map passes.
    #[test]
    fn delay_d_and_i_read_back_their_own_stories() {
        let a1 = z(&[(0, 1)]);
        let constant = Stream::of(&[a1.clone(), a1.clone(), a1.clone(), a1.clone(), a1.clone()]);
        // delay shifts the constant's onset by one tick.
        assert_eq!(constant.delay().at(0), ZSet::empty());
        assert_eq!(constant.delay().at(1), a1);
        // d of a constant stream is the initial impulse (nothing changes after onset).
        let d = constant.differentiate();
        assert_eq!(d.at(0), a1);
        assert_eq!(d.at(1), ZSet::empty());
        // i of an impulse is the constant from its onset.
        let impulse = Stream::of(std::slice::from_ref(&a1));
        assert_eq!(impulse.integrate(), constant);
        // and a retraction history integrates to presence-then-absence.
        let retract = Stream::of(&[a1.clone(), z(&[(0, -1)])]);
        let state = retract.integrate();
        assert_eq!(state.at(0), a1);
        assert_eq!(state.at(1), ZSet::empty());
    }
}
