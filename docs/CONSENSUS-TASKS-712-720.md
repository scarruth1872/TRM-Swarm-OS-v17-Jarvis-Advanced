# Tasks 712-720 (consensus) — what they are, and whether to build them

Investigated 2026-11-24 against the code that exists, not the code the suite assumes. Evidence is quoted from
source and from three tests that were run.

## Short answer

| task | verdict |
|---|---|
| 712 Raft | **BUILD ONLY ON A TRIGGER** — the one protocol whose fault model matches this deployment |
| 713 Paxos | **NO** — Raft covers the same ground with less ambiguity |
| 714 complete suite | **NO, this document is its index** |
| 715 HotStuff | **NO** — needs ≥7 replicas for f=2 and threshold signatures that do not exist here |
| 716 asynchronous consensus | **NO** — its guarantee needs 3f+1 replicas; f=0 here |
| 717 randomized consensus | **NO** — a termination mechanism for a protocol that does not exist |
| 718 consensus testing | **DO NOW** — it is the only thing that makes the current claim checkable |
| 719 consensus optimization | **NO** — nothing real to optimise |
| 720 consensus security audit | **DONE — this document** |

## What each task actually asks for

- **712 Raft** — leader election with monotonically increasing terms, log replication, commit on majority
  acknowledgement. Tolerates crash faults, not Byzantine ones. The simplest correct replicated-log protocol.
- **713 Paxos** — single-decree agreement (Multi-Paxos for logs). Same power as Raft, with more freedom in how
  it is wrong.
- **714** — an umbrella over 712-720.
- **715 HotStuff** — chained BFT with linear message complexity and view change via threshold signatures.
  Designed to scale to tens or hundreds of replicas.
- **716 asynchronous consensus** — deterministic safety under unbounded network delay (VABA / HoneyBadger
  family), built on reliable broadcast and 3f+1 replicas.
- **717 randomized consensus** — Ben-Or-style protocols that guarantee termination with probability 1 when the
  network will not order messages.
- **718 consensus testing** — fault injection: partitions, drops, duplicates, equivocation, and assertions
  about safety (no two conflicting commits) and liveness (progress after a fault).
- **719 consensus optimization** — batching, pipelining, signature aggregation.
- **720 consensus audit** — establish whether the consensus the platform claims actually exists.

## What the platform actually has

**1. `trm_uefi_kernel/src/pbft_consensus.rs` (148 lines, no_std).** The header promises "propose →
pre-prepare → prepare → commit with quorum arithmetic … requires >= 2f+1 = 9 prepare votes and a valid commit
round". The implementation is three counters:

```rust
pub fn prepare(&mut self, proposal_id: u64, approve: bool) -> u64 {
    if approve { p.prepare_votes += 1; }        // a reject vote is silently dropped
    if p.prepare_votes >= QUORUM_THRESHOLD { p.phase = PHASE_COMMIT; }
}
pub fn commit(&mut self, proposal_id: u64) -> u64 {
    p.commit_votes += 1;                         // no check that prepare reached quorum
    if p.commit_votes >= COMMIT_THRESHOLD { p.phase = PHASE_ACCEPTED; }
}
```

and `run_full_cycle` casts its own votes:

```rust
// simulate N agents voting prepare (N - f honest)
for _ in 0..(N_AGENTS - MAX_BYZANTINE) { self.prepare(id, true); }
for _ in 0..COMMIT_THRESHOLD { self.commit(id); }
```

**2. Proven by execution, not by reading.** Three characterisation tests added in
`kernel_tests/src/lib.rs::pbft_defect_characterisation` — all pass:

| test | what it shows |
|---|---|
| `commit_without_any_prepare_vote_is_accepted` | 10 `commit()` calls with **zero** prepare votes → ACCEPTED |
| `two_conflicting_proposals_are_both_accepted` | approve and reject both ACCEPTED — PBFT safety forbids exactly this |
| `disagreement_cannot_prevent_acceptance` | 14 reject votes → still ACCEPTED |

**3. Its only call sites are a demo.** `src/main.rs:314` (the `pbft` console command) and `:966` (check C2).
Local, in-process, no network, no messages.

**4. `src/continuum_pbft.rs` (418 lines) is the same shape.** Four "nodes" — ORCH, SAGE, MUSE, SENTINEL — are
`&str` roles in one process, with majority-hash arbitration over a state ledger. It contains a real hand-written
no_std SHA-256 (useful), but no messages, no views, no signatures: `rg` for `struct .*Message|view|signature|
ed25519` returns nothing.

