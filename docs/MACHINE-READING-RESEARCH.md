# Machine reading (581-590) — techniques that fit this machine, and how each could fail

Researched 2026-11-25 against what is actually installed. The point of this document is not to list techniques
in the abstract; it is to name, for each task, **the check that would prove the implementation wrong**.

## What the machine has

| asset | measured | source |
|---|---|---|
| retrieval index | **2,403 chunks from 191 files**, 768-dim, cosine with a 0.30 floor | `data/council_evidence_index_v4_meta.json` |
| embedding model | `nomic-embed-text` on **:8593** (local llama.cpp) | same meta |
| generator | `gpt-oss` on **:8587** (local llama.cpp, Vulkan) | `/health` on both ports |
| text corpus | 126 files / 5 MB under `docs/`, 227 files / 17 MB under `evidence/`, 5 blog posts | `find` + `du` |
| existing reading code | embeddings + ranking only: `evidence_index.rs` (`evidence_for`, `embed_query_pub`, `serialize`) | kernel source |
| **absent** | claim extraction, entailment/NLI, sentence segmentation, BM25, ROUGE, citation linking | `rg` over `*.rs`/`*.py` — zero hits each |

So machine reading starts from **retrieval that works** and **nothing above it**: no claim model, no entailment,
no summariser. Every technique below is therefore built as an explicit, inspectable pipeline rather than a
prompt.

## Techniques, per task

| task | technique | why this one | the check that can fail |
|---|---|---|---|
| **581** document understanding | structural parse (headings → sections → paragraphs), unit segmentation, per-unit metadata (file, offset, seal) | the corpus is markdown and JSON, not scanned PDFs; structure is recoverable exactly | every recovered offset must round-trip: `text[offset..offset+len]` must equal the unit verbatim. A parse that silently drops a section fails |
| **582** reading comprehension | extractive QA over retrieved chunks: retrieve → locate span → answer with citation | extractive answers are checkable; an abstractive answer is only checkable against a human label | a labelled question set whose answers are **verbatim strings** in the corpus; the returned span must be a substring of the cited file and contain the answer |
| **583** multi-document summarisation | MMR (maximal marginal relevance) over chunk embeddings: relevance minus redundancy | works with the embeddings already on disk; deterministic | coverage of query-relevant facts must beat a random-selection baseline at the same length, **and** mean pairwise redundancy must be lower than plain top-k |
| **584** evidence extraction | sentence segmentation + span selection, each span carrying file/offset/sha256 | evidence without provenance is an assertion | every span's offsets must resolve to the identical bytes in the file, and the file's sha256 must match the ledger |
| **585** claim verification | retrieve evidence → 3-way verdict SUPPORTED / REFUTED / UNKNOWN by numeric + entity consistency | the honest verdict set includes UNKNOWN; a 2-way verifier must guess | labelled set built by construction: claim with an exact quoting sentence (SUPPORTED), the same claim with one number flipped (REFUTED), a claim about something absent from the corpus (UNKNOWN). Fabricating support on the third class fails the test |
| **586** complete suite | an index + runner over 581-585, 587-590 | 579 showed the pattern works | every claim in the suite report must resolve to a harness that runs |
| **587** scientific claim extraction | pattern-and-entity extraction over scientific prose: quantity, unit, entity, polarity, modality | rule-based extraction is auditable line by line | hand-labelled extraction subset: precision measured, and every extracted quantity must appear verbatim in its source sentence |
| **588** evidence-based recommendation | evidence-weighted ranking with mandatory citations, and a counterfactual test | a recommendation that does not move when the evidence moves is not evidence-based | flip the evidence and the recommendation must flip; with citations removed the harness must refuse |
| **589** contradiction detection | normalise (subject, predicate) → compare values/polarity across documents | contradictions are usually numeric or polarity conflicts on the same relation | constructed set: sentences with one number flipped (contradiction) and unrelated sentences (not); precision and recall both reported, no cherry-picking |
| **590** knowledge synthesis report | aggregate claims + evidence into sections, each sentence carrying an evidence id or marked INFERENCE | synthesis that cannot be traced is a blog post | every sentence must resolve to an evidence id that exists, or be tagged INFERENCE; unmarked sentences fail |

## The one technique this machine should NOT pretend to do

**Abstractive summarisation with a faithfulness guarantee.** `gpt-oss` on :8587 can write fluent prose, but
nothing here can prove the prose is supported by the corpus. Where generation is used (590), it is confined to
wording and every sentence is checked back against an evidence id; the summariser itself (583) stays extractive
so that faithfulness is structural rather than aspirational.

## Cost

