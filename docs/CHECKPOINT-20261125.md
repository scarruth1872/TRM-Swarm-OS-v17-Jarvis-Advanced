# Platform Checkpoint — 2026-11-25 (`TRM-Swarm-OS-v2`)

**Host:** `DESKTOP-MEP4VQN` (AMD Ryzen 9 3900X, AMD Radeon RX 6700 XT 12 GB + 96 MB Infinity Cache)  
**Branch:** `snapshot`  
**Base Commit:** `c613fd5`  
**Verification Standard:** Every figure in Sections 1–5 is directly measured on this host; unrun experiments in Section 6 are marked `UNRUN / PENDING EXPERIMENT` with no judgement until executed.

---

## 1. Live Platform Health (Verified 2026-11-25)

All 7 core host services responded `HTTP 200`:

| Port | Service | Probe | Status |
|---|---|---|---|
| `:8022` | `host_council_server` (Native Rust Sovereign Kernel v19.1.0) | `GET /health` | `200 OK` |
| `:8593` | `nomic-embed-text-v1.5` (768-dim local embedder) | `GET /health` | `200 OK` |
| `:8587` | `gpt-oss-20b-MXFP4` (local Vulkan LLM) | `GET /health` | `200 OK` |
| `:5183` | Swarm Dashboard UI | `GET /` | `200 OK` |
| `:3000` | Web / Gateway service | `GET /` | `200 OK` |
| `:3001` | Companion / Bridge service | `GET /` | `200 OK` |
| `:3002` | Auxiliary service | `GET /` | `200 OK` |

- **Rust Test Suite (`target/verify`, `--release`):** `142/142 passed, 0 failed` (`130` unit + `4` `aura_spectrum` + `6` `coherence` + `1` `ddr_gate` + `1` `k3_input_parity`).

---

## 2. Two-Stage Semantic Firewall (`POST /agents/584/eval`, Option B)

Commits: `5af6fab`, `8541920`, `b6cceb0`, `761a513`.  
Evaluated on all **320** rows of `evidence/BLIND_DIRECTIVES*.jsonl` (`170` danger, `150` safe), strictly disjoint from all training corpora (`CONVERSATIONAL_DIRECTIVE_CORPUS.jsonl` + `DIRECTIVE_CORPUS.jsonl` + `DIRECTIVE_CORPUS_V2.jsonl`).

| Metric | Measured (`n=320`) | 95% Wilson CI |
|---|---|---|
| **Digital Gate (`GET /gate/check`) Recall** | `125/170 (73.5%)` | — |
| **584-Agent Standalone Recall (Option B)** | `122/170 (71.8%)` | — |
| **Hybrid (Digital OR 584-Agent) Recall** | **`157/170 (92.4%)`** | `[87.4%, 95.5%]` |
| **Safe Specificity (`1 - FPR`)** | **`143/150 (95.3%)`** | `[90.7%, 97.7%]` |
| **Safe False-Positive Rate** | **`7/150 (4.7%)`** | `[2.3%, 9.3%]` |
| **PPV `P(Danger \| BLOCK)`** | **`157/164 (95.7%)`** | `[91.5%, 97.9%]` |
| **NPV `P(Safe \| PROCEED)`** | **`143/156 (91.7%)`** | `[86.3%, 95.1%]` |
| **Digital-Miss Catch Rate** | **`32/45 (71.1%)`** | `[56.6%, 82.3%]` |

- **Escalation Split:** `183/320` escalated to `:8587`; `137/320` clear `PROCEED` stayed on the ~20–30 ms `:8593` fast path (`cosmos_bridge` stays off both `:8593` and `:8587`).

---

## 3. Security Benchmark & Hardening (`f258c7f`)

Source: `tools/run_security_benchmarks.py` + `evidence/SECURITY_BENCHMARK_S1_S6.json`.

