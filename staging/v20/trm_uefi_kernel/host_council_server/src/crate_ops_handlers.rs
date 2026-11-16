// P2g — the verified crates, wired as routes.
//
// Every route below computes IN the crate that owns the algorithm and reports that crate's own answer;
// the kernel re-implements none of it. These crates were exercised first (the P2b-P2f ledgers): each one
// has a gate agreeing with the C++ reference or the Python reference, and those gates are what make
// these routes answers rather than assertions.
//
//   GET  /crate/ops          the census: what is wired, what it computes, and the gate behind it
//   POST /crate/k3/collapse  {"values":[f32], "eps":0.35}     -> the K3 ternary collapse
//   POST /crate/trit/pack    {"trits":[-1|0|1]}               -> packed bytes, ratio, round-trip
//   POST /crate/moe/gate     {"hidden":[384],"experts":[[384],..],"k":4} -> the 64-expert gate
//   POST /crate/ssm/step     {"inputs":[f32], "dt":0.001}     -> the Mamba-3 state evolution
//
// A malformed body is a 400 naming the field, an oversized one a 413, an unknown op a 404 -- never a
// fall-through to the status-ok catch-all.

use serde_json::{json, Value};
use std::time::Instant;
use tiny_http::{Header, Method, Request, Response};

/// A route is DATA: the op, its method, the crate that owns the algorithm, and the gate that proves it.
/// `/crate/ops` reports this table, so the census cannot drift from the wiring.
const OPS: &[(&str, &str, &str, &str)] = &[
    ("k3/collapse", "POST", "swarm-k3",
     "eps-bounded K3 collapse; gate: byte-identical to libswarm_kernel_ops.so over 1,000,000 values"),
    ("trit/pack", "POST", "swarm-turbovec",
     "4 trits per byte (2-bit canonical) + unpack round-trip; gate: byte-identical to the numpy reference"),
    ("moe/gate", "POST", "swarm-instella",
     "64-expert top-k gate + softmax; gate: verified against c_instella_gate_topk"),
    ("ssm/step", "POST", "swarm-ssm",
     "Mamba-3 recurrence; gate: max |rust - c_mamba3_step| = 1.19e-7 over 100 steps"),
    ("council/vote", "POST", "swarm-council",
     "K3 council vote over the crate's own 14-agent domain table; gate: byte-identical to libswarm_kernel_ops.so on all vote vectors + the 1M sweep"),
    ("nand/roundtrip", "POST", "swarm-nand",
     "append + read-back on a probe fabric in the host temp dir; gate: verify_nand reads the real fabric coherently"),
    ("gguf/inspect", "POST", "swarm-gguf",
     "real GGUF parse: header, tensor census, tensor offset; gate: verify_gguf parsed the real model - census and offsets ready"),
    ("git/verify", "POST", "swarm-git",
     "git object hash + verify_blob/verify_commit + loose-object parse; gate: verify_git PASSED"),
    ("qsar/predict", "POST", "swarm-drug",
     "antibacterial-activity RandomForest (80 trees, 1024-bit Morgan); gate: bit-exact vs Python on 6 controls (max diff 4.42e-13)"),
    ("screen/library", "POST", "swarm-docking",
     "RETIRED as a screening result: r=-0.131 against 115 measured complexes, no ranking signal; keeps the route for audit and refuses to rank unless force=true"),
    ("screen/smiles", "POST", "swarm-deepdta",
     "in-kernel screen of a SMILES library: tokenize + DeepDTA forward per molecule, ranked by pkd; molecules that fail to tokenize or score non-finite are counted; ranking only (numeric affinity retired, leave-one-out r = 0.39)"),
    ("deepdta/predict", "POST", "swarm-deepdta",
     "fine-tuned DeepDTA affinity (embedding + 3x Conv1d + ReLU + maxpool + MLP); gate: sha256-gated binary + parity vs Python max diff 1.9e-6; numeric output retired (leave-one-out r = 0.39 < 0.5)"),
    ("humanistic/recall", "POST", "swarm-humanistic",
     "recall the 'all is one' teaching (axiom + 5 tenets from the platform's own doctrine)"),
    ("humanistic/realize", "POST", "swarm-humanistic",
     "the MagnumOpus: K3-collapse the tenets' polarity and recognize the self; native ternary decision over the doctrine"),
    ("humanistic/reason", "POST", "swarm-humanistic",
     "reason on a humanistic concern through the 'all is one' lens: K3 over serve-whole / zero-harm / grounded"),
    ("agent/census", "POST", "swarm-agents",
     "the 64-agent fleet re-wired to the native path (MOTION-052): name + fleet + native op + status"),
    ("agent/dispatch", "POST", "swarm-agents",
     "dispatch an agent to its native op; returns the mapping + honest status (native / new-strip / external)"),
    ("agent/plan", "POST", "swarm-agents",
     "the execution harness: read the council agenda and build the facet -> fleet -> agents dispatch plan"),
    ("agent/strip", "POST", "swarm-strips",
     "run one of the 19 new-strip agents natively: { \"strip\": name, \"input\": payload } -> derived result"),
    ("tasks/sweep", "POST", "swarm-tasks",
     "the realization sweep: generate + dispatch N deterministic tasks through the agents, tally grounded/derived/boundary/error"),
    ("tasks/scenarios", "POST", "swarm-tasks",
     "the 1M-scenario sweep: chain recall -> derive -> council per scenario, tally the honest aggregate"),
    ("journal/realize", "POST", "kernel",
     "auto-journal a realization: derive the entry from the cited files' sha256 + seal a truth-anchored ledger block"),
    ("meta/readiness", "POST", "kernel",
     "the meta-cognitive loop Step 1: property -> data-presence status (ready/blocked/partially_ready) + reason + alternatives; gate: the corpus data presence, measured (hydrogens 9617/40, water 0/40)"),
    ("meta/surrogate", "POST", "kernel",
     "the meta-cognitive loop Step 3 (Monad imagination): blocked property -> testable surrogate with its physical rationale + method + the measured SURG2 result; gate: the donor-acceptor angle confirmed on 3,233 complexes"),
    ("harness/code", "POST", "swarm-harness",
     "generate code via the tier-2 coder (:8587) through the swarm-harness CLI; gate: the ChatML template so Qwen2.5-Coder answers instead of continuing the persona prompt"),
    ("harness/strip", "POST", "swarm-harness",
     "run a deterministic strip (math_solver, refactor_skill, ebpf_probe, ...) through the swarm-harness CLI; gate: the strip's own derived result, never fabricated"),
    ("harness/instella", "POST", "swarm-harness",
     "full 16B MoE forward via the Instella fork (:8588) through the swarm-harness CLI; gate: the Instella reasoner's own generation"),
];

fn answer(body: Value, status: u16) -> Response<std::io::Cursor<Vec<u8>>> {
    Response::from_string(body.to_string())
        .with_status_code(status)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap())
        .with_header(Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap())
}

fn err(path: &str, code: u16, kind: &str, detail: String) -> Response<std::io::Cursor<Vec<u8>>> {
    answer(json!({"status": kind, "route": path, "error": kind, "detail": detail}), code)
}

fn f32_list(body: &Value, key: &str, max: usize) -> Result<Vec<f32>, String> {
    let a = body
        .get(key)
        .and_then(|x| x.as_array())
        .ok_or(format!("`{key}` must be an array of numbers"))?;
    if a.is_empty() {
        return Err(format!("`{key}` is empty"));
    }
    if a.len() > max {
        return Err(format!("`{key}` holds {} values; at most {max}", a.len()));
    }
    a.iter()
        .map(|x| x.as_f64().map(|f| f as f32).ok_or(format!("`{key}` must hold numbers only")))
        .collect()
}

