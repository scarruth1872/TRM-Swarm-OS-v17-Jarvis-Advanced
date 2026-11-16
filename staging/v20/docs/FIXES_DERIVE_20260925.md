# Fixes: derive, don't assert (2026-09-25)

The validation round named four defects that all turned out to be one class — a value **declared** where a
value should be **measured** — plus the strip's inability to tell its own patterns apart. Each is fixed at
the source and each fix has a live receipt.

## 1. `sram_pattern_computer.py` — declared values replaced by derived ones

| field | was | now |
|---|---|---|
| `L3_SIZE` | literal `32 * 1024 * 1024` | probed: kernel `/system l3_sram_kb` → OS `Win32_Processor.L3CacheSize` → documented target, with the winning basis reported as `L3_SIZE_BASIS` and cached for 24 h |
| `summary()['cpu']` | literal `'Ryzen 9 3900X'` | `HOST_CPU` → **AMD Ryzen 9 5900HX with Radeon Graphics** |
| `benchmark_all_native().speedup` | literal `'1910.6x'` | measured: the strip's `rc` and output array are read, and the Python path is timed over the same architectures for a real ratio (**11.3×**) |
| `summary()['l3_cache']` | `32MB` off the literal | `17MB` off the probe, with `l3_basis` and `host_l3_matches_design_target: false` |

The module and the runtime no longer disagree about the same silicon: `/system` measures 16,384 KB and the
module now reports that, with its basis named.

## 2. The SRAM strip — a reconfigurer that could not tell patterns apart

`swarm_v2_artifacts/native_voltage_strips/strip_sram_computer.rs` hashed `0xAA` for `idx == 0`, `0x01` for
`idx == 1`, and **one generic `0x5A`/`0xA5` line for every other index** — 15 architectures, **3 distinct
outputs**, and it never received the patterns it claimed to reconfigure.

Now generated from the module (`scratch/gen_strip_patterns.py`), hashing the pattern the index names:

- `rc = 0`, **15 distinct outputs of 15** (was 3), regression control `idx2 != idx3` pinned in the ruler.
- **Cross-language parity 7/7** (`scratch/verify_strip_parity.py`): every index's output equals the FNV-1a
  that Python computes over the same pattern bytes — Rust and Python agree, so the two cannot drift silently.
- Honest correction in the numbers: the old `0.0769 µs` per op measured a constant line; hashing the real
  patterns (one is 32,768 bytes) costs **2.23 µs** per reconfig, so the measured ratio is **11.3×**, not the
  asserted 1910.6×. The old figure was not a speedup — it was the cost of not doing the work.

## 3. `host_tranche/src/sram_pattern_computer.rs` — a 1:1 port that was 0:1 and never compiled

It claimed "Constants and structure extracted 1:1" while holding `L3_SIZE = 0`, `L3_CELLS = 0`,
`ARCHITECTURES = 0` and 15 functions returning `0 // TODO: complete 1:1 math` — and it was **dead source**:
`host_tranche/src/main.rs` declared only `mod radix_trie_matcher`, so nothing in the file was ever compiled.

Now generated from the module (`scratch/gen_tranche_sram.py`), with the module's parameterised arithmetic
translated and **checked against Python before emission** (a formula that disagrees raises), declared in
`main.rs` so it compiles, and carrying captured values as a Rust test:

```
test sram_pattern_computer::python_parity::every_architecture_matches_python ... ok
test sram_pattern_computer::python_parity::no_function_returns_a_placeholder ... ok
test sram_pattern_computer::python_parity::constants_are_consistent_with_the_geometry ... ok
test result: ok. 3 passed; 0 failed
```

`every_architecture_matches_python` compares all 15 architectures' numeric fields and pattern hashes against
the Python module's, and `no_function_returns_a_placeholder` is the anti-regression: no empty pattern, no
all-zero pattern, no zeroed field. The same generator reported which fields vary with their parameter rather
than assuming it — it stopped the run on `neural.neurons` until passthroughs were detected instead of guessed.

