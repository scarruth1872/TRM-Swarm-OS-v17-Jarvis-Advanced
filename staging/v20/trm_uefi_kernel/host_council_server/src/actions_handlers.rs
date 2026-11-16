// Rollout 13: the engine-action family, plus the one action this platform can answer with machinery
// it already has.
//
// Every route here fell through to the kernel catch-all ({"status":"ok"}), so twelve dashboard call
// sites reported success while nothing ran. Two deliberate exceptions to a blanket refusal:
//   1. POST /memory/query is wired to the kernel's OWN retrieval (/k3/retrieve over the quantized
//      corpus and the tercode store) and returns its real hits. No new engine was built for it.
//   2. GET is never claimed (handle returns None), because /memory, /ddr, /artifacts and /mailbox
//      already answer and a prefix-blind guard would bury the reads that work.
// Also exported: ddr_antibody_count(), so the three routes that asserted 1,801 can read the store.

use serde::de::{IgnoredAny, SeqAccess, Visitor};
use serde::Deserializer;
use serde_json::{json, Value};
use std::io::Read;
use std::time::{SystemTime, UNIX_EPOCH};
use tiny_http::{Header, Method, Request, Response};

type Resp = Response<std::io::Cursor<Vec<u8>>>;

const ACTIONS: &[(&str, &str)] = &[
    ("/artifacts/review", "the artifact pipeline's review state machine"),
    ("/artifacts/test", "the artifact pipeline's test runner"),
    ("/artifacts/integrate", "the artifact pipeline's integration step"),
    ("/artifacts/batch-review", "the artifact pipeline's batch review"),
    ("/artifacts/batch-integrate", "the artifact pipeline's batch integration"),
    ("/ddr/antibodies/add", "the DDR antibody vault writer"),
    ("/ddr/scan", "the DDR scanner"),
    ("/discovery/pipeline/run", "the discovery pipeline orchestrator"),
    ("/humanistic/tasks/run", "the humanistic task-suite runner"),
    ("/mailbox/send", "the IPC ring writer (agent_mailbox_bus owns the slot protocol)"),
    ("/sensory/mesh/triangulate", "the sensory-mesh triangulation engine"),
];

fn ok(v: Value) -> Resp {
    Response::from_string(v.to_string())
        .with_status_code(200)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap())
}

fn engine_only(path: &str, what: &str) -> Resp {
    Response::from_string(
        json!({ "status": "engine_required", "error": "engine_required", "route": path,
                "message": format!("{path} needs {what}, which runs in the Python engine and has no Rust port. The catch-all used to answer {{\"status\":\"ok\"}} here, so the control looked successful while nothing ran.") })
        .to_string(),
    )
    .with_status_code(501)
    .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap())
}

/// Curated code-scanning antibodies as a list, for the /ddr/antibodies vault view. Unreadable or
/// malformed -> empty, never a fabricated list.
pub fn curated_antibodies() -> Vec<Value> {
    read_antibody_store()
}

/// Entries in the curated code-scanning store (swarm_v2_artifacts/DDR_ANTIBODIES.json). None =
/// unreadable, reported as null rather than replaced by a number nobody measured.
pub fn curated_antibody_count() -> Option<usize> {
    let p = crate::k3_retrieval::repo_relative("swarm_v2_artifacts/DDR_ANTIBODIES.json")?;
    let raw = std::fs::read_to_string(p).ok()?;
    serde_json::from_str::<Value>(&raw).ok()?.as_array().map(|a| a.len())
}

/// Stream-count the entries of a JSON array without materializing the whole document (the
/// threat-matrix store is ~173 MB / ~400k entries; loading it as a Value tree spikes memory).
/// visit_seq + IgnoredAny walks the array in O(1) extra memory, discarding each element as the
/// parser advances past it.
struct ArrayCount;
impl<'de> Visitor<'de> for ArrayCount {
    type Value = usize;
    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "a JSON array")
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<usize, A::Error> {
        let mut n = 0usize;
        while seq.next_element::<IgnoredAny>()?.is_some() {
            n += 1;
        }
        Ok(n)
    }
}

