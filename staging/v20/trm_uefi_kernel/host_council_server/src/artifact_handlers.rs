//! The artifact pipeline listing, served from the lightweight index that
//! swarm_v2_artifacts/export_artifact_index.py derives from the ArtifactPipeline DB
//! (swarm_v2_memory/artifact_pipeline.json, owned by swarm_v2/core/artifact_pipeline.py).
//!
//! The DB embeds each artifact's full `content` (~21 MB of the 42 MB store), so the offline
//! export strips it; this arm reads the compact index and caches it keyed on (mtime, length),
//! so a re-export is picked up when the file changes without re-parsing on every request.

use serde_json::{json, Value};
use std::sync::Mutex;

use crate::file_cache;

static INDEX: file_cache::Slot<Value> = Mutex::new(None);

/// `/artifacts[?limit=N]` — artifact list, stats, and total, matching the shape the Python
/// route (swarm_v2/routes/artifacts.py) returns. Pending artifacts are listed first (the same
/// prioritization the Python route applies under `include_content`), then the rest, truncated
/// to `limit` (default 150, capped at 500).
pub fn artifacts_payload(url: &str) -> Value {
    let limit = parse_limit(url);

    let Some(path) =
        crate::k3_retrieval::repo_relative("swarm_v2_artifacts/artifact_pipeline_index.json")
    else {
        return unavailable(
            "artifact_pipeline_index.json not found; run swarm_v2_artifacts/export_artifact_index.py",
        );
    };

    let loaded = file_cache::get(&INDEX, &path, |p| {
        let raw = std::fs::read_to_string(p).map_err(|e| e.to_string())?;
        serde_json::from_str::<Value>(&raw).map_err(|e| format!("unparseable index: {e}"))
    });

    let idx = match loaded {
        Ok(arc) => arc,
        Err(e) => return unavailable(&format!("artifact index unreadable: {e}")),
    };

    let stats = idx.get("stats").cloned().unwrap_or_else(|| json!({}));
    let total = stats.get("total").cloned().unwrap_or_else(|| json!(0));

    let mut artifacts: Vec<Value> = idx
        .get("artifacts")
        .and_then(|a| a.as_array())
        .map(|a| a.clone())
        .unwrap_or_default();

    // Pending first, then everyone else, order preserved within each bucket (sort is stable).
    artifacts.sort_by_key(|a| {
        if a.get("status").and_then(|s| s.as_str()) == Some("pending") {
            0
        } else {
            1
        }
    });

    let shown: Vec<Value> = artifacts.into_iter().take(limit).collect();

    json!({
        "stats": stats,
        "total": total,
        "artifacts": shown,
    })
}

fn parse_limit(url: &str) -> usize {
    const DEFAULT: usize = 150;
    const MAX: usize = 500;
    let query = url.splitn(2, '?').nth(1).unwrap_or("");
    for pair in query.split('&') {
        if let Some(v) = pair.strip_prefix("limit=") {
            if let Ok(n) = v.trim().parse::<usize>() {
                return n.min(MAX);
            }
        }
    }
    DEFAULT
}

fn unavailable(reason: &str) -> Value {
    json!({
        "stats": {
            "total": null, "pending": null, "approved": null,
            "rejected": null, "tested": null, "test_failed": null, "integrated": null
        },
        "artifacts": [],
        "unavailable": { "reason": reason }
    })
}
