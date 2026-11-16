//! `/k3/*` routes — the kernel's own ternary retrieval engine, exposed on :8022.
//!
//! The platform does not answer by sampling a token. It quantizes the query into balanced
//! ternary trits and scores it against a corpus quantized the same way, by the kernel's own
//! primitives: `native_ternary` (a port of `swarm_v2_rust_kernel/src/ternary_knowledge_index.rs`)
//! and `native_encoder` (a port of that kernel's model-free query encoder). Each port is diffed
//! against the DLL export it came from by its own `parity` test, so the number the route reports
//! is still the number the kernel computed -- and it computes it on the machine the platform
//! boots on, where the DLL (a Windows PE) has never existed.
//!
//! Every failure is reported as a failure. A missing corpus or an unreachable encoder returns 503
//! with a named reason; there is no fallback that silently answers from a different path.

use crate::packed_ternary;
use serde_json::{json, Value};
use std::path::PathBuf;

/// Corpus this route scores against, relative to the repository root.
const INDEX_REL: &str = "swarm_v2_artifacts/kernel_knowledge_index.json";
/// Engine the route calls, relative to the repository root. Same artifact the Python
/// benchmark and the exports probe verified.
const ENGINE_REL: &str = "swarm_v2_rust_kernel/target/release/swarm_rust_kernel.dll";
/// The quantization multiplier the GREEN kernel-intelligence benchmark measured with.
/// Not tuned here: the route reproduces the benchmarked configuration or it says nothing.
const QUANT_MULTIPLIER: f32 = 1.0;
/// The encoder that built the index artifact (its meta.model). The query must be encoded
/// by the SAME encoder or the two live in different spaces. No model server is involved.
const ENCODER_NAME: &str = "native-turbovec-hdc";
const DIM: usize = 768;

type QuantizeFn = unsafe extern "C" fn(*const f32, usize, *mut i8, f32, *mut f32) -> usize;
type SearchFn = unsafe extern "C" fn(
    *const i8,
    usize,
    *const i8,
    usize,
    usize,
    *mut usize,
    *mut i32,
    *mut usize,
) -> i32;
type AgreementFn = unsafe extern "C" fn(*const i8, *const i8, usize) -> i32;
type MagnitudeFn = unsafe extern "C" fn(*const i8, usize) -> i32;
type FidelityFn = unsafe extern "C" fn(*const f32, *const i8, f32, usize) -> f32;

/// Resolve a repo-relative path by walking up from the running executable until the
/// relative path exists. Derived from the process's own location, never hardcoded.
pub fn repo_relative(rel: &str) -> Option<PathBuf> {
    let mut dir = std::env::current_exe().ok()?;
    for _ in 0..8 {
        dir = dir.parent()?.to_path_buf();
        let cand = dir.join(rel);
        if cand.exists() {
            return Some(cand);
        }
    }
    None
}

/// The repo root, found the same way `repo_relative` walks but without requiring the target to
/// exist -- a store being written for the first time does not exist yet.
pub fn repo_root() -> Option<PathBuf> {
    let mut dir = std::env::current_exe().ok()?;
    for _ in 0..8 {
        dir = dir.parent()?.to_path_buf();
        if dir.join("swarm_v2_artifacts").is_dir() || dir.join("trm_uefi_kernel").is_dir() {
            return Some(dir);
        }
    }
    None
}

// ── Engine + corpus: resolved once, on the first /k3 request ───────────────────────────

/// A resolved engine: the kernel's ternary primitives (in-tree, via `native_ternary`) plus the
/// corpus quantized by them.
/// Built whole or not at all — a partially constructed engine would be a fallback path.
struct Engine {
    quantize: QuantizeFn,
    search: SearchFn,
    agreement: AgreementFn,
    magnitude: MagnitudeFn,
    fidelity: FidelityFn,
    /// The parity oracle, when this build can see it: the DLL artifact these primitives were
    /// diffed against by `native_ternary::parity`. Reported so a response can say whether the
    /// reference artifact is present on THIS machine rather than naming one that is not --
    /// absent on the bare-metal build, which is a fact about the build, not a defect in it.
    oracle: Option<(String, u64)>,
    keys: Vec<String>,
    /// n × DIM trits, row-major — exactly the layout `rust_ternary_index_search` reads.
    corpus: Vec<i8>,
    /// Per-document fidelity: the measured cosine loss of that row's quantization.
    fidelity_by_doc: Vec<f32>,
    index_rel: String,
}

unsafe impl Send for Engine {}
unsafe impl Sync for Engine {}

fn load_engine() -> Result<Engine, String> {
    // The primitives are in-tree: `native_ternary` is a port of the kernel module the DLL was
    // built from, and its `parity` test diffs every one of them against that DLL's own exports
    // on Windows. The corpus below is either the compact trit store or plain JSON, so the engine
    // resolves on every platform -- including the one the platform boots on, where the DLL load
    // used to refuse outright.
    let (quantize, search, agreement, magnitude, fidelity): (
        QuantizeFn,
        SearchFn,
        AgreementFn,
        MagnitudeFn,
        FidelityFn,
    ) = (
        crate::native_ternary::nt_quantize,
        crate::native_ternary::nt_search,
        crate::native_ternary::nt_agreement,
        crate::native_ternary::nt_magnitude,
        crate::native_ternary::nt_fidelity,
    );
    // The oracle is REPORTED, not required: whether the reference artifact is present on THIS
    // machine. Absent on the bare-metal build, which is a fact about the build, not a fault.
    let oracle = repo_relative(ENGINE_REL)
        .and_then(|p| std::fs::metadata(&p).ok().map(|m| (p.display().to_string(), m.len())));

    let index =
        repo_relative(INDEX_REL).ok_or_else(|| format!("corpus not found: {INDEX_REL}"))?;

    // Fail closed, not out of memory: the packed TurboVec store is the primary path, and the exact
    // ternary path over the f32 JSON does not scale to the full corpus (4.49 GB at 183,879 docs).
    const MAX_JSON_LOAD: u64 = 512 * 1024 * 1024;
    if std::fs::metadata(&index).map(|m| m.len()).unwrap_or(0) > MAX_JSON_LOAD {
        return Err(format!(
            "corpus JSON is {} bytes, over the {} byte exact-ternary limit — the packed TurboVec store is primary",
            std::fs::metadata(&index).map(|m| m.len()).unwrap_or(0),
            MAX_JSON_LOAD
        ));
    }

    {
        let raw: Value = serde_json::from_slice(
            &std::fs::read(&index).map_err(|e| format!("cannot read corpus: {e}"))?,
        )
        .map_err(|e| format!("corpus is not JSON: {e}"))?;
        // The artifact wraps its rows under an "index" member; accept either shape so the
        // route reads the artifact as built rather than as assumed.
        let obj = match raw.get("index").and_then(|i| i.as_object()) {
            Some(inner) => inner,
            None => raw.as_object().ok_or("corpus is not keyed by document")?,
        };

        let mut keys: Vec<String> = Vec::new();
        let mut corpus: Vec<i8> = Vec::new();
        let mut fidelity_by_doc: Vec<f32> = Vec::new();
        for (key, val) in obj {
            let emb = match val.get("embedding").and_then(|e| e.as_array()) {
                Some(a) if a.len() == DIM => a,
                _ => continue,
            };
            let f: Vec<f32> = emb.iter().map(|x| x.as_f64().unwrap_or(0.0) as f32).collect();
            let mut trits = vec![0i8; DIM];
            let mut scale = 0f32;
            let written = unsafe {
                quantize(
                    f.as_ptr(),
                    DIM,
                    trits.as_mut_ptr(),
                    QUANT_MULTIPLIER,
                    &mut scale,
                )
            };
            if written != DIM {
                return Err(format!("quantizer wrote {written} of {DIM} trits"));
            }
            let fid = unsafe { fidelity(f.as_ptr(), trits.as_ptr(), scale, DIM) };
            keys.push(key.clone());
            corpus.extend_from_slice(&trits);
            fidelity_by_doc.push(fid);
        }
        if keys.is_empty() {
            return Err(format!(
                "corpus holds no {DIM}-dim document (examined {} entries)",
                obj.len()
            ));
        }

        Ok(Engine {
            quantize,
            search,
            agreement,
            magnitude,
            fidelity,
            oracle,
            keys,
            corpus,
            fidelity_by_doc,
            index_rel: INDEX_REL.to_string(),
        })
    }
}

/// The corpus, reloaded whenever the index file has changed since it was last read.
static ENGINE: crate::file_cache::Slot<Engine> = std::sync::Mutex::new(None);

/// The corpus's own census: how many documents the engine holds, and the leading path segment of
/// each key.
///
/// Read from the engine's already-loaded key list, so a status route costs no second parse of the
/// 53 MB index. The leading segment is the corpus's own filing scheme (`papers/`, `research/`, ...),
/// which is a measured breakdown rather than an invented taxonomy. None when the corpus could not be
/// loaded -- an unreadable corpus is not an empty one.
pub fn corpus_census() -> Option<(usize, std::collections::BTreeMap<String, u64>)> {
    // The packed TurboVec store is the primary corpus. The census reads its key list directly — no
    // f32 JSON load, which would OOM at the full 4.49 GB corpus.
    let store = crate::packed_ternary::store()?;
    let mut by_type: std::collections::BTreeMap<String, u64> = std::collections::BTreeMap::new();
    for k in store.keys() {
        // The corpus's own filing scheme. A key with no directory is a root-level document, not a
        // type named after itself -- returning the whole filename there produced 55 one-count
        // "types" instead of the 4 real segments.
        let seg = match k.split_once('/') {
            Some((dir, _)) => dir,
            None => "root",
        };
        *by_type.entry(seg.to_string()).or_insert(0) += 1;
    }
    Some((store.len(), by_type))
}

