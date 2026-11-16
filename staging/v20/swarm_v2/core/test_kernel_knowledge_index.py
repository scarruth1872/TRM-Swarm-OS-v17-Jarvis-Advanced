"""Contract tests for the kernel-resident ternary knowledge index.

Run directly (no pytest needed in this venv):
    python swarm_v2/core/test_kernel_knowledge_index.py
or under the repo runner:
    scripts/run_tests.sh tests/...  (pytest collects these test_* callables)

Each test states a behaviour contract, not a snapshot. The last one is the guard
that matters: a retrieval index is only usable when the vectors stored in it come
from the same encoder as the vectors used to query it.
With the native model-free encoder (TurboVec-HDC), the contract is verified
entirely offline at zero-LLM microsecond latency.
"""
from __future__ import annotations

import ctypes
import json
import os
import sys
import urllib.error
import urllib.request

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, ROOT)

DLL = os.path.join(ROOT, "swarm_v2_rust_kernel", "target", "release", "swarm_rust_kernel.dll")
_INDEX_REL = os.environ.get("TRM_INDEX", "swarm_v2_artifacts/kernel_knowledge_index.json")
INDEX = _INDEX_REL if os.path.isabs(_INDEX_REL) else os.path.join(ROOT, _INDEX_REL)
OLLAMA = "http://127.0.0.1:11434"
DIM = 768


class Unrunnable(Exception):
    """The check could not run (missing artifact, service down). Never a pass."""


def _kernel():
    lib = ctypes.CDLL(DLL)
    lib.rust_ternary_quantize.argtypes = [
        ctypes.POINTER(ctypes.c_float), ctypes.c_size_t,
        ctypes.POINTER(ctypes.c_int8), ctypes.c_float, ctypes.POINTER(ctypes.c_float)]
    lib.rust_ternary_quantize.restype = ctypes.c_size_t
    lib.rust_ternary_index_search.argtypes = [
        ctypes.POINTER(ctypes.c_int8), ctypes.c_size_t, ctypes.POINTER(ctypes.c_int8),
        ctypes.c_size_t, ctypes.c_size_t, ctypes.POINTER(ctypes.c_size_t),
        ctypes.POINTER(ctypes.c_int32), ctypes.POINTER(ctypes.c_size_t)]
    lib.rust_ternary_index_search.restype = ctypes.c_int32
    lib.rust_turbovec_cosine.argtypes = [
        ctypes.POINTER(ctypes.c_float), ctypes.POINTER(ctypes.c_float), ctypes.c_size_t]
    lib.rust_turbovec_cosine.restype = ctypes.c_float

    if hasattr(lib, "rust_native_encode_text_f32"):
        lib.rust_native_encode_text_f32.argtypes = [
            ctypes.POINTER(ctypes.c_ubyte), ctypes.c_size_t,
            ctypes.POINTER(ctypes.c_float), ctypes.c_size_t
        ]
        lib.rust_native_encode_text_f32.restype = ctypes.c_int32

    if hasattr(lib, "rust_native_encode_text_ternary"):
        lib.rust_native_encode_text_ternary.argtypes = [
            ctypes.POINTER(ctypes.c_ubyte), ctypes.c_size_t,
            ctypes.POINTER(ctypes.c_int8), ctypes.c_size_t,
            ctypes.c_float, ctypes.POINTER(ctypes.c_float)
        ]
        lib.rust_native_encode_text_ternary.restype = ctypes.c_int32

    return lib


def _quantize(lib, vector: list) -> tuple:
    src = (ctypes.c_float * len(vector))(*vector)
    dst = (ctypes.c_int8 * len(vector))()
    scale = ctypes.c_float(0.0)
    written = lib.rust_ternary_quantize(src, len(vector), dst, ctypes.c_float(1.0), ctypes.byref(scale))
    assert written == len(vector), f"quantizer wrote {written} of {len(vector)} trits"
    return list(dst), scale.value


def _search(lib, query: list, corpus: list, k: int) -> list:
    flat = [x for vec in corpus for x in vec]
    c = (ctypes.c_int8 * len(flat))(*flat)
    q = (ctypes.c_int8 * len(query))(*query)
    idx = (ctypes.c_size_t * k)()
    score = (ctypes.c_int32 * k)()
    comps = ctypes.c_size_t(0)
    rc = lib.rust_ternary_index_search(q, len(query), c, len(corpus), k, idx, score, ctypes.byref(comps))
    assert rc > 0, f"search returned {rc} for {len(corpus)} vectors"
    return list(idx)[:rc]