fn count_json_array(path: &std::path::Path) -> Option<usize> {
    let file = std::fs::File::open(path).ok()?;
    let reader = std::io::BufReader::new(file);
    let mut de = serde_json::Deserializer::from_reader(reader);
    de.deserialize_seq(ArrayCount).ok()
}

static THREAT_ANTIBODY_COUNT: std::sync::OnceLock<Option<usize>> = std::sync::OnceLock::new();

/// MITRE/CWE/CVE-derived threat-matrix antibodies (swarm_v2_artifacts/DDR_THREAT_ANTIBODIES.json).
/// Counted once and cached: the store is written by the Python ingest, not the kernel, so its
/// length is stable across the kernel's lifetime. None = unreadable.
pub fn threat_antibody_count() -> Option<usize> {
    *THREAT_ANTIBODY_COUNT.get_or_init(|| {
        let p = crate::k3_retrieval::repo_relative("swarm_v2_artifacts/DDR_THREAT_ANTIBODIES.json")?;
        count_json_array(&p)
    })
}

/// Total antibody inventory = curated + threat-matrix. None only when the curated store is
/// unreadable (the threat store, if unreadable, is surfaced separately as null, never as zero).
pub fn ddr_antibody_count() -> Option<usize> {
    let curated = curated_antibody_count()?;
    let threat = threat_antibody_count();
    Some(match threat {
        Some(t) => curated + t,
        None => curated,
    })
}

/// The canonical code-scanning antibody store (swarm_v2/core/ddr_antibody.py owns it). Unreadable
/// or malformed -> empty, never a fabricated count.
fn read_antibody_store() -> Vec<Value> {
    let p = match crate::k3_retrieval::repo_relative("swarm_v2_artifacts/DDR_ANTIBODIES.json") {
        Some(p) => p,
        None => return Vec::new(),
    };
    let raw = match std::fs::read_to_string(p) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    match serde_json::from_str::<Value>(&raw) {
        Ok(v) => v.as_array().cloned().unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

/// Code-scanning store + threat-matrix store (MITRE/CWE/CVE-derived antibodies), the union the
/// discovery bridge consumes. Order: the 51 curated first, threat-matrix antibodies appended.
fn all_antibodies() -> Vec<Value> {
    let mut v = read_antibody_store();
    if let Some(p) = crate::k3_retrieval::repo_relative("swarm_v2_artifacts/DDR_THREAT_ANTIBODIES.json") {
        if let Ok(raw) = std::fs::read_to_string(p) {
            if let Ok(arr) = serde_json::from_str::<Value>(&raw) {
                if let Some(a) = arr.as_array() {
                    v.extend(a.iter().cloned());
                }
            }
        }
    }
    v
}

/// Strip regex metacharacters so a fixed-pattern antibody matches as a literal substring (the
/// kernel has no regex engine). A backslash-escaped char is a literal (\. -> ., \" -> "), so it is
/// unescaped; the real metacharacters are dropped. 1:1 intent with ddr_antibody.rs::strip_regex.
fn strip_regex(pat: &str) -> String {
    let mut out = String::new();
    let mut chars = pat.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(&n) = chars.peek() {
                out.push(n);
                chars.next();
            }
        } else if !"()[]{}$*+?.|^".contains(c) {
            out.push(c);
        }
    }
    out
}

fn fnv1a64(b: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for &x in b {
        h ^= x as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

fn read_body(request: &mut Request, cap: usize) -> String {
    let mut body = String::new();
    let _ = request.as_reader().take(cap as u64).read_to_string(&mut body);
    body
}

pub fn enc(s: &str) -> String {
    let mut o = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => o.push(b as char),
            b' ' => o.push('+'),
            _ => o.push_str(&format!("%{b:02X}")),
        }
    }
    o
}

