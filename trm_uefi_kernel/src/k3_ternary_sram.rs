//! TRM Swarm OS — SRAM K3 TERNARY BINDING (no_std).
//! Unifies `k3_logic` (Kleene K3 ternary engine) with `sram_pattern_computer`
//! (L3 SRAM pattern computer / CAM) into a single content-addressable ternary
//! storage primitive: an SRAM-backed trit array in {-1,0,+1}, tagged lines for
//! pattern-content-addressable lookup, K3 conjunction over stored trits, and
//! utilization accounting.
//!
//! VOLTAGE ENERGIZATION — council 83563732.

extern crate alloc;
use alloc::vec::Vec;

use crate::k3_logic::{KleeneTernaryEngine, KleeneState, FALSE, TRUE, UNKNOWN};
use crate::sram_pattern_computer::{
    pattern_cellular_automaton, pattern_content_addressable, pattern_tcam_match,
};

/// SRAM-backed K3 trit plane size (one trit per cell, in {-1,0,+1}).
pub const SRAM_TRITS: usize = 1024;
/// Associative (CAM) line count over the plane (scaled from 64 to 256 lines).
pub const CAM_LINES: usize = 256;
/// Byte width of a CAM line tag.
pub const CAM_TAG_LEN: usize = 16;

/// Individual Content-Addressable Memory entry with LRU replacement metadata.
#[derive(Clone, Copy)]
pub struct CamEntry {
    pub tag: [u8; CAM_TAG_LEN],
    pub valid: bool,
    pub decision: i8,
    pub confidence: f32,
    pub access_tick: u64,
    pub hit_count: u64,
}

impl CamEntry {
    pub const fn empty() -> Self {
        Self {
            tag: [0u8; CAM_TAG_LEN],
            valid: false,
            decision: UNKNOWN,
            confidence: 0.0,
            access_tick: 0,
            hit_count: 0,
        }
    }
}

/// An SRAM K3 ternary bank: real Kleene trits + content-addressable tags.
pub struct K3Sram {
    /// The ternary trit plane. Each cell is +1 (TRUE), -1 (FALSE) or 0 (UNKNOWN).
    pub trits: [i8; SRAM_TRITS],
    /// CAM tag table: one tag per line, used for content-addressable lookup.
    pub tags: [[u8; CAM_TAG_LEN]; CAM_LINES],
    /// Fast associative table with LRU tracking and per-line decisions.
    pub entries: [CamEntry; CAM_LINES],
    /// Real Kleene engine shared for K3 operator evaluation.
    pub engine: KleeneTernaryEngine,
    /// Number of trits currently written in the plane.
    stored: usize,
    /// Whether a logical pattern has been stored (for "if there" gating).
    pattern_stored: bool,
    /// CAM lookup statistics.
    pub cam_lookups: u64,
    pub cam_hits: u64,
    pub evictions: u64,
    tick_counter: u64,
}

impl K3Sram {
    /// Build a fresh, all-UNKNOWN SRAM K3 bank.
    pub fn new() -> Self {
        K3Sram {
            trits: [UNKNOWN; SRAM_TRITS],
            tags: [[0u8; CAM_TAG_LEN]; CAM_LINES],
            entries: [CamEntry::empty(); CAM_LINES],
            engine: KleeneTernaryEngine::new(),
            stored: 0,
            pattern_stored: false,
            cam_lookups: 0,
            cam_hits: 0,
            evictions: 0,
            tick_counter: 0,
        }
    }

    /// Store an entry into the CAM associative table with LRU replacement.
    pub fn store_entry(&mut self, key: &[u8], decision: i8, confidence: f32) -> usize {
        self.tick_counter = self.tick_counter.wrapping_add(1);
        let mut padded = [0u8; CAM_TAG_LEN];
        let n = key.len().min(CAM_TAG_LEN);
        padded[..n].copy_from_slice(&key[..n]);

        // 1. If key already exists, update entry in place
        for i in 0..CAM_LINES {
            if self.entries[i].valid && self.entries[i].tag == padded {
                self.entries[i].decision = decision;
                self.entries[i].confidence = confidence;
                self.entries[i].access_tick = self.tick_counter;
                return i;
            }
        }

        // 2. Locate an empty line
        for i in 0..CAM_LINES {
            if !self.entries[i].valid {
                self.entries[i].tag = padded;
                self.entries[i].valid = true;
                self.entries[i].decision = decision;
                self.entries[i].confidence = confidence;
                self.entries[i].access_tick = self.tick_counter;
                self.entries[i].hit_count = 0;
                self.tags[i] = padded;
                return i;
            }
        }

        // 3. Table full -> LRU eviction
        let mut oldest_idx = 0;
        let mut oldest_tick = self.entries[0].access_tick;
        for i in 1..CAM_LINES {
            if self.entries[i].access_tick < oldest_tick {
                oldest_tick = self.entries[i].access_tick;
                oldest_idx = i;
            }
        }

        self.evictions += 1;
        self.entries[oldest_idx].tag = padded;
        self.entries[oldest_idx].valid = true;
        self.entries[oldest_idx].decision = decision;
        self.entries[oldest_idx].confidence = confidence;
        self.entries[oldest_idx].access_tick = self.tick_counter;
        self.entries[oldest_idx].hit_count = 0;
        self.tags[oldest_idx] = padded;
        oldest_idx
    }

