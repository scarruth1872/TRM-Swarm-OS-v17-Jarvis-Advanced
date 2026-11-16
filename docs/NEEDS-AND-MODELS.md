# What This Platform Needs To Train — derived from measurement, not ambition

Every line below starts from a number that was measured on this host. The rule used throughout: **train only what a
measurement says is broken and that constrained decoding cannot fix**, because training is the most expensive thing
we can do on a 12 GB single-GPU box with no cloud.

Sources: G114–G120 in `docs/PHASE-6-GAPS.md`, `docs/TASK-YIELD-MATRIX.md`, `scratch/router/ledger.json`.

---

## The four measured deficits, ranked by cost to the platform

| # | deficit | measured | fixable by training? |
|---|---|---|---|
| 1 | **Structure** — replies that are not JSON (or decay mid-token) | 2B 12.8 % fail, **109 of 128 are unclosed JSON**; 1.5B 6.7 % | **No — grammar fixes it for free** |
| 2 | **Grounding** — JSON that cites nothing real | strict: 1.5B **57.8 %**, 2B **42.8 %**, 27B **98 %** | **Yes — this is the real training target** |
| 3 | **Routing** — knowing which tier a task needs | router AUC **0.580**, ternary heads **0.493**; need ≥0.85 to gate | **Yes — tiny model, cheap** |
| 4 | **Thinking tail** — Instella spends its budget reasoning instead of answering | 500 tok → 16 % valid; runaway tasks ~105 s | **Yes — SFT to answer directly** |

The important split: **#1 is not a training problem.** A JSON-schema/GBNF grammar constrains sampling so an unclosed
object cannot be emitted. We are currently paying a 12.8 % failure rate for something `llama-server` supports as a
request parameter. That must be tried *before* any fine-tune is proposed — and it is free.

---

## Stage 0 — no training: grammar-constrained decoding  (do this first)

`llama-server` accepts `response_format: {"type": "json_schema", ...}` or a raw `grammar`. **Verified from the
binaries**, not assumed: both serving builds (`llama-vulkan-b12360` and the Instella fork) expose `--grammar`,
`--grammar-file`, `-j/--json-schema` and `-jf/--json-schema-file`. The five task shapes already
have exact schemas in `tools/loadtest_tasks.py`. Expected effect, stated before the run so it can fail:

* 2B schema 87.2 % → **100 %** (by construction, not by learning)
* unclosed-JSON failures (109/128) → **0**
* 1.5B schema 93.3 % → **100 %**

If that holds, ~13 % of the 2B's failures and ~7 % of the 1.5B's disappear with zero training cost. **Falsifier:** if
grammar-constrained output still fails the strict grounding check at the same rate, the deficit was content, not
structure — which strengthens Stage 2.

---

## Stage 1 — the router head: the smallest useful model on this box

**Need:** the platform cannot currently decide which tier a task belongs on. That is the whole point of a sparse
gate, and the measured state is that the 15 existing ternary heads (AUC 0.493) and a logistic router (AUC 0.580) are
both far too weak. Target: **AUC ≥ 0.80 on held-out files**, else do not deploy.

| property | value |
|---|---|
| architecture | the existing ternary head format (522,027 params) so it loads in the same SRAM slot array as the 15 agent heads |
| features | 36 text/shape features (already built) **+ excerpt embeddings** from `:8593` — the current router has no semantic feature, which is the most likely cause of 0.580 |
| labels | free: every `tools/loadtest_tasks.py` run emits `schema_ok` + strict grounding. 1,000/run, file-level split already enforced |
| training cost | minutes on CPU; inference 0.7 ms/call, ~0.4 % of model time |
| deploy rule | held-out AUC ≥ 0.80 **and** on a 200-task replay, routed-down tasks keep ≥95 % of their measured quality |

This is also the platform's stated aim (Mamba-3/K3 ternary logic driving council decisions) in its cheapest possible
form: one head, measured, in SRAM, deciding a real dispatch.

---

## Stage 2 — the distilled grounding student: the highest-value training on this box

**Need:** strict grounding is the axis where the fast tier is unusable. 57.8 % (1.5B) and 42.8 % (2B) mean roughly
every second answer cites something that does not exist in the file. The 27B scores **98 %** on the same tasks — so
the platform already owns a teacher that solves this, locally, with zero cloud.

