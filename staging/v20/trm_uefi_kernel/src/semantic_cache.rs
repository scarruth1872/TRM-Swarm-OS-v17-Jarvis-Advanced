//! TRM Swarm OS — Semantic Cache (no_std port).
//! 1:1 core semantics from swarm_v2/core/semantic_cache.py — Catalyst.
//! TTL cache with expiry check + LRU-ish eviction at capacity.

extern crate alloc;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

/// Cache constants (1:1 python).
pub const DEFAULT_TTL_SECONDS: u64 = 300;
pub const DEFAULT_MAX_ENTRIES: u64 = 256;

/// A cache entry.
#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub key: String,
    pub value: String,
    pub ttl_seconds: u64,
    pub stored_at: u64, // monotonic tick
}

/// Semantic cache.
#[derive(Debug, Clone)]
pub struct SemanticCache {
    pub entries: Vec<CacheEntry>,
    pub max_entries: u64,
    pub tick: u64,
    pub hits: u64,
    pub misses: u64,
}

impl SemanticCache {
    pub fn new(max_entries: u64) -> Self {
        SemanticCache {
            entries: Vec::new(),
            max_entries: if max_entries == 0 { DEFAULT_MAX_ENTRIES } else { max_entries },
            tick: 0,
            hits: 0,
            misses: 0,
        }
    }

    fn is_expired(&self, e: &CacheEntry) -> bool {
        self.tick.saturating_sub(e.stored_at) > e.ttl_seconds
    }

    /// Get a value; returns Option (expired entries are evicted).
    pub fn get(&mut self, key: &str) -> Option<String> {
        self.tick += 1;
        let mut found = None;
        let tick = self.tick;
        let key_owned = key.to_string();
        self.entries.retain(|e| {
            if e.key == key_owned {
                let expired = tick.saturating_sub(e.stored_at) > e.ttl_seconds;
                if expired {
                    false
                } else {
                    found = Some(e.value.clone());
                    true
                }
            } else {
                true
            }
        });
        if found.is_some() {
            self.hits += 1;
        } else {
            self.misses += 1;
        }
        found
    }

    /// Store a value; evicts oldest when over capacity.
    pub fn set(&mut self, key: &str, value: &str, ttl_seconds: u64) {
        self.tick += 1;
        self.entries.retain(|e| e.key != key);
        self.entries.push(CacheEntry {
            key: key.into(),
            value: value.into(),
            ttl_seconds: if ttl_seconds == 0 { DEFAULT_TTL_SECONDS } else { ttl_seconds },
            stored_at: self.tick,
        });
        while self.entries.len() as u64 > self.max_entries {
            self.entries.remove(0);
        }
    }

    /// Stats: (entries, hits, misses).
    pub fn get_stats(&self) -> (u64, u64, u64) {
        (self.entries.len() as u64, self.hits, self.misses)
    }
}

/// Singleton handle.
pub fn get_cache() -> u64 {
    0x5345_4D43_4143_4845
}

// ═════════════════════════════════════════════════════════════════════════════
// Faithful port of the swarm_v2/core/semantic_cache.py rules the generated body
// above did NOT carry: the sha256 query-hash cache key, the Jaccard similarity
// (strip/DLL path *and* the stopword-filtered path `_find_similar_entry` really
// uses), the two thresholds, the LRU access-order rule and the exact rounding of
// `get_stats`.  Names follow the Python oracle.  Nothing here duplicates an
// existing item.  no_std: SHA-256, floor and round are hand-rolled (std-only).
// ═════════════════════════════════════════════════════════════════════════════

use alloc::collections::BTreeSet;

/// `SemanticCache.__init__(max_size_mb=512)`
pub const DEFAULT_MAX_SIZE_MB: u64 = 512;

/// `SemanticCache.__init__(default_ttl_seconds=3600)`.
/// NOTE: differs from the legacy `DEFAULT_TTL_SECONDS` (300) above.
pub const ORACLE_DEFAULT_TTL_SECONDS: u64 = 3600;

