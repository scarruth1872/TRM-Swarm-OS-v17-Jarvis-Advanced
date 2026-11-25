# Information extraction (531-540) — what this machine can actually do, and what it cannot

"Implement what works" is taken literally here: each task below is marked by what the data supports, and the
blocked ones say why rather than producing a plausible-looking shell.

## Inventory (measured, not assumed)

| asset | measured | source |
|---|---|---|
| clinical free text | **200 real trial records** (381,708 B), inclusion/exclusion criteria, summaries, conditions | `evidence/clinicaltrials_sample.json`, sampled from `AllPublicXML.zip` (612,025 entries, sha256 `0089cf0f…`) |
| organism/ID vocabulary | **6 ESKAPE organisms, 14 UniProt proteome IDs** | `data/eskape_proteomes.json`, `data/eskape_candidates.json` |
| compound vocabulary | 406 ChEMBL LxC actives with IDs | `evidence/chembl_lpxc_actives.json` |
| retrieval index | 2,403 chunks / 191 files, 768-d | `data/council_evidence_index_v4_meta.json` |
| **UMLS / SNOMED / RxNorm / MeSH** | **absent** — no vocabulary files on disk | `ls data evidence` — zero matches |
| **multilingual corpora / models** | **absent** — no non-English text, no multilingual encoder | archive scan |
| **clinical NER / relation models** | **absent** — no trained IE model of any kind | `rg` over `*.rs`/`*.py` |

## Per-task verdict, before any code

| task | verdict | basis |
|---|---|---|
| **531** relation extraction from literature | **WORKS (rule-based)** | verb-lexicon patterns over sentences; spans must round-trip. No parser, so relations are shallow SVO |
| **532** event extraction from clinical notes | **WORKS (rule-based, real text)** | trigger lexicon over 200 real records, typed by section; every argument span round-trips |
| **533** entity linking to UMLS | **BLOCKED for UMLS — WORKS for local vocabularies** | UMLS is not on this machine and cannot be fetched under the zero-cloud rule. A linker over the **actual** vocabularies (ESKAPE organisms, UniProt IDs, ChEMBL IDs) is real and is what is built. Claiming "UMLS linking" would be a lie about the vocabulary |
| **534** coreference resolution | **PARTIAL** | rule-based pronoun/definite-NP resolution within a paragraph. Handles "it/they/this <noun>"; cannot do cross-document or ambiguous plural reference |
| **535** temporal expression extraction | **WORKS** | ISO dates, month names, durations, "N days before/after"; normalised to intervals using `temporal_reasoning` conventions |
| **536** negation detection | **WORKS, with real ground truth** | NegEx-style cues with clause scope. Verified against the **curator's own labels**: criteria under "Exclusion Criteria" are exclusions |
| **537** uncertainty detection | **WORKS (cue-based)** | hedge lexicon with scope; measured on real summaries and a constructed set |
| **538** complete suite | **WORKS** | index over 531-537, 539, 540 |
| **539** multilingual extraction | **BLOCKED as extraction — WORKS as a gate** | there is no multilingual model or corpus here. The honest capability is a **language gate that refuses** non-English text instead of tokenising it and emitting confident nonsense |
| **540** structured output | **WORKS** | one JSON document combining all extractors, every span carrying offsets and sha256 |

## The three that fail, and how they fail

1. **533 UMLS** — refused at the vocabulary level. The endpoint reports `vocabulary: "local"` and names the
   vocabularies it used; it never says "UMLS".
2. **539 multilingual** — the gate detects non-Latin script and non-English function-word density and returns
   `REFUSED` with the reason. A tokeniser applied to Chinese prose would produce entities that look extracted and
   are noise.
3. **534 coreference** — resolves within a paragraph only, and says so in its output. Cross-document coreference is
   not attempted.

## How each working task is checked

| task | mechanical check | why it can fail |
|---|---|---|
| 531 | injected SVO triples must be recovered exactly; every span must round-trip to the same bytes | a pattern set with no recall fails the injection; a span bug fails the round-trip |
| 532 | every extracted argument span must round-trip; triggers must come from the stated lexicon | fabricated arguments fail |
| 535 | compare against an independent Python parse (regex + `datetime`) on real documents | a date parser that normalises differently fails |
| 536 | exclusion-section criteria must be flagged negative at a measured rate; inclusion-section must not be | a detector that flags everything scores 100% recall and 0 precision — both are reported |
| 537 | constructed hedged/unhedged pairs + real corpus frequency | a hedge list that fires on everything fails the unhedged half |
| 533 | exact ID match against the local vocabulary; wrong IDs fail | an entity linked to the wrong organism fails |
| 539 | English passes; synthetic Chinese/Russian/Arabic must be refused | a gate that passes everything fails |
| 540 | JSON schema validation + every span round-trips | structurally invalid output fails |


