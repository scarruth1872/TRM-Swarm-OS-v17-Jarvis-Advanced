// Rollout 8: /system/modules — per-module reads, one real source each.
//
// The kernel answered EVERY /system/modules/<name> sub-path with the /system OVERVIEW, so each module
// panel in ModuleMatrix rendered the same global numbers and nothing ever changed per module. This
// sibling dispatches per module id: each read comes from that module's own source, derived where a
// derivation exists, and a module with no persisted state says so instead of borrowing the overview.
// Actions that need the module's own engine stay honest 501s; k3-kleene is deterministic and served.

use serde_json::{json, Value};
use std::io::Read;
use tiny_http::{Header, Method, Request, Response};

use crate::host_probe;
use crate::k3_retrieval;
use crate::mailbox_ring;
use crate::packed_ternary;

fn ok(value: Value) -> Response<std::io::Cursor<Vec<u8>>> {
    Response::from_string(value.to_string())
        .with_status_code(200)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap())
        .with_header(Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap())
}

fn engine_only(message: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    Response::from_string(json!({ "status": "engine_required", "message": message }).to_string())
        .with_status_code(501)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap())
        .with_header(Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap())
}

fn root() -> std::path::PathBuf {
    crate::kanban_handlers::repo_root()
}

fn read_json(rel: &str) -> Option<Value> {
    let p = root().join(rel);
    let raw = std::fs::read_to_string(&p).ok()?;
    serde_json::from_str::<Value>(&raw).ok()
}

fn count_files(rel: &str, ext: &str) -> u64 {
    let mut n = 0u64;
    if let Ok(rd) = std::fs::read_dir(root().join(rel)) {
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if name.ends_with(ext) && !name.contains("__pycache__") {
                n += 1;
            }
        }
    }
    n
}

/// The NAMES behind `count_files`. The counts were already derived from the tree while the name lists beside
/// them were hardcoded empty — a census answering nothing about a non-empty set. Same walk, same filter, so
/// the list and the count cannot disagree; `limit` caps the payload and is named in the basis, never silent.
fn list_files(rel: &str, ext: &str, limit: usize) -> (Vec<String>, u64) {
    let mut names: Vec<String> = Vec::new();
    let mut total = 0u64;
    if let Ok(rd) = std::fs::read_dir(root().join(rel)) {
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if name.ends_with(ext) && !name.contains("__pycache__") {
                total += 1;
                names.push(name);
            }
        }
    }
    names.sort();
    names.truncate(limit);
    (names, total)
}

/// The module inventory, counted from the tree rather than asserted: active core (.py under
/// swarm_v2/core), native Rust engines (.rs in the kernel trees), cognitive skills, MCP bridges.
fn inventory_payload() -> Value {
    let core_py = count_files("swarm_v2/core", ".py");
    let kernel_rs = count_files("trm_uefi_kernel/src", ".rs")
        + count_files("swarm_v2_rust_kernel/src", ".rs")
        + count_files("trm_uefi_kernel/host_council_server/src", ".rs");
    let skills = count_files("swarm_v2/skills", ".py") + count_files("skills", ".md");
    let mcp = count_files("swarm_v2/mcp", ".py");

    let (core_names, _) = list_files("swarm_v2/core", ".py", 150);
    let (mut rust_names, _) = list_files("trm_uefi_kernel/src", ".rs", 150);
    rust_names.extend(list_files("swarm_v2_rust_kernel/src", ".rs", 80).0);
    rust_names.extend(list_files("trm_uefi_kernel/host_council_server/src", ".rs", 80).0);
    let (mut skill_names, _) = list_files("swarm_v2/skills", ".py", 40);
    skill_names.extend(list_files("skills", ".md", 40).0);
    let (mcp_names, _) = list_files("swarm_v2/mcp", ".py", 40);

    json!({
        "summary": {
            "active_core": core_py,
            "native_rust": kernel_rs,
            "cognitive_skills": skills,
            "mcp_protocols": mcp,
            "dormant_core": 0,
            "total_modules": core_py + kernel_rs + skills + mcp,
        },
        "active_core": core_names,
        "native_rust_engines": rust_names,
        "cognitive_skills": skill_names,
        "mcp_protocols": mcp_names,
        "dormant_core_sample": [],
        "monad_anchor": "0x4D4F4E4144000001",
        "basis": "counted AND enumerated from the tree (swarm_v2/core/*.py, the three Rust kernel src \
                  trees, swarm_v2/skills + skills, swarm_v2/mcp) by the same walk, so list and count \
                  cannot disagree; each list is capped (150 core, 150+80+80 rust, 40+40 skills, 40 mcp) \
                  while `summary` always carries the true totals. `dormant_core_sample` stays empty \
                  because no import graph is computed here, and that is stated rather than filled in.",
    })
}

