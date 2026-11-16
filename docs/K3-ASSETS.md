# K3 shard assets - status and handling

Measured 2026-11-16 (G163). These files are **generated artifacts**, not ingested datasets, and they
are no longer tracked in git. `data/k3_assets_manifest.json` records the size and SHA-256 of each one so a
present copy can be verified.

## What they actually contain

`ingest_sovereign_datasets.py` and the `execute_*_ingestion*.py` scripts build them like this:

```python
# ingest_sovereign_datasets.py:104-105
# Generate sample 128-dim pseudo-vector for shard representative
seed_vec = [math.sin(shard["id"] * 1.5 + j * 0.2) * (1.0 / (1.0 + j * 0.05))
            for j in range(TURBOVEC_DIM)]
```

One 128-dim **sine-wave** vector per shard, quantized to 2-bit codes, then the payload is **repeated**
until the file reaches a target size (`while written < target_cap: f.write(chunk)`), and the SHA-256 of
that padding is written into the header as a "seal".

`SHARD_CATALOG` claims real corpora - PDBbind refined 5,300 records, BindingDB 2,540,000, ChEMBL 37
2,540,000 - but no record content is read from anywhere. No local copy of those datasets exists on this
machine (checked `F:/downloads` and the repo).

## Measured evidence

| file | size | non-zero bytes | distinct 64 KB blocks |
|---|---|---|---|
| `swarm_v2_rust_kernel/swarm_v2_nand_fabric.k3` | 73.2 MB | **18 of 76,772,685** | 3 (one zero block × 72) |
| `swarm_v2_nand_fabric.k3` | 57.6 MB | - | 58 |
| `clinical_trials.k3` | 15.0 MB | 83.7 % | 99 of 240 (payload repeated) |
| `faers_adverse_events.k3` | 25.0 MB | - | 25 |
| `bindingdb_affinities.k3` | 25.0 MB | - | 25 |

The 73 MB file GitHub warned about contained **18 non-zero bytes**; it was deleted (backup in
`F:/repo-backups/k3-empties-*`).

## Regenerating

```
python ingest_sovereign_datasets.py                 # root swarm_v2_nand_fabric.k3 (57.6 MB)
python execute_full_sovereign_ingestion_matrix.py   # 18-shard matrix
python execute_pdbbind_bindingdb_ingestion_and_calibration.py
```

They are deterministic (fixed seeds) but the output is synthetic. Do not cite the shard sizes as evidence
of ingested data until a real ingest exists; `docs/NOBEL-PAPER-2026-01-REV.md` cites
"76,772,685 bytes / 153,545,370 trits" of knowledge, which this measurement contradicts.

## If real shards are wanted later

1. Obtain the source corpora (PDBbind / BindingDB / ChEMBL / FAERS) - none are on this box.
2. Replace the sine seed with a real per-record embedding loop.
3. Publish per-shard record counts that equal the number of records actually embedded, never a padding
   target.
