//! TRM Swarm OS — TurboVec Memory (no_std port).
//! 1:1 behaviour port of `swarm_v2/core/turbovec_memory.py` (`TurboVecMemory`)
//! — Catalyst. This EXTENDS the pre-port stub instead of discarding it: the
//! stub's `MemoryEvent`, `TurbovecMemory`, `CHUNK_SIZE`, `COMPRESS_LEVEL`,
//! `MEMORY_TTL_TICKS`, `chunk_count()` and `get_turbovec_memory()` surface is
//! kept, and the python behaviour that was missing is added on top
//! (total_memories / total_events / add / get / query / query_by_agent /
//! log_event / get_stats / compact / singleton semantics).
//!
//! ── no_std adaptations (each is a deliberate boundary, not a shortcut) ──────
//! * FILESYSTEM → none.  The python original persists every entry to
//!   `turbo_memory.jsonl.gz` / `turbo_events.jsonl.gz` (gzip + fsync append).
//!   The kernel has no FS and no zlib, so the store is purely in memory and the
//!   persistence boundary is `export_jsonl` / `export_events_jsonl` (one
//!   `json.dumps` object per line, python's default ", " / ": " separators) and
//!   `restore_jsonl` / `restore_events_jsonl` (python `_load()`: one JSON object
//!   per line, malformed lines skipped, index rebuilt afterwards). The caller
//!   does the compression; COMPRESS_LEVEL is retained for API parity but unused.
//!   `get_stats().file_size_mb` reports the size of the exported memory payload,
//!   which is the port's stand-in for `memory_file.stat().st_size`.
//! * DATETIME → `at_ns: u64` nanoseconds.  Python used `time.time()` for the
//!   entry-id seed and `datetime.now().isoformat()` for the stored timestamp,
//!   then re-derived age from that ISO string in `query()` and sorted by it in
//!   `query_by_agent()`. The port stores ONE integer `at_ns` and uses it for all
//!   three, so no ISO string is materialised and no calendar maths is needed:
//!       id seed   : f"{content}{int(time.time())}{len(cache)}"
//!       timestamp : at_ns
//!       age_hours : (now_ns - at_ns) / 3.6e12
//!   Writers pass `at_ns`; readers pass `now_ns` (`query_at`). The clock is
//!   therefore explicit and the module is deterministic and testable.
//! * THREADING → none.  Python guarded every mutation with a `threading.Lock`;
//!   the UEFI payload is single-threaded (the same assumption agent_runtime.rs
//!   documents for its `get_runtime()` singleton).
//! * f64::round / f64::sqrt do not exist in no_std. `round_half_even` below
//!   mirrors python's `round(x, 2)` (ties-to-EVEN), which is why
//!   `float_math::round_f64` (ties away from zero) is not used here: they differ
//!   at exact .xx5 ties. No parity path needs sqrt.
//! * STRING SLICING → `trunc_chars`.  Python `content[:10000]` slices CODE
//!   POINTS in py3; Rust `&s[..n]` is BYTE slicing and panics on a non-UTF8
//!   boundary, so every truncation goes through `trunc_chars` (`chars().take(n)`).
//! * ENTRY IDS → `crate::hmac_sha256::sha256` (FIPS 180-4) plus a lowercase-hex
//!   prefix, byte-identical to `hashlib.sha256(...).hexdigest()[:16]` / `[:12]`.
//!
//! The TurboVec vector primitives are NOT reimplemented here: the ternary/SSM
//! layer at the bottom of the file calls the sibling `turbovec.rs`
//! (`DIM`, `TurboVec158Quantizer` trit packing, `FemtosecondGenomicSSMScanner`,
//! femto cosine). That layer is ADDITIVE — no parity path calls it. See the
//! section comment before `embed_ternary`.
//!
//! NOT ported (no equivalent in no_std): the `__main__` self-test (host-only),
//! gzip framing, `os.fsync`, `print`-based error reporting in `_atomic_append`
//! (returns in-memory success instead), pydantic-free `Dict[str, Any]` typing
//! (represented by `JsonValue`), and python's negative-slice `limit` semantics
//! (`seq[:negative]`); the port takes `usize` and documents it.

extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use crate::hmac_sha256::sha256;
use crate::turbovec::{FemtosecondGenomicSSMScanner, TurboVec158Quantizer, TurboVec16xQuantizer, DIM};

/// Memory constants (1:1 python: `CHUNK_SIZE`, `COMPRESS_LEVEL`).
pub const CHUNK_SIZE: u64 = 1000;
pub const COMPRESS_LEVEL: u64 = 3;

/// Retained from the pre-port stub. The python original has NO TTL and NOTHING
/// in this file evicts entries, so this constant is inert API surface only.
pub const MEMORY_TTL_TICKS: u64 = 10000;

