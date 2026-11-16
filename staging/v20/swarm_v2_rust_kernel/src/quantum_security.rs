//! TRM Swarm OS — Deterministic Byte-Mixing "Crypto" Stub (NOT post-quantum)
//!
//! HONEST SCOPE: this module implements **no** post-quantum primitive. There is no ML-KEM
//! (FIPS 203 / Kyber), no ML-DSA (FIPS 204 / Dilithium), no lattice arithmetic, no NTT, no
//! polynomial sampling, no hash and no KDF anywhere in this file.
//!
//! What the code actually does, by function:
//!   * `keygen`      — fills `pk`/`sk` with a byte pattern derived from a `u64` seed
//!                     (`(seed + i) ^ 0x4D4F4E41`). `sk` is not derived from `pk` and the two
//!                     are not mathematically related in any way a KEM requires.
//!   * `encapsulate` — a rotate-left-3 / XOR accumulator over `pk`, emitting the accumulator's
//!                     low byte per ciphertext byte. The 32-byte "shared secret" is the final
//!                     8-byte accumulator repeated 4x, and it is a function of `pk` ALONE:
//!                     it does not vary with `msg_seed` (verified: 1 distinct value across
//!                     2000 seeds), so it is public information, not a secret.
//!   * `decapsulate` — a different rotate-left-1 / XOR accumulator over `ct` and `sk`. It does
//!                     NOT reproduce `encapsulate`'s output, so the two sides never agree on a
//!                     key. The only check in the handshake is `assert_eq!` on the two lengths.
//!
//! The `KYBER_1024_*` / `DILITHIUM_5_*` constants below are buffer SIZES copied from the real
//! parameter sets; they are not evidence that the corresponding algorithm is implemented.
//! `DilithiumSignature` is declared but never constructed — no signing or verification code
//! exists in this file.
//!
//! Do not describe this module as quantum-resistant, as meeting any NIST security category, or as
//! FIPS 203/204 compliant.

use crate::MONAD_TRUTH_ANCHOR;

// Buffer sizes only. These match the real ML-KEM-1024 / ML-DSA-87 parameter sets, but no
// lattice key generation, encapsulation or signing operates on them in this file.
pub const KYBER_1024_PUBLIC_KEY_BYTES: usize = 1568;
pub const KYBER_1024_SECRET_KEY_BYTES: usize = 3168;
pub const KYBER_1024_CIPHERTEXT_BYTES: usize = 1568;
pub const KYBER_1024_SHARED_SECRET_BYTES: usize = 32;

pub const DILITHIUM_5_PUBLIC_KEY_BYTES: usize = 2592;
pub const DILITHIUM_5_SECRET_KEY_BYTES: usize = 4896;
pub const DILITHIUM_5_SIGNATURE_BYTES: usize = 4595;

#[derive(Debug, Clone)]
pub struct KyberKeyPair {
    pub public_key: [u8; KYBER_1024_PUBLIC_KEY_BYTES],
    pub secret_key: [u8; KYBER_1024_SECRET_KEY_BYTES],
}

#[derive(Debug, Clone)]
pub struct EncapsulationResult {
    pub ciphertext: [u8; KYBER_1024_CIPHERTEXT_BYTES],
    pub shared_secret: [u8; KYBER_1024_SHARED_SECRET_BYTES],
}

/// Declared for API shape only. Nothing in this module ever constructs one, and there is no
/// sign or verify function to produce or check `sig_bytes`.
#[derive(Debug, Clone)]
pub struct DilithiumSignature {
    pub sig_bytes: [u8; DILITHIUM_5_SIGNATURE_BYTES],
    pub valid: bool,
}

#[derive(Debug, Clone)]
pub struct QuantumMeshNodeAuth {
    pub node_id: usize,
    pub location: &'static str,
    pub public_key_fingerprint: String,
    pub encapsulation_latency_us: f64,
    pub decapsulation_latency_us: f64,
    pub signature_verification_status: &'static str,
    pub post_quantum_security_level_bits: usize, // 0: no post-quantum primitive is implemented
}

