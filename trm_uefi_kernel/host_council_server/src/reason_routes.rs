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
use trm_uefi_kernel::case_base::CaseBase;
use trm_uefi_kernel::spatial_reasoning::{contact_graph, near_set, Atom, BoundingBox, Point};
use trm_uefi_kernel::temporal_reasoning::{compose, relation, Allen, Interval};

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


// ---------------------------------------------------------------------------------------------------
// Task 577 — temporal. Three modes over the same primitive.
// ---------------------------------------------------------------------------------------------------

fn iv_from(v: &Value) -> Result<Interval, String> {
    let arr = v.as_array().ok_or("interval must be [start, end]")?;
    if arr.len() != 2 {
        return Err("interval must have exactly two elements [start, end]".to_string());
    }
    let s = arr[0].as_i64().ok_or("interval start must be an integer (seconds since epoch)")?;
    let e = arr[1].as_i64().ok_or("interval end must be an integer")?;
    Interval::try_new(s, e).ok_or_else(|| format!("interval [{s}, {e}) is reversed or empty"))
}

pub fn temporal(body: &Value) -> Result<Value, String> {
    match body.get("mode").and_then(|v| v.as_str()).unwrap_or("relation") {
        "relation" => {
            let pairs = body.get("pairs").and_then(|v| v.as_array()).ok_or("'pairs' must be an array of [a, b] interval pairs")?;
            let mut out = Vec::new();
            for (i, pr) in pairs.iter().enumerate() {
                let arr = pr.as_array().ok_or_else(|| format!("pair[{i}] must be [[s,e],[s,e]]"))?;
                if arr.len() != 2 {
                    return Err(format!("pair[{i}] must have exactly two intervals"));
                }
                let a = iv_from(&arr[0])?;
                let b = iv_from(&arr[1])?;
                out.push(json!({
                    "relation": relation(&a, &b).as_str(),
                    "inverse": relation(&a, &b).inverse().as_str(),
                    "a_duration": a.duration(),
                    "b_duration": b.duration(),
                }));
            }
            Ok(json!({ "relations": out, "semantics": "half-open [start, end); subject is the first interval" }))
        }
        "compose" => {
            let pairs = body.get("pairs").and_then(|v| v.as_array()).ok_or("'pairs' must be an array of [r1, r2] relation-name pairs")?;
            let mut out = Vec::new();
            for (i, pr) in pairs.iter().enumerate() {
                let arr = pr.as_array().ok_or_else(|| format!("pair[{i}] must be [\"BEFORE\", \"MEETS\"]"))?;
                if arr.len() != 2 {
                    return Err(format!("pair[{i}] must have two relation names"));
                }
                let r1 = arr[0].as_str().and_then(Allen::from_str)
                    .ok_or_else(|| format!("pair[{i}] has an unknown relation '{}'", arr[0]))?;
                let r2 = arr[1].as_str().and_then(Allen::from_str)
                    .ok_or_else(|| format!("pair[{i}] has an unknown relation '{}'", arr[1]))?;
                let set: Vec<&str> = compose(r1, r2).iter().map(|a| a.as_str()).collect();
                out.push(json!({ "r1": r1.as_str(), "r2": r2.as_str(), "possible": set }));
            }
            Ok(json!({ "compositions": out, "method": "witness intervals on a bounded integer grid, not a transcribed table" }))
        }
        "verify" => {
            // Soundness check on real data: for each triple, the derived relation must be one the algebra
            // allows. A composition table that is too small is caught here by the data itself.
            let triples = body.get("triples").and_then(|v| v.as_array()).ok_or("'triples' must be an array of [[s,e],[s,e],[s,e]]")?;
            let mut ok = 0;
            let mut failures = Vec::new();
            for (i, t) in triples.iter().enumerate() {
                let arr = t.as_array().ok_or_else(|| format!("triple[{i}] malformed"))?;
                if arr.len() != 3 {
                    return Err(format!("triple[{i}] must have three intervals"));
                }
                let a = iv_from(&arr[0])?;
                let b = iv_from(&arr[1])?;
                let c = iv_from(&arr[2])?;
                let r1 = relation(&a, &b);
                let r2 = relation(&b, &c);
                let rac = relation(&a, &c);
                let allowed = compose(r1, r2);
                if allowed.contains(&rac) {
                    ok += 1;
                } else {
                    failures.push(json!({
                        "index": i, "r1": r1.as_str(), "r2": r2.as_str(), "observed": rac.as_str(),
                        "allowed": allowed.iter().map(|x| x.as_str()).collect::<Vec<_>>()
                    }));
                }
            }
            Ok(json!({
                "checked": triples.len(), "ok": ok, "failures": failures.len(),
                "detail": failures,
                "claim": "every triple of real intervals must satisfy relation(A,C) in compose(relation(A,B), relation(B,C))"
            }))
        }
        other => Err(format!("unknown temporal mode '{other}' (relation | compose | verify)")),
    }
}

// ---------------------------------------------------------------------------------------------------
// Task 578 — spatial. Contact graph and proximity queries over caller-supplied coordinates.
// ---------------------------------------------------------------------------------------------------

