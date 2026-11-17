# Ingest throughput, measured on this box (Ryzen 9 3900X, 12C/24T)

Rule: every worker count must produce the SAME seal. A differing seal means the chunking changed content and the
speedup is a different shard. Worker count is measured, not inferred from core count.

## ChEMBL-37 descriptors (`tools/ingest_parallel.py`, RDKit + native encoder)

| workers | records/s | 20k wall | full 2,897,805 ETA | seal |
|---|---|---|---|---|
| 1 | 590.7 | 33.9 s | 81.8 min | 5d0abdd2cf51ab69 |
| 12 | 4,335.5 | 4.6 s | 11.1 min | 5d0abdd2cf51ab69 |
| 16 | 4,783.4 | 4.2 s | 10.1 min | 5d0abdd2cf51ab69 |
| 24 | 4,622.2 | 4.3 s | 10.4 min | 5d0abdd2cf51ab69 |

Actual full run, 16 workers: **2,897,805 records in 354.4 s (8,175 rec/s)**, seal 52a30b7e... (G207).
16 beats 24 on 12C/24T; oversubscribing past physical cores costs a little.

## ARC-AGI concept-aug-1000 (`--mode arc`)

| workers | records/s | 20k wall | full 291,778 ETA | seal |
|---|---|---|---|---|
| 1 | 67.9 | 294.5 s | 71.6 min | 4995342cfa7f1907 |
| 16 | 833.5 | 24.0 s | 5.8 min | 4995342cfa7f1907 |

Actual full run, 16 workers: **291,778 puzzles in 354.3 s (823.6 rec/s)**, fidelity 0.9291 (n=24), seal 5904b4fc...

## Reference points

- HTTP embedding server: 16-17 vec/s (274 ms single, 36.8 ms/vec batch-8, 56.7 ms/vec batch-32) - batching cannot
  move it; batch-32 is WORSE because it overruns the 512-token ubatch.
- Native encoder in-process: 4,135 rec/s desktop, 7,880 rec/s laptop (127 us).
- Small benchmarks under-predict: 20k projected 10.1 min, actual full run 5 m 54 s.
