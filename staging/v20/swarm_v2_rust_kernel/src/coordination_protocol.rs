//! TRM Swarm OS — Agent Coordination Protocol (real no_std port).
//! 1:1 pure semantics from swarm_v2/core/coordination_protocol.py.
//! Multi-agent task broadcast, bid/response collection, and deterministic
//! consensus aggregation — WITHOUT sockets, asyncio, mailbox, or wall clocks.
//!
//! Ported deterministic pieces: protocol/message constants, message-type
//! classification & validation, round state bookkeeping (sent_to / responses),
//! completion detection, and the three consensus aggregation methods
//! (majority / all / summary) with the Python tie-breaking and empty-input
//! rules reproduced exactly.
//!
//! // MANUAL notes mark uuid4()/wall-clock behaviour that is non-deterministic:
//! // round ids and respond_by deadlines are derived from caller-supplied
//! // seeds/now values via deterministic helpers instead.

extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

// ---------------------------------------------------------------------------
// Protocol constants (1:1 with the Python literals).
// ---------------------------------------------------------------------------
/// Body "type" of every task broadcast (python: "coordination_task").
pub const COORD_MSG_TYPE_TASK: &str = "coordination_task";
/// Mailbox subject prefix used when broadcasting (python: "[COORD] Task: ").
pub const COORD_SUBJECT_PREFIX: &str = "[COORD] Task: ";
/// Mission objective prefix: "[COORD-{round_id}]" (python startswith check).
pub const COORD_MISSION_PREFIX: &str = "[COORD-";
/// Round id prefix: python round_id = f"coord-{uuid4().hex[:8]}".
pub const COORD_ROUND_ID_PREFIX: &str = "coord-";
/// Default broadcast/response timeout in seconds (python: timeout=30).
pub const COORD_DEFAULT_TIMEOUT_S: u64 = 30;
/// Max collect wait in seconds (python: max_wait=60).
pub const COORD_MAX_COLLECT_WAIT_S: u64 = 60;
/// Subject truncation for task text (python: task[:50]).
pub const COORD_SUBJECT_TASK_MAX_CHARS: usize = 50;
/// Mission phase string that marks a mission as finished (python "completed").
pub const COORD_MISSION_PHASE_COMPLETED: &str = "completed";

// ---------------------------------------------------------------------------
// Message-type classification.
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoordMessageKind {
    /// {"type": "coordination_task", ...} — a broadcast task offer.
    Task,
    /// A mailbox body replying to a round (matches round_id; not a Task).
    Response,
    /// An ultrawork mission whose objective starts with "[COORD-{round_id}]"
    /// and whose phase is "completed".
    MissionCompletion,
    Unknown,
}

/// Deterministic classification of an incoming coordination-related message.
///
/// * A body whose "type" field is "coordination_task" -> Task.
/// * A body that carries the round's round_id but is not itself a task -> Response.
/// * A mission objective string matching "[COORD-{round_id}]..." with
///   phase == "completed" -> MissionCompletion.
pub fn classify_coord_message(
    msg_type: Option<&str>,
    round_id_match: bool,
    objective: Option<&str>,
    round_id: Option<&str>,
    phase: Option<&str>,
) -> CoordMessageKind {
    if msg_type == Some(COORD_MSG_TYPE_TASK) {
        return CoordMessageKind::Task;
    }
    if round_id_match {
        return CoordMessageKind::Response;
    }
    if let (Some(obj), Some(rid)) = (objective, round_id) {
        if is_mission_completion(obj, rid, phase) {
            return CoordMessageKind::MissionCompletion;
        }
    }
    CoordMessageKind::Unknown
}

/// python: m.get("objective", "").startswith(f"[COORD-{round_id}]")
///          && m.get("phase") == "completed"
pub fn is_mission_completion(objective: &str, round_id: &str, phase: Option<&str>) -> bool {
    objective.starts_with(&format!("{}{}]", COORD_MISSION_PREFIX, round_id))
        && phase == Some(COORD_MISSION_PHASE_COMPLETED)
}

/// Mission objective prefix used by the python collector:
/// "[COORD-{round_id}]" + task text.
pub fn mission_objective(round_id: &str) -> String {
    format!("{}{}]", COORD_MISSION_PREFIX, round_id)
}

