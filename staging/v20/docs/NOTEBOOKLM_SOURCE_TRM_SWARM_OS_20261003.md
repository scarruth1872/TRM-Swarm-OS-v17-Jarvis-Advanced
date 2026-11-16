# TRM Swarm OS v19 "Sovereign" (Private) — source document

Source document for NotebookLM. Compiled 2026-10-03 from the repository at C:/Users/shawn/Downloads/TRM-Swarm-OS-v19-Sovereign-Private-main/TRM-Swarm-OS-v19-Sovereign-Private-main. Every figure below is traceable to a file in that tree. This is an update of the 2026-10-01 source document; it adds the security substrate (threat index + CVE fold), the SRAM computer, and the DDR antibody system + antibody → drug-discovery bridge that landed 2026-10-02/03, and records the boot-drive refresh that staged the runtime stores into the platform image.

## What this system is

The system is a Rust HTTP platform — `trm_uefi_kernel/host_council_server`, a "sovereign" host-council server / microkernel — plus a bare-metal Linux pathway that boots it from a USB drive on real hardware.

Two listeners, defaults read out of `trm_uefi_kernel/host_council_server/src/main.rs`:

| thing | port | default in source |
|---|---|---|
| API | `8022` | `TRM_PORT` unset → `8022` |
| LCARS dashboard | `5183`, served at `/ui/` | `TRM_UI_PORT` unset → `5183` |

The dashboard is a static bundle whose HTML title is `TRM SWARM OS | LCARS INTERFACE`.

Routes the kernel serves (top-level namespaces read out of `main.rs`; this is not a recorded total): `/health`, `/system`, `/k3/*` (`/k3/retrieve`, `/k3/status`, `/k3/verdict`, `/k3/tercode/build`), `/parsers/*` (`/parsers/status`, `/parsers/formats`, `POST /parsers/fastq`), `/council/*` (`/council/status`, `/council/proposals`, `/council/members`, `/council/ledger`, `/council/propose`), `/infrastructure/status`, `/infra/*`, `/sram/*` (`/sram/architectures`, `POST /sram/execute`), `/memory/*`, `/swarm/*` (`/swarm/experts`, `/swarm/telemetry`, `/swarm/chat`, `/swarm/agents`), `/federation`, `/kanban`, `/journal/*`, `/research/*`, `/brain/*` (`/brain/status`, `/brain/reason`), `/verification/*`, `/models`, `/security/*` (`/security/audit`, `/security/threats/query`), `/ddr/*` (`POST /ddr/scan`, `POST /ddr/antibodies/add`), `/discovery/*` (`POST /discovery/pipeline/run`), `/secrets`, `/aura`.

Stores the platform reads, and their substance:

- **A ternary (three-state) knowledge store.** The engine is an in-tree port of `swarm_v2_rust_kernel/src/ternary_knowledge_index.rs` into `trm_uefi_kernel/host_council_server/src/native_ternary.rs`: quantize by absmean, compare integer trits, return sentinels. `search` also reports `out_comparisons` — the work actually performed. Stores on the boot volume: `swarm_v2_nand_fabric.k3` (76,761,697 B), `.tercode` (2 bits per trit, 505,545 B, 2,180 rows), `.turbovec` (packed, 2-bit, 525,297 B, 2,180 vectors, dim 768), `.text.jsonl` (3,168,214 B, 2,180 records).
- **A packed security-threat vector index** (`swarm_v2_artifacts/security_threat_index.tercode`, 89,461,152 B, plus `.meta.json`, 47,156,400 B) — 403,751 threat vectors served by `/security/threats/query`.
- **The DDR threat database** (`swarm_v2_artifacts/DDR_THREAT_DATABASE.json`, 1,029,685 B: MITRE ATT&CK + CWE) and the **CVE fold** (`swarm_v2_artifacts/DDR_CVE_SUMMARY.json`, 743 B: an fkie-cad NVD summary) — both read per-request by `/security/audit`.
- **The DDR antibody stores** — `DDR_ANTIBODIES.json` (52 curated code-scanning antibodies, 24,840 B) and `DDR_THREAT_ANTIBODIES.json` (403,751 generated threat-matrix antibodies, 172,985,239 B) — read by `/ddr/scan` and `/discovery/pipeline/run`.
- **An append-only monad ledger**, `swarm_v2_artifacts/monad_ledger.jsonl` — 223 blocks, sealed by a sealer that re-measures its evidence.
- **A logic anchor**: the Monad Truth Anchor is `0x4D4F4E4144000001`.
- **A supervisor** (`trm_uefi_kernel/host_supervisor`) starts the platform from its own service table and holds a single-instance lock on `127.0.0.1:8023`.

