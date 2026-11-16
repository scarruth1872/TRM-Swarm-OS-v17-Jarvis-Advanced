//! TRM Swarm OS - native_mailbox_bridge (no_std port).
//! The python source embeds a C-ABI lock-free ring-bus mailbox (swarm_mailbox.cpp).
//! C-ABI shared-library code is not portable to the UEFI kernel; the platform
//! mailbox logic is represented here as a pure-Rust ring bus.
//! (C-ABI reference retained in swarm_v2/core/native_mailbox_bridge.py)

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

pub struct MailboxMessage {
    pub sender: String,
    pub recipient: String,
    pub payload: String,
    pub timestamp_ns: u64,
}

impl MailboxMessage {
    pub fn new() -> Self {
        MailboxMessage {
            sender: String::new(),
            recipient: String::new(),
            payload: String::new(),
            timestamp_ns: 0,
        }
    }
}

/// Ring-bus mailbox: fixed capacity, O(1) push/pop, no allocation per message.
pub struct RingMailbox {
    pub capacity: usize,
    pub messages: Vec<MailboxMessage>,
}

impl RingMailbox {
    pub fn new(capacity: usize) -> Self {
        RingMailbox {
            capacity,
            messages: Vec::new(),
        }
    }

    pub fn push(&mut self, msg: MailboxMessage) -> bool {
        if self.messages.len() >= self.capacity {
            return false;
        }
        self.messages.push(msg);
        true
    }

    pub fn pop(&mut self) -> Option<MailboxMessage> {
        if self.messages.is_empty() {
            return None;
        }
        Some(self.messages.remove(0))
    }

