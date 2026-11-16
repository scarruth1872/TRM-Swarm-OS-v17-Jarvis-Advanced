# Embeddings for meaning - measured, not assumed

Date 2026-10-30. nomic-embed-text served at :8593 via llama.cpp --embeddings.

## What is present locally

| model | blob | size |
|---|---|---|
| nomic-embed-text | 970aa74c | 274.3 MB |
| mxbai-embed-large | 819c2adf | present |
| embeddinggemma | 0800cbac | present |

Verified working: 116/120 ARC tasks embedded, 768-dim.

## Does text embedding capture ARC transformation meaning?

Method: serialize each task (dims + first train pair grids), embed,
take top-1 nearest neighbour, check structural agreement. 20,000-sample
random baseline for comparison.

| property | random | embedding top-1 | lift |
|---|---|---|---|
| same output shape | 4.1% | 44.8% | 10.9x |
| same colour set | 1.7% | 6.0% | 3.5x |

## Honest reading

Real signal exists - 10.9x and 3.5x over chance is not noise.
But the shape lift is INFLATED: output dimensions are literally in the
serialized text, so the embedder can match them lexically, not semantically.
The colour-set result (6.0% absolute) is the truer measure, and it is poor:
the embedding rarely finds another task that transforms colour the same way.

Conclusion: text embeddings retrieve SURFACE structure, not transformation
semantics. They are the wrong encoder for grid reasoning. A grid-native
encoder (CNN/ViT over cells) would be required for meaning-level matching.

## Where embeddings genuinely work today

The chat memory path: TurboVec native backend, 25,031 entries, wired into
voice.speak() in step 2. That is the correct use - retrieving prior context
for language, where token co-occurrence IS the meaning.
