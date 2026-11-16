//! The kernel's model-free text encoder, in-tree.
//!
//! `native-turbovec-hdc` built the fabric's document vectors, and a query encoded by anything else
//! lives in a different space. That encoder was reachable only through `swarm_rust_kernel.dll`, and
//! the DLL load is `#[cfg(windows)]` -- so the bare-metal (musl) build could not encode a query at
//! all, and every `/k3/retrieve` answered 503 `engine_required` on metal.
//!
//! Ported from `swarm_v2_rust_kernel/src/native_query_encoder.rs` (Paper 150), algorithm and
//! constants unchanged. It is not a second encoder with its own opinion: `encoder_parity` (bottom of
//! this file) diffs this port against the DLL's own export, and it may stand in for the DLL only
//! where that diff is zero.

/// "MONAD\0\0\x01" in 64-bit ASCII hex -- the same salt the DLL's encoder uses.
pub const MONAD_TRUTH_ANCHOR: u64 = 0x4D4F_4E41_4400_0001;

/// FNV-1a 64 over the Monad-salted bytes.
pub fn fnv1a_64(data: &[u8], salt: u64) -> u64 {
    let mut hash = 0xcbf29ce484222325u64 ^ salt;
    for &byte in data {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3u64);
    }
    hash
}

/// SplitMix64 -- the deterministic pseudo-orthogonal hyperplane expansion.
pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }
}

/// Word tokens plus boundary markers plus character 3/4/5-grams, each weighted.
pub fn extract_features(text: &str) -> Vec<(String, f32)> {
    let clean = text.to_lowercase();
    let mut features: Vec<(String, f32)> = Vec::with_capacity(text.len() / 2);
    let mut word = String::with_capacity(32);
    for c in clean.chars() {
        if c.is_alphanumeric() || c == '_' || c == '-' {
            word.push(c);
        } else if !word.is_empty() {
            process_word(&word, &mut features);
            word.clear();
        }
    }
    if !word.is_empty() {
        process_word(&word, &mut features);
    }
    features
}

fn process_word(word: &str, features: &mut Vec<(String, f32)>) {
    if word.is_empty() {
        return;
    }
    // Full word token (high salience), the word enclosed by boundary markers, then subword n-grams
    // for morphological and typo robustness -- weights exactly as the corpus builder weighed them.
    features.push((format!("W:{word}"), 2.0f32));
    let chars: Vec<char> = word.chars().collect();
    let len = chars.len();
    features.push((format!("B:<{word}>"), 1.5f32));
    for n in 3..=5 {
        if len >= n {
            for i in 0..=(len - n) {
                let gram: String = chars[i..(i + n)].iter().collect();
                features.push((format!("N{n}:{gram}"), 1.2f32));
            }
        }
    }
}

/// Sublinear term-frequency superposition into R^dim, then L2 unit normalization.
///
/// Returns all zeros for text with no features (`""`, whitespace, punctuation only) -- the corpus
/// builder's behaviour, and the reason a zero query is a valid input rather than an error.
pub fn encode_f32(text: &str, dim: usize) -> Vec<f32> {
    let mut acc = vec![0.0f32; dim];
    if dim == 0 || text.trim().is_empty() {
        return acc;
    }
    let features = extract_features(text);
    if features.is_empty() {
        return acc;
    }
    let blocks = (dim + 63) / 64;
    for (feat, weight) in &features {
        let mut prng = SplitMix64::new(fnv1a_64(feat.as_bytes(), MONAD_TRUTH_ANCHOR));
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

/// Cosine similarity, spelled the way the encoder's own module spells it (f32 accumulation).
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
    if denom > 1e-12 {
        dot / denom
    } else {
        0.0
    }
}

/// The port is allowed to stand in for the DLL only where it REPRODUCES it. This diffs the two
/// encoders, element by element, over the corpus builder's own test texts plus the shapes that
/// break tokenizers -- so "same encoder" is a measurement taken on this machine, not a reading of
/// two files that look alike.
#[cfg(all(test, windows))]
mod encoder_parity {
    use super::encode_f32;
    use std::ffi::c_void;

    #[link(name = "kernel32")]
    extern "system" {
        fn LoadLibraryA(name: *const u8) -> *mut c_void;
        fn GetProcAddress(module: *mut c_void, name: *const u8) -> *mut c_void;
    }

    type EncodeFn = unsafe extern "C" fn(*const u8, usize, *mut f32, usize) -> i32;

    const DIM: usize = 768;

    fn dll_encode(text: &str) -> Option<Vec<f32>> {
        // Anchored on the CRATE, not on where the test binary happens to live in: a test built
        // into an out-of-tree target directory would otherwise fail to find the oracle and read
        // as a parity failure when the port is perfectly faithful.
        let rel = "swarm_v2_rust_kernel/target/release/swarm_rust_kernel.dll";
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join(rel);
        let name = std::ffi::CString::new(path.to_string_lossy().as_bytes()).ok()?;
        unsafe {
            let h = LoadLibraryA(name.as_ptr() as *const u8);
            if h.is_null() {
                return None;
            }
            let sym = std::ffi::CString::new("rust_native_encode_text_f32").ok()?;
            let p = GetProcAddress(h, sym.as_ptr() as *const u8);
            if p.is_null() {
                return None;
            }
            let f: EncodeFn = std::mem::transmute(p);
            let mut out = vec![0f32; DIM];
            let written = f(text.as_ptr(), text.len(), out.as_mut_ptr(), DIM);
            if written == DIM as i32 {
                Some(out)
            } else {
                None
            }
        }
    }

    const TEXTS: &[&str] = &[
        "LpxC zinc-dependent deacetylase inhibitor TRM-ABX-082",
        "bacterial LpxC lipid A biosynthesis enzyme",
        "LpxC deacetylase enzyme catalytic domain in Gram-negative bacteria",
        "high-frequency ionosphere radio propagation and solar flare dynamics",
        "Kleene K3 balanced ternary logic consensus engine",
        "pharmacophore coordination geometry",
        "pharmacophore coordinaton geometrey",
        "",
        "   ",
        "!!!",
        "a",
        "xyz",
        "UPPER lower MixedCase W:W B:<b>",
        "unicode cafe\u{301} na\u{ef}ve \u{2014} \u{65e5}\u{672c}\u{8a9e}",
        "hyphen-ated under_score 12345 0.5e-3",
        "tab\tseparated\nnewlines\rand\r\nmore",
        "N3:abc N4:abcd N5:abcde W:x B:<x>",
        "the quick brown fox jumps over the lazy dog and then does it again with more words to push the feature count up so the superposition and the normalisation are exercised across every one of the twelve 64-wide blocks that a 768-dimension vector has",
    ];

    #[test]
    fn port_reproduces_the_dll_export() {
        let mut worst = 0f64;
        let mut diverging: Vec<&str> = Vec::new();
        for t in TEXTS {
            let dll = dll_encode(t)
                .unwrap_or_else(|| panic!("engine unavailable or export missing, for {t:?}"));
            let port = encode_f32(t, DIM);
            assert_eq!(port.len(), DIM);
            let mut local = 0f64;
            for i in 0..DIM {
                assert!(port[i].is_finite(), "port wrote a non-finite value for {t:?}");
                local = local.max((port[i] as f64 - dll[i] as f64).abs());
            }
            if local != 0.0 {
                diverging.push(t);
            }
            worst = worst.max(local);
        }
        assert!(
            diverging.is_empty(),
            "port differs from the DLL export on {} of {} texts (max |diff| = {worst}); \
             it may not stand in for the engine",
            diverging.len(),
            TEXTS.len()
        );
    }
}
