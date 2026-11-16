# Training on a rented GPU node

The local measurement (G138) is the reason this directory exists: 77.2 M first-party tokens at 1,148 tok/s on this CPU
is **18.6 h per epoch** for a 58 M-param model, 30.8 h at 106 M and 64.7 h at 217 M, and there is no GPU path here (RX
6700 XT is Vulkan; Windows torch has no ROCm). A rented node removes that wall. Nothing about *serving* changes — the
node produces a checkpoint, the platform runs artifacts on-device, as before.

## What ships

| file | what it does |
|---|---|
| `prepare_corpus.py` | deterministic text shards from the platform tree (sorted walk, content-hash dedupe, source trees ranked ahead of docs, vendors and generated JSON excluded) |
| `train_lm.py` | self-contained byte-level LM: tied embeddings, Xavier init, AdamW, warmup+cosine, grad clip 0.5 (house conventions), throughput + val loss printed, resumable checkpoints |

Both are dependency-light: `torch` only. No `datasets`, no `peft`, no `accelerate` — matching what is actually
available on this box, so the same scripts run identically on the node.

## Run order

```bash
# 1. corpus (on the node, from a clone or a copied tree)
python prepare_corpus.py --root /path/to/TRM-Swarm-OS-v2 --out corpus --max-tokens 20000000

# 2. FIRST: prove the node's throughput before committing to a long run. Nothing here is pre-measured:
#    GPU tok/s is unknown until the node prints it.
python train_lm.py --corpus corpus --dim 256 --layers 4 --ctx 256 --batch 16 --steps 200 --eval-every 100

# 3. real run, house LR
python train_lm.py --corpus corpus --dim 768 --layers 12 --ctx 512 --batch 32 --steps 40000 \
    --lr 3e-5 --clip 0.5 --eval-every 500 --device cuda --out runs/lm768
```

Step 2 is not optional. The measured CPU numbers say nothing about the node, and a 40,000-step run launched on an
assumed throughput is how a cheap job becomes an expensive one. Read `summary.json` from step 2, multiply, decide.

## Budget arithmetic

`tok_per_s` from step 2 gives everything:

```
epoch_tokens = meta.json est_tokens
epoch_hours  = epoch_tokens / tok_per_s / 3600
total_hours  = epoch_hours * epochs * (params_scale_factor)
```

For reference only — **these are CPU measurements from a 3900X, not node figures**:

| model | CPU tok/s | 77 M tokens, 1 epoch |
|---|---|---|
| 58 M params | 1,148 | 18.6 h |
| 106 M params | 694 | 30.8 h |
| 217 M params | 331 | 64.7 h |

## What the model is for

Not conversation. The useful target is **code comprehension on this platform**: given a module, predict its intent,
its interfaces, and whether a proposed edit breaks a contract. That is the axis the gate work already uses, and it is
the one where a small model trained on 20 M tokens of *our* code can beat a 1.5B general model that has never seen
`council_ssm_pipeline.rs`.

Two honest limits to carry into the run:

1. **The corpus is not a label set.** Pretraining teaches style and structure, not correctness. Any claim about the
   model's judgement still has to clear a held-out benchmark built from real failures, the way
   `tools/signal_hunt.py` does with its positive and null controls.
2. **Byte-level vocabularies are wasteful on code.** The default here is bytes (vocab 260) so the scripts run with no
   extra dependency; on the node, swap in a real tokenizer and re-measure — a 32 k code vocabulary cuts sequence
   length roughly 3x, which is a bigger win than most architecture tweaks.

## Getting the result back

The node returns a checkpoint (`.pt`) and `summary.json`. Conversion to a servable artifact is deliberately out of
scope here: what the platform consumes today is the embeddings + linear-head path (`tools/train_scope_gate.py`), which
needs no LM at all. A trained LM earns its place only when it beats that path on a measured task.