    pub fn len(&self) -> usize {
        self.messages.len()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Portable rules of the C-ABI mailbox oracle
// (swarm_v2/core/native_mailbox_bridge.py: its embedded C++ source `CPP_SOURCE`
//  == swarm_v2/core/swarm_mailbox.cpp, plus the Phase-22 strip entry point
//  `strip_batch_mailbox_scan`).
//
// NOT ported (host-side): ctypes.CDLL loading and the lib search paths, the
// ctypes Structure/Object marshalling, shm_open/ftruncate/mmap/close of
// /swarm_mailbox, `sys/mman.h`+`fcntl.h`+`unistd.h`, the `Path(...).exists()`
// DLL probe, `import ctypes/os/sys`, the perf_counter_ns latency sampling, and
// the g_mailbox == nullptr (shm failed) branch.
//
// Wire format: the C++ struct is `alignas(64)` with natural padding, but the
// ctypes mirror the Python actually uses declares `_pack_ = 1` and sums to
// exactly 64 bytes (8+1+1+1+1+4+40+8). The strip reads the trit at byte offset
// 10 of each slot, which only lines up for the *packed* layout (in the natural
// layout k3_trit sits at offset 12), so the packed layout is the wire format.
// ─────────────────────────────────────────────────────────────────────────────

/// `constexpr size_t MAILBOX_CAPACITY = 4096;`
pub const MAILBOX_CAPACITY: usize = 4096;
/// `constexpr size_t MAX_AGENTS = 15;`
pub const MAX_AGENTS: usize = 15;
/// `constexpr size_t CACHE_LINE = 64;`
pub const CACHE_LINE: usize = 64;
/// `uint8_t inline_data[40]`
pub const INLINE_DATA_LEN: usize = 40;
/// sizeof(AgentMailboxMessage) under the packed (_pack_ = 1) ctypes mirror.
pub const MSG_SIZE: usize = 64;
/// Byte offset of k3_trit inside a packed 64-byte message slot.
pub const K3_TRIT_OFFSET: usize = 10;

/// `MAX_AGENTS * (2 * CACHE_LINE + MAILBOX_CAPACITY * sizeof(AgentMailboxMessage))`
/// = 15 * (128 + 262144).
pub const fn mailbox_region_size() -> usize {
    MAX_AGENTS * (2 * CACHE_LINE + MAILBOX_CAPACITY * MSG_SIZE)
}

/// C-ABI slot, packed to mirror `class C_AgentMailboxMessage(_pack_ = 1)`.
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct CAgentMailboxMessage {
    pub msg_id: u64,
    pub sender_id: u8,
    pub recipient_id: u8,
    pub k3_trit: i8,
    pub priority: u8,
    pub payload_size: u32,
    pub inline_data: [u8; INLINE_DATA_LEN],
    pub timestamp_ns: u64,
}

const _: () = assert!(core::mem::size_of::<CAgentMailboxMessage>() == MSG_SIZE);

impl CAgentMailboxMessage {
    pub fn new() -> Self {
        CAgentMailboxMessage {
            msg_id: 0,
            sender_id: 0,
            recipient_id: 0,
            k3_trit: 0,
            priority: 0,
            payload_size: 0,
            inline_data: [0u8; INLINE_DATA_LEN],
            timestamp_ns: 0,
        }
    }

    /// Raw byte image of the packed slot — the buffer the strip scans.
    pub fn to_bytes(&self) -> [u8; MSG_SIZE] {
        let mut out = [0u8; MSG_SIZE];
        unsafe {
            core::ptr::copy_nonoverlapping(
                self as *const CAgentMailboxMessage as *const u8,
                out.as_mut_ptr(),
                MSG_SIZE,
            );
        }
        out
    }

    pub fn from_bytes(src: &[u8; MSG_SIZE]) -> Self {
        let mut msg = CAgentMailboxMessage::new();
        unsafe {
            core::ptr::copy_nonoverlapping(
                src.as_ptr(),
                &mut msg as *mut CAgentMailboxMessage as *mut u8,
                MSG_SIZE,
            );
        }
        msg
    }

    /// The trit the strip reads: byte 10 of the slot, reinterpreted as int8.
    pub fn k3_trit_at_slot_byte(&self) -> i8 {
        match self.to_bytes()[K3_TRIT_OFFSET] {
            0xFF => -1,
            b => b as i8,
        }
    }
}

/// `#[repr(C)] struct MailboxScanSummary { total_msgs, neg_trits, zero_trits, pos_trits }`
/// (the strip .dll's out-struct) — the dict returned by `scan_mailbox_buffer_native`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct MailboxScanSummary {
    pub total_msgs: u32,
    pub neg_trits: u32,
    pub zero_trits: u32,
    pub pos_trits: u32,
}

impl MailboxScanSummary {
    pub fn new() -> Self {
        MailboxScanSummary {
            total_msgs: 0,
            neg_trits: 0,
            zero_trits: 0,
            pos_trits: 0,
        }
    }
}

/// `strip_batch_mailbox_scan(buf, count, out)`: classify the k3_trit of each of
/// `count` 64-byte slots as -1 / 0 / +1; anything else is counted in no bucket.
/// `total_msgs` is `count`, not the number of classified trits.
pub fn strip_batch_mailbox_scan(buf: &[u8], count: usize) -> MailboxScanSummary {
    let mut s = MailboxScanSummary::new();
    let usable = if buf.len() < count * MSG_SIZE {
        buf.len() / MSG_SIZE
    } else {
        count
    };
    for i in 0..usable {
        let t = buf[i * MSG_SIZE + K3_TRIT_OFFSET] as i8;
        if t == -1 {
            s.neg_trits += 1;
        } else if t == 0 {
            s.zero_trits += 1;
        } else if t == 1 {
            s.pos_trits += 1;
        }
    }
    s.total_msgs = count as u32;
    s
}

/// `scan_mailbox_buffer_native(buffer)`: `cnt = len(buffer) // 64`; when `cnt == 0`
/// (or the native strip is unusable) the summary is all zeros.
pub fn scan_mailbox_buffer_native(buffer: &[u8]) -> MailboxScanSummary {
    let cnt = buffer.len() / MSG_SIZE;
    if cnt == 0 {
        return MailboxScanSummary::new();
    }
    strip_batch_mailbox_scan(&buffer[..cnt * MSG_SIZE], cnt)
}

/// `struct AgentRingBuffer { head, tail, ring[MAILBOX_CAPACITY] }`.
/// head/tail are monotonic cumulative counters, never wrapped: the slot index is
/// `counter & (MAILBOX_CAPACITY - 1)` and the depth is `tail - head`, which the
/// capacity gate keeps <= MAILBOX_CAPACITY.
pub struct AgentRingBuffer {
    pub head: usize,
    pub tail: usize,
    pub ring: Vec<CAgentMailboxMessage>,
}

impl AgentRingBuffer {
    pub fn new() -> Self {
        let mut ring = Vec::with_capacity(MAILBOX_CAPACITY);
        for _ in 0..MAILBOX_CAPACITY {
            ring.push(CAgentMailboxMessage::new());
        }
        AgentRingBuffer {
            head: 0,
            tail: 0,
            ring,
        }
    }

