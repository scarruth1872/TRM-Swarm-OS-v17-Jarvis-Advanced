# TRM Swarm OS v19 "Sovereign" (Private) — source document

Source document for NotebookLM. Compiled **2026-10-14** from the repository at
`C:/Users/shawn/Downloads/TRM-Swarm-OS-v19-Sovereign-Private-main/TRM-Swarm-OS-v19-Sovereign-Private-main`.
Every figure below is traceable to a file in that tree. This is an update of the 2026-10-03 source
document. It adds: the **full-Rust GPU compute path** (wgpu + naga), the **full-duplex ring IPC**, the
**corpus-scale TRM-7M scoring run**, and the **completed five-domain recall surface** (fabric + ChEMBL +
structured-text + bioassay + clinical). It lands at monad ledger **363 blocks** and a council of **22
enacted motions**.

## What this system is

A Rust HTTP platform — `trm_uefi_kernel/host_council_server`, a "sovereign" host-council server /
microkernel — plus a bare-metal Linux pathway that boots it from a USB drive on real hardware. Two
listeners, defaults read out of `trm_uefi_kernel/host_council_server/src/main.rs`:

| thing | port | default in source |
|---|---|---|
| API | `8022` | `TRM_PORT` unset → `8022` |
| LCARS dashboard | `5183`, served at `/ui/` | `TRM_UI_PORT` unset → `5183` |

The logic anchor is the Monad Truth Anchor `0x4D4F4E4144000001`. The kernel returns key **names** only —
never values — and no secret material travels over HTTP. A supervisor (`swarm_supervisor.py` in this
checkout, or `host_supervisor` on the metal image) starts the platform and respawns it.

The platform's knowledge is a **token-free** substrate. The runtime model is **TRM-7M**, an in-kernel
SSM — "the CPU is the model": a 462-dim feature vector hashed (FNV-1a) to 128 dims, a Kleene-trit state,
3.9M parameters, 15.6 MB of weights, ~4M FLOPs per forward. The GGUF checkpoints in the tree (Qwen3.5-hybrid
/Bonsai, Instella, qwen3moe) are **distillation teachers**, not the serving model.

## The compute path is now full-Rust GPU

The voltage binder's GPU edge was realized with **wgpu 30.0.1 (Vulkan) + naga** — pure Rust, WGSL compiled
to SPIR-V at runtime. **No CUDA/Vulkan SDK is ingested**; the SDK ingestion remains an open research item.
Four parity-verified kernels live in `trm_uefi_kernel/host_council_server/src/gpu_compute.rs`:

| kernel | parity vs | result |
|---|---|---|
| `hdc_encode` (single + batch) | native DLL `rust_native_encode_text_f32` | bit-exact, worst `0.00e+00` |
| `packed_scan` (2-bit packed store) | `cpu_scan_in` reference | ~35× e2e (107 ms → ~3 ms cached) |
| `trm7m` (batch forward) | CPU `forward` | trit exact, confidence ≤1e-4 |

`gpu_dispatch.rs` binds four kinds: `hdc_encode` / `packed_scan` / `trm7m` → **gpu**; `sram_compute` → **cpu**
(there is no literal SRAM hardware on this host — the binding is fail-closed honest, not a performance claim).

The GPU device limits were raised for the ChEMBL store: the packed 2-bit codes buffer is **556 MB**, which
exceeds the wgpu downlevel default `max_buffer_size` (256 MB); the limit was raised to **1 GiB**. The RTX 3070
Laptop GPU / Vulkan trivially supports this, and `request_device` fail-closes to a CPU scan on rejection.

## The recall surface — five knowledge domains

The recall router (`recall_router.rs`) classifies a query into one of five stores and dispatches it. Four
are **packed-domain GPU scans** (each with its own GPU cache + a GPU-vs-CPU parity receipt); the fifth is a
**lexical lookup** (it is tabular data, and a vector scan would fake relevance).

| domain | route | store | vectors | size | scan |
|---|---|---|---|---|---|
| general fabric | `general` | `swarm_v2_nand_fabric.turbovec` | 183,879 | 44 MB | GPU |
| molecules | `molecule` | `swarm_v2_chembl_fabric.turbovec` | 2,897,805 | 648 MB | GPU |
| structured text | `structured` | `swarm_v2_structured_text.turbovec` | 97,480 | 20 MB | GPU |
| bioassay | `bioassay` | `swarm_v2_bioassay.turbovec` | 597,430 | 129 MB | GPU |
| clinical | `clinical` | `swarm_v2_artifacts/clinical_catalog.json` | 40 measurements | ~4 KB | lexical |

- **ChEMBL**: 2,897,805 compound vectors, 2-bit, dim 768, row 200 B. A non-ID molecule query scans all
  2.9M in **~256 ms cached** (first query ~5.5 s for store load + 556 MB upload). An exact ChEMBL ID
  (`CHEMBL153534`) short-circuits lexically (1 record) — the vector scan is only for queries that are not IDs.
- **Structured text**: the ethics (95,847) + prose (1,633) free-text rows of `structured_index.db`, extracted
  logging-first (0 skips) and packed with the native DLL (bit-exact). Scanned at ~23 ms cached.
- **Bioassay**: tox21 (577,987) + binding (19,443) = 597,430 SMILES/assay rows. Parity 3.10e-7 vs CPU.
- **Clinical**: 201,572 ICU-monitoring rows over only **40 distinct lab codes** (HR, AST, Creatinine, Glucose,
  …). It is tabular — numeric readings per patient × timepoint — so it is served by a **lexical measurement
  lookup**: a query names a measurement (case-insensitive **exact** match — an explicit, build-tested alias map
  first, then the 40 codes, **no fuzzy fallback**), and the kernel returns count/min/max/mean. The sidecar
  carries a **sum check**: the per-code counts must sum to 201,572; on mismatch the receipt says so.