pub fn spatial(body: &Value) -> Result<Value, String> {
    let raw = body.get("atoms").and_then(|v| v.as_array()).ok_or("'atoms' must be an array")?;
    let mut atoms: Vec<Atom> = Vec::with_capacity(raw.len());
    for (i, a) in raw.iter().enumerate() {
        let x = a.get("x").and_then(|v| v.as_f64()).ok_or_else(|| format!("atom[{i}] has no numeric x"))?;
        let y = a.get("y").and_then(|v| v.as_f64()).ok_or_else(|| format!("atom[{i}] has no numeric y"))?;
        let z = a.get("z").and_then(|v| v.as_f64()).ok_or_else(|| format!("atom[{i}] has no numeric z"))?;
        atoms.push(Atom {
            label: a.get("label").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            position: Point::new(x, y, z),
            element: a.get("element").and_then(|v| v.as_str()).unwrap_or("C").to_string(),
        });
    }

    let cutoff = body.get("cutoff").and_then(|v| v.as_f64()).unwrap_or(4.0);
    if !(cutoff > 0.0) {
        return Err("'cutoff' must be positive".to_string());
    }
    let include_h = body.get("include_hydrogens").and_then(|v| v.as_bool()).unwrap_or(false);
    let contacts = contact_graph(&atoms, cutoff, include_h);

    // an integer checksum over the pair list, so an independent implementation can compare exactly without
    // shipping megabytes back
    let checksum: u64 = contacts.iter().map(|c| (c.a as u64) * 1_000_003 + c.b as u64).sum();

    let bbox = BoundingBox::of(&atoms).map(|b| {
        let c = b.centre();
        let s = b.size();
        json!({ "centre": [c.x, c.y, c.z], "size": [s.x, s.y, s.z] })
    });

    let mut out = json!({
        "atom_count": atoms.len(),
        "cutoff": cutoff,
        "include_hydrogens": include_h,
        "contact_count": contacts.len(),
        "pair_checksum": checksum,
        "bounding_box": bbox,
        "semantics": "contact = squared distance <= cutoff^2; boundary included; checksum = sum(a*1000003 + b) over sorted pairs"
    });

    if let Some(reference) = body.get("reference").and_then(|v| v.as_array()) {
        let refs: Vec<usize> = reference.iter().filter_map(|v| v.as_u64().map(|x| x as usize)).collect();
        let radius = body.get("radius").and_then(|v| v.as_f64()).unwrap_or(cutoff);
        let near = near_set(&atoms, &refs, radius);
        if let Some(obj) = out.as_object_mut() {
            obj.insert("reference_count".into(), json!(refs.len()));
            obj.insert("radius".into(), json!(radius));
            obj.insert("near_count".into(), json!(near.len()));
            obj.insert("near_indices".into(), json!(near));
        }
    }
    Ok(out)
}

// ---------------------------------------------------------------------------------------------------
// Task 573 — case-based reasoning. Leave-one-out or a single query.
// ---------------------------------------------------------------------------------------------------

pub fn case_base(body: &Value) -> Result<Value, String> {
    let cases = body.get("cases").and_then(|v| v.as_array()).ok_or("'cases' must be an array")?;
    let k = body.get("k").and_then(|v| v.as_u64()).unwrap_or(5) as usize;
    if cases.is_empty() {
        return Err("'cases' is empty: a case base with no cases cannot reason about anything".to_string());
    }
    let first = cases[0].get("features").and_then(|v| v.as_array())
        .ok_or("case[0] has no 'features' array")?;
    let mut cb = CaseBase::new(first.len());
    for (i, c) in cases.iter().enumerate() {
        let id = c.get("id").and_then(|v| v.as_str()).ok_or_else(|| format!("case[{i}] has no 'id'"))?;
        let feats: Vec<f64> = c.get("features").and_then(|v| v.as_array())
            .ok_or_else(|| format!("case[{i}] has no 'features' array"))?
            .iter().map(|x| x.as_f64().unwrap_or(f64::NAN)).collect();
        if feats.iter().any(|f| !f.is_finite()) {
            return Err(format!("case[{i}] ('{id}') has a non-numeric or NaN feature"));
        }
        let outcome = c.get("outcome").and_then(|v| v.as_f64())
            .ok_or_else(|| format!("case[{i}] ('{id}') has no numeric 'outcome'"))?;
        cb.add(id, &feats, outcome)?;
    }

    match body.get("mode").and_then(|v| v.as_str()).unwrap_or("loo") {
        "loo" => {
            let r = cb.leave_one_out(k).ok_or("leave-one-out needs at least 3 cases")?;
            Ok(json!({
                "mode": "leave_one_out",
                "n": r.n, "k": r.k,
                "rmse": r.rmse, "mae": r.mae, "r2": r.r2,
                "baseline_rmse": r.baseline_rmse, "baseline_r2": r.baseline_r2,
                "worse_than_baseline": r.worse_than_baseline,
                "phases": { "retrieve": true, "reuse": "distance-weighted interpolation", "revise": false, "retain": false }
            }))
        }
        "query" => {
            let q = body.get("query").and_then(|v| v.as_array())
                .ok_or("mode 'query' needs a 'query' array of features")?;
            let qv: Vec<f64> = q.iter().map(|x| x.as_f64().unwrap_or(f64::NAN)).collect();
            if qv.iter().any(|f| !f.is_finite()) || qv.len() != first.len() {
                return Err(format!("query must have {} finite features", first.len()));
            }
            let neighbours = cb.retrieve(&qv, k, None);
            let prediction = cb.predict(&qv, k);
            let (mean, std) = cb.stats();
            Ok(json!({
                "mode": "query",
                "prediction": prediction,
                "neighbours": neighbours.iter().map(|n| json!({
                    "id": n.id, "distance": n.distance, "outcome": n.outcome
                })).collect::<Vec<_>>(),
                "standardisation": { "mean": mean, "std": std },
                "phases": { "retrieve": true, "reuse": "distance-weighted interpolation", "revise": false, "retain": false }
            }))
        }
        other => Err(format!("unknown case mode '{other}' (loo | query)")),
    }
}
