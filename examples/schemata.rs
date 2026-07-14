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

/// The from-scratch oracle: every site judged fresh, nothing carried — what
/// incremental re-judgment is audited against. `verify` reaches for this when the
/// attestation is missing, unreadable, from another toolchain, or disagreeing
/// under the countersign (a record that cannot stand must not be leaned on).
fn sweep() {
    sweep_with(None)
}

/// The mutation gate's ONE command, three honesty tiers by what the record affords:
/// a transcript matching this tree and toolchain is COUNTERSIGNED by sampled
/// re-judgment (derivation local, signature cold, cost O(sample)); a same-toolchain
/// transcript from another tree feeds INCREMENTAL re-judgment (verdicts carried
/// where their evidence — site module plus covering-test modules, by content —
/// still stands; only the moved sites judged); anything else earns the FULL sweep.
/// Every tier writes or preserves an attestation the next run can lean on, any
/// disagreement falls back a tier, and a false disagreement only ever costs time.
fn verify() {
    let root = std::path::Path::new(".");
    // the key is the SCHEMATA GATE's declared support — single-sourced from the
    // registry, so an inert edit (docs) neither invalidates the attestation nor
    // re-owes the sweep.
    let support = boundary_spec::discover::gates::GateRegistry::declared()
        .into_iter()
        .find(|g| g.name == "mutation (schemata)")
        .map(|g| g.support)
        .expect("the schemata gate is declared");
    let key = match boundary_spec::discover::verdict::VerdictStore::support_hash(root, &support) {
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
        Some(transcript) if transcript.toolchain == boundary_spec::discover::gates::TOOLCHAIN => {
            println!(
                "attestation names tree {}, this is tree {key} — carrying what its \
                 evidence still warrants, judging the moved",
                transcript.tree
            );
            sweep_with(Some(transcript));
        }
        Some(transcript) => {
            println!(
                "attestation toolchain {} — this toolchain is {}; nothing carries \
                 across a toolchain, sweeping anew",
                transcript.toolchain,
                boundary_spec::discover::gates::TOOLCHAIN
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
        if entry.verdict == "timeout" {
            // a timeout is a detection, but a load-sensitive one: re-observing it
            // on this machine proves nothing either way, so the audit passes over
            // it — and incremental re-judgment already refuses to carry it, so it
            // cannot outlive the next sweep of its site.
            println!("skipped  {} (timeout — load-sensitive)", entry.site);
            continue;
        }
        let observed_green = if entry.tests.is_empty() {
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
            // an audit-side timeout reads as the kill path (the standing doctrine);
            // it can only false-DISAGREE with a survived claim, and that direction
            // costs a re-sweep, never a false fact.
            outcome(cmd, limit) == "SURVIVED"
        };
        let claimed_green = entry.verdict == "SURVIVED";
        if observed_green != claimed_green {
            eprintln!(
                "countersign: `{}` — attested {} but observed {}",
                entry.site,
                entry.verdict,
                if observed_green { "SURVIVED" } else { "killed" }
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

/// Run to completion or the limit, tri-state: a green exit means the mutant
/// SURVIVED (the suite cannot see it), a red exit is a kill, and a blown limit is
/// a TIMEOUT — still a detection for the gate, but named apart because it is
/// load-sensitive: a QoS-demoted or loaded machine times out innocent sites, so a
/// timeout verdict is never carried forward and never deletes a register line on
/// the scheduler's word alone.
fn outcome(mut cmd: Command, limit: Duration) -> &'static str {
    use std::os::unix::process::CommandExt;
    // The child gets its OWN process group, so the timeout kill reaps the whole
    // tree (cargo → nextest → test binary). Killing only the direct child orphaned
    // mutant-flipped runaways: each timed-out mutant leaked a live binary on PPID 1,
    // and the leaked allocators eventually invited the kernel's memory killer
    // (observed at 42GB of pressure on a 16GB machine).
    let mut child = cmd
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .process_group(0)
        .spawn()
        .expect("cargo spawns");
    let start = Instant::now();
    loop {
        match child.try_wait().expect("child waitable") {
            Some(status) => {
                return if status.success() {
                    "SURVIVED"
                } else {
                    "killed"
                };
            }
            None if start.elapsed() > limit => {
                // negative pid = the whole group; then reap the direct child.
                let _ = Command::new("kill")
                    .args(["-9", &format!("-{}", child.id())])
                    .status();
                let _ = child.wait();
                return "timeout";
            }
            None => std::thread::sleep(Duration::from_millis(100)),
        }
    }
}

/// The sweep with a prior attestation to lean on: every census site whose EVIDENCE
/// (its module and its covering tests' modules, by content) matches the prior
/// transcript carries its verdict forward without a run — except timeouts, which
/// are load-sensitive and always re-judged — and only the moved sites pay for
/// judgment. `sweep` is this with nothing to lean on; the weekly from-scratch
/// shards stay the backstop for what content keys cannot see (a covering test's
/// behaviour shifting through code it calls without its own module moving — the
/// same ratified gap as the retired since-green gate).
fn sweep_with(prior: Option<boundary_spec::discover::attest::Transcript>) {
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
    if outcome(baseline_cmd, Duration::from_secs(3600)) != "SURVIVED" {
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

    // the CARRY partition: each site's evidence key is minted against THIS tree; a
    // prior verdict whose site and key both match is carried without a run. The
    // fallback path never learns coverage, so its keys can never match a committed
    // (nextest-written) transcript — reuse composes only with auditable records.
    let mut evidence = boundary_spec::discover::attest::Evidence::under(std::path::Path::new("."));
    let keyed: Vec<(&'static str, Vec<String>, String)> = plan
        .into_iter()
        .map(|(site, tests)| {
            let key = evidence.of(site, &tests);
            (site, tests, key)
        })
        .collect();
    // ratified divergence: the one class of timeout that MAY carry — the register
    // line (spec/divergence.register) is the signature that the timeout is the
    // flip's nature, not the machine's mood; staleness is judged after the run.
    let divergent: std::collections::BTreeSet<String> = spec_lock::Register {
        name: "ratified divergence".to_string(),
        path: std::path::PathBuf::from("spec/divergence.register"),
    }
    .entries()
    .map(|entries| entries.into_iter().map(|(k, _)| k).collect())
    .unwrap_or_default();
    let mut carried: Vec<Judged> = Vec::new();
    let mut owed: VecDeque<(&'static str, Vec<String>, String)> = VecDeque::new();
    for (site, tests, key) in keyed {
        match prior
            .as_ref()
            .and_then(|p| boundary_spec::discover::attest::Evidence::carry(p, site, &key))
        {
            Some(verdict) => {
                carried.push((site.to_string(), verdict.to_string(), tests, key));
            }
            None if divergent.contains(site)
                && !key.is_empty()
                && prior.as_ref().is_some_and(|p| {
                    p.sites
                        .iter()
                        .any(|s| s.site == site && s.evidence == key && s.verdict == "timeout")
                }) =>
            {
                carried.push((site.to_string(), "timeout".to_string(), tests, key));
            }
            None => owed.push_back((site, tests, key)),
        }
    }
    if prior.is_some() {
        println!(
            "carrying {} verdict(s) at standing evidence; judging {} moved site(s)",
            carried.len(),
            owed.len()
        );
    }

    let queue: Mutex<VecDeque<(&'static str, Vec<String>, String)>> = Mutex::new(owed);
    let survivors: Mutex<Vec<String>> = Mutex::new(
        carried
            .iter()
            .filter(|(_, verdict, _, _)| verdict == "SURVIVED")
            .map(|(site, _, _, _)| site.clone())
            .collect(),
    );
    let verdicts: Mutex<Vec<Judged>> = Mutex::new(carried);
    // SCHEMATA_WORKERS caps the worker pool. The derived default answers the two
    // observed ways a sweep kills its own machine: BYTES — each worker's cargo →
    // nextest → test-binary chain peaked near 5 GB on the heavy-grid sites (twelve
    // workers put 58 GB of pressure on a 16 GB machine), so one worker per 8 GB of
    // physical memory; and CORES — two of headroom, because a saturated machine
    // times out innocent slow sites, and a timeout is re-judged every run until a
    // calm one settles it: saturation builds the treadmill it then runs on. The
    // knob remains for CI and for deliberate full-throttle sweeps.
    let workers = std::env::var("SCHEMATA_WORKERS")
        .ok()
        .and_then(|w| w.parse::<usize>().ok())
        .filter(|w| *w > 0)
        .unwrap_or_else(|| {
            let cores = std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1);
            let by_memory = physical_memory_gb().unwrap_or(8) / 8;
            cores.saturating_sub(2).clamp(1, by_memory.max(1))
        });
    std::thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| loop {
                let Some((site, tests, key)) = queue.lock().expect("queue").pop_front() else {
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
                            key,
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
                    outcome(cmd, limit)
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
                    outcome(cmd, limit)
                };
                match verdict {
                    "SURVIVED" => {
                        println!("SURVIVED {site}");
                        survivors.lock().expect("survivors").push(site.to_string());
                    }
                    // a timeout is a DETECTION (the doctrine holds), named apart so
                    // the record shows which kills are load-sensitive — those are
                    // never carried by a later incremental run.
                    "timeout" => println!("timeout  {site} (detection)"),
                    _ => println!("killed   {site}"),
                }
                verdicts.lock().expect("verdicts").push((
                    site.to_string(),
                    verdict.to_string(),
                    tests,
                    key,
                ));
            });
        }
    });

    // the ATTESTATION: the whole derivation, committed for the sampled countersign —
    // written before the register judgment so a red gate still leaves an honest
    // record, and only under nextest (the fallback path never learned which tests
    // cover which site, so its transcript could not be audited).
    let judged: Vec<Judged> = verdicts.into_inner().expect("verdicts");
    if have_nextest {
        let mut sites: Vec<Judged> = judged.clone();
        sites.sort();
        let transcript = boundary_spec::discover::attest::Transcript {
            // keyed by the SCHEMATA GATE's declared support (single-sourced from the
            // registry): an inert edit cannot orphan the attestation.
            tree: boundary_spec::discover::verdict::VerdictStore::support_hash(
                std::path::Path::new("."),
                &boundary_spec::discover::gates::GateRegistry::declared()
                    .into_iter()
                    .find(|g| g.name == "mutation (schemata)")
                    .map(|g| g.support)
                    .expect("the schemata gate is declared"),
            )
            .expect("the tree fingerprints"),
            toolchain: boundary_spec::discover::gates::TOOLCHAIN.to_string(),
            baseline_secs: baseline.as_secs(),
            sites: sites
                .into_iter()
                .map(|(site, verdict, tests, evidence)| {
                    boundary_spec::discover::attest::SiteVerdict {
                        site,
                        verdict,
                        tests,
                        evidence,
                    }
                })
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

    // divergence staleness, judged one-way AFTER the attestation is written: every
    // ratified line must be claimed by a timeout verdict this run — carried or
    // observed. A line whose site now dies, survives, or no longer exists is a
    // stale claim, and a stale claim is a lie the register must shed.
    let timed_out: std::collections::BTreeSet<&str> = judged
        .iter()
        .filter(|(_, verdict, _, _)| verdict == "timeout")
        .map(|(site, _, _, _)| site.as_str())
        .collect();
    let stale: Vec<&String> = divergent
        .iter()
        .filter(|d| !timed_out.contains(d.as_str()))
        .collect();
    if !stale.is_empty() {
        for line in &stale {
            eprintln!(
                "divergence register: `{line}` did not time out this run — a ratified \
                 divergence that stopped diverging is a stale claim; delete the line \
                 or re-earn it"
            );
        }
        std::process::exit(1);
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

/// One judged (or carried) site as the sweep's ledger holds it: census name,
/// verdict, covering tests, evidence key — the transcript row's in-flight shape.
type Judged = (String, String, Vec<String>, String);

/// Physical memory in GB — macOS `sysctl -n hw.memsize`, Linux `/proc/meminfo` —
/// because bytes, not cores, are the judging pool's binding constraint. None when
/// neither source speaks; the caller falls back conservatively.
fn physical_memory_gb() -> Option<usize> {
    if let Ok(out) = Command::new("sysctl").args(["-n", "hw.memsize"]).output() {
        if out.status.success() {
            if let Ok(bytes) = String::from_utf8_lossy(&out.stdout).trim().parse::<u64>() {
                return Some((bytes / 1_073_741_824) as usize);
            }
        }
    }
    let text = std::fs::read_to_string("/proc/meminfo").ok()?;
    let kb: u64 = text
        .lines()
        .find(|l| l.starts_with("MemTotal:"))?
        .split_whitespace()
        .nth(1)?
        .parse()
        .ok()?;
    Some((kb / 1_048_576) as usize)
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
        // survivors died): a ratified line whose mutant now dies is a lie
        // the check must name.
        let dir = std::env::temp_dir().join(format!("schemata-register-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("fixture dir");
        let path = dir.join("stale.register");
        std::fs::write(
            &path,
            "`ghost::site:0: == -> !=`: a mutant the suite has since learned to kill\n",
        )
        .expect("fixture register");
        let fixture = spec_lock::Register {
            name: "schemata survivors".to_string(),
            path,
        };
        assert!(fixture.check([]).is_err(), "stale lines flag");
    }

    /// The judgment's tri-state, pinned on real processes: a green exit is a
    /// survival, a red exit is a kill, and a blown limit is a TIMEOUT — the
    /// detection that incremental re-judgment refuses to carry, so a loaded
    /// machine's verdicts die with the run that minted them.
    #[test]
    fn the_outcome_tristate_is_pinned() {
        let limit = std::time::Duration::from_secs(10);
        assert_eq!(outcome(Command::new("true"), limit), "SURVIVED");
        assert_eq!(outcome(Command::new("false"), limit), "killed");
        let mut slow = Command::new("sleep");
        slow.arg("30");
        assert_eq!(
            outcome(slow, std::time::Duration::from_millis(100)),
            "timeout"
        );
    }
}
