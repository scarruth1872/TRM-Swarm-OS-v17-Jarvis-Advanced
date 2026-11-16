# P6 — the fabrication quarantine, named and served (2026-09-25)

**P6 as the plan defines it:** "the 55 physical + ~39 operator tasks — LpxC structure and TRM-ABX-220
identity from the operator; the rest stay blocked and are reported as blocked, with the reason."

**What this phase actually had to fix first:** the platform's own `TASK_SUITE_TRUTH` finding — *artifacts
do not track feasibility* (31% artifact rate for satisfiable tasks vs 30% for gated ones). Two artifacts
cannot both be right, so the phase's first work is to name which ones claim what they cannot have, and to
put that on a surface the platform serves rather than in a report.

## 1. Council convened (5 motions, 5/5 agreeing with the oracle)

| motion | trit_sum | voters |
|---|---|---|
| CONTROL documented `MOTION_A` | 9 | 3 |
| verify / test / audit / compliance / governance / formal / proof / qa | 9 | 3 |
| truth / static analysis / fuzz / coverage / mutation / qa | 6 | 2 |
| verify / test / kernel / code / engineering / compiler / pipeline / nand / sram / retrieval | **18** | **6** |
| clinical / trial / regulatory / patient / monitor / compliance / governance | 6 | 2 |

Same caveat as P5, stated rather than buried: the tally is weakly discriminating, so read participation.
Wiring again draws the most agents.

## 2. The quarantine, derived per task (not summed from the doc)

`scratch/p6_quarantine_fabrications.py` joins the platform's own per-task classification
(`scratch/task_suite_domain{1..5}_classified.json`, 467 rows with `gate`/`why`/`catalyst`) with what is
actually on disk and **which producer wrote it** — 26 `run_tasks_*` band runners, **20 of which call
`random.uniform/randint`**.

| verdict | tasks | meaning |
|---|---|---|
| BLOCKED_NO_ARTIFACT | 167 | gated, no artifact claimed — the honest state |
| UNUSED_CAPACITY_NO_ARTIFACT | **155** | satisfiable, nothing produced — **matches the doc's 155 exactly** |
| **FABRICATED_RESULTS_FOR_A_GATED_TASK** | **74** | artifacts claim results for tasks the classification says are gated (doc: 72) |
| **FABRICATED_BY_A_SEEDED_RNG_PRODUCER** | **61** | satisfiable task, artifact written by a runner that fabricates — **the doc's rule cannot see this set** |
| ARTIFACT_UNVERIFIED_NO_CONTRACT | 10 | artifact present, no contract checked |

145 tasks have artifacts on disk (doc: 143 — this index also covers the repo root and
`swarm_v2_artifacts/`). **135 artifacts are quarantined**, 135 more than the doc's 72 captured, because a
satisfiable task answered by a seeded RNG is no more a derivation than an impossible task answered at all.

**Stated honestly:** the 61 are named by a **producer tell**, not by a value-level proof — the runner that
wrote them fabricates, so its output is not a derivation, but this pass does not prove every field in
them is invented. The 74 rest on the classification the platform itself made, quoted per row in the
artifact's `evidence` field.

## 3. Served, not filed

New route **`GET /verification/fabrication`** (`verification_handlers.rs`), added beside the existing
census and proof routes — a new block, never a re-pointed arm. It reads
`swarm_v2_artifacts/fabrication_quarantine.json` and **re-derives its tally from the rows on every
request**: the artifact's own stored summary is deliberately not what it reports, because a stored total
can drift from the data it summarises. `?verdict=` filters, `?limit=` truncates the list but never the
counts, POST answers **405**, and an absent artifact answers **501 `quarantine_absent`** naming the
command — an absent quarantine is not a clean bill of health.

**Deployed:** `6cf7ef08f57c6ced`, 2,775,552 B (P2j was 2,752,512 B). Staged as `.new` and moved in ahead
of the supervisor's ~3 s tick; kernel pid 28076 → 28216, `/health` in **4.46 s**, and the new image is
proved by **process start 12:07:28 > exe mtime 12:06:59**, not by the pid alone.

**Ruler `scratch/verify_fabrication_route.py`: 14/14**, including the negative control — with the artifact
renamed away the route answers 501 + the command, and the artifact is restored byte-identical before the
check that the route works again (a quarantine that silently reads as empty would be worse than none).

Live now: `checked 467 · with artifacts 145 · fabricated 135
{BLOCKED_NO_ARTIFACT 167, UNUSED_CAPACITY_NO_ARTIFACT 155, FABRICATED_RESULTS_FOR_A_GATED_TASK 74,
FABRICATED_BY_A_SEEDED_RNG_PRODUCER 61, ARTIFACT_UNVERIFIED_NO_CONTRACT 10}`.

## 4. The operator's half

Unchanged and still the operator's: **TRM-ABX-220's identity** (no SMILES or structure anywhere on disk —
task 1 asks for a 500-conformer docking sweep against LpxC and cannot start without it) and **LpxC's
structure**. The 55 non-acquirable tasks (patient cohorts 19, an isolated-lab malware corpus 13, TPM/HSM
7, a second federation node 7, mass spectrometry 4) remain blocked with reasons, and 155 satisfiable tasks
remain unanswered capacity.

## Not claimed

Not that every quarantined artifact is worthless (61 are quarantined by producer, not by value), and not
that the 155 are blocked — they are *unstarted*, which is the platform's real unused capacity and the
opposite problem to fabrication. What is claimed is narrow: the fabricated set is named per task with
evidence, it is served on a route the kernel answers, and the route cannot report a stored total, a
truncated list as a full count, or an absent artifact as clean.
