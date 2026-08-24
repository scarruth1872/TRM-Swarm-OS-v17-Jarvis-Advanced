//! TRM Swarm OS - telemetry_fabric (mechanical no_std port, TRM platform engine).
//! Constants and structure extracted 1:1 from swarm_v2/core/telemetry_fabric.py.
//! Functions marked // MANUAL need human verification of math semantics.

extern crate alloc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;


//  TelemetryFrame 
#[derive(Debug, Clone)]
pub struct TelemetryFrame {
    pub frame_id: String,
    pub source_node: String,
    pub channel: String,
    pub payload: Vec<(String, f64)>,
    pub timestamp: f64,
}

impl TelemetryFrame {
    pub fn new() -> Self {
        TelemetryFrame {
            frame_id: String::new(),
            source_node: String::new(),
            channel: String::new(),
            payload: vec![],
            timestamp: 0.0,
        }
    }
}

//  get_telemetry_fabric() 
static SINGLETON_TELEMETRY_FABRIC: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
pub fn get_telemetry_fabric() -> u64 {
    // Singleton handle (lazy-init pattern from telemetry_fabric.py).
    // Returns 1 once configured; 0 until first call initializes.
    let v = SINGLETON_TELEMETRY_FABRIC.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_TELEMETRY_FABRIC.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}
