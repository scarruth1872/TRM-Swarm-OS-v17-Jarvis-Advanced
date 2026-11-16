# CHECKPOINT — 2026-11-13 (pre-reboot, BIOS update pending)

Everything below was measured immediately before this file was written. Where something is *not* verified it says
so; nothing here is a plan dressed as a fact.

---

## 1. State at this moment: STOPPED, verified over HTTP

    python tools/platform_stop.py     -> "TRM platform stopped — every port re-probed and silent."
    STOP_TRM_PLATFORM.bat             -> rc=0, same verification

| Port | Service | State |
|---|---|---|
| 8022 | Kernel / Council (native Rust) | down — self-terminated on `POST /platform/shutdown` |
| 8612 | Bonsai2-27B PTQ1_0 | down — pidfile |
| 8620 | CPU ternary pool | down — pidfile |
| 8730 | GPU 8-slot pool | down — pidfile |
| 8740 | GPU 14-slot experiment | down — pidfile |
| 8593 · 8587 · 8591 · 8595 · 8596 | model fleet | down |
| 3000 | Jarvis | down — 9 processes, image + command-line matched |
| 3001 · 3002 | Companion proxy / bridge | down |
| 5183 · 5184 | LCARS / Kernel UI | down |

VRAM after shutdown: **1.6 GB peak** (desktop compositor only). Before: ~10.8 GB of the 12 GB card.

> **The old stop path was defective and is fixed (G108).** `netstat -ano` has **no rows** for this platform's ports
> while they serve HTTP — `:8593` and `:3000` return nothing, and the TCP table shows only TimeWait/FinWait1 owned by
> pid 0. The previous script therefore printed `[DOWN] port free` for a live embed server, `[STOPPED] Jarvis (pid
> 20424)` after killing a wrapper, and `TRM platform stopped` while `:8593` and `:3000` were still answering. It also
> never touched `:8612`, `:8620`, `:8730` — the three processes holding ~10.7 GB. `STOP_TRM_PLATFORM.bat` is now a
> thin wrapper around `tools/platform_stop.py`, whose only verdict is a re-probe of every port.

---

## 2. What changed today (G98 → G108)

