//! swarm-health — Motion C: the real-time health monitor.
//!
//! Pulse (vitals) + EventLedger (FDA/regulatory events) — reads the REAL
//! system_event_ledger.jsonl and produces a K3 health verdict. Honest:
//! every record read is real; the verdict is a computed collapse.

use std::fs;

/// A real event-ledger record.
#[derive(Debug, Clone)]
pub struct LedgerEvent {
    pub ts: String,
    pub category: String,
    pub event: String,
    pub source: String,
}

/// Parse the real event ledger (JSONL) — returns actual records.
pub fn parse_ledger(path: &str) -> Vec<LedgerEvent> {
    let mut out = Vec::new();
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return out,
    };
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        // The real ledger is JSONL: {"ts": ..., "category": ..., "event": ..., "source": ..., ...}.
        // Parse each line as JSON rather than splitting on commas/colons (timestamps contain colons).
        let v: serde_json::Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let field = |k: &str| v.get(k).and_then(|x| x.as_str()).unwrap_or("").to_string();
        let event = field("event");
        if event.is_empty() {
            continue;
        }
        out.push(LedgerEvent {
            ts: field("ts"),
            category: field("category"),
            event,
            source: field("source"),
        });
    }
    out
}

/// K3 health verdict from the event stream.
/// Signals: recent events (healthy), error/alert categories (concern),
/// event rate (activity).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthVerdict {
    Stable,   // TRUE dominates — system healthy
    Watch,    // UNKNOWN dominates — monitor
    Critical, // FALSE dominates — act now
}

pub fn health_verdict(events: &[LedgerEvent]) -> (HealthVerdict, String) {
    let total = events.len();
    let errors = events.iter().filter(|e| {
        let c = e.category.to_lowercase();
        c.contains("error") || c.contains("alert") || c.contains("critical") || c.contains("fail")
    }).count();
    let warnings = events.iter().filter(|e| {
        let c = e.category.to_lowercase();
        c.contains("warn") || c.contains("degrad")
    }).count();
    let healthy = total.saturating_sub(errors + warnings);

    // collapse: healthy > errors → Stable; errors dominate → Critical; else Watch
    let verdict = if errors == 0 && healthy >= total.saturating_sub(1) {
        HealthVerdict::Stable
    } else if errors > healthy && errors > 0 {
        HealthVerdict::Critical
    } else {
        HealthVerdict::Watch
    };
    let detail = format!("{} events: {} healthy, {} warnings, {} errors", total, healthy, warnings, errors);
    (verdict, detail)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ev(cat: &str) -> LedgerEvent {
        LedgerEvent { ts: "t".into(), category: cat.into(), event: "e".into(), source: "s".into() }
    }

    #[test]
    fn healthy_stream_is_stable() {
        let events = vec![ev("SYSTEM"), ev("SYSTEM"), ev("INFO")];
        assert_eq!(health_verdict(&events).0, HealthVerdict::Stable);
    }

    #[test]
    fn error_stream_is_critical() {
        let events = vec![ev("ERROR"), ev("ERROR"), ev("SYSTEM")];
        assert_eq!(health_verdict(&events).0, HealthVerdict::Critical);
    }

    #[test]
    fn parse_real_ledger() {
        let events = parse_ledger(&swarm_paths::resolve("/mnt/c/Users/shawn/Downloads/TRM-Swarm-OS-v17-Enterprise-Private-main/TRM-Swarm-OS-v17-Enterprise-Private-main/swarm_v2_global_memory/system_event_ledger.jsonl"));
        assert!(events.len() >= 30, "real ledger has 33 records, got {}", events.len());
    }
}
