# CHECKPOINT 2026-11-01 — Instella MoE parity: pos-1 defect FOUND AND FIXED

Read this with `RESUME_HERE.md`. This note covers only the Instella parity arc; the platform boot
procedure is in RESUME_HERE.md and is unchanged.

## One-line status

The Rust distill of Instella-MoE-16B-A3B now reproduces the reference **16/16 greedy tokens**, with
**exact top-10 ordering** and logit `mean_abs` **0.090881** (was 6.22 at the start of the arc, 0.2176
mid-arc). The remaining difference is a *scaled copy of the fork's own kernel disagreement*, not a
porting bug.

## Acceptance numbers (live crate, verified)

| metric | value | where |
|---|---|---|
| logits mean_abs vs fork FA | **0.090881** | `wsl_crates/.../logs/kqfix16_v3.log` |
| logits max_abs | 0.504295 (idx 4079) | same |
| top-10 tokens | **exact order**, `[128804,3,14,2596,16,8567,342,43225,1,1031]` | same |
| greedy tokens vs non-FA fork | **16/16** at 16 tokens; **25/32** at 32 tokens (first divergence index 25: fork 1820, crate 305) | same + `logs/gen32.log`, `ref_nofa32/run.log` |
| greedy tokens vs FA fork | 10/16 (fork disagree with itself, see below) | same |
| layer-0 attn_out pos0 | 5e-6 | `logs_cfg_k1_v1_q1` |
| layer-0 attn_out pos1 | 0.002560 vs FA / **0.002290 vs non-FA** (floor: FA-vs-nonFA 0.001165) | same |

## THE FIX (three changes, all source-justified, no free parameters)

The defect was that the crate computed attention in f32 while llama.cpp's attention runs
`ggml_flash_attn_ext`, whose builder casts k and v from F32 to F16 (`build_attn_mha`). Rounding the
same operands the same way is the fix.

1. **F16 rounding of q_absorbed, q_pe, kcur, vcur** — `f16_round` (RN-even) in `lib.rs`, applied in
   `mla.rs` after the K/V are built. Live crate: unconditional.
2. **ggml-exact Q8_1** — `quantize_q8_1` is now symmetric `amax/127` with the scale stored as f16 and
   an `amax == 0` guard, matching `quantize_row_q8_1_ref`. The previous "midrange scale + per-block
   mean" scheme was NOT ggml's; it was compensating for the missing f16 rounding.
3. **kq_scale = 0.165626** — `mscale^2 / sqrt(n_embd_head_k=128)`, `mscale = 1.368888`. Derived, not
   fitted (see Corrections).

Effect of each, measured on layer-0 attention output at position 1 (routing-independent, before any
MoE) and full logits:

| F16_K | F16_V | Q8_1 sym | attn_out pos0 | attn_out pos1 | logits mean_abs |
|---|---|---|---|---|---|
| 0 | 0 | 0 | 1.1e-4 | 9.0e-3 | 0.2176 |
| 0 | 0 | 1 | 1.1e-4 | 7.2e-3 | 1.2996 |
| 0 | 1 | 0 | 5e-6 | 9.1e-3 | 0.7268 |
| 0 | 1 | 1 | 5e-6 | 7.3e-3 | 3.6463 |
| 1 | 0 | 0 | 1.1e-4 | 7.0e-3 | 0.8394 |
| 1 | 0 | 1 | 1.1e-4 | 2.8e-3 | 0.5990 |
| 1 | 1 | 0 | 5e-6 | 6.9e-3 | 0.5501 |
| **1** | **1** | **1** | **5e-6** | **2.6e-3** | **0.0909** |

The all-three-on row is the only one where every setting matches llama.cpp's source. It is also the
best on both criteria. This is the first configuration in the arc that was both better AND principled.

## TOKEN PARITY IS KERNEL-RELATIVE (the arc's biggest conceptual correction)

The fork disagrees with ITSELF. Same model, same prompt, `-fa 0` vs default flash attention:

