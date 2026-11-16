# Port Boundary — what the "44 kernel-candidates" actually are

Measured 2026-11-03. Every number here came from a command run in this session;
the raw output is in `evidence/PORT_CLASSIFICATION_PHASE5_MEASURED.txt`.

---

## 1. The answer

**The strategic position is right in its conclusion and wrong in its arithmetic.**

Right: the port should not be chased to 100%. The decision and control path that
matters — council, K3 ternary algebra, Mamba-3 SSM, memory IPC, transistor fabric —
is native Rust and genuinely linked by the running council server on :8022.

Wrong: "44 modules, 60–92% ported, long tail" describes a backlog that does not
exist in that shape. Measured against the Rust files themselves:

| What the 44 really are | Count |
|---|---|
| Rust file is a **shell** — ≤2 public items, no logic | 17 |
| Rust file is **thin** — 3–9 public items | 24 |
| Rust file has a real surface — ≥10 public items | 3 (`pbft_consensus` 21, `resonance_engine` 11, `thz_wave_gateway` 10) |
| Rust file contains **nothing but a doc comment and imports** | 4 (`redis_mock`, `remediation_engine`, `monitor_daemon`, `background_tasks`) |

The shrink percentage measures how much *smaller* the Rust file is, not how much
of it is finished. `secrets_vault` at "92% shrink" is 357 Python lines reduced to
28 Rust lines holding 4 public items and one function body. That is not a port
that is 92% done; it is a port that has not started, wearing a header comment that
says `// MANUAL need human verification of math semantics`.

So the honest restatement is: **the 44 are not 44 partially-finished ports. They
are 3 real ports, 24 stubs, and 17 empty shells.** The reason that is *good* news
is section 4: most of them should never be ported at all.

---

## 2. Method

1. `python tools/surface_diff.py` — 157 modules compared, 44 KERNEL-CANDIDATE
   (17 SYNC-PORTABLE, 23 ASYNC-OTHER, 2 ASYNC-DAEMON, 2 NET), 26 HOST-ONLY, 5 NET-ONLY.
2. For each of the 44, resolve the **transitive import closure** of the live
   entrypoints, using an AST walk that resolves `from swarm_v2.core.X import Y`
   and relative imports to real files:
   - `swarm_v2/app_v2.py` — the FastAPI service on **:8099** (`python -m swarm_v2.app_v2`, PID 18816)
   - `tools/trm_server.py` — **:8588** (PID 29232)
   - `trm_uefi_kernel/trm_companion_proxy.py` — **:3001** (PID 37564)
   - `trm_uefi_kernel/host_council_server` — **:8022**, Rust, links the kernel crate
3. A module is **live** if it is in the closure of one of those. Closure sizes:
   app_v2 → 166 local modules; trm_server → 6; companion_proxy → 1.
   **34 of the 44 are in the app_v2 closure. 0 are in the other two.**
4. Bucket by *where the code has to run*, not by import count:
   - **ON-PATH** — participates in a decision, a control action, or state a running
     service depends on. Must be native if the kernel is to own that behaviour.
   - **HOST-TOOLING** — correctly Python: hardware probing, packaging, training,
     file management, or dead code with no caller.
   - **NET-BOUND** — terminates HTTP/sockets to an external system; the kernel has
     no network stack, so Python is correct.
5. Tier within ON-PATH: **CORE** (every request or mutated state), **SUPPORTING**
   (a feature endpoint), **TELEMETRY** (read-only status surface).

---

## 3. The classification

`rs` = public items in the Rust file (`pub fn/struct/enum/trait/const/static`).
`live` = first live importer found; `—` means no live process reaches it.