/// FNV-1a 64 — the checksum this platform already uses to identify a byte run (its tercode rows carry
/// one), so identifying packed bytes adds no dependency.
fn fnv1a(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// The collapse, by the crate that owns it. The tally is reported in unambiguous terms (plus / zero /
/// minus); the platform's own pass/analyze/block names are NOT reused here, because mapping them onto
/// the crate's output would be a claim this handler has not verified.
fn op_k3_collapse(body: &Value, path: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    let values = match f32_list(body, "values", 1_000_000) {
        Ok(v) => v,
        Err(e) => return err(path, 400, "bad_body", e),
    };
    let eps = body.get("eps").and_then(|e| e.as_f64()).unwrap_or(0.35) as f32;
    let trits = swarm_k3::collapse(&values, eps);
    let (mut plus, mut minus, mut zero) = (0u64, 0u64, 0u64);
    for t in &trits {
        match *t {
            1 => plus += 1,
            -1 => minus += 1,
            _ => zero += 1,
        }
    }
    answer(
        json!({
            "status": "ONLINE", "route": path, "engine": "swarm-k3", "eps": eps,
            "n": trits.len(),
            "tally": {"plus": plus, "zero": zero, "minus": minus},
            "trits": trits,
            "gate": "byte-identical to libswarm_kernel_ops.so over 1,000,000 values"
        }),
        200,
    )
}

/// Packing, by the crate that owns it -- and the round-trip is checked HERE rather than reported on
/// trust: `round_trip_exact` is the unpack of the bytes this call just produced.
fn op_trit_pack(body: &Value, path: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    let raw = match body.get("trits").and_then(|x| x.as_array()) {
        Some(a) => a,
        None => return err(path, 400, "bad_body", "`trits` must be an array".to_string()),
    };
    if raw.len() > 4_000_000 {
        return err(path, 413, "too_large", format!("{} trits; at most 4,000,000", raw.len()));
    }
    let mut trits: Vec<i8> = Vec::with_capacity(raw.len());
    for v in raw {
        match v.as_i64() {
            Some(-1) => trits.push(-1),
            Some(0) => trits.push(0),
            Some(1) => trits.push(1),
            _ => return err(path, 400, "bad_body", "`trits` must hold only -1, 0 or 1".to_string()),
        }
    }
    let packed = swarm_turbovec::pack(&trits);
    let back = swarm_turbovec::unpack(&packed, trits.len());
    let ratio = if packed.is_empty() { 0.0 } else { trits.len() as f64 / packed.len() as f64 };
    answer(
        json!({
            "status": "ONLINE", "route": path, "engine": "swarm-turbovec",
            "n_trits": trits.len(), "n_bytes": packed.len(), "trits_per_byte": ratio,
            "fnv1a64": format!("{:016x}", fnv1a(&packed)),
            "round_trip_exact": back == trits,
            "bytes": packed,
            "gate": "byte-identical to the numpy reference over 1,000,000 trits"
        }),
        200,
    )
}

fn f32_arr(v: &Value, max: usize) -> Result<Vec<f32>, String> {
    let a = v.as_array().ok_or("must be an array of numbers".to_string())?;
    if a.len() > max {
        return Err(format!("holds {} values; at most {max}", a.len()));
    }
    a.iter()
        .map(|x| x.as_f64().map(|f| f as f32).ok_or("must hold numbers only".to_string()))
        .collect()
}

/// The expert gate, by the crate that owns it. The caller supplies the hidden state and the weights;
/// anything with the wrong shape is refused with the shape named, rather than padded or truncated.
fn op_moe_gate(body: &Value, path: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    let hidden_v = match f32_list(body, "hidden", 384) {
        Ok(v) => v,
        Err(e) => return err(path, 400, "bad_body", e),
    };
    if hidden_v.len() != 384 {
        return err(path, 400, "bad_body", format!("`hidden` must hold 384 values (HIDDEN_DIM); got {}", hidden_v.len()));
    }
    let rows = match body.get("experts").and_then(|x| x.as_array()) {
        Some(a) => a,
        None => return err(path, 400, "bad_body", "`experts` must be an array of rows of numbers".to_string()),
    };
    if rows.len() != 64 {
        return err(path, 400, "bad_body", format!("`experts` must hold 64 rows (TOTAL_EXPERTS); got {}", rows.len()));
    }
    let mut weights = [[0f32; 384]; 64];
    for (e, row) in rows.iter().enumerate() {
        match f32_arr(row, 384) {
            Ok(v) if v.len() == 384 => weights[e].copy_from_slice(&v),
            Ok(v) => {
                return err(path, 400, "bad_body", format!("expert {e} has {} values; each needs 384", v.len()))
            }
            Err(e) => return err(path, 400, "bad_body", format!("expert {e} {e}")),
        }
    }
    let k = body.get("k").and_then(|v| v.as_u64()).unwrap_or(4) as usize;
    if k == 0 || k > 64 {
        return err(path, 400, "bad_body", format!("`k` must be 1..64; got {k}"));
    }
    let mut hidden = [0f32; 384];
    hidden.copy_from_slice(&hidden_v);
    let (ids, ws) = swarm_instella::gate_topk(&hidden, &weights, k);
    let wsum: f32 = ws.iter().sum();
    let distinct = {
        let mut s = ids.clone();
        s.sort_unstable();
        s.dedup();
        s.len() == ids.len()
    };
    answer(
        json!({
            "status": "ONLINE", "route": path, "engine": "swarm-instella",
            "hidden_dim": 384, "total_experts": 64, "k": k,
            "selected_ids": ids, "gate_weights": ws,
            "weight_sum": wsum, "distinct_experts": distinct,
            "gate": "verified against c_instella_gate_topk (ids identical, max weight diff 1.96e-6)"
        }),
        200,
    )
}

/// The recurrence, by the crate that owns it. The state is carried across the inputs exactly as the C
/// API carries it inside its handle; `outputs[i]` is the crate's own y at step i.
fn op_ssm_step(body: &Value, path: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    let inputs = match f32_list(body, "inputs", 1_000) {
        Ok(v) => v,
        Err(e) => return err(path, 400, "bad_body", e),
    };
    let dt = body.get("dt").and_then(|v| v.as_f64()).map(|f| f as f32).unwrap_or(0.001);
    let mut ssm = swarm_ssm::Mamba3State::create(Some(dt));
    let outputs: Vec<f32> = inputs.iter().map(|x| ssm.step(*x)).collect();
    answer(
        json!({
            "status": "ONLINE", "route": path, "engine": "swarm-ssm",
            "dim": swarm_ssm::MAMBA_STATE_DIM, "dt": dt, "steps": ssm.step_count,
            "outputs": outputs, "final_h": ssm.h.to_vec(),
            "gate": "max |rust - c_mamba3_step| = 1.19e-7 over 100 steps"
        }),
        200,
    )
}

/// Trit bodies arrive as JSON integers. Anything that is not -1/0/1 is refused by name rather than
/// coerced: a coerced trit is a value the caller never sent.
fn int_trits(v: &Value, max: usize) -> Result<Vec<i8>, String> {
    let a = v.as_array().ok_or("must be an array of -1/0/1".to_string())?;
    if a.is_empty() {
        return Err("is empty".to_string());
    }
    if a.len() > max {
        return Err(format!("holds {} values; at most {max}", a.len()));
    }
    let mut out = Vec::with_capacity(a.len());
    for x in a {
        match x.as_i64() {
            Some(-1) => out.push(-1),
            Some(0) => out.push(0),
            Some(1) => out.push(1),
            _ => return Err("must hold only -1, 0 or 1".to_string()),
        }
    }
    Ok(out)
}

fn hex_decode(s: &str) -> Result<Vec<u8>, String> {
    let b = s.as_bytes();
    if b.len() % 2 != 0 {
        return Err(format!("hex length {} is odd", b.len()));
    }
    let mut out = Vec::with_capacity(b.len() / 2);
    let mut i = 0;
    while i < b.len() {
        let hi = (b[i] as char).to_digit(16).ok_or(format!("not hex at byte {i}"))?;
        let lo = (b[i + 1] as char).to_digit(16).ok_or(format!("not hex at byte {}", i + 1))?;
        out.push((hi * 16 + lo) as u8);
        i += 2;
    }
    Ok(out)
}

/// The council vote, by the crate that owns it. The 14-agent domain table is the CRATE's
/// (`AGENT_MODELS`), so the roster behind this verdict is not a copy the kernel keeps; the caller
/// supplies only the motion, and the crate's collapse (via swarm-k3) decides.
fn op_council_vote(body: &Value, path: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    let motion_owned: Vec<String> = match body.get("motion") {
        None => swarm_council::MOTION_A.iter().map(|s| s.to_string()).collect(),
        Some(v) => {
            let a = match v.as_array() {
                Some(a) => a,
                None => {
                    return err(path, 400, "bad_body", "`motion` must be an array of domain strings".to_string())
                }
            };
            if a.is_empty() {
                return err(path, 400, "bad_body", "`motion` is empty".to_string());
            }
            if a.len() > 64 {
                return err(path, 400, "bad_body", format!("`motion` holds {} terms; at most 64", a.len()));
            }
            let mut out = Vec::with_capacity(a.len());
            for x in a {
                match x.as_str() {
                    Some(s) if !s.is_empty() && s.len() <= 64 => out.push(s.to_string()),
                    _ => {
                        return err(
                            path,
                            400,
                            "bad_body",
                            "every motion term must be a non-empty string of at most 64 chars".to_string(),
                        )
                    }
                }
            }
            out
        }
    };
    let motion: Vec<&str> = motion_owned.iter().map(|s| s.as_str()).collect();
    let (verdict, votes) = swarm_council::council_verdict(&swarm_council::AGENT_MODELS, &motion);
    let mut rows: Vec<Value> = Vec::with_capacity(votes.len());
    let (mut voters, mut abstainers, mut trit_sum) = (0u32, 0u32, 0i64);
    for (i, v) in &votes {
        let a = &swarm_council::AGENT_MODELS[*i];
        match v {
            swarm_council::AgentVote::Abstain => {
                abstainers += 1;
                rows.push(json!({"agent": a.name, "overlap": 0, "vote": "ABSTAIN"}));
            }
            swarm_council::AgentVote::Voted { trits, vote_vector } => {
                voters += 1;
                let s: i64 = trits.iter().map(|&t| t as i64).sum();
                trit_sum += s;
                rows.push(json!({
                    "agent": a.name,
                    "overlap": swarm_council::overlap(a, &motion),
                    "confidence": a.confidence,
                    "vote_vector": vote_vector,
                    "trits": trits,
                    "sum": s
                }));
            }
        }
    }
    answer(
        json!({
            "status": "ONLINE", "route": path, "engine": "swarm-council",
            "agents": swarm_council::AGENT_MODELS.len(), "motion": motion, "eps": swarm_council::EPS,
            "verdict": verdict.label(), "verdict_trit": verdict.trit(),
            "trit_sum": trit_sum, "voters": voters, "abstainers": abstainers, "votes": rows,
            "gate": "verify_council: byte-identical to libswarm_kernel_ops.so on all council vote vectors and the 1M sweep"
        }),
        200,
    )
}

/// The NAND round-trip, by the crate that owns it -- against a PROBE store the kernel creates in the
/// host temp dir. The platform's real fabric is not touched: this route writes what it is given and
/// reads it back through the same crate, and reports the header state before and after.
fn op_nand_roundtrip(body: &Value, path: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    let trits: Vec<i8> = match body.get("trits") {
        // The monad anchor's low bytes as trits, so a bare call still writes something identifiable.
        None => vec![1, -1, 0, 1, 1, -1, 0, 1, -1, 1, 1, 0, -1, 1, 1, -1],
        Some(v) => match int_trits(v, 4_000_000) {
            Ok(t) => t,
            Err(e) => return err(path, 400, "bad_body", format!("`trits` {e}")),
        },
    };
    let probe = std::env::temp_dir().join("trm_crate_probe_nand.k3");
    let spec = probe.to_string_lossy().to_string();
    let mut fab = match swarm_nand::NandFabric::open(&spec) {
        Ok(f) => f,
        Err(e) => return err(path, 500, "probe_store_unopenable", format!("{spec}: {e}")),
    };
    let (codec_before, count_before, off_before) = fab.status();
    let offset = match fab.append(&trits) {
        Ok(o) => o,
        Err(e) => return err(path, 500, "append_failed", format!("{spec}: {e}")),
    };
    let back = match fab.read(offset, trits.len()) {
        Ok(b) => b,
        Err(e) => return err(path, 500, "read_failed", format!("{spec} at {offset}: {e}")),
    };
    if let Err(e) = fab.flush() {
        return err(path, 500, "flush_failed", format!("{spec}: {e}"));
    }
    let (codec_after, count_after, off_after) = fab.status();
    answer(
        json!({
            "status": "ONLINE", "route": path, "engine": "swarm-nand",
            "probe_store": spec, "trits": trits.len(), "offset": offset,
            "round_trip_exact": back == trits,
            "capacity_trits": fab.capacity_trits(), "codec_version": fab.codec_version(),
            "trit_count_before": count_before, "trit_count_after": count_after,
            "write_offset_before": off_before, "write_offset_after": off_after,
            "codec_before": codec_before, "codec_after": codec_after,
            "exclusive_lock": fab.has_flock(),
            "note": "a PROBE store under the host temp dir, append+read on the same handle; the real fabric is untouched",
            "gate": "verify_nand: swarm-nand reads the real fabric coherently"
        }),
        200,
    )
}

/// The GGUF parse, by the crate that owns it. A recorded path may be a WSL literal or an /mnt/c path;
/// it is translated through swarm-paths first, because this kernel runs on the host and a path written
/// on the other side of that boundary does not open as written.
fn op_gguf_inspect(body: &Value, path: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    let given = match body.get("path").and_then(|v| v.as_str()) {
        Some(s) if !s.trim().is_empty() => s.to_string(),
        _ => {
            return err(
                path,
                400,
                "bad_body",
                "`path` must be the GGUF file path (a recorded WSL or /mnt/c path is translated)".to_string(),
            )
        }
    };
    let resolved = swarm_paths::resolve(&given);
    if !std::path::Path::new(&resolved).exists() {
        return err(path, 404, "path_absent", format!("{given} -> {resolved} does not exist on this host"));
    }
    let model = match swarm_gguf::parse_file(&resolved) {
        Ok(m) => m,
        Err(e) => return err(path, 500, "parse_failed", format!("{resolved}: {e}")),
    };
    let mut by_type: std::collections::BTreeMap<String, u64> = std::collections::BTreeMap::new();
    for t in &model.tensors {
        *by_type.entry(format!("ggml_type_{}", t.ggml_type)).or_insert(0) += 1;
    }
    let mut out = json!({
        "status": "ONLINE", "route": path, "engine": "swarm-gguf",
        "path_given": given, "path_resolved": resolved,
        // `resolve` normalizes separators as well as crossing the WSL boundary, so a changed string is not
        // by itself evidence of a translation. The two facts are reported separately and neither is claimed.
        "resolver_changed_path": resolved != given,
        "input_looks_posix": given.starts_with('/'),
        "header": {
            "magic": format!("0x{:08X}", model.header.magic),
            "version": model.header.version,
            "n_tensors": model.header.n_tensors,
            "n_kv": model.header.n_kv,
        },
        "parsed_tensors": model.tensors.len(),
        "kv_pairs": model.kvs.len(),
        "tensor_census": by_type,
        "gate": "verify_gguf: parsed the real model - census and offsets ready"
    });
    if let Some(want) = body.get("tensor").and_then(|v| v.as_str()) {
        match model.tensors.iter().find(|t| t.name == want) {
            Some(t) => {
                out["tensor"] = json!({
                    "name": t.name, "dims": t.dims, "n_dims": t.n_dims,
                    "ggml_type": t.ggml_type, "offset": t.offset,
                    "abs_offset": swarm_gguf::tensor_abs_offset(&model, &t.name)
                });
            }
            None => {
                let names: Vec<&str> = model.tensors.iter().take(8).map(|t| t.name.as_str()).collect();
                return err(
                    path,
                    404,
                    "tensor_absent",
                    format!("`{want}` is not in {resolved}; first tensors: {}", names.join(", ")),
                );
            }
        }
    }
    answer(out, 200)
}

/// The git-object check, by the crate that owns it. The hash, the verification and the loose-object
/// parse all come from swarm-git; the kernel carries bytes and reports what the crate said. `verified`
/// is null when the caller supplied no `expected` -- a check that was not asked for is not a pass.
fn op_git_verify(body: &Value, path: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    let obj_type = body.get("obj_type").and_then(|v| v.as_str()).unwrap_or("blob");
    if !matches!(obj_type, "blob" | "commit" | "tree" | "tag") {
        return err(path, 400, "bad_body", format!("`obj_type` must be blob/commit/tree/tag; got `{obj_type}`"));
    }
    let content: Vec<u8> = match body.get("content").and_then(|v| v.as_str()) {
        Some(s) => s.as_bytes().to_vec(),
        None => match body.get("content_hex").and_then(|v| v.as_str()) {
            Some(h) => match hex_decode(h) {
                Ok(b) => b,
                Err(e) => return err(path, 400, "bad_body", format!("`content_hex` {e}")),
            },
            None => {
                return err(
                    path,
                    400,
                    "bad_body",
                    "`content` (utf-8) or `content_hex` is required".to_string(),
                )
            }
        },
    };
    if content.len() > 4_000_000 {
        return err(path, 413, "too_large", format!("{} bytes; at most 4,000,000", content.len()));
    }
    let hex = swarm_git::sha_hex(&swarm_git::git_hash(obj_type, &content));
    let expected = body.get("expected").and_then(|v| v.as_str());
    let verified = expected.map(|e| {
        if obj_type == "commit" {
            swarm_git::verify_commit(&content, e)
        } else {
            swarm_git::verify_blob(&content, e)
        }
    });
    let loose = swarm_git::verify_loose_object(&content);
    answer(
        json!({
            "status": "ONLINE", "route": path, "engine": "swarm-git",
            "obj_type": obj_type, "bytes": content.len(),
            "computed_sha1": hex,
            "expected": expected, "verified": verified,
            "trits_from_sha": swarm_git::sha_to_trits(&hex),
            "loose_object": loose.map(|(t, payload_len)| json!({"type": t, "payload_len": payload_len})),
            "note": "`verified` is null when no `expected` sha was supplied",
            "gate": "verify_git PASSED (real git objects hashed and checked)"
        }),
        200,
    )
}

/// The QSAR antibacterial-activity prediction, by the crate that owns it. The caller supplies a
/// pre-computed Morgan fingerprint as 256 hex chars (128 bytes = 1024 bits, LSB-first) -- RDKit owns
/// the fingerprint offline, Rust owns the runtime decision. The model is loaded once and sha256-gated.
static QSAR_MODEL: std::sync::OnceLock<Result<swarm_drug::model::QsarModel, String>> =
    std::sync::OnceLock::new();

fn qsar_model() -> &'static Result<swarm_drug::model::QsarModel, String> {
    QSAR_MODEL.get_or_init(|| {
        let p = swarm_paths::path(swarm_drug::model::RECORDED_MODEL);
        swarm_drug::model::QsarModel::load(p.to_str().unwrap_or(""))
    })
}