- FA (default):       gen10-15 = `2052, 440, 611, 1117, 4218, 469`
- non-FA (`-fa 0`):   gen10-15 = `1, 1, 128804, 30, 37947, 1018`
- agreement: 10/16, first difference at index 10.

The crate emits the **non-FA** sequence exactly. Earlier notes ("diverges at gen 10") were therefore
scoring a faithful non-fused implementation against the fused kernel's output. Token parity must always
name a kernel path. Also: a 0.09-0.53 logit perturbation flips branches over 16 autoregressive steps,
so token agreement is a chaotic criterion; logit-field numbers are primary.

Related paradox, now explained (see A2' below): the crate's logits are 0.0909 from FA but 0.4576 from
non-FA - nearer FA - while its tokens match non-FA. Both readings are correct.

## A1' - the pos-1 error is DISTRIBUTED, not carried by any single layer

Per-layer relative error vs the non-FA fork, pos1 (`attn_res` / `l_out`):
L0 0.00172/0.00140 -> L1 0.00820/0.00714 (gain_attn **5.84**) -> L3 0.01357/0.02518
(gain_attn 1.37, **gain_routed 2.09**) -> L5 0.02444/0.06520 (**gain_routed 2.69**) -> decays to
L26 0.02177/0.01739 (gain 0.80/0.78, i.e. L26 CORRECTS).

- Shared expert never amplifies (gains 0.88-1.07); it is not a carrier at any layer.
- Attention weights match the fork to **1.1e-5 at L0**, worst **3.1% at L21** - inherited error, not
  generated per layer. Nothing approaches the 80% "carrier" threshold. A1' is refuted.
- Chain: L1 attention introduces it, L3 (attn+routed) doubles it, L5 (routed) doubles it, then the
  stack passes it through. L26 and lm_head are the WRONG places to spend effort.

## A2' - the residual lies on the fork's own kernel axis

Axis = FA_logits - nonFA_logits (|axis| 192.4492, mean_abs 0.532010).

- `mine - nonFA`: |delta| 166.65, **cos(delta, axis) = -0.9856** -> 98.6% of its norm on the axis.
- `mine - FA`:    |delta| 39.87, cos = -0.7072 -> 70.7% on the axis.
- The crate spans **20.7% of the fork's own kernel disagreement**.
- Removing that axis drops mine-vs-FA from 0.090881 to **0.061263** (1.48x).

So the remaining error is a scaled copy of the kernel effect at 21% amplitude: below the threshold that
flips tokens, above the threshold that moves logits. The lm_head mean-row / common-mode hypothesis is
refuted (delta is not parallel to the mean row).

## B2 - L26 is EXONERATED; the outlier is L5's routed output

The task carried "L26 @ pos1 residual is the blocker". It is not. Each layer's own contribution is
`l_out - attn_res`, so its error can be measured without any attention sub-taps:

| L | rel err of that layer's contribution | L | rel err |
|---|---|---|---|
| 0 | 0.0001 | 13 | 0.0503 |
| 1 | 0.0090 | 18 | 0.0337 |
| 3 | 0.0399 | 22 | 0.0561 |
| **5** | **0.2526** | 23 | 0.0527 |
| 6 | 0.0698 | 24 | 0.0414 |
| **8** | **0.1077** | 25 | 0.0346 |
| **10** | **0.1418** | **26** | **0.0148** |

- **L26's contribution error is 1.48%, the smallest in the stack.** Its large absolute delta (3.45) is
  purely proportional: the layer adds a 233-magnitude vector. L26 neither creates nor amplifies the error
  and is the wrong place to spend effort (consistent with A1' "L26 corrects", gain 0.80).
- **L5 is the real outlier: 25.3% relative, ~17x the stack median (~0.05)**, and 2.6% of total absolute
  error. L10 (14.2%) and L8 (10.8%) follow. Every layer from L4 on is "inherited" in the sense that its
  input error already dominates, so nothing above L5 is a *carrier* - but L5 is the only layer whose own
  arithmetic is measurably worse than its input.
- L5 splits cleanly to the routed path: `moe_out` rel 0.3213 / cos 0.9505 while `shexp` is 0.0392 / cos
  0.9992. Elsewhere `moe_out` sits at cos 0.998+. So the deficit is in the routed experts, not the router's
  input and not the shared expert.
- **RESOLVED: L5 is ILL-CONDITIONED, not buggy.** The crate's gating probs match the fork's at L5 "Hello"
  to **2.18% relative** (mean_abs 0.0011, corr 0.99940) - the router input and the 64-way gate are right.
  What differs is the top-k choice, and that row is pathologically flat: the fork's own probs there are
  `0.1973, 0.1960, 0.1412, 0.1192, 0.1107, 0.1044, 0.1035` - rank1/2 gap **0.65%**, rank6/7 gap 0.9%.
  Against the fork's own probs, the crate picks 5 of 6 identical experts (`{7,9,12,36,60}`) and differs only
  `26` vs `42`, a swap inside that flat plateau. A 2% probs agreement cannot resolve a 0.65% gap, so the
  disagreement is a chaotic near-tie, not arithmetic - matching it would need gating probs good to ~0.1%,
  i.e. ggml's exact sigmoid/dot order. Routing there also varies hugely with position: BOS picks
  `[55,40,0,33,2,15]` with a 13.4% rank1/2 gap (well conditioned), "Hello" picks a different set.
- **The fork's L5 router dumps are SELF-INCONSISTENT - do not use `ffn_moe_topk-5` row1 as ground truth.**
  Its dumped topk row1 `[11,21,26,44,46,60]` is NOT the top-6 of its own dumped probs row1
  (`{7,9,12,36,42,60}`, with or without the bias), while row0 IS consistent. So row1 of the topk dump is
  misaligned/stale. Every "2/6 expert overlap" alarm came from this bad row; the probs-consistent figure is
  5/6. Row alignment elsewhere is sound: mine pos0<->fork row0 and mine pos1<->fork row1 both at cos 0.99999
  / 0.99791 on `attn_res`, `l_out`, `ffn_shexp`, `ffn_moe_out`.
- **Expert arithmetic is exonerated too.** Q8_0 `ffn_down_exps` layers L1/L2 score cos 0.99997/0.99983, so
  the Q8_0 expert path is fine; only L5's routing is ill-conditioned. `ffn_down_exps` is mixed (type6 Q6_K
  on 14 layers, type8 Q8_0 on 12 - same split as shexp) and the crate already reads it per layer, so type is
  not a discriminator either way.
- **llama.cpp dumps `ffn_moe_weights` BEFORE `expert_weights_norm` and `expert_weights_scale`**, so a crate
  weight vector (already normalised to sum `expert_weights_scale` = 2.5, confirmed from gguf metadata) is
  not comparable to that dump. Compare raw probs instead - hence `moe_router_probs()` + `DUMP_ROUTER`.
- **Alignment trap closed.** The fork's single-row L26 dumps (`rows=1`) ARE position 1: mine's pos1 matches
  at 2.2e-2 while pos0 is off by 0.77. Earlier L26 pos1 numbers stand.
- **B2's literal spec is not runnable from existing dumps** - the fork has every L26 attention tap at pos1,
  but the crate's tap tree has no `attn_norm`/`q`/`kv_cmpr`/`attn_gated`/`attn_o_proj` at L26. The
  contribution test above answers the question without them, so no re-run was needed.

**PROVENANCE HOLE (fix before trusting any tap comparison).** No committed Rust writes the crate's tap tree
(`sandbox_yarn/swarm-dequant/logs_cfg_k1_v1_q1`); every bin in `src/bin` only *reads* the fork's dumps. The
dumps are internally consistent (`l_out == l_out_no_routed + ffn_moe_out` to 0.0000 at L5 pos1) and are not
the fork's FA run (`mine-vs-FA` exceeds `FA-vs-nonFA` at every layer), so they really are a third
implementation - but the generator is uncommitted and therefore unreproducible. Recover or rewrite it before
building further conclusions on tap diffs.

## TAP WRITER RECOVERED (the provenance hole is closed)

The crate's tap tree `logs_cfg_k1_v1_q1` was written by a binary nobody committed, so every tap
comparison rested on an unreproducible artifact. Fixed 2026-11-02: `tap_write()` in `lib.rs`
(`DUMP_TAPS=1`, `DUMP_TAPS_DIR`, default `logs/taps`), `mla_forward(..., layer, ...)`, and 11 taps in
`mla.rs` at the same stage boundaries the fork dumps. Layer 0 also writes the legacy short alias
`<name>-<pos>` so `scratch/stage_compare.py` and `scratch/layer_decomp.py` keep working.

Tap names follow the FORK's stage semantics, read from `instella-moe.cpp:215-330`, not invented:
`attn_gate_proj` (wg@h), `q` (wq@h), `kv_cmpr_raw`/`k_pe_raw` (the raw wkv_a split, pre-norm and
pre-RoPE — that is what the fork's `kv_cmpr`/`k_pe` dumps actually contain), `kv_cmpr_norm`,
`k_pe_rope`, `q_absorbed`, `attn_v`, `attn_out` (**pre-gate**), `attn_gated` (post-gate),
`attn_o_proj` (post-wo). Note the fork's "attn_out" is NOT the post-residual tensor — that one is
`attn_res`, and the crate's `verify_forward` computes it as `attn + res_full`.

**Layer 0 verified against the fork's dumps** (`DUMP_TAPS=1 GEN_STEPS=2`, vs `ref_layers/`):

| tap | pos0 | pos1 |
|---|---|---|
| `q` | 2.13e-07 | 2.02e-07 |
| `attn_gate_proj` | 1.10e-07 | 1.63e-07 |
| `attn_out` (pre-gate) | 2.63e-07 | **2.29e-03** |
| `attn_gated` | 1.89e-07 | **1.64e-03** |
| `attn_o_proj` | 3.13e-07 | **2.66e-03** |

So **every layer-0 projection is bit-level exact** (2e-7 = f32 round-trip on a 2048-dim vector) and the
ENTIRE layer-0 position-1 gap lives in the softmax-weighted sum and its downstream projection. That
sharpens open item 2 from "attention is 2.6e-3 off" to "the weighted sum is, with all inputs exact".
`attn_out` pos1 reproduces the previously recorded 0.002290 vs non-FA exactly, so the old measurement
is confirmed by the new writer — and the old writer's numbers were right even though its source was lost.

## CORRECTION: "L26 @ pos1 = 0.03995" does NOT reproduce

Measured 2026-11-02 with the committed tap writer, single-row L26 dumps correctly paired to position 1
(the documented trap: a `rows=1` dump is position 1, not 0). **L26 pos1 `l_out` = 1.7394e-02 vs the
non-FA fork** (`ref_layers`, `ref_nofa`, `ref_stride` all agree to the last digit) and **1.8102e-02 vs FA
(`ref_attn`)**. The 3.995e-02 previously recorded is not reproducible from committed code and is superseded.

That kills the "L26 is a 12x outlier" framing at the source. The full position-1 ladder is:

| L | 0 | 1 | 3 | 5 | 6 | 10 | 13 | 20 | 24 | 25 | 26 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| `l_out` pos1 rel | 1.40e-3 | 7.14e-3 | 2.52e-2 | **6.52e-2** | 5.55e-2 | 5.46e-2 | 3.50e-2 | 2.88e-2 | 3.32e-2 | 2.72e-2 | 1.74e-2 |

L26 sits below L5, L6, L10, L13, L24 and L25. The pos-1 error peaks mid-stack (L5/L6 ~6%) and *decays*
toward the end, which is the opposite of "the last layer carries it". `attn_res` tells the same story:
pos0 stays at 1.3e-7..5.3e-3 across all 27 layers, pos1 grows to 6.1e-2 at L6 and falls to 2.18e-2 at L26.

Two corollaries:
- **The old tree is sound for contiguous taps.** `ref_layers` (generated before the stride fix) and
  `ref_stride` (after it) agree exactly on `attn_res` and `l_out` at every layer tested, confirming the
  strided-view bug was confined to the view taps.
- **The pos-1 error is depth-accumulated, not layer-owned** - which is consistent with A1' ("no stage owns
  >=80%") and with B2 ("L26's own contribution is the smallest in the stack", 1.48%).

## REPRODUCE (exact commands, no env needed for the live crate)

Live crate (defaults now ARE the accepted config; binary already built):

```
cd wsl_crates/trm-rust/swarm-dequant
GEN_STEPS=16 ../target/release/verify_forward.exe      # ~175 s for 16 tokens
```

The run prints `LOGITS PARITY`, `ref top-10 tokens`, `mine top-10 tokens`, `sampled tokens`. Note the
parity line reads a HARDCODED path `D:/llama.cpp-instella/build/bin/Release/dump_logits.txt` - whatever
fork run last wrote it. Do not trust it blindly; compare against the named dump trees below.

Sandbox (all A/B toggles, same code plus env switches):

```
cd swarm_v2_artifacts/sandbox_yarn/swarm-dequant
F16_K=1 F16_V=1 Q81_SYM=1 GEN_STEPS=16 MOE_GATING=sigmoid FFN_DEQ=0   YARN_ROPE=1 YARN_SCALE=0.025 YARN_KQ=0.165626 ./target/release/verify_forward.exe
```

Toggle meanings: `F16_K` round q_absorbed/q_pe/kcur to f16; `F16_V` round vcur to f16; `F16_KV` master
switch for both; `Q81_SYM=1` ggml-exact symmetric Q8_1 (else the legacy midrange+mean scheme);
`MLA_Q8=0` leave the absorbed q in f32; `DUMP_MLA=1 DUMP_DIR=<dir> DUMP_POS=-1` write per-stage dumps.

Fork (oracle) - `D:/llama.cpp-instella`:

```
cmake --build build --config Release --target llama-dump-attn
build/bin/Release/llama-dump-attn.exe -m D:/trm-models/instella/amd.Instella-MoE-16B-A3B-Think.Q4_K_M.gguf   -p "Hello" -c 4096 -t 8            # FA path  -> no kq taps exist
build/bin/Release/llama-dump-attn.exe ... -fa 0    # non-FA    -> kq / kq_soft_max taps exist
```

`examples/dump_attn.cpp` dumps every row of every tensor, adds `kq*`, `ffn_moe_*`, `attn_res`,
`l_out_no_routed` taps, supports i32, layer-suffixes the stage keys, and `fclose`s each file after
writing (see Instrument lessons). It is **committed** (fork repo `instella-moe`, `1cc2c8d`) and was
**rebuilt 2026-11-02 06:56** - the binary in `build/bin/Release` is current as of that build.

## Dump trees (what to trust)

| path | what | state |
|---|---|---|
| `swarm_v2_artifacts/ref_nofa/` | fork `-fa 0`, all 27 layers, per-layer `kq_soft_max`, `attn_res`, `l_out*`, `ffn_*` | **COMPLETE - use this** |
| `swarm_v2_artifacts/ref_attn/` | fork FA default; logits + attn stages, no kq taps | complete |
| `swarm_v2_artifacts/ref_topk/` | earlier FA run; per-layer topk/weights/scores | complete |
| `swarm_v2_artifacts/ref_layers/` | layer-suffixed run, **REGENERATED 2026-11-02** with the rebuilt dumper | **COMPLETE - use this** (560/567 files validated: `n*rows` values, last token parses; the 7 empty files are unsuffixed static-key stubs, no layer-0 data in them) |
| `swarm_v2_artifacts/ref_layers_TRUNCATED_20261101/` | the old truncated tree, kept only for forensics | **do not use** (509 files, 17 corrupt mid-number) |
| `.../logs_cfg_k1_v1_q1/` | crate, accepted config, per-stage + per-layer weight dumps | good |
| `.../logs_layers/` | crate, per-layer `attn_wts-<L>-<pos>` | good |

## Journal (monad ledger, sealed this arc)

137 kq root cause; 138 all-rows localisation; 139 quantization-scheme compensation; 140 kq sweep
(V-shaped, mean_abs is offset); 141 f16 fix and kernel-relative parity; 142 A1' refuted; 143 **no-op
`kind:"probe"` entry** (path-resolution test - ignore); 144 A2' kernel axis; 145 kq derived not fitted;
146 gguf metadata confirmation + n_kv correction.

There is no listing endpoint on :8022 (only `POST /crate/journal/realize`); anchor files must be absolute
paths and must NOT include the 9 GB gguf (hashing it times out the endpoint).

## CORRECTIONS to earlier claims (do not re-inherit the wrong versions)

1. **`kq_scale = 0.165626` was NOT fitted.** It is `mscale^2/sqrt(128)` with
   `mscale = 1.0 * (1 + 0.1*ln(40)) = 1.368888`. The competing `0.31037` figure is `mscale^4/sqrt(128)` -
   mscale squared twice. The sweep merely confirmed the derivation.
2. **`n_kv = 1`, not 2.** gguf `attention.head_count_kv = 1`; the "2" in `dims=256x2x16x1` is the
   TOKEN/POSITION axis (16 heads x 2 positions = rows=32). This is MLA: one latent KV head shared by
   all 16 query heads.
3. **`YARN_SCALE = 0.025` is confirmed from the model, not assumed.** gguf KV section (hand-parsed):
   `rope.scaling.type=yarn`, `factor=40.0`, `original_context_length=4096`, `yarn_beta_fast=32.0`,
   `yarn_beta_slow=1.0`, `yarn_log_multiplier=0.1`, `rope.dimension_count=32`, `rope.freq_base=8e6`.
   The crate passes all five matching values.
4. **Weight-on-current-token is not "mostly > 0.7".** L0-L12 run 0.05-0.20; >0.7 only from L13
   (L13 0.725, L18 0.847, L26 0.841). Broad at the bottom, local at the top.
5. **`mean_abs` on logits is a b-measurement.** Across kq values the affine fit `mine ~ a*ref + b` gives
   a ~= 1 everywhere while b swings -2.2..+0.5 and the post-affine residual stays 0.093-0.114%.
   `mean_abs` was 90-287% pure offset. It mis-ranked builds four times in this arc.

## INSTRUMENT LESSONS (cost real time; check before trusting any dump)

- **Never keep dumps open.** The fork dumper opened ~250 files and never closed them; when a run died the
  buffered tail was lost and every file ended mid-number (e.g. a lone `-`). **Always verify a dump is
  complete before comparing**: expected `n*rows` values, last token parses. 17 of 509 files were corrupt.
- **A one-row dump silently pairs wrong positions.** Fork `l_out-1..25` were position 0 while `l_out-26`
  and logits were position 1; that produced the false "L26 is the only bad layer" reading for hours.
- **Static dump keys only capture layer 0.** `attn_out`/`q`/`kv_cmpr` were keyed by name, so every layer
  overwrote the same file and only L0 survived. Layer-suffix the key.
- **A broken f16 helper returns denormals, not errors.** An f32-exponent-from-f16-exponent bug gave
  ~1.9e-35 for 1.0 and `mean_abs` 50.3. Validate any float-format helper against numpy/half on plain
  values (0.1, 1.0, 3.14159, 1234.5, -4.1937) before using it.
- `ffn_moe_topk` is i32; the dumper must handle non-f32 tensors or it silently skips them.
- **NEVER index a ggml dump as `d[r*n + i]`.** That is only valid for contiguous tensors. `kv_cmpr`,
  `k_pe`, `q_nope` and `q_pe` are VIEWS into a wider buffer (row stride 544 for the MLA latent, 2048*n_head
  for q), so flat indexing reads the neighbouring tensor's bytes for every row after the first. It is
  deterministic, so two independent runs agree with each other and it looks like real signal. Proven for
  layer 0: fork `kv_cmpr` row1 = `[k_pe pos0 (32 vals) | latent pos1[0..480]]` (matched the crate at 6.4e-8
  and 3.0e-7) and `k_pe` row1 = `latent pos1[0:32]` (3.4e-7). Row 0 is always correct, which is why every
  single-row dump looked fine. Fixed in the fork dumper 2026-11-02 by walking `nb[0..3]`; **rows>=1 of a
  view tap in `ref_layers/` are junk** — use the regenerated `ref_stride/` tree instead.
- **A hardcoded loop bound masquerades as a result.** The fork dumper generated a fixed 16 tokens
  (`for step < 16`) and ignored `-n`, so "16/16 token parity" was always a 16-token comparison and every
  "window extension" run was silently capped at 16. Fixed 2026-11-02 (fork `3988780`); `n_gen` now reads
  `params.n_predict`, defaulting to 16 only when `-n` is unset. Check that a run length control is really
  wired before concluding anything about divergence position.
- **The crate does not stop at EOS.** `verify_forward` keeps generating past the position where the fork's
  dumper halts (32-token run: positions 16-31 = `33001,14,270,3967,1438,1359,582,19923,4,305,342,23676,
  418,582,19923,3`). Compare only within the reference's own generated length.

## OPEN ITEMS (next session, in priority order)

1. **0.061263 orthogonal residual** (after removing the kernel axis). Unexplained; the honest target.
   Structure check 2026-11-02: it is BROADBAND. `mine-nonFA` = 0.8535 * axis + r, `|r|` 28.187 vs `|delta|`
   166.65 (16.9% of the norm); `r` mean_abs 0.0613, max_abs 0.4063, and its mean on the top-20 candidates
   is only 0.123 vs 0.061 overall, `corr(|r|, |logit|) = 0.08`. No concentration on a vocabulary region, a
   candidate set, or a layer - which is what accumulated f16/f32 rounding noise looks like. There is no
   structure here to attack; the honest target is reproducing ggml's reduction order, not a bug hunt.
2. **Layer-0 pos1 attention is the only stage still outside the fork's envelope** - the stage ladder below
   puts the first divergence in the softmax-weighted sum (`attn_gated` 1.67x the floor), not in the QKV
   projection (`q` is bit-identical). And the L0 delta is ORTHOGONAL to the FA-vs-nonFA axis at that stage
   (cos 0.03), so it is a distinct small defect, not the kernel effect that dominates the logits. Not closed.

   Layer 0, position 1, each tap vs the fork's non-FA run, with FA-vs-nonFA as the per-stage floor:

   | stage | floor | mine | ratio | cos(mine, floor axis) |
   |---|---|---|---|---|
   | attn_norm | 0 | 0 | - | - |
   | q | 0 | 0 | - | - |
   | attn_gated | 0.000983 | 0.001638 | 1.67x | 0.028 |
   | attn_o_proj | 0.003139 | 0.002660 | 0.85x | 0.001 |
   | attn_out | 0.001165 | 0.002290 | 1.97x | 0.009 |

   Position 0 is clean throughout (5e-6 at `attn_out`, floor 0). So the L0 defect is position-1 only, and
   the projection is exonerated: only the weighted-sum/softmax step and its downstream `o_proj` differ.
3. **Match FA's fused kernel?** Only if FA token parity is actually required. It means reproducing
   `ggml_flash_attn_ext`'s reduction order - a kernel reimplementation, not a bug fix. Decide, don't drift.
4. ~~Rebuild the fork dumper and regenerate `ref_layers`~~ **DONE 2026-11-02** - rebuilt, tree regenerated
   and validated (560/567 complete), truncated tree moved to `ref_layers_TRUNCATED_20261101/`.
5. **Unverified assumption:** runs sit inside `original_context_length = 4096`, so the YaRN ramp path is
   only weakly exercised. A post-4096 test would exercise the ramp for real.
6. ~~Nothing committed~~ **DONE 2026-11-02** - crate + docs committed in the home repo (`Quantum-Nexus`,
   `e38ee74`); dumper + dequant checker committed in the fork repo (`instella-moe`, `1cc2c8d`).
7. **A1' re-checked on the complete tree (2026-11-02):** the three-way split reproduces the earlier numbers
   exactly (L1 attn 5.84x, L3 routed 2.09x, L5 routed 2.69x, L26 0.80) - so A1' stands on clean data and no
   stage owns >=80%. There is no single root-cause layer to fix; the residual is distributed rounding.
8. **Window extension not yet run:** B1 (`GEN_STEPS=32/64`) needs a fork `-fa 0` run of the same length.
   The crate run is cheap; the fork run is ~20-30 s/token under the current RAM load.

## 2026-11-02 delta (what this session actually did)

1. **Committed everything.** Home repo (branch `Quantum-Nexus`) `e38ee74`: crate `src/{lib,mla}.rs`,
   `src/bin/{verify_forward,dump_dequant,dump_kv2}.rs`, the five `scratch/` comparators, this checkpoint,
   `RESUME_HERE.md`, `CHANGELOG.md`. Fork repo (branch `instella-moe`) `1cc2c8d`: `examples/dump_attn.cpp`,
   `examples/dequant_check.cpp`, the CMakeLists target wiring, and the `server-context.cpp` logits guard.
   Note the TRM tree's own `.git` is vestigial (index+objects only, no HEAD/config) - git resolves up to
   `C:/Users/shawn`, whose repo carries ~20 500 dirty paths. Commits were done with explicit pathspecs so
   nothing else was swept in.
2. **Dumper rebuilt** (`cmake --build build --config Release --target llama-dump-attn`, MSVC, 2026-11-02
   06:56) - picks up `fclose` per file and layer-suffixed keys. Old binary predated both.
3. **`ref_layers/` regenerated** (`-fa 0`, same prompt/model) and validated: 560/567 files have exactly
   `n*rows` values and a parseable last token. The 7 zero-byte files are unsuffixed static-key stubs
   (`dump_attn_out.txt` etc.) that the layer-suffixed keys never write - ignore them. The truncated tree
   is preserved as `ref_layers_TRUNCATED_20261101/`. That run reports **EXIT=1 because it was killed on
   purpose** after the prefill dumps landed (it was mid-generation at gen 7, ~19 s/token); completeness was
   judged per file by value count, not by exit code. Do not read EXIT=1 there as a failed dump.
4. **A1' reproduced on the clean tree** - the split table is numerically identical to the one measured on
   the old tree (L1 5.84x, L3 2.09x routed, L5 2.69x routed, L26 0.80), so the A1' conclusion never depended
   on the corrupt files. Gate still NOT met: no stage carries >=80%.
5. **Layer-0 pos1 attn_out, both references:** mine-vs-FA 0.002560, mine-vs-nonFA 0.002290, against a
   fork FA-vs-nonFA spread of 0.001165 at that stage. So the crate is ~2x the fork's own kernel spread at
   L0 pos1 - the one stage still measurably outside the envelope. This is open item 2, not closed.
6. **RAM is the throughput constraint, not the code.** With three `llama-server` processes resident
   (~7.4 GB; two are duplicate qwen-coder-7B servers BOTH bound to :8587) free physical memory sits near
   0.5 GB, so a 10.4 GB gguf run thrashes. Measured this session: the crate's 32-token run took 1,515 s
   for its first 16 tokens = **94.7 s/token**, versus ~35 s for 16 tokens when the servers were down. That
   is a 50x slowdown, all of it page-cache churn. Free one duplicate before timing anything.
7. **B1 window extension MEASURED: 25/32 tokens.** The crate generated 32 tokens and positions 0-15 are
   bit-identical to the accepted 16; the fork's `-n 32` `-fa 0` run (after the `-n` fix, fork `3988780`)
   produced 32. They agree on indices 0-24 and **first diverge at index 25** (fork 1820, crate 305).
   So the gate ">=32 matching or divergence strictly beyond 32" is NOT met, but the honest number is
   25/32, not the 16/16 that a 16-step-capped dumper could ever have shown. Divergence at 25 is consistent
   with the measured logit field (0.09 vs FA, 0.46 vs non-FA) flipping a near-tie branch after ~25 steps;
   tokens remain a chaotic criterion and the logit numbers stay primary.
