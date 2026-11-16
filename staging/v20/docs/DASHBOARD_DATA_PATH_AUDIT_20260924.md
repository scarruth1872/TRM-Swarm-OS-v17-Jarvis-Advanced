# Dashboard data-path audit — why tabs stop updating (2026-09-24)

**Question asked:** is the backend not running something, and why do some tabs/pages not update?

**Answer:** the backend is running and correctly wired — **70 of 161 call sites serve real data**.
The tabs that don't update are the **71 call sites that hit the kernel's catch-all**
(`{"status":"ok"}`, 15 bytes): those fetches succeed with HTTP 200 and no data, so the component
renders empty and never changes. A further 19 are **honest 501s** by design.

## How this was measured (method matters)

- Frontend inventory: `scratch/scan_dashboard_endpoints.py` — walks `dashboard/src`, collects every
  `axios.*`/`fetch(...)` URL including template-literal bases like `` `${API_BASE}/x` ``. 163 paths.
- Live probing: `scratch/probe_dashboard_endpoints.py` (through **:5183**, the browser's own path).
- Method-accurate pass: `scratch/diagnose_dashboard_tabs.py` — a GET probe of a POST action reports
  a misleading 404/stub, so the method is inferred from each call site. Results:
  `scratch/dashboard_tab_diagnosis.json`.

## Topology (verified, not assumed)

- vite dev server on :5183 is launched from **this tree's** `dashboard/node_modules` and its config
  proxies **49 prefixes to :8022**; the served bundle contains components that exist only in THIS
  tree's source (`ServicesCatalogViewport`), so **the browser is running this tree's dashboard**.
- A `/@fs/…v17…` path inside `/@vite/client` is a dependency-resolution artifact, NOT the app root —
  I nearly reported "the wrong tree is being served" on that evidence alone. It was wrong.
- :8021 (the legacy Python shard) is referenced by the **v17 tree's** config, which is not the
  running config. Nothing listens on :8021 and nothing in the live path needs it.

## Verdict per criterion

| criterion | result |
|---|---|
| backend running | ✓ :8022 HEALTHY, 15 agents, `python_active:false` |
| proxy wired | ✓ 49 prefixes → :8022 |
| real data | ~ 70/161 call sites |
| tabs that never update | **71 call sites → catch-all no-op** |
| honest refusals | 19 (501) |
| broken reads | 1 (`/council/proposal/${id}` with a literal placeholder id) |

## The dead tabs, named

| component | dead endpoints |
|---|---|
| `DiscoveryPipelineViewport.jsx` | **7/7** — `discovery/*` (amp-candidates, adc-designs, cancer-neoantigens, drug-repurposing, synthetic-circuits, pipeline/status, pipeline/run) |
| `GenerativeArchitectStudio.jsx` | **4/4** — `generative-architect/*` |
| `HumanisticTasksViewport.jsx` | **3/3** — `humanistic/*` |
| `MobileSyncViewport.jsx` | **2/2** — `api/mobile-sync/*` |
| `ServicesCatalogViewport.jsx` | **2/2** — `services/*` |
| `PillarsRoadmapViewport.jsx` | **8/10** — `wetlab/protocols`, `sensory/mesh/*`, `thz/resonance/*`, `regulatory/ind/dossiers`, `ledger/monad/*` |
| `NeuroDynamicEvolutionEngine.jsx` | 1/1 — `api/continuum/genomics/refract` |
| `ModuleMatrix.jsx` | 9/31 — module action verbs (mcts-plan, k3-kleene, neural-wall, monad/absorb…) |
| `CouncilChamber.jsx` | 2/5 — `council/propose`, `council/{action}/{id}` |
| `SpatialMesh.jsx` | 1/4 — `mesh/route` |
| rust/K3 tab | `api/continuum/*` + `swarm/microkernel/*` (11 call sites) |

## The state files exist — this is not an engine gap

Inventoried behind the stubs: `evolution_proposals.json` (10.7 MB), `monad_ledger.jsonl` (185 KB, the
ledger this session has been sealing), `regulatory_ind_dossiers.json`, `nobel_research_papers.json`,
`lupus_phase6_wetlab_and_ind_dossier.json`, `humanistic_tasks_suite_summary.json`,
`environmental_sensor_telemetry*`, thz + microkernel artifacts, `kanban_state.json`. Only
`/generative-architect` and `/services` have no state file (genuinely engine-shaped).

## Two defects independent of the rollout

1. **`/kanban/board?<query>` → 404** while `/kanban/board` serves 702 KB. The board route matches the
   exact URL and ignores query strings, so any card-level or filtered call fails.
2. **`/k3/verdict` → 404**, called by `ServicesCatalogViewport.jsx`. `/k3/status` was added this
   session; `/k3/verdict` is its sibling and is still unimplemented.

## Fix path (proven on this platform)

The same Option A rollout that made kanban/verification/testing/skills/ultrawork live: one
`<family>_handlers.rs` per batch reading the real state file, registered in `main.rs` **before** the
catch-all, mirroring the Python route's response shape. 71 call sites ≈ 6 batches. Engine-only paths
(`/infra/*` watchdog FSM, `/secrets/*` keyring, `/evolution/*` sandbox actions) stay honest 501s.

---

## Rollout progress

### Batch 1 — Pillars families (LIVE)

`handle_batch4` in `state_handlers.rs` serves these from their real state files, with the artifact's
own field names mapped onto the keys the JSX reads. Verified through :5183:

| endpoint | now returns |
|---|---|
| `/wetlab/protocols` | 6 protocols with deck layouts (`wetlab_robotics_protocols.json`) |
| `/sensory/mesh/telemetry` | `active_nodes_count` 6, `spatial_coherence` 0.778, `anomaly_score` 0.391 |
| `/thz/resonance/status` | `mean_jitter_ps` 29.1, `spectral_coherence` 0.9842, 36 nodes locked |
| `/regulatory/ind/dossiers` | 3 dossiers (IND-AMP-1123 LpxC peptide…) |
| `/ledger/monad/blocks` | 103 blocks, tip `7be07c19…`, `f=4 of N=15 (quorum 9)` derived from the roster |

`/thz/resonance/sync` and `/ledger/monad/commit` return **honest 501s** with the concrete reason
(one is engine work; the ledger has exactly one writer — the sealing path — and this server must not
open a second).

### The two independent defects (fixed)

1. **Query strings broke the whole exact-match class.** `kanban_handlers::handle` now strips the
   query once: `/kanban/board?limit=1` was 404 while `/kanban/board` served 702,063 B.
2. **`/k3/verdict` now exists** — the Kleene truth proof the service catalog advertises, in all three
   forms (`?trits=`, `?q=` engine-quantized, and the advertised POST body). Deterministic logic, so
   it is served rather than 501'd.

### RDKit feature rebuild — complete and validated

**2,897,805 rows** (2,897,819 molecules − **14 rejected** = 2,897,805, exactly; RDKit is stricter
than the hand-rolled extractor, which rejected 1), 19 columns including `canonical_smiles`, built in
53 min at 912 mol/s. Independent
recomputation from each row's **own SMILES** over 200 sampled rows: MW, HBD, HBA, rings, rotatable,
TPSA and cLogP **all match**. The validator was wrong first (it read a `smiles` column that does not
exist and got 0.0 for everything) — corrected to `canonical_smiles`.

### Batch 2 — discovery + humanistic families (LIVE)

`handle_batch5` in `state_handlers.rs`, registered in `main.rs` before the catch-all. Every return
key was read from the **consuming JSX** (`candidates` / `top_candidates` / `top_designs` / `circuits`),
not assumed from the artifact. Verified through :5183:

| endpoint | now returns |
|---|---|
| `/discovery/pipeline/status` | campaign `DISC-CAMPAIGN-1789137611`, `SOVEREIGN_VALIDATED_RA*`, `stage_breakdowns` + `summary_metrics` |
| `/discovery/amp-candidates` | 100 candidates (`antimicrobial_peptide_candidates.json`) |
| `/discovery/cancer-neoantigens` | 25 `top_candidates` of 456 (`cancer_neoantigen_candidates.json`) |
| `/discovery/drug-repurposing/candidates` | 25 `top_candidates` (`drug_repurposing_candidates.json`) |
| `/discovery/adc-designs` | 15 `top_designs` of 66 (`adc_designs.json`) |
| `/discovery/synthetic-circuits` | 5 `circuits` (`synthetic_circuit_designs.json`) |
| `/humanistic/tasks/summary` | `ALL_TASKS_COMPLETED_AND_GROUNDED`, `TRUTH_AFFIRMED (+1)`, 10 `task_summaries` |
| `/humanistic/task/{n}` | the task's own artifact (`task28_clean_water_monitoring.json`, …); an unknown number reports `no_artifact_for_task` |

**The deploy that didn't take.** The first kill+copy left the OLD code serving while every indirect
check agreed the new build was live — file sha and size matched, the running PID's `ExecutablePath`
pointed at that file, its start time equalled the file's mtime, and the new literals were present in
the deployed bytes. Only a route-level probe showed it (a canary branch that would have reported
"guard reached" never fired). Repeating the kill+copy with the PID looked up fresh made all eight
routes live. **A replaced exe on disk is not proof; a route only the new code can answer is.**

### Batch 3 — continuum + microkernel families (LIVE)

`handle_batch6` in `state_handlers.rs`. **Finding first:** none of the four engines behind these
routes (`star_matrix_lattice`, `continuum_pbft`, `microkernel_spawner`, `continuum_genomics`)
persists state to disk — their accumulations are in-memory — so the reads are derived from what is
real and the mutations are refused instead of invented.

| endpoint | now returns |
|---|---|
| `/api/continuum/star-matrix/lattice` | 4 declared nodes + the 4×4 resonance matrix, **recomputed** with the module's own `R = α·cos(s₁,s₂) + β·m₁m₂/d²` |
| `/api/continuum/pbft/ledger` | 65 real PBFT blocks (`decentralized_monad_ledger.json`), tolerance `f=4 of N=14`, 4 declared nodes |
| `/swarm/microkernel/status` | `active_subagents_count: 0` (no persisted table asserted), recovered snapshot: 15 agents, recovery 1823.7 µs, `within_5ms: true` |
| `/api/continuum/oscilloscope/spectral` | `sync_percentage` 98.42 (derived from `spectral_coherence`), `frequency_ghz` 15.53 (real carrier), `signal_dbm: null` + reason |

All seven actions (`refract`, `star-matrix/node`, `star-matrix/edge`, `pbft/audit-repair`,
`genomics/preset`, `spawn-continuum`, `terminate/{id}`) return **501 `engine_required`** with the
concrete reason.

**Verified against the reference implementation:** the served resonance matrix was compared cell by
cell with `get_star_matrix_lattice().get_full_lattice()` — **16/16 cells identical, node
name/coords/mass identical**. The matrix is derived, not copied from a run.

**Deploy failure mode, reproduced and fixed.** `Stop-Process` → `sleep 2` → `cp` let the supervisor
respawn the process from the **pre-copy** file, so the new image never loaded even though the on-disk
sha256 and the running PID's path both matched the new build. And when a second kill was used to force
a restart, the port stayed free **past a 10 s wait** — contradicting the earlier "heals within ~5 s"
note. Starting the exe directly worked; the supervisor adopts a port already in use. Only a
route-level response is evidence that a deploy landed.

### Batch 4 — module reads: `/system/modules/*` (LIVE) + a client defect

Two defects, one of them worse than a stub.

1. **The kernel was prefix-blind.** The `/system` arm matches `url.starts_with("/system")` and *falls
   through* (sets the payload, no `return`), so **every** `/system/modules/<name>` read returned the
   `/system` overview. All 22 panels showed the same global numbers and never changed per module — and
   a shape check called them "real", which is why the earlier audit scored ModuleMatrix favourably. A
   prefix-blind handler passes a stub-detector that only looks for `{"status":"ok"}`.
2. **The component couldn't reach a server at all.** `ModuleMatrix.jsx` hardcoded
   `API_BASE = "http://localhost:8021"` — the legacy Python shard, deliberately not running — so in the
   browser every fetch failed with a connection error before it left the page. Fixed to
   `http://127.0.0.1:8022`, the base the sibling viewports use; **verified in the served bundle**
   (`:5183/src/components/ModuleMatrix.jsx` now carries 8022).

`module_handlers.rs` (new sibling, registered before the `/system` payload is serialized) gives each
module its **own** source:

| endpoint | now returns |
|---|---|
| `/system/modules` | inventory **counted from the tree**: 253 core `.py`, 305 Rust, 36 skills, 3 MCP → 597 |
| `/system/modules/mailbox-bus` | the ring's own bytes via the reader that owns the format |
| `/system/modules/pbft-consensus/status` | the sealed chain (106 blocks) + `f=4 of N=15`, quorum 9 derived |
| `/system/modules/swarm-council/status` | the live roster with quorum `2f+1` |
| `/system/modules/hardware-voltage-binder` | `host_probe::phi_v()` — the kernel's own derivation, with the rail term cancelling to 1.0 stated |
| `/system/modules/turbovec-femto` | `compression_ratio = (vectors × dim × 4) / packed store bytes` |
| `/system/modules/semantic-cache`, `persistent-mmap`, `performance-insights/report` | file-backed counts; no measured score is invented (`efficiency_score: null` + reason) |
| 13 further modules | `status: "no_persisted_state"` with a `see` pointer where a live endpoint exists |
| `POST /system/modules/k3-kleene` | deterministic Kleene: `[1,1,1]`→TRUE (1.0), `[1,0,1]`→UNKNOWN (0.667), `[1,-1]`→FALSE, not bypassable |
| 7 engine actions | **501 `engine_required`** (mcts-plan, small-diffusion, gan-synthesizer, test-gen, chain-of-verification, neural-wall, monad/absorb) |

`/system` itself still returns the overview — it was never the bug.

### Batch 5 — generative-architect + mobile-sync (LIVE), and the `/system/modules` class closed

`studio_handlers.rs` (new sibling). Both families are engine-shaped, and mobile-sync is the sharper
case: its Python bodies **return success without contacting anything** — `nodes_synced: 14`,
`latency_ms: 0.18`, `biometric_token: "HW_KEY_SECURE_FINGERPRINT_OK"` — on a host with no peer, no
WiFi transport and no biometric device.

| endpoint | now returns |
|---|---|
| `/generative-architect/status` | `engine_offline` — `mode` / `topology_node_count` / `total_modifications_verified` null **with the reason** (the engine keeps its topology in memory and persists nothing) |
| `/generative-architect/skg` | `engine_offline` — empty `nodes`/`edges` + reason, not a fabricated topology |
| `/api/mobile-sync/status` | `no_peer_transport` — the peer/biometric fields null; the Python route's literals are named as unmeasured |
| 4 actions (`synthesize`, `propose-self-modification`, `push-memory`, `biometric-ping`) | **501 `engine_required`** |

**The `/system/modules` class is now closed**, not just the 22 the UI calls: the table covers the
members `routes/system.py` declares that ModuleMatrix never fetches (generative-architect,
federation-sync, architectural-rl, memory-synchronizer, skill-matrix-sync, the four microkernel
members, continuum-*), and a **fallback** answers an unmatched GET with `unknown_module_path` and an
unmatched POST with a 501. Verified: six probe paths that previously returned the `/system` overview
now each answer for themselves, and `POST /system/modules/<anything>` is refused.

### Batch 7a — the services catalog (LIVE)

`/services/catalog` and `/services/provision` both fell through to the catch-all, so the tab rendered a
hardcoded client catalogue: a **Φ_V 0.9115**, **219/219 PE modules**, **14/14 quorum** (the roster is 15
with quorum 9), **2,167 DDR antibodies**, **"BLOCK 96 SEALED"** against a ledger past block 110, plus
pricing/SLA figures no route measured — and when provisioning failed, the **client minted a random
`trm_live_…` key and displayed it as provisioned credentials**.

The kernel now serves derived facts (`services_handlers.rs`: compiled roster, sealed ledger, the
families it answers, `commercial_terms.pricing = null`) and refuses to issue credentials; the client
deleted its fallback catalogue, reports refusals honestly, and reads the banner from the payload.

**The ruler was wrong and was rebuilt.** The first call-site scanner was method-blind (it probed
POST-only routes with GET, so they landed in the catch-all and looked like stubs) and template-blind
(`${API_BASE}/x` paths were invisible), and it counted strings inside the client's display catalogue as
call sites. `scratch/scan_dashboard_sites_v2.py` is method-aware: **160 paths — REAL 97 / REFUSAL 40 /
STUB 23**.

The served bundle check earned its keep: a first rendition of the client edit returned **HTTP 500 from
vite** (an em-dash used as a JSX fallback expression is not valid JS) — the source edit looked green.

### Remaining 23 stub call sites (batch 7b+)

`/aura` (5) · `/research` (7) · `/artifacts` (5) · `/ddr` (2) · `/discovery/pipeline/run` ·
`/humanistic/tasks/run` · `/mailbox/send` · `/memory/query` · `/sensory/mesh/triangulate`

### Batch 7d — the arms that answered 200 with asserted values (LIVE)

Ledger blocks **114** (`e2856017f4127c50…`) and **115** (`52554ffd0b0b208b…`). Deployed
`71959fa0c0527dfa` (2,108,928 B). Verification: `scratch/verify_batch7d_asserted_arms.py` — **51
assertions / 0 failures** on `:8022`.

The stub detector can only find a lie with a shape. These six arms had none — they answered **200**:

| route | was asserted | now |
|---|---|---|
| `/swarm/telemetry` | `throughput_ops_sec 4251552`, `avg_latency_ns 235.2`, `phi_v 0.86`, `active_nodes 2`, `k3_consensus_rate 1.0`, `status HEALTHY_SOVEREIGN` | **measured**: the accept loop times every request (`note_request`) and the route divides by its own uptime — 4…8 ms real means over the live process; `phi_v` and `k3_consensus_rate` are `null` + the reason there is no source |
| `/learning` | three skills `ACTIVE` | the **35 modules actually in `swarm_v2/skills`** with real byte sizes, `executed_by_kernel: false` (the kernel does not run them) |
| `/security` | `active_threats 0`, `vault_status LOCKED` | `active_threats: null` (no threat store exists), `vault_key_present` read from disk, `vault_status: null` (nothing tracks it), antibody count from the store |
| `/research/stats` | `pre_registered_leads 2`, `docking_engine "Vina r=0.9097"`, `status WET_LAB_READINESS_SEALED` | the **12,270** research files on disk, the lead id parsed from their names, and `docking_pose_resolution_benchmark.json`'s **own** `resolution_verdict` (`fails_0.8A_rotated`, `fails_1.5A_transverse`) |
| `/brain/status` | `architecture`, `O(1)`, `max_recursive_steps 16` as fact | kept but labelled `declared_not_measured` |
| `/memory` | `shards 18`, `status IMMUTABLE_LOCKED` | `shards`/`utilization_pct` null with a `design_figures` object carrying the source doc; the three real stores embedded from `/k3/status`; `total_trits` derived from the mapped file |

Client: the artifacts panel (which showed `Registry: 0 Entities` and nothing else) now prints the
kernel's `unavailable.reason`, and the mailbox panel's "No mailboxes found" now carries the route's
`messages_reason` (the ring holds counters and 64-byte slots, not decodable text) — verified in the
served bundle from `:5183` (HTTP 200, 806,712 B, 6 new bindings, 0 fabricated tokens).

Two derivations were then tightened and the image superseded (block 115): a lead id is parsed as
`TRM-ABX-<digits>` case-insensitively — the first exact-case `find()` returned an **empty list**
because the research filenames are lowercase, which the contract test caught — and the package marker
`__init__.py` is no longer listed as a skill.

**`/brain/reason` closed in the same pass** (ledger block **116** `67c1def10a753daa…`, deployed
`fd6b2828beea62b1`, 2,134,016 B). It was the last member of the class and the most literal: the arm
computed its trit from a **hardcoded array** (`sample_context = [0.25, 0.5, -0.1, 0.8, 0.3, -0.2, 0.1,
0.4]`) and returned `status COMPLETED, steps_taken 4, confidence 0.94, k3_trit 1, "Non-contradiction
verified via Mamba-3 SSM recurrence"` for **any** body. It now reads `{"query": …}` and answers with
the kernel's own retrieval (`/k3/retrieve?k=3`): the hits become steps with their real unit, fidelity
and scores, `steps_taken` is the hit count, `halted` states the threshold it uses, `engine: null`
carries the reason no reasoning engine is wired, and the old chain is kept under `previously_asserted`.
An empty query is 400 `empty_query`. `actions_handlers::enc()` is now `pub` and shared by both
retrieval call sites. Verification is **60 assertions / 0 failures**, whose decisive pair a canned arm
cannot pass: two different queries must give different steps, and the same query must reproduce with
timings excluded (`elapsed_us` is a real measurement — the first test version froze it and failed, a
change-detector on a timing field).

**Ruler unchanged this pass** (`scratch/dashboard_sites_SCAN_V2.json`: 167 rows — REAL 96 / REFUSAL 65
/ STUB 0 / ERROR 5 / UNREACHABLE 1); it needed no change because every value these arms now serve is a
refusal, a measurement, or a disk-derived fact. The five ERRORs remain prober artifacts (two template
literals, two empty POST bodies, one honest-absent payload).

### Batch 7c — the engine actions, and the asserted payloads no stub detector can see (LIVE) — rollout closed

Sealed as ledger **block 113** (`8d2b113a0cfbca72…`), 67/67 assertions on `:8022`, deployed `c411f5446dde7492` (2,067,456 B). **`/artifacts` (5) · `/ddr` (2) · `/discovery/pipeline/run` · `/humanistic/tasks/run` · `/mailbox/send` · `/memory/query` · `/sensory/mesh/triangulate`**

| site | answer now |
|---|---|
| `/artifacts/{review,test,integrate,batch-review,batch-integrate}` | 501 `engine_required` — the pipeline queue and review states are the Python engine's `swarm_v2.app_v2.pipeline` |
| `/ddr/antibodies/add`, `/ddr/scan` | 501 — the antibody vault writer / the DDR scanner |
| `/discovery/pipeline/run`, `/humanistic/tasks/run`, `/sensory/mesh/triangulate`, `/mailbox/send` | 501 — each names its engine |
| **`/memory/query`** | **REAL** — wired to the kernel's own `/k3/retrieve`; returns the ranks, fidelities and scores that route derives. `k` from the body, `empty_query` in-band, `match_percentage` = document fidelity with the raw ranked score kept as `kernel_score` (nothing rescaled into a claim) |

`GET` is never claimed by the new guard, so the reads that already worked are untouched (asserted).

**The harder half of the class — arms that answer 200 with asserted values** (invisible to any stub detector, which is why the audit kept missing them):

- `/artifacts` asserted `total/verified/pending = 14/14/0` and listed four files — `trm_uefi_kernel.efi`, `microkernel_ledger.json`, `TRM-ABX-219.lock`, `TRM-ABX-082.lock` — **none of which `find` locates anywhere in the tree**. Now serves no artifact it cannot derive, with `unavailable.reason`.
- `ddr_antibody_count: 1801` appeared in **three** routes (`/memory`, `/ddr`, `/infrastructure`); the real store `swarm_v2_artifacts/DDR_ANTIBODIES.json` — the one `swarm_v2/core/ddr_antibody.py` owns — holds **27**. All three now read the store (asserted equal to its length in the test itself), and `null` if unreadable.
- `/mailbox` · `/comms` served two invented messages (`170/170 REAL modules wired`, `Consolidation motion enacted 14-0-0`) with fabricated timestamps. The transport's real content is the ring's own state read from its bytes; the ring carries counters and 64-byte slots, **not decodable message text**, so the arm now serves the counters and says so.

**Client:** 12 call sites surfaced — App.jsx artifact actions, antibody add, the DDR scan and mailbox send buttons (which had **no catch at all**: an unhandled rejection that made the button look dead), and the memory query (`catch { }`, empty). All now state the kernel's own reason; the memory panel carries it inline.

**Ruler:** `scan_dashboard_sites_v2.py` → **STUB 23 → 0**; final **160 paths — REAL 96 / REFUSAL 65 / STUB 0 / ERROR 5 / UNREACHABLE 1**. The 5 ERRORs are prober artifacts (two template literals probed verbatim, two empty POST bodies, one honest-absent payload) and the 1 UNREACHABLE is an LLM generation timeout — not platform defects. The scanner itself needed the fix that made this visible: `probe()` truncated HTTPError bodies to **300 chars before** `classify()` parsed them, so every refusal with a long message was unparseable and got filed as OTHER.

**Recorded for the next pass (asserted values on 200 arms):** `/swarm/telemetry` (`throughput_ops_sec` 4251552, `avg_latency_ns` 235.2, `phi_v` 0.86, `active_nodes` 2, `k3_consensus_rate` 1.0) · `/learning` (three skills `ACTIVE`) · `/security` beyond the count (`active_threats` 0, `vault` LOCKED) · `/research/stats` (`pre_registered_leads` 2, `docking_engine` "Vina r=0.9097" against the measured 0.64) · `/brain/status` (`max_recursive_steps` 16, architecture string) · `/memory` `shards` 18 + `utilization_pct` 0.59 (already labelled a design figure) · the three GET consumers whose payloads changed still render empty without the stated reason.

### Batch 7b — Aura Studio (a separate product) + the research actions (LIVE)

Sealed as ledger **block 112** (`a1267dbfddacc4bb…`), 47/47 assertions on `:8022`, deployed `920ae61caa264c57` (2,036,736 B).

Two whole families were landing in the kernel catch-all (`{"status":"ok"}`):

| family | what it is | answer now |
|---|---|---|
| `/aura/*` (5 stubs + every sibling path) | **Aura Studio** — a *separate product* (`aipro2`), not a module of this platform; `:3000` closed | 501 `separate_service` naming the product |
| `/research` **actions** (start/stop/run/synthesis/topics, `DELETE topics/<t>`) | needs the research daemon: cycle scheduler, topic-store writer, synthesis model | 501 `engine_required`, per-route reason |

The `/research` **reads** were already real (`/research/stats` from `main.rs`, `/research/{tasks,theorems,campaign}` from `state_handlers` batch 3) — `research_handlers::handle` returns `None` for `GET`, so the new guard cannot shadow them. An unknown `/research/<x>` `GET` still falls through (asserted).

**The client was the other half of the defect:** the panels swallowed the error, so the refusal was invisible and the UI reported success.
- `AuraStudio.jsx` rendered *asserted* telemetry — `100 METACONCEPTS ACTIVE`, `15.53 THz PHONON LOCK`, `⚡ Energy: 1.0`, `Φ Coherence: 0.985` — none of which any engine produced. All four now derive from loaded data or state `no data` / `NO ENGINE`, and a banner carries the kernel's own message.
- `App.jsx` `console.error`'d all six research actions (a click looked like it worked) and set the synthesis panel from `res.data.synthesis`, which the catch-all left `undefined`. Each now surfaces the refusal; synthesis reports the reason.

**Ruler:** `scan_dashboard_sites_v2.py` → **STUB 23 → 12** (the 11 targeted sites closed). Remaining: `/artifacts` (5), `/ddr` (2), `/discovery/pipeline/run`, `/humanistic/tasks/run`, `/mailbox/send`, `/memory/query`, `/sensory/mesh/triangulate`.

**Served-bundle check** (the source edit is not the proof): both bundles fetched from `:5183` — 200, zero fabricated tokens, new bindings present.

Deploy note: the healer had to be stopped **first** (it re-heals `:8022` in ~3 s and the serving process holds the exe → `cargo` fails at the link step with no binary), then the new image started directly and the supervisor restored.

### Batch 6 — the mesh family + council propose (LIVE)

Two families, one of them a whole prefix.

**`/mesh/*` was prefix-blind the other way round**: the kernel answered *every* path under it —
including `/mesh/does-not-exist` — with **one hardcoded two-node topology**: `"Jarvis OS Gateway"` and
`"Antigravity Sovereign Microkernel"`, `ping_ms` 0.12 / 0.08, a `latency_ms: 0.10` link. Neither node
exists on this platform and nothing measured those numbers. The old arm is **deleted** from `main.rs`
and the family is served per path by the new `mesh_handlers.rs`:

| endpoint | now returns |
|---|---|
| `/mesh`, `/mesh/topology` | the live **15-agent roster** (each `latency_ms: null` — no inter-agent RTT is measured) + the registry's 4 probed peers → `node_count` 19; `links: []` **with a reason** |
| `/mesh/peers` | the federation registry's own entries, unmodified |
| `/mesh/nodes` | the roster alone |
| `/mesh/stats` | counts derived from roster + registry; no throughput asserted |
| `/mesh/log` | `no_persisted_state` |
| `/mesh/does-not-exist` | **`unknown_mesh_path`** — no path can borrow another member's payload |
| `POST /mesh/route`, `/announce`, `/ipc` | **501 `engine_required`** |

**The same class survived on two sibling routes** and was closed with it: `/infrastructure/nodes`
served the identical two invented nodes with the identical ping times, and
`/swarm/spatial/entanglement` served the identical 0.10 ms link. Both now build from one shared
builder (`mesh_handlers::mesh_nodes`) — so there is **one owner** of the mesh node set.

**`POST /council/propose` ignored its request entirely** and filed a hardcoded
`"Operator Sovereign Directive"` / `Architecture` / `OPERATOR` for every caller. It now parses the
submitted `topic`/`category`/`proposer`, answers **400 `topic_required`** on an empty topic, and states
in `basis` that the tally is the roster's **fixed per-agent rule** (a domain match approves, every 7th
agent abstains) — a procedural record, not a content-derived judgement. Added: `/council/members`
(roster + derived quorum 9, f=4) and `/council/ledger` (the sealed chain, from its single owner).
`POST /council/<action>/<id>` is now an honest **501** instead of falling to the stub.

