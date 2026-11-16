# Checkpoint — P2i (exercising the crates) — 2026-09-24

Resumed and finished after the shutdown note. The working tree is **untracked in git**, so
`backups/checkpoint_20260924_P2I_shutdown/` and `backups/checkpoint_20260924_P2I_closed/` are the
recovery sources for every file this phase touched.

## Result

**49 / 53 entries exit 0** — 19 binaries + 34 examples all exercised at least once. The phase started
at 27/53 (examples 15/34).

The four that are not exit 0, each named:

| entry | what it is |
|---|---|
| `bin:fit_server`, `bin:swarm-server` | SERVING — not run as one-shots; `:5281/health` → 200, and the QUIC server proven by the `bench`/`client` runs |
| `example:ingest_scale_native` | a **measured** timeout: ~1.2 KB/s (24,444 B in 20 s) writing its 256 MB shard arena through the `//wsl.localhost` 9p share → hours, not minutes. It opens and ingests; the store's location is the constraint |
| `example:verify_gpu_neural` | exit 2 **by design** — binds `c_k3_collapse_gpu`, a symbol defined nowhere in the tree; reports the absence instead of panicking, and its CPU AVX2 path is measured |

`fit_client` / `svc_client` print a usage line with no arguments (a correct refusal) and return **exit
0 with real verdicts** when given the arguments their usage documents:
`[fit] booking → BOOKED (persisted in the NAND fabric, byte-verified)`,
`[svc] jarvis → LIVE (persisted in the fabric, byte-verified)`.
`client`: `trm:// protocol works - K3 greeting answered with K3 TRUE`. `bench`: `1000 connections
served, zero errors`.

## Sealed

| block | hash | what |
|---|---|---|
| 129 | `992b06f433112d82` | examples + binaries exercised; the literal class closed |
| 130 | `275dae847b6d39d4` | the three fixes (cdylib relink, interpreter split, shard-open report) |
| 131 | `cfde051941023a8f` | client recheck with servers up — the corrected tally |

Receipt `swarm_v2_artifacts/wsl_crate_exercise.json` **28,465 B**; 131 blocks, tip reproduces, all
`prev_hash` links unbroken. Pre-existing UNVERIFIED blocks **97–101** (unchanged by this phase).

## What was wrong, and what fixed it

1. **One class, 21 sites:** recorded WSL literals in examples and in one library (`swarm-drug`). Every
   file behind them exists on the WSL side and reads through `//wsl.localhost/...`; the crates were
   written against a real host. Re-pointed through `swarm-paths`.
2. **Three real defects:** a **1 MiB array on the stack** in `verify_migrate` (0xC00000FD, exit
   3221225725); **`c_k3_collapse_gpu`**, a gate whose symbol exists nowhere; **`TRM_PYTHON="py -3"`**
   handed whole to `Command::new`, so the spawn failed and the crate read it as "python is broken".
3. **A store that persists a resolved path bakes in the resolver's state at write time** — three
   instances. `ShardedFabric::open` now re-resolves a recorded path through `swarm-paths` and
   **prints** the substitution; a shard that exists nowhere still fails (negative control in
   `scratch/verify_shard_reresolve.py`).
4. **A fix in a crate does not reach an example that loads the cdylib.** `swarm-shard` compiled and
   the example still failed, because it calls `c_shard_open` out of `swarm_kernel.dll` — relinked
   237,568 B → **253,440 B**, sha `c40abe63e137bd05` → **`d50e8816fe5beafc`**.
5. **Rulers fixed before the tree was measured again:** the sweep keeps the `panicked at` line and its
   reason (not just "run with RUST_BACKTRACE=1"), re-runs a non-zero once and reports pass-on-retry as
   FLAKY, gives long-runners their own budget, and classifies a usage line as a REFUSAL, not a defect.

## Open, recorded, not fixed

- **`verify_forward` exits 0 with `BUILD #1 VERIFY: FAILED — diagnose`** — it parses the real 964 MB
  Q4_K GGUF and runs the full 30-layer forward pass (~66 s); its own gate says the hidden state does
  not match the llama.cpp reference within 0.1. A research finding, not a harness defect.
- **The ingest store lives on the slow side** of the WSL/Windows boundary (~1.2 KB/s). Moving a
  full-scale store to the fast side is a design decision (one owner per store), not a silent edit.
- Earlier-phase items unchanged: 6 of 10 exercised crates still have no kernel route; the supervisor
  swarm heals nothing; executor-layer fabrication in `run_tasks_441_445_optimization.py`; 17 unported
  engines; federation needs a passwordless channel.
