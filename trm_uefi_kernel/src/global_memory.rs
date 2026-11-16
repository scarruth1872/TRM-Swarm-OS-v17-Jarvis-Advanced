//! TRM Swarm OS — Global Memory Sync (no_std port).
//! Production-grade distributed vector database for shared long-term memory.
//! 1:1 deterministic core from swarm_v2/core/global_memory.py:
//!   - TurboVec 16x vector memory integration
//!   - Shared semantic embeddings and metadata tagging
//!   - Sub-millisecond associative recall across 14 council agents
//!
//! ── PORT NOTES / ADAPTATIONS (every deviation from the Python is listed) ──────
//!
//! 1. PERSISTENCE. The Python `__init__` built a ChromaDB/TurboVec directory
//!    (`GLOBAL_MEMORY_DIR`), `_load_metadata()` read `metadata.json` and
//!    `_save_metadata()` wrote it. A no_std kernel has no filesystem, so the
//!    store is kept in RAM and persistence is exposed as a BOUNDARY:
//!    `save_metadata() -> MetadataSnapshot` (same content `_save_metadata`
//!    wrote: version, entries, `sync_log[-100:]`) and
//!    `load_metadata(snapshot)` (same content `_load_metadata` read).
//!    `query`'s trailing `_save_metadata()` call is therefore a no-op here.
//!
//! 2. NO VECTOR BACKEND. `CHROMA_AVAILABLE` is permanently false and the
//!    TurboVec JSONL backend does not exist in the kernel, so every call takes
//!    the branch Python labelled "Legacy Mode" (the keyword-overlap scorer).
//!    `get_stats()["backend"]` is therefore always "Mock/Legacy".
//!    The ChromaDB branch (`self.collection.query(...)`, cosine distance ->
//!    `score = 1.0 - dist`) is not portable: it needs an external vector DB.
//!    It is a native embedding extension point, not ported.
//!
//! 3. TIME. `datetime.now()` / `datetime.fromisoformat()` do not exist in
//!    no_std. Every time-dependent method takes an explicit `at_ns: u64`
//!    (nanoseconds since the Unix epoch) and the caller passes
//!    `crate::time::now_ns()` (monotonic TAI-ish ns is fine — only deltas are
//!    compared). `created_at` is stored as ns, not as an ISO-8601 string.
//!    Age in `optimize_collective_memory` is `(at_ns - created_at) / 86_400e9`
//!    floored, which is exactly Python's `timedelta.days` semantics.
//!
//! 4. `round()`. Python's `round(x, n)` is half-to-EVEN on the exact decimal
//!    value of the double. `crate::float_math::round_f64` is half-AWAY-FROM-ZERO
//!    on the scaled value, so it disagrees at ties (measured: Python
//!    `round(0.125, 2) == 0.12`, `round_f64(0.125*100)/100 == 0.13`). To stay
//!    faithful, `round_decimals()` is used instead: it formats the double to
//!    n+40 exact fractional digits, rounds that decimal string half-to-even,
//!    and re-parses with core's correctly-rounded `str::parse::<f64>()`.
//!
//! 5. `hash()`. Python's `entry_id` is `f"gm_{hash(safe+author) & 0xFFFFFFFF:08x}"`.
//!    CPython's `hash(str)` is SipHash-1-3 with a per-process random key, so it
//!    is not reproducible by default; with `PYTHONHASHSEED=0` the key is all
//!    zero and `hash(s)` is exactly `siphash13(0, 0, utf8(s))` (verified
//!    against CPython 3.11 — see the porting test). `py_hash_str` implements
//!    that, so entry ids are byte-identical to a CPython run with
//!    PYTHONHASHSEED=0. Under the default (randomised) seed Python's ids differ.
//!    Empty string hashes to 0 (CPython special-cases it), which is replicated.
//!
//! 6. `content` is inside the metadata. Python's `contribute` kept the text only
//!    in the external vector store and stored no `content` key in
//!    `entries_metadata`, yet `query`/`get_recent_entries`/`optimize` read
//!    `meta.get("content", "")`. Entries loaded from a real on-disk
//!    `metadata.json` DO carry `content` (checked: swarm_v2_global_memory/
//!    metadata.json), so the kernel keeps the same schema — self-contained,
//!    since there is no external store to hold the text. All scoring, ordering,
//!    filters and truncation lengths are unchanged.
//!
//! 7. DGE. `dge.validate_memory_contribution`, `dge.mask_pii` and
//!    `dge.log_access` are ported inline (`dge_mask_pii`,
//!    `dge_validate_memory_contribution`); the kernel's dge.rs is only a
//!    singleton stub. The three PII regexes are hand-coded byte scanners
//!    (`email`, `api_key`, `ipv4`) because no_std has no regex engine.
//!    `log_access` is a logging side effect -> no-op.
//!
//! 8. `_sync_to_shared_jsonld` is NOT ported: in the Python it sits after an
//!    unconditional `return entry_id` and is therefore dead code. It also
//!    writes `F:\...\shared_semantic_mesh.jsonld` — file IO, which no_std bans.
//!
//! 9. Absent metadata keys. Python reads fields with `.get(key, default)`;
//!    a missing key maps to "" here and the Python default is applied when the
//!    field is empty ("unknown"/"knowledge"). `created_at` is `Option<u64>`
//!    (None -> the caller's `at_ns`, matching Python's `now.isoformat()`
//!    default). `contribute` always writes all fields, so this only affects
//!    externally restored snapshots.
//!
//! 10. `[]` ordering. `entries_metadata` is a `Vec`, not a map, because Python
//!     dicts iterate in INSERTION order and `query` uses a STABLE sort — a
//!     BTreeMap would re-order ties and change the output. `Vec` reproduces it.

extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

// ════════════════════════════════════════════════════════════════════════════
//  Pre-existing kernel API (kept intact — main.rs self-test uses it)
// ════════════════════════════════════════════════════════════════════════════

pub const MAX_GLOBAL_MEMORIES: usize = 2048;
pub const EMBEDDING_DIM: usize = 128;

#[derive(Debug, Clone)]
pub struct GlobalMemoryEntry {
    pub id: u64,
    pub agent: &'static str,
    pub category: &'static str,
    pub content: String,
    pub embedding: [f32; EMBEDDING_DIM],
    pub timestamp_ns: u64,
}

// ════════════════════════════════════════════════════════════════════════════
//  Python-faithful types (swarm_v2/core/global_memory.py)
// ════════════════════════════════════════════════════════════════════════════

/// Corresponds to one value of Python's `entries_metadata[eid]` dict.
#[derive(Debug, Clone, Default)]
pub struct MemoryMetadata {
    pub author: String,
    pub author_role: String,
    pub memory_type: String,
    /// Python stored `json.dumps(tags)`; kept structured here, rendered by
    /// `tags_to_json` for the persistence boundary.
    pub tags: Vec<String>,
    /// Python: `created_at` ISO-8601 string. `None` == key absent.
    pub created_at: Option<u64>,
    pub access_count: u64,
    /// Python set `last_accessed` inside `query`.
    pub last_accessed: Option<u64>,
    /// Python read this as `meta.get("content", "")` (note 6).
    pub content: String,
}

/// One formatted result of `query`: `(score, entry_dict)`.
#[derive(Debug, Clone)]
pub struct MemoryQueryResult {
    pub entry_id: String,
    pub content: String,
    pub author: String,
    pub author_role: String,
    pub memory_type: String,
    pub tags: Vec<String>,
    pub created_at: Option<u64>,
    pub access_count: u64,
}

/// The Python module never appends to `sync_log`; it only read it back from
/// `metadata.json`. This is the shape that file carried in practice.
#[derive(Debug, Clone, Default)]
pub struct SyncEvent {
    pub kind: String,
    pub entry_id: String,
    pub author: String,
    pub at_ns: u64,
}

/// One element of the `memories: List[dict]` argument of `sync_from_agent`.
#[derive(Debug, Clone, Default)]
pub struct SyncMemoryInput {
    pub content: String,
    pub memory_type: String,
    pub tags: Vec<String>,
}

impl SyncMemoryInput {
    pub fn new(content: &str) -> Self {
        SyncMemoryInput { content: content.to_string(), ..Default::default() }
    }
    pub fn new_typed(content: &str, memory_type: &str, tags: Vec<String>) -> Self {
        SyncMemoryInput { content: content.to_string(), memory_type: memory_type.to_string(), tags }
    }
}

/// Python `get_stats()` return dict.
#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub total_memories: usize,
    /// In insertion-order-of-first-occurrence, like `dict(Counter(...))`.
    pub by_type: Vec<(String, u64)>,
    pub by_author: Vec<(String, u64)>,
    pub total_accesses: u64,
    pub sync_events: usize,
    pub backend: &'static str,
}

/// Python `get_memory_health()` return dict.
#[derive(Debug, Clone)]
pub struct MemoryHealth {
    pub total_memories: usize,
    pub total_accesses: u64,
    pub avg_access_rate: f64,
    pub type_diversity: usize,
    pub author_diversity: usize,
    pub backend: &'static str,
    pub health_score: f64,
}

/// Python `optimize_collective_memory()` return dict.
#[derive(Debug, Clone)]
pub struct OptimizeResult {
    pub total_before: usize,
    pub total_after: usize,
    pub pruned: usize,
    pub consolidated: usize,
    pub timestamp: u64,
}

/// Persistence boundary (note 1) — the payload of `metadata.json`.
#[derive(Debug, Clone, Default)]
pub struct MetadataSnapshot {
    pub version: &'static str,
    pub entries: Vec<(String, MemoryMetadata)>,
    pub sync_log: Vec<SyncEvent>,
}

/// Note 2: no ChromaDB in the kernel.
pub const CHROMA_AVAILABLE: bool = false;
/// Note 2: the branch every call takes here.
pub const BACKEND_LEGACY: &str = "Mock/Legacy";
/// `_save_metadata` writes `{"version": "2.0", ...}`.
pub const METADATA_VERSION: &str = "2.0";
/// `_save_metadata` keeps `self.sync_log[-100:]`.
pub const SYNC_LOG_KEEP: usize = 100;

pub struct GlobalMemorySync {
    // ── pre-existing kernel API state ──
    pub entries: Vec<GlobalMemoryEntry>,
    pub next_id: u64,
    // ── Python GlobalMemorySync state ──
    /// `self.entries_metadata` (insertion-ordered, note 10).
    pub entries_metadata: Vec<(String, MemoryMetadata)>,
    /// `self.sync_log`.
    pub sync_log: Vec<SyncEvent>,
}

