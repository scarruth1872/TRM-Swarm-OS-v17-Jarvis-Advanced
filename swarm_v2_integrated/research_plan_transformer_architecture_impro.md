
# Research Integration Plan: transformer architecture improvements
## Overview
This research enables the QIAE Swarm to replace its monolithic O(n²) attention mechanism with a configurable, multi-backend architecture supporting sub-quadratic attention (e.g., linear attention, ReLU attention), state space models (e.g., Mamba), and hybrid sparse-attention/SSM pipelines. Benefits include: 3-10x reduction in memory usage for long sequences, ability to process 100K+ token contexts on consumer hardware, and dynamic routing of reasoning vs. streaming tasks to the optimal backend.

## Proposed Changes
- **`swarm/core/attention.py`** — Refactor into abstract `AttentionBackend` base class; implement `StandardAttention`, `LinearAttention`, `ReLULinearAttention`, and `MambaSSM` subclasses.
- **`swarm/core/config.py`** — Add `attention_backend` field (enum: `standard`, `linear`, `relu_linear`, `mamba`, `hybrid`) and `hybrid_threshold` (int, token count for switching).
- **`swarm/core/transformer_block.py`** — Modify to accept `AttentionBackend` instance; add `HybridTransformerBlock` that delegates to attention or SSM based on sequence length.
- **`swarm/routing/message_router.py`** — Integrate backend selection into message processing pipeline; log backend used per message.
- **`tests/test_attention_backends.py`** — Unit tests for each backend with correctness and performance benchmarks.
- **`docs/architecture/attention_backends.md`** — Documentation of the new architecture and selection criteria.

## Verification Plan
1. **Unit Tests**: Verify output shape and numerical stability for each backend on random inputs (seq_len=128, 1024, 8192).
2. **Performance Benchmarks**: Measure throughput (tokens/sec) and peak memory (MB) for each backend at seq_len=512, 4096, 32768.
3. **Integration Test**: Run full Swarm message routing pipeline with each backend; confirm identical routing decisions (deterministic seed) and <5% accuracy deviation on a test corpus.
4. **Regression Test**: Ensure existing standard attention path produces identical results to current implementation (bit-exact comparison).
5. **Stress Test**: Process a 100K-token synthetic conversation with `hybrid` backend; verify memory stays below 16GB and latency < 30s.