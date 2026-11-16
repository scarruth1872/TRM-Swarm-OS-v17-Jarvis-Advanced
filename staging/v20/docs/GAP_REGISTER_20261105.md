# Gap register — 2026-11-05

Every row states what was measured, not what was intended. Status is `FIXED` (verified by a run),
`CLOSED` (did not reproduce), or `OPEN` (no fix yet).

| ID | Gap | Measured before | Status now |
|---|---|---|---|
| GAP-1 | Registry undercounts the strips | manifest listed **105**; **265** DLLs on disk, all loadable | **FIXED** — manifest regenerated to 265; `rust_strip_registry_count()` returns **265**, `rust_strip_audit()` reports **265 dispatchable / 265 present** |
| GAP-2 | Python-only core modules | 48 of 262 had no strip — but 37 are build/verify tooling | **PARTIAL** — true runtime gap was **11**; **`qsar_predictor`**, **`fractal_telemetry`** and **`lora_response_modulator`** now run in Rust, leaving **7** |
| GAP-3 | "QUIC" that is not QUIC | `trm_quic.rs` bound a `UdpSocket`; no quinn/quiche/TLS in any manifest | **FIXED** — renamed `trm_datagram`, headers corrected in both crates; listener still answers (`2 + 2 = 4.`) |
| GAP-4 | USB carried a stub | `EFI/BOOT/BOOTX64.EFI` was **3,072 bytes** (a minimal PE); exe and dll were 2026-10-11 | **FIXED** — 16,488,960-byte kernel written, read-back hash `4ffb4f85` matches the QEMU-booted image; exe `b636782e` and dll `c9137464` match their build outputs |
| GAP-5 | Council tally exceeds the roster | proposal 1001 reported 13+3+0 = 16 against 15 members | **CLOSED** — reads 12+3+0 = **15** votes from 15 distinct agents; no duplicate |
| GAP-6 | `federation` panicked outside the crate | exit 101 at `fs::read("test_data/100.hea")` | **FIXED** — data root resolved at startup; runs from `C:/Users/shawn` with exit 0 |
| GAP-7 | EFI builds are not reproducible | identical source produced `d0d4c783` then `4ffb4f85` | **OPEN** — a hash identifies the artifact, never its provenance |
| GAP-8 | `agent_models_generated.rs` lags its generator | 9 of 15 agents stale (Shield 12/24, Bridge 18/24, …) | **OPEN** |
| GAP-9 | Captions asserting unperformed work | federation printed "Post-Quantum SECURED (NIST Cat 5)", "0.00% data loss", "16/16 authenticated" | **PARTIAL** — 6 federation captions derive from their fields; `aura_cognition` dropped `phonon_mode_thz: 15.53`, `dissipation_uj: 0.084`, per-branch `frequency_hz` and the `cross_resonance` computed from them, all of which were arithmetic over declared constants; ~84 metric functions still unaudited |
| GAP-10 | `swarm-thsm` challenge replay | nanosecond timestamp only; 595/1000 consecutive pairs identical | **OPEN** |
| GAP-11 | `rootfs.squashfs` on the stick | 2026-10-11, 97,996,800 bytes | **OPEN** — needs a rebuild, not a copy |
| GAP-12 | Synthesized strips counted as ports | **30 of 265** strips are template output: pseudo-random `TRIT_WEIGHTS`, a magic `SCALE_FACTOR`, and a header saying `Synthesized: 2026-...`. They load and export a function that is not an implementation of the module they are named after. | **MEASURED** — the manifest now carries `stub: true` on those 30; `rust_strip_registry_count()` = 265 and `rust_strip_stub_count()` = **30**, so 235 are real implementations |

## GAP-2 — the runtime modules still Python-only (7)

**Closed in this band:** `qsar_predictor` — the kernel already served `qsar/predict` through
`swarm-drug` (bit-exact vs the joblib model, worst diff 4.42e-13 on six controls). The Python class
loaded its own 74 MB copy of that model; it now packs the RDKit fingerprint
(`np.packbits(..., bitorder="little")`, verified to reproduce every control `fp_hex`) and asks the
kernel. Six controls: 6/6 same class, worst deviation 4.36e-5 from the module's own 4-decimal
rounding.

