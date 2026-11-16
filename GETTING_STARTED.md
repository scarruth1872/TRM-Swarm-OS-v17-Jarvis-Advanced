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

**Do not run `cargo build --release --target x86_64-unknown-uefi` directly** — measured on rustc 1.99.1, the crate's
own release profile (opt-level 3) fails with `0xc00000fd STATUS_STACK_OVERFLOW` inside LLVM's thin-LTO module
thread. Neither `RUST_MIN_STACK` at 512 MB nor `codegen-units=256` avoids it; opt-level 2 without LTO builds in
~21 s.

Use the scripted path, which builds, validates the PE and can deploy:

```bash
bash trmboot/build_uefi.sh              # build + PE validation + refresh repo copies
bash trmboot/build_uefi.sh --deploy G   # + deploy to G: and prove four-name hash parity
bash trmboot/build_uefi.sh --check G    # verify media against the repo artifact (no build)
```

The artifact lands at `scratch/uefi_build/x86_64-unknown-uefi/release/trm_uefi_kernel.efi` and is copied to
`trm_uefi_kernel.efi` and `trmboot/trmboot_pe.efi` in the repo.

## 2b. Verifying the boot without rebooting

```bash
python tools/qemu_boot_test.py                     # headless QEMU/OVMF boot (screenshots + serial)
python tools/qemu_boot_test.py --fabric 76761697   # positive control: K3 fabric must report MOUNTED
python tools/qemu_boot_test.py --nic               # positive control: SimpleNetwork interface must appear
```

The kernel writes `TRM-BOOT|key=value` to COM1 at 115200 8N1, so every hardware line can be checked as text. Absent
hardware must read as absent (`NOT MOUNTED`, `NO SimpleNetwork DEVICE`, `TRITS UNMEASURED`) — if the panel prints a
nominal resolution, the media is carrying a stale artifact.

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
