//! TRM Swarm OS — Collaborative Reasoning Amplification (CRA) (Deterministic bare-metal no_std + alloc implementation).
//! Fully mirrors swarm_v2/core/collaborative_reasoning.py & strip_collab_reasoning.rs.
//! Amplifies intermediate cognitive representations between Reasoner (R), Architect (A), and Defense (D)
//! via 32-dimensional consensus projection, dual-cosine similarity, and Kleene K3 balanced ternary gating.

extern crate alloc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::format;
use core::sync::atomic::{AtomicU64, Ordering};

// --- CONSTANTS ---
pub const CRA_VECTOR_DIM: usize = 32;
pub const K3_TRUE_THRESHOLD: f32 = 0.25;
pub const K3_FALSE_THRESHOLD: f32 = -0.15;

// --- BARE-METAL SQUARE ROOT ---
#[inline(always)]
pub fn sqrt_f32(val: f32) -> f32 {
    if val <= 0.0 {
        return 0.0;
    }
    let mut x = val;
    let mut y = (x + 1.0) * 0.5;
    for _ in 0..12 {
        if (y - x).abs() < 1e-6 {
            break;
        }
        x = y;
        y = (x + val / x) * 0.5;
    }
    y
}

// --- CRA AMPLIFICATION RESULT ---
#[derive(Debug, Clone, Copy)]
pub struct CraAmplificationResult {
    pub consensus_vector: [f32; CRA_VECTOR_DIM],
    pub confidence: f32,
    pub k3_trit: i8, // +1 (True), 0 (Unknown), -1 (False)
    pub cosine_ra: f32,
    pub cosine_ad: f32,
}

// --- PURE NO_STD CRA STATE AMPLIFICATION ---
pub fn amplify_cra_states(
    r: &[f32; CRA_VECTOR_DIM],
    a: &[f32; CRA_VECTOR_DIM],
    d: &[f32; CRA_VECTOR_DIM],
) -> CraAmplificationResult {
    let mut consensus = [0.0f32; CRA_VECTOR_DIM];
    let mut sum_sq = 0.0f32;
    let mut r_sq = 0.0f32;
    let mut a_sq = 0.0f32;
    let mut d_sq = 0.0f32;
    let mut dot_ra = 0.0f32;
    let mut dot_ad = 0.0f32;

    for k in 0..CRA_VECTOR_DIM {
        let rv = r[k];
        let av = a[k];
        let dv = d[k];
        let syn = (rv + av + dv) / 3.0f32;
        consensus[k] = syn;
        sum_sq += syn * syn;

        r_sq += rv * rv;
        a_sq += av * av;
        d_sq += dv * dv;
        dot_ra += rv * av;
        dot_ad += av * dv;
    }

    let norm = sqrt_f32(sum_sq) + 1e-9f32;
    for k in 0..CRA_VECTOR_DIM {
        consensus[k] /= norm;
    }

    let r_norm = sqrt_f32(r_sq) + 1e-9f32;
    let a_norm = sqrt_f32(a_sq) + 1e-9f32;
    let d_norm = sqrt_f32(d_sq) + 1e-9f32;

    let cos_ra = dot_ra / (r_norm * a_norm);
    let cos_ad = dot_ad / (a_norm * d_norm);
    let conf = (cos_ra + cos_ad) * 0.5f32;

    let trit = if conf >= K3_TRUE_THRESHOLD {
        1i8
    } else if conf <= K3_FALSE_THRESHOLD {
        -1i8
    } else {
        0i8
    };

    CraAmplificationResult {
        consensus_vector: consensus,
        confidence: conf,
        k3_trit: trit,
        cosine_ra: cos_ra,
        cosine_ad: cos_ad,
    }
}

// --- COLLABORATIVE REASONING SESSION ---
#[derive(Debug, Clone)]
pub struct CollaborativeReasoningSession {
    pub session_id: String,
    pub task_prompt: String,
    pub r_vector: [f32; CRA_VECTOR_DIM],
    pub a_vector: [f32; CRA_VECTOR_DIM],
    pub d_vector: [f32; CRA_VECTOR_DIM],
    pub result: Option<CraAmplificationResult>,
    pub created_at_epoch_s: u64,
}

impl CollaborativeReasoningSession {
    pub fn new(id: &str, prompt: &str, epoch_s: u64) -> Self {
        Self {
            session_id: id.to_string(),
            task_prompt: prompt.to_string(),
            r_vector: [0.0f32; CRA_VECTOR_DIM],
            a_vector: [0.0f32; CRA_VECTOR_DIM],
            d_vector: [0.0f32; CRA_VECTOR_DIM],
            result: None,
            created_at_epoch_s: epoch_s,
        }
    }

    pub fn set_states(&mut self, r: &[f32; CRA_VECTOR_DIM], a: &[f32; CRA_VECTOR_DIM], d: &[f32; CRA_VECTOR_DIM]) {
        self.r_vector = *r;
        self.a_vector = *a;
        self.d_vector = *d;
    }

    pub fn amplify(&mut self) -> CraAmplificationResult {
        let res = amplify_cra_states(&self.r_vector, &self.a_vector, &self.d_vector);
        self.result = Some(res);
        res
    }
}

// --- COLLABORATIVE REASONING ENGINE ---
pub struct CollaborativeReasoningEngine {
    sessions: Vec<CollaborativeReasoningSession>,
    pub total_amplifications: u64,
}

impl CollaborativeReasoningEngine {
    pub fn new() -> Self {
        Self {
            sessions: Vec::new(),
            total_amplifications: 0,
        }
    }

    pub fn create_session(&mut self, session_id: &str, prompt: &str, epoch_s: u64) -> &mut CollaborativeReasoningSession {
        let session = CollaborativeReasoningSession::new(session_id, prompt, epoch_s);
        if self.sessions.len() >= 64 {
            self.sessions.remove(0);
        }
        self.sessions.push(session);
        self.sessions.last_mut().unwrap()
    }

    pub fn amplify_session(&mut self, session_id: &str) -> Option<CraAmplificationResult> {
        if let Some(s) = self.sessions.iter_mut().find(|s| s.session_id == session_id) {
            let res = s.amplify();
            self.total_amplifications += 1;
            Some(res)
        } else {
            None
        }
    }

    pub fn get_session(&self, session_id: &str) -> Option<&CollaborativeReasoningSession> {
        self.sessions.iter().find(|s| s.session_id == session_id)
    }

    pub fn total_sessions(&self) -> usize {
        self.sessions.len()
    }
}

// --- SINGLETON INTERFACE ---
static SINGLETON_CRA_ENGINE: AtomicU64 = AtomicU64::new(0);

pub fn get_cra_engine() -> u64 {
    let v = SINGLETON_CRA_ENGINE.load(Ordering::Relaxed);
    if v == 0 {
        SINGLETON_CRA_ENGINE.store(1, Ordering::Relaxed);
        1
    } else {
        v
    }
}

