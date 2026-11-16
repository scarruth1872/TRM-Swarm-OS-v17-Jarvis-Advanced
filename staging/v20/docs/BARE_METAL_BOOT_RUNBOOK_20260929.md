# TRM Swarm OS — bare-metal boot runbook

Written 2026-09-29 for the image `a8f7a713954d974162e1cb862e3bcfb484b001de5da6735b4cce7d7c8d0deb48`
(16,414,720 B) on this drive. Everything below that is stated as a measurement was measured; the one
thing NOT proven on PHYSICAL hardware - but the boot IS now proven on real UEFI firmware: it was run under
OVMF (via QEMU in WSL) at both 1280x800 and 1920x1080, and the screens above were read back from the
framebuffer. Screenshots and the record are in
`backups/bare_metal_boot_verification_20260929/`.

## Before you boot

- **`\startup.nsh` runs `\EFI\BOOT\BOOTX64.EFI`.** That EFI image *is* the kernel this drive boots.
- The image is **unsigned** — disable Secure Boot, or the firmware will refuse it.
- Boot in **UEFI** mode (not legacy/CSM) and select this USB.
- Files the boot reads from the volume root:

  | file | size | used for |
  |---|---|---|
  | `\EFI\BOOT\BOOTX64.EFI` | 16,414,720 B | the kernel itself |
  | `\swarm_v2_nand_fabric.k3` | 76,761,697 B | the K3 fabric (mounted, size + first/last sectors read) |
  | `\swarm_v2_nand_fabric.text.jsonl` | ~3.17 MB, 2,180 records | the readable corpus that `ask` answers from |

The corpus is **searched for on every volume the firmware exposes**, not assumed to sit beside the boot
image. Whichever volume carries it is named on the boot line (`... B from FS0:`) - so on a machine where
the intelligence lives on another drive, the boot finds it there instead of reporting nothing.
  | `\host_council_server.exe` | 19,144,704 B | **reported, not loaded** — see limits |

- Rollback: the previous image is in `backups/usb_efi_20260929_corpus/` (this repo) and in
  `trmboot/`. `backups/usb_efi_pre_intelfix_20260929/` holds the pre-session image.

## What the screen should show

These lines are emitted from measurements, not literals:

```
LOGIC:  K3 FABRIC MOUNTED: 76761697 bytes = 153523394 trits (read from the boot volume)
LOGIC:  SECTORS: first+last 512B read OK
JARVIS: cognition online. Fabric geometry verified against 153.5M trits.
LOGIC:  CORPUS LOADED: 2,180 documents, 3168214 bytes from the boot volume
JARVIS: 'ask <query>' recalls from those documents.
KERNEL: host_council_server.exe 19144704 B, PE32+ x86-64
        Windows console - needs an OS
KERNEL: the boot image IS the EFI application (startup.nsh loads BOOTX64.EFI)
```

Any of those reading `NOT MOUNTED` / `NOT LOADED` is the honest negative — it means that step failed,
and it is the signal to investigate. A green line means it was read.

## Commands

| command | what it does |
|---|---|
| `drives` / `volumes` | every UEFI SimpleFileSystem volume, with label, size and free space |
| `ls <path> [offset]` | list a directory; 14 entries per page, footer gives the real total |
| `cat <file> [offset]` | read a file; offset is a **byte** offset, footer gives the next one |
| `ask <query>` / `jarvis <query>` | recall from the loaded corpus, naming the record it matched |
| `recall <query>` / `corpus` | same, explicit; bare `corpus` reports what is loaded |
| `nand` | the fabric's measured status on its own |
| `status` | agent health check |
| `council <motion>` | council deliberation |
| `monad <goal>` | truth anchor |
| `help` | the command list |
| `[0]`–`[9]` | jump to a sector; `setup` opens the provisioning wizard |

### Reading a whole directory or file

The LCARS feed shows 18 lines, so anything larger is **paged, not truncated**:

```
ls FS0:\             -> entries 1-14 of 137, footer: "ls FS0:\ 14" for the next page
ls FS0:\ 14          -> entries 15-28
cat FS1:\boot.log    -> BYTES: 0-3900 of 51230. Next: 'cat FS1:\boot.log 3900'
cat FS1:\boot.log 3900
```

The footer always states the true total, so you can tell how much is left. Volumes are addressed
`FS0:`, `FS1:`, … `FS12:` — multi-digit indices work.

## What "all drives and files" can and cannot reach

This is the honest boundary, and it is a property of the firmware, not of this kernel:

- Volumes appear only if the **firmware implements that filesystem**. UEFI implementations ship FAT12/
- FAT16/FAT32 drivers. **Most Windows volumes are NTFS and most Linux volumes are ext4 — the firmware
  normally has no driver for either, so `C:\` and Linux roots will not be listed or readable at bare
  metal.** USB sticks (FAT32) and the ESP are readable. If a volume does not appear in `drives`, the
  firmware cannot read it, and no amount of code in this kernel changes that.
- **Read-only.** Nothing here writes to any volume.
- Volumes with no filesystem (unformatted, or a filesystem the firmware cannot parse) either do not
  appear or report `[Volume Open Error]` — that is reported, not hidden.

## Verified on the boot (OVMF, 2026-09-29)

```
LOGIC: K3 FABRIC MOUNTED: 76761697 bytes = 153523393 trits
LOGIC: SECTORS: first+last 512B read OK
JARVIS: cognition online. Fabric geometry verified against 153.5M trits.
LOGIC: CORPUS: 2,180 docs, 3168214 B from FS0:
JARVIS: 'ask <query>' recalls from those documents.
KERNEL: host_council_server.exe 19144704 B, PE32+ x86-64 / Windows console - needs an OS
KERNEL: the boot image IS the EFI application (startup.nsh loads BOOTX64.EFI)
```

The operator terminal is drawn on **every** sector, including the setup wizard the machine boots into. It
previously existed only inside the sector arm, so on the screen the machine boots to the boot report and
the prompt were drawn nowhere - which is indistinguishable from the intelligence failing to load.

## Known limits

- **The host kernel cannot be loaded by firmware.** `host_council_server.exe` is subsystem 3 (Windows
  console) and links `tiny_http` + `serde_json`, so it is a std binary needing an OS. On bare metal the
  EFI image is the kernel. This is structural.
- **No boot has been performed with this image.** The file reads, the corpus load and the payload
  classification are verified in source and against the real files off-target; only real firmware can
  prove the UEFI calls.
- Builds are **not reproducible** — two compiles of identical source produce different hashes.
