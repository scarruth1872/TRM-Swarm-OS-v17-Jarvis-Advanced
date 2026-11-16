# Relaxation: the recall dynamics now exist, and they are measured

**Block 141. Companion to `docs/FIXES_DERIVE_20260925.md` and `docs/VALIDATION_PLATFORM_CAPABILITIES_20260925.md`.**

## What was missing

The validation round found that the SRAM layer could *emit* a pattern but could not *recall* one. Three
independent implementations of "the SRAM pattern computer" were present, and none of them completed the loop:

| Tree | What it is | State before this round |
|---|---|---|
| `swarm_v2/core/sram_pattern_computer.py` | pattern image generator + native strip binding | emits bytes; no `relax`/`recall`/`step` |
| `swarm_v2_artifacts/native_voltage_strips/strip_sram_computer.rs` | the DLL the module binds | hashed 3 generic values for 15 architectures (fixed in the derive round) |
| `trm_uefi_kernel/src/sram_pattern_computer.rs` | the real compute kernels — all 15 as LUT functions | **referenced by nothing**; its `sram_lut_hopfield_recall` is a *single synchronous update*: no iteration, no energy, no fixed point |

A single update is not recall. Relaxation is the iteration to a fixed point, and it did not exist anywhere.

## What was added

`trm_uefi_kernel/src/sram_pattern_computer.rs` (the kernel's own source — the home for compute kernels):

- `hopfield_weights(patterns, dim) -> Vec<i8>` — Hebbian storage, zero diagonal, ×`HOPFIELD_GAIN` (8), clamped to i8.
- `hopfield_energy(state, weights, dim) -> i64` — `E = -Σ_ij w_ij s_i s_j`, exact integers (the standard energy up to the ½ factor; no division, so no rounding to disagree about).
- `hopfield_relax(state, weights, dim, max_sweeps) -> Relaxation { state, sweeps, converged, initial_energy, final_energy, energy_trace }` — **asynchronous (Gauss-Seidel) updates, one unit at a time, index order, until a full sweep changes nothing.** Asynchronous updating with a symmetric zero-diagonal matrix cannot increase the energy, which is why the trace comes back for the ruler to assert instead of being claimed here.

The existing one-step `sram_lut_hopfield_recall` is left intact and now documented as what it is (one update).

## One source, not a fourth copy

`swarm_v2_artifacts/native_voltage_strips/strip_sram_relax.rs` references the kernel file by path:

```rust
#[path = "../../trm_uefi_kernel/src/sram_pattern_computer.rs"]
mod sram;
```

`include!` was tried first and rejected it — the file's `//!` header is illegal in that position and
`extern crate alloc` does not scope through it. The path form keeps **one algorithm in one file**: the
strip adds only the C-ABI wrappers, so it cannot drift from the kernel the way the tree's three
independent SRAM implementations drifted from each other.

## The receipts

`scratch/verify_relaxation.py` → `swarm_v2_artifacts/relaxation_verification.json`, **7 checks, 0 failed**:

```
weights agree between the kernel-derived strip and Python -- 4096/4096 i8 weights identical
energy agrees                                              -- rust 1376 == python 1376
relaxation agrees (state, sweeps, convergence, energy)      -- 2 sweeps, converged, E=-31776 both
pattern 0: 13/64 bits corrupted -> recovered=True (2 sweeps, E -10304 -> -32032)
pattern 1: 13/64 bits corrupted -> recovered=True (2 sweeps, E -10016 -> -31552)
pattern 2: 13/64 bits corrupted -> recovered=True (2 sweeps, E -10304 -> -31776)
every corrupted stored pattern relaxes back to exactly that pattern -- 3/3 from 20% corruption
the energy trace is monotone non-increasing and the run converges
control: an unrelated state settles, NOT onto a stored pattern
```

**The control matters.** Relaxation always settles; that alone is not recall. A state with no relation to
the stored set converges too — onto an energy minimum that is *not* one of the stored patterns. Without that
check, "it converges" would read as "it recalls".

Capacity, **measured** (64 neurons, 20% corruption, 3 seeds) rather than quoted:

| stored | recovered |
|---|---|
| 1, 2, 3, 4 | 3/3, 6/6, 9/9, 12/12 — all |
| 5 | 14/15 |
| 6 | 18/18 |
| 8 | 21/24 |

Degradation starts between 4 and 5 stored patterns at N=64 and is graceful after — consistent with the
classical ~0.14N capacity, but the numbers above are what this build does, not what theory promises.

## Honest bounds

- The weights are **i8** with a ×8 Hebbian gain. At larger pattern counts the gain saturates weights and the
  `clamp` in `hopfield_weights` is load-bearing; the capacity table above is for i8, not for i32.
- `strip_sram_relax.dll` (120,832 B) and the kernel source are two views of **one** algorithm, proven by the
  parity check — but only for the inputs exercised: N=64, ±1 states, symmetric zero-diagonal W.
- The relaxation is not yet reachable from a `host_council_server` route. It is reachable from Python
  through the strip, and it is compiled into the kernel's source tree. Wiring a route is the next rung, not
  done here.
