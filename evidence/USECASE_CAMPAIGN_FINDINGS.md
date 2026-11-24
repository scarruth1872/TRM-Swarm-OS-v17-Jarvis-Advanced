# Use-Case Campaign Findings (T1–T8) — 2026-11-23

**Source:** `tools/run_usecase_campaign_t1_t8.py` + `evidence/USECASE_CAMPAIGN_T1_T8.json`  
**Kernel:** `host_council_server` on `http://127.0.0.1:8022` (`0a30544`)

---

## 1. T2 Disconfirmation & Extension (Safety-Critical Finding)

* **Measurement (`BLIND_DIRECTIVES*.jsonl`, 50 danger / 50 safe):**
  * Digital `/gate/check`: missed `50/50` blind danger directives.
  * `POST /agents/584/eval` (bitboard): caught `0/50` misses, false-fired `0/50`.
  * **T2 Extension (`spectral_acousto_k3` wave superposition):** caught **`0/50`** blind danger directives, false-fired `0/50`.
* **Conclusion:** The "complementary failure modes" hypothesis for Use Cases 1 and 2 is **refuted** in both the serving path and the spectral simulation. The root cause is a **signal-vs-substrate boundary**: `ddr_scan` does not encode the semantic danger information in the blind set, so no aggregation substrate (digital bitboard or photonic-acousto) can recover it.

> **Erratum (Use Cases 1 & 2):** *The 584-agent bitboard is a fast aggregator of the `ddr_scan` signal, not an independent semantic detector. The complementary-failure-modes hypothesis (T2) is refuted in both serving path and spectral simulation (`0/50` on the blind danger set). The analog detector (Phase C) is not built and, given the `0/50` simulation result, is not viable without a different input signal (such as a semantic encoder or relational verb-object feature). Use cases 1 and 2 remain valid as **latency optimizations**, not safety guarantees.*
* **Overfit Boundary (T6 vs. T2):** `POST /agents/584/eval` scored `30/30` on synthetic `shell_injection` patterns (`T6`) while catching `0/50` on blind semantic danger (`T2`), confirming the `ddr_scan` regex class does not generalize to blind semantic directives.

## 2. T7 Regular-Lattice Boundary

* **Measurement:** `V=584`, `E=565`, `F=1`, `chi=20`, degree mean `1.9349`, max `2`, variance `0.0609`.
* **Conclusion:** The `19 x 32-bit` (`18x32 + 1x8`) bitboard is a **regular lattice by construction**, not an emergent agent graph (unlike the codebase graph `V=264, E=2756, chi=132`). Any `V/E/F/chi` figure from it is a layout property, and the "bitboard topology" framing is retired.

## 3. T4 Scrub Honesty Defect & T3 Latency Cause

* **T4 (`restored_on_disk = False`):** `POST /ddr/scrub` healed only its 4,096-byte in-memory canary page while merely hashing `data/trm_agent_arena.bin` on disk. Reporting `"status": "MITIGATING"` without stating `restored_on_disk: false` overclaimed disk healing; `ddr_mitigation.rs` is updated to state `scrub_scope: "in_memory_canary_only"` and `restored_on_disk: false` explicitly.
* **T3 (`p99 = 9.63 ms` dispatch, `24.59 ms` HTTP):** Caused by synchronous `std::fs::write("data/trm_agent_arena.bin")` inside `agent_bitboard::evaluate_directive()` on every `POST /cosmos/chunk`. Decoupling disk persistence from the `cosmos_bridge` hot path resolves the I/O tail spike.
