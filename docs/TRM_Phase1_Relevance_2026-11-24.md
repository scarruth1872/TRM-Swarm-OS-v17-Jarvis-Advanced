# Phase 1 relevance gate: FAIL 6/24 — and 3 of those 6 are the corpus citing itself

**2026-11-24.** Pre-registered at `docs/phase1_relevance_prereg_2026-11-24.json` (16:56:04Z), before any
judgement: rubric DIRECT / TANGENTIAL / UNRELATED / UNGRADED, **PASS = 18 of 24 DIRECT**, ≥3 UNGRADED makes
the run invalid. Run unmodified on the 24 citations the council actually produced.

## Result — FAIL, with no useful ambiguity

    DIRECT      6/24   (requirement was 18/24)
    TANGENTIAL  9/24
    UNRELATED   9/24
    UNGRADED    0/24   (after fixing the grader, see below)

**And three of the six DIRECT are not evidence at all.** Every one of the 12 evaluation questions appears
verbatim inside the indexed corpus — 22 chunk-level hits — because the corpus contains
`docs/TRM_Blind_Run_2026-11-24.md` and `docs/TRM_Grounded_Voting_2026-11-24.md`, the reports I wrote that
quote the tasks. The retriever found the exam paper. Excluding self-citations, the honest number is
**3/24 DIRECT**.

The single biggest hub: **`docs/PHASE-6-GAPS.md`, 557 KB**, supplied **5 of 24 citations** across unrelated
proposals. A file that large has enough vocabulary to match anything.

## Instrument faults, recorded rather than hidden

| run | what happened | verdict |
|---|---|---|
| 1 | `max_tokens: 90` — the harmony reasoning block consumed the budget, `content` came back empty for 24/24 | **INVALID**, rule fired correctly |
| 2 | `max_tokens: 220` — still empty on 8 hard items | **INVALID** |
| 3 | `max_tokens: 600` + label fallback into `reasoning_content` and bare vocabulary | 24/24 graded |

At no point did the rubric, the threshold, or the item set change. Run 2's graded subset (5 DIRECT of 16)
already failed the rule even if every ungraded item had been counted DIRECT — so the instrument fix could not
have manufactured the verdict, and the three runs agree.

## What this establishes

- **The council's citations are real but mostly irrelevant.** Provenance passing 24/24 last round said only
  that the quotes exist. This round says the quotes do not address the proposal: 9 unrelated, 9 tangential,
  and the DIRECTs contaminated.
- **This is why the council is timid.** Fourteen agents are handed an evidence block that does not bear on
  the question, and abstention is the honest response to irrelevant evidence. The charge work moved benign
  approval from never to sometimes; it cannot fix evidence that says nothing.
- **Retrieval quality is the binding constraint, not the model and not the arithmetic.** The continuum ranks
  at 0.950 recall@10 *of the f32 ranking* — but the f32 ranking itself was never checked against relevance.
  Both retriever and index inherit the same defects: contaminated corpus, one 557 KB hub.

## The two defects, named

1. **Corpus contamination.** Documents written *about* the evaluation are inside the corpus being evaluated.
   A retriever that can cite the question has not retrieved evidence. Any fix must remove evaluation-report
   documents from the index and keep them out — not merely rank them down.
2. **No source diversity.** One 557 KB file can supply every citation. Without a per-source cap, a large
   document with broad vocabulary outranks the specific evidence a decision needs.

## Next round, pre-registered before running

A v4 index that (a) excludes `docs/TRM_Blind_Run_*`, `docs/TRM_Grounded_Voting_*` and the pre-registration
JSONs, and (b) enforces **at most one citation per source per proposal**, gets its own written gate — a
fresh set of dev tasks not present anywhere in the corpus, so contamination is impossible by construction
rather than by exclusion list. This round's FAIL stays on the record exactly as written; the corrected
property is stated beside it.

Script: `tools/phase1_relevance.py`. Raw judgements incl. the judge's text and mechanical overlap:
`docs/phase1_relevance_results_2026-11-24.json`.
