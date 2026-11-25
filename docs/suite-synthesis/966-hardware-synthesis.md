# Hardware knowledge synthesis

## The machines, as measured

| | desktop (TRM-DESK-6700XT) | laptop (TRM-LAP-3070) |
|---|---|---|
| CPU | Ryzen 9 3900X, 12C/24T | Ryzen 9 5900HX |
| GPU | RX 6700 XT 12 GB — **Vulkan only, no ROCm/CUDA** | RTX 3070 Laptop — **no nvcc** |
| RAM | 32 GB DDR4-2400 | 15 GB |
| Root of trust | **no TPM** (Get-Tpm: TpmPresent False) | not checked |
| Verified | 141 tests pass, UEFI kernel boots from `G:` | reachable over SSH |

## What is proven on this hardware

- **The UEFI kernel is real and bootable**: `trm_uefi_kernel.efi` is 17,488,896 B and byte-identical to
  `G:/EFI/BOOT/BOOTX64.EFI` on a FAT32 volume labelled `SWARM_OS`.
- **GPU inference runs through Vulkan** (`ggml-vulkan.dll`), not DirectML; the resident model is gpt-oss-20b
  MXFP4 on `:8587`.
- **The rails are measurable**: HWiNFO SM2 gives v_core 1.488 V, v_soc 1.03125 V over 417 readings; the
  3900X is pinned at 40× / 4000 MHz with C6 disabled.

## What the hardware cannot do

- **No CUDA kernel work** (10 suite tasks) — neither node has a CUDA toolkit, and the desktop GPU has no
  ROCm either.
- **No TPM attestation** (3 tasks) — the root of trust is absent, so any attestation story is software-only.
- **No FPGA, SDR, muon detector, robot or IoT device** (22 further tasks).
- **No 100 ns MD** — no OpenMM, and no GPU path to accelerate it.

## The synthesis

This is a two-node, GPU-inference, no-root-of-trust deployment. Its hardware strengths are a 12-core CPU with
honest telemetry and a Vulkan inference path that works; its hardware ceiling is the absence of CUDA, a TPM
and any laboratory instrument. Documents that describe it otherwise — as a swarm of 584 agents on 10 nodes,
or as an attested enclave — are describing a machine that does not exist here.

---

*Task 966 — 966-hardware-synthesis. Generated 2026-11-25 14:56 UTC by `tools/suite_synthesis.py`. Class C.
This is a written deliverable: it is class C/N, which means the platform can produce it and **cannot
verify it**. Every number above was read in the same process that wrote this file.*

Sources: verified shard manifests, suite_classification.json, live endpoints

Cannot claim: that any statement here is verified by the platform; this document is class C/N