impl GlobalMemorySync {
    pub fn new() -> Self {
        GlobalMemorySync {
            entries: Vec::new(),
            next_id: 1,
            entries_metadata: Vec::new(),
            sync_log: Vec::new(),
        }
    }

    // ── pre-existing kernel API ──────────────────────────────────────────────

    pub fn store(
        &mut self,
        agent: &'static str,
        category: &'static str,
        content: &str,
        embedding: Option<[f32; EMBEDDING_DIM]>,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let emb = embedding.unwrap_or([0.0f32; EMBEDDING_DIM]);

        if self.entries.len() >= MAX_GLOBAL_MEMORIES {
            self.entries.remove(0); // Evict oldest
        }

        self.entries.push(GlobalMemoryEntry {
            id,
            agent,
            category,
            content: content.to_string(),
            embedding: emb,
            timestamp_ns: crate::time::now_ns(),
        });

        id
    }

    pub fn recall_by_agent(&self, agent: &str) -> Vec<&GlobalMemoryEntry> {
        self.entries
            .iter()
            .filter(|e| e.agent.eq_ignore_ascii_case(agent))
            .collect()
    }

    pub fn recall_by_category(&self, category: &str) -> Vec<&GlobalMemoryEntry> {
        self.entries
            .iter()
            .filter(|e| e.category.eq_ignore_ascii_case(category))
            .collect()
    }

    pub fn total_entries(&self) -> usize {
        self.entries.len()
    }

    // ── internal helpers ─────────────────────────────────────────────────────

    fn meta_index(&self, entry_id: &str) -> Option<usize> {
        self.entries_metadata.iter().position(|(id, _)| id == entry_id)
    }

    pub fn get_metadata(&self, entry_id: &str) -> Option<&MemoryMetadata> {
        self.meta_index(entry_id).map(|i| &self.entries_metadata[i].1)
    }

    fn remove_entry(&mut self, entry_id: &str) -> bool {
        match self.meta_index(entry_id) {
            Some(i) => {
                self.entries_metadata.remove(i);
                true
            }
            None => false,
        }
    }

    /// Insert or replace the metadata of `entry_id` (Python dict assignment).
    pub fn put_metadata(&mut self, entry_id: &str, meta: MemoryMetadata) {
        match self.meta_index(entry_id) {
            Some(i) => self.entries_metadata[i].1 = meta,
            None => self.entries_metadata.push((entry_id.to_string(), meta)),
        }
    }

    /// Boundary used when restoring a snapshot written by the Python side.
    pub fn push_sync_event(&mut self, ev: SyncEvent) {
        self.sync_log.push(ev);
    }

    /// Note 1: `_save_metadata()` — build the snapshot a filesystem driver
    /// would serialise. `sync_log[-100:]` is applied exactly as Python did.
    pub fn save_metadata(&self) -> MetadataSnapshot {
        let n = self.sync_log.len();
        let start = if n > SYNC_LOG_KEEP { n - SYNC_LOG_KEEP } else { 0 };
        MetadataSnapshot {
            version: METADATA_VERSION,
            entries: self.entries_metadata.clone(),
            sync_log: self.sync_log[start..].to_vec(),
        }
    }

    /// Note 1: `_load_metadata()` — adopt a snapshot off the persistence layer.
    pub fn load_metadata(&mut self, snap: MetadataSnapshot) {
        self.entries_metadata = snap.entries;
        self.sync_log = snap.sync_log;
    }

    // ── _is_poisoned ─────────────────────────────────────────────────────────

    pub fn is_poisoned(&self, text: &str) -> bool {
        is_poisoned(text)
    }

    // ── contribute ───────────────────────────────────────────────────────────

    /// Python default `memory_type="knowledge"`, `tags=None` -> empty.
    pub fn contribute_default(
        &mut self,
        content: &str,
        author: &str,
        author_role: &str,
        at_ns: u64,
    ) -> String {
        self.contribute(content, author, author_role, "knowledge", &[], at_ns)
    }

    /// `contribute(content, author, author_role, memory_type="knowledge", tags=None) -> str`
    pub fn contribute(
        &mut self,
        content: &str,
        author: &str,
        author_role: &str,
        memory_type: &str,
        tags: &[String],
        at_ns: u64,
    ) -> String {
        // 1. DGE Policy Check
        if !dge_validate_memory_contribution(author, content) {
            return "rejected_by_dge".to_string();
        }

        // 2. DGE Masking
        let safe_content = dge_mask_pii(content);

        // 3. Poison Check
        if self.is_poisoned(&safe_content) {
            return "skipped_poison".to_string();
        }

        let mut key = safe_content.clone();
        key.push_str(author);
        let entry_id = format!("gm_{:08x}", (py_hash_str(&key) & 0xFFFF_FFFF) as u32);

        let metadata = MemoryMetadata {
            author: author.to_string(),
            author_role: author_role.to_string(),
            memory_type: memory_type.to_string(),
            tags: tags.to_vec(),
            created_at: Some(at_ns),
            access_count: 0,
            last_accessed: None,
            content: safe_content,
        };

        // Python: `self.entries_metadata[entry_id] = metadata` (dict insert).
        self.put_metadata(&entry_id, metadata);
        // `self._save_metadata()` -> note 1: no-op, the RAM store IS the state.
        entry_id
    }

