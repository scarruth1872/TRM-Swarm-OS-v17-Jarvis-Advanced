# Voltage instruments — harvested 2026-09-26

The four tools that measure the voltage node, plus the captures they produced. They came out of
`/home/shawn/prism-ml-llama.cpp/examples/` as **untracked** files in a WSL llama.cpp fork (no other copy), while
their outputs were **git-ignored** in `/home/shawn/trm-rust`. The WSL reconciliation classified all of it as
`harvest`; this directory is the harvest. `HARVEST_RECEIPT.json` carries a sha256 for every file, computed at
write time.

## The instruments (258 lines)

| file | what it is |
|---|---|
| `voltage_node.cpp` (60) | the GPU fast-path voltage node: llama.cpp CUDA runs the real forward, reads the hidden state, and the verified K3 collapse produces the action vector — no CPU matmul re-derivation. Writes `hidden_state.bin` (1024 f32) and prints the K3 vector. |
| `voltage_node_offload.cpp` (56) | the same node for models that do not fully fit (3.8 GB Bonsai against 8 GB with backend load): `<model> <n_gpu_layers> <n_tokens> <tok>...` |
| `capture_hidden.cpp` (91) | reads the final-layer hidden state before the output softmax (`llama_get_embeddings_ith`) and prints it as f32. |
| `capture_layers.cpp` (51) | captures the hidden state after **each** block (`[n_layers+1][1024]` f32), so a Rust forward is verified per-layer instead of in one giant run. |

## The references (`references/`, 69,036 B)

The instruments' other half: without these the comparison has nothing to compare against.

- `ref_hidden.bin`, `hidden_state.bin`, `field_qwen.bin` — **one identical file** (sha256
  `9c658dc8cebfdeac…`, 4,100 B = u32 1024 + 1024 f32) under three names: the qwen hidden state, all 1024
  values nonzero, min −28.319275 / max 12.265035.
- `field_bonsai.bin` — a distinct **5120-dim** vector (20,484 B, `d1d14604fe5f7ac5…`).
- `ref_hidden.log` (35,513 B) — the run that produced them, and the evidence that matters: **339 tensors and
  55 KV pairs loaded from `/home/shawn/models/qwen-moe-2.4b-q4_k_m.gguf`** on the **NVIDIA GeForce RTX 3070
  Laptop GPU, compute capability 8.6, VRAM 8191 MiB**, under CUDA. That is a measurement of a model the
  platform earlier recorded as merely "unverified", and the reconciliation manifest's `models` verdict now
  says so.
- `fabric_report.txt` / `fabric_report_ensemble.txt` — v18-era **declared** fabric state (trit count
  170757175, codec 2). Historical: the live fabric measures `76,761,697 B → 153,523,394` trits, so these are
  kept as a v18 record and must not be read as current.
- `agent_output.txt`, `display_payload.bin`, `display_payload_ensemble.bin` — small text payloads.

## Rebuilding them (stated, not claimed)

They are llama.cpp examples — they `#include "llama.h"`. This host has **no `g++` and no `cl`**, and the
Hermes runtime ships `AppData/Local/hermes/runtimes/llamacpp/b10964/cuda/llama.dll` **without headers**. The
WSL side has the source checkout and `g++` 15. So the harvest is source plus a build path, deliberately not a
claim that a binary was produced.

## Preserved twice

All of this (and `font8x8.bin`, which `swarm-boot/src/main.rs:30` `include_bytes!`s — an ignored file a crate
cannot build without) is also inside `backups/wsl_freeze_20260923_180202/trm-rust_source.tar.gz`. Verified by
listing the archive: every ignored artifact is in it; only `target/` build output is excluded, and that is
reproducible from the archived source.