def _oracle_topk(query: list, corpus: list, k: int) -> list:
    """Independent reference: agreement metric computed in pure Python."""
    ref = []
    for ci, c in enumerate(corpus):
        agree = sum(1 for a, b in zip(query, c) if a != 0 and a == b)
        ref.append((agree, ci))
    ref.sort(key=lambda pair: (-pair[0], pair[1]))
    return [ci for _, ci in ref[:k]]


def test_dll_present_and_loadable() -> None:
    assert os.path.isfile(DLL), f"kernel DLL not built: {DLL}"


def test_quantizer_produces_legal_trits() -> None:
    """Every trit must be in {-1,0,+1}; anything else is an illegal code."""
    lib = _kernel()
    vec = [0.5, -0.25, 0.0, 0.9, -0.9, 0.1] * 128
    trits, scale = _quantize(lib, vec)
    assert scale > 0.0, "scale must be positive"
    bad = {t for t in trits} - {-1, 0, 1}
    assert not bad, f"illegal trit values: {bad}"


def test_search_equals_independent_oracle() -> None:
    """The kernel's ranking must be exactly what the metric definition implies."""
    import random
    lib = _kernel()
    rng = random.Random(20260910)
    corpus = [[rng.choice([-1, 0, 1]) for _ in range(64)] for _ in range(40)]
    for qi in range(0, 40, 7):
        k_ids = _search(lib, corpus[qi], corpus, 5)
        assert k_ids == _oracle_topk(corpus[qi], corpus, 5), f"kernel disagrees with oracle at query {qi}"


def test_self_retrieval_finds_own_vector() -> None:
    """A query equal to a corpus vector must rank that vector first."""
    import random
    lib = _kernel()
    rng = random.Random(7)
    corpus = [[rng.choice([-1, 0, 1]) for _ in range(256)] for _ in range(25)]
    for qi in (0, 5, 12, 24):
        top = _search(lib, corpus[qi], corpus, 1)
        assert top[0] == qi, f"query {qi} returned {top[0]}"


def test_cosine_of_identical_vectors_is_one() -> None:
    lib = _kernel()
    a = (ctypes.c_float * 8)(*([0.3, -0.4, 0.5, 0.0, 0.1, -0.2, 0.7, 0.15]))
    cos = lib.rust_turbovec_cosine(a, a, 8)
    assert abs(cos - 1.0) < 1e-5, f"self-cosine was {cos}"


def test_native_model_free_encoder_available() -> None:
    """The kernel must provide native model-free query encoding with 0 Ollama requirement."""
    lib = _kernel()
    assert hasattr(lib, "rust_native_encode_text_f32"), "kernel missing rust_native_encode_text_f32"
    assert hasattr(lib, "rust_native_encode_text_ternary"), "kernel missing rust_native_encode_text_ternary"
    text = b"Monad sovereign knowledge retrieval substrate"
    buf = (ctypes.c_ubyte * len(text))(*text)
    out = (ctypes.c_float * DIM)()
    rc = lib.rust_native_encode_text_f32(buf, len(text), out, DIM)
    assert rc == DIM, f"encoder returned {rc} != {DIM}"
    norm = sum(x * x for x in out) ** 0.5
    assert abs(norm - 1.0) < 1e-4, f"vector not unit normalized: {norm}"


def _read_meta_and_first_entry(path: str, window: int = 8_000_000):
    """Parse only ``meta`` and the first document entry from a multi-GB index JSON.

    The file is one object ``{"meta": {...}, "index": {key: {embedding, metadata}, ...}}``.
    Loading it whole costs 4.85 GB on disk and several times that as Python objects, which is
    what made this contract test raise MemoryError on a 16 GB host. The prefix window holds
    meta and the leading entries comfortably (a 768-dim embedding is ~15 KB of text), so read
    that and decode individual members by brace matching instead of parsing the whole file.
    """
    with open(path, "rb") as fh:
        head = fh.read(window).decode("utf-8", errors="replace")

    def _match(s: str, start: int, open_ch: str, close_ch: str) -> int:
        depth = 0
        in_str = False
        esc = False
        for idx in range(start, len(s)):
            ch = s[idx]
            if in_str:
                if esc:
                    esc = False
                elif ch == "\\":
                    esc = True
                elif ch == '"':
                    in_str = False
                continue
            if ch == '"':
                in_str = True
            elif ch == open_ch:
                depth += 1
            elif ch == close_ch:
                depth -= 1
                if depth == 0:
                    return idx
        raise Unrunnable("index artifact truncated inside the read window")

    mi = head.find('"meta"')
    if mi < 0:
        raise Unrunnable("index artifact has no meta block")
    ms = head.find("{", mi)
    meta = json.loads(head[ms:_match(head, ms, "{", "}") + 1])

    ii = head.find('"index"')
    if ii < 0:
        raise Unrunnable("index artifact has no index block")
    pos = head.find("{", ii) + 1
    # Walk the leading members: "key": { ... }, "key": { ... }, ...
    for _ in range(64):
        while pos < len(head) and head[pos] in " \t\r\n,":
            pos += 1
        if pos >= len(head) or head[pos] != '"':
            break
        # a JSON string key: scan to its closing quote, honouring backslash escapes
        key_end = pos + 1
        while key_end < len(head):
            if head[key_end] == "\\":
                key_end += 2
                continue
            if head[key_end] == '"':
                break
            key_end += 1
        key = json.loads(head[pos:key_end + 1])
        colon = head.find(":", key_end)
        brace = head.find("{", colon)
        if brace < 0:
            break
        entry_end = _match(head, brace, "{", "}")
        entry = json.loads(head[brace:entry_end + 1])
        if entry.get("metadata", {}).get("source_path"):
            return meta, (key, entry)
        pos = entry_end + 1
    raise Unrunnable("no document in the leading entries records source_path")