    // ── query ────────────────────────────────────────────────────────────────

    /// `query(query_text, top_k=5, author_filter=None, type_filter=None)`
    ///
    /// Note 2: the keyword-overlap ("Legacy Mode") branch is the ported one.
    /// Python's `author_filter`/`type_filter` are falsy-checked, so `Some("")`
    /// behaves exactly like `None` (no filter) — replicated.
    pub fn query(
        &mut self,
        query_text: &str,
        top_k: usize,
        author_filter: Option<&str>,
        type_filter: Option<&str>,
        at_ns: u64,
    ) -> Vec<(f64, MemoryQueryResult)> {
        // `set(query_text.lower().split())`
        let mut query_words: BTreeMap<String, ()> = BTreeMap::new();
        let lowered = query_text.to_lowercase();
        for w in lowered.split_whitespace() {
            query_words.insert(w.to_string(), ());
        }
        let n_words = query_words.len();
        let blank_query = query_text.trim().is_empty();

        // (score, index) pairs, then a STABLE descending sort (like Python's
        // list.sort(key=..., reverse=True)).
        let mut scored: Vec<(f64, usize)> = Vec::new();
        for (i, (_eid, meta)) in self.entries_metadata.iter().enumerate() {
            if let Some(af) = author_filter {
                if !af.is_empty() && meta.author != af {
                    continue;
                }
            }
            if let Some(tf) = type_filter {
                if !tf.is_empty() && meta.memory_type != tf {
                    continue;
                }
            }

            let content_lower = meta.content.to_lowercase();
            // Basic TF-IDF/Overlap score
            let overlap = query_words.keys().filter(|w| content_lower.contains(w.as_str())).count();
            let denom = if n_words > 1 { n_words } else { 1 };
            let score = overlap as f64 / denom as f64;

            if overlap > 0 || blank_query {
                scored.push((score, i));
            }
        }

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(core::cmp::Ordering::Equal));

        let mut out: Vec<(f64, MemoryQueryResult)> = Vec::new();
        for (score, idx) in scored.into_iter().take(top_k) {
            // Increment access count
            self.entries_metadata[idx].1.access_count += 1;
            self.entries_metadata[idx].1.last_accessed = Some(at_ns);

            let eid = self.entries_metadata[idx].0.clone();
            let m = &self.entries_metadata[idx].1;
            let entry_dict = MemoryQueryResult {
                entry_id: eid.clone(),
                content: dge_mask_pii(&m.content),
                author: if m.author.is_empty() { "unknown".to_string() } else { m.author.clone() },
                author_role: if m.author_role.is_empty() {
                    "unknown".to_string()
                } else {
                    m.author_role.clone()
                },
                memory_type: if m.memory_type.is_empty() {
                    "knowledge".to_string()
                } else {
                    m.memory_type.clone()
                },
                tags: m.tags.clone(),
                created_at: m.created_at.or(Some(at_ns)),
                access_count: m.access_count,
            };
            // `self.dge.log_access(author_filter or "System", eid, "QUERY_READ")`
            // -> audit log side effect, no-op in the kernel.
            out.push((score, entry_dict));
        }

