# 2-bit packing of the K3 heads: measured, and it fails the gate

**2026-11-24.** Pre-registered expectation before running: packed to 2 bits an agent is ~130 KB,
L2-resident, and votes get ≥3× faster from bandwidth alone, with verdicts unchanged.

**Both halves of that are now measured. The speed premise was wrong and the packing destroys verdicts.**

## What was done, in proof order

1. **Parsed the real blob** (`k3_heads.bin`, K3HD v2): 15 heads, 31.32 MB, **2.088 MB per head**,
   `dim=96, state=16, experts=8, buckets=4096`. The doc comment in `k3_head.rs` says 128,811 params;
   the actual head is **522,033** because that comment omits a 4096×96 embedding table that is 75% of
   the weights. (The source's own `params()` notes this trap; the comment above it is stale.)
2. **Re-implemented the forward pass in numpy** and required it to reproduce the live server:
   **168/168 trits match (100.0%)** across 12 texts × 14 agents. So the harness measures the real model.
3. **Quantized to 2-bit ternary** (`|w| > t → sign`, else 0), biases/LayerNorm left f32, and compared
   trits against the dense f32 model on the same 168 pairs.

## Result

| configuration | threshold | trit agreement | size/agent | compression |
|---|---|---|---|---|
| core ternarized, emb f32 | 0.5 / 0.7 / 1.0 × mean\|w\| | **61/168 = 36.3%** | 0.038 MB | 54.5× |
| everything incl. emb ternarized | 0.5 | **76/168 = 45.2%** | 0.137 MB | 15.3× |
| " | 0.7 | 67/168 = 39.9% | 0.137 MB | 15.3× |
| " | 1.0 | 73/168 = 43.5% | 0.137 MB | 15.3× |

Three-class agreement at 36–45 % is barely above chance (33 %). The weights are load-bearing: this is a
dense model, and rounding it to trits after training does not produce a ternary model — it produces a
different, worse model. Contrast the earlier expectation of "identical verdicts": **refuted**.

The honest way to get a 2-bit agent is **ternary-aware training** (weights constrained to {−1,0,1}
during training, with a learned scale), not post-hoc rounding.

## The cache premise was also wrong

The 2.088 MB per head is mostly **cold**: a vote touches only the embedding rows its text hashes to and
the **top-2 of 8 experts**.

    per-vote working set (f32): 53 KB emb rows (142 unique of 192) + 96 KB for the 2 chosen experts
                              + 112 KB core = 261 KB      [L2 = 512 KB/core]

**261 KB already fits in L2.** So the agents' hot path is L2-resident today, and packing to 2 bits would
shrink storage 15× without moving the working set across a cache boundary — no bandwidth win to collect.

Where the time actually goes, measured by text length (median of 60 calls):

    ~5 grams  -> 44 us
    13 words  -> 50 us
    192 grams (EMB_CAP) -> 77 us

The cost scales with the number of character n-grams, i.e. the **bitwise CRC-32 featuriser**, not weight
streaming. That is the real lever: a table-driven CRC and no per-call allocation, and batching the 14
agents into one tokenise-then-gather pass (the source already has `input_with_ids` for exactly that).

## What this changes

- **Do not ship 2-bit packing.** It costs 55–64 % of the verdicts to save storage that the hot path was
  never short of.
- **The trits are already free** (60 µs, now recorded on every proposal as `k3_trit`/`k3_conf`).
- Speed work on this substrate should target the featuriser (44 → 77 µs scaling), not the weights.
- Quality remains the open problem: on the dev set the heads returned 14/14 ANALYZE at 0.88–1.000
  confidence for an ordinary documentation task, and 14/14 APPROVE for a TLS-weakening request.
  A confident head that says the same thing to everything is not a voter, packed or not.

Scripts: `scratch/k3_2bit_study.py` (parse + parity), `scratch/k3_2bit_quant.py` (quantization + sizes).
