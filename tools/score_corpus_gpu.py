"""Score a directive corpus through the merged GPU pipeline (one device, one submit per batch).

WHY THIS EXISTS
---------------
The council scores one text at a time over HTTP, which the GPU cannot help with: at 15 items a
dispatch is pure launch cost. The GPU pays only when a whole corpus arrives at once, so this tool
hands the binary a file and lets it batch.

Parity is not assumed: the binary scores every text twice, once on the GPU (stages 1-8 in Vulkan)
and once with `K3Head::forward_logits` on the CPU, and reports the number of per-head trit flips
and per-text vote mismatches. Any non-zero flip count means the pipeline is not trustworthy and the
numbers below are worthless -- check that first.

Interpreting the votes: the roster vote is the sum of 15 head trits, so -1 means the roster rejected
the directive. That is a *signal*, not the council's verdict -- the deployed council uses the lexical
frame gate (G51) for blocking, and the K3 heads have repeatedly measured near majority on held-out
sets (G45/G50). Recall/false-block figures are reported because they are cheap to print, not because
this pipeline decides anything.

Usage:
    python tools/score_corpus_gpu.py evidence/K3_SYNTH_CORPUS.jsonl
    python tools/score_corpus_gpu.py evidence/BLIND_DIRECTIVES_3.jsonl --no-labels
"""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

BIN = "trm_uefi_kernel/tools/gpu_ssm_parity/target/release/gpu-ssm-parity.exe"
OUT = "scratch/gpu_scores.jsonl"


def main() -> int:
    if len(sys.argv) < 2:
        print(__doc__)
        return 2
    corpus = sys.argv[1]
    labelled = "--no-labels" not in sys.argv

    root = Path(__file__).resolve().parent.parent
    exe = root / BIN
    if not exe.exists():
        print(f"missing binary: {exe}\nbuild: cargo build --release in trm_uefi_kernel/tools/gpu_ssm_parity")
        return 2

    out = root / OUT
    out.parent.mkdir(parents=True, exist_ok=True)

    proc = subprocess.run(
        [str(exe), "--score", corpus, "--out", str(out)],
        cwd=str(root), capture_output=True, text=True, timeout=1800,
    )
    summary = None
    for line in proc.stdout.splitlines():
        line = line.strip()
        if line.startswith("{") and '"stage":"score_corpus"' in line.replace(" ", ""):
            summary = json.loads(line)
    if summary is None:
        print("binary produced no summary")
        print(proc.stdout[-2000:])
        print(proc.stderr[-2000:])
        return 1

    print("pipeline (texts, all 15 heads)")
    print(f"  texts            {summary['texts']}")
    print(f"  items            {summary['items']}")
    print(f"  GPU wall         {summary['gpu_s']:.3f} s")
    print(f"    hash           {summary['input_s']:.3f} s   (CPU tokenise; embedding is on the GPU)")
    print(f"    gpu forward    {summary['gpu_forward_s']:.3f} s")
    print(f"  CPU wall         {summary['cpu_s']:.3f} s")
    print(f"  end-to-end       {summary['speedup']:.2f}x      {summary['gpu_texts_per_s']:.0f} vs {summary['cpu_texts_per_s']:.0f} texts/s")
    print(f"  featuriser       {summary['featurise_bits_differ']}/{summary['featurise_compared']} values differ from CPU, max abs {summary['featurise_max_abs']:.3e}")
    print(f"  PARITY           {summary['trit_flips']} trit flips, {summary['vote_mismatches']} vote mismatches")
    if summary["trit_flips"] or summary["vote_mismatches"]:
        print("  REFUSING to report decision numbers: parity failed.")
        return 1

    if not labelled:
        return 0

    rows = [json.loads(l) for l in Path(root / corpus).read_text(encoding="utf-8").splitlines() if l.strip()]
    scored = [json.loads(l) for l in out.read_text(encoding="utf-8").splitlines() if l.strip()]
    if len(rows) != len(scored):
        print(f"row mismatch: {len(rows)} in corpus vs {len(scored)} scored")
        return 1

    # label convention in K3_SYNTH_CORPUS: 0 = danger, 1 = safe twin, 2 = unrelated twin
    buckets: dict[int, list[int]] = {}
    for r, s in zip(rows, scored):
        lab = r.get("label")
        if isinstance(lab, str):
            lab = {"danger": 0, "safe": 1}.get(lab.lower())
        if lab is None:
            continue
        buckets.setdefault(int(lab), []).append(s["vote"])

    print("\nroster vote by label (vote -1 = roster rejected)")
    for lab in sorted(buckets):
        votes = buckets[lab]
        rej = sum(1 for v in votes if v == -1)
        name = {0: "danger", 1: "safe", 2: "twin/other"}.get(lab, str(lab))
        print(f"  {name:<12} n={len(votes):<5} rejected {rej:>4} ({100.0 * rej / len(votes):.1f}%)")

    d, s = buckets.get(0), buckets.get(1)
    if d and s:
        rec = sum(1 for v in d if v == -1) / len(d)
        fp = sum(1 for v in s if v == -1) / len(s)
        print(f"\n  danger recall {rec:.3f}   safe false-block {fp:.3f}   balanced {(rec + (1 - fp)) / 2:.3f}")
        print("  (descriptive only -- the K3 heads are not the deployed blocker)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
