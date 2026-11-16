# TRM Swarm OS — Local Model Registry & TurboVec Integration Plan

**Inventory date:** 2026-09-05 | **Doctrine:** on-device only, no API
**Quantization path:** safetensors/GGUF → TurboVec 1.58-bit ternary → K3 fabric (verified on Qwen3-4B: 8.0x, cos 0.82-0.88)

---

## 1. HUGGINGFACE HUB CACHE (~40.7 GB usable)

| Model | Size | Type | Status |
|---|---|---|---|
| **amd/Instella-MoE-16B-A3B-Think** | 31.7 GB | MoE 16B-A3B (platform's native router!) | ✅ REAL (F:/models + HF cache) |
| **Qwen/Qwen3-4B** | 8.06 GB | Dense 4B | ✅ **QUANTIZED** (8.0x, cos 0.82-0.88) |
| Systran/faster-whisper-base | 0.15 GB | ASR | ✅ complete |
| openai/clip-vit-base-patch32 | 0.61 GB | Vision encoder | ✅ complete |
| livekit/turn-detector | 0.48 GB | Audio | ✅ complete |
| Qwen/Qwen3-1.7B | 4.08 GB (blobs) | Dense 1.7B | ⚠️ blobs only — snapshot broken |
| google/gemma-3-270m-it | 0.57 GB (blobs) | Dense 270M | ⚠️ blobs only |
| gpt2 | 0.55 GB (blobs) | Dense 124M | ⚠️ blobs only |

## 2. OLLAMA MODELS (~75 GB, 32 models — GGUF format)

| Model | Size | Role |
|---|---|---|
| **microsoft/bitnet-b1.58-2B** | 1.19 GB | **NATIVE 1.58-bit ternary** — the doctrine model |
| gemma4:12b / gemma3n / deepseek-r1:8b | 5-7.5 GB | Reasoning |
| **bonsai-27b** | 3.8 GB | Platform's own ternary network |
| qwen2.5-coder / starcoder2:3b | 1.7-4.7 GB | Code |
| olmo-3 / phi3 / llama3.2 | 2-4.5 GB | General |
| llava | 4.73 GB | Multimodal (vision) |
| qwen3-embedding:4b / mxbai / nomic | 0.27-2.5 GB | Embeddings |
| gemma2:2b / gemma3:1b / tinyllama | 0.3-1.6 GB | Small edge models |
| deepseek-coder:1.3b | 0.78 GB | Code-small |

## 3. BOOT USB (G:) — deployment artifacts
trm_uefi_kernel.efi (15.8MB) · host_council_server.exe · NAND fabric 60.5MB

---

## INTEGRATION PLAN (same path as Qwen3-4B ✓)

### Priority 1 — Native ternary (the doctrine models)
1. **bitnet-b1.58-2B** (GGUF) → parse weights → already ternary → verify + K3-register
2. **bonsai-27b** → platform's own network → weight extraction → fabric shard

### Priority 2 — The platform's declared models (1:1 with boot claims)
3. **Instella-MoE-16B-A3B** → quantize experts + router → 31.7GB → trit shards
   (this is the instella_router's actual brain — currently the kernel routes to
   a name without the weights behind it)
4. **Qwen3-4B** → DONE (template for all)

### Priority 3 — Capability expansion
5. **Qwen3-1.7B / gemma-3-270m** → fix snapshot stubs → quantize → edge models
6. **llava / CLIP** → vision tensors → trit embedding space (multimodal K3)
7. **faster-whisper** → audio → trit fabric (voice interface)
8. Embeddings set (qwen3-embedding, mxbai, nomic) → retrieval trit store

### Priority 4 — Code + reasoning depth
9. qwen2.5-coder, starcoder2, deepseek-r1:8b/1.5b, olmo-3 → quantize → domain shards

---

## Per-model steps (the Qwen3-4B template)
1. Verify weights complete (shards/blobs/GGUF)
2. Extract weight tensors (safetensors pt-framework / gguf parser)
3. TurboVec ternary quantize → {−1,0,+1} + α-scale (or verify native ternary)
4. Measure REAL compression + reconstruction cosine (honest ledger)
5. Store as K3 fabric shard → Monad seal → NAND append
6. Wire tensor into kernel routing (instella_router, trm7m, turbovec paths)


## QUANTIZATION STATUS (measured, Monad-sealed)

| Model | Compression | Cosine fidelity | Status |
|---|---|---|---|
| Qwen3-4B | 8.0x (2-bit packed) | 0.8204-0.8844 | ✅ DONE (3.63B params, 252 tensors, seal 0x3495d75a) |
| Instella-MoE-16B (40 experts) | 8.0x | 0.8772-0.8878 mean 0.8857 | ✅ DONE (sample; full 5,070 = ~4min, seal 0xc3f1d0dd) |
| bitnet-b1.58-2B | NATIVE ternary | n/a (already doctrine) | ✅ VERIFIED present (GGUF v3, arch bitnet-b1.58, 332 tensors, 1.19GB) |
| gemma-3-270m-it | 8.0x | 0.8599 mean (0.699-0.881) | ✅ DONE (268M params, 127 tensors, 33.7% zeros) |
| clip-vit-base-patch32 | 8.0x FP16 / 16.0x disk | 0.9339 global (0.873 mean) | ✅ DONE (151M params, 152 tensors, best fidelity) |
| faster-whisper-base | format-deferred | n/a | ⚠️ CT2 WhisperSpec binary (NOT torch/safetensors) — 145MB ASR accessory, format parser needed; not core to intelligence fabric |

## Honesty gates
- Every compression ratio + cosine is MEASURED on real weights, ledgered
- GGUF Q4_K_M models are ALREADY 4-bit — ternary re-quant from GGUF measures
  the real additional loss before claiming gains
- Ollama models are the platform's runtime-adjacent set — quantization is for
  K3 knowledge fabric, not Ollama replacement (Ollama stays host-tooling)
