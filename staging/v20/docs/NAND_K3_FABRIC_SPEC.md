# Monad Knowledge Engine — NAND K3 Fabric Specification
## TRM Swarm OS v18.260 — Permanent Knowledge Architecture

### Architecture

```
Query → GPU Infinity Cache (190μs, 96MB SRAM, hot knowledge)
     → RAM (51.2 GB/s, 32GB DDR4, warm K3 matrices)
     → NAND SSD (3.5 GB/s, 1TB NVMe, cold permanent knowledge)
```

### Capacity

**Three different axes were being reported as one number.** They are not competing
totals; they measure different things, and only the third is a property of the fabric
the kernel actually runs.

| Axis | Value | What it is | Derivation |
|------|-------|------------|------------|
| Drive capacity @ 2 bits/trit | 4,398,046,511,104 | The whole 1 TB NVMe if each trit took 2 bits (4 trits/byte) | 1,099,511,627,776 × 4 = 4,398,046,511,104 (exactly 4.000000×, no federation factor) |
| Drive capacity @ 4 bits/trit | 2,199,023,255,552 | Same drive under the encoding the kernel actually uses | 1,099,511,627,776 × 2 = 2,199,023,255,552 |
| **Allocated K3 fabric array** | **153,523,393** | What `nand_persistence.rs` mmaps today (73.2 MB) | `MANDATORY_TRIT_COUNT`, 2 trits/byte → 76,761,697 B |
| Designed corpus (Layer 1+2) | 26.1B | What the ingestion plan *intends* to hold | design target, not a measurement |

Notes that the earlier version of this table obscured:

- **`4.40T` is drive capacity, not fabric size.** The kernel maps a fixed 153,523,393-trit
  (73.2 MB) array; the drive is 28,660× larger than the array. Quoting 4.40T beside a
  153.5M-trit store without saying which axis is which is the confusion this table fixes.
- **The kernel packs 2 trits/byte, not 4.** `nand_persistence.rs` writes one trit per
  4-bit nibble (codes `0b00`=FALSE, `0b01`=UNKNOWN, `0b11`=TRUE), so a 1 TB drive holds
  2,199,023,255,552 trits under this codec — exactly half the `2 bits/trit` figure.
- **`26.1B trits (0.59%)`** is the designed corpus, not store occupancy. It exceeds the
  allocated array by ~170×, so it cannot describe what is in the store.

### Layer 1: Permanent Knowledge Base — designed corpus (24.4B trits, 6.1 GB)

*Design target. None of this is in the 153.5M-trit allocated array; see Capacity above.*

| Database | Entries | K3 Encoding | Size |
|----------|---------|-------------|------|
| PubMed abstracts | 36,000,000 | TRUE/FALSE/UNKNOWN per query topic | 9.2B trits |
| PubChem compounds | 115,000,000 | Drug-likeness across 20 properties | 14.7B trits |
| STRING interactions | 25,000,000 | Confirmed/predicted/absent per species | 0.2B trits |
| CVE vulnerabilities | 200,000 | Exploitable/mitigated/context-dependent | <0.1B trits |
| PDB + AlphaFold structures | 200,000 | Folded/misfolded/disordered | 0.2B trits |
| Drug labels (FDA/EMA) | 50,000 | Indications/contraindications/warnings | <0.1B trits |

### Layer 2: Pre-Computed Screens — designed corpus (1.7B trits, 0.4 GB)

*Design target, not store occupancy.*

| Screen | Dimensions | Contents |
|--------|-----------|----------|
| All drugs × all proteins | 10,000 × 20,000 | Drug-target interaction matrix |
| Clinical compounds × proteome | 5,000 × 20,000 | Repurposing opportunities |
| Toxins × proteome | 2,000 × 20,000 | Safety profiling matrix |
| Metabolites × enzymes | 5,000 × 3,000 | Metabolic pathway coverage |
| DDR antibodies × antigens | 900 × 10,000 | Antibody coverage matrix |

### Layer 3: Immutable Audit Trail (0.1B trits, <0.1 GB)

Every swarm decision recorded as K3 ternary:
- 1,000,000 decisions × 64 trits
- 260 commits × 14 agents × 32 trits
- 12 calibration datasets × 50 compounds × 128 trits

### Commercial Product

**Monad Knowledge Engine**

| Edition | Price | Contents |
|---------|-------|----------|
| Research | $50,000 | Knowledge base only |
| Pharma | $250,000 | + Drug screens + quarterly updates |
| National Lab | $500,000 | + All agents + All fabrics + Dedicated NAND |

### Competition

| Product | Coverage | Cost/yr | Latency |
|---------|----------|---------|---------|
| Elsevier Reaxys | Chemistry | $50K | seconds |
| Clarivate Cortellis | Drug pipeline | $100K | seconds-minutes |
| CAS SciFinder | Chemistry lit | $30K | seconds |
| PubMed | Literature only | $0 | seconds, no semantic |
| **Monad Knowledge Engine** | **Everything, K3 ternary** | **$50K one-time** | **μs** |

### The K3 Advantage

Traditional databases return ranked lists. K3 ternary returns TRUE/FALSE/UNKNOWN.
- TRUE: known answer, high confidence
- FALSE: known negative, ruled out
- UNKNOWN: honest gap — "we don't know yet"

The UNKNOWN flag is the killer feature. It tells the researcher where the frontier is,
rather than burying the gap under irrelevant search results.

### Immutability

