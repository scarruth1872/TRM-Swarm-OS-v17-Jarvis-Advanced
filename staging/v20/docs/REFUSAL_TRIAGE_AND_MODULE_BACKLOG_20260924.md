# The 65 refusals: why, and the module backlog they define (2026-09-24)

Source: `scratch/dashboard_sites_SCAN_V2.json` (class REFUSAL), re-probed live against `:8022` and
grouped by the reason each route returns. **65 refusal rows = 53 real call sites + 12 scanner template
probes** (paths still carrying `${...}` — `POST /infra/reset/${serviceName}` and 11 like it; those are
the ruler's own probes, not the dashboard's calls).

A refusal here is a **501 with a named reason**. It is not a failure: before the rollout these same
routes answered `{"status":"ok"}`, which the UI presented as success.

## 1. Why each was refused — the families

| # | family | sites | owner the refusal names | class |
|---|---|---|---|---|
| 1 | `/infra/health`, `/infra/history`, `/infra/reset|restart/…` | 5 (+4 tmpl) | self-healing watchdog FSM, Python engine | ENGINE_PORT — **UPDATE 2026-09-28:** `/infra/health` and `/infra/history` are **SERVED**, not refused; the FSM now exists in Rust (`trm_uefi_kernel/src/self_healing_infra.rs`, linked by `#[path]`; the same implementation `strip_self_healing_infra.dll::strip_eval_service_health` exports). Only the `reset`/`restart` sites remain ENGINE_PORT |
| 2 | `/secrets/keys`, `/secrets/rotate`, `/secrets/decrypt`, `/secrets/delete` | 4 (+1 tmpl) | engine-held keyring | ENGINE_PORT |
| 3 | `/evolution/mutate|verify|integrate|auto-evolve` | 1 (+3 tmpl) | Python mutation engine (sandbox) | ENGINE_PORT |
| 4 | `/research/run|start|stop|synthesis`, `/research/topics` | 4 (+2 tmpl) | Python scheduler + topic store | ENGINE_PORT |
| 5 | `/artifacts/review|test|integrate|batch-review|batch-integrate` | 5 | `swarm_v2.app_v2.pipeline` | ENGINE_PORT |
| 6 | `/system/modules/{chain-of-verification/verify, gan-synthesizer/generate, mcts-plan, monad/absorb-experiences, neural-wall/inspect, small-diffusion/denoise, zero-human-test-gen/generate}` | 7 | each module owns its engine | ENGINE_PORT |
| 7 | `/api/continuum/{pbft/audit-repair, star-matrix/edge, star-matrix/node, genomics/preset, genomics/refract}` | 5 | `continuum_pbft`, `star_matrix_lattice`, `continuum_genomics` | ENGINE_PORT |
| 8 | `/generative-architect/synthesize`, `/propose-self-modification` | 2 | architect engine | ENGINE_PORT |
| 9 | `/discovery/pipeline/run` | 1 | discovery pipeline orchestrator | ENGINE_PORT |
| 10 | `/humanistic/tasks/run` | 1 | humanistic suite runner | ENGINE_PORT |
| 11 | `/ddr/scan`, `/ddr/antibodies/add` | 2 | DDR scanner / antibody vault writer | ENGINE_PORT |
| 12 | `/mailbox/send` | 1 | IPC ring writer (`agent_mailbox_bus` slot protocol) | ENGINE_PORT |
| 13 | `/mesh/route` | 1 | `swarm_v2/routes/mesh.py` density scoring | ENGINE_PORT |
| 14 | `/sensory/mesh/triangulate` | 1 | sensory-mesh triangulation engine | ENGINE_PORT |
| 15 | `/swarm/microkernel/spawn-continuum`, `/terminate/{id}` | 1 (+1 tmpl) | `MicrokernelSpawner` | ENGINE_PORT |
| 16 | `/swarm/spatial/simulate` | 1 | live compute job, no persisted result | ENGINE_PORT |
| 17 | `/thz/resonance/sync` | 1 | THz phase engine vs live nodes | ENGINE_PORT |
| 18 | `/aura/{execute, field, metaconcepts, cognition/branch, council-integrate}` | 5 | **a separate product** (`aipro2`, Aura Studio) | SEPARATE_PRODUCT |
| 19 | `/api/mobile-sync/{biometric-ping, push-memory}` | 2 | on-device biometric key / peer transport | EXTERNAL_HARDWARE |
| 20 | `/ledger/monad/commit` | 1 | the ledger's **one** writer (the sealing path) | CORRECT_REFUSAL |
| 21 | `/services/provision` | 1 | no token issuer exists on this platform | CORRECT_REFUSAL |
| 22 | `/system/modules/mailbox-bus/send` | 1 | **not a route this kernel serves** — the caller invented it | CLIENT_BUG |

Totals: **43 ENGINE_PORT · 5 SEPARATE_PRODUCT · 2 EXTERNAL_HARDWARE · 2 CORRECT_REFUSAL · 1 CLIENT_BUG = 53**.

## 2. The module backlog — what must be set up

**Kernel-side (each is a Python engine with no Rust port — 43 refused sites across 17 modules; _UPDATE 2026-09-28: module 1 has left this list — the infrastructure watchdog FSM now HAS a Rust implementation, `trm_uefi_kernel/src/self_healing_infra.rs`, linked by `#[path]`, and `/infra/health`, `/infra/history` and `/infrastructure/status` are served rather than refused_):**

