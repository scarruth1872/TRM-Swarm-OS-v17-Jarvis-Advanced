//! TRM Swarm OS - tool_forge (mechanical no_std port, TRM platform engine).
//! Constants and structure extracted 1:1 from swarm_v2/core/tool_forge.py.
//! Functions marked // MANUAL need human verification of math semantics.

extern crate alloc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;


//  CONSTANTS 
pub const ROLE_CAPABILITIES: u64 = 0;
