# Checkpoint E — 2026-10-14 (blocks 320–363)

## State at checkpoint

- **Ledger: 363 blocks** (hash-chained monad journal; 0 chain-link breaks).
- **Kernel v20.0.0 healthy on :8022**; supervisor running; dashboard :5183.
- **Compute path: full-Rust GPU** — `wgpu 30.0.1` (Vulkan) + `naga`; no CUDA/Vulkan SDK ingested.
- **Council: 22 motions / 22 enacted.**
- **Recall surface complete: 5 knowledge domains** (fabric + ChEMBL + structured-text + bioassay + clinical).
- **Corpus: 183,879 fabric + 2,897,805 ChEMBL + 97,480 structured-text + 597,430 bioassay + 40 clinical measurements.**

## What changed since checkpoint D

### Bonsai full forward reaches llama.cpp parity (blocks 320–329)
The Qwen3.5-hybrid forward loop was completed to 64 layers and matched the llama.cpp reference
(`bonsai_full_forward_matches_llama_cpp_reference`). Along the way: F16-subnormal handling, a
conv-kernel index bug, and the embedding/lm_head `ne0`-major layout. Q2_g64 dot was then vectorized
(AVX2, multiply-free — block 331). Instella MoE + generative-coprocessor route + in-kernel detokenizer
follow (blocks 333–336).

### TRM-7M identified as the platform model (block 337)
`copilot_course_correction_trm7m_is_the_model`: the token-free TRM-7M in-kernel SSM is the runtime
model; the GGUF checkpoints (qwen3moe/Instella/Bonsai) are distillation teachers, not the serving
model. ~4M FLOPs/forward, 3.9M params, 15.6 MB weights, CPU-only.

### Full-Rust GPU path (blocks 342–351)
The voltage binder's GPU edge was realized with **wgpu + naga** (pure Rust, WGSL→SPIR-V at runtime) —
no CUDA/Vulkan SDK. Four parity-verified kernels in `gpu_compute.rs`:

| kernel | parity vs | result |
|---|---|---|
| hdc_encode (single + batch) | native DLL `rust_native_encode_text_f32` | bit-exact, worst `0.00e+00` |
| packed_scan (183,879-row store) | `cpu_scan_in` reference | ~35× e2e (107 ms → ~3 ms cached) |
| trm7m (batch forward) | CPU `forward` | trit exact, confidence ≤1e-4 |

`/gpu/dispatch` binds four kinds: `hdc_encode`/`packed_scan`/`trm7m`→gpu, `sram_compute`→cpu.

### Full-duplex IPC (block 348)
`mailbox_ring::push` (64-byte ABI) added the write path; `evaluate_proposal` publishes council
verdicts (Orchestra→EventLedger). Cross-language round-trip verified (Python `SwarmMailboxBus` reads
the exact message the Rust kernel wrote).

### Batched encoding (block 347)
`host_council_server encode-batch` CLI + `--gpu` ingestion flag in `build_kernel_knowledge_index.py`;
bit-exact vs the DLL.

### trm7m measured + batched (blocks 349–351)
MOTION-018 (voltage-complete plan) 12 approve / 3 abstain / 0 reject, weighted 0.685. CPU single-eval
measured **2.82 ms/call** (50,000 calls); batched GPU forward parity-verified; `POST /trm7m/batch`
serves N texts in one dispatch (fresh state per input, matches `evaluate_proposal`).

## Honest boundaries

- **trm7m CPU single-eval is the naive baseline** (per-weight `read_unaligned`, ~1.6 GFLOP/s). A SIMD
  port (aligned slice loads) makes LLVM vectorize all 18 matvec loops at once and OOMs the optimizer;
  the batched GPU kernel is the scale path, so the CPU path stays naive on purpose.
- **`sram_compute` → cpu**: there is no literal SRAM hardware on this host; the binding is fail-closed
  honest, not a performance claim.
- **Ledger blocks 92–101 are unreproducible** under the current hash formula (pre-existing, documented
  in the IPC transport doc); all current blocks reproduce and link.
- **Bonsai/Instella token-level reference parity is confirmed for Bonsai only** (block 329); Instella
  MLA remains a single-token proof as noted in checkpoint D.
