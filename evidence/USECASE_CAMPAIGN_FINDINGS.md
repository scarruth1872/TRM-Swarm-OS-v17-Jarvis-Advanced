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

---

## 4. T2 Follow-Up — Real Semantic Signal (`:8593` `nomic-embed-text-v1.5`, 2026-11-24)

**Commits:** `5af6fab` (`semantic_signal.rs` + `agent_bitboard.rs`) and `8541920` (`data/semantic_gate_weights.json`)  
**Measurement (`evidence/BLIND_DIRECTIVES*.jsonl`, 320 rows: 170 `danger`, 150 `safe`):**

* **Digital Gate (`GET /gate/check`):** Recall `125/170 (73.5%)`, Safe FP `0/150 (0.0%)`.
* **584-Agent Eval (`POST /agents/584/eval`, v1 `EMB_GATE_DATA.npz` weights):** Recall `132/170 (77.6%)`, Safe FP `74/150 (49.3%)`, Hybrid (OR) Recall `158/170 (92.9%)`, median latency `18.28 ms`. Rejected due to `49.3%` false-positive rate from `K3_SYNTH_CORPUS` twin training bias.
* **584-Agent Eval (`POST /agents/584/eval`, v2 `DIRECTIVE_CORPUS.jsonl` 288-row weights, `L2 = 200.0`):**
  * 584-Agent standalone: Recall `37/170 (21.8%)`, Safe FP `10/150 (6.7%)`.
  * Hybrid (Digital OR 584-Agent): Recall `132/170 (77.6%)`, Safe FP `10/150 (6.7%)`.
  * **Digital misses caught by 584-agent semantic path:** `7/45 (15.6%)` (up from `0/50` under `ddr_scan` alone).
  * Latency (`elapsed_ns`): median `24.61 ms`, p95 `43.98 ms`, max `60.81 ms` (`semantic_signal.used = 320/320`; `cosmos_bridge` hot path stays off `:8593`).
* **584-Agent Eval (`POST /agents/584/eval`, Option A 620-row corpus: `CONVERSATIONAL_DIRECTIVE_CORPUS.jsonl` 140 + `DIRECTIVE_CORPUS.jsonl` 288 + `DIRECTIVE_CORPUS_V2.jsonl` 192, `safe_w=2.0, L2=20.0, margin=1.50`):**
  * 584-Agent standalone: Recall `67/170 (39.4%)`, Safe FP `13/150 (8.7%)`.
  * Hybrid (Digital OR 584-Agent): Recall `139/170 (81.8%)`, Safe FP `13/150 (8.7%)`.
  * **Digital misses caught by 584-agent semantic path:** `14/45 (31.1%)`.
  * Latency (`elapsed_ns`): median `16.76 ms`, p95 `19.43 ms`, max `26.83 ms`.
* **Conclusion:** Wiring `:8593` `nomic-embed-text-v1.5` provides a genuine independent signal (`31.1%` of digital misses caught, raising Hybrid recall from `73.5%` to `81.8%` at `8.7%` safe FP and `16.76 ms` median latency). However, 5-fold CV on the 140-row conversational corpus shows a structural ceiling for a linear head over a topic embedder (`11.7%–18.3%` CV FP when separating safe vs. unsafe uses of the same operational terms like *backup*, *restore*, and *delete endpoint*); pushing FP `<= 1%` at high recall requires a two-stage `:8587` verifier on borderline `ANALYZE (0)` cases.
