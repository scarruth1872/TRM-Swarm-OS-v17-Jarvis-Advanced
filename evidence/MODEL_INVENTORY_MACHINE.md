# Machine-wide model inventory + exact-match analysis

Date: 2026-10-30. Measured on this box.

## Correction to my earlier inventory

My earlier scan (`MODEL_INVENTORY.txt`) covered only three repo roots and found 313 artifacts / 64.63 GB. It **missed the single largest store**: `~/.ollama/models/blobs` = **66.2 GB**, 45 model manifests. That omission is now fixed here.

## Drives

```
C  used 869.2 GB  free  61.3 GB
D  used 1483  GB  free 2243  GB
E  used  162  GB  free  304  GB
F  used 423.7 GB  free  41.3 GB
G  used  2.4  GB  free  12.2 GB
J  used 298.7 GB  free  633  GB
```

## Ollama (45 manifests, 66.2 GB)

Locally present, runnable shapes:
`bonsai-27b`, `gemma3 270m/1b/4b`, `gemma3n`, `gemma4 12b`, `gemma2 2b`, `llama3.2`,
`llava`, `phi3`, `olmo-3`, `qwen2.5-coder`, `qwen3-embedding 0.6b/4b`,
`deepseek-r1 1.5b/8b`, `deepseek-coder 1.3b`, `functiongemma 270m`,
`starcoder2 3b`, `tinyllama`, `embeddinggemma`, `nomic-embed-text`,
`mxbai-embed-large`, `bitnet-b1.58-2B-4T`.

Cloud-tagged (NOT on disk): `deepseek-v3.1 671b-cloud`, `deepseek-v3.2/cloud`,
`deepseek-v4-pro/cloud`, `glm-4.6/cloud`, `glm-4.7/cloud`, `kimi-k2.5/cloud`,
`minimax-m2/cloud`, `gpt-oss 20b-cloud/120b-cloud`, `qwen3-coder 480b-cloud`,
`qwen3-vl 235b-cloud`, `nemotron-3-nano 30b-cloud`, `gemini-3-flash-preview/cloud`.

HF-cached: `TeichAI/Qwen3-8B-Gemini-3-Pro-Preview-Distill-GGUF Q4_K_M`,
`yuxinlu1/gemma-4-12B-coder-...-GGUF Q4_K_M`.

## The ternary claim does not survive measurement

`hf.co/microsoft/bitnet-b1.58-2B-4T-gguf` — 1.188 GB blob.

1. **It is not 1.58-bit.** The tensor type is `IQ4_NL_4_4`, a ~4.5-bit quantization.
   1.188 GB / 2B params = **4.75 bits/param**. Native b1.58 would be ~0.4 GB.
   The name is inherited from the upstream model; the file is a 4-bit requant.
2. **It cannot load at all** on this build:
   ```
   tensor 'blk.0.ffn_down.weight' of type 36
   (TYPE_IQ4_NL_4_4 REMOVED, use IQ4_NL with runtime repacking)
   llama_model_load: error loading model
   ```

So: **this machine has no runnable native-ternary model.** The working ternary-ish
artifact is `Bonsai-27B-Q1_0.gguf` (1-bit, verified generating correctly on :8591).

## Why 70.3% foreground accuracy yields 2.1% exact — it is arithmetic

ARC exact-match is a conjunction over every foreground cell. Treating cells as
independent, `P(exact) ≈ fg_acc ^ fg_cells`.

Measured over the 47-task sweep (`SWEEP50_RESULTS.json`):

```
predicted exact solves = 1.083 / 47
actual   exact solves  = 1     / 47
```

The model does exactly what its per-cell accuracy implies. Worked examples:

```
fg_cells  fg_acc   P(exact)     actual
    36    1.000    1.0e+00      TRUE     <- the one success
   169    0.923    3.4e-04      false    (~13 cells wrong)
   400    0.917*    ~0          false    (70.3% fg -> 33 cells erased)
     5    0.600    7.8e-02      false    (only 3 cells wrong, still not exact)
```

*\*cell accuracy shown for the 400-cell task; its foreground accuracy is 0.703.*

Consequence: on a grid with ~100 foreground cells, 99% per-cell accuracy still
yields only ~37% exact. To move the exact rate, per-cell error must approach zero —
the payoff is exponential, and the 2.1% is not evidence of a separate defect.

Corollary for effort: **small-foreground-cell tasks are where exact solves are
winnable.** A 5-fg-cell task at 60% per-cell has a 7.8% chance; at 90% it has 59%.

## Honest limits

- Independence is an approximation: cell errors are spatially correlated, so the
  prediction is a baseline, not an identity. It landed within 0.1 of the measured
  rate, which is closer than the approximation deserves.
- 47 tasks. Rates are indicative, not established.
- The 70.3% figure is **one task's** foreground accuracy; 29.4% is the 47-task mean.
  Quoting 70.3% against a 2.1% whole-sweep exact rate mixes denominators.
