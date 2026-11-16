//! Parity probe (G190): the same fixed inputs computed by both builds, so the two outputs can be diffed.
//! Identical in both trees except the ONE path prefix. Vectors are chosen INSIDE each function's domain:
//! the first version fed "critical"/"high" to a matcher keyed on "HEALTH_CHECK", got the default branch
//! everywhere, and would have reported perfect parity on inputs that never exercised the code.

fn s(v: &str) -> serde_json::Value { serde_json::json!(v) }

pub fn run() -> serde_json::Value {
    let mut out = serde_json::Map::new();
    // domain: UPPERCASE maintenance classes
    for class in ["HEALTH_CHECK", "MEMORY_PRUNING", "GARBAGE_COLLECTION", "METRICS_COLLECTION",
                  "CACHE_CLEANUP", "UNKNOWN_CLASS", "health_check"] {
        out.insert(format!("arbiter.base_priority.{}", class),
                   serde_json::json!(trm_uefi_kernel::task_arbiter::base_priority(class)));
    }
    // domain: keyword-carrying task sentences
    for task in ["analyze the ledger", "audit security posture", "list services", "write file x",
                 "code a parser", "research paper on SSM", "summarize the log", "plain words"] {
        out.insert(format!("arbiter.classify_task.{}", task.replace(' ', "_")),
                   s(trm_uefi_kernel::task_arbiter::classify_task(task)));
    }
    for cpu in [0.0f64, 5.0, 25.0, 50.0, 75.0, 95.0, 100.0] {
        out.insert(format!("arbiter.usage_window.{}", cpu), s(trm_uefi_kernel::task_arbiter::usage_window(cpu)));
    }
    // domain: UPPERCASE usage windows
    for w in ["LOW_USAGE", "NORMAL_USAGE", "HIGH_USAGE", "CRITICAL_USAGE", "low_usage"] {
        out.insert(format!("arbiter.window_adjustment.{}", w),
                   serde_json::json!(trm_uefi_kernel::task_arbiter::window_adjustment(w)));
    }
    out.insert("arbiter.dynamic_priority.low_on_time".into(),
               serde_json::json!(trm_uefi_kernel::task_arbiter::dynamic_priority(100, "LOW_USAGE", 0, 60)));
    out.insert("arbiter.dynamic_priority.critical_overdue".into(),
               serde_json::json!(trm_uefi_kernel::task_arbiter::dynamic_priority(100, "CRITICAL_USAGE", 300, 60)));
    out.insert("arbiter.dynamic_priority.never_negative".into(),
               serde_json::json!(trm_uefi_kernel::task_arbiter::dynamic_priority(10, "CRITICAL_USAGE", 0, 60)));
    out.insert("arbiter.compute_target.high_low".into(),
               s(trm_uefi_kernel::task_arbiter::optimal_compute_target("high", 0.3)));
    out.insert("arbiter.compute_target.low_high".into(),
               s(trm_uefi_kernel::task_arbiter::optimal_compute_target("low", 0.9)));
    // role present in the task text scores higher than role absent
    out.insert("arbiter.vote_score.role_present".into(),
               serde_json::json!(trm_uefi_kernel::task_arbiter::decentralized_vote_score("Devo", "devo fixes the parser", 5)));
    out.insert("arbiter.vote_score.role_absent".into(),
               serde_json::json!(trm_uefi_kernel::task_arbiter::decentralized_vote_score("Devo", "fix the parser", 5)));
    out.insert("arbiter.vote_score.history_capped".into(),
               serde_json::json!(trm_uefi_kernel::task_arbiter::decentralized_vote_score("Shield", "audit", 40)));
    for id in ["Agent 007 / ops!", "  Devo  ", "A.B/C", "", "kapha-und-zeichen"] {
        out.insert(format!("registry.sanitize.{}", id.replace(' ', "_")),
                   s(&trm_uefi_kernel::expert_registry::sanitize_identifier(id)));
    }
    serde_json::Value::Object(out)
}
