# trm_chainload — the bare-metal bridge

A Linux kernel **is** a UEFI application (measured: PE32+, machine 0x8664, **subsystem 10**), so firmware
can load one directly. This loader does the handoff and nothing else. From that point the machine is
running real Linux userspace, with the kernel's own driver stack — which is why every volume on the
computer becomes readable, not just the FAT stick.

See `docs/BARE_METAL_LINUX_PATHWAY_20260930.md` for the measurements and a boot log.

## Build

```bash
rustup target add x86_64-unknown-uefi
CARGO_TARGET_DIR=<out> cargo build --release --target x86_64-unknown-uefi
# -> <out>/x86_64-unknown-uefi/release/trm_chainload.efi
```

## The boot volume

```
\EFI\BOOT\BOOTX64.EFI   trm_chainload.efi      the bridge (this crate)
\EFI\TRM\vmlinuz        a Linux kernel           firmware-loadable as-is
\EFI\TRM\initrd.img     cpio.gz                  PID 1 and whatever the rootfs needs
```

The command line lives in `src/main.rs` (`CMDLINE`): `initrd=` is resolved by the kernel's EFI stub
through the firmware, so the loader passes the **path**, never a copy of the bytes.

## init/ — PID 1, and the platform it starts

`init/init.c` is the minimal PID 1 that proved the handoff reaches userspace: mounts `proc`/`sys` and
reports what the kernel can see.

`init/platform_init.c` is the one that runs the **full stack**. It mounts `proc`/`sys`/`devtmpfs`, brings
loopback up, starts the supervisor (which starts the platform from its own service table), and then
**asks the platform questions over HTTP and prints the replies** — a boot that printed a banner and nothing
else would prove nothing. It also writes `/dev/tty0`, because with two `console=` parameters a userspace
program's stdout goes to the last one and the screen would show kernel output only.

Besides `/infrastructure/status` and `/ui/`, it asks for `/parsers/status` and POSTs a small FASTQ payload
to `/parsers/fastq` through `http_req()` (raw `text/plain` body, which is the contract those routes take).
Those two are chosen as a **discriminator**: an earlier build of this same platform 404s `/parsers/*`, so a
`200` from inside the guest proves which build the firmware chain actually loaded, where a banner or a
`/health` 200 is produced by either image.

It then mounts the boot volume with the kernel's own driver, links the fabric stores (`turbovec`, `k3`,
`tercode`, `text.jsonl`) **and the 53 MB retrieval corpus**
(`swarm_v2_artifacts/kernel_knowledge_index.json`) from that volume to `/opt/trm/`, and asks the
questions that prove *which* build booted: the exact ternary path (which refused on this image's
predecessor), the tercode store, the trit alphabet, the packed path, and `/k3/status`.

The platform's `repo_relative()` walks up from the **running executable** (up to eight parents), so a
link has to sit within eight parents of `/opt/trm/trm_uefi_kernel/host_council_server/host_council_server`
— `/opt/trm` is three — and every store stays on the disk rather than being baked into the RAM image:
the point of the Linux pathway is that the kernel's driver stack is what makes the data reachable.

A **nested** rel path needs its directory first: `repo_relative` joins the rel path onto a candidate
ancestor, so `swarm_v2_artifacts/kernel_knowledge_index.json` is only found if
`/opt/trm/swarm_v2_artifacts/` exists for the link to resolve inside. The launcher creates each
intermediate directory and links **every** store the resolver looks for — wiring only the one its own
probe uses makes `/k3/status` report a store missing that is sitting on the disk.

Three things that boot taught, all now printed by the launcher: the device name is the **kernel's**
choice (so it enumerates `/dev` for block devices instead of assuming `sda`); `mount` returning `ENOENT`
can mean the *target* is missing — `/mnt` does not exist in a minimal initramfs, so `mkdir("/mnt")` has
to come first; and the corpus is 53 MB, so the volume image is sized to hold it.

`init/supervisor.services.conf` is the bare-metal service table: paths inside the image, the kernel
started by the supervisor, the dashboard declared with an empty argv (the kernel serves it).

## Building the rootfs

```bash
# the two binaries, cross-built from the host
cargo build --release --target x86_64-unknown-linux-musl   # host_council_server, host_supervisor
RUSTFLAGS="-C linker=rust-lld -C link-self-contained=yes"  # on a Windows host; rust-lld ships with rustc

# PID 1
gcc -static -O2 -Wall -Wextra -o platform_init init/platform_init.c

# the platform image: read-only, on the volume (mksquashfs, through WSL)
python scripts/make_rootfs.py --mode squashfs --platform <host_council_server> \
    --supervisor <host_supervisor> --dist ../dashboard/dist \
    --conf init/supervisor.services.conf --out rootfs.squashfs

# the RAM image: init and the mount points, nothing else -- the platform is not duplicated in RAM
python scripts/make_rootfs.py --init-only --init platform_init --platform <host_council_server> \
    --supervisor <host_supervisor> --dist ../dashboard/dist \
    --conf init/supervisor.services.conf --out initrd.img

# the writable layer: a small ext4 image beside the platform image (mkfs.ext4, through WSL)
python scripts/make_state.py --out trm-state.ext4 --size 64
```

