//! demo — exercises the morphological-testing algebra THROUGH the published
//! boundaries. The bin can name only `probe_algebra::boundary` (the grammar) and
//! `probe_algebra::ledger::boundary` (the ledger's interface); the aggregation
//! algorithm in `ledger::internal` is private and unreachable from here.

use probe_algebra::boundary::{probe, run, Compose, Morphism, ProbeResult};
use probe_algebra::ledger::boundary::{
    Account, Aggregate, AggregateDropsAmounts, Cents, NudgeCents, Posting, Round, Split,
    Transaction,
};

fn banner(s: &str) {
    println!("\n=== {} ===", s);
}
fn yn(b: bool) -> &'static str {
    if b {
        "yes"
    } else {
        "NO"
    }
}

fn sample() -> Transaction {
    Transaction::new(vec![
        Posting::new(Account::new("Cash").unwrap(), Cents::new(6000).unwrap()),
        Posting::new(Account::new("Cash").unwrap(), Cents::new(4000).unwrap()), // dup -> multiplicity
        Posting::new(
            Account::new("Revenue").unwrap(),
            Cents::new(-10000).unwrap(),
        ),
    ])
    .unwrap()
}

fn main() {
    let x = sample();

    banner("BOUNDARY: every cross-module value is a value object or value operator");
    println!("  main can name only crate::boundary + ledger::boundary.");
    println!("  ledger::internal (the aggregation algorithm) is PRIVATE — unreachable.");
    println!("  Aggregate is a Morphism: Transaction -> (AccountSummary, MultiplicityResidual).");

    banner("HONEST: forward output + residual, retained via the typestate");
    let carried = run(&Aggregate, &x); // Carried<Aggregate, Retained>
    println!("  summary totals : {:?}", carried.out().totals());
    println!("  residual       : {:?}", carried.residual());

    banner("INVERTIBILITY RESTORED: a retained residual makes a lossy map invert");
    let recovered = carried.invert(&Aggregate);
    println!(
        "  reconstructed transaction == original: {}",
        yn(recovered.as_ref() == Some(&x))
    );

    banner("TYPESTATE: discarding the residual REMOVES invertibility at compile time");
    let discarded = carried.discard(); // Carried<Aggregate, Discarded>
    println!("  output still available : {:?}", discarded.out().totals());
    println!("  discarded.invert(&Aggregate)  // <- does not compile: method gone");

    banner("PROBE (generic): perturb the lost dimension, check the residual");
    println!("  Operator: Split (perturbs multiplicity — the dimension Aggregate loses).\n");
    let honest = probe(&Aggregate, &Split, &x).unwrap();
    report("Aggregate (honest residual)", &honest);
    let buggy = probe(&AggregateDropsAmounts, &Split, &x).unwrap();
    report("AggregateDropsAmounts (records only counts)", &buggy);
    println!("  Same morphism TYPE, same Split probe — the probe alone catches the bug:");
    println!(
        "  the count-only residual cannot reconstruct, so round-trip FAILS ({}).",
        yn(buggy.round_trips)
    );

    banner("PROBE a different dimension: Round, perturbed by NudgeCents");
    let summary = run(&Aggregate, &x).out().clone();
    let round = probe(&Round, &NudgeCents, &summary).unwrap();
    report("Round (sub-dollar residual)", &round);

    banner("COMPOSITION: loss composes as a Pair value object");
    println!("  Compose {{ f: Aggregate, g: Round }} : Transaction -> rounded AccountSummary,");
    println!("  Residual = Pair<MultiplicityResidual, RoundingResidual>.");
    let pipeline = Compose {
        f: Aggregate,
        g: Round,
    };
    let (out, res) = pipeline.forward(&x);
    println!("  rounded totals : {:?}", out.totals());
    println!("  paired residual: {:?}", res);
    let back = pipeline.backward(&out, &res);
    println!(
        "  end-to-end round-trip through TWO lossy stages: {}",
        yn(back.as_ref() == Some(&x))
    );
    let composed = probe(&pipeline, &Split, &x).unwrap();
    report("Compose<Aggregate, Round> probed by Split", &composed);

    banner("WHAT THIS BUYS");
    println!("  - Loss is forced into typed residual value objects (visible in the type).");
    println!("  - Discarding a residual removes invertibility at COMPILE time (typestate).");
    println!("  - Every cross-module morphism is uniformly probeable for completeness.");
    println!("  - Residuals compose, so reference back-propagation flows THROUGH lossy");
    println!("    stages as long as the accumulated residual is retained.");

    banner("DONE");
}

fn report(label: &str, pr: &ProbeResult) {
    println!("  {}", label);
    println!(
        "    output invariant under perturbation : {}",
        yn(pr.output_invariant)
    );
    println!(
        "    residual responds                   : {}",
        yn(pr.residual_responds)
    );
    println!(
        "    round-trip on perturbed input       : {}",
        yn(pr.round_trips)
    );
    println!(
        "    => RESIDUAL COMPLETE                 : {}\n",
        yn(pr.residual_complete())
    );
}
