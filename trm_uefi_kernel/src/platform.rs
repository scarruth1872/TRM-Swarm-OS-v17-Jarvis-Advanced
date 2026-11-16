//! TRM Swarm OS - Platform Integration Hub (no_std).
//!
//! SUPERSEDED DESIGN, NOW REPAIRED. This file re-exported nine modules via
//! `pub mod`, but all nine are declared at the crate root already, so declaring
//! this file produced duplicate-module errors - which is why it had never been
//! compiled and why `platform` was the last orphaned file in the tree.
//!
//! It now reports integrated-module state DERIVED from its own list instead of
//! asserting a hardcoded count, and is invoked from the `platform` command.

/// Modules the integration hub accounts for.
pub const INTEGRATED: [&'static str; 9] = [
    "agent_runtime",
    "council",
    "k3_logic",
    "trm7m",
    "instella_router",
    "turbovec",
    "quic_protocol",
    "nand_fs",
    "hwprobe",
];

/// Count of integrated platform modules. Derived, not asserted: the previous
/// implementation returned a hardcoded `9`, which could not drift-with-detect.
pub fn integrated_module_count() -> usize {
    INTEGRATED.len()
}