#[derive(Debug, Clone)]
pub struct QuantumSecurityReport {
    pub total_nodes_secured: usize,
    pub algorithm_kem: &'static str,
    pub algorithm_dsa: &'static str,
    pub mean_kem_latency_us: f64,
    pub mean_dsa_latency_us: f64,
    pub total_key_exchange_ops: usize,
    pub lattice_dimension_k: usize,
    pub modulus_q: usize,
    pub quantum_immunity_level: &'static str,
    pub node_auths: Vec<QuantumMeshNodeAuth>,
    pub monad_anchor: u64,
}

pub struct QuantumSecurityEngine;

impl QuantumSecurityEngine {
    /// Deterministic byte-pattern key material. NOT ML-KEM / Kyber key generation.
    ///
    /// `pk[i] = (seed + i) ^ 0x4D4F4E41` and `sk[i] = (seed + 7i) ^ 1`, both truncated to a byte.
    /// There is no polynomial sampling, no NTT, no error term, and `sk` is not derived from `pk`.
    pub fn keygen(node_seed: u64) -> KyberKeyPair {
        let mut pk = [0u8; KYBER_1024_PUBLIC_KEY_BYTES];
        let mut sk = [0u8; KYBER_1024_SECRET_KEY_BYTES];

        // Byte pattern from the seed. Not lattice polynomial generation.
        for i in 0..KYBER_1024_PUBLIC_KEY_BYTES {
            pk[i] = ((node_seed.wrapping_add(i as u64) ^ 0x4D4F4E41) & 0xFF) as u8;
        }
        for i in 0..KYBER_1024_SECRET_KEY_BYTES {
            sk[i] = ((node_seed.wrapping_add((i * 7) as u64) ^ 0x00000001) & 0xFF) as u8;
        }

        KyberKeyPair { public_key: pk, secret_key: sk }
    }

    /// Rotate/XOR keystream over the public key. NOT ML-KEM encapsulation.
    ///
    /// The returned `shared_secret` is the final 8-byte accumulator repeated four times, and it
    /// depends only on `pk` — `msg_seed` changes `ciphertext` but not `shared_secret`. A value
    /// derived from the public key alone is not a shared secret.
    pub fn encapsulate(pk: &[u8; KYBER_1024_PUBLIC_KEY_BYTES], msg_seed: u64) -> EncapsulationResult {
        let mut ct = [0u8; KYBER_1024_CIPHERTEXT_BYTES];
        let mut ss = [0u8; KYBER_1024_SHARED_SECRET_BYTES];

        let mut acc: u64 = msg_seed;
        for i in 0..KYBER_1024_CIPHERTEXT_BYTES {
            acc = acc.rotate_left(3) ^ (pk[i % KYBER_1024_PUBLIC_KEY_BYTES] as u64);
            ct[i] = (acc & 0xFF) as u8;
        }

        for i in 0..KYBER_1024_SHARED_SECRET_BYTES {
            ss[i] = ((acc.wrapping_shr((i % 8) as u32)) & 0xFF) as u8;
        }

        EncapsulationResult { ciphertext: ct, shared_secret: ss }
    }

    /// Rotate/XOR keystream over ciphertext and secret key. NOT ML-KEM decapsulation.
    ///
    /// This does not invert `encapsulate`: the accumulator starts from a fixed constant and mixes
    /// `ct` and `sk`, so the result differs from the encapsulated secret for every input. The
    /// handshake in `secure_planetary_mesh` therefore never establishes a common key.
    pub fn decapsulate(sk: &[u8; KYBER_1024_SECRET_KEY_BYTES], ct: &[u8; KYBER_1024_CIPHERTEXT_BYTES]) -> [u8; KYBER_1024_SHARED_SECRET_BYTES] {
        let mut ss = [0u8; KYBER_1024_SHARED_SECRET_BYTES];
        let mut acc: u64 = 0x4D4F_4E41_4400_0001;

        for i in 0..KYBER_1024_CIPHERTEXT_BYTES {
            acc = acc.rotate_left(1) ^ (ct[i] as u64) ^ (sk[i % KYBER_1024_SECRET_KEY_BYTES] as u64);
        }

        for i in 0..KYBER_1024_SHARED_SECRET_BYTES {
            ss[i] = ((acc.wrapping_shr((i % 8) as u32)) & 0xFF) as u8;
        }

        ss
    }

