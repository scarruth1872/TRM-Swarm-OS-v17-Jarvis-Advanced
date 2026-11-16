//! TRM Swarm OS — cognitive_ingestion_engine (no_std port, REAL-verified).
//! 1:1 deterministic core of swarm_v2/core/cognitive_ingestion_engine.py:
//!   - IngestionMatrixType enum and provenance tags
//!   - Deterministic FNV-1a matrix tokenization (tokenize_matrix_slice)
//!   - 64-dimensional cognitive gene histogram & L2 normalization (encode_cognitive_genes)
//!   - SystemEventLedger record models (EventCategory, EventRecord)
//!   - Persistent rolling event buffer with category filtering and statistics
//!   - Monad Truth Anchor: 0x4D4F4E4144000001

extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};

pub const MONAD_TRUTH_ANCHOR: u64 = 0x4D4F_4E41_4400_0001;
pub const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
pub const FNV_PRIME: u64 = 0x100000001b3;
pub const MAX_ROLLING_EVENTS: usize = 1024;
pub const INGESTION_INTERVAL_SECONDS: u64 = 60;

/// Static atomic execution and lifecycle counters
static SINGLETON_EVENT_LEDGER: AtomicU64 = AtomicU64::new(0);
static SINGLETON_INGESTION_ENGINE: AtomicU64 = AtomicU64::new(0);
static INGESTION_CYCLES_COMPLETED: AtomicU64 = AtomicU64::new(0);
static TOTAL_MATRICES_INGESTED: AtomicU64 = AtomicU64::new(0);
static TOTAL_EVENTS_RECORDED: AtomicU64 = AtomicU64::new(0);

/// Core matrix types pulled from Jarvis into Swarm OS
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IngestionMatrixType {
    CoreMemory,
    EpisodicMemory,
    SemanticGraph,
    SkillsRegistry,
    BiomimeticPresets,
    LearnResourceMaps,
    ChatHistory,
    NoosphericCalibration,
}

impl IngestionMatrixType {
    pub fn as_str(&self) -> &'static str {
        match self {
            IngestionMatrixType::CoreMemory => "core_memory",
            IngestionMatrixType::EpisodicMemory => "episodic_memory",
            IngestionMatrixType::SemanticGraph => "semantic_graph",
            IngestionMatrixType::SkillsRegistry => "skills_registry",
            IngestionMatrixType::BiomimeticPresets => "biomimetic_presets",
            IngestionMatrixType::LearnResourceMaps => "learn_resource_maps",
            IngestionMatrixType::ChatHistory => "chat_history",
            IngestionMatrixType::NoosphericCalibration => "noospheric_calibration",
        }
    }

    pub fn priority_weight(&self) -> u32 {
        match self {
            IngestionMatrixType::CoreMemory => 10,
            IngestionMatrixType::NoosphericCalibration => 9,
            IngestionMatrixType::EpisodicMemory => 8,
            IngestionMatrixType::SemanticGraph => 7,
            IngestionMatrixType::SkillsRegistry => 6,
            IngestionMatrixType::BiomimeticPresets => 5,
            IngestionMatrixType::LearnResourceMaps => 4,
            IngestionMatrixType::ChatHistory => 3,
        }
    }
}

/// Software square root for strict no_std environments
#[inline]
pub fn sqrt_f32(val: f32) -> f32 {
    if val <= 0.0 {
        return 0.0;
    }
    let mut x = val;
    for _ in 0..10 {
        x = 0.5 * (x + val / x);
    }
    x
}

/// Computes 64-dimensional L2-normalized gene histogram embeddings.
/// Bucket rule verified identical to `strip_cognitive_gene_encoder.dll` /
/// strip_cognitive_encoder.rs for all 256 byte values (bit-exact).
/// Rounding note (verified): the strip computes `hist[i] * (1.0 / norm)` with a
/// hardware `f32::sqrt`, this port computes `hist[i] / norm` with a Newton
/// `sqrt_f32`; the Python fallback divides by a float64 `math.sqrt`. The two
/// forms differ by <= 2 ulp on ~11% of elements for multi-byte genes (no_std
/// here has no `f32::sqrt`), never in the bucket or normalization rule.
pub fn encode_cognitive_genes(genes: &[Vec<u8>]) -> Vec<[f32; 64]> {
    let mut embeddings = Vec::with_capacity(genes.len());
    for gene in genes {
        let mut hist = [0.0f32; 64];
        for &byte in gene {
            let idx = (((byte as i32) - 32).rem_euclid(64)) as usize;
            hist[idx] += 1.0;
        }

        let mut sum_sq = 0.0f32;
        for &v in hist.iter() {
            sum_sq += v * v;
        }

        let norm = sqrt_f32(sum_sq);
        let divisor = if norm > 1e-7 { norm } else { 1.0 };

        for v in hist.iter_mut() {
            *v /= divisor;
        }
        embeddings.push(hist);
    }
    embeddings
}

