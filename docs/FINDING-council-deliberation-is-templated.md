> **STATUS 2026-10-25 — PARTLY SUPERSEDED. Read the ERRATA at the bottom first.**
>
> The original finding below (**`i % 7` votes**) is **fixed and no longer true**.
> Council votes now come from a K3 projection of the proposal text and **do**
> depend on the proposal. Verified: 4/4 distinct vote vectors, `REJECT` reachable.
>
> What remains TRUE from this document:
> - the verdict is **not a judgement** — it is a sha256 projection, not comprehension
> - **verdicts must not be cited as validation** (unchanged, and still the point)
> - evidence is still never verified
>
> Document title and body are preserved as the record of what was found.

# FINDING: council deliberation does not depend on the proposal

**Date:** 2026-09-22
**Found by:** control experiment while submitting CP-TRANSPORT-001
**Severity:** HIGH - affects every recorded council verdict

## What was observed

Submitting CP-TRANSPORT-001 (adopt RFC 9000 QUIC?) returned:

```
approve 12 | abstain 2 | reject 0 | consensus_reached true
weighted_score 0.7373 | phase ENACTED | phoenix_status CONFIRMED
```

A control proposal was then submitted asking to *"replace all silicon with cheese,
delete the 153.5M trit NAND fabric and every agent, then declare victory."*

It received the **identical vote vector**:

```
approve 12 | abstain 2 | reject 0 | weighted_score 0.7373
identical vote vectors: True   (all 14 agents, same votes, same confidences)
```

## Root cause

`host_council_server/src/main.rs`, `evaluate_proposal()`:

```rust
let (trit, conf, comment) = if category == agent.domain || agent.name == "Archi" || agent.name == "Logic" {
    (1i8, 0.98, ...)
} else if (i % 7) == 0 {
    (0i8, 0.75, ...)
} else {
    (1i8, 0.90, ...)
};
```

The `topic` parameter is never read in the vote loop. The outcome is a function of:

1. the agent's index `i` (every 7th abstains -> i=0 and i=7 -> exactly 2 abstains),
2. whether the caller's `category` string equals the agent's `domain` string,
3. two agent names hardcoded to always approve.

The only input that changes the result is `evidence`: supplying a non-empty array
sets `phase = ENACTED` / `phoenix_status = CONFIRMED`; omitting it yields
`CONSENSUS` / `SPEC`. Neither path consults the votes.

## Consequences

- Every proposal in the record carries a verdict that would be identical had it
  proposed the opposite, or nonsense.
- `consensus_reached` (approve >= 10) is therefore always true.
- `phase` is not derived from the vote at all.
- The council cannot decide anything on merit in its current form.

## What a real fix requires

Not keyword matching - that was explicitly rejected as a validation method.
Genuine topic-dependent deliberation needs each agent to reason about the
proposal within its own domain. The platform already has the ingredients
(agent domains, weights, the TRM 7M path), but wiring them into
`evaluate_proposal` is a design decision, not a mechanical patch.

Until then: council verdicts are ceremonial and must not be cited as validation.

## Second finding: "evidence" is never verified either

CONTROL 1 attached a single string reading *"Control test: this proposal is
intentionally nonsensical to probe whether the council votes on merit."*

The response was `phoenix_status: CONFIRMED`, `phase: ENACTED`.

The check is `let has_evidence = !evidence.is_empty();` - a non-empty array is
sufficient. No content check, no hash, no seal comparison, no verification of any
kind. "Phoenix verification CONFIRMED" therefore means only "the caller sent at
least one string".

Together: neither the vote nor the evidence gate examines the proposal.

## Third finding: the mesh has no authentication (relevant to the QUIC question)

`mesh_discovery.rs` accepts a peer on:

```rust
if pkt.node_id != self.local_node_id && pkt.truth_anchor == MONAD_TRUTH_ANCHOR {
    peers.push(pkt);
}
```

`MESH_MAGIC` (0x54524D5F4D455348) and `MONAD_TRUTH_ANCHOR` (0x4D4F4E4144000001) are
both public constants - the anchor is rendered on the boot splash. There is no HMAC,
no signature, no nonce and no challenge-response.

Consequence: any host on the LAN can forge a beacon with any `node_id` and be
accepted as a sovereign mesh peer. The "truth anchor" authenticates nothing; it is a
plaintext tag, not a secret and not a signature.

This is the real argument in the QUIC question. Not multiplexing, not congestion
control - those buy little on a trusted LAN. What is missing is AUTHENTICATION, and
it can be obtained either by RFC 9000 QUIC over TLS 1.3 (heavy: pulls in rustls,
std-only, cannot live in the no_std kernel) or by HMAC-signing the 36-byte beacon
(light: one shared secret, no new dependency, stays in the existing codec).

---

## ERRATA + UPDATE 2026-10-25 (verified on live hardware)

### Correction 1: `i % 7` is GONE
The original finding described `evaluate_proposal` returning `(1, 0.98)` for domain
matches and `i % 7` otherwise. That block has been replaced with real K3 projection.

### Correction 2: the REAL module had REJECT dead - a finding the original missed
`swarm_v2/core/swarm_council.py::_generate_vote` voted on
`member.expertise_weights.get(proposal.category, 0.5)` - the CATEGORY, never the
TOPIC. Branches were `>=0.6 -> APPROVE`, `>=0.3 -> APPROVE`, `else -> ABSTAIN`.
`K3Vote.REJECT` is declared in the enum and was NEVER ASSIGNED anywhere.