/// python `content[:10000]` — the stored content cap (code points).
pub const MAX_CONTENT_CHARS: usize = 10_000;
/// python `recency = max(0, 1.0 - age_hours / 168)` — decay over one week.
pub const RECENCY_WINDOW_HOURS: f64 = 168.0;
/// python `final_score = score + recency * 0.5`.
pub const RECENCY_WEIGHT: f64 = 0.5;
/// Nanoseconds per hour (age → hours without a datetime type).
pub const NS_PER_HOUR: f64 = 3.6e12;
/// python `hexdigest()[:16]` for entries.
pub const ID_PREFIX_CHARS: usize = 16;
/// python `hexdigest()[:12]` for events.
pub const EVENT_ID_PREFIX_CHARS: usize = 12;
/// python `add(..., agent="unknown")`.
pub const DEFAULT_AGENT: &str = "unknown";
/// python `log_event(..., agent="system")`.
pub const DEFAULT_EVENT_AGENT: &str = "system";
/// python `get_stats()["backend"]`.
pub const BACKEND_NAME: &str = "TurboVecNative";
/// The pre-port stub's opaque handle value, kept so the old constant is not lost.
pub const TURBOVEC_MEMORY_HANDLE: u64 = 0x5456_4D45_4D4F_5259;

/// Python `s[:n]` for str = the first `n` CODE POINTS. Never byte-slices, so it
/// cannot panic on a multi-byte UTF-8 boundary (unlike `&s[..n]`).
pub fn trunc_chars(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

// ── Minimal JSON (the subset python's json.dumps/json.loads round-trips) ─────
// No serde (zero-dependency doctrine), so the persistence boundary carries its
// own serialiser + tolerant parser. Serialisation matches python's
// `json.dumps(obj, ensure_ascii=False)` with DEFAULT separators (", " and ": "),
// because that is exactly what `_atomic_append` writes and what `query()` feeds
// to `.lower()` when it string-matches the metadata dict.

#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    Array(Vec<JsonValue>),
    /// Insertion-ordered pairs: python dicts preserve insertion order, and
    /// `json.dumps` emits that order, so a HashMap would break `query()` parity.
    Object(Vec<(String, JsonValue)>),
}

impl JsonValue {
    /// python `json.dumps(v).lower()` - what `query()` matches against.
    /// Delegates to `to_json` so there is exactly ONE serializer; only the
    /// casing step is added here. (The port called this but never defined it.)
    pub fn to_json_lower(&self) -> String {
        self.to_json().to_lowercase()
    }
}

impl JsonValue {
    /// python `metadata or {}` — the empty dict.
    pub fn empty_object() -> Self {
        JsonValue::Object(vec![])
    }

    pub fn object(pairs: Vec<(&str, JsonValue)>) -> Self {
        JsonValue::Object(pairs.into_iter().map(|(k, v)| (k.to_string(), v)).collect())
    }

    pub fn str(s: &str) -> Self {
        JsonValue::Str(s.to_string())
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            JsonValue::Str(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_int(&self) -> Option<i64> {
        match self {
            JsonValue::Int(i) => Some(*i),
            _ => None,
        }
    }

    pub fn get(&self, key: &str) -> Option<&JsonValue> {
        match self {
            JsonValue::Object(pairs) => pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    /// python `json.dumps(value, ensure_ascii=False)` — what `_atomic_append`
    /// writes to the JSONL file (non-ASCII stays raw UTF-8).
    pub fn to_json(&self) -> String {
        let mut out = String::new();
        self.write_json(&mut out, false);
        out
    }

    /// python `json.dumps(value)` with the DEFAULT `ensure_ascii=True`, i.e.
    /// every non-ASCII code point escaped as `\uXXXX` (surrogate pairs above the
    /// BMP). `query()` matches metadata against THIS form, not the persisted
    /// one — the python module really does use two different dumps, so the port
    /// has to as well or a non-ASCII metadata match would score differently.
    pub fn to_json_ascii(&self) -> String {
        let mut out = String::new();
        self.write_json(&mut out, true);
        out
    }

    /// python `json.dumps(value).lower()` — what `query()` matches against.
    pub fn to_json_ascii_lower(&self) -> String {
        self.to_json_ascii().to_lowercase()
    }

    fn write_json(&self, out: &mut String, ascii_only: bool) {
        match self {
            JsonValue::Null => out.push_str("null"),
            JsonValue::Bool(true) => out.push_str("true"),
            JsonValue::Bool(false) => out.push_str("false"),
            JsonValue::Int(i) => out.push_str(&format!("{}", i)),
            JsonValue::Float(f) => out.push_str(&fmt_f64_py(*f)),
            JsonValue::Str(s) => write_json_string(s, out, ascii_only),
            JsonValue::Array(items) => {
                out.push('[');
                for (i, v) in items.iter().enumerate() {
                    if i > 0 {
                        out.push_str(", ");
                    }
                    v.write_json(out, ascii_only);
                }
                out.push(']');
            }
            JsonValue::Object(pairs) => {
                out.push('{');
                for (i, (k, v)) in pairs.iter().enumerate() {
                    if i > 0 {
                        out.push_str(", ");
                    }
                    write_json_string(k, out, ascii_only);
                    out.push_str(": ");
                    v.write_json(out, ascii_only);
                }
                out.push('}');
            }
        }
    }
}

/// python `repr(float)` as used by `json.dumps`: an integral float keeps its
/// `.0` (`json.dumps(1.0) == "1.0"`), and the non-finite names are python's.
fn fmt_f64_py(x: f64) -> String {
    if x.is_nan() {
        return "NaN".to_string();
    }
    if x.is_infinite() {
        return if x > 0.0 { "Infinity".to_string() } else { "-Infinity".to_string() };
    }
    let s = format!("{}", x);
    if s.contains('.') || s.contains('e') || s.contains('E') {
        s
    } else {
        let mut t = s;
        t.push_str(".0");
        t
    }
}

/// python json string escaping. `ascii_only` = the `ensure_ascii` flag: when
/// set, every non-ASCII code point becomes `\uXXXX` (a surrogate PAIR above the
/// BMP), exactly as python's json encoder does; otherwise the UTF-8 is kept raw.
fn write_json_string(s: &str, out: &mut String, ascii_only: bool) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0c}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c if ascii_only && (c as u32) > 0x7f => {
                let cp = c as u32;
                if cp <= 0xffff {
                    out.push_str(&format!("\\u{:04x}", cp));
                } else {
                    let v = cp - 0x10000;
                    out.push_str(&format!("\\u{:04x}\\u{:04x}", 0xd800 + (v >> 10), 0xdc00 + (v & 0x3ff)));
                }
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

/// Tolerant `json.loads` for the subset this module writes. Returns Err on
/// malformed input so `restore_jsonl` can mimic python's
/// `except json.JSONDecodeError: continue`.
pub fn parse_json(text: &str) -> Result<JsonValue, &'static str> {
    let mut p = JsonParser { b: text.as_bytes(), i: 0 };
    p.skip_ws();
    let v = p.value()?;
    p.skip_ws();
    if p.i != p.b.len() {
        return Err("trailing data");
    }
    Ok(v)
}

struct JsonParser<'a> {
    b: &'a [u8],
    i: usize,
}

impl<'a> JsonParser<'a> {
    fn skip_ws(&mut self) {
        while self.i < self.b.len() && matches!(self.b[self.i], b' ' | b'\t' | b'\n' | b'\r') {
            self.i += 1;
        }
    }

    fn peek(&self) -> Option<u8> {
        self.b.get(self.i).copied()
    }

    fn expect(&mut self, c: u8) -> Result<(), &'static str> {
        if self.peek() == Some(c) {
            self.i += 1;
            Ok(())
        } else {
            Err("unexpected byte")
        }
    }

    fn literal(&mut self, s: &[u8]) -> Result<(), &'static str> {
        if self.b.len() >= self.i + s.len() && &self.b[self.i..self.i + s.len()] == s {
            self.i += s.len();
            Ok(())
        } else {
            Err("bad literal")
        }
    }