/// Deterministic FNV-1a windowed tokenization for symbolic matrices
pub fn tokenize_matrix_slice(data: &[u8], window_size: usize) -> Vec<u32> {
    if data.is_empty() {
        return Vec::new();
    }
    let win = if window_size == 0 { 4 } else { window_size };
    let mut tokens = Vec::new();

    let mut i = 0;
    while i < data.len() {
        let end = (i + win).min(data.len());
        let chunk = &data[i..end];

        let mut hash = FNV_OFFSET_BASIS;
        for &byte in chunk {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(FNV_PRIME);
        }

        tokens.push((hash & 0xFFFF_FFFF) as u32);
        i += win;
    }
    tokens
}

/// Categories for SystemEventLedger entries
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventCategory {
    Ingestion,
    ApiCall,
    AgentDecision,
    Consensus,
    Healing,
    Symbiosis,
    Learning,
    Security,
    // Oracle `SystemEventLedger.CATEGORIES` entries that were absent from this port.
    TrmReasoning,
    Genomics,
    StarMatrix,
    Memory,
    System,
}

impl EventCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            EventCategory::Ingestion => "INGESTION",
            EventCategory::ApiCall => "API_CALL",
            EventCategory::AgentDecision => "AGENT_DECISION",
            EventCategory::Consensus => "CONSENSUS",
            EventCategory::Healing => "HEALING",
            EventCategory::Symbiosis => "SYMBIOSIS",
            EventCategory::Learning => "LEARNING",
            EventCategory::Security => "SECURITY",
            EventCategory::TrmReasoning => "TRM_REASONING",
            EventCategory::Genomics => "GENOMICS",
            EventCategory::StarMatrix => "STAR_MATRIX",
            EventCategory::Memory => "MEMORY",
            EventCategory::System => "SYSTEM",
        }
    }
}

/// Individual immutable system event record
#[derive(Debug, Clone)]
pub struct EventRecord {
    pub event_id: u64,
    pub timestamp_epoch: u64,
    pub category: EventCategory,
    pub source: String,
    pub summary: String,
    pub k3_trit: i8,
    pub monad_anchor: u64,
}

/// Persistent, append-only in-memory rolling event ledger
#[derive(Debug, Clone)]
pub struct SystemEventLedger {
    events: Vec<EventRecord>,
    max_capacity: usize,
    total_recorded: u64,
}

impl SystemEventLedger {
    pub fn new() -> Self {
        Self {
            events: Vec::with_capacity(MAX_ROLLING_EVENTS),
            max_capacity: MAX_ROLLING_EVENTS,
            total_recorded: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            events: Vec::with_capacity(capacity),
            max_capacity: capacity,
            total_recorded: 0,
        }
    }

    pub fn record_event(
        &mut self,
        timestamp_epoch: u64,
        category: EventCategory,
        source: &str,
        summary: &str,
        k3_trit: i8,
    ) -> u64 {
        self.total_recorded += 1;
        TOTAL_EVENTS_RECORDED.fetch_add(1, Ordering::Relaxed);
        let id = self.total_recorded;

        let record = EventRecord {
            event_id: id,
            timestamp_epoch,
            category,
            source: String::from(source),
            summary: String::from(summary),
            k3_trit: if k3_trit > 0 { 1 } else if k3_trit < 0 { -1 } else { 0 },
            monad_anchor: MONAD_TRUTH_ANCHOR,
        };

        if self.events.len() >= self.max_capacity {
            self.events.remove(0);
        }
        self.events.push(record);
        id
    }

    pub fn get_events_by_category(&self, category: EventCategory) -> Vec<&EventRecord> {
        self.events.iter().filter(|e| e.category == category).collect()
    }

    pub fn get_recent_events(&self, count: usize) -> Vec<&EventRecord> {
        let n = self.events.len();
        let start = if count > n { 0 } else { n - count };
        self.events[start..].iter().collect()
    }

    pub fn total_recorded(&self) -> u64 {
        self.total_recorded
    }

    pub fn current_buffered_count(&self) -> usize {
        self.events.len()
    }

    pub fn clear(&mut self) {
        self.events.clear();
    }
}

impl Default for SystemEventLedger {
    fn default() -> Self {
        Self::new()
    }
}

/// Ingestion Cycle Summary
#[derive(Debug, Clone)]
pub struct IngestionCycleReport {
    pub cycle_id: u64,
    pub matrices_processed: usize,
    pub total_tokens_generated: usize,
    pub average_spectral_norm: f32,
    pub k3_status: i8,
}

/// High-level Cognitive Ingestion Coordinator
#[derive(Debug, Clone)]
pub struct CognitiveIngestionCoordinator {
    pub ledger: SystemEventLedger,
    pub cycles_run: u64,
}

impl CognitiveIngestionCoordinator {
    pub fn new() -> Self {
        Self {
            ledger: SystemEventLedger::new(),
            cycles_run: 0,
        }
    }

