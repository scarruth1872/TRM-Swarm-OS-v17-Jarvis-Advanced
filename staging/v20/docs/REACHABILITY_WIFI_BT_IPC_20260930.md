# Wi-Fi, Bluetooth, or IPC: what this machine actually has (2026-09-30)

The question was whether the booted machine can be reached over Wi-Fi or Bluetooth, or over some IPC that
needs no network at all. Everything below is measured on this host or read out of the kernel that boots
the drive; nothing is assumed from a datasheet.

## The machine, by its own identifiers

Windows on this laptop -- the same machine that boots the drive -- reports:

| what | identifier | state |
|---|---|---|
| Wi-Fi | `PCI\VEN_8086&DEV_2723&SUBSYS_16541A56&REV_1A` -- Killer Wi-Fi 6 AX1650x 160MHz (200NGW) | Up, 600 Mbps |
| Ethernet | `PCI\VEN_10EC&DEV_8125&SUBSYS_0A6F1028` -- Realtek Gaming 2.5GbE | **Disconnected** |
| Bluetooth | `USB\VID_8087&PID_0029` -- Intel Wireless Bluetooth | OK (PAN adapter present) |

Two things follow directly.

**The cable is not in the port.** `VEN_10EC&DEV_8125` is the RTL8125B the metal boot bound as `eth0` and
reported `Link is Down`; Windows calls the same controller `Disconnected`. So "Link is Down" was the truth
about a cable, not the missing `rtl_nic/rtl8125b-2.fw` (which is now in the image anyway). That is the
whole story of the wired link, measured in the other operating system.

**The radios are known hardware, not a guess.** `DEV_2723` is the Intel AX200 (the "cc" generation,
`REV_1A`), the Wi-Fi half of the same combo card whose Bluetooth half is `8087:0029`. This is why the
firmware below can be named exactly rather than by family.

## What the kernel that boots the drive already carries

Read from `/proc/config.gz` and from the module tree under `/lib/modules/6.6.114.1-microsoft-standard-WSL2`,
which is the kernel this pathway boots:

| stack | what exists | size |
|---|---|---|
| 802.11 | `CONFIG_CFG80211=m`, `CONFIG_MAC80211=m` | 2,162,672 B + 2,175,584 B |
| Intel Wi-Fi | `CONFIG_IWLWIFI=m`, `CONFIG_IWLMVM=m` | 900,848 B + 1,198,416 B |
| Bluetooth | `CONFIG_BT=m`, `BT_HCIBTUSB=m`, `BT_LE=y` | 135,120 B (`btusb`), 105,456 B (`btintel`) |
| Bluetooth networking | `CONFIG_BT_BNEP=m` (IP over BT), `CONFIG_BT_RFCOMM=m` (serial over BT), `CONFIG_BT_HIDP=m` | 61,760 B, 195,624 B, - |
| USB Ethernet | `CONFIG_USB_NET_CDCETHER=m` (class-compliant dongles) | 35,816 B |
| USB serial | `CONFIG_USB_SERIAL=m` + `CH341=m`, `CP210X=m`, `FTDI_SIO=m`, `PL2303=m` | 36,256 / 61,960 / 133,584 B |
| IP over a line | `CONFIG_PPP=y` + `PPP_ASYNC=m`, `CONFIG_TUN=m`, `IPIP=m`, `GRE=m` | 27,888 B, 125,112 B |
| firmware loader | `CONFIG_FW_LOADER=y`, `_USER_HELPER=y`, `_COMPRESS_XZ=y` | -- |
| regulatory | `CONFIG_CFG80211_REQUIRE_SIGNED_REGDB=y`, `CFG80211_CRDA_SUPPORT=y` | -- |

Two absences matter as much as the presences: **`CONFIG_USB_GADGET` is not set**, so this laptop cannot
present itself as a USB device to another computer; and **`CONFIG_SLIP` is not set**, so IP over a serial
line has to go through PPP or TUN rather than the old in-kernel SLIP path.

`REQUIRE_SIGNED_REGDB=y` is the detail that turns a missing file into a silent half-failure: cfg80211 will
not use an unsigned regulatory database, so `regulatory.db` and `regulatory.db.p7s` are *mandatory* for
Wi-Fi, not optional. `FW_LOADER_COMPRESS_XZ=y` is the detail that pays for the rest: staged firmware may be
xz-compressed, which is how a 6.4 MB module set and a 1.37 MB firmware blob stay small.

## The firmware, named by the things that ask for it

Not from a table: the string is inside the module that will request it.

- **Wi-Fi.** `grep -a -o "iwlwifi-cc-a0-[0-9]*" iwlwifi.ko` returns `iwlwifi-cc-a0-77`. The file lives at
  **`intel/iwlwifi/iwlwifi-cc-a0-77.ucode`** -- 1,368,100 B, fetched HTTP 200 from linux-firmware. It is not
  at the repository root: the root name is a *symlink* into `intel/iwlwifi/` (`WHENCE`, the tree's own
  manifest, lists both `File: intel/iwlwifi/iwlwifi-cc-a0-77.ucode` and
  `Link: iwlwifi-cc-a0-77.ucode -> intel/iwlwifi/iwlwifi-cc-a0-77.ucode`), which is why fetching the root
  path returns 404 while the real path returns the blob. The kernel asks for the root name; stage it under
  that name (or carry the symlink) and the driver finds it.
