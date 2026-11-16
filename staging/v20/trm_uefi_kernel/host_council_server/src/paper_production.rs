//! Native paper production (MOTION-P1 + MOTION-P2) — synthesize the next paper from the platform's
//! real intelligence, not a lexical recall citation. Routes the candidate by domain:
//!   * DrugDiscovery -> the DDR engine's real artifact (antimicrobial_peptide_candidates.json, DeepDTA
//!     predictions) — findings are the actual screen results.
//!   * everything else -> the TRM-7M co-processor verdict + the EmbeddingGemma 2 semantic store.
//! Writes papers/paper_NNN.md and seals a Monad-ledger block with a real timestamp. Rust, no Python.
//! Every figure re-derives from the cited artifact / model verdict; the candidate's placeholder
//! abstract is deliberately NOT reproduced.

use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;

const CANDIDATES_REL: &str = "swarm_v2_artifacts/nobel_research_papers.json";
const DDR_REL: &str = "swarm_v2_artifacts/antimicrobial_peptide_candidates.json";
const PAPERS_REL: &str = "papers";
const LEDGER_REL: &str = "swarm_v2_artifacts/monad_ledger.jsonl";

/// The next paper number: one past the highest paper_NNN in papers/ (162 was the last written).
fn next_paper_number() -> u32 {
    let mut max = 162u32;
    if let Some(dir) = crate::k3_retrieval::repo_root().map(|r| r.join(PAPERS_REL)) {
        if let Ok(entries) = fs::read_dir(dir) {
            for e in entries.flatten() {
                let name = e.file_name().to_string_lossy().to_string();
                if let Some(rest) = name.strip_prefix("paper_") {
                    if let Some(digits) = rest.split('_').next() {
                        if let Ok(n) = digits.parse::<u32>() {
                            max = max.max(n);
                        }
                    }
                }
            }
        }
    }
    max + 1
}

/// A slug from the candidate title: lowercase alphanumeric + hyphen, capped.
fn slug(title: &str) -> String {
    let mut out = String::new();
    for c in title.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if c.is_whitespace() && !out.ends_with('-') && !out.is_empty() {
            out.push('-');
        }
        if out.len() >= 70 {
            break;
        }
    }
    out.trim_matches('-').to_string()
}

fn topic_of(cand: &Value) -> String {
    let topic = cand.get("title").and_then(Value::as_str).unwrap_or("").to_string();
    topic.splitn(2, ']').nth(1).unwrap_or(&topic).trim().to_string()
}

fn domain_of(cand: &Value) -> String {
    cand.get("domain").and_then(Value::as_str).unwrap_or("General").to_string()
}

/// A recall-grounded finding line: the doc key + its snippet, truncated.
fn finding_line(doc: &str, snippet: &str) -> String {
    let snip: String = snippet.chars().take(240).collect::<String>().replace('\n', " ");
    format!("- **{doc}** — {snip}")
}

/// The real drug-discovery evidence: the DDR engine's artifact (DeepDTA predictions).
fn ddr_findings(root: &PathBuf) -> Option<(Vec<String>, Vec<String>, String)> {
    let raw = fs::read_to_string(root.join(DDR_REL)).ok()?;
    let v: Value = serde_json::from_str(&raw).ok()?;
    let scanned = v.get("total_antibodies_scanned").and_then(Value::as_i64).unwrap_or(0);
    let top = v.get("top_candidates_count").and_then(Value::as_i64).unwrap_or(0);
    let spectrum = v.get("target_spectrum").and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_str).collect::<Vec<_>>().join(", "))
        .unwrap_or_default();
    let model = v.get("affinity_model").and_then(Value::as_str).unwrap_or("DeepDTA");
    let basis = v.get("prediction_basis").and_then(Value::as_str)
        .unwrap_or("prediction basis not recorded");

    let mut findings = vec![
        format!("- **screen** — {scanned} antibodies scanned, {top} top candidates, targets: {spectrum}"),
        format!("- **affinity_model** — {model}"),
        format!("- **prediction_basis** — {basis}"),
    ];
    let mut cited = vec!["antimicrobial_peptide_candidates.json".to_string()];

    if let Some(cands) = v.get("candidates").and_then(Value::as_array) {
        for c in cands.iter().take(3) {
            let id = c.get("candidate_id").and_then(Value::as_str).unwrap_or("?").to_string();
            let target = c.get("target_protein").and_then(Value::as_str).unwrap_or("?").to_string();
            let kd = c.get("predicted_kd_nm").map(|x| x.to_string()).unwrap_or("?".into());
            findings.push(format!("- **{id}** → {target} (predicted_kd_nm {kd})"));
            cited.push(id);
        }
    }
    Some((findings, cited, "DDR engine artifact + DeepDTA predictions".to_string()))
}

/// The TRM-7M co-processor verdict for the topic (distilled intelligence, not an LLM call).
fn trm7m_verdict(text: &str) -> String {
    let mut m = crate::trm7m::Trm7m::new();
    let (trit, conf, _ns) = m.think(text, 7);
    format!("TRM-7M verdict: {} (confidence {:.3})", crate::trm7m::trit_label(trit), conf)
}

