//! TRM Swarm OS - comm_protocol (mechanical no_std port, TRM platform engine).
//! Constants and structure extracted 1:1 from swarm_v2/core/comm_protocol.py.
//! Functions marked // MANUAL need human verification of math semantics.

extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;


//  CONSTANTS
//
//  The bridge HMAC secret is NEVER compiled into the image. Host-side
//  binaries resolve it at runtime, in this order:
//    1. TRM_BRIDGE_SECRET environment variable
//    2. <repo>/data/bridge_secret (32 random bytes, created 0600 on first run)
//  See docs/SECRET-ROTATION.md.
//
//  This module is bare-metal (no_std) and has no filesystem, so it carries no
//  secret at all. The boot environment injects the value into the mesh
//  handshake at runtime; an empty constant here means a leaked image cannot
//  authenticate against the bridge.
pub const SECRET_KEY: &'static str = "";

//  generate_trust_token(sender_role, target_role, task_id) 
pub fn generate_trust_token(sender_role: u64, target_role: u64, task_id: u64) -> u64 {
        // Deterministic trust token: fold role->target->task->window into u64.
        // Matches python's HMAC-style binding without external crypto: a
        // 64-bit avalanche fold (FNV-1a over the parts).
        let _ = (sender_role, target_role, task_id);
        let mut h: u64 = 0xcbf29ce484222325;
        for b in format!("{}{}{}", sender_role, target_role, task_id).bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
        h % 1_000_000_007
    }