/// The kernel's own retrieval, exposed as the memory query the UI already calls. The ranks, scores
/// and fidelities are the ones /k3/retrieve derives; `match_percentage` is the document fidelity,
/// and the raw ranked score travels under `kernel_score` so nothing is rescaled into a claim.
fn memory_query(request: &mut Request) -> Resp {
    let mut body = String::new();
    let _ = request.as_reader().take(65_536).read_to_string(&mut body);
    let parsed: Value = serde_json::from_str(&body).unwrap_or(json!({}));
    let q = parsed.get("query").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
    if q.is_empty() {
        return ok(json!({ "status": "error", "route": "/memory/query", "error": "empty_query",
                          "detail": "POST {\"query\": \"...\"}" }));
    }
    let k = parsed.get("k").and_then(|v| v.as_u64()).unwrap_or(5).clamp(1, 25);
    let (out, code) = crate::k3_retrieval::handle(&format!("/k3/retrieve?q={}&k={k}", enc(&q)));
    if code != 200 {
        return ok(json!({ "status": "unavailable", "route": "/memory/query", "results": [],
                          "engine": out.get("error").cloned().unwrap_or(json!("kernel retrieval unavailable")) }));
    }
    let hits = out.get("results").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    let results: Vec<Value> = hits
        .iter()
        .map(|h| {
            json!({
                "content": h.get("doc").cloned().unwrap_or(json!(null)),
                "source": "kernel K3 index (quantized corpus + tercode store)",
                "match_percentage": h.get("doc_fidelity").cloned().unwrap_or(json!(null)),
                "rank": h.get("rank").cloned().unwrap_or(json!(null)),
                "kernel_score": h.get("score").cloned().unwrap_or(json!(null)),
                "doc_fidelity": h.get("doc_fidelity").cloned().unwrap_or(json!(null)),
                "score_crosschecked": h.get("score_crosschecked").cloned().unwrap_or(json!(null)),
            })
        })
        .collect();
    ok(json!({
        "status": "ok", "route": "/memory/query", "query": q, "k": k,
        "engine": out.get("engine").cloned().unwrap_or(json!(null)),
        "candidates_from": out.get("candidates_from").cloned().unwrap_or(json!(null)),
        "corpus": out.get("corpus").cloned().unwrap_or(json!(null)),
        "elapsed_us": out.get("elapsed_us").cloned().unwrap_or(json!(null)),
        "metric_basis": "match_percentage is the document's fidelity to the query from the kernel's own retrieval; kernel_score is the ranked score before cross-check",
        "results": results,
    }))
}

/// POST /ddr/scan — run the antibody store against a code body, natively. Substring match of each
/// antibody's line_pattern (regex metacharacters stripped) gated by file_pattern suffix, 1:1 with
/// ddr_antibody.rs::check_antibodies but reading the JSON store that Python owns.
fn ddr_scan(request: &mut Request) -> Resp {
    let parsed: Value = serde_json::from_str(&read_body(request, 262_144)).unwrap_or(json!({}));
    let code = parsed.get("code").and_then(|v| v.as_str()).unwrap_or("");
    let filename = parsed.get("filename").and_then(|v| v.as_str()).unwrap_or("");
    if code.is_empty() {
        return ok(json!({ "status": "error", "error": "empty_code",
                          "detail": "POST {\"code\": \"...\", \"filename\": \"...\"}" }));
    }
    let store = read_antibody_store();
    let mut matches: Vec<Value> = Vec::new();
    for ab in &store {
        let file_pat = ab.get("file_pattern").and_then(|v| v.as_str()).unwrap_or("");
        let line_pat = ab.get("line_pattern").and_then(|v| v.as_str()).unwrap_or("");
        if !file_pat.is_empty() && !filename.is_empty() {
            let suffix = file_pat.trim_start_matches("*.");
            if !suffix.is_empty() && !filename.ends_with(suffix) {
                continue;
            }
        }
        let pat = strip_regex(line_pat);
        if !pat.is_empty() && code.contains(&pat) {
            matches.push(json!({
                "antibody_id": ab.get("antibody_id").cloned().unwrap_or(json!(null)),
                "error_type": ab.get("error_type").cloned().unwrap_or(json!(null)),
                "severity": ab.get("severity").cloned().unwrap_or(json!(null)),
                "fix_description": ab.get("fix_description").cloned().unwrap_or(json!(null)),
                "line_pattern": line_pat,
            }));
        }
    }
    ok(json!({
        "status": "ok", "route": "/ddr/scan",
        "antibodies_scanned": store.len(),
        "matches_found": matches.len(),
        "matches": matches,
        "basis": "substring match of each antibody's line_pattern (regex metacharacters stripped) against the code, gated by file_pattern suffix; store = swarm_v2_artifacts/DDR_ANTIBODIES.json",
    }))
}

