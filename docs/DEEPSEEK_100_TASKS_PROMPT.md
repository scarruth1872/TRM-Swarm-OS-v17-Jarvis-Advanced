# Prompt for DeepSeek — generate 100 complex tasks for the TRM Swarm OS platform

Copy everything below the line into DeepSeek as a single message.

---

You are a senior systems engineer writing a work backlog for a real, running, single-machine sovereign AI platform.
Your output will be executed by an engineering agent that has shell, filesystem and browser access to the machine.
Produce **exactly 100 tasks**, ordered by dependency, in the schema given at the end. No preamble, no summary, no
commentary outside the JSON array.

## The platform, as measured (do not invent anything beyond this)

Single Windows 11 box, Ryzen 9 3900X (12C/24T), RX 6700 XT 12 GB (Vulkan only — no ROCm, no CUDA), 32 GB DDR4, NVMe.
Repo: `F:/Development sites/TRM-Swarm-OS-v2`, branch `sovereign-v19-clean`.

Running services (verified by HTTP body markers, not by port scans):

| service | port | what it is |
|---|---|---|
| Host Council Server | 8022 | **native Rust** microkernel, PBFT council, 14 agents, K3 ternary logic, Mamba-3 SSM readout |
| LCARS Dashboard | 5183 | Vite + React dev server, 25 tabs, talks to `:8022` through the Vite proxy (`API_BASE = ''`) |
| Jarvis | 3000 | TypeScript operator surface (`npx tsx server.ts`), proxies kernel routes, gates command execution |
| Companion Bridge | 3002 | native Rust, `trm_companion_bridge.exe` |
| Companion Proxy | 3001 | proxies to 3002 |
| Embeddings | 8593 | nomic-embed-text-v1.5 F16 768-dim served from a GGUF blob |
| llama.cpp | 8587 | local GGUF inference (Vulkan) |

**Hard constraints — a task that violates any of these is a defective task:**

1. **No cloud, no Docker, no Ollama runtime, no external APIs.** Everything runs on this box. `llama.cpp` binaries and
   local GGUF files are fine.
2. **No fabricated values, ever.** A field with no measurement must be `null` plus a named `unmeasured` list — never a
   plausible-looking nominal, never `0` standing in for "unknown", never a hardcoded `5/5`. This rule has already
   produced eleven separate defects in this codebase.
3. **Decisions must come from measurement, not labels.** A theorem is not "FORMALLY_PROVED" unless a prover ran; a
   service is not "ONLINE" unless an HTTP request returned its body marker.
4. **Verify by reading back, not by trusting the write.** Every task's acceptance criteria must name a command or
   request that returns the evidence, and the expected value.
5. **Real fixes only — no stubs, no mocks, no placeholders, no "TODO implement".** If a capability cannot be built on
   this hardware, the correct task is to make the UI say so, not to fake the feature.
6. **Never delete refuted work.** Supersede it with an erratum entry.
7. Anything touching a running `.exe` must stop it via `POST :8022/platform/shutdown {"confirm":"stop"}` — never
   `taskkill /IM`.

## The known defect list — your first sixteen tasks must cover these

**Status: all sixteen were measured, fixed and re-verified in the live UI (G157, commit 691181d).** Do not write them
as new fixes — write each as a **regression task**: a task that would fail today if the fix were reverted, with the
browser assertion that proves it (e.g. tab 06 must never render `0 GB` while `/system/resources` returns
`vram_total_gb: 12.0`, and must never render `NaN`). Each is a separate task; do not merge them.

