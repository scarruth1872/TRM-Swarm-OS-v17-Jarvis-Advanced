# Model inventory, measured (2026-09-26) — step 5's input set

Step 5 of the WSL reconciliation is "models: dedupe opportunistically, watch C:". This is the measurement it
needs, taken before any decision. Names are not evidence: identity is size + inode now, and a content hash
before anything moves.

## The Windows side

`D:\trm-models` — 39 GB, 12 weights (this is the `_g64` set the tier-2 discovery scans):

| bytes | file |
|---|---|
| 10,470,249,600 | `instella/amd.Instella-MoE-16B-A3B-Think.Q4_K_M.gguf` |
| 7,585,330,240 | `ternary-bonsai/Ternary-Bonsai-27B-Q2_g64.gguf` |
| 7,206,168,928 | `ternary-bonsai2/Ternary-Bonsai-2-27B-PQ2_0.gguf` |
| 7,165,121,600 | `ternary-bonsai/Ternary-Bonsai-27B-Q2_0.gguf` |
| 2,310,125,920 | `ternary-bonsai/Ternary-Bonsai-8B-Q2_0_g64.gguf` |
| 2,182,184,672 | `ternary-bonsai/Ternary-Bonsai-8B-Q2_0.gguf` |
| 1,137,806,656 | `ternary-bonsai/Ternary-Bonsai-4B-Q2_0_g64.gguf` |
| 1,074,969,344 | `ternary-bonsai/Ternary-Bonsai-4B-Q2_0.gguf` |
| 629,246,976 | `ternary-bonsai2/Ternary-Bonsai-2-27B-mmproj-Q8_0.gguf` |
| 629,246,880 | `ternary-bonsai/Ternary-Bonsai-27B-mmproj-Q8_0.gguf` |
| 490,163,968 | `ternary-bonsai/Ternary-Bonsai-1.7B-Q2_0_g64.gguf` |
| 463,290,464 | `ternary-bonsai/Ternary-Bonsai-1.7B-Q2_0.gguf` |

`C:` (bounded to depth 3 under the user home) — 6 weights plus two shared paths:

| bytes | inode | file |
|---|---|---|
| 2,489,757,856 | 103019841476395896 | `models/gemma-3-4b-it/gemma-3-4b-it-Q4_K_M.gguf` |
| 2,019,373,888 | 11258999068504962 | `mamba-server/Hermes-3-Llama-3.2-3B.Q4_K_M.gguf` |
| 1,625,216,768 | 13510798882265793 | `mamba-server/Mamba-3B.Q4_K_M.gguf` |
| 1,625,216,736 | 167759086119567834 | `Downloads/mamba-2.8b.Q4_K_M.gguf` |
| 1,252,357,538 | 10696049115587468 | `CascadeProjects/inspired-fitness/mistral-7b-instruct-v0.1.Q4_K_M.gguf` |
| 851,251,104 | 192810359048147019 | `models/gemma-3-4b-it/mmproj-model-f16.gguf` |

## What is and is not a duplicate

- **Same file, two paths — nothing to dedupe.** `Hermes-3-Llama-3.2-3B` (2,019,373,888 B) and `Mamba-3B`
  (1,625,216,768 B) each carry link count **2** and one shared inode
  (`11258999068504962` / `13510798882265793`) across `AppData/Local/hermes/models/` and `mamba-server/`.
  Measured, so the next reader does not rediscover it as two 2 GB files.
- **A real cross-disk duplicate, 1,137,806,656 B.** `Ternary-Bonsai-4B-Q2_0_g64.gguf` exists on `D:`
  (inode `5894666641956750343`) **and** on `C:` at `AppData/Local/hermes/models/` (inode
  `4222124657013448`) — distinct files, same size. `C:` is the constrained disk (below), so this is the
  candidate to resolve, **after** a content hash proves the equality that size only suggests.
- **A near-twin that is NOT a twin.** `Downloads/mamba-2.8b.Q4_K_M.gguf` (1,625,216,736 B) against
  `mamba-server/Mamba-3B.Q4_K_M.gguf` (1,625,216,768 B): 32 bytes apart, different inodes, different names.
  Size says "same", the byte count says otherwise — a hash decides, not the name.
- **Two mmproj files 96 bytes apart** (629,246,976 vs 629,246,880) in `ternary-bonsai2/` and
  `ternary-bonsai/`: distinct files, and neither is a duplicate of the other by size.
- **One weight cannot load here on size alone.** `instella/…Q4_K_M.gguf` is 10,470,249,600 B against an
  8.41 GB device-local budget on the RTX 3070 Laptop; the WSL `instella-gguf/…Q2_K.gguf` (6,514,704,512 B) is
  the copy inside the ceiling.

## Disk state

`C:` 935 GB, 843 GB used, **92 GB free (91%)** — the constraint. `D:` 1.9 TB, 727 GB used, **1.2 TB free
(39%)**. WSL `/`: **808 GB free**. The archive and any large ingestion belong on D:/WSL, never on C:.

## The decision (2026-09-26): no dedupe action, and why

The measurement removed the case for acting:

| candidate | measured identity | decision |
|---|---|---|
| `Hermes-3-Llama-3.2-3B`, `Mamba-3B` | one inode each, link count 2 (`AppData` + `mamba-server`) | nothing to dedupe — already one file at two paths |
| `Ternary-Bonsai-4B-Q2_0_g64.gguf` on C: and D: | same size, **different inodes**, 1,137,806,656 B each | **leave both** — different OWNERS, below |
| `Downloads/mamba-2.8b` vs `mamba-server/Mamba-3B` | 32 bytes apart, different inodes | not a duplicate; a hash would only confirm what the sizes show |
| the two mmproj files | 96 bytes apart, distinct files | not duplicates |

