//! schemata — the compiled-mutant sweep, whole (`discover::schemata`).
//!
//!     cargo run --features schemata --example schemata -- sweep   # the gate
//!     cargo run --features schemata --example schemata -- list    # the census
//!     cargo run --features schemata --example schemata -- judge <survivors.txt>
//!
//! `sweep` is the entire pipeline in one typed place — no shell carries semantics:
//! build the test binary once; run the TIMED baseline, which doubles as the COVERAGE
//! run (`SCHEMATA_RECORD` + nextest's process-per-test makes each touch-file a
//! site→test edge list); assemble the plan (a mutant runs only its covering tests —
//! EXACT, since a flip cannot change behaviour where its guard never executes, and a
//! site no test reaches is a survivor before any run); fan the verdicts out over
//! `available_parallelism` workers against the one build; and judge survivors
//! against the ratified register with set-difference semantics. The timeout derives
//! from the baseline (5x + 10s, never hand-picked); exceeding it is a DETECTION.
//! Without nextest the sweep falls back to the full suite per mutant — slower, same
//! verdicts. Exit 1 on drift, every mutant named.

use boundary_spec::discover::schemata::Schemata;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("list") => match Schemata::census() {
            Ok(sites) => {
                for site in sites {
                    println!("{site}");
                }
            }
            Err(collision) => {
                eprintln!("{collision}");
                std::process::exit(1);
            }
        },
        Some("judge") if args.len() == 2 => {
            let live = std::fs::read_to_string(&args[1]).unwrap_or_default();
            let survivors: Vec<&str> = live
                .lines()
                .map(str::trim)
                .filter(|l| !l.is_empty())
                .collect();
            match Schemata::register().check(survivors.iter().copied()) {
                Ok(()) => println!(
                    "schemata sweep clean: {} survivor(s), all ratified.",
                    survivors.len()
                ),
                Err(drift) => {
                    eprintln!("{drift}");
                    std::process::exit(1);
                }
            }
        }
        Some("sweep") => sweep(),
        Some("verify") => verify(),
        _ => {
            eprintln!(
                "usage: schemata verify | schemata sweep | schemata list | schemata judge \
                 <survivors.txt>"
            );
            std::process::exit(2);
        }
    }
}

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// The per-mutant timeout, priced from the timed green baseline — never hand-picked.
fn derive_limit(baseline: Duration) -> Duration {
    baseline * 5 + Duration::from_secs(10)
}

/// One touch-file's edges: the `T`-headed test list and the `S`-lines it reached.
fn parse_touch(text: &str) -> (Vec<String>, Vec<String>) {
    let mut tests = Vec::new();
    let mut sites = Vec::new();
    for line in text.lines() {
        if let Some(t) = line.strip_prefix("T ") {
            tests = t.split_whitespace().map(str::to_string).collect();
        } else if let Some(site) = line.strip_prefix("S ") {
            sites.push(site.trim().to_string());
        }
    }
    (tests, sites)
}