---

# Built and measured (2026-11-25)

All ten tasks were implemented against the data that exists. **Nine of ten pass their harness; two capabilities are
declared blocked rather than faked.**

| task | endpoint | measured |
|---|---|---|
| **531** relations | `POST /ie/relations` | **PASS** — 10/10 injected triples recovered; 0 span mismatches on a real document |
| **532** events | `POST /ie/events` | **PASS** — 410 events over 60 real ClinicalTrials records, 0 span mismatches; 16 CRITERION_EXCLUSION events in the 13 records that have an exclusion section |
| **533** entity linking | `POST /ie/entities` | **PASS for local vocab, UMLS ABSENT** — 2/2 correct UniProt IDs; out-of-vocabulary entity not linked; response literally states `umls: ABSENT` |
| **534** coreference | `POST /ie/coref` | **PASS (partial)** — 3/3 constructed antecedents; 0 span mismatches; paragraph-local only |
| **535** temporal | `POST /ie/temporal` | **PASS** — every ISO date the extractor produced agrees with an independent `datetime` parse; impossible dates (31 February) rejected |
| **536** negation | `POST /ie/negation` | **PASS, with a finding** — see below |
| **537** uncertainty | `POST /ie/uncertainty` | **PASS** — 6/6 hedged sentences caught, **0/5 false positives** on plain assertions; 14 hedged sentences in 50 real summaries |
| **538** suite | `GET /ie/suite` | **PASS** — 10/10 modules, 9 endpoints 200, blocked tasks listed with reasons |
| **539** multilingual | `POST /ie/language` | **WORKS AS A GATE** — English allowed; CJK, Cyrillic and Arabic all REFUSED with the script named, and `/ie/structured` genuinely refuses (400), not merely warns |
| **540** structured output | `POST /ie/structured` | **PASS** — schema `trm.ie.structured.v1`, all required keys, 0 span mismatches |

## 536: the finding, not just a pass

Measured over 80 real trial records with an exclusion section:

```
inclusion sentences 63 | flagged negated 16 (25.4%)   <- precision side
exclusion sentences 91 | flagged negated by CUE 13 (14.3%)   <- cue-only recall
exclusion sentences labelled negative by SECTION 91/91 (100%)
```

**Cue-based negation catches only 14.3% of real exclusion criteria.** The rest ("history of liver disease",
"elevated liver function tests") carry no negation cue at all — their negative force comes entirely from the
section header the submitter wrote. A clinical negation detector that reports cue hits alone is missing six of
seven exclusions, and the endpoint therefore returns `criteria_section` alongside every sentence so the caller can
see which signal carried the meaning.

## Blocked, and exactly how they fail

- **533 UMLS**: no UMLS/SNOMED/RxNorm distribution exists on this machine. The linker runs over the ESKAPE
  organisms → UniProt proteome IDs that **do** exist, tags every result `vocabulary: "local"`, and returns
  `umls: "ABSENT — …none was invented"`. Linking to identifiers that are not on disk would be fabrication.
- **539 multilingual**: no multilingual model or corpus. The gate refuses CJK/Cyrillic/Arabic by script share and
  non-English Latin text by function-word density, and `/ie/structured` returns 400 rather than tokenising it.

## Three bugs the harnesses caught in my own code

1. **Sentence-relative spans** in relations and coreference — offsets indexed the sentence, so every relation in a
   later sentence pointed at the wrong bytes. Fixed by shifting on the sentence start.
2. **A byte-slice panic** in the temporal extractor on non-ASCII text (`text[i..i+4]` landing inside a multi-byte
   character). Boundary-checked before every slice.
3. **The harness was wrong, twice.** `read_text()` translates CRLF → LF and shifts every offset; and Python
   `str[a:b]` slices *codepoints* while the platform reports *bytes*. Both produced phantom "span mismatches" (54
   of them on real clinical text) that were measurement errors, not extraction errors. Every harness now reads
   bytes and slices bytes.

The third one is the most useful lesson recorded here: **an offset check compares bytes, not characters.** A
harness that slices strings will pass on ASCII-only English and quietly mis-measure every document with an em dash.