## 4. My own probe: a classifier that filed an array as empty

`/experts` and `/swarm/experts` were reported as `EMPTY_BODY`. They serve a **15-element JSON array** of
agents; the classifier only counted non-empty **objects**. Fixed to read the body's shape — the route sweep
is now `ANSWERS 48, HONEST_REFUSAL 4, NOT_HANDLED 2, HTTP_400 2`, with no empty bodies, and the capability
assertions still 9/9.

## Still open, named rather than implied

- **4 of the 5 skeleton modules** in `host_tranche` remain: `chaos_engine`, `expert_registry`,
  `gpu_infinity_computer`, `persona_matrix`. 23 of that crate's 28 files are still not declared in `main.rs`.
  **Resolved as a classification 2026-09-26 (block 158):** the crate was never compiled and nothing in the
  repository names it; its real gap is now measured per function -- 123 of 232 python target names have a
  live counterpart (53 %), most of the remainder are naming differences verified concept by concept, and six
  residual candidate groups remain: `docs/P7_HOST_TRANCHE_CENSUS_20260926.md`.
- `/system/modules` returns **empty lists** for `active_core` / `native_rust_engines` while `summary` has six
  populated keys — a census answering nothing about a non-empty set. Not touched this round.
- `/aura` still refuses with `engine_required`.

**CORRECTION (this line previously read "`/system` still reports `fabricated_modules_count: 5`").** Re-running
the platform's own producer, `swarm_v2/core/build_module_registry.py`, gives `/system`
**`fabricated_modules_count: 3`, `real_modules_count: 226`, `modules_compiled: 278`**, read live from
`module_registry.json`. Two audited signatures are gone — `ollama_knowledge_harvester` and
`turbovec_ternary_knowledge_fusion` — and the scan prints them under `STALE AUDIT: signature gone ... audit no
longer applies`, which is the correct behaviour: the audit retires itself when the code it indicted changes.
That drop is **not** this round's doing; the re-scan simply measured it. Also corrected here: the five
`TODO: complete 1:1 math` skeletons in `host_tranche` are a **different set** from the kernel's FABRICATED
count. FABRICATED is driven by `AUDITED_FABRICATED`, five audited names (`knowledge_rag_and_semantic_search`,
`knowledge_fabric_supremacy_campaign`, `ollama_knowledge_harvester`, `turbovec_ternary_knowledge_fusion`,
`swe_marathon_challenge_engine`); the two sets coincidentally both numbered five and were never the same
modules. `sram_pattern_computer` was never in the audited set — the regenerated port moved out of `PORTED`
(45→40) and into `REAL` (219→226) as a side effect of the content rule (`pub fn` and >1500 chars).
The kernel binary was built before this scan; the generated `module_registry_generated.rs` carries the new
constants and needs a rebuild to be baked into the image — the live route already reports them because it
reads the artifact at request time.

## 5. `GET /sram/architectures` — the same literal, still served (2026-09-30)

§1 fixed the Python module; the **served** route had not caught up. `/sram/architectures` computed 15
topologies from the shared Rust module and took the L3 block straight off its constants, so the route that
reads like a hardware measurement was reporting a declaration:

| field | served before | served now |
|---|---|---|
| `l3.size_bytes` | 33554432 (the module's literal) | **16777216** (the host's L3, probed) |
| `cells` | 268435456 | **134217728** |
| `sets` | 8192 | **4096** |
| `transistors` | 1610612736 | **805306368** |
| `l3_basis` | *absent* | `measured: host_probe.rs -- ...` |
| `declared_default` | *absent* | 33554432, kept visible beside the measurement |
| `host_l3_matches_design_target` | *absent* | **false** |

