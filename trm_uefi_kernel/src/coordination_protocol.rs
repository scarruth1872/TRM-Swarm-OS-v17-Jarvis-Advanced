//! TRM Swarm OS — coordination_protocol (no_std port, REAL-verified core).
//! 1:1 deterministic semantics of swarm_v2/core/coordination_protocol.py:
//!   - round record {task, context, sent_to, responses, started_at, timeout}
//!   - broadcast_task: round_id -> sent_to (all mailbox sends succeed core)
//!   - collect_responses: per-agent proposal merge (arrival order, overwrite
//!     per agent), response_count / agents_sent / elapsed=round(now-started,1)
//!   - consensus_aggregate: majority (Counter semantics, first-seen tie
//!     order), all, summary, unknown-method — python-identical
//! Python-bound parts (uuid RNG in round_id, asyncio mailbox loops, ultrawork
//! missions, run_coordination_round orchestration) are I/O — deterministic
//! equivalents take injected round_id / response pairs; the rest is MANUAL.

extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

/// One agent's response, preserving arrival order (python dict insertion).
#[derive(Debug, Clone, PartialEq)]
pub struct ResponseEntry {
    pub agent: String,
    pub proposal: String,
}

/// Mirror of python round record stored in `active_rounds[round_id]`.
#[derive(Debug, Clone)]
pub struct RoundRecord {
    pub task: String,
    pub context: String,
    pub sent_to: Vec<String>,
    pub responses: Vec<ResponseEntry>, // responses[agent]=proposal in arrival order
    pub started_at: f64,
    pub timeout: f64,
}

/// Result of broadcast_task — python returns
/// {"round_id", "agents": sent_to, "status": "broadcast"}.
#[derive(Debug, Clone, PartialEq)]
pub struct BroadcastResult {
    pub round_id: String,
    pub agents: Vec<String>,
    pub status: String,
}

/// Result of collect_responses — python dict minus the mailbox-only parts.
#[derive(Debug, Clone, PartialEq)]
pub struct RoundSummary {
    pub round_id: String,
    pub task: String,
    pub responses: Vec<(String, String)>, // agent -> proposal (arrival order)
    pub response_count: usize,
    pub agents_sent: usize,
    pub elapsed: f64, // python round(time.time() - started_at, 1)
}

/// Aggregated consensus value from consensus_aggregate.
#[derive(Debug, Clone, PartialEq)]
pub enum ConsensusValue {
    None,
    Single(String),            // majority winner
    All(Vec<String>),          // method "all": response list in order
    Summary(String),           // method "summary": rendered text
}

/// Result of consensus_aggregate mirroring the python return dicts.
#[derive(Debug, Clone, PartialEq)]
pub struct AggregateResult {
    pub method: String,
    pub consensus: ConsensusValue,
    pub votes: Vec<(String, u64)>, // majority counts, first-seen order
    pub total: usize,              // len(responses) for known methods
    pub count: usize,              // 0 for the empty-responses early return
    pub error: Option<String>,
}

/// Python round-half-even at 1 decimal for positive values (no_std).
fn round_1dec(x: f64) -> f64 {
    let q = x * 10.0;
    let t = q as i64; // trunc toward zero; callers pass x >= 0
    let f = q - t as f64;
    let mut r = if f < 0.5 {
        t
    } else if f > 0.5 {
        t + 1
    } else if t % 2 == 0 {
        t
    } else {
        t + 1
    };
    r as f64 / 10.0
}

/// Agent Coordination Protocol — multi-agent task broadcast, bid,
/// consensus, and aggregation.
#[derive(Debug, Clone)]
pub struct CoordinationProtocol {
    pub active_rounds: BTreeMap<String, RoundRecord>,
}

impl CoordinationProtocol {
    /// python `__init__`: active_rounds = {}.
    pub fn new() -> Self {
        CoordinationProtocol { active_rounds: BTreeMap::new() }
    }

    /// Deterministic core of python `broadcast_task`: registers the round
    /// record and returns the broadcast result. Python builds round_id as
    /// "coord-" + uuid4().hex[:8] (RNG) — no_std takes the id as a parameter
    /// and the mailbox side-effect (send to each agent) is reduced to its
    /// deterministic core: every agent that would receive a message appears
    /// in sent_to (python appends on successful send; send failure is I/O).
    pub fn broadcast_task_with(
        &mut self,
        round_id: &str,
        task: &str,
        context: &str,
        target_agents: &[String],
        timeout: f64,
        now: f64,
    ) -> BroadcastResult {
        let sent_to: Vec<String> = target_agents.to_vec();
        self.active_rounds.insert(
            round_id.to_string(),
            RoundRecord {
                task: task.to_string(),
                context: context.to_string(),
                sent_to: sent_to.clone(),
                responses: Vec::new(),
                started_at: now,
                timeout,
            },
        );
        BroadcastResult {
            round_id: round_id.to_string(),
            agents: sent_to,
            status: "broadcast".to_string(),
        }
    }

