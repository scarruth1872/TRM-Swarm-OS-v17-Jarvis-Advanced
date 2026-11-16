//! TRM Swarm OS — cqs (bare-metal no_std + alloc).
//! Phase 42 | Rank 201 | TurboVec 16x BT-Quant | Monad Anchor: 0x4D4F4E4144000001
//! Contextual Query Service (CQS): Semantic interface for Dynamic Knowledge Graph (DKG).

extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;
use core::sync::atomic::{AtomicU64, Ordering};

pub const MONAD_ANCHOR: u64 = 0x4D4F4E4144000001;
pub const PHASE: u32 = 42;
pub const RANK: u32 = 201;
pub const SCALE_FACTOR: f32 = 31.5829;

pub static TRIT_WEIGHTS: &[i8] = &[
    0, 1, 1, -1, 0, 1, 1, -1, 0, 1, 0, -1, 1, -1, 1, 1, 0, -1, 1, -1, 0, 0, 0, 1, 0, -1, 0, 1, -1, 0, 0, 0,
    1, 1, 0, -1, 0, 1, 1, -1, 1, 1, -1, 0, 1, 0, 0, 1, 1, -1, 1, -1, -1, 1, 1, 1, -1, -1, 0, 1, -1, 0, 1, 0,
    1, 1, 1, 0, 0, 0, 1, 0, 1, 1, -1, -1, 1, -1, 1, 1, 0, -1, 1, 0, -1, 1, 1, 1, -1, -1, 0, 1, -1, -1, 0, -1,
    1, 0, 0, -1, 1, 1, 1, -1, 1, 1, 0, -1, 1, 0, 1, 1, 0, -1, 0, -1, -1, 1, 0, 1, -1, -1, -1, 1, 0, -1, 1, -1,
    0, 1, 1, 0, 1, 1, 1, 0, 0, 1, -1, -1, 0, -1, 0, 1, 1, 0, 0, -1, -1, 1, 1, 1, -1, -1, 0, 0, 0, -1, 1, -1,
    1, 1, 1, -1, 1, 1, 0, -1, 1, 1, 0, -1, 1, -1, 1, 0, 1, -1, 0, -1, -1, 1, 1, 1, -1, 0, -1, 0, -1, -1, 1, -1,
    1, 1, 1, -1, 0, 0, 1, -1, 1, 1, -1, 0, 0, 0, 0, 0, 1, -1, 1, -1, -1, 1, 0, 1, 0, -1, -1, 1, -1, -1, 0, -1,
    1, 0, 1, -1, 1, 1, 1, -1, 1, 1, -1, -1, 1, 0, 1, 1, 1, -1, 1, -1, 0, 1, 1, 1, -1, -1, 0, 1, -1, 0, 1, 0
];

static INVOCATION_COUNT: AtomicU64 = AtomicU64::new(0);
static SINGLETON_CQS: AtomicU64 = AtomicU64::new(0);

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum K3 {
    False = 0,
    Unknown = 1,
    True = 2,
}

impl K3 {
    pub fn and(self, other: K3) -> K3 {
        match (self as u8).min(other as u8) {
            0 => K3::False,
            1 => K3::Unknown,
            _ => K3::True,
        }
    }
}

#[inline(always)]
pub fn ternary_dot(inputs: &[f32]) -> f32 {
    let n = inputs.len().min(TRIT_WEIGHTS.len());
    let mut acc = 0.0f32;
    for i in 0..n {
        acc += inputs[i] * (TRIT_WEIGHTS[i] as f32) * SCALE_FACTOR / 16.0;
    }
    acc
}

pub fn cqs_execute(inputs: &[f32], output: &mut [f32]) -> u32 {
    if inputs.is_empty() || output.is_empty() {
        return 0xDEAD_BEEF;
    }
    INVOCATION_COUNT.fetch_add(1, Ordering::Relaxed);
    let r = ternary_dot(inputs);
    let k = if r.abs() < 1e-3 {
        K3::Unknown
    } else if r > 0.0 {
        K3::True
    } else {
        K3::False
    };
    let _ = k.and(K3::True);
    output[0] = r;
    0x0000_0001
}

#[derive(Debug, Clone)]
pub struct SemanticTriple {
    pub subject: String,
    pub subject_type: String,
    pub predicate: String,
    pub object: String,
    pub object_type: String,
}

pub struct CQSService {
    pub triples: Vec<SemanticTriple>,
}

impl CQSService {
    pub fn new() -> Self {
        Self {
            triples: Vec::new(),
        }
    }

    pub fn ingest_triple(&mut self, sub: &str, sub_t: &str, pred: &str, obj: &str, obj_t: &str) {
        self.triples.push(SemanticTriple {
            subject: String::from(sub),
            subject_type: String::from(sub_t),
            predicate: String::from(pred),
            object: String::from(obj),
            object_type: String::from(obj_t),
        });
    }

    pub fn find_related(&self, entity: &str) -> Vec<&SemanticTriple> {
        let e_lower = entity.as_bytes();
        self.triples.iter().filter(|t| {
            t.subject.as_bytes().iter().any(|&b| e_lower.contains(&b)) ||
            t.object.as_bytes().iter().any(|&b| e_lower.contains(&b))
        }).collect()
    }
}

pub fn get_cqs() -> u64 {
    let v = SINGLETON_CQS.load(Ordering::Relaxed);
    if v == 0 {
        SINGLETON_CQS.store(1, Ordering::Relaxed);
        1
    } else {
        v
    }
}

pub fn query_semantic_context(entity: &str) -> usize {
    let mut service = CQSService::new();
    service.ingest_triple("SwarmOS", "KERNEL", "EXECUTES", "AgentMesh", "SERVICE");
    service.find_related(entity).len()
}