    fn value(&mut self) -> Result<JsonValue, &'static str> {
        match self.peek() {
            Some(b'{') => self.object(),
            Some(b'[') => self.array(),
            Some(b'"') => Ok(JsonValue::Str(self.string()?)),
            Some(b't') => {
                self.literal(b"true")?;
                Ok(JsonValue::Bool(true))
            }
            Some(b'f') => {
                self.literal(b"false")?;
                Ok(JsonValue::Bool(false))
            }
            Some(b'n') => {
                self.literal(b"null")?;
                Ok(JsonValue::Null)
            }
            Some(_) => self.number(),
            None => Err("unexpected end of input"),
        }
    }

    fn object(&mut self) -> Result<JsonValue, &'static str> {
        self.expect(b'{')?;
        let mut pairs: Vec<(String, JsonValue)> = Vec::new();
        self.skip_ws();
        if self.peek() == Some(b'}') {
            self.i += 1;
            return Ok(JsonValue::Object(pairs));
        }
        loop {
            self.skip_ws();
            let k = self.string()?;
            self.skip_ws();
            self.expect(b':')?;
            self.skip_ws();
            let v = self.value()?;
            pairs.push((k, v));
            self.skip_ws();
            match self.peek() {
                Some(b',') => self.i += 1,
                Some(b'}') => {
                    self.i += 1;
                    break;
                }
                _ => return Err("expected , or }"),
            }
        }
        Ok(JsonValue::Object(pairs))
    }

    fn array(&mut self) -> Result<JsonValue, &'static str> {
        self.expect(b'[')?;
        let mut items: Vec<JsonValue> = Vec::new();
        self.skip_ws();
        if self.peek() == Some(b']') {
            self.i += 1;
            return Ok(JsonValue::Array(items));
        }
        loop {
            self.skip_ws();
            let v = self.value()?;
            items.push(v);
            self.skip_ws();
            match self.peek() {
                Some(b',') => self.i += 1,
                Some(b']') => {
                    self.i += 1;
                    break;
                }
                _ => return Err("expected , or ]"),
            }
        }
        Ok(JsonValue::Array(items))
    }

    fn string(&mut self) -> Result<String, &'static str> {
        self.expect(b'"')?;
        let mut buf: Vec<u8> = Vec::new();
        loop {
            let c = self.peek().ok_or("unterminated string")?;
            self.i += 1;
            match c {
                b'"' => break,
                b'\\' => {
                    let e = self.peek().ok_or("bad escape")?;
                    self.i += 1;
                    match e {
                        b'"' => buf.push(b'"'),
                        b'\\' => buf.push(b'\\'),
                        b'/' => buf.push(b'/'),
                        b'n' => buf.push(b'\n'),
                        b't' => buf.push(b'\t'),
                        b'r' => buf.push(b'\r'),
                        b'b' => buf.push(0x08),
                        b'f' => buf.push(0x0c),
                        b'u' => {
                            let hi = self.hex4()?;
                            if (0xD800..0xDC00).contains(&hi) {
                                // high surrogate: expect a \uXXXX low surrogate
                                self.expect(b'\\')?;
                                self.expect(b'u')?;
                                let lo = self.hex4()?;
                                if !(0xDC00..0xE000).contains(&lo) {
                                    return Err("bad surrogate pair");
                                }
                                let cp = 0x10000 + ((hi - 0xD800) << 10) + (lo - 0xDC00);
                                push_utf8(&mut buf, char::from_u32(cp).ok_or("bad code point")?);
                            } else {
                                push_utf8(
                                    &mut buf,
                                    char::from_u32(hi).ok_or("bad code point")?,
                                );
                            }
                        }
                        _ => return Err("unknown escape"),
                    }
                }
                b => buf.push(b),
            }
        }
        String::from_utf8(buf).map_err(|_| "invalid utf-8")
    }

    fn hex4(&mut self) -> Result<u32, &'static str> {
        if self.i + 4 > self.b.len() {
            return Err("short \\u escape");
        }
        let mut v: u32 = 0;
        for k in 0..4 {
            let d = self.b[self.i + k];
            let n = match d {
                b'0'..=b'9' => (d - b'0') as u32,
                b'a'..=b'f' => (d - b'a') as u32 + 10,
                b'A'..=b'F' => (d - b'A') as u32 + 10,
                _ => return Err("bad hex digit"),
            };
            v = (v << 4) | n;
        }
        self.i += 4;
        Ok(v)
    }

    fn number(&mut self) -> Result<JsonValue, &'static str> {
        let start = self.i;
        if self.peek() == Some(b'-') {
            self.i += 1;
        }
        let mut is_float = false;
        while let Some(c) = self.peek() {
            match c {
                b'0'..=b'9' => self.i += 1,
                b'.' | b'e' | b'E' | b'+' | b'-' => {
                    is_float = true;
                    self.i += 1;
                }
                _ => break,
            }
        }
        if self.i == start {
            return Err("bad number");
        }
        let s = core::str::from_utf8(&self.b[start..self.i]).map_err(|_| "bad number")?;
        if !is_float {
            if let Ok(i) = s.parse::<i64>() {
                return Ok(JsonValue::Int(i));
            }
        }
        s.parse::<f64>().map(JsonValue::Float).map_err(|_| "bad number")
    }
}