    /// Look up an entry by key; on hit, updates LRU access tick and returns decision + confidence.
    pub fn lookup_entry(&mut self, key: &[u8]) -> Option<(i8, f32)> {
        self.cam_lookups += 1;
        self.tick_counter = self.tick_counter.wrapping_add(1);
        let mut padded = [0u8; CAM_TAG_LEN];
        let n = key.len().min(CAM_TAG_LEN);
        padded[..n].copy_from_slice(&key[..n]);

        for i in 0..CAM_LINES {
            if self.entries[i].valid && self.entries[i].tag == padded {
                self.cam_hits += 1;
                self.entries[i].hit_count += 1;
                self.entries[i].access_tick = self.tick_counter;
                return Some((self.entries[i].decision, self.entries[i].confidence));
            }
        }
        None
    }

    /// Store a ternary pattern into the plane and tag line 0 with `key` so the
    /// pattern becomes content-addressable. Returns the number of trits stored.
    pub fn store_pattern(&mut self, key: &[u8], pattern: &[i8]) -> usize {
        let n = pattern.len().min(SRAM_TRITS);
        for (i, &t) in pattern.iter().take(n).enumerate() {
            self.trits[i] = t;
        }
        self.stored = n;
        // Tag line 0 with the key (padded/truncated to tag width).
        let klen = key.len().min(CAM_TAG_LEN);
        self.tags[0][..klen].copy_from_slice(&key[..klen]);
        self.entries[0].tag[..klen].copy_from_slice(&key[..klen]);
        self.entries[0].valid = true;
        self.entries[0].decision = pattern.iter().copied().min().unwrap_or(UNKNOWN);
        self.entries[0].confidence = 1.0;
        self.pattern_stored = true;
        self.stored
    }

    /// Content-addressable lookup: count lines whose tag matches `key`.
    /// Reuses `sram_pattern_computer::pattern_content_addressable`.
    pub fn cam_lookup(&mut self, key: &[u8]) -> u64 {
        self.cam_lookups += 1;
        let rows: Vec<&[u8]> = self.tags.iter().map(|t| t.as_slice()).collect();
        // Pad query to CAM_TAG_LEN to match stored (padded) tags, so the
        // full-width length check in pattern_content_addressable passes.
        let mut padded = [0u8; CAM_TAG_LEN];
        let n = key.len().min(CAM_TAG_LEN);
        padded[..n].copy_from_slice(&key[..n]);
        let hits = pattern_content_addressable(&rows, &padded);
        if hits > 0 {
            self.cam_hits += 1;
        }
        hits
    }

    /// Boost tri-state; reuses `pattern_cellular_automaton` on the byte plane.
    pub fn cellular_evolve(&mut self) -> Vec<u8> {
        let bytes: Vec<u8> = self.trits.iter().map(|&t| (t + 1) as u8).collect();
        let evolved = pattern_cellular_automaton(&bytes);
        let n = evolved.len().min(SRAM_TRITS);
        for (i, &b) in evolved.iter().take(n).enumerate() {
            self.trits[i] = (b as i8 % 3) - 1;
        }
        evolved
    }

    /// Ternary CAM wildcard match against the plane; reuses `pattern_tcam_match`.
    /// `query` is the K3 query trit vector, `mask` marks wildcard positions (0).
    pub fn tcam_match(&self, query: &[i8], mask: &[u8]) -> bool {
        pattern_tcam_match(&self.trits, query, mask)
    }

    /// K3 conjunction over a window of stored trits with `operand`, in place.
    /// Uses the real Kleene AND (min). Returns a vector of resulting trits.
    pub fn k3_conjunction(&mut self, start: usize, len: usize, operand: i8) -> Vec<i8> {
        let end = (start + len).min(SRAM_TRITS);
        let mut out = Vec::with_capacity(end.saturating_sub(start));
        for i in start..end {
            let r = KleeneTernaryEngine::kleene_and(self.trits[i], operand);
            self.trits[i] = r;
            out.push(r);
        }
        out
    }

    /// Evaluate the conjunctive bypass decision over all stored trits. Returns
    /// the Kleene state; if no pattern was stored, reports an UNKNOWN gate.
    pub fn bypass_over_stored(&mut self) -> KleeneState {
        if !self.pattern_stored || self.stored == 0 {
            return KleeneState {
                trit: UNKNOWN,
                confidence: 0.0,
                ambiguity_ratio: 1.0,
            };
        }
        let window = &self.trits[..self.stored];
        let state = self.engine.bypass_decision(window);
        // Fold a K3 conjunction across the window first ("if there").
        // Reduce: all TRUE => TRUE, any FALSE => FALSE, else UNKNOWN.
        let conj = window
            .iter()
            .fold(TRUE, |acc, &t| KleeneTernaryEngine::kleene_and(acc, t));
        let _ = conj; // conjunction already reflected in bypass min; kept for clarity
        state
    }

    /// Utilization: fraction of non-UNKNOWN trits in the plane.
    pub fn utilization(&self) -> f32 {
        if self.trits.is_empty() {
            return 0.0;
        }
        self.trits
            .iter()
            .filter(|&&t| t != UNKNOWN)
            .count() as f32
            / SRAM_TRITS as f32
    }

    /// CAM hit ratio (0.0 when no lookups performed).
    pub fn cam_hit_ratio(&self) -> f32 {
        if self.cam_lookups == 0 {
            return 0.0;
        }
        self.cam_hits as f32 / self.cam_lookups as f32
    }

    /// Number of trits currently stored.
    pub fn stored(&self) -> usize {
        self.stored
    }
}