/// `SemanticCache.__init__(similarity_threshold=0.85)`.  Reported by `get_stats`
/// only — it does NOT gate a hit.
pub const DEFAULT_SIMILARITY_THRESHOLD: f64 = 0.85;

/// Literal `0.95` hard-coded inside `_find_similar_entry` — the threshold that
/// actually decides a semantic hit, compared with `>=` (inclusive).
pub const SEMANTIC_MATCH_THRESHOLD: f64 = 0.95;

/// `hashlib.sha256(...).hexdigest()[:16]` — the cache key width.
pub const QUERY_HASH_HEX_LEN: usize = 16;

/// `strip_semantic_jaccard.rs`: `let mut buf1 = [0u64; 64]` — words past the
/// 64th are silently dropped by the native strip.
pub const STRIP_WORD_CAP: usize = 64;

/// `_find_similar_entry` STOP_WORDS (18 words, in source order).
pub const STOP_WORDS: [&str; 18] = [
    "a", "an", "the", "in", "on", "of", "to", "for", "with", "that", "this", "uses", "is", "are",
    "by", "as", "from", "at",
];

/// Characters removed from both ends by `w.strip(", .?!:;\"'")` in
/// `_find_similar_entry`.
pub const STRIP_PUNCTUATION: [char; 9] = [',', ' ', '.', '?', '!', ':', ';', '"', '\''];

/// Python `str.isspace()` == Unicode White_Space + U+001C..U+001F.
/// (`char::is_whitespace` alone misses the four separators Python counts as space.)
#[inline]
pub fn is_python_space(c: char) -> bool {
    c.is_whitespace() || ('\u{1c}'..='\u{1f}').contains(&c)
}

/// Python `str.strip()` with no argument (strips `str.isspace()` chars at both ends).
pub fn trim_python_whitespace(s: &str) -> &str {
    let start = s.find(|c: char| !is_python_space(c)).unwrap_or(s.len());
    let end = s
        .rfind(|c: char| !is_python_space(c))
        .map(|i| i + s[i..].chars().next().map(|c| c.len_utf8()).unwrap_or(0))
        .unwrap_or(start);
    &s[start..end.max(start)]
}

/// Python `query.lower().strip()` — the exact input to the query hash.
pub fn normalize_query(query: &str) -> String {
    let lowered = python_lower(query);
    trim_python_whitespace(&lowered).to_string()
}

/// Unicode `Lt` (titlecase): the only `Cased` characters that are neither
/// upper- nor lower-case.  Enumerated so `is_cased` needs no property table.
fn is_titlecase(c: char) -> bool {
    matches!(c,
        '\u{01c5}' | '\u{01c8}' | '\u{01cb}' | '\u{01f2}'
        | '\u{1f88}'..='\u{1f8f}' | '\u{1f98}'..='\u{1f9f}' | '\u{1fa8}'..='\u{1faf}'
        | '\u{1fbc}' | '\u{1fcc}' | '\u{1ffc}')
}

/// DerivedCoreProperties `Cased` ≈ Lu ∪ Ll ∪ Lt.  (`Other_Uppercase/Lowercase`
/// characters that could sit next to Greek text are Lm, hence Case_Ignorable and
/// skipped before this is consulted.)
pub fn is_cased(c: char) -> bool {
    c.is_uppercase() || c.is_lowercase() || is_titlecase(c)
}

/// DerivedCoreProperties `Case_Ignorable`, restricted to the ranges that actually
/// occur beside Latin/Greek text.  Every codepoint in these ranges has category
/// Mn/Me/Cf/Lm/Sk (exact); marks of other scripts are not covered.
pub fn is_case_ignorable(c: char) -> bool {
    matches!(c,
        '\u{0300}'..='\u{036f}'      // combining diacritical marks
        | '\u{0483}'..='\u{0489}'    // Cyrillic combining
        | '\u{00ad}'                 // soft hyphen (Cf)
        | '\u{200b}'..='\u{200f}'    // ZWSP..RLM (Cf)
        | '\u{2019}' | '\u{0027}'    // apostrophes (Word_Break=Single_Quote)
        | '\u{02b0}'..='\u{02ff}'    // spacing modifiers (Lm/Sk)
        | '\u{1ab0}'..='\u{1aff}' | '\u{1dc0}'..='\u{1dff}'
        | '\u{fe00}'..='\u{fe0f}' | '\u{fe20}'..='\u{fe2f}')
}

