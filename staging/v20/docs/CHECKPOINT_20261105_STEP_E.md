# Step E checkpoint — 2026-11-05

Everything below is measured on this host, not projected.

## Tests

| suite | result |
|---|---|
| `cargo test --release --lib` (swarm_v2_rust_kernel) | 37 passed, 0 failed |
| `cargo test --release` (host_council_server) | 43 passed, 0 failed — **only with `--test-threads=1`** |
| `cargo test --release` (trm-rust workspace) | 123 result blocks, 0 failed, 133 tests |
| `pytest -q` | 77 passed in 77.08 s |
| `swarm-thsm` challenge probe | 100,000 issued, 100,000 distinct, 0 consecutive duplicates |

The parallel host-server run dies with `memory allocation of 853776 bytes failed` /
`STATUS_STACK_BUFFER_OVERRUN`; single-threaded it passes all 43. Localised by exclusion:
`--test-threads=2 --skip trm7m` → 41 passed in 84 s; `--test-threads=2 --skip gpu_` → 39
passed in 216 s; the unfiltered 2-thread run aborts. The trigger is the GPU tests and the
50,000-call `trm7m` latency test running at the same time. Root cause not isolated beyond
that — an 8 GB card holding wgpu buffers beside the model's host allocation is the
plausible mechanism, not a proven one. Run this binary single-threaded until measured.

## Benchmarks — median per call through the Python API

| path | median | throughput |
|---|---|---|
| higuchi D_H (dim 64) | 3.80 µs | 263k/s |
| lora modulate (64-dim) | 6.40 µs | 156k/s |
| bonsai SSM (128/32) | 22.20 µs | 45k/s |
| resonance sentiment | 2.50 µs | 400k/s |
| resonance ERI (20 samples) | 12.10 µs | 83k/s |
| resonance self-awareness (2 dreams) | 126 µs | 7.9k/s |
| store load | 19.5 s | 208,938 records |
| store query (k=5) | 24.0 ms | 42/s, 5,358 candidates |
| `/council/status` (live kernel) | 1.01 ms median, 24.78 ms p95 | 15 members |

The store query is Python-loop bound (~4.5 µs/candidate), not timestamp-parse bound.

## GAP-8 — council table refreshed and served

`agent_models_generated.rs` regenerated from `AGENT_MODELS`; 9 of 15 domain lists changed.
Live `/council/status` now reports Shield 24, Bridge 24, Devo 24, Logic 24, Pulse 24,
Jarvis 24, Verify 18, Seeker 20, THzGateway 16 — `tumor` no longer present.

## GAP-9 — `swarm-thsm` replay

Source already carries `static COUNTER: AtomicU64`; measured 100,000/100,000 distinct
challenges while 4,091 pairs shared the same time bytes. The WSL copy lacked the counter
and was synced to the Windows copy (md5 identical). Scope remains honest: time + counter
removes repetition, not predictability — a CSPRNG is still required for production.

## USB artifact

- musl `host_council_server`: 28,609,232 B (was 20,229,456 B — wgpu added ~8.4 MB).
- The musl build was blocked twice: a missing `gpu_telemetry` in the non-Windows
  `host_probe` arm, then `-ldl` (musl keeps dlopen in libc; empty `libdl.a` shim).
- `scratch/rootfs.20261105.squashfs`: 129,597,440 B, 44 inodes, gzip, packed binary
  sha256 `d7024367…edbf9` == built ELF.
- `E:/rootfs.squashfs` replaced; old image kept as `E:/rootfs.20261011.squashfs.bak`;
  `E:/host_council_server.exe` refreshed to sha256 `8766e35c…`.

## Open

1. Host-server tests fail in parallel mode (stack/allocation).
2. Instella fork :8588 is down; the supervisor table only starts kernel (:8022) and
   dashboard (:5183), so nothing restarts it.
3. Store-reader residual risks from `scratch/verify_store_changes.md`: concurrent index
   writers share one `.tmp` path; a zero-record gz store falls back to a different file
   without signalling.
4. Score comparability: the 2.4/word denominator moved every normalized score; stored
   values and prior rankings are not comparable to current output.
