# K3 Verdict Derivation Standard

A verdict field must be the **Kleene reduction of evidence the module actually produced**.
It may never be a literal.

## Why this exists

`swarm_v2_rust_kernel` carries a real Kleene engine (`k3_logic.rs`) and every pipeline
module published verdicts like `unanimous_council_verdict: K3Trit::True`. The modules are
dispatched from `main.rs`, so the kernel printed "+1 Absolute Approval" for runs whose
inputs were never consulted. Proof, from a mutation run of the zero-day pipeline with one
evidence input flipped to `false`:

```
before wiring:  Unanimous Council Consensus: True (+1 Absolute Approval)   <- and False printed the same caption
after wiring:   Logic Formal Verdict: False ... Unanimous Council Consensus: False
```

## The gate

```
python swarm_v2/core/verify_k3_derivation.py          # writes swarm_v2_artifacts/k3_derivation_audit.json
python swarm_v2/core/verify_k3_derivation.py --check  # exit 1 while any aggregate verdict is a literal
```

An **aggregate** verdict is a field whose name claims a whole-run result
(`unanimous|consensus|verdict|proof|truth`). 45 of them were literals.

## Helpers (already in `swarm_v2_rust_kernel/src/unified_resonance_stream.rs`)

```rust
pub fn k3_consensus_of(evidence: &[K3Trit]) -> K3Trit   // FALSE absorbs, UNKNOWN propagates, empty => Unknown
pub fn k3_trit_of(verified: bool) -> K3Trit
pub fn k3_from_ratio(passed: u32, total: u32) -> K3Trit // total == 0 => Unknown (no evidence is not a pass)
pub fn k3_caption(verdict: K3Trit, basis: &str) -> String // a caption must agree with its verdict
```

## Recipe (verified end to end on `federation_zero_day_defense_engine.rs`)

1. Read the module. Find the aggregate field, then the fields/functions beside it that are
   its evidence.
2. Convert each piece of evidence to a trit and fold: `k3_consensus_of(&[...])`.
3. Change **only the right-hand side**. Never rename, retype or reorder a field.
4. No computable evidence in the module? The verdict is `K3Trit::Unknown` and you add a
   `// BASIS: ...` line saying no measurement exists. Never silently keep a literal.
5. Add the helpers to that module's `use crate::unified_resonance_stream::{...}` line.
6. `cargo check --release` must show no error naming your files (the crate is shared with
   other agents — ignore errors in files you do not own, wait, retry).

## Hard rules

- Only the design becomes derived; no field is deleted, no type changes.
- One owner per store: do not re-implement AND, use `k3_consensus_of`.
- Do not touch a module's fabricated arithmetic or its publication numbers; that is a
  different workstream.
- If evidence is itself an unmeasured literal, the honest verdict is `Unknown`.
