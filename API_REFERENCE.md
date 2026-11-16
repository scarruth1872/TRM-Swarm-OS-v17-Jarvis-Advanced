# Swarm OS v19.026 — API Reference

**Base URL:** `http://127.0.0.1:8022` (native Rust council server)

> **Corrected 2026-09-21.** The base URL was documented as `:8021` — the **retired** Python HTTP proxy. The native platform has been on `:8022` since the Rust migration. The documented `/docs` and `/openapi.json` do **not** exist: the council is a `tiny_http` server with a fixed route set and generates no OpenAPI spec. Unknown paths now return an honest `404` (previously *any* path returned `200 {"status":"ok"}`, masking missing endpoints). `OPTIONS` returns `200` for CORS preflight.

All endpoints accept and return `application/json`.

---

## Route Domains (15 modules)

| Domain | Routes | Key Endpoints | Added |
|---|---|---|---|
| **Swarm** | /swarm/* | healing/status, health, topology, experts, meta-orchestrator, orchestrator/stats | ✅ |
| **Orchestrator** | /orchestrator/* | status, remediate, remediation-history, auto-remediate/toggle | ✅ |
| **Coordination** | /coordination/* | broadcast, round/{id}, rounds | ✅ |
| **Learning** | /learning/* | ingest, ingest-file, skills, use | ✅ |
| **Skills** | /skills/* | portable, matrix/sync, matrix/topology, matrix/render, github-acquire | ✅ |
| **Security** | /security/* | analyze, stats, threats, scan, false-positive | ✅ |
| **Artifacts** | /artifacts/* | list, approve, reject, security-verify | ✅ |
| **Mesh** | /mesh/* | topology, nodes, announce, agents | ✅ |
| **Memory** | /memory/* | store, recall, search | ✅ |
| **Research** | /research/* | paper, survey, analysis | ✅ |
| **Evolution** | /evolution/* | mutate, crossover, select | ✅ |
| **System** | /system/* | resources, health, info | ✅ |
| **Ultrawork** | /ultrawork/* | missions, stats | ✅ |
| **Mailbox** | /mailbox/* | send, agents, inbox | ✅ |
| **Verification** | /verification/* | verify, batch | ✅ |
| **K3 Ternary** | /k3/* | bench, reduction, reduce, sram/bench, sram/patterns, stress117, stress118 | ✅ 2026-09-22 |
| **TurboVec** | /turbovec/* | bench, femto, tritpack | ✅ 2026-09-22 |
| **GPU** | /gpu | adapter identity (RX 6700 XT) — VRAM utilisation unavailable via WMI | ✅ 2026-09-22 |
| **Council** | /council/* | status, proposals, proposal/{id}, propose, enact/{id}, cadence | ✅ |

### Core-primitive routes (added 2026-09-22)

These expose the platform's actual intelligence primitives as **measurable** endpoints — previously
K3 ternary and TurboVec existed in the kernel but had no route, so their behaviour could not be observed.

| Endpoint | Measures |
|---|---|
| `POST /k3/reduce` | SENSE→REDUCE stage: real `quick_reduction` over a caller-supplied f32 vector → trits |
| `GET /k3/bench` | K3 ternary ALU throughput |
| `GET /k3/reduction` | R^d → T^m reduction throughput (64-dim) |
| `GET /k3/sram/bench` | K3 SRAM store/lookup |
| `GET /k3/sram/patterns` | per-pattern timing for all **15** virtual SRAM architectures |
| `GET /k3/stress117` | Article 117 composite replication (500,000 tasks) |
| `GET /k3/stress118` | Article 118 composite replication (500,000 ops, all 15 patterns) |
| `GET /turbovec/bench` | 16× trit quantization throughput |
| `GET /turbovec/femto` | Mamba-3 `femto_step` SSM step latency |
| `GET /turbovec/tritpack` | trit↔code packing throughput |

> **Replication results (measured, not asserted).** Article 117: 0.9612 µs/op, 95% CI [0.9583, 0.9641] vs
> claimed 0.7824 — outside the CI (1.23×). Articles 118 & 157: 7,586 ns/op vs claimed 235.2 ns (32.26×).
> Errata are attached in the served journal payload; see `CHANGELOG.md` 19.024.
>
> **Council caveat.** `/council/propose` returns a verdict, but `evaluate_proposal()` never reads `topic` —
> votes are a function of array position (`i % 7`) and a category string. Any proposal, including a
> deliberately nonsensical one, returns an identical 12/2/0. See `docs/FINDING-council-deliberation-is-templated.md`.

**Total endpoints: 40+ documented.** An earlier revision of this line claimed "OpenAPI spec at /openapi.json includes all 174+ internal routes" — that contradicted the correction above and was false: **no OpenAPI spec exists**, and `/openapi.json` is not served. The route set is fixed in `host_council_server/src/main.rs`; that file is the only authoritative route list.

---

## Health & Soul

### `GET /health`

Complete system status — agents, distributed mesh, memory.

```json
{
  "status": "v11_online", "phase": 11, "agents": 12,
  "mesh": { "distributed": true, "parallel_availability": "100%" }
}
```

### `GET /swarm/soul`

Aggregates philosophical status, harmony index, and autonomous evolution proposals.

```json
{
  "harmony_index": 1.0,
  "reflection": "The swarm is aligned with the Architect's vision...",
  "autonomous_proposals": ["PROPOSAL_integration_plan.md"]
}
```

### `GET /swarm/telemetry`

Real-time emergence telemetry including per-agent stack health and VRAM stats.

> [!NOTE]
> Federated requests require HMAC signature verification.

---

## Swarm / Chat

### `GET /swarm/experts`

List all 12 agents with skills, mesh node IDs, and **Cognitive Stack status**.

```json
{
  "role": "Architect",
  "stack": { 
    "executive": "gemma3:270m", 
    "reasoning": "TRM-7M", 
    "offloads": 12 
  },
  "config": {
    "H_cycles": 2,
    "L_cycles": 1,
    "complexity_threshold": 0.6
  }
}
```

### `POST /swarm/chat`

```json
{ "role": "Lead Developer", "message": "write a FastAPI hello world" }
```

### `POST /swarm/broadcast`

Parallel coordination across all 12 agents.

```json
{ "message": "what are your primary specialties?" }
```

Returns responses from all 12 agents simultaneously.

### `POST /swarm/pipeline`

```json
{ "task": "build a JWT auth module", "agents": ["Architect", "Lead Developer", "QA Engineer"] }
```

Full multi-agent build pipeline.

### `POST /swarm/spawn`

```json
{ "parent_role": "Lead Developer", "sub_role": "Frontend Developer", "task": "build a login form" }
```

### `GET /swarm/subagents/{role}`

List sub-agents spawned by a parent role.

### `GET /swarm/memory/{role}`

View an agent's short-term conversation and long-term facts.

---

## Evolutionary Ecosystem (Phase 16/17)

> [!IMPORTANT]
> External requests to these endpoints require a valid **HMAC-SHA256** signature in the query parameters (`data` and `signature`).

### `GET /evolution/genome`

View the current structural complexity and "genetic" map of the codebase.

### `POST /evolution/mutate`

Manually trigger a mutation cycle on a target file.

```json
{ "target_file": "swarm_v2/core/agent_mesh.py" }
```

### `GET /evolution/proposals`

List all generated code mutations with status (pending, verified, failed).

### `POST /evolution/verify/{id}`

Re-trigger sandbox verification for a specific genetic proposal.

---

## Autonomous Action Tags

Agents emit structured tags in their chat responses (`/swarm/chat`, `/swarm/broadcast`) to trigger local side-effects. These are parsed and executed automatically by the backend.

### `WRITE_FILE:`

Single file creation.

```text
WRITE_FILE: path/to/file.py
# (Code content starts here)
```

### `CREATE_FILES:`

Multi-file generation block.

```text
CREATE_FILES:
--- file1.py ---
(content)
--- file2.py ---
(content)
---END---
```

### `MAKE_DIR:`

Explicit directory creation.

```text
MAKE_DIR: folder/subfolder
```

---

## Artifact Pipeline

### `GET /artifacts`

List all artifacts with status (`pending`, `approved`, `rejected`, `tested`, `integrated`).

### `GET /artifacts/{filename}`

Read artifact content and metadata.

### `GET /artifacts/stats`

Pipeline statistics summary.

### `POST /artifacts/review`

```json
{ "filename": "auth.py", "action": "approve", "notes": "LGTM" }
```

`action`: `"approve"` | `"reject"`

### `POST /artifacts/test`

```json
{ "filename": "auth.py" }
```

Run automated tests. Timeout: 120s.

### `POST /artifacts/integrate`

```json
{ "filename": "auth.py" }
```

Move approved artifact into `swarm_v2_integrated/` using a deterministic Lobster Pipeline (Asynchronous).

### `POST /artifacts/deploy`

```json
{ "project_name": "Nexus Platform" }
```

Full autonomous deployment pipeline. Timeout: 300s.

### `POST /artifacts/remediate`

Trigger autonomous remediation of rejected artifacts. Timeout: 300s.

---

## Learning Engine (Phase 3)

### `POST /learning/ingest`

```json
{ "name": "stripe-api", "content": "# Stripe API...", "source": "https://stripe.com/docs" }
```

Ingests documentation text, extracts endpoints/examples, registers a new skill.

**Response:**

```json
{ "status": "learned", "skill": "stripe-api", "endpoints_found": 12, "examples_found": 3 }
```

### `POST /learning/ingest-file`

```json
{ "filepath": "C:/docs/github_api.md" }
```

### `GET /learning/skills`

List all learned skills with usage stats and ingestion log.

**Response:**

```json
{
  "skills": [{ "skill_name": "stripe-api", "endpoints": {}, "usage_count": 4 }],
  "stats": { "total_learned": 1, "total_usages": 4 },
  "log": [{ "action": "learned", "skill": "stripe-api", "timestamp": "..." }]
}
```

### `POST /learning/use`

```json
{ "skill_name": "stripe-api", "task": "create a payment intent for $50 USD", "agent_role": "Lead Developer" }
```

### `DELETE /learning/skills/{name}`

Remove a learned skill from the registry.

---

## MCP Tool Synthesizer (Phase 3)

### `POST /synthesize/mcp`

Generate a complete FastAPI MCP server from a learned skill.

```json
{ "skill_name": "stripe-api", "description": "Stripe MCP bridge", "use_llm": false }
```

Set `use_llm: true` to have Devo review and enhance the code (slower, higher quality).

**Response:**

```json
{
  "status": "synthesized",
  "tool": { "name": "stripe-api", "port": 9100, "file": "swarm_v2_synthesized/stripe-api_server.py" },
  "message": "Run with: python swarm_v2_synthesized/stripe-api_server.py"
}
```

### `POST /synthesize/skill`

```json
{ "skill_name": "stripe-api" }
```

Generate a hot-loadable Python `Skill` class.

| Method | Path | Description |
| :---: | :--- | :--- |
| `POST` | `/synthesize/skill` | Generate Python Skill class from a learned skill |
| `GET` | `/synthesize/tools` | List all synthesized tools |
| `GET` | `/synthesize/log` | View synthesis history & port mappings |

List all synthesized MCP tools with port assignments.

---

## P2P Agent Mesh (Phase 3)

### `GET /mesh/topology`

Full mesh topology — all 12 nodes with status, specialties, task counts.

```json
{
  "version": 110, "total_nodes": 12, "alive": 12, "stale": 0,
  "nodes": [{ "node_id": "64c41ea12216", "name": "Archi", "status": "online" }],
  "connections": []
}
```

### `GET /mesh/stats`

```json
{ "total_nodes": 12, "alive_nodes": 12, "topology_version": 110, "total_tasks_routed": 8 }
```

### `GET /mesh/peers`

Discover peers with optional query filters:

- `?role=Security Auditor`
- `?specialty=Docker`

### `POST /mesh/route`

Route task to the best-matching node automatically.

```json
{ "task": "audit JWT auth for security vulnerabilities" }
```

**Response:**

```json
{
  "routed_to": { "node_id": "c9af0d296bae", "name": "Shield", "role": "Security Auditor" },
  "task": "audit JWT auth for security vulnerabilities",
  "mesh_version": 60
}
```

**Routing scoring:**

- +10 per specialty keyword match
- +8 if role name appears in task
- +5 per skill keyword match
- +20 if `required_specialty` matches
- -0.1 per accumulated task count (load balancing)

### `GET /mesh/log`

Recent mesh events (node joins, task routes). Optional `?limit=N` (default 20).

---

## Global Memory (Phase 3)

### `POST /memory/contribute`

```json
{
  "author": "Devo", "author_role": "Lead Developer",
  "content": "FastAPI Depends() enables clean auth middleware injection",
  "memory_type": "fact",
  "tags": ["fastapi", "auth"]
}
```

`memory_type`: `"fact"` | `"experience"` | `"observation"` | `"skill_use"`

### `POST /memory/query`

Keyword-vector cosine similarity search across all agent memories.

```json
{ "query": "JWT authentication security", "top_k": 5 }
```

**Response:**

```json
{
  "results": [{
    "score": 0.87,
    "entry": { "author": "Shield", "content": "Always validate JWT exp claim...", "memory_type": "fact" }
  }]
}
```

### `GET /memory/stats`

```json
{
  "total_memories": 24,
  "by_type": { "fact": 10, "experience": 14 },
  "by_author": { "Devo": 8, "Shield": 4 },
  "total_accesses": 31, "sync_events": 3
}
```

### `GET /memory/export`

Full JSON export of all global memories.

| Method | Path | Description |
| :---: | :--- | :--- |
| `GET` | `/memory/export` | Export full memory pool |
| `GET` | `/memory/health` | Memory subsystem health |
| `GET` | `/memory/stats` | Entry/turbovec counts and retrieval latency |
| `POST` | `/memory/query` | Text search over the memory index (`{query, k}`) |

> **Corrected 2026-09-21.** `POST /memory/sync/{agent_role}` was documented here but **does not exist** on the native server (it returns `404`) — nor does the ChromaDB backend it implied. The real text-search entry point is `POST /memory/query`, which was missing from this table. Verified live: 5,420 memories / 1,398 turbovec entries.

```bash
curl -X POST http://localhost:8022/memory/query \
  -H "Content-Type: application/json" -d '{"query":"council status","k":3}'
```

---

## Error Responses

| Code | Meaning |
| :---: | :--- |
| `200` | Success |
| `400` | Bad request |
| `404` | Not found (unknown skill, artifact, or agent role) |
| `422` | Validation error |
| `500` | Internal error (LLM unavailable, I/O error) |

```json
{ "detail": "Skill 'unknown-api' not found in registry" }
```

---

## Swarm Federation (Phase 17)

All federation endpoints require an **HMAC-SHA256 signature** using the shared `SWARM_FEDERATION_SECRET`.

### `POST /federation/handshake`

Initialize a secure connection with a peer swarm.

```json
{
  "data": {
    "node_id": "swarm_001",
    "name": "Primary Swarm",
    "host": "127.0.0.1",
    "port": 8022,
    "api_key": "local_key_abc",
    "capabilities": ["memory_sync"],
    "timestamp": "2026-05-16T15:00:00"
  },
  "signature": "hmac_sha256_signature_here"
}
```

### `POST /federation/memory/sync`

Exchange knowledge with a verified federated peer.

```json
{
  "data": {
    "source_node_id": "swarm_001",
    "source_name": "Primary Swarm",
    "local_memory_count": 120,
    "timestamp": "2026-05-16T15:01:00"
  },
  "signature": "hmac_sha256_signature_here"
}
```

### `GET /federation/stats`

Get federation health and connectivity stats.

### `GET /federation/collective`

Get high-level status of the entire multi-swarm mesh.

---

### `GET /system/voltage`

Live substrate telemetry. `phi_v` is **derived from real host reads**, never a
constant: `(V_core/1.25)² × (f_actual/4000) × (V_ddr/1.35) × max(0.5, 1 − mem%/200)`.

The rails are **read**, not declared: `Win32_Processor.CurrentVoltage` (→ 1.10 V)
and `Win32_PhysicalMemory.ConfiguredVoltage` (→ 1.20 V). `measured: false` is
reported whenever any input comes from a fallback rather than a reading.

```json
{ "status": "MEASURED", "phi_v": 0.4152, "voltage_rails": {"v_core": 1.1, "v_ddr": 1.2},
  "measured": true, "source": "wmic CurrentClockSpeed + OS memory; ...",
  "gate": { "open": true, "threshold": 0.3, "measured": true,
            "policy": "deny-by-default: a gate whose substrate is unmeasured does not open",
            "reason": "phi_v 0.4152 at or above the floor 0.30; substrate has headroom" } }
```

### `GET /testing/stats`

Real test results, read per request from `evidence/platform_test_results.json`.

**Returns `status: NO_RUN_ON_RECORD` with null counts when no run exists** — it does
not invent a pass. Produce a run with `python tools/run_platform_tests.py`.

### `POST /ddr/scan`

Lexical threat scan. Body `{"code": "..."}`. Blanking comments and string contents
before matching is what stops a comment *mentioning* `eval()` from flagging.

```json
{ "status": "CLEAN", "matches": [] }
{ "status": "SUSPECT", "matches": [ { "error_type": "dangerous_execution_primitive",
    "severity": "high", "detail": "call to 'eval'", "fix": "..." } ] }
```

Measured **15/15** against standard evasions, **0/5** false positives. **Lexical, not
semantic** — it does not see through indirection. Against this repo's own 1,657 Python
files it flags **324 (19.6 %)**, which are genuine `subprocess`/SQL use: usable for
**review-highlighting, not as a CI gate**.

---

### System Status Disclaimer

Swarm OS v19.026 — 121 routes across the native host — see `trm_uefi_kernel/host_council_server/src/main.rs` for the authoritative list)
