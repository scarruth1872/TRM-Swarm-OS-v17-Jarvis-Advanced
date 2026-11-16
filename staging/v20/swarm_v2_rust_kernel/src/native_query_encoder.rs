//! TRM Swarm OS — Native Model-Free Query & Knowledge Encoder (Paper 150)
//!
//! Sub-Microsecond Deterministic Subword Hyperdimensional Random Projection
//! Zero LLM / Zero Neural Network / Zero External Process Dependency
//!
//! Monad Truth Anchor: 0x4D4F4E4144000001
//!
//! Mathematical Basis:
//! 1. Multi-resolution Subword Tokenization (Words + Character 3/4/5-grams)
//! 2. Deterministic Monad-Salted Universal Hashing (FNV-1a 64-bit)
//! 3. SplitMix64 Pseudo-Orthogonal Hyperplane Expansion into R^D
//! 4. Sublinear Term-Frequency Superposition & L2-Unit Hypersphere Normalization
//! 5. Direct Balanced Ternary K3 Quantization {-1, 0, +1} matching kernel indexing.

use crate::MONAD_TRUTH_ANCHOR;
use crate::k3_logic::{DEFAULT_TAU_UPPER, DEFAULT_TAU_LOWER, TRUE, FALSE, UNKNOWN};

/// Default embedding dimension matching TRM Swarm kernel index standards
pub const DEFAULT_EMBED_DIM: usize = 768;

/// Fast 64-bit FNV-1a hash with Monad seed salt
#[inline(always)]
pub fn fnv1a_64(data: &[u8], salt: u64) -> u64 {
    let mut hash = 0xcbf29ce484222325u64 ^ salt;
    for &byte in data {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3u64);
    }
    hash
}

/// Fast SplitMix64 pseudo-random generator for deterministic hyperplane expansion
pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    #[inline(always)]
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    #[inline(always)]
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }
}

/// Native Model-Free Query & Document Encoder
pub struct NativeQueryEncoder;

impl NativeQueryEncoder {
    /// Tokenizes input text into word tokens and subword character n-grams (3, 4, 5)
    pub fn extract_features(text: &str) -> Vec<(String, f32)> {
        let mut features: Vec<(String, f32)> = Vec::with_capacity(text.len() / 2);
        let clean = text.to_lowercase();
        
        let mut current_word = String::with_capacity(32);
        for c in clean.chars() {
            if c.is_alphanumeric() || c == '_' || c == '-' {
                current_word.push(c);
            } else if !current_word.is_empty() {
                Self::process_word(&current_word, &mut features);
                current_word.clear();
            }
        }
        if !current_word.is_empty() {
            Self::process_word(&current_word, &mut features);
        }

        features
    }

    #[inline]
    fn process_word(word: &str, features: &mut Vec<(String, f32)>) {
        if word.is_empty() {
            return;
        }

        // Full word token (high salience)
        features.push((format!("W:{}", word), 2.0f32));

        let chars: Vec<char> = word.chars().collect();
        let len = chars.len();

        // Enclosed word with boundary markers
        let enclosed: String = format!("<{}>", word);
        features.push((format!("B:{}", enclosed), 1.5f32));

        // Subword n-grams for morphological and typo robustness
        for n in 3..=5 {
            if len >= n {
                for i in 0..=(len - n) {
                    let gram: String = chars[i..(i + n)].iter().collect();
                    features.push((format!("N{}:{}", n, gram), 1.2f32));
                }
            }
        }
    }

    /// Encodes text into a normalized float32 vector in R^dim
    pub fn encode_f32(text: &str, dim: usize) -> Vec<f32> {
        let mut acc = vec![0.0f32; dim];
        if text.trim().is_empty() {
            return acc;
        }

        let features = Self::extract_features(text);
        if features.is_empty() {
            return acc;
        }

        let blocks = (dim + 63) / 64;

        for (feat, weight) in &features {
            let seed = fnv1a_64(feat.as_bytes(), MONAD_TRUTH_ANCHOR);
            let mut prng = SplitMix64::new(seed);

            for b in 0..blocks {
                let bits = prng.next_u64();
                let base = b * 64;
                let limit = 64.min(dim.saturating_sub(base));
                for i in 0..limit {
                    let sign = if (bits & (1u64 << i)) != 0 { 1.0f32 } else { -1.0f32 };
                    acc[base + i] += sign * weight;
                }
            }
        }

        // L2 normalize
        let mut sum_sq = 0.0f32;
        for &x in &acc {
            sum_sq += x * x;
        }

        if sum_sq > 1e-12 {
            let inv_norm = 1.0 / sum_sq.sqrt();
            for x in &mut acc {
                *x *= inv_norm;
            }
        }

        acc
    }

