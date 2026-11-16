# CHECKPOINT — system reboot, 2026-09-27

Written to be read COLD: a session with no conversation history must be able to resume the platform from
this file alone. **This file states the state as of writing (ledger tip 179); block 180 is the seal on it.**

## State at write time

| what | value |
|---|---|
| ledger tip | **block 179 `63e6ea2f39dce93c82796cc4…`** — 179 blocks, canonical 169, `prev_hash links broken: n=0`; UNVERIFIED = the pre-existing **97–101** only |
| kernel | `host_council_server.exe` **19,031,552 B sha256 `c20b21e31608c0e7…`** (the Step-2 image), `:8022`, `/health` → `HEALTHY`, `python_active: false`, anchor `0x4D4F4E4144000001` |
| rollbacks | `…host_council_server.exe.prev-k3fabric` (19,025,920 B = the block-178 image), `…exe.prev-20260927` (3,319,296 B = the pre-(a)(b) original) |
| dashboard | vite `:5183` → 200 |
| tier-2 | llama-server `:8587` → 404 on `/` (normal); model `D:\trm-models\ternary-bonsai\Ternary-Bonsai-8B-Q2_0_g64.gguf` |
| Ollama | **stopped, intentionally** — not part of the runtime |
| Python | offline tooling ONLY (`SOVEREIGNTY.md`); do **not** start the `app_v2` stack. Canonical interpreter `C:/Users/shawn/AppData/Local/Programs/Python/Python313/python.exe` (`py -3` works) |
| repository | `C:/Users/shawn/Downloads/TRM-Swarm-OS-v19-Sovereign-Private-main/TRM-Swarm-OS-v19-Sovereign-Private-main` |

## What this session changed (blocks 175–179)

- **175** — the TRM 7M decision filed on both council instruments, with the measured finding that the live
  kernel's council returned an *identical* tally for three different topics. **176** — my own quorum
  arithmetic corrected (`f=(n-1)/3=4 → quorum 9`; block 170 used 11).
- **177** — options (a)+(b): `/council/status` now carries `assigned_model`/`domain`/`confidence` for **15/15**
  agents from a **generated** table (`scratch/gen_agent_models_rust.py` → `agent_models_generated.rs`, ast-parsed
  from `agent_intelligence_bridge.py::AGENT_MODELS`, the canonical 15 — *not* the 14-domain table), and the
  TRM 7M is now **runnable** at `POST /trm7m` (1:1 port in `host_council_server/src/trm7m.rs`, same seed-42
  blob `39dd18faae3ddc38…`, 15,637,788 B). Stated hobble: the docs say "7M"; the artifact holds **3,909,447**
  f32 parameters — the blob is the measurement.
- **178** — the model's trit **decides** the verdict: `phase == CONSENSUS ⇔ k3_trit > 0 ∧ approve ≥ 10`.
  Immediately measurable, and the hobble was recorded: the trit is a deterministic function of the topic's
  FNV-1a hash through untrained weights, so the council is topic-**sensitive**, not topic-**intelligent**
  (a nonsense topic scored PASS in the same run).
- **179** — Step 2: the K3 domain routed through the kernel's **own quantized memory**
  (`k3_retrieval::fabric_lookup`), with the TRM 7M as the module's own **named** fall-through
  (`k3_trit_source`). Measured: the fabric PLACED both real topics on the correct sealed document
  (`docs/K3_VERDICT_DERIVATION_STANDARD.md` score 116; `papers/paper_157_…` score 119) but returned
  `packed_trit 0`, so real topics are withheld while an unmatched one passes on the model hash — the flat
  space made visible.

## Bring-up after reboot (in this order)

1. `cd <repo> && py -3 swarm_supervisor.py` (background). **The supervisor is the healer**: start it first,
   stop it FIRST. It restores `:8022`/`:5183`/`:8587` within ~5 s and adopts an already-bound tier-2 instead
   of double-starting it. Nothing else needs starting — Ollama stays stopped, the Python stack stays down.
2. Verify with a checklist, not a port scan: `/health` (`HEALTHY`, `python_active:false`, anchor);
   `/council/status` (15 members, `pbft_quorum`); `/memory/stats`; tier-2 `GET /props` **plus one real
   completion**; dashboard `:5183` → 200; the deployed exe's sha against this file.
3. **Housekeeping trap carried in from this session:** supervisor instances ACCUMULATE (9 were found; 4 were
   still up at last check). Each one is a healer, so killing one process just gets undone. Stop them all, then
   start exactly one. **Compose the process-match needle from parts** (`"swarm" + "_super" + "visor"`) — a
   literal needle matches the shell running the command and kills your own session mid-run (it happened).

## Deploying a new server binary (the order that works)

Build with `CARGO_TARGET_DIR` pinned to a **staging** dir in the SAME command as the build; content-guard the
candidate (`MZ` magic, size actually moved, a string unique to the new build); stop every supervisor; kill the
listeners **by PORT, never a remembered pid**; wait for the file lock to clear (`cp` returns
`Device or resource busy` until the killed process finishes dying — retry in a loop, and check the port is
free in the same breath); `cp` staged → target keeping `…exe.prev-<label>`; restart ONE supervisor; then prove
the swap with **a behaviour the old image cannot produce** (a new field or route), never a timestamp or size.

## Open decisions — the operator's, not mine

1. **Acquire a 768-dim CPU embedder** (which model / which dim). Then re-encode the corpus and point
   `k3_retrieval.rs::ENCODER_NAME` at it: `build_kernel_knowledge_index.py --model <enc>`, with the
   verification pattern in `docs/FABRIC_REEMBED_20260923.md` (deterministic encoder proven by 178/180
   bit-identical vectors; round-trip cosine gate). Scale is trivial — 2,180 docs ≈ 13 s at the measured
   168.9 docs/s. **This is what converts the flat space into a semantic one, and it gates ladder step 2.**
2. **Tighten the fall-through?** Today an unmatched topic can reach CONSENSUS on the model's hash. Requiring
   fabric confirmation for consensus would withhold it from unmatched topics — and also from real topics
   until the embedder lands. One line; deliberately not done.
3. **Aura** — `aura_handlers.rs` refuses every `/aura/*` by design ("Rollout 12: /aura/* is a DIFFERENT
   product"). Wiring the language into the runtime means revoking that decision.
4. **MOTION-013 retirement gate** — CLOSED on condition 3: the archive passes (446,867,446 B) and all 36 crate
   fates are recorded, but **32 live files still name `/home/shawn/trm-rust`**.
5. **WSL superseded crates** with no live artifact and no confirmed public source — recorded, not deleted.

## Deliberately NOT done

Option **(c)** (the 14 seeded per-agent heads — its own rejection clause holds: nothing measurable
distinguishes them from a generic head). Ladder steps **3 and 4** (SRAM Hopfield attractors behind the
retrieval; fractal telemetry as a regime gate) — that is where the flat finding points. **Aura** wiring. No
embedder acquired. Blocks **97–101** remain UNVERIFIED and are not mine.

## Read next

- `docs/INTELLIGENCE_INGEST_INVESTIGATION_20260927.md` (sha256 `f4fcacc386732866f411…`) — the four-module
  investigation and the five-rung ladder.
- `swarm_v2_artifacts/monad_ledger.jsonl` blocks **175–180**; verify with
  `py -3 swarm_v2/core/verify_ledger_chain.py`.
- `README.md` + `swarm_v2_artifacts/README.md` — the live figures (ledger count, exe hashes, open hobbles).