| module | py→rs | shrink | rs | live importer | bucket | tier |
|---|---|---|---|---|---|---|
| base_agent | 1427→63 | 96% | 4 | base_agent.py ← 6 live modules | ON-PATH | CORE |
| remediation_engine | 169→9 | 95% | 0 | app_v2.py | ON-PATH | CORE |
| monitor_daemon | 139→9 | 94% | 0 | app_v2.py | ON-PATH | CORE |
| background_tasks | 253→9 | 96% | 0 | app_v2.py | ON-PATH | CORE |
| neural_wall | 505→54 | 89% | 4 | app_v2.py (middleware) | ON-PATH | CORE |
| chain_of_verification | 450→98 | 78% | 8 | app_v2.py, base_agent.py | ON-PATH | CORE |
| cognitive_stack | 190→23 | 88% | 7 | base_agent.py | ON-PATH | CORE |
| llm_brain | 268→72 | 73% | 5 | base_agent.py | ON-PATH | CORE |
| llm_orchestrator | 60→23 | 62% | 1 | cognitive_stack.py | ON-PATH | CORE |
| semantic_cache | 364→99 | 73% | 9 | llm_brain.py | ON-PATH | CORE |
| intent_buffer | 336→110 | 67% | 5 | app_v2.py, routes/swarm.py | ON-PATH | CORE |
| redis_mock | 87→9 | 90% | 0 | app_v2.py, sentinel.py, self_healing.py | ON-PATH | CORE |
| secrets_vault | 357→28 | 92% | 4 | routes/kanban.py (5 endpoints) | ON-PATH | CORE |
| self_healing_orchestrator | 305→23 | 92% | 1 | app_v2.py, routes/learning.py | ON-PATH | CORE |
| swarm_orchestrator | 150→23 | 85% | 1 | app_v2.py, routes/orchestrator.py | ON-PATH | CORE |
| lora_response_modulator | 216→29 | 87% | 5 | base_agent.py:871 | ON-PATH | CORE |
| collaborative_reasoning | 230→23 | 90% | 1 | routes/swarm.py | ON-PATH | SUPPORTING |
| auto_changelog | 587→69 | 88% | 5 | app_v2.py, routes/changelog.py | ON-PATH | SUPPORTING |
| reconnaissance_daemon | 309→38 | 88% | 4 | app_v2.py, routes/research.py | ON-PATH | SUPPORTING |
| mcp_tool_evolver | 560→77 | 86% | 5 | app_v2.py, routes/untapped.py | ON-PATH | SUPPORTING |
| ultrawork_loop | 544→82 | 85% | 6 | routes/kanban.py | ON-PATH | SUPPORTING |
| swarm_performance_insights | 641→107 | 83% | 7 | app_v2.py, routes/insights.py | ON-PATH | SUPPORTING |
| deterministic_forge | 119→23 | 81% | 1 | app_v2.py, routes/synthesis.py | ON-PATH | SUPPORTING |
| coordination_responder | 109→23 | 79% | 1 | routes/coordination_respond.py | ON-PATH | SUPPORTING |
| mutation_engine | 167→53 | 68% | 3 | routes/evolution.py | ON-PATH | SUPPORTING |
| resonance_engine | 228→86 | 62% | 11 | routes/untapped.py | ON-PATH | SUPPORTING |
| pbft_consensus | 493→148 | 70% | 21 | routes/untapped.py | ON-PATH | SUPPORTING |
| moltbook | 258→51 | 80% | 3 | routes/untapped.py | ON-PATH | SUPPORTING |
| continuum_genomics | 119→23 | 81% | 1 | routes/swarm.py, microkernel_spawner.py | ON-PATH | SUPPORTING |
| port_manager | 144→27 | 81% | 3 | kanban_board.py ← background_tasks, base_agent | ON-PATH | SUPPORTING |
| prism_fqm | 136→23 | 83% | 1 | routes/swarm.py (status) | ON-PATH | TELEMETRY |
| meta_orchestrator | 87→15 | 83% | 1 | routes/swarm.py (status) | ON-PATH | TELEMETRY |
| cosmos3_edge_bridge | 235→49 | 79% | 3 | radix_trie_matcher.py (chat keyword) | ON-PATH | TELEMETRY |
| thz_wave_gateway | 220→71 | 68% | 10 | radix_trie_matcher.py (chat keyword) | ON-PATH | TELEMETRY |
| brain_ai_microkernel_bridge | 224→28 | 88% | 4 | — | HOST-TOOLING | dead |
| cosmos_sparse_moe_teams | 161→26 | 84% | 2 | — | HOST-TOOLING | dead |
| trm_7m_full | 218→45 | 79% | 8 | — | HOST-TOOLING | dead |
| cosmos_bridge | 109→23 | 79% | 1 | — | HOST-TOOLING | dead |
| daily_evolution_engine | 219→53 | 76% | 4 | — | HOST-TOOLING | dead |
| cosmos_moe_instella_shunter | 104→28 | 73% | 4 | — | HOST-TOOLING | dead |
| cosmos_microkernel_entanglement | 96→26 | 73% | 2 | — | HOST-TOOLING | dead |
| optimization_engine | 86→23 | 73% | 1 | — (only scripts/verify_phase_11.py) | HOST-TOOLING | dead |
| mcp_bus | 250→71 | 72% | 5 | — | HOST-TOOLING | dead |
| microkernel_chat_bridge | 168→52 | 69% | 5 | — | HOST-TOOLING | dead |