    /// Deterministic core of the collect_responses mailbox loop: python does
    /// responses[agent] = body["proposal"] for each message whose body
    /// round_id matches. Duplicate agents overwrite in place (dict assign).
    /// Returns false when the round does not exist (python returns an error
    /// dict for unknown round_id).
    pub fn record_response(&mut self, round_id: &str, from_agent: &str, proposal: &str) -> bool {
        let rec = match self.active_rounds.get_mut(round_id) {
            Some(r) => r,
            None => return false,
        };
        match rec.responses.iter_mut().find(|e| e.agent == from_agent) {
            Some(e) => e.proposal = proposal.to_string(),
            None => rec.responses.push(ResponseEntry {
                agent: from_agent.to_string(),
                proposal: proposal.to_string(),
            }),
        }
        true
    }

    /// Deterministic summary of python `collect_responses` for a known round:
    /// {"round_id", "task", "responses", "response_count", "agents_sent",
    /// "elapsed": round(now - started_at, 1)}. Python additionally polls the
    /// mailbox and ultrawork missions (I/O) — those fill responses via
    /// record_response. Returns None for an unknown round (python error dict).
    pub fn collect_summary(&self, round_id: &str, now: f64) -> Option<RoundSummary> {
        let rec = self.active_rounds.get(round_id)?;
        Some(RoundSummary {
            round_id: round_id.to_string(),
            task: rec.task.clone(),
            responses: rec
                .responses
                .iter()
                .map(|e| (e.agent.clone(), e.proposal.clone()))
                .collect(),
            response_count: rec.responses.len(),
            agents_sent: rec.sent_to.len(),
            elapsed: round_1dec(now - rec.started_at),
        })
    }

    /// Python `consensus_aggregate(responses, method)` — fully deterministic,
    /// ported 1:1. `responses` is the ordered (agent, proposal) list matching
    /// the python dict insertion order.
    pub fn consensus_aggregate(&self, responses: &[(String, String)], method: &str) -> AggregateResult {
        // python: if not responses: return {consensus: None, method, count: 0}
        if responses.is_empty() {
            return AggregateResult {
                method: method.to_string(),
                consensus: ConsensusValue::None,
                votes: Vec::new(),
                total: 0,
                count: 0,
                error: None,
            };
        }
        if method == "majority" {
            // collections.Counter over response VALUES; votes dict keeps
            // first-seen order; consensus = most_common(1)[0][0] — python's
            // sorted(..., reverse=True) is stable, so equal counts resolve to
            // the first-seen value (strict-> replaces ties, never beats them).
            let mut counts: Vec<(String, u64)> = Vec::new();
            for (_, v) in responses {
                match counts.iter_mut().find(|(val, _)| val == v) {
                    Some(c) => c.1 += 1,
                    None => counts.push((v.clone(), 1)),
                }
            }
            let mut winner: Option<(String, u64)> = None;
            for (v, c) in &counts {
                let replace = match &winner {
                    None => true,
                    Some((_, wc)) => c > wc,
                };
                if replace {
                    winner = Some((v.clone(), *c));
                }
            }
            AggregateResult {
                method: "majority".to_string(),
                consensus: match winner {
                    Some((w, _)) => ConsensusValue::Single(w),
                    None => ConsensusValue::None,
                },
                votes: counts,
                total: responses.len(),
                count: 0,
                error: None,
            }
        } else if method == "all" {
            AggregateResult {
                method: "all".to_string(),
                consensus: ConsensusValue::All(
                    responses.iter().map(|(_, v)| v.clone()).collect(),
                ),
                votes: Vec::new(),
                total: responses.len(),
                count: 0,
                error: None,
            }
        } else if method == "summary" {
            // "Agent responses:\n- {a}: {r}\n..." joined over dict items.
            let mut text = String::from("Agent responses:");
            for (a, r) in responses {
                text.push('\n');
                text.push_str("- ");
                text.push_str(a);
                text.push_str(": ");
                text.push_str(r);
            }
            AggregateResult {
                method: "summary".to_string(),
                consensus: ConsensusValue::Summary(text),
                votes: Vec::new(),
                total: responses.len(),
                count: 0,
                error: None,
            }
        } else {
            AggregateResult {
                method: method.to_string(),
                consensus: ConsensusValue::None,
                votes: Vec::new(),
                total: 0,
                count: 0,
                error: Some(format!("Unknown method: {}", method)),
            }
        }
    }

    /// MANUAL: python `run_coordination_round` chains broadcast -> collect
    /// (mailbox + asyncio waits) -> aggregate; the mailbox/async plumbing is
    /// not portable deterministic logic. Use broadcast_task_with,
    /// record_response + collect_summary, and consensus_aggregate directly.
    pub fn run_coordination_round(&mut self) -> u64 {
        0
    }
}

impl Default for CoordinationProtocol {
    fn default() -> Self {
        CoordinationProtocol::new()
    }
}

/// Python module-level singleton `get_coordination_protocol()` — no_std
/// handle convention: 1 once initialized.
static SINGLETON_COORDINATION_PROTOCOL: core::sync::atomic::AtomicU64 =
    core::sync::atomic::AtomicU64::new(0);
pub fn get_coordination_protocol() -> u64 {
    let v = SINGLETON_COORDINATION_PROTOCOL.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_COORDINATION_PROTOCOL.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}

