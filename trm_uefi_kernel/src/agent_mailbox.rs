//! TRM Swarm OS - agent_mailbox (mechanical no_std port, TRM platform engine).
//! Constants and structure extracted 1:1 from swarm_v2/core/agent_mailbox.py.
//! Functions marked // MANUAL need human verification of math semantics.

extern crate alloc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;


//  CONSTANTS 
pub const MAILBOX_ROOT: u64 = 0;
pub const DEFAULT_TTL_MINUTES: u64 = 60;