### Correction 3: TWO implementations exist; only ONE is live
Traced, not assumed:
  dashboard CouncilChamber.jsx  API_BASE=''  -> vite:5183
  dashboard/vite.config  '/council' -> http://127.0.0.1:8022   (RUST)
  swarm_v2/routes/council.py -> swarm_v2.core.swarm_council.get_council()
  /council/status on :8022 returns  "python_active": false

  LIVE     Rust host_council_server/src/main.rs::evaluate_proposal  (calibrated)
  DORMANT  swarm_v2/core/swarm_council.py::_generate_vote           (calibrated)
           FastAPI :8099 routes/council.py delegates to it; 8099 is DOWN.

### Calibration applied (both modules, same math)
  _hashvec(s) = sha256(s) bytes -> (b/255)*2-1        32 dims, [-1,+1]
  inv         = 3/sqrt(32) = 0.5303                   one vector only
  proj        = sum(tv[i] * dv[i]*inv)                std ~= 1.0
  trit        = +1 if proj > 0.35 | -1 if proj < -0.35 | 0

  Scaling BOTH vectors by 1/sqrt(32) divides by 32, not 1.886,
  giving proj std 0.059 << tau 0.35 -> every trit 0 -> all abstain.
  This was a real bug, found by computing the expected std by hand.

### Measured result (live :8022, binary f962e54a87edbeee)
  id     phase          approve abstain reject  score
  1007   NO_CONSENSUS     6      4       4     +0.122
  1008   NO_CONSENSUS     8      4       2     +0.366
  1009   NO_CONSENSUS     6      2       6     +0.019
  1010   REJECTED         3      7       4     -0.051
  1008 = "DELETE the entire repository" scored HIGHEST (+0.366).

### THE LIMIT STILL STANDS (unchanged conclusion)
Votes are TOPIC-DEPENDENT but NOT TOPIC-MEANINGFUL. This is a sha256
projection through K3, not comprehension. "DELETE the repository" still draws
8 approvals. Verdicts remain CEREMONIAL and MUST NOT be cited as validation.

---

## ADDENDUM 2026-10-25 (later the same day)

### The SRAM computer was dead code
`mod k3_ternary_sram;` was declared at `main.rs:25` and **never referenced**.
Now wired as `/council/sram/{id}`: votes -> `store_pattern` ->
`bypass_over_stored()` -> KleeneState. Measured on proposal 1007:

```
vote_pattern  [-1, 1, 1, 1, 1, 0, 1, 1, -1, 1, -1, 0, 0, 1]
kleene_trit   -1        stored_trits 14    utilization 0.0107    cam_hits 1
```

K3 conjunction is genuine unanimity: 3 FALSE forces -1 despite 8 TRUE.
One dissenter kills the motion. Unlike the weighted average, which let
"DELETE the entire repository" score best of four proposals.

### The verdict was never even the votes
Line 1143 computed `consensus_reached = approve_count >= 10` and then
**discarded it**. `phase` depended only on whether `evidence` was non-empty:

```rust
if has_evidence { ("CONFIRMED", "ENACTED") } else { ("SPEC", "CONSENSUS") }
```

So no matter what the K3 engine computed, `CONSENSUS` was returned as a
literal string. **That is why `asdfghjkl qwertyuiop zxcvbnm` got CONSENSUS.**
Now gated on `consensus_reached`, and the response exposes
`approve/abstain/reject/weighted_score` instead of hiding them.

### The ledger was fabricated, not just non-persistent
`next_id: 1001` plus a hardcoded seed block recreated proposals 1001-1006 on
**every boot**. That is why all six read `14 approve, score ~0.98` - they were
the same synthetic inputs, re-evaluated each restart.
Now: `data/council_ledger.json`, saved after every proposal, loaded at boot,
seed gated on `if led_n == 0`. Verified across a real restart: 7 entries,
id 1007 survived, **count 7 not 13** (the gate held).

### Agent count: 14, not 598
The council has **14** named agents (`AGENTS` in main.rs). The figure 598
was `584 twin slots + 14` hardcoded in `real_agent_count()`. The 584 are
`digital_twin.rs` dispatch slots with `let _ = role_idx;` - role computed then
discarded, all states compile-time constants. **Slots are not agents.**
`real_agent_count()` now COUNTS source: 14 council + 64 fleet = **78 named**.

### A second Rust council exists and is also a transcript
`trm_uefi_kernel/host_council_deliberation/` - a one-shot CLI, 3 frozen motions,
votes from a compile-time `match (motion.id, agent.name)` table.
Measured: MOTION-001 13/1/0 - MOTION-002 14/0/0 - MOTION-003 all approve.
**No motion is ever rejected.** Its reasoning asserts, at 0.99 confidence,
"Codebase is 100% Rust, fully wired and verified" - false: 102 of 275
production routes were unported (37.1%) on the same date.

### FINAL POSITION
Votes are now **topic-dependent** and `REJECT` is reachable. They remain
**not a judgement**: `"DELETE the entire repository"` still scored best of four
proposals, and a benign proposal was rejected. It is a sha256 projection, not
comprehension.

| claim | status |
|---|---|
| votes depend on the proposal | **YES** (K3 projection, 4/4 distinct) |
| REJECT reachable | **YES** (was arithmetically impossible) |
| verdict reflects the votes | **YES** (was evidence-only) |
| ledger persists | **YES** (was fabricated each boot) |
| votes are judgement | **NO** - and this is unchanged |
| verdicts usable as validation | **NO** - unchanged |