## The bare-metal pathway, end to end

```
UEFI firmware
  -> EFI/BOOT/BOOTX64.EFI   trm_chainload, 3,072 B
  -> EFI/TRM/vmlinuz        16,023,552 B
  -> EFI/TRM/initrd.img     13,577,544 B   (init + supervisor + supplicant + 22 kernel modules + 4 firmware files)
  -> Linux kernel 6.6.114.1-microsoft-standard-WSL2
  -> /init  (PID 1 = trm_chainload/init/platform_init.c, gcc -static -O2)
  -> host_supervisor -> host_council_server   (:8022 API, :5183 dashboard)
```

- **The kernel is itself a UEFI application.** Measured on the kernel file: PE32+, machine `0x8664`, subsystem **10 = EFI application**. So `trm_chainload` loads `vmlinuz` with `LoadImage` and passes a command line by setting `LoadOptions` on the **new image** before `StartImage`. The kernel's EFI stub opens the volume itself, so only the initrd's *path* is passed, never a copy of the bytes.
- **Kernel modules come from a table in the launcher.** `init/platform_init.c` holds a `mods[]` list; `trm_chainload/scripts/make_modules.py --launcher init/platform_init.c` collects exactly those modules, `make_rootfs.py --modules <dir>` stages them flat into `/lib/modules` of the RAM image, and PID 1 loads them with `finit_module` before it asks `/dev` anything. The table holds exactly **22** names: usb-common, usbcore, xhci-hcd, xhci-pci, ehci-hcd, ehci-pci, usb-storage, hid, usbhid, hid-generic, realtek, r8169, i2c-algo-bit, igb, e1000, e1000e, libarc4, led-class, cfg80211, mac80211, iwlwifi, iwlmvm.
- **The volume is found by content, not by device name.** `find_volume()` asks `/dev` again every 250 ms for up to 40 s and accepts a volume only if `rootfs.squashfs` is on it; `block_devices()` enumerates `/dev` rather than assuming `sda`.
- **A read-only platform image plus a writable layer, joined by overlayfs.** `rootfs.squashfs` is attached to a loop device read-only and mounted squashfs read-only; `trm-state.ext4` (67,108,864 B = 64 MB) is loop-mounted ext4 as the overlay upper layer. The platform image itself is never modified. `pivot_root` is refused by this kernel (`EINVAL`), so `chroot` is the handover.
- **`link_stores()` in PID 1 symlinks exactly five stores** from the volume (`/mnt/esp/`) into `/opt/trm/`: `swarm_v2_nand_fabric.turbovec`, `.k3`, `.tercode`, `.text.jsonl`, and `swarm_v2_artifacts/kernel_knowledge_index.json` (53,166,343 B). This is a hardcoded list — stores added later are NOT linked by it.
- **The boot log is written to the volume.** PID 1 tees its whole console output to `/mnt/rw/trm_boot.log` on the state image; `trm_chainload/scripts/read_boot_log.sh <stick|image>` pulls it out from another computer with `debugfs`.

**2026-10-03 refresh — stores moved into the image.** Because `link_stores()` is fixed at five names and the metal kernel's `.ko` modules live in a kernel tree outside this checkout (so the RAM image is not rebuilt here), the new runtime stores were staged **inside** `rootfs.squashfs` at `/opt/trm/swarm_v2_artifacts/` instead of being linked. The platform resolves them through `repo_relative()`, which walks up from the executable looking for a directory that holds `swarm_v2_artifacts/` — so the in-image copy is found the same way a symlink would be. The refreshed image is 96,628,736 B (the 2026-10-01 image was 16,461,824 B) and carries, in addition to the platform and dashboard: `DDR_ANTIBODIES.json`, `DDR_THREAT_ANTIBODIES.json`, `DDR_THREAT_DATABASE.json`, `DDR_CVE_SUMMARY.json`, `security_threat_index.tercode`, `security_threat_index.meta.json`, `antimicrobial_peptide_candidates.json`. The platform in it is a musl cross-build (`x86_64-unknown-linux-musl`, linked with rust-lld): `host_council_server` 19,688,704 B, `host_supervisor` 753,736 B, both `static-pie` ELF. The deploy was verified by sha256 (`9b18ce93…`) matching the build. `make_rootfs.py` gained a `--stores` option to reproduce this; steps in `docs/USB_UPDATE_20261003.md`.

