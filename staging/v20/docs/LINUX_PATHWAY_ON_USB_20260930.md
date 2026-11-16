# This drive boots two things. Today it boots the first.

## What starts when you power on with this drive attached

Firmware → `EFI/BOOT/BOOTX64.EFI` → the **chainloader** → `EFI/TRM/vmlinuz` + `EFI/TRM/initrd.img`
→ the Linux kernel → `/init` (ours) → the Rust platform on **:8022** and its dashboard on **:5183**.

The platform is not in RAM: the kernel mounts `rootfs.squashfs` (read-only, 16,461,824 B) and joins it
with a writable `trm-state.ext4` (64 MB) over overlayfs, so the flash is never modified where the
platform is concerned and runtime state still survives a reboot. **The state is written to this stick,
in `trm-state.ext4`** — pull it and you lose the machine's memory, not the platform.

Measured, under QEMU/OVMF, with the volume on a USB stick and no other disk attached:

| build | what the guest saw |
|---|---|
| no kernel modules staged | `BLOCK DEVICES: sr0` → no volume → `NOTHING answered on :8022` |
| modules staged | `BLOCK DEVICES: sda sr0` → `sda mounted at /mnt/esp (rootfs.squashfs 16449536 B)` → **18 routes answered** |
| rebuilt platform (2026-09-30, the store round) | `sda mounted at /mnt/esp (rootfs.squashfs 16461824 B)` → **19 routes answered**; the guest reports its tercode store as `"vectors":2180`, `"serves_the_file_on_disk":true`, `"stale":false`, `"identical":true` |
| the same boot, rebuilt platform (2026-09-30, the L3 round) | `sda mounted at /mnt/esp (rootfs.squashfs 16457728 B)` → 200 on `/sram/architectures`, and the guest's own answer is in the section below |

The kernel shipped here is a VM kernel: it carries `ahci`/`nvme`/`sd_mod` and **no USB stack at all**,
so the ten modules `init` loads (`usb-common`, `usbcore`, `xhci-hcd`, `xhci-pci`, `ehci-hcd`,
`ehci-pci`, `usb-storage`, `hid`, `usbhid`, `hid-generic`, staged in `/lib/modules` of the RAM image)
are what make this drive readable *by the kernel that booted from it*. `init` prints which loaded:
`MODULES: ... -> 10 loaded, 0 not shipped`.

The boot volume is chosen by **content** — the first volume found that actually carries
`rootfs.squashfs`. It is not chosen by device name and not by being first: a PC with an internal disk
has another FAT partition, its own ESP, which mounts happily and carries no platform. Proven in the
sandbox with a decoy ESP attached first: `sda vfat but carries no rootfs.squashfs; sdb mounted`.

## The L3 geometry this drive's platform reports