All ten are small: they reuse the index on disk (no re-embedding), add no model, and the only generation is
optional. The expensive part is the labelled sets, which are built by construction (flip a number → REFUTED)
rather than by hand-labelling hundreds of items — and the documents say which are constructed, because a
constructed set tests a specific mechanism and not general reading ability.


---

# Council decisions and measured results (2026-11-25)

All ten tasks were put to the council with evidence and a pre-registered acceptance test. **All ten were
enacted**:

| task | proposal | verdict |
|---|---|---|
| 581 document understanding | 2595 | **ENACTED 14/0/0** |
| 582 reading comprehension | 2597 | **ENACTED 14/0/0** |
| 583 multi-document summarisation | 2596 | **ENACTED 12/2/0** |
| 584 evidence extraction | 2598 | **ENACTED 13/1/0** |
| 585 claim verification | 2601 | **ENACTED 12/2/0** |
| 586 complete reading suite | 2604 | **ENACTED 13/1/0** |
| 587 scientific claim extraction | 2600 | **ENACTED 12/2/0** |
| 588 evidence-based recommendation | 2603 | **ENACTED 12/2/0** |
| 589 contradiction detection | 2599 | **ENACTED 13/1/0** |
| 590 knowledge synthesis report | 2602 | **ENACTED 13/1/0** |

## Measured results

| task | harness | result |
|---|---|---|
| 581 | `reading_document_verify.py` | **PASS** — 521 units across 12 files; 0 bad bounds, 0 preview mismatches |
| 582 | `reading_qa_verify.py` | **PASS** — 13/20 answered correctly (65%) against a 7/20 baseline; **13/13 citations resolve** |
| 583 | `reading_summarize_verify.py` | **PASS** — MMR less redundant than top-k on 8/8 queries, coverage preserved within 10 points |
| 584 | `reading_evidence_verify.py` | **PASS** — 960 spans; 0 unresolved, 0 mismatches, 0 hash mismatches |
| 585 | `reading_claim_verify.py` | **PARTIAL** — see below |
| 586 | `reading_suite_verify.py` | **PASS** — 10/10 modules present, 9 endpoints answer 200 |
| 587 | `reading_scientific_verify.py` | **PASS** — 411 claims, 1,038 quantities, **0 unlocatable** |
| 588 | `reading_recommend_verify.py` | **PASS** — unsupported option refused; ranking flips when evidence flips; citations + support reported |
| 589 | `reading_contradiction_verify.py` | **PASS** — 12/12 constructed contradictions found, 0 false positives |
| 590 | `reading_synthesis_verify.py` | **PASS** — 30 sentences, 0 mismatched citations, 0 untagged |

## 585 is partial, and the reason is measured

The abstention discipline is **exactly right**: 3/3 absent-topic claims returned UNKNOWN, **0 fabricated support,
0 cases of silence treated as refutation**. That is the property that matters most — a checker that invents
support is worse than no checker.

But recall on claims drawn from the corpus is low:

```
quoting sentence SUPPORTED : 6/15   (40%)
flipped number REFUTED     : 5/15   (33%)
absent claim UNKNOWN       : 3/3    (100%)
fabricated support         : 0
silence treated as refuted : 0
```

The cause is **retrieval, not the checker**: when the evidence sentence is actually retrieved, coverage is
0.67–1.00 and the verdict is correct (spot-checked on AGENT-CHAT-INTELLIGENCE.md and REASONING-SUITE-COMPLETE.md).
Across arbitrary sentences the retriever often surfaces a different document, and the checker — correctly —
declines to support a claim whose evidence it never saw. Recorded as PARTIAL rather than dressed up.

## Three defects found in the machinery, not the tasks

1. **A panic on 21 of 40 corpus files.** The abbreviation check sliced backwards by a fixed byte count and landed
   inside a multi-byte character (`≥`, `–`, `§`). Fixed with a char-boundary guard; the harness now runs over
   files containing those characters.
2. **Proposals in EVALUATING are not persisted.** Ten proposals were submitted at once; a kernel restart for the
   next build erased all ten — the ledger kept no record that a decision had ever been requested. They were
   resubmitted in batches of three. A proposal that vanishes is a decision the operator believes is pending.
3. **Firing ten deliberations at once blocks the kernel.** CPU sat flat at 2.4 s across 65 threads with the model
   serving other work: the spawned threads never progressed. Batches of three complete in ~8 minutes; ten do not.

## The boundary that did not move

Every answer here is **extractive**: a sentence that exists in the corpus, cited by file, byte offset and sha256.
No endpoint generates prose. Where synthesis is involved (590) the sections are evidence spans, and a caller that
adds inference must tag it `INFERENCE` or fail the check. Abstractive summarisation with a faithfulness guarantee
remains out of reach on this machine, and nothing here pretends otherwise.
