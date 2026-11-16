# TRM Swarm OS v19 "Sovereign" (Private) — source document

Source document for NotebookLM. Compiled 2026-10-01 from the repository at C:/Users/shawn/Downloads/TRM-Swarm-OS-v19-Sovereign-Private-main/TRM-Swarm-OS-v19-Sovereign-Private-main. Every figure below is traceable to a file in that tree.

## What this system is

The system is a Rust HTTP platform — `trm_uefi_kernel/host_council_server`, a "sovereign" host-council server / microkernel — plus a bare-metal Linux pathway that boots it from a USB drive on real hardware.

Two listeners, defaults read out of `trm_uefi_kernel/host_council_server/src/main.rs`:

| thing | port | default in source |
|---|---|---|
| API | `8022` | `TRM_PORT` unset → `8022` |
| LCARS dashboard | `5183`, served at `/ui/` | `TRM_UI_PORT` unset → `5183` |

The dashboard is a static bundle whose HTML title is `TRM SWARM OS | LCARS INTERFACE`.

Routes the kernel serves (names read out of `main.rs`; this is not a recorded total): `/health`, `/system`, `/k3/retrieve`, `/k3/status`, `/k3/verdict`, `/k3/tercode/build`, `/parsers/status`, `/parsers/formats`, `POST /parsers/fastq`, `/council/status`, `/council/proposals`, `/council/members`, `/council/ledger`, `/infrastructure/status`, `/infra/health`, `/infra/history`, `/sram/architectures`, `/memory/stats`, `/memory/recent`, `/swarm/experts`, `/experts`, `/swarm/telemetry`, `/federation`, `/kanban`, `/journal/papers`, `/research/stats`, `/brain/status`, `/verification/*`, `/models`.

Three stores and one ledger are the substance:

- **A ternary (three-state) knowledge store.** The engine is an in-tree port of `swarm_v2_rust_kernel/src/ternary_knowledge_index.rs` into `trm_uefi_kernel/host_council_server/src/native_ternary.rs`: quantize by absmean, compare integer trits, return sentinels. `search` also reports `out_comparisons` — the work actually performed. Stores on the boot volume: `swarm_v2_nand_fabric.k3` (76,761,697 B), `.tercode` (2 bits per trit, 505,545 B, 2,180 rows), `.turbovec` (packed, 2-bit, 525,297 B, 2,180 vectors, dim 768), `.text.jsonl` (3,168,214 B, 2,180 records).
- **An append-only monad ledger**, `swarm_v2_artifacts/monad_ledger.jsonl` -- 212 blocks, sealed by a sealer that re-measures its evidence (section 5 below).
- **A logic anchor**: the Monad Truth Anchor is `0x4D4F4E4144000001`.
- **A supervisor** (`trm_uefi_kernel/host_supervisor`) starts the platform from its own service table and holds a single-instance lock on `127.0.0.1:8023`.

## The bare-metal pathway, end to end

```
UEFI firmware
  -> EFI/BOOT/BOOTX64.EFI   trm_chainload, 3,072 B
  -> EFI/TRM/vmlinuz        16,023,552 B
  -> EFI/TRM/initrd.img     3,898,581 B, sha256 1c6c5f93f00cd509100475eb9f0f2ff049cd5f30799a24deea54d556e8e96764
                            init + supervisor + supplicant + 22 kernel modules + 4 firmware files
  -> Linux kernel 6.6.114.1-microsoft-standard-WSL2
  -> /init  (PID 1 = trm_chainload/init/platform_init.c, gcc -static -O2, 846,488 B)
  -> host_supervisor -> host_council_server   (:8022 API, :5183 dashboard)
```