| Gap | Finding |
|---|---|
| G98 | The 584 "agents" are a dispatch table; the Bonsai family cannot be stacked as layers (shared tokenizer only) |
| G99 | Speculative decoding: 70.01 → 26.22 tok/s (×0.37) at 0.777 acceptance, and **not** output-identical |
| G100 | **Agents are slots, not processes**: 12 processes 0.53 tok/s vs 12 slots 245.40; 8 slots is the peak (312.80) |
| G101 | CPU-ternary tier was never broken (1.5–2.5 s load, 19.57 tok/s); best shape **8 GPU slots + 14 workers = 7.96 items/s** |
| G102 | Pinning is not exclusivity; overlap gain bounded at ~20 % |
| G103 | The tier penalty is **pre-fill** (52×); voltage is not askable over WMI |
| G104 | HWiNFO feed live (SVI2 1.25–1.29 V vs WMI's constant 1.1 V); CPU tier characterised; **packing refuted** |
| G105 | GPU **not throttled** (60 % PPT, hotspot 79 °C, 2,024–2,120 MHz); **offline queue** — cores work without gating a batch |
| G106 | Bigger ubatch does **not** convert that headroom (real workload −52.5 %); CPU audit: SVM off, DOCP off, CPB idle |
| G107 | Boost cannot be enabled from Windows (already mode 2; min-state 100→99→5 never moved 4,000 MHz) → firmware |
| G108 | Stop path rebuilt: pidfile → image-filtered command line → HTTP re-probe; four defects fixed |

Commits this arc (newest first): `3abb691`, `07de777`, `99d5b4d`, `251a8f3`, `0c9ee07`, `97510c4`, `b08d162`,
`a0f811c`, `84a935a`, `1ff4364`, `72cd6fa`, `38e5c2a`.

---

## 3. Artifacts and hashes

    Boot artifact      trm_uefi_kernel.efi   17,488,896 B   sha256 504c41ee647ff45f…
    PE                 machine 0x8664, subsystem 10 (EFI application)
    USB G: (SWARM_OS)  EFI/BOOT/BOOTX64.EFI, trm_uefi_kernel.efi, trmboot.efi, trmboot_pe.efi
                       → all four identical, parity PASS
    Previous media     scratch/usb_backup_20261113_091036/ (and 085037)
    QEMU boot runs     scratch/qemu_boot/20261113_090846 (no fabric), 090918 (fabric), 090957 (NIC)
    Journal manifest   phoenix_journal_audit_manifest.json — 115 articles (max id 216)

Rebuild/deploy/verify at any time:

    bash trmboot/build_uefi.sh --deploy G
    python tools/qemu_boot_test.py [--fabric 76761697] [--nic]

---

## 4. Measured baselines — compare against these after the reboot

| Quantity | Value before reboot |
|---|---|
| Memory Clock / FCLK / UCLK | **1,200 / 1,200 / 1,200 MHz** (DDR4-2400, 1:1) |
| Core clock under 12-process load | **4,000.0 MHz flat**, ratio 40×, `Frequency Limit - Global` 4,440–4,633 MHz |
| Package power / Tctl under load | 105–108 W / 74.9 °C |
| SVI2 vcore | 1.250–1.294 V (WMI claims a constant 1.1 V) |
| `phi_v_measured_svi2_rails` | 0.4805 vs published WMI 0.3505 |
| CPU 2B ternary (`:8620`) | prefill **52.6 tok/s**, decode **27.4 tok/s** |
| GPU 8-slot tier (`:8730`) | **423.4 tok/s** decode (8×200 tok) · **9.27 items/s** on a 60-file batch |
| GPU under sustained load | 113–118 W of 186 W PPT (60 %), hotspot 75–79 °C, 2,024–2,120 MHz |
| Council gate | 80.0 % danger recall · 0.000 % false-block — advisory, fail-open |
| Journal | 115 articles in the manifest (max id 216); last served 114 while the council was up |
| Offline queue | 120/120 drained, 0 errors, 8.6 jobs/min |

---

## 5. The BIOS change (user action — see `docs/BIOS-CHECKLIST-DOCP-CPB.md`)

    DOCP   Ai Tweaker -> Ai Overclock Tuner = D.O.C.P. -> DDR4-3200, DRAM 1.35 V, FCLK 1600 (1:1)
    BOOST  AMD CBS -> CPU Common Options -> Core Performance Boost = Enabled  (+ PBO Enabled)

Expected, **not yet measured**: memory 1,600/1,600/1,600; core clock above 4,000 MHz under load; +11–16 %
single-thread ceiling; +33 % peak memory bandwidth — but CPU decode sits at 12.7 GB/s ≈ 33 % of DDR4-2400 peak, so
the bandwidth gain will not be one-for-one.

Board: ASUS PRIME B550M-A (WI-FI), BIOS **1804 (2021-02-01)**. If DOCP will not train: Clear CMOS, retry at
3,000/3,133 with FCLK at half the memory clock, and treat a BIOS flash as a separate decision.

---

## 6. Start / stop, in the order that matters

    START_TRM_PLATFORM.bat    boots the stack; council pid recorded from Start-Process -PassThru
    STOP_TRM_PLATFORM.bat     -> tools/platform_stop.py
                                1) council: POST /platform/shutdown, wait for /health to STOP ANSWERING
                                2) models: pidfile first (run/*.pid), then image-filtered command line
                                3) every port re-probed; "NOT STOPPED: …" and rc=1 if anything answers

    python tools/platform_stop.py --keep 8612    # stop everything except one service

A stop that reports success while a port still answers is not a stop.

---

## 7. Known state that a reboot does not fix

1. **The gate is fail-open.** 80.0 % danger recall, **20.0 % dangerous miss**, 0.000 % false-block on 337,861
   judged jobs. Blind-only pooled recall 0.620 → 0.680 after two arms. Held-out set `_4` is the weakest at 0.525
   and no rule has been written for it. Treat every verdict as advisory.
2. **On-metal boot of the USB is unverified.** QEMU proves it is a valid EFI application that reaches the
   compositor and reports measured hardware; it cannot prove this board's firmware enumerates GOP/NVMe/AX200 the
   same way.
3. **opt-level 3 cannot build the kernel crate** on rustc 1.99.1 (`STATUS_STACK_OVERFLOW` in LLVM's thin-LTO
   thread; 512 MB stack and 256 codegen units both fail). The pipeline uses opt-level 2 without LTO.
4. **153 ledger blocks are errata**, not history to be deleted: they say `CONFIRMED` while their own severity says
   `WARNING`. Query them with `tools/ledger_contradictions.py`; the writer is fixed.
5. **Version strings disagree** in four places (README `v19.026`, Cargo `19.1.0`, boot banner `v19.001`,
   `staging/v20`). Recorded in the README table; do not bump one and call it unified.
6. **Third-party sensor feed is manual**: `/voltage/telemetry` needs HWiNFO logging to `TRM_HWINFO_CSV`. A frozen
   CSV silently repeats its last row, so check the row count grows. Absent → `available:false` with a `how` string,
   never a nominal.
7. **One test is a documented xfail.** `test_swarm_council_phoenix_enactment` cannot pass: the topic yields a
   deterministic 4 approve / 3 abstain / 7 reject of 14, weighted −0.12 against a ">0.3 and ≥10 approvals" gate, so
   `enact()` is never reached. The gate dates from the v19.001 baseline (`39f91e2`) and `swarm_v2` was untouched by
   this session, so it is not a regression. It is marked xfail with those numbers rather than re-tuned, and a new
   `test_swarm_council_rejection_is_honest` asserts the property that does hold: no consensus → REJECTED, no seal,
   reason present.
9. **`START_TRM_PLATFORM.bat` will probably end in "LAUNCH FAILED" again.** Measured this morning
   (`logs/platform_launch_20261113_082724.log`): **10 of 12 healthy**, with `:8099 Council API (app_v2)` and
   `:8588 TRM Service (trm_server)` refusing connections. The launcher treats any red service as a failure and says
   so loudly, so expect that message even though the platform is usable. `app_v2` is a heavy Python start (TurboVec,
   126 intents, ChromaDB, topology v27) and refuses to run a second instance while one holds `:8099`; nothing in the
   logs explains `:8588`. Not fixed here — the box is being rebooted first, and diagnosing it needs the stack up.
10. **Jarvis's stats panel is dead**: its Vite proxy calls `http://127.0.0.1:8021/api/system-stats` and nothing
    serves `:8021` (the council is `:8022`), so the dashboard logs `ECONNREFUSED` on every poll. Cosmetic until the
    stitched panel is needed.
11. **SVM is off** (HWiNFO "Secure Virtual Machine: Not Present"; `VMMonitorModeExtensions=False`). No Hyper-V/WSL2
   on this box until firmware enables it.

---

## 8. After the reboot

    START_TRM_PLATFORM.bat
    python tools/hw_verify.py                    # docp_3200 / fclk_1to1 / boost_active / stable
    python tools/cpu_fix.py --tokens 32          # CPU prefill + decode sweep
    python tools/agent_fabric.py --n 60 --workers 14 --ports 8730 --tag postreboot
    python tools/offline_queue.py status --queue swarm_v2_artifacts/OFFLINE_QUEUE.jsonl

Pass conditions after the BIOS change:

    docp_3200     Memory Clock >= 1,500
    fclk_1to1     FCLK == Memory Clock
    boost_active  core clock > 4,001 MHz under load
    stable        zero WHEA Error/Critical events since boot

Then boot the stick on metal and read the panel: it must show **the real resolution** and either
`K3 NAND FABRIC: MOUNTED … sectors first OK last OK` or `NOT MOUNTED … TRITS UNMEASURED`. If it shows a nominal
resolution instead, an older artifact is on the media — re-run `build_uefi.sh --check G`. If it reboots ~5 minutes
in, the watchdog clear is the first suspect (`set_watchdog_timer(0,0,0,NULL)` in `efi_main`).

---

## 9. Before you power down

- [x] Platform stopped and every port re-probed silent (VRAM 10.8 GB → 1.6 GB)
- [x] Journal written (115 articles, max id 216). Council is now down, so it is **not** currently served:
      re-check `/journal/articles` after `START_TRM_PLATFORM.bat`
- [x] Docs updated: README (state + reboot runbook), CHANGELOG `[19.028]`, this file, BIOS checklist
- [x] Boot artifact still deployed and parity-verified; QEMU runs recorded
- [x] Working tree committed
- [x] Test suite: **67 passed, 1 xfailed, 0 failed** (70 s)
- [ ] BIOS: D.O.C.P. + Core Performance Boost / PBO, then `F10`