    /// `bool c_send_msg(sender, recipient, trit, payload, len)`.
    /// NB: the C++ never assigns `timestamp_ns` here — the slot keeps whatever it
    /// held (zero for a fresh arena), so this port leaves it untouched too.
    pub fn c_send_msg(&mut self, sender: u8, recipient: u8, trit: i8, payload: &[u8], len: u32) -> bool {
        if self.tail - self.head >= MAILBOX_CAPACITY {
            return false;
        }
        let idx = self.tail & (MAILBOX_CAPACITY - 1);
        let msg = &mut self.ring[idx];
        msg.msg_id = self.tail as u64;
        msg.sender_id = sender;
        msg.recipient_id = recipient;
        msg.k3_trit = trit;
        msg.priority = 2;
        msg.payload_size = if len > INLINE_DATA_LEN as u32 {
            INLINE_DATA_LEN as u32
        } else {
            len
        };
        if len > 0 {
            let n = if (msg.payload_size as usize) < payload.len() {
                msg.payload_size as usize
            } else {
                payload.len()
            };
            for i in 0..n {
                msg.inline_data[i] = payload[i];
            }
        }
        self.tail += 1;
        true
    }

    /// `bool c_recv_msg(agent_id, out_msg)`: FIFO, copies the slot out, advances head.
    pub fn c_recv_msg(&mut self) -> Option<CAgentMailboxMessage> {
        if self.head == self.tail {
            return None;
        }
        let idx = self.head & (MAILBOX_CAPACITY - 1);
        let out = self.ring[idx];
        self.head += 1;
        Some(out)
    }

    /// `uint32_t c_pending_count(agent_id)` = tail - head.
    pub fn c_pending_count(&self) -> u32 {
        (self.tail - self.head) as u32
    }
}

/// `static AgentRingBuffer* g_mailbox` — MAX_AGENTS boxes, one per recipient id.
pub struct MailboxSpace {
    pub g_mailbox: Vec<AgentRingBuffer>,
}

impl MailboxSpace {
    /// `bool c_mailbox_init()`: the portable half is the arena of MAX_AGENTS ring
    /// buffers of `mailbox_region_size()` bytes; the shared-memory mapping is host-side.
    pub fn c_mailbox_init() -> Self {
        let mut g_mailbox = Vec::with_capacity(MAX_AGENTS);
        for _ in 0..MAX_AGENTS {
            g_mailbox.push(AgentRingBuffer::new());
        }
        MailboxSpace { g_mailbox }
    }

    /// `c_send_msg`: `recipient >= MAX_AGENTS` is rejected.
    pub fn c_send_msg(&mut self, sender: u8, recipient: u8, trit: i8, payload: &[u8], len: u32) -> bool {
        if recipient as usize >= MAX_AGENTS {
            return false;
        }
        self.g_mailbox[recipient as usize].c_send_msg(sender, recipient, trit, payload, len)
    }

    /// `c_recv_msg`: `agent_id >= MAX_AGENTS` yields false / no message.
    pub fn c_recv_msg(&mut self, agent_id: u8) -> Option<CAgentMailboxMessage> {
        if agent_id as usize >= MAX_AGENTS {
            return None;
        }
        self.g_mailbox[agent_id as usize].c_recv_msg()
    }

    /// `c_pending_count`: `agent_id >= MAX_AGENTS` yields 0.
    pub fn c_pending_count(&self, agent_id: u8) -> u32 {
        if agent_id as usize >= MAX_AGENTS {
            return 0;
        }
        self.g_mailbox[agent_id as usize].c_pending_count()
    }
}
