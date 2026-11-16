# Task-Yield Matrix — what this host can be trusted to do

Measured 2026-11-13 on the live stack. Same 1,000 grounded code-analysis tasks per tier, five task shapes, three
validators: **schema** (required keys + enum membership), **grounded-lenient** (every cited symbol occurs in the
2,400-character excerpt shown) and **grounded-strict** (every cited symbol is also an identifier of 3+ characters).

The point of this file is bare-metal planning: which task classes this machine can run **unattended**, which must go
to the reliable tier, and which must never be published without a human.

---

## The five task shapes

Each is sent verbatim, with a real repo file excerpt, and must come back as one JSON object.

| shape | what it asks | keys | enums |
|---|---|---|---|
| `testability` | Judge how testable this code is and what would block a test | testability, blockers, entry_point, evidence_symbols | easy / possible / hard |
| `docstring` | Write what the documentation for this code should say | summary, inputs, returns, evidence_symbols | — |
| `failure` | List ways this code can fail at runtime, most likely first | failure_modes, most_likely, evidence_symbols, guard | — |
| `risk` | What is this responsible for, how dangerous to change | responsibilities, risk, evidence_symbols, fix | none / low / medium / high |
| `coupling` | What it depends on and what depends on it | depends_on, exposes, coupling, evidence_symbols | low / medium / high |

---

## Results by shape and tier

| shape | 1.5B Q4 schema / grounded-strict / p50 | Bonsai 2 2B schema / grounded-strict / p50 | Bonsai 2 27B schema / grounded-strict / p50 |
|---|---|---|---|
| testability | **97.0 %** / **69.0 %** / 1.8 s | 95.5 % / 39.0 % / 4.6 s | 100 % / **100 %** / 58.6 s |
| docstring | 96.0 % / 58.5 % / 2.2 s | 78.5 % / 51.5 % / 4.3 s | 100 % / **100 %** / 75.5 s |
| failure | 94.5 % / 53.0 % / 3.0 s | 79.5 % / 27.5 % / 6.0 s | 100 % / 90.0 % / 96.2 s |
| risk | 92.5 % / 50.0 % / 2.8 s | 90.0 % / 36.5 % / 4.8 s | 100 % / **100 %** / 54.7 s |
| coupling | 86.5 % / 58.5 % / 2.2 s | 92.5 % / **59.5 %** / 4.8 s | 100 % / **100 %** / 67.5 s |

Aggregate: 1.5B 93.3 % / 57.8 % at 1.442 items/s · 2B 87.2 % / 42.8 % at 1.472 items/s · 27B 100 % / 98 % at
0.113 items/s.

## Which shapes yield the best results, and why

The two small models do **not** rank the shapes the same way, and the ranking is not what the schema numbers suggest.

**1.5B — best to worst strict grounding:** testability **69.0 %** > docstring 58.5 % = coupling 58.5 % > failure
53.0 % > risk **50.0 %**. `testability` wins because it is extraction with a closed vocabulary: one function name
copied from the text plus a three-way enum. `risk` is last because grading severity invites the model to invent
symbols to justify the grade.

**2B — a different order:** coupling **59.5 %** > docstring 51.5 % > testability 39.0 % > risk 36.5 % > failure
**27.5 %**. Note the trap: `testability` has the 2B's *best schema* score (95.5 %) and its *third-worst* strict
grounding (39.0 %) — it produces well-formed objects whose citations are junk. A schema-only gate would pass those
straight through.

**The 2B's real problem is degeneration, not ignorance.** 109 of its 128 schema failures are unclosed JSON that
stopped mid-string, concentrated in `docstring` (43) and `failure` (41). More token budget did not help.

**The 27B is flat.** 100 % schema on every shape, and 100 % strict grounding on four of five — `failure` is the one
at 90 %, which is itself the useful signal: enumerating *how* code breaks is the hardest thing asked here, and even
the reliable tier slips on it.

## Are the bad citations real fabrications?

Yes. Of 123 cited symbols the 1.5B got wrong, **112 (91.1 %) do not occur anywhere in the file** — not merely outside
the excerpt. For the 2B it is 99 of 101 (98.0 %). Excerpt strictness explains under a tenth of the error, so this is
invention, not truncation of context.

## What this means for bare metal

| task class | tier | gate before use |
|---|---|---|
| Triage labels, testability/severity enums, routing decisions | 1.5B on 4 slots | schema validator; expect ~⅓ rejected on strict grounding |
| Draft docstrings and notes for human editing | 1.5B | schema validator + human read; never publish raw |
| Risk assessment, dependency maps, failure analysis | **27B only** | schema validator; 98 % identifier-grade citations |
| Anything that gets quoted, cited or committed | 27B via `tools/offline_queue.py` | validator + the offline queue, never the latency path |
| Bulk drafts where a wrong citation is caught later | 1.5B | validator only |

**The 2B has no measured role on this host.** It is slower per slot than the 1.5B, scores worse on every structure
and grounding measure, and doubling its slots bought no aggregate throughput (1.472 vs 1.442 items/s). Keeping it
resident costs VRAM for a tier that does not win a single shape.

**Never treat a passing validator as correctness.** Schema + grounding proves the answer is *well-formed and
cites something real*. It does not prove the judgement is right — 1.5B `risk` can cite `MONAD_TRUTH_ANCHOR`
correctly and still grade the danger wrong. That is the class that must reach the 27B or a human.

## Reproduce

```bash
python tools/loadtest_tasks.py --n 1000 --ports 8730 --slots 8 --max-tokens 500 --no-think --seed 23 --out scratch/loadtest/run1000_bonsai2b
python tools/loadtest_tasks.py --n 100 --ports 8612 --slots 8 --max-tokens 500 --no-think --timeout 400 --seed 23 --out scratch/loadtest/run100_bonsai2_27b
```

Results: `scratch/loadtest/{run1000_clean,run1000_bonsai2b,run100_bonsai2_27b}/`
