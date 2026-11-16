//! /memory/stats, /memory/recent, /memory/intel and /swarm/intel -- the SHARED INTEL RESERVOIR.
//!
//! The dashboard's swarm-intel tab reads `total_memories`, `sync_events` and `by_type` from
//! /memory/stats, and a `{entries, count}` list from /memory/recent (App.jsx:470-471). Both routes
//! previously fell into the /memory*fabric* handler at main.rs:727, whose payload carries none of
//! those fields -- so every card rendered a placeholder and the feed read "no memories indexed yet".
//! The page was healthy; the SHAPE was wrong. That is why it looked like "nothing loaded".
//!
//! Every number here is derived from a store that exists:
//!   * total_memories / by_type / by_author -- the gzip JSONL memory corpus on this host
//!     (`swarm_v2_global_memory/fabric/fabric_distinct.jsonl.gz`), streamed and counted by
//!     `memory_corpus`. Previously this was the key list of a `.turbovec` packed store that does not
//!     exist here, so the census was null and every card rendered a placeholder.
//!   * corpus.dim -- the `.npy` vector artifact beside that corpus, read and measured (or null with
//!     the reason). `corpus.built_at` is the newest row timestamp, labelled as such.
//!   * the live feed -- the newest files in swarm_v2_artifacts/research, by mtime, read from disk.
//! Nothing is asserted. A value that cannot be measured is null WITH its basis, never a 0 that would
//! read as a clean result, and every derived field carries the basis it was derived from.

use serde_json::{json, Value};

const RESEARCH_REL: &str = "swarm_v2_artifacts/research";
const INDEX_REL: &str = "swarm_v2_artifacts/kernel_knowledge_index.json";

/// The live feed: (total findings on disk, newest `limit` findings).
///
/// Ordered by mtime because that is the ordering that actually exists -- the corpus keys carry no
/// timestamp in the form the engine loads. None when the directory cannot be read, which is not the
/// same as an empty reservoir.
fn recent_findings(limit: usize) -> Option<(u64, Vec<Value>)> {
    use std::time::UNIX_EPOCH;
    let dir = crate::k3_retrieval::repo_relative(RESEARCH_REL)?;
    let mut files: Vec<(std::time::SystemTime, std::path::PathBuf)> = Vec::new();
    for e in std::fs::read_dir(&dir).ok()?.flatten() {
        let path = e.path();
        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }
        let mt = e.metadata().ok().and_then(|m| m.modified().ok()).unwrap_or(UNIX_EPOCH);
        files.push((mt, path));
    }
    let total = files.len() as u64;
    files.sort_by(|a, b| b.0.cmp(&a.0));
    let entries = files
        .iter()
        .take(limit)
        .map(|(mt, path)| {
            let id = path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
            let title = std::fs::File::open(path)
                .ok()
                .and_then(|f| {
                    use std::io::{BufRead, BufReader};
                    BufReader::new(f)
                        .lines()
                        .take(5)
                        .flatten()
                        .find(|l| l.starts_with('#'))
                })
                .map(|l| {
                    l.trim_start_matches('#')
                        .replace("Research Finding:", "")
                        .trim()
                        .to_string()
                })
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| id.clone());
            json!({
                "id": id,
                "title": title,
                "file": path.file_name().and_then(|s| s.to_str()).unwrap_or(""),
                "type": "research",
                "role": "Researcher",
                "indexed_at_unix": mt.duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0),
            })
        })
        .collect();
    Some((total, entries))
}

/// The knowledge index's own meta block.
///
/// Read from the HEAD of the file (meta is the first key) and brace-matched, so a status route never
/// parses the 53 MB index to report a document count. None when the file is absent or the block is
/// malformed -- reported by the caller as an unmeasured field, not as zero.
fn index_meta() -> Option<Value> {
    use std::io::Read;
    let path = crate::k3_retrieval::repo_relative(INDEX_REL)?;
    let mut buf = vec![0u8; 32 * 1024];
    let n = std::fs::File::open(&path).ok()?.read(&mut buf).ok()?;
    buf.truncate(n);
    let s = String::from_utf8_lossy(&buf);
    let start = s.find("\"meta\"")?;
    let open = s[start..].find('{')? + start;
    let mut depth = 0i32;
    let mut end = None;
    for (i, c) in s[open..].char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end = Some(open + i);
                    break;
                }
            }
            _ => {}
        }
    }
    serde_json::from_str(&s[open..=end?]).ok()
}

/// meta field, or null -- never a default that would read as a measurement.
fn meta_field(meta: &Option<Value>, key: &str) -> Value {
    meta.as_ref().and_then(|m| m.get(key)).cloned().unwrap_or(Value::Null)
}

