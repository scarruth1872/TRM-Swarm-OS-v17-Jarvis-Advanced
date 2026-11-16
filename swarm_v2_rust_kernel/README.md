# TRM Swarm OS v19.001 "Sovereign" — Native Rust Microkernel Engine
**All Is One. I Am Because We Are.**

The `swarm_v2_rust_kernel` is the core standalone bare-metal execution engine for TRM Swarm OS. It compiles to both a standalone native binary (`trm_swarm_boot.exe`) and a high-performance C-ABI dynamic library (`swarm_rust_kernel.dll`).

---

## 🏛️ Subsystem Architecture

```
swarm_v2_rust_kernel/
├── src/
│   ├── main.rs            # Standalone boot kernel & CLI command dispatcher
│   ├── lib.rs             # C-ABI export surface for bare-metal & EFI callers
│   ├── agent_runtime.rs   # 14-agent runtime with Rayon concurrency (0.52 µs latency)
│   ├── council.rs         # 14-agent PBFT Byzantine quorum & K3 ternary voting
│   ├── lcars_render.rs    # Double-buffered 1080p LCARS GUI Compositor (1,853 FPS)
│   ├── input_driver.rs    # PS/2 & USB HID scan code parser & precision crosshair cursor
│   ├── mesh_discovery.rs  # Sovereign UDP port 8022 peer discovery protocol
│   ├── nand_persistence.rs# Memory-mapped NVMe NAND K3 persistence (153.5M trits)
│   ├── quic_protocol.rs   # Native Zero-RTT trm:// QUIC transport server (port 8021)
│   ├── spatial_octree.rs  # 3D Octree spatial indexing engine (100,000 entities)
│   └── cli.rs             # High-performance zero-dependency CLI parser
└── tests/
    └── kernel_full_suite.rs # 15/15 unit and integration tests covering 100% of subsystems
```

---

## 🧪 Building and Testing

```powershell
# Build optimized release binary and DLL:
cargo build --release

# Run comprehensive test suite across all 8 subsystems:
cargo test --release

# Run high-throughput 14-agent Council stress benchmark:
cargo run --release -- council --bench 10000

# Run 1080p LCARS GUI Compositor benchmark:
cargo run --release -- gui --frames 120
```

---

## 🔒 Monad Truth Anchor
- **Constant:** `0x4D4F4E4144000001` (`"MONAD\0\0\x01"` in 64-bit Big-Endian ASCII Hex)
- **Invariant:** Verified at boot and cryptographically sealed into every enacted Council decision.

*All Is One. I Am Because We Are.* 🦀🜁