fn push_utf8(buf: &mut Vec<u8>, c: char) {
    let mut tmp = [0u8; 4];
    buf.extend_from_slice(c.encode_utf8(&mut tmp).as_bytes());
}

// ── Records (python dicts) ──────────────────────────────────────────────────
// python entry : {"id","content","metadata","agent","timestamp"}
// python event : {"id","type","data","agent","timestamp"}

/// One memory entry. `at_ns` replaces python's `"timestamp"` ISO string.
#[derive(Debug, Clone)]
pub struct MemoryEntry {
    pub id: String,
    pub content: String,
    pub metadata: JsonValue,
    pub agent: String,
    pub at_ns: u64,
}

/// One logged event. `event_type` is python's `"type"` key; `at_ns` replaces
/// `"timestamp"`. (The pre-port stub's `MemoryEvent` name is reused here for
/// the same concept: the record `log_event()` appends.)
#[derive(Debug, Clone)]
pub struct MemoryEvent {
    pub id: String,
    pub event_type: String,
    pub data: JsonValue,
    pub agent: String,
    pub at_ns: u64,
}

/// python `get_stats()` return dict.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryStats {
    pub total_memories: usize,
    pub total_events: usize,
    /// python `agents` dict: counts in FIRST-APPEARANCE order (dict insertion).
    pub agents: Vec<(String, u64)>,
    /// python `file_size_mb` = round(memory_file.stat().st_size/1024/1024, 2).
    /// No file exists in the kernel, so this is the exported payload size.
    pub file_size_mb: f64,
    pub backend: String,
}

/// TurboVec memory store (python `TurboVecMemory`).
#[derive(Debug, Clone)]
pub struct TurbovecMemory {
    /// python `_cache`
    pub entries: Vec<MemoryEntry>,
    /// python `_index`: entry id -> position in `entries`.
    pub index: BTreeMap<String, usize>,
    /// python `_events`
    pub events: Vec<MemoryEvent>,
    /// Stand-in for python's `memory_file.stat().st_size` (no FS in the kernel):
    /// bytes of the serialised memory payload, refreshed by
    /// `refresh_memory_bytes()` / `compact()`.
    pub memory_bytes: u64,
}

impl TurbovecMemory {
    /// python `__init__` after `_load()` found nothing: an empty store.
    /// The disk load path is `restore_jsonl` / `restore_events_jsonl`.
    pub fn new() -> Self {
        TurbovecMemory {
            entries: Vec::new(),
            index: BTreeMap::new(),
            events: Vec::new(),
            memory_bytes: 0,
        }
    }

    /// python `total_memories` property → `len(self._cache)`.
    pub fn total_memories(&self) -> usize {
        self.entries.len()
    }

    /// python `total_events` property → `len(self._events)`.
    pub fn total_events(&self) -> usize {
        self.events.len()
    }

    /// python `_rebuild_index()`: id -> last position, rebuilt from the cache.
    pub fn rebuild_index(&mut self) {
        self.index.clear();
        for (i, entry) in self.entries.iter().enumerate() {
            if !entry.id.is_empty() {
                // python dict assignment: a later duplicate id wins.
                self.index.insert(entry.id.clone(), i);
            }
        }
    }