fn mailbox_bus_payload() -> Value {
    // The ring's own bytes, via the reader that already owns this format.
    let (ring, _) = mailbox_ring::handle("/mailbox/ring");
    json!({
        "architecture": "shared-memory ring bus, 64-byte lock-free AgentMailboxMessage (k3_trit -1/0/+1)",
        "initialized": ring.get("status").and_then(|s| s.as_str()) != Some("error"),
        "stats": ring,
    })
}

fn pbft_consensus_payload() -> Value {
    // The sealed chain is JSONL; count it and take the tip the same way the ledger route does.
    let raw = std::fs::read_to_string(root().join("swarm_v2_artifacts/monad_ledger.jsonl")).ok();
    let (mut n, mut tip) = (0u64, Value::Null);
    if let Some(text) = raw {
        for line in text.lines().filter(|l| !l.trim().is_empty()) {
            n += 1;
            if let Ok(v) = serde_json::from_str::<Value>(line) {
                if let Some(h) = v.get("hash") {
                    tip = h.clone();
                }
            }
        }
    }
    let agents = crate::AGENTS.len() as u64;
    let f = (agents.saturating_sub(1)) / 3;
    json!({
        "consensus": {
            "total_blocks": n,
            "latest_block_hash": tip,
            "byzantine_tolerance": format!("f={} of N={} (quorum {})", f, agents, 2 * f + 1),
            "nodes": agents,
        },
        "basis": "swarm_v2_artifacts/monad_ledger.jsonl, with f=(N-1)/3 derived from the live roster",
    })
}

fn swarm_council_payload() -> Value {
    let agents = crate::AGENTS.len() as u64;
    let f = (agents.saturating_sub(1)) / 3;
    json!({
        "council": {
            "total_agents": agents,
            "quorum": 2 * f + 1,
            "byzantine_tolerance": format!("f={f}"),
            "roster": crate::AGENTS.iter().map(|a| a.name).collect::<Vec<&str>>(),
        },
        "basis": "the live roster in main.rs; quorum = 2f+1 with f=(N-1)/3",
    })
}

fn voltage_binder_payload() -> Value {
    // The kernel's OWN derivation (host_probe), not a stored constant. The core rail term uses
    // the LIVE V_core (AMD P-state VID via MSR 3, SVI2 datasheet 1.55 - vid*0.00625); the DDR
    // rail ratio is 1.0 by construction. The VID is the CPU's REQUESTED voltage — the actual
    // VRM/SVI2-DELIVERED rail voltage is not readable from userspace (SMBus is ring-0 and
    // BitLocker-protected), so the VID is the honest closest-measurable rail proxy.
    let (phi, basis, clock_mhz, mem_pct) = host_probe::phi_v();
    json!({
        "telemetry": {
            "phi_v": phi,
            "basis": basis,
            "clock_mhz": clock_mhz,
            "memory_percent_used": mem_pct,
            "cpu_percent": host_probe::cpu_percent(),
            "active_threads": host_probe::active_threads(),
            "memory_used_mb": host_probe::used_memory_mb(),
            "memory_total_mb": host_probe::total_memory_mb(),
            "l3_cache_kb": host_probe::l3_cache_kb(),
            "derivation": "phi = (V_core/1.25)^2 x (clock/4000) x max(0.5, 1 - mem%/200), clamped [0,1]; V_core = AMD P-state VID (requested, MSR 3); actual VRM rail not readable from userspace (SMBus BitLocker)",
        },
        "source": "host_probe on this host - measured, not stored",
    })
}

fn persistent_mmap_payload() -> Value {
    let bytes = std::fs::metadata(root().join("swarm_v2_artifacts/ipc_shared_arena.mmap"))
        .map(|m| m.len())
        .unwrap_or(0);
    let snap = read_json("swarm_v2_artifacts/cold_start_snapshot.json");
    json!({
        "status": if bytes > 0 { "ONLINE" } else { "unavailable" },
        "arena": { "file": "swarm_v2_artifacts/ipc_shared_arena.mmap", "bytes": bytes },
        "cold_start_snapshot": snap.map(|v| json!({
            "agents_saved": v.get("agents_saved").cloned().unwrap_or(json!(null)),
            "mmap_size_bytes": v.get("mmap_size_bytes").cloned().unwrap_or(json!(null)),
            "write_latency_us": v.get("write_latency_us").cloned().unwrap_or(json!(null)),
            "recovery_time_us": v.get("recovery_time_us").cloned().unwrap_or(json!(null)),
            "recovery_within_5ms": v.get("recovery_within_5ms").cloned().unwrap_or(json!(null)),
        })).unwrap_or(json!(null)),
        "basis": "the arena file's own size plus the recorded cold-start snapshot",
    })
}

