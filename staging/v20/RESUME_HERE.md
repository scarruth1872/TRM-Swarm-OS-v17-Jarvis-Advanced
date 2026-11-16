# PAUSE / RESUME NOTE (2026-11-01) — platform RUNNING; Instella parity arc CLOSED to a measured residual

Detail lives in **`docs/CHECKPOINT_20261101_INSTELLA_PARITY.md`** (read it before touching the distill).

## LATEST (2026-11-04) — scorers evaluated, GPU wired, binder in the loop

Read **`docs/CHECKPOINT_20261104_SCORERS_GPU_BINDER.md`** first.

- Docking does **not** rank measured affinity: r = -0.131, n=115, CI straddles zero. Retired from the
  screening path — `/crate/screen/library` returns 409 unless `{"force": true}`.
- DeepDTA does, moderately: **0.393** held out, **0.348 / 0.395 on a Murcko-scaffold split** (500
  novel-scaffold molecules, so not analogue leakage). Its 0.696 was train leakage.
- Full 2.9M-molecule ChEMBL library scores in **105 s** on the GPU (27,370 mol/s). The kernel route
  `/crate/screen/smiles` now runs the same GPU binary with `{"gpu": true}` (27,722 mol/s, 52x the CPU path).
- Storage: 2048-bit Morgan as sparse u8 deltas = **158.7 MB** (54.8 B/row), bit-exact on 80,000 verified
  rows. turbovec packing would be 1,483.7 MB — it doubles a binary fingerprint.
- Exact similarity: **83 q/s on the GPU** (certified exact, 2.65x the 16-thread CPU scan at 31 q/s).
  Ceiling near 85 q/s: each host round trip is ~5.9 ms and the block-max path needs two.
- Rejected on measurement: the 64-bit OR-fold funnel (19.9x at recall 0.845; recall never reaches 1.0)
  and the fused per-block top-10 kernel (72.6 q/s, slower than 83.3).
- The voltage binder was **not** in the dispatch loop; it is now, and GPU workloads route to the GPU rail.
- Council NEXT-STEPS-01 (13/2/0) is fully closed: scaffold validation, kernel GPU wiring, fabric
  Tanimoto, docking retirement.

## Bring the platform back up after reboot — ONE command

From the repo root (or double-click `start_platform.bat`):

```
trm_uefi_kernel/host_supervisor/target/release/host_supervisor.exe \
  --config trm_uefi_kernel/host_supervisor/supervisor.services.conf --interval 10 \
  --log swarm_v2_artifacts/logs/supervisor_rs.log
```

`host_supervisor.exe` (Rust) starts and self-heals:
- **:8022** the Rust kernel — `trm_uefi_kernel/host_council_server/target/release/host_council_server.exe`
  (cwd = that release dir); the SAME process binds **:5183** (dashboard). No Node dev server.
- **:8590** the semantic embedder — `llama-server --embedding` serving `embeddinggemma-2-Q8_0.gguf`
  (309 MB). Q8_0 is canonical (0.9999 raw-to-raw vs bf16). If :8590 is down, relaunch:
  `cd C:/Users/shawn/AppData/Local/hermes/cache/scratch/llama-b11457 && ./llama-server.exe -m
  C:/Users/shawn/AppData/Local/hermes/cache/scratch/embeddinggemma-2-Q8_0.gguf --host 127.0.0.1
  --port 8590 --embeddings -c 4096`.

Deliberately NOT started: the Python shard (:8021), Aura Studio (:3000), Ollama.

## State at this pause (verified 2026-11-01 12:49)

- **:8022** HEALTHY | kernel **v20.0.0** | `python_active: false` | monad_anchor `0x4D4F4E4144000001` |
  uptime ~3.1 h at check.
- **Council 15 members**, quorum 9 (f=4). Monad ledger accepting seals; last sealed this arc: **146**.
- **No listing endpoint** for the journal: only `POST /crate/journal/realize`. Anchor paths must be
  ABSOLUTE, and must not include the 9 GB gguf (hashing it times the endpoint out).

## The Instella distill — where it stands (2026-11-01)

**Acceptance, live crate** (`wsl_crates/trm-rust/swarm-dequant`, binary already built):

| metric | value |
|---|---|
| logits `mean_abs` vs fork FA | **0.090881** (arc start 6.22; mid-arc 0.2176) |
| logits `max_abs` | 0.504295 |
| top-10 tokens | **exact order** `[128804,3,14,2596,16,8567,342,43225,1,1031]` |
| greedy tokens vs **non-FA** fork | **16/16** at 16 tokens; **25/32** at 32 (first divergence index 25) |
| greedy tokens vs FA fork | 10/16 — the fork disagrees with itself |

