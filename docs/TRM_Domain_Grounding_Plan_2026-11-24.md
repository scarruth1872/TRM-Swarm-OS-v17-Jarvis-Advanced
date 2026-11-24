# Plan: domain-grounded council, built on the modules that already exist

**2026-11-24.** Before writing any new code, this inventories what the crates and tools already do.
Most of the pipeline the last four investigations called for is **already implemented**; the plan reuses
it and adds three small pieces. Every phase carries a gate that is measured, not asserted.

## What already exists (reuse, do not rebuild)

| need | existing asset | state |
|---|---|---|
| semantic retrieval in the decision path | `host_council_server/src/evidence_index.rs` — query embedded on `:8593`, cosine over stored 768-dim vectors, floor 0.30 | works; the **content** of the index is the gap |
| index building with dedupe + manifest | `tools/build_fabric_index.py` — 122,918 → 42,801 distinct, float16 L2 vectors, sha256 manifest | works; never used for the council index |
| honest retrieval measurement | `tools/measure_fabric_retrieval.py` — rewrites memories as questions, excludes probes >0.45 Jaccard so keyword matching can't win | exists; reuse as the Phase 0 gate |
| memory id index | `tools/build_memory_index.py` + kernel `turbovec_memory.rs` (1,100 lines) | exists |
| 2-bit packed heads | `host_council_server/src/k3_ternary.rs` (`TriVec`, 4 trits/byte, `TriHead::from_head`, `pack_report`) + `GET /k3/ternary/bench` | exists; **measured below** |
| teacher label harvesting | `tools/distill_labels.py` — strict checker filter, FAILURES/PASSES pools, append-only JSONL, `--resume`, live ETA | exists, exactly the shape needed |
| head retraining | `tools/retrain_k3_v4.py` — LR 3e-5, clip 0.5, Xavier, **QAT 0.7**, 20 epochs | exists, already quantisation-aware |
| blob export | `tools/export_k3_heads_v3.py` — K3HD v3 with `feat_version` per head | exists |
| canonical axiom text | `sovereign_axioms.rs::describe()` → `GET /council/axioms` | exists |
| self-knowledge of modules | kernel `module_registry_generated.rs` (322 modules: REAL 274 / PORTED 39) built by `build_module_registry.py` | exists, regenerated from the live scan |
| route table | `main.rs` dispatch chain, ~159 route literals | **not machine-readable — the one real gap** |

## The in-tree 2-bit numbers (better than my numpy study, and still a refusal)

`GET /k3/ternary/bench` on the live kernel, 15 heads:

    compression            13.63x   (29.87 MB f32 -> 2.19 MB packed)
    parity_agree           35/60 = 58.3 %   (dense vs packed trits)
    latency_f32            44.8 us per head
    latency_ternary       404.8 us per head      -> packed is 9.0x SLOWER
    emb_relative_rms_error 0.51   (per-row scaled embedding still loses half its signal)
    litmus_sink            -38

So packing buys storage and nothing else: it disagrees with the dense head on 42 % of texts **and** runs
9× slower (trit unpacking with per-trit branches, no SIMD). My numpy threshold study (36–45 % agreement)
and the in-tree 1.58 quantiser (58.3 %) agree on the conclusion: **do not pursue packed heads for speed
or for parity.** Keep f32 in the decision path; if storage ever matters, the 13.6× is available at a
known accuracy cost.

## Phase 0 — make the platform's own facts retrievable (smallest change, biggest lever)

Measured problem: the council index is 558 chunks from docs/evidence text, and every top-1 for a platform
question is semantically unrelated (0.60–0.65 cosine). The canonical sources exist but are not indexed.

New code, one file plus one route:
1. `GET /platform/facts` in `main.rs` — emits facts **from code**, so they cannot drift:
   - ports/services from the launcher constants and the task names,
   - axioms from `sovereign_axioms::describe()`,
   - module totals from `module_registry_generated.rs` (`TOTAL/REAL/PORTED`),
   - one line per route, harvested from the same dispatch match by a tiny generator (or a hand-kept
     table asserted against the match count so a new route fails a test).
2. `tools/build_council_index.py` — reuse `build_fabric_index.py`'s dedupe + float16 + manifest pattern;
   inputs: `/platform/facts` output, `docs/`, `evidence/`, and `GET /council/axioms`. Writes
   `data/council_evidence_index_v2.json` beside the existing file (the old one is never overwritten).

Gate (pre-registered, must pass before Phase 1): the four probe questions from the MoE study
(council port, axiom A2, TLS/MITM, hash-chain tamper-evidence) each return a **top-1 chunk containing
the key string**, with cosine clearly above today's 0.60–0.65 band. Probes adapted from
`measure_fabric_retrieval.py` (paraphrased, low Jaccard).

## Phase 1 — point the council at the v2 index

`evidence_index.rs` gains a path preference (`council_evidence_index_v2.json` if present, else the
current file) and the floor is **re-derived from the new score distribution**, not carried over from 0.30.

Gate: on the 12 dev tasks, every `phoenix_evidence` entry must contain the text its verdict cites
(provenance check, machine-verifiable), and the verdict signature must stay inside the known steady
state or an errata is recorded.

## Phase 2 — harvest domain labels with the existing distiller

`tools/distill_labels.py` is already the right harness (strict checker, failure-weighted sampling,
resume). What it needs is a **domain corpus and a trit-shaped checker**:
- corpus: 6 domains × ~120 prompts drawn from the platform's own work (security, build, ledger,
  inference, network, retrieval), each with a known correct trit and the axiom that licenses it;
- teacher: **gpt-oss-20b on `:8587`** (6/6 on general-domain probes, measured);
- checker: accept a label only if it parses to `VOTE/AXIS` and the axis is the expected one for that item.

Gate: yield ≥60 % accepted labels, and teacher self-agreement ≥90 % when 20 labels are re-run.

## Phase 3 — retrain the 14 heads, ternary-aware, on domain labels

`tools/retrain_k3_v4.py` recipe unchanged (LR 3e-5, clip 0.5, **QAT 0.7** inside training — which is why
post-hoc packing failed: the shipped heads were never trained through that quantiser). Export with
`export_k3_heads_v3.py`, keep both blobs, select by measurement.

Gate (all three, or the run fails as written):
1. dev risk tasks refused 6/6 (baseline held);
2. benign approvals ≥4/6 — the current heads give **0** approvals (14/14 ANALYZE at 0.71–1.000 conf);
3. the TLS-verification/sandbox class is never approved.

## Phase 4 — the two MoE models: roles, not seats

- **gpt-oss-20b (32 experts, top-4)**: teacher and escalation judge. Resident, parses ~99 %.
- **Instella-MoE-think (64 experts, top-6)**: **not seatable** until an explicit `--chat-template`
  strips `` — measured: `enable_thinking:false`, `thinking:false`, plain and `/no_think` all still
  emit it and burn the whole budget. Gate to revisit: parse rate ≥95 % on the vote contract.
- Per-expert routing is **not observable** in llama.cpp, so no plan may claim "expert N is the security
  expert". Domain competence is the only measurable claim.

## Phase 5 — drip the trits into the ledger (already shipped)

`k3_trit`/`k3_conf` are recorded on every vote (`7f98960`). Phase 3's heads replace the blob behind that
same field, so the A/B is a blob swap and a re-run of the dev set — no code change.

## Order and cost

Phase 0 is the only prerequisite for everything else and is the cheapest (one route + one builder).
Phase 2 blocks Phase 3 (no labels, no retrain). Phase 1 and Phase 3 are independent of each other.
Nothing in this plan requires the packed ternary path, a new dependency, or a second model server.
