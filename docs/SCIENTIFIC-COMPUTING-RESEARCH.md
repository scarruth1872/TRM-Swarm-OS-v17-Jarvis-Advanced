# Scientific computing 421–430 — research before building

Date 2026-11-25. Question asked: **are these already completed?** Answer per task, from a search of the whole tree
(including `staging/`, `archive/`, `target/`, `phoenix`, `Jarvis-Advanced-main`) and the live module registry
(228 modules across 2 trees).

## What already exists — verified in code, not in prose

| capability | where it lives | what it actually does |
|---|---|---|
| SRAM pattern computer | `trm_uefi_kernel/src/sram_pattern_computer.rs` | 15 patterns incl. `pattern_ising_spin_glass` (one simulated-annealing spin sweep), `pattern_hopfield_recall`, `pattern_boltzmann_energy`, `pattern_systolic_matmul`, `pattern_gnn_aggregate`, `pattern_tcam_match`. This is the "SRAM computer". |
| Fourier / convolution bind | `host_council_server/src/sram_fourier.rs` | circular convolution, DFT/IDFT, HRR bind/unbind, cosine — with a documented parity test against the convolution path |
| quantum-inspired drug screen | `trm_uefi_kernel/src/quantum_drug_screen.rs` | deterministic amortised search over a candidate space; labels its output a **scoring hypothesis, never a Kd** |
| genomics | `trm_uefi_kernel/src/continuum_genomics.rs` | **stub only** — a singleton handle, no genetics |
| AMR surveillance | `host_council_server/src/amr_surveillance.rs` | one function, `deployment_report()`; no epidemic model |
| neural | `mamba3_turbovec_ssm.rs`, `neural_wall.rs` | SSM inference, not spike-train analysis |
| docking / pose | referenced in benchmark JSONs and prose only | **no docking engine in Rust** |

**Absent entirely (zero hits for the domain vocabulary in any `.rs`):** molecular dynamics, Verlet/Lennard-Jones,
Hartree-Fock/DFT/orbitals, protein folding, pharmacokinetics/ADME, SIR/SEIR epidemics, Hardy-Weinberg/Wright-Fisher,
systems-biology ODEs, Hodgkin-Huxley/spike trains.

So: **none of 421–430 are completed.** Two have adjacent partial assets (Ising annealing; a drug-screen scorer),
and one has a stub that looks like a module but does nothing (`continuum_genomics`).

## Data on disk that can support them

| dataset | size | use |
|---|---|---|
| `datasets/pdbbind_labels/pdbbind2019_labels.json` | **4,463 complexes** with pKd (2.00–11.92, median 6.51) + pocket PDB + ligand SDF paths | 424 |
| `datasets/pdbbind/v2020-refined_*.tar.gz` | 6 archives | 424 structures |
| `datasets/smpdb_proteins.csv/` | 89 MB of per-pathway protein lists | 428 (system topology) |
| `datasets/ncbi_amr/ReferenceGeneCatalog.txt` | AMR gene catalogue | 426 (parameterisation) |
| `datasets/clinicaltrials/AllPublicXML.zip` | 612,025 records | already used by 531–540 |

## What each task can honestly be

| task | verdict | what it will be |
|---|---|---|
| 421 MD engine | **BUILDABLE** | velocity Verlet + Lennard-Jones; validated by energy conservation against a harmonic reference |
| 422 quantum chemistry | **BUILDABLE, NOT DFT** | Hückel MO / Jacobi eigensolver on π systems; benzene eigenvalues (−2,−1,−1,1,1,2)β are the reference. No DFT, no SCF, and the report says so |
| 423 protein folding | **BUILDABLE (model)** | 2D HP lattice self-avoiding fold, annealed **through the SRAM Ising engine**; validated against exhaustive ground state on short sequences |
| 424 binding affinity | **BUILDABLE (scoring only)** | LJ + Coulomb + desolvation contact score; validated on the analytic LJ well. **No correlation claim** without extracting structures — 4,463 labels exist |
| 425 pharmacokinetics | **BUILDABLE** | 1- and 2-compartment models; analytic solutions as the reference |
| 426 epidemiology | **BUILDABLE** | SIR via RK4; final size checked against the transcendental equation by bisection; R0 = β/γ |
| 427 population genetics | **BUILDABLE** | Wright–Fisher neutral drift; fixation probability must equal p0, measured over replicates |
| 428 systems biology | **BUILDABLE** | RK4 on Lotka–Volterra; conserved quantity drift is the check |
| 429 spike trains | **BUILDABLE** | ISI, CV, Fano factor; Poisson → 1.0, refractory → CV < 1 |
| 430 suite | **BUILDABLE** | index over the nine, with the blocked claims named |

