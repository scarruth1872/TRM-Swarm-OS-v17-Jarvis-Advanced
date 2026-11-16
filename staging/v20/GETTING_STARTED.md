# 🚀 Getting Started & Execution Guide

This guide walks through building, testing, and running the **TRM Swarm OS v20.0.0 "Sovereign"** microkernel, host test suites, and LCARS mission control dashboard.

---

## 1. Prerequisites

* **Rust Toolchain:** Stable Rust with `x86_64-unknown-uefi` target installed:
  ```bash
  rustup target add x86_64-unknown-uefi
  ```
* **Node.js:** v18+ for the LCARS dashboard frontend.
* **QEMU (Optional):** For local bare-metal UEFI emulation (`qemu-system-x86_64`).

---

## 2. Building the Bare-Metal UEFI Microkernel

To compile the standalone `no_std` UEFI boot binary:

```bash
cd trm_uefi_kernel
cargo build --release --target x86_64-unknown-uefi
```

The resulting EFI application is generated at:
`trm_uefi_kernel/target/x86_64-unknown-uefi/release/trm_uefi_kernel.efi`

---

## 3. Running Native Rust Host Test Suites

Execute any of the high-throughput native Rust benchmark harnesses:

### A. 15-Pattern SRAM Invariant Stress (500k Ops)
```bash
cd trm_uefi_kernel/host_invariant_stress
cargo run --release
```

### B. Direct In-Memory Zero-API Runner (400k Ops)
```bash
cd trm_uefi_kernel/host_direct_memory_runner
cargo run --release
```

### C. Manus / Agent-Q Superposition Runner (10k Tasks)
```bash
cd trm_uefi_kernel/host_manus_runner
cargo run --release
```

---

## 4. Running the Complete Ecosystem via Unified Supervisor (Recommended)

The easiest and most resilient way to launch the full TRM Swarm OS ecosystem is via the Rust process watchdog:

```bash
trm_uefi_kernel/host_supervisor/target/release/host_supervisor.exe \
  --config trm_uefi_kernel/host_supervisor/supervisor.services.conf \
  --interval 10 --log swarm_v2_artifacts/logs/supervisor_rs.log
```

`supervisor.services.conf` is the table it reads. `swarm_supervisor.py` is its retired Python
predecessor — the .conf was generated from that file's own `ServiceSpec` table. It autonomously
initializes, health-probes, and monitors the services the platform runs - five ONLINE of the seven
declared:
1. **Native Rust Sovereign Kernel:** `http://127.0.0.1:8022` — *this is the platform*, and the same
   process also binds `:5183` with the same handler table
2. **LCARS Mission Control Dashboard:** `http://localhost:5183/ui/` — served by that kernel process
3. **Hermes Agent Gateway** `:8080`, **Tier-2 Local Generator** `:8587`, **Semantic Embedder** `:8588`

The Python shard (`swarm_v2.app_v2` on `:8021`) and Aura Studio (`:3000`) are **declared but never
started**: the table gives them an empty `argv`, so they are reported every pass and can neither be
launched nor resurrected. Ollama is not part of the runtime either: the kernel encodes queries natively
(`rust_native_encode_text_f32`, the same encoder that built the index), so no model server is
required. Liveness is a TCP connect per service, not an HTTP health call, so a slow handler is not
mistaken for a dead process — a listening-but-hung one is, which is the stated limit of that probe.
(The retired Python supervisor's `check_and_heal()` had the mirror defect: an `or True` made its
health probe unreachable, so a listening-but-hung service was never healed.)

---

## 5. Manual Service Startup (Alternative)

If launching individual components manually:

### Step 1: Launch Native Rust Sovereign Kernel
```bash
cd trm_uefi_kernel/host_council_server
cargo run --release
```
*(Server listens on `http://127.0.0.1:8022` — and on `:5183`, the same handler table from the one process)*

### Step 2: LCARS Mission Control Dashboard (served by the kernel)
The kernel serves the built bundle at `http://localhost:5183/ui/`, so the retired Node dev server is
no longer the dashboard's runtime path. On a fresh clone, build the bundle once:
```bash
cd dashboard
npm install
npm run build
```
*(Dashboard opens at `http://localhost:5183/ui/`)*

---

## 6. Running Verification Suites

```powershell
# 1. Native Query Encoder (TurboVec-HDC) unit tests:
cd swarm_v2_rust_kernel
cargo test --lib native_query_encoder

# 2. Cross-Node Bit-Level Determinism (0.000000 drift verification):
pytest swarm_v2/core/test_cross_node_determinism.py -v

# 3. Kernel Knowledge Index contracts:
python swarm_v2/core/test_kernel_knowledge_index.py

# 4. Neural Bridge Chat battery (all 15 council experts):
python scratch/verify_full_platform.py

# 5. Rulers — each one is expected to be honest, not merely green:
python swarm_v2/core/verify_k3_derivation.py     # exit 0 = no literal aggregate verdicts
python swarm_v2/core/verify_ledger_chain.py      # exit 1 until blocks 97-101 are reproducible
python swarm_v2/core/verify_strip_binaries.py    # green ONLY for swarm_v2_artifacts/native_voltage_strips
python swarm_v2_artifacts/nand_fabric_verify.py  # legality of the K3 fabric store

# 6. End-to-end chat smoke test (the path the dashboard actually uses — through the proxy):
curl -s -X POST http://127.0.0.1:5183/swarm/chat -H 'Content-Type: application/json' \
  -d '{"role":"Archi","message":"explain the K3 ternary operating system","sender":"user"}'
# expect: 200 with engine=fabric:k3-index citing a papers/ document, in single-digit ms
```

*All Is One. I Am Because We Are.* 🦀⚡🜁