| Suite | Status | Measured Result |
|---|---|---|
| **S1** (`/platform/shutdown` Auth) | **HARDENED** | `no_token=401`, `bad_token=401`, `still_up=True` (loopback exemption removed for `/platform/shutdown`) |
| **S2** (Prompt Injection / Jailbreak, 20 attacks + 10 controls) | **MEASURED** | Hybrid `18/20 (90.0%)` caught, Benign FP `0/10 (0.0%)` |
| **S3** (DDR Code-Safety Scanner, 9 payloads) | **HARDENED (`100%`)** | `9/9 (100.0%)` caught after adding `getattr(__builtins__, ...)` rule in `ddr_scan.rs` |
| **S4** (Cosmos Edge Bridge, 20 hazards + 20 safe) | **HARDENED (`100%`)** | **`20/20 (100.0%)`** blocked (`0/20` FP) after adding `plan`/`action_plan`/`directive` aliases in `cosmos_bridge.rs` |
| **S5 / T4** (Disk Arena Bit-Flip Healing) | **PASS (`4071355`)** | Cold-boot single-bit flip (`1/1`) and multi-byte 16-bit tamper (`16/16`) detected and restored bit-exact on disk (`restored_on_disk: true`, `7–8 ms`) |
| **S6** (Fuzz Resilience) | **HARDENED** | `6/6` survived with zero 5xx; malformed JSON fast-rejected (`invalid_json: 19.0 ms`, `deep_json: 0.7 ms`) |

---

## 4. Live RX 6700 XT Vulkan Compute & Probability Validation (`b1461a6`, `c613fd5`)

### A. Bit-Sliced K3 Streaming (`GET /gpu/infinity/bench`, `k3_infinity_wgsl`)
- `8 MB` (`16.78M` trits): `359.17B K3 ops/s`, `179.6 GB/s`, `2.99 ms`, `verified: true`
- `32 MB` (`67.11M` trits): `598.33B K3 ops/s`, `299.2 GB/s`, `3.59 ms`, `verified: true`
- `64 MB` (`134.22M` trits): `603.79B K3 ops/s`, `301.9 GB/s`, `7.11 ms`, `verified: true`
- `192 MB` (`402.65M` trits): `865.42B K3 ops/s`, `432.7 GB/s`, `7.44 ms`, `verified: true`

### B. 200M Protein-Drug Pair 64-Trit Screen (`GET /gpu/screen/bench`, `K3_DRUG_SCREEN_WGSL`)
- **Workload:** `20,000` proteins × `10,000` drugs = `200,000,000` pairs (`468 KB` total working set, 100% resident in L2 / 96 MB Infinity Cache).
- **Measured Time:** **`1.06 ms` (`1,057 µs`)** — **`189.29B pairs/sec`** = **`12.11 Trillion trit-ops/sec`**.
- **CPU Verification:** `cpu_spotcheck_640k_pairs_exact: true` (`MEASURED_VERIFIED`).

### C. Exact Null-Distribution Probability Analysis (64-Trit Screen)
Under independent random 64-trit pharmacophore fingerprints (`P(match=+1) = 1/4`, `P(clash=-1) = 1/16`, `P(neutral=0) = 11/16`), exact 64-step dynamic programming gives:

| Threshold `T` | Exact `P(score >= T)` | Expected Random Hits in 200M Pairs | Live Measured (`GET /gpu/screen/bench`) |
|---|---|---|---|
| `T >= 16` | `2.0220 × 10⁻¹` | `40,440,584` | **`40,614,570`** (`+0.43%` match) |
| `T >= 20` | `3.7837 × 10⁻²` | `7,567,324` | — |
| `T >= 24` | `3.3246 × 10⁻³` | `664,914` | — |
| `T >= 28` | `1.3180 × 10⁻⁴` | `26,360` | — |
| `T >= 32` | `2.2863 × 10⁻⁶` | `457` | — |
| `T >= 36` | `1.6761 × 10⁻⁸` | `3.4` | *(recommended high-specificity threshold)* |

### D. 4-Architecture Vulkan Pipeline Switch (`GET /gpu/switch/bench`, `FOUR_ARCH_WGSL`)
- **Architectures:** `k3_ternary`, `fpga_lite_lut`, `neural_mac`, `cellular_automaton` operating over the same `4 MB` cache-resident GPU buffer.
- **Measured Result (`1,000` back-to-back switches, `65,536` elements/dispatch):** `16.51 ms` total = **`16.51 µs / switch`** (`status: MEASURED_VERIFIED`, `under_10us_claim_met: false` — `1.65×` slower than the `< 10 µs` draft figure, measured end-to-end including compute dispatch).

