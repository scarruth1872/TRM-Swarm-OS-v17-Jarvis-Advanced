# USB boot-drive update — 2026-10-03

What changed on the stick and how to reproduce it.

## What was updated

- **`rootfs.squashfs`** — rebuilt with the current platform (the Oct-3 kernel carrying the native
  DDR antibody routes, the CVE fold, `/sram/execute`, and the antibody → drug-discovery bridge).
- **Runtime stores staged inside the image** at `/opt/trm/swarm_v2_artifacts/` (read-only, as on the
  metal): `DDR_ANTIBODIES.json`, `DDR_THREAT_ANTIBODIES.json`, `DDR_THREAT_DATABASE.json`,
  `DDR_CVE_SUMMARY.json`, `security_threat_index.tercode` + `.meta.json`,
  `antimicrobial_peptide_candidates.json`.
- **Docs** — `README.md` (top note), `docs/CHANGELOG.md`, `docs/DRUG_DISCOVERY_AND_SECURITY_ADVANCEMENTS.md`.

## What was NOT touched

- `EFI/TRM/initrd.img` — PID 1 (`platform_init.c`) and the kernel modules are unchanged since the
  Oct-1 boot. The metal kernel's `.ko` files live in a kernel tree that is not in this checkout, so
  the RAM image is not rebuilt; the store-linking list it carries (`swarm_v2_nand_fabric.*` +
  `kernel_knowledge_index.json`) is exactly what it still links.
- `trm-state.ext4` — the writable layer, left intact so nothing written on the last boot is lost.
- The fabric stores on the volume (`swarm_v2_nand_fabric.*`) and `swarm_v2_artifacts/kernel_knowledge_index.json`.

## Reproduce

```bash
# 1. cross-build the platform for the metal (musl, static) -- rust-lld ships with rustc
cd trm_uefi_kernel/host_council_server
RUSTFLAGS="-C linker=rust-lld -C link-self-contained=yes" \
  cargo build --release --target x86_64-unknown-linux-musl
cd ../host_supervisor
RUSTFLAGS="-C linker=rust-lld -C link-self-contained=yes" \
  cargo build --release --target x86_64-unknown-linux-musl

# 2. rebuild the image with the stores staged in (make_rootfs.py --stores is new)
cd ../..            # repo root
# 2a. build the framebuffer renderer (required — the machine's own screen shows the dashboard):
RUSTFLAGS="-C linker=rust-lld -C link-self-contained=yes" \
  cargo build --release --target x86_64-unknown-linux-musl   # in trm_uefi_kernel/trm_fb_dashboard
python trm_chainload/scripts/make_rootfs.py --mode squashfs \
  --platform trm_uefi_kernel/host_council_server/target/x86_64-unknown-linux-musl/release/host_council_server \
  --supervisor trm_uefi_kernel/host_supervisor/target/x86_64-unknown-linux-musl/release/host_supervisor \
  --dist dashboard/dist \
  --conf trm_chainload/init/supervisor.services.conf \
  --stores <flat dir of the runtime stores> \
  --dashboard trm_uefi_kernel/trm_fb_dashboard/target/x86_64-unknown-linux-musl/release/trm_fb_dashboard \
  --out rootfs.squashfs

# 3. copy the image to the stick (back up the old one first)
cp rootfs.squashfs /e/rootfs.squashfs
```

`mksquashfs` runs in WSL (it stages the tree into `/mnt/c/...`), so WSL must be available. gzip
compressor, `-all-time 0 -mkfs-time 0` for a reproducible image.

**`--dashboard` is not optional.** It stages `trm_fb_dashboard` — the framebuffer renderer that draws
the LCARS page on the machine's own screen (`/dev/fb0`) — at `/opt/trm/trm_fb_dashboard`. Without it
the dashboard serves only over HTTP and the metal screen stays on the console; the boot log ends
`DASHBOARD RENDERER: exec failed: No such file or directory`. The first 2026-10-03 refresh omitted it
(regression); the second, corrected build is 97,177,600 B.

## Verified receipts (from the running Windows-host build, same source)

- `/security/audit`: records_ingested 403,751 / cve_records 400,908 (critical 37,798 / high 152,403).
- discovery pipeline: 403,803 antibodies scanned → 345,425 viable → top 100 written (6.2 s).
- `/sram/execute`: dispatches all 15 virtual kernels.