        // `self._save_metadata()` -> no-op (note 1).
        out
    }

    // ── sync_from_agent ──────────────────────────────────────────────────────

    /// `sync_from_agent(agent_name, agent_role, memories) -> int`
    ///
    /// Faithful quirk: the dedupe guard reads the top score of a one-shot query,
    /// so it also BUMPS that entry's access count, and `added` is incremented
    /// even when `contribute` returns "rejected_by_dge"/"skipped_poison".
    pub fn sync_from_agent(
        &mut self,
        agent_name: &str,
        agent_role: &str,
        memories: &[SyncMemoryInput],
        at_ns: u64,
    ) -> usize {
        let mut added = 0usize;
        for mem in memories {
            let content = mem.content.clone();
            if content.is_empty() || content.chars().count() < 10 {
                continue;
            }
            let existing = self.query(&content, 1, None, None, at_ns);
            if !existing.is_empty() && existing[0].0 > 0.95 {
                continue;
            }
            let mtype = if mem.memory_type.is_empty() { "knowledge" } else { mem.memory_type.as_str() };
            self.contribute(&content, agent_name, agent_role, mtype, &mem.tags, at_ns);
            added += 1;
        }
        added
    }

    // ── get_context_for_agent ────────────────────────────────────────────────

    /// `get_context_for_agent(agent_name, task, max_memories=3) -> str`
    /// (`agent_name` is unused in the Python original too.)
    pub fn get_context_for_agent(
        &mut self,
        agent_name: &str,
        task: &str,
        max_memories: usize,
        at_ns: u64,
    ) -> String {
        let _ = agent_name;
        let results = self.query(task, max_memories, None, None, at_ns);
        let source_anchors = self.query(task, 2, None, Some("source_anchor"), at_ns);

        let mut lines: Vec<String> = Vec::new();

        for (score, entry) in source_anchors.iter() {
            if *score > 0.4 {
                lines.push(format!("Core System Philosophy: {}", entry.content));
            }
        }

        for (score, entry) in results.iter() {
            if entry.memory_type == "source_anchor" {
                continue;
            }
            if is_poisoned(&entry.content) {
                continue;
            }
            if *score > 0.35 {
                lines.push(format!(
                    "Past knowledge from {}: {}",
                    entry.author,
                    trunc_chars(&entry.content, 300)
                ));
            }
        }

        if lines.is_empty() {
            return String::new();
        }

        format!("Global Intelligence Feed:\n{}", lines.join("\n"))
    }

    // ── get_stats ────────────────────────────────────────────────────────────

    /// `get_stats() -> dict`. The TurboVec branch (`self._turbovec.get_stats()`)
    /// reports file sizes and is not portable; this is the Legacy branch.
    pub fn get_stats(&self) -> MemoryStats {
        let count = self.entries_metadata.len();

        let mut by_type: Vec<(String, u64)> = Vec::new();
        let mut by_author: Vec<(String, u64)> = Vec::new();
        let mut total_accesses: u64 = 0;
        for (_eid, m) in self.entries_metadata.iter() {
            bump_counter(&mut by_type, &m.memory_type);
            bump_counter(&mut by_author, &m.author);
            total_accesses += m.access_count;
        }

        MemoryStats {
            total_memories: count,
            by_type,
            by_author,
            total_accesses,
            sync_events: self.sync_log.len(),
            backend: BACKEND_LEGACY,
        }
    }

    // ── get_recent_entries ───────────────────────────────────────────────────

    /// `get_recent_entries(limit=20) -> List[dict]`
    /// Python: `list(keys())[-limit:]` — note `limit == 0` yields `[-0:]` which
    /// is the WHOLE list, replicated below.
    pub fn get_recent_entries(&self, limit: usize) -> Vec<(String, MemoryMetadata)> {
        let n = self.entries_metadata.len();
        let start = if limit == 0 { 0 } else { n.saturating_sub(limit) };
        self.entries_metadata[start..].to_vec()
    }

    // ── get_sync_log ─────────────────────────────────────────────────────────

    /// `get_sync_log(limit=20) -> List[dict]` — `self.sync_log[-limit:]`,
    /// with the same `limit == 0` -> whole-list behaviour.
    pub fn get_sync_log(&self, limit: usize) -> Vec<SyncEvent> {
        let n = self.sync_log.len();
        let start = if limit == 0 { 0 } else { n.saturating_sub(limit) };
        self.sync_log[start..].to_vec()
    }

    // ── optimize_collective_memory ───────────────────────────────────────────

    /// `optimize_collective_memory(max_age_days=90, min_access_count=0)`
    pub fn optimize_collective_memory(
        &mut self,
        max_age_days: i64,
        min_access_count: u64,
        at_ns: u64,
    ) -> OptimizeResult {
        const NS_PER_DAY: i64 = 86_400_000_000_000;
        let total_before = self.entries_metadata.len();
        let protected_types = ["source_anchor", "introspection", "reasoning_state"];
        let mut pruned = 0usize;
        let mut consolidated = 0usize;

        // ── pass 1: age pruning ──
        let mut to_remove: Vec<String> = Vec::new();
        for (eid, meta) in self.entries_metadata.iter() {
            if protected_types.contains(&meta.memory_type.as_str()) {
                continue;
            }
            // Python: try: age = (now - fromisoformat(created_at)).days
            //         except: age = 0
            let age_days: i64 = match meta.created_at {
                Some(c) => (at_ns as i64 - c as i64).div_euclid(NS_PER_DAY),
                None => 0,
            };
            if age_days > max_age_days && meta.access_count <= min_access_count {
                to_remove.push(eid.clone());
            }
        }

        for eid in to_remove.iter() {
            self.remove_entry(eid); // Python: del entries_metadata[eid] (+ backend delete)
            pruned += 1;
        }

        // ── pass 2: duplicate consolidation (over the post-prune snapshot) ──
        let snapshot: Vec<(String, MemoryMetadata)> = self.entries_metadata.clone();
        let mut seen: BTreeMap<String, String> = BTreeMap::new();
        for (eid, meta) in snapshot.into_iter() {
            let content = trunc_chars(&meta.content, 200).to_lowercase();
            let content = content.trim();
            let sig = format!(
                "{}:{:04x}",
                meta.author,
                (py_hash_str(content) & 0xFFFF) as u32
            );
            match seen.get(&sig).cloned() {
                Some(existing_eid) if existing_eid != eid => {
                    let ea = self
                        .get_metadata(&existing_eid)
                        .map(|m| m.access_count)
                        .unwrap_or(0);
                    let ca = meta.access_count;
                    let rid = if ca <= ea { eid.clone() } else { existing_eid.clone() };
                    if self.meta_index(&rid).is_some() {
                        self.remove_entry(&rid);
                        consolidated += 1;
                    }
                    if rid == existing_eid {
                        seen.insert(sig.clone(), eid.clone());
                    }
                }
                _ => {
                    seen.insert(sig.clone(), eid.clone());
                }
            }
        }

        if pruned > 0 || consolidated > 0 {
            // Python: self._save_metadata() + _logger.info(...) -> no-op (note 1).
        }

        OptimizeResult {
            total_before,
            total_after: self.entries_metadata.len(),
            pruned,
            consolidated,
            timestamp: at_ns,
        }
    }

    // ── get_memory_health ────────────────────────────────────────────────────

    /// `get_memory_health() -> dict`
    pub fn get_memory_health(&self) -> MemoryHealth {
        let stats = self.get_stats();
        let total = stats.total_memories;
        let total_accesses = stats.total_accesses;
        let avg_access = if total > 0 {
            total_accesses as f64 / total as f64
        } else {
            0.0
        };
        let type_count = stats.by_type.len();
        let author_count = stats.by_author.len();
        let raw_health =
            avg_access * 0.3 + type_count as f64 * 0.1 + author_count as f64 * 0.05;
        let capped = if 1.0 < raw_health { 1.0 } else { raw_health };
        MemoryHealth {
            total_memories: total,
            total_accesses,
            avg_access_rate: round_decimals(avg_access, 2),
            type_diversity: type_count,
            author_diversity: author_count,
            backend: stats.backend,
            health_score: round_decimals(capped, 3),
        }
    }
}

