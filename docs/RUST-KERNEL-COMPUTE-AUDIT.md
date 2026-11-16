# Rust kernel: what computes, what infers, what only claims

**Method.** Read the live kernel source (`trm_uefi_kernel/host_council_server/src`, 12 modules,
4,031 lines in `main.rs` alone), traced every `mod` to its call sites, then exercised the
running binary on `:8022` and read the vote records it produced. Numbers below are from that
run, not from documentation.

**Purpose.** Answer one question honestly: *something was computing and inferring — what
exactly, and where does the claim outrun the code?*

---

## 1. Shape of the tree: 43 crates, 2 binaries launched

```
trm_uefi_kernel/                 43 crates, ~40,000 lines of Rust
 └ host_council_server/          27 files, 7,563 lines  <- the live kernel
 └ trm_companion_bridge/         5 files,   740 lines   <- the other live binary
 └ 41 other host_* crates        ~31,000 lines, none launched
```

`START_TRM_PLATFORM.bat` starts exactly two Rust binaries: `host_council_server.exe` and
`trm_companion_bridge.exe`. Everything else in the tree is a library, a test harness, or a
module that only the bare-metal image links.

That is the first real answer: **most of the kernel is not "set up" in any running sense.**
It compiles and is tested (2,697 tests, 13/13 crates rc=0), but only two executables are
ever started.

## 2. What genuinely computes on the live server

### 2a. Trained ternary heads — the strongest real component

`POST /k3/classify` on the live server:

```
{"agent":"Shield","label":"BLOCK","latency_us":99.7,"params":522027,
 "probs":{"analyze":0.0000371,"block":0.9999321,"proceed":0.0000309},"trit":-1}
```

522,027 parameters, 3-way softmax, 99.7 µs, from `trm_uefi_kernel/assets/k3_heads.bin`.
This is real trained tensor math and it is genuinely fast. Its *judgement* is separately
disputed (G14: chance-level on unseen sets), but the arithmetic is not in question.

### 2b. Model inference per agent — this is where the time goes

Read from the vote record of proposal 1948, produced by the live council:

```
votes: 14
llm_trit set: 14 / 14            <- the on-device model ran for every agent
decision paths: llm (ssm dissents) 4
                llm (ssm undecided) 7
                llm (ssm agrees)    3
per-agent latency: 1,135 / 1,536 / 2,040 / 1,851 ms
```

Every one of the 14 agents called the 1.5B model on `:8587` (`local_inference::complete`,
real HTTP, real generation). 14 × ~1.5 s is the 11–38 s that `POST /council/propose` used
to cost synchronously — now moved to a worker thread (G20).

### 2c. The SSM — real recurrence, degenerate parameters, at chance

`council_ssm_pipeline.rs` → `Mamba3StateSpaceScanner` (`src/mamba3_turbovec_ssm.rs`):

| piece | reality |
|---|---|
| D_MODEL / D_STATE | 128 / 16 — state is 16×128 floats |
| input features | `token_vector()` = 4 salted hashes × 32 dims, unit-normalised |
| discretisation | `a_bar = 1 + da + da²/2` — a 2nd-order Taylor of `exp(da)` |
| **input gain B** | `b_bar = delta` — **a constant, not a matrix** |
| **output gain C** | `c_val = 1/4` — **a constant, not a matrix** |
| trained read-out | file `data/ssm_readout.json` — **does not exist** |

So the "Mamba-3 SSM" here is a **leaky integrator driven by hashed tokens with constant
B and C**. It computes, deterministically and cheaply — but with no learned input/output
projections it cannot represent which features matter, which is exactly what the code says
elsewhere: the fitted read-out scored **0.458 leave-one-topic-out against a 0.455
always-majority baseline**, so it was written with `deploy=false` and `readout()` refuses to
load it.

