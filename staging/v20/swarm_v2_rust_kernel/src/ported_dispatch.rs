//! TRM Swarm OS — Ported no_std Module Dispatch Registry (Track A wiring).
//!
//! The 8 algorithmic modules were ported 1:1 from `swarm_v2/core/*.py`,
//! verified bit-exact against Python in the UEFI no_std tree, and copied
//! here so the sovereign kernel dispatches through REAL ported logic instead
//! of inline duplicates. This registry is the single dispatch seam:
//!
//!   - `dispatch(name, op)`   — named module + operation → real ported result
//!   - `run_all()`            — executes every module's real logic (self-verify)
//!   - C-ABI exports          — `rust_ported_module_count`, `rust_ported_run`
//!
//! Every function below calls into the actual ported module implementations
//! (`comm_protocol`, `radix_trie_matcher`, `coherence_thresholds`,
//! `voltage_node_engine`, `continuum_pbft`, `coordination_protocol`,
//! `phoenix_protocol_engine`, `dram_pattern_computer`) — nothing is a stub.

/// Module names in registry order (matches the 8 Track-A ports).
pub const PORTED_MODULE_NAMES: &[&str] = &[
    "comm_protocol",
    "radix_trie_matcher",
    "coherence_thresholds",
    "voltage_node_engine",
    "continuum_pbft",
    "coordination_protocol",
    "phoenix_protocol_engine",
    "dram_pattern_computer",
];

pub const PORTED_MODULE_COUNT: usize = PORTED_MODULE_NAMES.len();

/// One verified execution of real ported logic per module, returning a
/// one-line status string per module (module | status | key value).
pub fn run_ported_self_verify() -> Vec<(&'static str, &'static str, String)> {
    let mut out: Vec<(&'static str, &'static str, String)> = Vec::new();

    // 1. comm_protocol — real HMAC-SHA256 trust token round-trip.
    {
        let tok = crate::comm_protocol::generate_trust_token("Hermes", "Devo", "T-1", 1_700_000_000);
        let ok = crate::comm_protocol::verify_trust_token("Hermes", "Devo", "T-1", &tok, 1_700_000_000)
            && !crate::comm_protocol::verify_trust_token("Hermes", "Devo", "T-1", &tok, 1_700_000_000 + 900);
        out.push(("comm_protocol", if ok { "OK" } else { "FAIL" }, format!("token={}", &tok[..16])));
    }

    // 2. radix_trie_matcher — real trie register + prefix match.
    {
        let trie = crate::radix_trie_matcher::get_radix_trie();
        let m = trie.match_prefix("health check please");
        let ok = m.is_some() && m.unwrap().pattern == "health check";
        out.push(("radix_trie_matcher", if ok { "OK" } else { "FAIL" }, "prefix=health check".into()));
    }

    // 3. coherence_thresholds — real stability verdict chain.
    {
        use crate::coherence_thresholds::StabilityMetrics;
        let sm = StabilityMetrics { cpu_load: 90.0, memory_usage: 40.0, error_rate: 0.01 };
        let v = crate::coherence_thresholds::check_stability(&sm);
        let ok = matches!(v, crate::coherence_thresholds::StabilityVerdict::Level1Warning);
        out.push(("coherence_thresholds", if ok { "OK" } else { "FAIL" }, format!("verdict={:?}", v)));
    }

    // 4. voltage_node_engine — real ActionVector decode (names + round4).
    {
        let av = crate::voltage_node_engine::ActionVector {
            agent_id: 5,
            intent_id: 1,
            confidence: 0.123456,
            k3_trit: 1,
        };
        let r = crate::voltage_node_engine::decode_voltage_eval(&av);
        let ok = r.agent == "Devo" && r.intent == "WRITE" && (r.confidence - 0.1235).abs() < 1e-9;
        out.push(("voltage_node_engine", if ok { "OK" } else { "FAIL" }, format!("{}|{}|{}", r.agent, r.intent, r.confidence)));
    }

    // 5. continuum_pbft — real Byzantine quorum math (f=4 → N=14 council).
    {
        let f = 4u64;
        let req = crate::continuum_pbft::pbft_required_votes(f);
        let nodes = crate::continuum_pbft::pbft_min_total_nodes(f);
        let ok = req == 9 && nodes == 13 && crate::continuum_pbft::pbft_quorum_met(req, f);
        out.push(("continuum_pbft", if ok { "OK" } else { "FAIL" }, format!("2f+1={} N={}", req, nodes)));
    }

    // 6. coordination_protocol — real mission-completion classifier.
    {
        let ok = crate::coordination_protocol::is_mission_completion(
            "[COORD-ab12] phase=completed x", "ab12", Some("completed"),
        );
        out.push(("coordination_protocol", if ok { "OK" } else { "FAIL" }, "mission=completed".into()));
    }

    // 7. phoenix_protocol_engine — real phase-0 → next() advance.
    {
        let p0 = crate::phoenix_protocol_engine::PHOENIX_PHASE_ORDER[0];
        let nxt = p0.next();
        let ok = nxt.is_some() && nxt.unwrap() != p0;
        out.push(("phoenix_protocol_engine", if ok { "OK" } else { "FAIL" }, format!("phase0->{:?}", nxt)));
    }

    // 8. dram_pattern_computer — real DRAM specs (274.9B cells, 128 banks).
    {
        let s = crate::dram_pattern_computer::dram_specs();
        let rb = crate::dram_pattern_computer::pathway_row_buffer(128);
        let ok = s.cells == 274_877_906_944 && s.banks == 128 && rb.total_cells > 0;
        out.push(("dram_pattern_computer", if ok { "OK" } else { "FAIL" }, format!("{}B cells|{} GB/s", s.cells / 1_000_000_000, s.bandwidth_gbps)));
    }

    out
}

