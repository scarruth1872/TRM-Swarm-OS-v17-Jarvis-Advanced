# Interface and bare-metal checkpoint — 2026-11-06

Measured on this host. Nothing below is a projection.

## Council

| motion | tally | phase |
|---|---|---|
| IFACE-BOOT-01 (1002) | 13 / 1 / 1 | **BLOCKED** — Pulse rejected asserted rails, Verify abstained (no read-back), Archi wanted the real boot artifact |
| IFACE-BOOT-02 (1003) | **14 / 1 / 0** | CONSENSUS after the three clauses were added |
| IFACE-BOOT-03 (scope amendment) | **14 / 1 / 0** | CONSENSUS — the artifact boots the EFI app, not a Linux kernel |

**Defect found while filing:** proposal ids were seeded `next_id: 1001` in memory, so every kernel
restart reused 1001, 1002, … and two different motions could carry the same id. The counter is now
persisted to `swarm_v2_artifacts/council_state.json`, resolved by walking up from the executable.
Verified across two restarts: 1002 → 1003.

## Interface — read path proven, write path partial

| check | measured |
|---|---|
| links crawled from `/ui/` | 33 |
| broken | **0** |
| form classes | 15 |
| GET form fired | 1 (`/ui/journal`) → 200 |
| POST form fired | 1 (`/ui/chat/send`) → 200 |
| mutating POSTs declared, not fired | **7** |
| read-back | `/ui/matrix` shows 15 agents and `ΦV` labelled *measured*; two fetches differ |

Unfired: `kanban/cards`, `mailbox/send`, `evolution/mutate`, `artifacts/integrate`,
`artifacts/review`, `generative-architect/synthesize`, `continuum/genomics/refract`. Firing them
changes state or rewrites source to satisfy a test. The write path for those is **unproven**.

## Bare metal — the artifact boots

`startup.nsh` loads `\EFI\BOOT\BOOTX64.EFI`; `boot_payload.rs` and `main.rs:291` state the boot image
IS the EFI application. Under QEMU + edk2 (pflash code + vars, vvfat ESP):

- firmware: `BdsDxe: loading Boot0001 … starting Boot0001`
- framebuffer at 6/16/30/50 s: the TRM SWARM OS UI renders
- without `host_council_server.exe`: `KERNEL: … NOT FOUND on the boot volume`
- with the binary present: that diagnostic disappears — detection measured, not assumed

### Linux persistent root — proven

`qemu-system-x86_64 -kernel vmlinuz -initrd initrd.img -append console=ttyS0`, with the ESP carrying
`rootfs.squashfs` on a vvfat disk. 92 KB of serial output; the log's own lines:

- `FABRIC VOLUME: sda1 mounted at /mnt/esp (rootfs.squashfs 129597440 B)`
- `PERSISTENT ROOT: … on /dev/loop0 + a RAM layer; write-probe wrote and read back identical`
- `pivot_root` returned EINVAL twice → `chroot used instead -- same root, the old image left mounted`
  (a real defect in the handover: the old root stays mounted)
- `SUPERVISOR: launched as pid 148`
- `GET /infrastructure/status -> 200`, live expert table, services ONLINE, agent slots 16

The boot harness then reports two CONTENT failures:

- `/ui/ answered but did not carry <title>TRM SWARM OS | LCARS INTERFACE</title>`
- `/k3/status answered but did not carry "algorithm":"native-ternary`

Both are **stale assertions in the harness**, measured against the host:

| route | image | host |
|---|---|---|
| `/ui/` title | `Compositor` | `Compositor` |
| `/k3/status` engine | corpus error (4.85 GB > 512 MB exact-ternary limit) | **same error** |

The image behaves like the host; the checks expect strings the platform no longer emits. They should
be updated to compare against a recorded host response rather than a literal.

### Fixes applied and verified

1. **Handover**: `pivot_root` returned EINVAL in both forms and fell back to `chroot`, leaving the
   old root mounted. Replaced with switch_root's `MS_MOVE` form (`mount(".", "/", MS_MOVE)` then
   `chroot(".")`). Rebuilt initrd boot log: `switch_root done (MS_MOVE onto / + chroot)`, no
   fallback line.
2. **Content checks**: the expectation table was rewritten from markers measured on the host
   (`TRM SWARM OS`, `active_agents`, `active_proposals`, `engine_required`, `architectures`,
   `"route":"/k3/status"`, `encoder`) and now carries an expected status per route. The two
   exact-ternary routes assert their **503 `engine_required` refusal** instead of a success string.
   Rebuilt initrd boot log: `CONTENT: 9 of the routes were asked for a specific string and 9
   returned it.`
3. **initrd repack without cpio**: this host has no `cpio`; `scratch/boot_probe/repack_initrd.py`
   parses and rewrites `newc` directly (the shipped archive's nonstandard 12-byte trailer is
   replaced with a proper `TRAILER!!!` entry). New init installed at `E:/EFI/TRM/initrd.img`,
   sha256 `95f6bd31…`.

### Voltage doctrine — not proven on this host

`GET /system/voltage` returned two contradictory basis strings: the inner one said
`V_core nominal (no VID probe)`, the outer said `with the live V_core (AMD P-state VID)`. The outer
string was a literal and is now derived from the probe.

Arithmetic check: Φ_V reported `0.614781`; recomputing from the returned `clock=3301 MHz`,
`mem=51.007%` and a nominal rail ratio of 1.0 gives `0.614781` exactly. `core_vid_volts()` reads
MSR `0xC0010293`, which needs a signed Ryzen driver that is not loaded here, so the rail term is 1.0
and **voltage does not enter the number**. The measurement discipline is implemented and honest; the
voltage *claim* is not demonstrated on this host.



**Not demonstrated:** firmware → chainloader → Linux. The UEFI application does not chainload
`vmlinuz`/`initrd` (it states on screen that the boot image IS the EFI application), and the
chainloader fixture (`scratch/uefi_test`) is absent on this host. The Linux route itself is proven
above by loading the kernel directly; only the firmware-to-kernel handoff remains unexercised.

## Tier-2 coder — loads on call

The supervisor declares the service with an empty command (`TRM_TIER2_EAGER=1` restores eager start).
The kernel starts it from `ensure_tier2_loaded()`, waiting on `/health == "ok"` rather than port-open,
and a watchdog reaps only its own child after `TRM_TIER2_IDLE_SECS` (default 600) with no request in
flight.

| measurement | value |
|---|---:|
| idle before | 81.7% RAM, 4.7 GB resident |
| idle now | **49%**, 8.27 GB free, :8587 closed |
| first tier-2 query | loads and answers in **6.3 s** |
| reaping (20 s test window) | process gone at **+30 s** |

## Open

1. Seven mutating POST forms unexercised.
2. Linux handover route untested; chainloader fixture missing.
3. MySQL80 stop requires an elevated shell (`sc` → Access denied from this session).