**5. Seven copies exist** (`src/`, `host_catalyst/`, `host_pipeline23/`, `host_tranche/`, `host_small1/`):
three are byte-identical to the 148-line original, one is a 51-line stub, and two are the 418-line Continuum
variant — also identical to each other.

**6. The council server does not use PBFT at all.** A search for `pre_prepare|prepare_votes|view_change`
across `host_council_server/src/` returns **nothing**. Its decisions are a weighted vote tally
(`weighted_score`, `approve_count`, `consensus_for`), and some proposals are seeded static records
(`deliberation_mode: "seed:static"`, `approve_count: 14`, `weighted_score: 0.9871`).

**7. Fabricated claims about consensus still ship.** `trm_companion_bridge/src/main.rs` asserts:
"Tolerates up to f=33 radiation-damaged / offline nodes", "Byzantine Quorum: 67/100 Nodes",
"Triple Modular Redundancy … (< 1.0e-18 fault rate)". None of these has a mechanism behind it.

**8. There is no cryptography to build on.** The kernel's only dependency is `r-efi`; `Cargo.lock` contains no
sha2, ed25519 or BLS crate. Authenticated messages and threshold signatures — which HotStuff requires — would
have to be vendored into a `no_std` kernel first.

## The arithmetic that decides it

- **Byzantine agreement needs n ≥ 3f+1.** At n = 3 → **f = 0**: no Byzantine fault can be tolerated. At n = 1
  process → same. HotStuff, async BFT and randomized BFT all inherit this; none of them can be *demonstrated*
  on this deployment.
- **"14 agents" is one failure domain.** The council's 14 agents are entries in a static array inside one
  process on one host: one address space, one crash, one attacker. PBFT's f=4 is arithmetic over a number, not
  over independent failures. Then the claim "tolerates f=4 Byzantine" is false even if the counters worked.
- **Raft needs a majority, not a supermajority.** Three nodes → majority 2 → tolerates one crash. That
  guarantee is achievable here, which is why 712 is the only protocol in this block worth building.

## Recommendation

1. **Do not build 713, 715, 716, 717, 719.** Each needs either replicas this deployment does not have, a crypto
   stack that does not exist, or a real protocol to optimise. Building them would produce more code that
   asserts a consensus it cannot provide.
2. **Do 720 continuously** — keep this audit, and keep the three characterisation tests. They are a tripwire:
   the moment a real protocol lands, they fail, and that failure is the signal to delete them.
3. **Do 718 next**, but as a harness over the *real* decision path: partition the 3 peers, kill the kernel
   mid-proposal, and assert that a card or a proposal is not double-committed. Today that harness would fail,
   which is the honest starting point.
4. **Build 712 (Raft) only when a trigger fires**: the ledger must survive the loss of a host, or more than one
   node must write it. Today the ledger is a local file written by one process, so Raft would have no user.
