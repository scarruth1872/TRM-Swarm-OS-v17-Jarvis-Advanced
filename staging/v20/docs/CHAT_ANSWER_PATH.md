# Neural Bridge — chat answer path

How a message typed into **tab 02 (NEURAL BRIDGE)** is answered, what each tier may claim, and
how to tell a real answer from a stalled one.

## The path

```
UI (App.jsx, tab 02)          API_BASE = ''  →  relative URL, same origin
  POST /swarm/chat        →   Vite dev server :5183 (proxy)
                          →   Rust microkernel :8022  /swarm/chat
```

`/swarm/chat` is served by the kernel (`host_council_server/src/chat_handlers.rs`). The dashboard
never talks to a Python backend: **every** proxy namespace in `dashboard/vite.config.js` targets
`:8022`.

## Tiers, in the order they are tried

| # | tier | what it may claim | engine string |
|---|------|-------------------|---------------|
| 1 | deterministic (greetings, math) | canned text, no model | `conversation` |
| 2 | **fabric** — the kernel's own index | only what a sealed document confirms | `fabric:k3-index` |
| 3 | Instella MoE routing → Bonsai Vulkan `:8587` | nothing while that server is offline | `bonsai:vulkan-gpu` |
| 4 | *(removed)* Ollama | — | — |

When no tier can serve, the reply says so explicitly (`[NO_INTELLIGENCE_TIER_AVAILABLE]`).
That is a correct answer, not a failure: the doctrine is that the platform answers from its own
fabric, or it admits the gap.

## The fabric tier's rule

The ternary score only **proposes** candidates — measured over this corpus, known queries and
nonsense overlap (137..171 vs 134..144), so a score threshold would be an invented criterion.
A candidate is **confirmed** only when the sealed document's own text shares **≥ 2** of the
message's content words. Among confirmed candidates the route prefers the document whose *path*
names the query's terms, because the native encoder's top five span only a few points.

## Invariants (breaking any of these breaks the chat)

1. **The query encoder must be the encoder that built the index.** The index is built with
   `native-turbovec-hdc` (`rust_native_encode_text_f32`). Encoding queries with anything else
   puts query and corpus in different spaces: measured, an Ollama-embedded query scored 76..81
   against this native index and returned CSS files, while the native encoder returned the
   topically correct papers.
2. **No proxy namespace may point at `:8021`.** A dead upstream makes the proxy answer 500;
   axios rejects on non-2xx; `App.jsx`'s catch appends `⚠️ [NEURAL_LINK_STALLED]`.
3. **The corpus cache is per-process.** The kernel resolves engine + corpus once, so replacing
   `swarm_v2_artifacts/kernel_knowledge_index.json` requires a kernel restart to take effect.

## Measured behaviour (2026-09-20, 180-document corpus)

| measurement | value |
|---|---|
| platform questions answered from the fabric | 10/12, citing `papers/…` |
| chat wall time, answered | p50 3.9–14.4 ms (no model call) |
| chat wall time, abstention | 0.82 s (offline Vulkan probe, uncached) |
| `/k3/retrieve` engine work | p50 ~110 µs, all scores reproduced by the agreement export |

## Verifying, end to end

```bash
curl -s -X POST http://127.0.0.1:5183/swarm/chat -H 'Content-Type: application/json' \
  -d '{"role":"Archi","message":"explain the K3 ternary operating system","sender":"user"}'
# 200, engine=fabric:k3-index, cites papers/paper_02_k3_ternary_os.md, single-digit ms
```

| symptom | cause to check first |
|---|---|
| `⚠️ [NEURAL_LINK_STALLED]` | proxy target for that namespace (must be `:8022`), kernel listening |
| answers cite CSS / `package.json` | query encoder ≠ index encoder |
| `[NO_INTELLIGENCE_TIER_AVAILABLE]` | no confirming document in the corpus — ingest the knowledge, don't lower the bar |
| cited document is off-topic | index coverage/discrimination, not the tier logic |

## Dead dependencies must fail fast

The fall-through path consults the model tiers only when the fabric has no confirming document —
and that probe must not cost the query. `ollama_available()` read `:11434` with a **3,000 ms**
timeout on every unanswered question, so a daemon that was *listening but unusable* stalled the
path. A fault injector on `:11434` (accepts TCP, never replies) proved it against the deployed
build:

| daemon state on `:11434` | before | after (400 ms timeout + 30 s probe cache) |
|---|---|---|
| stopped | 0.81 s | 0.41 s |
| listening, never replies (hung) | **3.42 s** | **0.83 s** first, **0.41 s** cached |
| listening, no usable model (live daemon, measured earlier) | 8.88 s | ~0.41 s |

`[NO_INTELLIGENCE_TIER_AVAILABLE]` is still the honest answer; it is just no longer expensive. The
fix lives in `intelligence_router.rs` **and** its generator `swarm_v2_artifacts/gen_intelligence_router.py`,
so regenerating the router cannot silently undo it.

Ollama is not part of the runtime: it was stopped (three processes) and `Ollama.lnk` was moved out of
the Startup folder to `~/Downloads/startup_disabled_20260920/` so it does not return at login.