Measured today: the SSM was non-UNKNOWN on **7 of 14** votes and **agreed with the model on
3 of 14 (21%)** — below the 34%-at-chance figure already recorded in the source. Fusion runs
with `gate=false`, so **the SSM never decides; it is recorded and dissented against.**

This is the most important find for the platform's self-understanding: the "Mamba-3 + K3"
council story is half real. K3 heads decide safety classification for real. The SSM is a
deterministic feature-reduction step that currently votes at chance.

### 2d. Kleene ternary engine + K3 SRAM CAM — real, but micro-scale

```
GET /k3/sram/bench
{"entries":20000,"hits":20000,"ns_per_store_plus_lookup":217.6,
 "primitive":"K3Sram store_entry + lookup_entry (ternary CAM)"}
```

`K3Sram` is a fixed-size software CAM: `trits: [i8; 1024]`, `tags: [[u8;16]; 256]`.
2,048 bytes of trit plane, 256 CAM lines, ~218 ns per store+lookup, 20,000/20,000 hits.
Real arithmetic; a *simulation* of an SRAM fabric held in host memory, not a hardware
residency.

### 2e. "5 virtual architectures" — algorithms, not fabric

`sram_pattern_computer.rs` implements ten ordinary functions over byte slices:
`pattern_fpga` (LUT eval), `pattern_cellular_automaton`, `pattern_neural_network`,
`pattern_content_addressable`, `pattern_quantum_simulator`, `pattern_ising_spin_glass`,
`pattern_hopfield_recall`, `pattern_boltzmann_energy`, `pattern_gnn_aggregate`,
`pattern_tcam_match`.

These **do compute** — they are working algorithms. They are not hardware architectures;
they are `&[u8]` functions running in host RAM. Calling them "virtual architectures in the
Infinity Cache" is the mislabel, not the code.

## 3. Where the claim outruns the code

### 3a. "70,606,060 measured agent-tasks/sec" is not reproducible

`/k3/stress117` runs 500,000 iterations of a six-primitive loop and reports both figures:

```
"claim":    {"tasks":500000,"total_ms":391.21,"title_ops_per_sec":1280000.0,"pass_rate_pct":100.0}
"measured": {"tasks":500000,"total_ms":437.964,"ops_per_sec":1141647.0,"per_op_ns":875.93}
"gpu_vs_cpu":"CPU (microkernel routes are host-executed; no GPU endpoint exists)"
"primitives_per_task":["k3_ternary_and/or/not","k3_quick_reduction","k3_sram_store+lookup",
                       "mamba3_femto_step","turbovec_cosine"]
```

**1,141,647 ops/s measured** — the number the endpoint itself reports, in its own units,
labelled `ops`, with an explicit statement that **no GPU endpoint exists**.

`docs/NOBEL-PAPER-2026-01-REV.md` presents **70,606,060 agent-tasks/sec** as the measured
figure. That is **62× above** what the live endpoint reports for the same 500,000-task run,
and it renames six scalar primitives per iteration as an "agent task". One iteration of that
loop is ~876 ns of scalar CPU work; an agent task is 1.1–2.0 s of model inference (2a/2b).
The two differ by six orders of magnitude and are not the same quantity.

### 3b. The GPU route exists; the GPU *compute* module is a stub  [CORRECTED 2026-11-10]

My first pass repeated a claim from the product instead of checking it. `/k3/stress117`
returns `"gpu_vs_cpu": "CPU ... no GPU endpoint exists"` — **that sentence is wrong.** A
`/gpu` route exists (`main.rs:2669` → `bench::gpu_state()`), and it is the honest one:

```
GET /gpu  (before the fix)
{"note":"adapter identity is a real WMI query; VRAM utilisation requires a vendor API",
 "queried":false, "vram_utilisation_available":false}
```

`queried:false` was not "no GPU" — it was **`wmic` missing**. Windows 11 build 26200 ships
without `wmic.exe`, so `Command::new("wmic")` failed and the error was folded into
`queried:false`. Fixed by falling back to `Get-CimInstance` and recording which backend
answered:

