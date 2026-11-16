//! ARC-AGI dataset routes - 20 endpoints over the kaggle/combined corpus.
//! Data: F:\Development sites\TRM agent swarm\kaggle\combined (5 challenge+solution pairs)
use serde_json::{json, Value};
use trm_uefi_kernel::memory_mapped_ipc::MemoryMappedIpc;
use trm_uefi_kernel::unified_transistor_fabric::UnifiedTransistorPool;
use std::path::{Path, PathBuf};

pub const ARC_DIR: &str = r"F:\Development sites\TRM agent swarm\kaggle\combined";

pub fn set_names() -> Vec<&'static str> {
    vec!["training", "training2", "evaluation", "evaluation2", "concept"]
}

use std::sync::{Mutex, OnceLock};
use std::collections::HashMap;

// IPC ARENA (trm_uefi_kernel::memory_mapped_ipc) used as the data cache.
static ARENA: OnceLock<Mutex<MemoryMappedIpc>> = OnceLock::new();
static PARSED: OnceLock<Mutex<HashMap<String, Value>>> = OnceLock::new();

fn arena() -> &'static Mutex<MemoryMappedIpc> {
    ARENA.get_or_init(|| Mutex::new(MemoryMappedIpc::new("arc_arena", 32)))
}
fn parsed() -> &'static Mutex<HashMap<String, Value>> {
    PARSED.get_or_init(|| Mutex::new(HashMap::new()))
}

fn read_json(name: &str) -> Option<Value> {
    if let Some(v) = parsed().lock().ok()?.get(name) {
        return Some(v.clone());
    }
    let text = {
        let mut a = arena().lock().ok()?;
        match a.get(name) {
            Some(s) => s,
            None => {
                let pb = PathBuf::from(ARC_DIR).join(name);
                let s = std::fs::read_to_string(&pb).ok()?;
                a.put(name, &s);
                s
            }
        }
    };
    let v: Value = serde_json::from_str(&text).ok()?;
    parsed().lock().ok()?.insert(name.to_string(), v.clone());
    Some(v)
}

pub fn arena_stats() -> Value {
    let a = match arena().lock() { Ok(g) => g, Err(_) => return json!({"error":"lock"}) };
    let s = a.stats();
    json!({"region": a.region_name, "slots_used": s.slots_used, "slots_total": s.slots_total,
           "bytes_used": s.bytes_used, "hits": s.hit_count, "misses": s.miss_count,
           "evictions": s.evict_count, "puts": s.put_count,
           "source": "trm_uefi_kernel::memory_mapped_ipc"})
}

pub fn challenges(set: &str) -> Option<Value> { read_json(&format!("arc-agi_{}_challenges.json", set)) }
pub fn solutions(set: &str) -> Option<Value> { read_json(&format!("arc-agi_{}_solutions.json", set)) }

fn grid_dims(g: &Value) -> (usize, usize) {
    let rows = g.as_array().map(|r| r.len()).unwrap_or(0);
    let cols = g.as_array().and_then(|r| r.get(0)).and_then(|c| c.as_array()).map(|c| c.len()).unwrap_or(0);
    (rows, cols)
}

fn grid_colors(g: &Value) -> Vec<i64> {
    let mut v: Vec<i64> = vec![];
    if let Some(rows) = g.as_array() {
        for r in rows {
            if let Some(cs) = r.as_array() {
                for c in cs {
                    if let Some(n) = c.as_i64() { if !v.contains(&n) { v.push(n); } }
                }
            }
        }
    }
    v.sort();
    v
}

