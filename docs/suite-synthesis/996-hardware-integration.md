# Hardware integration

## CPU ↔ kernel ↔ telemetry

The 3900X is pinned at 40× / 4000 MHz with C6 disabled; HWiNFO SM2 supplies v_core (1.488 V observed) and
v_soc across 417 readings. The kernel reads these rails; φ_v (0.8096) is derived and **saturates**, which means
the feature is effectively constant and cannot discriminate between states. That is a hardware-integration
defect with a measurement behind it.

## GPU ↔ inference

`ggml-vulkan.dll` serves gpt-oss-20b MXFP4 on `:8587`. There is no CUDA and no ROCm; DirectML figures are not
quoted because they were shown unreliable. GPU throughput claims therefore rest on Vulkan numbers only.

## Firmware ↔ boot

The UEFI kernel (17,488,896 B) is byte-identical to the boot media on a FAT32 `SWARM_OS` volume. This is the
one hardware integration that is proven end to end: source → build → image → media → boot.

## What hardware integration cannot reach

- **attestation** — no TPM, so no measured boot chain;
- **CUDA kernels** — no toolkit on either node;
- **instrumentation** — no SDR, detector or lab instrument is attached, so no measurement originates in
  physical hardware other than the rails.

## Verdict

Hardware is integrated where it is general-purpose (CPU, GPU inference, boot) and entirely absent where it is
special-purpose (attestation, instruments, accelerators). Any document claiming the latter is describing
purchases, not capabilities.

---

*Task 996 — 996-hardware-integration. Generated 2026-11-25 14:56 UTC by `tools/suite_synthesis.py`. Class C.
This is a written deliverable: it is class C/N, which means the platform can produce it and **cannot
verify it**. Every number above was read in the same process that wrote this file.*

Sources: verified shard manifests, suite_classification.json, live endpoints

Cannot claim: that any statement here is verified by the platform; this document is class C/N