/// SpecialCasing `Final_Sigma`: true when `chars[i]` is `Σ` and a `Cased`
/// character precedes it (skipping Case_Ignorable) with none following.
fn final_sigma(chars: &[char], i: usize) -> bool {
    let before = chars[..i]
        .iter()
        .rev()
        .find(|c| !is_case_ignorable(**c))
        .map(|c| is_cased(*c))
        .unwrap_or(false);
    let after = chars[i + 1..]
        .iter()
        .find(|c| !is_case_ignorable(**c))
        .map(|c| is_cased(*c))
        .unwrap_or(false);
    before && !after
}

/// Python `str.lower()`: Unicode full lowercase *including* the context-sensitive
/// Final_Sigma mapping (`Σ` → `ς` word-finally) that per-char `to_lowercase`
/// cannot see.  Used by the hash and by both set-based similarity paths — the
/// native strip is byte-level and deliberately not affected.
pub fn python_lower(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(s.len());
    for (i, &c) in chars.iter().enumerate() {
        if c == '\u{3a3}' {
            out.push(if final_sigma(&chars, i) {
                '\u{3c2}'
            } else {
                '\u{3c3}'
            });
        } else {
            out.extend(c.to_lowercase());
        }
    }
    out
}

/// `compute_query_hash` — sha256 of the normalized query, first 16 hex chars.
/// This is the cache key rule (`key = query_hash` in `set`).
pub fn compute_query_hash(query: &str) -> String {
    let normalized = normalize_query(query);
    let hex = sha256_hex(normalized.as_bytes());
    hex[..QUERY_HASH_HEX_LEN].to_string()
}

/// Lower-case hex of a byte slice.
fn to_hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}

/// SHA-256 round constants.
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

