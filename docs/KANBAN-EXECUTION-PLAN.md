# Platform Kanban Execution Plan (2026-11-23)

**Source:** Live `GET /kanban/board` on `http://127.0.0.1:8022` (`19` total cards: `14 DONE`, `5 IN_PROGRESS`).  
**Monad Journal Record:** Article 282 (`papers/paper_23_kanban_board_execution_plan.md`, `phoenix_journal_audit_manifest.json`).

---

## Active `IN_PROGRESS` Tasks (5)

| Order | Card ID | Priority | Assignee | Title | Current Measured State | Acceptance & Close Criteria |
| :---: | :--- | :--- | :--- | :--- | :--- | :--- |
| **1** | `TASK-CAD-01` | `critical` | `Devo` | Monthly UEFI Kernel & Boot Media Re-Flash | `G:\` (`SWARM_OS` FAT32) + `trmboot_pe.efi` present; no automated attestation run recorded this cycle | Run `trmboot/make_usb_bootable.py --check G:`, verify SHA-256 parity between built `.efi` / `host_council_server.exe` and `G:\` boot media, write `data/uefi_boot_attestation.json`, move `TASK-CAD-01` to `DONE`. |
| **2** | `TASK-CAD-07` | `high` | `Shield` | Platform Anomaly Sentinel Telemetry Radar | `\SwarmKernel` (`tools/swarm_kernel_guard.ps1`) active across 7 listeners; `/system/health_matrix` = `OPTIMAL` (`0` anomalies) | Verify `swarm_kernel_guard.ps1` fast-path + `/system/health_matrix` `anomaly_count == 0`, document in Monad Journal, move `TASK-CAD-07` to `DONE`. |
| **3** | `TASK-USR-0019` | `high` | `Bridge` | PBFT quorum arithmetically unreachable (`67` vs `2/5`) | **Resolved in code/config (`fc07159`):** `/council/federation/nodes` now computes dynamic `threshold: 3` over `3/3` active council peers (`quorum_reachable: true`) | Add a `kanban_close.py` rule for `PBFT quorum arithmetically unreachable` checking `byzantine_quorum.quorum_reachable == true`, run `kanban_close.py` to close `TASK-USR-0019`. |
| **4** | `TASK-CAD-03` | `medium` | `Hermes` | 100-Node Federation Byzantine Health Audit | Legacy seed title claims 100 nodes; real platform runs `3/3` `:8022` council servers + `2` Tailscale companion clients | Document the honest 3-server / 5-Tailscale-mesh audit result in Monad Journal and retire/close `TASK-CAD-03`. |
| **5** | `TASK-USR-0017` | `critical` | `Verify` | Wet-lab provenance audit reports 5 defects (`TRM-ABX-220`, `219`, `082`) | `TRM-ABX-220` is intentionally `REVISED_UNRESOLVED_PENDING_STRUCTURE_AND_EXPERIMENT` (no SMILES); `219` & `082` potency windows understated vs `eskape_spectrum_full.json` | Reconcile `219`/`082` ESKAPE windows in summary docs against `eskape_spectrum_full.json`, keep `TRM-ABX-220` explicitly withheld (`0` unacknowledged defects), verify `audit_wetlab_provenance.py`, then close `TASK-USR-0017`. |
