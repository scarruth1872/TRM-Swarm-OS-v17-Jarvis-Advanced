# P3 — the docking class, reproduced from a scripted path (2026-09-25)

**Acceptance (from `FULL_PLAN_ARTIFACTS_AND_SETUP_20260924.md`):** "the n=11 redocking result is
reproduced from a scripted path with the `swarm-docking` crate, with the 4 failing complexes recorded."

**Result: reproduced exactly.** All eleven complexes re-ran to the same RMSD and the same top affinity
as the recorded study (11/11 match to 1e-4 Å), and every pose was scored by the platform's own engine.

## Protocol (taken from the record, not re-chosen)

`scratch/task_001_docking_redock_FINAL.json`: box 24 Å cube centred on the crystal-ligand centroid;
exhaustiveness 16; 9 modes; seed 7; redock; compare the top pose to the crystal pose with `obrms`
(symmetry-aware). Engine: AutoDock Vina 1.2.7 in WSL. Preparation: `obabel protein.pdb -O rec.pdbqt -xr -h`
and `obabel ligand.sdf -O {lig,xtal}.pdbqt -h`.

Scripted path: `scratch/p3_redock_repro.py` (search, in WSL where vina is) → `scratch/score_p3_poses.py`
(the engine scores each pose, on this side where the crate builds — WSL has no cargo).

## The result

| pdb | heavy | Vina mode-1 kcal/mol | redock RMSD Å | | recorded RMSD | match |
|---|---|---|---|---|---|---|
| 6M2N | 20 | -7.310 | 4.31469 | fail | 4.31469 | ✅ |
| 6W63 | 34 | -8.358 | 6.22324 | fail | 6.22324 | ✅ |
| 7AP6 | 21 | -6.941 | 7.58219 | fail | 7.58219 | ✅ |
| 7EN8 | 34 | -9.197 | 0.441106 | PASS | 0.441106 | ✅ |
| 7EN9 | 42 | -10.490 | 0.450871 | PASS | 0.450871 | ✅ |
| 7JU7 | 36 | -8.370 | 1.40553 | PASS | 1.40553 | ✅ |
| 7KX5 | 38 | -8.971 | 5.85532 | fail | 5.85532 | ✅ |
| 7L0D | 32 | -7.859 | 0.915889 | PASS | 0.915889 | ✅ |
| 7L10 | 29 | -7.722 | 0.528105 | PASS | 0.528105 | ✅ |
| 7L11 | 32 | -8.751 | 1.69945 | PASS | 1.69945 | ✅ |
| 7L12 | 36 | -10.070 | 1.36520 | PASS | 1.36520 | ✅ |

**n=11 · 7 pass under 2.0 Å = 64% · median 1.40553 Å · mean 2.7983 Å.** The four failures are
**6M2N (4.31 Å), 6W63 (6.22 Å), 7AP6 (7.58 Å), 7KX5 (5.86 Å)** — reported, not hidden.

## What had to be fixed to get there

1. **PDBQT's last column is an AutoDock atom type, not an element** (`A` = aromatic carbon, `OA` =
   oxygen, `HD` = hydrogen). Reading it as an element manufactured atoms named "A", which have no vdw
   radius, and counted hydrogens as heavy atoms.
2. **The box centroid is part of the protocol.** The recorded study centred on *every* crystal-ligand
   atom including hydrogens; centring on heavy atoms alone moved the search (4.3351 Å vs the recorded
   4.31469 Å). With the producer's definition the reproduction is bit-exact — so the earlier 0.02 Å
   difference was a producer choice, not nondeterminism. Vina here is deterministic: three repeat runs
   and a `--cpu 1` run all returned identical numbers.
3. **A receptor atom set built as *pairs* double-counts atoms** (once per ligand contact). That scored
   one pose at −141 kcal/mol against Vina's −7.31.
4. **`swarm-docking`'s H-bond ramp was wrong** (a real defect in the crate). Its `-(d - 0.7)/0.5`
   saturated at 1 from d = −0.2 and still paid 0.4 at d = +0.5, so pairs nowhere near touching earned a
   full −0.587 bonus. Vina's published term ramps 0 at contact to 1 at d ≤ −0.7 and pays nothing beyond
   contact. Fixed in `swarm-docking/src/lib.rs`; on 6M2N that alone moved the engine from −11.16 to
   −7.97 against Vina's −7.31.

Proved by oracle, not by eye: `scratch/verify_docking_scoring.py` compares the engine's pair score with
Vina's published terms — **19/19 pair cases agree**, and whole-pose totals agree with the formula applied
over the same atom set (−7.970908 published vs −7.970907 engine).

## Recorded, still open: engine vs Vina's reported affinity

The crate implements the *published* function; Vina's *binary* reports a less negative number for the
same pose — mean **2.32 kcal/mol**, range **0.66–3.45** over the eleven complexes
(`scratch/p3_crate_vs_vina_fidelity.json`).

The obvious explanation was tested and **falsified**: Vina scores explicit polar hydrogens and the engine
is heavy-atom only, but including hydrogens moves the mean gap from 2.32 to **3.59** kcal/mol — the wrong
way on two of three complexes checked. The remaining difference is between Vina's published function and
Vina's own implementation (its typing/radius tables and how it evaluates H-bonds over donor-hydrogen
pairs). Not closed here; recorded as an open fidelity gap.

Also recorded: the engine models C/N/O/S/Cl/F/Zn only. 7EN8 and 7EN9 carry a bromine, which it refuses by
name rather than coercing; those two poses are scored with the bromine excluded and the exclusion counted
in the receipt (`ligand_atoms_unmodelled`).

## Not claimed

Not that the engine reproduces Vina's numbers, and not that 64% generalises beyond this eleven. What is
claimed is narrow and measured: this study reproduces from a scripted path, the platform's engine scores
every pose, and its relationship to Vina's reported affinity is stated with its size.