/// POST /ddr/antibodies/add — append one antibody to the store the Python module owns. Dedupes on
/// (error_type, line_pattern); the store is the source of truth, nothing is replaced.
fn ddr_add_antibody(request: &mut Request) -> Resp {
    let parsed: Value = serde_json::from_str(&read_body(request, 65_536)).unwrap_or(json!({}));
    let error_type = parsed.get("error_type").and_then(|v| v.as_str()).unwrap_or("").to_string();
    if error_type.is_empty() {
        return ok(json!({ "status": "error", "error": "missing_error_type",
            "detail": "POST {\"error_type\":\"...\", \"file_pattern\":\"*.py\", \"line_pattern\":\"...\", \"fix_description\":\"...\", \"severity\":\"critical\"}" }));
    }
    let file_pattern = parsed.get("file_pattern").and_then(|v| v.as_str()).unwrap_or("*.py").to_string();
    let line_pattern = parsed.get("line_pattern").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let fix = parsed.get("fix_description").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let severity = parsed.get("severity").and_then(|v| v.as_str()).unwrap_or("medium").to_string();

    let p = match crate::k3_retrieval::repo_relative("swarm_v2_artifacts/DDR_ANTIBODIES.json") {
        Some(p) => p,
        None => return ok(json!({ "status": "error", "error": "store_unresolvable" })),
    };
    let mut store = read_antibody_store();
    if store.iter().any(|a| {
        a.get("error_type").and_then(|v| v.as_str()) == Some(error_type.as_str())
            && a.get("line_pattern").and_then(|v| v.as_str()) == Some(line_pattern.as_str())
    }) {
        return ok(json!({ "status": "duplicate", "route": "/ddr/antibodies/add",
                          "error_type": error_type, "store_count": store.len() }));
    }
    let now = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let new_ab = json!({
        "antibody_id": format!("ab-rust-{now}-{:x}", fnv1a64(format!("{error_type}|{line_pattern}").as_bytes()) & 0xffff),
        "error_type": error_type,
        "file_pattern": file_pattern,
        "line_pattern": line_pattern,
        "fix_description": fix,
        "severity": severity,
        "recorded_by": "ddr_antibody_vault_rust",
        "created_at": now,
        "matches_prevented": 0,
    });
    store.push(new_ab.clone());
    let store_count = store.len();
    if let Ok(s) = serde_json::to_string_pretty(&Value::Array(store)) {
        let _ = std::fs::write(&p, s);
    }
    ok(json!({
        "status": "ok", "route": "/ddr/antibodies/add",
        "antibody": new_ab, "store_count": store_count,
        "basis": "appended to swarm_v2_artifacts/DDR_ANTIBODIES.json (dedupe on error_type+line_pattern)",
    }))
}

// ── DDR-Drug Discovery Bridge (native port of ddr_amp_discovery_engine.py) ──
// Deterministic antibody -> antimicrobial peptide mapping + in-silico physicochemical scoring.
// Hash is FNV-1a-64 (Python used SHA-256, so sequences differ); Kd/confidence are heuristic
// in-silico predictions, NOT measured binding — labeled as such in every candidate.