fn engine() -> Result<std::sync::Arc<Engine>, String> {
    let p = repo_relative(INDEX_REL).ok_or_else(|| format!("corpus not found: {INDEX_REL}"))?;
    crate::file_cache::get(&ENGINE, &p, |_| load_engine())
}

// ── Encoder: the kernel's own model-free text encoder, in-tree ─────────────────────────
//
// The corpus is built with `native-turbovec-hdc` (index artifact meta.model). A query
// encoded by anything else lives in a different space: measured, an Ollama-embedded query
// scored 76..81 against this native index and surfaced CSS files, while the kernel's own
// encoder surfaced the topically correct documents. So the route uses the kernel encoder
// and needs no model server at all.

/// Encode a query with the in-tree encoder.
///
/// The DLL's `rust_native_encode_text_f32` export is deliberately NOT called here any more: the load
/// is `#[cfg(windows)]`, so on the bare-metal build this returned "wired for windows only" and every
/// `/k3/retrieve` answered 503 `engine_required`. `native_encoder` is a port of that same algorithm
/// and its `encoder_parity` test diffs the two bit for bit against the DLL on this machine -- one
/// encoder on both platforms, rather than a fallback that quietly answers from a different space.
fn embed(text: &str) -> Result<Vec<f32>, String> {
    if text.as_bytes().len() != text.len() {
        return Err("encoder rejected a non-utf8 query".to_string());
    }
    let out = crate::native_encoder::encode_f32(text, DIM);
    if out.len() != DIM {
        return Err(format!("encoder wrote {} of {DIM} dims", out.len()));
    }
    Ok(out)
}

// ── Route ──────────────────────────────────────────────────────────────────────────────

/// Percent-decode a query-string component, treating `+` as a space (form convention).
/// Works on bytes so a multi-byte character can never be sliced mid-codepoint.
fn pct_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'%' if i + 2 < b.len() => {
                let d = |c: u8| (c as char).to_digit(16);
                match (d(b[i + 1]), d(b[i + 2])) {
                    (Some(h), Some(l)) => {
                        out.push((h * 16 + l) as u8);
                        i += 3;
                    }
                    _ => {
                        out.push(b'%');
                        i += 1;
                    }
                }
            }
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            c => {
                out.push(c);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).to_string()
}

/// Read one query-string parameter; `None` when absent.
fn query_param(url: &str, name: &str) -> Option<String> {
    let q = url.split_once('?')?.1;
    for pair in q.split('&') {
        let (k, v) = match pair.split_once('=') {
            Some(kv) => kv,
            None => (pair, ""),
        };
        if k == name {
            return Some(pct_decode(v));
        }
    }
    None
}

/// Entry point for the `handle_request` ladder. Always returns a payload and a status,
/// so a caller never has to invent one.
/// /k3/status -- the K3 store's own state, derived from the bytes on disk.
///
/// Every field names its source, because the K3 namespace was documented in the README while the
/// routes behind it 404'd. A bounded audit states its scope, and a store that cannot be read
/// reports `unscannable` rather than a clean empty result.
pub fn status_handle() -> (Value, u16) {
    let mut out = json!({
        "status": "ok",
        "route": "/k3/status",
        "python_active": false,
        "kernel": "Native Rust Sovereign Kernel v20.0.0",
    });

    match fabric_store() {
        Some((path, bytes)) => {
            out["fabric"] = json!({ "store": path, "bytes": bytes, "trits_if_2_per_byte": bytes * 2 });
            // Bounded content audit. Codes are the platform's 2-bit trit codes: 00 = -1, 01 = 0,
            // 11 = +1, 10 never legal (a 4-level/2-bit TurboVec code appears here instead when a
            // vector index has overwritten the trit store -- the ascii prefix below tells which).
            if let Ok(mut f) = std::fs::File::open(&path) {
                let mut buf = vec![0u8; 4096];
                let n = std::io::Read::read(&mut f, &mut buf).unwrap_or(0);
                buf.truncate(n);
                let prefix: String = buf
                    .iter()
                    .take(8)
                    .map(|b| if b.is_ascii_graphic() { *b as char } else { '.' })
                    .collect();
                // The store's OWN encoding: one nibble (4 bits) per trit, 2 trits/byte, legal
                // nibbles 0b0000 = -1, 0b0001 = 0, 0b0011 = +1 (nand_persistence.rs). Auditing
                // 2-bit fields instead is a DIFFERENT alphabet and reports a number that means
                // nothing for this store.
                let (mut legal, mut illegal) = (0u64, 0u64);
                let (mut pos, mut zero, mut neg) = (0i64, 0i64, 0i64);
                for b in &buf {
                    for nib in [b & 0x0F, (b >> 4) & 0x0F] {
                        match nib {
                            0b0011 => {
                                pos += 1;
                                legal += 1;
                            }
                            0b0001 => {
                                zero += 1;
                                legal += 1;
                            }
                            0b0000 => {
                                neg += 1;
                                legal += 1;
                            }
                            _ => illegal += 1,
                        }
                    }
                }
                // The monad anchor as a DERIVED check, not a byte grep: its trit projection occupies
                // the genesis seal (trit indices 0..31 = bytes 0..15). Re-derive the 64-bit value
                // from those trits and compare.
                const ANCHOR: [u8; 8] = [0x4D, 0x4F, 0x4E, 0x41, 0x44, 0x00, 0x00, 0x01];
                let mut rec = [0u8; 8];
                for i in 0..32usize {
                    let b = buf[i / 2];
                    let nib = if i % 2 == 1 { (b >> 4) & 0x0F } else { b & 0x0F };
                    let code: u8 = match nib {
                        0b0011 => 0b11,
                        0b0001 => 0b01,
                        0b0000 => 0b00,
                        _ => 0b01, // an illegal nibble cannot be vouched for; read as 0, not as TRUE
                    };
                    rec[i / 4] |= code << ((i % 4) * 2);
                }
                let diff: Vec<usize> = (0..8).filter(|&i| rec[i] != ANCHOR[i]).collect();
                out["fabric"]["content_sample"] = json!({
                    "bytes_read": buf.len(),
                    "ascii_prefix": prefix,
                    "encoding": "one nibble per trit, 2 trits/byte (the store's own rule)",
                    "legal_nibbles": "0b0000=-1, 0b0001=0, 0b0011=+1",
                    "trits_audited": legal + illegal,
                    "trits_legal": legal,
                    "trits_illegal": illegal,
                    "illegal_pct": if legal + illegal > 0 {
                        (illegal * 10000 / (legal + illegal)) as f64 / 100.0
                    } else { 0.0 },
                    "max_illegal_nibble_fraction_allowed": 0.0,
                    "trit_shape": { "positive": pos, "zero": zero, "negative": neg },
                    "scope": "first 4 KiB only - not a whole-store audit",
                });
                out["fabric"]["monad_anchor"] = json!({
                    "expected": "0x4D4F4E4144000001",
                    "read_back_from_genesis_seal": format!(
                        "0x{}",
                        rec.iter().map(|b| format!("{b:02X}")).collect::<String>()
                    ),
                    "byte_identical": diff.is_empty(),
                    "differing_bytes": diff,
                    "seal_written_by": "rust_nand_append_decision (the store's owning writer), 32 trits at indices 0..31",
                    "note": "the anchor is stored as its TRIT PROJECTION; 0x4E carries the 2-bit group 0b10, which the engine's map cannot produce, so that byte reads back as 0x4D",
                });
            } else {
                out["fabric"]["content_sample"] = json!({ "error": "unreadable - not a clean result" });
            }
        }
        None => {
            out["fabric"] = json!({ "error": "store_not_found" });
        }
    }

    // The tercode store keeps the engine's own ternary alphabet, so it ranks identically to the
    // engine; report its identity rather than assuming the .k3 path holds trits.
    match repo_relative("swarm_v2_nand_fabric.tercode") {
        Some(p) => match std::fs::metadata(&p) {
            Ok(md) => out["tercode"] = json!({ "store": p.display().to_string(), "bytes": md.len() }),
            Err(_) => out["tercode"] = json!({ "error": "unreadable" }),
        },
        None => out["tercode"] = json!({ "error": "store_not_found" }),
    }

    // The IPC transport, so K3 state and bus state are observable from one place.
    match repo_relative("swarm_mailbox.ring") {
        Some(p) => match std::fs::metadata(&p) {
            Ok(md) => out["ipc"] = json!({
                "transport": "shared-memory ring bus",
                "store": p.display().to_string(),
                "bytes": md.len(),
                "detail_route": "/mailbox/ring?detail=1",
            }),
            Err(_) => out["ipc"] = json!({ "error": "unreadable" }),
        },
        None => out["ipc"] = json!({ "error": "store_not_found" }),
    }

    out["routes"] = json!([
        "/k3/status",
        "/k3/retrieve?q=<text>&k=<n>",
        "/k3/retrieve?q=<text>&engine=packed&verify=1",
        "/k3/retrieve?q=<text>&engine=tercode&verify=1",
        "/k3/retrieve?q=<text>&engine=packed&alphabet=trit&verify=1",
        "/k3/tercode/build",
    ]);
    // Which engine answers, and whether the reference artifact this build's parity test diffs against
    // is on THIS machine. A measurement, not a requirement: the algorithm is in-tree on every target,
    // so a status route that could not name its engine would leave a reader guessing which build
    // answered -- and one that named a path the host lacks would be the declared-vs-measured defect.
    out["engine"] = match engine() {
        Ok(eng) => json!({
            "algorithm": "native-ternary (in-tree port of ternary_knowledge_index.rs)",
            "parity_oracle_present": eng.oracle.is_some(),
            "parity_oracle": eng
                .oracle
                .as_ref()
                .map(|(p, b)| json!({ "path": p, "bytes": b })),
            "quantizer_multiplier": QUANT_MULTIPLIER,
            "corpus": { "source": eng.index_rel, "documents": eng.keys.len(), "dim": DIM },
        }),
        Err(e) => json!({ "error": e }),
    };
    (out, 200)
}

