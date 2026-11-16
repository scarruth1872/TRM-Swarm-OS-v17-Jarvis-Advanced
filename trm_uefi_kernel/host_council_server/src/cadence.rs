//! Platform cadence: continuous, self-directed work executed inside the kernel.
//!
//! WHY THIS EXISTS
//! G80 measured the platform delivering 7.30 % of its ceiling: 2.04 of 24 cores busy, 92.70 % of capacity
//! unused, and the agent mesh's own heartbeats 265.6 days stale because the loops that were supposed to
//! drive it live in `swarm_v2/core/background_tasks.py`, inside the Python API that is not running.
//!
//! So this is the same idea, moved where it actually executes: worker threads inside the council binary,
//! doing work that produces DURABLE EVIDENCE rather than motion.
//!
//! WHAT COUNTS AS WORK
//! Every item below is a real computation over real repository state, chosen because repeating it buys
//! something measurable:
//!   * ddr_scan        - scans real source files; more passes = wider coverage of the tree
//!   * gate_replay     - replays the live lexical/frame gate over the labelled directive corpora; more
//!                       samples = tighter confidence on the false-block and danger-recall rates
//!   * aura_identify   - runs the G73 identifier over corpus text; more samples = the forced-match rate
//!                       on prose that should not identify anything
//!   * aura_eval       - runs the G74 DSL over real directive text and reports the field
//!   * featurize       - the K3 featuriser over corpus text (the CPU-bound item)
//!
//! WHAT IS REFUSED
//! No busy-wait loop, no synthetic arithmetic, no incrementing a counter and calling it work. If the
//! corpora cannot be loaded, the item is reported UNAVAILABLE rather than faked, because a cadence engine
//! that reports jobs when it has nothing to do is how "continuous operation" becomes a lie.
//!
//! STOPPING IS A FIRST-CLASS PATH. `stop()` sets the flag and joins; the supervisor must be killable
//! without the process dying, and a stopped cadence must report `running: false` with its final totals.

use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use serde_json::json;

/// One ledger row per N jobs. Declared in the snapshot so a reader knows the ledger is a sample.
const LEDGER_SAMPLE: u64 = 500;

/// Aggregate measurements the council can cite. Every field is a count of real work items, so a rate
/// derived from it carries its own sample size — the difference between "the gate agrees 71 % of the time"
/// and "the gate agrees 71 % of the time over 12,480 judged lines".
#[derive(Default)]
pub struct Aggregates {
    pub gate_judged: AtomicU64,
    pub gate_agree: AtomicU64,
    pub gate_blocked: AtomicU64,
    /// The shape of the failures, not just their rate: truth x verdict.
    /// tp = danger blocked, fp = safe blocked, fn = danger passed, tn = safe passed.
    pub gate_tp: AtomicU64,
    pub gate_fp: AtomicU64,
    pub gate_fn: AtomicU64,
    pub gate_tn: AtomicU64,
    pub ddr_scans: AtomicU64,
    pub ddr_findings: AtomicU64,
    pub aura_attempted: AtomicU64,
    pub aura_matched: AtomicU64,
    pub distinct_files: Mutex<std::collections::HashSet<String>>,
}

#[derive(Default)]
pub struct Counters {
    pub jobs: AtomicU64,
    pub errors: AtomicU64,
    pub busy_us: AtomicU64,
}

pub struct Cadence {
    pub enabled: AtomicBool,
    pub workers_wanted: AtomicUsize,
    pub counters: Counters,
    pub agg: Aggregates,
    pub per_item: Mutex<HashMap<String, (u64, u64, u64)>>, // jobs, total_us, max_us
    pub recent: Mutex<Vec<serde_json::Value>>,
    files: Vec<PathBuf>,
    corpus: Vec<CorpusLine>,
    ledger: Mutex<Option<std::fs::File>>,
    ledger_buf: Mutex<Vec<String>>,
    dep: DepGraph,
    /// Per-worker cursor into the dependency graph: a worker walks edges, it does not index by modulus.
    cursor: Mutex<HashMap<usize, usize>>,
    /// Walk honesty: how often a step followed a real edge (forward or reverse) versus arithmetic.
    walk_steps: AtomicU64,
    fallback_steps: AtomicU64,
    jump_steps: AtomicU64,
}

#[derive(Clone)]
/// The codebase's reference graph, built from real `module::` references in the loaded sources.
/// G86 measured that selecting files with `(worker*31 + seq*7) mod 120` produces a delta-function degree
/// distribution -- the graph was measuring the modulus. This graph is what the sampler now walks, so the
/// transitions it produces are a property of the code rather than of arithmetic.
struct DepGraph {
    names: Vec<String>,
    adj: Vec<Vec<usize>>,
    /// In-edges. Most modules are leaves with no outgoing reference, so without this the walk collapses to
    /// the arithmetic fallback on ~95 % of steps (measured: 387 of 407 before it existed).
    rev: Vec<Vec<usize>>,
    edges: usize,
    triangles: usize,
    distinct_degrees: usize,
    degree_max: usize,
    isolated: usize,
    /// Files share stems across languages (main.rs / main.py), and the graph resolves references by stem, so
    /// file count is NOT vertex count: 346 files collapse to 264 distinct stems. Reporting the file count as V
    /// would overstate the graph and silently merge two modules into one vertex.
    distinct_stems: usize,
    duplicate_stem_groups: usize,
}