```
GET /gpu  (after the fix)
{"adapters":[{"name":"spacedesk Graphics Adapter", ...},
             {"adapter_ram_bytes":4293918720,"adapter_ram_is_overflowed_u32":true,
              "driver_version":"32.0.21045.16010","name":"AMD Radeon RX 6700 XT"},
             {"name":"vs spacedesk Graphics Adapter", ...}],
 "backend":"powershell-Get-CimInstance","queried":true}
```

So the platform can now name its GPU from a live host query, with `AdapterRAM`'s 32-bit
overflow flagged rather than printed as if it were 12GB.

What remains true is the part I was actually after: **the GPU compute module is a stub.**

```
src/gpu_infinity_computer.rs                    5 pub fns, 5 x "0 // TODO: complete 1:1 math"
host_fanout/src/gpu_infinity_computer.rs        identical copy
host_tranche/src/gpu_infinity_computer.rs       identical copy
```

Every body is `0 // TODO: complete 1:1 math` under a commented-out Python dictionary — the
docstrings survived the port, the maths did not. None of these three files is linked into
`host_council_server`, so no running code path can reach them. `S_BITCMP_B32` appears in
them only as a string inside those comments.

Accurate statement: **a GPU diagnostic endpoint exists and works; a GPU compute fabric does
not exist in executable form.**

Everything performance-relevant on this platform executes on the host CPU, except LLM
inference which executes on the GPU through **Vulkan llama.cpp** (`ggml-vulkan.dll`) — a
path that is real, measured, and unrelated to the fabricated instruction.

## 4. So what was computing and inferring?

| Layer | Status | Evidence |
|---|---|---|
| 1.5B model inference, per agent | **REAL**, 1.1–2.0 s/agent | `llm_trit` present 14/14 |
| Trained K3 heads (522k params) | **REAL**, 99.7 µs | live `/k3/classify` |
| Kleene ternary + K3 SRAM CAM | **REAL arithmetic**, micro-scale | 217 ns store+lookup, 1024 trits |
| SSM recurrence | **REAL math, at chance** | 3/14 agreement, B/C constant, read-out refused |
| Ten "pattern architectures" | **REAL algorithms**, CPU slices | `sram_pattern_computer.rs` |
| Infinity Cache as compute fabric | **NO executing path** | endpoint self-reports CPU-only |
| "70.6M agent-tasks/s" | **NOT reproducible** | live endpoint: 1.14M ops/s |
| 41 of 43 crates | **built, tested, never launched** | launcher starts 2 binaries |

The platform genuinely infers and genuinely computes. What it does not do is any of the
silicon-level tricks the journal attributes to it — and the honest, already-measured
capabilities (sub-millisecond ternary classification, Vulkan inference, a real council with
recorded per-agent provenance) are more defensible than the fabric claims they sit next to.

## 5. Corrections this audit implies

1. `NOBEL-PAPER-2026-01-REV.md`: the **70.6M** figure must be replaced by the endpoint's own
   **1,141,647 ops/s** (or re-measured), and the units renamed from "agent-tasks" to
   "composite primitive iterations".
2. `JOURNAL.md`: `SRAM Pattern Computer: 5 Virtual Architectures` (PUBLISHED) describes
   functions over `&[u8]`; the words "in cache" and "virtual architecture" need to go, or the
   claim needs a hardware demonstration.
3. The SSM, not the cache, is the real opportunity: **B and C are constants and no read-out
   is deployed.** A fitted read-out with learned B/C is a tractable, measurable project whose
   success criterion already exists (beat 0.455 majority on leave-one-topic-out).
4. G14 (safety gate) stays open and is unaffected by any of this: the K3 heads are real and
   still at chance on unseen danger sets.

---

**Status:** recorded, nothing deleted. Every number above is reproducible with the commands
named in the section that cites it.
