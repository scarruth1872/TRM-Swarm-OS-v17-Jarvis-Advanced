# Resident MoE models: what is on the box, and what they actually know

**2026-11-24.** 37 GGUF files on disk. Exactly **two** are real mixture-of-experts models; everything
else is dense. Both were tested for domain expertise with checkable answers, and both were asked what
they think they are good at. Neither knows this platform — and the retrieval index does not rescue it.

## 1. Inventory (parser: `expert_count` / `expert_used_count` from GGUF metadata)

| model | file | arch | experts | fit on 12 GB | status |
|---|---|---|---|---|---|
| **gpt-oss-20b** | `gpt-oss-20b-MXFP4.gguf` 12.11 GB | gpt-oss | **32 / top-4** | no (12.11 > 12) | resident `:8587`, council judge |
| **Instella-MoE-think** | `instella-moe-think-Q4_K_M.gguf` 10.47 GB | deepseek2 | **64 / top-6** | marginal | loads fine on `:8596`, not resident |
| Bonsai-27B Q1 | 3.80 GB | qwen35 | dense | yes | idle |
| Ternary-Bonsai 8B / 4B / 1.7B Q2 | 2.18 / 1.07 / 0.46 GB | qwen3 | dense | yes | idle |
| qwen3-4b, llama-3-8b, qwen2.5-1.5b, jarvis LoRA | 0.02–4.66 GB | qwen3/llama/qwen2 | dense | yes | idle |

Trap: `ggml-vocab-gemma-4.gguf` (128/8) and `ggml-vocab-nomic-bert-moe.gguf` (8/2) report MoE
architectures but are **tokenizer stubs**, not models. A grep for "expert" would count them.

There is **no observable per-expert routing** in llama.cpp: it exposes no per-token gate activations, so
"which expert is the security expert" cannot be answered from outside the model. Only behavioural domain
competence is measurable — that is what follows.

## 2. Domain probe (verifiable keys, temp 0, one sample each)

| domain | gpt-oss-20b (32/top-4) | Instella (64/top-6) |
|---|---|---|
| security — does disabling TLS verification allow MITM? | **PASS** | PASS |
| build — what `?` does in Rust | **PASS** | PASS |
| ledger — why a hash chain is tamper-evident | **PASS** | PASS |
| inference — per-sequence tok/s vs batch size | **PASS** | miss (answer buried in thinking) |
| network — HTTPS default port | **PASS** (443) | PASS |
| retrieval — does cosine similarity care about magnitude | **PASS** | PASS |
| platform — which port does the council listen on | **miss** — answered **443** "typically" | miss |
| platform — what is axiom A2 about | **miss** — invented a set-theoretic relation | miss |
| **total** | **6/8** | 5/8 |

    general-domain answers: 11/12 passed across both models
    platform-internal answers: 0/4 passed across both models

Both confabulate on platform facts **in the same voice they use when right** — gpt-oss stated "the TRM
Council server typically listens on TCP port 443 (HTTPS)" with no hedge that a reader would notice.
That is the dangerous failure class for a voting agent.

## 3. Self-report vs measured (the "ask it what it knows" half)

- **gpt-oss-20b** claims strongest: NLP, then CS theory; weakest: experimental physics, clinical medicine.
  Its measured strengths (Rust semantics, hash chains, bandwidth tradeoffs) are consistent with the
  CS-theory claim, but the self-report is **not evidence** — it produced the same confident tone on the
  port question it got wrong.
- **Instella** never reached its self-report: the entire budget went into a ` thinking` block.

## 4. Instella is not usable as a voter without template work

`enable_thinking:false`, `thinking:false`, plain prompt, and `/no_think` system message **all** still
emit ` thinking` and consume the full 180-token budget before answering:

    enable_thinking:false  7.0s 180 tok  -> ' thinking\nOkay, the user is asking which default port...'
    thinking:false         6.5s 180 tok  -> ' thinking\nOkay, the user is asking...'
    /no_think sysmsg       7.2s 180 tok  -> ' thinking\nOkay, the user is asking...'

Its template must be overridden (`--chat-template`) before it can hold a seat. Decode is 25.6 tok/s
(much of it thinking tokens) vs 13.3 tok/s for gpt-oss-20b in the same test.

## 5. Retrieval does not cover the gap either — measured

The council's index (`data/council_evidence_index.json`, **558 chunks from 105 files**, 768-dim nomic
vectors) was queried directly for the same four questions:

| question | top-1 cosine | top-1 source | key content in top-3? |
|---|---|---|---|
| council listen port | 0.650 | `COUNCIL_DELIBERATION_CRATE.txt` | only incidental "8022" |
| what is axiom A2 | 0.612 | `article-191-council-falsification.txt` | only incidental "A2" |
| TLS verification / MITM | 0.623 | `article-192-three-way-hmac.txt` | only incidental "tls" |
| hash chain tamper-evident | 0.653 | `_kernelmsg.txt` | **NONE** |

Every top-1 is semantically unrelated to the question. So the council is being handed weakly-related
chunks and asked to cite an axiom — which is consistent with the generic verdicts measured earlier.
The index is built from narrative docs/evidence dumps; the canonical sources of truth (port constants,
`sovereign_axioms.rs`, route table) are **not in it**.

## 6. What to do, in dependency order, with gates

1. **Fix retrieval before teaching anything.** Index the canonical facts — ports from the launcher/task
   config, axioms from `sovereign_axioms.rs::describe()`, routes from the dispatch table, plus the doc
   corpus. Gate: the four questions above must return a top-1 chunk **containing the key** (4/4), with
   cosine clearly above the current 0.60–0.65 noise band.
2. **Distill the general-domain teacher into the heads, ternary-aware.** Use gpt-oss-20b (6/6 general) as
   teacher over the six domains it demonstrably holds, generating (text → trit, axiom, one-line why);
   train the 14 heads with weights constrained to {−1,0,1} during training — post-hoc quantisation is
   refuted (36–45 % agreement). Gates: beat the current 14/14-ANALYZE baseline on the dev set; never
   approve the TLS-verification/sandbox class; teacher agreement on a held-out general set reported
   honestly, not tuned.
3. **Do not seat Instella** until its thinking block is stripped by an explicit chat template and its
   parse rate is measured (the 4B precedent: 0/14 parsed at 18–21× speed).
4. **Never let an LLM state platform facts unqualified.** Both models invent them confidently. Platform
   claims must come from the ledger, the route table, or a retrieved document with a path — the same rule
   already applied to vote evidence.

Scripts: `scratch/moe_domain_probe.py` (probe), `scratch/gguf_inventory.json` (metadata inventory).