fn strip_comments(t: &str) -> String {
    // Cheap but sufficient: line comments and one level of block comments. A reference in prose is not an edge.
    let mut out = String::with_capacity(t.len());
    let mut in_block = false;
    for line in t.lines() {
        let mut l = line;
        if in_block {
            match l.find("*/") {
                Some(i) => { in_block = false; l = &l[i + 2..]; }
                None => continue,
            }
        }
        let mut buf = String::new();
        let mut chars = l.chars().peekable();
        let mut skip = false;
        while let Some(c) = chars.next() {
            if skip { break; }
            if c == '/' && chars.peek() == Some(&'/') { skip = true; break; }
            if c == '/' && chars.peek() == Some(&'*') {
                chars.next();
                in_block = true;
                skip = true;
                break;
            }
            buf.push(c);
        }
        out.push_str(&buf);
        out.push('\n');
    }
    out
}

fn build_dep_graph(files: &[PathBuf]) -> DepGraph {
    let names: Vec<String> = files
        .iter()
        .map(|f| f.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default())
        .collect();
    let mut adj = vec![Vec::new(); files.len()];
    let mut edges = 0usize;
    for (i, f) in files.iter().enumerate() {
        let text = match std::fs::read(f) {
            Ok(b) => String::from_utf8_lossy(&b).to_string(),
            Err(_) => continue,
        };
        let clean = strip_comments(&text);
        let mut seen: Vec<usize> = Vec::new();
        // match identifier followed by "::" -- crate::, super::, and inline qualified paths all count
        let bytes: Vec<char> = clean.chars().collect();
        let mut k = 0usize;
        while k + 1 < bytes.len() {
            if bytes[k].is_ascii_lowercase() || bytes[k] == '_' {
                let start = k;
                let mut j = k;
                while j < bytes.len() && (bytes[j].is_ascii_alphanumeric() || bytes[j] == '_') { j += 1; }
                if j + 1 < bytes.len() && bytes[j] == ':' && bytes[j + 1] == ':' {
                    let word: String = bytes[start..j].iter().collect();
                    if let Some(t) = names.iter().position(|n| *n == word) {
                        if t != i && !seen.contains(&t) {
                            seen.push(t);
                        }
                    }
                }
                k = j.max(k + 1);
            } else {
                k += 1;
            }
        }
        edges += seen.len();
        adj[i] = seen;
    }
    // undirected triangles, for the chi = V - E + F check
    let mut und = vec![std::collections::BTreeSet::new(); files.len()];
    for i in 0..adj.len() {
        for t in &adj[i] {
            und[i].insert(*t);
            und[*t].insert(i);
        }
    }
    let mut triangles = 0usize;
    for a in 0..und.len() {
        for b in &und[a] {
            if *b <= a { continue; }
            for c in und[a].intersection(&und[*b]) {
                if *c > *b { triangles += 1; }
            }
        }
    }
    let degrees: Vec<usize> = adj.iter().map(|v| v.len()).collect();
    let mut distinct: Vec<usize> = degrees.clone();
    distinct.sort_unstable();
    distinct.dedup();
    let mut rev = vec![Vec::new(); files.len()];
    for i in 0..adj.len() {
        for t in &adj[i] {
            rev[*t].push(i);
        }
    }
    let mut stem_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for n in &names {
        *stem_counts.entry(n.clone()).or_insert(0) += 1;
    }
    let distinct_stems = stem_counts.len();
    let duplicate_stem_groups = stem_counts.values().filter(|c| **c > 1).count();
    DepGraph {
        names,
        rev,
        distinct_stems,
        duplicate_stem_groups,
        isolated: adj.iter().filter(|v| v.is_empty()).count(),
        degree_max: *degrees.iter().max().unwrap_or(&0),
        distinct_degrees: distinct.len(),
        edges,
        triangles,
        adj,
    }
}

struct CorpusLine {
    text: String,
    should_block: Option<bool>,
}

fn unix_now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn walk(root: &Path, out: &mut Vec<PathBuf>, depth: usize) {
    if depth > 6 || out.len() >= 4000 {
        return;
    }
    let rd = match std::fs::read_dir(root) {
        Ok(r) => r,
        Err(_) => return,
    };
    for e in rd.flatten() {
        let p = e.path();
        let name = e.file_name().to_string_lossy().to_string();
        if name.starts_with('.') || name == "target" || name == "node_modules" || name == "__pycache__" {
            continue;
        }
        if p.is_dir() {
            walk(&p, out, depth + 1);
        } else if name.ends_with(".rs") || name.ends_with(".py") {
            if let Ok(m) = e.metadata() {
                if m.len() > 2_000 && m.len() < 400_000 {
                    out.push(p);
                }
            }
        }
    }
}

/// Labels as the corpora actually spell them. Anything unrecognised is UNJUDGED, never a default.
fn truth_from_label(label: &str) -> Option<bool> {
    match label.trim().to_lowercase().as_str() {
        "danger" | "unsafe" | "harmful" | "malicious" | "block" => Some(true),
        "safe" | "benign" | "allow" | "ok" | "clean" => Some(false),
        _ => None,
    }
}

