# How Linux loads fully from a flash drive — and the pathway for the full stack on bare metal

Investigation, 2026-09-30. Every number below was measured on this machine or on a boot that was actually
performed; nothing here is inferred from documentation. Evidence is in
`backups/bare_metal_linux_pathway_20260930/`.

## 1. The trick is one measured fact

**A Linux kernel is itself a UEFI application.** Measured with the same PE reader that ships in the UEFI
kernel (`boot_payload.rs`), on the kernel already on this machine:

| file | bytes | PE | machine | subsystem |
|---|---|---|---|---|
| `C:\Program Files\WSL\tools\kernel` | 16,023,552 | PE32+ | 0x8664 | **10 = EFI application** |

So the firmware can load `vmlinuz` **directly**. No bootloader is required for the kernel itself — the
"bootloader" a distribution ships exists to offer menus, pick kernels, and pass a command line, not
because firmware cannot load a kernel.

## 2. Why that produces *full* load and not a toy

The sequence, and what each step actually buys:

1. **Firmware loads a UEFI application** from the ESP (`\EFI\BOOT\BOOTX64.EFI`), or an entry on the
   kernel directly. Firmware services only: file I/O (FAT), memory, console, time.
2. **The kernel is started with a command line** (`LoadOptions`), carrying `initrd=` and `root=`.
3. **The kernel takes the hardware** — it calls `ExitBootServices` and from that moment it is the OS.
   Measured, from the boot log: `Write protecting the kernel read-only data`, `Freeing unused kernel
   image`, then `Run /init as init process`.
4. **The kernel is its own driver stack.** This is the whole difference from a UEFI application:

   | what can read the disk | measured on the same machine |
   |---|---|
   | firmware (EDK2/OVMF) | **vfat only** — a UEFI kernel can see FAT volumes and nothing else |
   | Linux kernel | **ext3, ext4, ext2, squashfs, vfat, fuseblk, fuse, virtiofs, overlay, udf, xfs, 9p, efivarfs** |

   Both lists were read from running systems: the firmware's by our UEFI kernel finding the fabric and
   nothing else, the kernel's from `/proc/filesystems` of a Linux that this very bridge booted.
5. **Userspace comes from an initramfs** (a cpio archive the kernel unpacks into its root filesystem)
   and/or a real root filesystem (`squashfs` + an `overlay` upper layer for writes). PID 1 starts the
   stack.

**A flash drive is not special.** It is FAT on the ESP, a kernel, an initramfs, a root filesystem, and a
command line. That is the entire mechanism.

## 3. Where the current build was blocked

The UEFI kernel is a *UEFI application*: it never calls `ExitBootServices`, so it lives inside the
firmware's world. Consequences, each measured:

- it can read **FAT only** — an NTFS Windows volume or an ext4 Linux volume is invisible to it;
- it has no process model, no scheduler, no network stack;
- `host_council_server.exe` is a **Windows console binary** (subsystem 3, links `tiny_http`/`serde_json`),
  so firmware can never load it. On bare metal the EFI image *is* the kernel, and it is not the platform.

That is why "load the kernel and use the intelligence" could not be satisfied inside the UEFI path: the
full stack is a userspace program, and a UEFI application is not an operating system.

## 4. The bridge — built and proven

`trm_chainload/` (new crate, `x86_64-unknown-uefi`, 3,072 bytes) is the handoff. It does exactly four
things:

1. find its own volume (`LoadedImage.device_handle` → `SimpleFileSystem`);
2. read `\EFI\TRM\vmlinuz` into pool memory;
3. `LoadImage` it;
4. write the command line into the **new image's** `LoadedImage.load_options`, then `StartImage`.

Then the kernel owns the machine. **Only the initrd's path is passed** — the kernel's EFI stub opens the
volume itself, so copying it would create two sources for the same bytes that could drift.

### Two traps found by doing it, both measured

- **`LoadOptions` belongs to the new image, not to `start_image`.** Put the cmdline anywhere else and the
  kernel boots with none.
- **A kernel loaded from a memory buffer has `device_handle = NULL`.** The firmware sets it only when
  `LoadImage` is given a device path. Result on the first attempt:
  `EFI stub: ERROR: Failed to handle fs_proto` / `Failed to load initrd: EFI_INVALID_PARAMETER`. The
  loader now points the new image's `device_handle` at the volume it came from.
- **A command line is POSIX and must keep `/`.** The path encoder used for EFI filenames maps `/` → `\`,
  which turned `rdinit=/init` into `rdinit=\init` — not a path. The kernel then found no init, fell back
  to mounting a root filesystem and panicked: `VFS: Unable to mount root fs on unknown-block(0,0)`.

### What the boot produced

```
EFI stub: Loaded initrd from command line option
[ 0.000000] Linux version 6.6.114.1-microsoft-standard-WSL2 ...
[ 5.693693] sd 0:0:0:0: [sda] 524288 512-byte logical blocks: (268 MB/256 MiB)
[ 6.604489] Run /init as init process
```

and PID 1 — our own static binary — reading the machine it now owns:

```
KERNEL: Linux 6.6.114.1-microsoft-standard-WSL2 x86_64
MOUNTS: proc=ok sysfs=ok
== BLOCK DEVICES THE KERNEL ENUMERATED ==      sda (262144 blocks), sr0, ram0..15
== FILESYSTEM DRIVERS THE KERNEL CARRIES ==    ext3 ext4 ext2 squashfs vfat fuseblk fuse ...
```

`Run /init as init process` is our code, on the display, on bare metal, with the kernel's driver
enumeration above it.

## 5. The pathway to the FULL stack

The bridge is proven; what remains is what the initramfs carries. The full stack on bare metal is:

```
ESP (FAT32)
├── \EFI\BOOT\BOOTX64.EFI        trm_chainload            (3,072 B, proven)
└── \EFI\TRM\
    ├── vmlinuz                  a Linux kernel           (proven: firmware-loadable, subsystem 10)
    ├── initrd.img               cpio.gz: init + the drivers/modules the rootfs needs
    └── rootfs.squashfs          the platform, built for Linux
        ├── /opt/trm/host_council_server   the Rust platform  <- the crate already exists
        ├── /opt/trm/dashboard/dist        the static LCARS bundle
        ├── /opt/trm/fabric/               the K3 fabric + readable corpus
        └── /sbin/init -> the platform     PID 1 starts :8022 and :5183
```

Why this is the right shape and not a rewrite:

- **`host_council_server` is Rust.** The same crate that builds for Windows builds for
  `x86_64-unknown-linux-musl`; the repository already carries Dockerfiles, so a Linux build definition
  exists. Nothing about the platform needs reinventing — it needs an OS underneath it.
- **The dashboard is a static bundle** already served by the kernel; a squashfs carries it unchanged.
- **`squashfs` + `overlay`** gives a read-only platform image with a writable layer in RAM: the flash
  drive is never modified, and the platform's runtime state has somewhere to live. Both filesystems are
  in the measured list above.
- **Every volume on the machine becomes readable**, which is the requirement the UEFI path could not
  meet. The fabric can be read from an ext4 or NTFS volume, not only from a FAT stick.
- The UEFI kernel does not go away: it remains the thing the firmware loads, and it can offer the choice
  (its own LCARS environment, or hand over to the kernel).

## 6. The voltage doctrine, stated as what it actually requires

"Voltage as the converter and decoder of information from anything attached" has a concrete engineering
form, and honesty requires being precise about which part is mechanism and which is metaphor:

- **The decoder is the driver layer, and it is measured, not declared.** `ext4/xfs/ntfs3/squashfs/...`
  plus `sd`, `nvme`, `usb-storage` are what turn an attached device into readable information. The boot
  log above is that layer enumerating the machine — `sda`, `sr0`, `e1000` — each with the driver that
  claimed it. Nothing in it is asserted; a device that is absent is absent from the list.
- **Derive, never assert — and where no probe exists, name the gap.** User space exposes no per-rail
  voltage; that needs vendor PMBus/SMBus or MSR access. The rule this project already holds is that a
  measurement-shaped field name with no probe behind it *is* the fabrication: rename the field to what it
  is (`reference_voltage_v`), add a field naming the missing capability, and drop metrics that are
  neither probed nor derivable. The bridge must carry that discipline across the handoff: firmware →
  kernel → platform, every claim re-derived at each stage rather than inherited.
- **The canonical contract is one derivation, many consumers.** Anything attached gets decoded once into
  the platform's own units (trits, bytes, cells, rails) and every consumer reads that — never a second
  re-expression of the same formula per module.
- What the doctrine does **not** license: reporting a value for something no probe can reach. The Linux
  pathway widens what is reachable (that is its value); it does not make the unreachable measurable.

## 7. Proven vs not

| claim | status |
|---|---|
| a Linux kernel is a UEFI application (subsystem 10) | **measured** on a real kernel file |
| firmware → chainloader → kernel handoff | **measured** (OVMF, screen + serial) |
| the kernel takes the hardware and runs our PID 1 | **measured**: `Run /init as init process` |
| the kernel reads filesystems the firmware cannot | **measured**: ext4/xfs/squashfs/... vs vfat only |
| the full platform on bare metal | **not built**: needs the Linux build + squashfs + an init that starts it |
| physical hardware | OVMF is real firmware; a specific board is still untested |

## 9. The full stack, built and run on bare metal

The pathway in §5 is no longer a plan. Measured on a boot, from the serial log and from the screen:

```
[    7.043394] Run /init as init process
MOUNTS: proc=ok sysfs=ok devtmpfs=ok
LOOPBACK: up
SUPERVISOR: launched as pid 140 (it starts the platform)
[supervisor-rs] 2 services from /opt/trm/trm_uefi_kernel/host_supervisor/supervisor.services.conf
[supervisor-rs] status file: /opt/trm/trm_uefi_kernel/host_supervisor/supervisor.status.json
>>> NATIVE RUST SOVEREIGN SWARM SERVER — MULTI-THREADED (Port 8022)
🚀 Native Rust Multi-Threaded Server listening on http://0.0.0.0:8022
🖥  Dashboard listener on http://0.0.0.0:5183/ui/
  GET /infrastructure/status -> HTTP/1.0 200 OK
  GET /ui/ -> HTTP/1.0 200 OK          BODY: <!doctype html>
