//! TRM Swarm OS - agent_mailbox (Deterministic bare-metal no_std + alloc implementation).
//! Fully mirrors swarm_v2/core/agent_mailbox.py & strip_agent_mailbox.dll interfaces.
//! Implements async agent-to-agent message passing, TTL-based expiry pruning,
//! priority queues, MAT trust tokens, and Monad truth anchor sealing.

extern crate alloc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::format;
use core::sync::atomic::{AtomicU64, Ordering};

// --- CONSTANTS ---
pub const MAILBOX_ROOT: &'static str = ".swarm/mailboxes";
pub const DEFAULT_TTL_MINUTES: u64 = 60;
pub const MAX_INBOX_CAPACITY: usize = 256;
pub const MONAD_ANCHOR: u64 = 0x4D4F_4E41_4400_0001;

// --- MESSAGE PRIORITY ---
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MessagePriority {
    Normal = 0,
    High = 1,
    Urgent = 2,
}

impl MessagePriority {
    pub fn as_str(&self) -> &'static str {
        match self {
            MessagePriority::Normal => "normal",
            MessagePriority::High => "high",
            MessagePriority::Urgent => "urgent",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "urgent" => MessagePriority::Urgent,
            "high" => MessagePriority::High,
            _ => MessagePriority::Normal,
        }
    }
}

// --- MAILBOX MESSAGE ---
#[derive(Debug, Clone)]
pub struct MailboxMessage {
    pub from_agent: String,
    pub to_agent: String,
    pub subject: String,
    pub body: String,
    pub priority: MessagePriority,
    pub timestamp_epoch_s: u64,
    pub expires_at_epoch_s: u64,
    pub read: bool,
    pub trust_token: u64,
    pub monad_seal: u64,
}

impl MailboxMessage {
    pub fn is_expired(&self, current_epoch_s: u64) -> bool {
        current_epoch_s >= self.expires_at_epoch_s
    }

    pub fn compute_trust_token(from: &str, to: &str, ts: u64) -> u64 {
        let mut h: u64 = 0xcbf29ce484222325;
        for &b in from.as_bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
        for &b in to.as_bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
        h ^ ts ^ MONAD_ANCHOR
    }
}

// --- AGENT MAILBOX ---
#[derive(Debug, Clone)]
pub struct AgentMailbox {
    pub agent_id: String,
    inbox: Vec<MailboxMessage>,
    outbox_log: Vec<MailboxMessage>,
}

impl AgentMailbox {
    pub fn new(agent_id: &str) -> Self {
        Self {
            agent_id: agent_id.to_string(),
            inbox: Vec::new(),
            outbox_log: Vec::new(),
        }
    }

    pub fn enqueue_incoming(&mut self, msg: MailboxMessage) -> bool {
        if self.inbox.len() >= MAX_INBOX_CAPACITY {
            // Prune oldest normal messages if full
            if let Some(pos) = self.inbox.iter().position(|m| m.priority == MessagePriority::Normal) {
                self.inbox.remove(pos);
            } else {
                return false;
            }
        }
        self.inbox.push(msg);
        // Sort descending by priority so high/urgent come first
        self.inbox.sort_by(|a, b| b.priority.cmp(&a.priority));
        true
    }

    pub fn create_outgoing(
        &mut self,
        target_agent: &str,
        subject: &str,
        body: &str,
        priority: MessagePriority,
        current_epoch_s: u64,
        ttl_minutes: u64,
    ) -> MailboxMessage {
        let ttl_s = if ttl_minutes > 0 { ttl_minutes * 60 } else { DEFAULT_TTL_MINUTES * 60 };
        let trust = MailboxMessage::compute_trust_token(&self.agent_id, target_agent, current_epoch_s);
        let seal = trust ^ 0x54524D_5345414C;

        let msg = MailboxMessage {
            from_agent: self.agent_id.clone(),
            to_agent: target_agent.to_string(),
            subject: subject.to_string(),
            body: body.to_string(),
            priority,
            timestamp_epoch_s: current_epoch_s,
            expires_at_epoch_s: current_epoch_s + ttl_s,
            read: false,
            trust_token: trust,
            monad_seal: seal,
        };

        if self.outbox_log.len() >= 128 {
            self.outbox_log.remove(0);
        }
        self.outbox_log.push(msg.clone());
        msg
    }