impl Default for GlobalMemorySync {
    fn default() -> Self {
        Self::new()
    }
}

// ════════════════════════════════════════════════════════════════════════════
//  Free functions
// ════════════════════════════════════════════════════════════════════════════

/// Patterns that indicate a poisoned/echoed response (1:1 from the Python).
pub const POISON_PATTERNS: [&str; 11] = [
    "[DIRECT NEURAL BRIDGE]",
    "[HMI BRIDGE ACTIVE]",
    "[EXECUTION MODE ACTIVE]",
    "[MESH OBSERVABILITY]",
    "[STRICT CONSTRAINT]",
    "I am currently processing this request internally.",
    "My linguistic output is currently stalled",
    "## CRITICAL: Action Execution Rules",
    "WRITE_FILE: architecture_overview.md",
    "SEARCH_QUERY:",
    "[Archi:Architect] Task:",
];

pub fn is_poisoned(text: &str) -> bool {
    POISON_PATTERNS.iter().any(|p| text.contains(p))
}

/// `s.chars().take(n)` — Python's `s[:n]` is a CODE POINT slice; Rust's
/// `s[..n]` is a BYTE slice and panics on a non-boundary. Use this everywhere.
pub fn trunc_chars(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

fn bump_counter(counter: &mut Vec<(String, u64)>, key: &str) {
    for e in counter.iter_mut() {
        if e.0 == key {
            e.1 += 1;
            return;
        }
    }
    counter.push((key.to_string(), 1));
}

/// Note 4: Python `round(x, n)` — half-to-even on the exact decimal value.
/// n is small (2 or 3) in this module. Returns x unchanged for non-finite or
/// negative-n inputs.
pub fn round_decimals(x: f64, n: u32) -> f64 {
    if !x.is_finite() {
        return x;
    }
    let keep = n as usize;
    // Correctly-rounded decimal expansion with 40 guard digits: any tie
    // decision that the exact binary value does not actually meet is visible
    // in those digits.
    let s = format!("{:.*}", keep + 40, x);
    let (neg, body) = match s.strip_prefix('-') {
        Some(r) => (true, r),
        None => (false, s.as_str()),
    };
    let (int_part, frac_part) = match body.find('.') {
        Some(i) => (&body[..i], &body[i + 1..]),
        None => (body, ""),
    };

    let fd: Vec<char> = frac_part.chars().collect();
    if fd.len() <= keep {
        return x; // already exact at this precision
    }

    let mut up = false;
    match fd[keep] {
        '5' => {
            if fd[keep + 1..].iter().any(|&c| c != '0') {
                up = true; // strictly above the tie
            } else {
                // exact tie -> half to even
                let last = fd[keep - 1];
                up = ((last as u32).wrapping_sub('0' as u32)) % 2 == 1;
            }
        }
        c if c > '5' => up = true,
        _ => {}
    }

    let mut comb: Vec<char> = int_part.chars().collect();
    comb.extend(fd.iter().take(keep));
    if up {
        let mut i = comb.len();
        loop {
            if i == 0 {
                comb.insert(0, '1');
                break;
            }
            i -= 1;
            if comb[i] == '9' {
                comb[i] = '0';
            } else {
                comb[i] = char::from_u32(comb[i] as u32 + 1).unwrap_or(comb[i]);
                break;
            }
        }
    }

    let len = comb.len();
    let split = len.saturating_sub(keep);
    let int_s: String = comb[..split].iter().collect();
    let frac_s: String = comb[split..].iter().collect();
    let mut text = String::new();
    if neg {
        text.push('-');
    }
    text.push_str(if int_s.is_empty() { "0" } else { &int_s });
    if keep > 0 {
        text.push('.');
        text.push_str(&frac_s);
    }
    text.parse::<f64>().unwrap_or(x)
}

// ════════════════════════════════════════════════════════════════════════════
//  CPython-compatible string hash (note 5)
// ════════════════════════════════════════════════════════════════════════════

const MASK64: u64 = u64::MAX;

#[inline]
fn rotl(x: u64, b: u32) -> u64 {
    (x << b) | (x >> (64 - b))
}

/// CPython's SipHash-1-3 (`Python/pyhash.c`), which is what `hash(str)` is
/// when `PYTHONHASHSEED=0` (key = 0). Verified against CPython 3.11.
fn siphash13(k0: u64, k1: u64, data: &[u8]) -> u64 {
    let mut v0 = k0 ^ 0x736f_6d65_7073_6575;
    let mut v1 = k1 ^ 0x646f_7261_6e64_6f6d;
    let mut v2 = k0 ^ 0x6c79_6765_6e65_7261;
    let mut v3 = k1 ^ 0x7465_6462_7974_6573;

    macro_rules! half_round {
        ($a:ident, $b:ident, $c:ident, $d:ident, $s:expr, $t:expr) => {
            $a = $a.wrapping_add($b);
            $c = $c.wrapping_add($d);
            $b = rotl($b, $s) ^ $a;
            $d = rotl($d, $t) ^ $c;
            $a = rotl($a, 32);
        };
    }
    macro_rules! single_round {
        () => {
            half_round!(v0, v1, v2, v3, 13, 16);
            half_round!(v2, v1, v0, v3, 17, 21);
        };
    }

    let len = data.len();
    let mut b: u64 = (len as u64 & 0xff) << 56;
    let mut i = 0usize;
    while len - i >= 8 {
        let mi = u64::from_le_bytes([
            data[i], data[i + 1], data[i + 2], data[i + 3],
            data[i + 4], data[i + 5], data[i + 6], data[i + 7],
        ]);
        i += 8;
        v3 ^= mi;
        single_round!();
        v0 ^= mi;
    }

    let mut t: u64 = 0;
    let rem = &data[i..];
    let mut j = 0usize;
    while j < rem.len() {
        t |= (rem[j] as u64) << (8 * j);
        j += 1;
    }
    b |= t;
    v3 ^= b;
    single_round!();
    v0 ^= b;
    v2 ^= 0xff;
    for _ in 0..3 {
        single_round!();
    }
    (v0 ^ v1 ^ v2 ^ v3) & MASK64
}

/// `hash(str)` under `PYTHONHASHSEED=0` (CPython special-cases "" -> 0).
pub fn py_hash_str(s: &str) -> u64 {
    if s.is_empty() {
        return 0;
    }
    siphash13(0, 0, s.as_bytes())
}

// ════════════════════════════════════════════════════════════════════════════
//  DGE policy (note 7): the three PII regexes as hand-written byte scanners
// ════════════════════════════════════════════════════════════════════════════

#[inline]
fn is_sep(b: u8) -> bool {
    // Python char class ["\s:=]
    matches!(b, b'"' | b':' | b'=' | b' ' | b'\t' | b'\n' | b'\r' | 0x0b | 0x0c)
}

#[inline]
fn is_email_local(b: u8) -> bool {
    b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'%' | b'+' | b'-')
}

