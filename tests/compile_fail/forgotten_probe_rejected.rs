//! A boundary edge declared in `edges!` but with NO `Probed` impl must not compile — the
//! edge analog of `incomplete_probe_rejected` (a partial DOF probe). `assert_all_probed`
//! requires every edge in the single-source list to carry a probe; `Unprobed` does not, so
//! `AllProbed` is unsatisfied and the program is rejected BEFORE any test runs. This is the
//! "you added an edge but forgot its probe" mistake, turned from a late surviving mutant into
//! an immediate build error.
use boundary_algebra::boundary::assert_all_probed;
use boundary_algebra::edges;

struct Unprobed; // an edge with no probe

type Edges = edges!(Unprobed);

fn main() {
    assert_all_probed::<Edges>();
}