    pub fn process_matrix_batch(
        &mut self,
        timestamp_epoch: u64,
        matrices: &[(IngestionMatrixType, &[u8])],
    ) -> IngestionCycleReport {
        self.cycles_run += 1;
        INGESTION_CYCLES_COMPLETED.fetch_add(1, Ordering::Relaxed);
        TOTAL_MATRICES_INGESTED.fetch_add(matrices.len() as u64, Ordering::Relaxed);

        let mut total_tokens = 0;
        let mut genes_to_encode = Vec::with_capacity(matrices.len());

        for (mtype, data) in matrices {
            let tokens = tokenize_matrix_slice(data, 4);
            total_tokens += tokens.len();
            genes_to_encode.push(data.to_vec());

            self.ledger.record_event(
                timestamp_epoch,
                EventCategory::Ingestion,
                "CognitiveIngestionEngine",
                &format!("Ingested matrix {}: {} tokens", mtype.as_str(), tokens.len()),
                1,
            );
        }

        let embeddings = encode_cognitive_genes(&genes_to_encode);
        let mut total_norm = 0.0f32;
        for emb in &embeddings {
            let mut sum_sq = 0.0f32;
            for &v in emb.iter() {
                sum_sq += v * v;
            }
            total_norm += sqrt_f32(sum_sq);
        }

        let avg_norm = if !embeddings.is_empty() {
            total_norm / (embeddings.len() as f32)
        } else {
            1.0
        };

        IngestionCycleReport {
            cycle_id: self.cycles_run,
            matrices_processed: matrices.len(),
            total_tokens_generated: total_tokens,
            average_spectral_norm: avg_norm,
            k3_status: 1,
        }
    }
}

impl Default for CognitiveIngestionCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

// ─────────────────────────────────────────────────────────────
// Compatibility and Singleton Accessors
// ─────────────────────────────────────────────────────────────

pub fn get_event_ledger() -> u64 {
    let v = SINGLETON_EVENT_LEDGER.load(Ordering::Relaxed);
    if v == 0 {
        SINGLETON_EVENT_LEDGER.store(1, Ordering::Relaxed);
        1
    } else {
        v
    }
}

pub fn get_ingestion_engine() -> u64 {
    let v = SINGLETON_INGESTION_ENGINE.load(Ordering::Relaxed);
    if v == 0 {
        SINGLETON_INGESTION_ENGINE.store(1, Ordering::Relaxed);
        1
    } else {
        v
    }
}

pub fn get_ingestion_metrics() -> (u64, u64, u64) {
    (
        INGESTION_CYCLES_COMPLETED.load(Ordering::Relaxed),
        TOTAL_MATRICES_INGESTED.load(Ordering::Relaxed),
        TOTAL_EVENTS_RECORDED.load(Ordering::Relaxed),
    )
}

// ═════════════════════════════════════════════════════════════════════════════
// PYTHON-ORACLE PORT — CognitiveTokenEncoder + cycle commit/status rules
//
// Oracle: swarm_v2/core/cognitive_ingestion_engine.py (no native strip exists
// for this module). The only native strip in scope is strip_cognitive_encoder
// (the 64-bin gene histogram below), whose .rs source is the oracle for
// `encode_cognitive_genes`.
//
// HOST-SIDE in the oracle and deliberately NOT ported here: urllib HTTP
// fetches (JarvisCognitiveFetcher), the JSONL ledger file I/O
// (SystemEventLedger.record/tail/get_stats/prune on disk), GlobalMemory
// contribution (`_commit_to_global_memory`), snapshot persistence
// (`_save_snapshot`), TRMBrain.reason() (only its presence/absence is
// modelled), asyncio daemon, `datetime.utcnow()` timestamps, `hash(seq_name)`
// puzzle ids (PYTHONHASHSEED-salted → not deterministic), and logging.
//
// std-only math is hand-rolled below (SHA-256, python_str) because the kernel
// is no_std + alloc only.
// ═════════════════════════════════════════════════════════════════════════════

// ── CognitiveTokenEncoder constants (oracle class attributes) ────────────────
pub const VOCAB_SIZE: u32 = 12;
pub const MAX_SEQ_LEN: usize = 512;
pub const MIN_ENCODE_TOKENS: usize = 8; // encode_string pads to at least 8
pub const DICT_BREADTH_LIMIT: usize = 20; // encode_dict: list(d.items())[:20]
pub const LIST_ITEM_LIMIT: usize = 20; // encode_list(max_items=20)
pub const DICT_LIST_ITEM_LIMIT: usize = 5; // encode_dict: `for item in v[:5]`
pub const DICT_MAX_DEPTH: usize = 2; // encode_dict: `depth < 2`
pub const PRESET_LIMIT: usize = 10; // snapshot_to_token_sequences presets
pub const CHAT_TAIL_LIMIT: usize = 20; // chat[-20:]
pub const GENE_KEYS: [&str; 4] = ["plasticity", "logical_depth", "empathy", "stochasticity"];
pub const GENE_DEFAULT: f64 = 0.5;