Both images come out of one layout (`build()`), so a change to the root's shape moves both. An
existing state image is kept unless `--force`, because rebuilding it discards the state in it.

Then boot it under OVMF with the harness in
`backups/bare_metal_persistent_root_20260930/linux_boot.sh`. It takes `--reuse` to boot the volume
exactly as the last boot left it, which is the only way a persistence claim can be measured: a
harness that rebuilds the volume first erases the state the previous boot wrote, and then every run
looks identical whatever the layer does.

## The kernel modules, and the drive it boots from

The kernel this pathway ships is a VM kernel: `ahci`, `nvme` and `sd_mod` are built in, and the USB
stack is not. Every QEMU boot so far was handed its disks as AHCI, so nothing noticed. On a real
machine the volume the firmware booted from is behind a USB port, and a kernel that cannot drive one
finds no disk at all.

```bash
# collect the modules the launcher's own mods[] table names, out of the kernel tree
python scripts/make_modules.py --launcher init/platform_init.c --out <dir>

# stage them flat into /lib/modules of the RAM image; init loads them before it asks /dev
python scripts/make_rootfs.py --init-only --modules <dir> --init platform_init \
    --platform <host_council_server> --supervisor <host_supervisor> --dist ../dashboard/dist \
    --conf init/supervisor.services.conf --out initrd.img
```

One list, in `mods[]`, read by the collector -- so a module added there is staged by the next pack
with no second list to drift. And what the boot waits for is the **volume**, not a disk (`find_volume()`):
on a machine with an internal NVMe "a disk" is present before the boot starts, so waiting on it is over
before it begins, and the stick -- behind the USB modules just loaded -- arrives a second or two later.
That is the failure the first physical boot hit (`docs/FIXES_DERIVE_20260925.md` section 9). A boot prints what loaded (`MODULES: ... -> 16 loaded, 0 not shipped`),

warns if a *device that is coming* is not there yet, and picks its volume by content
(`rootfs.squashfs` on it), never by device name: an internal disk's own ESP is vfat, mounts happily,
and carries no platform. The last line of a boot is counted from the routes that answered, so an
image that carries no platform cannot report itself up.

The table is not only the USB stack. The kernel carries the NIC drivers as modules too
(`CONFIG_E1000=m`, `CONFIG_E1000E=m`, `CONFIG_R8169=m`, `CONFIG_IGB=m`), so a machine booted without
them is loopback-only: the platform runs and nothing can reach it. They are in the same `mods[]` table,
in `modprobe --show-depends`' own order (`realtek` before `r8169`, `i2c-algo-bit` before `igb`) -- 16
modules, which is a 1.38 MB RAM image rather than a 945 KB one.

The radio is the same story one layer up. `iwlwifi`, `iwlmvm`, `cfg80211`, `mac80211`, `libarc4` and
`led-class` are in that same table -- names taken from `modprobe --show-depends iwlmvm`, not written by
hand -- and their firmware is staged beside them:

```bash
python scripts/make_rootfs.py --init-only --modules <dir> --init platform_init \
    --firmware ../trm_chainload/firmware --wifi <trm_wifi> --out initrd.img
```

That is 22 modules and four firmware files (the Intel image, plus the *signed* regulatory database, which
is mandatory here because `CONFIG_CFG80211_REQUIRE_SIGNED_REGDB=y`) -- a 3.9 MB RAM image. And a radio
with a driver, firmware and a regulatory database is still not on a network: association is userspace.
`--wifi` stages the supplicant as `/trm_wifi`, and PID 1 runs it **before the handover**, while the RAM
image is still the root -- run it after and the path no longer exists, which is a bug this pathway has
already had. It prints what the kernel answers (`wifi: nl80211 is family 33 (asked of the kernel, not
assumed)`) and, on a machine with no radio, exactly that (`no wiphy is registered`) rather than a silence.

## The address, and the receipt a blind boot leaves

A headless machine has no way to say what it did: no serial port, and if the firmware hands over no
framebuffer, no screen. Two things close that:

- **`network.conf` on the volume** -- `ip=192.168.1.50/24`, optional `gw=` and `iface=`. PID 1 reads it
  before the handover, brings the interface up and applies it, and prints what it applied. With no `ip=`
  line it keeps the loopback-only state and says so; it never invents an address for a network it cannot
  see. The file is read at its own size (a 512-byte window over a 1073-byte file lost both settings and
  reported `511 B`, which is how that was found).
