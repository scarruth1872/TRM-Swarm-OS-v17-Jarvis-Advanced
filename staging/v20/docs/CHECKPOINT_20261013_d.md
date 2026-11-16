# Checkpoint D — 2026-10-13 (blocks 310–319)

## State at checkpoint

- **Ledger: 319 blocks** (hash-chained monad journal).
- **Kernel v20.0.0 healthy on :8022**; supervisor running; dashboard builds clean.
- **Task executor: 40 compute modes** (up from 27 at checkpoint C).
- **Parsers: 22 native / 0 remaining** (unchanged).
- **All three on-disk checkpoints wired as first-class platform components.**

## What changed since checkpoint C

### GGUF dequantization (real weights, offline Rust)
`swarm_gguf` parse + `read_gguf_tensor` + `dequant_gguf` now read real checkpoint weights. Dequantizers: F32, F16, Q4_K, Q5_0, Q6_K, Q8_0, and the fork-specific **Q2_g64** (2-bit ternary, group-of-64, scale-first, 18 B/64 weights, value = (q-1)·d).

### qwen3moe 2.4B — full forward + lm_head (block 313–314)
Parsed `qwen-moe-2.4b-q4_k_m.gguf` (920 MB, 339 tensors, EXACT `swarm_voltage` arch match). Full 28-layer forward (attn_norm/q/k/v/output/q_norm/k_norm/ffn_norm/gate/up/down, Q4_K/Q6_K/F32) + RMSNorm → `hidden_rms 3.8409`. lm_head projects through `output.weight` [1024,151936] Q6_K → `argmax_token 3439, logit 4.5399`. Self-consistency: `output_norm.weight` RMS 3.91 ≈ `hidden_rms` 3.84.

### Instella-MoE 16B — sparse expert FFN (block 318)
`blk.0.ffn_gate`/`ffn_up` [2048,10944] Q4_K + `ffn_down` [10944,2048] Q8_0, 10944 = 64 experts × 171 dims. `compute_instella` dequants real weights, computes gate logits = ffn_gateᵀ·h over all 10944 dims, sigmoid, scores 64 experts → top-3, computes only those experts' gate⊙up → down (sparse: 513 of 10944 dims). Live: `top3_experts [30,31,56]`, `out_rms 0.00949`.

### Ternary-Bonsai 27B — Q2_g64 FFN, quant format corrected (block 318–319)
**Blocks 316–317 are superseded**: they sealed the Bonsai FFN primitive under the WRONG format (Q1_0, 6 B/32 weights, 1-bit). Empirical layout scan (smooth-scale test over block-size × field-order) proved the real format is **Q2_g64** — 2-bit ternary, group-of-64, scale-FIRST, 18 B/64 weights. Rewrote the type-42 byte-size, `dequant_gguf`, and the multiply-free dot (`tq2_0_dot`). FFN one-layer (post_attention_norm → gate/up → silu·gate·up → down) on real full-width weights: `multiply_free_dot == dequant_f32_dot` (`match=true`), gate ~O(1), output ~O(0.01). Block 319 validated the bit order empirically: dequant produces EXACTLY `{-d,0,+d}` per block, trit counts q∈{0,1,2} = {184,144,184}, q=3 (the +2d level) entirely UNUSED — genuinely ternary.

### Fixed along the way
- Bonsai FFN norm tensor name: `blk.0.ffn_norm.weight` → `blk.0.post_attention_norm.weight` (Qwen3.5 hybrid names it post-attention).
- Empty-tensor guard in `compute_bonsai` (returns an error receipt instead of panicking on a missing tensor).

## Honest boundaries (carried / new)

- **Both FFN forwards are single-token, single-layer (blk.0) proofs** with a synthetic sine input; no llama.cpp token-level reference comparison for any checkpoint.
- **Full forwards NOT built**: Bonsai Qwen3.5 hybrid attention (24 heads GQA, RoPE `[11,11,10,0]`) + Mamba SSM (`conv_kernel 4`, `state 128`, `inner 6144`) + 64-layer FFN loop; Instella MLA latent attention + full 64-layer sparse FFN. These are the next phase.
- Q2_g64 bit order validated on `blk.0.ffn_gate.weight` only (assumed uniform across tensors); the quantizing fork's source is not yet located.
- Remaining recall: ML/NLP/CV training runtimes, OWL reasoners, multi-node fabric — genuinely no engine.

## Council verdicts this phase
None new — the FFN work was direct execution on the prior `M_wire_remaining_wsl_crates` + re-run verdicts.

## What's next (gated — next phase)
1. Locate the quantizing fork's source to confirm Q2_g64 against its quantizer (close the last dequant assumption).
2. Bonsai full forward: Qwen3.5 hybrid attention + Mamba SSM + 64-layer FFN loop.
3. Instella full forward: MLA latent attention + 64-layer sparse FFN.

## Files that changed (key)
- `trm_uefi_kernel/host_council_server/src/task_executor.rs` — 40 modes; GGUF dequantizers incl. Q2_g64; `tq2_0_dot`; `compute_bonsai` FFN; `compute_instella` sparse FFN.
- `README.md` — latest-verified-state block (40 modes, 319 blocks).
- `scratch/seal_block318_ffn.py`, `scratch/seal_block319_bitorder.py` — seal scripts.
- `swarm_v2_artifacts/monad_ledger.jsonl` — blocks 310–319.
