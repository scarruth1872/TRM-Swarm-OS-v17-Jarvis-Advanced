# HOST KNOWLEDGE INVENTORY

Measured 2026-11-07 on the Windows 11 host. Read-only survey of `F:\` and `C:\Users\shawn`.
Companion machine-readable file: `evidence/HOST_KNOWLEDGE_INVENTORY.json` (52 records).

Every byte count below came from `os.stat`. Line counts for gzip JSONL were obtained by
streaming decompression (never loading the archive into memory). Where a count could not be
obtained cheaply it is `null` in the JSON and marked **n/m** here, with the reason given.

---

## 1. What exists, by type

| Category | Scope | Files | Bytes |
|---|---|---|---|
| JSON (ledgers, registries, memory blobs) | repo | 110,404 | 450,131,667 |
| Log / plain text | repo | 9,855 | 336,867,679 |
| Markdown | repo | 8,461 | 32,404,755 |
| Model weights (.pt/.gguf/.bin/.k3/trits) | repo | 330 | 54,387,480,731 |
| JSONL / JSONL.GZ | repo | 50 | 482,738,560 |
| CSV | repo | 46 | 2,513,733 |
| Vector (.npy/.lance/.faiss) | repo | 9 | 66,218,112 |
| Parquet | repo | 8 | 141,192 |
| SQLite / DB | repo | 7 | 78,765,052 |
| **repo total** | | **129,170** | **55,837,261,481** |
| Markdown (agent homes: .claude/.codex/.continue/.gemini) | host | 3,748 | 23,484,573 |
| JSON | host | 2,633 | 8,820,010 |
| JSONL (agent transcripts) | host | 755 | 250,208,820 |
| Text | host | 631 | 76,696,268 |
| SQLite / DB | host | 547 | 868,886,826 |
| **agent-home total** | | **8,314** | **1,228,096,497** |
| JSON / MD / TXT / CSV (Documents, Desktop, Downloads) | host | 4,642 | 64,534,603 |

`F:\` walk completed: **191,731 files**. `C:\Users\shawn` walk was aborted after ~1,125,000
files (see §7) - the per-type table above covers every directory that holds a candidate store,
plus the three user-document roots. The figure is therefore a census of *stores*, not of every
file on the host.

### The headline numbers

- **Personal-experience material: 3,164,368,390 bytes (2.95 GiB)** across 14 stores. This is
  the operator's own words, decisions and agent history. Roughly **455 MB** of it is text once
  protobuf/binary/tool-blob overhead is stripped; ~**220 MB** survives dedup. *(the 455/220 are
  estimates and labelled as such - only the state.db ratio below is measured)*
- **Reference material: 148,178,155 bytes (141 MiB)** across 16 stores, of which the genuinely
  unique prose is ~90 MB. The rest is compiled K3 domain data with no text form.
- **Existing indexes: 706,304,802 bytes** across 6 stores - reusable, not to be rebuilt.
- **Model / weight artefacts: 4,669,393,442 bytes** across 8 records (repo-wide weights total
  51.8 GB including checkpoints and duplicated Jarvis model trees).
- **Junk: 2,214,715,047 bytes** - of which 971 MB is operational logs, 880 MB archived
  snapshot trees, 363 MB templated synthetic curriculum, 1.1 MB ARC token streams.

---

## 2. Ranked top 20 for ingestion

Score = **unique text MB x query-value multiplier (QV 0.4-3.0)**. QV is a stated judgement, not
a measurement: 3.0 = the operator's own decisions in their own words; 2.4-2.6 = first-person
history carrying reasoning; 1.5-2.0 = supporting artefact or third-party text; <=1.2 = volume
without uniqueness. The text-MB column is *estimated* usable text after stripping JSON framing,
blobs and logs; only the rows marked measured in §3 are exact.

| # | Source | Size | Rows | Layer | Text MB | QV | Score |
|---|---|---|---|---|---|---|---|
| 1 | `C:\Users\shawn\AppData\Local\hermes\state.db` | 2030.4 MB | 224,632 | personal | 86.2 | 3.0 | **258.6** |
| 2 | `C:\Users\shawn\.claude\projects  (218 jsonl)` | 127.5 MB | 30,833 | personal | 60.0 | 2.4 | **144.0** |
| 3 | `C:\Users\shawn\.gemini\antigravity\brain  (530 jsonl + 210 md)` | 125.5 MB | 60,493 | personal | 64.0 | 2.2 | **140.8** |
| 4 | `C:\Users\shawn\.gemini\antigravity\conversations  (43)` | 477.3 MB | 43 | personal | 75.0 | 1.6 | **120.0** |
| 5 | `F:\...\quantum_cerebral_agent_platform\chroma_db\metadata.db` | 53.5 MB | 80,035 | personal | 45.0 | 2.6 | **117.0** |
| 6 | `F:\...\TRM-Swarm-OS-v2\swarm_v2_global_memory\turbo_memory.jsonl.gz` | 71.1 MB | 136,169 | reference | 60.0 | 1.2 | **72.0** |
| 7 | `F:\...\TRM-Swarm-OS-v2\.swarm\mailboxes` | 71.8 MB | 44,950 | personal | 37.0 | 1.8 | **66.6** |
| 8 | `F:\...\TRM-Swarm-OS-v2  .md total (8,461 files)` | 30.9 MB | 8,461 | reference | 30.9 | 1.9 | **58.7** |
| 9 | `F:\...\TRM-Swarm-OS-v2\swarm_v2_memory` | 23.9 MB | 10,137 | personal | 21.0 | 2.5 | **52.5** |
| 10 | `F:\...\quantum_cerebral_agent_platform\backend\social_memory.db` | 12.4 MB | 28,178 | personal | 11.0 | 2.4 | **26.4** |
| 11 | `F:\...\TRM-Swarm-OS-v2\data\council_ledger.json` | 5.6 MB | 942 | personal | 5.3 | 3.0 | **15.9** |
| 12 | `F:\...\TRM-Swarm-OS-v2\swarm_v2_global_memory\system_event_ledger.jsonl` | 13.1 MB | 28,265 | personal | 11.0 | 1.3 | **14.3** |
| 13 | `C:\Users\shawn\.continue\index\lancedb` | 588.2 MB | n/m | index | 20.0 | 0.4 | **8.0** |
| 14 | `F:\...\TRM-Swarm-OS-v2\evidence  (170 measured reports)` | 2.7 MB | 170 | evidence | 2.6 | 2.5 | **6.5** |
| 15 | `F:\...\TRM-Swarm-OS-v2\swarm_v2_global_memory\fabric\*` | 68.5 MB | 42,805 | index | 4.0 | 1.5 | **6.0** |
| 16 | `F:\...\ub3\uqi-platform\data\kanban-config\db.sqlite` | 104.6 MB | 4,135 | personal | 3.0 | 1.5 | **4.5** |
| 17 | `F:\...\TRM-Swarm-OS-v2\swarm_v2_global_memory\chroma` | 53.7 MB | 7,246 | index | 3.0 | 1.5 | **4.5** |
| 18 | `F:\...\TRM-Swarm-OS-v2\data\council_evidence_index.json` | 3.9 MB | 558 | index | 1.4 | 1.6 | **2.2** |
| 19 | `F:\...\TRM-Swarm-OS-v2\monad_ledger.jsonl` | 0.1 MB | 100 | evidence | 0.1 | 3.0 | **0.2** |
| 20 | `F:\...\TRM-Swarm-OS-v2\*.k3 domain stores (10 files)` | 45.8 MB | n/m | reference | 0.0 | 2.0 | **0.0** |

### Why each score lands where it does

**1. C:\Users\shawn\AppData\Local\hermes\state.db** - Operator's own 4,622 sessions / 224,632 messages; 19.8M chars user-authored. Dedup measured: 140.9M -> 86.2M chars (1.63x). FTS5 already present.
**2. C:\Users\shawn\.claude\projects  (218 jsonl)** - Coding sessions with real diffs and reasoning; ~45% text yield after stripping tool_result blobs.
**3. C:\Users\shawn\.gemini\antigravity\brain  (530 jsonl + 210 md)** - Agent streams plus implementation_plan / task / walkthrough markdown - the intent layer behind built features.
**4. C:\Users\shawn\.gemini\antigravity\conversations  (43)** - Largest raw personal store but protobuf/.db encoded; ~15% recoverable text. Value is in the steps table, not byte ratio.
**5. F:\...\quantum_cerebral_agent_platform\chroma_db\metadata.db** - 80,035 distilled memory rows with full text - already-summarised experience, highest signal per byte outside Hermes.
**6. F:\...\TRM-Swarm-OS-v2\swarm_v2_global_memory\turbo_memory.jsonl.gz** - 136,169 web-research summaries; many are dictionary/LLM boilerplate, so low uniqueness per row despite volume.
**7. F:\...\TRM-Swarm-OS-v2\.swarm\mailboxes** - 44,950 inter-agent messages across 103 mailboxes - includes refusals and corrections, which are scarce and valuable.
**8. F:\...\TRM-Swarm-OS-v2  .md total (8,461 files)** - Docs, papers, research, staging and artifact notes. Quality varies wildly (scratch/ alone is 9.8MB) - filter to docs+papers+research+root = 1.12MB of prime text.
**9. F:\...\TRM-Swarm-OS-v2\swarm_v2_memory** - Per-agent episodic memory: task -> result -> status. Logic_memory.json alone 10,137 entries.
**10. F:\...\quantum_cerebral_agent_platform\backend\social_memory.db** - 28,178 memory rows; likely overlaps the 80,035 above - dedupe against it before scoring.

**What this ranking says plainly:** the corpus is not won by the biggest files. The 500 MB of
Antigravity conversations and the 588 MB of stale LanceDB vectors rank mid-table, while 65 KB of
`monad_ledger.jsonl` and 1.1 MB of docs+papers carry near-maximum QV. The top four are all
*session transcripts of the operator working* - that is the actual asset.

---

## 3. Measured detail on the primary stores

| Store | Measured | Method |
|---|---|---|
| `hermes\state.db` | 2,129,072,128 B; 4,622 sessions; 224,632 messages; 140,886,626 chars content; 117,433,464 chars reasoning; 58,781 distinct contents (86,175,963 chars distinct) | SQLite `count`/`sum(length())`, read-only URI |
| messages by role | user 10,736 (19,802,927 chars) - assistant 105,137 (22,702,292) - tool 108,745 (98,378,502) - system 14 (2,905) | same |
| `turbo_memory.jsonl.gz` | 74,558,002 B; **136,169** lines | streaming `gzip.open` line count, 7.0 s |
| `fabric_distinct.jsonl.gz` | 6,273,637 B; **42,805** lines | streaming, 0.5 s |
| `fabric_vecs.npy` | (42,805, 768) float16, 65,748,608 B | `numpy.load(mmap_mode='r')` - header only, no payload read |
| `chroma.sqlite3` | 7,246 embeddings; collections `predictive_intents` (384d) + `swarm_global_memory` (384d); 50,222 metadata rows | SQLite counts |
| `qm chroma_db\metadata.db` | 80,035 rows, cols `id,text,metadata,timestamp` | SQLite counts |
| `social_memory.db` | 28,178 rows, same schema | SQLite counts |
| `.claude\projects` | 218 jsonl; 30,833 lines; 133,674,059 B | line count + stat |
| `.gemini\antigravity\brain` | 530 jsonl = **60,493** lines / 115,106,827 B; 210 md = 1,491,044 B; 1,360 .log = 1,253,132,063 B (excluded) | walk + per-extension sums |
| `.gemini\antigravity\conversations` | 43 files / 500,495,630 B; `.db` are SQLite with `steps`, `trajectory_meta`, `gen_metadata`; `.pb` are protobuf | schema read on one .db |
| `.swarm\mailboxes` | 103,006 files / 75,321,527 B; 103 mailboxes; **44,950** inbox messages | JSON parse of every `inbox.json` |
| `swarm_v2_memory` | 16 files / 25,106,160 B; `Logic_memory.json` = 10,137 entries | list length |
| `council_ledger.json` | 942 deliberations; keys `id,topic,category,proposer,phase,weighted_score,approve_count,abstain_count` | JSON load |
| `system_event_ledger.jsonl` | 28,265 lines / 13,754,483 B | line count |
| `curriculum_50k` | 15 files x 50,000 = **750,000** rows / 335,976,000 B | `wc -l` per file |
| `k3_heads.bin` | 31,322,002 B; 15 heads; magic `K3HD` v2; featuriser = hashed BoW crc32 dim 96; classes BLOCK/ANALYZE/PROCEED | `k3_heads.json` manifest |

### Deduplication status - measured, not assumed

- `state.db`: **58,781 distinct message contents out of 224,632** rows. Dedup removes 63% of rows
  and 39% of characters. This is the only measured dedup ratio on the host and the best available
  predictor for the rest.
- `fabric` is the **only already-deduplicated corpus** (42,805 distinct ids from a larger pool,
  with matching vectors). It is the natural dedup key registry.
- `curriculum_10k` is a strict subset of `curriculum_50k` (same generator, 1/5 the rows).
- `system_event_ledger` exists in three variants (canonical 13.75 MB, `.unverified_backup` 5.27 MB,
  `.corrupt-20261103` 4.83 MB) - ingest canonical only.
- `evidence/` and `preserved_sources/` contain byte-identical md (867,443 B each) - duplicate trees.

---

## 4. Personal vs reference - the split that must not be collapsed

**Personal-experience (the operator's words, decisions, history).** 14 stores, 2.95 GiB:
Hermes `state.db`; three agent-transcript trees (`.claude`, `.gemini/brain`, `.gemini/conversations`);
`.swarm/mailboxes`; `swarm_v2_memory`; `data/council_ledger.json`; `social_memory.db` and the
quantum-platform `metadata.db`; `system_event_ledger`; `kanban` DBs.

**Reference / generic text.** 16 stores, 141 MiB: platform docs, papers, research notes, root
markdown, the 136k web-research summaries, the 10 K3 domain stores, and (rejected) the IMDB parquet.

**Why the split is mandatory:** the two answer different questions and fail differently. A personal
record can be wrong and still be *true as history* - "on 2026-09-11 the operator believed X" is
correct even if X is false. A reference record is only allowed to be ingested when it is
verifiable. Mixing them lets an unverified claim inherit the authority of a dated observation, and
lets a dated observation be cited as fact. Every record therefore carries `layer` and must never be
queried without it.

---

## 5. Models, weights and reusable indexes

**Weight artefacts (do not treat as knowledge):** `k3_heads.bin` (31.3 MB, 15 x 522,027 ternary
params); `trm7m_weights.bin` (15.6 MB); `trm7m_weights_turbovec_158.bin` (1.27 MB); ten
`gguf_*_trits.bin` packs (up to 2.33 GB); `models\qwen2.5-1.5b-instruct-q4_k_m.gguf` (1.12 GB);
`swarm_v2_nand_fabric.k3` (60.5 MB); plus 51.8 GB of repo-wide `.pt`/`.safetensors`/`.gguf`
including a triplicated `Jarvis-Advanced-main` model tree.

**Reusable indexes - build on these instead of starting over:**

| Index | Rows | Dim | Reuse |
|---|---|---|---|
| `fabric\fabric_vecs.npy` + `fabric_distinct.jsonl.gz` | 42,805 | 768 | Primary embedding substrate; ids map 1:1 |
| `global_memory\chroma\chroma.sqlite3` | 7,246 | 384 | Small-embedding tier, already migrated |
| `data\council_evidence_index.json` | 558 | 768 | Proves chunk->vec path over docs+evidence |
| `.continue\index\lancedb` | n/m | n/m | 30 stale per-project tables; low value |

The 768-dim fabric and the 768-dim evidence index are dimension-compatible with each other and
with nomic-embed / mxbai-embed (both present as `gguf_*_trits.bin`). The 384-dim Chroma tier needs
its own model. **Do not mix 384 and 768 in one retrieval index** - pick 768 for the new corpus.

---

## 6. Recommended ingestion architecture

**Four layers, one record format, no cross-layer merging.**

1. **Personal experience layer** - `state.db`, agent transcripts, council ledger, agent memories,
   mailboxes. Query form: "what did I do / decide / say about X, and when". Never deduplicated
   against reference text; deduplicated only within itself, and *never* across sessions by content
   alone (the same sentence at two dates is two facts).
2. **Reference knowledge layer** - docs, papers, research, verified web research. Query form:
   "what does the platform claim about X". Requires a `verified` flag; unverified stays `draft`.
3. **Decision / training layer** - the K3 heads and the 10 K3 domain stores, plus
   `council_ledger` outcomes as labels. This is not text retrieval; it is the (feature -> action)
   record used to train or audit the 15 ternary heads. Keep it in its native ternary form and
   address it by `(agent, class, feature_hash)`.
4. **Evidence / measurement layer** - `evidence/*_MEASURED.json`, `hard_corpus`, chain-verified
   ledger blocks. Immutable, hash-anchored, quotable in isolation. This layer is what stops the
   corpus from repeating unverified claims.

**Provenance-stamped record format** (one JSONL line, compatible with the existing
`monad_ledger.jsonl` block schema):

```json
{"id":"<sha256(source|ordinal)>","layer":"personal|reference|decision|evidence",
 "source_path":"...","source_kind":"hermes_session|claude_jsonl|antigravity_step|mailbox|council|md",
 "session_id":"...","ordinal":1234,"ts":"2026-09-11T13:57:00-04:00",
 "role":"user|assistant|tool|document","text":"...","text_sha256":"...",
 "tokens":412,"lang":"en","topic":["k3","docking"],
 "verified":"measured|asserted|unverified","sensitivity":"private",
 "prev_block_hash":"0x...","ingest_run":"2026-11-07T18:00Z"}
```

`prev_block_hash` chains into the existing `monad_ledger` mechanism rather than inventing a second
one. `sensitivity: private` keeps personal records out of any export path by default - the
transcripts here contain paths, keys and inner reasoning.

**Dedup strategy across the two existing corpora.** Do not re-hash 136k turbo_memory rows. Reuse
`fabric_ids.json` as the seen-set, add exact `text_sha256` for new records, and apply
MinHash/SimHash (threshold ~0.9) only for the personal layer where near-duplicates are expected:
`state.db` shows 63% row redundancy, and mailbox/`swarm_v2_memory` are heavily templated. Expected
effective volume: **~220 MB personal + ~90 MB reference text**, plus ~48 MB of compiled
decision-layer stores - small enough to index in one pass on this host.

**Honest limits.**

- Retrieval over text is **not** understanding. A corpus that returns the right paragraph has
  demonstrated search, not comprehension.
- **No amount of ingestion makes an untrained ternary model knowledgeable.** `k3_heads.bin` is 15
  hashed-BoW heads x 522,027 ternary params = **7.83M ternary values** (the brief's "3.9M-parameter"
  figure does not match the measured head count - trust the manifest), featurised at dim 96. Such a
  head can learn to route a query to a layer, and nothing more. If the "Monad Corpus" is expected to
  *answer*, the answer comes from retrieval plus a separate generative model, not from the heads.
- 136k of the "memory" rows are LLM summaries of web searches - they inherit the errors of
  whatever produced them and must be marked `asserted`, not `measured`.
- The quantum-platform memory DBs (108k rows combined) have unknown provenance and no schema
  documentation; their quality is unverified until sampled.

**What would prove the corpus is useful.** One measurement, pre-registered:

1. Hold out the **most recent 5% of sessions by timestamp** (~230 sessions) before building
   anything. They never enter the index.
2. Write 200 queries *from those held-out sessions* - the operator's real questions, with the
   real answers as ground truth.
3. Measure **held-out query hit rate at k=5**: the fraction of queries where a chunk from the true
   source session appears in the top 5. Also record MRR.
4. Report against a control: the same retrieval over `fabric_distinct` alone (42,805 rows,
   already indexed). If the new corpus does not beat the control, the ingestion work added nothing.

Only a hit rate materially above the control - with the number stated, on held-out data - justifies
calling the corpus useful. Nothing in this inventory claims that yet.

---

## 7. What could NOT be measured, and why

1. **Full file inventory of `C:\Users\shawn`.** The walk exceeded 1,125,000 files and 19 minutes
   without finishing (a large subtree dominates; `.gemini` alone is only 127,208). Aborted. Per-type
   figures in §1 cover the store-bearing directories, not every file. Exact total: **null**.
2. **`.gemini/antigravity/implicit` row count** (13 files, 17.8 MB). Framing is binary-ish; a JSONL
   parse was not attempted to avoid a guess. **null** in JSON.
3. **`.continue/index/lancedb` row counts** (30 Lance tables, 588 MB). Requires the Lance reader;
   not installed. **null**.
4. **`.gemini/antigravity/conversations/*.pb`** (protobuf blobs). No schema available; only the
   `.db` variants were decoded. Text yield is an estimate, not a measurement.
5. **K3 domain store row counts** (`bindingdb_affinities.k3` etc., 48 MB). Requires the K3 reader;
   the ternary layout is documented only in `k3_heads.json`, which covers the heads, not the stores.
   **null**.
6. **The 8,461 repo markdown files individually.** Totals per directory were measured (32.4 MB);
   quality per file was not assessed, so the "unique prose ~90 MB" figure for reference material is
   an estimate.
7. **IMDB parquet row counts.** Not opened - rejected on judgement before measurement.
8. **Duplicate rate between `social_memory.db` (28,178) and quantum `metadata.db` (80,035).**
   Suspected overlap, not measured; would need a join on text hash.
9. **Whether `state.db` `tool` rows (98.4M chars) are worth keeping.** Measured volume only; the
   keep/drop decision needs a sample audit, which this pass did not perform.

---

## 8. Files written

- `F:\Development sites\TRM-Swarm-OS-v2\evidence\HOST_KNOWLEDGE_INVENTORY.md` (this file)
- `F:\Development sites\TRM-Swarm-OS-v2\evidence\HOST_KNOWLEDGE_INVENTORY.json` (52 records)

No file outside `evidence/` was created, modified, moved or deleted. No services started, no ports
touched.

## RECONCILIATION 2026-11-07 - the 60 GB that is not knowledge

Two independent sweeps of this host disagreed by orders of magnitude, so both were re-measured by
path rather than argued about.

A depth-7 extension sweep (files >1MB) reported: safetensors 25 files / 46.4 GB, gguf 31 / 22.7 GB,
csv 26 / 15.9 GB, bin 72 / 14.7 GB, txt 69 / 4.4 GB. This inventory reported reference knowledge at
141 MiB. A 15.9 GB CSV population cannot be 2.5 MB, so the scopes differed.

Measured by path, the discrepancy resolves completely:

  safetensors 46.4 GB   97 files in Development sites/ub3/qwen_moe_tiny_output (MoE training
                        checkpoints), 6 in F:/models/instella-moe-think, plus qwen_moe_tiny,
                        qwen_moe_model and the gemma_swarm_adapter. ALL MODEL WEIGHTS.
  csv 15.9 GB           ~10.7 GB of it is six files under
                        F:/downloads/hack-the-track_-ai-racing-dashboard (4)/.../telemetry_data.csv,
                        motorsport telemetry for an unrelated hackathon project. The remainder is
                        JetBrains open-telemetry-metrics logs and ~3 MB of llama.cpp per-backend op
                        tables. NONE of it is knowledge about this platform.
  gguf 22.7 GB          LLM weights already catalogued in the model registry.
  bin 14.7 GB           includes k3_heads.bin and weight blobs.

Conclusion: the big bytes on this host are weights and one telemetry dataset. This inventory's
knowledge figures stand unchanged - 2.95 GiB personal experience across 14 stores and 141 MiB
reference across 16 - and the extension sweep was counting tensors, not text.

Consequence: exclude by PATH, not by extension. "*.csv > 1MB" would have pulled 10.7 GB of lap data
into a corpus meant to hold the operator's experience.
