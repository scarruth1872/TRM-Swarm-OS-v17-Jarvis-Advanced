//! TRM Swarm OS — SRAM K3 TERNARY BINDING (no_std).
//! Unifies `k3_logic` (Kleene K3 ternary engine) with `sram_pattern_computer`
//! (L3 SRAM pattern computer / CAM) into a single content-addressable ternary
//! storage primitive: an SRAM-backed trit array in {-1,0,+1}, tagged lines for
//! pattern-content-addressable lookup, K3 conjunction over stored trits, and
//! utilization accounting.
//!
//! VOLTAGE ENERGIZATION — council 83563732.
//!
//! MULTI-ENTRY STORE + GENUINE ASSOCIATIVE LOOKUP
//! ----------------------------------------------
//! Two storage layouts are kept in lockstep:
//!   1. `planes` — row-major stride window (one resident pattern per CAM line).
//!   2. `cam_bits` — BIT-SLICED column plane. For every trit position and every
//!      trit value in {-1,0,+1} a 256-bit bitmap of matching lines is kept.
//! A query ANDs the bitmaps for its constrained positions: one pass over
//! positions produces the full set of matching lines without a per-line scan.
//! That is genuine content-addressable matching emulated in parallel, and it
//! replaces the previous Python-style `for i in 0..CAM_LINES` linear scan.
//!
//! Tag lookups use an open bucket index (`bucket_head` + `next_line` chaining),
//! so `store_entry`/`lookup_entry` are expected O(1) instead of O(256).

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
/// Max trits per resident pattern (stride window into `planes`).
pub const PATTERN_STRIDE: usize = 64;
/// 64-bit words per CAM bitmap (256 lines -> 4 words).
pub const CAM_WORDS: usize = CAM_LINES / 64;
/// Tag hash buckets (2x lines, load factor 0.5 expected).
pub const TAG_BUCKETS: usize = 512;

