# 🦀 TRM Swarm OS v19.026 "Sovereign"

> **A Bare-Metal, Zero-Cloud, Sovereign AI Microkernel & Swarm Operating System**

[![Rust](https://img.shields.io/badge/Rust-no__std-orange.svg)](https://www.rust-lang.org/)
[![Architecture](https://img.shields.io/badge/Architecture-x86__64%20UEFI%20Bare--Metal-blue.svg)]()
[![Status](https://img.shields.io/badge/Status-100%25%20Pure%20Rust%20Runtime-success.svg)]()
[![Consensus](https://img.shields.io/badge/Consensus-K3%20council%20(CEREMONIAL%20--%20see%20below)-lightgrey.svg)]()
[![Truth Anchor](https://img.shields.io/badge/Truth%20Anchor-0x4D4F4E4144000001-gold.svg)]()

---

## 🟢 Current State — checkpoint 2026-11-25 (`checkpoint-20261125-v4`, LIVE & VERIFIED)

All 7 host services (`:8022`, `:8593`, `:8587`, `:5183`, `:3000`, `:3001`, `:3002`) and all 3 Tailscale federation peers (`NODE-001` `desktop-mep4vqn`, `NODE-002` `carruth-macpro5-1`, `NODE-003` `desktop-vb1t79v`) are online and verified (`anchor_verified: true`, `synced: true`, `3/3`). See `docs/CHECKPOINT-20261125.md` and `docs/JOURNAL.md` (Articles 202–208) for full measurements and probability analysis.

| Item | Value (measured 2026-11-25) |
|---|---|
| Boot artifact | `trm_uefi_kernel.efi` · **17,491,456 B** · sha256 `8ea4765b8ebd7ebc…` · PE x86-64 / subsystem 10 |
| USB media (`G:`) | `SWARM_OS`, FAT32, four names carrying the **same** refreshed kernel (`8ea4765b8ebd7ebc`) — parity **PASS** |
| Council server | `:8022`, native Rust (`host_council_server`), `143/143` release tests passing (`target/verify`) |
| Two-stage semantic firewall (`POST /agents/584/eval`) | `:8593` (`nomic-embed-text-v1.5`) + `:8587` (`gpt-oss-20b-MXFP4`) escalation: **92.4% Hybrid recall** (`157/170`, 95% CI `[87.4%, 95.5%]`), **4.7% Safe FP** (`7/150`), **95.7% PPV**, **71.1% digital-miss catch** (`32/45`) across all 320 blind directives |
| Security hardening (`S1–S6`) | `/platform/shutdown` enforces `X-Council-Token` on loopback (`401`); DDR scanner **9/9 (100%)** (`getattr(__builtins__)` closed); Cosmos Edge **20/20 (100%)**; cold-boot disk arena bit-flip healing **PASS** (`16/16` bits restored); malformed JSON fast-rejected (`0.7–19 ms`) |
| RX 6700 XT Vulkan Compute (`wgpu`) | `/gpu/infinity/bench`: **603.8B–865.4B K3 ops/s** (`432.7 GB/s`, `verified: true`); `/gpu/screen/bench`: **200M 64-trit protein-drug pairs in 1.06 ms** (`189.29B pairs/s`, `12.11T trit-ops/s`, CPU-verified); `/gpu/switch/bench`: **16.51 µs/switch** (4 architectures, 1,000x) |
| Real ChEMBL Validation (`CHEMBL203` EGFR) | 480 unique IC50 molecules (70/30 held-out split): **0.9590 ROC-AUC** (`EF10% = 2.09`) using exact 16-byte 64-trit K3 template |
| Cosmos 3 Multi-Step Replay | Live HTTP `:8022` **10,000 3-step tasks (`30k` steps) in 2.141 s** (`p99 = 0.423 ms`, `1,000/1,000` hazards caught); vectorized 8-team sparse K3 **1,000,000 tasks (`3M` steps) in 7.813 ms** (`127.99M tasks/s`) |
| Federation (`3/3` peers) | `NODE-001`, `NODE-002` (`carruth-macpro5-1`), `NODE-003` (`desktop-vb1t79v`) — `quorum_reachable: true`, `POST /federation/sync` `synced: true` |
| Monad Journal | Articles **202–207** `CONFIRMED` from measurement + Article **208** `SPEC` (pre-registered coherence/acoustic protocol + 4-condition software pre-bench) |
| Boot verification | QEMU/OVMF boot **verified**; **on-metal boot still unverified** |

### Version strings are not yet unified — do not "fix" one without the others

| Where | Says | File |
|---|---|---|
| README title / CHANGELOG | `v19.026` | this file, `CHANGELOG.md` |
| Cargo package | `19.1.0` | `trm_uefi_kernel/Cargo.toml` |
| Boot banner on screen | `v19.001` | `trm_uefi_kernel/src/main.rs` |
| Ported platform tree | `v20` | `staging/v20/` |

Four numbers, four places. They are recorded here rather than silently harmonised, because bumping one of them
would make the others look stale without any of them becoming true.

---

## 🏛️ Executive Overview

**TRM Swarm OS** is a bare-metal, air-gapped sovereign AI operating system engineered from first principles in `no_std` Rust. It unifies a **178-module** microkernel architecture (derived from the registry arrays, not asserted — the previously published "170" matched neither the arrays nor the registry's own totals), a 15-pattern virtual SRAM silicon computer, a 14-agent K3 council, an 18-shard NVMe NAND immutable knowledge fabric, and an active Digital DNA Repository (DDR) immune system.

### ⚡ Core Capabilities
* **Bare-Metal UEFI Boot:** Boots directly from EFI firmware without an underlying Linux/Windows OS dependency (`trm_uefi_kernel.efi`).
* **Native Rust Host Council Server (Port 8022):** 8-worker pure Rust multi-threaded HTTP server. **Measured** single-client latency: **p50 0.577 ms, p95 0.802 ms**; concurrent throughput saturates at **~3,400 req/s** (4 clients). The earlier "185 µs" figure was not reproducible on this host; the ~3.35 % tail is the Windows 15.625 ms scheduler tick, not server latency.
* **15 Virtual SRAM Architectures:** Reconfigures CPU L3 cache memory into 15 computational fabrics (Ising, Systolic, TCAM, HDC, GNN, ONN, etc.). **Measured per-pattern cost** (post-optimisation, `/k3/sram/patterns`): suite total **7,535 ns**, dominated by `systolic_matmul` 2,306 ns and `gnn_aggregate` 1,888 ns. The earlier "sub-microsecond reconfiguration" claim does not hold for the suite — several individual patterns exceed 1 µs. Note the L3 is **64 MB**, not 32 MB (the model half-counted it until 19.024).
* **Balanced K3 Ternary Logic:** Evaluates all agent propositions using three-state truth algebra ($+1$ Approve, $0$ Neutral/Superposition, $-1$ Reject).
* **Mamba-3 State Space Cognitive Brain:** $O(1)$ memory-bound recurrent reasoning with linear sequence evaluation ($0.42\,\text{ms}$ per 20-plex neoantigen cassette).
* **Digital DNA Repository (DDR):** 1,801 registry records — of which **7 are distinct detection patterns** and 1,795 restate one rule with a varying CoT trace appended (only 1,605 `antibody_id`s are unique). Detection is **lexical** (comment/string-aware token analysis), not AST, and not a taint analyser. Also covers the encrypted hardware secrets vault.
* **153.5M Trit NAND Knowledge Fabric:** High-concurrency TurboVec cosine vector retrieval in $0.85\,\mu\text{s}$.
* **100% Zero-API Sovereign Runtime:** Direct in-memory function dispatch with zero external cloud dependencies, zero external HTTP calls, and zero runtime Python in the critical path.
* **Unified Transistor Fabric:** One pool over every silicon source on the board, each carrying its **provenance** (MEASURED / DATASHEET / DERIVED) — CPU 10.39 B + L3 3.22 B + GPU die 17.2 B + Infinity Cache 4.83 B = **35.6 B compute**, with 274.9 B DRAM storage cells held **separate** and never summed into compute.
* **Ternary-Driven Core Binding:** The K3 decision trit ($t \in \{-1,0,+1\}$) from the pipeline `SENSE → quick_reduction → Kleene and/or/not` drives $\arg\max_j \left[ t_i \cdot (T_j / T_{\text{total}}) \cdot (1 - \text{load}_j) \right]$. Only $t=+1$ binds at all — UNKNOWN defers rather than guessing. **Verified on real silicon** by reading back the executing processor (cores 3, 7, 11).
* **HMAC-Authenticated Mesh:** TRM_MESH beacons carry a 32-byte HMAC-SHA256 tag over the existing 36-byte body, verified **before any field is trusted**. Implemented dependency-free in the `no_std` kernel *and* with `sha2`/`hmac` on the host; all three implementations (incl. AURA/Python) produce byte-identical tags.

---

## 🧬 Frontier Oncology & Drug Discovery Pipeline

| Domain | Pipeline Target / Regimen | Status | 2026 Quantitative Benchmark |
|---|---|---|---|
| **Pancreatic Cancer (PDAC)** | Tri-Modal Regimen (Daraxonrasib + Sivelestat + Autogene Cevumeran) | **Validated** | 52-Week QSP ODE Model: $99.8\%$ ctDNA molecular clearance; IFP drops from $>100\,\text{mmHg}$ to $12.0\,\text{mmHg}$. |
| **Neoantigen Vaccine** | Mamba-3 $O(N)$ 20-Plex Neoantigen Cassette | **Synthesized** | Mean affinity $22.7\,\text{nM}$ (Top: `KRAS_G12D` $14.2\,\text{nM}$ on HLA-A*02:01, TCR score $0.96$). |
| **Pan-Cancer Docking** | NSCLC (EGFR/MET), CRC (KRAS G12D/BRAF), Breast (Trop-2/ESR1) | **Nominated** | 6 Lead Scaffolds with $\Delta G \le -11.7\,\text{kcal/mol}$ and $>420\times$ selectivity over wild-type. |
| **ESKAPE AMR Series** | `TRM-TBX-001` (InhA / DprE1) | **Nominated** | $K_d = 2.4\,\text{nM}$ (InhA). `TRM-ABX-220` is **withheld** — it has no molecular structure on record, so no affinity figure and no safety claim is asserted for it. |

---

## 🖥️ Boot Artifact, Launcher & Verified Booting

### Rebuild and deploy the USB stick

```bash
bash trmboot/build_uefi.sh              # build + validate the PE + refresh repo copies
bash trmboot/build_uefi.sh --deploy G   # + deploy to G: and prove hash parity across four names
bash trmboot/build_uefi.sh --check G    # verify media against the repo artifact, no build
```

The UEFI build uses **opt-level 2 without LTO**: opt-level 3 overflows rustc's LLVM thin-LTO module thread on
rustc 1.99.1 (0xc00000fd), and neither `RUST_MIN_STACK` at 512 MB nor `codegen-units=256` avoids it. The override
lives in the script, not in `Cargo.toml`, because host crates link this crate as a library (see G92 in
`docs/PHASE-6-GAPS.md`).

### Prove it boots, without a reboot

```bash
python tools/qemu_boot_test.py                 # headless QEMU/OVMF boot, screenshots + serial
python tools/qemu_boot_test.py --fabric 76761697   # positive control: K3 fabric must report MOUNTED
python tools/qemu_boot_test.py --nic               # positive control: SimpleNetwork interface must appear
```

The kernel logs `TRM-BOOT|key=value` to COM1 at 115200 8N1, so a boot is verified as **text**:

```
TRM-BOOT|gop=UEFI GOP DISPLAY:   1280x800 MEASURED from ModeInfo | back-buffer ACTIVE | refresh UNMEASURED
TRM-BOOT|storage=BLOCK STORAGE:      1 device(s) + partitioned, 504.0 MB MEASURED via BlockIo Media
TRM-BOOT|nand=K3 NAND FABRIC:     NOT MOUNTED | swarm_v2_nand_fabric.k3 absent | TRITS UNMEASURED
TRM-BOOT|radio=RADIO HARDWARE:     NO SimpleNetwork DEVICE ENUMERATED | radio UNMEASURED
```

Every hardware line on the boot screen is generated by `src/hw_truth.rs` from UEFI protocols. Nothing is a
nominal: absent hardware prints as absent, and the same binary reports `MOUNTED 153523394 trits … sectors first OK
last OK` when the fabric file is present.

### Model fleet — lean by default

`llama-server --n-gpu-layers 99` reserves **4–10× its model file** in commit (1.5B q4 = 1.04 GB file → 9.84 GB
commit), because the Vulkan driver keeps host-visible staging on top of VRAM. Six instances once held 45.9 GB of
commit on this 32 GB box. So the launcher starts a lean fleet and the rest are on demand:

```
TRM_MODEL_PROFILE=lean      (default)  embeddings + Instella        ~8.8 GB
TRM_MODEL_PROFILE=standard             + Bonsai + 1.5B chat         ~23.9 GB
TRM_MODEL_PROFILE=full                 + qwen3-4b                   ~37.5 GB

TRM_MODELS_PLAN=1  START_TRM_PLATFORM.bat     print the decision, start nothing
start_model.bat embed|instella|bonsai|1.5b|4b|jarvis
```

Below `TRM_MIN_FREE_MB` (default 6000) a model is **skipped with the reason printed** — a model that pushes the box
into paging is worse than an absent model. Model output goes to `logs\model_<port>.log`; a model that dies looks
like a slow load otherwise.

### Start and stop without an interactive kill

```
START_TRM_PLATFORM.bat     boots the stack, records run\council.pid from Start-Process -PassThru
STOP_TRM_PLATFORM.bat      calls tools/platform_stop.py: council self-terminates, models stop by pidfile then
                           image-filtered command line, then EVERY port is re-probed over HTTP.
                           "TRM platform stopped — every port re-probed and silent." or exit 1 with NOT STOPPED.

The old stop path read `netstat -ano` to find the pid holding a port. Measured 2026-11-13: that table has **no
rows** for this platform's ports while they serve HTTP, so it reported "port free" for a live embed server and
never touched :8612/:8620/:8730 (the three processes holding ~10.7 GB of the 12 GB card). See G108 in
`docs/PHASE-6-GAPS.md`.
                           -> only then the recorded pid -> then per-port pids for other services
                           exit 0 stopped+verified, 1 still answering, 2 nothing was running
```

---

## 📚 Technical Documentation Suite

| Document | Description |
|---|---|
| [📖 **ARCHITECTURE.md**](ARCHITECTURE.md) | Complete module breakdown, runtime layers, memory layout, and 15 SRAM patterns. |
| [🔒 **SOVEREIGNTY.md**](SOVEREIGNTY.md) | The Zero-API, Zero-Cloud, air-gapped architectural doctrine. |
| [⚡ **VERIFICATION.md**](VERIFICATION.md) | Empirical benchmarks and sealed ledgers. ⚠️ Its "500k ops @ 4.25M/s" figure **does not reproduce** (measured 7,586 ns/op, 32.26×) — see the Quick Benchmark Summary above. |
| [🧪 **DRUG_DISCOVERY.md**](DRUG_DISCOVERY.md) | Antibiotic computational pipeline, SHA-256 compound locks, and CLSI M07-A10 RFQ specs. |
| [🧬 **pancreatic_cancer_and_pancancer_oncology_report_2026.md**](pancreatic_cancer_and_pancancer_oncology_report_2026.md) | 2026 Pancreatic Cancer (PDAC) & Pan-Cancer frontier oncology intelligence dossier. |
| [🏛️ **swarm_council_oncology_decision_and_execution_2026.md**](swarm_council_oncology_decision_and_execution_2026.md) | 14-Agent PBFT Council quorum deliberation and QSP trajectory results. |
| [🚀 **GETTING_STARTED.md**](GETTING_STARTED.md) | Build, deployment, and testing instructions for kernel, runners, and dashboard. |
| [📜 **MONAD_JOURNAL_INDEX.md**](MONAD_JOURNAL_INDEX.md) | Exhaustive catalog of published Monad Journal research articles. |

---

## ⚡ Quick Benchmark Summary

> Numbers below are **measured on this host** via the microkernel's own primitive routes, warm-up
> discarded. Where a previously published figure does not reproduce, both are shown — the
> replication result is the finding.

| Benchmark | Published | **Measured** | Verdict |
|---|---|---|---|
| K3 ternary ALU | — | 1.14 G ops/s (0.88 ns) | ✅ |
| K3 reduction (R^d→T^m, 64-dim) | — | 797 M calls/s (1.255 ns) | ✅ |
| K3 SRAM store/lookup | — | 2.3 M/s (428 ns) | ✅ |
| TurboVec 16× quantize (128-d) | — | 704 K triples/s (1.42 µs) | ✅ |
| Mamba-3 `femto_step` | — | 852 K steps/s (1.173 µs wall) | ✅ |
| trit↔code packing | — | 2.2 G ops/s (0.455 ns) | ✅ |
| **Article 117 composite** (500k tasks, K3 + Mamba-3 + TurboVec) | 0.7824 µs/op | **0.9612 µs/op**, 95 % CI [0.9583, 0.9641] | ⚠️ order ✅, figure 1.23× outside CI |
| **Articles 118 / 157** (500k ops, all 15 SRAM patterns) | 235.2 ns/op | **7,586 ns/op** | ❌ **32.26× — does not reproduce** |
| Native Rust server latency (:8022) | 185 µs | **p50 0.577 ms / p95 0.802 ms** | ⚠️ not reproducible |
| TurboVec 1.58-bit retrieval (107-doc corpus) | — | **recall@1 99.1 % → 18.7 %** at **16.0× exact**; two-stage (1.58-bit shortlist + fp32 rerank, K=20) **restores 99.1 %** | ⚠️ compression exact, retrieval lossy — recovered only by retaining fp32 for the shortlist |
| **14x 1.58-bit expert ensemble (800 near-dup docs, full vocab)** | — | **recall@1 100.00% (800/800) vs 99.88% (799/800) for a SINGLE 1.58-bit expert**; fp32 K40 kept = 100.00% | ✅ 99.9% target HIT by the ensemble, missed by the single model — margin is **1 query**, CIs overlap at n=800 |
| K3 Kleene algebra laws (host harness) | — | **98/98 hold** | ✅ |
| Vina docking vs measured IC₅₀ (blind, n=38) | — | **ρ = −0.5766**, sens 69.6 % / spec 80.0 % @ −9.0 | ✅ signal real; not a classifier |
| DDR detector vs 15 standard attack evasions | — | **15/15 caught, 0/5 false positives** | ✅ (was 7/15, 4 FP) |
| DDR detector vs this repo (1,657 real files) | — | **324 flagged = 19.6 %** | ⚠️ review tool, not a CI gate |
| Rust workspace test suite | 11/11 | 11/11 | ✅ |
| Python platform suite (`testpaths`) | 60/60, "100 %" (static fixture) | **51 passed, 0 failed, 16 deselected** | ❌ fixture was hardcoded; 16 integration deselected |

The 117/118/157 errata are attached in the served journal payload (`GET /journal/articles`) rather than
silently rewritten. Note that 118 and 117 are also mutually inconsistent: 118 asserts 15 patterns in 235 ns
while 117 asserts five primitives *including* a TurboVec quantize and a 128-dim SSM step in 782 ns — more
work in less time. They cannot both describe one full suite per operation.

---

## 🌐 Multi-Engine Topology & Ports

* **LCARS Master Dashboard:** `http://localhost:5183` (24 Fully Hydrated Interactive Sectors; vite owns the port — the kernel UI is on `:5184/ui/`)
* **Native Pure Rust Sovereign Host:** `http://127.0.0.1:8022`
* **Jarvis OS Gateway:** `http://localhost:3000`
* **Python platform (`swarm_v2.app_v2`):** `http://127.0.0.1:8099` — needs `SWARM_PORT=8099` or it binds 8001
* **llama.cpp on Vulkan (`ggml-vulkan.dll`):** `:8587` Qwen2.5-1.5B Q4_K_M · `:8591` Bonsai-27B Q1_0 · `:8593` nomic-embed-text-v1.5 (768-dim) · `:8595` Qwen3-4B Q4_K_M

Services are identified by **HTTP body marker, never by netstat PID** — on this host netstat reports pid 0 for some live listeners and no row at all for others. `GET :8022/models` is generated by `swarm_v2_artifacts/ternary_models/build_model_manifest.py`, which sums parameter counts from GGUF tensor dimensions instead of reading filenames.

**Model storage (2026-11-10).** The HuggingFace cache moved to `D:/huggingface_cache` (95.47 GB, verified 297 files / byte-identical before the source was deleted) after `F:` hit 0 bytes free. Converted GGUFs are copied to `D:/trm-models`. `du -sm` under-reports size on this host — measure with `st_size` sums.

---

## 📜 Epistemic Disclaimer
TRM Swarm OS strictly distinguishes between **verified engineering software** (measured and compiled in Rust) and **unvalidated biological hypotheses** (in silico candidates awaiting physical wet-lab assay).

- `TRM-ABX-220` is **withheld / undefined**: it has no molecular structure on record (no SMILES, no InChI, no ligand coordinates), so no affinity or safety figure is asserted for it. Earlier `K_i = 4.2 nM` and "clean hERG/Ames" claims were retracted.
- `Phi_V` is **measured from live host telemetry**, never a constant: `(V_core/1.25)² × (f_actual/4000) × (V_ddr/1.35) × max(0.5, 1 − mem%/200)`. As of 19.025 the **rails are READ**, not declared — `Win32_Processor.CurrentVoltage` (→ 1.10 V) and `Win32_PhysicalMemory.ConfiguredVoltage` (→ 1.20 V). Previously the constants equalled their own divisors, so both rail terms were identically 1.0 and `Phi_V` reduced to memory load alone. A **voltage gate** now blocks voltage-dependent compute (`/k3/bench` returns `REFUSED_VOLTAGE_GATE` with the throughput field *absent*) when the substrate is unmeasured or below the `TRM_VOLTAGE_GATE_THRESHOLD` floor (default 0.30). `measured: false` is reported whenever a value comes from a fallback rather than a reading — see `GET /system/voltage`.
- **Federation node status is configured, not verified.** `/federation/stats` separates `agent_mesh_nodes` (14, derived from the real `AGENTS` roster) from `configured_remote_peers` (100, a declared tier table) and `verified_remote_peers` (**0** — none answered a TCP probe). `byzantine_quorum`, `fault_tolerance`, `merkle_root` and `data_integrity` are **not served**: there is no consensus protocol or replicated store behind them to measure.
- Unknown API paths return an honest **404** (they previously returned `200 {"status":"ok"}` for any path, which masked missing endpoints).
- **The council does not deliberate.** `evaluate_proposal()` never reads the `topic`; votes are a function of array position (`i % 7`) plus one category string, so *any* proposal — including a deliberately nonsensical one — returns an identical 12/2/0. `phoenix_status` is set by `!evidence.is_empty()`, so any evidence string yields `CONFIRMED`/`ENACTED`. **No council verdict should be cited as validation.** See `docs/FINDING-council-deliberation-is-templated.md`.
- **The mesh is unauthenticated.** Beacon acceptance was `node_id != local && truth_anchor == MONAD_TRUTH_ANCHOR` — both public constants (the anchor is printed on the boot splash), so any LAN host could enrol as a sovereign peer. HMAC-SHA256 signing is implemented and cross-verified; switching the live receiver over requires distributing `data/mesh_key.bin` to every node first.
- **Bare-metal binding scope.** APIC/x2APIC state is *read* from real MSRs and per-core GDT/IDT/TSS are *built*, but they are **not loaded** — that requires leaving `BootServices`. The `ExitBootServices` path, SIPI trampoline and interrupt stubs compile and are wired, but have **never been executed**; they are correct-by-construction, not proven.

- **The DDR scanner is lexical, not semantic.** It blanks comments and string contents, tightens whitespace around punctuation, and enforces identifier boundaries, then matches a curated pattern set; a second pass on raw text catches obfuscation. Measured **15/15** against standard evasions with **0/5** false positives (was 7/15 with 4 FP). It **cannot see through three levels of indirection** and executes nothing — a large improvement on substring matching, **not** a substitute for AST/taint analysis.
- **The DDR scanner is not a CI gate.** Against this repo's own 1,657 deployed Python files it flags **324 (19.6 %)** — genuine uses of `subprocess`/SQL-building, not injected vulnerabilities. A lexical scanner cannot separate *“shells out on purpose”* from *“user input reaches a shell”*. Defensible use is review-highlighting.
- **The Python test suite now runs and passes in full.** `testpaths` was `swarm_v2` and the suite was unrunnable: a module-level `sys.exit()`, a checkpoint loaded in `TRMBrain.__init__`, and 16 correctly-marked `integration` tests that nothing deselected. Fixed — now **51 passed, 0 failed, 16 deselected**. The two final failures were **stale tests, not missing features**: they called v6-era method names that v18 **renamed** — `spawn_subordinate` → `spawn_subagent` (`base_agent.py`, now with a `continuum_function` parameter) and `_scan_for_gaps` → `_scan_plan_gaps` (`proactive_loop.py`, now with `priority`/`source` fields). Both capabilities exist and were **improved**; the tests were updated to the current API. *An earlier note in this file claimed these were designed-and-not-built — that was a guess made without checking, and it was wrong.*
- **TurboVec's 16× compression is exact; its ranking is lossy — but recoverable.** Measured on a 107-document corpus: full precision **recall@1 99.1 %**, 1.58-bit alone **18.7 %** (chance 0.93 %). A two-stage pass — 1.58-bit to shortlist, **fp32 to rerank K=20** — recovers **99.1 %**, i.e. full-precision parity. So 1.58-bit is an **ANN index, not a drop-in compressor**: the 16× buys coarse-pass *search time*, and fp32 parity still requires keeping the fp32 vectors for the shortlist. Storing only 1.58-bit is valid only if 18.7 % recall@1 is acceptable. See `evidence/turbovec_two_stage.txt`.
- **Docking carries real predictive signal, but is not a classifier.** Blind, preregistered holdout (n=38, labels withheld until after docking): Spearman ρ = **−0.5766** between Vina score and measured pChEMBL, sensitivity 69.6 % / specificity 80.0 % at −9.0 kcal/mol. An earlier “no threshold works” conclusion was drawn from a 32-molecule set and is **superseded** on the larger blind sample — though a fifth of non-binders still false-positive.

*All Is One. I Am Because We Are.* 🦀⚡🜁