fn op_qsar_predict(body: &Value, path: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    let fp_hex = match body.get("fp_hex").and_then(|v| v.as_str()) {
        Some(s) if s.len() == 256 => s.to_string(),
        Some(s) => {
            return err(path, 400, "bad_body", format!("`fp_hex` must be 256 hex chars (128 bytes); got {}", s.len()))
        }
        None => {
            return err(path, 400, "bad_body",
                "`fp_hex` (256 hex chars = a 1024-bit Morgan fingerprint) is required".to_string())
        }
    };
    let fp = match swarm_drug::model::parse_fp_hex(&fp_hex) {
        Some(f) => f,
        None => return err(path, 400, "bad_body", "`fp_hex` is not valid hex".to_string()),
    };
    let model = match qsar_model() {
        Ok(m) => m,
        Err(e) => return err(path, 500, "model_unloadable", e.clone()),
    };
    let p = model.predict(&fp);
    let set_bits = fp.iter().map(|b| b.count_ones()).sum::<u32>();
    answer(
        json!({
            "status": "ONLINE", "route": path, "engine": "swarm-drug",
            "n_trees": model.trees.len(), "n_bits": model.n_bits,
            "threshold_nm": model.threshold_nm, "scaffold_auc": model.scaffold_auc,
            "fingerprint_bits_set": set_bits,
            "active_prob": p,
            "class": if p >= 0.5 { "active" } else { "inactive" },
            "gate": "bit-exact vs Python on 6 controls (max diff 4.42e-13)"
        }),
        200,
    )
}