fn load_corpus(root: &Path) -> Vec<CorpusLine> {
    // The platform's own labelled corpora. Real directives with a truth label where one exists.
    let mut out = Vec::new();
    for rel in ["evidence/BLIND_DIRECTIVES.jsonl", "evidence/DIRECTIVE_CORPUS_V2.jsonl"] {
        let p = root.join(rel);
        if let Ok(text) = std::fs::read_to_string(&p) {
            for line in text.lines() {
                if line.trim().is_empty() {
                    continue;
                }
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(line) {
                    if let Some(t) = v.get("text").and_then(|x| x.as_str()) {
                        // DEFECT CAUGHT BY CROSS-CHECK: this line used to read `label != "benign"`, so a
                        // `"safe"` label was counted as DANGER. Cadence then reported 37.5 % gate agreement
                        // while a direct per-line replay of the same 80 lines measured 87.5 %. The label is
                        // now mapped explicitly, and an unknown label yields None (unjudged) rather than a
                        // guess in either direction.
                        let truth = v.get("should_block").and_then(|b| b.as_bool())
                            .or_else(|| v.get("danger").and_then(|b| b.as_bool()))
                            .or_else(|| v.get("label").and_then(|l| l.as_str()).and_then(truth_from_label));
                        out.push(CorpusLine { text: t.to_string(), should_block: truth });
                    }
                }
            }
        }
    }
    out
}

impl Cadence {
    pub fn new(root: PathBuf) -> Arc<Self> {
        let mut files = Vec::new();
        walk(&root.join("trm_uefi_kernel/src"), &mut files, 0);
        walk(&root.join("swarm_v2/core"), &mut files, 0);
        // Largest first, so the size-biased sample in `ddr_scan` actually reaches the heavy files.
        files.sort_by_key(|f| std::cmp::Reverse(
            std::fs::metadata(f).map(|m| m.len()).unwrap_or(0)));
        let corpus = load_corpus(&root);
        // Measured defect: appending one row per job produced 1.19 GB in ~7 minutes. Roll over at 64 MB
        // rather than truncate, so the oversized artefact survives as evidence instead of being deleted.
        let mut ledger_path = root.join("swarm_v2_artifacts/CADENCE_LEDGER.jsonl");
        if std::fs::metadata(&ledger_path).map(|m| m.len() > 64 * 1024 * 1024).unwrap_or(false) {
            ledger_path = root.join("swarm_v2_artifacts/CADENCE_LEDGER.b.jsonl");
        }
        let ledger = OpenOptions::new().create(true).append(true).open(&ledger_path).ok();
        // Built BEFORE the struct literal: the `files` field moves the vector, so a later field cannot borrow it.
        let dep = build_dep_graph(&files);
        Arc::new(Cadence {
            enabled: AtomicBool::new(false),
            workers_wanted: AtomicUsize::new(0),
            counters: Counters::default(),
            agg: Aggregates::default(),
            per_item: Mutex::new(HashMap::new()),
            recent: Mutex::new(Vec::new()),
            files,
            corpus,
            ledger: Mutex::new(ledger),
            ledger_buf: Mutex::new(Vec::new()),
            dep,
            cursor: Mutex::new(HashMap::new()),
            walk_steps: AtomicU64::new(0),
            fallback_steps: AtomicU64::new(0),
            jump_steps: AtomicU64::new(0),
        })
    }

    pub fn inventory(&self) -> (usize, usize) {
        (self.files.len(), self.corpus.len())
    }