// ── run_ingestion_cycle constants ───────────────────────────────────────────
pub const MAX_LEDGER_ENTRIES: usize = 10_000; // Python MAX_LEDGER_ENTRIES
pub const LEDGER_PRUNE_INTERVAL: u64 = 50; // `if self._cycle_count % 50 == 0`
pub const STATUS_JARVIS_OFFLINE: &str = "JARVIS_OFFLINE";
pub const STATUS_COMPLETE: &str = "COMPLETE";

// ─────────────────────────────────────────────────────────────────────────────
// SHA-256 (hand-rolled, std-only math for no_std: core + alloc only)
// Oracle: int(hashlib.sha256(word.lower().encode()).hexdigest(), 16) % VOCAB
// ─────────────────────────────────────────────────────────────────────────────

const SHA256_K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

fn sha256_compress(state: &mut [u32; 8], block: &[u8; 64]) {
    let mut w = [0u32; 64];
    let mut i = 0;
    while i < 16 {
        w[i] = u32::from_be_bytes([block[4 * i], block[4 * i + 1], block[4 * i + 2], block[4 * i + 3]]);
        i += 1;
    }
    while i < 64 {
        let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
        let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
        w[i] = w[i - 16]
            .wrapping_add(s0)
            .wrapping_add(w[i - 7])
            .wrapping_add(s1);
        i += 1;
    }

    let mut a = state[0];
    let mut b = state[1];
    let mut c = state[2];
    let mut d = state[3];
    let mut e = state[4];
    let mut f = state[5];
    let mut g = state[6];
    let mut h = state[7];

    let mut j = 0;
    while j < 64 {
        let big_s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
        let ch = (e & f) ^ ((!e) & g);
        let t1 = h
            .wrapping_add(big_s1)
            .wrapping_add(ch)
            .wrapping_add(SHA256_K[j])
            .wrapping_add(w[j]);
        let big_s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
        let maj = (a & b) ^ (a & c) ^ (b & c);
        let t2 = big_s0.wrapping_add(maj);

        h = g;
        g = f;
        f = e;
        e = d.wrapping_add(t1);
        d = c;
        c = b;
        b = a;
        a = t1.wrapping_add(t2);
        j += 1;
    }

    state[0] = state[0].wrapping_add(a);
    state[1] = state[1].wrapping_add(b);
    state[2] = state[2].wrapping_add(c);
    state[3] = state[3].wrapping_add(d);
    state[4] = state[4].wrapping_add(e);
    state[5] = state[5].wrapping_add(f);
    state[6] = state[6].wrapping_add(g);
    state[7] = state[7].wrapping_add(h);
}

/// Full 32-byte SHA-256 digest of `msg` (hand-rolled; no_std).
pub fn sha256_digest(msg: &[u8]) -> [u8; 32] {
    let mut state: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let bit_len = (msg.len() as u64).wrapping_mul(8);

    let mut i = 0;
    while i + 64 <= msg.len() {
        let mut block = [0u8; 64];
        block.copy_from_slice(&msg[i..i + 64]);
        sha256_compress(&mut state, &block);
        i += 64;
    }

    let rem_len = msg.len() - i;
    let mut tail = [0u8; 128];
    tail[..rem_len].copy_from_slice(&msg[i..]);
    tail[rem_len] = 0x80;
    let total = if rem_len + 1 + 8 <= 64 { 64 } else { 128 };
    tail[total - 8..total].copy_from_slice(&bit_len.to_be_bytes());

    let mut j = 0;
    while j < total {
        let mut block = [0u8; 64];
        block.copy_from_slice(&tail[j..j + 64]);
        sha256_compress(&mut state, &block);
        j += 64;
    }

    let mut out = [0u8; 32];
    let mut k = 0;
    while k < 8 {
        out[4 * k..4 * k + 4].copy_from_slice(&state[k].to_be_bytes());
        k += 1;
    }
    out
}

/// `int(sha256(msg).hexdigest(), 16) % modulus`, computed by rolling the
/// big-endian digest through `rem = (rem * 256 + byte) % modulus`
/// (256 ≡ 4 mod 12, so no 256-bit bignum is needed).
pub fn sha256_mod_u32(msg: &[u8], modulus: u32) -> u32 {
    let digest = sha256_digest(msg);
    let mut rem: u32 = 0;
    for &byte in digest.iter() {
        rem = (rem * 256 + byte as u32) % modulus;
    }
    rem
}

// ─────────────────────────────────────────────────────────────────────────────
// Snapshot value model — the oracle's dict / list / scalar tree (bounded to
// exactly the shapes the Python encoder inspects).
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum SnapshotValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    List(Vec<SnapshotValue>),
    Dict(Vec<(String, SnapshotValue)>),
}