- **The kernel is itself a UEFI application.** Measured on the kernel file: PE32+, machine `0x8664`, subsystem **10 = EFI application**. So `trm_chainload` loads `vmlinuz` with `LoadImage` and passes a command line by setting `LoadOptions` on the **new image** before `StartImage`. The kernel's EFI stub opens the volume itself, so only the initrd's *path* is passed, never a copy of the bytes.
- **Kernel modules come from a table in the launcher.** `init/platform_init.c` holds a `mods[]` list; `trm_chainload/scripts/make_modules.py --launcher init/platform_init.c` collects exactly those modules, `make_rootfs.py --modules <dir>` stages them flat into `/lib/modules` of the RAM image, and PID 1 loads them with `finit_module` before it asks `/dev` anything. One list, read out of the code that consumes it, so it cannot drift. Staged counts recorded in the docs: **10** modules (USB stack), **16** (adding the six NIC modules), **22** (adding the six radio modules). The table itself holds exactly **22** names as measured today: usb-common, usbcore, xhci-hcd, xhci-pci, ehci-hcd, ehci-pci, usb-storage, hid, usbhid, hid-generic, realtek, r8169, i2c-algo-bit, igb, e1000, e1000e, libarc4, led-class, cfg80211, mac80211, iwlwifi, iwlmvm.
- **The volume is found by content, not by device name.** `find_volume()` asks `/dev` again every 250 ms for up to 40 s and accepts a volume only if `rootfs.squashfs` is on it; `block_devices()` enumerates `/dev` rather than assuming `sda`. A machine's own ESP is vfat and mounts cleanly, so it is probed and rejected. The boot prints which pass found the volume and how long it took.
- **A read-only platform image plus a writable layer, joined by overlayfs.** `rootfs.squashfs` (16,461,824 B, sha256 `df686a7d23901f8b…`) is attached to a loop device read-only and mounted squashfs read-only; `trm-state.ext4` (67,108,864 B = 64 MB) is loop-mounted ext4 as the overlay upper layer. The volume is mounted read-only and re-mounted writable **only** when a state image is present; the platform image itself is never modified. `pivot_root` is refused by this kernel (`EINVAL` for every new root), so `chroot` is the handover and the boot prints which of the two it used.
- **The boot log is written to the volume.** PID 1 tees its whole console output to `/mnt/rw/trm_boot.log` on the state image and `fsync`s it; `trm_chainload/scripts/read_boot_log.sh <stick|image>` pulls it out from another computer with `debugfs`, replaying the journal on a copy. Measured: **57,111 B** carrying the boot from the banner to the routes that answered.

## What has been proven on real hardware

Two physical boots took place on 2026-09-30 on the same board (`docs/BARE_METAL_LINUX_PATHWAY_20260930.md` §18). Every boot before them was QEMU/OVMF. The first physical boot reached the kernel and loaded all 16 modules but came up serving nothing (it found the machine's internal NVMe but never saw the USB stick, which arrived 2.65 s in and after PID 1's single scan). The volume race was fixed and the drive restaged, and the second boot on the same board is the fixed one. **The log below was read off the machine's own state image** — `backups/bare_metal_first_boot_20260930/metal_boot_log_off_the_stick.txt`, pulled from `trm-state.ext4` on the stick and read with `debugfs` without booting the machine.

What the fixed boot printed:

```
BLOCK DEVICES: nvme0n1p6 nvme0n1p5 ... nvme0n1 nvme0
nvme0n1p1 vfat but carries no rootfs.squashfs
(found on pass 7, 1500 ms in)
STATE: trm-state.ext4 (67108864 B) -> the volume is writable for it
PERSISTENT ROOT: ... write-probe wrote and read back identical
STORES: ... .tercode (505545 B) ... .text.jsonl (1869777 B)
SUPERVISOR: launched as pid 277
[t+03s] GET /infrastructure/status -> HTTP/1.0 200 OK
GET /ui/ -> HTTP/1.0 200 OK
init: 19 of the routes asked answered -- the platform is up; idling so the machine stays alive.
```

