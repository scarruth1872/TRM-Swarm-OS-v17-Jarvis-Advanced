# CHECKPOINT R2 — cold-start state (2026-09-27, after blocks 181-184)

Supersedes `docs/CHECKPOINT_20260927_REBOOT.md` (block 180). Everything below was MEASURED at write time.

## Live state

| surface | measured |
|---|---|
| kernel :8022 | `{"kernel":"Native Rust Sovereign Kernel v19.001","memory_kb":22100,"monad_anchor":"0x4D4F4E4144000001","python_active":false,"status":"HEALTHY","uptim` |
| dashboard :5183 | `200` |
| tier-2 :8587 | `200` |
| embedder :8588 | `{"status":"ok"} | pid=18816` |
| kernel exe | `19031552 B  sha256 c20b21e31608c0e7` |
| ledger | `blocks=184  canonical=174  alt_separators=5  UNVERIFIED=5 || prev_hash links broken: n=0 []` |
| procs | `3 python procs | port-8022 listeners: 1` |

## What changed since block 180

- **181** — the 10,000-dim space was tested, not migrated. The docs' own HDC dimension is 10,000 (`pattern_hyperdimensional_computing(dim=10000)`, paper_124), but re-encoding showed the width is NOT the constraint: the 768 and 10,000 encodings are collinear (cosine 1.000000) and the nonsense/on-topic ratio is 0.263 at both. Kernel untouched.
- **182** — a **semantic encoder is live**: `all-minilm` (all-MiniLM-L6-v2, 45,949,216 B) served by the host's own `llama-server.exe` on **:8588**, CPU-only (`-ngl 0`), weights from the local Ollama blob store. Dimension MEASURED 384. `embeddinggemma` (621,867,104 B) was REJECTED by mainline b10964 (`expected 316 tensors, got 314` — a fork-only arch). The live route is still the hash encoder.
- **183/184** — benchmarks. The semantic gate **kills the block-179 inversion** (nonsense cosines max 0.1103 -> zero false-confirms at any threshold >= 0.20) but **cannot certify document identity** (correct-doc cosines -0.0037..0.6748 vs wrong-doc up to 0.4286 — overlapping). And the real defect is upstream: the fabric places the known answer first in only **7/19 (37%)** of real topics, and **31% of topics get only text-less candidates**.
- **UNSEALED, this session's last work** — the corpus: recovery is exact in principle (`records.jsonl`, 1,994 records, source+index+full sha256+text, plus a byte-identical backup) but the builder has no relational branch, and my extractor only recovered **1,164/1,994 (58.4%)** before I ran out of context. **No block was sealed for it** — do not treat it as settled.

## Bring-up order after reboot

```bash
# 1) the platform
cd "<repo>" && py -3 swarm_supervisor.py          # starts kernel :8022, dashboard :5183, tier-2 :8587

# 2) the semantic encoder (:8588) -- SUPERVISED as of block 186, the supervisor launches it.
#    The command below is a FALLBACK only, for when the supervisor itself is not running:
C:/Users/shawn/AppData/Local/hermes/runtimes/llamacpp/b10964/cuda/llama-server.exe \
  -m C:/Users/shawn/.ollama/models/blobs/sha256-797b70c4edf85907fe0a49eb85811256f65fa0f7bf52166b147fd16be2be4662 \
  --embedding --pooling mean --host 127.0.0.1 --port 8588 -ngl 0 -c 1024 --ubatch-size 1024 -t 6
```

**Ollama stays stopped** — the weights are read as a file, no daemon.

## Verify after bring-up

1. `curl :8022/health` -> HEALTHY with `python_active:false` and anchor `0x4D4F4E4144000001`.
2. `curl :8588/v1/embeddings` with any string -> dim **384**, norm 1.0.
3. `py -3 swarm_v2/core/verify_ledger_chain.py` -> `prev_hash links broken: n=0`.
4. Kernel exe sha must equal the value in the live-state table above.

## Traps that have already bitten

- **Supervisors accumulate** (9 were found once; each instance heals). Stop them all, start exactly ONE. Never filter processes on a literal `swarm_supervisor` string — it matches the shell running your own command and kills the session; compose the needle from parts.
- `search_files` defaults to the session cwd (the Hermes repo), **not** this checkout — always pass absolute TRM paths.
- The embedder on :8588 WAS an untracked stray; as of block 186 the supervisor owns it. Its
  launch is proven by `logs/semantic_embedder.log`. (Historic note: it was an **untracked stray** (Hermes's tracked pid died, another pid serves the port). Restart it deliberately; do not assume anything owns it.
- Sealers re-derive and REFUSE on disagreement. Four refusals this session each caught a real overclaim — an expectation is not a measurement.

## Open items (highest value first)

1. **Corpus (3 steps, all local)**: enumerate the distinct `fields` keys per `kind` in `records.jsonl` -> an extractor covering all 1,994 -> add the relational branch to `build_kernel_knowledge_index.py` -> re-encode all 2,180 docs. Then re-run the block-184 sweep without the 31% unreadable topics.
2. **Proposal quality** — 37% first-place placement is the ceiling everything downstream inherits.
3. **Wire the semantic gate as a RELEVANCE filter at 0.45** (matches the incumbent, zero false-confirms) — NOT as an identity certifier. It also needs an owner in the supervisor.
4. Six `.md` documents referenced by the index are absent; their texts may exist only in the checkpoint backup zip.
5. Two holographic figures remain DECLARED, not measured: `global_knowledge_fabric.py` `'< 10ns'` recall, `aura_cognition.py` dispersion 0.94 / recovery 0.97.
6. Ledger blocks 97-101 UNVERIFIED (pre-existing). MOTION-013 retirement gate still closed on condition 3: 32 live files name `/home/shawn/trm-rust`.

## Ledger

184 blocks, canonical 174, alt_separators 5, UNVERIFIED 5, `prev_hash links broken: n=0`. Latest sealed: 181 `f29753df12198fc6`, 182 `e8265095953d099e`, 183 `627b993348f2c7bc`, 184 `c626959460b2cf0d`.

## Version and supervision (block 186)

- Kernel source declares **v19.002**; the RUNNING binary still reports **v19.001** because no Rust was
  rebuilt. This skew is deliberate and must not be mistaken for a failure: rebuild and swap the kernel
  (staging build -> content guard -> stop every supervisor -> kill by PORT -> wait out the file lock ->
  copy -> restart ONE) to make the live version match the source.
- `swarm_supervisor.py` now registers a `semantic_embedder` service on :8588 and, as of block 186, can
  heal a service it ADOPTED from an already-active port (previously such a service could be monitored,
  warn once, and stay dead forever). Services declared with `cmd=[]` (the Aura project) are still never
  resurrected by design.
- Kills on this host: `taskkill //F` is an INVALID OPTION on this MSYS shell and fails quietly enough to
  look like success. Use PowerShell `Stop-Process -Force`, and do process matching INSIDE PowerShell.