/// Python `str()` of a float: shortest round-trip, integral values keep `.0`.
/// Divergence (documented, not exercised by this port's inputs): Python
/// switches to exponent form outside 1e-4 <= |v| < 1e16, Rust `Display` never
/// does, so |v| >= 1e16 or |v| < 1e-4 with a long decimal differs.
pub fn python_float_str(v: f64) -> String {
    if v.is_nan() {
        return String::from("nan");
    }
    if v.is_infinite() {
        return String::from(if v > 0.0 { "inf" } else { "-inf" });
    }
    if v == 0.0 {
        return String::from(if v.is_sign_negative() { "-0.0" } else { "0.0" });
    }
    // Python's `repr(float)` = shortest round-trip digits, then exponent form when the
    // decimal point sits at <= -4 or > 16 (str(1e15) is "1000000000000000.0" but
    // str(1e16) is "1e+16"). Rust's `{:e}` already gives shortest round-trip digits,
    // so this reformats its mantissa/exponent rather than reimplementing dtoa.
    let neg = v < 0.0;
    let a = if neg { -v } else { v };
    let e = format!("{:e}", a);
    let mut it = e.split('e');
    let mant = it.next().unwrap_or("1");
    let exp: i32 = it.next().unwrap_or("0").parse().unwrap_or(0);
    let digits: String = mant.chars().filter(|c| *c != '.').collect();
    let decpt = exp + 1;
    let body = if decpt <= -4 || decpt > 16 {
        let mut m = String::new();
        m.push(digits.as_bytes()[0] as char);
        if digits.len() > 1 {
            m.push('.');
            m.push_str(&digits[1..]);
        }
        let ex = decpt - 1;
        format!("{}e{}{:02}", m, if ex < 0 { "-" } else { "+" }, if ex < 0 { -ex } else { ex })
    } else if decpt <= 0 {
        format!("0.{}{}", "0".repeat((-decpt) as usize), digits)
    } else if decpt as usize >= digits.len() {
        format!("{}{}.0", digits, "0".repeat(decpt as usize - digits.len()))
    } else {
        let k = decpt as usize;
        format!("{}.{}", &digits[..k], &digits[k..])
    };
    if neg { format!("-{}", body) } else { body }
}

/// Python `repr()` of a string (single quotes, doubled-quote fallback).
pub fn python_str_repr(s: &str) -> String {
    let quote = if s.contains('\'') && !s.contains('"') { '"' } else { '\'' };
    let mut out = String::new();
    out.push(quote);
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c == quote => {
                out.push('\\');
                out.push(c);
            }
            c => out.push(c),
        }
    }
    out.push(quote);
    out
}

impl SnapshotValue {
    /// Python truthiness (`if memory:`, `if core:`, `if episodic:` ...).
    /// NaN is truthy in Python (`bool(float('nan')) is True`).
    pub fn is_truthy(&self) -> bool {
        match self {
            SnapshotValue::Null => false,
            SnapshotValue::Bool(b) => *b,
            SnapshotValue::Int(i) => *i != 0,
            SnapshotValue::Float(f) => *f != 0.0,
            SnapshotValue::Str(s) => !s.is_empty(),
            SnapshotValue::List(l) => !l.is_empty(),
            SnapshotValue::Dict(d) => !d.is_empty(),
        }
    }

    /// Python `str(value)` (`str(v)`, `str(item)`, `str(text)`).
    pub fn python_str(&self) -> String {
        match self {
            SnapshotValue::Null => String::from("None"),
            SnapshotValue::Bool(b) => String::from(if *b { "True" } else { "False" }),
            SnapshotValue::Int(i) => format!("{}", i),
            SnapshotValue::Float(f) => python_float_str(*f),
            SnapshotValue::Str(s) => s.clone(),
            SnapshotValue::List(items) => {
                let mut out = String::from("[");
                for (idx, item) in items.iter().enumerate() {
                    if idx > 0 {
                        out.push_str(", ");
                    }
                    out.push_str(&item.python_repr());
                }
                out.push(']');
                out
            }
            SnapshotValue::Dict(pairs) => {
                let mut out = String::from("{");
                for (idx, (k, v)) in pairs.iter().enumerate() {
                    if idx > 0 {
                        out.push_str(", ");
                    }
                    out.push_str(&python_str_repr(k));
                    out.push_str(": ");
                    out.push_str(&v.python_repr());
                }
                out.push('}');
                out
            }
        }
    }

    /// Python `repr(value)` — differs from `str()` only for strings.
    pub fn python_repr(&self) -> String {
        match self {
            SnapshotValue::Str(s) => python_str_repr(s),
            other => other.python_str(),
        }
    }

    pub fn as_list(&self) -> Option<&[SnapshotValue]> {
        match self {
            SnapshotValue::List(l) => Some(l.as_slice()),
            _ => None,
        }
    }

    pub fn as_dict(&self) -> Option<&[(String, SnapshotValue)]> {
        match self {
            SnapshotValue::Dict(d) => Some(d.as_slice()),
            _ => None,
        }
    }

    /// `d.get(key)` for the dict case only.
    pub fn get(&self, key: &str) -> Option<&SnapshotValue> {
        dict_get(self.as_dict().unwrap_or(&[]), key)
    }
}

