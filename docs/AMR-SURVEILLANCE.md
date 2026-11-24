# Hospital AMR surveillance node — measured deployment report

`GET http://127.0.0.1:8022/amr/deployment` → `trm_uefi_kernel/host_council_server/src/amr_surveillance.rs`

## Why this replaced the scratch module

`scratch/kernel194/src/hospital_amr_surveillance_node.rs` produced five steps whose `measured_metric`
fields were **authored strings** — "Substrate recognized in 0.42 ms", "Total Parser Memory Footprint:
14.8 MB (< 20 MB constraint)", "SHA-256 State Anchor: 0x8FA4912DEBC31002" — and set every step to
`K3Trit::Unknown`. Its own source said so: *"Nothing verifies these steps: the metrics beside them are
authored strings."* A report that measures nothing cannot answer "did the deployment run".

## What each step now measures

| step | measurement | source |
|---|---|---|
| 1 Hardware | `std::env::consts::ARCH`, `OS`, `available_parallelism()` and whether that equals the aarch64 target | this process |
| 2 Parser suite | counts `*_serialize.py` in the directories the repo uses and publishes which dirs were searched; counts ARM64 parser binaries (0) | filesystem |
| 3 Knowledge fabric | **SHA-256 of the evidence file** (replacing the authored anchor) + how many of the 7 named resistance genes actually appear | `eskape_spectrum_full.json` |
| 4 Pipeline | times a real `serde_json` ingest, and states that the MIC/EUCAST parser and the longitudinal counts a Z-score needs are absent | timed |
| 5 Sovereign | reports that no offline drill, blackout test or cost measurement was run — from inside the process | none |

The trit is **derived from the measured value**, never chosen: `+1` meets the criterion, `-1` contradicts
it, `0` means the measurement exists but does not decide the criterion, or nothing measurable is possible
without hardware this host lacks. The verdict is the Kleene AND of the five trits, so a single refuted step
cannot read as a deployment.

## Measured result on this host (2026-11-23)

```
verdict REFUTED | steps 5 | passed 0 | refuted 3 | undetermined 2
k3 {and: -1, or: 0, unknown: 2, n: 5} | report_latency_ms 2.347

Step 1 trit=-1  arch=x86_64 on 24 cores; target is aarch64
Step 2 trit=-1  7 *_serialize.py found in ['tools'], 0 ARM64 parser binaries (claim: 20 ARM64 parsers)
Step 3 trit=-1  eskape_spectrum_full.json 6,515 B sha256 062b0cf3449094a6…, 3/7 named genes present
                (claim: 1,000 signatures)
Step 4 trit= 0  real serde_json ingest 0.244 ms; MIC/EUCAST parser and outbreak Z-score absent
Step 5 trit= 0  no offline drill, no blackout test, no cost measurement
```

**REFUTED is the correct answer here, not a failure of the report.** The target is a Raspberry Pi 4; this
host is x86_64, so step 1 is contradicted by construction. The value of the endpoint is that a reader learns
*which* claims fail and *by how much*, from numbers the process computed, in 2.3 ms.

## Defect found by running it

The first build counted **0** serialisers, which was a false negative: step 2 scanned only `.` and `../../`,
and the ingestion serialisers live in `tools/`. A count taken from the wrong directory is not a measurement
of absence. The search now covers the real directories and publishes which ones it used — hence
`searched_dirs: ['tools']` in the response.