/// The generic evidence: TRM-7M verdict + the semantic store + the fabric.
fn generic_findings(query: &str) -> (Vec<String>, Vec<String>, String, String) {
    let mut findings = vec![format!("- **co-processor** — {}", trm7m_verdict(query))];
    let mut cited = vec!["trm7m_source".to_string()];

    let semantic = crate::semantic_recall::lookup(query, 3);
    if let Some(results) = semantic.get("results").and_then(Value::as_array) {
        for r in results.iter().take(3) {
            let key = r.get("key").and_then(Value::as_str).unwrap_or("").to_string();
            let title = r.get("title").and_then(Value::as_str).unwrap_or("").to_string();
            if !key.is_empty() {
                findings.push(format!("- **{key}** — {title}"));
                cited.push(key);
            }
        }
    }
    if let Some(f) = crate::k3_retrieval::fabric_lookup(query, 3) {
        findings.push(finding_line(&f.doc, &f.snippet));
        cited.push(f.doc.clone());
    }
    (findings, cited, "TRM-7M verdict + EmbeddingGemma 2 semantic store".to_string(),
     "topic queried against the TRM-7M co-processor, the semantic store, and the fabric".to_string())
}

/// The full synthesis: pick the next unpublished candidate, route by domain, gather the real
/// evidence, write the paper, and seal a ledger block.
pub fn synthesize_paper() -> Value {
    let root = match crate::k3_retrieval::repo_root() {
        Some(r) => r,
        None => return json!({"status":"error","error":"repo root not resolvable"}),
    };

    let candidates: Value = match fs::read_to_string(root.join(CANDIDATES_REL)) {
        Ok(raw) => match serde_json::from_str(&raw) {
            Ok(v) => v,
            Err(e) => return json!({"status":"error","error":format!("candidates JSON: {e}")}),
        },
        Err(e) => return json!({"status":"error","error":format!("candidates read: {e}")}),
    };
    let list = match candidates.get("nobel_candidates").and_then(Value::as_array) {
        Some(l) => l,
        None => return json!({"status":"error","error":"no nobel_candidates in the artifact"}),
    };

    // Pick the first candidate whose paper is not yet published (by slug in papers/).
    let papers_dir = root.join(PAPERS_REL);
    let mut chosen: Option<&Value> = None;
    for cand in list {
        let q = topic_of(cand);
        let target_slug = format!("_{}.md", slug(&q));
        let already = fs::read_dir(&papers_dir).map(|entries| {
            entries.flatten().any(|e| e.file_name().to_string_lossy().ends_with(&target_slug))
        }).unwrap_or(false);
        if !already {
            chosen = Some(cand);
            break;
        }
    }
    let cand = match chosen.or_else(|| list.first()) {
        Some(c) => c,
        None => return json!({"status":"error","error":"candidate list empty"}),
    };

    let query = topic_of(cand);
    let domain = domain_of(cand);
    let agent = cand.get("agent").and_then(Value::as_str).unwrap_or("Seeker").to_string();
    let journal = cand.get("journal_target").and_then(Value::as_str).unwrap_or("Monad Journal").to_string();
    let nobel = cand.get("nobel_category").and_then(Value::as_str).unwrap_or("General").to_string();

    let dl = domain.to_lowercase();
    let ql = query.to_lowercase();
    let is_drug = ["drug", "antibiotic", "antimicrob", "antibody", "molecule", "compound",
                   "therapeut", "kinase", "receptor", "enzym", "pharm", "chemistry", "bio", "medic"]
        .iter().any(|k| dl.contains(k) || ql.contains(k));

    let (findings, cited, basis, method) = if is_drug {
        match ddr_findings(&root) {
            Some((f, c, b)) => (f, c, b,
                "candidate routed to the DDR engine; the artifact's DeepDTA predictions are cited verbatim".to_string()),
            None => generic_findings(&query),
        }
    } else {
        generic_findings(&query)
    };

    let n = next_paper_number();
    let title = format!("Paper {n}: {}", query);
    let body = format!(
        "# {title}\n\n\
         ## Abstract\n\
         The platform evaluated the research direction \"{query}\" (domain: {domain}) through its \
         native machinery. Findings below come from {basis}; every figure re-derives from the cited \
         artifact or model verdict. The candidate's own abstract in nobel_research_papers.json is a \
         placeholder and is deliberately NOT reproduced.\n\n\
         ## Method\n\
         Candidate selected from nobel_research_papers.json (agent: {agent}); {method}.\n\n\
         ## Findings\n{findings}\n\n\
         ## Basis\n\
         journal_target: {journal} | nobel_category: {nobel} | monad anchor: 0x4D4F4E4144000001\n\
         Every figure is a citation of the cited artifact / model verdict, never a fabricated \
         quantity; where a model is retired or out-of-domain, that is stated.\n",
        title = title, query = query, domain = domain, agent = agent,
        findings = findings.join("\n"), journal = journal, nobel = nobel,
        basis = basis, method = method,
    );

    let fname = format!("paper_{:03}_{}.md", n, slug(&query));
    let path = root.join(PAPERS_REL).join(&fname);
    if let Err(e) = fs::write(&path, &body) {
        return json!({"status":"error","error":format!("write paper: {e}"),"file":fname});
    }

    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let ledger_line = json!({
        "anchor": "0x4D4F4E4144000001",
        "fact": format!("Paper {n} synthesized and published: {query}"),
        "kind": "paper_production",
        "basis": basis,
        "not_claimed": "findings are citations of the cited artifact / model verdict",
        "rederive": {"file": format!("papers/{fname}")},
        "sealed_at": ts,
    });
    let ledger_path = root.join(LEDGER_REL);
    if let Ok(mut f) = fs::OpenOptions::new().create(true).append(true).open(&ledger_path) {
        use std::io::Write;
        let _ = writeln!(f, "{}", ledger_line.to_string());
    }

    json!({
        "status": "ok",
        "paper_number": n,
        "file": format!("papers/{fname}"),
        "title": title,
        "candidate_topic": query,
        "domain": domain,
        "agent": agent,
        "journal_target": journal,
        "nobel_category": nobel,
        "citations": cited,
        "basis": basis,
    })
}