- **Bluetooth.** `btintel.ko` builds the name from a template, `intel/ibt-%u-%u.%s`. For this card that is
  **`intel/ibt-20-1-3.sfi`** -- 801,016 B, HTTP 200 -- with **`intel/ibt-20-1-3.ddc`** (9 B) beside it.
- **Regulatory, which is the one that would fail quietly.** `regulatory.db` and `regulatory.db.p7s` are in
  neither linux-firmware's root nor its manifest (measured: no manifest entry; 404 at the root and under
  `wireless-regdb/`). They are published by wireless-regdb instead -- kernel.org lists
  `wireless-regdb-2026.09.03.tar.xz`. Because `REQUIRE_SIGNED_REGDB=y`, their absence is not a warning the
  system can shrug off.
- `iwl-debug-yoyo.bin` is optional debug firmware and is likewise absent from the manifest; nothing needs
  it to associate.

## The paths, cheapest first, with what each actually costs

**0. A cable.** Free, and measured to be the entire problem: nothing is in the port. The driver already
bound `eth0` on the metal boot, and the emulated proof returned the dashboard over that NIC
(`GET :15183/ui/ -> HTTP 200`, `<title>TRM SWARM OS | LCARS INTERFACE</title>`). Nothing needs building.

**1. A USB-Ethernet dongle.** `CONFIG_USB_NET_CDCETHER=m` is present (35,816 B), so a class-compliant
(CDC-ECM) dongle gives a live IP path with one module, no supplicant and no firmware. Note the caveat the
config implies: `CONFIG_USB_RTL8152` and `CONFIG_USB_NET_AX88179` are *not* set, so a dongle built on those
very common chipsets would not bind -- pick a CDC-ECM one, or add those modules.

**2. A USB-serial dongle -- the strongest IPC answer.** CH341 / CP210x / FTDI / PL2303 all have modules here
(36 KB to 134 KB). A $5 dongle gives a channel that exists with no network stack at all: the boot log can be
teed to `/dev/ttyUSB0` so the operator watches the boot live from another computer, and with `CONFIG_PPP=y` +
`ppp_async` (or a TUN device) the same wire can carry real IP -- the classic serial link, no 802.11 and no
supplicant involved. This is the one path that works when both the cable and the router are out of reach.

**3. Wi-Fi -- the driver, firmware and regulatory database are staged; the userspace is what is left.** The
kernel has the entire Intel stack (`cfg80211` + `mac80211` + `iwlwifi` + `iwlmvm`, 6.4 MB raw, ~2 MB
compressed) and the listeners already bind `0.0.0.0`, so the platform needs no change at all. Six modules,
`iwlwifi-cc-a0-77.ucode` and the signed `regulatory.db` pair are in the image (verified by content, and the
boot loads all 22 modules with `0 not shipped` -- see the pathway doc, section 19). A radio is still not a
network: **a supplicant** remains the one real cost, because association and the WPA2 four-way handshake are
userspace work.

**4. Bluetooth -- present in the kernel, blocked in userspace.** `btusb` + `btintel` + `BT_LE=y` would give
HID; `BNEP` (IP over BT) and `RFCOMM` (serial over BT) are both compiled as modules. But BNEP and RFCOMM
both need **BlueZ** -- D-Bus, pairing, the PAN daemon -- which is a far larger userspace project than a
supplicant, for a slower link. And BT-HID would only add a keyboard: the machine already has input, measured
on metal when the Logitech receiver enumerated. Verdict: not the cheap path here, and not the one to build.

## IPC: what the platform already exposes, and what it does not

Measured by grep across `host_council_server/src/` and `host_supervisor/src/`: **no unix socket, no FIFO,
no shared memory.** The platform's local surface is HTTP on loopback -- which is exactly what the boot's own
processes use: the supervisor probes `http://127.0.0.1:5183/ui/` every pass and PID 1 probes the routes, both
of which are visible in the metal boot's log. So a local client needs no new surface; it speaks HTTP to
`127.0.0.1:8022` like everything else on the machine does.

That leaves the channels that work when there is no network at all, in order of what they cost:

1. **The volume itself.** The state image carries the boot log and every receipt this campaign has travelled
   this way -- write on the machine, read on any computer, no interface between them. It is also the only
   channel that works while the machine is *off*, which is why it is how the receipts arrive.
2. **USB serial** (above): live, bidirectional, and it can carry IP if asked.
3. **The console.** Measured working on metal: the machine's own screen and keyboard, with the boot log tee'd
   into the state image.

`CONFIG_TUN=m` is the piece that makes the general case cheap: any channel that can move bytes -- serial,
Bluetooth RFCOMM, even a file on the volume -- becomes an IP link with a small userspace tunnel, and the
kernel's own `0.0.0.0` listeners then serve the dashboard over it unchanged.