const _: () = assert!(CAM_LINES % 64 == 0);

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
    /// Stride-window planes: one resident trit pattern per CAM line (row layout).
    pub planes: Vec<i8>,
    /// Bit-sliced column plane: `cam_bits[(p*3 + v)*CAM_WORDS + w]`.
    /// Bit set => line (w*64 + b) has trit value `v-1` at position `p`.
    pub cam_bits: Vec<u64>,
    /// Bitmap per position of lines that actually carry a trit there.
    pub cam_present: Vec<u64>,
    /// Bitmap per pattern length: lines whose stored length == index.
    pub len_bits: Vec<u64>,
    /// Stored trit length per CAM line (0 = line unused).
    pub lens: [u16; CAM_LINES],
    /// Number of patterns resident in `planes`.
    pub patterns_stored: usize,
    /// Open bucket index: head of each tag-hash chain (line+1, 0 = empty).
    pub bucket_head: [u16; TAG_BUCKETS],
    /// Next line in the same bucket chain (line+1, 0 = end).
    pub next_line: [u16; CAM_LINES],
    /// CAM lookup statistics.
    pub cam_lookups: u64,
    pub cam_hits: u64,
    pub evictions: u64,
    /// Bit-sliced matches performed (each = one whole-CAM pass).
    pub bit_slice_matches: u64,
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
            planes: alloc::vec![UNKNOWN; CAM_LINES * PATTERN_STRIDE],
            cam_bits: alloc::vec![0u64; PATTERN_STRIDE * 3 * CAM_WORDS],
            cam_present: alloc::vec![0u64; PATTERN_STRIDE * CAM_WORDS],
            len_bits: alloc::vec![0u64; PATTERN_STRIDE * CAM_WORDS],
            lens: [0u16; CAM_LINES],
            patterns_stored: 0,
            bucket_head: [0u16; TAG_BUCKETS],
            next_line: [0u16; CAM_LINES],
            cam_lookups: 0,
            cam_hits: 0,
            evictions: 0,
            bit_slice_matches: 0,
            tick_counter: 0,
        }
    }

    // ---- tag hash index ---------------------------------------------------

    #[inline]
    fn tag_hash(tag: &[u8; CAM_TAG_LEN]) -> usize {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut i = 0;
        while i < CAM_TAG_LEN {
            h ^= tag[i] as u64;
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
            i += 1;
        }
        (h as usize) & (TAG_BUCKETS - 1)
    }

    /// Insert `line` into the tag bucket chain. Caller must have set the tag.
    fn idx_insert(&mut self, line: usize) {
        let b = Self::tag_hash(&self.entries[line].tag);
        self.next_line[line] = self.bucket_head[b];
        self.bucket_head[b] = (line + 1) as u16;
    }

    /// Remove `line` from its tag bucket chain (no-op when absent).
    fn idx_remove(&mut self, line: usize) {
        let b = Self::tag_hash(&self.entries[line].tag);
        let target = (line + 1) as u16;
        let mut cur = self.bucket_head[b];
        let mut prev: u16 = 0;
        while cur != 0 {
            let li = (cur - 1) as usize;
            if cur == target {
                if prev == 0 {
                    self.bucket_head[b] = self.next_line[li];
                } else {
                    self.next_line[(prev - 1) as usize] = self.next_line[li];
                }
                self.next_line[li] = 0;
                return;
            }
            prev = cur;
            cur = self.next_line[li];
        }
    }

    /// O(1)-expected associative tag lookup through the bucket index.
    pub fn idx_find(&self, tag: &[u8; CAM_TAG_LEN]) -> Option<usize> {
        let b = Self::tag_hash(tag);
        let mut cur = self.bucket_head[b];
        while cur != 0 {
            let li = (cur - 1) as usize;
            if self.entries[li].valid && self.entries[li].tag == *tag {
                return Some(li);
            }
            cur = self.next_line[li];
        }
        None
    }

    /// Pad/truncate an arbitrary key to a CAM tag.
    #[inline]
    pub fn pad_key(key: &[u8]) -> [u8; CAM_TAG_LEN] {
        let mut padded = [0u8; CAM_TAG_LEN];
        let n = key.len().min(CAM_TAG_LEN);
        padded[..n].copy_from_slice(&key[..n]);
        padded
    }

    // ---- bit-sliced CAM maintenance ---------------------------------------

    /// Write a pattern into a line's stride window AND its bit-sliced column.
    fn write_plane(&mut self, line: usize, pattern: &[i8], len: usize) {
        let old_len = self.lens[line] as usize;
        if old_len > 0 {
            // Clear the previous length bucket and previous trit bits.
            let w = line / 64;
            let mask = !(1u64 << (line % 64));
            if old_len <= PATTERN_STRIDE {
                self.len_bits[(old_len - 1) * CAM_WORDS + w] &= mask;
            }
            let old = old_len.min(PATTERN_STRIDE);
            for p in 0..old {
                for v in 0..3 {
                    self.cam_bits[(p * 3 + v) * CAM_WORDS + w] &= mask;
                }
                self.cam_present[p * CAM_WORDS + w] &= mask;
            }
        }

        let base = line * PATTERN_STRIDE;
        for i in 0..PATTERN_STRIDE {
            self.planes[base + i] = if i < len { pattern[i] } else { UNKNOWN };
        }
        self.lens[line] = len as u16;

        let w = line / 64;
        let bit = 1u64 << (line % 64);
        for p in 0..len {
            let t = pattern[p];
            // Clamp into the ternary alphabet; anything else is treated as UNKNOWN.
            let v = if t > 0 { 2 } else if t < 0 { 0 } else { 1 };
            self.cam_bits[(p * 3 + v) * CAM_WORDS + w] |= bit;
            self.cam_present[p * CAM_WORDS + w] |= bit;
        }
        if len > 0 && len <= PATTERN_STRIDE {
            self.len_bits[(len - 1) * CAM_WORDS + w] |= bit;
        }
    }

    /// Rebuild all bit-slices from `planes`/`lens` (repair / verification hook).
    pub fn rebuild_bits(&mut self) {
        for b in self.cam_bits.iter_mut() {
            *b = 0;
        }
        for b in self.cam_present.iter_mut() {
            *b = 0;
        }
        for b in self.len_bits.iter_mut() {
            *b = 0;
        }
        for line in 0..CAM_LINES {
            let len = self.lens[line] as usize;
            if len == 0 {
                continue;
            }
            let base = line * PATTERN_STRIDE;
            let w = line / 64;
            let bit = 1u64 << (line % 64);
            for p in 0..len.min(PATTERN_STRIDE) {
                let t = self.planes[base + p];
                let v = if t > 0 { 2 } else if t < 0 { 0 } else { 1 };
                self.cam_bits[(p * 3 + v) * CAM_WORDS + w] |= bit;
                self.cam_present[p * CAM_WORDS + w] |= bit;
            }
            if len <= PATTERN_STRIDE {
                self.len_bits[(len - 1) * CAM_WORDS + w] |= bit;
            }
        }
    }

    // ---- tag entry store / lookup -----------------------------------------

    /// Store an entry into the CAM associative table with LRU replacement.
    pub fn store_entry(&mut self, key: &[u8], decision: i8, confidence: f32) -> usize {
        self.tick_counter = self.tick_counter.wrapping_add(1);
        let padded = Self::pad_key(key);

        // 1. Existing key -> update in place (O(1) via bucket index).
        if let Some(i) = self.idx_find(&padded) {
            self.entries[i].decision = decision;
            self.entries[i].confidence = confidence;
            self.entries[i].access_tick = self.tick_counter;
            return i;
        }

        // 2. Empty line.
        for i in 0..CAM_LINES {
            if !self.entries[i].valid {
                self.entries[i].tag = padded;
                self.entries[i].valid = true;
                self.entries[i].decision = decision;
                self.entries[i].confidence = confidence;
                self.entries[i].access_tick = self.tick_counter;
                self.entries[i].hit_count = 0;
                self.tags[i] = padded;
                self.idx_insert(i);
                return i;
            }
        }

        // 3. Full -> LRU eviction.
        let mut oldest_idx = 0;
        let mut oldest_tick = self.entries[0].access_tick;
        for i in 1..CAM_LINES {
            if self.entries[i].access_tick < oldest_tick {
                oldest_tick = self.entries[i].access_tick;
                oldest_idx = i;
            }
        }

        self.evictions += 1;
        self.idx_remove(oldest_idx);
        self.entries[oldest_idx].tag = padded;
        self.entries[oldest_idx].valid = true;
        self.entries[oldest_idx].decision = decision;
        self.entries[oldest_idx].confidence = confidence;
        self.entries[oldest_idx].access_tick = self.tick_counter;
        self.entries[oldest_idx].hit_count = 0;
        self.tags[oldest_idx] = padded;
        self.idx_insert(oldest_idx);
        oldest_idx
    }

    /// Look up an entry by key; on hit, updates LRU access tick and returns decision + confidence.
    pub fn lookup_entry(&mut self, key: &[u8]) -> Option<(i8, f32)> {
        self.cam_lookups += 1;
        self.tick_counter = self.tick_counter.wrapping_add(1);
        let padded = Self::pad_key(key);
        let i = self.idx_find(&padded)?;
        self.cam_hits += 1;
        self.entries[i].hit_count += 1;
        self.entries[i].access_tick = self.tick_counter;
        Some((self.entries[i].decision, self.entries[i].confidence))
    }

    /// Number of lines reachable through the tag index (integrity check).
    pub fn idx_len(&self) -> usize {
        let mut n = 0;
        for b in 0..TAG_BUCKETS {
            let mut cur = self.bucket_head[b];
            while cur != 0 {
                n += 1;
                cur = self.next_line[(cur - 1) as usize];
            }
        }
        n
    }

    // ---- single-plane API --------------------------------------------------

    /// Store a ternary pattern into the plane and tag line 0 with `key` so the
    /// pattern becomes content-addressable. Returns the number of trits stored.
    pub fn store_pattern(&mut self, key: &[u8], pattern: &[i8]) -> usize {
        let n = pattern.len().min(SRAM_TRITS);
        for (i, &t) in pattern.iter().take(n).enumerate() {
            self.trits[i] = t;
        }
        self.stored = n;
        let padded = Self::pad_key(key);
        self.tags[0] = padded;
        if self.entries[0].valid {
            self.idx_remove(0);
        }
        self.entries[0].tag = padded;
        self.entries[0].valid = true;
        self.entries[0].decision = pattern.iter().copied().min().unwrap_or(UNKNOWN);
        self.entries[0].confidence = 1.0;
        self.idx_insert(0);
        self.pattern_stored = true;
        self.stored
    }

    /// Content-addressable lookup: count lines whose tag matches `key`.
    /// Reuses `sram_pattern_computer::pattern_content_addressable`.
    pub fn cam_lookup(&mut self, key: &[u8]) -> u64 {
        self.cam_lookups += 1;
        let rows: Vec<&[u8]> = self.tags.iter().map(|t| t.as_slice()).collect();
        let padded = Self::pad_key(key);
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
        let _ = FALSE;
        state
    }

    // ---- multi-entry store --------------------------------------------------

    /// Store a pattern into a CAM line so MANY patterns stay resident.
    /// Unlike `store_pattern` (one plane, line 0 only).
    pub fn store_pattern_multi(&mut self, key: &[u8], pattern: &[i8]) -> usize {
        self.tick_counter = self.tick_counter.wrapping_add(1);
        let padded = Self::pad_key(key);
        let len = pattern.len().min(PATTERN_STRIDE);

        if let Some(i) = self.idx_find(&padded) {
            self.write_plane(i, pattern, len);
            self.entries[i].access_tick = self.tick_counter;
            self.entries[i].decision = pattern.iter().copied().min().unwrap_or(UNKNOWN);
            return i;
        }

        let mut idx = CAM_LINES;
        for i in 0..CAM_LINES {
            if self.lens[i] == 0 {
                idx = i;
                break;
            }
        }
        if idx == CAM_LINES {
            let (mut o, mut ot) = (0usize, self.entries[0].access_tick);
            for i in 1..CAM_LINES {
                if self.entries[i].access_tick < ot {
                    ot = self.entries[i].access_tick;
                    o = i;
                }
            }
            idx = o;
            self.evictions += 1;
        } else {
            self.patterns_stored += 1;
        }

        if self.entries[idx].valid {
            self.idx_remove(idx);
        }
        self.write_plane(idx, pattern, len);
        self.entries[idx].tag = padded;
        self.entries[idx].valid = true;
        self.entries[idx].decision = pattern.iter().copied().min().unwrap_or(UNKNOWN);
        self.entries[idx].confidence = 1.0;
        self.entries[idx].access_tick = self.tick_counter;
        self.entries[idx].hit_count = 0;
        self.tags[idx] = padded;
        self.idx_insert(idx);
        idx
    }

    /// Scalar wildcard match of `query` against ONE resident line (reference path).
    pub fn tcam_match_line(&self, line: usize, query: &[i8], mask: &[u8]) -> bool {
        let len = self.lens[line] as usize;
        if len == 0 || len != query.len() {
            return false;
        }
        let base = line * PATTERN_STRIDE;
        for i in 0..len {
            if mask.get(i).copied().unwrap_or(1) == 0 {
                continue;
            }
            if self.planes[base + i] != query[i] {
                return false;
            }
        }
        true
    }

    /// Reference implementation: linear scan over every CAM line.
    /// Kept for equivalence testing against the bit-sliced path.
    pub fn tcam_match_any_scalar(&mut self, query: &[i8], mask: &[u8]) -> Option<usize> {
        for i in 0..CAM_LINES {
            if !self.entries[i].valid || self.lens[i] == 0 {
                continue;
            }
            if self.tcam_match_line(i, query, mask) {
                return Some(i);
            }
        }
        None
    }

    /// Bit-sliced whole-CAM match. Returns a 256-bit bitmap of matching lines.
    /// One pass over constrained positions; no per-line loop.
    pub fn cam_match_bits(&self, query: &[i8], mask: &[u8]) -> [u64; CAM_WORDS] {
        let len = query.len().min(PATTERN_STRIDE);
        let mut bm = [0u64; CAM_WORDS];
        if len == 0 {
            return bm;
        }
        // Seed with the length bucket: only lines whose stored length == len.
        let w0 = (len - 1) * CAM_WORDS;
        for w in 0..CAM_WORDS {
            bm[w] = self.len_bits[w0 + w];
        }
        for p in 0..len {
            if mask.get(p).copied().unwrap_or(1) == 0 {
                continue;
            }
            let t = query[p];
            let v = if t > 0 {
                2
            } else if t < 0 {
                0
            } else {
                1
            };
            let base = (p * 3 + v) * CAM_WORDS;
            let mut any = false;
            for w in 0..CAM_WORDS {
                bm[w] &= self.cam_bits[base + w];
                if bm[w] != 0 {
                    any = true;
                }
            }
            if !any {
                return [0u64; CAM_WORDS];
            }
        }
        bm
    }

    /// Genuine associative lookup: match `query` against ALL resident patterns
    /// via the bit-sliced plane. Returns the first matching line index.
    pub fn tcam_match_any(&mut self, query: &[i8], mask: &[u8]) -> Option<usize> {
        self.cam_lookups += 1;
        self.tick_counter = self.tick_counter.wrapping_add(1);
        self.bit_slice_matches += 1;
        let bm = self.cam_match_bits(query, mask);
        for w in 0..CAM_WORDS {
            let mut word = bm[w];
            if word != 0 {
                while word != 0 {
                    let b = word.trailing_zeros() as usize;
                    let line = w * 64 + b;
                    if line < CAM_LINES {
                        self.cam_hits += 1;
                        self.entries[line].hit_count += 1;
                        self.entries[line].access_tick = self.tick_counter;
                        return Some(line);
                    }
                    word &= word - 1;
                }
            }
        }
        None
    }

    /// All matching lines (ascending), for multi-hit queries.
    pub fn tcam_match_all(&mut self, query: &[i8], mask: &[u8], out: &mut Vec<usize>) -> usize {
        self.cam_lookups += 1;
        self.tick_counter = self.tick_counter.wrapping_add(1);
        self.bit_slice_matches += 1;
        out.clear();
        let bm = self.cam_match_bits(query, mask);
        for w in 0..CAM_WORDS {
            let mut word = bm[w];
            while word != 0 {
                let b = word.trailing_zeros() as usize;
                let line = w * 64 + b;
                if line < CAM_LINES {
                    out.push(line);
                }
                word &= word - 1;
            }
        }
        if !out.is_empty() {
            self.cam_hits += 1;
        }
        out.len()
    }

    /// Patterns currently resident in the CAM (lines with length > 0).
    pub fn patterns_resident(&self) -> usize {
        self.lens.iter().filter(|&&l| l > 0).count()
    }

    /// Utilization: fraction of non-UNKNOWN trits in the plane.
    pub fn utilization(&self) -> f32 {
        if self.trits.is_empty() {
            return 0.0;
        }
        self.trits.iter().filter(|&&t| t != UNKNOWN).count() as f32 / SRAM_TRITS as f32
    }

    /// Utilization of the multi-entry CAM plane (resident trits / capacity).
    pub fn cam_utilization(&self) -> f32 {
        let used: usize = self.lens.iter().map(|&l| l as usize).sum();
        used as f32 / (CAM_LINES * PATTERN_STRIDE) as f32
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
