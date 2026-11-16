# MOTION A — RUST COUNCIL ABSORPTION: Verify-Gated Native Port of K3 Council-Vote Logic

**Status:** ABSORBED — verify gate PASSED (byte-identical vs `libswarm_kernel_ops.so`)
**Date:** 2026-08-14
**Doctrine:** native > bridged; prove byte-identical against the live C reference before declaring done

---

## 1. What was absorbed

The agent council-vote path that previously lived in the Python bridge
(`swarm_v2/core/agent_intelligence_bridge.py` — `AGENT_MODELS`) is now a native
Rust crate: **`swarm-council`** (workspace member of `/home/shawn/trm-rust`).

Pipeline ported:

```
motion domain-set M
  -> per-agent overlap = |agent.domains intersect M|        (exact token match)
  -> overlap == 0  => ABSTAIN
  -> else vote vector v = [ov_sig, conf, conf]
       ov_sig = 2.0 if overlap >= 2 else 1.0 if overlap == 1
  -> K3 collapse of v at eps = 0.35 (c_kleene_quantize_avx2 semantics)
  -> council tally: majority of collapsed trits
       sum > 0  => PROCEED
       sum < 0  => BLOCK
       sum == 0 => ANALYZE (tie)
```

The collapse itself reuses the already-verified `swarm-k3::collapse`
(no duplicated kernel logic), so the only new surface is the council semantics:
overlap, vote-vector construction, abstention, and the tally.

## 2. Data provenance

- **Domains:** mirrored verbatim from `AGENT_MODELS` in
  `swarm_v2/core/agent_intelligence_bridge.py` (14 agents, read from the file).
- **Confidence:** the Python bridge stores per-agent `loss` (~0.206–0.301), not
  confidence. Council doctrine assumes confidence ~0.7, so this port derives
  `confidence = 1.0 - loss` (0.699–0.794 — matches the ~0.7 band). This is the
  one **documented assumption**; if the live council uses a different confidence
  source it must be pointed at the static table in `swarm-council/src/lib.rs`.
- **Motions:** last meeting's sets
  `A = {code, engineering, software, compiler, test, verify, kernel}` and
  `E = {research, literature, drug, chemistry, biology, discovery, clinical}`.

## 3. Measured verification output (real kernel, real inputs)

Gate binary: `swarm-council/src/bin/verify_council.rs`
Reference: `c_kleene_quantize_avx2` in `libswarm_kernel_ops.so` (loaded via
`libloading` FFI — same .so the Python bridge calls).

```
=== VERIFY GATE: swarm-council vs libswarm_kernel_ops.so ===
kernel: .../swarm_v2/core/libswarm_kernel_ops.so
eps: 0.35 | agents: 14 | motions: A(7) E(7)

[1/3] Per-agent vote vectors (14 agents x 2 motions):
  agent        ovA   ovE    vec                rust_trits  c_trits   identical
  Archi            0     0
  Shield           0     0
  Logic            1     0  [1.0,0.776,0.776]  [1, 1, 1]  [1, 1, 1]  identical
  Devo             5     0  [2.0,0.794,0.794]  [1, 1, 1]  [1, 1, 1]  identical
  Seeker           0     6  [2.0,0.699,0.699]  [1, 1, 1]  [1, 1, 1]  identical
  Jarvis           0     0
  Pulse            0     1  [1.0,0.780,0.780]  [1, 1, 1]  [1, 1, 1]  identical
  Flow             0     1  [1.0,0.764,0.764]  [1, 1, 1]  [1, 1, 1]  identical
  EventLedger      0     1  [1.0,0.763,0.763]  [1, 1, 1]  [1, 1, 1]  identical
  Orchestra        0     0
  THzGateway       0     0
  Verify           2     0  [2.0,0.784,0.784]  [1, 1, 1]  [1, 1, 1]  identical
  Bridge           0     1  [1.0,0.756,0.756]  [1, 1, 1]  [1, 1, 1]  identical
  Hermes           0     0
  -> 8 vote vectors compared, 0 mismatched

[2/3] Council verdicts, Rust tally vs C-kernel tally:
  Motion A: rust=PROCEED c_kernel=PROCEED -> AGREE
  Motion E: rust=PROCEED c_kernel=PROCEED -> AGREE

[3/3] 1M-value LCG sweep (seed 42):
  values=1,000,000 identical=true
  C counts: pass=0 analyze=350385 block=649615
  Rust sum: -649615

VERIFY GATE: PASSED - byte-identical to libswarm_kernel_ops.so on all council vote vectors and the 1M sweep
```

Exit code: **0**. Unit tests: **4/4 passed** (`cargo test -p swarm-council`).

## 4. Vote ledger (both motions)

Motion A — voting agents: **Logic** (ov 1, [1.0,.776,.776]), **Devo** (ov 5,
[2.0,.794,.794]), **Verify** (ov 2, [2.0,.784,.784]); 11 abstain.
Tally: 9 trits all +1 -> **PROCEED**.

Motion E — voting agents: **Seeker** (ov 6, [2.0,.699,.699]), **Pulse** (ov 1,
[1.0,.780,.780]), **Flow** (ov 1, [1.0,.764,.764]), **EventLedger** (ov 1,
[1.0,.763,.763]), **Bridge** (ov 1, [1.0,.756,.756]); 9 abstain.
Tally: 15 trits all +1 -> **PROCEED**.

## 5. Honest findings

1. **Byte-identical verified** — all 8 real vote vectors collapse identically
   in Rust and C; the 1M-value LCG sweep is also byte-identical. The port
   satisfies the absorption doctrine.
2. **Council behavior is degenerate for this data.** With ov_sig >= 1.0 and
   conf ~0.7, every element of every vote vector exceeds eps=0.35, so every
   vote collapses to `[+1, +1, +1]` and the tally is always unanimous PROCEED
   for any agent with >= 1 domain overlap. A BLOCK or ANALYZE verdict is
   unreachable with the current domain table + confidence band (it would
   require conf <= 0.35). This is a property of the *inputs*, not the port —
   the collapse is verified identical either way.
3. **Confidence is derived, not stored** (see section 2) — the bridge records
   loss only. If the real council injects different confidences, update
   `AGENT_MODELS` in `swarm-council/src/lib.rs`; the verify gate will then
   re-prove identity on the new vectors.
4. **swarm-boot build failure is pre-existing and unrelated** — it is a no_std
   crate (`unwinding panics are not supported without std` under the host
   target); all other 10 workspace crates build clean with the new member.

## 6. Files

- New crate: `swarm-council/Cargo.toml`, `swarm-council/src/lib.rs` (semantics
  + 14-agent static table), `swarm-council/src/bin/verify_council.rs` (FFI gate)
- Workspace: `Cargo.toml` — added `swarm-council` to members
- This report: `research/MOTION_A_RUST_COUNCIL_ABSORPTION.md`