/// Dispatch one of the 20 ARC routes. Returns Some(json) if handled.
pub fn handle(path: &str, query: &str) -> Option<Value> {
    let seg: Vec<&str> = path.trim_start_matches("/arc/").trim_end_matches('/').split('/').collect();
    let q = |k: &str| -> Option<String> {
        query.split('&').filter_map(|kv| {
            let mut it = kv.splitn(2, '=');
            match (it.next(), it.next()) { (Some(a), Some(b)) if a == k => Some(b.to_string()), _ => None }
        }).next()
    };
    let set = q("set").unwrap_or_else(|| "training".to_string());
    let head = *seg.get(0).unwrap_or(&"");

    match head {
        // 1
        "status" => {
            let mut sets = serde_json::Map::new();
            for s in set_names() {
                let n = challenges(s).and_then(|v| v.as_object().map(|o| o.len())).unwrap_or(0);
                sets.insert(s.to_string(), json!(n));
            }
            Some(json!({"ok": true, "route": "arc/status", "dir": ARC_DIR, "sets": sets}))
        }
        // 2
        "stats" => {
            let ch = challenges(&set)?;
            let so = solutions(&set)?;
            let n = ch.as_object().map(|o| o.len()).unwrap_or(0);
            let sn = so.as_object().map(|o| o.len()).unwrap_or(0);
            let mut pairs = 0usize;
            if let Some(o) = ch.as_object() {
                for (_, t) in o { pairs += t.get("train").and_then(|x| x.as_array()).map(|a| a.len()).unwrap_or(0); }
            }
            Some(json!({"ok": true, "set": set, "tasks": n, "solutions": sn, "train_pairs": pairs}))
        }
        // 3
        "sets" => Some(json!({"ok": true, "sets": set_names()})),
        // 4
        "tasks" => {
            let ch = challenges(&set)?;
            let mut ids: Vec<String> = ch.as_object()?.keys().cloned().collect();
            ids.sort();
            Some(json!({"ok": true, "set": set, "count": ids.len(), "ids": ids}))
        }
        // 5
        "task" => {
            let id = seg.get(1)?;
            let ch = challenges(&set)?;
            Some(json!({"ok": true, "set": set, "id": id, "task": ch.get(id)?}))
        }
        // 6
        "solution" => {
            let id = seg.get(1)?;
            let so = solutions(&set)?;
            Some(json!({"ok": true, "set": set, "id": id, "solution": so.get(id)?}))
        }
        // 7 - train pair count + dims for one task
        "train" => {
            let id = seg.get(1)?;
            let ch = challenges(&set)?;
            let t = ch.get(id)?;
            let tr = t.get("train")?.as_array()?;
            let dims: Vec<Value> = tr.iter().map(|p| {
                let (ir, ic) = grid_dims(p.get("input").unwrap_or(&Value::Null));
                let (orr, oc) = grid_dims(p.get("output").unwrap_or(&Value::Null));
                json!({"in": [ir, ic], "out": [orr, oc]})
            }).collect();
            Some(json!({"ok": true, "id": id, "pairs": tr.len(), "dims": dims}))
        }
        // 8 - test pair count + dims
        "test" => {
            let id = seg.get(1)?;
            let ch = challenges(&set)?;
            let t = ch.get(id)?;
            let te = t.get("test")?.as_array()?;
            let dims: Vec<Value> = te.iter().map(|p| {
                let (ir, ic) = grid_dims(p.get("input").unwrap_or(&Value::Null));
                json!({"in": [ir, ic], "has_output": p.get("output").is_some()})
            }).collect();
            Some(json!({"ok": true, "id": id, "test_pairs": te.len(), "dims": dims}))
        }
        // 9 - one grid: /arc/grid/{id}/{pair}/{which}
        "grid" => {
            let id = seg.get(1)?;
            let pi: usize = seg.get(2)?.parse().ok()?;
            let which = *seg.get(3).unwrap_or(&"input");
            let ch = challenges(&set)?;
            let p = ch.get(id)?.get("train")?.as_array()?.get(pi)?.clone();
            Some(json!({"ok": true, "id": id, "pair": pi, "which": which, "grid": p.get(which)?.clone()}))
        }
        // 10 - distinct colors
        "colors" => {
            let id = seg.get(1)?;
            let ch = challenges(&set)?;
            let p = ch.get(id)?.get("train")?.as_array()?.get(0)?.clone();
            let cs = grid_colors(p.get("input")?);
            Some(json!({"ok": true, "id": id, "colors": cs, "count": cs.len()}))
        }
        // 11 - dimensions summary
        "dims" => {
            let id = seg.get(1)?;
            let ch = challenges(&set)?;
            let t = ch.get(id)?;
            let tr = t.get("train")?.as_array()?;
            let p = tr.get(0)?;
            let (ir, ic) = grid_dims(p.get("input")?);
            let (orr, oc) = grid_dims(p.get("output")?);
            Some(json!({"ok": true, "id": id, "input": [ir, ic], "output": [orr, oc]}))
        }
        // 12 - tasks containing a colour: /arc/search/color/7
        "search" => {
            let kind = *seg.get(1).unwrap_or(&"color");
            let val = *seg.get(2).unwrap_or(&"0");
            let ch = challenges(&set)?;
            let mut hits: Vec<String> = vec![];
            if let Some(o) = ch.as_object() {
                for (id, t) in o {
                    let mut found = false;
                    if let Some(tr) = t.get("train").and_then(|x| x.as_array()) {
                        for p in tr {
                            let cs = grid_colors(p.get("input").unwrap_or(&Value::Null));
                            if kind == "color" {
                                if let Ok(c) = val.parse::<i64>() { if cs.contains(&c) { found = true; } }
                            }
                        }
                    }
                    if found { hits.push(id.clone()); }
                }
            }
            hits.sort();
            Some(json!({"ok": true, "kind": kind, "value": val, "set": set, "count": hits.len(), "ids": hits}))
        }
        // 13 - largest tasks by input area
        "largest" => {
            let ch = challenges(&set)?;
            let mut v: Vec<(usize, String, usize, usize)> = vec![];
            if let Some(o) = ch.as_object() {
                for (id, t) in o {
                    if let Some(p) = t.get("train").and_then(|x| x.as_array()).and_then(|a| a.get(0)) {
                        let (r, c) = grid_dims(p.get("input").unwrap_or(&Value::Null));
                        v.push((r * c, id.clone(), r, c));
                    }
                }
            }
            v.sort_by(|a, b| b.0.cmp(&a.0));
            let top: Vec<Value> = v.iter().take(10).map(|(_, id, r, c)| json!({"id": id, "rows": r, "cols": c})).collect();
            Some(json!({"ok": true, "set": set, "top10": top}))
        }
        // 14 - deterministic "random" task by index
        "pick" => {
            let i: usize = seg.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
            let ch = challenges(&set)?;
            let mut ids: Vec<String> = ch.as_object()?.keys().cloned().collect();
            ids.sort();
            let id = ids.get(i % ids.len().max(1))?.clone();
            Some(json!({"ok": true, "set": set, "index": i, "id": id, "task": ch.get(&id)?}))
        }
        // 15 - encode grid to tokens in the REAL TRM TRAINING FORMAT
        // Source: dataset/build_arc_dataset.py (ARCMaxGridSize=30)
        //   PAD=0, EOS=1, colours = 2..11  (colour + 2)
        //   30x30 canvas; EOS row below the grid, EOS col right of it
        "encode" => {
            let id = seg.get(1)?;
            let ch = challenges(&set)?;
            let p = ch.get(id)?.get("train")?.as_array()?.get(0)?.clone();
            let g = p.get("input")?;
            let (r, c) = grid_dims(g);
            let w = 30usize;
            let mut canvas = vec![0i64; w * w];
            if let Some(rows) = g.as_array() {
                for (ri, row) in rows.iter().enumerate() {
                    if ri >= w { break; }
                    if let Some(cs) = row.as_array() {
                        for (ci, x) in cs.iter().enumerate() {
                            if ci >= w { break; }
                            canvas[ri * w + ci] = x.as_i64().unwrap_or(0) + 2;
                        }
                    }
                }
            }
            // EOS markers (value 1) along the row below and col right of the grid
            if r < w {
                for ci in 0..c.min(w) { canvas[r * w + ci] = 1; }
            }
            if c < w {
                for ri in 0..r.min(w) { canvas[ri * w + c] = 1; }
            }
            Some(json!({"ok": true, "id": id, "rows": r, "cols": c,
                        "len": canvas.len(), "vocab": 12,
                        "format": "TRM training (build_arc_dataset.py)",
                        "mapping": "PAD=0 EOS=1 colours=2..11 = colour+2",
                        "canvas": "30x30",
                        "tokens": canvas}))
        }
        // 16 - decode tokens back to a grid of given width
        "decode" => {
            let id = seg.get(1)?;
            let width: usize = seg.get(2).and_then(|s| s.parse().ok()).unwrap_or(3);
            let ch = challenges(&set)?;
            let p = ch.get(id)?.get("train")?.as_array()?.get(0)?.clone();
            let g = p.get("input")?;
            let mut toks: Vec<i64> = vec![];
            if let Some(rows) = g.as_array() {
                for row in rows {
                    if let Some(cs) = row.as_array() {
                        for x in cs { toks.push(x.as_i64().unwrap_or(10)); }
                    }
                }
            }
            let mut rows_out: Vec<Value> = vec![];
            for chunk in toks.chunks(width.max(1)) { rows_out.push(json!(chunk)); }
            Some(json!({"ok": true, "id": id, "width": width, "grid": rows_out}))
        }
        // 17 - compare dims of two tasks
        "compare" => {
            let a = seg.get(1)?;
            let b2 = seg.get(2)?;
            let ch = challenges(&set)?;
            let ga = ch.get(a)?.get("train")?.as_array()?.get(0)?.get("input")?;
            let gb = ch.get(b2)?.get("train")?.as_array()?.get(0)?.get("input")?;
            let (ar, ac) = grid_dims(ga);
            let (br, bc) = grid_dims(gb);
            Some(json!({"ok": true, "a": a, "a_dims": [ar, ac], "b": b2, "b_dims": [br, bc], "same_dims": ar == br && ac == bc}))
        }
        // 18 - validate solution shape against test input count
        "validate" => {
            let id = seg.get(1)?;
            let ch = challenges(&set)?;
            let so = solutions(&set)?;
            let te = ch.get(id)?.get("test")?.as_array()?;
            let sol = so.get(id)?.as_array()?;
            let ok = te.len() == sol.len();
            let shapes: Vec<Value> = sol.iter().map(|g| { let (r, c) = grid_dims(g); json!([r, c]) }).collect();
            Some(json!({"ok": ok, "id": id, "test_pairs": te.len(), "solutions": sol.len(), "solution_dims": shapes}))
        }
        // 19 - per-task summary
        "summary" => {
            let id = seg.get(1)?;
            let ch = challenges(&set)?;
            let so = solutions(&set)?;
            let t = ch.get(id)?;
            let tr = t.get("train")?.as_array()?;
            let te = t.get("test")?.as_array()?;
            let p = tr.get(0)?;
            let (ir, ic) = grid_dims(p.get("input")?);
            let (orr, oc) = grid_dims(p.get("output")?);
            Some(json!({"ok": true, "id": id, "set": set,
                        "train_pairs": tr.len(), "test_pairs": te.len(),
                        "input_dims": [ir, ic], "output_dims": [orr, oc],
                        "colors": grid_colors(p.get("input")?),
                        "has_solution": so.get(id).is_some()}))
        }
        // 21 - TRM predict (TEST pair) via PyTorch service on :8588
        "predict" => {
            let id = seg.get(1)?;
            let ch = challenges(&set)?;
            if ch.get(id).is_none() { return None; }
            Some(trm_predict(id, query))
        }
        // 20 - integrity: every task has a solution
        "arena" => Some(arena_stats()),
        "pool" => {
            let mut pool = UnifiedTransistorPool::new();
            let compute = pool.total_compute();
            let storage = pool.total_storage();
            let pin = pool.pin("arc_routes", 1_000_000_000);
            Some(json!({
                "source": "trm_uefi_kernel::unified_transistor_fabric",
                "total_compute_transistors": compute,
                "total_storage_cells": storage,
                "pinned": pin.is_some()
            }))
        }
        "integrity" => {
            let ch = challenges(&set)?;
            let so = solutions(&set)?;
            let ck: Vec<&String> = ch.as_object()?.keys().collect();
            let sk: Vec<&String> = so.as_object()?.keys().collect();
            let missing: Vec<&String> = ck.iter().filter(|k| !sk.contains(k)).cloned().collect();
            let orphan: Vec<&String> = sk.iter().filter(|k| !ck.contains(k)).cloned().collect();
            Some(json!({"ok": missing.is_empty() && orphan.is_empty(), "set": set,
                        "tasks": ck.len(), "solutions": sk.len(),
                        "missing_solution": missing, "orphan_solution": orphan}))
        }
        _ => None,
    }
}

