//! /reason/* — the reasoning engine exposed over the kernel.
//!
//! Three endpoints, all of them thin wrappers over `trm_uefi_kernel::reasoning_engine`:
//!
//!   GET  /reason/rulebases  — what can be asked, and what the engine refuses to do
//!   POST /reason/forward    — saturate a rule base from asserted facts (571)
//!   POST /reason/prove      — goal-directed proof with a derivation trace (572 + 580)
//!
//! A malformed body is refused with 400. It is NOT defaulted to `{}`: an empty fact set would answer
//! "UNKNOWN" to every question, which looks like a conservative answer and is really a broken request.
//! This is the same defect that let a malformed kanban move relocate a card (corrected 2026-11-24).

use serde_json::{json, Value};
use trm_uefi_kernel::reasoning_engine::{derivation_json, Engine, Rule};

pub fn rulebases() -> Value {
    json!({
        "rulebases": [
            { "name": "druglikeness", "source": "trm_uefi_kernel::reasoning_engine::Engine::druglikeness_rules",
              "rules": Engine::druglikeness_rules().len(),
              "note": "Lipinski/Veber/Ghose thresholds as propositional rules; 'two or more violations' is written as explicit pairwise conjunctions because the engine cannot count" },
            { "name": "druglikeness_strict", "source": "Engine::strict_druglikeness_rules",
              "rules": Engine::strict_druglikeness_rules().len(),
              "note": "tighter profile (mw>450, logp>4, hbd>4, hba>8) so that real compounds exercise the TRUE path; thresholds are stated, not tuned after the fact" },
            { "name": "custom", "note": "supply \"rules\": [{id,name,premises[],negatives[],conclusion,exception}]" }
        ],
        "semantics": {
            "truth": "Kleene K3: TRUE / UNKNOWN / FALSE (i8 +1 / 0 / -1)",
            "firing": "a rule fires only when every premise is TRUE, every negative FALSE, and the exception is not TRUE",
            "exception_unknown": "the default applies and the assumption is named in ForwardResult.assumptions",
            "exception_true": "the rule is defeated and the conclusion withdrawn",
            "conflict": "an atom derived TRUE and asserted FALSE becomes UNKNOWN and is reported in conflicts",
            "no_negation_as_failure": true,
            "no_variables": "propositional only: atoms are opaque strings",
            "no_counting": "arity > 1 must be written as explicit pairwise rules"
        },
        "limits": { "max_iterations": trm_uefi_kernel::reasoning_engine::MAX_ITERATIONS,
                    "max_proof_depth": trm_uefi_kernel::reasoning_engine::MAX_PROOF_DEPTH }
    })
}

/// Build an engine from a request body, or explain why the body is unusable.
pub fn build_engine(body: &Value) -> Result<Engine, String> {
    let mut e = Engine::new();

    match body.get("rulebase").and_then(|v| v.as_str()).unwrap_or("none") {
        "druglikeness" => {
            for r in Engine::druglikeness_rules() {
                e.add_rule(r);
            }
        }
        "druglikeness_strict" => {
            for r in Engine::strict_druglikeness_rules() {
                e.add_rule(r);
            }
        }
        "none" => {}
        other => return Err(format!(
            "unknown rulebase '{other}' (use 'druglikeness', 'druglikeness_strict', or 'none' with explicit rules)"
        )),
    }

    if let Some(rules) = body.get("rules") {
        let arr = rules.as_array().ok_or("'rules' must be an array")?;
        for (i, r) in arr.iter().enumerate() {
            let conclusion = r
                .get("conclusion")
                .and_then(|v| v.as_str())
                .ok_or_else(|| format!("rule[{i}] has no 'conclusion' string"))?;
            let premises: Vec<String> = r
                .get("premises")
                .and_then(|v| v.as_array())
                .map(|a| a.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect())
                .ok_or_else(|| format!("rule[{i}] ('{conclusion}') has no 'premises' array"))?;
            let mut rule = Rule {
                id: r.get("id").and_then(|v| v.as_u64()).unwrap_or(i as u64) as u32,
                name: r.get("name").and_then(|v| v.as_str()).unwrap_or("custom").to_string(),
                premises,
                negatives: r
                    .get("negatives")
                    .and_then(|v| v.as_array())
                    .map(|a| a.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect())
                    .unwrap_or_default(),
                conclusion: conclusion.to_string(),
                exception: r.get("exception").and_then(|v| v.as_str()).map(|s| s.to_string()),
            };
            if rule.premises.is_empty() && rule.negatives.is_empty() {
                return Err(format!("rule[{i}] ('{conclusion}') has no premises: an unconditional rule is a fact, assert it"));
            }
            rule.name = rule.name; // keep clippy quiet about the mut binding
            e.add_rule(rule);
        }
    }

    if let Some(facts) = body.get("facts") {
        let arr = facts.as_array().ok_or("'facts' must be an array")?;
        for (i, f) in arr.iter().enumerate() {
            let atom = f
                .get("atom")
                .and_then(|v| v.as_str())
                .ok_or_else(|| format!("fact[{i}] has no 'atom' string"))?;
            let source = f.get("source").and_then(|v| v.as_str()).unwrap_or("asserted without provenance");
            e.assert_fact(atom, source);
        }
    }

    if let Some(falses) = body.get("false_facts") {
        let arr = falses.as_array().ok_or("'false_facts' must be an array of atoms")?;
        for f in arr {
            let atom = f.as_str().ok_or("'false_facts' entries must be strings")?;
            e.assert_false(atom);
        }
    }

    if let Some(mi) = body.get("max_iterations").and_then(|v| v.as_u64()) {
        e.max_iterations = Some(mi as usize);
    }
    Ok(e)
}

pub fn forward(body: &Value) -> Result<Value, String> {
    let e = build_engine(body)?;
    let r = e.forward_chain();
    let truth: Value = r
        .truth
        .iter()
        .map(|(k, v)| (k.clone(), json!(v.as_str())))
        .collect::<serde_json::Map<_, _>>()
        .into();
    let derivations: Value = r
        .derivations
        .iter()
        .map(|(k, d)| (k.clone(), json!(derivation_json(d))))
        .collect::<serde_json::Map<_, _>>()
        .into();
    Ok(json!({
        "truth": truth,
        "derivations": derivations,
        "assumptions": r.assumptions,
        "conflicts": r.conflicts.iter().map(|c| json!({"atom": c.atom, "positive_rules": c.positive_rules, "negative_rules": c.negative_rules})).collect::<Vec<_>>(),
        "iterations": r.iterations,
        "rules_fired": r.rules_fired,
        "truncated": r.truncated,
        "semantics": "Kleene K3; a rule fires only when every premise is TRUE and the exception is not TRUE"
    }))
}

pub fn prove(body: &Value) -> Result<Value, String> {
    let goal = body
        .get("goal")
        .and_then(|v| v.as_str())
        .ok_or("'goal' must be a string atom")?;
    let e = build_engine(body)?;
    let d = e.prove(goal);
    Ok(json!({
        "goal": goal,
        "truth": d.trit.as_str(),
        // the tree is emitted as an embedded JSON document (string) so it survives the kernel's hand-built
        // serialiser intact; the client parses it with the rest of the response
        "derivation": derivation_json(&d),
        "note": d.note,
        "assumed": !d.note.is_empty() && d.note.contains("assumed"),
        "semantics": "Kleene K3, goal-directed; UNKNOWN always names the premise or bound that stopped it"
    }))
}
