# Production Readiness — Gap Analysis and Plan

Measured 2026-11-03 on the RX 6700 XT host. Every number below came from a command
run in this session, not from a prior document. Where a prior claim was wrong, the
correction is stated.

---

## 1. The direct answer on the Rust port

**The kernel is not fully ported, and "fully ported" is the wrong target.**

What is true:

| Fact | Measured |
|---|---|
| Kernel modules | 207 `.rs` files, 36,029 LOC |
| Kernel compiles | `cargo check --release` clean, 1,846 warnings, 0 errors |
| Crate targets | `lib` (src/lib.rs) + `bin` (src/main.rs), both real |
| Council server | 12 modules, 6,633 LOC, links the kernel as a path dependency |
| Kernel code actually called by the council | `k3_logic`, `mamba3_turbovec_ssm`, `memory_mapped_ipc`, `unified_transistor_fabric` |
| Modules compared Python↔Rust | 157 |
| Kernel-candidates (real backlog) | 44 |
| Host-only (correctly stubbed) | 26 |
| Net-only (correctly stubbed) | 5 |

So the critical path — council, K3 ternary algebra, Mamba-3 SSM, ARC, memory IPC —
is native Rust and genuinely linked. That is the part that matters and it is done.

The 44 kernel-candidates are the remaining port surface. They are not all equal:
`secrets_vault` at 92% shrink, `redis_mock` at 90%, `brain_ai_microkernel_bridge`
at 88% are near-complete. `llm_orchestrator` at 62% and `intent_buffer` at 67% are
not. The tail is long and low-value.

**Correction to a prior finding:** `tools/surface_diff.py` reports `lib` as
"NEVER-COMPILED (no `mod` declaration)". That is a false positive. `cargo metadata`
shows `src/lib.rs` is a real `lib` target. The tool looks for `mod` declarations
and a crate root never has one. The tool needs fixing before its "1 never-compiled"
line is quoted again.

**Recommendation:** stop treating 100% port as the goal. Define the port as done
when the *decision and control path* is native and the remaining Python is
explicitly classified as host tooling. That is close to true today. Chasing the
last 44 modules would move code that is not on the critical path.

---

## 2. Gap analysis for production release

Severity: **P0** blocks release, **P1** blocks unattended operation, **P2** debt.

### P0-1 — The audit ledger is corrupt and cannot be trusted

This is the most serious finding in the session.

```
manifest claims audited events : 12,517
actual ledger lines            : 10,095
shortfall                      :  2,422
first seq in file              : 13,588   (should be 1)
duplicate seq values           :    419
prev_hash chain breaks         :    418
distinct seq values            :  9,668
```

The ledger is an append-only hash-chained audit log. It currently has 419 duplicate
sequence numbers, 418 places where `prev_hash` does not match the previous record's
`hash`, and it no longer starts at seq 1. The chain is broken, so the integrity
guarantee the format exists to provide does not hold.

Root cause, read in the code (`swarm_v2/core/cognitive_ingestion_engine.py`):

```python
self._last_seq += 1          # in-memory counter, per process
...
with open(self.path, "a", encoding="utf-8") as f:   # no lock, no atomic seq
    f.write(json.dumps(entry) + "\n")
```

Each process that instantiates the engine recovers its own `_last_seq` from the
tail of the file at startup, then increments independently. With 8099, 8588, 3001
and others all writing, two processes allocate the same seq and both compute
`prev_hash` from their own private `_last_hash`. Interleaved appends produce
exactly the pattern measured: duplicate seqs, out-of-order records, broken chain.

This also fails the test suite: `tests/test_phoenix_ledger_schema.py::test_legacy_event_stream_phoenix_compliance`
asserts `>= 12515` lines and fails at 10,095.

**Fix required:** a single writer. Either one process owns the ledger and others
send it events over IPC, or use an OS file lock plus seq allocation inside the
lock. Then rebuild the chain from the surviving records and record the loss as
errata — do not silently renumber, and do not delete the damaged file.

### P0-2 — No authentication on any control-plane port

```
8022  0.0.0.0   council (authoritative, can enact proposals)
8099  0.0.0.0   council API
3000  0.0.0.0   Jarvis
3001  0.0.0.0   companion proxy
3002  0.0.0.0   companion bridge
5183  0.0.0.0   LCARS
8587  127.0.0.1  llama-server      (correctly local)
8588  127.0.0.1  trm_server        (correctly local)
8593  127.0.0.1  embeddings        (correctly local)
```