const KD: [f64; 20] = [
    1.8, 2.5, -3.5, -3.5, 2.8, -0.4, -3.2, 4.5, -3.9, 3.8,
    1.9, -3.5, -1.6, -3.5, -4.5, -0.8, -0.7, 4.2, -0.9, -1.3,
];
const CHARGE: [f64; 20] = [
    0.0, 0.0, -1.0, -1.0, 0.0, 0.0, 0.1, 0.0, 1.0, 0.0,
    0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0,
];
const BOMAN: [f64; 20] = [
    -1.81, -1.28, 8.72, 6.81, -2.98, -0.94, 4.66, -4.92, 5.55, -4.92,
    -1.50, 6.64, 0.0, 5.54, 9.38, 3.40, 2.57, -4.04, -2.33, -0.14,
];

fn aa_idx(aa: u8) -> Option<usize> {
    Some(match aa {
        b'A' => 0, b'C' => 1, b'D' => 2, b'E' => 3, b'F' => 4, b'G' => 5,
        b'H' => 6, b'I' => 7, b'K' => 8, b'L' => 9, b'M' => 10, b'N' => 11,
        b'P' => 12, b'Q' => 13, b'R' => 14, b'S' => 15, b'T' => 16, b'V' => 17,
        b'W' => 18, b'Y' => 19,
        _ => return None,
    })
}

struct Target {
    name: &'static str,
    organism: &'static str,
    charge_compl: f64,
    hydrophobicity: f64,
    anchors: &'static [u8],
    moa: &'static str,
}

const TARGETS: [Target; 3] = [
    Target { name: "LpxC", organism: "Pseudomonas aeruginosa / Acinetobacter baumannii",
        charge_compl: -3.2, hydrophobicity: 0.42, anchors: &[b'H', b'C', b'D', b'E'],
        moa: "Chelates catalytic Zn2+ cofactor and occludes UDP-3-O-acyl-GlcNAc substrate channel." },
    Target { name: "BamA", organism: "Escherichia coli / Klebsiella pneumoniae (Carbapenem-resistant)",
        charge_compl: -1.8, hydrophobicity: 0.58, anchors: &[b'W', b'F', b'Y', b'R'],
        moa: "Intercalates into the lateral gate seam, blocking outer-membrane beta-barrel insertion." },
    Target { name: "FtsZ", organism: "Staphylococcus aureus (MRSA) / Enterococcus faecium (VRE)",
        charge_compl: -2.5, hydrophobicity: 0.38, anchors: &[b'K', b'R', b'G', b'T'],
        moa: "Binds GTPase inter-domain cleft, arresting Z-ring polymerization and bacterial septation." },
];

fn next_byte(state: &mut u64) -> u8 {
    let mut x = *state;
    x ^= x >> 12;
    x ^= x << 25;
    x ^= x >> 27;
    *state = x;
    (x.wrapping_mul(0x2545F4914F6CDD1D) >> 56) as u8
}

fn map_peptide(ab: &Value, ti: usize) -> String {
    let ab_id = ab.get("antibody_id").and_then(|v| v.as_str()).unwrap_or("ab-seed");
    let fix = ab.get("fix_description").and_then(|v| v.as_str()).unwrap_or("");
    let line = ab.get("line_pattern").and_then(|v| v.as_str()).unwrap_or("");
    let t = &TARGETS[ti];
    let mut state = fnv1a64(format!("{ab_id}|{fix}|{line}|{}", t.name).as_bytes());
    let length = 14 + ((state >> 56) as usize % 9);
    let anchors = t.anchors;
    let mut seq: Vec<u8> = Vec::with_capacity(length);
    for i in 0..length {
        let b = next_byte(&mut state) as usize;
        let aa: u8 = if i % 4 == 0 || i % 4 == 2 {
            if b % 3 == 0 { anchors[b % anchors.len()] }
            else if b % 2 == 0 { b"KR"[b % 2] }
            else { b"STNQGY"[b % 6] }
        } else {
            b"LIVFWA"[b % 6]
        };
        seq.push(aa);
    }
    if seq[0] != b'K' && seq[0] != b'R' {
        seq[0] = if next_byte(&mut state) % 2 == 0 { b'K' } else { b'R' };
    }
    String::from_utf8_lossy(&seq).into_owned()
}

