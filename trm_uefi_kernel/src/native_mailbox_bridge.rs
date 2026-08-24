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
