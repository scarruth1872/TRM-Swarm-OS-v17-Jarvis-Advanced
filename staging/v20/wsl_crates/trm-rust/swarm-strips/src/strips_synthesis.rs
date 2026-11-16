//! Synthesis & Intent strips: SynthesisPipeline (multi-output aggregate), IntentBuffer (intent routing),
//! PersonaMatrix (deterministic persona traits).

/// Aggregate multiple numeric model outputs: mean + majority-sign consensus.
pub fn synthesis_pipeline(input: &str) -> String {
    let vs: Vec<f64> = input.split(',').filter_map(|t| t.trim().parse::<f64>().ok()).collect();
    if vs.is_empty() {
        return r#"{"agent":"SynthesisPipeline","status":"error","error":"input = comma-separated numeric outputs"}"#.to_string();
    }
    let mean = vs.iter().sum::<f64>() / vs.len() as f64;
    let pos = vs.iter().filter(|v| **v > 0.0).count();
    let consensus = if pos as f64 >= vs.len() as f64 / 2.0 { "positive" } else { "negative" };
    format!(r#"{{"agent":"SynthesisPipeline","status":"ok","mean":{mean:.4},"consensus":"{consensus}","kind":"derived"}}"#)
}

/// Keyword intent classifier routing a query to a facet/fleet.
pub fn intent_buffer(input: &str) -> String {
    let q = input.to_lowercase();
    let (facet, fleet) =
        if ["drug", "molecule", "smiles", "protein", "bind", "chembl"].iter().any(|k| q.contains(k)) {
            ("molecule", "Scientific Intelligence")
        } else if ["security", "vuln", "crypto", "scan"].iter().any(|k| q.contains(k)) {
            ("security", "Security & Defense")
        } else if ["logic", "proof", "math", "theorem"].iter().any(|k| q.contains(k)) {
            ("logic", "Logic & Mathematics")
        } else if ["code", "refactor", "test", "patch"].iter().any(|k| q.contains(k)) {
            ("code", "Code & Refactoring")
        } else if ["memory", "retrieval", "query", "recall"].iter().any(|k| q.contains(k)) {
            ("retrieval", "Telemetry & Memory")
        } else if ["heal", "chaos", "route", "health"].iter().any(|k| q.contains(k)) {
            ("healing", "Autonomous Healing")
        } else {
            ("synthesis", "Synthesis & Intent")
        };
    format!(
        r#"{{"agent":"IntentBuffer","status":"ok","query":"{}","facet":"{facet}","fleet":"{fleet}","kind":"derived"}}"#,
        crate::json_escape(input)
    )
}

/// Deterministic persona trait vector from a name.
pub fn persona_matrix(input: &str) -> String {
    let h = crate::fnv1a(input);
    let openness = ((h >> 0) % 101) as u8;
    let conscientiousness = ((h >> 8) % 101) as u8;
    let stability = ((h >> 16) % 101) as u8;
    format!(
        r#"{{"agent":"PersonaMatrix","status":"ok","persona":"{}","traits":{{"openness":{openness},"conscientiousness":{conscientiousness},"stability":{stability}}},"kind":"derived"}}"#,
        crate::json_escape(input)
    )
}