- **The volume was found on pass 7, 1500 ms in** — the machine's own ESP (`nvme0n1p1 vfat`) was probed and rejected first because it carries no `rootfs.squashfs`, which is the content rule working on real hardware.
- **The console works.** The board printed PID 1's voice on its own screen (`CONFIG_VGA_CONSOLE=y`, `CONFIG_FB_EFI=y`, `CONFIG_FRAMEBUFFER_CONSOLE=y`); an earlier revision had called the screen a possible blocker.
- **All 16 modules load on real silicon**: `16 loaded, 0 not shipped`. The machine's USB receiver enumerated (`HID 046d:c548 Logitech USB Receiver`) and the stick's own controller came up as `sda` (15.7 GB). **That boot was the 16-module image** -- the log's own line names usb-common through e1000e and stops, with no radio module and no `WIFI` line anywhere in it. The six radio modules were staged afterwards. **That board has since booted the 22-module image**, and the log says `22 loaded, 0 not shipped` — every module binds on the first attempt. **The radio bound and the firmware was accepted**: `NET: wlan0 up (mac c8:cb:9e:5f:0e:0b)` and the launcher's own report, `WIFI: wlan0 present (iwlwifi, mac c8:cb:9e:5f:0e:0b, link down) -- the driver bound and the firmware was accepted`. `link down` names a link state, not a failure: no access point was in range. That boot also took the first panic this code has ever raised (`WIFI PROBE: exited 101`), in the nl80211 attribute walk — a defect only a machine with a radio could reach.
- **The NIC binds to the board's own chip**: `r8169 0000:03:00.0 eth0: RTL8125B, b4:45:06:d4:2c:51` and `NET: eth0 up (mac b4:45:06:d4:2c:51)`. (An earlier photograph of the screen was transcribed as `4d`; the machine's own log says `d4`, and the log is the authority.)
- **The dashboard's own HTML was measured from outside the guest.** From the equivalent emulated boot with the port forwarded: `GET :15183/ui/ -> HTTP 200`, 566 B of `<!doctype html>` and `<title>TRM SWARM OS | LCARS INTERFACE</title>` reaching a client outside the guest, plus `GET :18022/infrastructure/status -> HTTP 200`.
- **Persistence was measured, not asserted.** Across two boots off the same volume every line of the two serial logs is identical except one field: a write from an earlier boot is `absent` on boot 1 and `PRESENT` on boot 2. The state image was then taken **out of the boot volume and read without mounting it**, from outside the guest (`debugfs -R "ls -l /upper/opt/trm" state.ext4`), showing `.write-probe` (55 B) and the post-pivot store links in the writable layer.
- **Route answers on metal.** The boot asks the platform questions over loopback and counts the ones that answered: `19 of the routes asked answered -- the platform is up`, including the five `/k3` routes on the exact ternary path.

    The count reconciles exactly: the boot's own log records **19 probes** -- 18 `GET` and one `POST`
    (`/parsers/fastq`) -- over 7 distinct paths (`/infrastructure/status`, `/ui/`, `/parsers/status`,
    `/k3/retrieve` four times with different engines and alphabets, `/k3/status`, `/sram/architectures`).
    All 19 answered. The route list in section 1 is longer because a list of the kernel's routes is not a
    list of what PID 1 probes: nothing failed to answer, and the deliberate refusals (`/aura` with 501
    `engine_required`, the Python-registry-only formats, `/council/<action>/<id>`) are simply not probed.

## The radio: staged, and the supplicant

`docs/REACHABILITY_WIFI_BT_IPC_20260930.md` records the machine's own hardware identifiers, read from Windows on the same laptop that boots the drive.

| what | identifier | state |
|---|---|---|
| Wi-Fi | `PCI\VEN_8086&DEV_2723&SUBSYS_16541A56&REV_1A` — Killer Wi-Fi 6 AX1650x 160MHz (200NGW) | Up, 600 Mbps |
| Ethernet | `PCI\VEN_10EC&DEV_8125&SUBSYS_0A6F1028` — Realtek Gaming 2.5GbE | Disconnected |
| Bluetooth | `USB\VID_8087&PID_0029` — Intel Wireless Bluetooth | OK (PAN adapter present) |

`DEV_2723` is the Intel AX200 (the "cc" generation, `REV_1A`) — the Wi-Fi half of the combo card whose Bluetooth half is `8087:0029`, which is why the firmware can be named exactly rather than by family. The Ethernet ID is the RTL8125B the metal boot bound as `eth0`.

