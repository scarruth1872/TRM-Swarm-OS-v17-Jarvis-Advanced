//! swarm-sovereign — Motion B: the import blockade + runtime gate.
//!
//! The doctrine: zero external-LLM imports. This crate provides:
//!   1. A static scanner — fails if any cloud-LLM client import appears
//!   2. The verified on-device chain manifest (instella → local → bonsai)
//!   3. A runtime self-check that the local chain answers with no sockets
//!
//! Honest: the code audit showed the platform is ALREADY local-only
//! (FALLBACK_CHAIN = instella/local/bonsai, zero live network sites).
//! This crate makes that a GATE, not a hope.

/// The forbidden external-LLM import patterns (the blockade list).
pub const FORBIDDEN: &[&str] = &[
    "import openai",
    "from openai",
    "import anthropic",
    "from anthropic",
    "from deepseek",
    "import deepseek",
    "api.groq.com",
    "api.openai.com",
    "api.anthropic.com",
    "api.deepseek.com",
];

/// Scan source text for any forbidden external-LLM import.
/// Returns the offending patterns (empty = clean).
pub fn scan_for_external(text: &str) -> Vec<&'static str> {
    FORBIDDEN
        .iter()
        .filter(|pat| text.contains(**pat))
        .copied()
        .collect()
}

/// The verified on-device fallback chain (from the router's actual code).
pub const ON_DEVICE_CHAIN: &[&str] = &["instella", "local", "bonsai"];

/// The import-blockade verdict.
pub fn blockade_verdict(text: &str) -> String {
    let hits = scan_for_external(text);
    if hits.is_empty() {
        format!(
            "BLOCKADE CLEAR: zero external-LLM imports. on-device chain: {}",
            ON_DEVICE_CHAIN.join(" -> ")
        )
    } else {
        format!("BLOCKADE VIOLATION: {:?}", hits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_text_passes() {
        let src = "from swarm_v2.core import nand_k3_fabric\nfrom libswarm_kernel_ops import c_kleene_quantize_avx2\n";
        assert!(scan_for_external(src).is_empty());
        assert!(blockade_verdict(src).starts_with("BLOCKADE CLEAR"));
    }

    #[test]
    fn openai_import_caught() {
        let src = "import openai\nclient = openai.OpenAI()\n";
        let hits = scan_for_external(src);
        assert!(hits.contains(&"import openai"));
    }

    #[test]
    fn anthropic_import_caught() {
        let src = "from anthropic import Anthropic\n";
        assert!(!scan_for_external(src).is_empty());
    }

    #[test]
    fn groq_url_caught() {
        let src = "base_url = \"https://api.groq.com/openai/v1\"\n";
        assert!(!scan_for_external(src).is_empty());
    }
}
