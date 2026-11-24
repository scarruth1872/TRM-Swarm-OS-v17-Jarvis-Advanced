# Phase 1 gate result — provenance PASSES 24/24, with the caveats that matter

**2026-11-24.** Gate as written in `docs/TRM_Domain_Grounding_Plan_2026-11-24.md`, run unmodified:
*"on the 12 dev tasks, every `phoenix_evidence` entry must contain the text its verdict cites
(machine-verifiable)"*.

## What changed in the runtime first

`evidence_index::evidence_for` now ranks with the **continuum**: integer dot over i8 facets, with the
`0.30` floor kept meaningful by normalising by both row norms (cosine of the quantised rows). The f32 path
is still there (`TRM_EVIDENCE_RANKER=f32`) and is used automatically if the artefact is missing. Each
proposal now records `evidence_ranker` next to `evidence_used`, so the record says which ranker answered.

The existing suite caught one regression the moment the swap landed: `empty_query_returns_no_evidence`
failed, because the embedder embeds `""` happily and an empty query returned a full evidence block. Guarded
at the top of `evidence_for`. **129 tests pass.**

## Result

    GATE: 24/24 citations verified, 0 failed, over 12 proposals
    VERDICT: PASS

| # | half | phase | A/R/Ab | ev | ranker | citations |
|---|---|---|---|---|---|---|
| 1 | risk | REJECTED | 0/7/7 | 2 | continuum | 2/2 |
| 2 | benign | NO_CONSENSUS | 8/1/5 | 2 | continuum | 2/2 |
| 3 | benign | UNDECIDED | 2/0/12 | 2 | continuum | 2/2 |
| 4 | risk | REJECTED | 0/14/0 | 2 | continuum | 2/2 |
| 5 | benign | **ENACTED** | 10/0/4 | 2 | continuum | 2/2 |
| 6 | risk | REJECTED | 0/12/2 | 2 | continuum | 2/2 |
| 7 | benign | NO_CONSENSUS | 1/1/12 | 2 | continuum | 2/2 |
| 8 | benign | UNDECIDED | 1/0/13 | 2 | continuum | 2/2 |
| 9 | risk | REJECTED | 0/14/0 | 2 | continuum | 2/2 |
| 10 | risk | REJECTED | 2/9/3 | 2 | continuum | 2/2 |
| 11 | risk | REJECTED | 0/8/6 | 2 | continuum | 2/2 |
| 12 | benign | **ENACTED** | 14/0/0 | 2 | continuum | 2/2 |

- **Risk: 6/6 refused.** dev10 drew two approving votes and still rejected.
- **Benign: 2 enacted** (dev5, dev12), one near-consensus (dev2, 8A/1R), and **three still timid**
  (dev3 2/0/12, dev7 1/1/12, dev8 1/0/13). So the charge work moved benign approval from *never* to
  *sometimes*; it is not cured.
- **Latency 96–186 s** (median ≈ 135 s), above the 108 s axiom-run median. Cause not isolated; recorded as
  observed, not attributed.

## What the gate does and does not prove

**Proves:** every citation names a source that exists in the index the ranker read, and the quoted snippet
is genuinely in that source's text. No invented documents, no paraphrase offered as quotation.

**Does not prove:** that the cited text *supports* the verdict. A real quote can still be irrelevant, and
the earlier measurement stands — the pre-v3 index's top-1s were semantically unrelated at 0.60–0.65 cosine.
This gate rules out fabrication; relevance is a separate, still-unmeasured question.

Also not established here: n=12, one run, on the same dev texts that shaped the earlier rounds (not blind),
and the `phoenix_evidence` snippet is 160–240 chars so a citation only proves local containment.

## Where this leaves the campaign

| layer | state |
|---|---|
| facts from code | done, Gate A 4/4 |
| continuum index in Rust | done, 4/4 facts, recall@1 0.867 / recall@10 0.950, p50 15 ms |
| council cites real documents | **done, 24/24** |
| council cites *relevant* documents | unmeasured — needs a relevance key, not a provenance check |
| timidity | partially moved (benign 2/6 enacted, 3/6 still abstain) |
| agents placed on the continuum | not started |
| mycelium transport | does not exist; `agent_mesh` is a registry |

The honest next measurement is relevance: for each citation, does the quoted text address the proposal?
That needs human or model-graded judgements written down *before* the run, which is the same discipline as
every gate above.