fn semantic_cache_payload() -> Value {
    match read_json("swarm_v2_artifacts/semantic_cache.json") {
        Some(v) => {
            let n = v.get("entries").and_then(|e| e.as_array()).map(|a| a.len()).unwrap_or(0);
            json!({ "status": "ONLINE", "entries_count": n, "stats": { "entries_count": n },
                    "metrics": v.get("metrics").cloned().unwrap_or(json!(null)),
                    "source": "swarm_v2_artifacts/semantic_cache.json" })
        }
        None => json!({ "status": "unavailable",
                        "error": "semantic_cache.json not readable - not a clean result" }),
    }
}

fn turbovec_payload() -> Value {
    match packed_ternary::terstore() {
        Some(s) => {
            let as_f32 = (s.keys.len() as u64).saturating_mul(s.dim as u64).saturating_mul(4);
            json!({
                "status": "ONLINE",
                "store": {
                    "path": s.path,
                    "bytes": s.bytes,
                    "vectors": s.keys.len(),
                    "dim": s.dim,
                    "fnv1a64": s.fnv,
                },
                "compression_ratio": if s.bytes > 0 { json!(as_f32 as f64 / s.bytes as f64) } else { json!(null) },
                "step_result": { "ranker": "kernel:tercode-store" },
                "derivation": "compression_ratio = (vectors x dim x 4 bytes) / packed store bytes",
            })
        }
        None => json!({ "status": "unavailable",
                        "error": "no packed ternary store is loaded - not a clean result" }),
    }
}

fn performance_insights_payload() -> Value {
    match read_json("swarm_v2_artifacts/performance-budget.json") {
        Some(v) => {
            let comps = v
                .get("components")
                .and_then(|c| c.as_object().map(|m| m.len()))
                .unwrap_or(0);
            json!({
                "status": "ONLINE",
                "components_defined": comps,
                "validation": v.get("validation").cloned().unwrap_or(json!(null)),
                "efficiency_score": Value::Null,
                "health_score": Value::Null,
                "error": "the budget file declares targets; no measured score is recorded here, so none is reported",
                "source": "swarm_v2_artifacts/performance-budget.json",
            })
        }
        None => json!({ "status": "unavailable",
                        "error": "performance-budget.json not readable - not a clean result" }),
    }
}

