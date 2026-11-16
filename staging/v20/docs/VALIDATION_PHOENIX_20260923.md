# Validation: PHOENIX Tolerance Engine report vs the live tree (2026-09-23)

The document is the prose twin of **`swarm_v2_artifacts/PHOENIX_TOLERANCE_ENGINE_REPORT.json`** (timestamped
2026-08-16, 1,051 B), and its metrics match that artifact exactly: 20/0/20/0, 100% detection, 0% false
positives, 5,790 µs, 172.7 threats/s, `hardcoded_flag: False`. So the document is not inventing its numbers —
it is faithfully reporting its artifact. What follows is whether the artifact's claims hold against the
machine.

## Validated

| claim | evidence |
|---|---|
| Three phases — Ares Breach / Athena Shield / Hera Watch | all three present in `trm_uefi_kernel/src/phoenix_protocol_engine.rs` (Ares ×15, Athena ×15, Hera ×17) |
| `c_instella_gate_topk` | defined in `swarm_v2/core/swarm_kernel_ops.cpp` + `.h`, and **exported by the built library** (checked with ctypes) |
| `nand_k3_fabric.append_decision(...)` | `append_decision` resolves in 9 files, including `swarm_v2_rust_kernel/src/nand_persistence.rs` |
| the engine exists | `phoenix_protocol_engine.rs` is 9.8 KB of real code, not a stub |
| the retired-claims *intent* | naming a prior claim as "unsourced, retired" rather than quietly dropping it is the right instinct |

## Refuted

**1. The three "removed" constants are still in the live tree.** The report says the 96.8% SLE figure was
retired, 94.8% BBB penetration was "removed", and the Cas13 window was "removed". They are live:

```
trm_uefi_kernel/src/phoenix_protocol_engine.rs:20   pub const PHOENIX_PREDICTED_CURE_RATE_PCT: f64 = 96.8;
trm_uefi_kernel/src/phoenix_protocol_engine.rs:175      bbb_penetration_rate_pct: 94.8,
trm_uefi_kernel/src/phoenix_protocol_engine.rs:181  /// Phase II: The Athena Shield — LNP-CRISPR-Cas13 mRNA silencing.
trm_uefi_kernel/src/jarvis_answers.rs:77            "* CLINICAL: 96.8% predicted drug-free remission | 94.8% BBB delivery to CNS."
```

The first two are inside **the PHOENIX engine the report says retired them**. The third is the Jarvis answer
surface — i.e. the platform still states the retired clinical percentages to a user as `CLINICAL`. (`main.rs:3342`
also contains `94.8`, but there it is an AlphaFold pLDDT readout — a coincidence of the number, not an instance.)

**2. Three of the four native ops the integration table names do not exist.**

| op named in the report | in the tree | exported by the built library |
|---|---|---|
| `c_instella_gate_topk` | yes (6 files) | **True** |
| `c_kleene_quantize_avx2` | **0 files** | False |
| `c_k3_kleene_ops` | **0 files** | False |
| `c_turbovec_quantize_2bit` | **0 files** | False |

**3. `libswarm_kernel_ops.so` — the substrate the report names twice — is not present.** The build script
`swarm_v2/core/build_kernel_ops.sh` exists and would produce it (`g++ -mavx2 -mfma … -o libswarm_kernel_ops.so`),
but no `libswarm*.so` exists anywhere under `/home/shawn` in WSL (searched to depth 8, excluding `target/` and
`.git/`), and the artifact actually built on this machine is **`swarm_v2/core/libswarm_kernel_ops.dll`** — a
Windows DLL, 392 KB. So on this host the ops are a DLL, not the `.so` the report describes.

**4. The WSL tree contains no PHOENIX engine.** `/home/shawn` has `phoenix*` files, but they belong to
`Agent-Q-master/` — the *simulator* (`phoenix_simulation.json`, `phoenix_lab.py`), i.e. the class of artifact
this report says it superseded. `append_decision` has **zero** hits in the WSL home.

## Precision notes

- **`hardcoded_flag: False` is itself a hardcoded literal** in the JSON — a self-declaration, not a computed
  check. It is the same class the platform's own `measurement_audit.json` flags (557 D1 "hardcoded measurement
  literal" hits across 106 files).
- **172.7 threats/s is `1 / 5.79 ms` derived**, not an independently measured rate: 1/0.00579 = 172.7 exactly.
  The total work measured was 40 × 5,790 µs = **231.6 ms**, so this is an inverse mean latency, not sustained
  throughput.
- The artifact is **1,051 bytes with no per-sample data** — the 20 malicious / 20 benign corpus is described but
  not recorded, so the 100% detection rate is not independently reproducible from what shipped.
- 100% detection with 0% false positives on a 40-sample corpus the detector's own lexicon was written for is an
  expected result, not evidence of generalization. The document itself asks for a larger corpus in §5, which is
  the honest framing.

## Verdict

As a **measured code/threat tolerance result**: credible and internally consistent, and its refusal to extend
the immune metaphor into clinical efficacy claims is correct.

As a **description of the system**: the metrics hold, the integration table is about half aspirational (3 of 4
named ops absent, the named `.so` absent in favour of a `.dll`), and its honesty section is contradicted by the
live tree — the constants it says were removed are still there, one of them inside the very engine that claims
the removal, and one of them still surfaced to users as `CLINICAL`.

Scope of the negatives: "absent" means absent from the paths searched — `swarm_v2/`, `trm_uefi_kernel/`,
`swarm_v2_rust_kernel/` on Windows, and `/home/shawn` (depth 8) in WSL. A `.so` built and later deleted in WSL,
or held on another drive, would not appear.