/// Dispatch a named module's core operation, returning a human-readable
/// result line. Unknown module/op returns an explicit error string.
pub fn dispatch(module: &str, op: &str) -> Result<String, String> {
    match (module, op) {
        ("comm_protocol", "token") => {
            let tok = crate::comm_protocol::generate_trust_token("Hermes", "Devo", "dispatch", 1_700_000_000);
            Ok(format!("64-hex MAT token: {}", tok))
        }
        ("radix_trie_matcher", "match") => {
            let trie = crate::radix_trie_matcher::get_radix_trie();
            match trie.match_prefix("health check please") {
                Some(m) => Ok(format!("prefix match: '{}' -> {}", m.pattern, m.endpoint)),
                None => Err("no prefix match".into()),
            }
        }
        ("coherence_thresholds", "verdict") => {
            use crate::coherence_thresholds::StabilityMetrics;
            let sm = StabilityMetrics { cpu_load: 95.0, memory_usage: 90.0, error_rate: 0.5 };
            Ok(format!("verdict: {:?}", crate::coherence_thresholds::check_stability(&sm)))
        }
        ("voltage_node_engine", "decode") => {
            let av = crate::voltage_node_engine::ActionVector { agent_id: 13, intent_id: 2, confidence: 0.987654, k3_trit: 1 };
            let r = crate::voltage_node_engine::decode_voltage_eval(&av);
            Ok(format!("agent={} intent={} conf={} trit={}", r.agent, r.intent, r.confidence, r.k3_trit))
        }
        ("continuum_pbft", "quorum") => {
            Ok(format!("f=4 -> req={} N={}", crate::continuum_pbft::pbft_required_votes(4), crate::continuum_pbft::pbft_min_total_nodes(4)))
        }
        ("coordination_protocol", "classify") => {
            Ok(format!("kind={:?}", crate::coordination_protocol::classify_coord_message(
                Some("coordination_task"), false, None, None, None)))
        }
        ("phoenix_protocol_engine", "phase") => {
            Ok(format!("phase0={:?} next={:?}", crate::phoenix_protocol_engine::PHOENIX_PHASE_ORDER[0], crate::phoenix_protocol_engine::PHOENIX_PHASE_ORDER[0].next()))
        }
        ("dram_pattern_computer", "specs") => {
            let s = crate::dram_pattern_computer::dram_specs();
            Ok(format!("{}GB {} | {} cells | {} GB/s | {} banks", s.capacity_gb, crate::dram_pattern_computer::DRAM_TYPE, s.cells, s.bandwidth_gbps, s.banks))
        }
        (m, _) => Err(format!("unknown ported module '{m}'")),
    }
}

/// C-ABI: number of registered ported modules.
#[no_mangle]
pub extern "C" fn rust_ported_module_count() -> i32 {
    PORTED_MODULE_COUNT as i32
}

/// C-ABI: run the full ported-module self-verify; returns number of OK
/// modules. `failures` is written with the first failing module name (empty
/// string when all pass) so the Python binder can report a real status.
#[no_mangle]
pub extern "C" fn rust_ported_run(failures_out: *mut u8, failures_cap: usize) -> i32 {
    let results = run_ported_self_verify();
    let ok_count = results.iter().filter(|(_, s, _)| *s == "OK").count();

    if !failures_out.is_null() && failures_cap > 0 {
        let first_fail = results.iter().find(|(_, s, _)| *s != "OK").map(|(n, _, _)| *n).unwrap_or("");
        unsafe {
            let dst = std::slice::from_raw_parts_mut(failures_out, failures_cap);
            for (i, b) in first_fail.as_bytes().iter().enumerate() {
                if i >= failures_cap { break; }
                dst[i] = *b;
            }
        }
    }
    ok_count as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ported_registry_all_eight_pass() {
        let results = run_ported_self_verify();
        assert_eq!(results.len(), 8, "all 8 ported modules must run");
        for (name, status, val) in &results {
            assert_eq!(*status, "OK", "module {name} failed: {val}");
        }
    }

    #[test]
    fn test_ported_dispatch_named_ops() {
        assert!(dispatch("comm_protocol", "token").unwrap().contains("MAT token"));
        assert!(dispatch("continuum_pbft", "quorum").unwrap().contains("req=9"));
        assert!(dispatch("dram_pattern_computer", "specs").unwrap().contains("274877906944"));
        assert!(dispatch("no_such_module", "x").is_err());
    }

    #[test]
    fn test_ported_c_abi() {
        let mut buf = [0u8; 64];
        let ok = rust_ported_run(buf.as_mut_ptr(), buf.len());
        assert_eq!(ok, 8);
    }
}
