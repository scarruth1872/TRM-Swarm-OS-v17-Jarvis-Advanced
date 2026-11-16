# BIOS CHECKLIST — PRIME B550M-A (WI-FI), BIOS 1804 (2021-02-01)

Two firmware settings were measured as off. Neither can be changed from Windows (G107): boost mode was already
"Aggressive" and minimum processor state 100 -> 99 -> 5 never moved the core off 4,000 MHz.

Baseline before the change (recorded 2026-11-13):

    Memory Clock      1,200 MHz   DDR4-2400    FCLK 1,200   UCLK 1,200   (1:1, but slow)
    Core clock        4,000.0 MHz flat under 12-process load, 105-108 W, Tctl 74.9 C
    Frequency Limit   4,440-4,625 MHz  (never reached)
    CPU 2B ternary    prefill 52.6 tok/s, decode 27.4 tok/s  (:8620)
    Fabric            9.27 items/s latency tier

---

## 1. DOCP — memory at its rated 3,200

    Del / F2 at boot  ->  F7 (Advanced Mode)

    Ai Tweaker
      Ai Overclock Tuner ............ D.O.C.P.
      Memory Frequency .............. DDR4-3200 (from the D.O.C.P. profile list)
      DRAM Voltage .................. 1.35 V   (Auto usually sets it — verify, do not leave 1.20 V)
      FCLK Frequency ................ 1600 MHz (must be 1:1 with the memory clock)

Expected after: Memory Clock 1,600, FCLK 1,600, UCLK 1,600.
Read the profile name on screen — the exact SPD timings (likely 16-18-18-38) were not readable from the OS.

## 2. Core Performance Boost / PBO — the +11-16 % ceiling

    Advanced -> AMD CBS -> CPU Common Options -> Core Performance Boost = Enabled
    Ai Tweaker -> Precision Boost Overdrive = Enabled
        (or  Advanced -> AMD Overclocking -> Precision Boost Overdrive = Enabled)

Expected after: core clock under load above 4,000 MHz, up to the 4,440-4,625 MHz limit.

## 3. Save

    F10 -> Save & Exit

## 4. Before you enter the BIOS

    STOP_TRM_PLATFORM.bat        # verified: "every port re-probed and silent", rc=0
                                 # VRAM drops 10.8 GB -> 1.6 GB; do not flash or retrain with the GPU occupied

Nothing in the stack survives a reboot on its own; `START_TRM_PLATFORM.bat` brings it back afterwards.

---

## If DOCP does not train (no POST / returns to 2,400)

1. Power off, hold Clear CMOS / short CLRTC for 10 s, POST at defaults.
2. Re-enter BIOS, try DDR4-3000 or 3,133 instead of 3,200 (keep FCLK at half the memory clock: 1,500 / 1,566).
3. BIOS 1804 predates several AGESA memory updates; a BIOS flash is the next lever but is riskier than a profile
   change — an explicit decision, not a default.

## After the reboot, from Windows

    START_TRM_PLATFORM.bat       # council, models, Jarvis, UIs
    python tools/hw_verify.py

Pass conditions:

    docp_3200     Memory Clock >= 1,500
    fclk_1to1     FCLK == Memory Clock (1:1)
    boost_active  core clock > 4,001 MHz under load
    stable        zero WHEA Error/Critical events since boot

Then compare to the baseline: prefill 52.6 tok/s, decode 27.4 tok/s, fabric 9.27 items/s.

## Rollback

    Ai Overclock Tuner = Auto        (memory back to 2,400)
    Core Performance Boost = Auto    (back to 4,000 flat)
    PBO = Auto / Disabled
    F10, save, reboot.
