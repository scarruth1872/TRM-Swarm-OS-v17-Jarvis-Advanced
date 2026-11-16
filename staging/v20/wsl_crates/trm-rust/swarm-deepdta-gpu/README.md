# swarm-deepdta-gpu

Standalone wgpu (Vulkan) implementation of the DeepDTA ligand branch + MLP head,
proven numerically against the CPU reference in `../swarm-deepdta` (that crate is
used as a path dependency, unmodified).

```bash
cargo run --release -- --model <bin> --smiles <tsv> --batch 1024 --limit 4096
```
Defaults: model `swarm_v2_artifacts/deepdta/deepdta_mpro_ft.bin`, smiles
`C:/Users/shawn/AppData/Local/hermes/cache/scratch/chembl_smiles.txt`,
batch 1024, limit 4096, threads = logical cores.

## Design

* The sha256 gate (`77300fe6…c7f2`) is enforced before any weight is used.
* The **protein branch runs once on the CPU** (`DeepDtaModel::protein_branch`,
  306-aa SARS-CoV-2 Mpro padded to SEQLEN=2000); its 96 pooled floats are
  uploaded once and broadcast into every molecule's concat vector.
* GPU per batch, 5 dispatches:

  | pass | workgroup | tiles | output |
  |---|---|---|---|
  | conv1 128→32 | 32×4 (128 thr) | 16 positions/wg | `[B][193][32]` |
  | conv2 32→64  | 64×4 (256 thr) | 16 positions/wg | `[B][186][64]` |
  | conv3 64→96  | 96×2 (192 thr) | 8 positions/wg  | `[B][179][96]` |
  | relu+maxpool+concat | 96 thr | 1 molecule/wg | `[B][192]` |
  | MLP 192→1024→1024→512→1 | 256 thr | 1 molecule/wg, weights streamed | `[B]` |

* Layernorm-free, channels-last like the reference: `t[(pos)*C + c]`, conv weight
  `w[(c*in_c+d)*K+k]`. Weights are **transposed at upload** to `[k][d][c]`
  (conv) and `[d][c]` (fc) so consecutive-c threads read contiguous memory.
  The per-output accumulation order (k outer, d inner; d ascending for FC) is
  unchanged, which is why the result lands within f32 round-off, not merely
  "close".
* Each thread computes 4 output positions from one weight load (register
  blocking), the input tile is staged in workgroup shared memory.
* The 4 FC matrices are packed into one storage buffer (and the biases into a
  second) to stay inside the portable 8-storage-buffer-per-stage limit.

## Measured on this host

RTX 3070 Laptop (Vulkan, driver 610.60), Ryzen-class 16-thread CPU, wgpu 30.0.1,
release build, 16 384 ChEMBL SMILES, batch 4096, protein branch cached:

```
# parity     max_abs=3.338e-6 max_rel=6.687e-7 bit_exact=4590/16384 over_tol=0
# throughput cpu(16t)=436.8 mol/s  gpu=28642.2 mol/s  batch=4096  speedup=65.57x
```

* GPU: **28 642 molecules/s** (0.572 s for 16 384), including token upload and
  score readback per batch.
* CPU: 436.8 molecules/s with 16 threads (single thread 57.5 mol/s). The
  task-reported baseline for the in-kernel 8-thread screen is 448 mol/s; the
  box is shared with other workers, so the CPU number moves between ~245 and
  ~437 mol/s run to run. Against the 448 mol/s baseline the GPU is ~64×.
* Parity over 16 384 molecules: max |Δ| = 3.34e-6 pKd, max relative 6.7e-7,
  0 molecules above 1e-4. Batch-size independent (256/1024/2048/4096 give the
  same max_abs bit for bit), so the kernels are deterministic.

Batch sweep (4096 molecules, GPU mol/s): 256 → 20 551, 512 → 23 433,
1024 → 25 346, 2048 → 27 478, 4096 → 28 266 (saturates at ≥2048).

VRAM at batch 4096: ~400 MB of intermediate buffers (conv3 output alone is
281 MB); batch 1024 needs ~150 MB and reaches 88 % of the peak rate.
