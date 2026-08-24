//! TRM Swarm OS - openclaw_gateway (mechanical no_std port, TRM platform engine).
//! Constants and structure extracted 1:1 from swarm_v2/core/openclaw_gateway.py.
//! Functions marked // MANUAL need human verification of math semantics.

extern crate alloc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;


//  CONSTANTS
// (host-side OpenClaw API URL removed — the no_std kernel makes no HTTP
//  calls; zero external API claim holds on-device)
pub const INBOX_FILE: &'static str = "swarm_v2_artifacts/openclaw_inbox.json";
pub const HEARTBEAT_FILE: &'static str = "HEARTBEAT.md";
pub const INTENT_ROUTING: u64 = 0;

//  InboundMessage 
#[derive(Debug, Clone)]
pub struct InboundMessage {
    pub text: String,
    pub sender: String,
    pub channel: String,
    pub channel_id: String,
    pub timestamp: String,
    pub intent: String,
    pub target_specialist: String,
    pub metadata: Vec<(String, f64)>,
}

impl InboundMessage {
    pub fn new() -> Self {
        InboundMessage {
            text: String::new(),
            sender: String::new(),
            channel: String::new(),
            channel_id: String::new(),
            timestamp: String::new(),
            intent: String::new(),
            target_specialist: String::new(),
            metadata: vec![],
        }
    }
}

//  ChannelAdapter 
#[derive(Debug, Clone)]
pub struct ChannelAdapter {
    pub channel_name: String,
}

impl ChannelAdapter {
    pub fn new() -> Self {
        ChannelAdapter {
            channel_name: String::new(),
        }
    }
}
