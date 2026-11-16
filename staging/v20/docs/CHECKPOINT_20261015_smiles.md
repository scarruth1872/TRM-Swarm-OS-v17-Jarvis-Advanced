# Checkpoint F — 2026-10-15 (blocks 364–390)

## State at checkpoint

- **Ledger: 390 blocks** (hash-chained monad journal; 0 chain-link breaks).
- **Kernel v20.0.0 healthy on :8022**; supervisor running; dashboard :5183.
- **Council: 31 motions / 31 enacted.**
- **Recall surface: 5 knowledge domains** (fabric + ChEMBL + structured-text + bioassay + clinical) plus a **SMILES canonical crossref** (CAP-201).
- **Rust SMILES canonicalizer** in-kernel (`smiles_canon.rs`, `smiles-canonicalize` CLI) — WIP, sequenced under MOTION-031.

## What changed since checkpoint E

### Recall tiers hardened (blocks 364–371)
- trm7m re-rank tier: stable sort (trit desc → confidence desc → vector_rank; unscored last) — block 366.
- Cross-store rank fusion (Reciprocal Rank Fusion, k=60 + cosine tie-break) over four stores — blocks 368–369.
- Truth filter (`?filter=1` drops trm7m trit −1; unscored kept fail-closed) — block 371.

### Fusion tested + amended (blocks 377–379)
RRF measured **P@1 = 0.625** (5/8) on an 8-query labeled set; the 3 misses are LABEL failures (ambiguous queries), not fusion bugs — cosine evidence (binding/6736 0.467 vs chembl 0.167; fabric 0.272 vs structured 0.175).

### Unsafe surface inventoried (blocks 380–381)
80 → **79 real blocks** (77 irreducible FFI/SIMD/syscall + 2 invariant-dependent Send/Sync), **0 replaceable**; the reduction was 1 grep false positive (`"unsafe_target"` string). Send/Sync invariant tested: 8 threads × 10 = 80 concurrent `k3/retrieve` calls byte-identical.

### SDK / Instella findings (blocks 383–386)
CUDA/Vulkan SDKs absent (finding, not a gap — wgpu+naga needs the driver, not the dev toolkit). No Instella MLA forward exists (finding — the MoE gate is the Instella surface, already parity-verified vs `c_instella_gate_topk`).

### Skill fixed (block 387)
The `trm-swarm-kernel-extension` skill had drifted past its 100,000-char edit limit (101,374); split one section to `references/` → 95,134 chars; verification-discipline lessons landed.

### SMILES canonicalization — CAP-201 (blocks 388–390)
The v3 agenda's item 1 executed. RDKit canonicalization (offline reference, NOT Rust) over 2,897,819 molecule SMILES × 23,941 distinct bioassay canonical SMILES → **11,647 molecules with a canonical match** (delta **+6,792** over the 4,855 exact-string baseline). Artifact `swarm_v2_artifacts/smiles_crossref_canonical.json` (8.6 MB, 531,803 refs); count independently verified (11,647 == 11,647).

### Rust canonicalizer sequenced — MOTION-031 (block 390)
The in-kernel Rust canonicalizer (`smiles_canon.rs`: parser → Morgan canonical labeling → two-pass CANGEN emitter) was built this session but is not correct: **26.1% idempotent** on a 2000-molecule sample, **0.05% RDKit parity**. MOTION-031 sequences it as three milestones, each with a named test:

| milestone | case size | test |
|---|---|---|
| bracket atoms | 76% of failures | curated `[nH]`/`[NH4+]`/`[O-]` set: output matches the reference emission + idempotent on both forms |
| stereochemistry | 66% | enantiomer pairs (`F[C@H](Cl)Br` vs `F[C@@H](Cl)Br`): canonicalize DIFFERENTLY + each idempotent |
| fused-ring labeling | remainder | naphthalene/anthracene/phenanthrene + fused heteroaromatic: idempotent; symmetric systems canonicalize to the same string regardless of input order |

Acceptance is **idempotency** (measured); RDKit parity is a **reference**, not a gate.
