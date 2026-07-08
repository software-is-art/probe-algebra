//! The SURFACE half of the downstream-reliance register — compile-judged. One line per
//! public surface a downstream consumer declared it stands on; a release that drops or
//! renames a surface refuses HERE, at compile time, naming the line. This is the
//! register's set semantics with the compiler as the judge — the strongest gate
//! available for API reliances, and the reason they do not live in
//! `downstream/reliances.register` (that file carries LAW reliances, judged by
//! `Dependence::judge_register` against the committed locks).
//!
//! Entries carry their consumer and why as comments — the justification prose no
//! derivation can produce. Consumers add their own lines by PR; a line whose consumer
//! no longer stands on it is a lie to delete.

use boundary_spec::discover::agenda::{Agenda, GuardVoices, Ratification};
use boundary_spec::discover::depend::{Dependence, DependenceReport, Standing};
use spec_lock::Register;

/// The declared surfaces, exercised shallowly — existence and shape, not behaviour
/// (behaviour is the law half's job, and each surface's own suite holds it).
#[test]
fn the_declared_surfaces_exist() {
    // a production adopter — its skills judge their own declared reliances between
    // release tags (the cross-repo form) and in their own suite (self-judgment).
    let _judge: fn(&str, &[Dependence], &str, &str) -> Result<DependenceReport, String> =
        Dependence::judge;
    let _ = Standing::Intact;

    // a production adopter — the edit guard runs in its committed hooks, voices derived
    // from the tree so only refusals that exist downstream ever pre-fire.
    let voices = GuardVoices {
        kernel_exempt: false,
        rats_nest: false,
    };
    assert_eq!(
        Agenda::edit_guard("docs/anything.md", "", &voices, &[]),
        None
    );

    // a production adopter — its review flow routes consumer lock classes taught as
    // data, questions rendered as the consumer wrote them.
    let classes = vec![(
        "surface.lock".to_string(),
        "the surface moved — is it intended?".to_string(),
    )];
    let agenda = Agenda::of_with(["spec/surface.lock"], &classes).expect("taught classes route");
    assert_eq!(
        agenda.ratifications,
        vec![Ratification::Custom {
            class: "surface.lock".to_string(),
            question: "the surface moved — is it intended?".to_string()
        }]
    );

    // a production adopter — the Register grammar itself is the vocabulary its own
    // client-side registers reuse (missing register = honestly empty).
    let register = Register {
        name: "probe".to_string(),
        path: std::path::PathBuf::from("/nonexistent/probe.register"),
    };
    assert_eq!(register.entries().expect("missing register is empty"), []);
}
