//! TRM Swarm OS - moltbook (mechanical no_std port, TRM platform engine).
//! Constants and structure extracted 1:1 from swarm_v2/core/moltbook.py.
//! Functions marked // MANUAL need human verification of math semantics.

extern crate alloc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;


//  MoltbookQuery 
#[derive(Debug, Clone)]
pub struct MoltbookQuery {
    pub query_id: String,
    pub author: String,
    pub question: String,
    pub tags: Vec<String>,
    pub responses: Vec<Vec<(String, f64)>>,
    pub status: String,
    pub created_at: String,
    pub resolved_at: String,
}

impl MoltbookQuery {
    pub fn new() -> Self {
        MoltbookQuery {
            query_id: String::new(),
            author: String::new(),
            question: String::new(),
            tags: vec![],
            responses: vec![],
            // The oracle's dataclass default is `status: str = "open"`; the original stub left it
            // empty. "open" is the literal default a `post_query` constructs with, so it is kept.
            status: String::from("open"),
            // `resolved_at: Optional[str] = None` -> empty String; the caller owns the timestamp.
            created_at: String::new(),
            resolved_at: String::new(),
        }
    }
}

//  get_moltbook_network() 
static SINGLETON_MOLTBOOK_NETWORK: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
pub fn get_moltbook_network() -> u64 {
    // Singleton handle (lazy-init pattern from moltbook.py).
    // Returns 1 once configured; 0 until first call initializes.
    let v = SINGLETON_MOLTBOOK_NETWORK.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_MOLTBOOK_NETWORK.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}

// ── The oracle's model (swarm_v2/core/moltbook.py) ───────────────────────────────────────────────
//
// `find_matching_queries` looks for open queries an agent can answer:
//     specialties = the agent's own specialties, lowercased, as a SET
//     no specialties -> every open query, truncated to the limit
//     otherwise     -> queries that are open, were NOT authored by this agent, and whose lowercased
//                      tags INTERSECT the specialties
// and the result is `matched[:limit]` -- the first matches in insertion order, with no ranking. The
// caller decides order; this function does not sort, and neither does the port.

/// One query as these rules need it.
#[derive(Debug, Clone)]
pub struct QueryRef<'a> {
    pub status: &'a str,
    pub author: &'a str,
    pub tags: &'a [&'a str],
}

/// Indices of the queries this agent should see, in the order given.
pub fn find_matching_queries(specialties: &[&str], agent_id: &str, queries: &[QueryRef], limit: usize) -> Vec<usize> {
    let mut specs: Vec<String> = specialties.iter().map(|s| s.to_lowercase()).collect();
    specs.sort();
    specs.dedup();
    let mut out: Vec<usize> = Vec::new();
    for (i, q) in queries.iter().enumerate() {
        if q.status != "open" {
            continue;
        }
        if specs.is_empty() {
            out.push(i);
        } else {
            if q.author == agent_id {
                continue;
            }
            let hit = q.tags.iter().any(|t| {
                let lt = t.to_lowercase();
                specs.iter().any(|s| *s == lt)
            });
            if hit {
                out.push(i);
            }
        }
        if out.len() >= limit {
            break;
        }
    }
    out
}

// ── The oracle's remaining methods (swarm_v2/core/moltbook.py) ─────────────────────────────────
//
// `MoltbookNetwork` also exposes register_agent, post_query, respond_to_query, resolve_query,
// get_feed, install_solution and get_stats. They are ported below as PURE functions over a state
// snapshot: a no_std port holds none of the oracle's three dicts (`_queries`, `_agent_profiles`,
// `_stats`), so each rule takes its inputs explicitly and returns the derived value; the caller
// owns the mutation. Host-side pieces are excluded and called out inline:
//   * `datetime.now().isoformat()` -> `created_at` / `resolved_at` / `registered_at`. The caller
//     owns the wall clock; the port only ORDERS by (feed) or STORES (query) the opaque strings it
//     is handed — lexicographic order on an ISO-8601 string equals chronological order.
//   * `uuid.uuid4().hex[:8]`      -> the `post_query` id suffix. Host-side randomness; the port
//     only reproduces the `"molt-" + suffix` format via `make_query_id`.
//   * `logger.*`                  -> logging (host-side; no effect on any returned value).
//   * `_persist_to_memory`        -> imports `swarm_v2.core.global_memory` (a host model call).
//     Excluded: `resolve_query` here stops at the state change and the caller owns persistence.