- **the address it gives itself when none is configured**: `network.conf` with no `ip=` line leaves a
  machine with no name to be reached at, so one is derived from its MAC in 169.254/16 (the range that means
  "no router answered") -- stable across boots, labelled as derived rather than probed, and printed. That
  reaches a directly-cabled machine and nothing on a routed LAN; setting `ip=` is what puts it on your own
  network. The last lines of a boot say where the dashboard is, on both counts:

      DASHBOARD: http://127.0.0.1:5183/ui/ on this machine, and http://169.254.53.87:5183/ui/ over the
                 network (eth0 is linked)

  and when the cable is the problem it says that instead, because "the platform is up" is not an answer to
  "where do I look at it".
- **the NIC's firmware**, staged into the RAM image: a driver that requests firmware when it binds needs
  the file in the root filesystem at that moment, and here that root filesystem is the RAM image. The
  board's RTL8125B asked for `rtl_nic/rtl8125b-2.fw` and got `failed with error -2`; the file now ships:

  ```bash
  python scripts/make_rootfs.py --init-only --modules <dir> --firmware ../trm_chainload/firmware \
      --init platform_init --platform <host_council_server> --supervisor <host_supervisor> \
      --dist ../dashboard/dist --conf init/supervisor.services.conf --out initrd.img
  ```

  Staging is by directory, so a second NIC is a second file rather than a second mechanism.
- **`/mnt/rw/trm_boot.log`** -- the whole console output, on the writable layer, which is the state image
  when the volume carries one. Read it from another computer without starting the machine:

  ```bash
  trm_chainload/scripts/read_boot_log.sh /mnt/e/trm-state.ext4   # or a whole volume image
  ```

  It works on a COPY and replays the journal there: a machine stopped mid-boot leaves its journal
  unreplayed, and the raw image then refuses to open at all (`Inode bitmap checksum does not match`).
  PID 1 flushes the log explicitly at open (`fsync` on the file and on the state image) rather than
  leaving a receipt to the kernel's writeback schedule.

Boot it with the volume **on an emulated USB stick and no other disk** to exercise that path:
`backups/bare_metal_usb_stage_20260930/usb_stick_boot.sh <initrd.img>`. What the staging onto the
physical drive (and the way back to the UEFI kernel) looks like is in `docs/LINUX_PATHWAY_ON_USB_20260930.md`.

## Two traps this had to get right

1. **`LoadOptions` belongs to the new image**, not to `StartImage`. Set it on the new image's
   `LoadedImage` before starting it.
2. **A kernel loaded from a memory buffer has `device_handle = NULL`** — the firmware only sets it when
   `LoadImage` is given a device path. Without pointing it at the right volume the stub fails:
   `Failed to handle fs_proto` / `Failed to load initrd: EFI_INVALID_PARAMETER`.

And a third, in the command line itself: **a kernel command line is POSIX and must keep `/`**. Reusing the
EFI path encoder (which maps `/` → `\\`) turned `rdinit=/init` into `rdinit=\init`, so the kernel found
no init and panicked with `VFS: Unable to mount root fs on unknown-block(0,0)`.

## The root the platform runs on

`init` mounts the volume, finds `rootfs.squashfs` on it, attaches it to a loop device read-only and
mounts it squashfs read-only. The state image is loop-mounted as an ext4 writable layer and the two
are joined with overlayfs. `/dev`, `/proc`, `/sys`, the volume and both halves of the overlay are
moved into the new root before the handover, so the new root has the screen, the stores and the
volume. Then it writes a file through the overlay, reads it back, and checks the same path is still
absent from the read-only image underneath — so one line measures both the write and the read-only
guarantee.

- **`pivot_root` is not used here.** On this kernel it returns `EINVAL` for every new root: the mount
  table shows nothing shared or slave, the roots are on different filesystems, and a child in its own
  mount namespace is refused for a plain squashfs root as well as an overlay one. `chroot` is the
  handover that works, and the boot prints which of the two it used.
- **`upper/` and `work/` are created after the layer is mounted, not before.** A mount hides whatever
  sat at its mount point, so directories made earlier are not on the layer, and overlayfs reports the
  upperdir it cannot find as `ENOENT`.
- **A loop device is no more writable than the descriptor it was attached with.** The state image is
  attached `O_RDWR`; attaching it read-only and mounting ext4 read-write fails with `EACCES`.
- The volume is mounted read-only and re-mounted writable **only** when a state image is present, so
  the platform image cannot change. The boot says which of the two happened, because "state lives in
  RAM" is a fact an operator needs rather than an assumption to make.