fn compute_props(seq: &str) -> (f64, f64, f64, f64) {
    let n = seq.len() as f64;
    let mut charge = 0.0;
    let mut hydro_count = 0.0;
    let mut boman = 0.0;
    let mut sum_sin = 0.0;
    let mut sum_cos = 0.0;
    for (i, &b) in seq.as_bytes().iter().enumerate() {
        let idx = match aa_idx(b) { Some(x) => x, None => continue };
        let kd = KD[idx];
        charge += CHARGE[idx];
        if kd > 0.0 { hydro_count += 1.0; }
        boman += BOMAN[idx];
        let ang = (i as f64) * 100.0_f64.to_radians();
        sum_sin += kd * ang.sin();
        sum_cos += kd * ang.cos();
    }
    let moment = (sum_sin * sum_sin + sum_cos * sum_cos).sqrt() / n;
    (charge, hydro_count / n, boman / n, moment)
}

fn predict_kd(net_q: f64, hydro: f64, moment: f64, ti: usize) -> (f64, f64) {
    let t = &TARGETS[ti];
    let q_score = (net_q * t.charge_compl.abs() / 20.0).clamp(0.0, 1.0);
    let hyd_score = (1.0 - (hydro - t.hydrophobicity).abs() / 0.30).clamp(0.0, 1.0);
    let moment_score = (moment / 0.60).clamp(0.0, 1.0);
    let composite = 0.40 * q_score + 0.35 * hyd_score + 0.25 * moment_score;
    let kd = 10f64.powf(3.5 - 3.2 * composite);
    let conf = (composite * 0.98).clamp(0.70, 0.99);
    (kd, conf)
}