**Kernel modules present** (from `/proc/config.gz` and the module tree of `6.6.114.1-microsoft-standard-WSL2`): `CONFIG_CFG80211=m`, `CONFIG_MAC80211=m` (2,162,672 + 2,175,584 B), `CONFIG_IWLWIFI=m`, `CONFIG_IWLMVM=m` (900,848 + 1,198,416 B); `CONFIG_BT=m`, `BT_HCIBTUSB=m`, `BT_LE=y` (`btusb` 135,120 B, `btintel` 105,456 B); `CONFIG_BT_BNEP=m` (IP over BT), `CONFIG_BT_RFCOMM=m` (serial over BT), `CONFIG_BT_HIDP=m`; `CONFIG_USB_NET_CDCETHER=m` (35,816 B). **Absent and load-bearing**: `CONFIG_USB_GADGET` (so the laptop cannot present itself as a USB device to another computer), `CONFIG_SLIP`, and `CONFIG_MAC80211_HWSIM` (so emulation has no radio).

**Firmware and the regulatory database.** `CONFIG_CFG80211_REQUIRE_SIGNED_REGDB=y` makes the **signed** `regulatory.db` + `regulatory.db.p7s` mandatory, not optional; they are published by wireless-regdb (`wireless-regdb-2026.09.03.tar.xz`), not by linux-firmware. The Intel firmware name is read out of the module itself (`grep -a -o "iwlwifi-cc-a0-[0-9]*" iwlwifi.ko` → `iwlwifi-cc-a0-77`): `intel/iwlwifi/iwlwifi-cc-a0-77.ucode`, **1,368,100 B**. Bluetooth: `intel/ibt-20-1-3.sfi` (801,016 B) with `intel/ibt-20-1-3.ddc` (9 B). The NIC's own file `rtl_nic/rtl8125b-2.fw` is 3,328 B. Staged: **22 modules and four firmware files**; the boot reports `iwlwifi.ko iwlmvm.ko -> 22 loaded, 0 not shipped`, and the firmware files were verified byte-identical to the repository copies (read out of the initrd's own cpio). Emulation has no Intel radio, so that boot printed the honest absence:

```
WIFI: no wireless interface appeared in 10 s. The modules are loaded, so what did not happen is one of
      two things: this machine has no radio of that make, or its firmware was refused -- and the
      kernel's own lines above say which.
```

**Nothing in the boot chain needs a dynamic loader.** Measured out of the images themselves: `/init` (872,752 B) and `/trm_wifi` (702,376 B) in the initramfs, and `host_council_server` (19,396,056 B) and `host_supervisor` (744,264 B) in the platform image, are all `static` ELF files with no `PT_INTERP`, and the platform image contains **zero** loaders (`ld-linux*`/`ld-musl*`). So a missing `ld.so` cannot be the explanation for any exec failure in this pathway -- there is no dynamic linking anywhere in it to fail.

**The supplicant** is `trm_uefi_kernel/trm_wifi/` — dependency-free for the same reason the supervisor is (one `rustc` invocation, no crate graph), so the one primitive std does not offer (a netlink or `AF_PACKET` socket) is issued as a raw x86_64 syscall in `src/syscall.rs`. It is staged as `/trm_wifi` and run by PID 1 **before the handover**, while the RAM image is still the root. What it proves:

- **The crypto is diffed against an implementation that is not it.** `trm_wifi --vectors` prints every derived value for fixed inputs, and `trm_chainload/scripts/verify_wifi_crypto.py` recomputes them with Python's `hashlib`/`hmac` and an independent AES key-wrap. The output is a diff, not an assertion: **pmk, ptk, kck, kek, tk and mic all agree**, and a GTK wrapped by `python-cryptography` under the KEK the binary derived comes back out of `--unwrap` byte for byte. `--selftest` passes **6 of 6** published vectors: SHA1, HMAC-SHA1 RFC 2202, AES FIPS-197, and AES key-wrap RFC 3394 in both directions, including refusing a corrupted wrap.
- **The nl80211 layer was answered by a real kernel.** `trm_wifi --probe` resolves the `nl80211` generic-netlink family and asks for the wiphys and interfaces; run against WSL's own kernel it reports `wifi: nl80211 is family 33 (asked of the kernel, not assumed)` and `wifi: no wiphy is registered -- the modules are loaded, but no radio has bound`. Family 33 is the kernel's answer to a `CTRL_CMD_GETFAMILY` request, not a constant in the source; the zero wiphys is this machine's truth.
- **The nl80211 constants were derived, not remembered.** `trm_chainload/scripts/derive_nl80211_consts.py` walks `include/uapi/linux/nl80211.h` at v6.6 and prints them; the first draft from memory had `NL80211_CMD_NEW_KEY` and `NL80211_ATTR_EXT_FEATURES` both wrong.