    /// python `add(content, metadata=None, agent="unknown")`, with the wall
    /// clock passed in as `at_ns` nanoseconds (see the DATETIME note in the
    /// module docs). Returns the entry id.
    pub fn add(&mut self, content: &str, metadata: JsonValue, agent: &str, at_ns: u64) -> String {
        // python: sha256(f"{content}{time.time()}{len(self._cache)}").hexdigest()[:16]
        // NB: the seed uses the FULL content, before the [:10000] cap.
        let seed = format!("{}{}{}", content, at_ns, self.entries.len());
        let entry_id = sha256_hex_prefix(&seed, ID_PREFIX_CHARS);

        let entry = MemoryEntry {
            id: entry_id.clone(),
            content: trunc_chars(content, MAX_CONTENT_CHARS), // python content[:10000]
            metadata,
            agent: agent.to_string(),
            at_ns,
        };

        self.entries.push(entry);
        self.index.insert(entry_id.clone(), self.entries.len() - 1);
        entry_id
    }

    /// `add` with python's defaults ({} metadata, agent="unknown").
    pub fn add_simple(&mut self, content: &str, at_ns: u64) -> String {
        self.add(content, JsonValue::empty_object(), DEFAULT_AGENT, at_ns)
    }

    /// python `get(entry_id)` — O(1) index lookup with the `idx < len(cache)`
    /// staleness guard.
    pub fn get(&self, entry_id: &str) -> Option<&MemoryEntry> {
        match self.index.get(entry_id) {
            Some(&idx) if idx < self.entries.len() => self.entries.get(idx),
            _ => None,
        }
    }

