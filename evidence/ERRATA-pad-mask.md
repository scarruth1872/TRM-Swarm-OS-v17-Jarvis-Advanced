# ERRATA — "86.7% of the distillation gradient was padding" was FALSE

Date: 2026-10-30
Status: **claim withdrawn.** The `IGNORE_LABEL_ID` change was reverted (`2209289`).

## What was claimed

That `models/losses.py` masked on `IGNORE_LABEL_ID = -100` while the dataset declares
`pad_id = 0`, so `valid_mask = (labels != -100)` was always true and ~90% of the
training gradient was spent on padding tokens. A change from `-100` to `0` was
committed (`b0a0a7c`) with the measured justification:

```
old mask counted 1,800,000 (100.00%)
new mask counted   238,990 ( 13.28%)
-> 86.7% of the gradient was padding
```

## Why it was wrong

The measurement was taken on the **raw `.npy` labels** and ignored a conversion step
in the data loader. `puzzle_dataset.py:152-153` performs:

```python
if self.metadata.ignore_label_id is not None:
    batch["labels"][batch["labels"] == self.metadata.ignore_label_id] = IGNORE_LABEL_ID
```

So the value the loss actually sees is `IGNORE_LABEL_ID`, not `0`. The labels are
`uint8`, so `-100` wraps to **156** — but it wraps consistently on *both* the
assignment and the comparison, so the mask still resolves correctly.

## Re-measurement (through the loader's conversion, 64 samples)

```
raw padding fraction            86.68%
OLD mask (IGNORE_LABEL_ID=-100) 13.32% of tokens counted
NEW mask (IGNORE_LABEL_ID=0)    13.32% of tokens counted
masks identical: True
```

**The two masks are byte-identical.** Padding was already excluded. There was no
padding-inflation bug, and no gradient was being wasted on padding.

## Consequence

- Padding is **not** the explanation for 2.1% exact / 29.4% foreground ARC accuracy.
- The distillation plan's premise ("padding dominates the loss") is **retracted**.
- `IGNORE_LABEL_ID` is back to `-100`; nothing changed functionally.

## What is still true, and is a real (latent) hazard

Relying on `uint8` wraparound (`-100` -> `156`) for a sentinel value is fragile:
if the label dtype ever becomes `int16`/`int32`, `-100` stores correctly, the
assignment still writes `-100`, the comparison still matches, and the mask keeps
working — but any code that inspects labels expecting a `156` sentinel breaks.
Conversely, if a future refactor stores `-100` in an `int8` tensor it saturates to
`-128`. This is a **correctness-by-coincidence** pattern worth hardening explicitly
(e.g. convert labels to `int32` in the loader and use `-100` as a true sentinel),
but it is not currently causing any measured defect, so no change was kept.

## Lesson applied

The original measurement was taken one layer too low in the stack. Mask correctness
must be measured **after** the loader's label conversion, not on raw dataset arrays.
