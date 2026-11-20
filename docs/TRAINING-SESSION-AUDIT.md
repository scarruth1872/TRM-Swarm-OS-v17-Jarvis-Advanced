# Training session audit — `swarm_v2_artifacts/training_session`

**Verdict: this is not a trained model. It is a three-hour crash loop that saved 30.6 GiB of its own
error log.** Nothing in the folder is servable, and the weights it contains were never updated.

Inspected without torch or numpy (both Python interpreters on this box have a numpy install whose
binaries are for cpython-3.14, so `import torch` fails): the `.pt` files are zip archives, so the
pickle was decoded with a stub unpickler that recovers keys, shapes and dtypes but never materialises
tensor data.

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

## Recommended actions

1. **Keep** `training_report.json` (4 KB) and this audit — the evidence of the failure.
2. Optional: keep one representative checkpoint (`checkpoint_e35_306s.pt`, 57 MB) as a specimen;
   the other 32 add nothing that e35 does not already contain, and each is larger for no new
   information.
3. **Reclaim ~30.6 GiB** by deleting the remaining checkpoints — *not done; requires the owner's
   approval.*
4. If this trainer is ever recovered, it needs three fixes: build the projection from the real input
   width (96, not `lora_rank`), let the first exception abort the run, and never embed the error list
   in a checkpoint.