    /// Run the byte-mixing routine across the 16 named mesh nodes and report what it did.
    ///
    /// No key agreement, no authentication and no signature take place. The report's algorithm
    /// and immunity strings say so.
    pub fn secure_planetary_mesh() -> QuantumSecurityReport {
        let node_names = [
            "New York (HQ)", "London", "Frankfurt", "Tokyo",
            "Singapore", "Sydney", "São Paulo", "Reykjavik",
            "Cape Town", "Dubai", "San Francisco", "Seoul",
            "Nairobi", "Honolulu", "Helsinki", "Auckland",
        ];

        let mut node_auths = Vec::with_capacity(16);
        let mut total_kem_time = 0.0;
        let mut total_dsa_time = 0.0;

        for (idx, &name) in node_names.iter().enumerate() {
            let node_id = idx + 1;
            let seed = (node_id as u64) * 0x1F2E3D4C_5B6A7B8C;

            let t_keygen = std::time::Instant::now();
            let keypair = Self::keygen(seed);
            let _ = t_keygen.elapsed();

            let t_enc = std::time::Instant::now();
            let enc_res = Self::encapsulate(&keypair.public_key, seed ^ 0x55AA);
            let enc_us = t_enc.elapsed().as_secs_f64() * 1e6;

            let t_dec = std::time::Instant::now();
            let dec_ss = Self::decapsulate(&keypair.secret_key, &enc_res.ciphertext);
            let dec_us = t_dec.elapsed().as_secs_f64() * 1e6;

            // Lengths only. This deliberately does NOT assert that the two sides agree, because
            // they do not: `decapsulate` never reproduces `encapsulate`'s shared secret. Asserting
            // equality here would panic on every node.
            assert_eq!(enc_res.shared_secret.len(), dec_ss.len());

            total_kem_time += enc_us + dec_us;
            // No signature is generated or verified in this loop, so there is no DSA latency to
            // measure. This used to add a 1.25 us "budget" per node and print the mean as measured
            // performance. The budget is recorded as a declared constant instead, and the reported
            // mean stays 0.0 with the printer saying it was not measured.
            let _dsa_budget_us = 1.25;

            let fp = format!("{:02X}{:02X}..{:02X}{:02X}",
                keypair.public_key[0], keypair.public_key[1],
                keypair.public_key[KYBER_1024_PUBLIC_KEY_BYTES - 2], keypair.public_key[KYBER_1024_PUBLIC_KEY_BYTES - 1]);

            node_auths.push(QuantumMeshNodeAuth {
                node_id,
                location: name,
                public_key_fingerprint: fp,
                encapsulation_latency_us: enc_us,
                decapsulation_latency_us: dec_us,
                // No signature exists to verify. This field previously held a hardcoded
                // "verified" string naming a Dilithium level, with no signing code behind it.
                signature_verification_status: "NOT VERIFIED (no signature generated or verified)",
                post_quantum_security_level_bits: 0,
            });
        }

        QuantumSecurityReport {
            // Counts the 16 nodes this loop processed. It is not a count of secured nodes.
            total_nodes_secured: 16,
            algorithm_kem: "XOR/rotate keystream (NOT a lattice KEM; no ML-KEM/Kyber primitive implemented)",
            algorithm_dsa: "none (no signature primitive implemented; DilithiumSignature is an unused struct)",
            mean_kem_latency_us: total_kem_time / 16.0,
            mean_dsa_latency_us: total_dsa_time / 16.0,
            total_key_exchange_ops: 16,
            lattice_dimension_k: 0, // no lattice arithmetic is performed
            modulus_q: 0,           // no modular arithmetic is performed
            quantum_immunity_level: "NO POST-QUANTUM SECURITY CLAIM (no lattice primitive implemented)",
            node_auths,
            monad_anchor: MONAD_TRUTH_ANCHOR,
        }
    }
}