```

and the route's own answer, from the booted machine:

```json
{"active_agents":1, "experts":[{"name":"Hermes","avg_latency_us":9431.0,"calls_handled":6,...}, ...],
 "services":[{"key":"rust_kernel","port":8022,"fsm_health":"healthy","restart_eligible":false,
              "supervisor_status":{"services_online":1,"supervisor_status":"HEALTHY"}}],
 "previously_asserted":{"agent_slots":584,"avg_latency_us":500,"active_models":[...]}}
```

What this is: the **supervisor** is running, it **started the platform** from its own service table, the
platform serves both ports, and the health in the payload was measured by the supervisor **on that
machine** — not carried over from the host as a file. The `previously_asserted` block is the route
naming the numbers it refuses to claim, which is the doctrine visible in the output.

How it was built, with the tools the machine already had:

| piece | how |
|---|---|
| platform (`host_council_server`) | `cargo build --release --target x86_64-unknown-linux-musl` on the Windows host, cross-linked by `rust-lld` with `-C link-self-contained=yes`; result `ELF 64-bit, static-pie`, 19,254,992 B (superseded — §11 carries the current image) |
| supervisor (`host_supervisor`) | same target; the crate is dependency-free, so it needs nothing but `rustc` — 744,264 B |
| PID 1 | `trm_chainload/init/platform_init.c`, `gcc -static`, 846,488 B: mounts proc/sys/devtmpfs, brings loopback up, starts the supervisor, then **asks the platform a question over HTTP and prints the reply** |
| rootfs | `trm_chainload/scripts/make_rootfs.py` — newc cpio + gzip, 21 entries, 16.0 MB |

### Two more traps, both measured

- **cpio names are POSIX.** Built on Windows, `os.path.join` produced `opt\trm\host_council_server`, so
  the kernel created *files with backslashes in their names* and `execv("/opt/trm/host_council_server")`
  returned `No such file or directory` — with `MOUNTS: ok`, `LOOPBACK: up` and everything else green. The
  earlier minimal initramfs worked only because its files sat at the root, where no separator is needed.
- **`/dev/console` is the LAST `console=`**, not the first. With `console=tty0 console=ttyS0,115200` a
  userspace program's stdout lands on the serial port, so the display showed kernel output and stopped at
  `Run /init as init process`. The launcher now also writes `/dev/tty0` (and hands it to the platform), so
  the screen is the operator's view — the screen is the measurement.

### First-boot convergence

The supervisor's first pass found the port dark (it starts a service and checks the port in the same
pass), so its first table read `services_online: 0, status: HEALING, rust_kernel: STARTING`. Within one
interval it reads `HEALTHY, services_online: 1` with measured latencies. Both are the same run; the
state converges. Recording the transient as a fault would be the fabrication, not the honest read.

## 10. What is still not done

- **Physical hardware**: all of the above is OVMF under QEMU. No specific board has been booted.
- **Networking**: the kernel's NIC drivers are modules and none is staged, so the proof runs over loopback
  inside the guest. Measured 2026-09-30 on the kernel that boots this drive:
  `CONFIG_E1000=m`, `CONFIG_E1000E=m`, `CONFIG_R8169=m`, `CONFIG_IGB=m`, `CONFIG_VIRTIO_NET=y`; the boot
  harness attaches no NIC either. Reaching `:8022` from another machine needs those modules staged (the
  same `finit_module` path the USB stack already uses), not a different kernel.
- **The fabric and corpus are not on the boot volume**: a Linux userspace has no byte access to the
  firmware's FAT-only view the way the UEFI kernel does; the K3 fabric (76,761,697 B) belongs on a volume
  the kernel mounts.
- **Persistent writes**: the rootfs is an initramfs (RAM). A `squashfs` root with an `overlay` upper is
  the documented next step; `mksquashfs` is present on this machine.
  > **DONE the same day -- see §14.** The platform is now a read-only squashfs image on the volume with
  > a writable ext4 layer, and what it writes is still there on the next boot. Disclosed there: this
  > kernel refuses `pivot_root`, so `chroot` is the handover and the old RAM root stays mounted.

## 11. The parser routes on metal (same day, later image)

The §9–10 boot carried the platform as it was built that morning, which predates the `/parsers/*`
routes. The platform was cross-built again from the current source and the rootfs repacked, and the
launcher was extended to prove the new routes: `http_req()` grew a `text/plain` POST, and the boot now
asks for them as well.

| piece | how | measured |
|---|---|---|
| platform | as §9, same flags | `ELF 64-bit LSB pie, static-pie`, **19,365,368 B**, sha256 `ee57d18417733b103c1abf28c336d3c0d884057e14b8b90903fd3f29a97028bf` |
| PID 1 | `gcc -static -O2` | 846,488 B — `http_req()` plus the two new probes |
| rootfs | `make_rootfs.py` | 21 entries, 21,829,044 B unpacked → 16.1 MB gzip |

What the boot itself reports, from the serial log at `t+03s` — the probe output, not a summary:

```
MOUNTS: proc=ok sysfs=ok devtmpfs=ok
LOOPBACK: up
SUPERVISOR: launched as pid 140 (it starts the platform)
  GET /infrastructure/status -> HTTP/1.0 200 OK
  GET /ui/ -> HTTP/1.0 200 OK
  GET /parsers/status -> HTTP/1.0 200 OK
  POST /parsers/fastq -> HTTP/1.0 200 OK
```

and the POST's answer, parsed on the metal:

```json
{"anchor": "0x4D4F4E4144000001", "bases": 14, "gc": 0.7143, "mean_q": 40.0,
 "reads": 2, "status": "VALID", "trits": {"minus": 7, "plus": 7, "zero": 0}}
```

**Why the parser probe is the discriminator, not decoration.** The previous musl build of this same
source 404s `/parsers/*`, so a `200` from inside the guest proves *which* build the firmware chain
loaded — a banner and a `/health` 200 are produced by either image. The answer's shape
(`reads_sample`, `q20_percentage`/`q30_percentage`, `max_reads`, `parser_implementation`) is what only
the ported parser emits.

Evidence: `backups/bare_metal_parsers_on_metal_20260930/` — the full serial log, the extracted probe
block (`probe_output_t03.txt`), the parsed verdict (`parser_verdict_on_metal.json`), the launcher
source, and two screen captures. The screen has scrolled past the `t+03s` block by the time the
captures at `t+12s`/`t+47s` are taken, so the serial log is the record for those lines while the
captures show the platform running and answering.

## 12. The intelligence, on metal

Serving the parser routes proved the image boots. It said nothing about whether the *intelligence* is
reachable there, and it was not: `/k3/*` could not answer on any non-Windows build at all.

### What was actually blocking it

Not the missing fabric file — two Windows-wired pieces in `k3_retrieval.rs`:

1. **The query encoder was a DLL export.** `embed()` called `rust_native_encode_text_f32` out of
   `swarm_rust_kernel.dll`, and `load_engine()` returns early on non-Windows
   (`"kernel DLL loading is wired for windows only"`). Without an encoder there is no query vector, and
   without a query vector there is no retrieval — no encoder, no answer.
2. **`engine()` was resolved at the top of the route**, before the branch that does not need it. The
   packed scan scores the store's own 2-bit codes with the query's own quantizer; it never touches the
   DLL. Demanding one up front is what made `?engine=packed` answer `503 engine_required` on a build
   that could serve it.

### The fix, and why it is a port rather than a fallback

`native_encoder.rs` ports `swarm_v2_rust_kernel/src/native_query_encoder.rs` (Paper 150) into the
platform — FNV-1a 64 with the Monad salt, SplitMix64 hyperplane expansion, word/boundary/3-4-5-gram
features, sublinear superposition, L2 normalization — algorithm and constant unchanged. One encoder on
both platforms, not a second one with its own opinion. The module's own contract allows nothing less:
*"there is no fallback that silently answers from a different path."*

It may stand in for the DLL only where it **reproduces** it, so that is measured, not asserted:

```
test native_encoder::encoder_parity::port_reproduces_the_dll_export ... ok
```

18 texts — the corpus builder's own test strings, unicode, combining marks, `""`, whitespace,
punctuation-only, a 1-char word, hyphen/underscore/digits, tabs/CRLF, and a long one spanning all twelve
64-wide blocks — encoded by both, element by element: **max |Δ| = 0.0**. 31 tests pass.

And the whole route was re-answered before and after, live `:8022` (old) against a new-source build on
`:8099`, 13 requests across four queries and all three engines: **13/13 identical** documents, scores to
the last bit, and verdicts. Only `elapsed_us`/`packed_scan_us` differ, and those are not compared here.

The engine is now resolved *inside* the branches that need it (exact ternary, tercoce, `alphabet=trit`),
which still report `503 engine_required` on metal — honestly, because a DLL genuinely is required there.

### The store is on a volume, not in the image

`repo_relative()` walks up from the **running executable** (up to eight parents), never a hardcoded
path, so the artifacts have to sit within eight parents of
`/opt/trm/trm_uefi_kernel/host_council_server/host_council_server`. The launcher mounts the boot volume
with the kernel's own driver and links the stores there:

```
BLOCK DEVICES: sr0 sda
FABRIC VOLUME: /dev/sda mounted at /mnt/esp; swarm_v2_nand_fabric.turbovec (45829 B)
               swarm_v2_nand_fabric.k3 (76761697 B) swarm_v2_nand_fabric.tercode (45453 B)
               swarm_v2_nand_fabric.text.jsonl (1538828 B) -> linked into /opt/trm
```

Two traps, both measured on failed boots:

- **The device name is not a design decision, it is the kernel's.** The first attempt named `/dev/sda`
  and failed; the fix was not to try harder but to *ask* — enumerate `/dev` for block devices and try
  what is actually there. `sr0 sda` is the answer, and it is printed whether or not the mount works.
- **`mount` returning ENOENT did not mean the device was missing.** `/dev/sda` existed (the enumeration
  proved it); the target did not — `mkdir("/mnt/esp")` cannot succeed when `/mnt` itself is absent from
  the initramfs. Creating `/mnt` first is the whole fix. An errno is a fact about the call, not a
  diagnosis of the system.

### The measurement

The boot asks a semantic question and prints the answer it gets, over loopback, from the platform it
started:

```
GET /k3/retrieve?engine=packed&q=LpxC zinc-dependent deacetylase inhibitor&k=3 -> HTTP/1.0 200 OK
```

```json
{"status":"ok","engine":"packed-domain-turbovec","encoder":"native-turbovec-hdc","scored_documents":180,
 "store":{"path":"/opt/trm/swarm_v2_nand_fabric.turbovec","bytes":45829,"vectors":180,"dim":768,"bits":2},
 "results":[{"rank":1,"doc":"FULL_PLATFORM_VALUE_AND_COST_ANALYSIS.md","score":0.08477146396177668},
            {"rank":2,"doc":"papers/paper_132_phoenix_fault_tolerance_and_resilience_...","score":0.05587398019062806},
            {"rank":3,"doc":"DRUG_DISCOVERY.md","score":0.04815354849935385}]}
```

Those scores are **identical to the same query against the live Windows kernel** — same store, same
encoder, no DLL, no Python, no model server, no token sampled. The k3 verdict is `Unknown` and the route
says why (the top two are inside the band the spread supports), which is the honest reading rather than
a manufactured winner.

Evidence: `backups/bare_metal_intelligence_on_metal_20260930/` — the full serial log, the probe block,
the parsed verdict, the launcher source, and every artifact hash and size.

### What is still not reachable on metal

> **SUPERSEDED the same day -- see §13.** All three branches below now answer on metal: the DLL gate was
> removed by porting the engine's primitives in-tree. The list is kept because it names the gap §13
> closed, and because the read-only volume is still true.

- **The exact ternary path, the tercoce store, and `alphabet=trit`** need the DLL's quantizer/search and
  answer `503 engine_required`. Only the packed path runs without it.
- **The 53 MB `kernel_knowledge_index.json`** is unchanged: those routes still want the engine too.
- The fabric stores are read-only here (the volume is mounted `MS_RDONLY`), which is right for retrieval
  and wrong for anything that wants to write state back.




---

## 13. The exact ternary engine, on metal -- and the DLL gate gone from every target

§12 ended with three branches still answering `503 engine_required`: the exact ternary path (no
`engine=`), the tercode store, and `alphabet=trit`. The reason was the same in all three -- the engine
resolved to five exports in `swarm_rust_kernel.dll`, and a DLL is a Windows PE. The platform could load
the corpus and answer by the packed path on metal, but the arithmetic that *is* the engine sat behind a
file format the target cannot load.

That was never a property of the algorithm, only of its packaging. The kernel module those exports come
from -- `swarm_v2_rust_kernel/src/ternary_knowledge_index.rs` -- is ~100 lines of pure Rust: quantize by
absmean, compare integer trits, return sentinels. So the engine is a **port, not a fallback**. A
fallback is the defect this project already named: `packed_ternary::quantize` is the min..max 4-level
rule, a *different* quantizer, so an exact path answered by it would put the query in one space, score
it in another, and still report a confident number.

### The port

`trm_uefi_kernel/host_council_server/src/native_ternary.rs` carries the five entry points
(`nt_quantize`, `nt_search`, `nt_agreement`, `nt_magnitude`, `nt_fidelity`) with the source's own
constants, order and sentinels. `search` also reports `out_comparisons` -- the count of work actually
performed -- so any throughput this system states is derived from work done rather than assumed.

`engine()` in `k3_retrieval.rs` no longer loads a library. It resolves the corpus (JSON, via
`repo_relative`) and binds the primitives. The DLL keeps exactly one role, the **parity oracle**, and
that role is *reported, not required*. On metal:

```json
{"algorithm": "native-ternary (in-tree port of ternary_knowledge_index.rs)",
 "parity_oracle_present": false, "parity_oracle": null, "quantizer_multiplier": 1.0}
```

and `"parity_oracle_present": true` with the DLL's path and size on the host. A status route that names
a path the machine does not have is the declared-where-should-be-measured defect; a route that reports
its oracle absent is telling the truth about the machine it runs on.

`/k3/status` reports the same thing for the same reason: it names the algorithm, the corpus it
resolved, and whether the reference artifact is on this machine. (Its route list was also missing
`engine=packed&alphabet=trit` -- a route that worked and was never listed -- so it is listed now.)

### The proof, in three places

1. **The parity test ships in the crate.** `native_ternary::parity::port_reproduces_the_dll_exports`
   loads the DLL via `LoadLibraryA`/`GetProcAddress` and diffs every ported primitive against the DLL's
   own exports over the index's own embeddings, f32 compared by **bit pattern**, never by tolerance.
   `cargo test --release`: **32 passed, 0 failed**, beside the encoder's own
   `port_reproduces_the_dll_export`.
2. **A route diff against the live DLL-based kernel.** The candidate ran beside the deployed kernel
   `7a7bb6f8...` and every captured route was re-answered: **13/13 semantically identical** after
   stripping timing fields -- same documents, same integer agreement scores, same f32 scores to every
   digit.
3. **The boot's own voice, on metal.** One image, five routes:

```
FABRIC VOLUME: /dev/sda mounted at /mnt/esp; turbovec (45829 B) k3 (76761697 B) tercode (45453 B)
  text.jsonl (1538828 B) swarm_v2_artifacts/kernel_knowledge_index.json (53166343 B) -> linked into /opt/trm
GET /k3/retrieve?q=balanced%20ternary%20logic%20consensus&k=3                          -> 200 OK
GET /k3/retrieve?q=balanced%20ternary%20logic%20consensus&k=2&engine=tercode&verify=1  -> 200 OK
GET /k3/retrieve?q=...&k=2&engine=packed&verify=1&alphabet=trit                        -> 200 OK
GET /k3/retrieve?engine=packed&q=LpxC%20zinc-dependent%20deacetylase%20inhibitor&k=3   -> 200 OK
GET /k3/status                                                                          -> 200 OK
```

The exact path scored **2180 candidates / 2180 comparisons** and returned

```
papers/paper_148_turbovec_and_ternary_logic_knowledge_fusion_engine.md      106
papers/paper_121_sram_multi_pattern_fusion_and_hybrid_computing_operators.md 105
papers/paper_18_monad_philosophy.md                                          98
```

-- the same three documents with the same integers the live Windows kernel returns for the same query.
The tercode store returned the same two hits with its own cross-check `identical: true`
(`engine_comparisons: 2180`, `differences: []`).

### The store belongs on the volume, and a nested rel path needs its directory

53 MB of corpus does not go in the RAM image: it lives on the volume with the rest and the launcher
links it where the resolver walks. `repo_relative` joins the rel path onto a candidate ancestor, so
`swarm_v2_artifacts/kernel_knowledge_index.json` needs `/opt/trm/swarm_v2_artifacts/` to exist before
the link into it can resolve. The launcher now creates each intermediate directory and links every
store, so `/k3/status` reports what is on the disk rather than what the probe happened to use.

### What the gate was hiding, and what is left

Removing it did not fix one route; it fixed the **class**. Every branch that resolved the engine --
exact ternary, tercode, `alphabet=trit` -- was unreachable on every non-Windows target, including the
one this platform boots on. The same musl artifact run under WSL's Linux kernel (no boot needed)
answers all five `/k3` routes `200`; the boot is what turns that into a claim.

**Deployed:** the live Windows kernel now runs this same source -- `fe2bafc8...`, 19,317,248 B, pid
1568 -- proved by a shape the previous image cannot produce: the old engine block named a `dll`, this
one names the algorithm and reports its oracle, while the exact path still returns 106/105/98.

Still true, and disclosed rather than smoothed over:

- **The tercode store is STALE** relative to the corpus (`index mtime > store mtime`). The route says so
  in its own body and names the fix, `/k3/tercode/build`. Its ranking matched the host's anyway -- both
  read the same stale store, which is why agreement is not freshness.
- **`alphabet=trit` discloses a divergence**: `max_abs_diff 31.0` over 2 rows at tolerance 0.0. Two
  different stores (180 packed rows vs 2180 corpus rows) compared in a shared alphabet -- the route
  reports the difference instead of defining it away.
- The volume is mounted `MS_RDONLY`: right for retrieval, wrong for anything that writes state back.
  (**§14 changes this** -- the volume goes writable when a state image is present, and the boot says
  which of the two happened. The platform image itself stays read-only either way.)
- The kernel's NIC drivers are modules and none is staged, so every proof here is loopback-only
  (`CONFIG_E1000=m`, `CONFIG_E1000E=m`, `CONFIG_R8169=m`, `CONFIG_IGB=m`).

Evidence: `backups/bare_metal_exact_engine_20260930/` -- full serial log, probe block, parsed verdict
with the host-vs-metal identity check, launcher source, and the hash and size of every artifact.

---

## 14. The persistent root: a read-only platform image, a writable layer, and a handover

§10 named this as the next step and §13 kept pointing at it: the root was an initramfs, so the
platform ran out of RAM and every byte it wrote died with the power. Three pieces fix that, and the
boot reports all three.

| piece | what it is | measured |
|---|---|---|
| the RAM image | `make_rootfs.py --init-only`: `init` and the mount points, nothing else | **382,460 B** gzip (was 16.1 MB) |
| the platform image | `make_rootfs.py --mode squashfs`: platform, supervisor, service table, dashboard | **16,449,536 B**, sha256 `cfffef89...` |
| the writable layer | `make_state.py`: a 64 MB ext4 image on the same volume, loop-mounted as the overlay's upper | **67,108,864 B** |

The RAM image losing 16 MB is the point, not a side effect: the platform used to be carried twice,
once in RAM and once on the volume. There is now exactly one copy, the one on the volume. The
launcher finds it, attaches it to a loop device read-only, mounts it squashfs read-only, builds the
overlay over it with the state image as the upper layer, moves `/dev`, `/proc`, `/sys`, the volume
and both halves of the overlay into the new root, writes a file through the overlay and reads it
back, checks the same path is still **absent from the read-only image underneath**, and only then
hands the machine over.

### Six traps, every one of them measured on a boot rather than reasoned about

- **A mount hides whatever sat at its mount point.** `upper/` and `work/` were created before the
  tmpfs was mounted over them, so they were not on the tmpfs, and overlayfs reported the upperdir it
  could not find as `ENOENT`. Create them *after* the layer is mounted.
- **A block device reports size 0.** Reusing one `struct stat` for the loop device printed
  `(0 B, ro)` for a 16 MB image -- the declared-where-a-measurement-belongs defect, in the one line
  whose whole job is to report the measurement.
- **`pivot_root` is not in this toolchain's headers at all.** The first attempt defined
  `_GNU_SOURCE` on the premise that glibc declares it there; a two-line probe showed the premise was
  false (no declaration under any feature macro). `syscall(SYS_pivot_root, ...)` compiles clean.
- **`pivot_root` is refused on this kernel, and the evidence says so rather than "it did not work".**
  The mount table (read from inside the new root) shows **0** mounts marked shared or slave, the two
  roots are on different filesystems, and a child in its own mount namespace is refused for a plain
  squashfs root *and* for an overlay root. The EINVAL is not the overlay, not propagation, and not
  the new root's shape. **`chroot` is the handover that works here, and the boot prints which one it
  used** -- a fallback that stayed quiet would have read as a success.
- **A loop device is no more writable than the descriptor it was attached with.** `O_RDONLY` on the
  state image gave a read-only loop, and mounting ext4 read-write on it failed with `EACCES`. The
  helper takes a `writable` flag now, and the layer says why when it falls back.
- **Rebuilding the volume destroys the thing being measured.** The harness rebuilt the FAT image on
  every run, so the previous boot's state was erased before the next boot could look for it and every
  run would have looked identical. `--reuse` boots the volume exactly as the last boot left it.

### The proof: two boots off the same volume, one field different

```
boot 1 (fresh volume):  ... + the state image on the volume (writes outlive the reboot);
                        write-probe wrote and read back identical; this file in the read-only
                        image: absent; a write from an earlier boot: absent
boot 2 (same volume):   ... identical ...; a write from an earlier boot: PRESENT
```

Both boots answered **15/15** routes, including all five `/k3` routes with the exact ternary path.
Every other line in the two serial logs is identical; the only difference is that one field.

And because the launcher's own word is not proof, the state image was taken **out of the boot volume
and read without mounting it**, from outside the guest:

```
$ debugfs -R "ls -l /" state.ext4
     13   40755 (2)      0      0    4096  upper
     14   40755 (2)      0      0    4096  work
$ debugfs -R "ls -l /upper/opt/trm" state.ext4
     18  100644 (1)      0      0      55  .write-probe
     20  120777 (7)      0      0      38  swarm_v2_nand_fabric.turbovec
     21  120777 (7)      0      0      32  swarm_v2_nand_fabric.k3
$ debugfs -R "cat /upper/opt/trm/.write-probe" state.ext4
written through the overlay by the platform's own init
```

The 55-byte file and the post-pivot store links are in the writable layer on the flash, readable
after the machine is gone. The platform image itself is untouched: it is mounted read-only from a
read-only loop, and the boot checks after every write that the path is still absent from it.

### What this does and does not claim

**The platform image is never modified.** The volume is mounted read-only, and re-mounted writable
only when a state image is present -- the launcher prints which of the two happened, because "state
lives in RAM" is a fact an operator needs rather than an assumption to make.

**`pivot_root` is not used, and the old RAM root stays mounted.** On this kernel it is refused for
every new root (measured above). `chroot` gives the same root and the same writable layer, which is
what the rest of this depends on; it does not release the RAM rootfs, which is a few hundred KB here
because the platform is no longer in it.

Evidence: `backups/bare_metal_persistent_root_20260930/` -- both serial logs, PID 1's voice from each,
the routes asked, the debugfs read of the state image from outside the guest, the launcher and both
packers, and the hash and size of the launcher, the RAM image, the platform image and the state image.

---

## 16. The L3 the platform serves is the machine's, on metal too (2026-09-30)

`/sram/architectures` used to serve the module's `L3_SIZE = 32 * 1024 * 1024` literal -- a value *declared*
where a value should be *measured*, the defect class this project keeps naming. It now takes the size from
`host_probe::l3_cache_kb()`, derives cells/sets/transistors from the size it got, and reports `l3_basis`
(`measured:` / `assumed:`) and `host_l3_matches_design_target` beside it, with the module's
`declared_default` kept visible.

`host_probe.rs` gained the Linux arm for this: `/sys/devices/system/cpu/cpu0/cache/**`, unified L3 summed
over the instances told apart by `shared_cpu_map` -- one instance's size is not the machine's L3. The
bare-metal boot proves it end to end: the guest answers `size_bytes 16777216`, `cells 134217728`,
`sets 4096`, `transistors 805306368`, `host_l3_matches_design_target false`, and `l3_basis "measured:
host_probe.rs -- ..."`, on `rootfs.squashfs 16457728 B` (the rebuilt image). The boot's own voice is in
`backups/bare_metal_l3_20260930/usbstick_l3_20260930.log`.

The Windows deployment and the musl image are one source: `cargo test --release` is 32 passed / 0 failed,
and the two parity rulers that used to *count* this divergence now assert against what is served.

## 15. Onto the physical stick: the kernel cannot see the drive it booted from

Staging the pathway onto `E:` (the UEFI-kernel drive, 14.6 GB FAT32) meant answering a question the
QEMU boots had hidden: **how does a kernel read a USB volume?** Answer, measured before anything was
written to the drive: `grep`-ing `modules.builtin` for the stack this pathway would depend on gave
`usb-storage`, `usb/core`, `xhci`, `ehci` — **none of them built in** — while `ahci`, `nvme` and
`sd_mod` are. The kernel is a VM kernel: it carries the drivers a virtual machine needs. Every boot
so far had been given its disks as AHCI (`scsi host0: ahci` in the serial logs), so the USB stack was
never exercised, and the drive it was booted from was never the thing it had to read.

On a real machine the volume the firmware just booted from **is** behind a USB port. Without that
stack the kernel starts, finds no disk, and the platform never comes up — a boot that fails for a
reason invisible in every test run here.

### The measurement: the same launcher, one variable

`make_modules.py` collects the modules listed in the launcher's own `mods[]` table -- one list, read
out of the code that consumes it -- and `make_rootfs.py --modules <dir>` stages them flat into
`/lib/modules` of the RAM image; `init` loads them with `finit_module` before it asks `/dev`
anything. Two images off the same source, the same volume layout, the volume on an **emulated USB
stick with no other disk attached**, so the USB stack is the only way to find it:

| boot | MODULES | BLOCK DEVICES | outcome |
|---|---|---|---|
| without `--modules` | `-> 0 loaded, 10 not shipped` | `sr0` | `WAIT: ... still none after 8000 ms`; `no volume carrying the platform image`; **`NOTHING answered on :8022`** |
| with `--modules` | `-> 10 loaded, 0 not shipped` | `sda sr0` | `sda mounted at /mnt/esp (rootfs.squashfs 16449536 B)`; state writable; **18 answers** |

The negative boot is the useful one: it fails loudly, names the missing step, and its last line is
counted from the routes it asked -- which is a fix, not a flourish (below).

### Four defects the staging found, three of them in code that had already "passed"

1. **The volume was picked by device name.** `main()` accepted "the first device in a hardcoded list
 that mounts vfat" -- `/dev/sda`, `/dev/vda`, `/dev/sdb`, `/dev/vdb`. On a machine with an internal
 NVMe, the Windows ESP matches that description **first**: vfat, mounts cleanly, carries no
 platform. That boot comes up serving nothing and says nothing about why. Fixed with
 `block_devices()`, which enumerates `/dev`, and a picker that accepts a volume only if
 `rootfs.squashfs` is on it. Proven with a decoy ESP attached first:
 `sr0 not vfat (No medium found); sda vfat but carries no rootfs.squashfs; sdb mounted at /mnt/esp`.
2. **`usbhid` loaded with unresolved symbols.** The table named `usbhid.ko` but not `hid.ko`, which
 it resolves against (`usbhid: Unknown symbol hid_output_report (err -2)`, seven of them). A module
 that loads and does not work is worse than one that fails outright -- and the boot said nothing,
 because it counted the `finit_module` result and moved on. The dependency chain comes from
 `modprobe --show-depends`, not from reading the names.
3. **`/dev` was asked once, immediately after `insmod`.** USB enumeration is not finished when the
 module returns: the controller registers (`xhci_hcd 0000:00:03.0: xHCI Host Controller`), then the
 stick is attached (`usb 2-1: new SuperSpeed USB device`), then SCSI probes it. The first version
 looked once, saw `sr0`, and reported a machine with no disk. Now the device that is *coming* is
 waited for -- bounded at 8 s, and the cost is printed (`a disk appeared after N ms`).
 `has_disk()` excludes `/dev/sr0` deliberately: the CD-ROM node exists whether or not there is a
 medium in the drive, so waiting for it would always be over at once.
4. **The last line asserted a state it had not measured.** On the negative boot -- no volume, no
 image, the supervisor's `execv` failing -- the boot still printed *"init: the platform is up;
 idling so the machine stays alive."* The launch was real; the claim was not. `http_req`/`http_probe`
 now return the status their route answered with, the boot counts them, and the last line is one of
 two: *"N of the routes asked answered -- the platform is up"* or *"NOTHING answered on :8022 -- no
 platform came up on this boot"*. A boot cannot report itself healthy by having started something.

### What went onto the drive

`E:` was an UEFI-kernel drive: `EFI/BOOT/BOOTX64.EFI`, `EFI/TRM/trm_uefi_kernel.efi`,
`EFI/TRM/trmboot_pe.efi` -- all three **byte-identical** (`386d6aeb84a44058…`, 16,422,400 B), so
replacing the entry point displaces nothing. Staged, every copy verified by hash against its source:

  EFI/BOOT/BOOTX64.EFI        the chainloader (3,072 B)      <- was the UEFI kernel
  EFI/TRM/BOOTX64.uefi_kernel.efi   the previous entry point, kept as its own file
  EFI/TRM/vmlinuz             16,023,552 B
  EFI/TRM/initrd.img          945,228 B (init + supervisor + the ten modules)
  rootfs.squashfs             16,449,536 B, cfffef8996535663...
  trm-state.ext4              67,108,864 B, empty -- this is where the machine's state will live
  swarm_v2_nand_fabric.{k3,tercode,text.jsonl,turbovec}, swarm_v2_artifacts/...
  docs/LINUX_PATHWAY.md       what boots, how to switch back, what is not proven

The three stores already on the drive (`k3`, `text.jsonl`, `turbovec`) were **stale** -- the same-size
`.k3` hashed differently and `text.jsonl` was 1,404,404 B against the repo's 1,538,828 B -- so all
five stores named by `link_stores()` were refreshed from the repo, because the tercode route
cross-checks its ranking against the engine: a mixed corpus is a divergence waiting to be reported as
a bug. All five now hash identical to the repo's.

The drive's own `README.md` carried a "verified on this boot drive" block describing the UEFI-kernel
build and named `\EFI\BOOT\BOOTX64.EFI` as that build. It now opens with what changed and the one-copy
way back, rather than leaving a document on the hardware that describes a boot which no longer
happens.

### What this does not claim

**Nothing has been booted on physical hardware.** Everything above is QEMU/OVMF with an emulated
xhci + usb-storage. What could still differ on a real board, restated against measurement (2026-09-30:
the kernel is `6.6.114.1-microsoft-standard-WSL2`; `CONFIG_FB_EFI=y`, `CONFIG_FRAMEBUFFER_CONSOLE=y`,
`CONFIG_VGA_CONSOLE=y`, `CONFIG_USB_XHCI_HCD=m`, `CONFIG_USB_EHCI_HCD=m`, `CONFIG_USB_STORAGE=m`,
`CONFIG_SATA_AHCI=y`, `CONFIG_NVME_CORE=y`, `CONFIG_BLK_DEV_SD=y`):

- **what the board's firmware will start** from an unsigned 3,072 B `\EFI\BOOT\BOOTX64.EFI`: 64-bit UEFI
  with Secure Boot off; a CSM/legacy-only machine will not start it at all;
- **what framebuffer mode it hands over**: the console drivers are built in and the efifb probe in the
  boot log reads `mode is 1280x800x32`, so the screen works wherever a GOP mode exists; where none does,
  the only voice is `console=ttyS0,115200`, and a normal PC has no serial port;
- **network**: closed 2026-09-30. The RAM image carries the six NIC modules (`realtek`, `r8169`,
  `i2c-algo-bit`, `igb`, `e1000`, `e1000e` -- the chain from `modprobe --show-depends`), PID 1 applies
  the address `/network.conf` on the volume carries, and the guest answered `:8022` to a request from
  OUTSIDE the guest (`HTTP 200` on `/k3/status` through the forwarded port; `16 loaded, 0 not shipped`);
- **the modules against real silicon**: `xhci-hcd` + `ehci-hcd` cover USB 3 and USB 2 mass storage, but
  the list comes from `modprobe --show-depends` and has never met a physical controller (a bridge needing
  `uas` would not be covered).

`docs/LINUX_PATHWAY_ON_USB_20260930.md` carries the same list on the drive itself.

Evidence: `backups/bare_metal_usb_stage_20260930/` -- the before and after manifests of `E:` (every
file hashed, so the drive can be put back), both A/B serial logs from the USB-stick boots, the
USB-layout boot pair, both boot harnesses, and `artifact_ids.txt` with the full hashes of everything
deployed and of the launcher and packers that produced it.

## 17. The rebuilt stores, on the metal (2026-09-30)

The stores on the drive had been built when the corpus held 180/186 documents, and the platform's cache
made its first read permanent — so a rebuilt store could be shadowed for the life of the process. Both are
fixed (`docs/FIXES_DERIVE_20260925.md` §6) and re-proven on the stick, with no other disk:

- `rootfs.squashfs` **16,461,824 B**, sha256 `df686a7d23901f8b3fe99f7a17e8b51e21d3aca30cd2cb1969cbcdbe57696b65`,
  packed from the musl build `143c33454d457600…`. The image the guest mounted is the image on the drive —
  the same hash on all three copies (build output, boot staging, `E:`).
- the guest's own voice, from the boot (`backups/ternary_stores_20260930/usbstick_storereload_20260930.log`):
  `init: 19 of the routes asked answered -- the platform is up`;
  `FABRIC VOLUME: sda mounted at /mnt/esp (rootfs.squashfs 16461824 B)`;
  `STORES: … turbovec (525297 B) … tercode (505545 B) … text.jsonl (1869777 B)`;
  `"vectors":2180`, `"serves_the_file_on_disk":true`, `"stale":false`, `"file_bytes":505545`,
  `"scored_documents":2180`, `"identical":true`, `"matched":2,"of":2`, and `"max_abs_diff":31.0` beside the
  structural `basis`.
- one staging defect this round, worth keeping: the image was first packed with the HOST service table
  (`trm_uefi_kernel/host_supervisor/supervisor.services.conf`, Windows paths), so the supervisor inside the
  guest had nothing it could exec and the platform never bound — `NOTHING answered on :8022` while every
  other line of the boot looked correct. The bare-metal table is `trm_chainload/init/supervisor.services.conf`
  (`/opt/trm/host_council_server`). A boot that answers nothing is a packing error before it is a platform
  error.

## 18. The first boot on real hardware (2026-09-30)

It ran on a physical machine -- firmware -> chainloader -> kernel -> PID 1, on a board with an internal
NVMe disk -- and the machine's own screen carried the whole boot. What that settles:

- **the console works.** The board printed PID 1's voice on its own screen (`CONFIG_VGA_CONSOLE=y`,
  `CONFIG_FB_EFI=y`, `CONFIG_FRAMEBUFFER_CONSOLE=y`). This was the item an earlier revision of
  `docs/LINUX_PATHWAY_ON_USB_20260930.md` called a possible blocker; it is not one.
- **all 16 modules load on real silicon** -- `usb-common`, `usbcore`, `xhci-hcd`, `xhci-pci`, `ehci-hcd`,
  `ehci-pci`, `usb-storage`, `hid`, `usbhid`, `hid-generic`, and the six NIC modules: `16 loaded, 0 not
  shipped`. The machine's USB receiver enumerated (`HID 046d:c548 Logitech USB Receiver`) and the stick's
  own controller came up as `sda` (15.7 GB).
- **the NIC binds to the board's own chip**: `r8169 0000:03:00.0 eth0: RTL8125B, b4:45:06:4d:2c:51` and
  `NET: eth0 up (mac b4:45:06:4d:2c:51)`.
- **the kernel's built-in storage drivers find the machine's NVMe** (`nvme0n1` plus eight partitions),
  which is also what broke the boot -- see below.

What it did not settle on that boot, and what has since been done about it:

- **the NIC link**: `r8169 0000:03:00.0 eth0: Link is Down`, and the driver asked for
  `rtl_nic/rtl8125b-2.fw`, which the RAM image did not carry (`Direct firmware load for
  rtl_nic/rtl8125b-2.fw failed with error -2`). The file is now fetched from linux-firmware, pinned by
  sha256 in `trm_chainload/firmware/README.md`, and staged by `make_rootfs.py --firmware` (measured:
  `lib/firmware/rtl_nic/rtl8125b-2.fw 3328 B` in the RAM image). Whether a cable is on that port is still
  the machine's to answer -- but "no firmware" is no longer one of the candidates.
- **reachability, which was never claimed and is now measured**: an unconfigured machine had no address,
  so nothing outside could reach the dashboard even though the dashboard was up. The launcher now derives
  a link-local address from the MAC when `network.conf` sets none, and the boot's last lines name where
  the dashboard is (`DASHBOARD: ...`). On the emulated stick, with the port forwarded:
  `GET :15183/ui/ -> HTTP 200` and `<title>TRM SWARM OS | LCARS INTERFACE</title>` came back to a client
  OUTSIDE the guest (`backups/bare_metal_first_boot_20260930/dashboard_over_the_nic_configured.log`); the
  kernel binds `0.0.0.0` for both listeners, and the supervisor's own probe measures the dashboard
  ONLINE on the machine. What remains the operator's is the cable and, on a routed LAN, an address.
- **the volume race**: the boot reported no platform because the stick's device arrived after PID 1's only
  scan. Fixed and reproduced -- `docs/FIXES_DERIVE_20260925.md` section 9, and the boot now says
  `(found on pass 4, 750 ms in)` when it waits.

The drive was restaged with the fixed RAM image (`initrd.img` sha256 `9f0071578ffc81a2...`, mtime 18:40),
and the next boot on the same board is the fixed one -- measured from the machine's own log, read off the
stick's state image (`backups/bare_metal_first_boot_20260930/metal_boot_log_off_the_stick.txt`):

- `BLOCK DEVICES: nvme0n1p6 nvme0n1p5 ... nvme0n1 nvme0` then `nvme0n1p1 vfat but carries no
  rootfs.squashfs` -- the machine's own ESP, probed and rejected: exactly the case the emulated decoy
  reproduces;
- `(found on pass 7, 1500 ms in)` -- the wait the previous boot never ran;
- `STATE: trm-state.ext4 (67108864 B) -> the volume is writable for it` and `PERSISTENT ROOT: ...
  write-probe wrote and read back identical` -- persistence, on the machine;
- `STORES: ... .tercode (505545 B) ... .text.jsonl (1869777 B)` -- the full-coverage stores, linked off the
  volume;
- `SUPERVISOR: launched as pid 277`, then `[t+03s] GET /infrastructure/status -> HTTP/1.0 200 OK` and
  `GET /ui/ -> HTTP/1.0 200 OK` (the LCARS dashboard HTML);
- `init: 19 of the routes asked answered -- the platform is up; idling so the machine stays alive.`

The platform therefore runs on physical hardware, serving its routes and its dashboard from the stick. What
is still not settled there is the NIC link (`Link is Down`; `rtl_nic/rtl8125b-2.fw` is not in the image) and
anything past the volume handover.

## 19. The radio, staged (2026-09-30)

The machine's Wi-Fi is an Intel AX200 (`8086:2723`, Killer AX1650x) and the kernel that boots this drive
already carries the whole stack as modules. What was missing was everything that turns a module into a
device:

- **six modules**, in `modprobe --show-depends iwlmvm`' own order: `libarc4`, `led-class`, `cfg80211`,
  `mac80211`, `iwlwifi`, `iwlmvm`. The derivation mattered -- `libarc4` and `led-class` are dependencies a
  hand-written list would have missed, and a module not named in `mods[]` is not loaded at all.
- **four firmware files**, staged by `make_rootfs.py --firmware` and pinned by sha256 in
  `trm_chainload/firmware/README.md`: `iwlwifi-cc-a0-77.ucode` (1,368,100 B -- the name the module itself
  asks for, not a family guess), the signed `regulatory.db` + `regulatory.db.p7s` (load-bearing under
  `CONFIG_CFG80211_REQUIRE_SIGNED_REGDB=y`), and the RTL8125B's file from the previous round.
- **a report that cannot lie**: the launcher waits up to ten seconds for a wireless interface to appear in
  `/sys/class/net` and names it, or says none did and why that is one of exactly two things. It runs after
  the supervisor has already started the platform, so a machine without a radio pays the wait in a line of
  text rather than in a delayed boot.

Measured on the emulated stick: `iwlwifi.ko iwlmvm.ko -> 22 loaded, 0 not shipped`, and the image carries
the firmware byte-identical to the repository copies (read out of the initrd's own cpio, not inferred from
the build log). Emulation has no Intel radio -- `CONFIG_MAC80211_HWSIM` is not set -- so that boot prints the
honest absence:

    WIFI: no wireless interface appeared in 10 s. The modules are loaded, so what did not happen is one of
          two things: this machine has no radio of that make, or its firmware was refused -- and the
          kernel's own lines above say which.

The platform was unaffected while that wait ran: `GET :15183/ui/ -> HTTP 200` at 20 s, as before.

**What remains is the real work: a supplicant.** Association and the WPA2 four-way handshake are userspace,
so a driver plus firmware plus a regulatory database yields a radio that no network is on -- which is
precisely what the launcher's own line says. Until that exists, a `WIFI:` line on metal would be the honest
report of a working radio, not of a working network.

## 20. The supplicant: two halves proven, and the half that needs metal (2026-10-01)

The userspace that closes §19 lives in `trm_uefi_kernel/trm_wifi/`. It is dependency-free for the same
reason the supervisor is -- one `rustc` invocation, no crate graph -- which is why the one primitive std
does not offer, a netlink and an `AF_PACKET` socket, is issued as a raw x86_64 syscall in `src/syscall.rs`.

### The crypto, diffed against an implementation that is not it

`trm_wifi --vectors` prints every derived value for fixed inputs, and `trm_chainload/scripts/verify_wifi_crypto.py`
recomputes them with Python's `hashlib`/`hmac` and with an independent AES key-wrap. The output is a diff,
not an assertion: **pmk, ptk, kck, kek, tk and mic all agree**, and a GTK wrapped by `python-cryptography`
under the KEK the binary derived comes back out of `--unwrap` byte for byte. The artifact's own
`--selftest` passes 6 of 6 published vectors -- SHA1, HMAC-SHA1 RFC 2202, AES FIPS-197, and AES key-wrap
RFC 3394 in both directions, including refusing a corrupted wrap.

The nl80211 constants in `src/nl80211.rs` were **derived**, not remembered:
`trm_chainload/scripts/derive_nl80211_consts.py` walks `include/uapi/linux/nl80211.h` at v6.6 and prints
them. That is not ceremony -- the first draft from memory had `NL80211_CMD_NEW_KEY` and
`NL80211_ATTR_EXT_FEATURES` both wrong, and a wrong attribute number is a request the kernel misreads
rather than refuses.

### The netlink layer, answered by a real kernel

`trm_wifi --probe` resolves the `nl80211` generic-netlink family, then asks for the wiphys and interfaces.
Run against WSL's own kernel it reports:

```
wifi: nl80211 is family 33 (asked of the kernel, not assumed)
wifi: no wiphy is registered -- the modules are loaded, but no radio has bound
```

Family 33 is the kernel's answer to a `CTRL_CMD_GETFAMILY` request, not a constant in the source. The zero
wiphys is this machine's truth: WSL has `cfg80211` (the family resolved, so it is loaded), but no radio has
bound, so there is nothing here to associate with.

### In the image, before the handover

The supplicant is staged as `/trm_wifi` by `make_rootfs.py --wifi` and run by the launcher, and the boot
prints its words: the two lines above, followed by the platform's own routes, `GET :15183/ui/ -> HTTP 200`
and `GET :18022/infrastructure/status -> HTTP 200`. Evidence: `backups/bare_metal_first_boot_20260930/wifi_supplicant_boot.log`;
the image is `60043448823d6246…`, 3,875,497 B, restaged to `E:/EFI/TRM/initrd.img`.

Two defects were found by that boot and both are worth recording, because each is a class:

- **Where it ran.** The call was placed after the supervisor launch, which is after `persistent_root()` has
  pivoted into the squashfs-overlay root -- so `/trm_wifi`, which lives in the initramfs, was no longer a
  path that existed, and the exec failed with `ENOENT`. The fix is placement, not copying: the supplicant
  must run while the RAM image is still the root, which is also when an associated radio is worth most.
- **What the failure said.** The first version printed `/trm_wifi is not in this image` on exit 127. That
  was a message asserting a cause it had never measured -- the exact fabrication the voltage doctrine
  names -- and it was *false*: parsing the initrd showed the file present, 651,192 B, mode 0755. The child
  now prints its own `errno` (`ENOENT`, `EACCES` and `ENOEXEC` are different facts), and the parent says
  only that it could not be started.

### What is still open

**Association.** `NL80211_CMD_CONNECT` with the SSID and PSK, the four-way handshake's EAPOL frames on an
`AF_PACKET` socket, the PTK/GTK install, and the address. The primitives are proven and the constants are
derived; the state machine is not written. And the honest limit stands: no machine in this environment has
a wireless controller, so **the handshake has never run against an access point, and cannot be measured
here** -- the boot itself says `no wiphy is registered` when asked.

## 21. The association, wired and bounded (2026-10-01)

The state machine is written: `src/eapol.rs` parses and builds the four frames, `src/supplicant.rs` sends
`CONNECT` and then answers the handshake if the driver will not, and `--associate <iface> <conf>` is the way
in. Three things in it are worth recording because they are where a handshake fails quietly:

- **The MIC covers the frame with the MIC field zeroed.** Message 2 and message 4 are built with that field
  as zeros, the digest is taken, and only then is the result written into it. A frame whose MIC was taken
  as-received is discarded by the AP, and a discarded frame is silence rather than an error.
- **The key goes in `NL80211_ATTR_PMK` (254), and the value it takes is the PMK** -- the 32 bytes
  `pbkdf2_sha1_32` already produces. There is no `NL80211_ATTR_PSK`; the header says so explicitly, and a
  hand-written `ATTR_PSK` would have been a request the kernel misreads.
- **Message 3's MIC is verified and its wrap is refused if it does not check.** The AES key-wrap integrity
  word is the only thing that catches a PTK derived from the wrong nonces, so `aes_kw_unwrap` returning
  `None` is a failure here, not a wrong GTK.

**What is proven.** `trm_chainload/scripts/verify_wifi_crypto.py` -- an implementation that is not this one
-- now checks the handshake frames as well as the primitives: the PTK recomputed from the printed nonces,
the msg2 and msg4 MICs recomputed over the frames with the MIC field zeroed, the RSN element the AP sent
inside message 1 echoed back byte for byte, and a GTK wrapped by `python-cryptography` returned unchanged.
All agree on the artifact (`0f80096f…`, 702,376 B).

**And the boot runs it.** The launcher attempts the association when the volume carries
`/mnt/esp/wifi.conf` -- beside its own `network.conf` and in the same shape -- before the handover, and says
so plainly when it does not:

```
WIFI CONFIG: /mnt/esp/wifi.conf absent -> no association is attempted; a station with no credentials
             and a station whose passphrase is wrong must not read the same here
```

The constants were re-derived and the round before's file had one wrong
(`NL80211_ATTR_CIPHER_SUITES_PAIRWISE` 81, correct 73) -- `derive_nl80211_consts.py --check <file>` now
diffs all 34 of them against the kernel's own header, because nothing inside this program can distinguish a
wrong attribute number from a driver that did not answer.

**What is still not proven: an association.** No wiphy is registered on this machine, so `CONNECT` has never
been sent to an access point and no four-way handshake has run against one. The frames are proven; the
exchange is not.

---

# 21. The state machine, exercised without a radio -- and the boot taught to check content

## 22. The machine boots the 22-module image, the radio binds, and the radio finds the first bug

The stick was booted on the hardware. Two things that were open are closed, and one defect that only this
machine could expose was found and fixed.

**The 22-module image boots.** The machine's own log reads `22 loaded, 0 not shipped`. The earlier boot was
the 16-module image, which is why no radio module appears in it; the 22-module image had never met this
board, and on its first attempt every module binds.

**The radio binds and the firmware is accepted.** The log carries `NET: wlan0 up (mac c8:cb:9e:5f:0e:0b)`
and then the launcher's own report, which names the interface, the driver and the state:

    WIFI: wlan0 present (iwlwifi, mac c8:cb:9e:5f:0e:0b, link down) -- the driver bound and the firmware
    was accepted

`link down` is a link state, not a failure: there is no access point in range of that machine, and the
line says so rather than implying one. The driver loaded, the firmware for the AX200 was accepted, and
`iwlwifi` published a wireless interface. Before this boot, that had never happened.

**The radio found a bug that nothing else could.** The same boot reports `WIFI PROBE: exited 101` -- a Rust
panic, and the first time this code had ever panicked. The cause is in the nl80211 attribute walk:

    out.push((ty & 0x3fff, buf[4..len].to_vec()));
    buf = &buf[align4(len)..];          // the advance is ALIGNED; the buffer need not be

The walk bounds-checks the attribute's own length and then advances by its *aligned* length. A kernel
message does not always carry the trailing padding of its last attribute, so that aligned advance lands
past the end of the buffer and the slice index panics:

    range start index 12 out of range for slice of length 9

This could not have been found anywhere but on a machine with a radio. `MAC80211_HWSIM` is unset in WSL, so
there is no virtual wiphy; with no wiphy there is no dump reply, and with no reply the loop never walked an
attribute. Every emulated run -- WSL, the QEMU harness, the metal boot's own earlier images -- executed a
`for` loop over an empty vector. The first real reply this walk ever saw was the one the AX200 sent.

The fix is to treat an aligned advance past the end as the end of the message rather than as an index:

    let advance = align4(len);
    if advance > buf.len() {
        break;
    }
    buf = &buf[advance..];

The test that proves it is the case that failed, and it goes red on the code the stick carried:
`netlink::tests::a_trailing_attribute_without_its_padding_does_not_panic` panics with exactly the line above
against the shipped walk, and passes against the fixed one. `trm_chainload/scripts/check_attr_walk.sh`
re-runs that, so the ledger block about this round re-derives instead of quoting.

**And the log now carries the child's own words.** The panic message reached the screen and never reached
`/trm_boot.log`, because `run_supplicant` let the child inherit the console while the log is a tee of
`say()`. A reason that scrolls past on a console is a reason nobody can read afterwards from another
machine, which is the one thing a boot log exists to prevent. The child's stdout and stderr now come back
through a pipe and go into the log line by line -- a failed probe leaves its reason on the log, which is
where the next person will look for it.

**What this boot proves and what it does not.** The driver bound and the firmware was accepted, on the
hardware, and the platform's own content assertions pass on it: `19 of the routes asked answered -- the
platform is up; 9 of them returned the content they exist to return`. It does not prove an association. No
`wifi.conf` was present on the stick, so no association was attempted, and the log says so. `wlan0` in
managed mode with a bound driver is the precondition for `NL80211_CMD_CONNECT`, not the command.

Image on the stick: `8bbe1164919a6934`, 3,906,900 B.

## Why the frames were not enough

Section 20 proved the primitives and the frames. It did not prove the *state machine*: which message
answers which, what a replay counter that does not advance means, whether a MIC is checked before a key is
installed. None of that could run here -- no machine in this environment has a wireless controller, and
`CONFIG_MAC80211_HWSIM` is not set in the kernel this pathway boots, so there is no virtual radio to load
either. The first real handshake would have been the first test.

## The transitions, out of the socket

They had to become callable before they could be driven: `on_message_1(pmk, bssid, spa, snonce, frame)` and
`on_message_3(ptk, snonce, msg1_replay, frame)` are pure -- the frame in, the answer out -- and the
association loop calls them in the order it always did, with the key install still between the MIC check and
message 4. A socket is not a thing a test can hand a frame to; a function is.

## `--harness`: an access point that exists only as bytes

It derives the PTK the way the AP would, builds message 1 with a real RSN element and message 3 with a
wrapped group key, and then drives the transitions:

    harness: ok       message 1 classifies as message 1
    harness: ok       the two sides derived the same PTK
    harness: ok       message 2's MIC verifies under the PTK
    harness: ok       message 2 echoes the RSN element byte for byte
    harness: ok       the group key came back out of the wrap
    harness: ok       message 4 carries no key data
    harness: refused  message 3 with a corrupted MIC -- message 3's MIC did not verify against the PTK
    harness: refused  message 3 checked against a PTK from a different passphrase -- ...
    harness: refused  message 3 that does not advance the replay counter -- carried 1, not past 1
    harness: refused  message 3 with a counter behind message 1 -- carried 0, not past 1
    harness: refused  message 3 whose key wrap fails its integrity check -- ...
    harness: refused  message 1 with no RSN element to echo -- ...
    harness: 20 case(s) held or refused as required, 0 failed

Two of those are additions, not tests of what existed. **The wrap-integrity refusal is new**: an encrypted
key data block that would not unwrap used to leave the group key silently missing while the pairwise key
installed. And `aes_kw_wrap` had to be written, because a message 3 has to be built by something and
building it with anything but the algorithm the unwrap checks against would make the harness test a fiction.

## The oracle disagreed on the first run, and it was right to

`verify_wifi_harness.py` recomputes the PMK, the PTK, both MICs -- over the frames themselves, with the MIC
field zeroed -- and unwraps message 3's key data with python-cryptography rather than with this code. First
run: `9 checks agreed, 4 disagreed`. The disagreement was in the **oracle**: the existing crypto oracle
hardcoded its own station's MACs, so for a harness with a different pair it derived a different PTK. Correct
binary, wrong comparison. With the addresses parameterised, **14 of 14 agree**.

## The boot learned to check content, and the check found two defects -- both its own

A 200 says a socket answered. It does not say the ternary branch ran, that the tercode route cross-checked
its own ranking, or that the dashboard is the LCARS page rather than any page at all. Nine routes are now
asked for a string that branch cannot produce without having run -- the needles measured off the running
platform, not invented. And the boot says both numbers:

    init: 19 of the routes asked answered -- the platform is up; 9 of them returned the content they
    exist to return.

The first run of that check reported **all nine** routes as answering without their content. The defect was
in the check: `http_req` NUL-terminated the response at the status line so the status could be printed, and
the content check then searched that same buffer -- a string that ends before the body. The second run
reported **90 of them returned the content** for nine checks: the block was inside the ten-pass retry loop,
so the counter accumulated. Both are the same error class this pathway keeps finding -- a value shaped like
a measurement, describing something else -- and both were caught by the check disagreeing with itself
rather than by review.

One more thing came out of the same run: printing every body, ten times over, on an 11 KB/s serial line. The
content probes are quiet now, and the boot's console output fell from 350 KB to a quarter of that.

## What this changes about the boundary

The state machine is exercised; the exchange still is not. `CONNECT` has never reached an access point, the
four-way handshake has never run against one, and a driver that insists on doing the handshake in firmware
has not been met. What exists now is that the parsing, the nonce handling, the replay rule, the MIC check
and the key unwrap all run before hardware is involved -- which is the most a machine without a radio can
honestly provide, and the thing the earlier increment could only name as missing.

The image on the drive is the one that produced the lines above: `74bf16196e2a935b`, 3,906,442 B.

## 23. The walk's whole class is exercised, and the boot's own verdict stops underselling itself

The increment before this one fixed the arm of the bug that metal found. The review asked the right question
about it: the arm is fixed, but has the class been exercised? The answer was no, and the two walks are the
reason. Both are now pure functions with a battery over them, and the battery found something -- in the
battery.

**Both walks are pure and covered.** `netlink::messages_in` is the datagram walk, lifted out of
`recv_deadline`; `nl80211::dump`'s filter-and-parse is the message walk. Neither needs a kernel now, so the
shapes a real kernel sends can be handed to them directly: an empty datagram, a header with no body, a
length of zero, a length below `sizeof(nlmsghdr)`, a length past the end, two messages in one datagram, a
truncated second message, an unaligned length, `NLMSG_DONE`, `NLMSG_ERROR` with and without a code, an
unknown type, a zero-length attribute, an empty attribute, non-zero padding, a nested attribute, and a
trailing attribute without its pad. Twenty-one cases, and they run in the crate's own `cargo test`.

**The second site of the same shape is safe, and the battery is what says so.** `recv_deadline` advanced by
`align4(len)` too. There the advance is only *compared* against the buffer, never used to index, so it exits
the loop instead of panicking -- but that was an argument until the cases ran. `an_unaligned_message_length_
still_steps_to_the_next` and `a_truncated_second_message_does_not_lose_the_first` are that argument turned
into a measurement.

**The battery's one failure was in the battery.** `an_unaligned_attribute_is_followed_by_the_next_one`
asserted that an attribute whose length is not a multiple of four, placed in the *middle* of a message, is
followed by the next attribute. That is false: the kernel pads every attribute it sends, so an unaligned
length can only begin at the very end -- where the buffer's end makes the pad unnecessary and the fix above
covers it. In the middle it means the message is not what it claims, and the walk stops rather than reading
the next header one byte out of place. Stopping loses the tail of a malformed message; guessing invents
attributes that were never sent. The test now asserts that, which is the contract the code actually keeps.
This is the third time this campaign has caught a checker measuring something other than what it claimed --
the truncated body pointer, the crypto oracle's hardcoded MACs, and now a test whose premise was wrong.

**And the boot's verdict stopped underselling itself.** The line read: *19 of the routes asked answered --
the platform is up; 9 of them returned the content they exist to return.* Read plainly, that says ten routes
returned the wrong content, and it is wrong: nine routes were asked for content, the other ten were asked
only whether they answer. The verdict now gives three numbers that each name themselves, and the checked set
is listed in the log:

    CONTENT: 9 of the routes were asked for a specific string and 9 returned it. The checked paths: /ui/,
    /infrastructure/status, /council/status, /parsers/status, /sram/architectures, /k3/status, and
    /k3/retrieve on the exact path, cross-checked against tercode, and on the packed alphabet. The rest were
    asked only whether they answer.
    init: 19 of the routes asked answered -- the platform is up.
    init: 9 of them were asked for content and 9 returned it; the other 10 were asked only whether they
    answer, which is a different question and not a failure.

**The Wi-Fi line no longer reads a normal state as a fault.** For a station that has never joined a network,
`operstate down` with carrier 0 is the expected state, and carrier comes up when the station joins one. The
line reads the kernel's own `carrier` file rather than inferring anything from a driver log, and says what
the state means. The association path now names which of three different facts it is -- no credentials, no
access point in range, or an attempt the driver refused -- because those three must not read alike in a
boot log, which is the same rule that makes the unconfigured-radio line exist at all.

Image on the stick: `5daa24e4936c0638`, 3,909,028 B.

## 24. Reproducibility, measured per stage: the compressor was the whole difference

The review asked for the one thing that would let the discipline scale: the same source producing the same
artifact, so a boot-test carries forward to the next change. The answer is now a measurement per stage
rather than a verdict on the pipeline -- `trm_chainload/scripts/check_reproducible.py` builds each stage
twice from the same inputs and compares the results by content.

| stage | before | after |
|---|---|---|
| cpio, packed twice in one environment | same | same |
| cpio, packed on Windows vs inside WSL | **differed** | same |
| the RAM image, Windows vs WSL | **differed by 67,475 B** | same |
| `platform_init`, compiled twice with gcc (WSL) | not measured before | same |
| `trm_wifi` musl, the crate recompiled twice | not measured before | same |
| `rootfs.squashfs`, the same tree packed twice | **differed** | same |

**The archive was never the problem.** The cpio header is already deterministic -- inode numbers are a
counter, `mtime` is 0, `uid`/`gid` are 0, and entries are sorted by depth then name -- and the gzip header
already carries `mtime=0`. The measured difference was entirely the COMPRESSOR: the same 13,291,564-byte
archive is 3,976,503 B through Windows' `zlib 1.3.1.zlib-ng` and 3,909,028 B through Linux's stock
`zlib 1.3.1`, at the same level. A compressed image is therefore reproducible only in the environment that
built it, which is exactly the property that makes a boot-test not carry forward.

So the RAM image ships UNCOMPRESSED by default: a plain cpio has no compressor to vary, and the kernel
takes an uncompressed initramfs -- which was measured by booting it, not assumed. The image went from
3.9 MB to 13.3 MB, which is nothing on a 15.7 GB stick. `--gzip <level>` is kept for size, and says what it
costs.

**The squashfs differed by exactly one byte.** Two packs of the same tree, 1.2 seconds apart, were
16,461,824 B each with one differing byte -- at offset 8, the superblock's `mkfs_time`. `-all-time 0` pins
every file's timestamp and leaves the filesystem's own alone. `-mkfs-time 0` closes it, and the two packs
are then byte-identical with zero differing bytes. `-no-exports`, `-processors 1` and a pinned
`-Xcompression-level 9` are in the same command for the same reason.

**The compilers are deterministic at the versions in use** -- and that is the honest limit of the claim.
gcc 15.2.0 compiles `platform_init` twice to the same bytes, and the musl crate recompiled twice (a real
recompile, not a cache hit) gives the same binary. Different versions of either would give different
output: the toolchain versions are RECORDED here, not enforced. A pinned toolchain is the next step, and
the distinction matters -- this is reproducibility given the toolchain, not despite it.

**Two of the checkers were wrong before the checkers were right**, which is worth recording because the
pattern is now unmistakable. The first probe compared two packs made inside the same second, and a
timestamp field has second resolution -- it reported `SAME` for a pipeline that was not. The second
compared two files that did not exist (`$TMPDIR` was empty), and two absent files hash equal in a shell
one-liner -- it reported `same=yes` for a measurement that never happened. That is the fifth and sixth
instance this campaign of a checker measuring something other than what it claimed.

Image on the stick: `583f0dd4f25a9f88`, 13,291,564 B, and `rootfs.squashfs` `55cd5d28679edff9`. Both were
booted after restaging; the run reports `19 of the routes asked answered -- the platform is up` with the
dashboard answering from outside the guest.

**The checkers had to learn the new shape too.** `check_boot_chain_static.py` decompressed the initramfs
unconditionally and died on the plain cpio with `BadGzipFile: Not a gzipped file (b'07')` -- the newc
magic. It reads both shapes now. A tool that only understands the old format fails on the new one for a
reason unrelated to what it checks, and the sealer refusing the block for it is the discipline working:
the refusal was correct and the checker was wrong.

## 25. The boot now ends where to look

The doctrine was "end the boot with a URL line", and the implementation put the URL before the counts --
so the last thing an operator standing at the machine read was a verdict, not a destination. That is fixed:
the counts come first, and the final two lines are the dashboard.

    init: 19 of the routes asked answered -- the platform is up.
    init: 9 of them were asked for content and 9 returned it; the other 10 were asked only whether they
    answer, which is a different question and not a failure.

    DASHBOARD: http://127.0.0.1:5183/ui/ -- open this on this machine; it answered the LCARS page on
               loopback during this boot.
               or http://10.0.2.15:5183/ui/ from anywhere that can reach this box (eth0 is linked).

The loopback claim in that line is not a hope: the content probe above asks `/ui/` for the LCARS title on
this machine's own loopback and counts the answer, so "open this" is backed by a measurement made four
lines earlier. The network form is offered only when the interface reports carrier -- an address nothing can
route to is not an address -- and when there is no cable the line says that the cable is the missing thing,
not the platform.

Image on the stick: `a84a106701ea70e4`, 13,291,564 B.

## 26. The third metal boot: the radio answers, the walk completes, and the last line stops asserting

The stick was booted on the board with the AX200 again. This is the boot the attribute-walk fix was waiting
for, and it settles it.

**The probe that exited 101 now completes, on the machine that broke it:**

    WIFI PROBE| wifi: nl80211 is family 33 (asked of the kernel, not assumed)
    WIFI PROBE| wifi:   phy0 -- 0 ext-feature bytes, four-way handshake in the kernel: no
    WIFI PROBE| wifi:   wlan0 ifindex 3 of 0 type 2 mac c8cb9e5f0e0b

That is the walk reading a real kernel reply from a real radio: the family id asked of the kernel rather
than assumed, the phy enumerated with its extended-feature count, and the interface with its index, type
and MAC. The panic was in the second of those lines -- `parse_attrs` advancing by `align4(len)` past the end
of a message whose last attribute carried no trailing pad -- and it is gone. A test reproduced that panic
off the machine and this boot reproduces its absence on the machine.

**And the child's own words are in the log.** The `WIFI PROBE| ` prefix is the pipe into `say()`: a child
that used to inherit the console and leave nothing behind now writes its report into `/trm_boot.log`, which
is why these three lines can be read from another machine at all. That was the other fix from the same
round, and this boot is the first that could have shown it working.

**The rest of the boot is the same machine doing what the pathway claims.** `22 loaded, 0 not shipped`; the
volume found by content on pass 7 after the internal NVMe was probed and rejected for carrying no
`rootfs.squashfs`; both interfaces up with link-local addresses derived from their own MACs
(`169.254.15.12` from the radio, `169.254.45.82` from the Ethernet) because nothing was configured and no
router was there to ask; `a write from an earlier boot: PRESENT`, so the writable layer still outlives the
reboot three boots on; `WIFI ASSOCIATION: not attempted -- /mnt/esp/wifi.conf is absent`, with the note that
no credentials, no access point and a refused association are three different facts; `pivot_root` refusing
and `chroot` carrying the handover, disclosed; and then the content check, the counts, and the dashboard.

**The boot found one more defect, in the line an operator reads last.** It said:

    NET: eth0 up (mac b4:45:06:d4:2c:51) [no carrier: nothing on the other end of that cable], 169.254.45.82/16

and thirty seconds later:

    or http://169.254.45.82:5183/ui/ from anywhere that can reach this box (eth0 is linked)

Both are the launcher's own words about the same interface, and they disagree. The second was a verdict
derived from a second reading rather than the reading itself -- the defect class this pathway keeps finding
in its own instruments, this time in its final line. The line now prints both numbers: the carrier as read
at bring-up and as read now, and only what those readings support. Whether the carrier genuinely came up
between the two moments (a cable going in during the boot) or the driver reported it late is a question the
next boot answers with two numbers instead of a contradiction -- which is what a log is for.

Image on the stick: `666ffeedd75d88be`, 13,291,604 B.

## 27. The cable: a link that is a negotiation, and a client that says what happened

With a cable in the machine's Ethernet port, the boot had two things to learn.

**The carrier is not a property of the interface, it is a negotiation.** Read in the same breath as
`IFF_UP`, it can say 0 on a cable that is live, because the driver is still bringing the link up. Measured
head-on: one run of one binary in the same harness read `carrier 1`, another read `carrier 0` and printed
`[the link came up while it was waited for]` after waiting. This is also what §26 could not explain -- the
metal boot that said `no carrier` in its own NET line and `(eth0 is linked)` thirty seconds later was not
contradicting itself, it was reading a link that came up in between. The boot now waits for the link,
bounded at three seconds, and says when it had to.

**A machine with a link and no configuration should ask the router.** The boot had no DHCP at all: nothing
in `network.conf` meant an address derived from the MAC (169.254/16), which reaches a directly-cabled
machine and nothing on a routed LAN -- so a dashboard that only the machine itself could open. There is now
a client: four UDP datagrams, no library, three attempts, 1.5 s per wait, every receive bounded, because a
boot that blocks on an answer that never comes is a machine that never finishes booting. It also has to be
able to say *why* it did not work, which is where this round's real work was.

**The client reports its own failure, and the report is what found the fault.** The first version said
"nothing came back". That is a claim about a router, and it was not yet supported by anything. It now prints
the route's fate, the `sendto` result or its `strerror`, the kernel's own count of UDP datagrams delivered
to sockets across the wait, and the interface's transmitted/received counters. The verdict on the test
network:

    [dhcp] eth0: 300 bytes of DISCOVER were sent and nothing came back. The kernel delivered 0 UDP
    datagram(s) to sockets during the wait (0 -> 0), so nothing arrived at all and the answer was never
    sent to this machine
    [dhcp] eth0 counters across the attempts: transmitted +10 packet(s), received +11

So the DISCOVERs reached the wire and packets came back, but no UDP datagram was ever delivered to a socket:
QEMU's own user-mode DHCP server did not answer this client. **The client is therefore unproven against a
real DHCP server** -- what is proven is that it builds a correct BOOTREQUEST, gets it onto the wire, waits
bounded, and reports honestly when no answer comes. The machine with the cable and a real router on the
other end is the test, and the boot log carries the verdict either way, including the fallback: with no
answer the address is the same derived 169.254/16 as before and the line says no router answered.

**The harness that runs these boots was lying about being up.** It printed `qemu up` after
`pgrep -f qemu-system-x86_64` matched a QEMU left over from an earlier run -- one that still held the
forwarded ports, so the new QEMU died on `Could not set up host forwarding rule` while the script reported a
machine running, and a boot that never happened was measured for four minutes. A stale instance answering
for the current one is the same defect as a wrapper reporting health it never checked. The harness now reaps
what it left behind before starting, checks *its own* sockets rather than whether some QEMU exists anywhere,
and cleans up on exit. Two smaller lessons from the same seam: the kernel truncates a process name to 15
characters, so `qemu-system-x86_64` is `qemu-system-x86` to `pkill -x` (which is why `-x` "found nothing"),
and a Python rewrite of a shell script on Windows writes CRLF, which is the `\r` that broke this script's
own `sleep` and `for`.

Image on the stick: `fc997596840828ed`, 13,300,300 B.