| role | model | measured basis |
|---|---|---|
| teacher | **Bonsai 2 27B PTQ1_0** (`:8612`) | 100/100 schema, 98 % strict, p50 67 s, 0.113 items/s |
| student | **Qwen2.5-1.5B-Instruct Q4_K_M** (`:8587`) | 93.3 % schema, 57.8 % strict, 1.442 items/s, p50 2.38 s |
| judge | the same strict checker | identifier ≥3 chars, must appear verbatim in the excerpt |

**Dataset:** N tasks over real repo files, answered by the teacher, **filtered by the strict checker** — only
schema-valid + strict-grounded teacher answers are kept. That filter is what makes this distillation honest: the
student is trained on verified-correct reasoning, not on teacher noise.

**Budget, from measured throughput** (teacher 0.113 items/s single arm, offline queue so it does not disturb the
fleet):

| labels | teacher wall-clock |
|---|---|
| 2,000 | ~4.9 h (one overnight) |
| 5,000 | ~12.3 h (two nights) |

**Training:** LoRA (r=16, α=32) on attention + MLP projections, Xavier init, LR 3e-5, clip 0.5 (the values already
pinned in memory), seq 2,048, 2 epochs. **On CPU, not GPU** — PyTorch has no Vulkan backend and ROCm is not available
for the RX 6700 XT on Windows, so GPU training would need Docker, which this host forbids. ~1.5–2 h for 2,000
examples on 24 threads.

**Acceptance gate, stated before the run:** strict grounding ≥ **85 %** on the same 1,000 held-out tasks (from
57.8 %), **with schema not below 93.3 %**. If it lands under 85 %, the student is not deployed and the result is
written up as a refuted attempt — the platform's errata rule.

**Do not attempt to fine-tune Bonsai 2 itself.** Its weights are private ternary types 42/143 (402 of 851 tensors),
no dequantiser exists, so there is no correct path from a loss gradient back into those blocks. A "fine-tune" there
would silently corrupt the file. Distil *from* it instead.

---

## Stage 3 — Instella: train out the thinking tail  (only if Stage 0/2 leave it as the best tier)

**Need:** this fork ignores `enable_thinking`, `-rea off` and `--reasoning-budget 0`, so reasoning competes with the
answer for tokens: 500-token budget → 16 % valid, 2,000-token budget → a bimodal 4–5 s / ~105 s. It is fast (90–100
tok/s) with an unbounded tail.

**Fix by SFT, not by serving flags:** collect the ~5 % of tasks where it answers directly and correctly in under 1 s
(measured: a warm trivial task in **0.84 s**), plus teacher-style direct answers, and train a small LoRA that answers
without a think block. Acceptance: ≥40 % valid at a 500-token budget (from 16 %) with p95 latency ≤ 20 s.

**Gate:** if Stage 0 gives structure for free and Stage 2 lifts the 1.5B to ≥85 % strict, Instella stops being
necessary — 2.8 s/task single-slot versus the 1.5B's 2.38 s means the fast tier wins on every axis that matters. In
that case do **not** spend training budget here; record the decision and the reason.

---

## Explicitly NOT worth training here

| candidate | why not |
|---|---|
| Bonsai 2 27B / 2B themselves | private ternary tensor types, no gradient path to those blocks (see G117) |
| Anything requiring Docker/cloud | user rule: zero Docker, zero cloud, native kernel only |
| A larger router than ~1M params | inference must stay ~0.7 ms/call to be a free gate; a 100 M-param router costs more than it saves |
| TurboVec as a "training" step | it is a post-hoc quantisation; on Bonsai 2 it *adds* loss and grows the file 13 % (G117) |

---

## Order of work, and what each step is worth if it works

1. **Grammar on the two GGUF tiers** — free, ~13 %/7 % of schema failures gone. Verifiable in an hour.
2. **Router with embeddings** — minutes of training; unlocks sparse gating (the platform's stated goal) or refutes it.
3. **Distil the 1.5B from the 27B** — one to two overnight label runs; the only change that moves strict grounding,
   which is the axis that decides whether this platform can be trusted on bare metal.
4. **Instella SFT** — only if 1–3 do not make it redundant.