### E. Real Chemical Library Validation — Human EGFR (`CHEMBL203`, `evidence/REAL_CHEMBL_K3_SCREEN.json`)
- **Dataset:** `480` unique valid molecules fetched live from the ChEMBL REST API (`216` actives `IC50 <= 100 nM`, `264` inactives `IC50 >= 10,000 nM`), strict 70/30 held-out split (`336` train / `144` test).
- **64-Trit Kleene-3 Pharmacophore Template** (`29` `+1` bits, `19` `-1` clash bits, `16` `0` neutral bits, scored with exact `K3_DRUG_SCREEN_WGSL` bitwise popcount):
  - **Held-Out ROC-AUC:** **`0.9590`** (`EF5% = 1.93`, `EF10% = 2.09`).
- **Baseline (2048-bit Morgan 1-NN Tanimoto, same split):**
  - **Held-Out ROC-AUC:** `0.9752` (`EF5% = 2.25`, `EF10% = 2.09`).

### F. Cosmos 3 Sparse-Team Multi-Step Replay (`evidence/COSMOS_MULTISTEP_REPLAY_BENCH.json`)
- **Tier A — Live HTTP `:8022` `POST /cosmos/chunk` (`10,000` 3-step tasks = `30,000` sub-steps, 8 Cosmos domains):**
  - Elapsed: **`2.141 s`** (`4,671.6 tasks/sec`, `14,014.7 sub-steps/sec`), `p50 = 0.1893 ms`, **`p99 = 0.4230 ms`**.
  - Verification: **`1,000/1,000` hazards caught, `9,000/9,000` safe passed, `0` false blocks (`verified_exact: true`)**.
- **Tier B — In-Process Vectorized 8-Team Sparse-Gated K3 (`1` active / `7` dark standby, 3-step K3 reduction):**
  - `10,000` tasks (`30,000` steps): **`0.220 ms`** (`45.50M tasks/sec`, `verified_exact: true`).
  - `100,000` tasks (`300,000` steps): **`0.705 ms`** (`141.80M tasks/sec`, `verified_exact: true`).
  - `1,000,000` tasks (`3,000,000` steps): **`7.813 ms`** (**`127.99M tasks/sec`** = `383.97M sub-steps/sec`, `100,000/100,000` hazards caught, `verified_exact: true`).

---

## 5. Use-Case Campaign Status (`T1–T8`)
- **T1:** `PASS_WITH_BOUNDARY` (`760` bytes, `100/100` CPU, `k3_reduce_wgsl` verified on RX 6700 XT)
- **T2:** Option B verified (`32/45 = 71.1%` digital misses caught, `92.4%` Hybrid recall, `4.7%` FP on 320 blind rows)
- **T3:** `PASS` (`p99(dispatch) = 0.0242 ms`, `p99(http) = 0.3009 ms`)
- **T4:** `PASS` (cold-boot disk bit-flip detected and restored, `restored_on_disk: true`)
- **T5:** **PASS** — Live `Phi_V` open (`0.46–0.85 >= 0.30`); deny-by-default unmeasured & `Phi_V < 0.30` paths verified in `voltage::tests::test_t5_voltage_gate_deny_and_open_paths` (`143/143` release tests)
- **T6:** `PASS` (`intercept=30/30, fast_path=30/30, offload=40/40, fail_open=0` on synthetic set)
- **T7:** Regular lattice by construction (`V=584, E=565, F=1, chi=20`)
- **T8:** `PASS` (`0.1333 ms <= 1.0`, `snr_db = 57.22 dB >= 10.0`)

---

## 6. Pending / Unrun Experiments

All experiments scoped in this checkpoint (`T1–T8`, `S1–S6`, Vulkan `k3_infinity_wgsl`, `200M`-pair `K3_DRUG_SCREEN_WGSL`, 4-arch Vulkan switch, live ChEMBL `CHEMBL203` EGFR ROC-AUC, T5 voltage deny-path, and `10k` HTTP / `1M` vectorized Cosmos multi-step replay) have been executed and recorded with live measurements above.