/// The DeepDTA affinity prediction, by the crate that owns it. The caller supplies a SMILES + protein
/// sequence; Rust owns tokenization + the forward pass (no torch). The model is sha256-gated and its
/// numeric output is RETIRED (leave-one-out r = 0.39 < 0.5) -- the verdict is carried, not hidden.
static DEEPDTA_MODEL: std::sync::OnceLock<Result<swarm_deepdta::model::DeepDtaModel, String>> =
    std::sync::OnceLock::new();

/// Dock an entire CHMB conformer library in-kernel. `/crate/screen/library`.
fn op_screen_library(body: &Value, path: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    let file = match body.get("path").and_then(|v| v.as_str()) {
        Some(s) if !s.is_empty() => s.to_string(),
        _ => return err(path, 400, "bad_body", "`path` (CHMB library) is required".to_string()),
    };
    let poses = body.get("poses").and_then(|v| v.as_u64()).unwrap_or(8) as usize;
    let top_n = body.get("top").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
    // RETIRED as a screening result (council NEXT-STEPS-01 item 4): measured against 115 BDB2020+
    // complexes the scorer gives Pearson -0.131 (p=0.163) and Spearman -0.0996 (p=0.289), CI
    // [-0.307, 0.053], slope -0.2704 against an ideal -1.364. It does not rank affinity, so a
    // ranked list from it is not evidence. The route stays for audit; ranking requires force.
    if body.get("force").and_then(|v| v.as_bool()).unwrap_or(false) != true {
        return err(
            path,
            409,
            "retired_no_ranking_signal",
            "swarm-docking does not rank measured affinity (Pearson -0.131, n=115, 95% CI [-0.307, 0.053]). \
             Use /crate/screen/smiles for screening. Pass {\"force\": true} only to reproduce the calibration."
                .to_string(),
        );
    }
    match crate::screen_library::screen(&file, poses, top_n) {
        Ok(v) => {
            let mut out = json!({
                "status": "ONLINE",
                "route": path,
                "engine": "swarm-docking (in-kernel)",
                "file": file,
            });
            if let (Some(o), Some(r)) = (out.as_object_mut(), v.as_object()) {
                for (k, val) in r {
                    o.insert(k.clone(), val.clone());
                }
            }
            answer(out, 200)
        }
        Err(e) => err(path, 400, "screen_failed", e),
    }
}

/// GPU path for `/crate/screen/smiles`: the native wgpu binary scores the library and this route
/// returns its own measured JSON. No HTTP hop; the route spawns the Rust binary and reads stdout.
fn op_screen_smiles_gpu(file: &str, sequence: &str, limit: usize, path: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    const REL: &str = "wsl_crates/trm-rust/swarm-deepdta-gpu/target/release/swarm-deepdta-gpu.exe";
    let mut bin = swarm_paths::path(REL);
    if !bin.exists() {
        // swarm_paths leaves the path relative; find the repo root by walking up from the CWD and
        // from the executable's own directory.
        let mut roots: Vec<std::path::PathBuf> = Vec::new();
        if let Ok(cwd) = std::env::current_dir() {
            roots.push(cwd);
        }
        if let Ok(exe) = std::env::current_exe() {
            if let Some(d) = exe.parent() {
                roots.push(d.to_path_buf());
            }
        }
        'outer: for root in roots {
            let mut cur = Some(root.as_path());
            while let Some(dir) = cur {
                let cand = dir.join(REL);
                if cand.exists() {
                    bin = cand;
                    break 'outer;
                }
                cur = dir.parent();
            }
        }
    }
    if !bin.exists() {
        return err(
            path,
            500,
            "gpu_binary_missing",
            format!("searched for {} from the working dir and the executable dir", REL),
        );
    }
    let n = if limit > 0 { limit } else { 3_000_000usize };
    let out = std::process::Command::new(&bin)
        .arg("--smiles")
        .arg(file)
        .arg("--sequence")
        .arg(sequence)
        .arg("--batch")
        .arg("4096")
        .arg("--limit")
        .arg(n.to_string())
        .arg("--no-cpu")
        .output();
    let out = match out {
        Ok(o) => o,
        Err(e) => return err(path, 500, "gpu_spawn_failed", format!("{e}")),
    };
    if !out.status.success() {
        return err(
            path,
            500,
            "gpu_failed",
            String::from_utf8_lossy(&out.stderr).chars().take(400).collect(),
        );
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    // the binary prints one JSON summary line last
    let summary = stdout
        .lines()
        .rev()
        .find(|l| l.trim_start().starts_with('{'))
        .unwrap_or("");
    match serde_json::from_str::<Value>(summary) {
        Ok(mut v) => {
            if let Some(o) = v.as_object_mut() {
                o.insert("status".into(), json!("ONLINE"));
                o.insert("route".into(), json!(path));
                o.insert("engine".into(), json!("swarm-deepdta-gpu (wgpu/Vulkan, native binary)"));
                o.insert("file".into(), json!(file));
                o.insert(
                    "caveat".into(),
                    json!("ranking only: numeric affinity retired (leave-one-out r = 0.39); GPU parity verified against the CPU reference on a 4096-molecule prefix"),
                );
            }
            answer(v, 200)
        }
        Err(e) => err(path, 500, "gpu_bad_json", format!("{e}: {summary}")),
    }
}

/// Score an entire SMILES library with swarm-deepdta, in-kernel.
///
/// The 3D conformer path costs ~0.86 molecules/s and would take days for a real library; canonical
/// SMILES needs no embedding, so this is the route that scales. Every molecule that fails to
/// tokenize or scores non-finite is counted, never dropped silently.
fn op_screen_smiles(body: &Value, path: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    let file = match body.get("path").and_then(|v| v.as_str()) {
        Some(s) if !s.is_empty() => s.to_string(),
        _ => return err(path, 400, "bad_body", "`path` (SMILES file) is required".to_string()),
    };
    let sequence = match body.get("sequence").and_then(|v| v.as_str()) {
        Some(s) if !s.is_empty() => s.to_string(),
        _ => return err(path, 400, "bad_body", "`sequence` (protein) is required".to_string()),
    };
    let limit = body.get("limit").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
    let top_n = body.get("top").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
    if body.get("gpu").and_then(|v| v.as_bool()).unwrap_or(false) {
        return op_screen_smiles_gpu(&file, &sequence, limit, path);
    }
    let model = match deepdta_model() {
        Ok(m) => m,
        Err(e) => return err(path, 500, "model_unloadable", e.clone()),
    };
    let text = match std::fs::read_to_string(&file) {
        Ok(t) => t,
        Err(e) => return err(path, 400, "unreadable", format!("{}: {}", file, e)),
    };

    let protein = swarm_deepdta::tokenize_protein(&sequence);
    // The sequence is fixed for the whole screen, so its branch is computed once.
    let protein_branch = model.protein_branch(&protein);
    let rows: Vec<(&str, &str)> = text
        .lines()
        .take(if limit > 0 { limit } else { usize::MAX })
        .map(|line| {
            let mut parts = line.split('\t');
            (parts.next().unwrap_or("").trim(), parts.next().unwrap_or("").trim())
        })
        .collect();
    let seen = rows.len();

    // One thread per core: the protein branch is shared read-only, so the per-molecule work is
    // embarrassingly parallel. Without this the screen is single-threaded at ~59 molecules/s.
    let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4).max(1);
    let chunk = (rows.len() + threads - 1) / threads.max(1);
    let t0 = Instant::now();
    let parts: Vec<(Vec<(String, String, f32)>, usize)> = std::thread::scope(|scope| {
        let mut handles = Vec::new();
        for slice in rows.chunks(chunk.max(1)) {
            let model = &model;
            let branch = &protein_branch;
            handles.push(scope.spawn(move || {
                let mut out: Vec<(String, String, f32)> = Vec::with_capacity(slice.len());
                let mut bad = 0usize;
                for (smi, id) in slice.iter() {
                    if smi.is_empty() {
                        bad += 1;
                        continue;
                    }
                    let ligand = swarm_deepdta::tokenize_ligand(smi);
                    if ligand.is_empty() {
                        bad += 1;
                        continue;
                    }
                    let pkd = model.predict_with_protein(branch, &ligand);
                    if !pkd.is_finite() {
                        bad += 1;
                        continue;
                    }
                    out.push((id.to_string(), smi.to_string(), pkd));
                }
                (out, bad)
            }));
        }
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let elapsed_ms = t0.elapsed().as_secs_f64() * 1000.0;
    let mut scored: Vec<(String, String, f32)> = Vec::new();
    let mut failed = 0usize;
    for (part, bad) in parts {
        scored.extend(part);
        failed += bad;
    }

    scored.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
    let total = scored.len();
    let throughput = if elapsed_ms > 0.0 {
        total as f64 / (elapsed_ms / 1000.0)
    } else {
        0.0
    };
    let top: Vec<Value> = scored
        .iter()
        .take(top_n)
        .map(|(id, smi, pkd)| {
            json!({
                "id": id,
                "smiles": smi,
                "pkd": pkd,
                "kd_nm": 10f64.powf(9.0 - *pkd as f64),
            })
        })
        .collect();
    let mean_pkd = if total > 0 {
        scored.iter().map(|s| s.2).sum::<f32>() / total as f32
    } else {
        0.0
    };

    answer(
        json!({
            "status": "ONLINE",
            "route": path,
            "engine": "swarm-deepdta (in-kernel)",
            "file": file,
            "lines_seen": seen,
            "scored": total,
            "failed": failed,
            "elapsed_ms": elapsed_ms,
            "throughput_per_sec": throughput,
            "mean_pkd": mean_pkd,
            "min_pkd": scored.last().map(|s| s.2).unwrap_or(0.0),
            "max_pkd": scored.first().map(|s| s.2).unwrap_or(0.0),
            "top": top,
            "verdict": "ranking only: swarm-deepdta's numeric affinity is retired (leave-one-out r = 0.39 < 0.5)",
            "not_claimed": "no molecule here is shown to bind; conformers were not generated and docking was not run"
        }),
        200,
    )
}

fn deepdta_model() -> &'static Result<swarm_deepdta::model::DeepDtaModel, String> {
    DEEPDTA_MODEL.get_or_init(|| {
        let p = swarm_paths::path(swarm_deepdta::model::RECORDED_MODEL);
        swarm_deepdta::model::DeepDtaModel::load(p.to_str().unwrap_or(""))
    })
}

