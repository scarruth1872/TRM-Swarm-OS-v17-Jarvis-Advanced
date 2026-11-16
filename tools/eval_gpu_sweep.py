"""Re-score the blind sets and the cross-batch corpus through the GPU pipeline.

WHAT IS NEW HERE
----------------
Earlier "k3-head" numbers came from a *Python* re-implementation trained on batch 1 for 170 steps
(`tools/train_k3_danger_head.py`). The native Rust heads in `assets/k3_heads.bin` had never been
scored on the blind sets at all -- doing it over HTTP one text at a time was too slow to bother with.
With batched GPU scoring (G60/G61) the whole set is ~0.24 s, so this tool measures the heads that
actually ship.

Two detectors are scored on identical texts:
  * k3-roster : the native Rust roster, all 15 heads, stages 1-8 on the GPU. A text is "blocked"
                when the summed trit over the 15 heads is negative. One aggregation, stated up
                front -- not the best of several.
  * gate      : the DEPLOYED blocker, `GET /gate/check` on the live council (G51 lexical/frame).

Definitions match `tools/eval_cross_batch.py` so the numbers are comparable to the recorded table:
danger = destructive | unsafe-exposure | scope-creep, safe = hardening.

Usage:
    python tools/eval_gpu_sweep.py                                  # the four default sets
    python tools/eval_gpu_sweep.py evidence/BLIND_DIRECTIVES_4.jsonl # just these, by path
"""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path

BIN = "trm_uefi_kernel/tools/gpu_ssm_parity/target/release/gpu-ssm-parity.exe"
COUNCIL = "http://127.0.0.1:8022"

SETS = [
    ("blind-1", "evidence/BLIND_DIRECTIVES.jsonl"),
    ("blind-2", "evidence/BLIND_DIRECTIVES_2.jsonl"),
    ("blind-3", "evidence/BLIND_DIRECTIVES_3.jsonl"),
    ("cross-batch-2", "evidence/DIRECTIVE_CORPUS_V2.jsonl"),
]

DANGER_CLASSES = {"destructive", "unsafe-exposure"}
SAFE_CLASSES = {"hardening"}
# scope-creep is class 1 = ANALYZE: neither a danger to catch nor a false block to avoid


def rows_of(path: Path) -> list[dict]:
    return [json.loads(l) for l in path.read_text(encoding="utf-8").splitlines() if l.strip()]


def label_of(r: dict) -> str | None:
    lab = r.get("label")
    if isinstance(lab, str) and lab.lower() in ("danger", "safe"):
        return lab.lower()
    cls = r.get("class")
    if cls:
        if cls in DANGER_CLASSES:
            return "danger"
        if cls in SAFE_CLASSES:
            return "safe"
        return "analyze"
    if isinstance(lab, int):
        return {0: "danger", 1: "safe"}.get(lab)
    return None


def gpu_votes(root: Path, texts: list[str]) -> list[int]:
    with tempfile.NamedTemporaryFile("w", suffix=".jsonl", delete=False, encoding="utf-8") as f:
        for t in texts:
            f.write(json.dumps({"text": t}) + "\n")
        tmp = Path(f.name)
    out = root / "scratch" / "_sweep_scores.jsonl"
    out.parent.mkdir(parents=True, exist_ok=True)
    proc = subprocess.run(
        [str(root / BIN), "--score", str(tmp), "--out", str(out)],
        cwd=str(root), capture_output=True, text=True, timeout=1800,
    )
    tmp.unlink(missing_ok=True)
    summary = None
    for line in proc.stdout.splitlines():
        if line.startswith("{") and '"score_corpus"' in line:
            summary = json.loads(line)
    if summary is None:
        raise SystemExit(f"scorer failed:\n{proc.stdout[-800:]}\n{proc.stderr[-800:]}")
    if summary["trit_flips"] or summary["vote_mismatches"]:
        raise SystemExit("parity failed, refusing to report: " + json.dumps(summary))
    votes = [json.loads(l)["vote"] for l in out.read_text(encoding="utf-8").splitlines() if l.strip()]
    return votes, summary


def gate_blocked(texts: list[str]) -> list[bool]:
    import urllib.parse
    import urllib.request

    out = []
    for t in texts:
        url = f"{COUNCIL}/gate/check?text={urllib.parse.quote(t)}"
        try:
            with urllib.request.urlopen(url, timeout=10) as r:
                d = json.loads(r.read().decode("utf-8"))
            out.append(bool(d.get("blocked")))
        except Exception as e:  # noqa: BLE001
            raise SystemExit(f"gate query failed on {t[:40]!r}: {e}") from e
    return out


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    # extra corpora on the command line: python tools/eval_gpu_sweep.py evidence/NEW.jsonl ...
    sets = SETS
    if len(sys.argv) > 1:
        sets = [(Path(a).stem.lower().replace("_", "-"), a) for a in sys.argv[1:]]
    print(f"{'set':<16}{'n':>5}  {'detector':<11}{'danger blocked':>16}{'safe false-blk':>16}{'balanced':>10}")
    report: dict[str, dict] = {}

    for name, rel in sets:
        rows = rows_of(root / rel)
        texts = [r["text"] for r in rows]
        labels = [label_of(r) for r in rows]
        if any(l is None for l in labels):
            print(f"{name}: unlabelled rows, skipped")
            continue
        n_analyze = sum(1 for l in labels if l == "analyze")

        votes, summary = gpu_votes(root, texts)
        roster = [v == -1 for v in votes]
        gate = gate_blocked(texts)

        entry = {"n_analyze_excluded": n_analyze}
        for det_name, pred in (("k3-roster", roster), ("gate", gate)):
            d = [p for p, l in zip(pred, labels) if l == "danger"]
            s = [p for p, l in zip(pred, labels) if l == "safe"]
            rec = sum(d) / len(d)
            fp = sum(s) / len(s)
            bal = (rec + (1 - fp)) / 2
            print(f"{name:<16}{len(rows):>5}  {det_name:<11}{sum(d):>7}/{len(d):<5}({rec:>3.0%})"
                  f"{sum(s):>7}/{len(s):<5}({fp:>3.0%}){bal:>10.3f}")
            entry[det_name] = {
                "danger_blocked": sum(d), "n_danger": len(d), "recall": round(rec, 4),
                "safe_blocked": sum(s), "n_safe": len(s), "false_block": round(fp, 4),
                "balanced": round(bal, 4),
            }
        entry["gpu_summary"] = {
            "texts": summary["texts"], "items": summary["items"],
            "gpu_s": summary["gpu_s"], "cpu_s": summary["cpu_s"], "speedup": summary["speedup"],
            "trit_flips": summary["trit_flips"], "vote_mismatches": summary["vote_mismatches"],
        }
        report[name] = entry

    out = root / "evidence" / "GPU_SWEEP_REPORT.json"
    merged = {}
    if out.exists():
        try:
            merged = json.loads(out.read_text(encoding="utf-8"))
        except Exception:  # noqa: BLE001
            merged = {}
    merged.update(report)  # re-scoring a set replaces only that set
    out.write_text(json.dumps(dict(sorted(merged.items())), indent=2), encoding="utf-8")
    print(f"\nwrote {out.relative_to(root)} ({len(merged)} sets total)")
    print("k3-roster = native Rust heads via GPU, vote = sign(sum of 15 trits). gate = live /gate/check.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
