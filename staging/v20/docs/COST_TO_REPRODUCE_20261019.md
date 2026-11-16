# TRM Swarm OS — Cost to Reproduce vs. One Person

Prepared 2026-10-19. The reproduction cost is an **estimate** — a defensible range grounded in
market labor/compute rates and the platform's measured scale (1,107,169 LOC, 630 days, 317 kernel +
247 platform + 36 crates). The one-person cost is the actual cash outlay, not an estimate.

---

## 1. What a conventional team would need to reproduce it

The platform spans six disciplines at once — systems/kernel, ML/retrieval, cheminformatics, genomics,
full-stack, and data engineering. Reproducing 1.1M LOC *and* a measured drug pipeline *and* a
self-governing council is not a one-specialty job:

| role | headcount | why |
|---|---|---|
| Systems / embedded (Rust kernel, UEFI, GPU) | 2 | the microkernel + HDC on GPU |
| ML / retrieval (HDC, SSM, distillation) | 2 | the token-free recall engine |
| Cheminformatics / docking | 1 | the 9 leads + triage + docking |
| Genomics / clinical | 1 | the ACMG classifier, 201K clinical codes |
| Full-stack / dashboard | 1–2 | the React dashboard + 247 platform modules |
| Data engineering | 1 | 2.9M compounds, 597K bioassay, 201K clinical, the NAND fabric |
| **Total** | **8–10** | |

Timeline: **24–36 months** to reach equivalent depth (real benchmarks, not a demo).

## 2. The labor + research cost to reproduce (conventional)

| cost line | range | basis |
|---|---|---|
| Engineering labor (8–10 FTE × ~$225K fully-loaded × 2.5–3 yr) | **$4.5M–$6.8M** | senior engineers at $180K–$250K loaded |
| Model training / HDC research compute | $100K–$500K | GPU training runs (if done conventionally, not distilled) |
| Data licensing (ChEMBL/UniProt/PubChem/ClinVar/AlphaFold) | $100K–$500K | commercial redistribution terms |
| Cloud / infrastructure (30–36 mo) | $0.5M–$2.0M | managed infra vs. on-prem |
| **Total to reproduce** | **~$5.5M–$10M** | **over ~3 years, with a team of 8–10** |

## 3. What it actually cost — one person (Shawn Carruth)

| cost line | amount | basis |
|---|---|---|
| One consumer workstation (RTX GPU, amortized) | ~$2,500 | one-time hardware |
| Electricity | ~$2,200 | ~$105/mo × 21 months |
| Open-source tooling / frameworks | $0 | Rust, FastAPI, React, PyTorch, Lucide |
| Cloud / API fees | $0 | local inference + local retrieval |
| **Total cash outlay** | **~$5,000** | one architect, 630 days |

The *labor* was Shawn Carruth's own time — 630 days of architecture + direction — directed through an
autonomous engineering system (262 commits: Task Runner + NEXUS System), not a paid team.

## 4. The number to present

$$\\frac{\\text{cost to reproduce}}{\\text{cost paid}} = \\frac{\\$5.5M\\text{–}\\$10M}{\\$5,000} \\approx \\mathbf{1{,}100×\\text{–}2{,}000×}$$

**One person, ~$5,000 and 630 days, produced what a funded team would spend ~$5.5M–$10M and 2–3 years
to reproduce.** That is the platform's defensible efficiency claim — and it is the same class of
claim the platform's own voltage doctrine demands: every input (LOC, days, labor rates) is measured
or publicly benchmarked, and the reproduction cost is stated as a range, not a precise number.

## 5. Why the gap is real (not hype)

1. **Distillation over training.** The intelligence is distilled into the NAND fabric + infinity
   cache, so there is no $100K+ model-training bill to reproduce it.
2. **Token-free retrieval over LLM APIs.** The HDC engine runs on-host; there is no per-query cloud
   cost to reproduce, so a competitor's ongoing token bill is a *fixed zero* here.
3. **Autonomous engineering over a paid team.** One architect directing an agent system replaces the
   8–10 FTE line item — the single largest cost in the conventional column.
4. **Sovereign data over commercial licensing.** The knowledge stores are ingested once, locally,
   not re-licensed per seat.

## 6. The honest caveats

- The reproduction estimate is a **range** ($5.5M–$10M), not a point; labor rates vary by geography
  and the team mix is a judgment call.
- It excludes the **opportunity cost** of Shawn Carruth's own time (630 days at a senior rate is
  worth ~$400K as sweat equity) — the $5K is cash outlay, not total economic cost.
- It is a *reproduction* comparison, not a *revenue* claim; the revenue model lives in
  `DIFFERENTIAL_PRICING_20261019.md`.