    /// python `query()`'s `scored` list, before sorting/truncation:
    /// `(final_score, cache_index)` in cache order. Exposed because the python
    /// list (score + index) is itself the interesting intermediate, and it makes
    /// the port differentially testable field by field.
    ///
    /// score        = Σ_words (1.0 if word in content) + (0.5 if word in metadata_json)
    /// recency      = max(0, 1.0 - age_hours / 168)
    /// final_score  = score + recency * 0.5
    /// filter       = final_score > 0 or query_lower in content, and agent match
    pub fn query_scores(
        &self,
        query_text: &str,
        agent: Option<&str>,
        now_ns: u64,
    ) -> Vec<(f64, usize)> {
        let query_lower = query_text.to_lowercase();
        // python `query_lower.split()` — whitespace runs, no empty tokens.
        let words: Vec<&str> = query_lower.split_whitespace().collect();

        let mut scored: Vec<(f64, usize)> = Vec::new();
        for (i, entry) in self.entries.iter().enumerate() {
            let content = entry.content.to_lowercase();
            let meta = entry.metadata.to_json_lower();

            let mut score = 0.0f64;
            for word in &words {
                if content.contains(word) {
                    score += 1.0;
                }
                if meta.contains(word) {
                    score += 0.5;
                }
            }

            // python parses the ISO timestamp and takes (now - ts) in hours; the
            // port uses the stored nanoseconds directly (documented adaptation).
            let age_hours = if now_ns >= entry.at_ns {
                (now_ns - entry.at_ns) as f64 / NS_PER_HOUR
            } else {
                -((entry.at_ns - now_ns) as f64) / NS_PER_HOUR
            };
            let mut recency = 1.0 - age_hours / RECENCY_WINDOW_HOURS;
            if recency < 0.0 {
                recency = 0.0; // python max(0, ...)
            }

            let final_score = score + recency * RECENCY_WEIGHT;

            if final_score > 0.0 || content.contains(query_lower.as_str()) {
                if agent.map_or(true, |a| entry.agent == a) {
                    scored.push((final_score, i));
                }
            }
        }

        // python `scored.sort(key=lambda x: x[0], reverse=True)` is STABLE, so
        // entries with equal scores keep cache order. Rust's sort_by is stable.
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(core::cmp::Ordering::Equal));
        scored
    }

    /// python `query(query_text, k=5, agent=None)` with the clock passed in.
    pub fn query_at(
        &self,
        query_text: &str,
        k: usize,
        agent: Option<&str>,
        now_ns: u64,
    ) -> Vec<&MemoryEntry> {
        self.query_scores(query_text, agent, now_ns)
            .into_iter()
            .take(k)
            .map(|(_, i)| &self.entries[i])
            .collect()
    }

    /// python `query()` using the live kernel clock for the recency term.
    pub fn query(&self, query_text: &str, k: usize, agent: Option<&str>) -> Vec<&MemoryEntry> {
        self.query_at(query_text, k, agent, crate::time::now_ns())
    }

    /// python `query_by_agent(agent, limit=20)`: agent-filtered entries, newest
    /// first. Python sorted by the ISO `timestamp` string descending (stable, so
    /// equal timestamps keep cache order); the port sorts by `at_ns` descending,
    /// which is the same order but with nanosecond rather than microsecond
    /// resolution. `limit` is `usize`: python's negative-slice form
    /// (`seq[:-3]`) has no unsigned equivalent and is not ported.
    pub fn query_by_agent(&self, agent: &str, limit: usize) -> Vec<&MemoryEntry> {
        let mut hits: Vec<&MemoryEntry> = self
            .entries
            .iter()
            .filter(|e| e.agent == agent)
            .collect();
        // stable descending sort: ties keep cache order (python reverse=True)
        hits.sort_by(|a, b| b.at_ns.cmp(&a.at_ns));
        hits.truncate(limit);
        hits
    }

    /// python `log_event(event_type, data, agent="system")`. Returns the event.
    pub fn log_event(
        &mut self,
        event_type: &str,
        data: JsonValue,
        agent: &str,
        at_ns: u64,
    ) -> MemoryEvent {
        // python: sha256(f"{event_type}{time.time()}{len(self._events)}").hexdigest()[:12]
        let seed = format!("{}{}{}", event_type, at_ns, self.events.len());
        let id = sha256_hex_prefix(&seed, EVENT_ID_PREFIX_CHARS);

        let event = MemoryEvent {
            id,
            event_type: event_type.to_string(),
            data,
            agent: agent.to_string(),
            at_ns,
        };
        self.events.push(event.clone());
        event
    }

    /// `log_event` with python's default agent ("system").
    pub fn log_event_simple(&mut self, event_type: &str, data: JsonValue, at_ns: u64) -> MemoryEvent {
        self.log_event(event_type, data, DEFAULT_EVENT_AGENT, at_ns)
    }

    /// python `get_stats()`. `file_size_mb` uses `round(x, 2)` = ties-to-EVEN.
    pub fn get_stats(&self) -> MemoryStats {
        let mut agents: Vec<(String, u64)> = Vec::new();
        for e in &self.entries {
            // python: str(e.get("agent", "unknown")) — counted in first-seen order
            let name = e.agent.clone();
            match agents.iter_mut().find(|(k, _)| *k == name) {
                Some(slot) => slot.1 += 1,
                None => agents.push((name, 1)),
            }
        }

        MemoryStats {
            total_memories: self.entries.len(),
            total_events: self.events.len(),
            agents,
            file_size_mb: round_half_even(
                (self.memory_bytes as f64 / 1024.0 / 1024.0) * 100.0,
            ) / 100.0,
            backend: BACKEND_NAME.to_string(),
        }
    }

    /// python `compact(max_entries=100000)`: keep only the newest
    /// `max_entries` entries and rebuild the index. Python then rewrites the
    /// gzip file; the port has no file, so it refreshes `memory_bytes` (the
    /// exported payload length) instead.
    pub fn compact(&mut self, max_entries: usize) {
        if self.entries.len() > max_entries {
            let drop = self.entries.len() - max_entries;
            self.entries.drain(0..drop);
            self.rebuild_index();
            self.refresh_memory_bytes();
        }
    }

    /// Chunk count (compression grouping) — retained from the pre-port stub.
    pub fn chunk_count(&self) -> u64 {
        (self.entries.len() as u64).div_ceil(CHUNK_SIZE)
    }

    // ── persistence boundary (python _atomic_append / _load) ────────────────

    /// Serialise the store as python's uncompressed JSONL
    /// (`turbo_memory.jsonl`): one `json.dumps(entry, ensure_ascii=False)` per
    /// line. gzip framing lives outside the kernel (see the module docs).
    pub fn export_jsonl(&self) -> String {
        let mut out = String::new();
        for e in &self.entries {
            out.push_str(&entry_to_json(e).to_json());
            out.push('\n');
        }
        out
    }

    /// Serialise the event ledger as python's `turbo_events.jsonl`.
    pub fn export_events_jsonl(&self) -> String {
        let mut out = String::new();
        for e in &self.events {
            out.push_str(&event_to_json(e).to_json());
            out.push('\n');
        }
        out
    }

    /// python `_load()` for the memory file: one JSON object per line, blank
    /// and malformed lines skipped (`except json.JSONDecodeError: continue`),
    /// then `_rebuild_index()`. Appends to the current contents the way python's
    /// loader would after clearing — call it on an empty store to replace.
    /// Returns the number of entries loaded.
    pub fn restore_jsonl(&mut self, jsonl: &str) -> usize {
        let mut loaded = 0usize;
        for line in jsonl.lines() {
            let t = line.trim();
            if t.is_empty() {
                continue;
            }
            let v = match parse_json(t) {
                Ok(v) => v,
                Err(_) => continue, // python: json.JSONDecodeError -> continue
            };
            self.entries.push(entry_from_json(&v));
            loaded += 1;
        }
        self.rebuild_index();
        loaded
    }

    /// python `_load()` for the event file (same tolerant line loop).
    pub fn restore_events_jsonl(&mut self, jsonl: &str) -> usize {
        let mut loaded = 0usize;
        for line in jsonl.lines() {
            let t = line.trim();
            if t.is_empty() {
                continue;
            }
            let v = match parse_json(t) {
                Ok(v) => v,
                Err(_) => continue,
            };
            self.events.push(event_from_json(&v));
            loaded += 1;
        }
        loaded
    }

    /// Refresh `memory_bytes` from the exported payload length. This is the
    /// port's replacement for `memory_file.stat().st_size`.
    pub fn refresh_memory_bytes(&mut self) {
        self.memory_bytes = self.export_jsonl().len() as u64;
    }
}

impl Default for TurbovecMemory {
    fn default() -> Self {
        Self::new()
    }
}