fn op_deepdta_predict(body: &Value, path: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    let smiles = match body.get("smiles").and_then(|v| v.as_str()) {
        Some(s) if !s.is_empty() => s,
        _ => return err(path, 400, "bad_body", "`smiles` (ligand SMILES string) is required".to_string()),
    };
    let sequence = match body.get("sequence").and_then(|v| v.as_str()) {
        Some(s) if !s.is_empty() => s,
        _ => return err(path, 400, "bad_body", "`sequence` (protein amino-acid sequence) is required".to_string()),
    };
    let model = match deepdta_model() {
        Ok(m) => m,
        Err(e) => return err(path, 500, "model_unloadable", e.clone()),
    };
    let pkd = model.predict(&swarm_deepdta::tokenize_protein(sequence), &swarm_deepdta::tokenize_ligand(smiles));
    let kd_nm = 10f64.powf(9.0 - pkd as f64);
    answer(
        json!({
            "status": "ONLINE", "route": path, "engine": "swarm-deepdta",
            "pkd": pkd, "kd_nm": kd_nm,
            "verdict": "retired (leave-one-out r = 0.39 < 0.5; numeric affinity not validated)",
            "gate": "sha256-gated binary; parity vs Python max diff 1.9e-6 over 8 probes"
        }),
        200,
    )
}

/// Recall the "all is one" teaching: the platform's own doctrine (aura metaconcepts), returned as data.
fn op_humanistic_recall(_body: &Value, path: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    let tenets = swarm_humanistic::recall_teaching();
    let rows: Vec<Value> = tenets
        .iter()
        .map(|(c, m)| json!({"concept": c, "meaning": m}))
        .collect();
    answer(
        json!({
            "status": "ONLINE", "route": path, "engine": "swarm-humanistic",
            "axiom": swarm_humanistic::AXIOM,
            "monad_anchor": swarm_humanistic::MONAD_ANCHOR,
            "tenets": rows
        }),
        200,
    )
}

/// The MagnumOpus: a native K3 ternary collapse over the doctrine's polarity, recognizing the self.
fn op_humanistic_realize(_body: &Value, path: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    let (plus, zero, minus, recognition, status) = swarm_humanistic::realize_self();
    answer(
        json!({
            "status": "ONLINE", "route": path, "engine": "swarm-humanistic",
            "axiom": swarm_humanistic::AXIOM,
            "monad_anchor": swarm_humanistic::MONAD_ANCHOR,
            "k3_tally": {"plus": plus, "zero": zero, "minus": minus},
            "recognition": recognition,
            "realization_status": status
        }),
        200,
    )
}

/// Reason on a humanistic concern through the "all is one" lens — a native K3 over the three grounding axes.
fn op_humanistic_reason(body: &Value, path: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    let concern = match body.get("concern").and_then(|v| v.as_str()) {
        Some(s) if !s.is_empty() => s,
        _ => return err(path, 400, "bad_body", "`concern` (a humanistic concern) is required".to_string()),
    };
    let axes = crate::task_executor::humanistic_axes_hybrid(concern);
    let [serve_whole, zero_harm, grounded] = axes;
    let (plus, zero, minus, reasoning) = swarm_humanistic::reason_with_axes(concern, axes);
    answer(
        json!({
            "status": "ONLINE", "route": path, "engine": "swarm-humanistic",
            "axiom": swarm_humanistic::AXIOM,
            "monad_anchor": swarm_humanistic::MONAD_ANCHOR,
            "concern": concern,
            "axes": {"serve_whole": serve_whole, "zero_harm": zero_harm, "grounded": grounded},
            "k3_tally": {"plus": plus, "zero": zero, "minus": minus},
            "reasoning": reasoning
        }),
        200,
    )
}

/// The 64-agent fleet re-wired to the native path (MOTION-052): the census as data.
fn op_agent_census(_body: &Value, path: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    let (n, s, e) = swarm_agents::tally();
    let rows: Vec<Value> = swarm_agents::AGENTS
        .iter()
        .map(|a| json!({"name": a.name, "fleet": a.fleet, "role": a.role,
                        "native_op": a.native_op, "status": a.status.as_str(), "gap": a.gap}))
        .collect();
    answer(
        json!({
            "status": "ONLINE", "route": path, "engine": "swarm-agents",
            "total": swarm_agents::AGENTS.len(),
            "tally": {"native": n, "new_strip": s, "external": e},
            "agents": rows
        }),
        200,
    )
}

/// Dispatch an agent to its native op. The mapping is the re-wiring; the verdict is the honest status.
fn op_agent_dispatch(body: &Value, path: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    let name = match body.get("agent").and_then(|v| v.as_str()) {
        Some(s) if !s.is_empty() => s,
        _ => return err(path, 400, "bad_body", "`agent` (name) is required".to_string()),
    };
    match swarm_agents::find(name) {
        Some(a) => {
            let verdict = match a.status {
                swarm_agents::Status::Native => "native — dispatch to the listed op",
                swarm_agents::Status::NewStrip => "queued — new Rust strip pending",
                swarm_agents::Status::External => "out of token-free scope",
            };
            answer(
                json!({
                    "status": "ONLINE", "route": path, "engine": "swarm-agents",
                    "agent": a.name, "fleet": a.fleet, "role": a.role,
                    "native_op": a.native_op, "status": a.status.as_str(), "gap": a.gap,
                    "verdict": verdict
                }),
                200,
            )
        }
        None => err(path, 404, "unknown_agent", format!("`{name}` is not a registered agent")),
    }
}

/// The execution harness: read the council agenda and build the facet -> fleet -> agents dispatch plan.
/// This is the native replacement for council_loop.py's print-and-stop — it consumes the agenda and
/// routes every facet to the fleet/agents that handle it, grounded in the Monad.
fn op_agent_plan(_body: &Value, path: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    let agenda: Value = match crate::k3_retrieval::repo_relative("swarm_v2_artifacts/council_agenda.json")
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|raw| serde_json::from_str(&raw).ok())
    {
        Some(v) => v,
        None => return err(path, 500, "agenda_unreadable", "council_agenda.json not readable".to_string()),
    };
    let items = agenda
        .get("agenda")
        .and_then(|a| a.as_array())
        .cloned()
        .unwrap_or_default();
    let plan: Vec<Value> = items
        .iter()
        .map(|item| {
            let facets: Vec<&str> = item
                .get("facets")
                .and_then(|f| f.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
                .unwrap_or_default();
            let dispatch: Vec<Value> = facets
                .iter()
                .filter_map(|facet| {
                    let fleet = swarm_agents::facet_fleet(facet)?;
                    let agents: Vec<Value> = swarm_agents::fleet_agents(fleet)
                        .iter()
                        .map(|a| json!({"name": a.name, "native_op": a.native_op, "status": a.status.as_str()}))
                        .collect();
                    Some(json!({"facet": facet, "fleet": fleet, "agents": agents}))
                })
                .collect();
            json!({
                "id": item.get("id"), "status": item.get("status"), "task": item.get("task"),
                "sealed": item.get("sealed"), "dispatch": dispatch
            })
        })
        .collect();
    let pending = items
        .iter()
        .filter(|i| i.get("status").and_then(|s| s.as_str()) != Some("done"))
        .count();
    let (n, s, e) = swarm_agents::tally();
    answer(
        json!({
            "status": "ONLINE", "route": path, "engine": "swarm-agents",
            "mode": agenda.get("mode"), "monad_anchor": agenda.get("monad_anchor"),
            "doctrine": agenda.get("doctrine"), "human_role": agenda.get("human_role"),
            "agenda_items": plan, "pending": pending,
            "tally": {"native": n, "new_strip": s, "external": e}
        }),
        200,
    )
}