`grep` for `Authorization`, `Bearer`, `api_key` in `host_council_server/src/main.rs`
returns no header checks — only prose inside article text. Anyone on the LAN can
POST `/council/propose` and enact a proposal on the authoritative council.

**Fix required:** bind the control plane to `127.0.0.1` and reach it over Tailscale,
or add a shared-secret header check. Binding to loopback is the smaller change and
matches how the model servers are already configured.

### P1-1 — The launcher does not reproduce the running system

`START_TRM_PLATFORM.bat` starts 7 services. `grep -c 8593` returns **0**. The
embedding server is not in the launcher, so after a reboot the semantic router
silently degrades to `keyword_fallback`. The router reports this honestly, but the
platform is not the same system it was before the reboot.

Also known: the launcher aborts when `logs/platform_launch.log` is locked, and
returns `LAUNCHER_RC=0` despite the abort — a failure that reports success.

**Fix required:** add 8593 as step 8, make the log lock non-fatal, and make the
launcher exit non-zero when a step fails.

### P1-2 — Voltage is a fallback, not a reading, and one surface still lies

`GET /system/voltage` returns `phi_v=0.6884, measured=false`. That is honest: this
box has no voltage sensor, so the value is derived from nominal rails. The
`measured:false` flag is the right design.

But `0.9115` — the value the voltage doctrine forbids — is still live in
`swarm_v2/core/quantize_clip.py:155` and `council_paper162.py:12`. Any artifact
produced by those paths carries a fabricated constant.

**Fix required:** replace with the derived value or mark the field `measured:false`
in the emitted artifact.

### P1-3 — The SSM cannot be load-bearing, and is now labelled as such

Two read-out attempts, both measured, both failed:

| Features | Leave-one-topic-out | Baseline |
|---|---|---|
| K3 token hashes | 0.458 | 0.455 |
| nomic 768→128 embeddings | 0.474 | 0.455 |

Learning curves flat from 3 to 18 training topics for both. 19% of (topic, agent)
pairs flip verdicts across repeated temperature-0 runs. The limit is the label,
not the method, the sample count, or the feature semantics.

Status is now `ACTIVE (ADVISORY)` in `/learning/skills`, and the fitted artifact
is `deploy:false` with the loader refusing it. This is correctly handled — listed
here so it is not mistaken for a working decision component at release.

### P2-1 — Test suite has one real failure

```
50 passed, 1 failed, 16 deselected in 39.11s
```

The single failure is P0-1. No other test fails. 16 deselected are `integration`
tests requiring live services; they are not run in CI and their state is unknown.

### P2-2 — 1,846 compiler warnings

Not a blocker, but it hides real signal. `cargo check` emits 1,846 warnings
including non-snake-case struct fields in `trm_7m_full.rs`. A codebase that warns
on every build cannot use warnings as a defect signal.

### P2-3 — Uncommitted working tree

`data/council_ledger.json`, `arc_object_sram.rs`, two submodule pointers, and
several untracked artifacts are dirty. A release needs a clean tree and a tag.

---

## 3. Plan

Ordered so each phase removes a blocker for the next. No phase starts before the
prior phase's acceptance test passes.

### Phase 0 — Make the tree honest (half a day)

1. Commit or discard the dirty working tree; tag the result.
2. Fix `surface_diff.py`'s `lib.rs` false positive so the port number is quotable.
3. Write errata for the ledger corruption: what was lost, when, why. Do not delete
   the damaged file.

**Acceptance:** `git status` clean, tag exists, errata committed, `surface_diff.py`
no longer reports a never-compiled crate root.

### Phase 1 — Ledger integrity (1–2 days) — **DONE 2026-11-03, commit 175106f**