### Summary

| Bucket | Count |
|---|---|
| **ON-PATH** | **34** — 16 CORE, 14 SUPPORTING, 4 TELEMETRY |
| **HOST-TOOLING** | **10** |
| **NET-BOUND** | **0** |
| Total | 44 |

**NET-BOUND is zero, and that is a finding, not an omission.** The tool's own NET
bucket (2: `microkernel_chat_bridge`, `thz_wave_gateway`) is keyword-driven — it
fires on the string "http" appearing anywhere in the file, including docstrings.
Neither module imports `socket`, `requests`, `httpx`, `aiohttp`, `urllib` or
`websockets`. The genuinely network-bound modules in this codebase were already
caught by the HOST-ONLY pass (`task_arbiter`, `federation`, `openclaw_gateway`,
`mpu` — all `requests`; `external_socket_bridge` — `socket`). Nothing in the 44
needs a network stack.

---

## 4. Why 34 ON-PATH does not mean 34 ports to do

This is the part that decides the question, and it is a distinction the shrink
percentage cannot express.

The 34 live modules run **inside the FastAPI process on :8099**. That process is a
host process with an OS, a filesystem, an event loop and an HTTP stack. The kernel
is a `no_std` UEFI binary with none of those. So "porting" a module that exists to
serve an HTTP endpoint means porting the endpoint too — and the kernel has no
network stack to serve it from. The port would have no caller.

Concretely, of the 34:

- **16 CORE** are the agent runtime and its state: `base_agent`, `cognitive_stack`,
  `llm_brain`, `llm_orchestrator`, `semantic_cache`, `intent_buffer`, `redis_mock`,
  `secrets_vault`, `neural_wall`, `chain_of_verification`, `remediation_engine`,
  `self_healing_orchestrator`, `monitor_daemon`, `swarm_orchestrator`,
  `background_tasks`, `lora_response_modulator`. These are the *host service's*
  decision path. They are not the kernel's. The kernel's decision path is the four
  modules the council server actually links: `k3_logic`, `mamba3_turbovec_ssm`,
  `memory_mapped_ipc`, `unified_transistor_fabric`.
- **14 SUPPORTING** are feature endpoints (kanban, changelog, evolution, untapped,
  insights, synthesis, coordination). Pure host-service surface.
- **4 TELEMETRY** are read-only status strings behind the radix-trie chat dispatch
  in `routes/swarm.py`. Worth flagging separately: several handlers in that
  dispatch table return hardcoded literals (`"Bridge: HTTP/3 QUIC ready, OAuth2
  tokens cached, rate-limited at 100 req/s"`) rather than reading the object they
  claim to report on. That is a claim-honesty problem in the chat surface, not a
  porting problem, and it is out of scope here — but it should not be quoted as
  evidence that those subsystems are live.

