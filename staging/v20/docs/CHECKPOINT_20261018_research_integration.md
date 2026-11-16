# Checkpoint — Research Integration & Fabric Ingestion (2026-10-18)

## State at this checkpoint

- **Ledger**: 416 blocks, chain intact. Council: 37 motions / 37 enacted (MOTION-037 complete).
- **Kernel**: v20.0.0 healthy on :8022. Dashboard 200 on :5183 (vite HMR). Ollama stopped.

## What landed since MOTION-037 (blocks 407–416)

1. **DDR antibody tab fixed** — `ddr_antibody_count` now reports the full inventory: 52 curated
   code-scanning + 403,751 MITRE/CWE/CVE threat-matrix = **403,803** (was showing 52). Streaming
   count (visit_seq + IgnoredAny) cached in a OnceLock; `/ddr/antibodies` + `/ddr/stats` routes
   added before the `/ddr` catch-all. Commits `4aeca8e`, `645e599`.

2. **Research + Monad Journal tabs fixed** — `/research/stats` was miswired to the drug-discovery
   docking benchmark; every action returned 501. Ported the research daemon to Rust
   (`research_handlers.rs`): start/stop/run/synthesis/topics, all deterministic + honest (no LLM on
   this host). Journal sections trimmed; journal now carries a platform feed (ledger/motions/
   research-notes). Commits `7da0e79`, `194d091`.

3. **Seeker recall unified across all stores** — was `agent_retrieval` (single lexical fabric, 6/124
   hits); now `recall_router::fused_lookup` (fabric + chembl 2.9M + structured 97K + bioassay 597K +
   clinical) → **124/124**. Classify-first `fused_lookup_text` skips the 2M-record chembl/bioassay
   scans for general topics: 40.8s → 12.1s. Commits `79f85d0`, `72aa58f`.

4. **Fabric domain audit** — the fabric was 96.5% ethics + enwikiquote quotations (183,879 segments);
   the Seeker's own findings were absent. Committed the audit in `469adf5`.

5. **Research notes ingested into the fabric** — 15,316 Seeker research notes encoded (encode-batch,
   GPU, native-turbovec-hdc) and appended to the index + fabric. Fabric **183,879 → 199,195 vectors**.
   Text-sidecar regression (relational `#fragment` keys lose text on rebuild) repaired by restoring
   the old sidecar + appending note text. Verified: sidecar 199,195 == turbovec 199,195; notes
   retrievable via `/k3/retrieve`. Commits `469adf5` + `scratch/{ingest_research_notes,repair_fabric_text_sidecar}.py`.

## Honest gaps (carried forward)

- The 15,316 notes are Seeker **recall logs** (many `recall_only`, `compute: null`) — the platform's
  real accumulated history, not synthesized conclusions.
- The fabric's general recall surface still skews values-heavy; scientific content is the papers (124)
  + the notes (15,316) + the separate domain stores.
- The `motions` count in the journal platform feed counts motion-referencing blocks (52), not the bare
  37 enacted motions.

## Next

Run real research tasks (search the system's existing leads), then convene the council for the next
motion.