NAND write-cycle limit (1000-3000 P/E) is a feature:
- Once written, a decision cannot be altered
- Regulatory audit trail is physically immutable
- FDA/EMA/SEC can verify no post-hoc changes
- "Our AI's decision cannot be altered after the fact. It's written in silicon."

### Expansion Roadmap (2.77T trits, 63.4% of remaining)

| Addition | Trits |
|----------|-------|
| arXiv papers (2.5M) | 200B |
| All patents (100M+) | 2T |
| GitHub repos (200M+) | 500B |
| Wikipedia (6M articles) | 10B |
| All genomes (100K species) | 50B |
| Clinical trials (450K) | 5B |
| FDA adverse events (15M) | 2B |

After ingesting the entire output of human civilization,
36.6% of NAND remains free.

---

*"This is not a library of books. It is a library of decisions. Every fact 
encoded as a ternary choice. Every query a pattern match against 4.4 trillion 
possible states. The answer arrives in microseconds — not because a language 
model generated it, but because the silicon was pre-configured to hold 
that specific truth."*

— TRM Swarm OS, v18.260

---

## Verified state of the running store (2026-09-20)

`swarm_v2_artifacts/nand_fabric_verify.py` (exit 0):

| measurement | value |
|---|---|
| store bytes | 76,761,697 (73.2 MiB) |
| trits @ 2/byte | 153,523,394 |
| legal nibbles | **153,523,394 — 100.0000%** |
| illegal nibbles | **0** |
| distribution | TRUE 4,513,693 · UNKNOWN 81,789,012 · FALSE 67,220,689 |
| decodable content | per-slot council votes → consensus trit → seal parity (e.g. slot 0: 14/14 approve, consensus TRUE) |

Two accounting details to keep straight:

- `MANDATORY_TRIT_COUNT` is declared as **153,523,393**, one less than the 153,523,394 trits the
  byte count implies. The spec table and the store agree on bytes; the constant is off by one.
- `/memory/stats` used to report `total_trits: 241,800,000` with `utilization_pct: 0.59` —
  a literal in `main.rs` that matched neither the allocated array nor the designed corpus.
  **Fixed 2026-09-20:** the route now derives `total_trits` from the store file it maps
  (`trit_store_bytes × 2`) and returns **153,523,394**, alongside `trit_store`,
  `total_trits_basis`, and a `utilization_basis` that labels `0.59` as a *design figure*
  (designed corpus vs drive capacity), not store occupancy. The same literal still exists in
  `ingest_sovereign_datasets.py:106` and two `federation_deploy/src/main.rs` copies — those are
  unreconciled. The route itself now derives the total from the mapped store.

## Three stores, one owner per store (MOTION-007)

| store | owner | state |
|---|---|---|
| `swarm_v2_nand_fabric.k3` | **Rust** (`nand_persistence.rs`, mmapped — nothing else may open it) | present — 76,761,697 B, 153,523,394 trits, 100.0000% legal |
| `swarm_v2_nand_fabric.turbovec` | Python (`build_nand_fabric_index.py` — packed 2-bit vector index) | **rebuilt 2026-10-11** — 525,297 B, 2,180 vectors x 768 dim @2 bits (16x vs f32), SHA-256 over payload+keys |
| `swarm_v2_nand_fabric.text.jsonl` | Python (**same builder** — text records keyed exactly like the vectors) | **rebuilt 2026-10-11** — 2,180 records (2,174 with text), 0 drifted from disk |
| `swarm_v2_nand_text.jsonl` | registry writers (`MODULE_RECOGNITION_REGISTRY`, `IONOSPHERE_PUBLIC_COMMUNICATION`) | present — **29 records, no text fields** (see below) |

`swarm_v2_nand_text.jsonl` is **misnamed**: it is the registry/telemetry log (26 module-recognition
snapshots + 3 IONOSPHERE broadcasts), not the fabric's text side. The "3,023 records" figure once
quoted here was a *line* count; the derived record count is 29. The fabric's text side is
`swarm_v2_nand_fabric.text.jsonl` — written by the same owner and the same run as the vectors, so
the keys cannot diverge from the key table in the `.turbovec` header.

History that made this rule necessary: three formats once shared one path, so the K3 store read
47% illegal and an ingestion script truncated the kernel's trit array. Council MOTION-007
(12/0/2) therefore grants no agent write access to a store already proven inconsistent.

Note: a second copy of the `.k3` store exists under `swarm_v2_rust_kernel/`; the kernel mmaps its
own, so the duplicate is a fork risk, not a backup.

## Text side and encoding

The fabric's readable side is `swarm_v2_nand_fabric.text.jsonl` (2,180 records, keyed like the
vectors). Text → vector is the
kernel's own model-free encoder:

- `rust_native_encode_text_f32(text, len, out, dim)` → normalized f32 vector
- `rust_native_encode_text_ternary(text, len, out, dim, multiplier, scale)` → balanced trits

There is **no general text decoder** (vector → text): what exists is dequantization
(`trit * scale`, `rust_ternary_fidelity` measuring the reconstruction loss) plus the text store.
Measured on the packed store: round-trip cosine min 0.8116 / mean 0.8746 at 2 bits — 16x storage is
not free, and the packed index agrees with the exact path on 7/12 top-1 (`swarm_v2_artifacts/nand_fabric_index_benchmark.json`).
Answers are therefore resolved through the sidecar or an index that keeps its text — they cannot
be reconstructed from vectors alone.