## What is refused

- **DFT / ab initio quantum chemistry**: no basis-set library, no SCF, no correlation functional. Hückel is a
  π-electron model and will be labelled as one.
- **A folding model that predicts real structures**: the HP lattice is a toy energy model. It will be validated as
  a *search* and never described as structure prediction for a real sequence.
- **A binding-affinity correlation claim** without extracting and scoring real complexes — the labels exist, the
  structures are still in tarballs, and "docking is not evidence" already cost this platform a false result once
  (receptor/ligand frame mismatch, RMSD 13.82 Å).


---

# Built and measured (2026-11-25)

`trm_uefi_kernel/src/scientific_computing.rs` (no_std) + `host_council_server/src/sci_routes.rs` (9 endpoints +
suite). **21 of 21 independent checks pass**, and **15 kernel tests** assert against references that do not come
from the implementation.

| task | endpoint | reference it is checked against | measured |
|---|---|---|---|
| **421** MD | `/sci/md` | energy conservation under a symplectic integrator | **relative drift 2.86e-7** over 800 steps |
| **422** Hückel | `/sci/huckel` | published benzene spectrum | eigenvalues **−2, −1, −1, 1, 1, 2** to 1e-15; E = **−8 β**; butadiene **−4.4721 β** |
| **423** folding | `/sci/fold` | exhaustive DFS ground state | exhaustive complete, **annealer −4 = optimum −4** (never better) |
| **424** affinity | `/sci/affinity` | analytic LJ well and Coulomb law | LJ pair **−0.15 = −ε** at 2^(1/6)σ; Coulomb **33.2064** vs 33.2064 |
| **425** PK | `/sci/pk` | closed form | C(0)=10, C(t½)=**5.0000**, t½=**6.9315** |
| **426** epidemiology | `/sci/sir` | transcendental final size, re-solved by bisection in Python | R0=2.5, final size **0.89256 vs 0.89266** independent |
| **427** popgen | `/sci/drift` | neutral theory: fixation = p0 | **0.28 measured vs 0.30 expected** over 300 replicates |
| **428** systems bio | `/sci/lotka` | conserved quantity H | **drift 2.7e-12** over 5,000 steps |
| **429** spikes | `/sci/spikes` | Poisson CV = Fano = 1 | CV **1.014**, Fano **1.267**; refractory CV **0.571** |
| **430** suite | `/sci/suite` | 10 entries, 3 refusals named | PASS |

## Bugs the references caught in my own code

1. **`exp` was 1.2e-4 wrong** — `(1+x/4096)^4096` has truncation error x²/2n, so `exp(1)` returned 2.71795 instead
   of 2.71828. Replaced with range reduction + Taylor; the half-life test then passed at 1e-9.
2. **SIR final size returned 0 for every epidemic** — the bisection guard tested `f(0)`, which is always negative
   for a seeded outbreak; the existence condition is `f(1) > 0`.
3. **Bisection ran in the wrong direction** — `f` is negative *below* the root, so a positive value must move the
   upper bound down. A root-finder that silently returns an endpoint looks like it works.
4. **One test reference was wrong, not the code** — I asserted `cos(2π)` for a run that ends at t = 6.28. The
   failure message (`y0 = 0.9999949` vs expected `0.9999986`) is what made that visible; the integrator was right.

## What is refused, in the API not just the docs

- **422 is Hückel π-electron theory.** No basis set, no SCF, no correlation functional. `/sci/huckel` returns
  `not_claimed` saying exactly that.
- **423 is the HP lattice toy model.** `/sci/fold` refuses sequences longer than 12 residues rather than report an
  annealer result as a ground state.
- **424 has no affinity correlation.** 4,463 PDBbind complexes with pKd exist in `datasets/`, but they were not
  extracted or scored — and docking already produced one false result here (frame mismatch, RMSD 13.82 Å). The
  endpoint states this in `not_claimed`.

## On the SRAM computer

The Ising spin-glass pattern (`sram_pattern_computer::pattern_ising_spin_glass`) is exercised by
`ising_relax_sram`, and the test requires it to reach the exact ferromagnetic ground state **−ΣJ = −8** on an
8-spin chain. Lattice folding uses Metropolis instead of being forced through that pattern: the SRAM pattern is a
1-D nearest-neighbour sweep, and mapping a 2-D self-avoiding walk onto it would be decorative rather than correct.
