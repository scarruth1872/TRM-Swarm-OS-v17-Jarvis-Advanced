
# Research Integration Plan: transformer architecture improvements

## Overview
This research enables two critical upgrades to the QIAE Swarm's neural core:
1.  **Sub-Quadratic Attention (Source 1):** Replace the standard O(n²) attention mechanism with a sub-quadratic variant (e.g., linear attention, flash attention with kernel fusion, or state-space model hybrids). This directly addresses context-length bottlenecks in the swarm's long-term memory and inter-agent message passing.
2.  **Architectural Minimalism (Source 2):** Apply the "minimum viable transformer" principle to strip non-essential components (e.g., redundant normalization layers, unnecessary feed-forward expansion) from the QIAE encoder/decoder. This reduces inference latency and memory footprint without sacrificing representational capacity.

## Proposed Changes

### 1. Core Architecture (`qiae/nn/transformer.py`)
- **Modification:** Introduce a configurable `AttentionMechanism` enum (`STANDARD`, `LINEAR`, `FLASH`, `MAMBA`).
- **New Class:** `SubQuadraticAttention(nn.Module)` implementing linear attention (e.g., based on Performer or Linformer principles).
- **Refactor:** `TransformerBlock.__init__()` to accept `attention_type` parameter and instantiate the appropriate attention module.
- **Minimalism Pass:** Remove the second LayerNorm in each block (pre-normalization only) and reduce the FFN hidden dimension ratio from 4:1 to 2:1, as suggested by the paper.

### 2. Swarm Memory Module (`qiae/memory/hierarchical_memory.py`)
- **Modification:** Replace the standard cross-attention between query and memory slots with a sub-quadratic variant.
- **Impact:** Enables the memory module to scale to 10x more memory slots (from 1M to 10M) with the same compute budget.

### 3. Configuration & Registry (`qiae/configs/model_config.yaml`)
- **New Fields:**
  ```yaml
  transformer:
    attention_type: "linear"  # Options: standard, linear, flash, mamba
    minimal_mode: true        # Enables the architectural reduction pass
    ff_multiplier: 2          # Default 4, reduced to 2 when minimal_mode=True
  ```

### 4. Training Pipeline (`qiae/training/trainer.py`)
- **Modification:** Add a `--attention-type` CLI argument and pass it to the model constructor.
- **Logging:** Record attention type and minimal mode flag in experiment tracking (Weights & Biases).

## Verification Plan

### Unit Tests
- **Test 1:** `test_sub_quadratic_attention_output_shape` – Verify that `SubQuadraticAttention` produces the same output shape as `StandardAttention` for random inputs.
- **Test 2:** `test_minimal_transformer_forward` – Ensure the reduced transformer block runs without errors and produces non-NaN gradients.
- **Test 3:** `test_memory_scale` – Measure memory usage of the hierarchical memory module with 10M slots under linear vs. standard attention.

### Integration Tests
- **Test 4:** `test_training_loop_attention_types` – Run a 100-step training loop with each attention type (`standard`, `linear`, `flash`). Verify loss decreases monotonically for all types.
- **Test 5:** `test_minimal_mode_convergence` – Train a small model (4 layers, 256 dim) on a language modeling task for 1000 steps in both normal and minimal mode. Compare final loss values (should be within 5% of each other).

### Performance Benchmarks
- **Benchmark 1:** `benchmark_attention_speed` – Measure tokens/second for context lengths [512, 1024, 4096, 16384] with each attention type. Target: linear attention is 2x faster at 4096 context.
- **Benchmark 2:** `benchmark_memory_footprint` – Measure GPU memory usage for the full model under standard vs. minimal mode. Target: 30% reduction in peak memory.

### Rollout Strategy
1.  **Phase 1 (Experimental):** Merge behind a feature flag (`--attention-type linear`). Run on a single GPU with small batch size.
2.  **Phase 2 (Validation):** Run full training run (10k steps) on the standard benchmark dataset. Compare perplexity and throughput.
3.  **Phase 3 (Production):** If perplexity degradation < 1% and throughput improvement > 50%, set `attention_type: "linear"` as the new default in `model_config.yaml`.