`host_council_server/src/sram_handlers.rs::l3_block()` now takes the size from `host_probe::l3_cache_kb()`
(summed over the cache instances the host reports — one instance's size is not the machine's L3); every other
field is derived from the size actually served, so the block cannot contradict itself. When the probe cannot
answer, the module's literal is served with the basis saying `assumed:`. The field vocabulary and the
arithmetic are the module of record's (`L3_SIZE_BASIS`, `host_l3_matches_design_target`, and the same
`cells = size * 8`, `sets = size // (line * ways * slices)`, `transistors = cells * 6`).

`host_probe.rs` also gained the Linux arm (`/sys/devices/system/cpu/cpu0/cache/**`, distinct instances told
apart by `shared_cpu_map`) — what the bare-metal platform needs in order to measure the machine it is
actually running on instead of quoting a declaration.

Receipts: `cargo test --release` **32 passed / 0 failed**, including the two rulers that used to *count* this
divergence — `python_constants_match_the_rust_bodies` and `known_declared_vs_measured_divergences` — which now
assert against what is served rather than against the literal. Live `:8022` serves the block above, with
`/k3/status`, `/parsers/status`, `/infrastructure/status`, `/system` and the `:5183` dashboard all 200.
`host_l3_matches_design_target: false` is the honest reading: this host is not the documented 32 MB part.

## 6. `GET /k3/retrieve?engine=tercode` — a report about a store the process was not using (2026-09-30)

The route reported its tercode store as fresh, 186 rows, 45,453 B, minutes after the file on disk had been
rebuilt to 2,180 rows, 505,545 B. Every field was true of *something*: `freshness` was computed from a live
`stat` of the FILE against the corpus index's mtime, while `bytes` and `vectors` came from the value a
`OnceLock` had loaded at the process's first read. The report described the disk; the routing served the
memory — the same defect as §1–§5 in another costume, a field that reads like a measurement of the thing
being served while being derived from a different source.

| | before | after |
|---|---|---|
| tercode store, served | 186 rows / 45,453 B | 2,180 rows / 505,545 B |
| tercode store, reported | no file fields; `freshness` read from the file | `file_bytes`, `file_mtime`, `serves_the_file_on_disk` — read at report time |
| packed `.turbovec` | 180 vectors / 45,829 B | 2,180 vectors / 525,297 B |
| `alphabet=trit` verify | `DIVERGED`, `max_abs_diff 31.0`, no stated basis | the same measured numbers, with `basis` naming them structural and pointing at the store that does reproduce the engine |
| corpus index | loaded once; a rebuilt corpus needed a restart | re-read when the file's signature changes |

**Root cause:** three `OnceLock`s — the corpus, the packed store, the tercode store — made the first read
permanent. `trm_uefi_kernel/host_council_server/src/file_cache.rs` replaces all three with a value keyed on
the file's own `(mtime_ns, len)`: one `stat` per call, a re-read when it changes, and a rejected load
cached under the same key so a broken file is not re-parsed on every request. Three tests pin that
contract, including that an unchanged file is not re-read.

**The stores were the other half.** Both had been built when the corpus held 180/186 documents. Rebuilt
from the current 2,180-document corpus (offline tooling `swarm_v2/core/build_nand_fabric_index.py`; kernel
route `/k3/tercode/build`), the independent verifier `swarm_v2/core/verify_nand_fabric_index.py` now
passes: *"intact — counts, key table and vectors all re-derived from the file"*. Its first check is this
defect exactly — *"the key table matches the source corpus keys exactly (no silent subsetting)"* — and it
failed before the rebuild.

**`max_abs_diff 31.0` does not close, and is not a defect.** The packed store is min..max quantized, so its
4-level codes cannot carry the engine's zero trit: the two integer sums are not the same quantity. The store
written from those trits (`.tercode`, 2 bits per trit) is the one that reproduces the engine exactly —
`?engine=tercode&verify=1` reports `identical: true`. The route now states this in `basis` instead of
publishing a bare `DIVERGED`, and the counts that used to be frozen in `search_trits`'s doc comment moved to
the checkpoint that measured them (they were measured against the 180-row store).