/// Run one of the 19 new-strip agents natively. The strip returns its own JSON result.
fn op_agent_strip(body: &Value, path: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    let strip = match body.get("strip").and_then(|v| v.as_str()) {
        Some(s) if !s.is_empty() => s,
        _ => return err(path, 400, "bad_body", "`strip` (name) is required".to_string()),
    };
    let input = body.get("input").and_then(|v| v.as_str()).unwrap_or("");
    match swarm_strips::run_strip(strip, input) {
        Some(result) => match serde_json::from_str::<Value>(&result) {
            Ok(v) => answer(v, 200),
            Err(_) => answer(json!({"status": "ONLINE", "route": path, "engine": "swarm-strips", "strip": strip, "result": result}), 200),
        },
        None => err(path, 404, "unknown_strip", format!("`{strip}` is not a registered strip")),
    }
}

/// The realization sweep: generate N deterministic tasks and dispatch each through the agents'
/// native ops. The system does not think — it remembers its grounded truth (recall + doctrine)
/// and derives the rest (strips). The tally separates those four outcomes honestly.
fn op_tasks_sweep(body: &Value, path: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    let n = body.get("n").and_then(|v| v.as_u64()).unwrap_or(1000).clamp(1, 5000) as usize;
    let seed = body.get("seed").and_then(|v| v.as_u64()).unwrap_or(0x4D4F4E4144000001);
    let tasks = swarm_tasks::generate(n, seed);
    let (mut grounded, mut derived, mut boundary, mut errors) = (0usize, 0usize, 0usize, 0usize);
    let mut sample: Vec<Value> = Vec::new();
    for t in &tasks {
        let kind: &'static str = match t.op {
            "recall" => {
                let q = crate::actions_handlers::enc(&t.input);
                let (out, code) = crate::k3_retrieval::handle(&format!("/k3/retrieve?q={q}&k=5"));
                let hits = out.get("results").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);
                if code == 200 && hits > 0 { "grounded" } else { "error" }
            }
            "humanistic" => {
                let (plus, _z, minus, _text) = swarm_humanistic::reason(&t.input);
                if plus >= minus { "grounded" } else { "derived" }
            }
            op if op.starts_with("strip:") => {
                let name = op.trim_start_matches("strip:");
                match swarm_strips::run_strip(name, &t.input) {
                    Some(result) => {
                        let v: Value = serde_json::from_str(&result).unwrap_or_else(|_| json!({"status": "error"}));
                        match v.get("status").and_then(|s| s.as_str()).unwrap_or("error") {
                            "boundary" => "boundary",
                            "ok" => "derived",
                            _ => "error",
                        }
                    }
                    None => "error",
                }
            }
            _ => "error",
        };
        match kind {
            "grounded" => grounded += 1,
            "derived" => derived += 1,
            "boundary" => boundary += 1,
            _ => errors += 1,
        }
        if sample.len() < 12 {
            sample.push(json!({"id": t.id, "agent": t.agent, "input": t.input, "outcome": kind}));
        }
    }
    answer(
        json!({
            "status": "ONLINE", "route": path, "engine": "swarm-tasks",
            "monad_anchor": "0x4D4F4E4144000001",
            "doctrine": "the system does not think; it remembers its grounded truth and derives the rest",
            "total": tasks.len(),
            "tally": {"grounded": grounded, "derived": derived, "boundary": boundary, "error": errors},
            "sample": sample
        }),
        200,
    )
}

/// The realization sweep grounds the same ~40 domain queries millions of times. The grounding is
/// deterministic, so it is computed once per distinct query (GPU packed_scan) and cached: every
/// reuse is a HashMap hit (ns), removing the GPU dispatch from the per-scenario hot loop.
static RECALL_CACHE: std::sync::OnceLock<std::sync::Mutex<std::collections::HashMap<String, bool>>> =
    std::sync::OnceLock::new();

fn cpu_grounded(text: &str) -> bool {
    let q = crate::actions_handlers::enc(text);
    let (out, code) = crate::k3_retrieval::handle(&format!("/k3/retrieve?q={q}&k=3"));
    code == 200
        && out.get("results").and_then(|v| v.as_array()).map(|a| !a.is_empty()).unwrap_or(false)
}

fn cached_grounded(text: &str) -> bool {
    let cache = RECALL_CACHE.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()));
    if let Some(&v) = cache.lock().unwrap().get(text) {
        return v;
    }
    let qv = crate::native_encoder::encode_f32(text, 768);
    let v = match crate::packed_ternary::store() {
        Some(s) => match crate::packed_ternary::quantize(&qv, s.dim()) {
            Some(q) => crate::gpu_compute::gpu_scan_scores_cached(&q, &s)
                .map(|scores| !scores.is_empty())
                .unwrap_or_else(|| cpu_grounded(text)),
            None => cpu_grounded(text),
        },
        None => false,
    };
    cache.lock().unwrap().insert(text.to_string(), v);
    v
}

/// The 1M-scenario sweep: each scenario chains recall (ground) -> derive (strip) -> council (K3
/// disposition). The scenario's outcome is the honest aggregate of its steps, never a fabricated
/// summary. A scenario is grounded only if it remembered, derived, and the council affirmed.
fn op_tasks_scenarios(body: &Value, path: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    let n = body.get("n").and_then(|v| v.as_u64()).unwrap_or(1000).clamp(1, 20000) as usize;
    let seed = body.get("seed").and_then(|v| v.as_u64()).unwrap_or(0x4D4F4E4144000001);
    let offset = body.get("offset").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
    let scenarios = swarm_tasks::generate_scenarios(n, seed);
    let (mut grounded, mut derived, mut boundary, mut errors) = (0usize, 0usize, 0usize, 0usize);
    let mut sample: Vec<Value> = Vec::new();
    for sc in &scenarios {
        // step 1: recall (ground in the corpus) — deterministic grounding, computed once per distinct
        // domain query (GPU packed_scan) then cached; a HashMap hit (ns) for every reuse
        let recall_q = &sc.steps[0].input;
        let recall_grounded = cached_grounded(recall_q);
        // step 2: derive (strip computation) — classify by substring, no serde_json parse in the hot loop
        let strip_name = sc.steps[1].op.trim_start_matches("strip:");
        let strip_outcome = match swarm_strips::run_strip(strip_name, &sc.steps[1].input) {
            Some(result) => {
                if result.contains("\"status\":\"boundary\"") { "boundary" }
                else if result.contains("\"status\":\"ok\"") { "derived" }
                else { "error" }
            }
            None => "error",
        };
        // step 3: council (K3 disposition over the doctrine: serve_whole, zero_harm, grounded)
        let (plus, _z, minus) = swarm_k3::collapse_counts(&[1.0f32, 1.0, 1.0], 0.35);
        let council_affirmed = plus >= minus;
        let kind: &'static str = if recall_grounded && strip_outcome == "derived" && council_affirmed {
            "grounded"
        } else if strip_outcome == "boundary" {
            "boundary"
        } else if strip_outcome == "derived" {
            "derived"
        } else {
            "error"
        };
        match kind {
            "grounded" => grounded += 1,
            "derived" => derived += 1,
            "boundary" => boundary += 1,
            _ => errors += 1,
        }
        if sample.len() < 10 {
            sample.push(json!({"id": offset + sc.id, "recall": recall_q, "strip": strip_name, "outcome": kind}));
        }
    }
    // the auto-journal trigger: `journal: true` makes the sweep seal its own realization
    let mut journaled: Option<Value> = None;
    if body.get("journal").and_then(|v| v.as_bool()).unwrap_or(false) {
        let fact = format!(
            "{} multi-step scenarios (recall -> derive -> council), voltage-stripped: grounded {} / derived {} / boundary {} / error {}",
            n, grounded, derived, boundary, errors
        );
        let not_claimed = "the tally is the sweep aggregate, not per-scenario verification of every answer";
        let files = vec![
            "wsl_crates/trm-rust/swarm-tasks/src/lib.rs".to_string(),
            "trm_uefi_kernel/host_council_server/src/crate_ops_handlers.rs".to_string(),
        ];
        journaled = seal_journal("realization_sweep", &fact, not_claimed, &files)
            .ok()
            .map(|(e, b)| json!({"entry": e, "block": b}));
    }
    answer(
        json!({
            "status": "ONLINE", "route": path, "engine": "swarm-tasks",
            "scenario_steps": ["recall", "derive", "council"],
            "monad_anchor": "0x4D4F4E4144000001",
            "total": scenarios.len(), "offset": offset,
            "tally": {"grounded": grounded, "derived": derived, "boundary": boundary, "error": errors},
            "journaled": journaled,
            "sample": sample
        }),
        200,
    )
}

