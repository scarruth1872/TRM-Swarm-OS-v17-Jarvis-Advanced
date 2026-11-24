# Article 282 — Platform Kanban Board Execution Plan & Task Ledger

**Journal:** The Monad Journal of Autonomous Discovery (ISSN 2998-277X)  
**Article ID:** 282  
**Monad Anchor:** `0x4D4F4E4144000001`  
**Date:** 2026-11-23  
**Phoenix Status:** `CONFIRMED`

---

## Abstract

Following the provenance audit (`G233`–`G238`) and the closure of `TASK-USR-0014` (DDR scan persistence), `TASK-USR-0015` (DDR software parity scrub and 7-rule gate enforcement), `TASK-USR-0016` (Tailscale federation peer reconciliation), and the retirement of unbacked seed cards (`TASK-CAD-02, 04, 05, 06, 08, 09`), exactly **five** `IN_PROGRESS` tasks remain on the live platform Kanban board (`/kanban/board`). This article defines the concrete execution plan, empirical verification criteria, and closure path for all five tasks, beginning with **Task 1 (`TASK-CAD-01`)**.

---

## Task Inventory & Execution Plan

### Task 1 — `TASK-CAD-01` (Critical / `Devo`): Monthly UEFI Kernel & Boot Media Re-Flash
* **Evidence Target:** `trmboot/make_usb_bootable.py --check G:`, SHA-256 parity between workspace build artifacts (`trmboot/trmboot_pe.efi` / `trm_uefi_kernel.efi` and `host_council_server.exe`) and `G:\` (`SWARM_OS` FAT32).
* **Execution:** Run non-destructive boot compliance check and SHA-256 attestation, persist `data/uefi_boot_attestation.json`, and close `TASK-CAD-01`.

### Task 2 — `TASK-CAD-07` (High / `Shield`): Platform Anomaly Sentinel Telemetry Radar
* **Evidence Target:** `tools/swarm_kernel_guard.ps1` + `/system/health_matrix`.
* **Execution:** Verify all 7 supervised listeners (`8022`, `3000`, `3001`, `3002`, `5183` + Tailnet `8022`, `3001`) report `OPTIMAL` (`anomaly_count: 0`), then close `TASK-CAD-07`.

### Task 3 — `TASK-USR-0019` (High / `Bridge`): PBFT Quorum Unreachable (`67` vs `2/5`)
* **Evidence Target:** `/council/federation/nodes` `byzantine_quorum.quorum_reachable == true` (`threshold: 3`, `active_nodes: 3`).
* **Execution:** Add a verified `kanban_close.py` predicate for `"PBFT quorum arithmetically unreachable"` and close `TASK-USR-0019`.

### Task 4 — `TASK-CAD-03` (Medium / `Hermes`): Federation Byzantine Health Audit
* **Evidence Target:** `/federation/stats` (`3/3` active `:8022` council servers) + `tailscale status` (`5/5` mesh nodes active).
* **Execution:** Record the honest 3-server / 5-mesh audit outcome and close `TASK-CAD-03`.

### Task 5 — `TASK-USR-0017` (Critical / `Verify`): Wet-Lab Provenance Audit (`TRM-ABX-219`, `082`, `220`)
* **Evidence Target:** `audit_wetlab_provenance.py` + `eskape_spectrum_full.json` + `trm_abx_220_recalibration.json`.
* **Execution:** Align `TRM-ABX-219` and `TRM-ABX-082` potency window citations with `eskape_spectrum_full.json`, preserve `TRM-ABX-220` as `REVISED_UNRESOLVED_PENDING_STRUCTURE_AND_EXPERIMENT` (withheld, no fabricated SMILES), and close `TASK-USR-0017`.
