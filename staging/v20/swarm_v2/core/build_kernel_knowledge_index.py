"""Build a kernel knowledge index from real documents using the native model-free encoder,
so the platform answers from its own memory rather than from an external Ollama model call.

Why this exists: `swarm_v2_artifacts/semantic_index.json` and `kernel_knowledge_index.json`
must share the exact same encoder space as the query vectors used at runtime.
By using the native, sovereign model-free encoder (TurboVec-HDC), retrieval operates
entirely offline at microsecond latency with zero external dependencies.

Run:  python swarm_v2/core/build_kernel_knowledge_index.py --model native-turbovec-hdc
"""
from __future__ import annotations

import argparse
import json
import os
import sys
import time
import urllib.error
import urllib.request

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, ROOT)
OLLAMA = "http://127.0.0.1:11434"
SKIP_DIRS = {"node_modules", ".git", "target", "dist", "build", "__pycache__",
             ".venv", "venv", ".mypy_cache", ".pytest_cache"}


def ollama_models() -> list:
    try:
        with urllib.request.urlopen(OLLAMA + "/api/tags", timeout=5) as r:
            return [m["name"] for m in json.load(r).get("models", [])]
    except Exception:
        return []


def embed_ollama(text: str, model: str, attempts: int = 4) -> list:
    """Embed text via Ollama HTTP API (legacy fallback)."""
    for _ in range(attempts):
        body = json.dumps({"model": model, "prompt": text}).encode()
        req = urllib.request.Request(OLLAMA + "/api/embeddings", data=body,
                                     headers={"Content-Type": "application/json"})
        try:
            with urllib.request.urlopen(req, timeout=120) as r:
                return json.load(r)["embedding"]
        except urllib.error.HTTPError as ex:
            if ex.code >= 500 and len(text) > 600:
                text = text[: len(text) // 2]
                continue
            raise
    raise RuntimeError(f"embedding failed after {attempts} attempts (input truncated to {len(text)} chars)")


def resolve_key(key: str, fidx: dict) -> str | None:
    """Resolve an index key, which may be a repo-relative path or a bare filename."""
    norm = key.replace("\\", "/").lstrip("/")
    direct = os.path.join(ROOT, norm)
    if os.path.isfile(direct):
        return direct
    base = os.path.basename(norm)
    if base in fidx:
        return fidx[base]
    stem = base.rsplit(".", 1)[0]
    for f, p in fidx.items():
        if f.startswith(stem[:32]):
            return p
    return None


def file_index(roots: list) -> dict:
    out = {}
    for root in roots:
        for base, dirs, files in os.walk(root):
            dirs[:] = [d for d in dirs if d not in SKIP_DIRS]
            for f in files:
                out.setdefault(f, os.path.join(base, f))
    return out


def read_doc(path: str, limit: int) -> str:
    try:
        with open(path, encoding="utf-8", errors="replace") as fh:
            return fh.read(limit)
    except OSError:
        return ""


def _gpu_batch_encode(resolved, args, out, failed) -> bool:
    """Batch-encode the resolved docs via the kernel's `encode-batch` CLI (GPU accumulation, with a
    CPU fallback inside the kernel). Populates `out` and returns True when every doc got a vector;
    returns False on any failure so the caller falls back to the per-doc CPU loop. The vectors the
    CLI emits are bit-exact against `rust_native_encode_text_f32`, so this path cannot change the
    encoder space."""
    import subprocess
    import tempfile

    bin_path = os.path.join(ROOT, "trm_uefi_kernel", "host_council_server",
                            "target", "release", "host_council_server.exe")
    if not os.path.isfile(bin_path):
        return False
    fd_in, in_path = tempfile.mkstemp(suffix=".jsonl", prefix="trm_enc_in_")
    fd_out, out_path = tempfile.mkstemp(suffix=".jsonl", prefix="trm_enc_out_")
    os.close(fd_in)
    os.close(fd_out)
    try:
        with open(in_path, "w", encoding="utf-8") as fh:
            for key, text, _path in resolved:
                fh.write(json.dumps({"key": key, "text": text}) + "\n")
        rc = subprocess.run(
            [bin_path, "encode-batch", "--input", in_path, "--output", out_path,
             "--batch", str(args.gpu_batch)],
            capture_output=True, text=True, timeout=3600,
        )
        if rc.returncode != 0:
            print(f"  encode-batch exited {rc.returncode}; falling back to CPU encoder", file=sys.stderr)
            return False
        vecs = {}
        with open(out_path, encoding="utf-8") as fh:
            for line in fh:
                d = json.loads(line)
                vecs[d["key"]] = d["embedding"]
        for key, text, path in resolved:
            if key not in vecs:
                failed.append({"key": key, "error": "encode-batch returned no vector"})
                continue
            out[key] = {
                "embedding": vecs[key],
                "metadata": {
                    "model": args.model,
                    "source_path": os.path.relpath(path, ROOT),
                    "chars_embedded": len(text),
                    "created_by": "build_kernel_knowledge_index",
                    "sovereign_zero_llm": True,
                    "encoder": "gpu-batch (native-turbovec-hdc)",
                },
                "indexed_at": int(time.time()),
            }
        return True
    except Exception as ex:  # noqa: BLE001
        print(f"  GPU batch path failed ({type(ex).__name__}: {ex}); falling back to CPU encoder", file=sys.stderr)
        return False
    finally:
        for p in (in_path, out_path):
            try:
                os.unlink(p)
            except OSError:
                pass


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--model", default="native-turbovec-hdc",
                    help="native-turbovec-hdc (default sovereign model-free) or Ollama model name")
    ap.add_argument("--source-index", default="swarm_v2_artifacts/semantic_index.json",
                    help="keys define WHICH documents to index")
    ap.add_argument("--out", default="swarm_v2_artifacts/kernel_knowledge_index.json")
    ap.add_argument("--chars", type=int, default=6000, help="characters of each doc to embed")
    ap.add_argument("--dim", type=int, default=768,
                    help="embedding width; the corpus and the route must agree")
    ap.add_argument("--limit", type=int, default=0, help="0 = all")
    ap.add_argument("--gpu", action="store_true",
                    help="batch-encode via the kernel's encode-batch CLI (GPU accumulation; CPU fallback)")
    ap.add_argument("--gpu-batch", type=int, default=256,
                    help="docs per GPU dispatch (encode-batch --batch)")
    args = ap.parse_args()

    is_native = (args.model == "native-turbovec-hdc" or args.model == "native")

    if is_native:
        from swarm_v2.skills.embedding_skill import NativeEmbeddingSkill
        native_skill = NativeEmbeddingSkill(embedding_dim=args.dim)
        embed_fn = native_skill.embed_text
        print(f"Using Sovereign Native Model-Free Encoder: {native_skill.get_embedding_stats()}", flush=True)
    else:
        available = ollama_models()
        if not any(m.startswith(args.model) for m in available):
            print(f"model {args.model} is not installed in Ollama; available: {available[:20]}", file=sys.stderr)
            return 2
        embed_fn = lambda text: embed_ollama(text, args.model)

    src_path = os.path.join(ROOT, args.source_index)
    if os.path.exists(src_path):
        src = json.load(open(src_path, encoding="utf-8"))
        keys = list(src.keys())
    else:
        print(f"Source index not found at {src_path}, scanning markdown files...", flush=True)
        keys = []

    if not keys:
        fidx = file_index([os.path.join(ROOT, "swarm_v2", "knowledge")])
        keys = [f for f in fidx.keys() if f.endswith(".md") or f.endswith(".rs") or f.endswith(".py")][:80]

    if args.limit:
        keys = keys[:args.limit]
    print(f"documents to index: {len(keys)} | model: {args.model} | chars/doc: {args.chars}", flush=True)

    # The fallback index is deliberately BOUNDED. os.walk(ROOT) here was unbounded: on a WSL /mnt/c mount
    # with multi-GB artifacts beside the tree, each build re-walked everything and never returned, and a
    # stem-prefix match anywhere in the tree can bind a key to an unrelated file. Keys resolve by their own
    # repo-relative path first; the fallback only looks inside the harvested corpus.
    fidx = file_index([os.path.join(ROOT, "swarm_v2", "knowledge")])
    out, dim, missing, failed = {}, None, [], []
    t_start = time.perf_counter()

    # Resolve + read every document ONCE so the GPU batch path and the per-doc CPU loop share the
    # same (key, text, path) triples and can never disagree about which docs are missing.
    resolved = []
    for key in keys:
        path = resolve_key(key, fidx)
        if not path:
            missing.append(key)
            continue
        text = read_doc(path, args.chars)
        if not text.strip():
            missing.append(key)
            continue
        resolved.append((key, text, path))

    gpu_used = False
    if args.gpu and is_native and resolved:
        gpu_used = _gpu_batch_encode(resolved, args, out, failed)
        if gpu_used:
            dim = len(out[next(iter(out))]["embedding"]) if out else None
            print(f"  [GPU] batch-encoded {len(out)} documents", flush=True)

    if not gpu_used:
        for n, (key, text, path) in enumerate(resolved, 1):
            try:
                vec = embed_fn(text)
            except Exception as ex:  # noqa: BLE001
                failed.append({"key": key, "error": f"{type(ex).__name__}: {ex}"})
                continue
            dim = len(vec)
            out[key] = {
                "embedding": vec,
                "metadata": {
                    "model": args.model,
                    "source_path": os.path.relpath(path, ROOT),
                    "chars_embedded": len(text),
                    "created_by": "build_kernel_knowledge_index",
                    "sovereign_zero_llm": is_native
                },
                "indexed_at": int(time.time()),
            }
            if n % 10 == 0 or n == len(resolved):
                print(f"  [{n}/{len(resolved)}] embedded {key[:48]}", flush=True)

    elapsed = time.perf_counter() - t_start
    payload = {
        "meta": {
            "model": args.model,
            "dim": dim,
            "documents": len(out),
            "missing": missing,
            "failed": failed,
            "built_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
            "elapsed_s": round(elapsed, 3),
            "docs_per_s": round(len(out) / max(elapsed, 1e-6), 3),
            "chars_per_doc": args.chars,
            "sovereign_zero_llm": is_native,
            "monad_anchor": "0x4D4F4E4144000001",
        },
        "index": out,
    }

    out_path = os.path.join(ROOT, args.out)
    os.makedirs(os.path.dirname(out_path), exist_ok=True)
    with open(out_path, "w", encoding="utf-8") as fh:
        json.dump(payload, fh, indent=2)

    # Also update semantic_index.json to keep both synchronized
    if args.out == "swarm_v2_artifacts/kernel_knowledge_index.json":
        sem_path = os.path.join(ROOT, "swarm_v2_artifacts", "semantic_index.json")
        try:
            with open(sem_path, "w", encoding="utf-8") as fh:
                json.dump(out, fh, indent=2)
            print(f"Synchronized semantic_index.json with {len(out)} native vectors.", flush=True)
        except Exception as e:
            print(f"Warning: could not sync semantic_index.json: {e}", flush=True)

    print(f"wrote {len(out)} documents to {out_path} in {elapsed:.3f}s ({len(out)/max(elapsed, 1e-6):.1f} docs/s)")
    if missing:
        print(f"  MISSING ({len(missing)}): {missing[:5]}")
    if failed:
        print(f"  FAILED ({len(failed)}): {failed[:5]}")
    return 0 if not failed else 1


if __name__ == "__main__":
    sys.exit(main())