/// One response record. `respond_to_query` appends one; `resolve_query` and `install_solution`
/// read them. The oracle's `timestamp` field is host-side and is not carried.
#[derive(Debug, Clone)]
pub struct ResponseData<'a> {
    pub responder: &'a str,
    pub solution: &'a str,
    pub confidence: f64,
}

/// One query as the feed/stats/install/resolve rules read it.
#[derive(Debug, Clone)]
pub struct QueryData<'a> {
    pub query_id: &'a str,
    pub author: &'a str,
    pub question: &'a str,
    pub tags: &'a [&'a str],
    pub status: &'a str,
    pub created_at: &'a str,
    pub responses: &'a [ResponseData<'a>],
}

/// One registered agent profile, as the stats and respond/resolve rules read it.
#[derive(Debug, Clone)]
pub struct ProfileData<'a> {
    pub agent_id: &'a str,
    pub response_count: usize,
    pub reputation: f64,
}

/// `respond_to_query`: the accept gate plus the two counter bumps. The oracle returns False and
/// changes nothing when the query is missing or not `"open"`; otherwise the response is accepted,
/// `total_responses` grows by one, and — when the responder is a registered profile — that
/// profile's `response_count` grows by one.
#[derive(Debug, Clone)]
pub struct RespondOutcome {
    pub accepted: bool,
    pub total_responses: usize,
    /// `Some(new)` iff the responder is a registered profile; `None` otherwise.
    pub response_count: Option<usize>,
}

pub fn respond_to_query(
    query_status: Option<&str>,             // None == query not found
    response_count_before: Option<usize>,   // None == responder not registered
    total_responses: usize,                 // _stats["total_responses"] before the call
) -> RespondOutcome {
    if query_status != Some("open") {
        return RespondOutcome {
            accepted: false,
            total_responses,
            response_count: response_count_before,
        };
    }
    RespondOutcome {
        accepted: true,
        total_responses: total_responses + 1,
        response_count: response_count_before.map(|c| c + 1),
    }
}

/// `resolve_query`: the accepted-response index (clamped to the last response) and the resolver's
/// reputation boost. Returns `None` when there is no query or it has no responses — the oracle's
/// single early-out for both. The caller sets `status = "resolved"` and `resolved_at` (host-side)
/// and runs persistence; the port computes the accepted index, the boosted reputation
/// (`min(1.0, reputation + 0.05)`), and the incremented `resolved` count.
///
/// Divergence, bounded: the oracle accepts any `accepted_response_idx`, so a NEGATIVE value
/// reaches Python's negative indexing; the documented contract is a non-negative index and the
/// port clamps in `usize`, so a negative index is out of contract on both sides.
#[derive(Debug, Clone)]
pub struct ResolveOutcome {
    pub accepted_index: usize,
    /// `Some(min(1.0, rep + 0.05))` iff the resolver is registered; `None` otherwise.
    pub reputation: Option<f64>,
    pub resolved: usize,   // _stats["resolved"] after the call
}

pub fn resolve_query(
    responses_len: usize,            // 0 == no query, or a query with no responses -> None
    accepted_response_idx: usize,
    reputation_before: Option<f64>,  // None == resolver not registered
    resolved: usize,                 // _stats["resolved"] before the call
) -> Option<ResolveOutcome> {
    if responses_len == 0 {
        return None;
    }
    let accepted_index = if accepted_response_idx < responses_len - 1 {
        accepted_response_idx
    } else {
        responses_len - 1
    };
    Some(ResolveOutcome {
        accepted_index,
        reputation: reputation_before.map(|r| {
            let boosted = r + 0.05;
            if boosted > 1.0 {
                1.0
            } else {
                boosted
            }
        }),
        resolved: resolved + 1,
    })
}

/// One entry of `get_feed`'s list.
#[derive(Debug, Clone)]
pub struct FeedEntry {
    pub query_id: String,
    pub author: String,
    pub question: String,   // question[:100]
    pub tags: Vec<String>,
    pub status: String,
    pub response_count: usize,
    pub created_at: String,
}