#[inline]
fn is_email_domain(b: u8) -> bool {
    b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-')
}

#[inline]
fn is_re_word(b: u8) -> bool {
    // Approximation: Python's \b/\w are Unicode-aware; ASCII + any high byte is
    // the closest cheap stand-in (documented limitation for non-ASCII input).
    b.is_ascii_alphanumeric() || b == b'_' || b >= 0x80
}

fn all_digits(s: &[u8], pos: usize, len: usize) -> bool {
    if pos + len > s.len() {
        return false;
    }
    s[pos..pos + len].iter().all(|b| b.is_ascii_digit())
}

/// `[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}` — leftmost match in
/// regex backtracking order.
fn find_email(s: &[u8], from: usize) -> Option<(usize, usize)> {
    let n = s.len();
    let mut i = from;
    while i < n {
        if is_email_local(s[i]) {
            let mut e1 = i;
            while e1 < n && is_email_local(s[e1]) {
                e1 += 1;
            }
            let mut k = e1;
            while k > i {
                if k < n && s[k] == b'@' {
                    let mut m = k + 1;
                    while m < n && is_email_domain(s[m]) {
                        m += 1;
                    }
                    if m >= k + 2 {
                        let mut l = m - k - 1;
                        while l >= 1 {
                            let dot = k + 1 + l;
                            if dot < n && s[dot] == b'.' {
                                let t0 = dot + 1;
                                let mut t1 = t0;
                                while t1 < n && s[t1].is_ascii_alphabetic() {
                                    t1 += 1;
                                }
                                if t1 - t0 >= 2 {
                                    return Some((i, t1));
                                }
                            }
                            l -= 1;
                        }
                    }
                }
                k -= 1;
            }
        }
        i += 1;
    }
    None
}

/// `(?:key|token|secret|password)["\s:=]+[a-zA-Z0-9]{16,}`
fn find_api_key(s: &[u8], from: usize) -> Option<(usize, usize)> {
    const KEYWORDS: [&[u8]; 4] = [b"key", b"token", b"secret", b"password"];
    let n = s.len();
    let mut i = from;
    while i < n {
        for kw in KEYWORDS.iter() {
            if i + kw.len() <= n && &s[i..i + kw.len()] == *kw {
                let mut j = i + kw.len();
                let sep_start = j;
                while j < n && is_sep(s[j]) {
                    j += 1;
                }
                if j > sep_start {
                    let mut e = j;
                    while e < n && s[e].is_ascii_alphanumeric() {
                        e += 1;
                    }
                    if e - j >= 16 {
                        return Some((i, e));
                    }
                }
            }
        }
        i += 1;
    }
    None
}

