//! TRM Swarm OS — Jarvis Substrate Agent Platform (no_std port).
//! 1:1 with Jarvis-Advanced-main/substrate_agent_platform.py:
//! AlphaFold fold simulation (helix/sheet estimation, pseudo-pLDDT),
//! AlphaGenome pathogenicity classification (deterministic string hash),
//! semantic-graph query matching.

extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

/// AlphaFold 3 fold simulation on a protein sequence.
/// Returns (helices, sheets, plddt, active_residues).
pub fn run_alphafold_simulation(sequence: &str) -> (u64, u64, f64, String) {
    let seq_len = sequence.len() as u64;
    let estimated_helices = (seq_len / 40).max(1);
    let estimated_sheets = (seq_len / 50).max(1);

    // pseudo-pLDDT: 85.0 + (len % 15) * 0.8, capped at 98.0
    let mut plddt = 85.0 + (seq_len % 15) as f64 * 0.8;
    if plddt > 98.0 {
        plddt = 98.0;
    }

    // active residues from H/C/D positions
    let mut residues: Vec<String> = Vec::new();
    if let Some(pos) = sequence.find('H') {
        residues.push(format!("His{}", pos + 1));
    }
    if let Some(pos) = sequence.find('C') {
        residues.push(format!("Cys{}", pos + 1));
    }
    if let Some(pos) = sequence.find('D') {
        residues.push(format!("Asp{}", pos + 1));
    }
    if residues.is_empty() {
        residues.push("Val12".into());
        residues.push("Arg45".into());
    }
    (estimated_helices, estimated_sheets, plddt, residues.join(", "))
}

/// AlphaGenome pathogenicity classification.
/// Deterministic: hash(gene+variant) % 100 / 100. Matches run_alphagenome_classification.
pub fn run_alphagenome_classification(gene: &str, variant: &str) -> f64 {
    let hash_val: u64 = gene.bytes().chain(variant.bytes()).map(|b| b as u64).sum();
    (hash_val % 100) as f64 / 100.0
}

/// Semantic graph query: keyword match over relation triples.
/// Returns up to 5 matching "source --[label]--> target" lines.
pub fn query_semantic_graph(query_term: &str, relations: &[(&str, &str, &str)]) -> Vec<String> {
    let term_lower = query_term.to_lowercase();
    let mut matches: Vec<String> = Vec::new();
    for (src, tgt, lbl) in relations {
        if term_lower.contains(&src.to_lowercase())
            || term_lower.contains(&tgt.to_lowercase())
            || term_lower.contains(&lbl.to_lowercase())
        {
            matches.push(format!("{} --[{}]--> {}", src, lbl, tgt));
        }
        if matches.len() >= 5 {
            break;
        }
    }
    matches
}