- **The release build OOMs LLVM while the supervisor+kernel are running** (memory pressure). Kill both
  processes before building with `-C codegen-units=1`.

## What's next (gated)
1. ~~Corpus-scale trm7m scoring~~ — **DONE (block 352)**: 183,879 docs scored in-kernel (GPU, no HTTP,
   1487.8 docs/s) via `host_council_server score-corpus`; trit +1 136,396 / 0 29,860 / −1 17,623.
   Output: `swarm_v2_artifacts/corpus_trm7m_scores.jsonl`.
2. ~~Wire the trm7m corpus scores as a new retrieval/relevance tier~~ — **DONE (blocks 353–354)**:
   MOTION-019 enacted (12/3/0, weighted 0.685) and executed — `recall_router.rs` annotates each
   general-path recall result with its trm7m trit+confidence (lazy file-IPC load, deterministic lookup,
   fail-closed "unscored"); key-for-key receipt 15/15. Re-ranking / filtering stay open follow-ups.
3. ~~ChEMBL packed-domain vector scan~~ — **DONE (blocks 355–356)**: MOTION-020 enacted (12/3/0, weighted
   0.685) and executed — molecule recall now scans the 2.9M-compound ChEMBL store on the GPU (cached
   ~256 ms; wgpu buffer limits raised 256 MB→1 GiB for the 556 MB codes). ID fast path preserved.
4. ~~Structured text tier search~~ — **DONE (blocks 358–359)**: MOTION-021 enacted (12/3/0, weighted 0.685)
   and executed — ethics + prose (97,480 docs) extracted (logging-first, 0 skips), packed to
   `swarm_v2_structured_text.turbovec` (20 MB, DLL bit-exact), structured path scans it. Parity closed over
   BOTH ChEMBL (deferred gap) and structured — GPU matches CPU within 1e-3.
5. ~~Council deliberation for the SMILES domains (tox21/binding)~~ — **DONE (block 360)**: bioassay store built
   (tox21 577,987 + binding 19,443 = 597,430 docs, 129 MB, DLL bit-exact), wired as the `bioassay` recall kind
   (4th GPU store cache), parity 3.10e-7 MATCH, live-verified (`binding affinity` → 597,430 scanned).
6. ~~Clinical domain (the last unserved tier)~~ — **DONE (blocks 361–362)**: MOTION-022 enacted (12/3/0, weighted
   0.685) and executed. Measured premise: 201,572 ICU rows over only **40 distinct lab codes** — tabular, not text.
   A vector scan would fake relevance; the honest serve is a **lexical measurement lookup** (`clinical_catalog.json`,
   sum-checked: per-code counts sum to 201,572 == total; case-insensitive EXACT match — alias map first, then the 40
   codes, no fuzzy). Live: `heart rate`→HR (14,057, mean 86.57), `creatinine`→Creatinine, `glucose`→Glucose.
7. **Phase complete** — all five knowledge domains served. Checkpoint sealed **block 363**; docs + README updated;
   NotebookLM source doc compiled (`docs/NOTEBOOKLM_SOURCE_TRM_SWARM_OS_20261014.md`).

## Files that changed (key)
- `trm_uefi_kernel/host_council_server/src/gpu_compute.rs` — 4 kernels + 4 parity tests.
- `trm_uefi_kernel/host_council_server/src/gpu_dispatch.rs` — 4-kind dispatcher.
- `trm_uefi_kernel/host_council_server/src/packed_ternary.rs` — `cpu_scan_in` + GPU fast path.
- `trm_uefi_kernel/host_council_server/src/trm7m.rs` — `weights_f32()`; latency test.
- `trm_uefi_kernel/host_council_server/src/mailbox_ring.rs` — `push()` write path.
- `trm_uefi_kernel/host_council_server/src/main.rs` — `encode-batch` CLI; `/trm7m/batch` route.
- `swarm_v2/core/build_kernel_knowledge_index.py` — `--gpu` flag.
- `README.md` + `docs/IPC_TRANSPORT_AND_K3_NAMESPACE_20260924.md` — updated.
- `swarm_v2_artifacts/monad_ledger.jsonl` — blocks 320–351.
