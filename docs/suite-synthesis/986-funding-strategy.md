# Funding strategy

## What a funder can verify here

| claim | verifiable now | how |
|---|---|---|
| "We can predict LpxC potency" | yes, with an error bar | CV5 R² 0.7226 on 406 actives, reproducible by a command |
| "We hold 10.8M curated records" | yes | 33 sealed shard manifests, payload hashes verify |
| "We boot a custom kernel" | yes | EFI image byte-identical to the boot media on `G:` |
| "We run a 3-node federation" | yes | `/federation/stats`, 3 active |
| "We will deliver a 10-node, 584-agent swarm" | **no** | 3 nodes, 14 mesh nodes |
| "We will produce wet-lab validation" | **no** | no assay has run on this machine |

## The proposal shape that fits

Fund **the join and the controls**, not the architecture:

1. BindingDB × ChEMBL-LpxC × PDBbind integration (a week, falsifiable, produces a table).
2. A redock control harness that fails published protocols (differentiating, cheap).
3. A claim register that makes every document sentence traceable (infrastructure for trust).
4. One wet-lab collaboration with a named lab and a named assay, with the platform's role limited to
   prioritisation.

## What to say about the failures

Funders increasingly discount "AI discovered X" claims. A proposal that says *our docking numbers were
unreliable until we built a control, here is the control, here is what it rejects* is more credible than one
that reports only successes — and it is true here, which is the only reason to say it.

---

*Task 986 — 986-funding-strategy. Generated 2026-11-25 14:56 UTC by `tools/suite_synthesis.py`. Class C.
This is a written deliverable: it is class C/N, which means the platform can produce it and **cannot
verify it**. Every number above was read in the same process that wrote this file.*

Sources: verified shard manifests, suite_classification.json, live endpoints

Cannot claim: that any statement here is verified by the platform; this document is class C/N