/// /k3/verdict — the Kleene truth proof the service catalog advertises. Deterministic K3 logic over
/// supplied trits, or over trits the engine's own quantizer derives from a query: no model call, so
/// this is not engine work and must not be stubbed or 501'd.
pub fn verdict_payload(trits: &[i8], source: &str) -> Value {
    let (mut t, mut u, mut f) = (0i64, 0i64, 0i64);
    for &x in trits {
        match x {
            1 => t += 1,
            -1 => f += 1,
            _ => u += 1,
        }
    }
    let n = trits.len() as i64;
    // Kleene AND: FALSE if any element is FALSE, TRUE only if every element is TRUE, else UNKNOWN.
    let verdict: i8 = if n == 0 {
        0
    } else if f > 0 {
        -1
    } else if u > 0 {
        0
    } else {
        1
    };
    json!({
        "verdict": verdict,
        "verdict_name": match verdict { 1 => "TRUE", -1 => "FALSE", _ => "UNKNOWN" },
        "trits": trits,
        "counts": { "true": t, "unknown": u, "false": f },
        "truth_density": if n > 0 { t as f64 / n as f64 } else { 0.0 },
        "rule": "Kleene AND: FALSE if any element is FALSE; TRUE only if every element is TRUE; UNKNOWN otherwise",
        "trit_source": source,
    })
}

/// Parse a comma-separated trit spec; values are clamped to the alphabet {-1, 0, +1}.
fn parse_trits(spec: &str) -> Vec<i8> {
    spec.split(',')
        .filter_map(|p| p.trim().parse::<i32>().ok())
        .map(|v| if v > 0 { 1 } else if v < 0 { -1 } else { 0 })
        .collect()
}

/// Validate a body's `trits` array and evaluate it. Used by the advertised POST form.
pub fn verdict_from_body(body: &str) -> Value {
    match serde_json::from_str::<Value>(body) {
        Ok(v) => {
            let trits: Vec<i8> = v
                .get("trits")
                .and_then(|a| a.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|x| x.as_i64())
                        .map(|v| if v > 0 { 1 } else if v < 0 { -1 } else { 0 })
                        .collect()
                })
                .unwrap_or_default();
            if trits.is_empty() {
                json!({ "status": "error", "route": "/k3/verdict", "error": "missing_trits",
                        "detail": "POST {\"trits\": [1, 0, -1, ...]} with at least one element" })
            } else {
                verdict_payload(&trits, "supplied POST body trits")
            }
        }
        Err(e) => json!({ "status": "error", "route": "/k3/verdict", "error": "invalid_body",
                          "detail": format!("expected JSON {{\"trits\": [...]}}: {e}") }),
    }
}

fn verdict_handle(url: &str) -> Value {
    if let Some(spec) = query_param(url, "trits") {
        let trits = parse_trits(&spec);
        if trits.is_empty() {
            return json!({ "status": "error", "route": "/k3/verdict", "error": "empty_triter",
                           "detail": "?trits= needs at least one of -1, 0, +1" });
        }
        return verdict_payload(&trits, "supplied ?trits=...");
    }
    if let Some(q) = query_param(url, "q") {
        if let (Ok(eng), Ok(qv)) = (engine(), embed(&q)) {
            let mut tr: Vec<i8> = vec![0i8; DIM];
            let mut scale = 0f32;
            let written = unsafe {
                (eng.quantize)(qv.as_ptr(), DIM, tr.as_mut_ptr(), QUANT_MULTIPLIER, &mut scale)
            };
            if written == DIM {
                let mut v = verdict_payload(&tr, "the engine's own quantization of ?q=");
                v["quantizer_scale"] = json!(scale);
                v["query"] = json!(q);
                return v;
            }
        }
        return json!({ "status": "error", "route": "/k3/verdict", "error": "engine_unavailable",
                       "detail": "could not quantize ?q=; the engine artifact is required for that path. Pass ?trits=1,0,-1 instead." });
    }
    json!({ "status": "error", "route": "/k3/verdict", "error": "missing_input",
            "detail": "pass ?trits=<comma-separated -1|0|1> or ?q=<text>, or POST {\"trits\": [...]}",
            "example": "/k3/verdict?trits=1,1,0,-1" })
}