/// POST /discovery/pipeline/run — the antibody -> drug-discovery bridge, native. Maps every
/// antibody (code-scanning + threat-matrix) to an AMP seed, scores it in-silico, keeps the
/// viable ones, writes the top 100 to antimicrobial_peptide_candidates.json. The other 7 paper
/// stages (docking, ADMET, repurposing, circuits, neoantigens, ADC) are NOT re-run here — those
/// artifacts are pre-existing; this route regenerates the one stage that is deterministic.
fn discovery_pipeline_run(_request: &mut Request) -> Resp {
    let abs = all_antibodies();
    let mut candidates: Vec<Value> = Vec::new();
    for (idx, ab) in abs.iter().enumerate() {
        let ti = idx % 3;
        let seq = map_peptide(ab, ti);
        let (net_q, hydro, boman, moment) = compute_props(&seq);
        if net_q >= 2.0 && (0.28..=0.62).contains(&hydro) {
            let (kd, conf) = predict_kd(net_q, hydro, moment, ti);
            let t = &TARGETS[ti];
            candidates.push(json!({
                "candidate_id": format!("TRM-AMP-{:04}", idx + 1),
                "sequence": seq,
                "length": seq.len(),
                "target_protein": t.name,
                "target_organism": t.organism,
                "predicted_kd_nm": (kd * 100.0).round() / 100.0,
                "net_charge": (net_q * 100.0).round() / 100.0,
                "hydrophobic_ratio": (hydro * 1000.0).round() / 1000.0,
                "boman_index_kcal_mol": (boman * 100.0).round() / 100.0,
                "amphipathic_moment": (moment * 1000.0).round() / 1000.0,
                "k3_confidence": (conf * 1000.0).round() / 1000.0,
                "derived_from_antibody": ab.get("antibody_id").cloned().unwrap_or(json!(null)),
                "ddr_source_error_type": ab.get("error_type").cloned().unwrap_or(json!(null)),
                "mechanism_of_action": t.moa,
                "prediction_basis": "in-silico heuristic (charge/hydrophobicity/moment -> Kd); NOT a measured binding affinity",
            }));
        }
    }
    candidates.sort_by(|a, b| {
        let ak = a.get("predicted_kd_nm").and_then(|v| v.as_f64()).unwrap_or(f64::MAX);
        let bk = b.get("predicted_kd_nm").and_then(|v| v.as_f64()).unwrap_or(f64::MAX);
        let ac = a.get("k3_confidence").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let bc = b.get("k3_confidence").and_then(|v| v.as_f64()).unwrap_or(0.0);
        ak.partial_cmp(&bk).unwrap_or(std::cmp::Ordering::Equal)
            .then(bc.partial_cmp(&ac).unwrap_or(std::cmp::Ordering::Equal))
    });
    let viable = candidates.len();

    candidates.truncate(100);
    let top = candidates.len();
    let now = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let out = json!({
        "monad_anchor": "0x4D4F4E4144000001",
        "generated_at": now,
        "total_antibodies_scanned": abs.len(),
        "top_candidates_count": top,
        "target_spectrum": ["LpxC", "BamA", "FtsZ"],
        "prediction_basis": "deterministic antibody->peptide map (FNV-1a-64) + in-silico heuristic scoring; Kd/confidence are computed predictions, NOT measured binding",
        "candidates": candidates,
    });
    if let Some(p) = crate::k3_retrieval::repo_relative("swarm_v2_artifacts/antimicrobial_peptide_candidates.json") {
        if let Ok(s) = serde_json::to_string_pretty(&out) {
            let _ = std::fs::write(p, s);
        }
    }
    ok(json!({
        "status": "ok", "route": "/discovery/pipeline/run",
        "antibodies_scanned": abs.len(),
        "viable_candidates": viable,
        "top_candidates_written": top,
        "artifact": "swarm_v2_artifacts/antimicrobial_peptide_candidates.json",
        "stages": {
            "ran_natively": ["antibody_intake", "amp_discovery_and_scoring"],
            "not_rerun_here": ["binding_docking", "antibody_evolution", "drug_repurposing", "synthetic_circuits", "cancer_neoantigens", "toxicity_profiling", "adc_engineering"],
            "basis": "only the deterministic antibody->AMP mapping is regenerated; the other stages read pre-existing artifacts and are not re-derived by this route"
        },
    }))
}

pub fn handle(request: &mut Request, url: &str) -> Option<Resp> {
    let method = request.method().clone();
    let path = url.split('?').next().unwrap_or(url);
    if method != Method::Post && method != Method::Delete {
        return None; // never shadow a read
    }
    if path == "/memory/query" {
        return Some(memory_query(request));
    }
    // Native Rust ports of the DDR + discovery write routes (replacing the honest 501s). These
    // now run in the kernel, so the refusal table below no longer shadows them.
    if path == "/ddr/scan" {
        return Some(ddr_scan(request));
    }
    if path == "/ddr/antibodies/add" {
        return Some(ddr_add_antibody(request));
    }
    if path == "/discovery/pipeline/run" {
        return Some(discovery_pipeline_run(request));
    }
    if let Some((p, what)) = ACTIONS.iter().find(|(p, _)| *p == path) {
        return Some(engine_only(p, what));
    }
    // Siblings of the same class: a POST/DELETE under one of these prefixes is an engine action
    // whether or not it was in the audited list. Unknown prefixes fall through untouched.
    let what = if path.starts_with("/artifacts") {
        "the artifact pipeline engine"
    } else if path.starts_with("/ddr") {
        "the DDR antibody engine"
    } else if path.starts_with("/discovery") {
        "the discovery pipeline orchestrator"
    } else if path.starts_with("/humanistic") {
        "the humanistic task-suite runner"
    } else if path.starts_with("/mailbox") || path.starts_with("/comms") {
        "the IPC ring writer (agent_mailbox_bus)"
    } else if path.starts_with("/memory") {
        "the memory engine"
    } else if path.starts_with("/sensory") {
        "the sensory-mesh triangulation engine"
    } else {
        return None;
    };
    Some(engine_only(path, what))
}
