//! composition — what holds ACROSS a pipeline of modules, not within one.
//!
//! A transform seam is a conversion `h : A → B` that preserves structure (a homomorphism). A program
//! chains them — `A → B → C` — and the question a single module cannot answer is whether structure
//! SURVIVES the chain. It does: the composite of two homomorphisms is a homomorphism, so along the
//! pipeline the operation changes at every stage but the LAW is invariant. We verify it by running.
//!
//! The demo is a three-stage reading pipeline (`raw → scaled → reported`), each stage relabelling the
//! value while `combine` (a max) stays structurally the same. `composition` discovers that the whole
//! `raw → reported` pipeline preserves `combine` end to end. The real domains (arithmetic, the
//! router, the date calculus) have no transform chain, so they correctly report none.
//!
//! Run `cargo run --example composition`.

use boundary_spec::discover::arithmetic::Arithmetic;
use boundary_spec::discover::composition::PipelineLaw;
use boundary_spec::discover::date::Calendar;
use boundary_spec::discover::router::Router;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
enum Stage {
    Raw,
    Scaled,
    Reported,
}

struct Readings;

fn combine_raw(v: &[(u8, i64)]) -> Option<(u8, i64)> {
    Some((0, v[0].1.max(v[1].1)))
}
fn combine_scaled(v: &[(u8, i64)]) -> Option<(u8, i64)> {
    Some((1, v[0].1.max(v[1].1)))
}
fn combine_reported(v: &[(u8, i64)]) -> Option<(u8, i64)> {
    Some((2, v[0].1.max(v[1].1)))
}
fn scale(v: &[(u8, i64)]) -> Option<(u8, i64)> {
    Some((1, v[0].1))
}
fn report(v: &[(u8, i64)]) -> Option<(u8, i64)> {
    Some((2, v[0].1))
}

boundary_spec::theory! {
    Readings : "reading pipeline", Value = (u8, i64), Obs = (u8, i64), Sort = Stage,
    sort_of = |v: &(u8, i64)| match v.0 { 0 => Stage::Raw, 1 => Stage::Scaled, _ => Stage::Reported },
    observe = |v: &(u8, i64)| *v,
    vars { Stage::Raw => &["a"], Stage::Scaled => &["b"], Stage::Reported => &["c"], }
    inhabit {
        Stage::Raw => vec![(0, 0), (0, 1), (0, 2)],
        Stage::Scaled => vec![(1, 0), (1, 1), (1, 2)],
        Stage::Reported => vec![(2, 0), (2, 1), (2, 2)],
    }
    ops {
        Infix  "combineRaw"      "cR" (Stage::Raw, Stage::Raw) -> Stage::Raw = combine_raw;
        Infix  "combineScaled"   "cS" (Stage::Scaled, Stage::Scaled) -> Stage::Scaled = combine_scaled;
        Infix  "combineReported" "cP" (Stage::Reported, Stage::Reported) -> Stage::Reported = combine_reported;
        Prefix "scale"           "scale"  (Stage::Raw) -> Stage::Scaled = scale;
        Prefix "report"          "report" (Stage::Scaled) -> Stage::Reported = report;
    }
}

fn main() {
    println!("Composition analysis — what survives a transform pipeline:\n");
    print!("{}", PipelineLaw::render::<Readings>());
    print!("{}", PipelineLaw::render::<Arithmetic>());
    print!("{}", PipelineLaw::render::<Router>());
    print!("{}", PipelineLaw::render::<Calendar>());
    println!(
        "\nThe operation changes at every stage (combineRaw → combineScaled → combineReported), but\n\
         the composite report∘scale is still a homomorphism: the dataflow preserves the algebra end\n\
         to end. That composite equation is the whole-program spec of the pipeline."
    );
}
