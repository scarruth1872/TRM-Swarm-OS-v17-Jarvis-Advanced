//! TRM Swarm OS - mcp_bus (mechanical no_std port, TRM platform engine).
//! Constants and structure extracted 1:1 from swarm_v2/core/mcp_bus.py.
//! Functions marked // MANUAL need human verification of math semantics.

extern crate alloc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;


//  ToolSchema 
#[derive(Debug, Clone)]
pub struct ToolSchema {
    pub name: String,
    pub description: String,
    pub input_schema: Vec<(String, f64)>,
    pub output_schema: Vec<(String, f64)>,
    pub provider: String,
    pub version: String,
    pub registered_at: String,
}

impl ToolSchema {
    pub fn new() -> Self {
        ToolSchema {
            name: String::new(),
            description: String::new(),
            input_schema: vec![],
            output_schema: vec![],
            provider: String::new(),
            version: String::new(),
            registered_at: String::new(),
        }
    }
}

//  ToolCallResult 
#[derive(Debug, Clone)]
pub struct ToolCallResult {
    pub tool_name: String,
    pub success: bool,
    pub result: u64,
    pub error: String,
    pub elapsed_ms: f64,
}

impl ToolCallResult {
    pub fn new() -> Self {
        ToolCallResult {
            tool_name: String::new(),
            success: false,
            result: 0,
            error: String::new(),
            elapsed_ms: 0.0,
        }
    }
}

//  get_mcp_bus() 
static SINGLETON_MCP_BUS: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
pub fn get_mcp_bus() -> u64 {
    // Singleton handle (lazy-init pattern from mcp_bus.py).
    // Returns 1 once configured; 0 until first call initializes.
    let v = SINGLETON_MCP_BUS.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_MCP_BUS.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}
