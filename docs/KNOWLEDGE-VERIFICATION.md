# Knowledge verification (591-600) — research, what was built, and what the numbers say

## What the machine already had, before writing anything

| asset | what it is | used by |
|---|---|---|
| `claims.rs` (tasks 585/587/589) | content-word claim model, 3-way verdict, contradiction rule | 591, 592 |
| `evidence_index.rs` | 2,403 chunks / 191 files, 768-d nomic embeddings on :8593 | 591, 595 |
| `data/corpus_ledger_measured.json` | per-shard **claimed vs measured** counts (10 gaps) | 591, 592 |
| `text_reading.rs` | quantitities with byte offsets, sentence segmentation | 591, 595, 597 |
| corpus documents | 76 docs under `docs/`; **12 carry a correction marker** | 599 |
| **absent** | source verification, conflict resolution, calibration, peer review, reproducibility, retraction detection | all ten |

So this suite is not a second reading pipeline — it is a set of checks **over** the reading pipeline. That
distinction is what makes it possible to say what it can and cannot catch.

## The rules every endpoint obeys

1. **A check that cannot fail is not a check** — each harness constructs at least one case that must fail.
2. **Absence of evidence is not evidence of absence** — 591/595 abstain rather than returning a confident zero.
3. **A verdict carries its evidence** — file, byte offset, sha256, or the rule that decided.
4. **A tie is stated, not broken arbitrarily** — 594 returns UNRESOLVED.

## Measured results

| task | endpoint | result |
|---|---|---|
| **591** fact checking | `POST /kv/factcheck` | **PARTIAL** — 18/27 verdict accuracy (66.7%); abstains on 17/27, of which **7 are true claims** the corpus failed to retrieve |
| **592** consistency | `POST /kv/consistency` | **PASS** — 190 sentences, 11,431 cross-doc pairs; ledger gaps endpoint/recomputation **agree 10/10** |
| **593** source verification | `POST /kv/source` | **PASS** — hash agreement with hashlib 3/3; wrong hash rejected; unrecorded refused; out-of-tree path refused |
| **594** conflict resolution | `POST /kv/conflict` | **PASS** — 4/4 cases, each on the correct rule, incl. the tie → UNRESOLVED |
| **595** uncertainty | `POST /kv/uncertainty` | **PASS (conditional)** — Brier **0.0898** over the 10 claims it scored; reliability: 6 high-confidence predictions, 5 correct |
| **596** suite | `GET /kv/suite` | **PASS** — 10/10 modules, 9 endpoints 200 |
| **597** peer review | `POST /kv/peerreview` | **PASS** — fires on a constructed dirty doc, silent on a clean one; still finds **3 blocking** in a real doc |
| **598** reproducibility | `POST /kv/reproducibility` | **PASS** — referenced artifacts resolve; nonexistent document refused |
| **599** retraction | `POST /kv/retraction` | **PASS** — constructed ERRATUM found and applied to the named claim; **12 of 76 docs** carry a real marker |
| **600** report | `POST /kv/report` | **PASS** — aggregates to `FINDINGS` with totals |

## The calibration defect, and why the fix was structural

The first version returned a probability for every claim. Measured:

```
Brier score 0.2924     <- worse than always predicting 0.5 (0.25)
bin 0.0-0.2: n=20, mean p 0.016, observed 0.350
```

Twenty claims were assigned ~2% and **35% of them were true**. The cause was not the weights — it was publishing a
confidence when no evidence had been found. On this corpus that happens because retrieval misses, not because the
claim is false.

The fix was to make abstention a first-class outcome: no evidence ⇒ `p_support: null`, `abstention: true`. After
that, Brier over the claims it *does* score is **0.0898**, and 17 of 27 claims abstain — 7 of which are true
statements the index could not reach. That limitation is the headline, not a footnote: **the verification stack's
binding constraint is retrieval, not the checks.**

## What the checks caught in real material

- **594's rules are ordered, and the order matters.** A newer *summary* does not beat an older *measurement*;
  recency is the last rule, not the first.
- **597 flagged its own author.** Run on `docs/MACHINE-READING-RESEARCH.md` it produced 5 BLOCKING findings, of
  which 2 were markdown table rows — a false-positive class. Tables now sit out the provenance rules; 3 real
  findings remain (lines asserting results with no path on them).
- **592 caught a false positive of its own kind.** The single cross-document "numeric contradiction" is
  `586 vs 579` — proposal IDs, not measurements. The rule requires shared subject coverage, and two documents
  about the same reasoning suite satisfy it while differing in ID numbers. Reported as-is: a numeric conflict
  detector will treat identifiers as quantities unless they are excluded, and this one does not yet.

## Honest boundaries

- **591/595 are retrieval-limited**, measured above, not hidden.
- **598 checks that referenced artifacts exist; it does not re-run the measurements.** Stated in the response.
- **597 is four textual rules**, not understanding. A well-written unsupported claim passes it.
- **599 detects markers at the start of a line** (`ERRATUM`, `RETRACTED`, `SUPERSEDED`, `CORRECTION`). A correction
  written mid-paragraph is invisible to it.
- **592's contradiction rule is the 589 rule** — same subject, then numeric or polarity conflict. It will not find
  a contradiction phrased with different vocabulary.