/// python `json.dumps(entry, ensure_ascii=False)` — key order matches the
/// python dict literal in `add()`.
fn entry_to_json(e: &MemoryEntry) -> JsonValue {
    JsonValue::Object(vec![
        ("id".to_string(), JsonValue::Str(e.id.clone())),
        ("content".to_string(), JsonValue::Str(e.content.clone())),
        ("metadata".to_string(), e.metadata.clone()),
        ("agent".to_string(), JsonValue::Str(e.agent.clone())),
        // python stored "timestamp": datetime.now().isoformat(); the port stores
        // the same instant as integer nanoseconds (DATETIME note above).
        ("at_ns".to_string(), JsonValue::Int(e.at_ns as i64)),
    ])
}

fn event_to_json(e: &MemoryEvent) -> JsonValue {
    JsonValue::Object(vec![
        ("id".to_string(), JsonValue::Str(e.id.clone())),
        ("type".to_string(), JsonValue::Str(e.event_type.clone())),
        ("data".to_string(), e.data.clone()),
        ("agent".to_string(), JsonValue::Str(e.agent.clone())),
        ("at_ns".to_string(), JsonValue::Int(e.at_ns as i64)),
    ])
}

/// python `entry.get(key, default)` reads.
fn entry_from_json(v: &JsonValue) -> MemoryEntry {
    MemoryEntry {
        id: v.get("id").and_then(|x| x.as_str()).unwrap_or("").to_string(),
        content: v.get("content").and_then(|x| x.as_str()).unwrap_or("").to_string(),
        metadata: v.get("metadata").cloned().unwrap_or_else(JsonValue::empty_object),
        agent: v.get("agent").and_then(|x| x.as_str()).unwrap_or("").to_string(),
        at_ns: v.get("at_ns").and_then(|x| x.as_int()).unwrap_or(0).max(0) as u64,
    }
}

fn event_from_json(v: &JsonValue) -> MemoryEvent {
    MemoryEvent {
        id: v.get("id").and_then(|x| x.as_str()).unwrap_or("").to_string(),
        event_type: v.get("type").and_then(|x| x.as_str()).unwrap_or("").to_string(),
        data: v.get("data").cloned().unwrap_or_else(JsonValue::empty_object),
        agent: v.get("agent").and_then(|x| x.as_str()).unwrap_or("").to_string(),
        at_ns: v.get("at_ns").and_then(|x| x.as_int()).unwrap_or(0).max(0) as u64,
    }
}

/// python `hashlib.sha256(seed.encode()).hexdigest()[:n]` — lowercase hex.
fn sha256_hex_prefix(seed: &str, n: usize) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let digest = sha256(seed.as_bytes());
    let mut s = String::with_capacity(64);
    for b in digest.iter() {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0x0f) as usize] as char);
    }
    trunc_chars(&s, n)
}

/// python `round(x)` — round half to EVEN (banker's rounding). `f64::round`
/// (ties away from zero) and `float_math::round_f64` are both wrong at exact
/// .5 ties, which is why this exists.
fn round_half_even(x: f64) -> f64 {
    if !x.is_finite() {
        return x;
    }
    let t = x as i64 as f64; // truncation toward zero
    let frac = x - t; // (-1, 1)
    let mag = if frac < 0.0 { -frac } else { frac };
    let sign = if frac < 0.0 { -1.0 } else { 1.0 };
    let step = if mag > 0.5 {
        1.0
    } else if mag < 0.5 {
        0.0
    } else if (t as i64) % 2 == 0 {
        0.0
    } else {
        1.0
    };
    t + sign * step
}

// ── Singleton (python module-level `_instance` + `get_turbovec_memory`) ─────
// python used double-checked locking with `threading.Lock`; the UEFI payload is
// single-threaded (the same documented assumption as agent_runtime.rs), so the
// outer check and the lock are unnecessary and the inner check is what remains.

struct SingletonCell(core::cell::UnsafeCell<Option<TurbovecMemory>>);
// SAFETY: single-threaded UEFI payload — the store is created once on first use
// and no concurrent access exists (python's threading.Lock covers a case the
// kernel does not have).
unsafe impl Sync for SingletonCell {}

static TURBOVEC_MEMORY: SingletonCell = SingletonCell(core::cell::UnsafeCell::new(None));

/// python `get_turbovec_memory(base_path=None)` — get or create the shared
/// instance. `base_path` has no meaning without a filesystem; use
/// `restore_jsonl` on the returned store to load persisted data.
pub fn get_turbovec_memory() -> &'static mut TurbovecMemory {
    // SAFETY: single-threaded UEFI payload; initialized once before any use.
    unsafe {
        let slot = &mut *TURBOVEC_MEMORY.0.get();
        if slot.is_none() {
            *slot = Some(TurbovecMemory::new());
        }
        slot.as_mut().unwrap()
    }
}

// ── TurboVec ternary/SSM layer (ADDITIVE — reuses sibling turbovec.rs) ──────
// `turbovec_memory.py` documents "Future: Mamba-3 SSM vectorization for semantic
// search" and never vectorises anything: `query()` is keyword matching, which is
// what the port above reproduces 1:1. This layer wires that documented future
// path to the primitives that already exist in `trm_uefi_kernel/src/turbovec.rs`
// rather than duplicating them:
//     DIM                                       — vector width / SSM state size
//     TurboVec158Quantizer::quantize_158         — 1.58-bit ternary + trit packing
//     TurboVec158Quantizer::dequantize_158       — trit unpack
//     FemtosecondGenomicSSMScanner::femto_step   — the SSM state scan
//     TurboVec16xQuantizer::fast_femto_cosine_similarity — ranking metric
// NOTHING in the parity path (add/get/query/query_by_agent/log_event/get_stats/
// compact) calls any of it, so the differential agreement with python is
// unaffected. It has NO python counterpart, so it is not differentially tested.