- **The association is written, and its arithmetic is proven.** `src/eapol.rs` parses and builds the four frames and `src/supplicant.rs` sends `NL80211_CMD_CONNECT` and answers the handshake when the driver will not. Three rules are handled explicitly, each one a place a handshake fails in silence: the MIC is taken over the frame **with the MIC field zeroed**; the preshared key goes in `NL80211_ATTR_PMK` (254) -- the header is explicit that there is no `ATTR_PSK`, and the value it takes is the PMK itself, which is what `pbkdf2_sha1_32` produces; and message 3's MIC is verified with its AES key-wrap **refused** on failure, because that integrity word is the only thing that catches a PTK derived from the wrong nonces.
- **The oracle checks the frames now, not only the primitives.** `verify_wifi_crypto.py` recomputes the PTK from the printed nonces, the msg2 and msg4 MICs over the frames with the MIC field zeroed, the RSN element message 1 carried echoed back byte for byte, and the GTK round trip. All agree on the artifact. `--check` mode of `derive_nl80211_consts.py` diffs all **34** constants against the kernel's own uapi header -- and it exists because one was wrong: `NL80211_ATTR_CIPHER_SUITES_PAIRWISE` was written as 81 and is 73.

**The state machine is exercised, without a radio.** `trm_wifi --harness` plays message 1 and message 3 at the transitions directly -- an access point that exists only as bytes. It checks that message 2 answers message 1 with the right nonce and replay counter, that its MIC verifies over the frame with the MIC field zeroed, that the RSN element is echoed byte for byte, that message 3's key wrap opens to the group key, and that message 4 carries no key data. Then it runs six cases that must be REFUSED, and reports each one: a corrupted MIC; a PTK from a different passphrase; a replay counter that does not advance; one that goes backwards; a key wrap that fails its integrity check; and a message 1 with no RSN element to echo. **20 of 20 held or refused, 0 failed.** One of those refusals is new: an encrypted key data block that would not unwrap used to leave the group key silently missing while the pairwise key installed. An independent oracle (`verify_wifi_harness.py`) recomputes the PMK, the PTK and both MICs from the frame bytes and unwraps the group key with a second implementation of AES key wrap -- **14 of 14 agree**. It disagreed on the first run, and the defect was in the oracle: it hardcoded its own station's MACs.

**The boot now checks content, not only that a route answered.** A 200 says a socket answered; it does not say the ternary branch ran or that the dashboard is the LCARS page. Nine routes are asked for a string their branch cannot produce without having run, and the boot reports both numbers: `19 of the routes asked answered -- the platform is up; 9 of them returned the content they exist to return.` The first two runs of that check failed, both times because of the check: it searched a buffer the status line had NUL-terminated, and then it counted nine checks across ten retry passes as ninety.