`/sram/architectures` no longer quotes the module's `L3_SIZE = 32 MB` literal. On the machine it is running
on, it asks the machine: `host_probe::l3_cache_kb()` sums the unified L3 over the cache instances the host
reports (on Linux that is `/sys/devices/system/cpu/cpu0/cache/**`, distinct instances told apart by
`shared_cpu_map`), derives cells/sets/transistors from the size it actually got, and says which of the two
it served. A boot of this drive answers with the **guest's own** L3, from the boot's own voice:

    "l3":{"cells":134217728,"declared_default":33554432,"host_l3_matches_design_target":false,
          "l3_basis":"measured: host_probe.rs -- unified L3 summed over the cache instances the host
          reports, which is not the same as one instance's size; cells, sets and transistors derived
          from it","line_size":64,"sets":4096,"size_bytes":16777216,"slices":4,"transistors":805306368,"ways":16}

`declared_default` stays beside the measurement, and `host_l3_matches_design_target: false` is the honest
reading: the module's 32 MB is a documented Zen 2 design target, and this machine is not that part. When a
target cannot be probed the block says `assumed:` instead of `measured:` rather than presenting the
declaration as a reading.

## The stores this drive carries (rebuilt 2026-09-30)

The two packed stores had been built when the corpus held 180/186 documents, and the platform kept its
first read of them for the life of the process. Both are fixed and re-proven here:

    swarm_v2_nand_fabric.turbovec   2,180 vectors (was 180)   525,297 B   sha256 f0b31c4451e2fd70…  (re-built 2026-10-11)
    swarm_v2_nand_fabric.tercode    2,180 rows    (was 186)   505,545 B   sha256 6dc828a00f2b4b1d…
    swarm_v2_nand_fabric.text.jsonl 2,180 records (was 180) 3,168,214 B   sha256 bced5bd1f14ced57…  (relational tier fixed + 10 docs re-embedded, 2026-10-11)

The guest's own voice, from this boot (`backups/ternary_stores_20260930/usbstick_storereload_20260930.log`):
`STORES: swarm_v2_nand_fabric.turbovec (525297 B) … tercode (505545 B) … text.jsonl (1869777 B)`, then
`/k3/retrieve?…&engine=tercode&verify=1` answering `"scored_documents":2180`, `"vectors":2180`,
`"file_bytes":505545`, `"serves_the_file_on_disk":true`, `"stale":false`, `"identical":true`.

`?engine=packed&verify=1&alphabet=trit` still reports `"max_abs_diff":31.0`, and that is structural rather
than open: a min..max quantized 4-level code cannot carry the engine's zero trit, so its integer sum is not
the engine's. The store written from the engine's own trits (`.tercode`, 2 bits per trit) is the one that
reproduces the engine exactly, and the route now says so in `basis` instead of serving a bare `DIVERGED`.

## How to go back to the UEFI kernel

The UEFI-kernel build is still on this drive, byte-identical, three times over
(`EFI/TRM/trm_uefi_kernel.efi`, `EFI/TRM/trmboot_pe.efi`, `EFI/TRM/BOOTX64.uefi_kernel.efi`,
all `386d6aeb84a44058…`, 16,422,400 B). Restoring the previous boot is one copy:

    cp EFI/TRM/BOOTX64.uefi_kernel.efi EFI/BOOT/BOOTX64.EFI

## What is on the drive

    EFI/BOOT/BOOTX64.EFI            the chainloader (3,072 B) -- the entry point
    EFI/TRM/vmlinuz                 the Linux kernel (16,023,552 B)
    EFI/TRM/initrd.img              our init + the platform's supervisor + the kernel modules
    rootfs.squashfs                 the read-only platform image (16,461,824 B, sha256 df686a7d23901f8b…)
    EFI/TRM/initrd.img              our init + the kernel modules (945,263 B, sha256 a16bf898bf087c12…)
    trm-state.ext4                  the writable layer -- THIS is where state lives (64 MB)
    swarm_v2_nand_fabric.*          the corpus: .k3, .tercode, .text.jsonl, .turbovec
    swarm_v2_artifacts/             kernel_knowledge_index.json
    host_council_server.exe, swarm_rust_kernel.dll, trm_swarm_boot.exe
                                    the Windows-side build, unchanged and unused by this pathway

## What is not proven

- **It has since been booted on physical hardware** (2026-09-30). The firmware started the chainloader
  from the stick, the console printed PID 1's voice on the board's own screen, all 16 modules loaded, and
  the NIC bound to the machine's RTL8125B -- all measured on metal, see section 18 of
  `docs/BARE_METAL_LINUX_PATHWAY_20260930.md`. That boot also found the volume race this document's
  proofs had never exercised (an internal NVMe is "a disk" from the first instant, so the wait for one was
  over before it began); it is fixed in `docs/FIXES_DERIVE_20260925.md` section 9 and the drive carries the
  fixed image. What is still NOT measured on that board: the NIC link (`Link is Down`, and
  `rtl_nic/rtl8125b-2.fw` is not in the RAM image) and anything past the volume handover.
- **The screen depends on the firmware's mode, not on a missing driver** (corrected 2026-09-30 by reading
  the kernel that boots this drive: `6.6.114.1-microsoft-standard-WSL2`, `CONFIG_FB_EFI=y`,
  `CONFIG_FRAMEBUFFER_CONSOLE=y`, `CONFIG_VGA_CONSOLE=y`, `CONFIG_DRM_SIMPLEDRM` absent). The boot log
  shows `efifb: framebuffer at 0x80000000, mode is 1280x800x32` and `fb0: EFI VGA frame buffer device`, so
  there is a framebuffer for the console to attach to wherever the firmware sets a GOP mode. The serial
  log is the serial console's own copy, so it does not show the fbcon switch; the open question is what a
  given board's firmware hands over, not whether the kernel can drive it.
- **The network drivers are staged, and the address comes off the volume** (closed 2026-09-30). The
  kernel carries `e1000`/`e1000e`/`r8169`/`igb` as modules; the RAM image now ships those plus their
  chains (`realtek` for `r8169`, `i2c-algo-bit` for `igb`) -- `16 loaded, 0 not shipped`. PID 1 reads
  `/network.conf` off the volume (`ip=`, optional `gw=`/`iface=`), brings the interface up and applies
  it; with no `ip=` line it says so and stays on loopback rather than inventing an address. Measured on
  the emulated stick with a NIC attached: `NET: eth0 up (mac 52:54:00:12:34:56), 10.0.2.15/24, gw
  10.0.2.2`, and the platform answered `HTTP 200` on `/k3/status` to a request made from OUTSIDE the
  guest.
- **A boot leaves a receipt on the machine's state image.** PID 1 tees its whole console output to
  `/mnt/rw/trm_boot.log` -- the state image, when the volume carries one -- so a machine with no serial
  port and no framebuffer still says what it did. `trm_chainload/scripts/read_boot_log.sh <stick|image>`
  pulls it out from another computer without starting the machine (debugfs; the journal is replayed on a
  copy). Measured: 57,111 B carrying the boot from the banner to the routes that answered.
- **`pivot_root` is refused by this kernel** (`EINVAL` for every new root, measured in a fresh mount
  namespace), so the handover is `chroot` and the old RAM root stays mounted underneath.

## If it comes up with nothing

The boot says so itself, and each line names the step that failed:

- `MODULES: ... 0 loaded, 10 not shipped` → the RAM image was built without `--modules`.
- `BLOCK DEVICES: sr0` with no `sd*` → the kernel did not bring the USB stack up.
- `no volume carrying the platform image` → the stick mounted, but this is not the volume with
  `rootfs.squashfs` on it (an internal ESP looks exactly like that).
- `init: NOTHING answered on :8022` → no platform came up, whatever the lines above it say.