| # | Tab / area | Defect (measured) |
|---|---|---|
| D1 | 06 MESH NET | shows `TOTAL NODES 14` and `ALIVE NODES 0`, `VRAM TOTAL 0 GB`, `VRAM USED 0 GB`, `AVG UTILIZATION 0%`, `HOT/COLD 0` — reads wrong keys from `/mesh/topology` and `/system/resources` |
| D2 | 08 FEDERATION | `ACTIVE PEERS 0` and `LOCAL NODE OFFLINE` while `/federation/stats` reports verified 1, active 1, configured 5 — the host serving the page reports itself offline |
| D3 | 10 RESEARCH | `ACTIVE TASKS 0 / COMPLETED 0 / TOPICS TRACKED 0 / DAEMON STOPPED` while `/research/tasks` returns a populated list |
| D4 | 11 VERIFICATION | `ITEMS IN QUEUE 0`, `VERIFIED LAST HOUR 0` beside a populated `/verification/queue`; and shows `Status: FORMALLY_PROVED` while `/verification/stats` says `theorem_status_source: "author-assigned label; NOT machine-checked here"` |
| D5 | 12 INFRASTRUCTURE | `SYSTEM HEALTH STATUS Unknown` while `/infra/health` returns `OPTIMAL` with 4 HEALTHY services; latency and restart cells render empty (`ms` with no number) |
| D6 | 13 TESTING | `SUCCESS RATE (24H) 0%` while `/testing/stats` returns `accuracy_percent 100.0`, `status ALL_PASSED`; the only run shown is stale (2026-10-05) |
| D7 | 15 DDR & VAULT | `ERRORS PREVENTED 279` while `/ddr/stats` reports `total_prevented: 0`, `prevented_corruptions: 0` — a fabricated number on a security panel (source was a `|| 279` literal fallback) |
| D8 | 18 EVOLUTION | `STABILITY INDEX NaN%` (same `undefined * 100` class of bug fixed elsewhere) and a permanent "No targets found. Scanning codebase..." |
| D9 | 16 AGENT COMMS | ultrawork missions render as bare `ID:` with no identifier |
| D10 | 17 SPATIAL MESH | header says `SWARM OS V14`; the platform is v19/v20 |
| D11 | 24 SWARM COUNCIL | every agent shows `0%` while quorum is 14/14 |
| D12 | 21 MOBILE SYNC | copy advertises transport `trm:// QUIC (:8021)`; the council is on `:8022` |
| D13 | 20 GENERATIVE ARCHITECT | `v18.224`, `Mode standalone`, `Verified 224`, `Commits 224` — no endpoint produces any of it |
| D14 | 22 CONTINUUM LEDGER / 23 NEURO ENGINE | `94.2% chromatic stability`, `81-88.7% quantum flux / phenotype integrity` — invented; no backend exists for any of it |
| D15 | 19 MONAD JOURNAL | reports `89 papers / 88 published` while the kernel journal holds 145 articles; and the pane renders all 145 articles at once (**114,137 px** of DOM) — needs virtualisation |
| D16 | cross-cutting | the served `jarvis_module_inventory.json` copy drifted from the scanner (showed `wired:4` against `9`) because nothing regenerates it on write |

## What "complex" means here

A task is complex when it requires reasoning across subsystems — not when it is long. Include tasks that require:

- **Wiring a real sensor to a real pane** and proving it with a live read-back (the honesty rule applied 25 times).
- **Making a fake panel honest**: either connect it to something measurable, or replace the number with `not measured`
  plus a visible reason. Both are acceptable outcomes; inventing a metric is not.
- **Kernel-side work in Rust** (`trm_uefi_kernel/host_council_server`): new endpoints, PBFT proposal flow, K3 readouts,
  Mamba-3 SSM state pooling — with the constraint that a fitted model may not be deployed unless its holdout supports
  the exact claim the UI makes (this rule already caught a gate that passed a verb holdout while its object holdout
  was at chance).
- **Performance work with a measured budget**: the journal pane is 114k px; the council serves 373 KB of JSON on
  `/council/status`; VRAM is 12 GB and shared.
- **Cross-service verification**: dashboard → Vite proxy → Rust kernel → back, proven end-to-end with the failing case
  as well as the passing one (e.g. a BLOCK must actually block, an ABSTAIN must carry reason + candidates + next step).
- **Regression protection**: a task must exist for making `tools/build_portal_articles.py --check` run as a pre-commit
  or CI gate so published content cannot drift from the journal again.

## Required output format

A single JSON array, 100 objects, no other text:

```json
[
  {
    "id": "T001",
    "title": "<= 80 chars, imperative, specific>",
    "area": "one of: kernel-rust | dashboard-ui | jarvis | companion-bridge | data-pipeline | tooling | verification",
    "defect": "D1..D16 or 'none' — the measured defect this fixes, if any",
    "why_complex": "2-4 sentences naming the subsystems involved and the hard part",
    "steps": ["3-8 concrete steps, each naming a file, endpoint or command"],
    "files": ["exact paths under F:/Development sites/TRM-Swarm-OS-v2"],
    "acceptance": [
      {"check": "the exact command / HTTP request / browser assertion", "expected": "the exact value or state"}
    ],
    "failure_case": "the negative test that must also pass — what the fix must NOT do",
    "honesty_note": "which value is measured and what happens when it is absent (null + named unmeasured entry)",
    "depends_on": ["task ids, or []"],
    "risk": "low | medium | high + one sentence on blast radius"
  }
]
```

Hard requirements on the output:

- All 100 ids `T001`–`T100`, ascending, no gaps, no duplicates.
- `depends_on` must only reference lower-numbered ids.
- Every task must name at least one exact file path and at least one machine-checkable `acceptance` entry.
- At least 15 tasks must be kernel/Rust, at least 20 dashboard, at least 10 verification/tooling.
- No task may require network access, Docker, Ollama, or a cloud API.
- No task may be satisfied by adding a hardcoded value. If you find yourself writing a number that no command in
  `acceptance` can reproduce, rewrite the task.
- If a capability genuinely cannot exist on this hardware (e.g. a sensor the board does not have), the task must be
  "make the interface state that it is unmeasured, and name what would be required" — that is a valid, valuable task.

Before you emit, self-check: for each task ask *"could a dishonest agent complete this by printing a plausible number
and claiming success?"* If yes, strengthen `acceptance` until the answer is no.