pub fn handle(url: &str) -> (Value, u16) {
    // The store builder lives beside the retrieval route, not under it, so it is dispatched before
    // the prefix guard below -- otherwise it can never be reached.
    if url.starts_with("/k3/verdict") {
        return (verdict_handle(url), 200);
    }
    if url.starts_with("/k3/stores") {
        return stores_handle();
    }
    if url.starts_with("/k3/status") || url.starts_with("/k3/stats") {
        return status_handle();
    }
    if url.starts_with("/k3/tercode") {
        return tercode_handle(url);
    }
    if !url.starts_with("/k3/retrieve") {
        return (
            json!({
                "status": "error",
                "route": "/k3",
                "error": "unknown_k3_route",
                "routes": [
                    "/k3/retrieve?q=<text>&k=<n>",
                    "/k3/retrieve?q=<text>&engine=packed&verify=1",
                    "/k3/retrieve?q=<text>&engine=tercode&verify=1",
                    "/k3/tercode/build",
                ],
            }),
            404,
        );
    }

    let q = query_param(url, "q").unwrap_or_default().trim().to_string();
    if q.is_empty() {
        return (
            json!({
                "status": "error",
                "route": "/k3/retrieve",
                "error": "missing_query",
                "detail": "pass ?q=<text>; the route scores the query's own ternary quantization",
            }),
            400,
        );
    }
    let k = query_param(url, "k")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(5)
        .clamp(1, 20);
    // Optional candidate-generator shortlist size (0 = the exact path over the whole corpus).
    let gen = query_param(url, "gen")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(0)
        .clamp(0, 512);
    // The packed TurboVec store is the PRIMARY path (the platform's own 2-bit representation, an
    // integer scan over the codes with cosine expanded from each row's moments — no float vector
    // is ever materialized, and it scales to the full corpus where the f32 JSON would not load).
    // `engine=tercode` opts into the exact {-1,0,+1} ternary path (load_engine over the JSON index).
    // `verify=1` re-derives the top rows the slow way (f32 dequant) and reports the largest
    // disagreement, because a fast path that cannot be checked is a claim.
    let use_packed = query_param(url, "engine").as_deref() != Some("tercode");
    let verify_packed = query_param(url, "verify").as_deref() == Some("1");

    let qv = match embed(&q) {
        Ok(v) => v,
        Err(e) => {
            return (
                json!({
                    "status": "error",
                    "route": "/k3/retrieve",
                    "error": "embedder_unavailable",
                    "detail": e,
                    "encoder": ENCODER_NAME,
                }),
                503,
            )
        }
    };

    // `store=<name>` selects a distilled model store: the query is encoded by the local embedder
    // that built the store, then scored on the same on-silicon packed engine. The default is the
    // zero-LLM HDC fabric (native, in-process, no HTTP): the answer path uses the encoder that
    // built the index. The Ollama/llama distilled stores remain opt-in via `store=<name>`.
    let requested = query_param(url, "store").unwrap_or_default();
    let defaulted = requested.is_empty();
    let store_name = if defaulted { "fabric".to_string() } else { requested };

    if store_name != "fabric" {
        if let Some((rel, proto, addr, model, enc)) = model_store_spec(&store_name) {
            let embedded = if proto == "ollama" {
                crate::intelligence_router::embed_ollama(model, &q)
            } else {
                crate::intelligence_router::embed_via(addr, model, &q)
            };
            match embedded {
                Some(v) => {
                    return packed_report_store(&q, &v, rel, &store_name, enc, k, verify_packed)
                }
                None if defaulted => {
                    // the default store's embedder is down: fall back to the HDC fabric below.
                }
                None => {
                    return (
                        json!({
                            "status": "error", "route": "/k3/retrieve", "store": store_name,
                            "error": "embedder_unavailable",
                            "detail": format!("local embedder {addr} did not answer"),
                        }),
                        503,
                    )
                }
            }
        } else if !defaulted {
            return (
                json!({
                    "status": "error", "route": "/k3/retrieve",
                    "error": "unknown_store", "store": store_name,
                    "stores": ["fabric", "nomic", "allminilm", "nomicv2", "mxbai", "embgemma", "qwen3emb", "realmodel", "bonsai4b", "bonsai17b"],
                }),
                400,
            );
        }
    }

    if use_packed {
        let trit_alphabet = query_param(url, "alphabet").as_deref() == Some("trit");
        return packed_report(&q, &qv, k, verify_packed, trit_alphabet);
    }
    if query_param(url, "engine").as_deref() == Some("tercode") {
        return tercode_report(&q, &qv, k, verify_packed);
    }

    // The engine is resolved HERE, and only where it is actually needed. The packed scan scores
    // the store's own 2-bit codes with the query's own quantizer, so it is answered before this
    // point and never asks for an engine at all -- demanding one up front is what made
    // `?engine=packed` answer 503 `engine_required` on a build that could serve it.
    //
    // The engine itself no longer loads anything platform-specific: the corpus is JSON and the
    // primitives are `native_ternary` (the kernel module ported in-tree, diffed against the DLL
    // export by its own parity test), so this resolves on every platform -- including the one the
    // platform boots on, where it used to return "wired for windows only".
    let eng = match engine() {
        Ok(e) => e,
        Err(e) => {
            return (
                json!({"status":"error","route":"/k3/retrieve","error":"engine_required","detail":e}),
                503,
            )
        }
    };

    let n = eng.keys.len();
    let kk = k.min(n);
    let started = std::time::Instant::now();

    let outcome: (Value, u16) = unsafe {
        let mut qtrits = vec![0i8; DIM];
        let mut qscale = 0f32;
        let written = (eng.quantize)(
            qv.as_ptr(),
            DIM,
            qtrits.as_mut_ptr(),
            QUANT_MULTIPLIER,
            &mut qscale,
        );
        if written != DIM {
            return (
                json!({"status":"error","route":"/k3/retrieve","error":"quantizer_mismatch",
                       "detail": format!("engine wrote {written} of {DIM} trits")}),
                500,
            );
        }

        // Candidate generation: shortlist with the packed store, then let the exact ternary search
        // confirm. Candidates come back by KEY, so an index-order difference cannot mis-map a hit.
        let mut sub_corpus: Vec<i8> = Vec::new();
        let mut remap: Vec<usize> = Vec::new();
        if gen > 0 {
            if let Some(cand) = packed_candidates(&qv, gen) {
                let row_of: std::collections::HashMap<&str, usize> =
                    eng.keys.iter().enumerate().map(|(i, k)| (k.as_str(), i)).collect();
                for key in &cand {
                    if let Some(&row) = row_of.get(key.as_str()) {
                        sub_corpus.extend_from_slice(&eng.corpus[row * DIM..(row + 1) * DIM]);
                        remap.push(row);
                    }
                }
            }
        }
        let generated = remap.len();
        let (corpus_ptr, search_n) = if generated > 0 {
            (sub_corpus.as_ptr(), generated)
        } else {
            (eng.corpus.as_ptr(), n)
        };
        let kk_search = kk.min(search_n.max(1));

        let mut idx = vec![0usize; kk_search];
        let mut score = vec![0i32; kk_search];
        let mut comparisons = 0usize;
        let hits = (eng.search)(
            qtrits.as_ptr(),
            DIM,
            corpus_ptr,
            search_n,
            kk_search,
            idx.as_mut_ptr(),
            score.as_mut_ptr(),
            &mut comparisons,
        );
        for i in 0..hits.max(0) as usize {
            if let Some(&row) = remap.get(idx[i]) {
                idx[i] = row;
            }
        }
        if hits < 0 {
            return (
                json!({"status":"error","route":"/k3/retrieve","error":"engine_rejected_query"}),
                500,
            );
        }

        let results: Vec<Value> = (0..hits as usize)
            .map(|r| {
                let d = idx[r];
                let row = &eng.corpus[d * DIM..(d + 1) * DIM];
                json!({
                    "rank": r + 1,
                    "doc": eng.keys[d],
                    "score": score[r],
                    "score_crosschecked": (eng.agreement)(qtrits.as_ptr(), row.as_ptr(), DIM),
                    "doc_magnitude": (eng.magnitude)(row.as_ptr(), DIM),
                    "doc_fidelity": eng.fidelity_by_doc.get(d).copied().unwrap_or(0.0),
                })
            })
            .collect();

        let payload = json!({
            "status": "ok",
            "route": "/k3/retrieve",
            "doctrine": "the platform retrieves from its own quantized memory; no token is sampled to answer",
            "query": q,
            "encoder": ENCODER_NAME,
            "corpus": {"source": eng.index_rel, "documents": n, "dim": DIM},
            "engine": {
                // The algorithm is in-tree and its parity against the reference artifact is a
                // TEST, not a runtime dependency. What is reported here is whether that artifact
                // is present on this machine -- a measurement, so a reader can tell which build
                // answered, rather than a path named on a machine that does not have it.
                "algorithm": "native-ternary (in-tree port of ternary_knowledge_index.rs)",
                "parity_oracle_present": eng.oracle.is_some(),
                "parity_oracle": match &eng.oracle {
                    Some((p, n)) => json!({"path": p, "bytes": n}),
                    None => Value::Null,
                },
                "quantizer_multiplier": QUANT_MULTIPLIER,
            },
            "query_magnitude": (eng.magnitude)(qtrits.as_ptr(), DIM),
            "query_fidelity": (eng.fidelity)(qv.as_ptr(), qtrits.as_ptr(), qscale, DIM),
            "k": kk,
            "comparisons": comparisons,
            // Which path produced this ranking, and how much of the corpus it scored -- a shortlist
            // that is not disclosed is a silently different retrieval.
            "candidates_from": if generated > 0 { "packed-generator-16x" } else { "exact-full-corpus" },
            "candidates_scored": if generated > 0 { generated } else { n },
            "elapsed_us": started.elapsed().as_micros(),
            "scoring": "exact integer ternary agreement over non-zero trits; score is not a distance",
            "results": results,
        });
        (payload, 200)
    };

    outcome
}

/// `/security/threats/query?q=<text>&k=<n>` — the threat-index counterpart of /k3/retrieve.
/// Same ternary engine, same encoder, over the voltage-stripped MITRE+CWE+CVE corpus stored in the
/// platform's native TRITCODE packed format (2 bits/trit, FNV-1a-64 verified). The JSON index form
/// does not scale to ~400k CVE vectors; the packed store does (16x vs f32). No model server, no
/// fabricated score.
struct ThreatStore {
    keys: Vec<String>,
    corpus: Vec<i8>,
    meta: std::collections::HashMap<String, Value>,
    bytes: u64,
    payload_fnv1a64: u64,
}

fn fnv1a64(b: &[u8]) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for &x in b {
        h ^= u64::from(x);
        h = h.wrapping_mul(0x100_0000_01b3);
    }
    h
}

