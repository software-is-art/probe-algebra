//! bundle — the continuation verbs as a CLI: THE INTERFACE TO THE CODE (the peak the
//! operator named: agents should not write text into open files — and agents already
//! drive everything else through a CLI, so the CLI becomes the interaction mode).
//!
//! Every verb wraps the library form in `discover::bundle` / `discover::lift` — the CLI
//! adds only the file I/O and the exit code, so the judged transaction IS the interface:
//! a refused verb writes nothing and says why, a successful verb leaves the module
//! canonically placed. No open files, no partial states.
//!
//!     cargo run --example bundle -- add <module.rs> <snippet.rs | ->
//!     cargo run --example bundle -- declare <module.rs> "<shape(op, ...)>"
//!     cargo run --example bundle -- place <module.rs>
//!     cargo run --example bundle -- check <module.rs>
//!     cargo run --example bundle -- lift <module.rs> <theory-name> [declaration ...]
//!
//! `add` grows a module additively (a missing module file starts empty — birth is the
//! degenerate case of continuation); `declare` adds an expectation to its `#[algebra]`;
//! `place` rewrites a module into canonical placed order (the one verb that can move
//! code, still never changing a byte inside any item); `check` judges canonicality
//! without writing; `lift` prints the generated zero-annotation `impl Liftable` — with
//! the declarations baked in — for the caller to commit and drift-gate.

use std::io::Read;
use std::process::ExitCode;

use boundary_spec::discover::bundle::Bundle;
use boundary_spec::discover::lift::AutoLift;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(message) => {
            println!("{message}");
            ExitCode::SUCCESS
        }
        Err(refusal) => {
            eprintln!("{refusal}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: &[String]) -> Result<String, String> {
    let usage = || {
        "usage: bundle add <module.rs> <snippet.rs | ->\n\
         \x20      bundle declare <module.rs> \"<shape(op, ...)>\"\n\
         \x20      bundle place <module.rs>\n\
         \x20      bundle check <module.rs>\n\
         \x20      bundle lift <module.rs> <theory-name> [declaration ...]"
            .to_string()
    };
    match args {
        [verb, module_path, rest @ ..] => {
            let module = std::fs::read_to_string(module_path).unwrap_or_default();
            match (verb.as_str(), rest) {
                ("add", [snippet_source]) => {
                    let snippet = if snippet_source == "-" {
                        let mut buf = String::new();
                        std::io::stdin()
                            .read_to_string(&mut buf)
                            .map_err(|e| format!("bundle add: stdin unreadable: {e}"))?;
                        buf
                    } else {
                        std::fs::read_to_string(snippet_source)
                            .map_err(|e| format!("bundle add: snippet unreadable: {e}"))?
                    };
                    let grown = Bundle::add(&module, &snippet)?;
                    std::fs::write(module_path, &grown)
                        .map_err(|e| format!("bundle add: cannot write: {e}"))?;
                    Ok(format!("added into {module_path} — canonically placed"))
                }
                ("declare", [declaration]) => {
                    let declared = Bundle::declare(&module, declaration)?;
                    std::fs::write(module_path, &declared)
                        .map_err(|e| format!("bundle declare: cannot write: {e}"))?;
                    Ok(format!("declared `{declaration}` in {module_path}"))
                }
                ("place", []) => {
                    let placed = Bundle::parse(&module)?.render();
                    if placed == module {
                        Ok(format!("{module_path} is already canonically placed"))
                    } else {
                        std::fs::write(module_path, &placed)
                            .map_err(|e| format!("bundle place: cannot write: {e}"))?;
                        Ok(format!("{module_path} re-placed canonically"))
                    }
                }
                ("check", []) => {
                    if Bundle::parse(&module)?.is_canonical() {
                        Ok(format!("{module_path}: canonical"))
                    } else {
                        Err(format!(
                            "{module_path}: NOT canonical — `bundle place` would move it"
                        ))
                    }
                }
                ("lift", [theory_name, declarations @ ..]) => {
                    let declarations: Vec<&str> = declarations.iter().map(String::as_str).collect();
                    // the generated impl ends with its own newline; println adds the last
                    // one back, so a `> file` redirect captures the artifact byte-exact.
                    AutoLift::scan_module(&module, theory_name, &declarations)
                        .map(|generated| generated.trim_end().to_string())
                }
                _ => Err(usage()),
            }
        }
        _ => Err(usage()),
    }
}