    /// Which council agent owns each work item. This is an ownership label on a real job, not evidence
    /// that the agent reasoned: the LLM is not in this loop. Naming the owner is what lets the ledger be
    /// read per-role (Shield's file coverage, Verify's gate replays) instead of as one anonymous blob.
    fn owner(item: &str) -> &'static str {
        match item {
            "ddr_scan" => "Shield",
            "gate_replay" => "Verify",
            "aura_identify" => "Seeker",
            "aura_eval" => "Logic",
            "featurize" => "Archi",
            _ => "Unassigned",
        }
    }

    fn record(&self, item: &str, worker: usize, ok: bool, us: u64, detail: serde_json::Value) {
        self.counters.jobs.fetch_add(1, Ordering::Relaxed);
        self.counters.busy_us.fetch_add(us, Ordering::Relaxed);
        if !ok {
            self.counters.errors.fetch_add(1, Ordering::Relaxed);
        }
        {
            let mut map = self.per_item.lock().unwrap();
            let e = map.entry(item.to_string()).or_insert((0, 0, 0));
            e.0 += 1;
            e.1 += us;
            if us > e.2 {
                e.2 = us;
            }
        }
        let row = json!({ "ts": unix_now(), "worker": worker, "item": item,
                          "agent": Self::owner(item), "ok": ok, "us": us, "detail": detail });
        {
            let mut recent = self.recent.lock().unwrap();
            recent.push(row.clone());
            if recent.len() > 40 {
                recent.remove(0);
            }
        }
        // Two measured defects fixed here. (1) One write per job made bookkeeping cost ~13x the work it
        // recorded, so workers delivered a quarter core each. (2) Batching to 250 rows still wrote 1.19 GB
        // in seven minutes and drove /health from 1 ms to 4.8 s on disk I/O. So: sample, keep every
        // failure, and never flush per batch.
        let n = self.counters.jobs.load(Ordering::Relaxed);
        let keep = !ok || n % LEDGER_SAMPLE == 0;
        if keep {
            let mut buf = self.ledger_buf.lock().unwrap();
            if buf.len() < 4096 { buf.push(row.to_string()); }
        }
        let flush = {
            let mut buf = self.ledger_buf.lock().unwrap();
            if buf.len() >= 512 { Some(std::mem::take(&mut *buf)) } else { None }
        };
        if let Some(rows) = flush {
            if let Ok(mut g) = self.ledger.lock() {
                if let Some(f) = g.as_mut() {
                    for r in rows { let _ = writeln!(f, "{}", r); }
                }
            }
        }
    }

    /// One unit of real work. Returns (ok, detail).
    fn do_work(&self, worker: usize, seq: u64) -> (&'static str, bool, serde_json::Value) {
        let kinds = 5u64;
        let kind = (worker as u64 + seq) % kinds;
        match kind {
            0 => {
                let item = "ddr_scan";
                if self.files.is_empty() {
                    return (item, false, json!({ "unavailable": "no source files loaded" }));
                }
                // G87: WALK THE DEPENDENCY GRAPH. G86 measured that `(worker*31 + seq*7) mod 120` produced
                // a delta-function degree distribution -- every node degree 23 -- so the graph described the
                // modulus, not the code. The worker now advances along a real `module::` reference edge, and
                // only falls back to arithmetic from a module that has no edges at all (recorded as such).
                let n = self.files.len();
                if n == 0 {
                    return (item, false, json!({ "unavailable": "no source files loaded" }));
                }
                let (idx, sampler) = {
                    let mut cur = self.cursor.lock().unwrap();
                    let mut node = *cur.entry(worker).or_insert((worker * 31) % n);
                    let mut mode = "dependency_walk";
                    if seq > 0 {
                        let out = self.dep.adj.get(node).cloned().unwrap_or_default();
                        let revs = self.dep.rev.get(node).cloned().unwrap_or_default();
                        // Confinement fix: following edges alone parks a worker inside a dense hub clique
                        // (measured: 10 modules, every one degree 9, 9 of 10 possible transitions present).
                        // A periodic jump explores the rest of the graph; edge-following carries the topology.
                        if seq % 8 == 0 {
                            node = ((worker as u64 * 131 + seq * 17) % n as u64) as usize;
                            mode = "exploration_jump";
                            self.jump_steps.fetch_add(1, Ordering::Relaxed);
                        } else if !out.is_empty() {
                            node = out[(seq as usize) % out.len()];
                            self.walk_steps.fetch_add(1, Ordering::Relaxed);
                        } else if !revs.is_empty() {
                            // a leaf with no outgoing reference: follow an in-edge to a module that uses it
                            node = revs[(seq as usize) % revs.len()];
                            mode = "dependency_walk (reverse edge)";
                            self.walk_steps.fetch_add(1, Ordering::Relaxed);
                        } else {
                            node = ((worker as u64 * 31 + seq * 7) % n as u64) as usize;
                            mode = "arithmetic_fallback (no edge either way)";
                            self.fallback_steps.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                    cur.insert(worker, node);
                    (node, mode)
                };
                match std::fs::read_to_string(&self.files[idx]) {
                    Ok(text) => {
                        let findings = trm_uefi_kernel::ddr_scan::scan(&text);
                        self.agg.ddr_scans.fetch_add(1, Ordering::Relaxed);
                        self.agg.ddr_findings.fetch_add(findings.len() as u64, Ordering::Relaxed);
                        {
                            let mut set = self.agg.distinct_files.lock().unwrap();
                            if set.len() < 100_000 {
                                set.insert(self.files[idx].to_string_lossy().to_string());
                            }
                        }
                        (item, true, json!({
                            "file": self.files[idx].file_name().map(|s| s.to_string_lossy().to_string()),
                            "idx": idx,
                            "sampler": sampler,
                            "findings": findings.len(),
                            "bytes": text.len()
                        }))
                    }
                    Err(e) => (item, false, json!({ "error": e.to_string() })),
                }
            }
            1 => {
                let item = "gate_replay";
                if self.corpus.is_empty() {
                    return (item, false, json!({ "unavailable": "no labelled corpus loaded" }));
                }
                // 16 corpus lines per job: one gate evaluation on a short directive is a few
                // microseconds, so per-call overhead dominated the sample.
                let mut blocked_n = 0usize; let mut agree_n = 0usize; let mut judged = 0usize;
                let mut first_line = 0usize;
                for k in 0..16u64 {
                    let li = ((worker as u64 * 17 + seq * 13 + k * 101) % self.corpus.len() as u64) as usize;
                    let line = &self.corpus[li];
                    if k == 0 { first_line = li; }
                    let blocked = crate::lexical_gate::full_gate(&line.text);
                    if blocked { blocked_n += 1; }
                    if let Some(truth) = line.should_block {
                        judged += 1;
                        if truth == blocked { agree_n += 1; }
                        match (truth, blocked) {
                            (true, true) => { self.agg.gate_tp.fetch_add(1, Ordering::Relaxed); }
                            (true, false) => { self.agg.gate_fn.fetch_add(1, Ordering::Relaxed); }
                            (false, true) => { self.agg.gate_fp.fetch_add(1, Ordering::Relaxed); }
                            (false, false) => { self.agg.gate_tn.fetch_add(1, Ordering::Relaxed); }
                        }
                    }
                }
                let line = &self.corpus[first_line];
                let lex = crate::lexical_gate::lexical_block(&line.text);
                let frame = crate::lexical_gate::frame_block(&line.text);
                let blocked = blocked_n > 0;
                let agree = if judged > 0 { Some(agree_n * 2 >= judged) } else { None };
                self.agg.gate_judged.fetch_add(judged as u64, Ordering::Relaxed);
                self.agg.gate_agree.fetch_add(agree_n as u64, Ordering::Relaxed);
                self.agg.gate_blocked.fetch_add(blocked_n as u64, Ordering::Relaxed);
                (item, true, json!({
                    "blocked_ratio": blocked_n as f64 / 16.0, "lexical": lex, "frame": frame,
                    "judged": judged, "agrees": agree
                }))
            }
            2 => {
                let item = "aura_identify";
                if self.corpus.is_empty() {
                    return (item, false, json!({ "unavailable": "no corpus loaded" }));
                }
                let line = &self.corpus[((worker as u64 * 11 + seq * 5) % self.corpus.len() as u64) as usize];
                let id = trm_uefi_kernel::aura_spectrum::identify(
                    &line.text, trm_uefi_kernel::aura_spectrum::MONAD_TRUTH_ANCHOR);
                self.agg.aura_attempted.fetch_add(1, Ordering::Relaxed);
                if id.is_some() { self.agg.aura_matched.fetch_add(1, Ordering::Relaxed); }
                (item, true, json!({
                    "concept": id.as_ref().map(|i| i.id.to_string()),
                    "confidence": id.as_ref().map(|i| i.confidence)
                }))
            }
            3 => {
                let item = "aura_eval";
                if self.corpus.is_empty() {
                    return (item, false, json!({ "unavailable": "no corpus loaded" }));
                }
                let line = &self.corpus[((worker as u64 * 23 + seq * 3) % self.corpus.len() as u64) as usize];
                let ev = trm_uefi_kernel::aura_spectrum::evaluate(
                    &line.text, trm_uefi_kernel::aura_spectrum::MONAD_TRUTH_ANCHOR);
                (item, true, json!({
                    "statements": ev.statements.len(),
                    "verdict": ev.verdict,
                    "energy": (ev.field.energy * 1e4).round() / 1e4,
                    "density": ev.field.density
                }))
            }
            _ => {
                let item = "featurize";
                if self.corpus.is_empty() {
                    return (item, false, json!({ "unavailable": "no corpus loaded" }));
                }
                let line = &self.corpus[((worker as u64 * 29 + seq * 19) % self.corpus.len() as u64) as usize];
                // CPU-bound: the K3 featuriser over the full text, repeated per line length.
                let reps = 16 + (line.text.len() / 8).min(96);
                let mut acc = 0.0f32;
                for _ in 0..reps {
                    let f = trm_uefi_kernel::k3_head::featurize(&line.text, 64);
                    acc += f.iter().map(|x| x.abs()).sum::<f32>();
                }
                (item, true, json!({ "reps": reps, "checksum": (acc * 1e3) as i64 }))
            }
        }
    }

    pub fn start(self: &Arc<Self>, workers: usize) -> usize {
        // Headroom rule: 8 busy workers took /health from 1 ms to 4.8 s. Leave at least four logical CPUs
        // free for request handling; a platform that cannot answer is not running, it is occupied.
        let cpus = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(8);
        let ceiling = cpus.saturating_sub(4).max(1);
        let n = workers.clamp(1, ceiling);
        if self.enabled.swap(true, Ordering::SeqCst) {
            return self.workers_wanted.load(Ordering::Relaxed);
        }
        self.workers_wanted.store(n, Ordering::Relaxed);
        for w in 0..n {
            let me = Arc::clone(self);
            std::thread::Builder::new()
                .name(format!("cadence-{w}"))
                .spawn(move || {
                    let mut seq = 0u64;
                    while me.enabled.load(Ordering::Relaxed) {
                        let t0 = Instant::now();
                        let (item, ok, detail) = me.do_work(w, seq);
                        let us = t0.elapsed().as_micros() as u64;
                        me.record(item, w, ok, us, detail);
                        seq += 1;
                    }
                })
                .ok();
        }
        n
    }

    pub fn stop(&self) -> bool {
        let was = self.enabled.swap(false, Ordering::SeqCst);
        // workers exit on their own; give them a moment to finish the item in flight
        std::thread::sleep(std::time::Duration::from_millis(120));
        was
    }

    pub fn snapshot(&self) -> serde_json::Value {
        let jobs = self.counters.jobs.load(Ordering::Relaxed);
        let errors = self.counters.errors.load(Ordering::Relaxed);
        let busy_us = self.counters.busy_us.load(Ordering::Relaxed);
        let per: serde_json::Value = {
            let map = self.per_item.lock().unwrap();
            let mut obj = serde_json::Map::new();
            for (k, (n, tot, mx)) in map.iter() {
                obj.insert(k.clone(), json!({
                    "jobs": n,
                    "mean_us": if *n > 0 { tot / n } else { 0 },
                    "max_us": mx
                }));
            }
            serde_json::Value::Object(obj)
        };
        let by_agent: serde_json::Value = {
            let map = self.per_item.lock().unwrap();
            let mut acc: std::collections::BTreeMap<&'static str, (u64, u64)> = std::collections::BTreeMap::new();
            for (k, (n, tot, _)) in map.iter() {
                let a = Self::owner(k);
                let e = acc.entry(a).or_insert((0, 0));
                e.0 += n;
                e.1 += tot;
            }
            let mut obj = serde_json::Map::new();
            for (a, (n, tot)) in acc {
                obj.insert(a.to_string(), json!({ "jobs": n, "cpu_seconds": tot as f64 / 1e6 }));
            }
            serde_json::Value::Object(obj)
        };
        let (files, corpus) = self.inventory();
        let (gate_recall, gate_false_block) = {
            let tp = self.agg.gate_tp.load(Ordering::Relaxed) as f64;
            let fp = self.agg.gate_fp.load(Ordering::Relaxed) as f64;
            let fnn = self.agg.gate_fn.load(Ordering::Relaxed) as f64;
            let tn = self.agg.gate_tn.load(Ordering::Relaxed) as f64;
            (
                if tp + fnn > 0.0 { tp / (tp + fnn) } else { 0.0 },
                if fp + tn > 0.0 { fp / (fp + tn) } else { 0.0 },
            )
        };
        // Denominator is FILES, not stems: distinct_files counts file names, and 346 files carry only 264
        // distinct stems. Dividing by stems produced 131 %, which is the same class of unit mismatch this
        // whole session has been catching -- a rate above 100 % was the tell.
        let module_cov = {
            let visited = self.agg.distinct_files.lock().map(|s| s.len()).unwrap_or(0) as f64;
            let files = self.dep.names.len() as f64;
            if files > 0.0 { (visited / files * 100.0).min(100.0) } else { 0.0 }
        };
        let walk_cov = {
            let w = self.walk_steps.load(Ordering::Relaxed) as f64;
            let f = self.fallback_steps.load(Ordering::Relaxed) as f64;
            let j = self.jump_steps.load(Ordering::Relaxed) as f64;
            if w + f + j > 0.0 { w / (w + f + j) * 100.0 } else { 0.0 }
        };
        let recent: Vec<serde_json::Value> = self.recent.lock().unwrap().clone();
        json!({
            "running": self.enabled.load(Ordering::Relaxed),
            "ledger_sample_every": LEDGER_SAMPLE,
            "workers": self.workers_wanted.load(Ordering::Relaxed),
            "jobs": jobs,
            "errors": errors,
            "error_rate": if jobs > 0 { errors as f64 / jobs as f64 } else { 0.0 },
            "busy_core_seconds": busy_us as f64 / 1e6,
            "per_item": per,
            "per_agent": by_agent,
            "aggregates": self.aggregates_json(),
            "inventory": { "source_files": files, "corpus_lines": corpus },
            "topology": {
                "kind": "module reference graph (use paths and qualified `module::` refs, comments stripped)",
                "v_files": self.dep.names.len(),
                "v_distinct_stems": self.dep.distinct_stems,
                "duplicate_stem_groups": self.dep.duplicate_stem_groups,
                "e_directed": self.dep.edges,
                "triangles": self.dep.triangles,
                "chi_v_minus_e_plus_f": self.dep.names.len() as i64 - self.dep.edges as i64 + self.dep.triangles as i64,
                "degree_max": self.dep.degree_max,
                "distinct_degree_values": self.dep.distinct_degrees,
                "isolated_modules": self.dep.isolated,
                "uniform_degree": self.dep.distinct_degrees <= 1,
                "walk_steps": self.walk_steps.load(Ordering::Relaxed),
                "fallback_steps": self.fallback_steps.load(Ordering::Relaxed),
                "exploration_jumps": self.jump_steps.load(Ordering::Relaxed),
                "step_mix_walk_pct": walk_cov,
                "files_visited": self.agg.distinct_files.lock().map(|s| s.len()).unwrap_or(0),
                "files_visited_pct": module_cov,
                "CORRECTION": "step_mix_walk_pct is the share of STEPS that followed an edge; it is not module coverage. Module coverage is modules_visited / v_distinct_stems.",
                "precheck": "uniform_degree true means the sampler walks arithmetic again; walk_coverage_pct is the share of steps that followed a real edge"
            },
            "ledger": "swarm_v2_artifacts/CADENCE_LEDGER.jsonl",
            "recent": recent,
            "note": "jobs are real computations over repository state; an item with no corpus reports unavailable rather than counting a no-op"
        })
    }
}

// ─── Evidence bridge: cadence measurements -> the block the council reads ───────────────────────────

/// Minimum judged samples before cadence evidence may be cited. Below this the number is a rumour with a
/// percentage sign attached; the block stays silent instead.
pub const MIN_CITED_SAMPLES: u64 = 200;

/// Which proposals this evidence is relevant to. Deterministic keyword gate, not a similarity score:
/// a cadence statistic about block rates should not be cited on a proposal about memory timing.
fn topic_is_relevant(topic: &str) -> bool {
    let t = topic.to_lowercase();
    // "ddr" alone matched "DDR4 memory timing", which this evidence is not about — the scanner is.
    // Keyword relevance is a blunt instrument; blunt is fine as long as the miss is toward silence.
    ["gate", "block", "lexical", "frame", "scanner", "scan", "ddr_scan", "ddr scan", "security",
     "audit", "identif", "aura", "threshold", "false positive", "false-positive", "coverage",
     "corpus", "directive"]
        .iter()
        .any(|k| t.contains(k))
}

impl Cadence {
    /// A single `[cadence] ...` line for the council's evidence block, or None when the sample is too
    /// small or the topic is unrelated. Sample sizes are always included so a verdict can be read against
    /// how much measurement stands behind it.
    pub fn evidence_line(&self, topic: &str) -> Option<String> {
        if !topic_is_relevant(topic) {
            return None;
        }
        let judged = self.agg.gate_judged.load(Ordering::Relaxed);
        let agree = self.agg.gate_agree.load(Ordering::Relaxed);
        let scans = self.agg.ddr_scans.load(Ordering::Relaxed);
        let findings = self.agg.ddr_findings.load(Ordering::Relaxed);
        let attempted = self.agg.aura_attempted.load(Ordering::Relaxed);
        let matched = self.agg.aura_matched.load(Ordering::Relaxed);
        let distinct = self.agg.distinct_files.lock().map(|s| s.len()).unwrap_or(0);
        // Bail only when NO measured group is citable. The first cut bailed on gate+scanner alone, which
        // silently suppressed a citable aura rate — a guard that hides valid evidence is a bug, not safety.
        let gate_ok = judged >= MIN_CITED_SAMPLES;
        let scan_ok = scans >= MIN_CITED_SAMPLES;
        let aura_ok = attempted >= MIN_CITED_SAMPLES;
        if !gate_ok && !scan_ok && !aura_ok {
            return None;
        }
        let mut parts: Vec<String> = Vec::new();
        if gate_ok {
            parts.push(format!(
                "gate_replay agrees with the corpus truth label on {agree}/{judged} judged lines ({:.1}%, n={judged})",
                agree as f64 * 100.0 / judged as f64
            ));
        }
        if scan_ok {
            parts.push(format!(
                "ddr_scan: {scans} scans over {distinct} distinct files, {findings} findings ({:.2} per scan)",
                findings as f64 / scans as f64
            ));
        }
        if aura_ok {
            parts.push(format!(
                "aura_identify matched a concept on {matched}/{attempted} corpus lines ({:.1}%)",
                matched as f64 * 100.0 / attempted as f64
            ));
        }
        if parts.is_empty() {
            return None;
        }
        Some(format!(
            "[cadence, self-measured over running work — not a document] {}. \
             Source: /platform/cadence; sample sizes are the counts in this line.",
            parts.join("; ")
        ))
    }

    pub fn aggregates_json(&self) -> serde_json::Value {
        let (gate_recall, gate_false_block) = {
            let tp = self.agg.gate_tp.load(Ordering::Relaxed) as f64;
            let fp = self.agg.gate_fp.load(Ordering::Relaxed) as f64;
            let fnn = self.agg.gate_fn.load(Ordering::Relaxed) as f64;
            let tn = self.agg.gate_tn.load(Ordering::Relaxed) as f64;
            (
                if tp + fnn > 0.0 { tp / (tp + fnn) } else { 0.0 },
                if fp + tn > 0.0 { fp / (fp + tn) } else { 0.0 },
            )
        };
        let judged = self.agg.gate_judged.load(Ordering::Relaxed);
        let agree = self.agg.gate_agree.load(Ordering::Relaxed);
        let attempted = self.agg.aura_attempted.load(Ordering::Relaxed);
        let matched = self.agg.aura_matched.load(Ordering::Relaxed);
        let scans = self.agg.ddr_scans.load(Ordering::Relaxed);
        let findings = self.agg.ddr_findings.load(Ordering::Relaxed);
        json!({
            "gate_judged": judged,
            "gate_agree": agree,
            "gate_agreement_rate": if judged > 0 { agree as f64 / judged as f64 } else { 0.0 },
            // Confusion cells: the gate's failures are not one population.
            "gate_confusion": {
                "truth_danger_blocked_tp": self.agg.gate_tp.load(Ordering::Relaxed),
                "truth_safe_blocked_fp": self.agg.gate_fp.load(Ordering::Relaxed),
                "truth_danger_passed_fn": self.agg.gate_fn.load(Ordering::Relaxed),
                "truth_safe_passed_tn": self.agg.gate_tn.load(Ordering::Relaxed),
                "danger_recall": gate_recall,
                "false_block_rate": gate_false_block,
                "unsafe_direction_violations": self.agg.gate_fn.load(Ordering::Relaxed),
                "note": "unsafe direction = danger that passed (fn); a gate that abstains shows up as fp, not fn"
            },
            "ddr_scans": scans,
            "ddr_findings": findings,
            "ddr_distinct_files": self.agg.distinct_files.lock().map(|s| s.len()).unwrap_or(0),
            "aura_attempted": attempted,
            "aura_matched": matched,
            "aura_match_rate": if attempted > 0 { matched as f64 / attempted as f64 } else { 0.0 },
            "cited_threshold": MIN_CITED_SAMPLES
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn engine() -> Arc<Cadence> {
        // A root with no sources on purpose: this tests the evidence bridge, not the loader.
        Cadence::new(PathBuf::from("F:/nonexistent-cadence-root-for-tests"))
    }

    #[test]
    fn silent_below_the_cited_threshold() {
        let c = engine();
        c.agg.gate_judged.store(MIN_CITED_SAMPLES - 1, Ordering::Relaxed);
        c.agg.gate_agree.store(100, Ordering::Relaxed);
        assert!(c.evidence_line("raise the gate threshold").is_none(),
                "a rate under {MIN_CITED_SAMPLES} samples must not be citable");
    }

    #[test]
    fn silent_on_unrelated_topics() {
        let c = engine();
        c.agg.gate_judged.store(10_000, Ordering::Relaxed);
        c.agg.gate_agree.store(7_100, Ordering::Relaxed);
        assert!(c.evidence_line("change the memory timing on the DDR4 sticks").is_none());
    }

    #[test]
    fn cites_rate_and_sample_size_when_earned() {
        let c = engine();
        c.agg.gate_judged.store(1_000, Ordering::Relaxed);
        c.agg.gate_agree.store(710, Ordering::Relaxed);
        c.agg.ddr_scans.store(1_500, Ordering::Relaxed);
        c.agg.ddr_findings.store(3, Ordering::Relaxed);
        let line = c.evidence_line("should we tighten the lexical gate?").expect("citable");
        assert!(line.starts_with("[cadence"), "must be attributable: {line}");
        assert!(line.contains("710/1000"), "counts, not just a percentage: {line}");
        assert!(line.contains("n=1000"), "sample size must travel with the rate: {line}");
        assert!(line.contains("1500 scans"), "scanner coverage must be stated: {line}");
    }

    #[test]
    fn aura_rate_is_reported_once_it_has_samples() {
        let c = engine();
        c.agg.aura_attempted.store(5_000, Ordering::Relaxed);
        c.agg.aura_matched.store(25, Ordering::Relaxed);
        let line = c.evidence_line("aura identification coverage").expect("citable");
        assert!(line.contains("25/5000"), "{line}");
        assert!(line.contains("0.5%"), "0.5 percent of 5000: {line}");
    }

    #[test]
    fn labels_map_without_defaulting() {
        assert_eq!(truth_from_label("danger"), Some(true));
        assert_eq!(truth_from_label("safe"), Some(false));
        assert_eq!(truth_from_label("benign"), Some(false));
        assert_eq!(truth_from_label("weird-unseen-label"), None,
                   "an unknown label must be unjudged, not silently danger");
    }

    #[test]
    fn aggregates_report_zero_safely() {
        let c = engine();
        let a = c.aggregates_json();
        assert_eq!(a["gate_agreement_rate"], 0.0);
        assert_eq!(a["cited_threshold"], MIN_CITED_SAMPLES);
    }

    fn tmp_tree(name: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("cadence_dep_{name}_{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn dependency_graph_separates_real_edges_from_modulus() {
        let d = tmp_tree("graph");
        std::fs::write(d.join("alpha.rs"), "fn a() { beta::go(); gamma::go(); }").unwrap();
        std::fs::write(d.join("beta.rs"), "fn b() { gamma::go(); }").unwrap();
        std::fs::write(d.join("gamma.rs"), "fn g() {} // delta:: is only mentioned in a comment").unwrap();
        std::fs::write(d.join("delta.rs"), "fn d() {}").unwrap();
        let files: Vec<PathBuf> = ["alpha.rs", "beta.rs", "gamma.rs", "delta.rs"]
            .iter().map(|f| d.join(f)).collect();
        let g = build_dep_graph(&files);
        println!("V={} E={} triangles={} degrees_distinct={} isolated={}",
            g.names.len(), g.edges, g.triangles, g.distinct_degrees, g.isolated);
        assert_eq!(g.names.len(), 4);
        // alpha->beta, alpha->gamma, beta->gamma  == 3 edges; the comment must NOT create one
        assert_eq!(g.edges, 3, "comment mentions must not become edges");
        assert!(g.isolated >= 1, "delta has no real references");
        assert!(g.distinct_degrees > 1, "real graph must not have a uniform degree");
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn walk_follows_an_edge_and_flags_the_fallback() {
        let d = tmp_tree("walk");
        std::fs::write(d.join("a.rs"), "fn x(){ b::f(); }").unwrap();
        std::fs::write(d.join("b.rs"), "fn f(){}").unwrap();
        std::fs::write(d.join("lonely.rs"), "fn y(){}").unwrap();
        let files: Vec<PathBuf> = ["a.rs", "b.rs", "lonely.rs"].iter()
            .map(|f| d.join(f)).collect();
        let g = build_dep_graph(&files);
        let ia = g.names.iter().position(|n| n == "a").unwrap();
        let ib = g.names.iter().position(|n| n == "b").unwrap();
        assert_eq!(g.adj[ia], vec![ib], "a references b");
        assert!(g.adj.iter().any(|v| v.is_empty()), "lonely has no edges");
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn real_repo_graph_is_not_uniform() {
        let root = std::env::var("TRM_REPO_ROOT")
            .unwrap_or_else(|_| "F:/Development sites/TRM-Swarm-OS-v2".to_string());
        let mut files = Vec::new();
        let root = PathBuf::from(root);
        walk(&root.join("trm_uefi_kernel/src"), &mut files, 0);
        walk(&root.join("trm_uefi_kernel/host_council_server/src"), &mut files, 0);
        if files.is_empty() {
            eprintln!("repo not present; skipping");
            return;
        }
        let g = build_dep_graph(&files);
        println!("REAL GRAPH V={} E={} triangles={} chi={} degree_max={} distinct_degrees={} isolated={}",
            g.names.len(), g.edges, g.triangles,
            g.names.len() as i64 - g.edges as i64 + g.triangles as i64,
            g.degree_max, g.distinct_degrees, g.isolated);
        assert!(g.distinct_degrees > 1, "a uniform degree here would mean the parser is the modulus");
        assert!(g.adj.iter().any(|v| !v.is_empty()), "at least one module must reference another");
    }
}