/// The plan: every census site paired with its covering tests, in census order. An
/// empty test list is the UNCOVERED verdict — unexecuted is unkillable, disclosed.
fn assemble_plan(
    census: &[&'static str],
    touches: &[(Vec<String>, Vec<String>)],
) -> Vec<(&'static str, Vec<String>)> {
    let mut coverage: BTreeMap<&str, BTreeSet<String>> = BTreeMap::new();
    for (tests, sites) in touches {
        for site in sites {
            coverage
                .entry(site.as_str())
                .or_default()
                .extend(tests.iter().cloned());
        }
    }
    census
        .iter()
        .map(|site| {
            let tests = coverage
                .get(*site)
                .map(|t| t.iter().cloned().collect())
                .unwrap_or_default();
            (*site, tests)
        })
        .collect()
}

/// Run to completion or the derived limit: a green exit is a SURVIVAL signal, a red
/// exit a kill, and exceeding the limit is a DETECTION (the kill path) — the same
/// doctrine as the source sweeps.
fn survives(mut cmd: Command, limit: Duration) -> bool {
    let mut child = cmd
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("cargo spawns");
    let start = Instant::now();
    loop {
        match child.try_wait().expect("child waitable") {
            Some(status) => return status.success(),
            None if start.elapsed() > limit => {
                let _ = child.kill();
                let _ = child.wait();
                return false;
            }
            None => std::thread::sleep(Duration::from_millis(100)),
        }
    }
}

fn sweep() {
    // one build serves every verdict below.
    let built = Command::new("cargo")
        .args([
            "test",
            "-q",
            "-p",
            "boundary-spec",
            "--lib",
            "--features",
            "schemata",
            "--no-run",
        ])
        .status()
        .expect("cargo runs");
    if !built.success() {
        std::process::exit(1);
    }
    let census = match Schemata::census() {
        Ok(sites) => sites,
        Err(collision) => {
            eprintln!("{collision}");
            std::process::exit(1);
        }
    };
    let have_nextest = Command::new("cargo")
        .args(["nextest", "--version"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    // the baseline: unmutated, timed, RECORDED — a red baseline would make every
    // mutant verdict meaningless, so it refuses before judging anything.
    let record = std::env::temp_dir().join(format!("schemata-record-{}", std::process::id()));
    std::fs::create_dir_all(&record).expect("record dir");
    let start = Instant::now();
    let mut baseline_cmd = Command::new("cargo");
    if have_nextest {
        baseline_cmd
            .args([
                "nextest",
                "run",
                "-p",
                "boundary-spec",
                "--lib",
                "--features",
                "schemata",
            ])
            .env("SCHEMATA_RECORD", &record);
    } else {
        baseline_cmd.args([
            "test",
            "-q",
            "-p",
            "boundary-spec",
            "--lib",
            "--features",
            "schemata",
        ]);
    }
    if !survives(baseline_cmd, Duration::from_secs(3600)) {
        eprintln!("schemata: the UNMUTATED suite is red — fix the suite before judging mutants");
        std::process::exit(1);
    }
    let baseline = start.elapsed();
    let limit = derive_limit(baseline);
    println!(
        "baseline {}s green; per-mutant timeout {}s (derived, 5x + 10)",
        baseline.as_secs(),
        limit.as_secs()
    );

    let mut touches = Vec::new();
    for entry in std::fs::read_dir(&record).expect("record readable") {
        let text = std::fs::read_to_string(entry.expect("entry").path()).unwrap_or_default();
        touches.push(parse_touch(&text));
    }
    let plan = assemble_plan(&census, &touches);

    let queue: Mutex<VecDeque<(&'static str, Vec<String>)>> = Mutex::new(plan.into());
    let survivors: Mutex<Vec<String>> = Mutex::new(Vec::new());
    let verdicts: Mutex<Vec<(String, String, Vec<String>)>> = Mutex::new(Vec::new());
    let workers = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
    std::thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| loop {
                let Some((site, tests)) = queue.lock().expect("queue").pop_front() else {
                    return;
                };
                let verdict = if have_nextest {
                    if tests.is_empty() {
                        // no test executes this site's guard: unkillable, disclosed.
                        println!("SURVIVED {site} (uncovered — no test reaches it)");
                        survivors.lock().expect("survivors").push(site.to_string());
                        verdicts.lock().expect("verdicts").push((
                            site.to_string(),
                            "SURVIVED".to_string(),
                            Vec::new(),
                        ));
                        continue;
                    }
                    let expr = tests
                        .iter()
                        .map(|t| format!("test(={t})"))
                        .collect::<Vec<_>>()
                        .join(" or ");
                    let mut cmd = Command::new("cargo");
                    cmd.args([
                        "nextest",
                        "run",
                        "-p",
                        "boundary-spec",
                        "--lib",
                        "--features",
                        "schemata",
                        "--fail-fast",
                        "--test-threads",
                        "2",
                        "-E",
                        &expr,
                    ])
                    .env("PROBE_MUTANT", site);
                    survives(cmd, limit)
                } else {
                    let mut cmd = Command::new("cargo");
                    cmd.args([
                        "test",
                        "-q",
                        "-p",
                        "boundary-spec",
                        "--lib",
                        "--features",
                        "schemata",
                    ])
                    .env("PROBE_MUTANT", site);
                    survives(cmd, limit)
                };
                if verdict {
                    println!("SURVIVED {site}");
                    survivors.lock().expect("survivors").push(site.to_string());
                } else {
                    println!("killed   {site}");
                }
                verdicts.lock().expect("verdicts").push((
                    site.to_string(),
                    if verdict { "SURVIVED" } else { "killed" }.to_string(),
                    tests,
                ));
            });
        }
    });

    // the ATTESTATION: the whole derivation, committed for the sampled countersign —
    // written before the register judgment so a red gate still leaves an honest
    // record, and only under nextest (the fallback path never learned which tests
    // cover which site, so its transcript could not be audited).
    if have_nextest {
        let mut sites: Vec<(String, String, Vec<String>)> =
            verdicts.into_inner().expect("verdicts");
        sites.sort();
        let transcript = boundary_spec::discover::attest::Transcript {
            tree: boundary_spec::discover::verdict::VerdictStore::tree_hash(std::path::Path::new(
                ".",
            ))
            .expect("the tree fingerprints"),
            toolchain: boundary_spec::discover::gates::TOOLCHAIN.to_string(),
            baseline_secs: baseline.as_secs(),
            sites: sites
                .into_iter()
                .map(
                    |(site, verdict, tests)| boundary_spec::discover::attest::SiteVerdict {
                        site,
                        verdict,
                        tests,
                    },
                )
                .collect(),
        };
        std::fs::create_dir_all("attest").expect("attest dir");
        let location =
            boundary_spec::discover::attest::Transcript::location(std::path::Path::new("."));
        std::fs::write(&location, transcript.render()).expect("attestation written");
        println!(
            "attested: {} — {} sites at tree {}",
            location.display(),
            transcript.sites.len(),
            transcript.tree
        );
    }

    let survivors = survivors.into_inner().expect("survivors");
    match Schemata::register().check(survivors.iter().map(String::as_str)) {
        Ok(()) => println!(
            "schemata sweep clean: {} survivor(s), all ratified.",
            survivors.len()
        ),
        Err(drift) => {
            eprintln!("{drift}");
            std::process::exit(1);
        }
    }
}

