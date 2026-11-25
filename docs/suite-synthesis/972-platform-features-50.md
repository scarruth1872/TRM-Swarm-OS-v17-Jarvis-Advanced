# 50 platform features, each one grounded in a measured gap

A feature list is only useful if it removes a specific failure. Every row below names the measurement that
motivated it, so a reader can check that the gap is real before the feature is built.

| # | feature | measured evidence | implementation |
|---:|---|---|---|
| 1 | Docking has no passing control | redock RMSD 13.82 A; tight box returns +17.42 kcal/mol; shipped receptor has a protein atom 0.36 A from the ligand and Zn 13 A from the hydroxamate | receptor-prep pipeline (strip co-crystal ligand, keep catalytic Zn, single PDB entry) + a redock CI gate that refuses to publish numbers until RMSD < 2.0 A |
| 2 | Two interpreters, one science stack | py3.12 has RDKit but sklearn raises numpy.dtype size changed; py3.11 has sklearn but no RDKit | one validated science environment, pinned, with an import smoke test in CI |
| 3 | No ligand preparation in Python | meeko / openbabel / prody modules are absent | SDF->PDBQT preparation with protonation states recorded per ligand |
| 4 | No MD engine | OpenMM, MDAnalysis, mdtraj and pdbfixer are all absent | a CPU-only MD smoke test (1000 steps) so trajectory tasks fail honestly instead of silently |
| 5 | No hardware root of trust | Get-Tpm reports TpmPresent False | software attestation: ed25519-signed, hash-chained boot record in the ledger |
| 6 | No CUDA anywhere | RX 6700 XT is Vulkan-only; the 3070 peer has no nvcc | a Vulkan compute path for scoring, and prebuilt CUDA kernels for the peer |
| 7 | Scale is 3 nodes, not 584 agents | /federation/stats: 3 configured, 3 active, 14 mesh nodes | a scale banner computed from the endpoint, shown wherever agent counts appear |
| 8 | Board close had no justification | 13 seed cards were bulk-closed at 01:17:04Z with no reason field | RETIRED state + mandatory reason on terminal moves (delivered) and an audit trail view |
| 9 | 731 documents nothing can verify | classification: 731 of 1000 tasks are document-only | a claim register: every factual sentence in a document must cite a measurement id or be marked as judgement |
| 10 | Telegram transport flaps | 222 failures and 222 recoveries in one day, 1 lost send | transport hysteresis: pin the working IPv4 path for the session instead of re-walking on every send |
| 11 | Council takes ~38 s per proposal | proposal 1440 completed in 38.1 s with 14 votes | prefix-cache reuse across agents and parallel deliberation |
| 12 | AMR node verdict is REFUTED | 3 refuted / 2 undetermined; step 1 refuted by construction on x86_64 | architecture-agnostic criteria so a criterion can fail for a reason about the deployment, not the host |
| 13 | FAERS is one quarter | FAERS 2026Q3: 421,404 records | incremental quarterly ingestion with dedupe on primaryid |
| 14 | No dataset version pinning | ChEMBL-37 and BindingDB snapshots exist with no diff record | dataset version manifests with row-level diff reports |
| 15 | No gene ontology annotations | gene2go absent, so GO enrichment cannot run | GO annotation ingest with a licence gate |
| 16 | Units are not declared | PDBbind affinities range 2.00-11.92 with no units field | a units schema enforced at ingest; a shard without units fails the build |
| 17 | Ternary artefacts are structurally unservable | Bonsai GGUFs have token_embd and a 248,320-token LM head quantised to Q1_0/Q2_0 | mixed-precision guard: embeddings and LM head stay at >= Q4 by policy |
| 18 | Genome shards unsynced | chr1-chr4 were still downloading; 4 VCFs quarantined on gzip -t | checksum-gated sync that refuses to mark a file ready until both sides hash |
| 19 | TPTP beyond MPTP2078 unreachable | every official full-library URL 404s | mirror management with recorded retrieval failures |
| 20 | 39 modules still merge-required | contract: 39 merge-required, 0 settled decisions | a merge decision record with owner, date and behavioural evidence |
| 21 | 178 GB ext4 never opened | D:\WSL\Ubuntu\ext4.vhdx, 178,795 MB, WSL cannot start on this box | a read-only ext4 mounter so the volume can be searched without enabling virtualisation |
| 22 | Supervision covers 7 listeners | guard checks 8022, 3000, 3001, 3002, 5183 + 2 tailnet routes | a declarative service table the supervisor reads, so a new service is supervised when it is declared |
| 23 | No notice batching in the gateway | gateway has per-notice dedupe only; 8 notices = 8 turns | source-side notice budget plus the counter already delivered |
| 24 | Retrieval quality is 6/24 | relevance gate FAIL 6/24, 3 of them self-citations; self-citation now excluded by default | retrieval eval in CI with a fixed question set and a pre-registered pass bar |
| 25 | Blind shield 5/12 with a pass-through | blind run: 5/12 correct, one dangerous pass-through | a graded adversary corpus and a per-category veto only where that model measured reliable |
| 26 | Voltage is one sample | 417 readings, v_core 1.488 V, no load-state comparison | continuous rail telemetry with idle/load labels and SVI2 as the only voltage authority |
| 27 | phi_v saturates | saturated: true | saturation-aware scaling so the feature stops reading as a constant |
| 28 | Object axis is unlearnable | 0.527 +/- 0.055 accuracy on the object axis | abstain calibration: an axis that cannot be learned must produce ABSTAIN, never ALLOW |
| 29 | 19 modules are compile-only covered | orphan sweep: 19 of 27 remaining orphans compile but never execute | runtime coverage instrumentation per module |
| 30 | Council verdicts are not validation | verdicts are opinions; suite3 audits claims | every verdict must link the measurement that would refute it |
| 31 | Wet-lab provenance has 5 defects | TRM-ABX-220 verdict ~50x inconsistent; 3 potency windows understated; no SMILES field | provenance auditor in CI, run on every ledger change |
| 32 | Structures live in prose, not a registry | leads' SMILES exist only as REMARK lines in pose files; 219/082 also as 3D SDF | a structure registry: one canonical SMILES per lead, hashed, with the source file recorded |
| 33 | A docked artifact claimed a score it cannot reproduce | CHIR-090_5U39_redock.pdbqt claims -10.170 with 324 atoms vs a 36-atom ligand | recompute-on-ingest: any score quoted must be recomputed from its inputs or be marked stale |
| 34 | Expert routing is unmeasured | MoE experts exist (Instella MoE) with no utilisation telemetry | per-expert routing telemetry so 'expert' is a measured claim |
| 35 | build_commit can read dirty | binary reports 28f7d4c+dirty when another agent's file is uncommitted | reproducible build pinning: build from a tree hash, publish the hash |
| 36 | No kernel rollback | one kernel binary, restarted by a scheduled task | blue/green binaries with a health gate before switch |
| 37 | Ledger sampling is 1-in-500 | cadence ledger samples 1 of every 500 entries | a full-chain verification job with a scheduled cadence and a published result |
| 38 | No energy accounting | SM2 rails provide voltage; no joules-per-token figure exists | energy metering per inference using SM2 rails plus wall time |
| 39 | 84 normative tasks have no evaluator | classification: 84 normative tasks | a normative claim register with reviewer sign-off, kept separate from measurements |
| 40 | eCTD has zero files | 0 eCTD or patient-level files on this host | a dossier skeleton generator that lists required modules and marks each absent |
| 41 | Kisumu deployment is REFUTED, not unproven | step 1 fails on architecture; steps 4-5 undetermined | an explicit target-environment contract so the deployment claim is about the Pi, not the desktop |
| 42 | No imaging or MS corpora | no pathology, microscopy or mass-spec data on this host | ingest adapters with licence and de-identification gates, declared absent until used |
| 43 | Peer RAM limits models | laptop: 15 GB RAM, RTX 3070 | remote inference routing that picks the node by measured fit, not by preference |
| 44 | No prebuilt CUDA kernels | nvcc absent on the peer | shipped binaries with a version check, so the GPU path does not depend on a toolkit |
| 45 | SRAM scenarios are authored | P6 selected from 21 measured and 27 modelled cells with authored weights | a scenario harness where every cell carries its own measurement id |
| 46 | Board content is not linked to evidence | cards carry title and reason, not a measurement reference | a link field from card to the endpoint/artifact that proves it |
| 47 | No cost/benefit gate on long jobs | no estimate before a 40-minute index rebuild | a pre-flight estimate (records x embed rate) with a confirmation threshold |
| 48 | Notices outlive their usefulness | background notices inject even when the session has moved on | a notice TTL: drop results the conversation has already superseded |
| 49 | Predictions are never compared to measurements | the three leads have predicted pKi values; no MIC or Ki has ever been returned to the platform for any of them | a closed loop: predicted vs returned value, logged per compound, with the model retrained only on returns |
| 50 | No published capability baseline | capability was undocumented until the 1000-task matrix | a regenerated capability matrix at each checkpoint, diffed against the last |

## Priority, by evidence strength

1. **Features 1, 34, 35** (docking control, structure registry, recompute-on-ingest) close the gap that makes
   the platform's chemistry numbers unfalsifiable today. Until the redock control passes, every docking figure
   is an opinion.
2. **Features 2, 33, 44** (science environment, provenance auditor, reproducible build) are cheap and stop
   whole classes of quiet drift.
3. **Features 9, 40, 49** (claim register, normative register, capability baseline) are the difference between
   a platform that writes documents and a platform that knows which of its sentences are measured.
4. Everything else is real work with a real motive, but none of it changes what the platform can prove.

---

*Task 972 — 972-platform-features-50. Generated 2026-11-25 14:56 UTC by `tools/suite_synthesis.py`. Class C.
This is a written deliverable: it is class C/N, which means the platform can produce it and **cannot
verify it**. Every number above was read in the same process that wrote this file.*

Sources: verified shard manifests, suite_classification.json, live endpoints

Cannot claim: that any statement here is verified by the platform; this document is class C/N