/// Mailbox subject for a task broadcast: "[COORD] Task: {task[:50]}".
pub fn coord_subject(task: &str) -> String {
    let end = task
        .char_indices()
        .nth(COORD_SUBJECT_TASK_MAX_CHARS)
        .map(|(i, _)| i)
        .unwrap_or(task.len());
    format!("{}{}", COORD_SUBJECT_PREFIX, &task[..end])
}

/// // MANUAL: python uses uuid.uuid4().hex[:8] (non-deterministic). The no_std
/// // port derives a deterministic 8-hex-char id from a caller seed via FNV-1a,
/// // keeping the "coord-" prefix and the 8-char hex shape.
pub fn coordination_round_id(seed: u64) -> String {
    // low 32 bits of the FNV digest, rendered as exactly 8 hex chars (like
    // uuid4().hex[:8]).
    format!("{}{:08x}", COORD_ROUND_ID_PREFIX, (fnv1a64(seed) & 0xFFFF_FFFF))
}

/// python: respond_by = time.time() + timeout  (seconds, wall clock).
/// // MANUAL: deterministic port takes now_ms from the caller.
pub fn respond_by_deadline_ms(now_ms: u64, timeout_s: u64) -> u64 {
    now_ms + timeout_s * 1000
}

/// Extract the payload of an agent response body:
/// python: body.get("proposal", body.get("response", "")).
pub fn extract_agent_response(proposal: Option<&str>, response: Option<&str>) -> String {
    match (proposal, response) {
        (Some(p), _) => p.into(),
        (None, Some(r)) => r.into(),
        (None, None) => String::new(),
    }
}

// ---------------------------------------------------------------------------
// Round state (deterministic subset of active_rounds + collect logic).
// ---------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct CoordinationRound {
    pub round_id: String,
    pub task: String,
    pub context: String,
    pub sent_to: Vec<String>,
    /// (agent_id, response) pairs in first-arrival order; dict semantics —
    /// a re-vote from an already-recorded agent replaces its earlier response.
    pub responses: Vec<(String, String)>,
    pub started_at_ms: u64,
    pub timeout_s: u64,
}

impl CoordinationRound {
    pub fn new(
        round_id: String,
        task: String,
        context: String,
        sent_to: Vec<String>,
        started_at_ms: u64,
        timeout_s: u64,
    ) -> Self {
        CoordinationRound {
            round_id,
            task,
            context,
            sent_to,
            responses: Vec::new(),
            started_at_ms,
            timeout_s,
        }
    }

    pub fn response_count(&self) -> u64 {
        self.responses.len() as u64
    }

    pub fn agents_sent(&self) -> u64 {
        self.sent_to.len() as u64
    }

    /// python break condition: len(responses) >= len(sent_to).
    pub fn is_complete(&self) -> bool {
        self.responses.len() >= self.sent_to.len()
    }

    /// Register one agent's response. Mirrors dict assignment
    /// `round_data["responses"][agent] = ...`: an existing agent's earlier
    /// response is overwritten, otherwise the pair is appended.
    pub fn register_response(&mut self, agent: &str, response: &str) {
        for slot in self.responses.iter_mut() {
            if slot.0 == agent {
                slot.1 = response.into();
                return;
            }
        }
        self.responses.push((agent.into(), response.into()));
    }

    /// Deterministic elapsed-seconds helper (python: round(now - started, 1)).
    /// // MANUAL: python round() is round-half-even; this target's core has no
    /// // f64::round, so 0.1s rounding is done in the integer domain
    /// // ((ms+50)/100 — half-up). Values differ from python only on exact
    /// // .05s boundaries, and the input is a wall clock anyway.
    pub fn elapsed_s(&self, now_ms: u64) -> f64 {
        let ms = now_ms.saturating_sub(self.started_at_ms);
        ((ms + 50) / 100) as f64 / 10.0
    }

    /// The python collector also folds completed ultrawork missions into the
    /// round as responses from their assigned agents.
    pub fn register_mission_result(&mut self, agent: &str, verification_result: &str) {
        self.register_response(agent, verification_result);
    }
}