**What it does NOT prove.** An association. A wiphy is now registered on the board that carries the AX200 — the driver bound and the firmware was accepted — but `NL80211_CMD_CONNECT` has still never reached an access point, and no four-way handshake has run against one: the parsing, the nonces, the replay rule, the MIC check and the key unwrap all run, while the exchange itself does not. A driver that insists on performing the handshake in firmware has not been met either. The boot that first bound the radio attempted no association, and said why: `/mnt/esp/wifi.conf absent -> no association is attempted`. The frames are proven; the exchange is not. `CONFIG_MAC80211_HWSIM` is not set in the kernel this pathway boots (Microsoft's WSL2 kernel), so there is no virtual radio — the state machine is exercised instead by a protocol-level harness that plays message 1 and message 3 at it (`trm_wifi --harness`: 20 cases held or refused, verified against an independent oracle, 14 of 14). The boot also says plainly when the volume carries no credentials — `/mnt/esp/wifi.conf absent -> no association is attempted` -- because an unconfigured radio and a wrong passphrase must not read the same in a boot log.

## The monad ledger and the journal

**The ledger** is `swarm_v2_artifacts/monad_ledger.jsonl`: append-only, one JSON object per line (JSON Lines). It holds **218 blocks**. Blocks 207-212 were sealed on 2026-10-01 by `swarm_v2/core/seal_monad_journal.py`, which re-runs the claims a block makes rather than copying a receipt. A block carries `index`, `hash`, `prev_hash` and, where set, `anchor` — the Monad Truth Anchor `0x4D4F4E4144000001`. The first block's shape (fields verbatim):

```json
{"anchor": "0x4d4f4e4144000001", "avg_speedup": 3095.34, "hash": "d8a8a8c4…", "index": 1,
 "kleene_k3_passed": false, "modules_stripped": [...], "phase": 23,
 "prev_hash": "0000000000000000000000000000000000000000000000000000000000000000",
 "timestamp": 1788872119}
```

Later blocks (the fixes/monitor rounds) carry `kind`, `contract`, `evidence`, `summary`, `caveat`, `measured` and, where a repair happened, `hash_before_repair`. Across the file there are 19 distinct `kind` values and many `phase` values (`23`…`46`, plus named phases such as `DASHBOARD_BACKEND_ROLLOUT`, `PHASE_57_CORRECTIVE_RECONCILIATION_SEALED`). Block `index: 206`, `kind: "ledger_chain_repair"`, is the repair of 2026-09-30. The last blocks are `207 bare_metal_first_boot`, `208 dashboard_reachability`, `209 radio_staged`, `210 supplicant_proven_halves`, `211 association_frames_proven` and `212 journal_resumed` -- the five arcs that landed while the journal was quiet, plus the resumption itself.

**How a hash is formed.** The verifier is `swarm_v2/core/verify_ledger_chain.py`. For each block it takes the body — every key **except** `hash` — serialises it with `json.dumps(body, sort_keys=True, separators=(",", ":"))` (the canonical recipe), and compares `hashlib.sha256(text.encode()).hexdigest()` with the stored `hash`. A second recipe with default separators (`", "`, `": "`) is also accepted, because two sealers wrote this store; the verifier is recipe-aware so an alternative encoding cannot hide an unreproducible block.

**Integrity is claimed only over sealed blocks.** A block with `sealed: false` (and a stated `unsealed_reason`) is "DECLARED unsealed": its body could not be reproduced, so it is named rather than given a hash that would vouch for it, and it is **not counted as verified**. The verifier's own rule: "the chain's integrity claim covers the sealed blocks alone, and the rest are listed by index." Running it now prints:

```
blocks=212  canonical=207  alt_separators=0  unsealed=5  UNVERIFIED=0
prev_hash links broken: n=0 []
DECLARED UNSEALED (named, not vouched for, not counted as verified): [97, 98, 99, 100, 101]
chain_intact=True (a chain with unreproducible blocks is NOT intact)
```

Block 206 recorded the repair that produced this state: `blocks_before: 205`, `prev_hash_links_broken_before: 13`, `verdicts_before {UNVERIFIED: 5, alt_separators: 5, canonical: 195}`, and it added `prev_hash` to blocks 193–205, repointed 93–97, re-encoded 92–96 to the canonical recipe, and recorded 97–101 as unsealed.

**The journal index.** `MONAD_JOURNAL_INDEX.md` catalogs the Monad Journal research papers, "integrated into the LCARS Dashboard and accessible via `GET /journal/papers`". It states **Total Published Papers: 100** -- a figure that does not match the corpus: `papers/` holds **124** markdown files, numbered from 1 to 162 with gaps, so the index's total is stale and the listing extends past it. (The same class of error as the 81/73 constant: a figure that does not match its source.) It also states the Monad Truth Anchor `0x4D4F4E4144000001`, and lists the corpus in tables — articles 111–126 (foundation architecture & verification), papers 023–031 (federation expansion & voltage harmony), papers 032–034, papers 035–052, and an entry for papers 064–100, then papers 101–162 covering hardware, security, clinical and substrate topics. It also carries a "Clinical Milestone: Sanger Confirmation" entry for patient `PED_RARE_0042` (`COL6A1` c.877G>A, p.Gly293Arg). **Its provenance is not stated anywhere in this tree**: the index, `README.md`, `CHANGELOG.md` and `papers/paper_036_rare_disease_genomics.md` present it as a "Real-World Rare Disease Diagnostic Pilot" with Sanger confirmation (`paper_036` calls the identifier "cryptographically masked"), and no file labels it synthetic or as a test fixture. `LIMITATIONS.md` disclaims wet-lab validation for the platform's compounds, which is a different claim. This document therefore records the entry as the repository states it, and flags that whether the record is real, de-identified or synthetic is not established by anything in the tree -- a clinical record attached to a system without clinical validation needs that label, in the repository, not only here.

## Honest boundaries

- **Wi-Fi association is written but not proven against hardware.** The four-way handshake's EAPOL frames, the PTK/GTK install and the MIC rules are implemented, exercised by a protocol-level harness (20 cases held or refused) and checked against an independent oracle (14 of 14), but "the handshake has never run against an access point". The board with the AX200 has now bound the radio — driver loaded, firmware accepted, `wlan0` published — which is the precondition for `NL80211_CMD_CONNECT`, not the command: no `wifi.conf` was on the drive, so none was attempted.
- **Bluetooth userspace is not built.** `btusb`/`btintel` would give HID and `BNEP`/`RFCOMM` are modules, but both "need **BlueZ** — D-Bus, pairing, the PAN daemon — which is a far larger userspace project than a supplicant, for a slower link"; the doc's verdict is "not the cheap path here, and not the one to build".
- **Routes that return 501 `engine_required`.** `/aura` "still refuses with `engine_required`" (`docs/FIXES_DERIVE_20260925.md` §4). Formats whose parser lives only in the Python registry: *"the Python registry serves this format and is not in this process, so this kernel refuses rather than answer from a parser it does not have"*. `POST /council/<action>/<id>`: *"/council/{action}/<id> advances a proposal by deliberating it, which host_council_deliberation owns. This kernel stores and serves proposals; it does not vote on them."* Why they are explicit: *"The catch-all used to answer {"status":"ok"} here, so the control looked successful while nothing ran."*
- **The tercode store is STALE** relative to the corpus (`index mtime > store mtime`); the route says so and names the fix, `/k3/tercode/build`.
- **`alphabet=trit` discloses a divergence**, `max_abs_diff 31.0` over 2 rows at tolerance 0.0 — structural (a min..max quantized 4-bit code cannot carry the engine's zero trit), reported rather than defined away.
- **The k3 verdict for the packed query is `Unknown`** — the top two hits are inside the band the spread supports, "the honest reading rather than a manufactured winner".
- **The physical board's NIC link.** On the metal boots: `r8169 0000:03:00.0 eth0: Link is Down`; Windows on the same machine calls the same controller `Disconnected`, i.e. nothing is in the port — "whether a cable is on that port is still the machine's to answer".
- **`pivot_root` is refused by this kernel**, so the handover is `chroot` and the old RAM root stays mounted underneath.
- **`host_tranche` remains partly skeleton**: 4 of its 5 skeleton modules (`chaos_engine`, `expert_registry`, `gpu_infinity_computer`, `persona_matrix`) remain. **`/system/modules` returns empty lists** for `active_core` / `native_rust_engines` while `summary` has six populated keys.
- **`LIMITATIONS.md`**: `TRM-ABX-219` and `TRM-ABX-082` "have **not yet undergone physical in vitro or in vivo wet-lab synthesis or biological validation**"; the system "is **not Artificial General Intelligence (AGI)** and does not possess conscious sentience".
- **Builds are not reproducible**, and the rebuilt EFI is not boot-tested: "two compiles of identical source produced differing hashes at the same size, so a hash match is artifact identity, not a re-derivable build"; "the rebuilt EFI has NOT been boot-tested ... so 'it builds' is not 'it boots'". Measured since: the artifact/source gap is **closed for the artifacts on the drive** -- `trm_chainload/dist/trm_chainload.efi` is `5db0e5617b9bc22f` and `EFI/BOOT/BOOTX64.EFI` on the drive is the same file, and the QEMU harness's own ESP is byte-identical to the drive for the chainloader, `vmlinuz`, the initrd and `rootfs.squashfs`, so those QEMU boots exercise exactly what the stick carries. What remains is reproducibility: the build is not re-derivable, so every rebuild is a new artifact that needs its own boot-test.
- **All QEMU/OVMF proofs** (the parser, intelligence and persistent-root rounds) predate the two physical boots; the two boots of 2026-09-30 are the only physical-hardware evidence.