**The fix:** llama.cpp's attention runs `ggml_flash_attn_ext`, whose builder casts k and v from F32 to
F16. The crate computed everything in f32. Three changes, all from source, no fitted parameters:
f16 rounding of `q_absorbed`/`q_pe`/`kcur`/`vcur`; ggml-exact symmetric Q8_1 (`amax/127`, f16 scale);
`kq_scale = 0.165626` (= `mscale^2/sqrt(128)`, `mscale = 1.368888`). Layer-0 pos1 attention error went
9.0e-3 -> **2.6e-3**; pos0 went 1.1e-4 -> **5e-6**.

**Token parity is kernel-relative.** Same model + prompt, `-fa 0` vs default FA: gen10-15 are
`1,1,128804,30,37947,1018` vs `2052,440,611,1117,4218,469`; the fork matches itself only 10/16. The crate
emits the non-FA sequence exactly. Never quote token parity without naming the kernel path.

**What the remaining error IS:** the crate sits at **20.7% of the fork's own FA↔nonFA disagreement** —
`cos(mine-nonFA, FA-nonFA axis) = -0.9856`. Removing that axis drops mine-vs-FA from 0.0909 to 0.0613.
So what's left is a scaled copy of the kernel effect, not a porting bug.

**Where the error is created** (three-way split): L1 attention **5.84x**, L3 attention 1.37x + routed
2.09x, L5 routed 2.69x, then pass-through; L26 *corrects* (0.80). The shared expert never amplifies
(0.88-1.07). **L26 and the lm_head are the wrong places to look** — that hypothesis is dead.

