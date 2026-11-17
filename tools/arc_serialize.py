"""arc_serialize.py - one place that turns ARC npy rows into text.

Both the serial ingest (`k3_ingest_real.py --source arc`) and the parallel one (`ingest_parallel.py --mode arc`)
import from here. Two copies of a serialiser would drift, and a drifted serialiser produces a different corpus
while still reporting success - the failure mode this project keeps having to un-learn.

Encoding facts, measured rather than assumed (G205/G206):
  * inputs/labels are (rows, 900) uint8: 900 = 30x30 grid.
  * token 1..10 = ARC colour 0..9; token 0 = padding; token 11 = an extra vocabulary entry whose meaning is NOT
    established here and is rendered 'x' rather than guessed.
  * each row is one example input; the row at the same index in all__labels is its output.
  * all__puzzle_indices gives each puzzle's row range; all__puzzle_identifiers its ids.
"""
import os
import numpy as np

DIM_ROWS = 30
DIM_COLS = 30
SEQ = DIM_ROWS * DIM_COLS


def render_grid(row) -> str:
    """Crop to the non-pad bounding box. Pad (0) and colour 0 (token 1) are distinguishable, so the crop is
    exact rather than a guess."""
    grid = np.asarray(row, dtype=np.uint8).reshape(DIM_ROWS, DIM_COLS)
    rows = [r for r in grid if any(v != 0 for v in r)]
    if not rows:
        return "(empty)"
    cols = [c for c in range(DIM_COLS) if any(r[c] != 0 for r in rows)]
    c0, c1 = min(cols), max(cols)
    out = []
    for r in rows:
        out.append("".join((str(v - 1) if 1 <= v <= 10 else "x") for v in r[c0:c1 + 1]))
    return f"{len(out)}x{c1 - c0 + 1}:" + "/".join(out)


def puzzle_text(split, pid, inp, lab, lo, hi) -> str:
    parts = []
    for k, row_i in enumerate(range(lo, hi)):
        parts.append(f"example {k + 1} input {render_grid(inp[row_i])} output {render_grid(lab[row_i])}")
    return (f"ARC-AGI puzzle {split} number {int(pid)} with {hi - lo} examples: " + "; ".join(parts))


class ArcSplit:
    """Lazy handle on one split's arrays; safe to construct inside a worker (mmap, no full read)."""

    def __init__(self, root: str, split: str):
        d = os.path.join(root, split)
        self.split = split
        self.dir = d
        self.inputs = np.load(os.path.join(d, "all__inputs.npy"), mmap_mode="r")
        self.labels = np.load(os.path.join(d, "all__labels.npy"), mmap_mode="r")
        self.indices = np.load(os.path.join(d, "all__puzzle_indices.npy"))
        self.ids = np.load(os.path.join(d, "all__puzzle_identifiers.npy"))

    def __len__(self):
        return len(self.ids)

    def record(self, i: int):
        lo, hi = int(self.indices[i]), int(self.indices[i + 1])
        pid = int(self.ids[i])
        return f"arc/{self.split}/{pid}", puzzle_text(self.split, pid, self.inputs, self.labels, lo, hi)