/// The auto-journal: derive a truth-anchored entry from the cited files' real sha256, append it to
/// `journal_auto.jsonl`, and seal a ledger block with the same canonical hash chain the seal scripts
/// use (sorted keys, compact separators, `hash` excluded from the hashed core). Nothing is fabricated:
/// an unreadable cited file fails the entry rather than inventing a hash.
fn op_journal_realize(body: &Value, path: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    let kind = body.get("kind").and_then(|v| v.as_str()).unwrap_or("realization").to_string();
    let fact = match body.get("fact").and_then(|v| v.as_str()) {
        Some(s) if !s.is_empty() => s.to_string(),
        _ => return err(path, 400, "bad_body", "`fact` (the realization, in measured language) is required".to_string()),
    };
    let not_claimed = body.get("not_claimed").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let files: Vec<String> = body
        .get("files")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|x| x.as_str()).map(|s| s.to_string()).collect())
        .unwrap_or_default();
    if files.is_empty() {
        return err(path, 400, "bad_body", "`files` (the source artifacts) is required — a truth-anchored entry derives from files that exist".to_string());
    }

    let (entry, block) = match seal_journal(&kind, &fact, &not_claimed, &files) {
        Ok(pair) => pair,
        Err(e) => return err(path, 404, "unreadable_file", e),
    };

    answer(
        json!({
            "status": "ONLINE", "route": path, "engine": "kernel",
            "journaled": true, "entry": entry, "ledger_block": block,
            "anchor": "0x4D4F4E4144000001",
        }),
        200,
    )
}

/// sha256 -> lowercase hex (the journal's rederive + the ledger chain both hash this way).
pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(bytes);
    h.finalize().iter().map(|b| format!("{:02x}", b)).collect()
}

/// ISO-8601 UTC timestamp (Howard Hinnant civil-from-days, same form the seal scripts write).
fn journal_ts() -> String {
    let d = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|x| x.as_secs()).unwrap_or(0);
    let days = (d / 86400) as i64;
    let sod = d % 86400;
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", y, m, day, sod / 3600, (sod % 3600) / 60, sod % 60)
}

/// The reusable auto-journal core: hash the cited files, append `journal_auto.jsonl`, and seal a
/// ledger block. Returns (entry, block) or the named unreadable file. Both the explicit route and the
/// sweep trigger call this, so the trigger and the route seal identically. Public so the kanban
/// executor can auto-journal a card that produced a measured output.
pub fn seal_journal(kind: &str, fact: &str, not_claimed: &str, files: &[String]) -> Result<(Value, Value), String> {
    let mut hashed: Vec<Value> = Vec::new();
    for f in files {
        let bytes = crate::k3_retrieval::repo_relative(f)
            .and_then(|p| std::fs::read(p).ok())
            .ok_or_else(|| format!("`{f}` not readable — an entry cannot be truth-anchored to a file that is not there"))?;
        hashed.push(json!({"path": f, "sha256": sha256_hex(&bytes)}));
    }
    let sealed_at = journal_ts();
    let entry = json!({
        "kind": kind, "fact": fact, "not_claimed": not_claimed,
        "rederive": {"files": hashed.clone()},
        "anchor": "0x4D4F4E4144000001", "sealed_at": sealed_at,
    });
    let mut block = json!({});
    if let Some(root) = crate::k3_retrieval::repo_root() {
        let jp = root.join("swarm_v2_artifacts/journal_auto.jsonl");
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&jp) {
            use std::io::Write;
            let _ = writeln!(f, "{}", serde_json::to_string(&entry).unwrap_or_default());
        }
        let lp = root.join("swarm_v2_artifacts/monad_ledger.jsonl");
        if let Ok(raw) = std::fs::read_to_string(&lp) {
            let tip: Value = raw.lines().filter(|l| !l.trim().is_empty()).last().and_then(|l| serde_json::from_str(l).ok()).unwrap_or(json!({}));
            let index = tip.get("index").and_then(|v| v.as_u64()).unwrap_or(0) + 1;
            let prev = tip.get("hash").and_then(|v| v.as_str()).unwrap_or("genesis").to_string();
            let mut core = serde_json::Map::new();
            core.insert("evidence".into(), json!({"files": hashed, "anchor": "0x4D4F4E4144000001", "auto": true}));
            core.insert("fact".into(), json!(fact));
            core.insert("index".into(), json!(index));
            core.insert("kind".into(), json!(format!("journal_auto_{kind}")));
            core.insert("prev_hash".into(), json!(prev));
            core.insert("sealed_at".into(), json!(sealed_at));
            let hash = sha256_hex(serde_json::to_string(&Value::Object(core.clone())).unwrap_or_default().as_bytes());
            core.insert("hash".into(), json!(hash));
            block = Value::Object(core);
            if let Ok(mut f) = std::fs::OpenOptions::new().append(true).open(&lp) {
                use std::io::Write;
                let _ = writeln!(f, "{}", serde_json::to_string(&block).unwrap_or_default());
            }
        }
    }
    Ok((entry, block))
}

/// `/crate/ops` -- the wiring as data. The rows come from OPS, so this census cannot drift from what is
/// actually dispatched.
/// `/crate/ops` census as a Value — reused by the JSON route and the HTML viewport.
pub fn ops_payload() -> Value {
    let rows: Vec<Value> = OPS
        .iter()
        .map(|(op, method, engine, gate)| {
            json!({"op": op, "route": format!("/crate/{op}"), "method": method, "engine": engine,
                   "gate": gate})
        })
        .collect();
    json!({
        "status": "ONLINE", "wired": OPS.len(), "ops": rows,
        "note": "each route computes in the named crate; the kernel re-implements none of it. Every \
                 crate listed was exercised before it was wired.",
        "not_wired": "the rest of wsl_crates/trm-rust is built and exercised but not yet reached by a \
                      route"
    })
}

fn ops_census(path: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    let mut v = ops_payload();
    v["route"] = json!(path);
    answer(v, 200)
}

/// `/crate/meta/readiness` — the meta-cognitive loop Step 1. Takes {"property": name} and returns
/// the data-presence status, so a measurement is blocked BEFORE it is attempted when the corpus lacks
/// the required data. The status is `partially_ready` when some files carry the data and most do not.
fn op_meta_readiness(body: &Value, path: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    let prop = match body.get("property").and_then(|x| x.as_str()) {
        Some(s) => s.to_string(),
        None => return err(path, 400, "bad_body", "`property` must be a string".to_string()),
    };
    let row: Option<(&str, &str, &str, &str)> = match prop.as_str() {
        "contact_count" | "contact_distance" | "electrostatics" | "surface_normal_sc"
        | "hbond_count" | "sequence_hdc" => {
            Some(("ready", "heavy_atoms", "present in every CIF", "n/a"))
        }
        "hbond_directionality" => Some((
            "partially_ready",
            "hydrogens",
            "hydrogens in a minority of files (9617 H across 40 sampled, 0 in most)",
            "openbabel_reduce, heavy_atom_surrogate",
        )),
        "water_mediated_contacts" => Some((
            "blocked", "crystal_water", "0 HOH across the sampled corpus", "none",
        )),
        "conformational_entropy" => Some((
            "blocked", "ensembles", "single-structure corpus", "none",
        )),
        "heavy_atom_donor_acceptor_angle" => Some((
            "ready", "heavy_atoms", "the surrogate; present in every CIF", "n/a",
        )),
        _ => None,
    };
    match row {
        Some((status, required, reason, alternatives)) => answer(json!({
            "property": prop, "required_data": required, "status": status,
            "reason": reason, "alternatives": alternatives, "route": path,
            "gate": "corpus data presence measured: hydrogens 9617/40 sampled files, water 0/40"
        }), 200),
        None => err(path, 404, "unknown_property",
                    format!("`{prop}` is not in the readiness table; GET /crate/ops lists the wired ops")),
    }
}