**Closed in this band:** `fractal_telemetry` — the Higuchi D_H loop moved to
`swarm_v2_rust_kernel/src/higuchi.rs` and is **bit-identical** to the numpy version on 300 random
windows (worst diff 0.0; 0/300 differ at 4 dp). It is exact only because the Rust mirrors a numpy
quirk: `float + np.float32` promotes to float32, so the reference accumulated in f32 — first attempt
with f64 accumulators disagreed by 2.05e-7 and changed the 4th decimal in 1 of 300 windows. The Python module keeps the ring buffer and calls the DLL;
there is no second implementation, so the two cannot drift.

**Closed in this band:** `lora_response_modulator` — `strip_lora_modulator.dll` turned out to be one of
the 30 synthesized stubs (pseudo-random ternary weights, `SCALE_FACTOR = 31.9556`), which is why the
Python module loaded it, set `_NATIVE_LOADED = True`, and then built every response in Python. The
modulation now lives in `swarm_v2_rust_kernel/src/lora_modulate.rs` and is **string-identical on 400
random deltas** (0 mismatches, whole-output comparison). The stub is no longer loaded.

`alphagenome_bridge`, `aura_cognition`, `cognitive_ingestion_engine`,
`cosmos3_edge_bridge`, `cosmos_sparse_moe_teams`,
`on_device_knowledge_engine`, `persistent_mmap_allocator`.

The other 37 (test_/verify_/build_/measure_/benchmark_/ingest_/…) are developer tooling. They run at
build and audit time, not in a turn, so porting them to Rust buys nothing and costs the audit trail.

## What the remaining 7 actually do (measured)

| module | evidence from the source | verdict |
|---|---|---|
| `on_device_knowledge_engine` | calls `rust_native_encode_text_ternary` (L134), `rust_ternary_index_search` (L145), `rust_voltage_node_eval` (L197), `rust_council_evaluate` (L233) | **already native** — the Python only shapes paths and JSON |
| `alphagenome_bridge` | env lookup, URL builder, optional gRPC client | **client bridge** — an HTTP client in the kernel buys nothing |
| `cosmos_sparse_moe_teams` | an 8-team literal table plus a keyword `in` match; its only arithmetic is `len()` | **table router** |
| `cosmos3_edge_bridge` | action-dict validation, callback registry, intent-buffer injection | **glue** |
| `cognitive_ingestion_engine` | HTTP fetch, JSONL ledger, token encode (native strip), kernel call | **orchestration** |
| `persistent_mmap_allocator` | `mmap` wrapper over ctypes | **OS glue** |
| `aura_cognition` | K3 through the Rust binder; metaconcept table and tree walk in Python | **mixed** — only the table/traversal is Python |

**Conclusion:** none of the seven contains an unported numeric algorithm. The remaining port work is
table/traversal movement (aura) and OS/network glue that belongs where it is.

**Duplicate removed:** `alphagenome_kernel_engine.py` was a near-copy of `alphagenome_bridge.py` —
same class, same five methods, differing only in docstrings and no-key status strings — and nothing
imported it. It was untracked by git, so deleting it is not recoverable; the surviving bridge was
exercised afterwards (`ONLINE_AWAITING_API_KEY`, a real Atlas URL, `ATLAS_EXPLORATION_LINK_GENERATED`).

## Corrections this band

- The first port-gap number (**175**) was wrong: it only consulted the manifest, so the 161 strips
  that were built and loadable but never registered were counted as Python-only. The corrected
  measurement reads the filesystem and **loads** each DLL.
- The reported UEFI text defect was **withdrawn**: it came from OCR on a downscaled screenshot. Pixel
  analysis showed 81 glyph runs for 81 characters, single-drawn, at both 1024×768 and 1280×800.
