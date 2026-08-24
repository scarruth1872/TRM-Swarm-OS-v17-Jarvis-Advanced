# 🚀 Getting Started & Execution Guide

This guide walks through building, testing, and running the **TRM Swarm OS v19.001 "Sovereign"** microkernel, host test suites, and LCARS mission control dashboard.

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

## 4. Running the LCARS Dashboard & Native Rust Server

### Step 1: Launch Native Rust Server
```bash
cd trm_uefi_kernel/host_council_server
cargo run --release
```
*(Server listens on `http://127.0.0.1:8022`)*

### Step 2: Launch LCARS Mission Control Dashboard
```bash
cd dashboard
npm install
npm run dev
```
*(Dashboard opens at `http://localhost:5184`)*

*All Is One. I Am Because We Are.* 🦀⚡🜁