## What has been proven on real hardware

Two physical boots took place on 2026-09-30 on the same board (`docs/BARE_METAL_LINUX_PATHWAY_20260930.md` §18). Every boot before them was QEMU/OVMF. The first physical boot reached the kernel and loaded all 16 modules but came up serving nothing (the USB stick arrived 2.65 s in, after PID 1's single scan). The volume race was fixed and the drive restaged, and the second boot on the same board is the fixed one. The log was read off the machine's own state image — `backups/bare_metal_first_boot_20260930/metal_boot_log_off_the_stick.txt`, pulled from `trm-state.ext4` with `debugfs` without booting the machine.

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

- **The volume was found on pass 7, 1500 ms in** — the machine's own ESP was probed and rejected first because it carries no `rootfs.squashfs`, which is the content rule working on real hardware.
- **The console works** (`CONFIG_VGA_CONSOLE=y`, `CONFIG_FB_EFI=y`, `CONFIG_FRAMEBUFFER_CONSOLE=y`).
- **All modules load on real silicon**, and the board has since booted the 22-module image: `22 loaded, 0 not shipped`. **The radio bound and the firmware was accepted**: `NET: wlan0 up (mac c8:cb:9e:5f:0e:0b)` and `WIFI: wlan0 present (iwlwifi, ... link down)` — `link down` names a link state, not a failure: no access point was in range. That boot also took the first panic this code has ever raised (`WIFI PROBE: exited 101`) in the nl80211 attribute walk.
- **The NIC binds to the board's own chip**: `r8169 0000:03:00.0 eth0: RTL8125B, b4:45:06:d4:2c:51`.
- **Persistence was measured, not asserted.** Across two boots every line of the two serial logs is identical except one field: a write from an earlier boot is `absent` on boot 1 and `PRESENT` on boot 2. The state image was read without mounting it (`debugfs -R "ls -l /upper/opt/trm" state.ext4`).
- **Route answers on metal**: `19 of the routes asked answered -- the platform is up`, including the five `/k3` routes on the exact ternary path. (The 19 reconcile exactly: 18 `GET` + one `POST` over 7 paths. The full route list is longer than what PID 1 probes; the deliberate refusals are simply not probed.)

## The radio and the supplicant

The board's own hardware identifiers (read from Windows on the same laptop that boots the drive): Wi-Fi `PCI\VEN_8086&DEV_2723` (Killer Wi-Fi 6 AX1650x / Intel AX200, `iwlwifi-cc-a0-77` firmware), Ethernet `PCI\VEN_10EC&DEV_8125` (Realtek RTL8125B, bound as `eth0` on the metal), Bluetooth `USB\VID_8087&PID_0029` (Intel, `ibt-20-1-3` firmware). 22 kernel modules and four firmware files are staged; the signed `regulatory.db` + `.p7s` are mandatory because `CONFIG_CFG80211_REQUIRE_SIGNED_REGDB=y`.

**The supplicant** (`trm_uefi_kernel/trm_wifi/`) is dependency-free — one `rustc` invocation, the one primitive std lacks (a netlink socket) issued as a raw x86_64 syscall. It is staged as `/trm_wifi` and run by PID 1 **before the handover**. What is proven about it:

- **The crypto is diffed against an implementation that is not it.** `trm_wifi --vectors` prints every derived value; `verify_wifi_crypto.py` recomputes them with Python's `hashlib`/`hmac` and an independent AES key-wrap. pmk/ptk/kck/kek/tk/mic all agree. `--selftest` passes **6 of 6** published vectors.
- **The nl80211 layer was answered by a real kernel** (family 33 asked of the kernel, not assumed), and the 34 constants were **derived** from `nl80211.h` at v6.6, not remembered — the first draft had two wrong.
- **The state machine is exercised without a radio** (`--harness` plays message 1 and 3): **20 of 20 held or refused, 0 failed**; an independent oracle (`verify_wifi_harness.py`) agrees **14 of 14**.

**What it does NOT prove.** An association. The driver bound and the firmware was accepted and `wlan0` published, but `NL80211_CMD_CONNECT` has never reached an access point and no four-way handshake has run against one. The frames are proven; the exchange is not. The boot says plainly when no credentials are present — `/mnt/esp/wifi.conf absent -> no association is attempted`.

## The security substrate (threat index + CVE fold)

Two routes serve the threat data, both native Rust, both read the stores per-request rather than caching a snapshot:

- **`/security/threats/query`** serves the packed vector index (`security_threat_index.tercode` + `.meta.json`), 403,751 threat vectors, via the same TRITCODE packed-store path the ternary engine uses.
- **`/security/audit`** folds two stores together: `DDR_THREAT_DATABASE.json` (MITRE ATT&CK + CWE) and `DDR_CVE_SUMMARY.json` (an fkie-cad NVD summary). Live figures, as the route reports them: **records_ingested 403,751 / cve_records 400,908 / critical 37,798 / high 152,403 / medium 172,356 / low 16,593**.

**Honest count note.** The NVD feed header declares 401,009 CVE records; the ingester extracted 400,908. The 101-record delta is **uninvestigated** — it is recorded here as a discrepancy, not explained away. fkie-cad was chosen over the NVD 1.1 API because the API returned HTTP 403.

## The SRAM computer

`/sram/architectures` lists the virtual kernels; `POST /sram/execute` dispatches all 15 of them (`tcam`, `hdc`, `systolic`, `gnn`, and the other LUT kernels) and returns `{kernel, output, basis}`. An architecture name that is not one of the 15 answers HTTP 400 rather than fabricating an output. These are in-memory computations over the SRAM LUT model — the "speedup" and "L3" figures an earlier dashboard revision claimed were fabricated, and were removed; the route now reports only what it actually computes.

## The DDR antibody system and the antibody → drug-discovery bridge

These landed 2026-10-02/03. The three DDR write routes that had returned an honest `engine_required` 501 now run **native in the kernel** (`trm_uefi_kernel/host_council_server/src/actions_handlers.rs`):

| route | what it does |
|---|---|
| `POST /ddr/scan` | substring-matches every antibody's `line_pattern` against `{code, filename}` (regex metacharacters stripped; gated by `file_pattern` suffix) |
| `POST /ddr/antibodies/add` | appends to `DDR_ANTIBODIES.json`, deduped on `error_type` + `line_pattern` |
| `POST /discovery/pipeline/run` | maps every antibody to an antimicrobial-peptide seed, scores it in-silico, writes the top candidates |

**Threat-matrix antibodies.** `DDR_THREAT_ANTIBODIES.json` holds **403,751** antibodies generated offline (`scratch/generate_threat_antibodies.py`) from the ingested threat databases: 2,843 MITRE ATT&CK + CWE records plus 400,908 fkie-cad CVE records. Each carries `error_type` (the threat id), `line_pattern`, `fix_description`, and CVSS `severity` where present. Combined with the 52 curated code-scanning antibodies in `DDR_ANTIBODIES.json`, the pipeline scans a union of **403,803**.

**The discovery run (executed live, 2026-10-03).** `POST /discovery/pipeline/run` returned `antibodies_scanned 403,803`, `viable_candidates 345,425`, `top_candidates_written 100` in 6.2 s, writing `antimicrobial_peptide_candidates.json` (77,003 B). The three targets are LpxC (P. aeruginosa / A. baumannii), BamA (E. coli / K. pneumoniae CR), and FtsZ. The top five candidates, each derived from a CVE antibody:

| candidate | target | peptide | derived from |
|---|---|---|---|
| TRM-AMP-7611 | FtsZ | KIGARLKWKWKIYVKIYLKLK | CVE-2002-0347 |
| TRM-AMP-10027 | LpxC | KVKVHWKAKWKVKVTAKFC | CVE-2003-0377 |
| TRM-AMP-11455 | LpxC | KLKATFKVTLKWKAKICWK | CVE-2004-0254 |
| TRM-AMP-21970 | LpxC | RIYWKIKIYAKFKVKLYFK | CVE-2006-3309 |
| TRM-AMP-25564 | LpxC | KIYWKVKFKLTAYFKFKLK | CVE-2006-6975 |

**Honesty labels, stated in the artifact itself.** Every candidate carries `prediction_basis`: the Kd and confidence are *"in-silico heuristic … NOT a measured binding affinity"* — computed predictions, not wet-lab binding. The bridge hashes peptide seeds with FNV-1a-64 where the Python engine used SHA-256, so the sequences **differ** from any earlier Python artifact; this is stated, not hidden. Only the deterministic antibody → AMP stage is regenerated by the route; the other seven paper stages (docking, ADMET, repurposing, circuits, neoantigens, ADC) read pre-existing artifacts and are marked `not_rerun_here`.

## The monad ledger and the journal

**The ledger** is `swarm_v2_artifacts/monad_ledger.jsonl`: append-only, one JSON object per line. It holds **223 blocks** (218 at the 2026-10-01 compile). A block carries `index`, `hash`, `prev_hash` and, where set, `anchor` — the Monad Truth Anchor `0x4D4F4E4144000001`. The first block's shape (fields verbatim):

```json
{"anchor": "0x4d4f4e4144000001", "avg_speedup": 3095.34, "hash": "d8a8a8c4…", "index": 1,
 "kleene_k3_passed": false, "modules_stripped": [...], "phase": 23,
 "prev_hash": "0000000000000000000000000000000000000000000000000000000000000000",
 "timestamp": 1788872119}
```

**How a hash is formed.** The verifier is `swarm_v2/core/verify_ledger_chain.py`. For each block it takes the body — every key **except** `hash` — serialises it with `json.dumps(body, sort_keys=True, separators=(",", ":"))`, and compares `hashlib.sha256(text.encode()).hexdigest()` with the stored `hash`. A second recipe with default separators is also accepted because two sealers wrote this store.

**Integrity is claimed only over sealed blocks.** A block with `sealed: false` is "DECLARED unsealed": named, not vouched for, not counted as verified. Block 206 (`kind: "ledger_chain_repair"`) recorded the repair of 2026-09-30 that produced the current chain state.

**The journal index.** `MONAD_JOURNAL_INDEX.md` states "Total Published Papers: 100" — a figure that does not match the corpus: `papers/` holds **124** markdown files. The index total is stale. It also carries a "Clinical Milestone: Sanger Confirmation" entry for patient `PED_RARE_0042` whose provenance is not established by anything in the tree — recorded here as the repository states it, and flagged that whether it is real, de-identified or synthetic is not established.

## Honest boundaries

- **Wi-Fi association is written but not proven against hardware.** The four-way handshake's frames, PTK/GTK install and MIC rules are implemented and exercised by a protocol-level harness (20 cases held or refused) and an independent oracle (14 of 14), but the handshake has never run against an access point; no `wifi.conf` was on the drive, so none was attempted.
- **Bluetooth userspace is not built** — `btusb`/`btintel` and `BNEP`/`RFCOMM` are modules, but pairing/BlueZ is not the path taken.
- **Routes that still return 501 `engine_required`.** `/aura` still refuses with `engine_required`. `POST /council/<action>/<id>` advances a proposal by deliberating it, which `host_council_deliberation` owns — this kernel stores and serves proposals, it does not vote. (The DDR routes are no longer in this class: they are native now.)
- **The CVE count has an unexplained 101-record delta** (feed header 401,009 vs extracted 400,908) — uninvestigated, not papered over.
- **The drug-discovery candidates are in-silico predictions.** Kd and confidence are heuristic, not measured binding; no wet-lab synthesis or biological validation has occurred (`LIMITATIONS.md` disclaims the same for `TRM-ABX-219` and `TRM-ABX-082`).
- **The tercode store is STALE** relative to the corpus; the route says so and names the fix, `/k3/tercode/build`.
- **`alphabet=trit` discloses a divergence** (`max_abs_diff 31.0` over 2 rows at tolerance 0.0), reported rather than defined away.
- **The k3 verdict for the packed query is `Unknown`** — the top two hits are inside the band the spread supports, "the honest reading rather than a manufactured winner".
- **The physical board's NIC link is down** (`eth0: Link is Down`); Windows on the same machine calls the same controller `Disconnected` — whether a cable is on the port is the machine's to answer.
- **`pivot_root` is refused by this kernel**, so the handover is `chroot`.
- **`host_tranche` remains partly skeleton**: 4 of its 5 skeleton modules remain; `/system/modules` returns empty lists for `active_core` / `native_rust_engines`.
- **Builds are not reproducible.** Two compiles of identical source produced differing hashes at the same size; the rebuilt image is a new artifact, not a re-derivable build. The 2026-10-03 rootfs was verified by sha256 against the build, but has NOT been boot-tested — "it builds" is not "it boots".
- **All QEMU/OVMF proofs predate the two physical boots** of 2026-09-30, which remain the only physical-hardware evidence. The 2026-10-03 store refresh has not been boot-tested on metal either.


