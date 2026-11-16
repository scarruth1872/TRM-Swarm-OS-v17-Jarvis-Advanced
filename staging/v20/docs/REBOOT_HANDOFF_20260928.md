# Reboot handoff — 2026-09-28

Written immediately before a planned reboot. Everything below was measured, not assumed.

## Re-establishing the runtime after reboot

Two processes, and the ORDER matters. The supervisor owns `:8023`, spawns the kernel as its child,
and **passes its entire environment to that child**.

```
cd <repo root>
# 1) supervisor FIRST — it spawns the kernel. cwd must be the repo root (its --config path is relative).
trm_uefi_kernel/host_supervisor/target/release/host_supervisor.exe \
  --config trm_uefi_kernel/host_supervisor/supervisor.services.conf \
  --interval 10 --log swarm_v2_artifacts/logs/supervisor_rs.log
```

**Export NOTHING first.** A measured incident: `TRM_PORT=8097 TRM_UI_PORT=8098` exported for a sandbox
leaked into the supervisor, which handed them to the kernel child, which then bound 8097/8098 instead of
8022/5183 while everything looked healthy. If that ever happens, `unset TRM_PORT TRM_UI_PORT` and
restart the supervisor — not just the kernel, because the supervisor holds the stale environment.

If the supervisor starts two instances, the `:8023` socket bind is the single-instance guard; instances
started before that guard existed will not have taken it. Stop them all, start exactly one.

## Expected healthy state

| check | expected |
|---|---|
| `:8022`, `:5183` | ONE pid, same handler table, both 200 |
| `:8023` | the supervisor's own socket |
| `/health` | `HEALTHY`, kernel `Native Rust Sovereign Kernel v19.002`, `python_active: false` |
| `/infrastructure/status`, `/infra/health`, `/infra/history` | 200, served by the FSM |
| `/infra/health` body | `status: HEALTHY`, `by_health {healthy: 5, not_started: 2}`, `interval_s: 10` |
| `/memory/stats` | `total_memories 2180`, `sync_events 1994` |
| `/ui/` | 200; `/ui` with no slash → 301 |
| runtime inventory | 1 kernel, 1 supervisor, 3 `llama-server`, 0 node, 0 TRM Python |

The supervisor's status file is anchored to `trm_uefi_kernel/host_supervisor/supervisor.status.json`
(not the cwd — it was a relative write until 2026-09-28, which meant two files claimed to be live state).

## PENDING — the ledger repair (not started)

Blocks **193–205 carry no `prev_hash`** field; the documented shape is `index`/`hash`/`prev_hash`, and
the canonical sealer's rule hashes over every key except `hash` with `prev_hash` INCLUDED. Store's own
ruler: `blocks=205 canonical=195 prev_hash links broken: n=13 [193..205] chain_intact=False` (plus the
pre-existing UNVERIFIED 97–101).

**Measured blast radius before repairing: ZERO files outside the ledger cite any of those 13 hashes.**
So adding the field and recomputing each block's hash in sequence is safe — no doc or record would go
stale. The repair must also fix its own cause: seal through the canonical sealer
(`scratch/seal_*.py`, which includes `prev_hash`) rather than an inline snippet, and record the
old→new hash mapping plus a repair block that itself carries `prev_hash`.

## State of the working tree

13 files edited on 2026-09-28 and committed as the doc sweep: the root live docs, the JSON metadata,
`dashboard/README.md`, and two `docs/` live documents. Dated records were deliberately left alone —
checkpoints, campaign logs, and the 2026-09-23/24 reports state what was true on their dates, and
rewriting one falsifies evidence. Three sibling files were flagged as possibly stale and NOT edited:
`DASHBOARD_DATA_PATH_AUDIT_20260924.md`, `VALIDATION_PLATFORM_CAPABILITIES_20260925.md`,
`EXECUTIVE_SUMMARY_48H_20260924.md`.

## USB boot drive (E:, FAT32 STORAGE3)

Kernel `29890bcf86b38ded` (v19.002, 19,144,704 B) and bootloader `ee885290ad0304d5` (16,388,096 B) on
all three ESP copies. **The rebuilt EFI has not been boot-tested** — the first boot from this drive is
unproven. Rollback: `backups/usb_pre_r17_20260928_213547/RESTORE.sh` restores all 6 replaced files.
`trm_swarm_boot.exe` on the drive is deliberately NEWER than the repo copy; do not overwrite it.