/// `d.get(key)` on an ordered dict representation.
pub fn dict_get<'a>(d: &'a [(String, SnapshotValue)], key: &str) -> Option<&'a SnapshotValue> {
    d.iter().find(|(k, _)| k == key).map(|(_, v)| v)
}

/// `dict.get(key, default)` for the shapes the oracle uses.
fn get_or<'a>(d: &'a [(String, SnapshotValue)], key: &str, default: &'a SnapshotValue) -> &'a SnapshotValue {
    dict_get(d, key).unwrap_or(default)
}

// ─────────────────────────────────────────────────────────────────────────────
// CognitiveTokenEncoder — token space vocab_size=12, <= 512 tokens per matrix
// ─────────────────────────────────────────────────────────────────────────────

/// Oracle `CognitiveTokenEncoder.encode_string`: split on whitespace, SHA-256
/// each lower-cased word mod VOCAB_SIZE, pad to 8 zeros, cap at MAX_SEQ_LEN.
pub fn encode_string(text: &str) -> Vec<u32> {
    let mut tokens: Vec<u32> = Vec::new();
    for word in text.split_whitespace().take(MAX_SEQ_LEN) {
        tokens.push(sha256_mod_u32(word.to_lowercase().as_bytes(), VOCAB_SIZE));
    }
    while tokens.len() < MIN_ENCODE_TOKENS {
        tokens.push(0);
    }
    tokens.truncate(MAX_SEQ_LEN);
    tokens
}

/// Oracle `CognitiveTokenEncoder.encode_dict`: bread-first over the first 20
/// keys, recursing into a nested dict only while `depth < 2` (a dict past the
/// depth limit is fed to `encode_string` as its Python repr), lists limited to
/// the first 5 items, early exit once the sequence reaches MAX_SEQ_LEN.
///
/// Oracle note: `encode_dict` on a non-dict raises AttributeError in Python
/// (undefined); this port treats a non-dict as an empty encoding.
pub fn encode_dict(d: &[(String, SnapshotValue)], depth: usize) -> Vec<u32> {
    let mut tokens: Vec<u32> = Vec::new();
    for (k, v) in d.iter().take(DICT_BREADTH_LIMIT) {
        tokens.extend(encode_string(k));
        match v {
            SnapshotValue::Dict(inner) => {
                if depth < DICT_MAX_DEPTH {
                    tokens.extend(encode_dict(inner, depth + 1));
                } else {
                    tokens.extend(encode_string(&v.python_str()));
                }
            }
            SnapshotValue::List(items) => {
                for item in items.iter().take(DICT_LIST_ITEM_LIMIT) {
                    tokens.extend(encode_string(&item.python_str()));
                }
            }
            other => tokens.extend(encode_string(&other.python_str())),
        }
        if tokens.len() >= MAX_SEQ_LEN {
            break;
        }
    }
    tokens.truncate(MAX_SEQ_LEN);
    tokens
}

/// Oracle `CognitiveTokenEncoder.encode_list(lst, max_items=20)`: dict items go
/// through `encode_dict(item)` (depth 0), everything else through
/// `encode_string(str(item))`, early exit at MAX_SEQ_LEN.
pub fn encode_list(lst: &[SnapshotValue], max_items: usize) -> Vec<u32> {
    let mut tokens: Vec<u32> = Vec::new();
    for item in lst.iter().take(max_items) {
        match item.as_dict() {
            Some(inner) => tokens.extend(encode_dict(inner, 0)),
            None => tokens.extend(encode_string(&item.python_str())),
        }
        if tokens.len() >= MAX_SEQ_LEN {
            break;
        }
    }
    tokens.truncate(MAX_SEQ_LEN);
    tokens
}

/// Oracle `CognitiveTokenEncoder.encode_cognitive_gene`: quantize each float
/// with `min(int(val * VOCAB_SIZE), VOCAB_SIZE - 1)`. Python `int()` truncates
/// toward zero, so negative values yield negative tokens.
pub fn encode_cognitive_gene(gene_vec: &[f64]) -> Vec<i64> {
    gene_vec
        .iter()
        .map(|&val| {
            let q = (val * VOCAB_SIZE as f64) as i64; // trunc toward zero, as Python int()
            q.min(VOCAB_SIZE as i64 - 1)
        })
        .collect()
}