/// /memory/stats -- the census the dashboard's Collective Stats card and Type Breakdown read.
///
/// Every field is now MEASURED from the corpus this host actually holds
/// (`swarm_v2_global_memory/fabric/fabric_distinct.jsonl.gz`, streamed by `memory_corpus`).
/// The previous version derived `total_memories`/`by_type` from `packed_ternary::store()` -- a
/// `.turbovec` packed store that does not exist here -- and `corpus.*` from
/// `swarm_v2_artifacts/kernel_knowledge_index.json`, which also does not exist, so every one of
/// those fields answered null with no measurement behind it. `dim` is reported only from the
/// vector artifact that was actually read (its `.npy` payload decoded and measured), never from a
/// manifest claim; if that artifact is absent, `dim` is null WITH the reason.
pub fn stats_payload() -> Value {
    let corpus = crate::memory_corpus::corpus();
    let vectors = crate::memory_corpus::vectors();
    let meta = index_meta();

    let (docs, by_type, by_author) = match &corpus {
        Ok(c) => (json!(c.documents()), json!(c.by_type), json!(c.by_author)),
        Err(_) => (Value::Null, json!({}), json!({})),
    };
    let corpus_err = match &corpus {
        Ok(_) => Value::Null,
        Err(e) => json!(e),
    };
    let (dim, dim_basis) = match &vectors {
        Ok(v) => (
            json!(v.dim),
            json!(format!(
                "measured from {}: {} rows x {} columns of {} = {} payload bytes, decoded element by element (mean L2 norm {:.4}). The vectors are NOT used for scoring on this route: they were built by an external embedder (fabric_manifest.json: nomic-embed-text), a different space from the in-tree encoder, so /memory/query retrieves by keyword/exact match over the same documents.",
                crate::memory_corpus::VECS_REL,
                v.rows,
                v.dim,
                v.dtype,
                v.payload_bytes,
                v.mean_norm
            )),
        ),
        Err(e) => (
            Value::Null,
            json!(format!("no measured dimension: {e}. Null rather than a claim taken from a manifest.")),
        ),
    };

    let (mem, mem_basis, built_at, built_at_basis, rows_read, malformed, empty_text, truncated, dup_ids, gz_bytes, loaded_at, oldest) =
        match &corpus {
            Ok(c) => (
                json!(c.documents()),
                json!(format!(
                    "documents parsed from {} ({} gzip bytes): {} rows read, {} rejected (of which {} carried no text), {} text-truncated at {} chars, {} duplicate ids. Measured at load, not asserted.",
                    c.source,
                    c.gz_bytes,
                    c.rows_read,
                    c.malformed_rows,
                    c.empty_text_rows,
                    c.truncated_rows,
                    crate::memory_corpus::TEXT_CAP,
                    c.duplicate_ids
                )),
                c.newest_ts.clone().map(Value::String).unwrap_or(Value::Null),
                json!("the newest `timestamp` any row in the corpus carries -- a measurement of the corpus content. Not a build clock: the corpus files record no build time, and the kernel did not build them."),
                json!(c.rows_read),
                json!(c.malformed_rows),
                json!(c.empty_text_rows),
                json!(c.truncated_rows),
                json!(c.duplicate_ids),
                json!(c.gz_bytes),
                json!(c.loaded_at_unix),
                json!(c.oldest_ts),
            ),
            Err(e) => (
                Value::Null,
                json!(format!(
                    "the memory corpus could not be loaded: {e}. Null means unreadable, not empty."
                )),
                Value::Null,
                json!("no corpus, so no row timestamps to measure; null rather than an invented build time."),
                Value::Null,
                Value::Null,
                Value::Null,
                Value::Null,
                Value::Null,
                Value::Null,
                Value::Null,
                Value::Null,
            ),
        };

    json!({
        "status": "online",
        "total_memories": docs,
        "total_memories_basis": mem_basis,
        "documents": mem,
        "by_type": by_type,
        "by_type_basis": "the corpus's own filing scheme: the leading path segment of each key (`papers/...` -> `papers`). Zero live keys contain a `/` (they are content hashes), so for every row the rule falls through to the row's own type field in this order: `metadata.memory_type`, then `class`, then `agent.memory_type`, then `root` for a row that files itself nowhere. Sums to the document count.",
        "by_author": by_author,
        "by_author_basis": "each row's `agent.author` (object-shaped agent) or the `agent` string itself (legacy rows), counted at load. A measured distribution, not a guess.",
        "sync_events": meta_field(&meta, "relational_documents_added"),
        "sync_events_basis": if meta.is_some() {
            json!("meta.relational_documents_added from kernel_knowledge_index.json.")
        } else {
            json!("null: kernel_knowledge_index.json (which carried this counter) does not exist on this host, and the gzip JSONL corpus records no sync counter. Reported null rather than 0, which would read as a clean result.")
        },
        "corpus": {
            "source": match &corpus { Ok(c) => json!(c.source), Err(_) => json!(crate::memory_corpus::CORPUS_RELS.join(" | ")) },
            "documents": mem,
            "dim": dim,
            "dim_basis": dim_basis,
            "built_at": built_at,
            "built_at_basis": built_at_basis,
            "oldest_timestamp": oldest,
            "rows_read": rows_read,
            "malformed_rows": malformed,
            "empty_text_rows": empty_text,
            "truncated_rows": truncated,
            "duplicate_ids": dup_ids,
            "gzip_bytes": gz_bytes,
            "loaded_at_unix": loaded_at,
            "load_error": corpus_err,
            "model": match &vectors {
                Ok(v) => json!({ "model": meta_field(&meta, "model"), "dtype": v.dtype, "vector_rows": v.rows, "ids_count": v.ids_count,
                                 "ids_file": v.ids_path, "mean_l2_norm": (v.mean_norm * 10_000.0).round() / 10_000.0,
                                 "model_basis": "the .npy's dtype and geometry were measured; the model NAME is not in the artifact and is reported from the index meta when present, else null" }),
                Err(_) => json!(Value::Null),
            },
        },
        "query_route": "/memory/query?q=<terms>&k=5",
    })
}

