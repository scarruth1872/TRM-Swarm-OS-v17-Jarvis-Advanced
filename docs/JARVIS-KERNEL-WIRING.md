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

## Known compromises

- The only model resident right now is the **gpt-oss gate model on :8587**, which expects Harmony
  formatting while both Jarvis paths build ChatML. It answers — it even grounds on the kernel snapshot —
  but it leaks its reasoning into the reply ("We need to answer: …"). **Instella on :8596 is the correct
  chat model** for this prompt shape; start it with `tools/model_tiers.ps1 analysis` (the two cannot
  co-reside on 12 GB).
- `[Jarvis] Native bridge unavailable — legacy fallback: require is not defined` still appears in the
  log: an ESM/CJS mismatch on the native-bridge path, unrelated to the port fix and not yet addressed.
- The grounding payload lists kernel status but not its port, so the model answered "the listening port
  is not measured" when asked directly. That is the model being honest about its context; the fix is to
  add the port to the grounding, not to soften the instruction.
