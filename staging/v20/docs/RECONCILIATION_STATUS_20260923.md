# TRM reconciliation - current status (2026-09-23)

Single source of truth for where the WSL reconciliation and the roster change stand. Supersedes the
per-step docs where they disagree; each of those now carries a correction banner.

## Live runtime (verified)

```
:8022  Native Rust Sovereign Kernel v19.001 | HEALTHY | python_active false
       /council/status -> total_agents 15, pbft_quorum 9, LogicMaster in the roster
       verify_council_roster.py --live -> RC 0, VERDICT: consistent (all 7 rosters at 15)
       deployed: host_council_server.exe 1,617,408 B sha256 7f4e9a9871cd908c
                 swarm_rust_kernel.dll   727,552 B sha256 97624ea35ff51f86
```

## USB boot drive (E:, SanDisk Extreme, 15.7 GB FAT32)

Updated in place - **not formatted**, because it was already the platform's boot drive (`EFI/BOOT/BOOTX64.EFI`
launched by `startup.nsh`, plus `EFI/TRM/`). Five files replaced, each hash-verified after the copy:

| file | before | after |
|---|---|---|
| EFI/BOOT/BOOTX64.EFI | 15,923,200 (Sep 16) | 16,390,144 |
| EFI/TRM/trm_uefi_kernel.efi | 15,923,200 | 16,390,144 |
| host_council_server.exe | 1,467,392 (Sep 20) | 1,617,408 |
| swarm_rust_kernel.dll | 726,528 | 727,552 |
| trm_swarm_boot.exe | 2,028,032 | 2,082,816 |

Rollback: `backups/usb_pre_r15_20260923_192506/` (the drive's previous files, hashed). The old boot image's
size matches the stale `target/.../deps/*.efi` from Sep 16 in the repo tree - that file was the last flashed
build, not clutter.

## Structured index

```
ethics      95,847 rows   all five subsets, every row with text
molecule 2,897,819 rows   complete ChEMBL 37 release, checksum-verified
clinical   201,572 rows   40 measurements x 400 patients
prose        1,633 rows
structure        9 rows
                507,248,640 B   sha256 2f5db380bb0859d9...
```

Source integrity: `chembl_37.sdf.gz` re-fetched to the published 931,156,420 B and sha256
`2cdf4d33...` (the previous copy was 78.8% of the file, which bounded reads had hidden).
Generator: `swarm_v2/core/build_structured_index.py`.

## v18 archive reconciliation

Frozen and archived (`backups/wsl_freeze_20260923_180202/`, commit `89d2d75`). Claims re-verified against
the recovered v17 references: `swarm-k3` **verified** (0 diffs, 4030 values x 3 epsilons), `swarm-council`
**faithful** (11/11 rows) though its rule cannot ever BLOCK, `swarm-ssm` **rule verified**, Motion B GPU
**identity verified** but its 177M/s **not reproduced** (measured 309 M/s GPU vs 1333 M/s CPU).

## Open

- EFI image: built and now on the USB; flashing to the internal ESP is a boot-time action.
- `swarm_v2_artifacts/swarm_v2_nand_fabric.{turbovec,text.jsonl}` are absent - the live fabric store sits at
  the repo root (`swarm_v2_nand_fabric.k3`, 76,761,697 B, Sep 10). Worth confirming which path the kernel
  loads before assuming the earlier re-embed touched the served store.
- Two `% 14` sites are deliberately unchanged (`job_id % 14` parser id, `(i % 14)` ring-buffer offset).


---

## Addendum - 2026-09-24 (tasks 463/466, vision tier, service restore)

Everything above still holds. Added since:

- **Task 463** (3D reconstruction for protein structures) and **Task 466** (feature extraction for
  docking) were executed against real data - the platform's own 9 PDB structures and the full ChEMBL 37
  connection tables. Full numbers, methods and the two ruler faults found mid-flight:
  `docs/TASKS_463_466_AND_VISION_20260924.md`.
- **Tasks 451-460 were validated as generated artifacts, not executed work** (seeded RNG, hardcoded
  metrics, fabricated PMC ids). Tasks **461-467 had no artifacts at all**. Evidence:
  `docs/PLATFORM_TEST_461_467_20260923.md`.
- **A vision tier exists now**: Gemma-3-4B-it + mmproj on :8588, proven to read text from an image
  ("TRM 4271"), unreliable at counting colonies, and unable to coexist with tier-2 on this 8 GB GPU.
  Model files are on disk; the tier is stopped.
- **Service note:** a GPU OOM cascade killed :8022 and :5183 because the healing supervisor was not
  running. Both were restored by starting `swarm_supervisor.py`. Bring-up command and the full
  shutdown state are in `RESUME_HERE.md`.