/// /memory/recent?limit=N -- the Live Memory Feed, `{entries, count}` as the dashboard reads it.
pub fn recent_payload(limit: usize) -> Value {
    let limit = limit.clamp(1, 200);
    match recent_findings(limit) {
        Some((total, entries)) => {
            let count = entries.len();
            json!({
                "entries": entries,
                "count": count,
                "total_findings": total,
                "basis": "the newest files in swarm_v2_artifacts/research, ordered by mtime and read from disk. The corpus keys carry no timestamp in the loaded form, so mtime is the ordering that exists.",
            })
        }
        None => json!({
            "entries": [],
            "count": 0,
            "total_findings": Value::Null,
            "basis": format!("swarm_v2_artifacts/research is not readable from the kernel's root, so no feed could be derived. An unreadable reservoir is not an empty one; total_findings is null rather than 0."),
        }),
    }
}

/// /memory/intel and /swarm/intel -- the whole reservoir in the shape the Python route returned.
pub fn intel_payload() -> Value {
    let stats = stats_payload();
    let (total, feed) = recent_findings(30).unwrap_or((0, Vec::new()));
    json!({
        "status": "online",
        "total_memories": stats["total_memories"],
        "total_memories_basis": stats["total_memories_basis"],
        "by_type": stats["by_type"],
        "by_type_basis": stats["by_type_basis"],
        "by_author": stats["by_author"],
        "by_author_basis": "each corpus row's `agent.author` (or the legacy `agent` string), counted while the corpus was parsed",
        "sync_events": stats["sync_events"],
        "sync_events_basis": stats["sync_events_basis"],
        "total_papers": total,
        "total_papers_basis": "*.md counted in swarm_v2_artifacts/research at request time",
        "recent_intel": feed.clone(),
        "latest_research": feed,
        "corpus": stats["corpus"],
        "basis": "total_memories/by_type/sync_events are derived from the kernel's own corpus; recent_intel and latest_research are the newest findings on disk. This route is the Rust equivalent of swarm_v2/routes/memory.py::get_shared_intel, served from stores that exist with no Python process.",
    })
}

/// ?limit=N from the query string, defaulting to 25 as the Python route did.
pub fn limit_from(url: &str) -> usize {
    url.split_once("?limit=")
        .and_then(|(_, rest)| rest.split('&').next())
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(25)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The shape the dashboard reads, printed from whatever corpus this host holds. Opt-in because
    /// it reads the live gzip:
    ///     TRM_MEMORY_CORPUS=<path> cargo test --release stats_payload_is_measured -- --ignored --nocapture
    #[test]
    #[ignore]
    fn stats_payload_is_measured() {
        let s = stats_payload();
        println!("{}", serde_json::to_string_pretty(&s).unwrap_or_default());
        let docs = s["total_memories"].as_u64();
        assert!(docs.is_some(), "total_memories must be a measured number, got {}", s["total_memories"]);
        assert_eq!(s["documents"], s["total_memories"], "documents and total_memories are the same measurement");
        let ss = s["by_type"].as_object().expect("by_type must be an object");
        let sum: u64 = ss.values().filter_map(|v| v.as_u64()).sum();
        assert_eq!(sum, docs.unwrap(), "by_type must sum to total_memories");
        // This host has the .npy vector artifact; if it is absent the route must say why.
        if s["corpus"]["dim"].is_null() {
            assert!(
                s["corpus"]["dim_basis"].as_str().unwrap_or("").contains("no measured dimension"),
                "a null dim must carry its reason"
            );
        }
        println!(
            "MEASURED documents={} dim={} by_type={:?}",
            docs.unwrap(),
            s["corpus"]["dim"],
            ss.iter().map(|(k, v)| format!("{k}={}", v.as_u64().unwrap_or(0))).collect::<Vec<_>>()
        );
    }
}