/// Calls the PyTorch TRM service (127.0.0.1:8588) for a TEST-pair prediction.
fn trm_predict(id: &str, qs: &str) -> serde_json::Value {
    use std::io::{Read, Write};
    use std::net::TcpStream;
    let pid = qs.split('&').find(|k| k.starts_with("pid="))
        .and_then(|k| k.split('=').nth(1)).unwrap_or("1");
    let path = format!("/predict?task={}&pid={}", id, pid);    let req = format!("GET {} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n", path);
    let mut s = match TcpStream::connect("127.0.0.1:8588") {
        Ok(v) => v,
        Err(e) => return serde_json::json!({"ok": false, "error": format!("trm service unreachable: {}", e)}),
    };
    if s.write_all(req.as_bytes()).is_err() {
        return serde_json::json!({"ok": false, "error": "write failed"});
    }    let mut buf = String::new();
    let _ = s.read_to_string(&mut buf);
    let body = match buf.split("\r\n\r\n").nth(1) {
        Some(b) => b,
        None => return serde_json::json!({"ok": false, "error": "no body"}),
    };
    match serde_json::from_str::<serde_json::Value>(body) {
        Ok(v) => v,
        Err(e) => serde_json::json!({"ok": false, "error": format!("bad json: {}", e)}),
    }
}