/// The mutation gate's ONE command: countersign a matching committed attestation by
/// sampled re-judgment, or sweep from scratch (writing a fresh attestation) when the
/// transcript is missing, foreign, or disagreeing. Locally, right after a sweep, this
/// is a fast audit of your own record; on a cold checkout it is the sampled
/// countersign — derivation local, signature cold, cost O(sample) — and any mismatch
/// falls back to the full sweep, so a false disagreement only ever costs time.
fn verify() {
    let root = std::path::Path::new(".");
    let key = match boundary_spec::discover::verdict::VerdictStore::tree_hash(root) {
        Ok(key) => key,
        Err(refusal) => {
            eprintln!("{refusal}");
            std::process::exit(1);
        }
    };
    let committed =
        std::fs::read_to_string(boundary_spec::discover::attest::Transcript::location(root))
            .ok()
            .and_then(|text| boundary_spec::discover::attest::Transcript::parse(&text).ok());
    match committed {
        Some(transcript)
            if transcript.tree == key
                && transcript.toolchain == boundary_spec::discover::gates::TOOLCHAIN =>
        {
            if countersign(&transcript) {
                return;
            }
            eprintln!("countersign: the attestation cannot stand — sweeping from scratch");
            sweep();
        }
        Some(transcript) => {
            println!(
                "attestation names tree {}, this is tree {key} — sweeping anew",
                transcript.tree
            );
            sweep();
        }
        None => {
            println!("no readable attestation at attest/sweep.transcript — sweeping");
            sweep();
        }
    }
}

