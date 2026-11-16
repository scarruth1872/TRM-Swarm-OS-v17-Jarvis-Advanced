# CHECKPOINT 2026-09-28 -- REBOOT R3 (supersedes R2)

Measured this session, not asserted.

## Version: source and runtime now AGREE
- Running kernel reports **v19.002**; `main.rs` declares **v19.002**. First agreement on this stack.
- Deployed: `host_council_server.exe`, 19,031,552 B, sha256 `ad174ae40bec9001...`
- Rollback: `host_council_server.exe.prev-v19001` -- the v19.001 image, one copy away.
- The bump was INCOMPLETE at first. `main.rs` held 4 sites; three more renderings lived in
  `k3_retrieval.rs:317`, `services_handlers.rs:55`, `state_handlers.rs:1342`. The CONTENT GUARD
  refused the swap and exposed them. **The guard, not the grep, is the reliable instrument.**

## The swap, repeatable
1 Build to a staging CARGO_TARGET_DIR exported in the same command. 2 Content-guard the candidate
(must contain the new version, must NOT contain the old); if it fails, STOP -- nothing touched yet.
3 Copy the deployed image aside as rollback. 4 Stop EVERY supervisor (PowerShell Stop-Process,
matching inside PowerShell). 5 Kill the pid LISTENING on :8022, re-resolved at that moment.
6 Wait for the port to go dark. 7 Copy, start exactly ONE supervisor, it spawns the kernel.
8 Verify version, exe hash, council count, all four surfaces.

## Traps that cost time
- `taskkill //F` is an INVALID OPTION on this MSYS shell and fails quietly enough to read as success.
  Use `Stop-Process -Force`; verify the port went dark before concluding anything.
- A capped listing hides later matches: `grep | head -20` never reached `trm_uefi_kernel/` because
  `swarm_v2_artifacts/` sorts first. The three missed version sites were invisible for that reason.
- An unbounded grep from the repo root TIMES OUT -- the repository root is the user's HOME directory.
  Bound every search (target swarm_v2*, trm_uefi_kernel*).
- A background wrapper's exit code says nothing about a service; wrappers exited 127 while their
  services were verified live (200).
- `git add -A` here would sweep unrelated trees and possibly credentials into a push.

## Benchmark: kernel intelligence -- MEASURED on v19.002
Run 16:22:05Z -> 16:26:06Z; artifact bench_kernel_intelligence.json (2,276 B).
- Kleene laws **21/21** exact; rank1 self-hits **2180/2180**, **P@1 = 1.0**, indistinguishable pairs **0**
- quantization: **1,674,240 trits**, fidelity_mean **0.889572** (min 0.837919, max 0.9063)
- council consensus, weighted score 0.840325, monad seal 0x4c5e5f50551141b7; anchor 0x4d4f4e4144000001
- latency n=1000: ternary p50 **1,380,700 ns** vs float cosine p50 **338,290.1 us** -- ~**245x**
- **live_path: attempted 0 / answered 0 -- THIS ARM MEASURED NOTHING.** Do not cite it as evidence
  about live behaviour.
- **No baseline exists**: the artifact path is not in git and this run overwrote its only copy.

## Benchmark: 100-parsers crucible -- NOT RUNNABLE
No script in the tree produces it; only artifacts/configs mention it. The artifact claims 100 tasks
across 20 parsers in `total_wall_time_ms: 1.44` (0.0144 ms/task) with a **14-agent** council -- a
historical claim, not a reproducible result. Reconstructible from `100_parsers_tasks_crucible_report.md`.

## State (measured)
:8022 v19.002 HEALTHY python_active false; :5183/:8587/:8588 all 200 (embedder supervised);
exactly one supervisor; Ollama stopped (0 processes). Ledger sealed through the block below.

## Git
Branch `Quantum-Nexus`; ~78 commits ahead of origin (fast-forward, no divergence).
**PUSH BLOCKED**: `git@github.com: Permission denied (publickey)`, `gh` not installed.
No git tag carries v19.x -- `git tag -a v19.002` is one command. Repo root is the HOME directory;
commit by explicit path only.

## Still open
The push (needs the key) and a v19.002 tag. The corpus recovery is 58.4% (1,164/1,994);
`relational_src/records.jsonl` is IRREPLACEABLE. The fabric proposes the wrong document ~63% of the
time; 31% of topics return text-less candidates. The semantic gate kills the nonsense inversion but
cannot certify document identity. Ledger blocks 97-101 remain UNVERIFIED (pre-existing).
