//! AURA transform — the real, offline symbolic classifier behind the /aura refusal.
//!
//! Port of `swarm_v2/core/aura_cognition.py` (find_relevant_metaconcepts + branch_cognitive_tree).
//! The transform is: text -> metaconcept (keyword score) -> Kleene K3 gate. No model, no embeddings,
//! no network. The fabricated telemetry in the Python (phi_v phase-lock, 15.53 THz phonon mode,
//! 0.084 uJ dissipation, solfeggio frequencies) is deliberately NOT ported: the verdict here is
//! purely K3 over metaconcept polarities, so every number in the response is derivable from the
//! input and the table.

use serde_json::{json, Value};

/// Real transform fields only. density/frequency/wavefunction/voltage_rail are omitted from the
/// generated table because they carried fabricated substrate numbers.
pub struct AuraMetaconcept {
    pub key: &'static str,
    pub name: &'static str,
    pub category: &'static str,
    pub polarity: i8,
    pub description: &'static str,
    pub intent: &'static [&'static str],
    pub links: &'static [&'static str],
}

// Kleene K3 on the {-1, 0, +1} alphabet: AND = min, OR = max, NOT = negation.
#[inline]
fn k3_and(a: i8, b: i8) -> i8 {
    a.min(b)
}
#[inline]
fn k3_or(a: i8, b: i8) -> i8 {
    a.max(b)
}
#[inline]
fn k3_not(a: i8) -> i8 {
    -a
}

fn k3_label(t: i8) -> &'static str {
    match t {
        1 => "+1 (TRUE)",
        -1 => "-1 (FALSE)",
        _ => "0 (UNKNOWN)",
    }
}

/// Resolve a metaconcept by key or name (case-insensitive), mirroring the Python get_metaconcept.
pub fn find_metaconcept(key_or_name: &str) -> Option<&'static AuraMetaconcept> {
    use crate::aura_metaconcepts::METACONCEPTS;
    let clean = key_or_name.trim();
    METACONCEPTS
        .iter()
        .find(|m| m.key == clean || m.name.eq_ignore_ascii_case(clean) || m.key.eq_ignore_ascii_case(clean))
}

/// The actual transform: score each metaconcept against the text by lexical overlap, return top_k.
/// Faithful port of aura_cognition.find_relevant_metaconcepts.
pub fn find_relevant_metaconcepts(text: &str, top_k: usize) -> Vec<&'static AuraMetaconcept> {
    use crate::aura_metaconcepts::METACONCEPTS;
    let t_low = text.to_lowercase();
    let mut scored: Vec<(f64, &'static AuraMetaconcept)> = Vec::with_capacity(METACONCEPTS.len());
    for m in METACONCEPTS {
        let mut score = 0.0f64;
        if t_low.contains(&m.key.to_lowercase()) {
            score += 10.0;
        }
        if t_low.contains(&m.name.to_lowercase()) {
            score += 8.0;
        }
        if t_low.contains(&m.category.to_lowercase()) {
            score += 3.0;
        }
        for w in m.description.split_whitespace() {
            let w = w.to_lowercase();
            if w.len() > 4 && t_low.contains(&w) {
                score += 1.5;
            }
        }
        for it in m.intent {
            if t_low.contains(&it.to_lowercase()) {
                score += 2.5;
            }
        }
        if t_low.contains("voltage") && m.description.to_lowercase().contains("voltage") {
            score += 4.0;
        }
        if t_low.contains("ternary")
            && (m.description.to_lowercase().contains("ternary")
                || m.description.to_lowercase().contains("k3"))
        {
            score += 4.0;
        }
        if t_low.contains("resonance") && m.description.to_lowercase().contains("resonance") {
            score += 3.0;
        }
        scored.push((score, m));
    }
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    let mut out: Vec<&'static AuraMetaconcept> = scored
        .iter()
        .filter(|(s, _)| *s > 0.0)
        .map(|(_, m)| *m)
        .collect();
    out.truncate(top_k.max(1));
    if out.is_empty() {
        use crate::aura_metaconcepts::METACONCEPTS as M;
        for want in ["Unity", "Resonance", "MonadTruth"] {
            if let Some(m) = M.iter().find(|m| m.key == want) {
                out.push(m);
            }
        }
    }
    out
}

/// The cognitive branching tree: root -> resonance children -> K3 gates -> composite verdict.
/// Faithful port of aura_cognition.branch_cognitive_tree, minus the fabricated voltage telemetry.
pub fn branch_cognitive_tree(directive: &str, seed_concepts: &[String], depth: usize) -> Value {
    let roots: Vec<&'static AuraMetaconcept> =
        seed_concepts.iter().filter_map(|s| find_metaconcept(s)).collect();
    let roots = if roots.is_empty() {
        find_relevant_metaconcepts(directive, 2)
    } else {
        roots
    };
    let root = roots[0];

    let mut visited: Vec<&'static str> = vec![root.key];
    let mut branches: Vec<Value> = Vec::new();
    let mut composite: i8 = root.polarity;

    for child_key in root.links.iter().take(3) {
        let child = match find_metaconcept(child_key) {
            Some(c) => c,
            None => continue,
        };
        if visited.contains(&child.key) {
            continue;
        }
        visited.push(child.key);

        let k3_and_v = k3_and(root.polarity, child.polarity);
        let k3_or_v = k3_or(root.polarity, child.polarity);
        // GATE in the Python is k3_and(x, NOT(-1)) == k3_and(x, +1) == x (identity).
        let k3_gate = k3_and(k3_and_v, k3_not(-1));

        let mut sub_branches: Vec<Value> = Vec::new();
        if depth > 1 {
            for sub_key in child.links.iter().take(2) {
                if let Some(sub) = find_metaconcept(sub_key) {
                    if !visited.contains(&sub.key) {
                        visited.push(sub.key);
                        let sub_k3 = k3_and(k3_gate, sub.polarity);
                        sub_branches.push(json!({
                            "concept": sub.name,
                            "key": sub.key,
                            "category": sub.category,
                            "polarity": sub.polarity,
                            "k3_gate": sub_k3,
                            "k3_label": k3_label(sub_k3),
                        }));
                    }
                }
            }
        }

        composite = k3_and(composite, k3_gate);
        branches.push(json!({
            "source": root.key,
            "target": child.key,
            "target_name": child.name,
            "category": child.category,
            "target_polarity": child.polarity,
            "k3_and": k3_and_v,
            "k3_or": k3_or_v,
            "k3_gate": k3_gate,
            "k3_verdict": k3_label(k3_gate),
            "sub_branches": sub_branches,
        }));
    }

    let action = match composite {
        1 => "PROCEED",
        -1 => "SHIELD",
        _ => "DELIBERATE",
    };

    json!({
        "directive": directive,
        "root_metaconcept": {
            "key": root.key,
            "name": root.name,
            "category": root.category,
            "polarity": root.polarity,
            "description": root.description,
            "intent": root.intent,
        },
        "cognitive_branches": branches,
        "branch_count": branches.len(),
        "visited_concepts": visited,
        "k3_logic_evaluation": {
            "composite_trit": composite,
            "verdict": k3_label(composite),
            "action": action,
            "rule": "Kleene K3 AND over metaconcept polarities: FALSE if any branch is FALSE, TRUE only if every branch is TRUE, else UNKNOWN",
        },
        "monad_anchor": "0x4D4F4E4144000001",
        "honest_note": "verdict is purely K3 over the metaconcept graph; no voltage/THz/dissipation telemetry is fabricated",
    })
}
