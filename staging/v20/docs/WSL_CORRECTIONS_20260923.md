# CORRECTIONS — WSL reconciliation (2026-09-23)

Two findings I published earlier today are **wrong**, and both for the same reason: I scoped my
searches to the v19 repo root. The parent directory holds **three** trees:

```
C:/Users/shawn/Downloads/
  TRM-Swarm-OS-v2-main/                       (dir + zip)
  TRM-Swarm-OS-v17-Enterprise-Private-main/   (dir + zip)
  TRM-Swarm-OS-v19-Sovereign-Private-main/    (dir + zip)  <- everything I searched
```

## Correction 1 — `libswarm_kernel_ops.so` is NOT unrecoverable

`docs/WSL_STEP1_CLASSIFICATION_20260923.md` and `docs/WSL_STEP2_LIVE_COUNCIL_20260923.md` state that
the v18 workstream's reference artifact is gone and its receipts are "unfalsifiable as shipped".

It survives:

```
TRM-Swarm-OS-v17-Enterprise-Private-main/TRM-Swarm-OS-v17-Enterprise-Private-main/
    swarm_v2/core/libswarm_kernel_ops.so     112,496 B
    swarm_v2/core/swarm_kernel_ops.cpp        14,753 B   (v19's is 7,389 B)
```

It is `ELF 64-bit LSB shared object, x86-64, not stripped` and it **exports every op the v18 crate
verified against**, confirmed with `nm -D` in WSL:

```
c_kleene_quantize_avx2   c_k3_kleene_ops   c_instella_gate_topk
c_turbovec_quantize_2bit c_fold_scan       c_mamba3_turbovec_query
c_gguf_* (11 symbols)    c_instella_* (7)  c_mamba3_* c_dct2d/idct2d_avx2 c_nvme_mamba_*
```

I also confirmed the source declares them (`c_kleene_quantize_avx2`, `c_k3_kleene_ops` are in the v17
`.cpp`) — and the *artifact* exports them, which is the claim that matters.

## Correction 2 — the PHOENIX report's integration table is not aspirational

`docs/VALIDATION_PHOENIX_20260923.md` reports "three of the four native ops the integration table names
do not exist" and that the named `.so` is absent in favour of a `.dll`. That is true **of the v19 tree
only**. In v17, all of them exist — including `c_kleene_quantize_avx2`, `c_k3_kleene_ops`,
`c_turbovec_quantize_2bit` and `c_fold_scan`. The report is dated 2026-08-13/16, i.e. the **v17 era**,
and it was describing that build accurately.

The correct statement of the difference: **the v19 native C ops surface is much smaller than v17's**.
v19's `swarm_kernel_ops.cpp` is half the size and its built `libswarm_kernel_ops.dll` exports only
`c_instella_gate_topk` plus the mamba/nvme set. Whether that is a regression or a deliberate migration
into the Rust kernel is not established here — it is an observed difference, not a verdict.

## And the claim itself now verifies

With the real reference available, the v18 `swarm-k3::collapse` claim was tested directly (WSL +
ctypes, `verify_k3_v18.py`):

```
eps=0.35  rc=0 | compared 4030 trits (full length True) | differences: 0 | C counts sum to n: True ([1322, 1440, 1268])
eps=0.5   rc=0 | compared 4030 trits (full length True) | differences: 0 | C counts sum to n: True ([986, 2050, 994])
eps=0.25  rc=0 | compared 4030 trits (full length True) | differences: 0 | C counts sum to n: True ([1516, 1025, 1489])
VERDICT: IDENTICAL
```

The C rule from the v17 source is symmetric (`v > eps → +1`, `v < -eps → −1`, else 0) and matches the
Rust exactly, with the C's pass/analyze/block counts matching the Rust distribution element-for-element.

Note on my own first attempt at this test: it reported IDENTICAL on a **truncated prefix**. The 5th
parameter is `int* out_counts`, a three-element array (pass/analyze/block) — not an output length. I
passed a single int, the C wrote out of bounds, and `pass` read back as a plausible "output size". A
full-length comparison and a counts-sum-to-n assertion caught it.

## The lesson worth keeping

**"Absent" in a version-scoped search is not absent from the platform.** The same trap this campaign hit
with `/tmp`-vs-Windows paths and with WSL-vs-Windows filesystems now appeared as version ancestry: the
artifact was one directory up, in the tree the workstream actually built against. Scan the platform's
ancestry — sibling trees, older releases — before declaring an artifact missing.
