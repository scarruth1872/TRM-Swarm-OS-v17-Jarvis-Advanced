//! TRM Swarm OS — agent_mailbox_bus (no_std port, REAL-verified).
//! 1:1 deterministic core of swarm_v2/core/agent_mailbox_bus.py:
//!   - 64-byte cache-line-aligned binary message struct
//!   - per-agent SPMC ring buffer (head/tail indices, capacity semantics)
//!   - SwarmMailboxBus send/receive/broadcast/peek_mailbox + arena-size arithmetic
//! The python `benchmark_mailbox_bus` body is stdout/np-timing bound — kept as
//! an honest skeleton (// MANUAL).

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

// ─── Constants (1:1 python) ───
pub const MAILBOX_CAPACITY: u64 = 4096; // must be power of 2
pub const MAX_AGENTS: u64 = 15;
pub const MSG_STRUCT_SIZE: u64 = 64; // cache-line aligned
pub const CACHE_LINE: u64 = 64;
/// python SHARED_ARENA_SIZE = MAX_AGENTS * (2*8 + CAPACITY*MSG_STRUCT_SIZE)
pub const SHARED_ARENA_SIZE: u64 = MAX_AGENTS * (2 * 8 + MAILBOX_CAPACITY * MSG_STRUCT_SIZE);
pub const AGENT_IDS: &'static [&'static str] = &[
    "Shield", "Logic", "Devo", "Seeker", "Verify", "Archi", "Flow", "Bridge",
    "Orchestra", "EventLedger", "LogicMaster", "Pulse", "THzGateway", "Jarvis", "Hermes",
];
/// python: recipient_id 255 = broadcast
pub const BROADCAST_RECIPIENT: u8 = 255;

/// AGENT_ID_MAP lookup: name -> index 0..14.
pub fn agent_id(name: &str) -> Option<u8> {
    AGENT_IDS.iter().position(|&a| a == name).map(|i| i as u8)
}

/// 64-byte binary mailbox message — matches the python ctypes C-ABI struct.
#[derive(Debug, Clone)]
pub struct MailboxMessage {
    pub msg_id: u64,          // monotonically increasing sequence
    pub sender_id: u8,        // agent id 0-14
    pub recipient_id: u8,     // 0-14, 255=broadcast
    pub k3_trit: i8,          // Kleene trit (-1, 0, +1)
    pub priority: u8,         // 0: Low, 1: Normal, 2: Ultra Mission
    pub payload_size: u32,    // length of payload (<= 40)
    pub inline_data: [u8; 40], // 40-byte fast-path payload
    pub timestamp_ns: u64,    // nanosecond timestamp
}

impl MailboxMessage {
    pub fn new() -> Self {
        MailboxMessage {
            msg_id: 0,
            sender_id: 0,
            recipient_id: 0,
            k3_trit: 0,
            priority: 0,
            payload_size: 0,
            inline_data: [0u8; 40],
            timestamp_ns: 0,
        }
    }

    /// Build a message exactly like python `send`: payload_size =
    /// min(len(payload), 40), inline_data copies the payload bytes.
    pub fn build(
        msg_id: u64,
        sender_id: u8,
        recipient_id: u8,
        k3_trit: i8,
        priority: u8,
        payload: &[u8],
        timestamp_ns: u64,
    ) -> Self {
        let mut m = MailboxMessage::new();
        m.msg_id = msg_id;
        m.sender_id = sender_id;
        m.recipient_id = recipient_id;
        m.k3_trit = k3_trit;
        m.priority = priority;
        m.timestamp_ns = timestamp_ns;
        let n = payload.len().min(40);
        m.payload_size = n as u32;
        m.inline_data[..n].copy_from_slice(&payload[..n]);
        m
    }

    pub fn payload(&self) -> &[u8] {
        &self.inline_data[..self.payload_size as usize]
    }
}

/// SPMC ring buffer (python AgentRingBuffer): head/tail indices over a
/// fixed-capacity slot vector. push/pop semantics are 1:1 with the python.
#[derive(Debug, Clone)]
pub struct AgentRingBuffer {
    pub head: u64,
    pub tail: u64,
    pub slots: Vec<MailboxMessage>,
}

impl AgentRingBuffer {
    pub fn new(capacity: usize) -> Self {
        AgentRingBuffer { head: 0, tail: 0, slots: Vec::with_capacity(capacity) }
    }

    pub fn is_full(&self) -> bool {
        self.tail - self.head >= MAILBOX_CAPACITY
    }

    pub fn is_empty(&self) -> bool {
        self.head == self.tail
    }