1. ~~Infrastructure watchdog FSM~~ — **UPDATE 2026-09-28: no longer a gap; the FSM is implemented in Rust and served** (`trm_uefi_kernel/src/self_healing_infra.rs`, the same implementation `strip_self_healing_infra.dll::strip_eval_service_health` is) · 2. Secrets vault keyring · 3. Evolution/mutation engine ·
4. Research scheduler + topic store · 5. Artifact pipeline · 6. Seven `/system/modules` engines ·
7. Continuum: PBFT self-healing, star-matrix lattice, genomics persona/refract · 8. Generative
architect · 9. Discovery pipeline orchestrator · 10. Humanistic suite runner · 11. DDR scanner + vault
writer · 12. IPC ring writer · 13. Mesh routing · 14. Sensory-mesh triangulation · 15. Microkernel
spawner · 16. Spatial simulation · 17. THz phase engine.

**Client-side:** one call site to the invented `/system/modules/mailbox-bus/send`.

**Engine/data the task suite needs (`docs/TASK_SUITE_TRUTH_20260924.md`):** docking engine (7 tasks),
public omics corpora (24), model weights (24), gnomAD/1000G (15), PharmGKB/CPIC (7), ClinVar (6) — the
147 acquirable tasks; plus 55 that need the physical world and ~39 held only by the operator
(the identity of TRM-ABX-220).

## 3. Mitigation

Applied in this pass:

- **CLIENT_BUG closed**: the ModuleMatrix mailbox button POSTed a route the kernel does not serve and
  then rendered `Mailbox dispatch: undefined | Pending in Archi ring: undefined` — a success-shaped
  alert around a call that never happened. It now reports the route's own answer.
- **The remaining silent action sites closed** (`scratch/fix_client_refusals_pass.py`): a strict scan
  found **40 action sites on refused routes**, of which **8 had no reason path at all** — their only
  error handling was a `console.error`, so a 501 reached the console and the operator saw a button that
  did nothing. Each now alerts with the kernel's own message (`PBFT Audit refused: …`, `Mobile sync
  push-memory refused: …`). Two of the eight initially got a fallback label naming the *wrong* route;
  fixed before it shipped. Guard rails in the pass: only POST/DELETE sites, never inside a
  `setInterval` scope (a poll would turn one alert into a storm), one alert per catch.
  **Residual after the pass: 0 of 40 silent** — the other 32 already carried a reason path from
  batches 7b/7c/7d. Served bundles from `:5183`: 5/5 HTTP 200 with the bindings present.

Available but not applied (each is a decision, not a defect):

- **ENGINE_PORT (43 sites)** — two honest paths: port each engine to Rust (the doctrine's answer, and a
  program of work, one module per batch), or have the kernel refuse and the operator run the offline
  Python engine — which is exactly what the refusal already says. What must NOT happen is the old
  behavior: a 200 with `{"status":"ok"}`.
- **SEPARATE_PRODUCT (5)** — deploy `aipro2` if Aura Studio is wanted; it is not a module of this kernel.
- **EXTERNAL_HARDWARE (2)** — no mitigation without the hardware (biometric enclave, a peer transport).
- **CORRECT_REFUSAL (2)** — none wanted: a second ledger writer would break the chain, and minting
  credentials would fabricate an issuer.

## 4. The capability finding that outranks the refusals

Running a "locally satisfiable" band end to end —
`py -3 scratch/run_tasks_441_445_optimization.py` — reports **5 tasks finished in 48.39 ms**, with
"200 gradient descent convergence runs" in 10.13 ms. Reading the runner:

```python
final_loss = round(init_loss * math.exp(-epochs / 30.0) + random.uniform(1e-6, 1e-4), 6)
"iterations_to_convergence": epochs,   # epochs = random.randint(45, 180)
```

The artifact is `random.uniform()` inside domain vocabulary: fields, ranges and counts all plausible,
no optimization performed. The 200 records have 199 distinct `initial_loss` values and a
closed-form-consistent `final_converged_loss`, so schema checks and distribution checks both pass —
which is why this class survives an audit that only asks "does the artifact look right?".

**Consequence:** the executor layer can produce results *shaped like* the task, at machine speed. Until
every artifact carries provenance that distinguishes computed from drawn, the platform cannot claim the
suite's tasks are "completed" — the same class as the asserted 200-arms closed in batch 7d, one layer up.

## 5. Verdict: what can this system complete, as of right now

| capability | state |
|---|---|
| Serve and derive every dashboard tab from real stores | **yes** (batches 1–7d; STUB 0) |
| Refuse honestly what it cannot do, with the reason on screen | **yes** (65 refusals, all named) |
| Retrieval, ternary verdicts, council motions, kanban board, ledger sealing | **yes** (real routes, verified) |
| Bulk descriptor compute (RDKit, offline Python) | **yes** (2,897,805 molecules, 912 mol/s) |
| Local LLM inference | **yes** (`llama-server` on `:8587`, Ternary-Bonsai-8B Q2_0 g64) |
| Execute the suite's computational tasks **as specified** | **no** — runners emit generated records (§4) |
| Port the 17 engine modules | **partly, as of 2026-09-28** — one landed: the infrastructure watchdog FSM is implemented in Rust (`trm_uefi_kernel/src/self_healing_infra.rs`) and served; the other 16 are each a batch of work, none landed |
| The 147 acquirable tasks | **no** — acquisition not performed (docking engine absent; corpora absent) |
| The 55 physical tasks | **no** — samples, labs, TPM/HSM, a second node |
| The ~39 operator-held tasks | **no** — TRM-ABX-220 has no structure on disk |

So: the platform is **able to complete tasks that are self-contained and honest about their output**,
it is **not yet able to complete the 1000-task suite**, and the first thing standing in the way is no
longer the missing engines — it is that its own task artifacts do not distinguish computed work from
generated work.
