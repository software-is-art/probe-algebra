//! genesis — generate a whole crate layout from ONE blank-slate declaration.
//!
//!     cargo run --example genesis -- <declaration.rs> <target-dir> [--deps-path <checkout> | --deps-version <v>]
//!
//! The declaration is a `.rs` file containing a `system! { ... }` invocation (grammar: the
//! header of `src/discover/genesis.rs`; committed sample: `examples/genesis_demo.rs`). The
//! generator PARSES it — it never expands or compiles it — and emits the entire
//! downstream-fixture-shaped crate into `<target-dir>`: manifest, enforcement shim, tier-marked
//! boundary/interior/algebra files, TARGET spec locks (red until discovery matches the
//! declaration), the freeze loop, and the seam obligations. Every non-mechanical decision lands
//! as a greppable `todo!("MEANING: …")` hole.
//!
//! Dependencies default to path-deps into THIS checkout (`--deps-path` overrides the checkout
//! root; `--deps-version` switches to registry versions).

use std::path::Path;

use boundary_spec::discover::genesis::{Deps, Genesis};

fn usage() -> ! {
    eprintln!(
        "usage: cargo run --example genesis -- <declaration.rs> <target-dir> \
         [--deps-path <checkout> | --deps-version <v>]"
    );
    std::process::exit(2);
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut positional = Vec::new();
    let mut deps = Deps::Path(env!("CARGO_MANIFEST_DIR").to_string());
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--deps-path" => {
                i += 1;
                deps = Deps::Path(args.get(i).cloned().unwrap_or_else(|| usage()));
            }
            "--deps-version" => {
                i += 1;
                deps = Deps::Version(args.get(i).cloned().unwrap_or_else(|| usage()));
            }
            other => positional.push(other.to_string()),
        }
        i += 1;
    }
    let [declaration, target] = positional.as_slice() else {
        usage();
    };

    let source = std::fs::read_to_string(declaration)
        .unwrap_or_else(|e| panic!("read the declaration `{declaration}`: {e}"));
    let plan = Genesis::plan(&source, &deps).unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(1);
    });
    let written = Genesis::apply(&plan, Path::new(target)).expect("write the generated tree");

    println!(
        "genesis: `{}` — {} files derived from the declaration, under {target}:",
        plan.system.name,
        written.len()
    );
    for path in plan.listing() {
        println!("    {path}");
    }
    println!();
    println!("next (the generated src/lib.rs runbook says the same, in order):");
    println!("    1. grep -rn \"MEANING:\" — the complete list of holes to fill");
    println!("    2. first build blesses the census (see the generated build.rs)");
    println!("    3. cargo test — the freeze gate is RED until discovery earns the declaration");
    println!("    4. cargo run --example freeze — the bless diff vs the targets is the review");
}