Receipts: `cargo test --release` **35 passed / 0 failed** (the three cache tests are new). Live `:8022`
serves `"vectors":2180` / `"serves_the_file_on_disk":true` / `"identical":true`, with `/k3/status`,
`/sram/architectures`, `/parsers/status`, `/infrastructure/status`, `/system` and the `:5183` dashboard all
200. On the drive, the same measurement comes from the guest's own voice
(`backups/ternary_stores_20260930/usbstick_storereload_20260930.log`).

## 7. The launcher read a fixed window of its own config file (2026-09-30)

`read_netconf()` read `/mnt/esp/network.conf` into a 512-byte buffer. The file it was written against is
1,073 bytes -- a header comment, then `ip=` and `gw=` at the END -- so both settings fell outside the
window and the boot printed `NET CONFIG: /mnt/esp/network.conf (511 B)` followed by `NET: eth0 up (mac
52:54:00:12:34:56), no address configured`. The interface came up, the platform started, and the machine
had no address: nothing failed. A value that reads as a measurement (511 B) described a file that is
1,073 B, which is the defect class -- and the platform was unreachable from anywhere but itself.

Fix: the file is read at its own size (`stat`, then read to that length, walking it line by line with
leading whitespace and CRLF tolerated), and the boot log records what will be applied
(`NET CONFIG: ip=... / gw=...`) beside the input it came from. A config reader that sees only the first N
bytes of a file reports a machine with no address, not a truncated read.

## 8. A receipt left to the kernel's writeback schedule is not a receipt (2026-09-30)

The boot log lands on the state image, which is a FILE on the volume's FAT filesystem: this file -> the
loop device -> that file -> the flash. Read from another computer 80 s after the platform answered, the
log showed `Size: 0`. The same reader, later, showed 57,111 B. Nothing was wrong with the writing; the
receipt was simply still in a cache, and a machine that dies unexpectedly never drains it.

Fix: PID 1 flushes the log explicitly (`fsync` on the log file, then on `/mnt/esp/trm-state.ext4`), so a
boot that prints and dies still leaves what it printed. For the reader, the journal is replayed on a COPY
before reading at all: a machine stopped mid-boot leaves it unreplayed and the raw image then refuses to
open (`Inode bitmap checksum does not match bitmap`).

Receipts: `backups/bare_metal_nic_20260930/usbstick_nic_20260930.log` -- `NET CONFIG:
/mnt/esp/network.conf (1073 B of 1073)`, `NET: eth0 up (mac 52:54:00:12:34:56), 10.0.2.15/24, gw
10.0.2.2`, `HTTP 200` on `/k3/status` from outside the guest, and the log read back off the stick.

## 9. The boot waited for a disk, and the volume it needed was not one yet (2026-09-30, first metal boot)

The first boot on physical hardware reached the kernel, loaded all 16 modules (the machine's RTL8125B
came up as `eth0`), and then reported a machine with no platform. The machine's own screen says why: at
1.19 s PID 1 asked `/dev` and saw `nvme0n1` and its eight partitions; the stick this boot came from
appeared at 2.65 s (`scsi 2:0:0:0: Direct-Access ASolid Extreme`, `[sda] 15.7 GB`). The wait meant to
cover exactly this was gated on `has_disk()` -- "is any disk present?" -- and on a machine with an
internal NVMe the answer is yes before the boot begins. So the wait never ran, one scan was the only
look, and it found no `rootfs.squashfs` anywhere: `no volume carrying the platform image`, then
`EXEC FAILED: No such file or directory`, then the honest `NOTHING answered on :8022`.

A wait gated on the wrong question is not a wait. Fix: `find_volume()` asks `/dev` again every 250 ms for
up to 40 s and tries every candidate for the platform image -- the device LIST is what changes, so it is
the thing to re-ask -- with a heartbeat while it waits and the time it took when it succeeds. `has_disk()`
is gone; nothing else used it.

