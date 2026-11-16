# P5 — the council was convened, the system was researched, and nothing was acquired (2026-09-25)

**P5 as the plan defines it:** "run the task, and if it fails on a named missing artifact, acquire that
artifact — the 7 docking tasks first (engine present), then gnomAD/ClinVar/PharmGKB, then aligners."

**Standing instruction for this phase:** follow the voltage doctrine and the ternary pipeline, and do not
keep adding capability without using what is already set up.

**Result: no acquisition was warranted, and the highest-value action available was wiring an artifact the
platform had already built and never loaded.** The store went from 9,937 chemistry matches to 444,048 for
the same query.

## 1. Council convened (real deliberation, through the P2j route)

`POST /crate/council/vote` — the route wired in P2j — with every verdict re-derived independently from the
platform's own Python `AGENT_MODELS` (ast-parsed, not imported). **4 motions, 4 agreements with the oracle.**

| motion | verdict | trit_sum | voters | abstainers |
|---|---|---|---|---|
| CONTROL documented `MOTION_A` (code, engineering, software, compiler, test, verify, kernel) | PROCEED | 9 | 3 | 12 |
| CONTROL documented `MOTION_E` (research, literature, drug, chemistry, biology, discovery, clinical) | PROCEED | 15 | 5 | 10 |
| P5 frame 1 — ACQUIRE external info | PROCEED | **3** | **1** | 14 |
| P5 frame 2 — WIRE what is already here | PROCEED | **18** | **6** | 9 |

**Read it for what it is.** The tally is weakly discriminating: every voting agent contributes all `+1`
trits, so any motion that draws a voter returns PROCEED. The signal is **participation, not the verdict** —
wiring draws six agents across the roster, acquisition draws one (Seeker alone). The motion terms are
recorded verbatim in `swarm_v2_artifacts/p5_council_deliberation.json` because the verdict is a
deterministic function of them; the two P5 frames are framings of one question, not evidence.

## 2. Research: what is already set up, derived rather than assumed

| need | what exists | where |
|---|---|---|
| docking (7 tasks) | `vina` + `obabel` installed; crate scores a pose; **LP-PDBBind dataset already downloaded** | WSL `/usr/bin/vina`, `~/datasets/LP-PDBBind` (128 MB) |
| molecular descriptors | **the full feature table, 2,897,819 rows** — Task 466, gated against RDKit | `~/drugs/molecule_features.csv` (196 MB) |
| ChEMBL structures | SDF 931 MB + 199,997 RDKit-derived SMILES | `~/drugs/chembl_37.sdf.gz`, `structured_index.db` |
| gnomAD / ClinVar | derived samples already in the tree (5,000 gene constraints / 1,000 classifications) | `scratch/task_113_*`, `scratch/acmg_1000_clinvar_*` |
| relational + clinical | 234 MB ingested in P4 | `~/relational`, `structured_index.db` |
| **PharmGKB** | **nothing** — and no task or license file names it (planning docs only) | — |
| **aligners** (minimap2/bwa/samtools) | **nothing** — `minimap2` appears nowhere in the tree at all | — |

That table is the phase's answer: the plan's two named acquisitions are named only by the plan. By P5's own
rule — acquire only what a task names as missing — **there is nothing to acquire yet.** The one task that
fails (`task_113_gnomad_5000_gene_constraints`, P1) fails a *derivation* contract, not an absent artifact:
its gnomAD rows are present, but `mis_z` was drawn independently of its own `obs`/`exp` (r = 0.0162 over
5,000 rows). The blocker there is executor-layer fabrication, not a download.

## 3. The wiring, measured

`~/drugs/molecule_features.csv` → `structured_index.db`, piped from WSL (no 9p staging):

| measure | value |
|---|---|
| rows stored | **2,897,818 of 2,897,819 (100.00%)** |
| throughput | 24.6 s, **118,013 rows/s** |
| fingerprints present | 2,897,818 |
| duplicate/short/unknown ids | 0 / 0 / 0 |
| property filter `mw 300–350 ∧ hbd ≤ 3`, full coverage | **444,048 matches** (was 9,937 at 6.90%) — measured before the rename below, i.e. on the column now called `nh_oh_implicit`; the stricter `n_plus_o ≤ 3` gives 72,502 |
| cross-check vs the store's RDKit rows (n = 199,997) | mw within 0.05: 186,045 · hbd agree: 184,936 · **hba agree: 52,283** |

## 4. The defect the cross-check found, named not smoothed

`hba` agreed with RDKit on **26%** of rows, so it was tested against candidates over 4,000 molecules:
it matches a plain **N+O atom count on 100.0%**. The producer's own source says so
(`scratch/extract_molecular_features.py:142`), and `hbd` is `N/O atoms carrying an implicit H` (:141) —
which is neither RDKit's Lipinski HBD (91.7%) nor a naive N-H/O-H count (93.1%), the gap being
explicit-H and charged-N/O records. Both names therefore read as Lipinski descriptors while measuring
something else, so they were **renamed to what they measure**:

- `hba` → **`n_plus_o`** (verified 100% against the N+O count)
- `hbd` → **`nh_oh_implicit`** (verified against the producer's rule; 93.1% of a naive count)

No value changed — `ALTER TABLE ... RENAME COLUMN` moves no data, and the range check confirms it
(`n_plus_o` 0–327, `nh_oh_implicit` 0–113 over 2,897,818 rows). Lipinski HBA/HBD remain available from
`molecule.props`, which P4 derived through RDKit itself and which agrees with RDKit 100%.

Also carried forward honestly: `fp` is a 64-bit folded fingerprint **hash**, so it supports exact
fingerprint matching and **not** substructure search, which still needs a bit-vector index that does not
exist here.

## Not claimed

Not that the fabric knows these molecules (it holds 2,180 documents; chemistry belongs to the store), not
that the council chose a course (it PROCEEDs to anything with a voter — participation is the signal), and
not that P5 is finished: it is finished *for what the tasks demand today*. If a task later fails on a named
artifact, P5's rule applies then, with the acquisition and its license recorded.