**The one genuine kernel contract in the 44 is `intent_buffer`.** The architecture
is explicit that Cosmos-3 action chunks land in a shared-memory intent buffer that
the microkernel fast-path reads (`cosmos3_edge_bridge` docstring: "Shared Memory
Intent Buffer → Microkernel Fast-Path"). That is a contract the kernel must honour
in native code, and its Rust twin is 110 lines with 5 public items — the most
substantial of the SYNC-PORTABLE set after `pbft_consensus`. If exactly one module
from this list deserves finishing, it is that one.

`trm_7m_full` is the other candidate on merit — a 7M-parameter Mamba-3 forward pass
is kernel compute — but it is already superseded: `trm_7m_model` is what
`agent_intelligence_bridge` imports, and the kernel carries `trm7m.rs` and
`trm_7m_optimized.rs` alongside `trm_7m_full.rs`. Three copies, one of them dead.

### The 10 dead ones

None of these is imported by any of the 1,560 Python files scanned outside
backups, tests and vendored snapshots — except `optimization_engine`, which only
`swarm_v2/scripts/verify_phase_11.py` touches. No process is running them
(`daily_evolution_engine` claims to start at boot via `startup.bat`; no such
process exists). The Rust kernel already contains a file for each. They are dead
Python, not backlog. Deleting them would change nothing that runs.

---

## 5. Warning debt

`CARGO_TARGET_DIR=C:/Users/shawn/AppData/Local/Temp/port_class_target cargo check --release`
→ **1,857 warnings, 0 errors**, 11.3 s, artifacts `trm_uefi_kernel [lib]` and
`trm_uefi_kernel [bin]`.

| Lint | Count | Share |
|---|---|---|
| `dead_code` | 1,577 | 84.9% |
| `unused_imports` | 222 | 12.0% |
| `unused_variables` | 28 | 1.5% |
| `non_camel_case_types` | 10 | 0.5% |
| `unused_mut` | 9 | 0.5% |
| `static_mut_refs` | 4 | 0.2% |
| `non_snake_case` | 4 | 0.2% |
| `unused_unsafe` | 3 | 0.2% |

Top individual messages: `associated function 'new' is never used` (70),
`unused import: alloc::vec::Vec` (69), `unused import: alloc::vec` (66),
`unused import: alloc::string::String` (59), `unused import: alloc::format` (20).

### What the 1,577 dead_code warnings mean

They are spread across **190 of 207 modules**; only 17 modules produce none
(`allocator`, `artifact_pipeline`, `background_tasks`, `daemon_discovery`,
`epistemology_core`, `font`, `hmac_sha256`, `lib`, `modmap_all`, `monitor_daemon`,
`platform`, `redis_mock`, `remediation_engine`, `time`, `trm_integration`,
`trm_orchestrator`, `zero_human_test_gen`).

Two structural facts make this number what it is:

1. **All 1,857 warnings come from the `bin` target.** `src/lib.rs` opens with
   `#![allow(dead_code, unused_imports, unused_variables, non_snake_case)]`, so the
   library target is silent by construction. The compiler has no visibility into
   which of the 121 lib modules are live.
2. **The binary does not use the library.** `src/main.rs` contains no
   `use trm_uefi_kernel::` — it re-declares its own `mod` tree (122 modules) and
   reaches into `crate::modmap_all::*`. So the crate is compiled twice, and the
   `lib` target that `cargo metadata` reports is type-checked but never linked
   into the UEFI image.

Read together: the compiler is saying that most items in the binary's module tree
are not reachable from `efi_main`. That is a floor on unreachability, not a
ceiling — a module with one called function still reports its other items — but
it is the strongest available signal that the shipped control path is much
narrower than 207 modules.

### The safe mechanical subset

**Do not run `cargo fix --allow-dirty`.** Reasons, in order of severity:

1. It rewrites source in place across 190 files with no review, in a tree where
   the parent commits and a parallel build may be running.
2. `cargo fix` will not touch `dead_code` (1,577 of 1,857 — 85% of the debt), so it
   buys almost nothing while touching almost everything.
3. `unused_imports` fixes are *not* always safe here: `extern crate alloc;` and
   `use alloc::vec;` are load-bearing in `no_std` for macro expansion
   (`vec![]`), and the fixer's judgement about which of the duplicated
   `alloc::vec` / `alloc::vec::Vec` imports is redundant has already produced 222
   warnings from a mechanical pass. A second mechanical pass over the same lines
   is how you get a build that no longer compiles.

What I would run instead, in this order, each as its own reviewed commit:

1. **`cargo check --release --message-format=json` → triage, not edit.** Group the
   1,577 `dead_code` warnings by module and decide per module: *wire it into
   `efi_main`*, *delete it*, or *`#[allow(dead_code)]` with a reason*. That is a
   design decision per module, not a mechanical fix. It is also the only way the
   warning count becomes meaningful.
2. **`cargo clippy --release -- -W clippy::all`** to see whether the 1,577 are
   hiding real defects. `dead_code` is noise; `static_mut_refs` (4) is not — that
   one is a genuine aliasing hazard and should be fixed by hand first.
3. **`unused_imports` (222) by hand, module by module**, starting with the
   `alloc::vec` / `alloc::vec::Vec` pairs, verifying `cargo check` after each
   batch. This is the only category where a mechanical fix is *plausible*, and
   even here the `no_std` macro dependency makes it a review job.
4. **`unused_variables` (28), `unused_mut` (9), `unused_unsafe` (3)** — safe to fix
   by hand in one pass; 40 warnings total, low risk, low value.
5. **`non_camel_case_types` (10) / `non_snake_case` (4)** — cosmetic. Rename or
   `#[allow]` at the module level. Do not rename public types that the council
   server or `modmap_all` reference without checking both.

The honest framing for the release note: 1,857 warnings is not 1,857 defects. It
is 1,577 statements that the binary does not reach most of its own code, plus 222
imports a previous mechanical pass left behind. Neither blocks a release; the
first one is a question about what the kernel is supposed to be running.

---

## 6. Tool fix

`tools/surface_diff.py` reported `lib 129 lines <- not type-checked by the
compiler` as a NEVER-COMPILED file. False positive: `cargo metadata --no-deps`
lists `trm_uefi_kernel [lib] src\lib.rs` and `[bin] src\main.rs` as real targets.
The orphan detector looks for `mod X;` declarations, and a crate root never has
one — `main` was hardcoded as an exception, `lib` was not.

Fixed by replacing the hardcoded `files.discard("main")` with a `crate_roots()`
helper that reads the target list from `Cargo.toml` (falling back to the Cargo
defaults `src/lib.rs` / `src/main.rs` when no explicit `path` is declared) and
subtracts every crate root. Nothing else in the tool changed.

Re-run: `=== NEVER-COMPILED FILES (no `mod` declaration): 0 ===` /
`none - every .rs file is declared`. Negative control: `crate_roots()` returns
`['lib', 'main']`, the declared-`mod` set still holds 205 names against 207 `.rs`
files, and `lib`/`main` are absent from it — so the detector is still live and
would still catch a genuinely undeclared file.

---

## 7. What to do with the number

Stop quoting 44. Quote this instead:

- **3** modules with a real Rust surface (`pbft_consensus`, `resonance_engine`,
  `thz_wave_gateway`).
- **1** module that is a genuine kernel contract and is not finished
  (`intent_buffer`).
- **10** dead Python files whose Rust twins already exist — delete or ignore.
- **30** host-service modules that run in the FastAPI process and have no reason
  to be in a `no_std` binary, because the kernel cannot host the HTTP surface they
  exist to serve.

The port is finished enough for production **on the condition that "production"
means the council server and the kernel's own decision path**. It is not finished
if "production" means the kernel replaces the Python service — that was never
going to happen without a network stack and an executor, and the 44 were never
going to get it there.