    /// Encodes text and directly quantizes into Balanced Ternary Trits {-1, 0, +1}
    /// rigorously aligned with the microkernel's K3 gate:
    /// Psi_i = +1 if S_i > tau_upper (+0.35); -1 if S_i < tau_lower (-0.35); else 0 (UNKNOWN).
    pub fn encode_ternary(text: &str, dim: usize, multiplier: f32) -> (Vec<i8>, f32) {
        let fvec = Self::encode_f32(text, dim);
        let absmean: f32 = fvec.iter().map(|x| x.abs()).sum::<f32>() / (dim as f32);
        let m = if multiplier > 0.0 { multiplier } else { 1.0 };
        let k3_scale = if absmean > 0.0 { DEFAULT_TAU_UPPER / (absmean * m) } else { 0.0 };

        let mut trits = vec![UNKNOWN; dim];
        if k3_scale > 0.0 {
            for (i, &v) in fvec.iter().enumerate() {
                let voltage_signal = v * k3_scale;
                trits[i] = if voltage_signal > DEFAULT_TAU_UPPER {
                    TRUE
                } else if voltage_signal < DEFAULT_TAU_LOWER {
                    FALSE
                } else {
                    UNKNOWN
                };
            }
        }

        (trits, absmean * m)
    }

    /// Computes cosine similarity between two float vectors
    pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() || a.is_empty() {
            return 0.0;
        }
        let mut dot = 0.0f32;
        let mut na = 0.0f32;
        let mut nb = 0.0f32;
        for i in 0..a.len() {
            dot += a[i] * b[i];
            na += a[i] * a[i];
            nb += b[i] * b[i];
        }
        let denom = (na * nb).sqrt();
        if denom > 1e-12 { dot / denom } else { 0.0 }
    }
}

// -----------------------------------------------------------------------------
// Unit Tests
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identical_text_cosine_is_one() {
        let text = "LpxC zinc-dependent deacetylase inhibitor TRM-ABX-082";
        let v1 = NativeQueryEncoder::encode_f32(text, DEFAULT_EMBED_DIM);
        let v2 = NativeQueryEncoder::encode_f32(text, DEFAULT_EMBED_DIM);
        let cos = NativeQueryEncoder::cosine_similarity(&v1, &v2);
        assert!((cos - 1.0).abs() < 1e-5, "Self-cosine must be 1.0, got {}", cos);
    }

    #[test]
    fn test_semantic_affinity_preservation() {
        let t_target = "bacterial LpxC lipid A biosynthesis enzyme";
        let t_similar = "LpxC deacetylase enzyme catalytic domain in Gram-negative bacteria";
        let t_unrelated = "high-frequency ionosphere radio propagation and solar flare dynamics";

        let v_target = NativeQueryEncoder::encode_f32(t_target, DEFAULT_EMBED_DIM);
        let v_similar = NativeQueryEncoder::encode_f32(t_similar, DEFAULT_EMBED_DIM);
        let v_unrelated = NativeQueryEncoder::encode_f32(t_unrelated, DEFAULT_EMBED_DIM);

        let cos_sim = NativeQueryEncoder::cosine_similarity(&v_target, &v_similar);
        let cos_unrelated = NativeQueryEncoder::cosine_similarity(&v_target, &v_unrelated);

        assert!(cos_sim > cos_unrelated,
            "Similar text must have higher cosine ({}) than unrelated ({})",
            cos_sim, cos_unrelated);
        assert!(cos_sim > 0.25, "Related text cosine must be substantial: {}", cos_sim);
        assert!(cos_unrelated < 0.20, "Unrelated text cosine must be low: {}", cos_unrelated);
    }

    #[test]
    fn test_subword_typo_resilience() {
        let t1 = "pharmacophore coordination geometry";
        let t2 = "pharmacophore coordinaton geometrey";

        let v1 = NativeQueryEncoder::encode_f32(t1, DEFAULT_EMBED_DIM);
        let v2 = NativeQueryEncoder::encode_f32(t2, DEFAULT_EMBED_DIM);
        let cos = NativeQueryEncoder::cosine_similarity(&v1, &v2);

        assert!(cos > 0.65, "Subword n-grams must preserve high similarity (>0.65) under typos, got {}", cos);
    }

    #[test]
    fn test_ternary_quantization_valid_trits() {
        let text = "Kleene K3 balanced ternary logic consensus engine";
        let (trits, scale) = NativeQueryEncoder::encode_ternary(text, DEFAULT_EMBED_DIM, 1.0);
        assert!(scale > 0.0);
        assert_eq!(trits.len(), DEFAULT_EMBED_DIM);
        for &t in &trits {
            assert!(t == -1 || t == 0 || t == 1, "Illegal trit value: {}", t);
        }
        let non_zeros = trits.iter().filter(|&&t| t != 0).count();
        assert!(non_zeros > 100, "Must have non-trivial non-zero trits, got {}", non_zeros);
    }
}