Reproduced before the fix by giving the guest an internal NVMe disk carrying no platform: the decoy is
there at once, the stick arrives at 8.9 s, and the boot failed exactly as the machine did
(`backups/bare_metal_first_boot_20260930/decoy_before.log`, `HTTP 000`). After the fix the same boot reads
`nvme0n1 vfat but carries no rootfs.squashfs; WAIT: ... sda mounted at /mnt/esp (rootfs.squashfs 16461824
B)`, `(found on pass 4, 750 ms in)`, and the platform answers `HTTP 200` from outside the guest
(`decoy_after.log`). The no-decoy path is unchanged and still 200.

**The build that "succeeded" without compiling the change.** The first rebuild after this edit printed the
PREVIOUS binary's hash and packed it: `gcc` failed (GCC 14 rejects the implicit declaration of
`block_devices`, which `find_volume` used before its definition) but the old `platform_init` was still in
the output directory, so the `[ -x ... ]` check passed and the pack shipped the old image as if nothing
had happened. The packer script now removes its output first and checks the compiler's own exit status
(`${PIPESTATUS[0]}`, since the command is piped), and fails the pack instead of shipping a stale binary.

## 10. The boot said the platform was up and never said where to look (2026-09-30)

The machine booted the dashboard -- measured: the supervisor's own probe recorded
`"key":"dashboard","port":5183,"supervisor_status":"ONLINE"`, and PID 1's route probe got
`GET /ui/ -> HTTP/1.0 200 OK`. And none of that was reachable: `eth0: Link is Down` and no address, because
`network.conf` ships with `ip=` inert and this image has no DHCP client. A dashboard nobody can open is a
dashboard that has not booted, as far as the operator at the keyboard is concerned.

Two changes, both about naming the machine rather than asserting it:

- with no `ip=` line the launcher now gives the interface an address of its own -- `169.254.x.y`, derived
  from the MAC (so it is stable across boots) -- and says so in the line that applies it
  (`169.254.53.87/16 (link-local, derived from the MAC: nothing was configured)`). The range means exactly
  "no router answered", the machine at the other end of a cable takes one of its own, so a direct
  connection works with nothing set up. It is not probed for collisions and the line does not claim it was.
- the boot's last lines name the dashboard on every count it has, including the one where it is not
  reachable and why:

      DASHBOARD: http://127.0.0.1:5183/ui/ on this machine, and http://169.254.53.87:5183/ui/ over the
                 network (eth0 is linked)

      DASHBOARD: http://127.0.0.1:5183/ui/ on this machine; eth0 holds 169.254.53.87 but reports NO
                 CARRIER -- nothing is on the other end of its cable, so nothing outside can route to it.
                 The dashboard itself is up: the supervisor probes it every pass

The carrier is read from `/sys/class/net/<dev>/carrier` -- the kernel's own answer, not a driver's log line
-- and re-read at the end of the boot, because a cable's link comes up a moment after the driver binds.

Receipts: `backups/bare_metal_first_boot_20260930/dashboard_over_the_nic_configured.log` -- with the port
forwarded, `GET :15183/ui/ -> HTTP 200`, 566 B of `<!doctype html>` and
`<title>TRM SWARM OS | LCARS INTERFACE</title>` reaching a client outside the guest, plus
`GET :18022/infrastructure/status -> HTTP 200`. `dashboard_address_without_config.log` -- the unconfigured
path: `NET CONFIG: ... (1075 B of 1075)`, `NET: eth0 up (mac 52:54:00:12:34:56)` and
`http://169.254.53.87:5183/ui/ over the network (eth0 is linked)`.

Also closed here: `rtl_nic/rtl8125b-2.fw`, the file the board's NIC asked for and the image did not carry.
The file is pinned by sha256 in `trm_chainload/firmware/README.md` and staged by
`make_rootfs.py --firmware`; the packer gained the flag, not a special case.