Every one of the four GPU scans has a **parity receipt**: GPU vs CPU `cpu_scan_in` over top-K, 0 key
mismatches, max score diff within 1e-3 (ChEMBL 9.71e-7, structured 1.03e-6, bioassay 3.10e-7, fabric bit-level).

## Corpus-scale TRM-7M scoring (token-free, no HTTP)

The whole general corpus was scored through TRM-7M **in-kernel** — `host_council_server score-corpus` reads the
text sidecar in-process and dispatches batches to the GPU trm7m kernel. No HTTP on the hot path.

- **183,879 docs scored in 123.589 s = 1487.8 docs/s**, backend `gpu`, 0 drops.
- Trit distribution: +1 → 136,396, 0 → 29,860, −1 → 17,623 (sums to 183,879); average confidence 0.399.
- Output: `swarm_v2_artifacts/corpus_trm7m_scores.jsonl`.
- These scores are wired as a **model tier**: every general-path recall result is annotated with its
  `trm7m_trit` + `confidence` (lazy file-IPC load, deterministic key lookup, fail-closed to `"unscored"`).
  Re-ranking/filtering on that tier are open follow-ups.

The scoring source is the TurboVec text sidecar `swarm_v2_nand_fabric.text.jsonl` (146 MB), **not** the 4.2 GB
`semantic_index.json` — the packed store is the corpus of record.

## Full-duplex IPC

`mailbox_ring::push` (a 64-byte ABI) added the **write** path to the ring bus, completing the full-duplex
transport. `evaluate_proposal` publishes council verdicts (Orchestra → EventLedger) over it. The cross-language
round-trip is verified: the Python `SwarmMailboxBus` reads the exact message the Rust kernel wrote. Retrieval,
answer, and scoring hot paths use ring/file IPC — **never HTTP**.

## Council and ledger

- **Council: 22 motions, 22 enacted** — every one 12 approve / 3 abstain / 0 reject, weighted 0.685. The
  abstaining trio is always **Verify / Shield / LogicMaster**, whose abstentions carry binding conditions
  (measured proof, no new surface / no secrets / no network egress, deterministic routing).
- **Monad ledger: 363 blocks**, hash-chained (`canon()` excludes the `hash` field; each block's evidence is
  re-measured by its sealer). Ledger blocks 92–101 are a pre-existing, documented unreproducibility; all
  current blocks reproduce and link.
- MOTION-018 through MOTION-022 in this span: voltage-complete plan (018), trm7m scores → recall (019), ChEMBL
  scan (020), structured-text tier (021), clinical lexical lookup (022).

## Honest boundaries (measured, not claimed)

- **trm7m CPU single-eval is the naive baseline** (per-weight `read_unaligned`, ~1.6 GFLOP/s, 2.82 ms/call).
  A SIMD port (aligned loads) makes LLVM vectorize all 18 matvec loops at once and OOMs the optimizer; the
  batched GPU kernel is the scale path, so the CPU path stays naive on purpose.
- **`sram_compute` → cpu**: no literal SRAM hardware; the binding is fail-closed honest.
- **Bonsai/Instella token-level parity** is confirmed for Bonsai only; Instella MLA remains a single-token proof.
- **The release build OOMs LLVM while supervisor+kernel are running** (memory pressure ~4.8 GB). Kill both plus
  any `llama-server` before building with `-C codegen-units=1`.
- **Clinical is lexical, not vector**: a packed-domain scan over 40 numeric codes would fabricate relevance, so
  the honest serve is an exact-match lookup. This is a design decision, not a gap.

## Bare-metal pathway (unchanged since 2026-10-03 — see that doc for the full walk)

UEFI → `EFI/BOOT/BOOTX64.EFI` (trm_chainload) → `vmlinuz` → `initrd.img` (init + supervisor + supplicant +
22 kernel modules + 4 firmware files) → Linux 6.6.114.1-microsoft-standard-WSL2 → `/init` (PID 1,
`platform_init.c`) → `host_supervisor` → `host_council_server` (:8022 API, :5183 dashboard). The volume is
found by content (a `rootfs.squashfs` marker), not device name; the image is read-only squashfs + a writable
ext4 overlay. The platform resolves stores via `repo_relative()`, which walks up from the executable for a
directory holding `swarm_v2_artifacts/`.

## Key files (this checkout)

- `trm_uefi_kernel/host_council_server/src/gpu_compute.rs` — 4 kernels + 4 store caches + wgpu limits (1 GiB).
- `trm_uefi_kernel/host_council_server/src/gpu_dispatch.rs` — 4-kind dispatcher.
- `trm_uefi_kernel/host_council_server/src/packed_ternary.rs` — `cpu_scan_in` + per-store GPU fast paths.
- `trm_uefi_kernel/host_council_server/src/trm7m.rs` — the token-free SSM (naive CPU + GPU batch).
- `trm_uefi_kernel/host_council_server/src/recall_router.rs` — 5-domain classify + route + trm7m annotation.
- `trm_uefi_kernel/host_council_server/src/mailbox_ring.rs` — full-duplex ring IPC.
- `trm_uefi_kernel/host_council_server/src/main.rs` — `score-corpus` + `encode-batch` CLIs, routes.
- `trm_uefi_kernel/host_council_deliberation/src/main.rs` — the 15-agent council + 22 motions.
- `swarm_v2_artifacts/monad_ledger.jsonl` — 363 hash-chained blocks.
- Stores: `swarm_v2_nand_fabric.turbovec`, `swarm_v2_chembl_fabric.turbovec`,
  `swarm_v2_structured_text.turbovec`, `swarm_v2_bioassay.turbovec`, `swarm_v2_artifacts/clinical_catalog.json`.