pub fn handle(request: &mut Request, url: &str) -> Option<Response<std::io::Cursor<Vec<u8>>>> {
    let method = request.method().clone();
    let path = url.split('?').next().unwrap_or(url).trim_end_matches('/');
    if !path.starts_with("/system/modules") {
        return None;
    }
    if path == "/system/modules" && method == Method::Get {
        return Some(ok(inventory_payload()));
    }
    if method == Method::Get && path == "/system/modules/mailbox-bus" {
        return Some(ok(mailbox_bus_payload()));
    }
    if method == Method::Get && path == "/system/modules/pbft-consensus/status" {
        return Some(ok(pbft_consensus_payload()));
    }
    if method == Method::Get && path == "/system/modules/swarm-council/status" {
        return Some(ok(swarm_council_payload()));
    }
    if method == Method::Get && path == "/system/modules/hardware-voltage-binder" {
        return Some(ok(voltage_binder_payload()));
    }
    if method == Method::Get && path == "/system/modules/persistent-mmap" {
        return Some(ok(persistent_mmap_payload()));
    }
    if method == Method::Get && path == "/system/modules/semantic-cache" {
        return Some(ok(semantic_cache_payload()));
    }
    if method == Method::Get && path == "/system/modules/turbovec-femto" {
        return Some(ok(turbovec_payload()));
    }
    if method == Method::Get && path == "/system/modules/performance-insights/report" {
        return Some(ok(performance_insights_payload()));
    }

    // Modules with no persisted state say so. Every one of these previously returned the /system
    // OVERVIEW, so each panel showed the same global numbers and looked live while carrying nothing
    // about the module it named; the `see` field points at a live endpoint where one exists.
    let no_state: &[(&str, &str)] = &[
        ("/system/modules/sram-patterns", "/k3/status"),
        ("/system/modules/radix-trie-matcher", ""),
        ("/system/modules/benchmark-epistemology/verify", ""),
        ("/system/modules/proactive-loop/stats", ""),
        ("/system/modules/cognitive-ingestion/status", ""),
        ("/system/modules/ultrawork-loop/stats", "/ultrawork/missions"),
        ("/system/modules/digital-dna-loop/status", "/evolution/proposals"),
        ("/system/modules/mcp-tool-evolver/status", ""),
        ("/system/modules/auto-changelog/status", ""),
        ("/system/modules/artifact-pipeline/status", ""),
        ("/system/modules/sentinel-waf/status", ""),
        ("/system/modules/trm-orchestrator/status", "/swarm/orchestrator/stats"),
        ("/system/modules/cognitive-pipeline/optimal-path", ""),
        // The rest of the family: declared by swarm_v2/routes/system.py but not called by the UI, so
        // the first pass left them still borrowing the /system overview.
        ("/system/modules/generative-architect/status", "/generative-architect/status"),
        ("/system/modules/federation-sync/status", "/federation/stats"),
        ("/system/modules/architectural-rl/status", ""),
        ("/system/modules/memory-synchronizer/status", ""),
        ("/system/modules/skill-matrix-sync/status", ""),
        ("/system/modules/continuum-genomics/status", ""),
        ("/system/modules/microkernel-spawner/status", "/swarm/microkernel/status"),
        ("/system/modules/voltage-microkernel/status", "/system/modules/hardware-voltage-binder"),
        ("/system/modules/cosmos-microkernel-entanglement/status", ""),
        ("/system/modules/continuum-pbft/status", "/api/continuum/pbft/ledger"),
        ("/system/modules/microkernel-chat-bridge/status", ""),
        ("/system/modules/neuro-continuum/status", ""),
    ];
    if method == Method::Get {
        for (p, see) in no_state {
            if path == *p {
                return Some(ok(json!({
                    "status": "no_persisted_state",
                    "module_path": p,
                    "see": if see.is_empty() { Value::Null } else { json!(see) },
                    "reason": "this module keeps its state in process memory and writes no artifact this \
                               server can read, so the kernel reports that instead of echoing the /system \
                               overview as if it were this module's data",
                })));
            }
        }
    }

    // k3-kleene is deterministic Kleene logic - the same evaluator /k3/verdict serves.
    if method == Method::Post && path == "/system/modules/k3-kleene" {
        let mut body = String::new();
        let _ = request.as_reader().take(65_536).read_to_string(&mut body);
        let v = k3_retrieval::verdict_from_body(&body);
        let verdict = v.get("verdict").and_then(|x| x.as_i64()).unwrap_or(0);
        return Some(ok(json!({
            "label": v.get("verdict_name").cloned().unwrap_or(json!("UNKNOWN")),
            "confidence": v.get("truth_density").cloned().unwrap_or(json!(0.0)),
            "is_bypassable": verdict != -1,
            "counts": v.get("counts").cloned().unwrap_or(json!(null)),
            "rule": v.get("rule").cloned().unwrap_or(json!(null)),
        })));
    }

    // The rest are the modules' own engines; the kernel refuses rather than inventing a result.
    if method == Method::Post {
        let actions: &[(&str, &str)] = &[
            ("/system/modules/mcts-plan", "runs Monte-Carlo tree search in the platform's MCTS planner"),
            ("/system/modules/small-diffusion/denoise", "runs the small-diffusion ternary denoiser"),
            ("/system/modules/gan-synthesizer/generate", "runs the GAN synthesizer"),
            ("/system/modules/zero-human-test-gen/generate", "runs the zero-human test generator"),
            ("/system/modules/chain-of-verification/verify", "runs the chain-of-verification engine"),
            ("/system/modules/neural-wall/inspect", "runs the neural-wall threat inspector"),
            ("/system/modules/monad/absorb-experiences", "runs the monad experience-absorption write"),
            ("/system/modules/continuum-genomics/refract", "runs the Continuum genomics refraction engine"),
            ("/system/modules/microkernel-spawner/spawn", "spawns a microkernel sub-agent process"),
            ("/system/modules/voltage-microkernel/scan", "runs the voltage microkernel scan"),
            ("/system/modules/federation-sync/broadcast", "broadcasts to federation peers"),
        ];
        for (p, what) in actions {
            if path == *p {
                return Some(engine_only(&format!(
                    "{p} {what}. That is engine work owned by the module, not by this server, so no \
                     result is produced rather than a fabricated one."
                )));
            }
        }
        // Any other POST under the family is refused too: nothing here may fall through to the
        // /system overview, which is what made every module read look "live".
        return Some(engine_only(&format!(
            "{path} is not a route this kernel serves. The /system/modules family answers per module \
             (reads) or refuses with a reason (engine actions); no path under it falls back to the \
             /system overview."
        )));
    }

    // An unmatched GET must not borrow the parent overview either.
    Some(ok(json!({
        "status": "unknown_module_path",
        "module_path": path,
        "reason": "this path is not a module this kernel serves; the family answers per module or \
                   reports no_persisted_state, and never echoes the /system overview",
    })))
}