/// The sampled audit: census and attestation must name the same site population, the
/// register judges the CLAIMED survivor set (an unratified survivor is a red gate, not
/// a disagreement), and then a random sample of sites is re-judged by exactly the
/// covering tests the transcript names. The seed comes from entropy AFTER the
/// transcript is fixed — which sites get audited cannot be known when the claim is
/// written, so a false `killed` cannot be placed where the audit won't look.
fn countersign(transcript: &boundary_spec::discover::attest::Transcript) -> bool {
    use std::hash::{BuildHasher, Hasher};
    let census = match Schemata::census() {
        Ok(sites) => sites,
        Err(collision) => {
            eprintln!("{collision}");
            std::process::exit(1);
        }
    };
    let attested: BTreeSet<&str> = transcript.sites.iter().map(|s| s.site.as_str()).collect();
    if census.len() != attested.len() || !census.iter().all(|site| attested.contains(site)) {
        eprintln!(
            "countersign: the census ({} sites) and the attestation ({}) name different \
             populations",
            census.len(),
            attested.len()
        );
        return false;
    }
    let survivors: Vec<&str> = transcript
        .sites
        .iter()
        .filter(|s| s.verdict == "SURVIVED")
        .map(|s| s.site.as_str())
        .collect();
    if let Err(drift) = Schemata::register().check(survivors.iter().copied()) {
        eprintln!("{drift}");
        std::process::exit(1);
    }
    let have_nextest = Command::new("cargo")
        .args(["nextest", "--version"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if !have_nextest {
        println!("countersign needs nextest for per-site re-judgment — sweeping instead");
        return false;
    }
    // one build serves every sampled verdict — the sweep's own economy.
    let built = Command::new("cargo")
        .args([
            "test",
            "-q",
            "-p",
            "boundary-spec",
            "--lib",
            "--features",
            "schemata",
            "--no-run",
        ])
        .status()
        .expect("cargo runs");
    if !built.success() {
        std::process::exit(1);
    }
    let seed = std::collections::hash_map::RandomState::new()
        .build_hasher()
        .finish();
    let sample = transcript.sample(transcript.sites.len().div_ceil(64).max(8), seed);
    let limit = derive_limit(Duration::from_secs(transcript.baseline_secs));
    println!(
        "countersigning {} of {} sites (seed {seed:016x}; timeout {}s from the attested \
         baseline)",
        sample.len(),
        transcript.sites.len(),
        limit.as_secs()
    );
    for entry in &sample {
        let observed_survives = if entry.tests.is_empty() {
            // no covering test was claimed: survival IS the claim's content, and a
            // claimed kill with no tests to kill by is a lie on its face.
            true
        } else {
            let expr = entry
                .tests
                .iter()
                .map(|t| format!("test(={t})"))
                .collect::<Vec<_>>()
                .join(" or ");
            let mut cmd = Command::new("cargo");
            cmd.args([
                "nextest",
                "run",
                "-p",
                "boundary-spec",
                "--lib",
                "--features",
                "schemata",
                "--fail-fast",
                "--test-threads",
                "2",
                "-E",
                &expr,
            ])
            .env("PROBE_MUTANT", entry.site.as_str());
            survives(cmd, limit)
        };
        let claimed_survives = entry.verdict == "SURVIVED";
        if observed_survives != claimed_survives {
            eprintln!(
                "countersign: `{}` — attested {} but observed {}",
                entry.site,
                entry.verdict,
                if observed_survives {
                    "SURVIVED"
                } else {
                    "killed"
                }
            );
            return false;
        }
        println!("agrees   {} ({})", entry.site, entry.verdict);
    }
    println!(
        "countersigned: {} sampled sites agree — the attestation stands for tree {}",
        sample.len(),
        transcript.tree
    );
    true
}

#[cfg(test)]
mod probes {
    use super::*;

    /// The sweep's semantics are PINNED where they live: the timeout derives 5x+10
    /// from the baseline; a touch-file parses to its edge list; the plan pairs every
    /// census site with exactly its covering tests, and an unreached site gets the
    /// empty list — the uncovered-survivor rule's precondition.
    #[test]
    fn the_sweep_semantics_are_pinned() {
        assert_eq!(
            derive_limit(std::time::Duration::from_secs(4)),
            std::time::Duration::from_secs(30)
        );
        let (tests, sites) = parse_touch("T a::b c::d\nS x:0: == -> !=\nS y:deaf -> None\n");
        assert_eq!(tests, vec!["a::b", "c::d"]);
        assert_eq!(sites, vec!["x:0: == -> !=", "y:deaf -> None"]);
        let plan = assemble_plan(
            &["covered:0: == -> !=", "unreached:0: == -> !="],
            &[(
                vec!["a::b".to_string()],
                vec!["covered:0: == -> !=".to_string()],
            )],
        );
        assert_eq!(plan[0].1, vec!["a::b"]);
        assert!(plan[1].1.is_empty(), "unreached site: empty = survivor");
    }

    /// The runner's two data paths hold: the census lists the committed sites, and
    /// the register judgment is the standard set-difference (an unratified survivor
    /// drifts as a new finding).
    #[test]
    fn the_census_and_register_paths_hold() {
        let sites = Schemata::census().expect("collision-free");
        if cfg!(feature = "schemata") {
            assert!(
                sites.contains(&"boundary_spec::discover::substrate::TagLaw::matches:0: == -> !=")
            );
        } else {
            assert!(
                sites.is_empty(),
                "instrumentation must not leak into normal builds"
            );
        }
        // the committed register and the ratified survivors are one set: exactly the
        // ratified keys hold (today that set is EMPTY — every compiled mutant dies), and
        // an unratified survivor drifts, named.
        let ratified: Vec<String> = Schemata::register()
            .entries()
            .expect("register parses")
            .into_iter()
            .map(|(k, _)| k)
            .collect();
        assert!(Schemata::register()
            .check(ratified.iter().map(String::as_str))
            .is_ok());
        let err = Schemata::register()
            .check(
                ratified
                    .iter()
                    .map(String::as_str)
                    .chain(["classify:1: == -> !="]),
            )
            .unwrap_err();
        assert!(err.contains("classify:1"), "{err}");
        // the STALE-LINE flag, drilled on a fixture so the drill outlives the live
        // register's population (which reached zero when the last four ratified
        // survivors died): a ratified line whose mutant no longer survives is a lie
        // the check must name.
        let dir = std::env::temp_dir().join(format!("schemata-register-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("fixture dir");
        let path = dir.join("stale.register");
        std::fs::write(
            &path,
            "`ghost::site:0: == -> !=`: a mutant that no longer survives\n",
        )
        .expect("fixture register");
        let fixture = spec_lock::Register {
            name: "schemata survivors".to_string(),
            path,
        };
        assert!(fixture.check([]).is_err(), "stale lines flag");
    }
}
