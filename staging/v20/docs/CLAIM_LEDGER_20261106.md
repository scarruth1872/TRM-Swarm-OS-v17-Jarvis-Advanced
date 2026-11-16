# Claim ledger — 2026-11-06

Protocol: a claim is a thought in superposition (+1 true / −1 false / 0 unmeasured). Only a
measurement collapses it, and the collapse records its instrument and evidence. A claim that no
instrument reaches stays at 0 — it is not "probably true".

Harness: `scratch/collapse_claims.py` → `scratch/claim_ledger.json`, 13 checks, each reading a second
source rather than the route's own description of itself.

## Collapsed — 13 of 13 verified (at the time of the run)

| id | claim | instrument |
|---|---|---|
| C1 | 15 agents on every surface | member sets equal across `/council/status` and `/infrastructure/status` |
| C2 | PBFT quorum 9 | derived `2f+1` from n=15, f=4; equals served value |
| C3 | Φ_V is the doctrine formula | recomputed from clock/mem; delta 0.0e+00 |
| C4 | both basis strings agree on rail provenance | qualifier present in both |
| C5 | engine refusal names the real corpus | refusal's number == `st_size` 4,849,197,487 |
| C6 | fabric reports its store's size | served 767,616,965 == file size |
| C7 | matrix census is the `/system` census | `modules_compiled` 322 found in the page |
| C8 | `agent_slots` is the host ceiling | served 16 == `os.cpu_count()` 16 |
| C9 | motions persist in a durable ledger | served 4 == ledger rows 4, ids unique |
| C10 | stick carries the built artifacts | rootfs `e2afdb4f…`, initrd `95f6bd31…` both equal |
| C11 | `/ui/` serves the platform page | marker present, 9,889 B |
| C12 | tier-2 is not resident until called | :8587 closed and `TRM_TIER2_EAGER` gate present |

## Refuted during the run — each fixed, then re-measured

1. **C5 first read the limit, not the corpus.** The extractor took any long integer and hit
   536,870,912. Minimal fix: parse `corpus JSON is (\d+) bytes`. The platform was right; the
   instrument was wrong.
2. **C9: motions did not survive a restart.** `state.proposals` is in-process; the served count was 1
   after five motions. Added `swarm_v2_artifacts/council_proposals.jsonl` (append per motion) and made
   `/council/status` report the ledger's line count. Verified: 3 motions filed → restart → 4 rows,
   ids continued 1006/1007 without reuse.
3. **Voltage basis contradicted itself** (earlier this session): inner said `no VID probe`, outer
   claimed a live VID. Outer string now derives from the probe.
4. **Boot content checks were stale** (earlier): asserted LCARS/native-ternary strings the host does
   not emit. Replaced with host-measured markers and per-route expected status; 9/9 pass.
5. **`pivot_root` fell back to chroot** leaving the old root mounted. Replaced with switch_root's
   `MS_MOVE`; boot log confirms.

## Still in superposition — no instrument reaches these yet

| claim | why it stays at 0 |
|---|---|
| voltage controls the compute term | `core_vid_volts()` returns None (no signed Ryzen driver); rail ratio fixed at 1.0 |
| firmware chainloads Linux | chainloader fixture absent; UEFI app states it IS the boot image |
| the seven write routes work | mutating POSTs deliberately unfired |
| DeepDTA generalises beyond EGFR | one protein, held-out ρ 0.335 |
| exact-ternary engine at corpus scale | refuses above 512 MB; 4.85 GB corpus |
| host test suite runs in parallel | stack/allocation crash under concurrency |

## Artifact note

The Windows kernel carries the ledger fix. The musl image rebuild is blocked by rustc ICEs under the
current commit limit (17.4/19.4 GB, 2.4 GB free); the packer repackaged the stale binary once, which
is exactly why the build must be proven by output hash rather than by the packer's success line.