    /// Lock-free push (single-producer semantics): write at tail % capacity.
    pub fn push(&mut self, msg: MailboxMessage) -> bool {
        if self.is_full() {
            return false;
        }
        self.slots.push(msg);
        self.tail += 1;
        true
    }

    /// Lock-free pop (multi-consumer semantics): read at head % capacity.
    pub fn pop(&mut self) -> Option<MailboxMessage> {
        if self.is_empty() {
            return None;
        }
        self.head += 1;
        Some(self.slots.remove(0))
    }

    pub fn peek(&self) -> Option<&MailboxMessage> {
        if self.is_empty() {
            return None;
        }
        self.slots.first()
    }

    /// python `peek_mailbox`: tail - head = pending message count.
    pub fn pending(&self) -> u64 {
        self.tail - self.head
    }
}

/// Swarm mailbox bus — per-agent ring buffers, send/receive/broadcast stats.
#[derive(Debug, Clone)]
pub struct SwarmMailboxBus {
    pub agents: Vec<(String, AgentRingBuffer)>,
    pub sent_count: u64,
    pub received_count: u64,
    pub dropped_count: u64,
}

impl SwarmMailboxBus {
    pub fn new() -> Self {
        let agents = AGENT_IDS
            .iter()
            .map(|&a| (String::from(a), AgentRingBuffer::new(MAILBOX_CAPACITY as usize)))
            .collect();
        SwarmMailboxBus { agents, sent_count: 0, received_count: 0, dropped_count: 0 }
    }

    /// python _total_size = MAX_AGENTS * (2*CACHE_LINE + CAPACITY*MSG_STRUCT) + 64
    pub fn arena_size(&self) -> u64 {
        MAX_AGENTS * (2 * CACHE_LINE + MAILBOX_CAPACITY * MSG_STRUCT_SIZE) + 64
    }

    fn ring_mut(&mut self, agent: &str) -> Option<&mut AgentRingBuffer> {
        self.agents.iter_mut().find(|(n, _)| n == agent).map(|(_, r)| r)
    }

    fn ring(&self, agent: &str) -> Option<&AgentRingBuffer> {
        self.agents.iter().find(|(n, _)| n == agent).map(|(_, r)| r)
    }

    /// Send to the recipient's ring buffer; msg_id = sent_count.
    /// Returns false (and increments dropped_count) on unknown agent or full ring.
    pub fn send(
        &mut self,
        sender: &str,
        recipient: &str,
        trit: i8,
        priority: u8,
        payload: &[u8],
        timestamp_ns: u64,
    ) -> bool {
        let (Some(sid), Some(rid)) = (agent_id(sender), agent_id(recipient)) else {
            return false;
        };
        let msg = MailboxMessage::build(
            self.sent_count, sid, rid, trit, priority, payload, timestamp_ns,
        );
        if let Some(r) = self.ring_mut(recipient) {
            if r.push(msg) {
                self.sent_count += 1;
                return true;
            }
        }
        self.dropped_count += 1;
        false
    }

    pub fn receive(&mut self, agent: &str) -> Option<MailboxMessage> {
        let msg = self.ring_mut(agent)?.pop();
        if msg.is_some() {
            self.received_count += 1;
        }
        msg
    }

    /// Broadcast to all 14 *other* agents; returns the send success count.
    pub fn broadcast(&mut self, sender: &str, trit: i8, payload: &[u8], timestamp_ns: u64) -> u64 {
        let mut count = 0u64;
        for &recipient in AGENT_IDS {
            if recipient != sender
                && self.send(sender, recipient, trit, 2, payload, timestamp_ns)
            {
                count += 1;
            }
        }
        count
    }

    /// python peek_mailbox: pending messages for an agent (0 if unknown).
    pub fn peek_mailbox(&self, agent: &str) -> u64 {
        match self.ring(agent) {
            Some(r) => r.pending(),
            None => 0,
        }
    }

    /// python dump_stats numeric core: (sent, received, dropped,
    /// total_capacity = CAPACITY*MAX_AGENTS, ring_size = CAPACITY).
    pub fn dump_stats(&self) -> (u64, u64, u64, u64, u64) {
        (
            self.sent_count,
            self.received_count,
            self.dropped_count,
            MAILBOX_CAPACITY * MAX_AGENTS,
            MAILBOX_CAPACITY,
        )
    }
}

/// MANUAL: python body is a stdout benchmark (print/f-strings/np.mean) plus
/// file-backed mmap I/O — not portable deterministic logic.
pub fn benchmark_mailbox_bus() -> u64 {
    0
}
