// /native/fit -- the Inspired Fitness service decided by swarm-fit itself.
//
// swarm-fit is the v18 crate holding the service layer absorbed into Rust: K3-encoded business records, the
// K3 verdict on a business action, and NAND-persisted records whose write is read back to prove it landed. It
// is wired as a path dependency like the others: the crate decides and this boundary adds no rule about the
// verdict, the encoding or the fabric.
//
// The one rule this boundary DOES add is about the filesystem, and it is stated rather than hidden: `serve`
// takes a `record` name, not a path. The name is sanitised to [A-Za-z0-9_-] (max 48 chars) and the path handed
// to the crate is always "<temp>/trm_fit_<name>.k3", resolved by the crate's own
// `swarm_paths::resolve` -- which maps `/tmp/...` onto this host's temp dir itself. An unsanitised caller path
// would let a request write anywhere the kernel can, so the rule is a boundary concern, not a crate
// correction; the crate's own behaviour (append, flush, read back) is untouched.
//
// Recorded rather than smoothed: `serve` returns offset 0 when `fabric.append` fails, so a persistence failure
// is visible as offset 0 with `persisted: false` and the reason -- it is never reported as a successful write.

use serde_json::{json, Value};

use swarm_fit::{encode as fit_encode, serve as fit_serve, verdict as fit_verdict, NandFabric,
                MONAD_ANCHOR};
use swarm_paths::resolve as resolve_path;

fn strings(doc: &Value, key: &str) -> Result<Vec<String>, String> {
    let rows = doc.get(key).and_then(|v| v.as_array())
        .ok_or(format!("`{}` (array of strings) is required", key))?;
    if rows.is_empty() {
        return Err(format!("`{}` is empty: there is nothing to encode", key));
    }
    rows.iter()
        .enumerate()
        .map(|(i, r)| r.as_str().map(|s| s.to_string())
            .ok_or(format!("{}[{}] must be a string", key, i)))
        .collect()
}

fn numbers(doc: &Value, key: &str) -> Result<Vec<f32>, String> {
    let rows = doc.get(key).and_then(|v| v.as_array())
        .ok_or(format!("`{}` (array of numbers, the K3 signals) is required", key))?;
    if rows.is_empty() {
        return Err(format!("`{}` is empty: there is nothing to collapse", key));
    }
    rows.iter()
        .enumerate()
        .map(|(i, r)| r.as_f64().map(|f| f as f32)
            .ok_or(format!("{}[{}] must be a number", key, i)))
        .collect()
}

fn record_name(doc: &Value) -> Result<String, String> {
    let raw = doc.get("record").and_then(|v| v.as_str())
        .ok_or("`record` (string: a name, not a path) is required for the persisted route")?;
    let safe: String = raw.chars().filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
        .take(48).collect();
    if safe.is_empty() {
        return Err(format!("`record` {:?} has no usable characters: [A-Za-z0-9_-] is accepted", raw));
    }
    Ok(safe)
}

/// Deterministic K3 encoding of a business record (pure).
pub fn encode(doc: &Value) -> Result<Value, String> {
    let entity = doc.get("entity").and_then(|v| v.as_str())
        .ok_or("`entity` (string, e.g. \"booking\" | \"client\" | \"wellness\" | \"grant\" | \"proof\") is required")?;
    let fields = strings(doc, "fields")?;
    let refs: Vec<&str> = fields.iter().map(|s| s.as_str()).collect();
    let trits = fit_encode(entity, &refs);
    Ok(json!({
        "entity": entity,
        "trits": trits,
        "len": trits.len(),
        "header": MONAD_ANCHOR.to_vec(),
        "fields": fields,
        "basis": "swarm_fit::encode -- the crate's FNV-derived K3 encoding with its own Monad anchor header; \
                  the boundary adds nothing",
    }))
}

/// The K3 verdict on a business action (pure).
pub fn verdict(doc: &Value) -> Result<Value, String> {
    let signals = numbers(doc, "signals")?;
    let tau = doc.get("tau").and_then(|v| v.as_f64()).unwrap_or(0.35) as f32;
    let v = fit_verdict(&signals, tau);
    Ok(json!({
        "verdict": v.as_str(),
        "signals": signals,
        "tau": tau,
        "basis": "swarm_fit::verdict -- Booked when t >= 2 and f == 0, Blocked when f > t, else Analyze \
                  (the crate's rule on its own collapse at the given tau)",
    }))
}

/// The persisted path: NAND append + flush, then the bytes are read back (the crate's persistence guarantee).
pub fn serve(doc: &Value) -> Result<Value, String> {
    let entity = doc.get("entity").and_then(|v| v.as_str())
        .ok_or("`entity` (string) is required")?;
    let fields = strings(doc, "fields")?;
    let signals = numbers(doc, "signals")?;
    let name = record_name(doc)?;
    let path = resolve_path(&format!("/tmp/trm_fit_{}.k3", name));
    let mut fabric = NandFabric::open(&path)
        .map_err(|e| format!("the fabric at {:?} could not be opened: {:?}", path, e))?;
    let refs: Vec<&str> = fields.iter().map(|s| s.as_str()).collect();
    let (verdict, offset, trits) = fit_serve(&mut fabric, entity, &refs, &signals);
    // read back exactly what was written -- the crate's own guarantee, exercised here rather than asserted
    let read_back = if offset > 0 { fabric.read(offset, trits.len()).unwrap_or_default() } else { Vec::new() };
    Ok(json!({
        "verdict": verdict.as_str(),
        "offset": offset,
        "trits": trits,
        "persisted": offset > 0,
        "byte_identical": !read_back.is_empty() && read_back == trits,
        "record_path": path,
        "basis": "swarm_fit::serve -- encode + fabric append + flush, with the bytes read back through the \
                  same fabric to prove they landed",
        "notes": [
            "`record` is a NAME, not a path: [A-Za-z0-9_-] only, written to <temp>/trm_fit_<name>.k3",
            "the path is resolved by the crate's own swarm_paths::resolve, which maps /tmp onto this host",
            "offset 0 means the crate's append failed -- reported as persisted:false, never as a write",
        ],
    }))
}
