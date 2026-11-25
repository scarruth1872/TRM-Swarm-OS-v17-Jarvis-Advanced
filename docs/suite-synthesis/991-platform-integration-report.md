# Platform integration report

## Components and their measured state

| component | endpoint / artifact | state today |
|---|---|---|
| kernel | `:8022` | HEALTHY, build `3a63c34+dirty` |
| health matrix | `/system/health_matrix` | OPTIMAL (0 anomalies) |
| board | `/kanban/*` | 27 cards, retired state live |
| DDR | `/ddr/stats` | 439 scans, mitigation `SOFTWARE_PARITY_SCRUB_AND_GATE` |
| federation | `/federation/stats` | 3/3 peers, 14 mesh nodes |
| AMR node | `/amr/deployment` | **REFUTED** |
| evidence fabric | 33 shard manifests | 7,793,536 records, seals 33/33 verified |
| Jarvis | `:3000` | routes through the kernel when the kernel is up |
| LCARS | `:5183` | up |

## Integration gaps, measured

1. **Documents are not in the ledger.** A generated report has a sha256 but no ledger entry, so the platform
   cannot prove it wrote it.
2. **The board is not linked to evidence.** Cards carry a reason, not a measurement reference.
3. **Cron reports are local.** Only two jobs deliver to Telegram; the rest are invisible to the operator.
4. **Two interpreters for one science stack.** RDKit (3.12) and sklearn (3.11) cannot share a process.
5. **Docking inputs are unvalidated.** Engine runs, control fails.

## Verdict

The platform is **integrated at the process level and unintegrated at the evidence level**: services talk to
each other, but a claim cannot travel with its proof from document to ledger to board to report. That single
gap explains most of the failures in this suite's history.

---

*Task 991 — 991-platform-integration-report. Generated 2026-11-25 14:56 UTC by `tools/suite_synthesis.py`. Class C.
This is a written deliverable: it is class C/N, which means the platform can produce it and **cannot
verify it**. Every number above was read in the same process that wrote this file.*

Sources: verified shard manifests, suite_classification.json, live endpoints

Cannot claim: that any statement here is verified by the platform; this document is class C/N