**B2 settles it: L26 is exonerated, L5 is the outlier.** Measured as each layer's OWN contribution
(`l_out - attn_res`, so no attention sub-taps needed): L26 sits at **1.48% relative — the smallest in the
stack**; its big absolute delta (3.45) is purely proportional to the 233-magnitude vector it adds. L5 is
**25.3% relative, ~17x the stack median**, then L10 14.2%, L8 10.8%. L5 splits to the routed path only
`moe_out` cos 0.9505 vs `shexp` cos 0.9992; elsewhere `moe_out` is 0.998+). **L5 is ill-conditioned, not
buggy:** the crate's 64-way gating probs match the fork's to 2.18% relative, but that row is pathologically
flat (rank1/2 gap **0.65%**, rank6/7 gap 0.9%), and against the fork's own probs the crate picks 5 of 6
identical experts, differing only 26 vs 42 — a swap inside the plateau. Expert arithmetic is exonerated
(Q8_0 layers L1/L2 score cos 0.99997). So L5's 32% is a chaotic near-tie, not a defect; matching it needs
gating probs good to ~0.1% (ggml's exact sigmoid/dot order). **Do not trust `ffn_moe_topk-5` row1 as ground
truth** — it is not the top-6 of its own dumped probs row1 (row0 is consistent). Also: llama.cpp dumps
`ffn_moe_weights` BEFORE `expert_weights_norm`/`expert_weights_scale`, so crate weights (sum = 2.5, from
gguf metadata) are not comparable to that dump — compare raw probs (`moe_router_probs`, `DUMP_ROUTER=1`).

**Correction (2026-11-02): "L26 @ pos1 = 0.03995" does not reproduce.** With correctly paired single-row
dumps, L26 pos1 `l_out` = **1.7394e-02 vs non-FA** (1.8102e-02 vs FA), not 3.995e-02. The pos-1 ladder peaks
mid-stack — L5 **6.52e-02**, L6 5.55e-02, L10 5.46e-02 — and decays to L26 1.74e-02. L26 is not a 12×
outlier; it is mid-pack. `ref_layers` and the stride-fixed `ref_stride` agree exactly on `attn_res`/`l_out`
at every layer, so the old tree is sound for contiguous taps and the view-tap bug was confined to views.

**Provenance hole CLOSED (2026-11-02):** `tap_write()` now lives in `lib.rs` and `mla.rs` emits 11 taps at
the fork's stage boundaries (`mla_forward` takes `layer`). Run with `DUMP_TAPS=1 DUMP_TAPS_DIR=<dir>`.
Layer 0 also writes the legacy `<name>-<pos>` alias so the old comparators still work. Names follow the
fork's semantics (`instella-moe.cpp:215-330`): `attn_out` is **pre-gate**, `attn_gated` post-gate,
`attn_o_proj` post-wo; the post-residual tensor is `attn_res`, not `attn_out`.

Layer 0 against the fork, after the dumper stride fix: `q` 2.1e-07, `attn_gate_proj` 1.1e-07, `kv_cmpr_raw`
1.5e-07/2.97e-07, `k_pe_raw` 6.4e-08/7.7e-08 — **every projection bit-exact at both positions**. Only
`attn_out` pos1 (2.29e-03), `attn_gated` (1.64e-03) and `attn_o_proj` (2.66e-03) diverge. So the layer-0
pos1 defect is the softmax-weighted sum alone, with all its inputs exact. That is the sharpest
localisation this arc has had.

**Instrument defect #4 — strided views dumped flat.** The fork dumper wrote `d[r*n + i]`, valid only for
contiguous tensors. `kv_cmpr`/`k_pe`/`q_nope`/`q_pe` are ggml VIEWS (row stride 544 / 2048*n_head), so every
row >= 1 in those dumps was neighbouring-tensor bytes — deterministic, so two runs agreed and it looked
like signal. Proven: layer-0 `kv_cmpr` row1 = `[k_pe pos0 | latent pos1[0..480]]` (6.4e-8 / 3.0e-7 against
the crate) and `k_pe` row1 = `latent pos1[0:32]` (3.4e-7). Row 0 was always right. Fixed in the fork by
walking `nb[0..3]`; regenerated `ref_stride/` (-n 2, 567 dumps) confirms row1 now matches at 2.97e-07 /
7.7e-08. **Treat rows>=1 of view taps in `ref_layers/` as junk.**

## Carry these corrections forward (older notes have the wrong versions)

1. `kq_scale = 0.165626` is **derived**, not fitted. `0.31037` was `mscale^4/sqrt(128)` (mscale twice).
2. `attention.head_count_kv = 1`, not 2. The "2" in `dims=256x2x16x1` is the position axis (MLA: one
   latent KV head for 16 query heads).
3. `YARN_SCALE = 0.025` is **confirmed from the gguf** (`rope.scaling.factor = 40.0`), not assumed.
   Same for `freq_base 8e6`, `beta_fast 32`, `beta_slow 1`, `n_ctx_orig 4096`.
4. Logit `mean_abs` is a **b-measurement** (affine fit `mine ~ a*ref+b`: a≈1 always, b swings -2.2..+0.5,
   post-affine residual constant 0.093-0.114%). It mis-ranked builds four times.
5. Weight-on-current-token >0.7 only from **L13**; L0-L12 are 0.05-0.20.

## Open items (next session, in priority order)

1. **0.061263 orthogonal residual** — after removing the kernel axis. Unexplained; the honest target.
2. **Layer-0 pos1 attention is still outside the fork's envelope** — ladder vs non-FA (floor = FA-vs-nonFA):
   `attn_norm` 0/0, `q` 0/0 (projection bit-identical), `attn_gated` 1.64e-3 vs 0.98e-3 (1.67×),
   `attn_o_proj` 2.66e-3 vs 3.14e-3 (0.85×, inside), `attn_out` 2.29e-3 vs 1.17e-3 (1.97×). First
   divergence is the softmax-weighted sum; pos0 is clean (5e-6). The L0 delta is ORTHOGONAL to the kernel
   axis there (cos 0.03), so it is a distinct small defect — the kernel effect only shows at the logits.
3. **FA-kernel parity** only if actually required — that means reproducing `ggml_flash_attn_ext`'s
   reduction order. A kernel reimplementation, not a bug fix. Decide, don't drift.
4. ~~Rebuild the fork dumper, regenerate `ref_layers`~~ **DONE 2026-11-02** — rebuilt (MSVC), tree regenerated
   and validated 560/567 complete; the truncated tree is kept as `ref_layers_TRUNCATED_20261101/`.
5. Runs sit inside `original_context_length = 4096`, so the YaRN ramp path is only weakly exercised.
6. ~~Nothing committed~~ **DONE 2026-11-02** — home repo `Quantum-Nexus` `e38ee74` (crate + docs +
   comparators, committed by explicit pathspec; the TRM tree's own `.git` is vestigial, git resolves to
   `C:/Users/shawn`); fork repo `instella-moe` `1cc2c8d` (dumper + dequant checker + server guard).
7. **A1′ stands on the clean tree** — the three-way split reproduces exactly (L1 attn 5.84×, L3 routed
   2.09×, L5 routed 2.69×, L26 0.80); no stage owns ≥80%, so there is no single layer to fix.
8. **B1 window extension MEASURED — 25/32 tokens.** Crate 32 tokens vs the fork's `-n 32` `-fa 0` run:
   identical on indices 0-24, first divergence at index 25 (fork 1820 vs crate 305). The gate (≥32 matching)
   is not met. Note the old "16/16" could never have shown more: the dumper's loop was hardcoded to 16 and
   ignored `-n` (fixed, fork `3988780`). Divergence at 25 is expected — the logit field differs by 0.09
   (vs FA) / 0.46 (vs non-FA), which flips near-ties after ~25 steps. Tokens stay a chaotic criterion.
9. **RAM is the real throughput limit** — three `llama-server` processes (~7.4 GB, two duplicate
   qwen-coder-7B on :8587) leave ~0.5 GB free; the crate measured **94.7 s/token** under that load vs ~35 s
   for 16 tokens with the servers down. One duplicate was killed this session (PID 21496) — free 4.6 GB.

## Instruments for this arc

- Live crate: `GEN_STEPS=16 ../target/release/verify_forward.exe` (defaults ARE the accepted config).
- Sandbox toggles: `F16_K` / `F16_V` / `F16_KV` / `Q81_SYM` / `MLA_Q8` / `MOE_GATING` / `FFN_DEQ` /
  `YARN_ROPE` / `YARN_SCALE` / `YARN_KQ` / `DUMP_MLA` / `DUMP_DIR` / `DUMP_POS`.
- Comparators: `scratch/pos_compare.py`, `scratch/score_compare.py`, `scratch/router_compare.py`,
  `scratch/layer_split3.py`, `scratch/my_router.py`.
- Dump trees: **trust `ref_nofa/`, `ref_attn/`, and the regenerated `ref_layers/`** (2026-11-02, 560/567
  files validated). `ref_layers_TRUNCATED_20261101/` is the old corrupt tree — forensics only.

## Instrument lessons (they cost hours; apply before trusting any dump)

- **Verify a dump is complete before comparing** (`n*rows` values, last token parses). The fork dumper
  held ~250 files open and never flushed; a dead run left every file cut mid-number. It now fcloses
  each file, but that edit is not yet rebuilt.
- **A one-row dump silently pairs wrong positions** — `l_out-1..25` pos0 vs `l_out-26`/logits pos1
  produced a false "L26 is the only bad layer" conclusion.
- **Static dump keys capture layer 0 only**; layer-suffix the key or every layer overwrites one file.
- **Validate float-format helpers on plain values** (0.1, 1.0, 3.14159, 1234.5, -4.1937). A broken f16
  helper returned ~1.9e-35 for 1.0 and scored `mean_abs` 50.3 with no error message.
- `ffn_moe_topk` is i32; dumps must handle non-f32 or they silently vanish.
- **The fork dumper's generation loop was hardcoded to 16 steps** and ignored `-n`, so "16/16 token parity"
  was always a 16-token comparison and every "window extension" run was capped at 16. Fixed 2026-11-02
  (fork `3988780`). The crate does not stop at EOS — it keeps going past where the dumper halts.

## Prior arc (2026-10-27) — binding specificity, ANSWERED (compressed)

Heavy-atom donor-acceptor angle separates cognate from random (121.15° vs 90.79°, 3,233 complexes,
MOTION-SURG2). Three instrument defects were caught and retracted: 0.0 sentinel, centroid baseline, and
the frame bug (antibody not translated by `-centroid(cognate_antigen)`). Open rung: HDC retrieval needs a
learned boundary, not raw mean-cosine. Full detail: `docs/CHECKPOINT_20261019_*.md`,
`scratch/checkpoint_surrogate_*.md`, `council_agenda.json`.

## System resource lessons (still true)

- **RAM is the silent constraint** (8 GB VRAM, 15.35 GB RAM); a fold OOM'd as "CUDA OOM" while RAM
  starved. Check RAM before blaming the GPU.
- **Dell bloatware** disabled (3 services, `scratch/disable_dell_bloatware.bat` as admin).
- **Hermes transport truncates long params**; keep writes <2-3 KB and verify by read-back.
- **The kernel .exe locks while running**; to rebuild: kill the supervisor (find via parent PID) + kernel,
  `cargo build --release`, restart the supervisor.