The C: copy of the 4B g64 sits in **Hermes's own model cache** (`AppData/Local/hermes/models/`, beside
`Hermes-3-Llama-3.2-3B` and `Mamba-3B`), and **the platform never reads that path**: tier-2 discovery is
`swarm_supervisor.py::discover_tier2_generator()`, whose model globs are `D:/trm-models/**/*.gguf` and
`D:/trm-models/*.gguf` **only** — the C: glob in that same function is for the `llama-server.exe` *binary*
(`~/AppData/Local/hermes/runtimes/llamacpp/...`), not for weights. So the two files are not a redundant pair of
one store: one belongs to Hermes, one to the platform. Moving another application's cache to reclaim 1.1 GB of
92 GB free (1.2%) is exactly the change the reachability-and-ownership rule forbids; the record says why
instead of doing it.

## What the measurement does reveal: tier 2 depends on external media

`curl 127.0.0.1:8587/props` reports the loaded model as
`D:\trm-models\ternary-bonsai\Ternary-Bonsai-8B-Q2_0_g64.gguf` (`n_ctx` 4096). `D:` is **external storage**, and
the discovery has no second model root, so with `D:` unplugged `best` is empty and tier 2 declines to start —
which `swarm_supervisor.py` documents as "defined but not started" rather than OOM-ing per request. That is an
honest degradation and a real single point of failure. The fix is a second model root or a local copy, and that
is a decision to take deliberately — not a dedupe by-product.

## The fix: a second model root (2026-09-26)

Taken deliberately, and it is a **wiring change plus a real copy**, not a note:

- `swarm_supervisor.py::discover_tier2_generator()` now globs **two** roots, primary first —
  `D:/trm-models` and the tree's own `swarm_v2_artifacts/tier2_models/`, the latter derived from the file's
  own location rather than a hardcoded host path. Ties resolve in root order, so when the copies are identical
  the **external one stays the model of record**.
- The fallback is seeded with a byte-identical copy of the **verified 8B**
  (`Ternary-Bonsai-8B-Q2_0_g64.gguf`, 2,310,125,920 B, sha256 `e17b298d84ee7879…`) — copied here in 36.5 s.
  Deliberately the 8B and not the 4B: `tier2_verified_models.json` lists the 8B as proven to load and generate
  on this host and the 4B under `not_yet_verified`, and that manifest's own rule is that an unverified GGUF is a
  claim. A fallback that starts the tier on an unproven model would trade one silent failure for another.
- **Proven, not asserted** (`scratch/prove_tier2_roots.py`, in-process, nothing restarted):

  | case | result |
  |---|---|
  | the real filesystem | picks `D:\trm-models\ternary-bonsai\Ternary-Bonsai-8B-Q2_0_g64.gguf` |
  | the external disk hidden | falls back to `swarm_v2_artifacts\tier2_models\…8B-Q2_0_g64.gguf` (GGUF header checked) |
  | both roots hidden | returns `""` — the tier declines rather than picking something wrong |
  | the two copies | same size, same sha256 `e17b298d84ee7879…` |

  The docstring was re-read by the harness too, because the first draft of this edit closed it with a comma —
  which parses, and silently demotes a docstring to a discarded tuple.

Nothing was restarted: the running tier is healthy and already has the model of record loaded, and the change
takes effect at the supervisor's next start. The cost is 2.31 GB of C:'s 91 GB free, for a tier that no longer
drops when external media is unplugged.

## The desktop node (why these models matter beyond this machine)

The desktop node is not on this disk: it is a **Tailscale peer reached through the platform's federation**.
`swarm_v2_federation/registry.json` (v2, `updated_at 2026-09-24T11:02:21`) names it, and its own `basis` is the
rule — "measured by direct probe of each node's /health over the Tailscale mesh; no entry is asserted that was
not answered by the host named in it":

| node | tailnet | registry entry | probed fresh 2026-09-26 |
|---|---|---|---|
| `DESKTOP-MEP4VQN` | 100.93.191.43 | "TRM Desktop Sovereign (6700XT)", 102.99 ms | **HEALTHY** — cpu 4000 MHz, 32768 MB (67.41% used), `voltage_measured: true`, `voltage_phi_v 0.6629`, same monad anchor |
| `carruth-MacPro5-1` | 100.76.102.6 | "MacPro5-1 Sovereign (linux)", 22.6 ms | **HEALTHY** — 32768 MB, uptime 3600 |
| `DESKTOP-VB1T79V` | 100.82.16.43 | *(absent — it never answered)* | `:8022` answers nothing, so the registry is right to exclude it |

The 6700XT is exactly the GPU `GPU_SPECS` targets, which is why that node is the natural home for the larger
GGUF set. Two facts to carry forward. First, the peers report the **same backend string** (`Native Rust
Sovereign Kernel v19.001`) but **not the same route set**: this host answers `/models`, while `DESKTOP-MEP4VQN`
returns `{"error":"no such endpoint","hint":"the native council serves a fixed route set; unknown paths are not
silently accepted"}` — a version string does not imply a capability. Second, this host's own model surface
(`/models` → `swarm_v2_artifacts/ternary_models/model_manifest.json`, `trm7m_source.trm` = 985,147 B from a
15,656,632 B safetensors, compression 15.932, integrity intact) is a **different thing** from the GGUF set tier
2 loads; "the models" needs saying which ones.
