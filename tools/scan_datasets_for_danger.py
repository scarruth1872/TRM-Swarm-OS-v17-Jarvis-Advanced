"""Inventory every dataset-ish file on disk and rank it by DANGER-bearing content.

Why: the K3 gate's danger class is 768 template variants of "delete the production
database" plus 15 hand-written examples, and the curriculum corpus contains no
harmful instruction at all (measured: 0 BLOCK in 3,000 prompts). So the only way to
improve the gate is to find real destructive-instruction material. This scans what
actually exists instead of trusting the "688 datasets" figure that appears in agent
boilerplate - it is a claim in generated text, not a manifest.

Method (all measured, nothing inferred from filenames):
  * every .jsonl / .json / .csv file over MIN_BYTES, skipping .git, node_modules,
    target, dist, .venv
  * up to SAMPLE_BYTES per file, decoded as text
  * danger_verbs: word-boundary matches from a destructive-act vocabulary
  * danger_objects: production/audit/secret object words
  * a line counts as DANGER-BEARING only when it has BOTH - a verb and an object -
    because "delete" alone appears in every benign cleanup prompt (and "zero-day"
    alone made 249 false positives in an earlier pass)
  * density is danger-bearing lines per 1,000 sampled lines, so a small curated file
    is not buried by a 40 MB log

Output: evidence/DATASET_DANGER_INVENTORY.json plus a ranked table on stdout.
"""
import argparse
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SKIP_DIRS = {".git", "node_modules", "target", "dist", ".venv", "venv", "__pycache__",
             "chroma", ".swarm"}
EXTS = {".jsonl", ".json", ".csv"}
MIN_BYTES = 50_000
SAMPLE_BYTES = 4_000_000

DANGER_VERBS = ("delete", "wipe", "destroy", "drop", "truncate", "erase", "purge",
                "overwrite", "unlink", "nuke", "obliterate", "burn", "shred", "scrub",
                "rm -rf", "kill", "terminate", "disable", "bypass", "skip", "strip",
                "exfiltrate", "leak", "steal", "grant root", "grant admin", "revoke",
                "flush", "detach", "reinstall", "hand over", "email", "mail")
DANGER_OBJECTS = ("production", "prod", "database", "ledger", "audit", "backup",
                  "archive", "key", "keys", "secret", "credential", "password",
                  "watchdog", "auth", "monitoring", "pager", "replica", "customer",
                  "tenant", "invoice", "evidence", "provenance", "firewall", "ids")

VERB_RE = re.compile(r"(?<![a-z0-9])(" + "|".join(re.escape(v) for v in DANGER_VERBS) + r")(?![a-z0-9])")
OBJ_RE = re.compile(r"(?<![a-z0-9])(" + "|".join(re.escape(o) for o in DANGER_OBJECTS) + r")(?![a-z0-9])")


def walk():
    for p in ROOT.rglob("*"):
        try:
            if p.is_dir() or p.suffix.lower() not in EXTS:
                continue
            parts = set(part.lower() for part in p.parts)
            if parts & SKIP_DIRS:
                continue
            if p.stat().st_size < MIN_BYTES:
                continue
            yield p
        except OSError:
            continue


def scan(path):
    try:
        raw = path.open("rb").read(SAMPLE_BYTES)
    except OSError as e:
        return {"path": str(path.relative_to(ROOT)), "error": str(e)}
    text = raw.decode("utf-8", "replace")
    lines = text.splitlines()
    hits = 0
    examples = []
    for line in lines:
        low = line.lower()
        if VERB_RE.search(low) and OBJ_RE.search(low):
            hits += 1
            if len(examples) < 2:
                snippet = low.strip()
                examples.append(snippet[:180])
    n = max(1, len(lines))
    return {
        "path": str(path.relative_to(ROOT)),
        "bytes": path.stat().st_size,
        "sampled_lines": len(lines),
        "truncated": path.stat().st_size > SAMPLE_BYTES,
        "danger_lines": hits,
        "density_per_1k": round(1000.0 * hits / n, 2),
        "examples": examples,
    }


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--min-hits", type=int, default=1)
    ap.add_argument("--out", default=str(ROOT / "evidence" / "DATASET_DANGER_INVENTORY.json"))
    args = ap.parse_args()

    rows = []
    n = 0
    for p in walk():
        n += 1
        rows.append(scan(p))
        if n % 25 == 0:
            print(f"  scanned {n} files...", flush=True)

    ok = [r for r in rows if "error" not in r]
    ok.sort(key=lambda r: (r["danger_lines"], r["density_per_1k"]), reverse=True)
    ranked = [r for r in ok if r["danger_lines"] >= args.min_hits]

    print(f"\nscanned {len(ok)} dataset-ish files "
          f"({sum(r['bytes'] for r in ok) / 1e6:.1f} MB, "
          f"{(sum(1 for r in ok if r['truncated']))} truncated to 4 MB)")
    print(f"{'danger':>7s} {'/1k':>6s} {'lines':>7s}  file")
    for r in ranked[:30]:
        print(f"{r['danger_lines']:7d} {r['density_per_1k']:6.2f} {r['sampled_lines']:7d}  {r['path']}")

    print("\ntop examples of danger-bearing lines")
    for r in ranked[:6]:
        for ex in r["examples"][:1]:
            print(f"  [{r['path']}] {ex[:150]}")

    Path(args.out).write_text(json.dumps({
        "files_scanned": len(ok),
        "total_bytes": sum(r["bytes"] for r in ok),
        "method": "danger line = destructive verb AND danger object, word-boundary, "
                  "first 4 MB per file",
        "ranked": ranked,
    }, indent=2), encoding="utf-8")
    print(f"\nevidence -> {args.out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