    pub fn receive(&mut self, limit: usize, current_epoch_s: u64) -> Vec<MailboxMessage> {
        self.prune_expired(current_epoch_s);
        let take_count = limit.min(self.inbox.len());
        let mut consumed = Vec::with_capacity(take_count);

        for _ in 0..take_count {
            if !self.inbox.is_empty() {
                let mut msg = self.inbox.remove(0);
                msg.read = true;
                consumed.push(msg);
            }
        }
        consumed
    }

    pub fn prune_expired(&mut self, current_epoch_s: u64) -> usize {
        let initial_len = self.inbox.len();
        self.inbox.retain(|m| !m.is_expired(current_epoch_s));
        initial_len - self.inbox.len()
    }

    pub fn unread_count(&self, current_epoch_s: u64) -> usize {
        self.inbox.iter().filter(|m| !m.read && !m.is_expired(current_epoch_s)).count()
    }

    pub fn peek_urgent(&self, current_epoch_s: u64) -> Option<&MailboxMessage> {
        self.inbox.iter().find(|m| m.priority == MessagePriority::Urgent && !m.is_expired(current_epoch_s))
    }
}

// --- MAILBOX ROUTER ---
pub struct MailboxRouter {
    mailboxes: Vec<AgentMailbox>,
}

impl MailboxRouter {
    pub fn new() -> Self {
        Self {
            mailboxes: Vec::new(),
        }
    }

    pub fn ensure_mailbox(&mut self, agent_id: &str) -> &mut AgentMailbox {
        if !self.mailboxes.iter().any(|m| m.agent_id == agent_id) {
            self.mailboxes.push(AgentMailbox::new(agent_id));
        }
        self.mailboxes.iter_mut().find(|m| m.agent_id == agent_id).unwrap()
    }

    pub fn send(
        &mut self,
        from_agent: &str,
        to_agent: &str,
        subject: &str,
        body: &str,
        priority: MessagePriority,
        current_epoch_s: u64,
        ttl_minutes: u64,
    ) -> bool {
        let msg = {
            let sender = self.ensure_mailbox(from_agent);
            sender.create_outgoing(to_agent, subject, body, priority, current_epoch_s, ttl_minutes)
        };

        let recipient = self.ensure_mailbox(to_agent);
        recipient.enqueue_incoming(msg)
    }

    pub fn broadcast(
        &mut self,
        from_agent: &str,
        subject: &str,
        body: &str,
        priority: MessagePriority,
        current_epoch_s: u64,
    ) -> usize {
        let all_recipients: Vec<String> = self.mailboxes.iter()
            .map(|m| m.agent_id.clone())
            .filter(|id| id != from_agent)
            .collect();

        let mut sent = 0;
        for to_agent in all_recipients {
            if self.send(from_agent, &to_agent, subject, body, priority, current_epoch_s, DEFAULT_TTL_MINUTES) {
                sent += 1;
            }
        }
        sent
    }

    pub fn get_unread_count(&self, agent_id: &str, current_epoch_s: u64) -> usize {
        if let Some(m) = self.mailboxes.iter().find(|m| m.agent_id == agent_id) {
            m.unread_count(current_epoch_s)
        } else {
            0
        }
    }
}

// --- SINGLETON INTERFACE ---
static SINGLETON_AGENT_MAILBOX: AtomicU64 = AtomicU64::new(0);

pub fn get_agent_mailbox() -> u64 {
    let v = SINGLETON_AGENT_MAILBOX.load(Ordering::Relaxed);
    if v == 0 {
        SINGLETON_AGENT_MAILBOX.store(1, Ordering::Relaxed);
        1
    } else {
        v
    }
}