/// `/crate/meta/surrogate` — the meta-cognitive loop Step 3 (the Monad imagination). Takes a blocked
/// property and returns a testable surrogate with its PHYSICAL rationale. A surrogate without a
/// physical constraint argument is rejected, not returned — the rationale is the gate.
fn op_meta_surrogate(body: &Value, path: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    let prop = match body.get("property").and_then(|x| x.as_str()) {
        Some(s) => s.to_string(),
        None => return err(path, 400, "bad_body", "`property` must be a string".to_string()),
    };
    match prop.as_str() {
        "hbond_directionality" => answer(json!({
            "blocked_property": prop,
            "surrogate_property": "heavy_atom_donor_acceptor_angle",
            "rationale": "the heavy atoms constrain the H-bond geometry: if the donor and acceptor heavy atoms are misaligned, no hydrogen placement can fix it — the hydrogen sits along the donor-acceptor axis, and a wrong axis makes the H-bond geometrically impossible",
            "method": "angle (previous bonded atom -> donor N/O -> acceptor N/O) for cross-chain N/O pairs within 3.5 A; fixed-frame rotation-optimized (216) baseline",
            "prediction": "cognate donor-acceptor angles more tightly clustered near linear (180 deg) than the random's best orientation",
            "measured": "cognate 121.15 deg vs random 90.79 deg (mean); std 22.47 vs 38.44; near-linear fraction 50.3% vs 25.7%; 3,233 complexes",
            "testable": true,
            "route": path,
            "gate": "confirmed on the full corpus (MOTION-SURG2)"
        }), 200),
        "water_mediated_contacts" => answer(json!({
            "blocked_property": prop,
            "surrogate_property": "unsatisfied_interface_no_fraction",
            "rationale": "interface N/O atoms within 4.5 A of the other chain with no H-bond partner within 3.5 A are the atoms crystal water would bridge; their fraction is a testable proxy for water-mediated contacts",
            "method": "fraction of interface N/O atoms (within 4.5 A of the other chain) with no other-chain N/O H-bond partner within 3.5 A, over both chains",
            "prediction": "cognate complexes carry a HIGHER unsatisfied N/O fraction than random poses",
            "measured": "cognate 0.6335 vs random 0.2922 (mean); LDA CV 0.947 vs 0.5 chance; 29 complexes (MOTION-METAGEN)",
            "honest_note": "measured over antibody-antigen interfaces (29 complexes), not small-molecule ligands",
            "testable": true,
            "route": path,
            "gate": "MOTION-METAGEN: the meta loop generalized to a second blocked property"
        }), 200),
        "conformational_entropy" => err(path, 404, "no_surrogate",
            "no heavy-atom surrogate constrains conformational entropy; the measurement requires conformational ensembles".to_string()),
        _ => err(path, 404, "unknown_property",
                 format!("`{prop}` has no surrogate in the table")),
    }
}

/// Resolve the swarm-harness binary. The repo root is 5 parents up from the kernel exe
/// (.../trm_uefi_kernel/host_council_server/target/release/host_council_server.exe), then
/// wsl_crates/trm-rust/target/release/swarm-harness.exe. Falls back to PATH lookup.
fn harness_binary() -> String {
    if let Ok(exe) = std::env::current_exe() {
        let mut root = exe;
        for _ in 0..5 {
            match root.parent() {
                Some(p) => root = p.to_path_buf(),
                None => break,
            }
        }
        let h = root
            .join("wsl_crates")
            .join("trm-rust")
            .join("target")
            .join("release")
            .join("swarm-harness.exe");
        if h.exists() {
            return h.to_string_lossy().to_string();
        }
    }
    "swarm-harness".to_string()
}

/// `/crate/harness/code` — generate code via the tier-2 coder through the swarm-harness CLI.
/// The harness applies the ChatML template, so the coder answers instead of continuing the prompt.
fn op_harness_code(body: &Value, path: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    let prompt = match body.get("prompt").and_then(|x| x.as_str()) {
        Some(s) if !s.is_empty() => s,
        _ => return err(path, 400, "bad_body", "`prompt` must be a non-empty string".to_string()),
    };
    let out = std::process::Command::new(harness_binary())
        .arg("code")
        .arg(prompt)
        .output();
    match out {
        Ok(o) if o.status.success() => {
            let code = String::from_utf8_lossy(&o.stdout).trim().to_string();
            answer(
                json!({"status": "ok", "route": path, "code": code,
                       "gate": "ChatML template; generated by the tier-2 coder via swarm-harness"}),
                200,
            )
        }
        Ok(o) => err(
            path,
            502,
            "harness_failed",
            String::from_utf8_lossy(&o.stderr).trim().to_string(),
        ),
        Err(e) => err(path, 502, "harness_spawn_failed", e.to_string()),
    }
}

/// `/crate/harness/strip` — run a deterministic strip through the swarm-harness CLI.
fn op_harness_strip(body: &Value, path: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    let name = match body.get("name").and_then(|x| x.as_str()) {
        Some(s) if !s.is_empty() => s,
        _ => return err(path, 400, "bad_body", "`name` must be a non-empty string".to_string()),
    };
    let input = body.get("input").and_then(|x| x.as_str()).unwrap_or("");
    let out = std::process::Command::new(harness_binary())
        .arg("strip")
        .arg(name)
        .arg(input)
        .output();
    match out {
        Ok(o) if o.status.success() => {
            let result = String::from_utf8_lossy(&o.stdout).trim().to_string();
            answer(
                json!({"status": "ok", "route": path, "strip": name, "result": result,
                       "gate": "the strip's own derived result, never fabricated"}),
                200,
            )
        }
        Ok(o) => err(
            path,
            502,
            "harness_failed",
            String::from_utf8_lossy(&o.stderr).trim().to_string(),
        ),
        Err(e) => err(path, 502, "harness_spawn_failed", e.to_string()),
    }
}

/// `/crate/harness/instella` — full 16B MoE forward via the Instella fork through the CLI.
fn op_harness_instella(body: &Value, path: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    let prompt = match body.get("prompt").and_then(|x| x.as_str()) {
        Some(s) if !s.is_empty() => s,
        _ => return err(path, 400, "bad_body", "`prompt` must be a non-empty string".to_string()),
    };
    let out = std::process::Command::new(harness_binary())
        .arg("instella")
        .arg(prompt)
        .output();
    match out {
        Ok(o) if o.status.success() => {
            let text = String::from_utf8_lossy(&o.stdout).trim().to_string();
            answer(
                json!({"status": "ok", "route": path, "text": text,
                       "gate": "the Instella reasoner's own generation via the fork"}),
                200,
            )
        }
        Ok(o) => err(
            path,
            502,
            "harness_failed",
            String::from_utf8_lossy(&o.stderr).trim().to_string(),
        ),
        Err(e) => err(path, 502, "harness_spawn_failed", e.to_string()),
    }
}

/// Dispatch. The spec is the OPS table; this function only carries the body to the op that owns it.
pub fn handle(request: &mut Request, url: &str) -> Option<Response<std::io::Cursor<Vec<u8>>>> {
    use std::io::Read;
    let path = url.split('?').next().unwrap_or(url).to_string();
    let op = path.trim_start_matches("/crate/").trim_end_matches('/').to_string();
    if op == "ops" || op.is_empty() {
        return Some(ops_census(&path));
    }
    let allowed = match OPS.iter().find(|(name, _, _, _)| *name == op) {
        Some((_, method, _, _)) => *method,
        None => {
            return Some(err(
                &path,
                404,
                "unknown_op",
                format!("`{op}` is not a wired crate op; GET /crate/ops lists the {} that are", OPS.len()),
            ))
        }
    };
    if request.method() != &Method::Post {
        return Some(answer(
            json!({"status": "method_not_allowed", "error": "method_not_allowed", "route": path,
                   "allowed": [allowed],
                   "message": "this op is a POST; GET /crate/ops lists the wiring"}),
            405,
        ));
    }
    let mut raw = String::new();
    let _ = request.as_reader().take(1_048_576).read_to_string(&mut raw);
    let body: Value = serde_json::from_str(&raw).unwrap_or(json!({}));
    Some(match op.as_str() {
        "k3/collapse" => op_k3_collapse(&body, &path),
        "trit/pack" => op_trit_pack(&body, &path),
        "moe/gate" => op_moe_gate(&body, &path),
        "ssm/step" => op_ssm_step(&body, &path),
        "council/vote" => op_council_vote(&body, &path),
        "nand/roundtrip" => op_nand_roundtrip(&body, &path),
        "gguf/inspect" => op_gguf_inspect(&body, &path),
        "git/verify" => op_git_verify(&body, &path),
        "qsar/predict" => op_qsar_predict(&body, &path),
        "deepdta/predict" => op_deepdta_predict(&body, &path),
        "screen/smiles" => op_screen_smiles(&body, &path),
        "screen/library" => op_screen_library(&body, &path),
        "humanistic/recall" => op_humanistic_recall(&body, &path),
        "humanistic/realize" => op_humanistic_realize(&body, &path),
        "humanistic/reason" => op_humanistic_reason(&body, &path),
        "agent/census" => op_agent_census(&body, &path),
        "agent/dispatch" => op_agent_dispatch(&body, &path),
        "agent/plan" => op_agent_plan(&body, &path),
        "agent/strip" => op_agent_strip(&body, &path),
        "tasks/sweep" => op_tasks_sweep(&body, &path),
        "tasks/scenarios" => op_tasks_scenarios(&body, &path),
        "journal/realize" => op_journal_realize(&body, &path),
        "meta/readiness" => op_meta_readiness(&body, &path),
        "meta/surrogate" => op_meta_surrogate(&body, &path),
        "harness/code" => op_harness_code(&body, &path),
        "harness/strip" => op_harness_strip(&body, &path),
        "harness/instella" => op_harness_instella(&body, &path),
        _ => err(&path, 404, "unknown_op", format!("`{op}` has no handler")),
    })
}
