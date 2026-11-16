# GPU compute path for the agent roster — measured scope

Status: **steps 1-5 built and parity-gated** (`trm_uefi_kernel/tools/gpu_ssm_parity`). Measured
result: full head on one device, one submit, 0 decision flips vs `K3Head::forward_logits`,
**3.73x on 3,840 items** (29.8 ms vs 111.2 ms) and **0.91x on a single text** (891 us vs 815 us).
See G56-G59 in `PHASE-6-GAPS.md`. Every number below came from probe runs on this machine.

## 1. What already exists (inventory, not assumption)

| item | state |
|---|---|
| Vulkan loader | `C:\Windows\System32\vulkan-1.dll` present (also SysWOW64) |
| GPU | **AMD Radeon RX 6700 XT**, Vulkan, `DiscreteGpu` — enumerated by wgpu |
| Infinity Cache | 96 MB on Navi 22. The f32 roster (29.87 MB) fits **3.3x over** |
| Rust crates | `ash 0.38`, `wgpu 30.0.1`, `spirv`, `gpu-allocator 0.28` all in the cargo registry cache |
| Existing wgpu crate | `staging/v20/wsl_crates/trm-rust/swarm-deepdta-gpu` — wgpu + 9.4 KB WGSL, 5-dispatch conv+MLP pipeline, sha256-gated weights, parity harness |
| Its recorded result | 65.57x speedup — **but the log's adapter is "NVIDIA GeForce RTX 3070 Laptop GPU"**, not this box. Unverifiable here: its model bin (`deepdta_mpro_ft.bin`) and SMILES input are both missing from disk |
| Vulkan compute in the kernel tree | **none** — `host_fanout/src/gpu_infinity_computer.rs` is a mechanical port of a Python calculator that returns cell counts (`cells = 96*1024*1024*8`), not a compute path |

So the crates and the runtime were set up; what does not exist is a kernel for *our* workload, and the one
precedent measurement came from someone else's laptop.

## 2. What the probe measured

`gpu_probe` (wgpu 30.0.1, Vulkan backend, release) dispatches a row-per-invocation matvec
(`y[r] = Σ_j w[r*96+j] · x[j]`), 60 timed round trips per shape, median reported:

```
adapter                 AMD Radeon RX 6700 XT | Vulkan | DiscreteGpu
one head, submit+read  134.4 us
15 heads, one submit   204.9 us
30 MB weight upload     34,942 us   (35 ms, one-time; write_buffer + poll per pass)
throughput mode        183.5 us per roster dispatch, 200 queued behind one sync
```

**The constraint is dispatch overhead, not arithmetic.** A roster dispatch is 92,160 MACs
(~0.18 MFLOP); 200 of them in 36.7 ms is ~0.5 GFLOP/s on a 13 TFLOPS card. The GPU is idle and the
driver is the bottleneck.

Against the CPU, on the same shapes:

```
latency     one head: CPU 41.6 us (measured, /k3/classify in-process)  vs  GPU 134.4 us  -> CPU wins 3.2x
throughput  15-head roster: CPU ~624 us (15 x 41.6)                     vs  GPU 183.5 us  -> GPU wins 3.4x
```

That is the whole design consequence: **never dispatch one head for one vote.** Per-vote GPU is a
regression. GPU only pays when many heads — or many texts — share one submission.

Honest caveat: the probe's kernel is the core matvec only (0.18 MFLOP), while the CPU 41.6 us figure
is a whole head (522k params, embedding table included). The throughput comparison is therefore
favourable to the GPU by construction and should be treated as an upper bound until the kernels are
real.

## 3. The roster forward pass, as kernels

One head (`K3Head::forward_logits`) is 7 matmul stages plus 3 elementwise stages:

| # | stage | shape | kernel |
|---|---|---|---|
| 1 | `dh = x·Bᵀ` | 96 -> 64 | GEMM, weights resident |
| 2 | `h = 0.05·dh` | 64 | elementwise scale |
| 3 | `bonsai = h·Cᵀ + x·Dᵀ` | 64 -> 96, 96 -> 96 | two GEMMs + add |
| 4 | MoE gate | 96 -> 4, softmax | GEMM + softmax |
| 5 | top-2 experts | 96 -> 64 -> 96 | 2x GEMM + ReLU |
| 6 | combine, ReLU, out projection | 96 -> 96 | add/scale + GEMM |
| 7 | residual + LayerNorm | 96 | reduction + elementwise |
| 8 | classifier | 96 -> 64 -> 3 , ReLU | 2x GEMM + argmax |

Plus the embedding bag (`4096 x 96` gather-mean) which is 75% of the parameters and is a pure gather.

**Layout decision: batch by TEXT, not by head.** All 15 heads share dim=96, and grid
`(rows = heads x out_features, y = texts)` lets every head and every text share one dispatch per
stage. Stage count per batch: 8, plus 1 for the embedding bag. Weights upload once (35 ms) and stay
resident — 29.87 MB inside 96 MB of Infinity Cache.

## 4. Build order (each step measurable, none speculative)

1. **Port stage 1-3 only** (the SSM block) and check bit-parity against `K3Head::forward_logits` on
   the 20 probes already in `/k3/ternary/bench`. Gate: max abs diff < 1e-5, and no decision flips.
2. **Add stages 4-8** for one head, same parity gate.
3. **Batch 15 heads x 1 text in one submission.** Gate: total < 183 us (the measured dispatch bound)
   and decisions identical to CPU for all 15 heads.
4. **Batch 15 heads x 256 texts.** This is where the GPU wins: the eval harness currently scores
   corpora through HTTP one text at a time.
5. **Only then** consider the live vote, and only if a kernel path can beat 41.6 us end to end —
   which requires cutting the 134 us dispatch round trip first (persistent command buffer, no
   readback per vote, or a resident compute queue).

## 5. Risks, stated up front

- **AMD Windows dispatch overhead.** 134-205 us is the wall. If it cannot be cut, GPU stays a
  bulk-scoring accelerator and never a vote path. This must be measured before any kernel work.
- **Readback cost.** Every vote needs a trit back on the CPU; the probe's 134 us includes
  `map_async` + `poll`. A persistent mapped buffer or a device-side accumulator would avoid it.
- **Precision.** GPU f32 vs CPU f32 differ in accumulation order; the DeepDTA precedent showed
  `max_abs 3.3e-6` and `4590/16384` bit-exact, which flipped no decisions there. The K3 heads are
  bell-shaped near ties, so parity must be measured on decisions, not just values.
- **No Vulkan compute in the kernel tree yet**, and the only precedent measurement came from
  different hardware. Both are reasons to build the smallest kernel first and measure.
- **The ternary path is not an alternative here.** Post-hoc 1.58-bit quantisation scored 0.51
  relative RMS error and 35/60 decision parity (G53). The Infinity Cache is big enough that
  quantisation buys nothing — 29.87 MB already fits.

## 6. Recommendation

Build steps 1-3 as a **parity instrument**, not a deployment: it proves the kernels are right and
establishes whether 183 us can be beaten with realistic batching. Keep the live vote on the CPU
(41.6 us, no readback, no driver in the loop). The measurable win available today is bulk corpus
scoring — thousands of directives x 15 heads — where the harness is currently the slow part.