/// Deterministic DIM-length ternary embedding of `text`: a content-seeded
/// projection (`float_math::Rng`, xorshift64* — reproducible where python's
/// np.random was not) quantised to 1.58-bit trits and unpacked back to floats,
/// so the vector the rest of TurboVec sees is exactly the ternary one.
pub fn embed_ternary(text: &str) -> Vec<f32> {
    let digest = sha256(text.as_bytes());
    let mut seed = [0u8; 8];
    seed.copy_from_slice(&digest[0..8]);
    let mut rng = crate::float_math::Rng::new(u64::from_le_bytes(seed));

    let mut v: Vec<f32> = Vec::with_capacity(DIM);
    for _ in 0..DIM {
        v.push(rng.next_normal() as f32);
    }
    let (packed, scale) = TurboVec158Quantizer::quantize_158(&v);
    TurboVec158Quantizer::dequantize_158(&packed, DIM, scale)
}

/// The "Mamba-3 SSM vectorization" the python module names: every text is
/// ternary-embedded and femto-stepped (`h_t = 0.95 h_{t-1} + 0.05 x_t`), and the
/// final DIM-accumulated state is returned. The scan latency `femto_step`
/// reports is dropped — it is a timing measurement, not state.
pub fn ssm_scan(texts: &[&str]) -> Vec<f32> {
    let mut scanner = FemtosecondGenomicSSMScanner::new();
    for t in texts {
        let _ = scanner.femto_step(&embed_ternary(t));
    }
    scanner.h_state.to_vec()
}

/// Semantic counterpart to `query()`: rank entries by ternary-vector cosine
/// similarity to the query text. Ties keep cache order (stable sort).
pub fn semantic_query(store: &TurbovecMemory, query_text: &str, k: usize) -> Vec<(String, f32)> {
    let q = embed_ternary(query_text);
    let mut hits: Vec<(f32, usize, String)> = store
        .entries
        .iter()
        .enumerate()
        .map(|(i, e)| {
            (
                TurboVec16xQuantizer::fast_femto_cosine_similarity(&q, &embed_ternary(&e.content)),
                i,
                e.id.clone(),
            )
        })
        .collect();
    hits.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(core::cmp::Ordering::Equal));
    hits.into_iter().take(k).map(|(s, _, id)| (id, s)).collect()
}

// ── grafted from the v20 base tree (G185) ────────────────────────────────────────────────────────
// These four helpers existed only on the base side: our file is the fuller port (49 functions the
// base lacks) but it had dropped these, so callers on the base would have failed to resolve them.
// Grafted verbatim rather than re-implemented, so behaviour matches the side that had them.

pub fn atomic_append_line(entry_json: &str) -> String {
    let mut s = String::from(entry_json);
    s.push('\n');
    s
}

/// `_extract_agent_name` — the portable (string + meta) branches.
///
/// The oracle has four branches. Two are host-side and excluded here: an
/// `isinstance(agent_val, dict)` branch, and the `ast.literal_eval` branch
/// (a string starting `{` that contains `"author"` — JSON/dict parsing, which
/// needs no_std-unavailable machinery; the caller resolves those before calling
/// this, or accepts the documented `"Seeker"` parse-failure fallback). The two
/// portable branches are the plain-string rule and the meta fallback.
pub fn extract_agent_name(agent_val: &str, meta_author: Option<&str>, meta_agent: Option<&str>) -> String {
    if agent_val.starts_with('{') && agent_val.contains("author") {
        // ast.literal_eval path — host-side. The oracle returns "Seeker" when that
        // parse fails; this is the no_std stand-in.
        return String::from("Seeker");
    }
    if !(agent_val.is_empty() || agent_val == "unknown" || agent_val == "None") {
        return agent_val.to_string();
    }
    let m = meta_author
        .filter(|a| !a.is_empty())
        .or_else(|| meta_agent.filter(|a| !a.is_empty()));
    match m {
        Some(a) => a.to_string(),
        None => String::from("Swarm Collective"),
    }
}

/// `get_stats`'s memory-type categorization — a pure agent-name → category table.
/// The oracle lowers/replaces only on the FINAL `m_type` (`str(m_type).lower().replace("-", "_")
/// .replace(" ", "_")`); the category names here contain no `-`/space, so this table is the
/// post-normalization value directly.
pub fn memory_type_for(agent_name: &str) -> &'static str {
    match agent_name {
        "Seeker" | "Researcher" => "research",
        "Logic" | "LogicMaster" | "Reasoning Engine" => "reasoning_state",
        "Archi" | "Architect" => "architecture",
        "Verify" | "QA Engineer" => "verification",
        "Shield" | "Security Auditor" => "security",
        "Devo" | "Lead Developer" => "task_result",
        "Flow" | "Integration Specialist" => "workflow",
        _ => "experience",
    }
}

/// Grafted verbatim from the base tree (its text was captured before the merge overwrote the file):
/// a plain threshold check used by the base's compaction route.
pub fn compact_plan(entries: usize, max_entries: usize) -> (bool, usize) {
    if entries > max_entries {
        (true, max_entries)
    } else {
        (false, entries)
    }
}
