/// One committed trial: what the weaver wrote and what the judge ruled, keyed by
/// (fanout, seed) so the corpus it was scored against regenerates at gate time.
/// v2 adds the gain census (`stated` — emergent facts the narrative made explicit)
/// and the return leg (`edges` — the topology a fresh reader recovered from the
/// narrative alone).
#[derive(Debug, Clone, PartialEq)]
pub struct Record {
    pub fanout: usize,
    pub seed: u64,
    pub weaver: String,
    pub judge: String,
    pub quality: u8,
    pub narrative: String,
    pub verdicts: Vec<(String, bool)>,
    pub stated: Vec<(String, bool)>,
    pub edges: Vec<(String, String)>,
}

/// Render a record into the committed `.trial` text — the exact inverse of `parse`
/// for narratives normalised to no trailing whitespace.
pub fn render(r: &Record) -> String {
    let mut s = String::new();
    s.push_str("weave-knee trial v2\n");
    s.push_str(&format!("fanout: {}\n", r.fanout));
    s.push_str(&format!("seed: {}\n", r.seed));
    s.push_str(&format!("weaver: {}\n", r.weaver));
    s.push_str(&format!("judge: {}\n", r.judge));
    s.push_str(&format!("quality: {}\n", r.quality));
    s.push_str("narrative:\n");
    for line in r.narrative.lines() {
        if line.is_empty() {
            s.push('\n');
        } else {
            s.push_str("  ");
            s.push_str(line);
            s.push('\n');
        }
    }
    s.push_str("verdicts:\n");
    for (id, entailed) in &r.verdicts {
        s.push_str(&format!(
            "  {id} {}\n",
            if *entailed {
                "ENTAILED"
            } else {
                "NOT-ENTAILED"
            }
        ));
    }
    s.push_str("stated:\n");
    for (id, stated) in &r.stated {
        s.push_str(&format!(
            "  {id} {}\n",
            if *stated { "STATED" } else { "NOT-STATED" }
        ));
    }
    s.push_str("edges:\n");
    for (from, to) in &r.edges {
        s.push_str(&format!("  {from} -> {to}\n"));
    }
    s
}

/// Parse a committed `.trial` file. Refusals name the offending line: a corrupt
/// committed trial is a gate failure, never a shrug. v1 trials refuse by version —
/// they were measured under the v1 prompt and live archived in `trials-v1/`.
pub fn parse(text: &str) -> Result<Record, String> {
    let mut lines = text.lines();
    let head = lines.next().unwrap_or_default();
    if head != "weave-knee trial v2" {
        return Err(format!("not a weave-knee v2 trial: first line is `{head}`"));
    }
    let fanout: usize = field(&mut lines, "fanout")?
        .parse()
        .map_err(|e| format!("fanout: {e}"))?;
    let seed: u64 = field(&mut lines, "seed")?
        .parse()
        .map_err(|e| format!("seed: {e}"))?;
    let weaver = field(&mut lines, "weaver")?.to_string();
    let judge = field(&mut lines, "judge")?.to_string();
    let quality: u8 = field(&mut lines, "quality")?
        .parse()
        .map_err(|e| format!("quality: {e}"))?;
    if !(1..=5).contains(&quality) {
        return Err(format!("quality {quality} outside 1-5"));
    }
    let sentinel = lines.next().unwrap_or_default();
    if sentinel != "narrative:" {
        return Err(format!("expected `narrative:`, found `{sentinel}`"));
    }
    let mut narrative_lines: Vec<&str> = Vec::new();
    loop {
        match lines.next() {
            None => return Err("missing `verdicts:` section".to_string()),
            Some("verdicts:") => break,
            Some("") => narrative_lines.push(""),
            Some(l) => narrative_lines.push(
                l.strip_prefix("  ")
                    .ok_or_else(|| format!("narrative line not indented: `{l}`"))?,
            ),
        }
    }
    let mut verdicts = Vec::new();
    loop {
        match lines.next() {
            None => return Err("missing `stated:` section".to_string()),
            Some("stated:") => break,
            Some(l) => {
                let l = l
                    .strip_prefix("  ")
                    .ok_or_else(|| format!("verdict line not indented: `{l}`"))?;
                let (id, word) = l
                    .split_once(' ')
                    .ok_or_else(|| format!("unreadable verdict line: `{l}`"))?;
                verdicts.push((id.to_string(), verdict_word(word, id)?));
            }
        }
    }
    let mut stated = Vec::new();
    loop {
        match lines.next() {
            None => return Err("missing `edges:` section".to_string()),
            Some("edges:") => break,
            Some(l) => {
                let l = l
                    .strip_prefix("  ")
                    .ok_or_else(|| format!("stated line not indented: `{l}`"))?;
                let (id, word) = l
                    .split_once(' ')
                    .ok_or_else(|| format!("unreadable stated line: `{l}`"))?;
                stated.push((id.to_string(), stated_word(word, id)?));
            }
        }
    }
    let mut edges = Vec::new();
    for l in lines {
        let l = l
            .strip_prefix("  ")
            .ok_or_else(|| format!("edge line not indented: `{l}`"))?;
        let (from, to) = l
            .split_once(" -> ")
            .ok_or_else(|| format!("unreadable edge line: `{l}`"))?;
        edges.push((from.to_string(), to.to_string()));
    }
    if verdicts.is_empty() {
        return Err("no verdicts recorded".to_string());
    }
    if stated.is_empty() {
        return Err("no stated census recorded".to_string());
    }
    Ok(Record {
        fanout,
        seed,
        weaver,
        judge,
        quality,
        narrative: narrative_lines.join("\n"),
        verdicts,
        stated,
        edges,
    })
}