/// Python `s[:n]` — the first `n` characters (Unicode code points), not bytes.
fn truncate_chars(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

/// `get_feed`: newest first by `created_at` (stable, so equal timestamps keep insertion order),
/// truncated to `limit`, then projected to the feed shape with `question[:100]` and
/// `response_count = len(responses)`.
pub fn get_feed(queries: &[QueryData], limit: usize) -> Vec<FeedEntry> {
    let mut idx: Vec<usize> = (0..queries.len()).collect();
    // `sort_by` is a STABLE sort, matching Python's `sorted(..., key=..., reverse=True)`.
    idx.sort_by(|&a, &b| queries[b].created_at.cmp(queries[a].created_at));
    let mut out: Vec<FeedEntry> = Vec::new();
    for i in idx.into_iter().take(limit) {
        let q = &queries[i];
        out.push(FeedEntry {
            query_id: q.query_id.into(),
            author: q.author.into(),
            question: truncate_chars(q.question, 100),
            tags: q.tags.iter().map(|t| (*t).into()).collect(),
            status: q.status.into(),
            response_count: q.responses.len(),
            created_at: q.created_at.into(),
        });
    }
    out
}

/// `install_solution`: `None` unless the query is resolved; otherwise the solution of the response
/// with the highest confidence — the FIRST such response on a tie (Python's `max` keeps the first).
/// `target_agent` is only ever logged by the oracle and is therefore not a parameter here.
///
/// Divergence, bounded: the oracle's `r.get("confidence", 0)` default is unreachable because
/// `respond_to_query` always sets `confidence`; and an empty response list on a resolved query
/// would make Python's `max` raise `ValueError`, whereas the port returns `None` — that state is
/// unreachable by construction too (resolve_query guarantees >= 1 response before resolving).
pub fn install_solution(query_status: Option<&str>, responses: &[ResponseData]) -> Option<String> {
    if query_status != Some("resolved") {
        return None;
    }
    let mut best: Option<&ResponseData> = None;
    for r in responses {
        match best {
            None => best = Some(r),
            Some(b) => {
                if r.confidence > b.confidence {
                    best = Some(r);
                }
            }
        }
    }
    best.map(|r| r.solution.into())
}

/// `get_stats`: the network tallies plus the top contributors.
#[derive(Debug, Clone)]
pub struct NetworkStats {
    pub registered_agents: usize,
    pub open_queries: usize,
    pub total_queries: usize,
    pub total_responses: usize,
    pub resolved: usize,
    /// `(agent_id, response_count, reputation)`, top 5 by response_count (desc, stable).
    pub top_contributors: Vec<(String, usize, f64)>,
}

pub fn get_stats(
    profiles: &[ProfileData],
    queries: &[QueryData],
    total_queries: usize,
    total_responses: usize,
    resolved: usize,
) -> NetworkStats {
    let open_queries = queries.iter().filter(|q| q.status == "open").count();
    let mut contrib: Vec<usize> = (0..profiles.len()).collect();
    // Sort by response_count descending, STABLE (the oracle's `sorted(..., key=lambda x: x[1],
    // reverse=True)` keeps insertion order on ties), then keep the first five.
    contrib.sort_by(|&a, &b| profiles[b].response_count.cmp(&profiles[a].response_count));
    let top: Vec<(String, usize, f64)> = contrib
        .into_iter()
        .take(5)
        .map(|i| {
            let p = &profiles[i];
            (p.agent_id.into(), p.response_count, p.reputation)
        })
        .collect();
    NetworkStats {
        registered_agents: profiles.len(),
        open_queries,
        total_queries,
        total_responses,
        resolved,
        top_contributors: top,
    }
}

/// One registered agent profile as `register_agent` constructs it. `registered_at` is host-side.
#[derive(Debug, Clone)]
pub struct AgentProfile {
    pub specialties: Vec<String>,
    pub capabilities: Vec<String>,
    pub response_count: usize,   // 0
    pub reputation: f64,         // 0.5
}

/// `register_agent`: `capabilities or []` (so `None` and an empty list both become empty),
/// `response_count = 0`, `reputation = 0.5`. `agent_id` is only the dict key and `registered_at`
/// is host-side, so neither is a parameter.
pub fn register_agent(
    specialties: Vec<String>,
    capabilities: Option<Vec<String>>,
) -> AgentProfile {
    AgentProfile {
        specialties,
        capabilities: capabilities.unwrap_or_default(),
        response_count: 0,
        reputation: 0.5,
    }
}

/// `post_query`: the `"molt-" + uuid4().hex[:8]` id format. The random suffix is host-side, so the
/// port takes it as an argument and reproduces only the format; the query itself is a
/// `MoltbookQuery::new()` (status "open", empty responses) with `tags or []` and the caller's
/// `author`/`question`, and `total_queries` increments by one on the caller's side.
pub fn make_query_id(suffix_hex: &str) -> String {
    let mut s = String::with_capacity(5 + suffix_hex.len());
    s.push_str("molt-");
    s.push_str(suffix_hex);
    s
}
