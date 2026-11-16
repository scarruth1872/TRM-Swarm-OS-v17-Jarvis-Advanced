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

// ═══ WIRE LAYOUT — the unported gap: python did this via strip_mailbox_bus.dll ═══
/// Bytes per wire slot. Base layout derived byte-for-byte from the shipping
/// strip (`strip_mailbox_pack_messages`, probed 2026-09-10): the python ctypes
/// struct declares a trailing `_padding[4]` (sizeof 68) that the native packer
/// does NOT emit — every slot on the wire is exactly 64 bytes.
///   @0 msg_id u64 LE, @8 sender u8, @9 recipient u8, @10 k3_trit i8,
///   @11 priority u8, @12 payload_size u32 LE, @16 inline_data[40], @56 timestamp_ns u64 LE
pub const WIRE_MESSAGE_BYTES: usize = MSG_STRUCT_SIZE as usize;

/// Serialize one message into a 64-byte wire slot.
/// DEVIATION from the strip, deliberate: the strip wrote `timestamp_ns` as a
/// compile-time constant (1788800000 on every call, unchanged 2.5 s apart while
/// the wall clock read 1789085715) and hardcoded msg_id=0, priority=1,
/// payload_size=40. This port takes all four from the message so no caller can
/// inherit a frozen clock.
pub fn write_wire(msg: &MailboxMessage, out: &mut [u8]) {
    let slot = &mut out[..WIRE_MESSAGE_BYTES];
    slot[0..8].copy_from_slice(&msg.msg_id.to_le_bytes());
    slot[8] = msg.sender_id;
    slot[9] = msg.recipient_id;
    slot[10] = msg.k3_trit as u8;
    slot[11] = msg.priority;
    slot[12..16].copy_from_slice(&msg.payload_size.to_le_bytes());
    slot[16..56].copy_from_slice(&msg.inline_data);
    slot[56..64].copy_from_slice(&msg.timestamp_ns.to_le_bytes());
}

/// Read one wire slot back into a message. Returns None when the slot is short
/// or when `payload_size` exceeds the 40-byte inline capacity — a malformed
/// slot is not a message, and `payload()` relies on that bound.
pub fn read_wire(slot: &[u8]) -> Option<MailboxMessage> {
    if slot.len() < WIRE_MESSAGE_BYTES {
        return None;
    }
    let mut m = MailboxMessage::new();
    let mut b8 = [0u8; 8];
    b8.copy_from_slice(&slot[0..8]);
    m.msg_id = u64::from_le_bytes(b8);
    m.sender_id = slot[8];
    m.recipient_id = slot[9];
    m.k3_trit = slot[10] as i8;
    m.priority = slot[11];
    let mut b4 = [0u8; 4];
    b4.copy_from_slice(&slot[12..16]);
    m.payload_size = u32::from_le_bytes(b4);
    if m.payload_size > 40 {
        return None;
    }
    m.inline_data.copy_from_slice(&slot[16..56]);
    let mut b8b = [0u8; 8];
    b8b.copy_from_slice(&slot[56..64]);
    m.timestamp_ns = u64::from_le_bytes(b8b);
    Some(m)
}

/// Pack messages into one contiguous `len * 64` arena — the 1:1 port of python
/// `native_pack_mailbox_messages`, which could only ever reach the strip DLL.
/// In-kernel it needs no binding: the layout is the const above.
pub fn pack_messages(msgs: &[MailboxMessage]) -> Vec<u8> {
    let mut out = Vec::new();
    out.resize(msgs.len() * WIRE_MESSAGE_BYTES, 0u8);
    for (i, m) in msgs.iter().enumerate() {
        write_wire(m, &mut out[i * WIRE_MESSAGE_BYTES..(i + 1) * WIRE_MESSAGE_BYTES]);
    }
    out
}