Acceptance met: 51/51 tests pass [ERRATA 2026-11-03: "51/51" was the count from
`pytest -m "not integration"`, and it included test_llm_brain_direct, which was
unmarked while hitting a live inference endpoint. The honest split is 50 unit +
18 integration; the 51 was never 51 unit tests. See
`evidence/INTEGRATION_COVERAGE_MEASURED.txt`]; 8 processes x 125 concurrent events produced
1000/1000 records with 0 duplicate seqs and 0 chain breaks; the same test against
the pre-fix code produced 996/1000 with 1 torn line, 125 duplicate seqs and 115
breaks, so the test can fail. Chain rebuilt to 22,521 events, contiguous 1..N,
manifest hash matches. Evidence: `evidence/LEDGER_INTEGRITY_PHASE1_MEASURED.txt`.
Data was never lost — the 2,422-event "shortfall" was rotation, and the raw
pre-rotation stream survived in the sidecar backup.

1. Single-writer design: one process owns `system_event_ledger.jsonl`; others send
   events over IPC. If that is too large a change, take an OS file lock and
   allocate `seq` inside the lock.
2. Rebuild the chain from surviving records into a new file, preserving the
   original as `system_event_ledger.corrupt-20261103.jsonl`.
3. Re-run `tests/test_phoenix_ledger_schema.py`.

**Acceptance:** 51/51 tests pass; a 1,000-event concurrent write test produces zero
duplicate seqs and zero `prev_hash` breaks; the chain verifies end to end.

### Phase 2 — Close the control plane (half a day)

1. Bind 8022, 8099, 3000, 3001, 3002, 5183 to `127.0.0.1`.
2. Reach them over Tailscale for remote access.
3. If LAN access is genuinely needed, add a shared-secret header and reject
   unauthenticated writes.

**Acceptance:** `netstat` shows no control-plane port on `0.0.0.0`; a request from
another LAN host is refused; the Tailscale path still works.

### Phase 3 — Reproducible boot (1 day)

1. Add 8593 to the launcher as step 8.
2. Make the log lock non-fatal; make the launcher exit non-zero on any failed step.
3. Add a post-boot self-check that asserts all 9 services answer and that the
   router reports `method=embedding`, not `keyword_fallback`.

**Acceptance:** reboot, run the launcher once, self-check passes 9/9 with
`method=embedding`. Run it twice; second run is idempotent.

### Phase 4 — Remove fabricated constants (half a day)

1. Replace `0.9115` in `quantize_clip.py` and `council_paper162.py` with the derived
   value, or emit `measured:false`.
2. Grep the whole tree for other hardcoded doctrine values and fix or flag them.

**Acceptance:** `grep -rn "0\.9115"` returns only audit/errata files that describe
the historical error.

### Phase 5 — Decide the port boundary (1 day, decision not code)

1. Classify all 44 kernel-candidates: on the decision/control path, or host tooling.
2. Port only the ones on the path. Document the rest as intentionally host-side.
3. Fix the 1,846 warnings down to a level where a new warning is visible.

**Acceptance:** a written boundary document; every one of the 44 has a
classification and a reason; warning count under 100.

### Phase 6 — Release gate (half a day)

1. Full test suite green, including the 16 integration tests run against live
   services.
2. A cold-boot rehearsal from a clean checkout.
3. A benchmark run recorded as the release baseline: `bench1000.exe` and the
   7-stage master task.

**Acceptance:** 51/51 plus integration green; cold boot reproduces 9/9; benchmark
numbers recorded in `evidence/` with the release tag.

---

## 4. What is already good and should not be touched

- The council is genuinely deliberating: 14 agents, per-agent model judgement at
  temperature 0.0, `deliberation_mode` on every proposal, Rust authoritative on
  8022 with Python delegating. Parity 3/3 measured after the last change.
- Retrieval grounding works and changes verdicts: a proposal the council's own
  rule 6 forbids went 14-0 approve with no citation, then 11-3 reject with 14/14
  citing a source.
- The semantic router is calibrated per domain with honest abstention, and
  degrades to `keyword_fallback` with an `error` field rather than guessing.
- The SSM/K3 pipeline is wired, deterministic, auditable, and correctly labelled
  advisory. It is not load-bearing and no longer claims to be.
- The kernel compiles and the council genuinely links and calls kernel modules.

---

## 5. Bottom line

The platform is **not production ready**, and the blockers are not the ones the
port backlog suggests. The Rust port is far enough along that the decision path is
native. What blocks release is:

1. a corrupt audit ledger (P0),
2. an unauthenticated control plane on all interfaces (P0),
3. a launcher that does not reproduce the system (P1).

Those three are roughly three days of work. The remaining 44 port candidates are
not on the critical path and should be classified, not chased.