// ---------------------------------------------------------------------------
// Consensus aggregation (1:1 consensus_aggregate — the pure heart).
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq)]
pub enum ConsensusValue {
    Single(String),
    List(Vec<String>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConsensusResult {
    /// None when there were no responses (any method) or the method is unknown.
    pub consensus: Option<ConsensusValue>,
    pub method: String,
    /// python "total" (majority/all/summary) / "count": 0 for the empty input.
    pub total: u64,
    /// majority only: value -> vote count, in first-occurrence order.
    pub votes: Vec<(String, u64)>,
    /// unknown-method error text (python: f"Unknown method: {method}").
    pub error: Option<String>,
}

/// Deterministic tally of response values: first-occurrence order with counts
/// (python: Counter over dict values, whose insertion order is first-arrival).
pub fn tally_response_values(responses: &[(String, String)]) -> Vec<(String, u64)> {
    let mut tally: Vec<(String, u64)> = Vec::new();
    for (_, value) in responses {
        match tally.iter_mut().find(|(v, _)| v == value) {
            Some((_, c)) => *c += 1,
            None => tally.push((value.clone(), 1)),
        }
    }
    tally
}

/// python: counts.most_common(1) — the value with the largest count; on ties
/// python's stable sort keeps the value that appeared FIRST.
pub fn majority_consensus(responses: &[(String, String)]) -> Option<(String, u64)> {
    let tally = tally_response_values(responses);
    let mut best: Option<(&String, u64)> = None;
    for (value, count) in &tally {
        match best {
            None => best = Some((value, *count)),
            Some((_, bc)) if *count > bc => best = Some((value, *count)),
            _ => {}
        }
    }
    best.map(|(v, c)| (v.clone(), c))
}

/// Faithful port of CoordinationProtocol.consensus_aggregate(responses, method).
/// responses are (agent, response) pairs in arrival order.
pub fn consensus_aggregate(responses: &[(String, String)], method: &str) -> ConsensusResult {
    // python early-return: empty input short-circuits before the method check.
    if responses.is_empty() {
        return ConsensusResult {
            consensus: None,
            method: method.into(),
            total: 0,
            votes: Vec::new(),
            error: None,
        };
    }

    match method {
        "majority" => {
            let (value, _count) = majority_consensus(responses)
                .expect("non-empty input always yields a majority");
            ConsensusResult {
                consensus: Some(ConsensusValue::Single(value.clone())),
                method: "majority".into(),
                total: responses.len() as u64,
                votes: tally_response_values(responses),
                error: None,
            }
        }
        "all" => {
            let values = responses.iter().map(|(_, v)| v.clone()).collect();
            ConsensusResult {
                consensus: Some(ConsensusValue::List(values)),
                method: "all".into(),
                total: responses.len() as u64,
                votes: Vec::new(),
                error: None,
            }
        }
        "summary" => {
            let mut lines = String::new();
            for (i, (agent, value)) in responses.iter().enumerate() {
                if i > 0 {
                    lines.push('\n');
                }
                lines.push_str(&format!("- {}: {}", agent, value));
            }
            ConsensusResult {
                consensus: Some(ConsensusValue::Single(format!("Agent responses:\n{}", lines))),
                method: "summary".into(),
                total: responses.len() as u64,
                votes: Vec::new(),
                error: None,
            }
        }
        other => ConsensusResult {
            consensus: None,
            method: other.into(),
            total: responses.len() as u64,
            votes: Vec::new(),
            error: Some(format!("Unknown method: {}", other)),
        },
    }
}

// ---------------------------------------------------------------------------
// Payload validation for a coordination task broadcast.
// ---------------------------------------------------------------------------
/// Deterministic structural validation of a received coordination task body:
/// type == "coordination_task", a round_id present, a non-empty task, and a
/// respond_by deadline that has not already passed (now_ms from the caller).
pub fn validate_coordination_task(
    msg_type: &str,
    round_id: Option<&str>,
    task: Option<&str>,
    respond_by_ms: Option<u64>,
    now_ms: u64,
) -> Result<(), String> {
    if msg_type != COORD_MSG_TYPE_TASK {
        return Err(format!("Not a coordination task message: type='{}'", msg_type));
    }
    match round_id {
        None => return Err("Missing round_id".into()),
        Some(rid) if rid.is_empty() => return Err("Empty round_id".into()),
        _ => {}
    }
    match task {
        None => return Err("Missing task".into()),
        Some(t) if t.is_empty() => return Err("Empty task".into()),
        _ => {}
    }
    if let Some(deadline) = respond_by_ms {
        if deadline < now_ms {
            return Err("Task expired (respond_by deadline passed)".into());
        }
    }
    Ok(())
}

fn fnv1a64(seed: u64) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let bytes = seed.to_le_bytes();
    for b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// Singleton handle (lazy-init pattern from the Python getter; round state is
/// caller-owned CoordinationRound values).
pub fn get_coordination_protocol() -> u64 {
    0x434f_4f52_4456_3031 // "COORDV01"
}
