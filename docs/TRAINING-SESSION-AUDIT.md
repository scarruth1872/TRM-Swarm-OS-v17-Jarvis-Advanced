# Training session audit — `swarm_v2_artifacts/training_session`

**Verdict: this is not a trained model. It is a three-hour crash loop that saved 30.6 GiB of its own
error log.** Nothing in the folder is servable, and the weights it contains were never updated.

Two independent inspections, because the first one was preceded by a wrong diagnosis.

**ERRATUM.** An earlier version of this document claimed "both Python interpreters on this box have a
numpy install whose binaries are for cpython-3.14, so `import torch` fails". **That was false.** The
system interpreters are healthy:

| interpreter | numpy | torch | other |
|---|---|---|---|
| Python 3.11.9 | 1.26.4 | 2.5.1+cpu | — |
| Python 3.12.6 | 2.0.2 | 2.9.1+cpu | rdkit 2026.03.6 |

The cp314 numpy belongs to the *tool* environment: `execute_code` runs on **Python 3.14.9** and exports
a `PYTHONPATH` containing its own venv, so a child `py3.11`/`py3.12` process inherits it and imports
numpy 2.5.4 (cp314 wheel) instead of its own. The fix is on the calling side — run child interpreters
with `PYTHONPATH=` cleared or `-E`. Nothing needed repairing in either interpreter.

**Method 1 — stub unpickler (no torch).** The `.pt` files are zip archives; the pickle was decoded with
a stub unpickler that recovers keys, shapes and dtypes from `_rebuild_tensor_v2` arguments and never
materialises tensor data.

**Method 2 — real `torch.load`** on `checkpoint_e35_306s.pt` under Python 3.12.6 with `PYTHONPATH`
cleared. It confirms method 1 and adds what a structural read cannot see:

- `optimizers['Shield']['state']` has **0 entries**, `param_groups[0]['lr'] = 3e-4` — a rate that was
  configured and never applied. **Zero steps, confirmed by torch itself.**
- weights contain **no NaN and no Inf**; all 8,288 parameters per agent are non-zero (Gaussian
  initialisation, untouched).
- `B_lora` is **not** all-zero, so this is not a standard LoRA init (which zeroes B so ΔW = 0 at step
  0); it is a random init that no step ever moved.
- `stats['epoch_loss'][:3]` returns `np.float64(nan)`, `nan`, `nan`.

## What is on disk

| item | value |
|---|---|
| files | 34 (33 checkpoints + `final_model.pt`) + `training_report.json` |
| total | **30.59 GiB** (32,850,767,836 bytes / 31,328.9 MiB) |
| timespan | 2026-08-06 04:37 → 07:33 (report: `duration_s: 10802`) |
| checkpoint cadence | every ~35 epochs (e35, e69, e103 … e1155), final at epoch 1187 |
| file growth | 57.7 MB (e35) → 1.94 GB (final) — see cause below |

## What the weights actually are

Every checkpoint carries 15 agents (`Shield, Logic, Devo, Seeker, Verify, Archi, Flow, Bridge,
Orchestra, EventLedger, LogicMaster, Pulse, THzGateway, Jarvis, Hermes`), each with exactly 9 tensors:

```
A_lora (16,32)  B_lora (32,16)  C_lora (16,32)  D_lora (32,16)
W_in   (32,96)  W_out  (96,32)
A_diag (32,)    B_vec  (32,)    C_vec  (32,)
```

8,288 parameters per agent, **124,320 parameters in total** (~0.5 MB of float32). That is less than
0.5 % of one checkpoint's smallest file, which is the first clue about where the bytes went.

## The run never took a step

- `stats["epoch_loss"]`: **1,187 entries, all `NaN`**.
- optimizer `state`: **`{}`** in every checkpoint — an empty state dict means no update step ever
  executed.
- `training_report.json`: `total_steps: 0`, `avg_loss`/`best_loss`/`final_loss` all `NaN`.
- `errors: 13,923,510` with **two distinct messages**, repeated:
  - `mat1 and mat2 shapes cannot be multiplied (64x96 and 16x16)` — 13,905,705 times
  - `mat1 and mat2 shapes cannot be multiplied (16x96 and 16x16)` — 17,805 times
  - agent `Shield`, `step: 0`, span 04:32:16 → 07:32:18 (the whole session)

`mat2` is `16x16`: a `nn.Linear(16, 16)` — i.e. an in-feature dimension taken from `lora_rank` (16)
where the actual input is 96 wide (`state_dim` 32, `W_in` maps 96 → 32). The trainer built the
projection with the wrong first argument, so **every** forward pass raised, and the loop caught the
exception per agent per epoch instead of aborting.

## Why 30 GiB for 0.5 MB of weights

Each checkpoint embeds the **entire accumulated `errors` list** (plus `stats`), and that list is
pickled inside `data.pkl`:

| checkpoint | total | `data.pkl` | tensors |
|---|---|---|---|
| e35 (306 s) | 55.0 MiB | **54.5 MiB** | ~0.02 MiB |
| final (1187) | 1.94 GB | ~1.94 GB | ~0.02 MiB |

So the checkpoints record failure, not learning — the file grows because the error log grows.

## The report contradicts itself, and is wrong twice

```json
"total_steps": 0,  "errors": 13923510,
"error_rate_pct": 1392351000.0,   // 1.39 billion percent
"avg_loss": NaN, "best_loss": NaN, "final_loss": NaN
```

A rate computed against `total_steps: 0` produced a percentage four orders of magnitude past 100 %.
The report's `error_log` keeps only the first 20 of the 13.9 M entries — all identical, all `step: 0`.
By this platform's own rule, a metric that can only return `NaN` or a billion percent is not a metric,
and this report should never have been presented as a training result.

## What this means

- **Nothing here can be served, evaluated or fine-tuned from.** There is no gradient step to resume,
  no loss curve to compare, and the weights are at initialisation.
- The folder costs **30.6 GiB** to preserve a 4 KB conclusion: the run was broken from its first
  second and should have aborted on the first exception instead of looping 1,187 times.

## Recommended actions — taken

1. **Kept** `training_report.json` (4 KB) and this audit — the evidence of the failure.
2. **Kept** one specimen, `checkpoint_e35_306s.pt` (57.7 MB), which contains the failure at its
   smallest.
3. **Deleted the other 32 checkpoints on 2026-11-18 — 30.54 GiB reclaimed** (32,793,072,830 bytes),
   0 errors. Each file's name, size, mtime and a first-1-MiB sha256 are recorded in
   `data/training_session_pruned_20261118.json`, so the deletion is auditable even though the bytes
   are gone. The folder now holds 2 files / 55 MiB.
4. If this trainer is ever recovered, it needs three fixes: build the projection from the real input
   width (96, not `lora_rank`), let the first exception abort the run, and never embed the error list
   in a checkpoint.
