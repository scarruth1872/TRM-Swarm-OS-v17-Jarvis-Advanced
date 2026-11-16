//! TRM Swarm OS — agent_mailbox (no_std port).
//! Deterministic core from swarm_v2/core/agent_mailbox.py — Phase 2 mailbox:
//! file-based async agent-to-agent messaging with TTL expiry.
//! The host-side file I/O (inbox.json) is replaced by an in-memory inbox map;
//! TTL expiry, consume-on-receive, peek, broadcast exclusion and pending
//! counting are ported 1:1.

extern crate alloc;
use alloc::collections::vec_deque::VecDeque;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

/// Root of the mailbox store (path semantics preserved as constant).
pub const MAILBOX_ROOT: &str = ".swarm/mailboxes";
/// Default message TTL in minutes (1:1 python module constant).
pub const DEFAULT_TTL_MINUTES: u64 = 60;
/// Priority levels accepted by send(): normal, high, urgent.
pub const PRIORITY_NORMAL: u8 = 0;
pub const PRIORITY_HIGH: u8 = 1;
pub const PRIORITY_URGENT: u8 = 2;

/// One mailbox message (deterministic subset of the python dict payload).
#[derive(Debug, Clone)]
pub struct MailMessage {
    pub from_agent: String,
    pub to_agent: String,
    pub subject: String,
    pub message: String,
    /// 0 = normal, 1 = high, 2 = urgent.
    pub priority: u8,
    /// Unix seconds at creation (caller-provided monotonic clock).
    pub timestamp: u64,
    /// Unix seconds when the message expires (timestamp + TTL*60).
    pub expires_at: u64,
    pub read: bool,
}

impl MailMessage {
    pub fn new(
        from_agent: &str,
        to_agent: &str,
        subject: &str,
        message: &str,
        priority: u8,
        now: u64,
        ttl_minutes: u64,
    ) -> Self {
        MailMessage {
            from_agent: from_agent.to_string(),
            to_agent: to_agent.to_string(),
            subject: subject.to_string(),
            message: message.to_string(),
            priority: priority.min(PRIORITY_URGENT),
            timestamp: now,
            expires_at: now.saturating_add(ttl_minutes.saturating_mul(60)),
            read: false,
        }
    }

    /// `_is_expired(msg, now)`: expires_at exists and exp_dt < now.
    #[inline]
    pub fn is_expired(&self, now: u64) -> bool {
        self.expires_at < now
    }
}

/// In-memory AgentMailbox: per-agent inbox FIFO queues with TTL expiry.
#[derive(Debug, Clone)]
pub struct AgentMailbox {
    pub agent_id: String,
    /// Inboxes of every agent that has ever been sent to (== mailbox dirs).
    pub inboxes: BTreeMap<String, VecDeque<MailMessage>>,
}

impl AgentMailbox {
    pub fn new(agent_id: &str) -> Self {
        AgentMailbox {
            agent_id: agent_id.to_string(),
            inboxes: BTreeMap::new(),
        }
    }

    fn inbox_mut(&mut self, target: &str) -> &mut VecDeque<MailMessage> {
        self.inboxes
            .entry(target.to_string())
            .or_insert_with(VecDeque::new)
    }

    /// send(): append a TTL'd message to the target agent's inbox. Returns true.
    pub fn send(
        &mut self,
        target_agent: &str,
        message: &str,
        subject: &str,
        priority: u8,
        now: u64,
    ) -> bool {
        let msg = MailMessage::new(
            &self.agent_id,
            target_agent,
            subject,
            message,
            priority,
            now,
            DEFAULT_TTL_MINUTES,
        );
        self.inbox_mut(target_agent).push_back(msg);
        true
    }

    /// broadcast(): message every registered mailbox except self and exclusions.
    pub fn broadcast(&mut self, message: &str, subject: &str, exclude: &[&str], now: u64) -> u64 {
        let mut count = 0u64;
        let agent_ids: Vec<String> = self.inboxes.keys().cloned().collect();
        for agent_dir in agent_ids {
            if agent_dir == self.agent_id {
                continue;
            }
            if exclude.iter().any(|&e| e == agent_dir) {
                continue;
            }
            self.send(&agent_dir, message, subject, PRIORITY_NORMAL, now);
            count += 1;
        }
        count
    }

    /// receive(): consume up to `limit` unexpired messages (marked read).
    /// Expired messages are dropped; the remainder stays queued.
    pub fn receive(&mut self, limit: usize, now: u64) -> Vec<MailMessage> {
        let mut consumed = Vec::new();
        if let Some(q) = self.inboxes.get_mut(&self.agent_id) {
            // Filter expired, then take first `limit` valid messages.
            let mut valid: Vec<MailMessage> = Vec::new();
            while let Some(m) = q.pop_front() {
                if !m.is_expired(now) {
                    valid.push(m);
                }
            }
            let take = limit.min(valid.len());
            for m in valid.drain(..take) {
                let mut mm = m;
                mm.read = true;
                consumed.push(mm);
            }
            for m in valid {
                q.push_back(m);
            }
        }
        consumed
    }

    /// peek(): read unexpired messages without consuming them.
    pub fn peek(&self, limit: usize, now: u64) -> Vec<MailMessage> {
        let mut out = Vec::new();
        if let Some(q) = self.inboxes.get(&self.agent_id) {
            for m in q.iter() {
                if m.is_expired(now) {
                    continue;
                }
                out.push(m.clone());
                if out.len() >= limit {
                    break;
                }
            }
        }
        out
    }

    /// count_pending(): number of unexpired messages in own inbox.
    pub fn count_pending(&self, now: u64) -> u64 {
        self.pending_len(now)
    }

    /// clear(): drop every message in own inbox.
    pub fn clear(&mut self) {
        if let Some(q) = self.inboxes.get_mut(&self.agent_id) {
            q.clear();
        }
    }

    /// Raw unexpired length of the own inbox (get_stats pending_messages).
    pub fn pending_len(&self, now: u64) -> u64 {
        self.inboxes
            .get(&self.agent_id)
            .map(|q| q.iter().filter(|m| !m.is_expired(now)).count() as u64)
            .unwrap_or(0)
    }

    /// list_agents(): all agent ids that own a mailbox.
    pub fn list_agents(&self) -> Vec<String> {
        self.inboxes.keys().cloned().collect()
    }

    /// Total raw messages queued for `agent` (used by harness to assert order).
    pub fn raw_len(&self, agent: &str) -> usize {
        self.inboxes.get(agent).map(|q| q.len()).unwrap_or(0)
    }
}

/// Mailbox handle: python instantiates AgentMailbox(agent_id) per agent;
/// in the kernel each agent owns its AgentMailbox value (no global needed).
pub const MAILBOX_HANDLE: u64 = 0x4D41_494C_424F_58; // "MAILBOX"
