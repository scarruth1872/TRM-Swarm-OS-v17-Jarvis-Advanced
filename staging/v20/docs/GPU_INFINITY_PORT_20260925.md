# gpu_infinity_computer: generated strip, bit-exact parity, declared-vs-measured fixed

**Block 144 companion.** The campaign log lists this module at 18% ported with 10 functions remaining and a
target of "strip DLL bit-exact". This round took it to that target for the whole pure-integer surface, and
fixed the same two defect classes the SRAM round fixed — because this module carried both.

## 1. The strip carried a 3-branch demonstrator, not the module's architectures

`strip_gpu_infinity.rs` chose a byte from a `match` on the index:

```rust
let byte_val = match idx { 0 => 0xAA, 1 => 0x55, 2 => 0x01, _ => 0xFF };
```

so architectures 3..N hashed an identical line — four declared architectures produced **3 distinct hashes**,
and a "reconfiguration" could not carry which architecture it was. The oracle knows the architectures and
their names (`ARCHITECTURES_GPU`, each factory's `name`), so the strip is now **generated from that list**
(`scratch/gen_gpu_strip.py`) and hashes each line from the architecture's own name:

```
GPU FPGA-Lite · GPU SRAM Neural Network · GPU Cellular Automaton · GPU K3 Ternary Engine
```

Because the names come from the oracle at generation time they cannot drift, and the Python side can compute
the identical FNV-1a to check the result. The strip also links the kernel's own port by path
(`#[path = "../../trm_uefi_kernel/src/gpu_infinity_computer.rs"]`), so the pattern math is the kernel's, and
the ABI the module already binds (`strip_batch_gpu_infinity_reconfig`) is preserved — a negative index is now
an honest `-1` rather than a fourth architecture.

## 2. The oracle declared what it should have measured

| defect | before | now |
|---|---|---|
| speedup | `"speedup": "7.5x over pure Python struct pack & sha256"` — a literal, with `rc` ignored and `out_h` never read | checks `rc`, reads `out_h` back, requires distinct hashes == architectures, times both paths on the same work → **`measured_speedup`: 13.58x on one run, 11.74x on another — it varies, so report the range** |
| GPU spec | `GPU_SPECS` describes an RX 6700 XT (96MB Infinity Cache, 2.1 TB/s) with nothing comparing it to the machine | `_probe_gpu()` probes `win32_VideoController` and reports `described_gpu` / `installed_gpu` / `target_present` / `basis`; `summary()` carries it |
| architecture count | `compare_fabrics()` hardcoded 5 GPU while the dict beside it held 4 | derived `len(ARCHITECTURES_GPU)` → **4**, CPU's count labelled with its owner |

On this host the probe returns `installed_gpu: [NVIDIA GeForce RTX 3070 Laptop GPU, AMD Radeon(TM) Graphics]`
with `target_present: false` — so every 96MB/805M-cell/2.1 TB/s figure in this module describes the design
target, not the machine, and now says so.

**Two things are worth stating plainly.** The declared literal *understated* the real ratio (7.5x declared vs
11.7–13.6x measured across two runs) — a declared number is neither conservative nor aggressive, it is
unverifiable in both directions, which is why it is the defect class rather than merely a wrong value. And the
measured ratio is not a constant: two runs of the same comparison on the same host gave 13.58x and 11.74x, so
a single figure quoted as *the* speedup would be a new declaration replacing the old one. The range is the
honest form.

## 3. The ruler

`scratch/verify_gpu_parity.py` → `swarm_v2_artifacts/gpu_strip_parity.json`, **18 checks, 0 failed**:

| group | checks |
|---|---|
| architecture identity | strip count == oracle count (4); names match the oracle in order; per-architecture hashes equal the Python rule; **4 distinct of 4** (the old rule gave 3) |
| pattern parity | `fpga` at 1/8/64/96/1024 MB, `neural(64)`, `cellular(96)`, `k3` — Rust `==` Python on every field, including the f64 update rate (2625.0) and throughput (845571686.4) |
| ABI | 400-index batch: `rc 0`, all 4 hashes present; negative index → `-1` |
| module defects | summary count derived (4); host probe present beside the described GPU; measured speedup is a number and the literal is gone; `compare_fabrics()` GPU == dict with a basis |

## Honest bounds

- Ported and gated: **the four pure-integer pattern generators and the cache-line hash**. Not ported, and
  said so in the kernel module's own header: `load_architecture`/`benchmark_all` (they drive a real GPU) and
  `summary` reporting.
- The strip is on the **DLL** path, not behind a `host_council_server` route — unlike `/sram`. Wiring it as a
  route is the next rung for this module.
- `compare_fabrics()`'s CPU figure is labelled, not re-derived: the CPU set is owned by
  `sram_pattern_computer` and is not imported here.
- The measured 11.7–13.6x is a ratio of two Python-level timings on one host (native batch vs
  `struct.pack`+`sha256` per line), and it varies between runs; it is a measurement of that comparison, not a
  general GPU speedup and not a constant.

## Ruler hygiene note

Immediately after `os.replace` swaps a DLL, a fresh process can fail `GetProcAddress` on the new file
(`function not found`) even though the swap succeeded. Re-run before believing it — the file was correct and
the symbol resolved on the next process. Same family as the earlier `WinError 32` on a loaded strip.
