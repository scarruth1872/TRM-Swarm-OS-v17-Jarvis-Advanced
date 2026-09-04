//! TRM Swarm OS — trm_orchestrator (no_std port).
//! Deterministic core ported 1:1 from swarm_v2/core/trm_orchestrator.py:
//! SuperpositionState collapse/consensus, partition_context (line/token/word/
//! char-chunk partitioning), sub-task confidence schedule, symbolic detection,
//! context→token conversion, merge_contexts, round-robin core allocation and
//! the EMA metrics update. Async/thread/psutil/GPU bodies are // MANUAL (no_std
//! has no executor, filesystem probes or torch).

#![allow(dead_code)]

extern crate alloc;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

/// Default confidence threshold used by spawn_sub_reasoning / collapse.
pub const DEFAULT_COLLAPSE_THRESHOLD: f64 = 0.8;
/// Compute target: CPU cores (the only deterministic default target).
pub const COMPUTE_CPU_CORES: u8 = 0;
/// Compute target: GPU model execution (// MANUAL resource path).
pub const COMPUTE_GPU_MODEL: u8 = 1;

/// Interpretation discriminator: text partition vs symbolic token result.
pub const INTERP_TEXT: u8 = 0;
pub const INTERP_SYMBOLIC: u8 = 1;

// ---------------------------------------------------------------------------
// Small helpers shared across the module
// ---------------------------------------------------------------------------

/// Round-robin schedule of sub-tasks onto available CPU core ids.
/// Mirrors _allocate_resources(): sub-task i gets [cores[i]] while cores
/// last; GPU memory (512MB/task) is only granted to GPU_MODEL targets when a
/// non-None GPU pool > 512MB exists. `available` lists actual core ids
/// (python: cores [2,3] when 4 logical cores exist — 2 reserved for OS).
pub fn allocate_cores_round_robin(task_count: u64, available: &[u64]) -> Vec<Vec<u64>> {
    let mut assignments: Vec<Vec<u64>> = vec![vec![]; task_count as usize];
    if available.is_empty() {
        return assignments;
    }
    let n = core::cmp::min(task_count as usize, available.len());
    for i in 0..n {
        assignments[i] = vec![available[i]];
    }
    assignments
}

/// _detect_available_cores: python psutil shape — reserve 2 cores for the
/// OS/system when more than 2 exist, else all cores.
pub fn detect_available_cores(total_cores: u64) -> Vec<u64> {
    if total_cores > 2 {
        (2..total_cores).collect()
    } else {
        (0..total_cores).collect()
    }
}

/// no_std floor (truncation + sign correction) — f64 lacks floor().
fn floor_f64(x: f64) -> f64 {
    let t = x as i64 as f64;
    if x < t {
        t - 1.0
    } else {
        t
    }
}