/// SHA-256 (FIPS 180-4), hand-rolled: `std` has no digest and `sha2` is not a
/// dependency of this no_std kernel.  Returns the 64-char lowercase hex digest
/// that Python's `hashlib.sha256(...).hexdigest()` produces.
pub fn sha256_hex(data: &[u8]) -> String {
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];

    let bit_len = (data.len() as u64).wrapping_mul(8);
    let mut msg: Vec<u8> = Vec::with_capacity(data.len() + 72);
    msg.extend_from_slice(data);
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());

    let mut w = [0u32; 64];
    for chunk in msg.chunks(64) {
        for i in 0..16 {
            w[i] = u32::from_be_bytes([chunk[4 * i], chunk[4 * i + 1], chunk[4 * i + 2], chunk[4 * i + 3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }

        let mut a = h[0];
        let mut b = h[1];
        let mut c = h[2];
        let mut d = h[3];
        let mut e = h[4];
        let mut f = h[5];
        let mut g = h[6];
        let mut hh = h[7];

        for i in 0..64 {
            let big_s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = hh
                .wrapping_add(big_s1)
                .wrapping_add(ch)
                .wrapping_add(SHA256_K[i])
                .wrapping_add(w[i]);
            let big_s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = big_s0.wrapping_add(maj);

            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }

        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }

    let mut digest = [0u8; 32];
    for (i, word) in h.iter().enumerate() {
        digest[4 * i..4 * i + 4].copy_from_slice(&word.to_be_bytes());
    }
    to_hex_lower(&digest)
}

/// The strip's `fnv1a_hash` — ASCII-uppercase bytes folded to lowercase first.
#[inline]
pub fn fnv1a_hash64(word: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for &b in word {
        let lower = if b >= b'A' && b <= b'Z' { b + 32 } else { b };
        h ^= lower as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

/// The strip's `extract_word_hashes`: split on runs of *ASCII* whitespace (note:
/// `is_ascii_whitespace` excludes `\x0b`), hash each word with FNV-1a, stop after
/// 64 words, sort and de-duplicate the hash set.
pub fn strip_word_hashes(bytes: &[u8]) -> Vec<u64> {
    let mut out: Vec<u64> = Vec::new();
    let mut idx = 0usize;
    let len = bytes.len();
    while idx < len && out.len() < STRIP_WORD_CAP {
        while idx < len && bytes[idx].is_ascii_whitespace() {
            idx += 1;
        }
        if idx >= len {
            break;
        }
        let start = idx;
        while idx < len && !bytes[idx].is_ascii_whitespace() {
            idx += 1;
        }
        out.push(fnv1a_hash64(&bytes[start..idx]));
    }
    out.sort_unstable();
    out.dedup();
    out
}

/// `_compute_semantic_similarity` — the accelerated branch actually taken when
/// `swarm_v2_artifacts/native_voltage_strips/strip_semantic_jaccard.dll` loads.
/// Word-hash Jaccard: `|A ∩ B| / |A ∪ B|`, 0.0 when either side is empty.
pub fn semantic_similarity(query1: &str, query2: &str) -> f64 {
    let a = strip_word_hashes(query1.as_bytes());
    let b = strip_word_hashes(query2.as_bytes());
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let mut i = 0usize;
    let mut j = 0usize;
    let mut inter = 0usize;
    while i < a.len() && j < b.len() {
        if a[i] == b[j] {
            inter += 1;
            i += 1;
            j += 1;
        } else if a[i] < b[j] {
            i += 1;
        } else {
            j += 1;
        }
    }
    let union = a.len() + b.len() - inter;
    if union > 0 {
        inter as f64 / union as f64
    } else {
        0.0
    }
}

/// Python `str.split()` (splits on `str.isspace()` runs, `\x1c`..`\x1f` included).
pub fn split_python_whitespace(s: &str) -> Vec<&str> {
    let mut out: Vec<&str> = Vec::new();
    let bytes = s.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        // advance past separators
        let mut j = i;
        while j < bytes.len() {
            let c = s[j..].chars().next().unwrap();
            if is_python_space(c) {
                j += c.len_utf8();
            } else {
                break;
            }
        }
        i = j;
        if i >= bytes.len() {
            break;
        }
        let start = i;
        let mut end = i;
        while end < bytes.len() {
            let c = s[end..].chars().next().unwrap();
            if is_python_space(c) {
                break;
            }
            end += c.len_utf8();
        }
        out.push(&s[start..end]);
        i = end;
    }
    out
}

/// `_compute_semantic_similarity` Python fallback branch (reachable only when the
/// strip DLL is absent): plain `set(query.lower().split())` Jaccard, no stopwords,
/// no punctuation stripping.  Distinct words never collide here, unlike the strip.
pub fn semantic_similarity_py_fallback(query1: &str, query2: &str) -> f64 {
    let lower1 = python_lower(query1);
    let lower2 = python_lower(query2);
    let w1: BTreeSet<&str> = split_python_whitespace(&lower1).into_iter().collect();
    let w2: BTreeSet<&str> = split_python_whitespace(&lower2).into_iter().collect();
    if w1.is_empty() || w2.is_empty() {
        return 0.0;
    }
    let inter = w1.intersection(&w2).count();
    let union = w1.union(&w2).count();
    if union > 0 {
        inter as f64 / union as f64
    } else {
        0.0
    }
}

/// `w.strip(", .?!:;\"'")` — Python strips any of those chars from both ends.
pub fn strip_entry_punctuation(word: &str) -> &str {
    word.trim_matches(|c: char| STRIP_PUNCTUATION.contains(&c))
}

/// The stopword/punctuation-filtered word set `_find_similar_entry` builds for
/// each side (`STOP_WORDS` removed afterwards).
pub fn filtered_word_set(query: &str) -> BTreeSet<String> {
    let lower = python_lower(query);
    let mut set: BTreeSet<String> = BTreeSet::new();
    for w in split_python_whitespace(&lower) {
        let cleaned = strip_entry_punctuation(w);
        if !STOP_WORDS.contains(&cleaned) {
            set.insert(cleaned.to_string());
        }
    }
    set
}

/// The similarity `_find_similar_entry` computes: stopword-filtered Jaccard.
/// `None` mirrors the Python `continue` when either filtered set is empty
/// (such a pair can never match, whatever the threshold).
pub fn semantic_similarity_filtered(query1: &str, query2: &str) -> Option<f64> {
    let w1 = filtered_word_set(query1);
    let w2 = filtered_word_set(query2);
    if w1.is_empty() || w2.is_empty() {
        return None;
    }
    let inter = w1.intersection(&w2).count();
    let union = w1.union(&w2).count();
    Some(if union > 0 { inter as f64 / union as f64 } else { 0.0 })
}

/// The `sim >= 0.95` decision of `_find_similar_entry` (inclusive, so exactly
/// 0.95 IS a hit), and `false` for the empty-set `continue` case.
pub fn is_semantic_match(query1: &str, query2: &str) -> bool {
    match semantic_similarity_filtered(query1, query2) {
        Some(sim) => sim >= SEMANTIC_MATCH_THRESHOLD,
        None => false,
    }
}

/// `CacheEntry.is_expired`: `(now - created_at) > ttl_seconds`, i.e. STRICT —
/// an entry exactly `ttl_seconds` old is still live.  (The tick-based
/// `SemanticCache::is_expired` above applies the same strict rule.)
pub fn is_expired_after(age_seconds: f64, ttl_seconds: f64) -> bool {
    age_seconds > ttl_seconds
}

/// `get()`: on a hit the key is removed from `_access_order` (first occurrence
/// only — `list.remove`) and re-appended, which is what makes eviction LRU.
pub fn lru_touch(access_order: &mut Vec<String>, key: &str) {
    if let Some(pos) = access_order.iter().position(|k| k == key) {
        access_order.remove(pos);
    }
    access_order.push(key.to_string());
}

/// `set()`: the key is appended unconditionally — a re-`set` leaves a DUPLICATE
/// in `_access_order`.
pub fn lru_record_insert(access_order: &mut Vec<String>, key: &str) {
    access_order.push(key.to_string());
}

/// `_evict_if_needed`: `self._access_order.pop(0)` — the victim is the least
/// recently touched key at the front of the list (the Python loop re-checks the
/// byte budget between pops; the size accounting itself comes from
/// `json.dumps(...)`, i.e. host serialization, so only the order rule is ported).
pub fn lru_pop_victim(access_order: &mut Vec<String>) -> Option<String> {
    if access_order.is_empty() {
        None
    } else {
        Some(access_order.remove(0))
    }
}

/// Hand-rolled `floor` (std-only in Rust) for the rounding below.
fn floor_f64(x: f64) -> f64 {
    let t = x as i64 as f64;
    if x < 0.0 && t != x {
        t - 1.0
    } else {
        t
    }
}

/// Hand-rolled `round(value, places)` reproducing Python's ties-to-even
/// behaviour on the binary double (std's `f64::round` is half-away-from-zero and
/// std-only besides).
pub fn round_half_even(value: f64, places: u32) -> f64 {
    let mut scale = 1.0f64;
    for _ in 0..places {
        scale *= 10.0;
    }
    let scaled = value * scale;
    let floor = floor_f64(scaled);
    let frac = scaled - floor;
    let rounded = if frac > 0.5 {
        floor + 1.0
    } else if frac < 0.5 {
        floor
    } else if (floor as i64) % 2 == 0 {
        floor
    } else {
        floor + 1.0
    };
    rounded / scale
}

/// `get_stats()["hit_rate_percent"] = round(hits / (hits + misses) * 100, 2)`,
/// or 0 when no query has been seen.
pub fn hit_rate_percent(hits: u64, misses: u64) -> f64 {
    let total = hits + misses;
    if total == 0 {
        0.0
    } else {
        round_half_even(hits as f64 / total as f64 * 100.0, 2)
    }
}