fn field<'a>(lines: &mut std::str::Lines<'a>, name: &str) -> Result<&'a str, String> {
    let line = lines.next().unwrap_or_default();
    line.strip_prefix(&format!("{name}: "))
        .ok_or_else(|| format!("expected `{name}: ...`, found `{line}`"))
}

fn verdict_word(word: &str, id: &str) -> Result<bool, String> {
    match word {
        "ENTAILED" => Ok(true),
        "NOT-ENTAILED" => Ok(false),
        other => Err(format!("unknown verdict `{other}` for `{id}`")),
    }
}

/// Everything the judge's one reply must contain: both censuses and the felicity
/// score. Parsed whole so an absent census refuses before anything is written.
#[derive(Debug, Clone, PartialEq)]
pub struct JudgeReading {
    pub verdicts: Vec<(String, bool)>,
    pub stated: Vec<(String, bool)>,
    pub quality: u8,
}

fn stated_word(word: &str, id: &str) -> Result<bool, String> {
    match word {
        "STATED" => Ok(true),
        "NOT-STATED" => Ok(false),
        other => Err(format!("unknown stated word `{other}` for `{id}`")),
    }
}

/// Read the judge's raw output: `verdict <id> ENTAILED|NOT-ENTAILED` lines, `stated
/// <id> STATED|NOT-STATED` lines, and one `quality <1-5>` line. Surrounding prose is
/// ignored (models preamble); a malformed word refuses, and an absent census —
/// either one — refuses rather than scoring around the hole.
pub fn read_judge(raw: &str) -> Result<JudgeReading, String> {
    let mut verdicts = Vec::new();
    let mut stated = Vec::new();
    let mut quality = None;
    for line in raw.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("verdict ") {
            let (id, word) = rest
                .split_once(' ')
                .ok_or_else(|| format!("unreadable verdict line: `{line}`"))?;
            verdicts.push((id.to_string(), verdict_word(word.trim(), id)?));
        } else if let Some(rest) = line.strip_prefix("stated ") {
            let (id, word) = rest
                .split_once(' ')
                .ok_or_else(|| format!("unreadable stated line: `{line}`"))?;
            stated.push((id.to_string(), stated_word(word.trim(), id)?));
        } else if let Some(rest) = line.strip_prefix("quality ") {
            let q: u8 = rest
                .trim()
                .parse()
                .map_err(|_| format!("unreadable quality line: `{line}`"))?;
            if !(1..=5).contains(&q) {
                return Err(format!("quality {q} outside 1-5"));
            }
            quality = Some(q);
        }
    }
    if verdicts.is_empty() {
        return Err("the judge produced no verdict lines".to_string());
    }
    if stated.is_empty() {
        return Err("the judge produced no stated lines".to_string());
    }
    match quality {
        Some(q) => Ok(JudgeReading {
            verdicts,
            stated,
            quality: q,
        }),
        None => Err("the judge produced no quality line".to_string()),
    }
}