/// Deterministic 64-bit FNV-1a string hash (replaces non-deterministic
/// Python `hash(part)` fallback used by _context_to_tokens).
pub fn fnv1a_str(s: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in s.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// Python `abs(hash(part)) % 10` is per-process random (PYTHONHASHSEED).
/// Deterministic replacement: FNV-1a folded to 0..=9.
pub fn hash_mod10(s: &str) -> i64 {
    (fnv1a_str(s) % 10) as i64
}

/// _looks_like_symbolic: numeric+whitespace chars / len > 0.7.
pub fn looks_like_symbolic(context: &str) -> bool {
    if context.is_empty() {
        return false;
    }
    let len = context.chars().count() as f64;
    if len == 0.0 {
        return false;
    }
    let mut numeric_chars = 0u64;
    for c in context.chars() {
        if c.is_ascii_digit() || c.is_ascii_whitespace() {
            numeric_chars += 1;
        }
    }
    (numeric_chars as f64) / len > 0.7
}

/// _context_to_tokens: whitespace split; int() when possible else hash%10.
pub fn context_to_tokens(context: &str) -> Vec<i64> {
    let mut tokens = vec![];
    for part in context.split_whitespace() {
        match part.parse::<i64>() {
            Ok(v) => tokens.push(v),
            Err(_) => tokens.push(hash_mod10(part)),
        }
    }
    tokens
}

/// Confidence schedule from _execute_single_task:
/// conf = clamp(0.7 - depth*0.1, 0.3, 0.9).
pub fn depth_confidence(depth: u64) -> f64 {
    let c = 0.7 - (depth as f64) * 0.1;
    if c < 0.3 {
        0.3
    } else if c > 0.9 {
        0.9
    } else {
        c
    }
}

// ---------------------------------------------------------------------------
// SuperpositionState (deterministic core)
// ---------------------------------------------------------------------------

/// A collapsed interpretation: the winning content (text/symbolic) plus
/// bookkeeping python attaches during collapse().
#[derive(Clone, Debug)]
pub struct CollapsedInterpretation {
    pub interp_type: u8,
    pub content: String,
    pub confidence: f64,
    pub alternative_count: u64,
}

/// Weighted-consensus summary from SuperpositionState.get_consensus().
#[derive(Clone, Debug)]
pub struct ConsensusSummary {
    pub interpretation_count: u64,
    pub avg_confidence: f64,
    pub max_confidence: f64,
    pub collapsed: bool,
}

/// SuperpositionState: deterministic collapse/consensus over a fixed array of
/// (interpretation_type, content, confidence) triples.
pub struct SuperpositionState {
    pub interp_types: Vec<u8>,
    pub contents: Vec<String>,
    pub confidences: Vec<f64>,
}

impl SuperpositionState {
    pub fn new() -> Self {
        SuperpositionState {
            interp_types: vec![],
            contents: vec![],
            confidences: vec![],
        }
    }

    pub fn is_empty(&self) -> bool {
        self.confidences.is_empty()
    }

    pub fn len(&self) -> usize {
        self.confidences.len()
    }

    /// add_interpretation: record (type, content, confidence).
    pub fn add_interpretation(&mut self, interp_type: u8, content: String, confidence: f64) {
        self.interp_types.push(interp_type);
        self.contents.push(content);
        self.confidences.push(confidence);
    }

    /// merge(other): extend with other's interpretations; adopt other's
    /// content when this state has none and other is non-empty.
    pub fn merge(&mut self, other: &SuperpositionState) {
        for i in 0..other.confidences.len() {
            self.interp_types.push(other.interp_types[i]);
            self.contents.push(other.contents[i].clone());
            self.confidences.push(other.confidences[i]);
        }
    }

    /// collapse(threshold): pick highest-confidence interpretation if it
    /// clears `threshold`, tagging confidence + alternative_count.
    pub fn collapse(&self, threshold: f64) -> Option<CollapsedInterpretation> {
        if self.confidences.is_empty() {
            return None;
        }
        let mut max_idx = 0usize;
        let mut max_confidence = self.confidences[0];
        for i in 1..self.confidences.len() {
            if self.confidences[i] > max_confidence {
                max_confidence = self.confidences[i];
                max_idx = i;
            }
        }
        if max_confidence < threshold {
            return None;
        }
        Some(CollapsedInterpretation {
            interp_type: self.interp_types[max_idx],
            content: self.contents[max_idx].clone(),
            confidence: max_confidence,
            alternative_count: self.confidences.len() as u64,
        })
    }

    /// get_consensus: weighted summary over all interpretations.
    pub fn get_consensus(&self) -> Option<ConsensusSummary> {
        if self.confidences.is_empty() {
            return None;
        }
        let n = self.confidences.len() as f64;
        let mut total = 0.0f64;
        let mut max_confidence = 0.0f64;
        for c in &self.confidences {
            total += c;
            if *c > max_confidence {
                max_confidence = *c;
            }
        }
        if total == 0.0 {
            return None;
        }
        Some(ConsensusSummary {
            interpretation_count: self.confidences.len() as u64,
            avg_confidence: total / n,
            max_confidence,
            collapsed: false,
        })
    }
}

// ---------------------------------------------------------------------------
// partition_context — the deterministic core of TRMOrchestrator
// ---------------------------------------------------------------------------

/// Partition a context for parallel processing (python semantics):
/// multi-line -> group lines; single line -> symbolic token partition or
/// word partition, else char chunks (>=10 chars) or the whole line.
pub fn partition_context(context: &str, num_partitions: u64) -> Vec<String> {
    if context.is_empty() {
        return vec![];
    }
    let np = if num_partitions == 0 { 1 } else { num_partitions };

    let lines: Vec<&str> = context.split('\n').collect();
    let mut result = vec![];

    if lines.len() > 1 {
        if (lines.len() as u64) > np {
            // group lines into `np` partitions
            let partition_size = (lines.len() as u64) / np;
            for i in 0..np {
                let start = (i * partition_size) as usize;
                let end = if i < np - 1 {
                    ((i + 1) * partition_size) as usize
                } else {
                    lines.len()
                };
                let part = lines[start..end].join("\n");
                if !part.is_empty() {
                    result.push(part);
                }
            }
        } else {
            // each non-blank line its own partition
            for line in lines {
                if !line.trim().is_empty() {
                    result.push(line.to_string());
                }
            }
        }
        return result;
    }

    // single line
    let line = lines[0];
    let nchars = line.chars().count() as f64;
    if nchars == 0.0 {
        return vec![];
    }
    let mut numeric_chars = 0u64;
    for c in line.chars() {
        if c.is_ascii_digit() || c.is_ascii_whitespace() {
            numeric_chars += 1;
        }
    }
    let is_symbolic = (numeric_chars as f64) / nchars > 0.7;

    if is_symbolic {
        let tokens: Vec<&str> = line.split_whitespace().collect();
        if (tokens.len() as u64) > np {
            let partition_size = (tokens.len() as u64) / np;
            for i in 0..np {
                let start = (i * partition_size) as usize;
                let end = if i < np - 1 {
                    ((i + 1) * partition_size) as usize
                } else {
                    tokens.len()
                };
                let part = tokens[start..end].join(" ");
                if !part.is_empty() {
                    result.push(part);
                }
            }
        } else {
            for t in tokens {
                if !t.is_empty() {
                    result.push(t.to_string());
                }
            }
        }
        return result;
    }

    // text line
    let words: Vec<&str> = line.split_whitespace().collect();
    if (words.len() as u64) > np * 5 {
        let words_per_partition = (words.len() as u64) / np;
        for i in 0..np {
            let start = (i * words_per_partition) as usize;
            let end = if i < np - 1 {
                ((i + 1) * words_per_partition) as usize
            } else {
                words.len()
            };
            let part = words[start..end].join(" ");
            if !part.is_empty() {
                result.push(part);
            }
        }
        return result;
    }

    // equal character chunks
    let chunk_size = (line.chars().count() as u64) / np;
    if chunk_size < 10 {
        // too small — do not partition
        return vec![line.to_string()];
    }
    let char_vec: Vec<char> = line.chars().collect();
    for i in 0..np {
        let start = (i * chunk_size) as usize;
        let end = if i < np - 1 {
            ((i + 1) * chunk_size) as usize
        } else {
            char_vec.len()
        };
        if end > start {
            let part: String = char_vec[start..end].iter().collect();
            if !part.is_empty() {
                result.push(part);
            }
        }
    }
    result
}

/// Group partition results: join symbolic parts by spaces, text parts by
/// newlines; then join merged parts by newlines (python _merge_contexts).
pub fn merge_contexts(interp_types: &[u8], contents: &[String]) -> String {
    let mut merged_parts: Vec<String> = vec![];
    for i in 0..interp_types.len() {
        if interp_types[i] == INTERP_SYMBOLIC {
            // python re-joins the token slice " ".join(map(str, tokens)).
            // Our content already carries the merged partition text.
            if !contents[i].is_empty() {
                merged_parts.push(contents[i].clone());
            }
        } else if interp_types[i] == INTERP_TEXT {
            if !contents[i].is_empty() {
                merged_parts.push(contents[i].clone());
            }
        }
    }
    merged_parts.join("\n")
}

// ---------------------------------------------------------------------------
// spawn_sub_reasoning — deterministic synchronous emulation of the async
// recursion. Python detail: a sub-task whose partition looks symbolic routes
// through trm_brain and returns result_tokens[:len(tokens)] (identity for a
// faithful reason()); a text partition returns its content with
// confidence = clamp(0.7 - depth*0.1). Collapse requires the winning
// confidence to clear the threshold; text partitions at depth 0 carry
// conf 0.7 < 0.8 so they never collapse (python behavior for pure text).
// ---------------------------------------------------------------------------

/// Result discriminator of spawn_sub_reasoning.
pub const SPAWN_COLLAPSED: u8 = 0;
pub const SPAWN_MAX_DEPTH: u8 = 1;
pub const SPAWN_NO_PARTITIONS: u8 = 2;
pub const SPAWN_INCONCLUSIVE: u8 = 3;

/// Deterministic outcome of one spawn_sub_reasoning round.
#[derive(Clone, Debug)]
pub struct SpawnResult {
    pub status: u8,
    pub depth: u64,
    pub parallel_tasks: u64,
    pub collapsed: Option<CollapsedInterpretation>,
    pub consensus: Option<ConsensusSummary>,
    /// depth<max_depth and merge produced a non-empty context
    pub recursed: bool,
    pub merged_context: String,
}

/// Faithful deterministic emulation of the orchestrator recursion. Text
/// partitions are treated as `interpretation` results (python: `result` is a
/// dict with type/content), symbolic partitions run the deterministic token
/// identity reasoner. Returns (result, executed_depth) — executed_depth is 0
/// on max_depth/no_partitions, else the depth of the terminal round.
pub fn spawn_sub_reasoning_deterministic(
    context: &str,
    max_depth: u64,
    num_partitions: u64,
    confidence_threshold: f64,
) -> SpawnResult {
    let mut depth = 0u64;
    let mut current_context = context.to_string();

    loop {
        if depth >= max_depth {
            return SpawnResult {
                status: SPAWN_MAX_DEPTH,
                depth,
                parallel_tasks: 0,
                collapsed: None,
                consensus: None,
                recursed: false,
                merged_context: String::new(),
            };
        }

        let partitions = partition_context(&current_context, num_partitions);
        if partitions.is_empty() {
            return SpawnResult {
                status: SPAWN_NO_PARTITIONS,
                depth,
                parallel_tasks: 0,
                collapsed: None,
                consensus: None,
                recursed: false,
                merged_context: String::new(),
            };
        }

        // deterministic "parallel execution" of each partition
        let mut sup = SuperpositionState::new();
        for part in &partitions {
            let interp_type;
            let content;
            if looks_like_symbolic(part) {
                let tokens = context_to_tokens(part);
                let result_tokens: Vec<String> =
                    tokens.iter().map(|t| t.to_string()).collect();
                interp_type = INTERP_SYMBOLIC;
                content = result_tokens.join(" ");
            } else {
                interp_type = INTERP_TEXT;
                content = part.clone();
            }
            sup.add_interpretation(interp_type, content, depth_confidence(depth));
        }

        let collapsed = sup.collapse(confidence_threshold);
        if collapsed.is_some() {
            return SpawnResult {
                status: SPAWN_COLLAPSED,
                depth,
                parallel_tasks: partitions.len() as u64,
                collapsed,
                consensus: None,
                recursed: false,
                merged_context: String::new(),
            };
        }

        // inconclusive: maybe recurse with a merged context
        let merged = merge_contexts(&sup.interp_types, &sup.contents);
        let mut recurse = false;
        if depth < max_depth - 1 {
            if !merged.is_empty() {
                recurse = true;
            }
        }

        if recurse {
            current_context = merged;
            depth += 1;
            continue;
        }

        let consensus = sup.get_consensus();
        return SpawnResult {
            status: SPAWN_INCONCLUSIVE,
            depth,
            parallel_tasks: partitions.len() as u64,
            collapsed: None,
            consensus,
            recursed: false,
            merged_context: merged,
        };
    }
}

// ---------------------------------------------------------------------------
// Metrics — running EMA updates (python _update_metrics)
// ---------------------------------------------------------------------------

/// Orchestrator performance counters, mirroring python `self.metrics`.
#[derive(Clone, Debug)]
pub struct OrcMetrics {
    pub total_tasks: u64,
    pub parallel_tasks: u64,
    pub avg_response_time: f64,
    pub error_rate: f64,
}

impl Default for OrcMetrics {
    fn default() -> Self {
        OrcMetrics {
            total_tasks: 0,
            parallel_tasks: 0,
            avg_response_time: 0.0,
            error_rate: 0.0,
        }
    }
}

/// Update running averages after a batch: python
///   avg_time = EMA(avg*0.9 + avg_time*0.1); error_rate = EMA(prev*0.9+rate*0.1)
/// where times are per-sub-task durations, errors are failed tasks. Returns
/// the updated metrics. A zero-length `times` batch only bumps nothing
/// (python: completed==[] -> return).
pub fn update_metrics(m: &mut OrcMetrics, times: &[f64], error_count: u64, sub_task_count: u64) {
    if times.is_empty() {
        return;
    }
    let mut sum = 0.0f64;
    for t in times {
        sum += t;
    }
    let avg_time = sum / times.len() as f64;
    m.avg_response_time = m.avg_response_time * 0.9 + avg_time * 0.1;

    let completed = times.len() as f64;
    let error_rate = (error_count as f64) / completed;
    m.error_rate = m.error_rate * 0.9 + error_rate * 0.1;

    m.total_tasks += completed as u64;
    if sub_task_count > m.parallel_tasks {
        m.parallel_tasks = sub_task_count;
    }
}

// ---------------------------------------------------------------------------
// Module-level status snapshot + singleton handle
// ---------------------------------------------------------------------------

/// TRMOrchestrator.get_status() — pure summary built from supplied state.
#[derive(Clone, Debug)]
pub struct OrcStatus {
    pub active_tasks: u64,
    pub subagents: u64,
    pub available_cores: u64,
    pub gpu_memory_mb: Option<u64>,
    pub max_parallel_tasks: u64,
    pub metrics: OrcMetrics,
    pub superposition_states: u64,
}

/// get_trm_orchestrator singleton — mirror of python's lazy `_orchestrator`.
pub fn get_trm_orchestrator() -> u64 {
    0x5452_4D4F_5243_4853
}

// ---------------------------------------------------------------------------
// // MANUAL — async/OS/GPU bodies that cannot run in no_std
// ---------------------------------------------------------------------------

/// _estimate_gpu_memory: torch.cuda probe. // MANUAL -> None.
pub fn estimate_gpu_memory_manual() -> Option<u64> {
    None
}

/// create_subagent / _execute_parallel / cleanup touch live agent objects
/// and thread pools. // MANUAL -> 0/false.
pub fn create_subagent_manual() -> u64 {
    0
}