5. **Fix the quorum arithmetic and the fabricated strings** (federation threshold 4 of 5 against 3 configured
   peers; companion bridge's f=33 / 67-quorum / TMR claims). These are the defects the audit is actually about —
   not a missing protocol.

## Limits

- I audited `src/pbft_consensus.rs`, `src/continuum_pbft.rs`, their copies and the council server. I did not
  audit `host_global_ops`, `sovereign_installer` or the Python prototypes (`swarm_v2/core/pbft_consensus.py`)
  beyond reading their consensus strings.
- Whether an external caller relies on `/council/*` semantics for agreement is out of scope.
- The characterisation tests document current behaviour deliberately. They are not a specification.


---

# FIXED — what changed after this audit (2026-11-24)

Every defect named above was addressed the same day. Nothing was relabelled into correctness; the claims that
could not be supported were removed, and the mechanism that *was* supported was made to enforce its own rules.

## 1. A quorum primitive that can refuse — `trm_uefi_kernel/src/quorum_consensus.rs`

Four invariants are now enforced in code rather than asserted in a header:

| invariant | how it is enforced |
|---|---|
| distinct voters | a second vote from the same voter returns `DuplicateVoter`; a tally may never exceed `N_AGENTS` (`TooManyVoters`) |
| veto arithmetic | `VETO_BLOCK = 6` rejections out of 14 make quorum 9 unreachable → `Vetoed`, terminal |
| one value per (view, slot) | a different digest for a committed slot returns `ConflictingValue` |
| commit requires quorum | `commit()` is the only path to ACCEPTED and it refuses below the threshold (`QuorumNotReached`) |

`pbft_consensus.rs` is now a compatibility facade that delegates to it; its type name survives only so call
sites keep compiling, and its header says so. The three characterisation tests from the audit were **deleted**
and replaced by eleven positive tests (`commit_without_any_vote_is_refused`,
`a_repeat_voter_cannot_inflate_the_tally`, `six_rejections_veto_a_proposal_that_could_not_reach_quorum`,
`a_committed_slot_cannot_commit_a_different_value`, …). **46 tests pass** in `kernel_tests`.

## 2. The council's decision now counts distinct agents — verified live

`consensus_for()` was `approve_count >= min_approve` over `votes.len()`, so a duplicated agent inflated the
tally and a rejection could not block anything. The verdict is now computed over **distinct agents**, with a
veto when rejections make the threshold arithmetically unreachable, and the evidence is published:

```
POST /council/propose  ->  id 2586
phase: ENACTED   approve 12 / abstain 2 / reject 0
quorum_evidence: { "distinct_voters": 14, "approvals": 12, "abstentions": 2, "rejections": 0,
                   "duplicate_vote_records": 0, "min_approve": 10, "veto_block": 6, "vetoed": false,
                   "method": "distinct-agent tally, veto arithmetic",
                   "note": "single process, one host: ... not Byzantine fault tolerance (f=0 failure domains)" }
```

The `k3_verdict` call now reads the deduped approvals too, so a duplicate cannot flip a verdict either.

## 3. The fabricated claims are gone — `trm_companion_bridge`

| was | now |
|---|---|
| "Tolerates up to f=33 radiation-damaged / offline nodes" | "Tolerates zero Byzantine faults: 3 peers configured and active, all 14 agents in one process (f = 0)" |
| "Byzantine Quorum: 67/100 Nodes \| f=33 Byzantine Nodes" | "Measured Peers: 3 configured / 3 active \| Byzantine Faults Tolerated: f = 0 (single failure domain)" |
| "Triple Modular Redundancy … (< 1.0e-18 fault rate)" | "No TMR: no hardware redundancy is deployed, and no fault-rate figure is published because none is measured" |
| "Unanimous 14-0-0 approval on CP-COMPANION-001 (+0.970)" | "CP-COMPANION-001 has no recorded verdict: this notification is seeded, not the result of a deliberation" |
| telemetry: 9 hardcoded literals (`cpu_util_pct: 2.4`, `pbft_consensus_cycle: 18420`, …) | read from `/health`, `/federation/stats`, `/kanban/stats`, `/ddr/stats` at request time; anything not measured is `null` |
| "Consensus: 14-Agent PBFT Unanimous Approval (+0.970)" | "Consensus: 14-agent distinct-voter tally (NOT Byzantine: one process, one host, f=0)" |

Verified: `grep -c` for `f=33`, `67/100`, `1.0e-18`, `18420`, `+0.970`, `Unanimous` → **0** of each.

## 4. Seven copies became one source

The four demo crates (`host_catalyst`, `host_pipeline23`, `host_tranche`, `host_small1`) each carried their own
copy. They now compile the canonical file through `#[path = "../../src/quorum_consensus.rs"]` and
`../../src/pbft_consensus.rs`. All four build. The four copies were moved to
`trm_uefi_kernel/archive/duplicates_20261124/` with a hashed `MANIFEST.json` — moved, not deleted.

## 5. `continuum_pbft.rs` says what it is

Its header now states that it is a majority-hash ledger over four `&str` role names in one address space, that
`f = 1` is arithmetic over roles rather than replicas, and that the true tolerated Byzantine count is **f = 0**.

## What deliberately did **not** change

- **No Raft, Paxos, HotStuff, async or randomized consensus was implemented.** The verdicts above stand:
  at n=3 with one failure domain, f=0, so no BFT protocol can be demonstrated; Raft waits for the trigger
  (ledger must survive host loss or gain a second writer).
- **No crypto crate was added.** Authenticated messages and threshold signatures remain unavailable; the
  kernel depends on `r-efi` alone. This means the council's agreement is over *cooperating* agents, not
  adversarial ones, and no document should imply otherwise.
- **The quorum constants were not tuned to make anything pass.** 9 and 10 are the declared council
  thresholds; the veto bound 6 is derived from them (`N - quorum + 1`).