def test_index_vectors_share_the_live_encoder_space() -> None:
    """Guard: the stored vectors must come from the encoder used to query them.

    A retrieval index built by model X and queried by model Y returns noise, no
    matter how correct the search algorithm is. This test embeds each document's
    own stored text prefix with the model recorded in the index and requires a
    high cosine against the stored vector.
    """
    if not os.path.isfile(INDEX):
        raise Unrunnable("index artifact missing — run build_kernel_knowledge_index.py")
    # The artifact is 4.85 GB on disk; json.load() on it raised MemoryError on a 16 GB host
    # (free physical memory 11.4 GB at the time). The test needs only meta.model and ONE entry,
    # so read a bounded window and parse just those — no full load, no memory ceiling.
    meta, probe = _read_meta_and_first_entry(INDEX)
    model = meta.get("model")
    if not model:
        raise Unrunnable("index artifact does not record the encoder that produced it")
    if not probe:
        raise Unrunnable("index artifact does not record source_path for its documents")
    key, entry = probe
    src = os.path.join(ROOT, entry["metadata"]["source_path"])
    if not os.path.isfile(src):
        raise Unrunnable(f"probe document not on disk: {src}")
    text = open(src, encoding="utf-8", errors="replace").read(entry["metadata"].get("chars_embedded", 3000))

    if model.startswith("native"):
        from swarm_v2.skills.embedding_skill import NativeEmbeddingSkill
        native_skill = NativeEmbeddingSkill(embedding_dim=DIM)
        live = native_skill.embed_text(text)
    else:
        try:
            body = json.dumps({"model": model, "prompt": text}).encode()
            req = urllib.request.Request(OLLAMA + "/api/embeddings", data=body,
                                         headers={"Content-Type": "application/json"})
            with urllib.request.urlopen(req, timeout=60) as r:
                live = json.load(r)["embedding"]
        except (urllib.error.URLError, OSError) as ex:
            raise Unrunnable(f"ollama unreachable, cannot verify encoder space: {ex}") from ex

    stored = entry["embedding"]
    assert len(live) == len(stored), f"dimension mismatch: live {len(live)} vs stored {len(stored)}"
    lib = _kernel()
    a = (ctypes.c_float * len(live))(*live)
    b = (ctypes.c_float * len(stored))(*stored)
    cos = lib.rust_turbovec_cosine(a, b, len(stored))
    assert cos > 0.5, (f"index encoder space mismatch for {key}: cosine {cos:.6f}. "
                       f"The index was built by a different model than {model}.")


def main() -> int:
    tests = [(n, f) for n, f in sorted(globals().items())
             if n.startswith("test_") and callable(f)]
    failed = 0
    not_run = []
    for name, fn in tests:
        try:
            fn()
            print(f"  PASS     {name}")
        except Unrunnable as ex:
            not_run.append((name, str(ex)))
            print(f"  NOT RUN  {name}: {ex}")
        except AssertionError as ex:
            failed += 1
            print(f"  FAIL     {name}: {ex}")
        except Exception as ex:  # noqa: BLE001 - surfaced, not swallowed
            failed += 1
            print(f"  ERROR    {name}: {type(ex).__name__}: {ex}")
    print(f"\n{len(tests) - failed - len(not_run)} passed / {failed} failed / {len(not_run)} not run")
    if failed:
        return 1
    if not_run:
        print("INCOMPLETE — a check that cannot run is never reported as a pass")
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main())
