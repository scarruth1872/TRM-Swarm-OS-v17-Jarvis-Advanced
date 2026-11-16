# Audit: "Infinity Cache as Ternary Compute Fabric" (draft dated Aug 10, 2026)

**Verdict: the paper is not publishable and its central claim is not physically realizable.
The arithmetic does not survive its own abstract, and the mechanism it names does not exist.**

This is an audit of the draft, not a deletion. Every number below is recomputed from the
paper's own figures or checked against this repository's measured records.

---

## 1. The arithmetic fails before any hardware argument

| Paper claim | Recomputed | Result |
|---|---|---|
| 96MB = 805,306,368 SRAM cells | 96 MiB = 100,663,296 B = **805,306,368 bits** | ✅ consistent |
| 6T/cell = 4.83B transistors | 805,306,368 × 6 = **4,831,838,208** | ✅ consistent |
| 2-bit/trit → 402,653,184 trits | 805,306,368 / 2 = **402,653,184** | ✅ consistent |
| CA: "100,663,296 cells in an 8,192×8,192 grid" | 8192² = **67,108,864** | ❌ **wrong by 33,554,432** |

The CA figure is the **byte count of 96 MiB** (100,663,296), reused as if it were a cell
count. A grid that size holds 67.1M cells, not 100.7M. This is the signature of a number
assembled from constants rather than derived from the structure it describes.

## 2. The pharmaceutical claim contradicts the paper's own throughput ceiling

    genome screen: 200,000,000 pairs in 190 µs = 1,052,631,578,947 predictions/s

That is **1.05 trillion predictions per second**. At the optimistic 10 ops/prediction it
needs 10.5 Tera-ops/s; at a realistic 100 it needs **105.3 Tera-ops/s**.

    the paper's own stated ceiling            = 0.845 Tera-ops/s

So the application outruns the machine the same paper describes by **~125×** (at 100
ops/prediction), and by ~1,250× at 1,000 ops/prediction. A paper cannot be internally
consistent while its headline workload requires 125× more compute than its headline
throughput provides.

## 3. The cost comparison is off by four orders of magnitude

p4d.24xlarge is roughly **$32.77/hour**.

| Duration | Cost |
|---|---|
| 16 min | $8.74 |
| 1 hour | $32.77 |
| 10 hours | $327.70 |

The paper claims **~$5,000,000** for this run. $5M buys **152,579 hours ≈ 6,357 days** of a
p4d.24xlarge — about 17 years of continuous rental. Meanwhile the claimed "5,000,000× faster"
implies the cloud run took `190 µs × 5e6 = 950 s ≈ 15.8 min`, which costs **$8.74**.

The paper's speed claim and its cost claim describe two different universes: one where the
cloud run takes 16 minutes, and one where it costs 17 years of GPU time.

## 4. The mechanism is not real: the Infinity Cache is not software-addressable

This is the part that no arithmetic fixes. On RDNA 2, the 96 MB Infinity Cache is a
**hardware-managed cache** behind the L2 slices. There is no load/store path to it, no
software-visible address space, no pin operation, and no coherence protocol a program can
drive. Lines are placed by a hardware hash and evicted by hardware policy.

Consequences the paper depends on, none of which hold:

* "**writing specific bit patterns to cache lines**" — you cannot address a cache line. You
  write to a memory address; whether the line lives in the Infinity Cache is the
  hardware's decision, and it may be evicted immediately.
* "**exploiting the cache's native coherence protocol**" — L2/Infinity Cache in a discrete
  GPU is not a coherence domain in the CPU sense; no such programmable protocol exists.
* "**executing S_BITCMP_B32 natively**" — computation happens in the CUs, never *in* the
  cache. The cache is bandwidth and latency, not an execution substrate.
* "**4 virtual architectures switchable in under 10 µs through pattern writes**" — with no
  persistent, addressable storage, there is nothing to switch.

**`S_BITCMP_B32` does not appear in this repository as executable code.** It appears in
`trm_uefi_kernel/host_batchA/ref_values.json` and in three comments inside
`gpu_infinity_computer.rs`. The GPU path this platform actually runs is Vulkan
(`ggml-vulkan.dll`). The instruction the entire performance claim rests on is a string in
a comment.

*Limitation of this audit:* I could not verify the RDNA 2 ISA name list online — the
environment's web-search provider is unregistered (`duckduckgo` not available). So the
"no such instruction" conclusion rests on local evidence (not implemented anywhere here)
plus the architecture argument above, not on an ISA reference table.

## 5. This repository already retracted a related figure

`docs/NOBEL-PAPER-2026-01-REV.md` states:

> "Operating on an AMD Radeon RX 6700 XT, Geometric Resonance achieves **70,606,060
> measured agent-tasks/sec** ... The 154.5T ops/s figure in the original paper was **a
> projection, not a measurement — it is retracted.**"

Against that **70.6M tasks/s measured** figure, this draft's **845B ops/s** is **~11,968×
higher**. The 154.5T projection was already withdrawn once for exactly this reason; the
845B number in this draft is the same error at a different magnitude, and it carries no
measurement behind it.

`docs/JOURNAL.md` lists the paper as `PREPRINT` with the single line "845B ternary ops/s via
S_BITCMP_B32" — a throughput claim with no gate, no bench, and no artifact.

## 6. What is actually true and defensible

* The RX 6700 XT does have 96 MB of Infinity Cache — the die area and cell counts in §1
  check out.
* A cache **does** help bandwidth-bound ternary/bitwise work: it raises effective
  throughput by reducing memory latency and traffic. That is a real, measurable effect.
* The honest framing is the one already in `NOBEL-PAPER-2026-01-REV.md`: a measured
  **70.6M agent-tasks/s** on this card, with peak SIMD acknowledged as hardware-bound.

## 7. Required corrections before this draft could go anywhere

1. Delete the 845B ops/s claim or replace it with a reproduced bench artifact (script,
   invocation, raw output) — the standard `docs/NOBEL-PAPER-2026-01-REV.md` already set.
2. Delete "190 µs / 200M pairs / 5,000,000× / $5,000,000" unless a timing harness and a
   cost derivation are attached. As written they are mutually inconsistent (§2, §3).
3. Remove "executing S_BITCMP_B32 natively" and "pattern writes to cache lines", or supply
   the ISA document and a driver-level demonstration that such an instruction and such
   addressing exist.
4. Fix the CA cell count (67,108,864) or change the grid.
5. Retitle: the cache is a **bandwidth substrate**, not a compute fabric. "Cache-assisted
   ternary throughput on commodity RDNA 2" would be supportable.

---

**Status:** claim not adopted. No platform capability is claimed on the basis of this draft.
Recorded rather than deleted, per the errata rule.
