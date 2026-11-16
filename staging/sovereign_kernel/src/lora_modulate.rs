//! LoRA response modulation: turn a 64-dim adapter delta into a modified response.
//!
//! Ported from `swarm_v2/core/lora_response_modulator.py`. The existing `strip_lora_modulator.dll`
//! does NOT implement this: it is one of the 30 synthesized stubs (pseudo-random ternary weights and
//! a magic SCALE_FACTOR), which is why the Python module loaded it and then never called it.
//!
//! The ABI writes UTF-8 into a caller-owned buffer and returns the byte length the result needs, so
//! ownership never crosses the boundary; a caller that passed too small a buffer can retry.

use std::os::raw::c_uint;

const DOMAINS: [&str; 10] = [
    "security", "logic", "development", "research", "verification",
    "architecture", "integration", "data", "orchestration", "telemetry",
];
const TONES: [&str; 10] = [
    "formal", "technical", "concise", "confident", "proactive",
    "analytical", "direct", "diplomatic", "urgent", "speculative",
];
const ACTIONS: [&str; 10] = [
    "investigate", "defend", "optimize", "build", "verify",
    "design", "connect", "fetch", "schedule", "monitor",
];

fn argmax_abs(values: &[f32]) -> usize {
    let mut best = 0usize;
    let mut best_v = values[0].abs();
    for (i, v) in values.iter().enumerate().skip(1) {
        if v.abs() > best_v {
            best = i;
            best_v = v.abs();
        }
    }
    best
}

fn domain_prefix(domain: &str) -> &'static str {
    match domain {
        "security" => "🛡️ [Shield Analysis]",
        "logic" => "🧮 [Logic Assessment]",
        "development" => "⚙️ [Devo Build]",
        "research" => "🔬 [Seeker Finding]",
        "verification" => "✓ [Verify Audit]",
        "architecture" => "🏗️ [Archi Blueprint]",
        "integration" => "🔗 [Flow Route]",
        "data" => "📡 [Bridge Fetch]",
        "orchestration" => "🎯 [Orchestra Schedule]",
        "telemetry" => "📊 [EventLedger Monitor]",
        _ => "",
    }
}

fn action_phrase(action: &str) -> &'static str {
    match action {
        "investigate" => " Recommend deeper investigation.",
        "defend" => " Security posture hardening advised.",
        "optimize" => " Optimization opportunity identified.",
        "build" => " Ready for implementation.",
        "verify" => " Verification checkpoint passed.",
        "design" => " Architectural review recommended.",
        "connect" => " Service integration gate open.",
        "fetch" => " Data retrieval standing by.",
        "schedule" => " Task queued for execution.",
        "monitor" => " Continuous monitoring engaged.",
        _ => "",
    }
}

/// Build the modified response. `delta` shorter than 32 returns `base` unchanged, as the Python does.
pub fn modulate(base: &str, delta: &[f32], agent: &str) -> String {
    if delta.len() < 32 {
        return base.to_string();
    }

    let confidence = (1.0 + delta[0]).clamp(0.1, 2.0);
    let verbosity = (1.0 + delta[1]).clamp(0.3, 3.0);

    let domain_scores = &delta[2..12];
    let top_domain_idx = argmax_abs(domain_scores);
    let top_domain = DOMAINS[top_domain_idx];

    // numpy: np.argsort(|x|)[-2:][::-1] -> [largest, second largest]
    let tone_scores = &delta[12..22];
    let mut order: Vec<usize> = (0..tone_scores.len()).collect();
    order.sort_by(|&a, &b| {
        tone_scores[a]
            .abs()
            .partial_cmp(&tone_scores[b].abs())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let primary_tone = TONES[order[9]];
    let secondary_tone = TONES[order[8]];

    let action_scores = &delta[22..32];
    let top_action_idx = argmax_abs(action_scores);
    let top_action = ACTIONS[top_action_idx];

    let mut parts: Vec<String> = vec![base.trim().to_string()];

    if confidence > 0.7 {
        let p = domain_prefix(top_domain);
        let prefix = if p.is_empty() { format!("[{agent}]") } else { p.to_string() };
        if !parts[0].contains(&prefix) {
            parts.insert(0, prefix);
        }
    }

    if confidence < 0.5 {
        parts.push(format!("(confidence: low, tone: {primary_tone})"));
    } else if confidence > 1.5 {
        parts.push(format!("(high confidence, {primary_tone} assessment)"));
    } else {
        parts.push(format!("({primary_tone} mode)"));
    }

    if action_scores[top_action_idx].abs() > 0.3 {
        parts.push(action_phrase(top_action).to_string());
    }

    let mut response = parts
        .into_iter()
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join(" ");

    if verbosity > 2.0 {
        response.push_str(&format!(
            "\n  • Domain: {top_domain} (emphasis: {:.2})",
            domain_scores[top_domain_idx]
        ));
        response.push_str(&format!("\n  • Tone: {primary_tone}/{secondary_tone}"));
        response.push_str(&format!("\n  • Bias: {top_action}"));
    }

    response
}

/// Writes the result into `out` and returns the full byte length. Bytes beyond `cap` are dropped.
#[no_mangle]
pub extern "C" fn rust_lora_modulate(
    base: *const u8,
    base_len: usize,
    delta: *const f32,
    n: usize,
    agent: *const u8,
    agent_len: usize,
    out: *mut u8,
    cap: usize,
) -> c_uint {
    if base.is_null() || delta.is_null() || agent.is_null() {
        return 0;
    }
    let base_s = unsafe { std::slice::from_raw_parts(base, base_len) };
    let agent_s = unsafe { std::slice::from_raw_parts(agent, agent_len) };
    let delta_v = unsafe { std::slice::from_raw_parts(delta, n) };

    let result = modulate(
        &String::from_utf8_lossy(base_s),
        delta_v,
        &String::from_utf8_lossy(agent_s),
    );
    let bytes = result.as_bytes();
    if !out.is_null() && cap > 0 {
        let take = bytes.len().min(cap);
        unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), out, take) };
    }
    bytes.len() as c_uint
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_delta_passes_through() {
        assert_eq!(modulate("hello", &[0.0; 8], "Shield"), "hello");
    }

    #[test]
    fn high_confidence_gets_a_domain_prefix() {
        let mut d = [0.0f32; 64];
        d[0] = 0.5; // confidence 1.5
        d[2] = 0.9; // security
        d[12] = 0.8; // formal is the strongest tone (all-zero tone scores pick the last index)
        let out = modulate("scan complete", &d, "Shield");
        assert!(out.starts_with("🛡️ [Shield Analysis]"), "{out}");
        assert!(out.ends_with("(formal mode)"), "{out}");
    }

    #[test]
    fn verbosity_appends_the_breakdown() {
        let mut d = [0.0f32; 64];
        d[1] = 1.5; // verbosity 2.5
        d[3] = 0.8; // logic
        let out = modulate("done", &d, "Logic");
        assert!(out.contains("• Domain: logic"), "{out}");
        assert!(out.contains("• Bias: investigate"), "{out}");
    }
}
