# Jarvis ↔ kernel wiring, and why the model port was dead

**2026-11-23.** Jarvis was answering "I encountered an issue while processing that request: fetch
failed" while the kernel, the model and the proxies were all healthy. Three separate constants were
pointing at ports that nothing serves.

## What was wrong

| where | old default | what actually listens |
|---|---|---|
| `server.ts` — `/api/status` upstream probe | **8021** (retired Python proxy) | kernel on **8022** |
| `server.ts` — chat model URL | **8597** | model on **8587** (veto) or **8596** (analysis) |
| `jarvis_harness.ts` — `CHAT_URL` | **8597** | same as above |
| `jarvis_harness.ts` — `CODE_URL` | **8595** (qwen3-4b slot) | only when started on demand |
| `vite.config.ts` — dev `/api` proxy | **8021** | Jarvis's own server on **3000** |

Measured evidence, not inference: `logs/jarvis_3000.log` held 16 × `ECONNREFUSED 127.0.0.1:8597`
while `:8587` answered `/health` 200 the whole time, and `/api/status` reported the platform DOWN while
`curl :8022/health` returned `status: HEALTHY`.

The training data has the same defect in prose — `jarvis_comprehensive_chat.json` teaches "TRM runs on
port 8021" in dozens of rows. **Those rows are now wrong and are deliberately not rewritten**: they are
a record of what the model was taught, and rewriting a dataset to match today's port is how a
plausible falsehood gets laundered into training. If the adapter is ever retrained, the port rows must
be regenerated from the kernel's own `/health`, not from the old prose.

## What changed

- **`model_targets.ts` (new)** — resolves the live model at call time: `JARVIS_MODEL_URL` wins if set,
  otherwise it probes `JARVIS_MODEL_PORTS` (default `8596,8597,8587`) for a 200 on `/health`, caches the
  answer, and re-probes on failure so a tier swap recovers without restarting Jarvis.
  `resolveCodeUrl()` falls back to the chat model instead of a dead slot.
- **`server.ts`** — imports that resolver (its local copy was removed); upstream probe default 8022.
- **`jarvis_harness.ts`** — all four call sites resolve per call; no hardcoded ports remain in code.
- **`vite.config.ts`** — dev proxy target 3000, because `server.ts` registers the `/api` routes itself.
- **`tools/start_jarvis_3000.bat` (new)** — the only supported way to start Jarvis. A process filter
  matching `server.ts` also matches the shell whose own command line contains that string, so
  pkill-by-commandline from an inline command kills the thing running it (learned the hard way here:
  the kill loop took out its own bash/powershell parents).
- **`tools/swarm_kernel_guard.ps1`** — now supervises `:3000` as a sixth listener. Jarvis had no
  supervisor at all, which is why it stayed dead until someone noticed.

## Verified

- `/api/status` → `dependencies: [{target: "127.0.0.1:8022", state: "up", latencyMs: 2}]`
- `POST /api/chat` → a real model answer, grounded in a live kernel snapshot (`[Harness] grounding:
  LIVE PLATFORM STATE … status HEALTHY, kernel: Native Rust Sovereign Kernel v19.1.0-sovereign`)
- Guard healed Jarvis end-to-end: server killed (pid 39796) → next guard tick printed
  `JARVIS STARTED (:3000)` → `/api/status` answered from pid 123616
- Guard healthy path **184 ms** for six listeners; kernel uptime continuous at 253.5 min, so nothing in
  this loop is restarting the kernel

## Addendum — the "native bridge" was two bugs and a myth; the last dead listener fixed

**1. `require is not defined` was hiding a dead protocol.** `server.ts` called
`require("dgram")` / `require("crypto")` inside an ES module. That throws immediately, the throw was
caught, and the log blamed a missing bridge. Importing them properly would not have helped: measured
2026-11-23, a UDP datagram to `127.0.0.1:8022` returns **WSAECONNRESET** — nothing listens on UDP there.
The kernel is **TCP only** (`/health`, `/system/*`, `/council/*`, `/k3/*`), and the harness already
grounds every prompt in those routes, so the old 3000 ms wait bought latency and no answer.

Now: the bridge lives in `udpBridgeProbe()` behind `JARVIS_NATIVE_UDP=1` (off by default, 800 ms, and a
real `error` handler so a reset rejects instead of hanging). The one remaining `require` — `ioredis`,
lazily, only when `REDIS_URL` is set — goes through `createRequire(import.meta.url)`.

Side effect: the six long-standing `Property 'ok'/'reply' does not exist on type 'unknown'` errors in
`server.ts` are gone, because the probe is now typed. `tsc --noEmit` is **0 errors**.

**2. The reply was clean, the framing was not.** The resident model decides what comes back: Instella
returns plain text, while the gpt-oss gate model on `:8587` emits Harmony channels, so a reply arrived as
`We have to answer: … <|end|><|start|>assistant<|channel|>final<|message|>The kernel listens on port 8022`.
Both call sites now keep the final channel and strip `<|…|>` control tokens — the same class of cleanup as
the existing `</think>` strip.

**3. Grounding now carries what people actually ask about.** The model had answered "the listening port is
not measured" because the grounding omitted it. It now includes `tcp_port: 8022`, `build`, and
`python_active`. Measured answer afterwards: **"Boss, the kernel listens on TCP port 8022."**

**4. `:5183` LCARS dashboard — started and supervised.** `dashboard/` is a vite dev server
(`npm run dev`, port 5183) and nothing ever started it, which is why the kernel's health matrix reported
it as its only anomaly. `tools/start_dashboard_5183.bat` starts it; the guard now supervises it as a
seventh listener. Result:

```
/system/health_matrix → status: OPTIMAL, anomaly_count: 0
services: council_server ONLINE, companion_proxy ONLINE, companion_bridge ONLINE,
          lcars_dashboard ONLINE, native_cognitive_engine ONLINE
```

**5. `:8588` is not a gap — it is a retired port.** `/arc`, `/arc/stats` and `/arc/manifest` all return
200 **on the kernel at 8022**. Nothing serves 8588 and nothing needs to; reporting it as "down" was
measuring a port the platform moved off. Same pattern as 8021 → 8022.

Guard after all of this: `ALL LISTENING (8022,3000,3001,3002,5183 + tailnet 8022,3001)` in **183 ms**.


## Known compromises

- The only model resident right now is the **gpt-oss gate model on :8587**, which expects Harmony
  formatting while both Jarvis paths build ChatML. It answers and it grounds on the kernel snapshot, and
  its Harmony framing is now stripped rather than shown — but a ChatML prompt to a Harmony model still
  costs answer quality. **Instella on :8596 is the correct chat model** for this prompt shape; start it
  with `tools/model_tiers.ps1 analysis` (the two cannot co-reside on 12 GB, and swapping it in evicts the
  veto model, so it is a deliberate choice, not a free improvement).
- Both of the items that used to sit here are now fixed and are described in the addendum above:
  `require is not defined` (ESM/CJS on the bridge path) and the grounding gap that made the model answer
  "the listening port is not measured". They are kept in the record rather than deleted, because a
  compromise list that quietly shrinks teaches nothing about what the defects looked like.
