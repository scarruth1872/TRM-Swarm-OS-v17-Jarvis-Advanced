# Knowledge synthesis — deployment record

## How this suite is produced and kept true

| step | mechanism | failure mode it prevents |
|---|---|---|
| inputs | read from verified shard manifests, probe JSONs and live endpoints in one process | a document quoting a number from memory |
| seals | payload hash after the 128-byte header, 33/33 verified | treating a changed shard as authentic |
| classification | `tools/suite_classify.py`, one reason string per task | claiming 1000 runnable tasks |
| regeneration | `python tools/suite_synthesis.py` | drift between the documents and the platform |
| manifest | `data/suite_synthesis_manifest.json` with per-file sha256 and section counts | an edited document passing as generated |

## Deployment state at generation

- kernel: **HEALTHY**, build `3a63c34+dirty`, uptime 506 min
- board: 27 cards — 6 todo / 0 doing / 0 review / 14 done / 7 retired
- federation: 3 of 3 peers active, mesh nodes 14, anchor `0x4D4F4E4144000001`
- DDR: 439 scans, mitigation `SOFTWARE_PARITY_SCRUB_AND_GATE`
- AMR node: **REFUTED** (3 refuted / 2 undetermined)

## Regeneration contract

Re-running `tools/suite_synthesis.py` rewrites every file in `docs/suite-synthesis/` and the manifest. If a
number moves between runs, the diff shows which measurement moved. A document that cannot be regenerated is
not part of this suite.

## What deployment does **not** mean

These files are not served by the kernel and are not sealed into the ledger. Deployment here means "produced
by a command, with provenance", not "attested by the platform". That gap — a document that the ledger can
vouch for — is one of the 50 features in `972-platform-features.md`.

---

*Task 967 — 967-knowledge-synthesis-deployment. Generated 2026-11-25 14:56 UTC by `tools/suite_synthesis.py`. Class C.
This is a written deliverable: it is class C/N, which means the platform can produce it and **cannot
verify it**. Every number above was read in the same process that wrote this file.*

Sources: verified shard manifests, suite_classification.json, live endpoints

Cannot claim: that any statement here is verified by the platform; this document is class C/N