/// Read the reconstructor's raw output: `edge <from> <to>` lines, exact module
/// names. An empty answer is a measurement (topology recall zero), not a refusal —
/// but a malformed edge line refuses.
pub fn parse_edges(raw: &str) -> Result<Vec<(String, String)>, String> {
    let mut edges = Vec::new();
    for line in raw.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("edge ") {
            let (from, to) = rest
                .split_once(' ')
                .ok_or_else(|| format!("unreadable edge line: `{line}`"))?;
            edges.push((from.trim().to_string(), to.trim().to_string()));
        }
    }
    Ok(edges)
}

#[cfg(test)]
mod probes {
    use super::*;

    fn sample() -> Record {
        Record {
            fanout: 3,
            seed: 7,
            weaver: "claude-sonnet-5".to_string(),
            judge: "claude-sonnet-5".to_string(),
            quality: 4,
            narrative: "First line.\n\n  An indented line.\nLast line.".to_string(),
            verdicts: vec![
                ("c0.0".to_string(), true),
                ("c0.0F".to_string(), false),
                ("r0".to_string(), true),
            ],
            stated: vec![("e.ring".to_string(), true), ("e.ringF".to_string(), false)],
            edges: vec![
                ("Pamzin".to_string(), "Felsax".to_string()),
                ("Felsax".to_string(), "Pamzin".to_string()),
            ],
        }
    }

    #[test]
    fn render_then_parse_is_identity() {
        let r = sample();
        assert_eq!(parse(&render(&r)).expect("round trip"), r);
        let mut bare = sample();
        bare.edges.clear();
        assert_eq!(parse(&render(&bare)).expect("round trip"), bare);
    }

    #[test]
    fn refusals_name_the_offence() {
        assert!(parse("not a trial").unwrap_err().contains("first line"));
        assert!(parse("weave-knee trial v1\n")
            .unwrap_err()
            .contains("first line"));
        let mut doctored = render(&sample()).replace("ENTAILED", "MAYBE");
        assert!(parse(&doctored).unwrap_err().contains("unknown verdict"));
        doctored = render(&sample()).replace("quality: 4", "quality: 9");
        assert!(parse(&doctored).unwrap_err().contains("outside 1-5"));
        doctored = render(&sample()).replace(" STATED", " SORT-OF");
        assert!(parse(&doctored)
            .unwrap_err()
            .contains("unknown stated word"));
        doctored = render(&sample()).replace(" -> ", " => ");
        assert!(parse(&doctored)
            .unwrap_err()
            .contains("unreadable edge line"));
    }

    #[test]
    fn the_judge_parser_skips_preamble_and_demands_both_censuses_and_quality() {
        let raw = "Sure, here are my verdicts:\n\nverdict c0.0 ENTAILED\nverdict c0.0F \
                   NOT-ENTAILED\nstated e.ring STATED\nstated e.ringF NOT-STATED\n\
                   quality 3\n";
        let reading = read_judge(raw).expect("read the judge");
        assert_eq!(reading.verdicts.len(), 2);
        assert_eq!(reading.stated.len(), 2);
        assert_eq!(reading.quality, 3);
        assert!(read_judge("verdict c0.0 ENTAILED\nstated e.ring STATED\n")
            .unwrap_err()
            .contains("no quality line"));
        assert!(read_judge("quality 3\n")
            .unwrap_err()
            .contains("no verdict lines"));
        assert!(read_judge("verdict c0.0 ENTAILED\nquality 3\n")
            .unwrap_err()
            .contains("no stated lines"));
    }

    #[test]
    fn the_edge_parser_reads_edges_and_tolerates_none() {
        let edges =
            parse_edges("Here:\nedge Pamzin Felsax\nedge Felsax Pamzin\n").expect("parse edges");
        assert_eq!(edges.len(), 2);
        assert_eq!(edges[0], ("Pamzin".to_string(), "Felsax".to_string()));
        assert!(parse_edges("no edges here\n")
            .expect("empty is a reading")
            .is_empty());
        assert!(parse_edges("edge OnlyOne\n").is_err());
    }
}
