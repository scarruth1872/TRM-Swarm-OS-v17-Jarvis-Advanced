# TRM Swarm OS — System Briefing, 2026-11-23 → 24

Written 2026-11-24 00:50 EST by Hermes. Every number below was read from the live system or from
git in this same session. Items I did not verify are labelled CLAIMED, not stated as fact.

## 1. Live state (verified just now)

- **Kernel** `v19.1.0-sovereign`, HEALTHY, `127.0.0.1:8022`, uptime 12,743 s → started ~21:05 on 11/23.
  `build_commit 0a30544+dirty`; `python_active: false`; `cpu_clock 3800 MHz`; `memory_percent 74.05`;
  `voltage_measured true`, `phi_v 0.4118`.
- **Ports answering 200:** 8022 kernel · 8587 vote model · 8593 embeddings · 8099 swarm.
- **Scheduled tasks:** `SwarmKernel` Ready · `TRM-Embedder-8593` Running · `TRM-VoteModel-8587` Running ·
  `TRM-Swarm-OS-Microkernel` Disabled.
- **Tests:** 128 pass, 0 fail (116 lib + 12 across other targets), built into `target/verify` so the
  running kernel's locked exe was not disturbed.
- **Repo:** HEAD `12333b4`. Kernel exe built 21:04 from `0a30544` — one commit behind HEAD, and newer
  than every source file at that moment, so it is a coherent binary.
- **Uncommitted:** only runtime state (`data/ddr_scan_state.json`, `data/kanban_board.json`,
  `evidence/BENCH_COMPLEX_TASKS.json`, federation registry) plus submodule pointers; 39 untracked.
- **Disk:** C: 84 GB free (**92 % used** — the only number on this page that is a warning) · F: 106 GB free.
  Commit charge 38.1 of 69.9 GB.

## 2. Today's commits — 30 total, two identities

16 by `TRM`, 14 by `NEXUS System`. The day was not one workstream.

Morning (council campaign, the thread I was on):
- `8c9a842` 00:23 — serialise votes; llama.cpp is not batch-invariant
- `dd7e423` 07:32 — record the documents a verdict actually read
- `8c92085` 08:14 — `:8587` vote model gets the `TRM-VoteModel-8587` task
- `43fba29` 08:20 — launcher alias table made true; `tools/*.bat` un-ignored

Midday (`TRM`): `G226`–`G228` — cron reports owned by the platform, guard WMI storm fixed, Jarvis reaches
kernel + live model, LCARS `:5183` supervised, ARC `:8588` retired.

Evening (`NEXUS System`, 19:38–21:10): federation peers reconciled · K3 bitboard arena + spectral
photoacoustic modules · RX 6700 XT Vulkan wgpu K3 compute shader + Cosmos 3 Edge bridge · software DDR
parity scrub + gate · 5-task Kanban plan (Journal 282) · 4 tasks completed (Journal 283) · live per-card
`/kanban/verify` instrumented · T2/T7 campaign findings · signal-vs-substrate erratum.

## 3. What was measured today (council reproducibility)

The open question entering the day: verdicts changed run to run at temperature 0.

- **Cause found:** llama.cpp is **not batch-invariant**. One vote prompt at temp 0 is byte-identical
  alone (5/5) and with 4 *identical* requests in flight (7/7), but with 3 *unrelated* prompts in flight
  the same prompt flipped from "none of the provided documents…" to citing a document and abstaining.
  The embedding path shows the same physics: same text bit-identical 5/5 alone, **9e-4 different batched
  vs alone**.
- **Fix:** votes hold a process-global mutex (`TRM_VOTE_CONCURRENCY=unbounded` opts out). Sequential is
  also faster than 4-way contention on a 2-slot server.
- **Result:** steady state is reproducible — **8 identical runs** of one topic (A6/R3/Ab5,
  `sig=..A.ARARA.AA.R`) across kernel restarts, a model-server restart, and 8099 up or down.
- **Hypotheses killed by experiment, not argument:** 8099 co-tenant traffic (stopped it → same steady
  state) and `:8587` prompt/KV cache history (relaunched the model server on the identical command
  line → same steady state).
- **Still open:** the FIRST deliberation after any restart differs every time (2508 A5, 2514 A7′,
  2517 A7″), and the steady state itself shifted A7→A6 across a binary generation. Both are recorded,
  neither is explained.

Audit-trail defect found and fixed in the same thread: `phoenix_evidence` was the caller's empty
parameter, so a verdict reported "no evidence" while agents quoted the documents they read. Now every
verdict names its sources, and `evidence_used` / `deliberation_mode` / `raw_topic` are published on the
single-record route.

## 4. Ledger state (verified)

1,522 council proposals: REJECTED 711 · NO_CONSENSUS 612 · CONSENSUS 164 · EVALUATING 27 · UNDECIDED 4 ·
ESCALATED 3 · **ENACTED 1**. Latest six are 2517–2522, produced after the vote fix.

## 5. Kanban board (verified live via /kanban/verify)

20 cards: **8 verified, 12 unverified**. The instrumented verifier is honest about this — the unverified
ones are mostly cards it identifies as "retired unbacked seed card" — but two deserve attention:

- `TASK-CAD-03` **"100-Node Federation Byzantine Health Audit" is marked DONE while its own evidence
  says `active_nodes: 1, configured_nodes: 3, offline_nodes: 2`.** A completion flag on a 100-node claim
  supported by one live node is the exact claim-vs-measurement gap this campaign exists to close.
- `TASK-CAD-01` (UEFI boot attestation) is verified with `efi_match: true`, `server_match: true`.

## 6. CLAIMED but NOT verified by me

Code for these landed tonight; I have not exercised any of their numbers:

- 584-agent K3 bitboard arena and spectral photoacoustic wave simulator (`agent_bitboard.rs` 284 lines).
- RX 6700 XT Vulkan wgpu K3 compute shader and "Cosmos 3 Edge bridge" (`cosmos_bridge.rs` 204 lines).
- Software DDR parity scrub + gate enforcement (`ddr_mitigation.rs` 142 lines).
- T1–T8 usecase campaign results (`evidence/USECASE_CAMPAIGN_T1_T8.json`).

Worth noting as a *good* signal: the T4/T3 erratum in `evidence/USECASE_CAMPAIGN_FINDINGS.md` retracts
two of its own claims (a scrub that only healed an in-memory page while reporting MITIGATING; a 9.63 ms
p99 traced to a synchronous disk write on the hot path). Self-correction is the behaviour we want — but
it also means these figures should be re-measured before they are quoted.

## 7. Open items, in priority order

1. **Warm-up outlier** — first deliberation after any restart is untrustworthy. Fix: one throwaway vote
   at kernel start. Cheap, and it removes the last source of "different answer from the same input".
2. **`TASK-CAD-03` overclaim** — a 100-node audit marked DONE on 1-of-3 evidence. Either the card is
   wrong or the evidence is stale; both are one-line answers and neither is currently stated.
3. **C: at 92 %** — 84 GB free. Not urgent, but it is the only capacity number moving the wrong way.
4. **Steady-state drift A7→A6** across binary generations, unexplained.
5. `start_model.bat` moved `1.5b` from 8587 to 8598; no caller of that combination was found, but none
   was proven absent either.