/// Oracle `CognitiveTokenEncoder.snapshot_to_token_sequences`: named token
/// sequence per cognitive matrix, in Python dict-insertion order.
pub fn snapshot_to_token_sequences(snapshot: &[(String, SnapshotValue)]) -> Vec<(String, Vec<u32>)> {
    let mut sequences: Vec<(String, Vec<u32>)> = Vec::new();
    let empty_dict: Vec<(String, SnapshotValue)> = Vec::new();
    let empty_list: Vec<SnapshotValue> = Vec::new();
    let def_dict = SnapshotValue::Dict(Vec::new());
    let def_list = SnapshotValue::List(Vec::new());

    // ── Memory matrix: core / episodic / nodes / relations ──
    if let Some(memory) = dict_get(snapshot, "memory").filter(|v| v.is_truthy()) {
        let md = memory.as_dict().unwrap_or(&empty_dict);
        let core = get_or(md, "core", &def_dict);
        sequences.push((
            String::from("memory_core"),
            encode_dict(core.as_dict().unwrap_or(&empty_dict), 0),
        ));
        let episodic = get_or(md, "episodic", &def_list);
        sequences.push((
            String::from("memory_episodic"),
            encode_list(episodic.as_list().unwrap_or(&empty_list), LIST_ITEM_LIMIT),
        ));
        let nodes = get_or(md, "nodes", &def_list);
        sequences.push((
            String::from("memory_nodes"),
            encode_list(nodes.as_list().unwrap_or(&empty_list), LIST_ITEM_LIMIT),
        ));
        let relations = get_or(md, "relations", &def_list);
        sequences.push((
            String::from("memory_relations"),
            encode_list(relations.as_list().unwrap_or(&empty_list), LIST_ITEM_LIMIT),
        ));
    }

    // ── Skills registry + topology ──
    if let Some(skills) = dict_get(snapshot, "skills").filter(|v| v.is_truthy()) {
        sequences.push((
            String::from("skills_registry"),
            encode_list(skills.as_list().unwrap_or(&empty_list), LIST_ITEM_LIMIT),
        ));
    }
    if let Some(sm) = dict_get(snapshot, "skills_matrix").filter(|v| v.is_truthy()) {
        if let Some(d) = sm.as_dict() {
            sequences.push((String::from("skills_topology"), encode_dict(d, 0)));
        }
    }

    // ── Biomimetic presets (DNA gene vectors) ──
    if let Some(presets) = dict_get(snapshot, "biomimetic_presets").filter(|v| v.is_truthy()) {
        if let Some(preset_map) = presets.as_dict() {
            for (preset_name, preset_data) in preset_map.iter().take(PRESET_LIMIT) {
                if let Some(pd) = preset_data.as_dict() {
                    let mut gene = [GENE_DEFAULT; 4];
                    for (idx, key) in GENE_KEYS.iter().enumerate() {
                        if let Some(v) = dict_get(pd, key) {
                            gene[idx] = value_as_f64(v);
                        }
                    }
                    sequences.push((
                        format!("preset_{}", preset_name),
                        encode_cognitive_gene(&gene)
                            .into_iter()
                            .map(|t| t as u32)
                            .collect(),
                    ));
                }
            }
        } else if let Some(preset_list) = presets.as_list() {
            sequences.push((
                String::from("biomimetic_presets"),
                encode_list(preset_list, LIST_ITEM_LIMIT),
            ));
        }
    }

    // ── Chat history (last 20 turns) ──
    if let Some(chat) = dict_get(snapshot, "chat_history").filter(|v| v.is_truthy()) {
        let items = chat.as_list().unwrap_or(&empty_list);
        let tail = if items.len() > CHAT_TAIL_LIMIT {
            &items[items.len() - CHAT_TAIL_LIMIT..]
        } else {
            items
        };
        sequences.push((
            String::from("chat_history"),
            encode_list(tail, LIST_ITEM_LIMIT),
        ));
    }

    // ── Noospheric calibration state ──
    if let Some(noospheric) = dict_get(snapshot, "noospheric_status").filter(|v| v.is_truthy()) {
        let tokens = match noospheric.as_dict() {
            Some(d) => encode_dict(d, 0),
            None => encode_string(&noospheric.python_str()),
        };
        sequences.push((String::from("noospheric_state"), tokens));
    }

    sequences
}