/// `\b(?:\d{1,3}\.){3}\d{1,3}\b`
fn find_ipv4(s: &[u8], from: usize) -> Option<(usize, usize)> {
    let n = s.len();
    let mut i = from;
    while i < n {
        if s[i].is_ascii_digit() && (i == 0 || !is_re_word(s[i - 1])) {
            let mut g1 = 3;
            while g1 >= 1 {
                if all_digits(s, i, g1) {
                    let p1 = i + g1;
                    if p1 < n && s[p1] == b'.' {
                        let mut g2 = 3;
                        while g2 >= 1 {
                            if all_digits(s, p1 + 1, g2) {
                                let p2 = p1 + 1 + g2;
                                if p2 < n && s[p2] == b'.' {
                                    let mut g3 = 3;
                                    while g3 >= 1 {
                                        if all_digits(s, p2 + 1, g3) {
                                            let p3 = p2 + 1 + g3;
                                            if p3 < n && s[p3] == b'.' {
                                                let mut g4 = 3;
                                                while g4 >= 1 {
                                                    if all_digits(s, p3 + 1, g4) {
                                                        let e = p3 + 1 + g4;
                                                        if e >= n || !is_re_word(s[e]) {
                                                            return Some((i, e));
                                                        }
                                                    }
                                                    g4 -= 1;
                                                }
                                            }
                                        }
                                        g3 -= 1;
                                    }
                                }
                            }
                            g2 -= 1;
                        }
                    }
                }
                g1 -= 1;
            }
        }
        i += 1;
    }
    None
}

fn sub_all(s: &str, finder: fn(&[u8], usize) -> Option<(usize, usize)>, repl: &str) -> String {
    let b = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(b.len());
    let mut pos = 0usize;
    while pos < b.len() {
        match finder(b, pos) {
            Some((s0, e0)) if e0 > s0 => {
                out.extend_from_slice(&b[pos..s0]);
                out.extend_from_slice(repl.as_bytes());
                pos = e0;
            }
            _ => break,
        }
    }
    out.extend_from_slice(&b[pos..]);
    // Every span this can cut on is pure ASCII (all matchers are byte-class
    // ASCII), so out is valid UTF-8; the fallback is defensive only.
    String::from_utf8(out).unwrap_or_else(|_| s.to_string())
}

/// `dge.mask_pii(text)` — sequential `re.sub`, in the Python dict's order.
pub fn dge_mask_pii(text: &str) -> String {
    let a = sub_all(text, find_email, "[MASKED_EMAIL]");
    let b = sub_all(&a, find_api_key, "[MASKED_API_KEY]");
    sub_all(&b, find_ipv4, "[MASKED_IPV4]")
}

/// `dge.validate_memory_contribution(agent, content) -> bool`
/// (rejects raw secrets; the audit-log line is a side effect -> dropped).
pub fn dge_validate_memory_contribution(_agent: &str, content: &str) -> bool {
    find_api_key(content.as_bytes(), 0).is_none()
}

/// `json.dumps(tags)` with `ensure_ascii=True` (Python's default), for the
/// persistence boundary and for the `tags` field's on-disk representation.
pub fn tags_to_json(tags: &[String]) -> String {
    let mut out = String::from("[");
    for (i, t) in tags.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        out.push('"');
        for c in t.chars() {
            match c {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\t' => out.push_str("\\t"),
                '\u{8}' => out.push_str("\\b"),
                '\u{c}' => out.push_str("\\f"),
                c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
                c if (c as u32) >= 0x80 => {
                    let cp = c as u32;
                    if cp > 0xFFFF {
                        let v = cp - 0x10000;
                        out.push_str(&format!(
                            "\\u{:04x}\\u{:04x}",
                            0xD800 + (v >> 10),
                            0xDC00 + (v & 0x3FF)
                        ));
                    } else {
                        out.push_str(&format!("\\u{:04x}", cp));
                    }
                }
                c => out.push(c),
            }
        }
        out.push('"');
    }
    out.push(']');
    out
}

// ════════════════════════════════════════════════════════════════════════════
//  Singletons
// ════════════════════════════════════════════════════════════════════════════

static SINGLETON_GLOBAL_MEMORY: core::sync::atomic::AtomicU64 =
    core::sync::atomic::AtomicU64::new(0);

/// Pre-existing handle-token accessor. Kept for API compatibility (the kernel
/// uses this idiom in dge.rs). This is NOT the Python accessor — see
/// `get_global_memory_instance` for that.
pub fn get_global_memory() -> u64 {
    let v = SINGLETON_GLOBAL_MEMORY.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_GLOBAL_MEMORY.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}

static mut GLOBAL_MEMORY_INSTANCE: Option<GlobalMemorySync> = None;

/// Python `get_global_memory()`: lazily-created process-wide singleton.
///
/// SAFETY: single-threaded UEFI payload (same contract as
/// `agent_runtime::get_runtime`); initialised once before any use.
#[allow(static_mut_refs)]
pub fn get_global_memory_instance() -> &'static mut GlobalMemorySync {
    unsafe {
        if GLOBAL_MEMORY_INSTANCE.is_none() {
            GLOBAL_MEMORY_INSTANCE = Some(GlobalMemorySync::new());
        }
        GLOBAL_MEMORY_INSTANCE.as_mut().unwrap()
    }
}