**The four latencies the mesh serves (22.22 / 102.99 / 22.6 / 927.65 ms) are real**: they are the
federation registry's own `/health` probe values, and the verifier asserts they are equal to
`swarm_v2_federation/registry.json` — re-served data, not kernel measurement.

Deployed `0977fa82019d44dc`. `scratch/verify_batch6_mesh_council.py` — **39 assertions, 0 failures**,
run against `:8022` and again **through the `:5183` proxy** (the path the browser uses). Ledger
**block 109**.

### Remaining batches

Batch 1: 8 Pillars · batch 2: 10 (discovery + humanistic) · batch 3: 12 (rust/K3 + refract) ·
batch 4: ModuleMatrix fully covered · batch 5: generative-architect + mobile-sync (6 call sites) +
the module family closed · batch 6: the `/mesh` family (7 paths) + council propose/members/ledger
+ the two sibling mesh routes · batch 7a: services catalog + its client fallback · batch 7b: the
whole `/aura` family (a separate product) + the 6 research actions and their silent client · batch 7c:
the 12 engine actions + `ddr_antibody_count` 1801→the store's 27, the fabricated `/artifacts` list and
the two invented comms messages.
**Rollout closed: STUB 0.** Final ruler — 160 paths: REAL 96 / REFUSAL 65 / STUB 0 / ERROR 5 /
UNREACHABLE 1, the five ERRORs being prober artifacts and the one UNREACHABLE an LLM timeout.
The next class is the *asserted 200 arms* listed under batch 7c.
