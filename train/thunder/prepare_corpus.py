"""Build a training corpus from the TRM platform tree, for an off-box GPU node.

Why a subset and not everything: measured on this host, 77.2 M first-party tokens at 1,148 tok/s (58 M params, CPU)
is 18.6 h per epoch (G138). A rented GPU makes the full corpus affordable, but 77 M tokens is still mostly vendored
headers and generated JSON, so the default selection keeps source only and strips the trees that would teach the model
our dependencies instead of our platform.

Deterministic: same tree in, same shards out (sorted walk, content hash dedupe).

    python prepare_corpus.py --out corpus/ --max-tokens 20000000
    -> corpus/train.bin, corpus/val.bin, corpus/meta.json
"""
from __future__ import annotations

import argparse
import hashlib
import json
import random
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]

EXCLUDE_DIRS = {
    ".git", "node_modules", "target", "__pycache__", ".venv", "venv", "dist", "build", "scratch",
    "models", "uv3", "Jarvis-Advanced-main", "trm-sovereign-portal", "openfang", ".vite", ".next",
}
EXCLUDE_SUFFIX = {".json", ".jsonl", ".lock", ".bin", ".gguf", ".safetensors", ".png", ".jpg", ".pdf", ".zip",
                  ".mp3", ".wav", ".ico", ".woff", ".woff2", ".ttf", ".so", ".dll", ".exe", ".o", ".a"}
KEEP_SUFFIX = {".py", ".rs", ".ts", ".tsx", ".js", ".jsx", ".svelte", ".vue", ".c", ".h", ".cpp", ".hpp",
               ".go", ".java", ".css", ".html", ".md", ".toml", ".yaml", ".yml", ".sh", ".bat", ".ps1", ".sql"}
# Files first: these are the modules that define the platform's behaviour, so they should not be the ones truncated.
PRIORITY_PREFIX = ("trm_uefi_kernel/", "swarm_v2/", "tools/", "crates/", "agents/", "council", "docs/")


def walk(root: Path):
    for p in sorted(root.rglob("*")):
        if not p.is_file():
            continue
        parts = set(p.relative_to(root).parts)
        if parts & EXCLUDE_DIRS:
            continue
        if p.suffix.lower() not in KEEP_SUFFIX or p.suffix.lower() in EXCLUDE_SUFFIX:
            continue
        yield p


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", default=str(ROOT))
    ap.add_argument("--out", default="corpus")
    ap.add_argument("--max-tokens", type=int, default=20_000_000)
    ap.add_argument("--val-frac", type=float, default=0.02)
    ap.add_argument("--chars-per-token", type=float, default=3.6, help="for the budget estimate only")
    args = ap.parse_args()

    root = Path(args.root).resolve()
    files = list(walk(root))
    # priority trees first, then everything else, both in sorted order for determinism
    def rank(p: Path):
        rel = str(p.relative_to(root)).replace("\\", "/")
        return (0 if rel.startswith(PRIORITY_PREFIX) else 1, rel)
    files.sort(key=rank)

    budget_chars = int(args.max_tokens * args.chars_per_token)
    seen: set[str] = set()
    docs: list[str] = []
    chars = 0
    per_tree: dict[str, int] = {}
    for p in files:
        try:
            text = p.read_text(encoding="utf-8", errors="strict")
        except (UnicodeDecodeError, OSError):
            continue
        if len(text) < 200:  # stubs and empty __init__ teach nothing
            continue
        h = hashlib.sha256(text.encode()).hexdigest()
        if h in seen:  # vendored copies of the same file
            continue
        seen.add(h)
        docs.append(text)
        chars += len(text)
        tree = str(p.relative_to(root)).split("/")[0].split("\\")[0]
        per_tree[tree] = per_tree.get(tree, 0) + len(text)
        if chars >= budget_chars:
            break

    random.Random(23).shuffle(docs)  # shuffle so val is not a whole tree
    n_val = max(1, int(len(docs) * args.val_frac))
    val, train = docs[:n_val], docs[n_val:]

    out = Path(args.out)
    out.mkdir(parents=True, exist_ok=True)
    # tokenisation happens on the GPU node (its tokenizer is the one the model will use); we ship text shards and
    # let train_lm.py tokenise once, cached.
    (out / "train.txt").write_text("\n\n<|file|>\n\n".join(train), encoding="utf-8")
    (out / "val.txt").write_text("\n\n<|file|>\n\n".join(val), encoding="utf-8")
    meta = {
        "files_used": len(docs),
        "train_files": len(train),
        "val_files": len(val),
        "chars": chars,
        "est_tokens": int(chars / args.chars_per_token),
        "max_tokens_budget": args.max_tokens,
        "per_tree_bytes": dict(sorted(per_tree.items(), key=lambda kv: -kv[1])[:15]),
        "note": "text shards; the node tokenises with its own vocabulary so the corpus is portable",
    }
    (out / "meta.json").write_text(json.dumps(meta, indent=2), encoding="utf-8")
    print(json.dumps(meta, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
