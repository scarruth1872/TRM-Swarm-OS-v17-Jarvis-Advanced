# phoenix_k3_analysis

Analyses PHOENIX docking results with the platform's OWN primitives — the same
code the bare-metal kernel ships, compiled for the host:

| primitive | source | how it is used |
|---|---|---|
| K3 Kleene ternary logic | `trm_uefi_kernel/src/k3_logic.rs` | `quick_reduction` then `bypass_decision` over each docking score, with the real thresholds `tau_upper=0.35` / `tau_lower=-0.35` |
| TurboVec 1.58-bit quantizer | `trm_uefi_kernel/src/turbovec.rs` | `quantize_158` packs each `[affinity, ligand_efficiency]` vector to 1 byte (16x); `fast_femto_cosine_similarity` scores the round trip |
| Voltage doctrine | `hardware_voltage_microkernel_binder` | `phi_v` is a REQUIRED argument, supplied from a live measurement. With no argument the program refuses to run. |

## Build and run

```
rustc --edition 2021 -O -o phx_k3.exe main.rs
./phx_k3.exe <measured_phi_v> "NAME,UNIPROT,AFFINITY,LIG_EFF" ...
```

## What the trits mean

`+1` BIND · `0` UNKNOWN · `-1` REJECT — a ternary classification **of a docking
score**, not of binding truth. One vina pose against one rigid AlphaFold
conformation is weak evidence; K3 exists here to fold several such predictions
into a single triage state a pipeline can act on, and to say UNKNOWN loudly when
the evidence does not reach a threshold.

## Verified

Measured `phi_v = 0.651` at 4000 MHz (`REAL_HARDWARE_BOUND`). Sample run:

```
TYK2   affinity -10.13  norm 0.8442  trit +1  BIND
C1QA   affinity  -4.20  norm 0.3500  trit  0  UNKNOWN
BLK    affinity  -1.10  norm 0.0917  trit  0  UNKNOWN
```

Note C1QA landing exactly on `tau_upper` returns UNKNOWN rather than BIND — the
engine does not round a marginal result up.

## Why phi_v is mandatory

The voltage doctrine holds that the multiplier must be **measured, never a
constant**. An earlier version of the platform hardcoded `0.9115`. Making the
argument required means a run without a real measurement cannot produce a result
at all — the refusal is the enforcement.
