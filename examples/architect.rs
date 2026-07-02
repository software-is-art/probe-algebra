//! architect — the cohesion suggestion as an editor dev tool: LSP diagnostics + an auto-applicable
//! code action that scaffolds the split. The core a rust-analyzer / VS-Code extension would call.
//!
//!   cargo run --example architect            # the LSP payloads (diagnostics + code actions) as JSON
//!   cargo run --example architect -- --apply <dir>   # write the scaffolded split under <dir>

use std::path::PathBuf;

use boundary_spec::discover::architect::Architect;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let findings = Architect::analyze();
    if let Some(pos) = args.iter().position(|a| a == "--apply") {
        let dir = PathBuf::from(
            args.get(pos + 1)
                .map(String::as_str)
                .unwrap_or("scaffold_out"),
        );
        for f in &findings {
            let written = Architect::apply(&f.action, &dir).expect("apply");
            println!("applied `{}` -> {} files", f.action.title, written.len());
        }
    } else {
        println!("{}", Architect::render_lsp(&findings));
    }
}