fn load_threat_store(path: &std::path::Path) -> Result<ThreatStore, String> {
    const TER_MAGIC: &[u8; 8] = b"TRITCODE";
    const STRIDE: usize = (DIM * 2 + 7) / 8;
    let md = std::fs::metadata(path).map_err(|e| format!("metadata: {e}"))?;
    let b = std::fs::read(path).map_err(|e| format!("read: {e}"))?;
    if b.len() < 128 || &b[0..8] != TER_MAGIC {
        return Err("not a TRITCODE store".to_string());
    }
    let dim = u32::from_le_bytes([b[12], b[13], b[14], b[15]]) as usize;
    let vectors = u32::from_le_bytes([b[16], b[17], b[18], b[19]]) as usize;
    let bits = u32::from_le_bytes([b[20], b[21], b[22], b[23]]) as usize;
    let stride = u32::from_le_bytes([b[24], b[25], b[26], b[27]]) as usize;
    let keys_off =
        u64::from_le_bytes([b[32], b[33], b[34], b[35], b[36], b[37], b[38], b[39]]) as usize;
    let keys_len = u32::from_le_bytes([b[40], b[41], b[42], b[43]]) as usize;
    let fnv = u64::from_le_bytes([b[48], b[49], b[50], b[51], b[52], b[53], b[54], b[55]]);
    if dim != DIM || bits != 2 || stride != STRIDE || keys_off + keys_len > b.len() {
        return Err("TRITCODE header mismatch".to_string());
    }
    let end = 128 + vectors * stride;
    if end > b.len() {
        return Err("truncated codes".to_string());
    }
    let codes = &b[128..end];
    // An unverifiable store is not a store: check the payload checksum before scoring anything.
    if fnv1a64(codes) != fnv {
        return Err("payload FNV-1a-64 mismatch".to_string());
    }
    let keys: Vec<String> = serde_json::from_slice::<Value>(&b[keys_off..keys_off + keys_len])
        .map_err(|e| format!("keys JSON: {e}"))?
        .get("keys")
        .and_then(|k| k.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
        .ok_or("keys table missing".to_string())?;
    if keys.len() != vectors {
        return Err("keys count != vectors".to_string());
    }
    let mut corpus = vec![0i8; vectors * DIM];
    for i in 0..vectors {
        for j in 0..DIM {
            let c = (codes[i * stride + (j * 2) / 8] >> ((j * 2) % 8)) & 3;
            corpus[i * DIM + j] = if c == 2 { 1 } else if c == 0 { -1 } else { 0 };
        }
    }
    let meta = match repo_relative("swarm_v2_artifacts/security_threat_index.meta.json") {
        Some(mp) => serde_json::from_slice::<Value>(&std::fs::read(&mp).map_err(|e| format!("meta read: {e}"))?)
            .map_err(|e| format!("meta JSON: {e}"))?
            .get("meta")
            .and_then(|m| m.as_object())
            .map(|o| o.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default(),
        None => std::collections::HashMap::new(),
    };
    Ok(ThreatStore { keys, corpus, meta, bytes: md.len(), payload_fnv1a64: fnv })
}

fn threat_store() -> Option<std::sync::Arc<ThreatStore>> {
    static S: crate::file_cache::Slot<ThreatStore> = std::sync::Mutex::new(None);
    let p = repo_relative("swarm_v2_artifacts/security_threat_index.tercode")?;
    crate::file_cache::get(&S, &p, |p| {
        load_threat_store(p).map_err(|e| format!("threat store rejected: {e}"))
    })
    .ok()
}

pub fn threat_query_handle(url: &str) -> (Value, u16) {
    let q = query_param(url, "q").unwrap_or_default().trim().to_string();
    if q.is_empty() {
        return (
            json!({"status":"error","route":"/security/threats/query","error":"missing_query",
                   "detail":"pass ?q=<text>"}),
            400,
        );
    }
    let k = query_param(url, "k")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(5)
        .clamp(1, 20);

    let store = match threat_store() {
        Some(s) => s,
        None => {
            return (
                json!({"status":"error","route":"/security/threats/query","error":"threat_index_missing",
                       "detail":"security_threat_index.tercode absent or failed verification; voltage-strip the threat DB first"}),
                503,
            )
        }
    };
    let keys = &store.keys;
    let corpus = &store.corpus;

    let qv = match embed(&q) {
        Ok(v) => v,
        Err(e) => {
            return (
                json!({"status":"error","route":"/security/threats/query","error":"embedder_unavailable","detail":e}),
                503,
            )
        }
    };
    let n = keys.len();
    let kk = k.min(n);
    let mut qtrits = vec![0i8; DIM];
    let mut qscale = 0f32;
    let written = unsafe {
        crate::native_ternary::nt_quantize(qv.as_ptr(), DIM, qtrits.as_mut_ptr(), QUANT_MULTIPLIER, &mut qscale)
    };
    if written != DIM {
        return (
            json!({"status":"error","route":"/security/threats/query","error":"quantizer_mismatch"}),
            500,
        );
    }

    let mut idx = vec![0usize; kk];
    let mut score = vec![0i32; kk];
    let mut comparisons = 0usize;
    let started = std::time::Instant::now();
    let hits = unsafe {
        crate::native_ternary::nt_search(
            qtrits.as_ptr(), DIM, corpus.as_ptr(), n, kk,
            idx.as_mut_ptr(), score.as_mut_ptr(), &mut comparisons,
        )
    };
    if hits < 0 {
        return (
            json!({"status":"error","route":"/security/threats/query","error":"engine_rejected_query"}),
            500,
        );
    }
    let results: Vec<Value> = (0..hits as usize)
        .map(|r| {
            let d = idx[r];
            let row = &corpus[d * DIM..(d + 1) * DIM];
            let doc = &keys[d];
            let m = store.meta.get(doc.as_str());
            json!({
                "rank": r + 1,
                "doc": doc,
                "id": m.and_then(|v| v.get("id")).and_then(|x| x.as_str()),
                "name": m.and_then(|v| v.get("name")).and_then(|x| x.as_str()),
                "type": m.and_then(|v| v.get("type")).and_then(|x| x.as_str()),
                "severity": m.and_then(|v| v.get("severity")).and_then(|x| x.as_str()),
                "score": score[r],
                "score_crosschecked": unsafe {
                    crate::native_ternary::nt_agreement(qtrits.as_ptr(), row.as_ptr(), DIM)
                },
            })
        })
        .collect();

    (
        json!({
            "status":"ok","route":"/security/threats/query","query":q,"k":kk,
            "encoder": ENCODER_NAME,
            "corpus": {"source": "swarm_v2_artifacts/security_threat_index.tercode", "documents": n, "dim": DIM,
                       "bytes": store.bytes, "payload_fnv1a64": format!("{:016x}", store.payload_fnv1a64)},
            "comparisons": comparisons,
            "elapsed_us": started.elapsed().as_micros(),
            "results": results,
        }),
        200,
    )
}

// ── Fabric tier: answer the chat from sealed knowledge with no model call ───────────────
//
// The ternary score alone does NOT separate known from unknown: measured on this corpus,
// known queries score 137..155 and nonsense 134..144, overlapping. So the score only
// PROPOSES candidates; a candidate is confirmed only when the sealed document itself shares
// at least two of the message's content words. Nothing confirmed -> None, and the caller
// falls through to its model tiers honestly.

/// A fabric-confirmed answer source.
pub struct FabricHit {
    pub doc: String,
    pub score: i32,
    pub matched_terms: Vec<String>,
    /// Query terms that appear in the document's own path/name.
    pub path_hits: usize,
    pub snippet: String,
    pub candidates: usize,
    pub documents: usize,
    /// Which ranker produced this candidate list: the kernel's tercode store or the engine's own
    /// search. A label that does not say which path ran is a label that cannot be checked.
    pub ranker: &'static str,
    /// The kernel's packed-domain ternary verdict for this query: trit, the band it cleared, and the
    /// document that engine ranked first. Derived per query, so a fabric answer states its basis.
    pub packed_trit: i8,
    pub packed_basis: String,
    pub packed_top1: String,
    pub packed_us: f64,
}

/// Content words of a message: lowercase alphanumeric tokens of 4+ chars, stopwords dropped.
fn content_terms(text: &str) -> Vec<String> {
    const STOP: &[&str] = &[
        "what", "which", "when", "where", "with", "that", "this", "from", "have", "does",
        "your", "about", "into", "they", "them", "than", "then", "there", "these", "those",
        "will", "would", "should", "could", "explain", "please", "tell", "give", "make",
        "made", "using", "used", "many", "much", "does",
    ];
    let mut out: Vec<String> = Vec::new();
    for tok in text.split(|c: char| !c.is_alphanumeric()) {
        let t = tok.to_lowercase();
        if t.len() >= 4 && !STOP.contains(&t.as_str()) && !out.contains(&t) {
            out.push(t);
        }
    }
    out
}

/// Resolve a corpus key to the sealed document's text, walking the same repo root the
/// engine resolves against.
fn doc_text(key: &str) -> Option<String> {
    for prefix in ["", "swarm_v2_artifacts/", "docs/", "swarm_v2/papers/"] {
        if let Some(p) = repo_relative(&format!("{prefix}{key}")) {
            if let Ok(bytes) = std::fs::read(&p) {
                let cut = bytes.len().min(200_000);
                return Some(String::from_utf8_lossy(&bytes[..cut]).to_string());
            }
        }
    }
    None
}

/// The largest char boundary <= i (string slicing must land on a boundary).
fn floor_boundary(s: &str, i: usize) -> usize {
    let mut j = i.min(s.len());
    while j > 0 && !s.is_char_boundary(j) {
        j -= 1;
    }
    j
}

/// Case-insensitive ASCII find returning a byte offset. Byte scanning keeps every offset on a
/// char boundary: an ASCII needle only matches where the haystack bytes are ASCII, and no
/// UTF-8 continuation byte can equal an ASCII byte.
fn find_ci(hay: &str, needle: &str) -> Option<usize> {
    let h = hay.as_bytes();
    let n = needle.as_bytes();
    if n.is_empty() || h.len() < n.len() {
        return None;
    }
    (0..=h.len() - n.len()).find(|&i| h[i..i + n.len()].eq_ignore_ascii_case(n))
}

/// A ~320-character window around the matched terms, snapped to word boundaries and chosen to
/// contain as many DISTINCT terms as possible. Anchoring at every occurrence, not just the
/// earliest, avoids the earlier failure where the quote collapsed to a single stray token.
fn snippet_of(text: &str, terms: &[String]) -> String {
    let flat = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut anchors: Vec<usize> = Vec::new();
    for t in terms {
        let mut from = 0usize;
        while anchors.len() < 16 {
            match find_ci(&flat[from..], t) {
                Some(rel) => {
                    let at = from + rel;
                    anchors.push(at);
                    from = at + t.len();
                }
                None => break,
            }
        }
    }
    if anchors.is_empty() {
        let cut = floor_boundary(&flat, 320);
        return format!("{}...", flat[..cut].trim());
    }
    let mut best: Option<(usize, usize)> = None;
    for &a in &anchors {
        let mut start = flat[..a.min(flat.len())]
            .rfind(' ')
            .map(|i| i + 1)
            .unwrap_or(0);
        start = floor_boundary(&flat, start.saturating_sub(160));
        start = flat[..start].rfind(' ').map(|i| i + 1).unwrap_or(0);
        let end = floor_boundary(&flat, (start + 320).min(flat.len()));
        let end = flat[..end].rfind(' ').map(|i| i).unwrap_or(end);
        let distinct = terms
            .iter()
            .filter(|t| find_ci(&flat[start..end], t).is_some())
            .count();
        if best.map(|(d, _)| distinct > d).unwrap_or(true) {
            best = Some((distinct, start));
        }
    }
    let start = best.map(|(_, s)| s).unwrap_or(0);
    let end = floor_boundary(&flat, (start + 320).min(flat.len()));
    let end = flat[..end].rfind(' ').map(|i| i).unwrap_or(end);
    format!("{}...", flat[start..end].trim())
}

/// Candidate generation now runs on the kernel's own packed-domain engine (`packed_ternary`): it
/// scores the 2-bit codes directly and expands each row's cosine from that row's moments, so nothing
/// here dequantizes the store to f32 at load (which is what made the packed path the float path with
/// a decode tax). Measured agreement with the exact ternary path over the platform's own questions:
/// the exact top-1 lands inside the packed top-5 for 7/10 and inside the packed top-16 for 9/10
/// (swarm_v2_artifacts/nand_fabric_index_benchmark.json holds the earlier shortlist run).
fn packed_candidates(qv: &[f32], gen: usize) -> Option<Vec<String>> {
    let out = packed_ternary::search(qv, gen)?;
    Some(out.hits.into_iter().map(|h| h.key).collect())
}

/// The kernel's ternary verdict for a query, from the packed engine: (trit, basis, top document, us).
/// Attached to fabric answers so a retrieval states the band it cleared rather than only a score.
pub fn packed_verdict(qv: &[f32], k: usize) -> Option<(i8, String, String, f64)> {
    let out = packed_ternary::search(qv, k)?;
    let top = out.hits.first().map(|h| h.key.clone()).unwrap_or_default();
    Some((out.verdict, out.basis, top, out.us))
}

/// /k3/stores — the honest report of every distilled store the on-silicon engine can load now that
/// the reader is dimension-agnostic. Each header is read from the file itself, so a dim here is a
/// measured fact, not an asserted design figure.
fn stores_handle() -> (Value, u16) {
    let known: [(&str, &str, &str); 10] = [
        ("fabric", "swarm_v2_nand_fabric.turbovec", "zero-LLM HDC encoder (native-turbovec-hdc)"),
        ("nomic", "swarm_v2_nand_fabric.nomic.turbovec", "nomic-embed-text 768-dim (Ollama)"),
        ("allminilm", "swarm_v2_nand_fabric.allminilm.turbovec", "all-MiniLM-L6-v2 384-dim (Ollama)"),
        ("nomicv2", "swarm_v2_nand_fabric.nomicv2.turbovec", "nomic-embed-text-v2-moe 768-dim (Ollama)"),
        ("mxbai", "swarm_v2_nand_fabric.mxbai.turbovec", "mxbai-embed-large 1024-dim (Ollama)"),
        ("embgemma", "swarm_v2_nand_fabric.embgemma.turbovec", "embeddinggemma 768-dim (Ollama)"),
        ("qwen3emb", "swarm_v2_nand_fabric.qwen3emb.turbovec", "qwen3-embedding:0.6b 1024-dim (Ollama)"),
        ("realmodel", "swarm_v2_nand_fabric.realmodel.turbovec", "Phi-4 model embeddings (vanilj/Phi-4)"),
        ("bonsai4b", "swarm_v2_nand_fabric.bonsai4b.turbovec", "Ternary-Bonsai-4B embeddings"),
        ("bonsai17b", "swarm_v2_nand_fabric.bonsai17b.turbovec", "Ternary-Bonsai-1.7B embeddings"),
    ];
    let mut stores = Vec::new();
    for (name, rel, encoder) in known {
        match packed_ternary::store_at(rel) {
            Some(s) => {
                let mut h = s.header();
                if let Value::Object(o) = &mut h {
                    o.insert("name".to_string(), json!(name));
                    o.insert("encoder".to_string(), json!(encoder));
                    o.insert("loaded".to_string(), json!(true));
                }
                stores.push(h);
            }
            None => stores.push(json!({
                "name": name, "encoder": encoder, "rel": rel,
                "loaded": false, "note": "store missing or malformed",
            })),
        }
    }
    (
        json!({
            "status": "ok",
            "route": "/k3/stores",
            "on_silicon_engine": "packed-domain integer scan; cosine expanded from per-row moments, no f32 row materialized",
            "stores": stores,
        }),
        200,
    )
}

/// The kernel-owned K3 fabric store: (path, bytes). Derived from the file itself so callers report
/// what is mapped rather than an asserted design figure.
pub fn fabric_store() -> Option<(String, u64)> {
    for rel in ["swarm_v2_nand_fabric.k3", "swarm_v2_rust_kernel/swarm_v2_nand_fabric.k3"] {
        if let Some(p) = repo_relative(rel) {
            if let Ok(md) = std::fs::metadata(&p) {
                return Some((p.display().to_string(), md.len()));
            }
        }
    }
    None
}

/// Ask the kernel's own index whether the message is known knowledge.
/// Returns None unless a retrieved, score-cross-checked document is LEXICALLY confirmed.
pub fn fabric_lookup(message: &str, k: usize) -> Option<FabricHit> {
    let terms = content_terms(message);
    if terms.len() < 2 {
        return None;
    }
    let eng = engine().ok()?;
    let qv = embed(message).ok()?;
    // The kernel's own ternary extraction over the packed store, alongside the engine's path.
    let (packed_trit, packed_basis, packed_top1, packed_us) =
        packed_verdict(&qv, 3).unwrap_or((0, "packed store unavailable: no verdict derived".to_string(), String::new(), 0.0));
    let n = eng.keys.len();
    let kk = k.min(n);
    let mut best: Option<FabricHit> = None;

    unsafe {
        let mut qtrits = vec![0i8; DIM];
        let mut qscale = 0f32;
        let written = (eng.quantize)(
            qv.as_ptr(),
            DIM,
            qtrits.as_mut_ptr(),
            QUANT_MULTIPLIER,
            &mut qscale,
        );
        if written != DIM {
            return None;
        }
        // The ranking comes from the kernel's own tercode store when it exists: that store holds the
        // engine's trits, so its order is identical to `rust_ternary_index_search` -- measured 10/10
        // queries and 100/100 ranks, document AND score -- while being 3.1x smaller. The engine's own
        // search stays as the fallback, and its agreement export must still reproduce each score
        // before a candidate is acted on, whichever path proposed it.
        let mut candidates: Vec<(String, i32)> = Vec::new();
        let mut ranker = "engine:rust_ternary_index_search";
        if let Some(t) = packed_ternary::search_tercode(&qtrits, kk) {
            candidates = t.hits.into_iter().map(|h| (h.key, h.score as i32)).collect();
            ranker = "kernel:tercode-store";
        }
        if candidates.is_empty() {
            let mut idx = vec![0usize; kk];
            let mut score = vec![0i32; kk];
            let mut comparisons = 0usize;
            let hits = (eng.search)(
                qtrits.as_ptr(),
                DIM,
                eng.corpus.as_ptr(),
                n,
                kk,
                idx.as_mut_ptr(),
                score.as_mut_ptr(),
                &mut comparisons,
            );
            if hits <= 0 {
                return None;
            }
            candidates = (0..hits as usize)
                .map(|r| (eng.keys[idx[r]].clone(), score[r]))
                .collect();
        }
        let proposed = candidates.len();
        for (key, sc) in candidates {
            let d = match eng.keys.iter().position(|k| k == &key) {
                Some(d) => d,
                None => continue,
            };
            let row = &eng.corpus[d * DIM..(d + 1) * DIM];
            // The independent agreement export must reproduce the score, or this is not a hit we are
            // willing to act on.
            if (eng.agreement)(qtrits.as_ptr(), row.as_ptr(), DIM) != sc {
                continue;
            }
            let text = match doc_text(&key) {
                Some(t) => t,
                None => continue,
            };
            let lower = text.to_lowercase();
            let key_lower = key.to_lowercase();
            let matched: Vec<String> = terms
                .iter()
                .filter(|t| lower.contains(t.as_str()))
                .cloned()
                .collect();
            let path_hits = terms.iter().filter(|t| key_lower.contains(t.as_str())).count();
            // MOTION-037 seam 2: accept a match on either >=2 text-term matches OR the document's own
            // path naming a query term (a strong topical signal). A single strong term ('turbovec')
            // must not be rejected just because the doc text lacks a second term.
            if matched.len() < 2 && path_hits < 1 {
                continue;
            }
            // A document whose own path names the query's terms is a stronger topical signal
            // than a few points of ternary score: measured, the native encoder's top five span
            // only 95..89, so ranking alone cannot separate an on-topic source file from a UI
            // component that merely contains the same words.
            let better = best
                .as_ref()
                .map(|b| {
                    (path_hits, matched.len(), sc)
                        > (b.path_hits, b.matched_terms.len(), b.score)
                })
                .unwrap_or(true);
            if better {
                best = Some(FabricHit {
                    doc: key,
                    score: sc,
                    matched_terms: matched.clone(),
                    path_hits,
                    snippet: snippet_of(&text, &matched),
                    candidates: proposed,
                    ranker,
                    documents: n,
                    packed_trit,
                    packed_basis: packed_basis.clone(),
                    packed_top1: packed_top1.clone(),
                    packed_us,
                });
            }
        }
    }
    best
}

/// The packed-domain store (native-turbovec-hdc) — the PRIMARY store on hosts whose exact ternary
/// index exceeds the load limit (so `fabric_lookup` returns None without a full read). Returns the
/// top hit as (doc, snippet, score), so the chat and other callers still answer from the sealed
/// knowledge when the exact store can't load. No model is sampled; the score is the packed cosine.
pub fn packed_retrieval(message: &str) -> Option<(String, String, f64)> {
    // Query the packed store on the CONTENT terms, so "what is the X" queries the meaningful X, not
    // the stopwords. If no content term survives, fall back to the raw message.
    let terms = content_terms(message);
    let query = if terms.is_empty() {
        message.trim().to_string()
    } else {
        terms.join(" ")
    };
    let q = query
        .replace('&', "%26")
        .replace('?', "%3F")
        .replace(' ', "%20");
    let (payload, code) = handle(&format!("/k3/retrieve?q={q}"));
    if code != 200 {
        return None;
    }
    let results = payload.get("results").and_then(|r| r.as_array())?;
    if results.is_empty() {
        return None;
    }
    // The packed store's top hits can be corpus fragments (relational/enwikiquote.xml.bz2#N),
    // which are not readable documents. Walk the ranked list and return the first readable
    // sealed document. If NO hit is readable, return None so the caller falls through to the
    // co-processor verdict — surfacing "ranked #1; not directly readable" as the answer was
    // a non-answer (the platform claimed a sealed document it could not show).
    //
    // Lexical-overlap gate (council MOTION-R1, 15/15): the packed cosine is compressed into
    // 0.17-0.40 and cannot separate a real match from noise alone (measured: 'monad'->0.20 is
    // real, 'zxqwpvy'->0.176 is noise). A hit is surfaced only when the query's content terms
    // literally appear in the document key, OR the cosine clears a high-confidence threshold.
    let lex_overlap = |doc: &str| {
        let dl = doc.to_ascii_lowercase();
        terms.iter().any(|t| !t.is_empty() && dl.contains(&t.to_ascii_lowercase()))
    };
    for top in results.iter() {
        let doc = top.get("doc").and_then(|d| d.as_str()).unwrap_or("");
        if doc.contains('#') {
            continue;
        }
        let Some(text) = doc_text(doc) else { continue };
        let score = top.get("score").and_then(|s| s.as_f64()).unwrap_or(0.0);
        if !lex_overlap(doc) && score < 0.35 {
            continue; // no lexical overlap and below the high-confidence cosine -> not a match
        }
        let head: String = text.chars().take(400).collect();
        let snippet = if text.chars().count() > 400 {
            format!("{head}…")
        } else {
            head
        };
        return Some((doc.to_string(), snippet, score));
    }
    None
}

/// The routed agent's best-effort answer from its own knowledge store: the top semantic hit from
/// the nomic store, then the lexical zero-LLM fabric. Returns (doc, snippet, source). A retrieval
/// answer is always attempted before the platform admits it cannot answer — the agent queries its
/// own store instead of failing.
pub fn agent_retrieval(message: &str) -> Option<(String, String, String)> {
    let head = |text: &str| {
        let h: String = text.chars().take(320).collect();
        if text.chars().count() > 320 { format!("{h}…") } else { h }
    };
    // The answer path is the zero-LLM HDC fabric (native, in-process, no HTTP): the query is
    // encoded by the same encoder that built the index, so query and corpus live in one space.
    if let Ok(qv) = embed(message) {
        if let Some(out) = packed_ternary::search(&qv, 1) {
            if let Some(hit) = out.hits.first() {
                if let Some(text) = doc_text(&hit.key) {
                    return Some((
                        hit.key.clone(),
                        head(&text),
                        "zero-LLM HDC fabric (lexical, no model)".to_string(),
                    ));
                }
            }
        }
    }
    None
}

/// The distilled stores that need a local model for query encoding: (repo-relative store path,
/// loopback embedder addr, model name, encoder label). The fabric store is intentionally absent —
/// it uses the in-tree HDC encoder and needs no model at all.
fn model_store_spec(name: &str) -> Option<(&'static str, &'static str, &'static str, &'static str, &'static str)> {
    match name {
        "realmodel" => Some(("swarm_v2_nand_fabric.realmodel.turbovec", "llama", "127.0.0.1:8588", "phi4", "Phi-4 (vanilj/Phi-4) 384-dim")),
        "allminilm" => Some(("swarm_v2_nand_fabric.allminilm.turbovec", "ollama", "127.0.0.1:11434", "all-minilm", "all-MiniLM-L6-v2 384-dim")),
        "nomic" => Some(("swarm_v2_nand_fabric.nomic.turbovec", "ollama", "127.0.0.1:11434", "nomic-embed-text", "nomic-embed-text 768-dim")),
        "nomicv2" => Some(("swarm_v2_nand_fabric.nomicv2.turbovec", "ollama", "127.0.0.1:11434", "nomic-embed-text-v2-moe", "nomic-embed-text-v2-moe 768-dim")),
        "mxbai" => Some(("swarm_v2_nand_fabric.mxbai.turbovec", "ollama", "127.0.0.1:11434", "mxbai-embed-large", "mxbai-embed-large 1024-dim")),
        "embgemma" => Some(("swarm_v2_nand_fabric.embgemma.turbovec", "ollama", "127.0.0.1:11434", "embeddinggemma", "embeddinggemma 768-dim")),
        "qwen3emb" => Some(("swarm_v2_nand_fabric.qwen3emb.turbovec", "ollama", "127.0.0.1:11434", "qwen3-embedding:0.6b", "qwen3-embedding:0.6b 1024-dim")),
        "bonsai4b" => Some(("swarm_v2_nand_fabric.bonsai4b.turbovec", "llama", "127.0.0.1:8589", "bonsai4b", "Ternary-Bonsai-4B 2560-dim")),
        "bonsai17b" => Some(("swarm_v2_nand_fabric.bonsai17b.turbovec", "llama", "127.0.0.1:8591", "bonsai17b", "Ternary-Bonsai-1.7B 2048-dim")),
        _ => None,
    }
}

/// Report for a specific distilled store, searched on the same packed-domain engine. The query is
/// already encoded in the store's space by the caller; this scores and (optionally) re-checks it.
fn packed_report_store(q: &str, qv: &[f32], rel: &str, name: &str, enc: &str, k: usize, verify: bool) -> (Value, u16) {
    let store = match packed_ternary::store_at(rel) {
        Some(s) => s,
        None => {
            return (
                json!({
                    "status": "error", "route": "/k3/retrieve", "engine": "packed-domain-turbovec",
                    "store": name, "error": "packed_store_unavailable",
                    "detail": format!("{rel} is absent, unreadable, or malformed"),
                }),
                503,
            )
        }
    };
    let out = match packed_ternary::search_in(&store, qv, k) {
        Some(o) => o,
        None => {
            return (
                json!({
                    "status": "error", "route": "/k3/retrieve", "engine": "packed-domain-turbovec",
                    "store": name, "error": "query_rejected",
                    "detail": format!("query ({} dims) does not match store dim {}", qv.len(), store.dim()),
                }),
                400,
            )
        }
    };

    let agreement = if verify {
        match packed_ternary::quantize(qv, store.dim()) {
            Some(qq) => {
                let qd = packed_ternary::dequant_query(&qq);
                let (mut worst, mut checked) = (0f64, 0usize);
                for h in &out.hits {
                    if let Some(i) = store.index_of(&h.key) {
                        if let Some(slow) = store.f32_cosine(&qd, i) {
                            worst = worst.max((slow - h.score).abs());
                            checked += 1;
                        }
                    }
                }
                json!({
                    "rows_checked": checked, "max_abs_diff": worst, "tolerance": 1e-5,
                    "verdict": if worst <= 1e-5 { "reproduced" } else { "DIVERGED" },
                    "how": "the quantized query was put back to f32 and each reported row re-scored by f32 dequantization of the same 2-bit codes",
                })
            }
            None => json!({"rows_checked": 0, "verdict": "unavailable"}),
        }
    } else {
        Value::Null
    };

    let results: Vec<Value> = out
        .hits
        .iter()
        .map(|h| json!({"rank": h.rank, "doc": h.key, "score": h.score}))
        .collect();
    let mut store_header = store.header();
    if let Value::Object(o) = &mut store_header {
        o.insert("name".to_string(), json!(name));
        o.insert("encoder".to_string(), json!(enc));
    }

    (
        json!({
            "status": "ok", "route": "/k3/retrieve", "engine": "packed-domain-turbovec",
            "path": "integer scan over the 2-bit codes; cosine expanded from per-row moments, no f32 row materialized",
            "query": q, "k": k, "encoder": enc,
            "scored_documents": out.scanned, "packed_scan_us": out.us,
            "store": store_header,
            "k3": {
                "verdict": packed_ternary::trit_name(out.verdict),
                "trit": out.verdict,
                "basis": out.basis,
                "score_spread_sd": out.spread,
            },
            "verify": agreement,
            "results": results,
        }),
        200,
    )
}

/// Report for the kernel's packed-domain ternary engine, with an optional independent check: the
/// same rows re-derived from the same bytes the slow way (f32 dequantization), so "the fast path
/// agrees" is a measurement rather than a claim.
fn packed_report(q: &str, qv: &[f32], k: usize, verify: bool, trit_alphabet: bool) -> (Value, u16) {
    let unavailable = || {
        (
            json!({
                "status": "error", "route": "/k3/retrieve", "engine": "packed-domain-turbovec",
                "error": "packed_store_unavailable",
                "detail": "swarm_v2_nand_fabric.turbovec is absent, unreadable, or not 768-dim 2-bit",
            }),
            503,
        )
    };

    // With `alphabet=trit` the query is reduced by the engine's own quantizer, so the packed store is
    // scored in the same ternary alphabet the engine's corpus uses -- the two paths then rank the same
    // documents, which is what "16x smaller" has to mean if it is to mean anything.
    let mut engine_trits: Vec<i8> = Vec::new();
    let out = if trit_alphabet {
        let eng = match engine() {
            Ok(e) => e,
            Err(e) => {
                return (
                    json!({"status":"error","route":"/k3/retrieve","error":"engine_required","detail":e}),
                    503,
                )
            }
        };
        let mut tr: Vec<i8> = vec![0i8; DIM];
        let mut qscale = 0f32;
        let written = unsafe {
            (eng.quantize)(qv.as_ptr(), DIM, tr.as_mut_ptr(), QUANT_MULTIPLIER, &mut qscale)
        };
        if written != DIM {
            return (
                json!({"status":"error","route":"/k3/retrieve","error":"quantizer_mismatch",
                       "detail": format!("engine wrote {written} of {DIM} trits")}),
                500,
            );
        }
        engine_trits = tr.clone();
        match packed_ternary::search_trits(&tr, k) {
            Some(o) => o,
            None => return unavailable(),
        }
    } else {
        match packed_ternary::search(qv, k) {
            Some(o) => o,
            None => return unavailable(),
        }
    };

    let agreement = if verify && trit_alphabet {
        match engine() {
            Ok(eng) => {
                let row_of: std::collections::HashMap<&str, usize> =
                    eng.keys.iter().enumerate().map(|(i, k)| (k.as_str(), i)).collect();
                let (mut worst, mut checked) = (0f64, 0usize);
                for h in &out.hits {
                    if let Some(&row) = row_of.get(h.key.as_str()) {
                        let base = row * DIM;
                        let mut slow = 0i64;
                        for (j, &qt) in engine_trits.iter().enumerate() {
                            slow += i64::from(qt) * i64::from(eng.corpus[base + j]);
                        }
                        worst = worst.max((slow as f64 - h.score).abs());
                        checked += 1;
                    }
                }
                json!({
                    "rows_checked": checked,
                    "max_abs_diff": worst,
                    "tolerance": 0.0,
                    "verdict": if worst == 0.0 { "reproduced" } else { "DIVERGED" },
                    "how": "the engine's own integer sum rebuilt from the engine's trit corpus, compared per rank against this path's score",
                    "basis": "structural and expected, not a bug: the packed store is min..max quantized, so its 4-level codes cannot carry the engine's zero trit and the two sums are not the same quantity. The store written from those trits -- /k3/retrieve?engine=tercode&verify=1 -- is the one that reproduces the engine's sums exactly.",
                })
            }
            Err(_) => json!({"rows_checked": 0, "verdict": "unavailable"}),
        }
    } else if verify {
        let store = packed_ternary::store();
        let qq = store.as_ref().and_then(|s| packed_ternary::quantize(qv, s.dim()));
        match (store, qq) {
            (Some(s), Some(qq)) => {
                let qd = packed_ternary::dequant_query(&qq);
                let (mut worst, mut checked) = (0f64, 0usize);
                for h in &out.hits {
                    if let Some(i) = s.index_of(&h.key) {
                        if let Some(slow) = s.f32_cosine(&qd, i) {
                            worst = worst.max((slow - h.score).abs());
                            checked += 1;
                        }
                    }
                }
                json!({
                    "rows_checked": checked,
                    "max_abs_diff": worst,
                    "tolerance": 1e-5,
                    "verdict": if worst <= 1e-5 { "reproduced" } else { "DIVERGED" },
                    "how": "the quantized query was put back to f32 and each reported row re-scored by f32 dequantization of the same 2-bit codes",
                })
            }
            _ => json!({"rows_checked": 0, "verdict": "unavailable"}),
        }
    } else {
        Value::Null
    };

    let results: Vec<Value> = out
        .hits
        .iter()
        .map(|h| json!({"rank": h.rank, "doc": h.key, "score": h.score}))
        .collect();

    (
        json!({
            "status": "ok",
            "route": "/k3/retrieve",
            "engine": "packed-domain-turbovec",
            "path": "integer scan over the 2-bit codes; cosine expanded from per-row moments, no f32 row materialized",
            "query": q,
            "k": k,
            "encoder": ENCODER_NAME,
            "scored_documents": out.scanned,
            "packed_scan_us": out.us,
            "store": out.store,
            "k3": {
                "verdict": packed_ternary::trit_name(out.verdict),
                "trit": out.verdict,
                "basis": out.basis,
                "score_spread_sd": out.spread,
            },
            "agreement": agreement,
            "results": results,
        }),
        200,
    )
}

/// `/k3/tercode/build` — write the engine's own trit corpus as a 2-bit store. The alphabet is the
/// engine's by construction, which is the only way the packed scan can rank exactly as it does.
fn tercode_handle(url: &str) -> (Value, u16) {
    if !url.contains("build") {
        return (
            json!({
                "status": "ok", "route": "/k3/tercode",
                "routes": ["/k3/tercode/build"],
                "note": "build writes swarm_v2_nand_fabric.tercode from the engine's corpus; retrieve it with /k3/retrieve?engine=tercode",
            }),
            200,
        );
    }
    let eng = match engine() {
        Ok(e) => e,
        Err(e) => {
            return (
                json!({"status":"error","route":"/k3/tercode/build","error":"engine_required","detail":e}),
                503,
            )
        }
    };
    let root = match repo_root() {
        Some(r) => r,
        None => {
            return (
                json!({"status":"error","route":"/k3/tercode/build","error":"repo_root_unresolved"}),
                500,
            )
        }
    };
    let path = root.join("swarm_v2_nand_fabric.tercode");
    match packed_ternary::write_tercode(&path, &eng.keys, &eng.corpus, DIM) {
        Ok((bytes, rows, fnv)) => (
            json!({
                "status": "ok", "route": "/k3/tercode/build",
                "path": path.display().to_string(),
                "bytes": bytes, "rows": rows, "dim": DIM, "bits_per_trit": 2,
                "payload_fnv1a64": format!("{fnv:016x}"),
                "source": "the engine's own trit corpus (the bytes rust_ternary_index_search reads), packed 2 bits per trit",
                "vs_engine_i8_corpus": format!("{rows} rows x {DIM} dims as i8 = {} bytes", rows * DIM),
            }),
            200,
        ),
        Err(e) => (
            json!({"status":"error","route":"/k3/tercode/build","error":"write_failed","detail":e}),
            500,
        ),
    }
}

/// `/k3/retrieve?engine=tercode` — score the tercode store with the engine's own agreement basis.
/// `verify=1` adds the decisive check: the ENGINE's own top-k for the same query, compared hit by
/// hit and score by score. Identical ranking is then a measurement, not a design claim.
fn tercode_report(q: &str, qv: &[f32], k: usize, verify: bool) -> (Value, u16) {
    let eng = match engine() {
        Ok(e) => e,
        Err(e) => {
            return (
                json!({"status":"error","route":"/k3/retrieve","engine":"tercode","error":"engine_required","detail":e}),
                503,
            )
        }
    };
    let mut tr: Vec<i8> = vec![0i8; DIM];
    let mut qscale = 0f32;
    let written =
        unsafe { (eng.quantize)(qv.as_ptr(), DIM, tr.as_mut_ptr(), QUANT_MULTIPLIER, &mut qscale) };
    if written != DIM {
        return (
            json!({"status":"error","route":"/k3/retrieve","engine":"tercode","error":"quantizer_mismatch",
                   "detail": format!("engine wrote {written} of {DIM} trits")}),
            500,
        );
    }
    let out = match packed_ternary::search_tercode(&tr, k) {
        Some(o) => o,
        None => {
            return (
                json!({"status":"error","route":"/k3/retrieve","engine":"tercode",
                       "error":"tercode_store_unavailable",
                       "detail":"no readable swarm_v2_nand_fabric.tercode (build it with /k3/tercode/build)"}),
                503,
            )
        }
    };

    let agreement = if verify {
        let n = eng.keys.len();
        let kk = k.min(n);
        let mut idx = vec![0usize; kk];
        let mut score = vec![0i32; kk];
        let mut comparisons = 0usize;
        let hits = unsafe {
            (eng.search)(
                tr.as_ptr(),
                DIM,
                eng.corpus.as_ptr(),
                n,
                kk,
                idx.as_mut_ptr(),
                score.as_mut_ptr(),
                &mut comparisons,
            )
        };
        let mut same = 0usize;
        let mut diffs: Vec<Value> = Vec::new();
        for (r, h) in out.hits.iter().enumerate() {
            let theirs = if r < hits as usize {
                Some((idx[r], score[r]))
            } else {
                None
            };
            match theirs {
                Some((i, s)) if eng.keys[i] == h.key && f64::from(s) == h.score => same += 1,
                other => diffs.push(json!({
                    "rank": r + 1,
                    "tercode": {"doc": h.key, "score": h.score},
                    "engine": other.map(|(i, s)| json!({"doc": eng.keys.get(i), "score": s})).unwrap_or(Value::Null),
                })),
            }
        }
        json!({
            "engine_hits": hits,
            "engine_comparisons": comparisons,
            "identical": diffs.is_empty() && same == out.hits.len(),
            "matched": same,
            "of": out.hits.len(),
            "differences": diffs,
            "how": "the engine's own rust_ternary_index_search (the in-tree port) over its i8 corpus, compared per rank and per score",
        })
    } else {
        Value::Null
    };

    let results: Vec<Value> = out
        .hits
        .iter()
        .map(|h| json!({"rank": h.rank, "doc": h.key, "score": h.score}))
        .collect();
    (
        json!({
            "status": "ok",
            "route": "/k3/retrieve",
            "engine": "tercode",
            "path": "engine trits packed 2 bits each, scored by the engine's own agreement rule (shared non-zero trits, ties by ascending index)",
            "query": q,
            "k": k,
            "encoder": ENCODER_NAME,
            "scored_documents": out.scanned,
            "tercode_scan_us": out.us,
            "store": out.store,
            "k3": {"verdict": packed_ternary::trit_name(out.verdict), "trit": out.verdict,
                   "basis": out.basis, "score_spread_sd": out.spread},
            "agreement": agreement,
            "results": results,
        }),
        200,
    )
}
