# CHECKPOINT 2026-11-04 — Scorers evaluated, GPU wired, voltage binder in the loop

Council motion **NEXT-STEPS-01**: 13 approve / 2 abstain / 0 reject, consensus.

## 1. What the platform can do, measured

| surface | route / artifact | measured |
|---|---|---|
| DeepDTA screen (CPU, in-kernel) | `POST /crate/screen/smiles` | 448 mol/s |
| Docking screen (in-kernel, 16 threads) | `POST /crate/screen/library` | 1,001 mol/s |
| DeepDTA screen (GPU, standalone crate) | `swarm-deepdta-gpu` | **27,370–27,606 mol/s** |
| Full ChEMBL 37 library | `/mnt/d/chembl_smiles_full.txt` | 2,897,805 molecules, 203.6 MB |
| Full library on GPU | 2.9M in **104.97 s** | parity max_abs 3.338e-6 |
| Packed-ternary fabric | `scratch/fabric_probe.json` | 46.4 MB, 1,055 q/s @2.9M |
| Voltage binder | `task_result.silicon_binding` | live nvidia-smi per dispatch |

## 2. Scorer ladder (this is the load-bearing result)

| scorer | dataset | n | Pearson | Spearman | verdict |
|---|---|---|---|---|---|
| `swarm-docking` | BDB2020+ measured Kd/Ki | 115 | −0.131 (p=0.163) | −0.0996 (p=0.289) | **no ranking** |
| `swarm-deepdta` | LP-PDBBind held-out test | 500 | 0.393 | 0.335 | real, moderate |
| `swarm-deepdta` | size-controlled partial r | 500 | **0.395** | — | not a size proxy |
| cross-scorer | 600 molecules, both scorers | 600 | 0.375 | 0.424 | **size artifact** (partial r = 0.079) |

Docking: slope −0.2704 against an ideal −1.364, mean bias +3.55 kcal/mol, both CIs straddle zero.
Robustness slices (no clashes n=86 r=−0.32; ligands ≤35 heavy n=94 r=−0.10) do not rescue it.
DeepDTA: a random sample first measured 0.696 — that was **train leakage**; the held-out number is 0.393.

**Consequence:** every docking "hit" produced before this checkpoint is not evidence of binding.

## 3. Changes landed (all pushed to `private-sovereign/TRM-Swarm-v2.0`)

| commit | change |
|---|---|
| `4286592` | retire severity-1 security/safety claims (MITM PROVED SECURE, seals, smoke PASS) |
| `95de206` | provenance trace; activate the real docking counterpart on the measured 1M17 pocket |
| `8c88124` | real ChEMBL library replaces poses of one scaffold |
| `8bd9959` | screen in the microkernel; protein branch cached (81×) |
| `aacb98d` | 3D docking screen in the microkernel, 16 threads |
| `735cd02` | score the scorers; docking has no signal, DeepDTA 0.696 was leakage |
| `a2752498` | voltage binder in the dispatch loop; GPU keywords; GPU DeepDTA crate |

## 4. Voltage binder — what was wrong and what is true now

It existed, it was honest, and **it was not in the loop**: `execute()` never called it. It now returns
`silicon_binding` on every task. The first live dispatch classified GPU docking work as `LOGIC` on the
CPU rail; the classifier had no GPU keyword. Added gpu/wgpu/vulkan/cuda/docking/vina/screen/deepdta/
molecular/ligand/inference to `GENERATIVE`.

Verified live (card `card-57546f08`): class GENERATIVE, rail "GPU V_core (not measured on host)",
region "Compute Unit SIMD Vector Lanes & L2 Cache Slice", telemetry 15.27 W / 210 MHz / 49 °C / 29% util.
Rail voltage stays `0.0 / not measured` — nvidia-smi does not expose core voltage, and the binder does
not invent it.

## 5. Next steps (council NEXT-STEPS-01, 13/2/0) — all four DONE

1. **Scaffold-split validation — DONE.** 3,537 of 4,834 test rows have a scaffold absent from train
   (1,297, 26.8%, share one). 500 novel-scaffold molecules: **Spearman 0.3478, Pearson 0.3946, n=500,
   t=8.28**. Essentially the random-split 0.335/0.393, so analogue leakage does not explain the signal.
2. **GPU in the kernel — DONE.** `/crate/screen/smiles` takes `{"gpu": true}` and spawns the native
   Rust binary (no HTTP hop). Measured through the live kernel: 5,000 molecules at **27,722 mol/s** vs
   530 CPU (**52×**), parity max_abs 3.8e-6, over_tol 0.
3. **Fabric Tanimoto — DONE.** wgpu popcount kernel over the 2-bit store: **1,889 q/s at 2.9M**
   (0.53 ms/query), 3,202 q/s at 10k, **recall@10 1.0000** (was 0.658 with cosine), 12.1× the CPU exact
   scan (156 q/s). Exactness: ranked indices 100/100 identical, max |ΔT| 5.96e-8 (1 ULP). Shortlist
   width B=32 already reaches recall 1.0; 0 store-vs-CSV mismatches over 2,897,818 rows.
4. **Docking retired — DONE.** `/crate/screen/library` returns **409 retired_no_ranking_signal**,
   naming Pearson −0.131 (n=115, CI [−0.307, 0.053]). Runs only with `{"force": true}` for audit.

## 6. Standing caveats

- No molecule in any run is shown to bind.
- Only 1M17 has a measured pocket; the other seven screening targets are hand-authored.
- The GPU full-library parity covers a 4,096-molecule prefix, not all 2.9M.
- Docking's `rotatable_bonds` is fixed at 5 per molecule — a declared assumption.
- The fabric's 1.000 recall is against cosine, not against the chemistry metric.


## 7. Chemistry storage and similarity (added later the same day)

**Encoding.** A 2048-bit Morgan (radius 2) fingerprint carries a median 46 set bits (mean 52.53 over the
full library, 2.5% density). Measured sizes for 2,897,805 molecules:

| encoding | bytes | MB | B/row |
|---|---|---|---|
| sparse u8 delta | 158,742,678 | **158.74** | **54.78** |
| sparse u16 delta | 310,222,530 | 310.22 | 107.05 |
| raw bitset | 741,838,080 | 741.84 | 256.0 |
| turbovec 2-bit | 1,483,676,160 | 1,483.68 | 512.0 |

turbovec packs **2 bits per element**, so on a binary fingerprint it doubles the size. The win is
sparsity, not packing. Round-trip verified bit-exact on 80,000 rows (20,000 per chunk, 0 mismatches).

**Voltage strip** (dropping low-document-frequency bits) trades badly: df>=1% saves 13% of bytes and
costs 12% of recall; df>=5% halves the store and loses a third of the chemistry. Only the lossless
encoding is adopted.

**Similarity, measured.**

| path | q/s | exact? |
|---|---|---|
| CPU exact, 16 threads | 31.4 | yes |
| GPU exact, naive readback | 28.4 | yes |
| GPU exact, block-max shortlist | **83.3** | yes, certified |
| GPU exact, fused per-block top-10 | 72.6 | yes, but slower |
| 64-bit OR-fold funnel | 605 (B=256) | **no — recall 0.845** |

The GPU ceiling is ~85 q/s because each host round trip measures ~5.9 ms and the block-max path needs
two. The fused variant was built specifically to remove one and was slower: ten barrier-heavy
reductions per block cost more than the round trip they save.

**Rejected on measurement:** the 64-bit sketch funnel (recall never reaches 1.0 even at 9% of the
library) and the fused GPU kernel. **Adopted:** sparse delta storage, block-max GPU scan, DeepDTA for
ranking, docking retired.
