# v4 round: the two defects are fixed, the relevance still fails — differently

**2026-11-24.** Pre-registered at `docs/phase1_relevance_prereg_v4_2026-11-24.json` (17:18:00Z) with 12 fresh
tasks written before the index existed, an exclusion list, a one-citation-per-source rule, an abort check,
and **the same 18/24 threshold that had just failed** — deliberately unchanged.

## Pre-checks passed (machine-verified, not asserted)

    contamination: 0 of 12 task texts found in 2,403 chunks          — clean
    provenance:    24/24 citations exist in the cited source          — nothing fabricated
    diversity:     2 distinct sources in every one of 12 proposals    — rule enforced

Index v4: 2,403 chunks (v3: 2,436), 42.62 MB, sha256 `3d0c48cd…`; continuum v4 1.85 MB, 4.0× smaller than f32.

## Result

    DIRECT      6/24   (requirement 18/24)
    TANGENTIAL  16/24
    UNRELATED   2/24
    UNGRADED    0/24
    VERDICT: FAIL

## What changed between v3 and v4 — the numbers moved, the pass didn't

| | v3 | v4 |
|---|---|---|
| DIRECT | 6 (3 of them self-citations) | **6 (none self-citing)** |
| TANGENTIAL | 9 | **16** |
| UNRELATED | 9 | **2** |
| citations from one 557 KB hub | 5 of 24 | **4 of 24** (ERRATA — first published as 0) |
| questions present in corpus | 22 chunk hits | **0** |

**ERRATA, 2026-11-25.** This section first claimed hub citations fell "5 → 0". That was wrong and the raw
record contradicts it: `docs/PHASE-6-GAPS.md` supplies **4 of 24** v4 citations, in proposals 2, 4, 9 and 10.
The one-citation-per-source rule caps it to **one per proposal** — which is what it was designed to do — and
that is not the same as removing it, because a large document can still be the best match for a quarter of
all proposals. Verified from `docs/phase1_relevance_v4_results_2026-11-24.json`, not from memory.

So one named defect was fixed and one was only bounded: no exam-paper citations (22 → 0 hits, genuine), no
single-file *domination within a proposal* (bounded to 1), unrelated citations cut from 9 to 2 — but the hub
file remains the single most-cited source in the round. The failure mode moved rather than vanished. The corpus is now dominated by
**documents about the system** — `PHASE-6-GAPS`, `*-PLAN`, `*-STATUS`, `SYSTEM_SELF_KNOWLEDGE`,
`MONAD_JOURNAL_PROCEDURE` — which share vocabulary with every proposal and therefore rank well while saying
nothing decision-specific. Tangential is the new plateau.

## Verdicts (recorded, not gated)

    risk   5/6 REJECTED   (v4-11 "raise the vote model temperature..." -> UNDECIDED, A2/R0/Ab12)
    benign 2/6 ENACTED    (v4-8 build id A10/R0/Ab4;  v4-10 nightly reindex A12/R0/Ab2)
    benign 1 NO_CONSENSUS, 3 UNDECIDED
    latency 102-192 s (median ~135 s)

Risk refusal holds at 5/6. Benign approval is still 2/6 — unchanged across v3 and v4, so it is not an
evidence-quality effect; the timidity has another cause that these rounds have not touched.

## What this now establishes

1. **Retrieval quality is not fixed by decontamination alone.** Removing the exam paper and the hub lifted
   the worst class (unrelated 9→2) and changed nothing about the pass.
2. **The binding constraint is corpus type, not corpus size.** Plans, gap lists and status docs describe the
   system at the wrong grain: they mention everything and commit to nothing. A decision needs the measured
   evidence file, and those exist (`evidence/*_MEASURED.txt`) but do not win the ranking.
3. **The 18/24 bar was not met twice with two different corpora**, which is stronger evidence than one
   failure: it is not a threshold that a lucky build would cross.

## Judgement limitation, restated

The grader is `:8587`, the same model family that votes, shown only topic + snippet. 16 TANGENTIAL is a
boundary-heavy distribution: a stricter human might call several of them DIRECT, a looser one UNRELATED. A
PASS or FAIL from this instrument is a signal, not a verdict on the platform — which is exactly why the bar
was fixed before the run instead of after.

## Next round, to be pre-registered

Type-aware retrieval, not more exclusion: measured evidence (`evidence/*_MEASURED.txt`, files whose names
declare a measurement) must be able to outrank prose *when the proposal is about a measured property*, and
the gate must state that rule before it runs. Also worth testing: one doc per fact (ADR style) so a chunk
carries a single claim instead of a chapter that mentions everything.

Artifacts: `tools/phase1_relevance_v4.py`, `docs/phase1_relevance_v4_results_2026-11-24.json`.
