# CHECKPOINT — 2026-10-19: Recall arc closed, exact-ID bug fixed

## Final state (measured, this host)

| facet | state |
|---|---|
| Monad ledger | **436 blocks**, chain intact, tip `0354cf928f317adf` |
| Council | **44/44** motions enacted (MOTION-038..044 all 12/3/0, weighted 0.677) |
| Kernel | Native Rust Sovereign Kernel **v20.0.0 HEALTHY** on :8022 |
| Dashboard | :5183 |
| Module registry | kernel 317 / platform 247 / repos 48; REAL 265, PORTED 39, EXCLUDED 9, SKELETON 4 |
| Native strips | 263/263 REAL_PE, 0 MARKER |
| Recall P@1 | **0.9487** over 234 queries (SE 1.44%, 95% CI 91.3%–97.0%) |

### Recall arc (the full journey, every fix measured before/after)

| store | start (8q) | final (234q) |
|---|---|---|
| clinical | 0 | **1.000** (40/40) |
| fabric | — | **0.944** (117/124) |
| structured | — | **0.933** (14/15) |
| chembl | — | **0.939** (31/33) |
| bioassay | — | **0.909** (20/22) |
| **overall** | **0.692** | **0.9487** |

Five fixes, each named + measured: (1) clinical side-fact → ranked top-1; (2) structured/fabric
provenance relabel; (3) chembl/bioassay drug-name relabel; (4) fabric papers-only provenance boost;
(5) drug-name → ChEMBL id + SMILES via the voltage-doctrine name→SMILES sidecars.

## Class of finding (the exact-ID bug)

**A reader/checker must not assume properties of the data it reads — it must verify them.**

`chembl_lookup`'s exact-ID path *assumed* (asserted in a comment, never checked) that the ChEMBL text
sidecar is sorted by chembl_id string, so its `key > needle` early-exit was trusted. The file is
actually sorted by molregno/ingestion order: `CHEMBL25` (aspirin) sits at line 4247, but the scan hit
`CHEMBL440060` at line 2 (`"CHEMBL440060" > "CHEMBL25"` as a string) and returned `found:false`,
falling through to the vector scan and returning the wrong compound.

The fix does not patch around it — it removes the unsound assumption: a `chembl_id → byte-offset`
index is derived once from the actual file (2,897,805 ids) and the ID path becomes an O(1) hash
lookup + seek + read-one-record. The lookup is then **verified** by direct ID queries
(CHEMBL25 → aspirin `CC(=O)Oc1ccccc1C(=O)O`, CHEMBL941 → imatinib, CHEMBL1223 → penicillin,
CHEMBL153534 → first record; unknown id → vector fallback).

The same class of bug had already bitten the clinical fix (a fact carried as a side field, never a
ranked candidate). The rule: **verify the data's actual shape before writing code that depends on it.**

## Two remaining gaps

1. **chembl SMILES→bioassay overlap** — 2/234 queries (2 SMILES strings) return the *bioassay*
   store's tox21/binding record instead of the chembl record, because both stores legitimately carry
   the same SMILES. Residual, pre-existing, not a regression.

2. **Docking blocked on 3D conformers** — `swarm-docking` cannot run on the 9 leads yet: the ChEMBL
   sidecar provides SMILES/formula/MW/heavy_atoms/atom_count only, not ready-to-dock 3D ligand
   conformers. Honest blocker, not a stub.

## Other open items (not the "two gaps", tracked separately)

- 3 unsealed hypotheses: `toroidal-routing`, `phonon-clock`, `platonic-light-codes`.
- TRM-7M head is a measured dead contraction map (seed-42 weights hold distilled intelligence; needs
  ≥7 forward turns to warm the conv-state — a single pass is not representative).
- `field_qwen.bin` / `field_bonsai.bin` stranded on WSL (`/home/shawn`).
- Main `.k3` NAND trit store header clobbered (evidence preserved).

## Commits this session (scoped)

`ceddacb` · `2277362` · `cbb82d0` · `18628b4` · `d389877` · `a39e981` · `5bf637f` · `7e2c8e1` ·
`0b59fdb` · `4be1673` · `e2b6dbe` · `f890167` · `317a59e` · `38723db` · `0c7be10` · `427e3c0` ·
`54284fb` · `4403802` · `2e5a261` · `6e4943a` · `31640cb` · `bde945b` · `10cf74a` · `19be621`
