# Agent coordination — this repo has more than one writer

Two agents edit this tree concurrently:

| agent | edits | commit prefix |
|---|---|---|
| **Hermes** (deepseek-flash, desktop) | cron tools, docs, kanban, `amr_surveillance.rs` | `G###:` |
| **Antigravity** (Gemini, IDE) | `host_council_server/src/main.rs` and friends | names its own change in the subject |

Neither is a guest. The rules below exist because on 2026-11-23 an unprompted `main.rs` edit landed at
19:44:57 mid-session, was built at 19:47:30 and restarted the kernel — correct work, but the other agent had
no idea it had happened, and could have reverted or overwritten it.

## Rules

1. **Look before you edit.** `git status --porcelain -- <path>` before touching a file. If it is modified
   and the diff is not yours, stop and say so instead of editing on top of it.
2. **Never `git add -A`.** Commit explicit paths, or another agent's half-written file rides along.
3. **Never revert or delete an uncommitted change you did not write.** Uncommitted ≠ abandoned. Ask.
4. **Commit small and often**, each commit verified, so the other agent's work is never more than a few
   minutes from safety.
5. **Name the author in the subject.** `G239: ... (antigravity)` for a change taken over from the other
   agent; `G###:` for my own. A reader must be able to tell who did what.
6. **Do not stop the kernel to run tests.** `CARGO_TARGET_DIR=target/verify cargo test --release` builds into
   a separate directory, so the running `host_council_server.exe` is not locked and nobody's session is
   interrupted. Measured: 122 tests, no downtime.
7. **Restart through the scheduler, never from your own shell.** `POST /platform/shutdown
   {"confirm":"stop"}` then `schtasks /Run /TN SwarmKernel`. A kernel started as a grandchild of an agent's
   shell dies with that shell — that is how the platform went dark at 19:45, and the task's own tick then
   skipped starting one because the dying instance still answered the port.
8. **Verify the other agent's claim, don't restate it.** Antigravity's DDR-persistence claim was checked by
   actually restarting: `scans_performed` held at 1 while `scans_since_boot` reset to 0.

## Hotspot

`trm_uefi_kernel/host_council_server/src/main.rs` — expected collision point. Both agents add routes and
counters there. **New capability belongs in its own module** (`amr_surveillance.rs` is the pattern: one
`mod` line and one route branch in `main.rs`, everything else elsewhere). Treat repeated conflicts on
`main.rs` as a signal to split the file, not to reconcile it again.

## Shared state both agents write

`data/kanban_board.json` (live board), `data/council_ledger.json` (proposals), `data/ddr_scan_state.json`
(DDR counters). These change on every request — never hand-edit them, never treat a modified copy as a
conflict, and never commit one without saying which measurement produced it.

## Background notices cost turns — do not spray them

Measured 2026-11-24: the Telegram channel received **8 background-process notices** in one day and each one
was injected into the session as a turn — three of them inside four minutes with no user message, at ~30 s
each. The gateway has **no batching setting** (its only mechanism is per-notice dedupe by notice key), so the
reduction has to happen at the source:

- use `notify=true` for a **terminal** event only (the process finishing), never `notify=['pattern']` watch
  patterns for progress chatter;
- prefer polling a log once over being pinged on every match;
- if several results are expected, run one background job that reports once rather than many that report each.

`tools/cron/telegram_transport.py` counts notices per hour and complains at 4 or more, so the volume is
measured instead of remembered.
