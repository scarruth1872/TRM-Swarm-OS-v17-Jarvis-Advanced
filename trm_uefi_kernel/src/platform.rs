//! TRM Swarm OS — Complete Platform Integration Hub (no_std).
//! Aggregates every ported intelligence module into the booted kernel.
//! Populated as porting batches land.

// Verified & integrated:
pub mod agent_runtime;        // 14-agent dispatch matrix + health
pub mod council;              // NativeSwarmCouncil K3 PBFT
pub mod k3_logic;             // Kleene ternary engine
pub mod trm7m;                // TRM 7M Mamba-3 SSM (embedded weights)
pub mod instella_router;      // 64-expert / 8-team MoE router
pub mod turbovec;             // 2-bit 16x quantizer + femto SSM scan
pub mod quic_protocol;        // trm:// TRM0 frame codec
pub mod nand_fs;              // NAND fabric mount + sector verify
pub mod hwprobe;              // CPUID / rdmsr / GetTime probes

/// Count of integrated platform modules (grows with each landing batch).
pub fn integrated_module_count() -> usize {
    9
}
