# Motion B verified: GPU collapse identity holds, the throughput claim does not (2026-09-23)

The last unverified v18 claim. Both artifacts exist in the v17 tree and the RTX 3070 is visible in WSL
(`nvidia-smi`: NVIDIA GeForce RTX 3070 Laptop GPU, driver 596.36, 8192 MiB), so this was run for real.

## Identity: VERIFIED

1,000,000 values on the v18 gate's own deterministic field
(`sin(x*0.001)*0.5 + cos(x*0.0003)*0.4`), eps = 0.35, CPU `c_kleene_quantize_avx2` vs GPU
`c_k3_collapse_gpu`:

```
CPU  rc=0  counts=[235541, 526945, 237514] sum=1000000
GPU  rc=0  counts=[235541, 526945, 237514] sum=1000000
CPU distribution: {1: 235541, 0: 526945, -1: 237514}
GPU distribution: {1: 235541, 0: 526945, -1: 237514}
byte-identical: True | differing positions: 0
```

`k3_collapse.cu`'s rule is the same one the AVX2 path uses
(`out[i] = (v > eps) ? 1 : ((v < -eps) ? -1 : 0)`), and the GPU agrees with it element for element.

## Throughput: the claim is wrong, and the reason is in the source

The v18 gate states the GPU chamber is "byte-identical to CPU, **measured 177M/s**". Measured here:

```
CPU  c_kleene_quantize_avx2   0.75 ms   1333.1 M/s
GPU  c_k3_collapse_gpu        3.24 ms    309.0 M/s      -> GPU/CPU = 0.23x
```

The GPU is **not** faster for this operation at this size - it is ~4.3x slower. The reason is visible in
`k3_collapse.cu` and needs no speculation: the host wrapper does `cudaMalloc`, a 4 MB host->device copy,
the trivial kernel, a 1 MB device->host copy, then a **serial host loop over all 1M elements** to build
the pass/analyze/block counts. At 1M elements the transfers and that counting loop dominate; the collapse
itself is not the cost. The v18 gate even computes a speedup ratio - on this host that ratio is < 1, so
its own printout would have contradicted the framing.

Caveat: their 177M/s may have been a real measurement on a different memory state (a warm context, a
resident buffer). What is established here is that the number is **not reproducible as stated**, and that
the structural reason is in the code rather than in the measurement.

## What holds either way

The GPU chamber is real: `k3_collapse.cu`, `instella_gate.cu`, `mamba3_step.cu`, `turbovec_pack.cu` and
`vram_partition_manager.cu` are genuine CUDA sources, `libk3_gpu.so` exports `c_k3_collapse_gpu`, and the
identity claim is exact. "Byte-identical" is true. "Faster" is not - for this op, at this size, on this host.
