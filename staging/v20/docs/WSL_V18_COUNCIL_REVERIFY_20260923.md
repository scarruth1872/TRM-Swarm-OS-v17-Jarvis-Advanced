# Re-verifying the v18 council claim against the recovered reference (2026-09-23)

With `libswarm_kernel_ops.so` recovered from the v17 tree, the MOTION's §3 claim
("8 vote vectors compared, 0 mismatched") was tested for real.

## The claim reproduces exactly

Parsed from the crate's own `AGENT_MODELS` mirror, the same 8 agents vote on the same motions:

```
Logic        ovA=1  [1.000,0.776,0.776]    Devo   ovA=5  [2.000,0.794,0.794]
Seeker       ovE=6  [2.000,0.699,0.699]    Pulse  ovE=1  [1.000,0.780,0.780]
Flow         ovE=1  [1.000,0.764,0.764]    EventLedger ovE=1 [1.000,0.763,0.763]
Verify       ovA=2  [2.000,0.784,0.784]    Bridge ovE=1  [1.000,0.756,0.756]
```

8 vectors, matching §3's overlaps and vectors, and matching the C kernel on every one.

## The port is faithful

Run through the real C reference (`verify_council_vote_vs_c.py`, WSL + ctypes):

```
C oracle: rc=0, 33 trits for 33 inputs, counts [24, 7, 2] sum-to-n=True
8 MOTION vectors  -> identical (rust == c_kernel == [1,1,1])
probe [ 0.30, 0.30, 0.30] -> [ 0, 0, 0]   (below eps)
probe [-0.50, 0.20,-1.00] -> [-1, 0,-1]   (the negative arm)
probe [-0.35, 0.35, 0.00] -> [ 0, 0, 0]   (exact boundary; strict > and < in both)
VERDICT: PORT FAITHFUL (0 mismatches of 11 rows)
```

## But the claim's gate could not have caught a rule defect

All 8 of the MOTION's vectors lie entirely above EPS=0.35, so the check exercised **only the +1
arm** — one distinct trit-vector, `(1,1,1)`. The arms that decide a verdict (0 and −1) were never
reached by §3; the probes above reach them.

## The v18 rule is structurally degenerate: BLOCK is unreachable

`vote_vector` builds `[ov_sig, conf, conf]` with `ov_sig in {1.0, 2.0}` and `conf = 1 - loss`, where
every `loss` in the table is 0.206..0.301, so **conf is 0.699..0.794**. `collapse` returns +1 for
`v > 0.35`, −1 for `v < -0.35`. The minimum component anywhere in the table is **0.699**, so:

- every voting agent contributes exactly **+3** to the tally;
- the tally is positive whenever at least one agent votes;
- `ANALYZE` occurs only when every agent abstains;
- **`BLOCK` cannot be produced — for any motion, any domain table, any confidence in the table.**

That is a property of the construction, not of the sample: no motion can make this rule block. A
council that cannot block is a rubber stamp, which is precisely what the same MOTION's doctrine warns
against ("a rubber-stamp vote is worthless").

## Scope: the live council does not use this rule

v19's `swarm_v2_rust_kernel/src/council.rs` contains no collapse/verdict scheme (no EPS, no
collapse, no PROCEED arm). The live verdict path is `host_council_deliberation/src/main.rs`, which
compares approvals against the PBFT threshold and carries **both** branches — `CONSENSUS REACHED
(ENACTED)` and `QUORUM FAILED`. So the degeneracy is confined to the v18 crate and does not describe
the running kernel.


## Bonus: the live `pbft_quorum: 10` root cause, found while scoping

Step 4 recorded that the served value 10 disagrees with `2f+1 = 9`. The operative line is in the live
verdict path itself:

```rust
trm_uefi_kernel/host_council_deliberation/src/main.rs
    let weighted_consensus = total_score / (AGENTS.len() as f32);
    let pbft_passed = approve >= 10;
    if pbft_passed { "CONSENSUS REACHED (ENACTED)" } else { "QUORUM FAILED" }
    "pbft_consensus": pbft_passed,
```

So the platform holds **three different numbers for one concept**:

| where | value | meaning |
|---|---|---|
| `pbft_consensus.rs` `QUORUM_THRESHOLD` | 8 | prepare quorum, 2f |
| LCARS UI (`main.rs:3269`) | 9 | commit quorum, 2f+1 |
| live veto (`host_council_deliberation/src/main.rs`) | **10** | hardcoded |

With N=14, `f = (N-1)//3 = 4`, so safety needs >= `2f+1 = 9`. Requiring 10 is **over-conservative, not
unsafe** — it cannot enact with too few approvals, but it can refuse to enact a proposal that the
protocol says has quorum. That is a **liveness** defect, and the direction matters: it is a
stricter-than-required threshold, not an unsafe one. Stated precisely because the difference decides
whether this needs an urgent fix or a correctness fix.