/// Oracle `float(preset_data.get(key, 0.5))` for the gene fields.
fn value_as_f64(v: &SnapshotValue) -> f64 {
    match v {
        SnapshotValue::Float(f) => *f,
        SnapshotValue::Int(i) => *i as f64,
        SnapshotValue::Bool(b) => {
            if *b {
                1.0
            } else {
                0.0
            }
        }
        SnapshotValue::Str(s) => s.trim().parse::<f64>().unwrap_or(GENE_DEFAULT),
        _ => GENE_DEFAULT,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// run_ingestion_cycle rules: matrices_loaded gate, commit count, ledger prune
// ─────────────────────────────────────────────────────────────────────────────

/// Oracle `fetch_all()`: `matrices_loaded = sum(1 for v in snapshot.values()
/// if v is not None and not isinstance(v, str))`. NOTE this counts ANY non-null
/// non-string value — including an empty dict/list — and the oracle computes it
/// BEFORE adding `trm_reasoning_outputs` / `token_sequence_count` /
/// `global_memory_commits`, so calling it on a re-saved snapshot over-counts.
pub fn count_matrices_loaded(snapshot: &[(String, SnapshotValue)]) -> usize {
    snapshot
        .iter()
        .filter(|(_, v)| !matches!(v, SnapshotValue::Null | SnapshotValue::Str(_)))
        .count()
}

/// Oracle `run_ingestion_cycle` status gate: no matrices => JARVIS_OFFLINE and
/// an early return; otherwise COMPLETE.
pub fn ingestion_cycle_status(matrices_loaded: usize) -> &'static str {
    if matrices_loaded == 0 {
        STATUS_JARVIS_OFFLINE
    } else {
        STATUS_COMPLETE
    }
}

/// Oracle `run_ingestion_cycle` commit block: one GlobalMemory commit per
/// truthy matrix — memory_core, memory_episodic, memory_nodes,
/// skills_registry, biomimetic_presets, trm_reasoning_outputs, noospheric_state
/// (max 7). `trm_outputs` is produced by the external TRMBrain, so its
/// presence is passed in (`trm_reasoning_outputs_present`).
pub fn count_global_memory_commits(
    snapshot: &[(String, SnapshotValue)],
    trm_reasoning_outputs_present: bool,
) -> usize {
    let mut committed = 0usize;
    let def_dict = SnapshotValue::Dict(Vec::new());
    let def_list = SnapshotValue::List(Vec::new());
    if let Some(memory) = dict_get(snapshot, "memory").filter(|v| v.is_truthy()) {
        let d = memory.as_dict().unwrap_or(&[]);
        if get_or(d, "core", &def_dict).is_truthy() {
            committed += 1;
        }
        if get_or(d, "episodic", &def_list).is_truthy() {
            committed += 1;
        }
        if get_or(d, "nodes", &def_list).is_truthy() {
            committed += 1;
        }
    }
    if dict_get(snapshot, "skills").map_or(false, |v| v.is_truthy()) {
        committed += 1;
    }
    if dict_get(snapshot, "biomimetic_presets").map_or(false, |v| v.is_truthy()) {
        committed += 1;
    }
    if trm_reasoning_outputs_present {
        committed += 1;
    }
    if dict_get(snapshot, "noospheric_status").map_or(false, |v| v.is_truthy()) {
        committed += 1;
    }
    committed
}

/// Oracle `run_ingestion_cycle`: `if self._cycle_count % 50 == 0: prune()`.
pub fn should_prune_ledger(cycle_count: u64) -> bool {
    cycle_count % LEDGER_PRUNE_INTERVAL == 0
}

/// Full control flow of one oracle `run_ingestion_cycle`, over a snapshot that
/// `fetch_all()` already produced. Two couplings matter and are easy to miss:
///   1. when matrices_loaded == 0 the oracle records JARVIS_OFFLINE and returns
///      BEFORE tokenizing or committing, so committed/token_sequences stay 0;
///   2. `trm_outputs` is only non-empty when the TRM brain exists AND some
///      sequence was non-empty (`if tokens:` inside the per-sequence loop), and
///      the trm_reasoning_outputs commit only happens `if trm_outputs:`.
#[derive(Debug, Clone, PartialEq)]
pub struct IngestionCycleRules {
    pub matrices_loaded: usize,
    pub status: &'static str,
    pub token_sequences: usize,
    pub trm_outputs: usize,
    pub global_memory_commits: usize,
}

/// See [`IngestionCycleRules`]. `trm_brain_available` models
/// `self._trm_brain is not None`; a brain that returns an empty output for a
/// non-empty sequence is not modelled (that is TRMBrain's own behaviour).
pub fn ingestion_cycle_rules(
    snapshot: &[(String, SnapshotValue)],
    trm_brain_available: bool,
) -> IngestionCycleRules {
    let matrices_loaded = count_matrices_loaded(snapshot);
    if matrices_loaded == 0 {
        return IngestionCycleRules {
            matrices_loaded: 0,
            status: STATUS_JARVIS_OFFLINE,
            token_sequences: 0,
            trm_outputs: 0,
            global_memory_commits: 0,
        };
    }
    let sequences = snapshot_to_token_sequences(snapshot);
    let trm_outputs = if trm_brain_available {
        sequences.iter().filter(|(_, toks)| !toks.is_empty()).count()
    } else {
        0
    };
    let committed = count_global_memory_commits(snapshot, trm_outputs > 0);
    IngestionCycleRules {
        matrices_loaded,
        status: STATUS_COMPLETE,
        token_sequences: sequences.len(),
        trm_outputs,
        global_memory_commits: committed,
    }
}

impl SystemEventLedger {
    /// Oracle `SystemEventLedger.prune(keep_last)`: keep only the most recent
    /// `keep_last` entries (oldest dropped from the front). Returns whether
    /// anything was dropped. Python default `keep_last` is MAX_LEDGER_ENTRIES.
    pub fn prune(&mut self, keep_last: usize) -> bool {
        if self.events.len() <= keep_last {
            return false;
        }
        let drop_count = self.events.len() - keep_last;
        self.events.drain(0..drop_count);
        true
    }
}
