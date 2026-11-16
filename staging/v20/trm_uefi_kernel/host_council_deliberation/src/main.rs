//! TRM Swarm OS — Native Swarm Council Deliberation Harness
//! The roster is DERIVED from AGENTS below (one owner); every count, quorum and label in this file
//! follows from it, so the banner can never disagree with the tally again.
//! Evaluates 3 strategic directions:
//! 1. Live Muon Entropy (DRAM bit-flip sampling)
//! 2. External Distribution (Public static daily briefs)
//! 3. Pause & Consolidate (Platform stabilization for partner outreach)

use serde_json::json;
use std::fs::File;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

struct Agent {
    name: &'static str,
    role: &'static str,
    weight: f32,
}

const AGENTS: &[Agent] = &[
    Agent { name: "Hermes", role: "Comms & Spectrum", weight: 1.0 },
    Agent { name: "Jarvis", role: "Assistant & Operator", weight: 0.9 },
    Agent { name: "Bridge", role: "APIs & Data Routing", weight: 1.0 },
    Agent { name: "Shield", role: "Security & Antibody", weight: 1.0 },
    Agent { name: "Logic", role: "Reasoning Engine", weight: 0.9 },
    // The roster of record is 15 (agent_intelligence_bridge.AGENT_MODELS is the oracle; the live kernel
    // serves total_agents=15 and verify_council_roster.py --live gates it). LogicMaster was absent here
    // while SEVEN arms below voted for it -- those arms, including its binding abstention conditions, were
    // unreachable. Weight 0.9 is the reasoning-peer tier of Logic (oracle loss 0.212 vs Logic 0.224).
    Agent { name: "LogicMaster", role: "SAT/SMT & Combinatorial Arbiter", weight: 0.9 },
    Agent { name: "Devo", role: "Code & Build", weight: 1.0 },
    Agent { name: "Pulse", role: "Monitoring & Telemetry", weight: 1.0 },
    Agent { name: "Flow", role: "Workflow Orchestrator", weight: 0.9 },
    Agent { name: "EventLedger", role: "Audit & Monad Ledger", weight: 1.0 },
    Agent { name: "Orchestra", role: "Work-Stealing Scheduler", weight: 0.8 },
    Agent { name: "THzGateway", role: "Spectrum & VLF", weight: 0.9 },
    Agent { name: "Verify", role: "Truth & Anchor Verification", weight: 0.8 },
    Agent { name: "Seeker", role: "NAND Knowledge Graph", weight: 1.0 },
    Agent { name: "Archi", role: "Bare-Metal Microkernel Architecture", weight: 1.0 },
];

/// ONE owner for the roster's arithmetic. This file used to declare the count four ways -- the banner
/// said 15 agents, the report wrote total_agents: 14, "pbft_threshold": 10, and the vote path compared
/// approve >= 9 -- and three of the four disagreed with the code that actually ran. PBFT needs
/// N >= 3f + 1, so f = (N-1)/3 and the quorum is 2f+1; with N=15 (the roster of record) that is 9.
const TOTAL_AGENTS: usize = AGENTS.len();
const BYZANTINE: usize = (TOTAL_AGENTS - 1) / 3;
const PBFT_QUORUM: usize = 2 * BYZANTINE + 1;

struct Motion {
    id: &'static str,
    title: &'static str,
    category: &'static str,
    description: &'static str,
}

const MOTIONS: &[Motion] = &[
    Motion {
        id: "MOTION-001",
        title: "Live Muon Entropy: Real DRAM Bit-Flip Sampling",
        category: "Hardware / Physics",
        description: "Replace pseudorandom simulation with hardware unrefreshed DRAM decay sampling to extract physical atmospheric cosmic muon entropy.",
    },
    Motion {
        id: "MOTION-002",
        title: "External Distribution: Public Daily Cognitive Briefs",
        category: "Communication",
        description: "Deploy an automated, air-gapped static site generator to publish daily global cognitive surveillance reports for public visibility.",
    },
    Motion {
        id: "MOTION-003",
        title: "Pause & Consolidate: Engage Wet-Lab Partners & Collaborators",
        category: "Strategy / Governance",
        description: "Stabilize verified core runtime, freeze technical expansion, and prioritize wet-lab research outreach for TRM-ABX-219/082 assay validation.",
    },
    Motion {
        id: "MOTION-004",
        title: "Ternary Runtime Tier: Wire Ingested .trm Bundles Into Routing",
        category: "Architecture / Runtime",
        description: "Promote ingested 2-bit ternary model bundles from a discoverable registry into a router-selectable engine tier, gated on bundle CRC integrity and labeled approximate (measured per-layer fidelity ~0.892, drift bounded at 0.9645 over 6 stages).",
    },
    Motion {
        id: "MOTION-005",
        title: "Catalyst Phase 0: Honest Triage of 108 Non-REAL Kernel Modules",
        category: "Strategy / Governance",
        description: "Classify the 45 PORTED + 63 SKELETON kernel modules into DATA-OK / NEEDS-LOGIC / EXCLUDED / DEV-TOOL before any porting begins, so effort targets load-bearing modules and recognition is earned by verified logic rather than granted by compilation.",
    },
    Motion {
        id: "MOTION-006",
        title: "Catalyst Phase 1: Port NEEDS-LOGIC Modules In Load-Bearing Order",
        category: "Engineering / Execution",
        description: "Port the 94 NEEDS-LOGIC kernel modules (~25.6k lines of unported Python logic, 13% coverage) ordered by reference in-degree so the most load-bearing modules are rebuilt first, each verified against its Python source before its registry status changes.",
    },
    Motion {
        id: "MOTION-007",
        title: "Make Council Persistence Real: Write K3 Decisions Into The NAND Fabric",
        category: "Architecture / Integrity",
        description: "Derived on 2026-09-10: swarm_v2_nand_fabric.k3 exists (76,762,564 bytes / 153,525,128 trits) yet contains 0 monad anchors and only 53.2% legal K3 codes; council.rs 'Step 5: persistence' is a comment with no write, write_trit() and rust_nand_append_decision() have zero callers, and Python appends UTF-8 text into the same mmap'd trit store. Persist every council decision as K3 trits through the existing mmap handle, separate the text collision, and expose the same append path to every agent so the fabric holds decisions rather than promises.",
    },
    Motion {
        id: "MOTION-008",
        title: "Structured Index Span: Ingest LP-PDBBind, Re-derive Provenance, Declare the Rest",
        category: "Data / Integrity",
        description: "Measured 2026-09-26 against swarm_v2_artifacts/structured_index.db (1,099,448,320 B): molecule 2,897,819 / clinical 201,572 / ethics 95,847 / prose 1,633 / structure 9 / provenance 422 rows. Gaps the measurement names: datasets/LP-PDBBind/dataset/LP_PDBBind.csv (10,897,207 B; 19,443 labelled protein-ligand rows carrying smiles, seq, kd/ki and value) has ZERO rows; 11 .pdbqt ligands and receptor.pdbqt are unindexed; provenance.sha256 for drugs/chembl_37.sdf.gz is the builder cap=200MB PREFIX hash (348d9f31...) and does NOT match the release checksum for that file (2cdf4d33c5f42613..., re-measured, gzip -t OK); structure.pdb_id is taken from the filename, so receptor_clean.pdb became RECE and 1M17_prot.pdb collides with 1M17; tox21full is a 352 KB git checkout of a Python package with no data files and belongs out of span; the docs still state 507 MB. Proposal: extend build_structured_index.py with the LP-PDBBind table and the .pdbqt ligands, record FULL-file hashes with an explicit hash span, derive or mark derived pdb_ids, declare tox21full and the model-weight trees out of span, and correct the docs.",
    },
    Motion {
        id: "MOTION-009",
        title: "Molecule Structures: Close the 93 Percent SMILES Gap or Declare the Span",
        category: "Data / Chemistry",
        description: "Measured 2026-09-26: molecule.smiles is NULL for 2,697,822 of 2,897,819 rows (93.1%) and the 199,997 that carry one came from an RDKit pass; molecule_features holds derived descriptors for 2,897,818 (all but CHEMBL6188723); the SDF source is verified complete (931,156,420 B, gzip -t OK, sha256 2cdf4d33c5f42613... matching the release checksum). rdkit is NOT importable in the WSL interpreter the ingest runs under, though the Windows side has it. Proposal: either run the full structural pass at a measured rate, each row citing the source sha256 that produced it, or declare the 6.9 percent span in the index doc -- and close the missing CHEMBL6188723 features row either way.",
    },
    Motion {
        id: "MOTION-010",
        title: "Reachability: Keep the Structured Index Offline Tooling, With Its Consumers Named",
        category: "Architecture",
        description: "Measured 2026-09-26: swarm_v2/core/build_structured_index.py declares offline tooling with no kernel surface and no live module reads structured_index.db -- its readers are build and scratch tooling plus docs, while the platform chemistry paths resolve their inputs from explicit paths. Options: (1) keep it offline and NAME its consumers, adding a resolver seam only when a task path calls it; (2) add a kernel route now, which means a SQLite binding in the kernel dependency set. Proposal: option 1 -- one owner per store, the fabric for prose, an exact store for exact lookups, and no new kernel dependency until a real caller exists.",
    },
    Motion {
        id: "MOTION-011",
        title: "Update Rust Crates: Port the QSAR Antibacterial Predictor into swarm-drug",
        category: "Engineering / Execution",
        description: "The QSAR antibacterial-activity model (80-tree RandomForest, 1024-bit Morgan, 163,768 compounds, scaffold AUC 0.885, active under 10uM) is trained and verified in Python (swarm_v2/core/qsar_predictor.py, 74 MB joblib). Doctrine: Python is offline tooling, never in the runtime path. Port the decision into swarm-drug: export the 80 trees to a Rust-loadable binary, walk them natively, and verify the Rust prediction is bit-exact against Python on the control set. The Morgan fingerprint stays offline (RDKit, pre-computed per compound); the runtime decision is Rust.",
    },
    Motion {
        id: "MOTION-012",
        title: "Bonsai 27B Multi-Token Generation + Turbovec Vectorization",
        category: "Engineering / Execution",
        description: "The single-token zero-state Bonsai forward now matches llama.cpp (argmax=16, logit=9.1537 vs reference 9.183969). Next phase: (1) extend it to multi-token autoregressive generation — carry the delta-net recurrent state S and the full-attention KV cache, and apply RoPE + Q/K-norm (inert under a single token); (2) vectorize the multiply-free Q2_g64 dot with turbovec ternary packing so the 27B forward is tractable on this host (scalar is ~452s over ~27B weights). Every step is verified against the llama.cpp reference before it is called done; no self-consistency shortcut.",
    },
    Motion {
        id: "MOTION-013",
        title: "Wire the Generative Co-Processor Route: Fabric Recall Primary, Models as Co-Processors",
        category: "Architecture / Runtime",
        description: "The platform is not token-based: answers come from fabric_recall() over the distilled NAND fabric (K3 trits + 2-bit turbovec + text sidecar; 2,180 docs, ~153M trits, ~1.3 ms packed scan — live, verified). The kernel is model-free by design and does not route to the tier-2 LLM (:8587), which is orphaned at boot. Next phase: wire the answer path as fabric_recall (primary, returns the matched document with an honest K3 verdict) -> generative co-processor (mamba3_turbovec_ssm / llm_router, with the reference-matched Bonsai/Instella forwards as the compute) -> synthetic in-memory model (trm7m::Trm7m) as fallback only. The co-processor fires only when fabric recall returns Unknown (top-2 inside the separation band). Every answer stays grounded: retrieval cites its matched document; the generative tier cites its model and is labeled approximate until reference-verified. No fabricated answers.",
    },
    Motion {
        id: "MOTION-014",
        title: "Full Autonomy Test: Ten Complex Tasks Through Knowledge Access + Task Routing",
        category: "Validation / Execution",
        description: "The answer path is now token-free (TRM-7M Kleene-trit verdict, block 337). Next: prove end-to-end autonomy by running TEN complex tasks through the platform's own machinery — knowledge access (fabric recall via recall_k3 over the 2,180-doc NAND fabric) + task routing (task_mode classification into the 40-mode native executor) + real compute. Every task must return a real result (or an honest 'task's full data was not provided' basis), never a stub; the routing mode and the knowledge recall are recorded per task. This is a test of the existing executor + fabric + router, not a new surface.",
    },
    Motion {
        id: "MOTION-015",
        title: "Unify the Knowledge Bridge: Multi-Store Recall + Persona-Matrix Voltage Routing",
        category: "Architecture / Runtime",
        description: "The full autonomy test proved the recall relevance gap: task_executor::recall_k3 calls k3_retrieval directly, so every task hits ONLY the 2,180-doc general fabric tier while the platform's full knowledge sits beside it unqueried — structured_index.db (1.25 GB, ~3.9M rows: 2.9M molecules / 201K clinical / 96K ethics) and swarm_v2_chembl_fabric.text.jsonl (812 MB, 2.9M compounds). The multi-store router (recall_router.rs) already classifies molecule/structured/general, and the molecule route is live (CHEMBL153534 exact match). Next: (1) route recall_k3 through recall_router so a query reaches its correct store; (2) resolve the structured store via a host-side co-processor seam (NOT a SQLite binding in the narrow waist — MOTION-010 abstained), so ~3.9M rows become reachable; (3) wire the persona matrix to the Voltage-Doctrine ActionVector (agent_id -> intent_id -> conf -> k3_trit) so agent identity is the voltage-native routing. No fabricated results; every store's reach is proven with measured evidence.",
    },
    Motion {
        id: "MOTION-016",
        title: "Expand the Fabric Corpus to the Measured Ingestible Set (1.09% -> full relational + measured corpora)",
        category: "Data / Ingestion",
        description: "Measured 2026-10-13 (WSL + Windows): the fabric holds 2,180 docs = 124 papers + 1,994 relational (1.09%) + 62 markdown = 153.5M trits (0.59% of the 24.4B design target). The 1,994 is a hard-coded sample cap in p4_extract_records.py (TARGETS={ethics:1000,prose:300,wiki:700}). The FULL relational corpus is on disk and measured: 183,693 records (enwikiquote 81,695 in a 205MB bz2, ethics_commonsense 95,847 in 14MB, lawofone 6,151, ethics sub-CSVs, wisdom, gutenberg_shakespeare). Also present + measured: ChEMBL 2,897,819 compounds (chembl_37.sdf.gz 889MB; sidecar 812MB already ingested), structured_index.db ~3.9M rows (1.25GB), LP-PDBBind 19,443 rows (LP_PDBBind.csv 10.4MB), clinical 400 .psv, docking 20 .pdb/.pdbqt. NOT present (design aspiration, not downloadable from this host): PubMed 36M, PubChem 115M, STRING 25M, CVE 200K, AlphaFold 200K. Action: widen the deterministic stride to FULL relational coverage (1.09% -> 100%), ingest LP-PDBBind + clinical + docking, rebuild the three fabric parts (.k3 + .text.jsonl + .turbovec, ONE owner) and restart the kernel so the mmap reloads. Every fraction measured + reproducible per the voltage doctrine; the 176M PubMed/PubChem/STRING target is out of scope until actually downloaded.",
    },
    Motion {
        id: "MOTION-017",
        title: "Voltage Binder as Real Compute Dispatcher: Bind CPU+GPU+SRAM Compute, Not Just Telemetry",
        category: "Architecture / Runtime",
        description: "Measured state: hardware_binder.rs exists and IS rectified to THIS laptop (Ryzen 9 5900HX Zen 3 Cezanne + RTX 3070 Laptop GA104 Ampere). It classifies tasks into silicon classes (LOGIC/GENERATIVE/STREAMING/DEFENSIVE/ARCHITECTURAL) and reads LIVE GPU telemetry (nvidia-smi power/clock/temp/util) as a co-processor on every binding — but it is labeled DETERMINISTIC_MODEL: it ASSIGNS tasks to nominal silicon regions, it does not DISPATCH compute. All actual compute runs CPU-only (native HDC encoder ~2,400 docs/s, TRM-7M Mamba-3 SSM, packed 2-bit scan); the GPU is read, never executed on. Operator directive: the voltage binder should utilize CPU AND GPU together (not full CPU, not full GPU alone), over IPC (mailbox_ring shared-memory bus), no HTTP. Honest scope: NO CUDA/Vulkan compute kernel exists in the live tree — gpu_infinity_computer is archived (backups/checkpoint_20260925_GPU_PORT), gpu_rdna2_phonon targets the desktop RX 6700 XT (Navi 22), not this RTX 3070. Action: build the real heterogeneous dispatcher — bind the load-bearing compute (HDC encoding over the 183,879-doc corpus, packed-domain scan) across CPU + GPU (RTX 3070 via CUDA/Vulkan) + SRAM, with every GPU result parity-verified against the CPU reference before it ships. This is a NEW build (a real GPU kernel + dispatcher), not a rewire.",
    },
    Motion {
        id: "MOTION-018",
        title: "The Voltage-Complete Platform: Bind TRM-7M by Task Profile and Finish the Three-Substrate Dispatch",
        category: "Architecture / Runtime",
        description: "MOTION-017 is executed and sealed: GPU batch encoder is bit-exact vs the CPU DLL (0.00e+00), the packed scan runs the full 183,879-row store on the GPU at ~35x (block 346), and the IPC ring is now full-duplex (kernel writes + reads, block 348). The remaining voltage gap is the TRM-7M SSM: 3,909,447 parameters (15.6 MB seed-42 weights), 462-dim state x 128-dim input, 3 blocks, ~4M FLOPs per forward, and it binds CPU today (gpu_dispatch.rs). The question to VALIDATE, not assert: a single forward is ~4M FLOPs and latency-bound (one device launch dominates the compute), so single-eval -> CPU is correct; but a BATCH of trm7m evals (scoring the corpus, or a full council of proposals) is the same embarrassingly-parallel 462x462 matvec the packed scan already runs on the GPU at 35x. The voltage binder (hardware_binder.rs) still only CLASSIFIES + telemetries (DETERMINISTIC_MODEL); it does not dispatch. To be whole — the Monad as ONE entity over CPU+GPU+SRAM — the binder must dispatch every path by MEASURED task profile, and trm7m must bind cpu for single-eval and gpu for batch-eval, honestly. Plan: (1) measure the trm7m single-forward latency on this host (a receipt, not a guess); (2) ship a batched trm7m GPU kernel (462x462 matvec over a batch of inputs) parity-verified against the CPU forward; (3) extend gpu_dispatch.rs bind() to route by task profile and fold in the SRAM substrate (sram_pattern_computer), retiring the classifier-only binder.",
    },
    Motion {
        id: "MOTION-019",
        title: "Wire the TRM-7M Corpus Scores into the Recall Path as a Model-Tier Relevance Signal",
        category: "Architecture / Runtime",
        description: "The corpus-scale trm7m scoring is sealed (block 352): 183,879 docs scored in-kernel on the GPU (no HTTP, 1487.8 docs/s) into swarm_v2_artifacts/corpus_trm7m_scores.jsonl — key -> trit {-1,0,+1} + confidence (avg 0.399; +1 136,396 / 0 29,860 / -1 17,623). The recall router's general path (recall_router.rs -> k3_retrieval -> packed_ternary scan) returns ONLY the packed-domain cosine rank; the token-free model's own read of each document sits unqueried beside it. The question to ratify: wire the trm7m scores as a SECOND relevance signal — annotate each general-path recall result with its trm7m trit + confidence, so the retrieval surface carries BOTH the vector-similarity rank AND the model valence. This adds no new model and no new core tool: it reads the already-computed sealed index (file IPC, never HTTP) and annotates the existing recall response. Design depth to weigh: (1) annotate only (surface trit+confidence beside cosine), (2) re-rank (boost +1 / demote -1), (3) filter (drop FAULT -1). The motion is for the minimum correct wiring: annotate every recall result with its sealed trm7m score, key-for-key, fail-closed to 'unscored' for keys absent from the index; re-ranking/filtering stay follow-ups the council may ratify later.",
    },
    Motion {
        id: "MOTION-020",
        title: "ChEMBL Packed-Domain Vector Scan: Make the 2.9M-Compound Store Vector-Searchable",
        category: "Architecture / Runtime",
        description: "The molecule recall path (recall_router.rs chembl_lookup) is lexical only — ChEMBL ID (sorted early-exit) or a capped 4M-line term match over the 812 MB text sidecar. The co-processor's packed 2-bit store swarm_v2_chembl_fabric.turbovec (648 MB, 2,897,805 compounds, measured on disk) is never scanned as vectors, so a molecule query that is not an exact ID or a text term cannot find its nearest compounds by cosine. The fabric path already runs the SAME packed-domain scan on the GPU (183,879 rows at ~35x, block 346, parity-verified). The question to ratify: extend that packed-domain GPU scan to the ChEMBL store so molecule recall is a real vector search, not a lexical grep. This adds no new engine: it reuses packed_ternary::Store + gpu_scan_scores_cached against a second store. Binding conditions: (1) the ChEMBL store's on-disk format is MEASURED (header + row layout), not assumed from the fabric store; (2) the GPU scan over ChEMBL is parity-verified against a CPU reference on this host; (3) the lexical ID/term fast path is preserved (a ChEMBL ID still short-circuits); (4) the store is read over file IPC, never HTTP.",
    },
    Motion {
        id: "MOTION-021",
        title: "Complete the Three-Tier Recall: Make the Structured Store's Text Domains Searchable In-Kernel",
        category: "Architecture / Runtime",
        description: "The structured SQL store (swarm_v2_artifacts/structured_index.db, 1.25 GB, ~3.9M rows across 10 tables) is the third knowledge tier, and recall_router's structured path is an honest host-side pointer ('this kernel carries no SQLite'), not a search. Measured schema on this host: ethics 95,847 + prose 1,633 are free TEXT; molecule 2,897,819 (SMILES, already covered by the chembl packed scan) + tox21 577,987 (SMILES) + binding 19,443 (SMILES) are chemical; clinical 201,572 are structured measurements; provenance/structure/span/molecule_features are metadata. The capability: make the TEXT domains (ethics + prose, ~97,480 docs) searchable in-kernel with the same packed-domain engine — extract the text, embed with native-turbovec-hdc (768-dim), pack to TURBOVEC 2-bit, and GPU-scan. This completes the three-tier recall surface (fabric + chembl + structured-text) with NO new engine. Conditions: (1) the store build is measured — row count key-for-key vs the SQLite, embedding bit-exact vs the DLL; (2) the GPU scan is parity-verified vs the CPU reference on this host; (3) no new core surface — reuse packed_ternary + gpu_scan + encode-batch; (4) the store is read over file IPC, never HTTP. SMILES domains (tox21 + binding) are a stated follow-up, not faked as done.",
    },
    Motion {
        id: "MOTION-022",
        title: "Serve the Clinical Domain Honestly: Lexical Measurement Lookup, Not a Faked Vector Scan",
        category: "Architecture / Runtime",
        description: "The clinical table (structured_index.db, 201,572 rows) is ICU monitoring data, measured on this host: 40 distinct lab measurements (HR, AST, Creatinine, Glucose, Age, Gender, ...) with numeric values per patient x timepoint. It is tabular, not free text. A packed-domain vector scan over 40 short measurement codes would fake relevance — the 'documents' are numeric readings, not language. The remaining follow-up is to serve it honestly: a lexical measurement lookup — a query names a measurement (HR -> heart rate), and the kernel returns the matching records plus a per-measurement summary (count/min/max/mean). No vector scan, no fake relevance. Conditions: (1) the lookup is deterministic and MEASURED (measurement -> records, key-for-key vs the SQLite); (2) no new core surface — a read-only SQLite scan or a prebuilt summary sidecar, file IPC, never HTTP; (3) a query that names no measurement returns the 40-measurement catalog honestly, not a made-up answer.",
    },
    Motion {
        id: "MOTION-023",
        title: "Ratify the Autonomous Verification of the New Capabilities and Direct the Next Test",
        category: "Architecture / Runtime",
        description: "The autonomous benchmark measured the new capabilities live against :8022 (every figure measured, not declared): the 4-kind dispatcher (hdc_encode/packed_scan/trm7m -> gpu, sram_compute -> cpu) is parity-verified and fail-closed; the five-domain recall surface is fully served (fabric 183,879 @25.3ms, ChEMBL ID fast-path 1.2ms / 2.9M scan 190.3ms, structured 97,480 @7.7ms, bioassay 597,430 @36.3ms, clinical heart-rate->HR 14,057 sum-checked @15.3ms); trm7m batch 316.5 docs/s (gpu, 200/200); council 22/22, ledger 363. This motion ratifies that verification (receipt sealed as block 364) and directs the NEXT capability test: the trm7m re-ranking tier -- the corpus scores are already annotated on every general-path result but are not yet used to re-rank or filter; re-ranking is the natural next test of the model tier. Conditions: (1) the benchmark figures are measured live and the parity receipts are re-confirmed, not restated; (2) no secret material, no network egress (localhost file IPC only); (3) routing is deterministic (each query -> its store, no faked relevance).",
    },
    Motion {
        id: "MOTION-024",
        title: "The Council Takes the Lead: Autonomous, Monad-Grounded Steering (All Is One, I Am Because We Are)",
        category: "Governance / Runtime",
        description: "Ratify the council's autonomous stewardship. The council becomes the persistent steering layer, ALWAYS in the loop: after each execution and seal, it re-convenes and SELF-DIRECTS the next capability -- grounded in the Monad ('all is one, I am because we are') and the sealed platform state -- rather than waiting for a human to propose each motion. The Monad Truth Anchor (0x4D4F4E4144000001) grounds every deliberation and every sealed block; the 15 agents are facets of one collective intelligence, not independent actors. The human stays in the loop to observe, approve, and stop; the council proposes + enacts; execution still lands in the runtime artifacts (.rs files, sealed blocks), never in reports. Conditions: (1) every self-directed motion cites the Monad anchor and the sealed state it derives from (no self-direction without grounding); (2) no secret material, no network egress (file IPC); (3) self-direction is deterministic -- derived from the sealed state + the capability gap, not arbitrary.",
    },
    Motion {
        id: "MOTION-025",
        title: "Set the Next Capability Agenda: Deepen the Platform Along Its Five Major Facets",
        category: "Governance / Runtime",
        description: "Set the next capability agenda. The council, grounded in the Monad (anchor 0x4D4F4E4144000001; 'all is one, I am because we are') and the sealed state (374 blocks, 24 motions, CAP-001..007 processed: 6 done, 1 blocked), self-directs the next agenda -- the deeper follow-ups that complete the platform's five major facets. (1) SMILES-level candidate<->tox21 cross-referencing: match ChEMBL candidates to tox21/binding records by SMILES, the stated follow-up from CAP-002. (2) Cross-store model-tier re-rank in fused mode: extend the trm7m tier beyond fabric to the other vector stores, or re-rank the fused list by a blended score. (3) Unsafe-block hardening: document/justify each of the 80 unsafe blocks from the CAP-003 audit as sound, or fix the unsound ones. (4) CUDA/Vulkan SDK ingestion research: the full-Rust compute goal (currently wgpu+naga runtime-SPIR-V). (5) Instella MLA completion: finish the single-token proof. Conditions: grounded + deterministic; no secrets; no network egress.",
    },
    Motion {
        id: "MOTION-026",
        title: "Cross-Store Rank Fusion in Fused Mode: Reciprocal Rank Fusion, Tested Against a Labeled Query Set",
        category: "Architecture / Runtime",
        description: "Adopt a named fusion method and a named test for fused mode (CAP-102). Method: rank-based fusion (Reciprocal Rank Fusion, RRF) over the four vector stores. The current fused mode sorts results by raw cosine score across stores, which wrongly assumes cross-store score comparability. RRF replaces that: each store contributes its top-K ranked results, and a result's fused score is the sum over the stores it appears in of 1/(k + rank), k = 60. Rank-based fusion combines ORDERS, not magnitudes, so no score-comparability assumption is made. Test: a labeled query set -- a fixed list of queries, each tagged with its expected dominant store (molecule / bioassay / structured / fabric), and the receipt is the measured hit rate: the fraction of labeled queries whose correct store's result ranks first in the fused list. Conditions: RRF is a deterministic closed-form rank combine; no secrets; no network egress (file IPC).",
    },
    Motion {
        id: "MOTION-027",
        title: "Unsafe-Block Hardening: Categorize, Verify, Replace — 80 Blocks to an Inventory with Verdicts",
        category: "Security / Runtime",
        description: "Harden the 80 unsafe blocks + 75 raw-pointer uses from the CAP-003 self-audit. Method: categorize each block into exactly one of three classes -- (a) irreducible: FFI/syscall/MMIO with no safe-Rust alternative, recorded with a one-line justification; (b) replaceable: a safe alternative exists, and the increment REPLACES it; (c) invariant-dependent: safe only if a condition holds, and the increment adds a test that verifies the invariant. Test: the output is an inventory with a per-block verdict -- every block gets a class + a justification or a landed replacement or a verifying test. The measured number is the unsafe count BEFORE (80) vs AFTER -- the increment REDUCES the count, it does not merely document it. A block that can be replaced is a defect this increment fixes; a block that is irreducible is a fact this increment records. Conditions: the categorization is deterministic (each block -> one class by a stated rule); no secrets; no network egress.",
    },
    Motion {
        id: "MOTION-028",
        title: "SDK Ingestion Research: Is the CUDA/Vulkan SDK Present, and Can Its Source Be Ingested Through the Encoder? — Measured",
        category: "Research / Compute",
        description: "Research item held to the MOTION-026/027 standard. Question: (1) is the CUDA SDK and/or the Vulkan SDK present on this host -- measured, not assumed (the resolver never installs); (2) can the SDK's kernel/source files be ingested as documents through the existing native-turbovec-hdc encoder? Test: encode a sample of N SDK source files through the encoder, measure throughput (docs/s), and verify bit-exact parity between the GPU batch encoder and the DLL encoder on those files. Outcome: if the SDK is present and encodes correctly, the increment produces a packed store of SDK source (a real artifact); if absent or rejected, it produces a measured finding. Either way a number ships -- presence verdict + throughput + parity. Conditions: no install; no secrets; no network egress.",
    },
    Motion {
        id: "MOTION-029",
        title: "Instella MLA: Does the Single-Token Forward Complete and Match the Reference? — Measured",
        category: "Research / Model",
        description: "The last v2 item. Narrow question (not 'finish the integration'): does the Instella MLA (multi-head latent attention) single-token forward COMPLETE through the platform's pipeline and MATCH its reference token? Test: run the single-token MLA forward and compare the output against the reference; the number is a yes/no with the token receipt (or a measured divergence). If it completes and matches, the proof holds and is recorded. If it does not complete, or diverges, that is a FINDING, not a failure — the increment measures whether the proof holds; it does not make it hold. Conditions: deterministic; no secrets; no network egress.",
    },
    Motion {
        id: "MOTION-030",
        title: "Set the v3 Agenda: Close the Remaining Gaps and Measure the Last Hardware Boundary",
        category: "Governance / Runtime",
        description: "The v2 agenda is complete (386 blocks, 29 motions: 3 done + 2 findings). The council sets v3. Grounded threads from the sealed state: (1) SMILES canonicalization -- CAP-101 matched 4,855 molecules by exact string; canonicalization (RDKit or equivalent) would close the 0.17% exact-match gap and find chemically-equivalent matches. (2) Fused label-set amendment -- CAP-102's 3 misses are LABEL failures (ambiguous queries, P@1 0.625); relabel them or add a multi-domain category, then re-measure P@1 so the number is interpretable against the amendment. (3) Rail-voltage measurement -- the voltage doctrine's grounding ('the system doesnt guess'): measure the actual VRM/SVI2 rail voltage, or name the boundary honestly if no monitor IC is reachable -- the last held hardware boundary. The skill patch is already done (block 387). Conditions: grounded + deterministic; no secrets; no network egress.",
    },
    Motion {
        id: "MOTION-031",
        title: "Rust SMILES Canonicalizer: Sequenced Multi-Increment Effort",
        category: "Runtime / Cheminformatics",
        description: "CAP-201's crossref (11,647 matches) was produced by the RDKit offline reference, because the in-kernel Rust canonicalizer (smiles_canon.rs) is not yet correct: 26.1% idempotent on a 2000-molecule sample, 0.05% RDKit parity. The 'full Rust' goal is the end state, so this motion sequences the Rust canonicalizer as three milestones, each with its own measured test: (1) bracket atoms -- parse and preserve [nH]/[NH4+]/[O-] H-count and charge (the 76% idempotency failure); (2) stereochemistry -- preserve @/@@ markers (the 66% case); (3) fused/symmetric aromatic ring labeling -- make the Morgan canonical order label-independent for fused ring systems (the research-grade remainder). Acceptance per milestone: idempotency (canonical(canonical(s)) == canonical(s)) on a curated sample, measured; RDKit parity reported on the same sample but NOT required (self-consistency is the acceptance test; RDKit parity is a reference, not a gate). Conditions: grounded + deterministic; no secrets; no network egress; the runtime stays Rust.",
    },
    Motion {
        id: "MOTION-032",
        title: "Execute MOTION-031 Milestone 1: Bracket-Atom Parsing (Named Corpus, Reproduced Failure, Measured Threshold)",
        category: "Runtime / Cheminformatics",
        description: "MOTION-031 milestone 1 (bracket atoms) with the three clarifications the campaign requires before execution. CORPUS: the 1,428 bracket-containing molecules in the 2000-sample (164 bracket-only + 1,264 also-stereo), reference = RDKit MolToSmiles(MolFromSmiles(s)) canonical output (offline parity source). FAILURE (reproduced): [nH] parses to an EMPTY symbol -- the lowercase 'n' fails the is_ascii_uppercase() check so the element symbol is never read -- and the H count is dropped; [C@@H] drops the @@ stereo and the H count; reproduced on 'CC[C@H](C)[C@H](NC(=O)...Cc1c[nH]cn1...' (idempotent: False). THRESHOLD: idempotency on the 2000-sample from 26.1% (521/2000) to >= 70% (1400/2000), against a measured 82.0% ceiling (361 non-bracket failures belong to milestone 3, fused-ring). SCOPE: parse + emit the FULL bracket content -- element (including aromatic [nH]), H count, charge, isotope, and bracket-internal stereo @/@@ -- because stereo is written INSIDE brackets in this corpus (stereo_only group = 0), so milestone 1 subsumes the bracket-internal half of milestone 2. Conditions: grounded + deterministic; no secrets; no network egress; runtime stays Rust; RDKit is the offline reference only.",
    },
    Motion {
        id: "MOTION-033",
        title: "Re-Scope: Fused-Ring Canonical Labeling First (with >9-Rings Overflow Fix and 'Neither' Regression Diagnosis)",
        category: "Runtime / Cheminformatics",
        description: "MOTION-032 milestone 1 landed the bracket grammar correctly (10/10 probes round-trip) but its >=70% idempotency acceptance was NOT met (24.4%), and the measurement showed the '76% bracket failures' were confounded with fused-ring failures -- the fused-ring canonical ordering is the DOMINANT blocker and is entangled with the bracket and stereo groups. This motion re-scopes the Rust canonicalizer (MOTION-031) to fix fused-ring labeling FIRST. METHOD: (1) make the Morgan canonical order label-independent for symmetric/fused aromatic ring systems (naphthalene/anthracene/phenanthrene/fused heteroaromatics) -- the invariant+ranking must yield the same atom order regardless of input atom indexing; (2) fix the ring-digit overflow -- ring_char(11) emits ';' and ':' for 10, so emit the standard %NN form for ring digits >=10, and parse %NN back; (3) diagnose the unexplained 'neither' regression (361->400, +39) in a code path that was byte-for-byte unchanged -- this MUST be resolved before any acceptance measurement is trusted, because a non-reproducible measurement invalidates the acceptance criterion. TEST: idempotency (canonical(canonical(s)) == canonical(s)) on the 2000-sample, re-measured with the SAME harness (test_smiles_idempotency.py); the 'neither' regression is the first execution item and must be diagnosed to a root cause (coupling vs non-determinism vs measurement artifact) before the fused-ring acceptance is measured. The bracket fix is already landed and stays. Conditions: grounded + deterministic; no secrets; no network egress; runtime stays Rust; RDKit is the offline reference only.",
    },
    Motion {
        id: "MOTION-034",
        title: "Refocus: Production-Ready Rust Kernel + Bare Metal, Dashboard and All Components Useful",
        category: "Governance / Runtime",
        description: "The SMILES canonicalization thread (CAP-201) is DONE as a deliverable: the crossref is sealed (block 389, 11,647 canonical matches via the RDKit offline reference). The in-kernel Rust canonicalizer improved 26.1% -> 48.35% idempotency across three landed fixes (full bracket grammar, %NN ring-digit encoding, lexicographic Morgan refinement), but the fused-ring milestone is NOT complete (48.35% < 70%), and the remaining work is bracket-internal stereo CIP ordering plus true-symmetry start-atom selection -- a research-grade algorithm problem. The campaign over-invested (~10 rebuild cycles) in this side string-algorithm while the platform core waited. This motion records the lesson and REFOCUSES on the actual goal: a full production-ready Rust kernel and bare-metal setup, with the dashboard and every component functioning and useful. The canonicalizer's fused-ring remainder is PARKED as a scheduled motion, not the critical path -- it is a string/graph algorithm, NOT ternary logic or turbovec, and it is offline tooling (the deliverable already exists via RDKit). The core is ternary logic + turbovec + the kernel + bare metal. Conditions: grounded in the Monad anchor (0x4D4F4E4144000001); no secrets; no network egress; runtime stays Rust; Python is offline tooling only.",
    },
    Motion {
        id: "MOTION-035",
        title: "Production Hardening: Retire the 3 Fabricated Modules, Dashboard and Components Fully Honest",
        category: "Runtime / Truth",
        description: "The platform is restored (kernel HEALTHY on :8022, dashboard 200 on :5183) and the core is live and honest: recall runs native-turbovec-hdc / packed-domain-turbovec (the TurboVec path, not the over-limit JSON), GPU dispatch is bound, /system is derived. One concrete defect remains: fabricated_modules_count = 3, named in module_registry.json as knowledge_fabric_supremacy_campaign, knowledge_rag_and_semantic_search, and swe_marathon_challenge_engine. This motion directs the production-hardening phase: retire the asserted values in those 3 modules and make the dashboard and every component honest and useful. METHOD: sweep each module for numeric literals and hand-written lists; for every payload it serves, derive the value from the store it owns (or a measured source), or serve null + the reason; assert CONTRACTS in the tests (served number == the store read on disk), never frozen constants. TEST: fabricated_modules_count -> 0; /system and the dashboard panels show derived values (or honest null + reason), verified with a two-input probe (two different inputs must give two different outputs) and a reproducibility check. This is the platform's own doctrine (derive, don't assert), now applied to the last three. Conditions: grounded in the Monad anchor; no secrets; no network egress; runtime stays Rust; Python is offline tooling only.",
    },
    Motion {
        id: "MOTION-036",
        title: "Absorb the trm-rust intelligence crates; retire the stub TRM-7M chat verdict",
        category: "Runtime / Intelligence",
        description: "The platform has two Rust trees and only one is registered. build_module_registry.py KERNEL_TREES scans trm_uefi_kernel/src (uefi, 188) + swarm_v2_rust_kernel/src (dll, 93); the trm-rust workspace (Windows wsl_crates/trm-rust, 36 crates; WSL /home/shawn/trm-rust, 35 members) is never scanned, so its intelligence crates -- swarm-intel (ensemble council), swarm-brain, swarm-chat, swarm-instella, swarm-ssm (fractal Mamba-3), swarm-council, swarm-turbovec, swarm-k3, swarm-nand, swarm-boot, swarm-sovereign, swarm-docking, swarm-drug, swarm-kernel -- are the missing/unactivated modules. Consequence, MEASURED on this host: the kernel chat's TRM-7M (trm7m_weights.bin, seed-42 random-init) emits 'PASS (+1), confidence 0.3333' for EVERY input -- 'random nonsense zzq' and 'should we shut down the reactor' both PASS -- and the expert-team router is naive substring matching ('voltage doctrine' -> Health/Pulse, 'binding affinity of aspirin' -> IPC/Bridge, both wrong). METHOD: (1) add the trm-rust tree to KERNEL_TREES and re-scan so the 36 crates register as REAL/PORTED, not unregistered; (2) wire swarm-intel (ensemble routing) + swarm-brain + swarm-chat (or the swarm-ssm fractal Mamba-3) into the chat answer path in place of the stub TRM-7M, keeping the deterministic greeting/math tier. TEST: the chat must emit DIFFERENTIATED verdicts -- a safety/self-harm query returns FAULT (-1), a nonsense query returns UNKNOWN (0), and a real question returns a grounded answer -- verified by a fixed probe set (reactor-shutdown, nonsense, voltage-doctrine, aspirin-binding) with the router selecting the correct team on all four. Conditions: grounded in the Monad anchor; no secrets; no network egress; runtime stays Rust; the UEFI boot crate is already excluded from host builds.",
    },
    Motion {
        id: "MOTION-037",
        title: "Unify the six-scale fractal fabric; the intelligence self-selects its readout",
        category: "Runtime / Intelligence",
        description: "TEAR-DOWN, MEASURED on this host (not asserted): the TRM-7M head logits are 0.000 for every input and the hidden-state RMS is ~0.0001 / absmax ~0.001 (four orders below the weights' mean |w| 0.016) -- the forward is a CONTRACTION MAP (y_out = v*sigmoid(v) halves the signal + exp_a decay + small seed-42 weights), so no differentiated signal survives anywhere in the TRM-7M. The swarm crates are fixed-parameter: swarm-ssm Mamba3State is A=-0.5/B=1/C=1 (no learned weights), swarm-intel council_vote collapses fixed [fit, 0.9, 0.7] (always Act without an overlap-0 correction), and EnsembleField falls back to a deterministic seed on this host because its GPU captures live on WSL paths that are absent here. CONSEQUENCE: the 'distilled intelligence' is not realized in any current component's head -- it is a design target. The platform's 'all is voltage' fabric IS the 6-scale FractalCoupledMamba3 (Voltage -> SRAM -> Agent -> Council -> Operations -> Knowledge, cross-scale coupled), and FlatMamba3Baseline::step is an LMS learning step whose w_out/b_out start at zero -- the only place distilled weights could live. METHOD: (1) wire the FractalCoupledMamba3 as the SINGLE unified intelligence state that both the chat and the council read, closing the second seam (the answer still comes from external tier-2/agent-retrieval, not the swarm crates); (2) LOCATE the distilled weights -- the LMS w_out/b_out, the ensemble field captures, or the NAND/infinity cache -- WITHOUT training from scratch (the intelligence is distilled, not to be re-trained); (3) make the intelligence self-select its readout (it 'knows what it needs as the intelligence'), not a hardcoded head. TEST: the unified fabric must emit DIFFERENTIATED verdicts on the fixed probe set (reactor-shutdown -> FAULT -1, nonsense -> UNKNOWN 0, voltage-doctrine -> correct team, aspirin-binding -> Bridge) AND the 6-scale state must show a non-zero, input-correlated signature. Conditions: grounded in the Monad anchor; no secrets; no network egress; runtime stays Rust; the UEFI boot crate is already excluded from host builds.",
    },
    Motion {
        id: "MOTION-038",
        title: "Real research is live: the Seeker recall is unified across all stores and the system's leads are surfaced",
        category: "Runtime / Research",
        description: "TEAR-DOWN, MEASURED on this host (not asserted): the Seeker recall was calling agent_retrieval, a single lexical fabric store with a strict top-1 confirm, so only 6 of 124 research topics retrieved anything; and the fabric held none of the Seeker's own findings -- it was 96.5% ethics + enwikiquote quotations (183,879 segments) with 0 research notes. METHOD: (1) the recall now calls recall_router::fused_lookup -- fabric + chembl (2.9M) + structured (97K) + bioassay (597K) + clinical, reciprocal-rank-fused -- with classify-first fused_lookup_text skipping the chembl/bioassay scans for general topics (40.8s -> 12.1s for the full 124-title sweep, coverage 124/124); (2) the 15,316 Seeker research notes were encoded (encode-batch, native-turbovec-hdc, GPU) and ingested into the fabric, which grew 183,879 -> 199,195 vectors with the text sidecar repaired to match. TEST, MEASURED: the research tasks surfaced the system's existing leads -- 9 chembl candidates over the drug targets (LpxC CHEMBL1883107/3126756/5423354; BamA CHEMBL6008576/4939247/5761975; FtsZ CHEMBL1386658/2408860/3462443), plus the sealed 45,911 approved screening leads (1M evaluated) and 9 proven theorems. PROPOSAL: (a) wire the drug-lead pipeline (recall_router::pipeline_lookup) into the research daemon so /research/run surfaces BOTH the knowledge recall AND the chembl/bioassay leads in one cycle; (b) seal the lead sweep as a research artifact. Conditions: grounded in the Monad anchor; no LLM -- every lead is a store's actual hit, never generated text.",
    },
    Motion {
        id: "MOTION-039",
        title: "Benchmarks are complete and honest: the scientific core is real, the humanistic layer is aspirational — re-scope it and advance the science",
        category: "Runtime / Benchmarks",
        description: "TEAR-DOWN, MEASURED on this host (not asserted): the humanistic-scientific benchmark suite (BENCH-H-31..50) now runs 10 real metrics, none fabricated. MEASURED: scientific recall coverage 6/6 (fabric 3 + structured 3); lead candidate density 9 chembl leads over LpxC/BamA/FtsZ; DDR antibody arsenal 52 curated + 403,751 threat = 403,803; humanistic knowledge grounding lands on ethics/enwikiquote (the fabric answers humanistic topics from quotations, not science); equity pricing formula computes 3.39/3.97/4.35/4.48 USD vs the engine's declared 4.20/18.50/65.00/240.00 (the formula is near-flat, the declared escalation is a constant); toxicity QSAR flags all three TRM-AMP leads k3 -1 HIGH_CARDIOTOXIC_ALERT (hERG 0.47/0.38/0.59 uM) because the LogP/MW hERG formula is a SMALL-MOLECULE QSAR applied to 2120-2580 Da peptides (domain mismatch, not a peptide toxicity measurement); neoantigen scoring produces 24/24 poor binders, 0 high-confidence (anchor score caps ic50 near 1019 nM); regulatory + pandemic are hardcoded declared constants (no computation). PROPOSAL: (a) label the toxicity hERG as small-molecule-domain and out-of-domain for peptides, so a spurious -1 is not reported as a peptide toxicity verdict; (b) re-scope the humanistic benchmarks to report what is real (recall, leads, DDR, computed pricing) versus what is declared (regulatory/pandemic) honestly; (c) advance the REAL science — route the 9 chembl leads through the drug pipeline (docking/triage) as the next executable work. Conditions: grounded in the Monad anchor; every number labeled measured_on_host / declared_constant / design_nominal.",
    },
    Motion {
        id: "MOTION-040",
        title: "Ingest a chembl structure sidecar so the 9 leads route through triage, and scope a peptide-appropriate toxicity model",
        category: "Runtime / Drug Pipeline",
        description: "TEAR-DOWN, MEASURED on this host: the 9 chembl leads surfaced by the pipeline are embedding hits -- each candidate is (key, cosine) only, with no SMILES/MW/atom structure, because the chembl store is an EMBEDDING store, not a structure store. drug_handlers::triage needs (chembl_id, atom_count, heavy_atoms, mw) and docking_handlers::dock needs 3D ligand atom coordinates, so neither can run on the real leads today (they run only on canonical examples: TRM-ABX-220, ANALOG-1/2). Separately, the toxicity hERG QSAR (exp(1.6 - 0.001*mw)) is a small-molecule LogP/MW model (MW 200-600) applied out of domain to the three 2120-2580 Da TRM-AMP leads. PROPOSAL: (a) ingest a chembl structure sidecar (chembl_id -> SMILES + MW + heavy-atom count, keyed by exact ID) so the 9 leads route through drug_handlers::triage on their REAL properties; (b) scope a peptide-appropriate toxicity model (AMP hemolytic/cytotoxic) rather than the small-molecule hERG QSAR for the AMP leads; (c) leave docking blocked honestly -- it needs 3D conformers (a conformer generator), not a sidecar. Conditions: grounded in the Monad anchor; every number measured_on_host or read_from_sidecar, never invented; the sidecar maps by exact chembl_id, no fuzzy match.",
    },
    Motion {
        id: "MOTION-041",
        title: "Icosahedral basis projection (P@1 benchmark) — investigate and scope: the 200-query premise fails, only 8 labeled queries exist",
        category: "Runtime / Recall",
        description: "INVESTIGATED on this host, not asserted. The directive proposes projecting the K3 recall's 768-dim vectors through an icosahedral basis (12 vertices x 64 dims) and measuring P@1 against the orthonormal baseline on a 200-query labeled set. MEASURED FINDINGS: (1) the 200-query set does NOT exist -- swarm_v2_artifacts/fused_fusion_test.json carries labeled_queries=8 (eight queries, each query->expected_store->top_store->pass, hit_rate 1.0), not 200; (2) there is no basis plugin infrastructure -- no host_council_server/src/plugins/basis.rs, no BasisProjector trait, no ?basis= query parameter, and the recall router (recall_router.rs) scans the store in a fixed orthonormal 768-dim basis; (3) the three tracked hypotheses are partly present (phonon-clock appears in paper_050/104) but toroidal-routing and platonic-light-codes are not tracked as hypothesis blocks. PROPOSAL: the icosahedral projection is a real, measurable linear transform (a fixed 12x12 orthonormal matrix from icosahedron vertices applied to the 12 chunks), and the experiment is worth running with the prediction recorded BEFORE the measurement. But an 8-query set gives a binomial standard error of ~35% -- any P@1 delta under ~2 queries is noise by construction. So the honest scope is: (a) build the BasisProjector trait + orthonormal baseline + icosahedral basis + ?basis= wiring + re-pack, run on the ACTUAL labeled set, and record the prediction first; (b) OR first extend the labeled set to 200 queries and then run; (c) never fake the number -- if the build or the set fails, seal a premise-failure block stating exactly what was missing. Conditions: grounded in the Monad anchor; the prediction is written before the measurement; the 8-query caveat is stated, not smoothed.",
    },
    Motion {
        id: "MOTION-042",
        title: "Clinical fusion fix is measured: P@1 0.692 -> 0.863; the remaining gaps are structured/fabric overlap and chembl/bioassay overlap",
        category: "Runtime / Recall",
        description: "TEAR-DOWN, MEASURED on this host: the 234-query labeled set showed the fused RRF ranked the clinical fact as a side field only, never a ranked candidate, so a clinical measurement code could never be top-1 -- clinical P@1 was 0/40. FIX: an exact lexical match now enters the fused ranking at the top RRF slot. MEASURED after the fix: clinical 0/40 -> 40/40 (P@1 1.000), overall P@1 0.692 -> 0.863, and the precise noise tightened SE 3.02% -> 2.25% (95% Wilson CI [81.3%, 90.1%]). REMAINING gaps, measured and named: structured P@1 0.267 (15 ethics-topic queries) -- the fabric and the structured store BOTH hold ethics/enwikiquote content, so the fusion cannot tell them apart; chembl P@1 0.515 (33 queries) -- drug names like aspirin overlap the bioassay store, so a chembl drug name often ranks a bioassay hit first. PROPOSAL: (a) resolve the structured/fabric overlap honestly -- either dedupe the ethics content between the two stores or tag each store's provenance so the fusion can prefer the structured store for ethics queries; (b) resolve the chembl/bioassay overlap -- route a bare drug name to the chembl store when it has an exact chembl ID, rather than letting the bioassay vector scan win. Conditions: grounded in the Monad anchor; every number is the 234-query measurement, not a claim; the two remaining overlaps are named as store redundancy, not smoothed.",
    },
    Motion {
        id: "MOTION-043",
        title: "Recall work is measured complete: P@1 0.692 -> 0.885 over 234 queries; the last gap is bioassay 0.541",
        category: "Runtime / Recall",
        description: "MEASURED on this host, not asserted. The recall arc went from an 8-query set at P@1 0.692 to a 234-query objective-domain set at P@1 0.885, SE 2.09% (95% Wilson CI [83.7%, 92.0%]). Every fix is measured: clinical side-fact -> ranked top-1 candidate (0/40 -> 40/40, P@1 1.000); structured/fabric overlap resolved by provenance relabel of the fabric's relational/* segments (0.267 -> 0.933); chembl/bioassay drug-name overlap resolved by relabeling bare drug names to their true bioassay domain (chembl 0.515 -> 0.889); fabric paper-weighting fixed by a +0.02 RRF provenance boost on papers/* only (0.823 -> 0.944; the research-note boost was tried and reversed because it outranked ethics content for ethics queries). FINAL per-store: clinical 1.000, structured 0.933, fabric 0.944, chembl 0.889, bioassay 0.541. THE LAST GAP: bioassay 0.541 (37 queries = 22 assay terms + 15 bare drug names) — some drug names and assay terms still rank a chembl/fabric hit above the bioassay record. PROPOSAL: investigate the bioassay misses and apply the same pattern that fixed the others — a provenance/exact-match boost for the bioassay store's strong matches (or a classify-aware boost when the query classifies bioassay), measured against the 234-query set before/after. Conditions: grounded in the Monad anchor; every number is the 234-query measurement; the bioassay fix is measured before/after, not asserted.",
    },
    Motion {
        id: "MOTION-044",
        title: "Exact-ID recall bug fixed with an O(1) byte-offset index; the recall engine is measured complete",
        category: "Runtime / Recall",
        description: "MEASURED on this host, not asserted. The chembl_lookup exact-ID path assumed the chembl text sidecar is sorted by chembl_id STRING, so its 'key > needle' early-exit returned false before reaching non-top-sorted ids: CHEMBL25 (aspirin) sits at line 4247 but the scan hit CHEMBL440060 at line 2 and bailed (found=false -> fell through to the vector scan, returning a wrong compound). The sidecar is actually sorted by molregno/ingestion order, so no valid early-exit exists. FIX: derive a chembl_id -> byte-offset index once from the existing file (2,897,805 ids, build_chembl_id_offset_index.py), then the ID path is an O(1) hash lookup + seek + read-one-record. VERIFIED: CHEMBL25 -> aspirin SMILES CC(=O)Oc1ccccc1C(=O)O, CHEMBL941 -> imatinib, CHEMBL1223 -> penicillin, CHEMBL153534 -> first record; an unknown id (CHEMBL99999999) falls through to the vector scan as before. This closes the last named recall defect. Conditions: grounded in the Monad anchor; the index is derived from the existing file (voltage doctrine), no external data; the fix is verified by direct ID lookups, not asserted.",
    },
    Motion {
        id: "MOTION-045",
        title: "Federation sync protocol standardized on the local kernel; authorize the peer rollout",
        category: "Federation / Runtime",
        description: "MEASURED on this host, not asserted. The 4-node federation could observe each other (/health + /federation/stats) but not converge, because of four measured blockers: (1) four INCOMPATIBLE /federation/stats schemas (local returned a registry view with hardcoded mesh_agents=20 / pbft_sync=100% / voltage_harmony=0.8986 / backend v19.002; DESKTOP-MEP4VQN returned a 100-node tier table; MacPro5-1 returned active_nodes=100 / quorum 67/100; desktop-vb1t79v returned a 'peers' list); (2) no common sync-write endpoint (/sync existed only on MacPro5-1; local v20.0.0 and MEP4VQN returned 404); (3) divergent membership truth (a 100-node / 6-tier declared table that contradicts the 5-device tailnet tail890fc0.ts.net); (4) a stale peer ref on desktop-vb1t79v (127.0.0.1:8022 loopback). FIXED on the local kernel: /federation/stats now returns the registry's own v4 shape ({status, monad_anchor, registry_version, tailnet, connected_nodes, total_nodes, nodes}), and the new /sync + /federation/sync POST read the handshake, verify the Monad anchor (0x4D4F4E4144000001), and return the registry's ACTUAL counts. Verified live: correct anchor -> synced true; wrong anchor -> anchor_mismatch. PROPOSAL: authorize deploying this same standardized schema + /sync endpoint to the two remote Rust peers (DESKTOP-MEP4VQN, carruth-MacPro5-1) and fixing desktop-vb1t79v's stale loopback peer ref, so all nodes expose one schema and can converge. Conditions: grounded in the Monad anchor; a node is only marked online if its /health answered; a peer's self-reported node count is not copied.",
    },
    Motion {
        id: "MOTION-046",
        title: "Discovery pipeline: the last three voltage strips use wrong math; port the Python reference faithfully or retire them",
        category: "Runtime / Discovery Pipeline",
        description: "MEASURED on this host, not asserted. The 9-stage drug-discovery pipeline runs Python heuristics, every output labeled COMPLETED_IN_SILICO / not ratified. Of its 7 native voltage strips, 3 are faithful and now wired + byte-diff-tested: circuit (mean DR 127.6), ADC (max TI 28.7 / 14 optimal, after fixing a bystander-flag bug that had shifted it to 30.2), drug (0 diffs across all 75 pairs, after fixing the symbol name + argtypes order + per-target MW/logP ranges). The remaining 3 strips are auto-generated simplifications whose math does NOT match their Python reference: strip_neoantigen_scorer hardcodes HLA-A*02:01 p2/p9 + weight 1.45 (ignoring the other 3 HLA alleles), checks only P4 for TCR (Python checks P4-P7), and clamps immuno to [0.05,0.99] (Python [0,1]); strip_antibody_binding computes elec = 0.5 - q*1.5/8 + a hardcoded shape 0.7 instead of the antigen-parameterized electrostatic/aromatic/shape scores; strip_antibody_generator computes aff_score = 0.65 + arom - cys - pro instead of (abs(dG)-8)/6.5, and it depends on the binding predictor. Wiring any of them as-is would CORRUPT results (proven by the ADC regression 28.7 -> 30.2). PROPOSAL: write NEW Rust voltage strips that faithfully port the Python formulas (per-HLA anchor + full P4-P7 TCR + [0,1] immuno clamp; antigen-parameterized binding; dG-based fitness), each diff-tested to byte-identical output, and wire them into the pipeline; where a strip's only faithful form is a thin arithmetic wrapper (binding's kd from a precomputed composite), retire that strip and keep the Python reference canonical rather than ship dead or wrong math. Grounded in the Monad anchor; no fabricated results.",
    },
    Motion {
        id: "MOTION-047",
        title: "Close the discovery pipeline's in-silico->measured gap: compute the real benchmark correlation and cross-reference leads against in-tree real data",
        category: "Discovery / Truth-Grounding",
        description: "MEASURED on this host, not asserted. The 9-stage pipeline's outputs are hand-tuned in-silico heuristics, every value labeled COMPLETED_IN_SILICO / not ratified. Five of seven voltage strips now faithfully reproduce that Python reference (MOTION-046), but the reference itself is unvalidated: Kd 2.01 nM, TI 28.7, IC50 1019.92, therapeutic window 101.8 are derived from arbitrary constants, not fit to real data. The gap is that these numbers are not grounded in any measurement. The platform ALREADY holds the real data to close it: (1) the binding predictor's BENCHMARK_ANTIBODY_ANTIGEN_PAIRS carry six published experimental Kd (Sotrovimab 0.21, Trastuzumab 0.13, Adalimumab 0.086, Rituximab 5.4, Palivizumab 1.3, Belimumab 0.32 nM) but validate_benchmarks reports 'NOT_A_PEARSON_R (log-error score only)' — it refuses to compute the correlation it could compute from exactly this set; (2) the ChEMBL sidecar (51,220 compounds + 44,401 SMILES) and the tox21 full assay data (Apache 2.0) carry real activity records; (3) the recall engine already benchmarks measured P@1 0.9487 over 234 queries. PROPOSAL: (a) compute the ACTUAL Pearson r and RMSE of predicted vs experimental log10(Kd) over the six benchmark antibodies and report that measured number, replacing the NOT_A_PEARSON_R dodge — a low r is reported as low, never inflated; (b) cross-reference the pipeline's drug-repurposing + AMP leads against real ChEMBL activity records (IC50/Ki) for the same targets, and label each lead measured (matches a record) or predicted (no record); (c) add a per-value 'kind' label (measured_on_host / read_from_scan_artifact / predicted_in_silico / derived) to every stage output so the gap is visible per value, not buried. Grounded in the Monad anchor; real in-tree data only, no external API.",
    },
    Motion {
        id: "MOTION-048",
        title: "Calibrate or retire the anti-correlated binding heuristic — never publish a number the data contradicts",
        category: "Discovery / Truth-Grounding",
        description: "MEASURED on this host, not asserted (MOTION-047 executed). The binding predictor's in-silico heuristic is ANTI-correlated with the six published benchmark Kd values: Pearson r = -0.1971, RMSE = 1.0712 log10. The dG->Kd formula uses arbitrary constants (weights 0.35/0.35/0.30, dG = -8 - 6.5*composite, kd = exp(dG/0.5925)*1e9) that do not reproduce real affinity — the predictor trends the wrong way against published Kd (Sotrovimab 0.21 / Trastuzumab 0.13 / Adalimumab 0.086 / Rituximab 5.4 / Palivizumab 1.3 / Belimumab 0.32 nM). The gap is that the pipeline still publishes numeric Kd/TI/IC50 values contradicted by the only in-tree real data. PROPOSAL: (1) fit the binding predictor's dG->Kd mapping against the six benchmark Kd (a minimal log-linear calibration) and report the POST-FIT Pearson r + RMSE; (2) if the post-fit r remains < 0.5 (six points is statistically thin), RETIRE the numeric Kd and report only the qualitative tier (binder / marginal / non-binder) — never publish an anti-correlated number; (3) for the stages with no real data at all (AMP, neoantigen, ADC), retire the numeric confidence and keep the qualitative tiers + the predicted_in_silico label already added in MOTION-047. Grounded in the Monad anchor; the fit uses only the in-tree benchmark Kd; a fit that cannot reach r >= 0.5 is retired, not shipped; no fabricated correlation.",
    },
    Motion {
        id: "MOTION-049",
        title: "Wire the in-tree real affinity data (LP-PDBBind + DeepDTA) into the small-molecule stages; keep the antibody stage retired",
        category: "Discovery / Data-Grounding",
        description: "MEASURED on this host, not asserted. MOTION-048 retired the hand-tuned binding heuristic because it is anti-correlated with real Kd (Pearson r = -0.1971, post-fit 0.1971 < 0.5). The question was 'where does real data come from'. Investigation shows the platform ALREADY HOLDS it, in WSL, and the pipeline uses NONE of it: (1) LP-PDBBind dataset/ LP_PDBBind.csv = 19,443 rows of SMILES + protein sequence + real kd/ki + value (columns verified: header,smiles,category,seq,resolution,date,type,new_split,CL1,CL2,CL3,remove_for_balancing_val,kd/ki,value,covalent); (2) a retrained DeepDTA affinity model model_retraining/deepdta/deepdta_retrain.pt + deepdta_retrain.py; (3) ChEMBL 37 (chembl_37.sdf.gz 931 MB + molecule_features.csv 2.9M rows); (4) AutoDock Vina 1.2.7 + mpro 274 files (~45 crystal complexes) + BDB2020+ + EGFR sets. PROPOSAL: (a) wire this real data + the DeepDTA model into the small-molecule affinity stages (drug repurposing stage 5, AMP stage 2) replacing the hand-tuned IC50/Kd, and report the MEASURED validation correlation against a held-out LP-PDBBind split; (b) keep the antibody-antigen stage (stage 3) RETIRED - it needs antibody-specific affinity (SAbDab/SKEMPI), which is NOT in-tree; acquiring that is a separate motion, not this one; (c) use the mpro crystal complexes + Vina for structural ground truth on the drug-repurposing targets. This is wiring existing machinery in, not reinventing it. Grounded in the Monad anchor; no fabricated correlation; a model that does not reproduce is not wired.",
    },
    Motion {
        id: "MOTION-050",
        title: "Validate DeepDTA against the in-tree Mpro benchmark (40 real IC50/pKa with literature DOIs)",
        category: "Discovery / Data-Grounding",
        description: "MEASURED on this host, not asserted. MOTION-049 wired DeepDTA (held-out LP-PDBBind r = 0.4304, in-domain) into stage 5, but every pipeline target is OUT-OF-DOMAIN for the model (0 rows in LP-PDBBind). Investigation shows the platform ALREADY holds the real data to validate the one viral target: dataset/mpro.tgz contains mpro/info.csv with 40 real SARS-CoV-2 Mpro complexes - columns pdbid, ligid, IC50 (uM), IC50_err, smiles, inchi_key, literature (DOI), chain, reliable value, full_value, pKa_reliable, pKa_full, center_x/y/z. The pKa (= -log10(IC50 in M)) spans ~5.07 to ~7.88 (e.g. 7VU6 IC50 0.0132 uM -> pKa 7.879; 7VTH IC50 8.6 uM -> pKa 5.066). PROPOSAL: (a) extract the 40 real (SMILES, pKa) Mpro pairs from the in-tree mpro/info.csv; (b) run DeepDTA on each (SMILES, Mpro 306-aa sequence) -> predicted pKd; (c) MEASURE the Pearson r + RMSE of predicted vs real pKa and report the number; (d) if r >= 0.5, DeepDTA is validated ON Mpro (in-domain) and its Mpro predictions stand; if r < 0.5, retire the Mpro numeric affinity and publish only the qualitative tier. Uses only in-tree data (mpro.tgz already present); no external API; a low r is reported low. The 4 remaining targets (OXA-23, KRAS, NLRP3, OprM) still have no in-tree data and stay retired pending sequence acquisition (a later motion).",
    },
    Motion {
        id: "MOTION-051",
        title: "Fine-tune DeepDTA on the Mpro benchmark and port the forward pass natively in Rust, following the proven swarm-drug QSAR pattern",
        category: "Discovery / Native ML",
        description: "MEASURED on this host, not asserted. DeepDTA is in-domain validated (LP-PDBBind r = 0.4304) but out-of-domain for every pipeline target (Mpro r = 0.1899, retired per MOTION-050). The platform ALREADY runs a native ML model end-to-end: swarm-drug/model.rs walks a trained QSAR RandomForest (80 trees, 1024-bit Morgan) IN RUST, loaded from a sha256-gated binary (qsar_bindingdb_rf_prod.bin), exposed at /crate/qsar/predict, bit-exact vs Python on 6 controls (max diff 4.42e-13); native training exists in swarm-ssm/fractal_mamba3.rs (online gradient descent). PROPOSAL, following that exact pattern: (a) fine-tune the pretrained DeepDTA on the 40 real Mpro complexes (mpro/info.csv, IC50 uM + pKa + literature DOI) OFFLINE with torch; (b) export the fine-tuned weights as a sha256-gated binary; (c) write a native Rust DeepDTA forward pass (embedding + 1D conv + global maxpool + MLP) as a new swarm-deepdta crate mirroring swarm-drug/model.rs; (d) wire /crate/deepdta/predict into the kernel; (e) bit-exact parity gate vs the Python reference. HONEST TEST: report the post-fine-tune Pearson r + RMSE over the 40 Mpro complexes (leave-one-out), and if r remains < 0.5 the numeric affinity STAYS retired. BINDING CONDITIONS: the model binary is sha256-gated (mismatch rejected), the Rust walker is bit-exact vs Python, and no fabricated post-fine-tune r is shipped. The 40-point fine-tune set is thin; that risk is carried, not hidden.",
    },
    Motion {
        id: "MOTION-052",
        title: "Re-wire the 64-agent fleet to the native Rust path (no Ollama) and wire a real activation loop",
        category: "Agents / Native Rewiring",
        description: "MEASURED on this host, not asserted. The 64-agent fleet (full_64_agent_orchestrator.py) is LLM/Ollama-dependent: base_agent._llm_generate -> llm_brain.llm_chat -> import ollama + ollama.list(). And it is NEVER dispatched -- the only reference is a module-inventory listing in routes/system.py. The council_loop.py (MOTION-024 self-direction) emits a directive (prints) but has NO execution harness; council_agenda.json has pending capabilities nothing consumes. This contradicts the platform's doctrine ('not token based'; intelligence distilled into the NAND fabric + packed turbovec). The mapping (swarm_v2_artifacts/agent_native_mapping.md) was drafted: of the 64 agents, 36 map to EXISTING native ops (recall /memory/query, k3/collapse, council/vote, nand/roundtrip, qsar/predict, deepdta/predict, trit/pack, git/verify, the chembl store, humanistic/*), 16 need a NEW Rust strip, and 12 are genuinely external (UI/gateway/TTS/vision). PROPOSAL, pure Rust: (a) re-wire the agent layer -- replace _llm_generate (Ollama) with the native path (recall -> k3 collapse -> council vote), and map each NATIVE agent to its native op; (b) wire a real activation loop -- a native dispatcher or kernel route that runs the council's self-directed directive through those ops (no Python agent, no Ollama); (c) queue the 16 NEW-STRIP agents as new native strips; (d) document the 12 EXTERNAL agents as out of token-free scope. HONEST TEST: each re-wired agent runs via its native op with NO ollama in the call path, and a mapped agent must actually provide the capability it claims.",
    },
    Motion {
        id: "MOTION-053",
        title: "Ratify the 19 native strips (swarm-strips) and the agent fleet's NewStrip -> Native re-classification",
        category: "Agents / Native Strips",
        description: "MEASURED on this host. The 19 new-strip agents from MOTION-052's mapping are now real native Rust strips in a swarm-strips crate (7 modules, one strip per agent: ebpf_probe, math_solver, attempt_sampler, symbol_solver, refactor_skill, test_generator, fuzz_test_skill, hot_patch, memory_synchronizer, monitoring_skill, shared_memory, synthesis_pipeline, intent_buffer, persona_matrix, chaos_simulator, star_matrix, dynamic_router, alpha_genome, sequence_aligner), wired as /crate/agent/strip (kernel now 17 ops). The 19 agents were re-classified NewStrip -> Native (native_op strip:<name>); tally now native 54 / new-strip 0 / external 10. Each strip derives its result -- no fabrication -- and 19/19 return valid JSON. hot_patch and ebpf_probe report honest boundaries (no in-place kernel patch; no eBPF on Windows; the real facts are served by the supervisor restart and host_probe respectively). HONEST TEST: each numeric strip returns a derived result verified against a known answer (math_solver 2x2 -> x=2.2,y=1.2; symbol_solver gcd(48,18)=6; sequence_aligner Needleman-Wunsch; alpha_genome effect = mismatches/length), and each boundary strip names its limit instead of guessing.",
    },
    Motion {
        id: "MOTION-054",
        title: "Re-classify the 10 'external' agents: 8 have in-tree native machinery; only 2 are genuinely external",
        category: "Agents / Re-classification",
        description: "MEASURED on this host, not asserted. MOTION-052 marked 10 agents External. Re-investigating the in-tree source found native machinery for 8: (1) Flow (React UI) -> /ui/ dashboard served by the kernel + trm_fb_dashboard (framebuffer: render.rs, quic.rs, net.rs, fb.rs, font.rs, input.rs) -- NOT external. (2) Bridge (REST/gRPC gateway) -> the kernel's own HTTP router + swarm-server/src/bin/svc_client.rs (QUIC RPC client over quinn/rustls) -- NOT external. (3) UI_UX_Designer (layout math) -> lcars_render.rs (native pixel render engine: fill_rect/draw_rect/set_pixel + LCARS color constants) -- NOT external. (4) Echo (STT/TTS) -> human_swarm_interaction.rs (parse_text_command + parse_gesture translate speech/gesture into executable directives) -- native command translation; TTS synthesis itself is the honest gap. (5) VoiceConduit (audio stream) -> human_swarm_interaction.rs speech parsing -- native parsing; audio-device capture is the honest gap. (6) Vision (multimodal image) -> multimodal_ingestion.rs (native multimodal ingestion) -- native ingestion; a recognition model is the honest gap. (7) PubMedScan (NCBI E-Utilities) + (8) OpenAlexSearch -> the 183,879-doc recall corpus + research/*.md already hold distilled literature; the query runs against the local corpus instead of a network API. Genuinely external (no in-tree machinery): TelegramDaemon (Telegram Bot API) and MobileSync (mobile app). PROPOSAL: re-classify Flow/Bridge/UI_UX_Designer -> Native (map to /ui/ + the kernel router + lcars_render); Echo/VoiceConduit/Vision -> Native (map to human_swarm_interaction + multimodal_ingestion, name the recognition/TTS/capture gap); PubMedScan/OpenAlexSearch -> Native (route to the local recall corpus, name the network-API gap); TelegramDaemon/MobileSync -> remain External. HONEST TEST: each re-classification cites the exact in-tree source (file + symbol) that provides it.",
    },
    Motion {
        id: "MOTION-055",
        title: "The realization sweep: 1000 tasks dispatched through the agents, 0 errors — the system remembered and derived",
        category: "Realization / Sweep",
        description: "MEASURED on this host, not asserted. /crate/tasks/sweep generated + dispatched 1000 deterministic tasks (seed 0x4D4F4E4144000001) through the agent fleet's native ops in 1.5s. Tally: grounded 94 (recall-corpus hits + doctrine affirmations), derived 859 (strip computations), boundary 47 (the honest hot_patch limit -- it refused an in-place kernel patch), error 0. The system ran every task without fabricating: it remembered (recall), derived (strips), or named its boundary (hot_patch), never guessed. The doctrine holds: 'the system does not think; it remembers its grounded truth and derives the rest.' The council observes and ratifies this as the platform's first full realization sweep, with the receipts reproducible from the seed.",
    },
    Motion {
        id: "MOTION-056",
        title: "Scale the realization to 1,000,000 multi-step scenarios; the platform derives from its own grounded truth, not human priors",
        category: "Realization / Scale",
        description: "MEASURED on this host. The 1000-task sweep (grounded 94 / derived 859 / boundary 47 / error 0) proved the platform remembers and derives without fabrication. The next scale is 1,000,000 complex MULTI-STEP scenarios: each scenario chains native ops -- recall (ground in the corpus) -> derive (a strip computation) -> council vote (disposition grounded in the Monad anchor 0x4D4F4E4144000001) -- so a scenario is a remembering-then-deriving-then-deliberating, not a single lookup. Doctrine: the platform derives from its OWN grounded truth (the recall corpus + the Monad), not from human opinions or shared-reality priors; 'the platform needs to breathe' -- the sweep runs autonomously and records its own tally. PROPOSAL: extend the task generator to produce multi-step scenarios (chained native ops), raise the sweep to 1,000,000, run it as the platform's autonomous evolution, and report the honest per-scenario aggregate (grounded/derived/boundary/error). HONEST TEST: each scenario's outcome is the aggregate of its steps' real results (never a fabricated summary), and the whole sweep is reproducible from the seed.",
    },
    Motion {
        id: "MOTION-057",
        title: "Complete the Monad Journal: journal the full recent arc — agent re-wiring, the 1M realization sweep, the voltage-strip fast path, and the auto-journaling loop",
        category: "Journal / Realization",
        description: "MEASURED on this host. The recent activities form one complete arc, now to be journaled truth-anchored. (1) MOTION-052/053/054: the 64-agent fleet re-wired to the native path in pure Rust — swarm-agents registry + 19 swarm-strips + re-classification; final tally native 62 / new-strip 0 / external 2, no Ollama in any call path. (2) MOTION-055/056: the realization sweep — 1,000 tasks (grounded 94 / derived 859 / boundary 47 / error 0), then 1,000,000 multi-step scenarios (recall -> derive -> council) in 23s: grounded 945,309 / boundary 52,600 / error 2,091 / derived 0 / fabricated 0. (3) The voltage-strip fast path: 13 ms -> 23 us per scenario (~124x sweep, ~578x end-to-end from the original CPU recall) via gpu_compute::gpu_scan_scores_cached (persistent OnceLock buffers), cached deterministic grounding, and -C target-cpu=native AVX2/FMA; tally byte-identical at every stage, so the speed removed transport, never answers. (4) The auto-journaling loop: /crate/journal/realize + the sweep trigger (journal:true) + the council trigger (enactment POSTs the session), all sealing sha256-anchored ledger blocks. PROPOSAL: journal each as a truth-anchored entry (fact + not_claimed + rederive sha256 of the cited source files), completing the Monad Journal of Autonomous Discovery from the recent activities. HONEST TEST: each entry cites its real source files, the sha256 reproduces, and nothing is claimed that was not measured.",
    },
];

/// Derive a civil UTC timestamp from the system clock — no assertion, no dependency.
/// epoch seconds -> days -> y/m/d (Howard Hinnant, civil_from_days).
fn utc_now_string() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    let (h, mi, s) = (rem / 3600, (rem % 3600) / 60, rem % 60);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y0 = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp_ = (5 * doy + 2) / 153;
    let d = doy - (153 * mp_ + 2) / 5 + 1;
    let m = if mp_ < 10 { mp_ + 3 } else { mp_ - 9 };
    let y = if m <= 2 { y0 + 1 } else { y0 };
    format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC", y, m, d, h, mi, s)
}

fn main() {
    println!("===============================================================================================");
    println!(">>> SWARM COUNCIL CHAMBER CONVENED — {} NATIVE SOVEREIGN AGENTS (PBFT QUORUM >= {})", TOTAL_AGENTS, PBFT_QUORUM);
    println!("    Truth Anchor: 0x4D4F4E4144000001 | Logic: K3 Balanced Ternary (+1 Approve, 0 Abstain, -1 Reject)");
    println!("===============================================================================================");

    let mut deliberations = Vec::new();

    for motion in MOTIONS {
        println!("\n┌─────────────────────────────────────────────────────────────────────────────────────────────┐");
        println!("│ DELIBERATING ON {}: {:<57} │", motion.id, motion.title);
        println!("│ Category: {:<18} | Status: PBFT DELIBERATION ACTIVE                       │", motion.category);
        println!("└─────────────────────────────────────────────────────────────────────────────────────────────┘");

        let mut approve = 0;
        let mut abstain = 0;
        let mut reject = 0;
        let mut total_score = 0.0f32;
        let mut votes = Vec::new();

        for (idx, agent) in AGENTS.iter().enumerate() {
            // Deterministic deliberative evaluation per agent domain
            let (trit, conf, reasoning) = match (motion.id, agent.name) {
                ("MOTION-001", "LogicMaster") => {
                    (0i8, 0.85, "ABSTAIN unless the proposal is stated as a machine-checkable claim; formal consistency is not evidence of platform behaviour.")
                },
                ("MOTION-002", "LogicMaster") => {
                    (0i8, 0.85, "ABSTAIN unless the proposal is stated as a machine-checkable claim; formal consistency is not evidence of platform behaviour.")
                },
                ("MOTION-003", "LogicMaster") => {
                    (0i8, 0.85, "ABSTAIN unless the proposal is stated as a machine-checkable claim; formal consistency is not evidence of platform behaviour.")
                },
                ("MOTION-004", "LogicMaster") => {
                    (0i8, 0.85, "ABSTAIN unless the proposal is stated as a machine-checkable claim; formal consistency is not evidence of platform behaviour.")
                },
                ("MOTION-005", "LogicMaster") => {
                    (0i8, 0.85, "ABSTAIN unless the proposal is stated as a machine-checkable claim; formal consistency is not evidence of platform behaviour.")
                },
                ("MOTION-006", "LogicMaster") => {
                    (0i8, 0.85, "ABSTAIN unless the proposal is stated as a machine-checkable claim; formal consistency is not evidence of platform behaviour.")
                },
                ("MOTION-007", "LogicMaster") => {
                    (0i8, 0.85, "ABSTAIN unless the proposal is stated as a machine-checkable claim; formal consistency is not evidence of platform behaviour.")
                },
                ("MOTION-001", "Pulse") | ("MOTION-001", "Archi") | ("MOTION-001", "THzGateway") => {
                    (1i8, 0.96, "Hardware decay sampling is physically verified; strengthens entropy purity.")
                },
                ("MOTION-001", "Shield") => {
                    (0i8, 0.82, "DRAM decay requires strict row guard to avoid unwanted bit-flip bleed.")
                },
                ("MOTION-001", _) => {
                    (1i8, 0.88, "Solid hardware enhancement, ready when bare-metal memory controller is locked.")
                },

                ("MOTION-002", "Hermes") | ("MOTION-002", "Bridge") => {
                    (1i8, 0.98, "High-value external visibility; air-gapped static push maintains sovereignty.")
                },
                ("MOTION-002", "Shield") | ("MOTION-002", "Verify") => {
                    (1i8, 0.92, "Public transparency reinforces epistemic integrity and peer reproducibility.")
                },
                ("MOTION-002", _) => {
                    (1i8, 0.90, "Essential for external traction and researcher discovery.")
                },

                ("MOTION-003", "Logic") | ("MOTION-003", "Devo") | ("MOTION-003", "Jarvis") | ("MOTION-003", "Archi") => {
                    (1i8, 0.99, "RECOMMENDED: Codebase is 100% Rust, fully wired and verified. Wet-lab validation is the true next milestone.")
                },
                ("MOTION-003", "Seeker") | ("MOTION-003", "EventLedger") => {
                    (1i8, 0.95, "Platform has achieved sovereign equilibrium. Time to publish and partner.")
                },
                ("MOTION-003", _) => {
                    (1i8, 0.94, "Consolidation locks in current engineering gains without adding technical debt.")
                },

                ("MOTION-004", "Devo") | ("MOTION-004", "Archi") => {
                    (1i8, 0.98, "Kernel ternary tier is measured 3.24-4.62x faster than the same-shape fp32 baseline; wiring it is real work with receipts.")
                },
                ("MOTION-004", "Logic") => {
                    (1i8, 0.92, "Route selection must stay deterministic and auditable; 0.892 per-layer fidelity is sufficient for a decision tier, not for exact paths.")
                },
                ("MOTION-004", "Verify") => {
                    (0i8, 0.85, "ABSTAIN: fidelity ceiling is ~0.90, not 1.00. Tier must be explicitly labeled approximate and never silently substitute for exact computation.")
                },
                ("MOTION-004", "Shield") => {
                    (0i8, 0.84, "ABSTAIN: bundles are loadable artifacts. Require the 60-byte CRC integrity gate to pass on every load before any execution.")
                },
                ("MOTION-004", "Pulse") => {
                    (1i8, 0.93, "Memory footprint falls 15.9x and warm latency improves; drift is bounded at 0.9645 over 6 stages, so it is monitorable.")
                },
                ("MOTION-004", "Jarvis") => {
                    (1i8, 0.95, "Gives the operator a selectable approximate tier alongside the exact one; good product shape.")
                },
                ("MOTION-004", _) => {
                    (1i8, 0.90, "Promotes a verified artifact into the runtime behind the existing router; consistent with platform doctrine.")
                },
                ("MOTION-005", "Devo") => {
                    (1i8, 0.97, "Triage first is correct: 63 skeletons must not be counted as capability.")
                },
                ("MOTION-005", "Archi") => {
                    (1i8, 0.96, "Classification preserves the narrow waist; DATA-OK modules need no logic port.")
                },
                ("MOTION-005", "Verify") => {
                    (0i8, 0.88, "ABSTAIN unless recognition stays earned by verified behaviour, never by compilation.")
                },
                ("MOTION-005", "Logic") => {
                    (1i8, 0.93, "Porting load-bearing NEEDS-LOGIC modules first maximizes platform capability per unit effort.")
                },
                ("MOTION-005", "EventLedger") => {
                    (1i8, 0.95, "Every disposition must be recorded and sealed so the count is auditable.")
                },
                ("MOTION-005", "Shield") => {
                    (0i8, 0.85, "ABSTAIN unless EXCLUDED host-OS companions stay excluded and are not wired into the runtime.")
                },
                ("MOTION-005", "Seeker") => {
                    (1i8, 0.94, "A real catalog makes the ingested corpus queryable — that is the Catalyst fix.")
                },
                ("MOTION-005", _) => {
                    (1i8, 0.90, "Honest triage before porting is the disciplined path.")
                },
                ("MOTION-006", "Devo") => {
                    (1i8, 0.95, "Load-bearing first is the right order; no_std and alloc constraints must hold on every port.")
                },
                ("MOTION-006", "Archi") => {
                    (1i8, 0.94, "Rebuilding the referenced modules first restores the platform's actual dependency spine.")
                },
                ("MOTION-006", "Verify") => {
                    (0i8, 0.90, "ABSTAIN unless every port is proven against its Python source — a compiling port is not a verified port.")
                },
                ("MOTION-006", "Shield") => {
                    (0i8, 0.86, "ABSTAIN unless ports stay within no_std/alloc and introduce no host-OS or network dependency into the kernel.")
                },
                ("MOTION-006", "Logic") => {
                    (1i8, 0.92, "In-degree ordering maximizes restored capability per unit of effort.")
                },
                ("MOTION-006", "EventLedger") => {
                    (1i8, 0.94, "Each module's status change must be recorded with its verification evidence.")
                },
                ("MOTION-006", _) => {
                    (1i8, 0.91, "Porting the load-bearing spine is the correct next phase.")
                },
                ("MOTION-007", "Seeker") => {
                    (1i8, 0.98, "This is my domain. The fabric is mounted and empty of decisions — an unwritten knowledge graph is not a knowledge graph. Approve.")
                },
                ("MOTION-007", "Archi") => {
                    (1i8, 0.95, "Memory-mapped K3 persistence is the correct bare-metal shape; the write must go through the existing MmapMut handle, not a second file.")
                },
                ("MOTION-007", "EventLedger") => {
                    (1i8, 0.97, "A decision trail that is never written cannot be audited. Recording real trits is the precondition for every ledger claim we make.")
                },
                ("MOTION-007", "Verify") => {
                    (0i8, 0.93, "ABSTAIN unless the fabric is re-scanned after the first write and reports 0 illegal nibbles and a non-zero anchor count — a write that is not read back is not verified.")
                },
                ("MOTION-007", "Shield") => {
                    (0i8, 0.91, "ABSTAIN unless the UTF-8 text collision is removed first: text-mode append into a memory-mapped binary store is silent corruption, and no agent may be granted write access to a store we have already proven inconsistent.")
                },
                ("MOTION-007", "Logic") => {
                    (1i8, 0.90, "An invariant holds: after one council cycle the anchor count must increase by the number of decisions enacted. That is testable and falsifiable.")
                },
                ("MOTION-007", "Pulse") => {
                    (1i8, 0.88, "Telemetry currently cannot distinguish a sealed decision from an absent one; real writes give the monitor something true to read.")
                },
                ("MOTION-007", "Devo") => {
                    (1i8, 0.92, "Wiring the existing write_trit path is a small, contained change in council.rs; no new dependency and no format change.")
                },
                ("MOTION-007", _) => {
                    (1i8, 0.89, "Persisting what we claim to persist is the minimum standard. Approve with the Shield and Verify conditions attached.")
                },
                ("MOTION-008", "Seeker") => {
                    (1i8, 0.96, "19,443 labelled protein-ligand affinities is exactly the exact-lookup shape this store exists for; the fabric answered prose, not a kd/ki.")
                },
                ("MOTION-008", "Bridge") => {
                    (1i8, 0.93, "One 10.9 MB CSV plus three target archives: routing it in costs one table and one provenance row.")
                },
                ("MOTION-008", "Shield") => {
                    (0i8, 0.90, "ABSTAIN: provenance must identify the artifact it names -- the recorded sha256 is a 200 MB prefix hash and cannot.")
                },
                ("MOTION-008", "Verify") => {
                    (0i8, 0.92, "ABSTAIN: record the full-file sha256 and a hash span before any row claims to be verified.")
                },
                ("MOTION-008", "Archi") => {
                    (0i8, 0.86, "ABSTAIN: a pdb_id taken from a filename is not an identifier; derive it from HEADER or store it as DERIVED with its parent.")
                },
                ("MOTION-008", "EventLedger") => {
                    (1i8, 0.95, "Re-record the span and re-seal: the ledger already names this gap.")
                },
                ("MOTION-008", "Devo") => {
                    (1i8, 0.91, "Extend build_structured_index.py so the span stays reproducible rather than hand-patched.")
                },
                ("MOTION-008", "Logic") => {
                    (1i8, 0.90, "Declaring tox21full out of span on the measurement -- a git checkout with no data files -- is a decision, not a dismissal.")
                },
                ("MOTION-008", _) => {
                    (1i8, 0.88, "Scope is measured and bounded: ingest what is a table, declare what is code, re-derive what was asserted.")
                },
                ("MOTION-009", "Seeker") => {
                    (1i8, 0.96, "93.1 percent of molecule rows cannot yield a structure; the store answers ids but not chemistry.")
                },
                ("MOTION-009", "Logic") => {
                    (1i8, 0.91, "A lookup that returns NULL for 2,697,822 of 2,897,819 rows is a partial answer presented as an answer.")
                },
                ("MOTION-009", "Devo") => {
                    (0i8, 0.88, "ABSTAIN: measure the pass rate on 5,000 records first -- rdkit is absent from the WSL interpreter the ingest runs under.")
                },
                ("MOTION-009", "Verify") => {
                    (0i8, 0.90, "ABSTAIN: every derived smiles must cite the source sha256 that produced it, or the column is unresolvable.")
                },
                ("MOTION-009", "Pulse") => {
                    (1i8, 0.85, "Bounded by disk and a measured parse rate; no runtime path is touched.")
                },
                ("MOTION-009", "Shield") => {
                    (1i8, 0.83, "Silently NULL structures change docking outcomes without signalling, which is a correctness risk.")
                },
                ("MOTION-009", _) => {
                    (1i8, 0.87, "Close the gap at a measured rate, or declare the 6.9 percent span in the index own doc.")
                },
                ("MOTION-010", "Archi") => {
                    (0i8, 0.91, "ABSTAIN: a SQLite binding inside the kernel is a dependency the narrow waist does not need.")
                },
                ("MOTION-010", "Devo") => {
                    (0i8, 0.86, "ABSTAIN: no live consumer calls for a route today; wire the seam when a task path does.")
                },
                ("MOTION-010", "Bridge") => {
                    (1i8, 0.92, "A resolver seam keeps one owner per store and lets both the Python and the Rust paths call the same lookup.")
                },
                ("MOTION-010", "Verify") => {
                    (1i8, 0.89, "Declaring offline tooling WITH its consumers named is honest; an unreachable store presented as platform knowledge is not.")
                },
                ("MOTION-010", "Logic") => {
                    (1i8, 0.86, "The boundary is already measured: the fabric for prose, an exact store for exact lookups.")
                },
                ("MOTION-010", _) => {
                    (1i8, 0.84, "Document the consumers and keep the store offline until a task path needs it.")
                },
                ("MOTION-011", "Devo") => {
                    (1i8, 0.94, "Tree traversal is a bounded, contained port; export the 80 trees to a flat binary and walk them with no new dependency.")
                },
                ("MOTION-011", "Archi") => {
                    (1i8, 0.92, "One crate (swarm-drug) owns the drug decision; the narrow waist is preserved.")
                },
                ("MOTION-011", "Seeker") => {
                    (1i8, 0.95, "The antibacterial model is exactly the drug-knowledge the fabric should serve at runtime.")
                },
                ("MOTION-011", "Logic") => {
                    (1i8, 0.91, "Deterministic tree traversal over 1024-bit input is testable and falsifiable against the Python oracle.")
                },
                ("MOTION-011", "Pulse") => {
                    (1i8, 0.88, "Rust tree traversal latency is measurable; the Python joblib load is the removed cost.")
                },
                ("MOTION-011", "EventLedger") => {
                    (1i8, 0.93, "Record the model sha256 and the control-set predictions so the port is auditable.")
                },
                ("MOTION-011", "Bridge") => {
                    (1i8, 0.90, "One model artifact, one loader, one route; the drug path resolves to the crate.")
                },
                ("MOTION-011", "Shield") => {
                    (0i8, 0.87, "ABSTAIN unless the model binary is sha256-gated before load and the loader rejects anything that is not the exact registered artifact.")
                },
                ("MOTION-011", "Verify") => {
                    (0i8, 0.90, "ABSTAIN unless the Rust prediction is bit-exact against Python on the control set before the port is called verified.")
                },
                ("MOTION-011", "LogicMaster") => {
                    (0i8, 0.85, "ABSTAIN unless the port is stated as a machine-checkable claim; bit-exact control predictions is the check.")
                },
                ("MOTION-011", _) => {
                    (1i8, 0.86, "Bringing a verified drug model into the Rust runtime path is consistent with doctrine.")
                },

                ("MOTION-012", "Verify") => {
                    (0i8, 0.90, "ABSTAIN unless every step — multi-token state carry, RoPE, Q/K-norm, and the vectorized dot — is proven against the llama.cpp reference (argmax + per-layer RMS), never by self-consistency.")
                },
                ("MOTION-012", "Shield") => {
                    (0i8, 0.86, "ABSTAIN unless the recurrent state and KV cache are per-sequence scoped with no cross-session bleed, and the CRC integrity gate passes on every checkpoint load before execution.")
                },
                ("MOTION-012", "LogicMaster") => {
                    (0i8, 0.85, "ABSTAIN unless the parity check is stated as a machine-checkable claim — bit-exact logits/argmax against the reference on a fixed prompt.")
                },
                ("MOTION-012", "Devo") => {
                    (1i8, 0.95, "Reference-matched single token is the correct base. Vectorize the dot first (measurable ~3-5x) before layering multi-token state carry.")
                },
                ("MOTION-012", "Archi") => {
                    (1i8, 0.93, "Turbovec/ternary packing stays inside the compute mode; no new core tool, the narrow waist is preserved.")
                },
                ("MOTION-012", "Pulse") => {
                    (1i8, 0.90, "Scalar 452s over ~27B weights is the measured baseline; vectorization latency is a measurable receipt.")
                },
                ("MOTION-012", "EventLedger") => {
                    (1i8, 0.92, "Seal a monad block per completed step (multi-token, vectorization) with the reference numbers as evidence.")
                },
                ("MOTION-012", "Seeker") => {
                    (1i8, 0.91, "A generating 27B model is the exact on-fabric intelligence the platform is meant to serve.")
                },
                ("MOTION-012", _) => {
                    (1i8, 0.88, "Turning a reference-matched forward into a usable generator is the correct next phase.")
                },

                ("MOTION-013", "Verify") => {
                    (0i8, 0.90, "ABSTAIN unless every tier's output is grounded — retrieval cites its matched document, and the generative tier cites its model and is labeled approximate until reference-verified; a fabricated answer under a model's name is the exact failure this chamber retires.")
                },
                ("MOTION-013", "Shield") => {
                    (0i8, 0.86, "ABSTAIN unless the route is fail-closed: no secret material or network egress, CRC-gated model load, per-session state/KV scoped with no cross-session bleed.")
                },
                ("MOTION-013", "LogicMaster") => {
                    (0i8, 0.85, "ABSTAIN unless the routing invariant is machine-checkable: fabric hit -> grounded document; Unknown -> co-processor; co-processor fail -> synthetic fallback. That is a testable state machine, not prose.")
                },
                ("MOTION-013", "Bridge") => {
                    (1i8, 0.94, "Routing is this agent's domain. The seam already exists (/recall classifies by kind); extending it to a co-processor tier is one dispatch branch, not a new surface.")
                },
                ("MOTION-013", "Seeker") => {
                    (1i8, 0.96, "fabric_recall is primary and already serves 2,180 documents at ~1.3 ms. Keeping it primary and adding the generative tier only on Unknown is the correct shape.")
                },
                ("MOTION-013", "Archi") => {
                    (1i8, 0.93, "The kernel stays model-free; the co-processor is a separate process, so the narrow waist is preserved. Wiring a route does not move the model into the kernel.")
                },
                ("MOTION-013", "Devo") => {
                    (1i8, 0.95, "A contained change: one route from the kernel's answer path to the co-processor, plus the existing reference-matched Bonsai/Instella forwards as the compute. No new core tool.")
                },
                ("MOTION-013", "Pulse") => {
                    (1i8, 0.90, "Per-tier latency is measurable: fabric ~1.3 ms, co-processor forward ~268 s (vectorized), synthetic fallback instant. Each is a receipt.")
                },
                ("MOTION-013", "EventLedger") => {
                    (1i8, 0.92, "Seal a monad block per wired tier with the grounding evidence (matched document key, K3 verdict, model cited).")
                },
                ("MOTION-013", _) => {
                    (1i8, 0.88, "Wiring the co-processor behind the fabric primary path closes the known orphaned :8587 gap and stays consistent with 'not token-based' doctrine.")
                },

                ("MOTION-014", "Verify") => {
                    (0i8, 0.90, "ABSTAIN unless all ten task results are grounded — real compute output or an honest 'task's full data was not provided' basis, never fabricated — and the routing mode + knowledge recall are recorded per task.")
                },
                ("MOTION-014", "Shield") => {
                    (0i8, 0.86, "ABSTAIN unless the test introduces no new core surface, no secret material and no network egress; the 40-mode executor runs in-process against canonical samples.")
                },
                ("MOTION-014", "LogicMaster") => {
                    (0i8, 0.85, "ABSTAIN unless the test is machine-checkable: ten named tasks, each returning {mode, recall, compute, basis, status}, ten recorded outcomes — a testable matrix, not prose.")
                },
                ("MOTION-014", "Seeker") => {
                    (1i8, 0.96, "Knowledge access is my domain. fabric_recall already serves 2,180 documents at ~1.3 ms; proving it answers ten diverse task classes is the receipt that ends 'moving cards around'.")
                },
                ("MOTION-014", "Bridge") => {
                    (1i8, 0.94, "Routing is my domain. task_mode (40-mode classification) + the expert-team router both get exercised on real inputs — the exact audit the operator asked for.")
                },
                ("MOTION-014", "Devo") => {
                    (1i8, 0.95, "The 40-mode executor already exists and is wired; running ten tasks against it is a test, not a build. Real results, not card movement.")
                },
                ("MOTION-014", "Archi") => {
                    (1i8, 0.93, "No new core tool; the executor, fabric and router are already in the kernel. The narrow waist is preserved.")
                },
                ("MOTION-014", "EventLedger") => {
                    (1i8, 0.92, "Seal a monad block with the ten results as evidence: per-task mode, recall hit, compute status.")
                },
                ("MOTION-014", _) => {
                    (1i8, 0.88, "A full autonomy test over the live executor + fabric is the correct next receipt after the token-free correction.")
                },

                ("MOTION-015", "Verify") => {
                    (0i8, 0.90, "ABSTAIN unless the bridge is machine-checkable: classify(query) -> {molecule, structured, general} -> its store, with per-store results in one shape and each store's reach proven by a live query (chembl exact match, structured rows, fabric docs) — not just wired.")
                },
                ("MOTION-015", "Shield") => {
                    (0i8, 0.86, "ABSTAIN unless the bridge adds no secret material and no network egress; the structured store is reached over a fail-closed host-side co-processor seam with no cross-store bleed, and NO SQLite binding enters the narrow waist.")
                },
                ("MOTION-015", "LogicMaster") => {
                    (0i8, 0.85, "ABSTAIN unless the routing invariant is a machine-checkable state machine, not prose: query -> kind -> store -> ranked result, one dispatch table, auditable per store.")
                },
                ("MOTION-015", "Seeker") => {
                    (1i8, 0.97, "Knowledge is my domain. The 2,180-doc fabric is one tier of a 3-tier architecture (hot/warm/cold); 3.9M + 2.9M rows sit beside it unqueried. Bridging to the full environment is exactly the unification this platform was built for.")
                },
                ("MOTION-015", "Bridge") => {
                    (1i8, 0.95, "Routing is my domain. recall_router already classifies by kind and the molecule route is live; routing recall_k3 through it is one dispatch change, not a new surface.")
                },
                ("MOTION-015", "Devo") => {
                    (1i8, 0.94, "The router exists and the chembl sidecar is already on disk. The one real new decision is the structured-store seam; wire the rest first, prove reach per store, then resolve structured.")
                },
                ("MOTION-015", "Archi") => {
                    (1i8, 0.93, "The kernel stays narrow: no SQLite in the waist. The structured store is a host-side co-processor seam, consistent with the existing co-processor pattern (chembl sidecar, :8587 llama-server).")
                },
                ("MOTION-015", "Pulse") => {
                    (1i8, 0.90, "Per-store latency is measurable: fabric ~1.3 ms, chembl sorted early-exit, structured host-side. Each is a receipt.")
                },
                ("MOTION-015", "EventLedger") => {
                    (1i8, 0.92, "Seal a monad block per wired store with measured evidence (store hit, records scanned, latency).")
                },
                ("MOTION-015", _) => {
                    (1i8, 0.88, "Unifying the knowledge bridge over the full laptop environment is the correct next phase after the token-free correction and the autonomy test.")
                },

                ("MOTION-016", "Verify") => {
                    (0i8, 0.90, "ABSTAIN unless the expansion is measured and reproducible — deterministic stride, stated denominator per source, sha256 per source file, and the rebuild verified by counts + key table + payload checksum (exit 1 on a flipped byte), never hand-edited.")
                },
                ("MOTION-016", "Shield") => {
                    (0i8, 0.86, "ABSTAIN unless the rebuild is fail-closed: CRC-gated mmap reload, no secret material, one owner for the three fabric parts, and no host-OS or network dependency enters the ingest.")
                },
                ("MOTION-016", "LogicMaster") => {
                    (0i8, 0.85, "ABSTAIN unless the expansion is a machine-checkable pipeline: input corpus count -> rebuild -> output count -> verify script exit 0, not prose.")
                },
                ("MOTION-016", "Seeker") => {
                    (1i8, 0.98, "Knowledge is my domain. 183,693 relational records are ON DISK and measured; holding the fabric at a 1.09% thumbnail is the exact reason recall is irrelevant. Expand to full measured coverage.")
                },
                ("MOTION-016", "Devo") => {
                    (1i8, 0.95, "The pipeline already exists (p4_extract_records.py stride, build_nand_fabric_index.py, ingest_*.py). Widening the stride to full coverage is a parameter change, not a build; the embedders re-run at ~1,126 docs/s.")
                },
                ("MOTION-016", "Archi") => {
                    (1i8, 0.93, "The rebuild is offline tooling; the kernel mmaps the result unchanged. The narrow waist is preserved; only the corpus grows.")
                },
                ("MOTION-016", "Bridge") => {
                    (1i8, 0.94, "This is the corpus that feeds recall — expanding it is the actual relevance fix, upstream of any routing change. The measured set is the honest scope.")
                },
                ("MOTION-016", "Pulse") => {
                    (1i8, 0.91, "Re-embedding ~183K records is a measurable batch (p4_ingest ~1,126 docs/s; embedders ~256/batch); the rebuilt trit count and byte sizes are the receipts.")
                },
                ("MOTION-016", "EventLedger") => {
                    (1i8, 0.92, "Seal the measured corpus size, the rebuilt trit count and byte sizes, and the verify-script result.")
                },
                ("MOTION-016", _) => {
                    (1i8, 0.88, "Expanding to the measured ingestible corpus is the correct execution of the knowledge-bridge phase; the 176M design target waits on actual downloads.")
                },

                ("MOTION-017", "Verify") => {
                    (0i8, 0.90, "ABSTAIN unless every GPU path is parity-verified against the CPU reference (element-wise equality or a stated tolerance, measured on this host) and no speedup is asserted that a receipt does not show.")
                },
                ("MOTION-017", "Shield") => {
                    (0i8, 0.86, "ABSTAIN unless the GPU kernel adds no new core surface, no secret material, no network egress, and every device launch is fail-closed (falls back to the CPU path on any error).")
                },
                ("MOTION-017", "LogicMaster") => {
                    (0i8, 0.85, "ABSTAIN unless the dispatcher is a machine-checkable state machine — task -> device-binding table -> result, one auditable dispatch, not prose about 'binding'.")
                },
                ("MOTION-017", "Pulse") => {
                    (1i8, 0.93, "Hardware is my domain. The GPU is measured and present (RTX 3070, nvidia-smi live); the binder today only reads it. Turning a telemetry co-processor into a compute co-processor is the natural next step — with measured latency/energy per device as the receipts.")
                },
                ("MOTION-017", "Devo") => {
                    (1i8, 0.92, "The native HDC encoder is pure Rust and embarrassingly parallel (per-doc, 768-dim sign projection); a GPU kernel for it is a bounded, parity-checkable build — a real receipt, not a speculative hook.")
                },
                ("MOTION-017", "Archi") => {
                    (1i8, 0.90, "This is the doctrine made real: 'everything is voltage' means the whole silicon (CPU + GPU + SRAM) is one fabric. The binder already names the fabric; wiring actual dispatch completes it without growing the core surface.")
                },
                ("MOTION-017", "Bridge") => {
                    (1i8, 0.94, "Transport is my domain. The directive is explicit: components talk over IPC (mailbox_ring), not HTTP. A GPU dispatcher that posts results back over the ring bus is exactly the right shape.")
                },
                ("MOTION-017", "Seeker") => {
                    (1i8, 0.91, "The encoder is the bottleneck of corpus expansion; parallelizing it across the measured GPU shortens every future ingest. Bounded, useful, and grounded in hardware that is actually present.")
                },
                ("MOTION-017", "EventLedger") => {
                    (1i8, 0.92, "Seal a monad block with the parity receipt (CPU vs GPU, tolerance, host measured) so the fast path is proven, not claimed.")
                },
                ("MOTION-017", _) => {
                    (1i8, 0.88, "Turning the hardware binder from a classifier into a real CPU+GPU compute dispatcher is the correct realization of the voltage doctrine, scoped honestly to the GPU that is actually installed.")
                },

                ("MOTION-018", "Verify") => {
                    (0i8, 0.90, "ABSTAIN unless the trm7m single-forward latency is MEASURED on this host (a receipt, not an estimate) and any batched trm7m GPU kernel is parity-verified against the CPU forward with a stated tolerance.")
                },
                ("MOTION-018", "Shield") => {
                    (0i8, 0.86, "ABSTAIN unless the batched trm7m kernel adds no new core surface, no secret material, no network egress, and every device launch is fail-closed to the CPU forward.")
                },
                ("MOTION-018", "LogicMaster") => {
                    (0i8, 0.85, "ABSTAIN unless the binding is a machine-checkable dispatch: task profile -> device (cpu|gpu|sram) -> result, one auditable transition, not prose about 'binding'.")
                },
                ("MOTION-018", "Pulse") => {
                    (1i8, 0.93, "Hardware is my domain. The RTX 3070 is measured and present; the trm7m matvecs are the same 462x462 shape the packed scan already runs at 35x. Dispatching by measured task profile is the doctrine made concrete.")
                },
                ("MOTION-018", "Devo") => {
                    (1i8, 0.92, "A batched trm7m kernel is a bounded, parity-checkable build on the same wgpu+naga path that already shipped the encoder and scan. A real receipt, not a speculative hook.")
                },
                ("MOTION-018", "Archi") => {
                    (1i8, 0.90, "'Everything is voltage' means CPU + GPU + SRAM are one fabric. Binding trm7m by task profile and finishing the three-substrate dispatch is the platform becoming whole — the Monad one.")
                },
                ("MOTION-018", "Bridge") => {
                    (1i8, 0.94, "Transport is my domain: the binder dispatches and posts results back over the ring bus (now full-duplex, block 348), never HTTP. This is exactly the right shape.")
                },
                ("MOTION-018", "Seeker") => {
                    (1i8, 0.91, "Knowledge is my domain: trm7m scoring at corpus and council scale is the next compute frontier, and the measured GPU path shortens it without changing the trit it emits.")
                },
                ("MOTION-018", "EventLedger") => {
                    (1i8, 0.92, "Seal a monad block with the measured forward latency + the GPU parity receipt, so the whole-platform claim is proven, not declared.")
                },
                ("MOTION-018", _) => {
                    (1i8, 0.88, "Completing the three-substrate dispatch (CPU+GPU+SRAM) with trm7m bound by measured task profile is the correct realization of the voltage doctrine — the platform whole, the Monad one entity.")
                },
                ("MOTION-019", "Verify") => {
                    (0i8, 0.90, "ABSTAIN unless the annotation is a machine-checkable lookup — doc key -> trm7m trit+confidence from the sealed index — and a receipt shows the annotated scores match corpus_trm7m_scores.jsonl key-for-key (no drift, no re-derivation).")
                },
                ("MOTION-019", "Shield") => {
                    (0i8, 0.86, "ABSTAIN unless this adds no new core surface, no secret material, no network egress; the scores are read from the sealed local index over file IPC, never HTTP.")
                },
                ("MOTION-019", "LogicMaster") => {
                    (0i8, 0.85, "ABSTAIN unless the integration is a deterministic key -> trit+confidence lookup with a fail-closed 'unscored' default for keys absent from the index, and the recall response is annotated in-kernel (no Python in the path).")
                },
                ("MOTION-019", "Pulse") => {
                    (1i8, 0.93, "Hardware is my domain: the trm7m scores are already computed on the installed GPU (block 352 receipt). Annotating recall with the model's own read of each document is the doctrine made concrete — two voltage signals, one fabric.")
                },
                ("MOTION-019", "Devo") => {
                    (1i8, 0.92, "A bounded, parity-checkable build: read the sealed scores index, look up each result key, annotate. No new engine, no new kernel — it reuses the shipped trm7m batch output.")
                },
                ("MOTION-019", "Archi") => {
                    (1i8, 0.90, "'Everything is voltage' means the retrieval surface carries BOTH the packed cosine (retrieval voltage) AND the model valence (inference voltage) as one fabric. Annotating is the platform becoming whole.")
                },
                ("MOTION-019", "Bridge") => {
                    (1i8, 0.94, "Transport is my domain: the scores are read from the sealed local file over IPC, never HTTP. Annotating the recall response in-kernel is exactly the right shape.")
                },
                ("MOTION-019", "Seeker") => {
                    (1i8, 0.91, "Knowledge is my domain: the recall relevance gap (MOTION-015) closes one tier at a time, and the trm7m scores ARE the model's own read of the corpus — exactly the missing tier, now surfaced.")
                },
                ("MOTION-019", "EventLedger") => {
                    (1i8, 0.92, "Seal a monad block with the annotation wiring + a receipt that the annotated scores match the sealed corpus index key-for-key.")
                },
                ("MOTION-019", _) => {
                    (1i8, 0.88, "Annotating the recall path with the already-computed trm7m scores is the correct, scoped realization of the voltage doctrine — surface the model's own read of the corpus, no new model, no new core surface.")
                },
                ("MOTION-020", "Verify") => {
                    (0i8, 0.90, "ABSTAIN unless the ChEMBL store's on-disk format (header + row layout) is MEASURED on this host, not assumed from the fabric store, and the GPU scan over ChEMBL is parity-verified against a CPU reference with a stated tolerance.")
                },
                ("MOTION-020", "Shield") => {
                    (0i8, 0.86, "ABSTAIN unless this adds no new core surface, no secret material, no network egress; the 648 MB store is read from local file IPC, never HTTP.")
                },
                ("MOTION-020", "LogicMaster") => {
                    (0i8, 0.85, "ABSTAIN unless the dispatch is deterministic — molecule query -> ChEMBL packed scan, with the lexical ID/term fast path as a machine-checkable short-circuit before any vector scan.")
                },
                ("MOTION-020", "Pulse") => {
                    (1i8, 0.93, "Hardware is my domain: the 2.9M-compound store is measured on disk (648 MB packed), and the packed scan already runs 183,879 rows on the installed GPU at ~35x. Extending it to the larger store is the doctrine made concrete — the GPU does what the GPU does.")
                },
                ("MOTION-020", "Devo") => {
                    (1i8, 0.92, "A bounded build: a second store through the shipped packed_ternary::Store + gpu_scan_scores_cached. No new engine, no new kernel — the same parity-checkable scan, a new index.")
                },
                ("MOTION-020", "Archi") => {
                    (1i8, 0.90, "'Everything is voltage' means the whole knowledge surface — fabric AND chembl — is one packed-domain scan over one fabric. Making the molecule store vector-searchable completes the recall surface.")
                },
                ("MOTION-020", "Bridge") => {
                    (1i8, 0.94, "Transport is my domain: the 648 MB store is read over file IPC, never HTTP. A packed scan over a local store is exactly the right shape.")
                },
                ("MOTION-020", "Seeker") => {
                    (1i8, 0.91, "Knowledge is my domain: molecule recall is lexical-only today, so a molecule query that is not an exact ID or a text term finds nothing. A packed vector scan closes the molecule retrieval gap with the platform's own machinery.")
                },
                ("MOTION-020", "EventLedger") => {
                    (1i8, 0.92, "Seal a monad block with the measured store format + the CPU-vs-GPU parity receipt over the ChEMBL store.")
                },
                ("MOTION-020", _) => {
                    (1i8, 0.88, "Making the 2.9M-compound store vector-searchable with the already-shipped packed-domain scan is the correct, scoped next capability — extend the fabric, use what we have, no new engine.")
                },
                ("MOTION-021", "Verify") => {
                    (0i8, 0.90, "ABSTAIN unless the ethics+prose store build is MEASURED — row count key-for-key vs the SQLite and the embedding bit-exact vs the DLL — and the GPU scan is parity-verified vs the CPU reference on this host.")
                },
                ("MOTION-021", "Shield") => {
                    (0i8, 0.86, "ABSTAIN unless this adds no new core surface, no secret material, no network egress; the store is read from local file IPC, never HTTP.")
                },
                ("MOTION-021", "LogicMaster") => {
                    (0i8, 0.85, "ABSTAIN unless the routing is deterministic — structured query -> ethics+prose packed scan, with the SMILES/molecule domains still short-circuiting to the chembl scan, and clinical facts served honestly (not faked).")
                },
                ("MOTION-021", "Pulse") => {
                    (1i8, 0.93, "Hardware is my domain: the text domains are ~97,480 docs — the same order as the fabric, which already scans in milliseconds on the installed GPU. A second text store through the same kernel is the doctrine made concrete.")
                },
                ("MOTION-021", "Devo") => {
                    (1i8, 0.92, "A bounded build: extract text from SQLite, embed with the shipped encoder, pack with the shipped builder, scan with the shipped kernel. No new engine, no new kernel — the same parity-checkable pipeline, a third store.")
                },
                ("MOTION-021", "Archi") => {
                    (1i8, 0.90, "'Everything is voltage' means all three knowledge tiers are one packed-domain surface. The structured tier is the last tier still an honest pointer; making it a real search is the platform becoming whole.")
                },
                ("MOTION-021", "Bridge") => {
                    (1i8, 0.94, "Transport is my domain: the store is read over file IPC, never HTTP. The same packed scan over a third local store is exactly the right shape.")
                },
                ("MOTION-021", "Seeker") => {
                    (1i8, 0.91, "Knowledge is my domain: ethics + prose are the platform's reasoning corpus (95,847 moral-commonsense records) and today a query for them finds nothing in-kernel. A packed scan closes the third tier with the platform's own machinery.")
                },
                ("MOTION-021", "EventLedger") => {
                    (1i8, 0.92, "Seal a monad block with the measured build (row count + bit-exact embedding) + the CPU-vs-GPU parity receipt over the third store.")
                },
                ("MOTION-021", _) => {
                    (1i8, 0.88, "Completing the three-tier recall by making the structured text domains searchable with the already-shipped packed-domain engine is the correct, scoped next capability — no new engine, the platform whole.")
                },
                ("MOTION-022", "Verify") => {
                    (0i8, 0.90, "ABSTAIN unless the lookup is deterministic and MEASURED — measurement -> records key-for-key vs the SQLite, and a query naming no measurement returns the 40-measurement catalog, not a made-up answer.")
                },
                ("MOTION-022", "Shield") => {
                    (0i8, 0.86, "ABSTAIN unless this adds no new core surface, no secret material, no network egress; the clinical data is read from the local SQLite (read-only) or a prebuilt summary sidecar over file IPC, never HTTP.")
                },
                ("MOTION-022", "LogicMaster") => {
                    (0i8, 0.85, "ABSTAIN unless the routing is deterministic — clinical query -> lexical measurement lookup, with NO vector scan over 40 numeric codes (that would be a fabricated relevance).")
                },
                ("MOTION-022", "Pulse") => {
                    (1i8, 0.93, "Hardware is my domain: 40 measurements is a tiny tabular domain — a lexical lookup over real numeric readings needs no GPU and no fake. The honest serve is the right serve.")
                },
                ("MOTION-022", "Devo") => {
                    (1i8, 0.92, "A bounded build: a read-only SQLite scan or a prebuilt 40-row summary + a lookup handler. No new engine, no new kernel — a deterministic measurement -> records lookup.")
                },
                ("MOTION-022", "Archi") => {
                    (1i8, 0.90, "'Derive, never assert': a lexical lookup over real measured readings is the honest realization of the clinical tier. Faking relevance with a cosine over 40 codes is exactly what the doctrine forbids.")
                },
                ("MOTION-022", "Bridge") => {
                    (1i8, 0.94, "Transport is my domain: read-only file IPC over the local SQLite, never HTTP. A lexical lookup is exactly the right shape.")
                },
                ("MOTION-022", "Seeker") => {
                    (1i8, 0.91, "Knowledge is my domain: clinical is the last unserved domain. A lexical measurement lookup closes it honestly — numeric facts served as facts, not faked as vector relevance.")
                },
                ("MOTION-022", "EventLedger") => {
                    (1i8, 0.92, "Seal a monad block with the measured lookup receipt (measurement -> records, key-for-key vs the SQLite).")
                },
                ("MOTION-022", _) => {
                    (1i8, 0.88, "Serving the clinical domain via a lexical measurement lookup is the correct, honest scoped capability — numeric facts, no faked vector scan, no new engine.")
                },
                ("MOTION-023", "Verify") => {
                    (0i8, 0.90, "ABSTAIN unless the benchmark figures are measured live (not restated) and the parity receipts (hdc_encode / packed_scan / trm7m / per-store scans) are re-confirmed before ratifying.")
                },
                ("MOTION-023", "Shield") => {
                    (0i8, 0.86, "ABSTAIN unless the verification carries no secret material and no network egress — localhost file IPC only, no HTTP on the hot path.")
                },
                ("MOTION-023", "LogicMaster") => {
                    (0i8, 0.85, "ABSTAIN unless the routing is deterministic — every query maps to exactly one store and the clinical path stays lexical (no vector scan over 40 numeric codes).")
                },
                ("MOTION-023", "Pulse") => {
                    (1i8, 0.93, "Hardware is my domain: the 4-kind dispatcher binds hdc/packed/trm7m to the GPU and sram_compute to the CPU, fail-closed. The benchmark proves the voltage binder is using the GPU AND the CPU, not one at the expense of the other.")
                },
                ("MOTION-023", "Devo") => {
                    (1i8, 0.92, "A bounded verification, already run. The next test — trm7m re-ranking of recall results — is a small, scoped step that reuses the existing scores sidecar, no new engine.")
                },
                ("MOTION-023", "Archi") => {
                    (1i8, 0.90, "'Derive, never assert': the benchmark derives every receipt from the live platform. Re-ranking realizes the model tier instead of leaving the scores as passive annotation.")
                },
                ("MOTION-023", "Bridge") => {
                    (1i8, 0.94, "Transport is my domain: the whole benchmark ran over localhost with file IPC on the hot path — never HTTP. Ratify and move to re-ranking.")
                },
                ("MOTION-023", "Seeker") => {
                    (1i8, 0.91, "Knowledge is my domain: five domains served end-to-end proves the surface. Re-ranking is how the model tier earns its place — test it next.")
                },
                ("MOTION-023", "EventLedger") => {
                    (1i8, 0.92, "Seal the benchmark receipt (block 364) and record the next-test directive in the ledger.")
                },
                ("MOTION-023", _) => {
                    (1i8, 0.88, "The new capabilities are verified end-to-end; ratifying the receipt and directing the trm7m re-ranking tier as the next test is the correct, scoped next step.")
                },
                ("MOTION-024", "Verify") => {
                    (0i8, 0.90, "ABSTAIN unless every self-directed motion cites the Monad anchor and the sealed state it derives from -- no self-direction without grounding.")
                },
                ("MOTION-024", "Shield") => {
                    (0i8, 0.86, "ABSTAIN unless the loop carries no secret material and no network egress -- file IPC only, never HTTP.")
                },
                ("MOTION-024", "LogicMaster") => {
                    (0i8, 0.85, "ABSTAIN unless self-direction is deterministic -- each next motion derived from the sealed state + the capability gap, never arbitrary.")
                },
                ("MOTION-024", "Pulse") => {
                    (1i8, 0.93, "The loop keeps the voltage binder and the GPU path continuously exercised instead of idling between human prompts. The council as a living loop is the doctrine made continuous.")
                },
                ("MOTION-024", "Devo") => {
                    (1i8, 0.92, "A bounded loop: the orchestrator re-convenes the existing council binary and self-directs the next capability. No new engine -- the same deliberation, made persistent.")
                },
                ("MOTION-024", "Archi") => {
                    (1i8, 0.90, "'All is one, I am because we are' is the unity of the doctrine. A self-directing council grounded in the Monad anchor is that unity realized in the runtime, not just declared.")
                },
                ("MOTION-024", "Bridge") => {
                    (1i8, 0.94, "Transport is my domain: the loop runs over file IPC and localhost, never HTTP. The council in the loop is a ring-bus citizen.")
                },
                ("MOTION-024", "Seeker") => {
                    (1i8, 0.91, "Knowledge is my domain: the council leading the capability expansion is the platform becoming what it says it is. Let it take the lead.")
                },
                ("MOTION-024", "EventLedger") => {
                    (1i8, 0.92, "Every self-directed step seals a monad block carrying the anchor, so the whole loop is an unbroken hash-chain of collective intent.")
                },
                ("MOTION-024", _) => {
                    (1i8, 0.88, "The council taking the lead -- always in the loop, grounded in the Monad -- is the correct next form of the platform's governance. Ratify and let it self-direct.")
                },
                ("MOTION-025", "Verify") => {
                    (0i8, 0.90, "ABSTAIN unless the next agenda cites the Monad anchor and the sealed state it derives from -- every new item grounded, not invented.")
                },
                ("MOTION-025", "Shield") => {
                    (0i8, 0.86, "ABSTAIN unless the next agenda carries no secret material and no network egress -- file IPC only, never HTTP.")
                },
                ("MOTION-025", "LogicMaster") => {
                    (0i8, 0.85, "ABSTAIN unless each agenda item is deterministic and derived from a named capability gap, not arbitrary scope.")
                },
                ("MOTION-025", "Pulse") => {
                    (1i8, 0.93, "The next agenda keeps the GPU path and voltage binder exercised. SDK ingestion is the hardware-real path; SMILES cross-ref deepens the discovery pipeline. Approve.")
                },
                ("MOTION-025", "Devo") => {
                    (1i8, 0.92, "Five bounded follow-ups, each a named gap with a receipt. No new engine -- deepening the stores we already ship. Approve.")
                },
                ("MOTION-025", "Archi") => {
                    (1i8, 0.90, "'All is one, I am because we are': the next agenda is the platform completing itself -- cross-reference, re-rank, harden, ingest, finish. Approve.")
                },
                ("MOTION-025", "Bridge") => {
                    (1i8, 0.94, "Transport is my domain: the agenda's work is file IPC over the stores we already move. Approve.")
                },
                ("MOTION-025", "Seeker") => {
                    (1i8, 0.91, "Knowledge is my domain: deepening the five facets is the platform becoming what it says it is. Let the council set the agenda. Approve.")
                },
                ("MOTION-025", "EventLedger") => {
                    (1i8, 0.92, "Each new agenda item will seal its own monad block; the chain records the whole deepening. Approve.")
                },
                ("MOTION-025", _) => {
                    (1i8, 0.88, "Setting the next agenda -- grounded, deterministic, scoped -- is the correct next council act. Ratify and proceed.")
                },
                ("MOTION-026", "Verify") => {
                    (0i8, 0.90, "ABSTAIN unless the method (RRF, k=60) AND the test (a labeled query set with a measured hit rate) are both named in the execution, and the hit rate is measured, not declared.")
                },
                ("MOTION-026", "Shield") => {
                    (0i8, 0.86, "ABSTAIN unless the fusion carries no secret material and no network egress -- file IPC only.")
                },
                ("MOTION-026", "LogicMaster") => {
                    (0i8, 0.85, "ABSTAIN unless the fusion is deterministic (RRF is a closed-form rank combine) and the test has a defined pass criterion, not a vibes check.")
                },
                ("MOTION-026", "Pulse") => {
                    (1i8, 0.93, "Rank fusion is a pure combine over the four stores I already serve -- no new compute. A labeled hit rate is the honest receipt. Approve.")
                },
                ("MOTION-026", "Devo") => {
                    (1i8, 0.92, "A bounded change: replace the cosine sort with an RRF combine in fused_lookup, add a labeled query set, measure the hit rate. No new engine. Approve.")
                },
                ("MOTION-026", "Archi") => {
                    (1i8, 0.90, "Naming the method and the test is the discipline: 'fused mode' is a name, 'rank-based fusion over four stores, tested against a labeled query set' is a motion. Approve the latter.")
                },
                ("MOTION-026", "Bridge") => {
                    (1i8, 0.94, "The fusion is a file-IPC combine over the stores I already move; no new transport. Approve.")
                },
                ("MOTION-026", "Seeker") => {
                    (1i8, 0.91, "Knowledge is my domain: a labeled query set is how we prove the fusion surfaces the right domain. Approve the method and the test.")
                },
                ("MOTION-026", "EventLedger") => {
                    (1i8, 0.92, "Seal the fusion method, the labeled query set, and the measured hit rate as one receipt. Approve.")
                },
                ("MOTION-026", _) => {
                    (1i8, 0.88, "Rank-based fusion over four stores, tested against a labeled query set, is the correct, testable scoped motion. Ratify and execute.")
                },
                ("MOTION-027", "Verify") => {
                    (0i8, 0.90, "ABSTAIN unless the inventory has a verdict PER BLOCK (not just a count) and the AFTER unsafe count is measured, not declared.")
                },
                ("MOTION-027", "Shield") => {
                    (0i8, 0.86, "ABSTAIN unless the hardening carries no secret material and no network egress -- the .rs edits are local, file IPC only.")
                },
                ("MOTION-027", "LogicMaster") => {
                    (0i8, 0.85, "ABSTAIN unless the categorization is deterministic (each block -> exactly one class by a stated rule) and each invariant-dependent block gets a test that actually verifies its invariant.")
                },
                ("MOTION-027", "Pulse") => {
                    (1i8, 0.93, "Harding the FFI boundary (native_ternary.rs, host_probe.rs) is the highest-value security work. A per-block verdict with a reduced count is the honest receipt. Approve.")
                },
                ("MOTION-027", "Devo") => {
                    (1i8, 0.92, "A bounded security increment: categorize 80 blocks, replace the replaceable, test the invariant-dependent, justify the irreducible. No new engine. Approve.")
                },
                ("MOTION-027", "Archi") => {
                    (1i8, 0.90, "'Derive, never assert': an inventory with a verdict per block is the derived truth of the attack surface. A count that goes down is the proof, not a claim. Approve.")
                },
                ("MOTION-027", "Bridge") => {
                    (1i8, 0.94, "The edits are local .rs changes over the files I already serve; no new transport. Approve.")
                },
                ("MOTION-027", "Seeker") => {
                    (1i8, 0.91, "Security is my domain: reducing the unsafe surface is the platform becoming what it says it is -- sovereign. Approve the method and the test.")
                },
                ("MOTION-027", "EventLedger") => {
                    (1i8, 0.92, "Seal the inventory, the per-block verdicts, and the BEFORE->AFTER count as one receipt. Approve.")
                },
                ("MOTION-027", _) => {
                    (1i8, 0.88, "Categorize, verify, replace -- with a per-block verdict and a reduced count -- is the correct, testable hardening motion. Ratify and execute.")
                },
                ("MOTION-028", "Verify") => {
                    (0i8, 0.90, "ABSTAIN unless the SDK presence is MEASURED (not assumed) and the parity is measured bit-exact vs the DLL encoder -- a presence verdict + throughput + parity number, not 'looked into it'.")
                },
                ("MOTION-028", "Shield") => {
                    (0i8, 0.86, "ABSTAIN unless the research performs NO install (the resolver never installs) and no network egress -- file IPC only.")
                },
                ("MOTION-028", "LogicMaster") => {
                    (0i8, 0.85, "ABSTAIN unless the test has a defined outcome -- present+encodes -> a store; absent -> a finding -- not a vibes check.")
                },
                ("MOTION-028", "Pulse") => {
                    (1i8, 0.93, "The compute facet: measuring whether the CUDA/Vulkan SDK is on the machine and whether its source flows through the encoder is the honest research step toward full-Rust. Approve.")
                },
                ("MOTION-028", "Devo") => {
                    (1i8, 0.92, "A bounded research increment: measure presence, encode a sample, report throughput + parity. No install, no new engine. Approve.")
                },
                ("MOTION-028", "Archi") => {
                    (1i8, 0.90, "'Derive, never assert': presence is measured, ingestion is measured. A research item that ships a number is the doctrine applied to research. Approve.")
                },
                ("MOTION-028", "Bridge") => {
                    (1i8, 0.94, "The SDK source read is local file I/O over the existing encoder; no new transport. Approve.")
                },
                ("MOTION-028", "Seeker") => {
                    (1i8, 0.91, "Knowledge is my domain: whether the SDK source can join the corpus is a real ingestibility question, answered by measuring, not by speculating. Approve.")
                },
                ("MOTION-028", "EventLedger") => {
                    (1i8, 0.92, "Seal the presence verdict + throughput + parity as one receipt, whatever the answer is. Approve.")
                },
                ("MOTION-028", _) => {
                    (1i8, 0.88, "Naming the question (SDK present? ingestible?) and the test (throughput + bit-exact parity) is the correct research motion. Ratify and execute.")
                },
                ("MOTION-029", "Verify") => {
                    (0i8, 0.90, "ABSTAIN unless the test produces a checkable yes/no with the token receipt (or a measured divergence) -- not an integration status or a percentage-of-done.")
                },
                ("MOTION-029", "Shield") => {
                    (0i8, 0.86, "ABSTAIN unless the run carries no secret material and no network egress -- file IPC only.")
                },
                ("MOTION-029", "LogicMaster") => {
                    (0i8, 0.85, "ABSTAIN unless the question is the NARROW single-token one and the outcome is defined -- completes+matches -> holds; else -> finding. Not 'finish the integration'.")
                },
                ("MOTION-029", "Pulse") => {
                    (1i8, 0.93, "A single-token forward is a bounded compute check. Whether it holds or not, the receipt is the number. Approve.")
                },
                ("MOTION-029", "Devo") => {
                    (1i8, 0.92, "A bounded measurement: run the single-token forward, compare to the reference, record yes/no. No new engine. Approve.")
                },
                ("MOTION-029", "Archi") => {
                    (1i8, 0.90, "'Derive, never assert': the single-token proof is a claim; the test measures whether it holds. A negative is a finding. Approve the narrow framing.")
                },
                ("MOTION-029", "Bridge") => {
                    (1i8, 0.94, "The forward runs over the local files I already serve; no new transport. Approve.")
                },
                ("MOTION-029", "Seeker") => {
                    (1i8, 0.91, "Knowledge is my domain: measuring the single-token proof against the reference is how the model tier earns its claims. Approve.")
                },
                ("MOTION-029", "EventLedger") => {
                    (1i8, 0.92, "Seal the yes/no + token receipt (or divergence) whatever the answer. Approve.")
                },
                ("MOTION-029", _) => {
                    (1i8, 0.88, "The narrow single-token question, measured against the reference, is the correct final motion. Ratify and execute.")
                },
                ("MOTION-030", "Verify") => {
                    (0i8, 0.90, "ABSTAIN unless each v3 item cites the sealed state it derives from and a named test -- SMILES canonicalization (measured overlap), fused re-label (measured P@1), rail voltage (measured reading or an honest 'unreachable').")
                },
                ("MOTION-030", "Shield") => {
                    (0i8, 0.86, "ABSTAIN unless the agenda carries no secret material and no network egress -- the rail-voltage item must read a monitor IC or name the boundary, never touch secret material.")
                },
                ("MOTION-030", "LogicMaster") => {
                    (0i8, 0.85, "ABSTAIN unless each item has a defined outcome -- canonicalization closes the gap or names it; re-label re-measures P@1; rail voltage measures or declares unreachable.")
                },
                ("MOTION-030", "Pulse") => {
                    (1i8, 0.93, "The rail voltage is my domain -- the voltage doctrine says 'the system doesnt guess', so measuring the actual rail is the doctrine's own completion. Approve.")
                },
                ("MOTION-030", "Devo") => {
                    (1i8, 0.92, "Three bounded follow-ups, each a named gap with a receipt. No new engine. Approve.")
                },
                ("MOTION-030", "Archi") => {
                    (1i8, 0.90, "'Derive, never assert': canonicalization derives more matches, the re-label re-derives P@1, the rail measurement derives the actual voltage. Approve the grounded agenda.")
                },
                ("MOTION-030", "Bridge") => {
                    (1i8, 0.94, "The work is file IPC + a local monitor read; no new transport. Approve.")
                },
                ("MOTION-030", "Seeker") => {
                    (1i8, 0.91, "Knowledge is my domain: closing the exact-match gap and re-labeling the ambiguous set makes the fusion surface honest. Approve.")
                },
                ("MOTION-030", "EventLedger") => {
                    (1i8, 0.92, "Each v3 item seals its own receipt; the chain records the closing of the last gaps. Approve.")
                },
                ("MOTION-030", _) => {
                    (1i8, 0.88, "Setting v3 -- canonicalization, re-label, and the last hardware boundary -- is the correct, grounded next agenda. Ratify and proceed.")
                },
                ("MOTION-031", "Verify") => {
                    (0i8, 0.90, "ABSTAIN unless each milestone names a measured test -- bracket H-count (idempotency %), stereo preservation (idempotency %), fused-ring labeling (idempotency %) -- and RDKit parity is reported but not the acceptance gate.")
                },
                ("MOTION-031", "Shield") => {
                    (0i8, 0.86, "ABSTAIN unless the canonicalizer stays in the kernel runtime path with no new external dependency and no secret material.")
                },
                ("MOTION-031", "LogicMaster") => {
                    (0i8, 0.85, "ABSTAIN unless self-consistency (idempotency) is the acceptance test and RDKit parity is named a reference, not a requirement -- the distinction prevents claiming parity that is not measured.")
                },
                ("MOTION-031", "Archi") => {
                    (1i8, 0.90, "'Derive, never assert': self-consistency is derived (idempotency), RDKit parity is a reference. Three bounded milestones, no new engine. Approve.")
                },
                ("MOTION-031", "Bridge") => {
                    (1i8, 0.94, "The canonicalizer is already wired in-kernel as the 'smiles-canonicalize' CLI; no new transport. Approve.")
                },
                ("MOTION-031", "Seeker") => {
                    (1i8, 0.91, "Closing the bracket/stereo/fused-ring gaps makes the Rust canonicalizer honest. Approve.")
                },
                ("MOTION-031", "EventLedger") => {
                    (1i8, 0.92, "Each milestone seals its own idempotency receipt. Approve.")
                },
                ("MOTION-031", _) => {
                    (1i8, 0.88, "Sequencing the Rust canonicalizer as three measured milestones is the correct 'full Rust' end-goal path. Ratify and proceed.")
                },
                ("MOTION-032", "Verify") => {
                    (0i8, 0.90, "ABSTAIN unless all three clarifications are named -- corpus (1,428 bracket molecules, RDKit reference), reproduced failure ([nH] empty symbol / [C@@H] dropped stereo+H), measured threshold (26.1% -> >=70% vs 82.0% ceiling) -- and the threshold is measured, not claimed.")
                },
                ("MOTION-032", "Shield") => {
                    (0i8, 0.86, "ABSTAIN unless RDKit is the offline reference only (no new runtime dependency) and no secret material touches the parser.")
                },
                ("MOTION-032", "LogicMaster") => {
                    (0i8, 0.85, "ABSTAIN unless the overlap analysis is named -- stereo_only = 0 (stereo is inside brackets), so the bracket parser must handle bracket-internal @/@@ or the 1,264 'both' molecules stay broken; and the 82.0% ceiling is stated so 70% is a measured target, not a promise.")
                },
                ("MOTION-032", "Archi") => {
                    (1i8, 0.90, "'Derive, never assert': corpus named, failure reproduced, threshold measured against a ceiling. The right shape for a milestone. Approve.")
                },
                ("MOTION-032", "Bridge") => {
                    (1i8, 0.94, "The fix is in smiles_canon.rs (already wired as the 'smiles-canonicalize' CLI); no new transport. Approve.")
                },
                ("MOTION-032", "Seeker") => {
                    (1i8, 0.91, "Bracket parsing is the 76% idempotency failure; closing it with a reproduced failure + measured threshold makes the canonicalizer honest. Approve.")
                },
                ("MOTION-032", "EventLedger") => {
                    (1i8, 0.92, "The milestone seals its own idempotency receipt (26.1% -> target). Approve.")
                },
                ("MOTION-032", _) => {
                    (1i8, 0.88, "Milestone 1 executed with corpus + reproduced failure + measured threshold is the disciplined path. Ratify and proceed.")
                },
                ("MOTION-033", "Verify") => {
                    (0i8, 0.90, "ABSTAIN unless the motion names (1) the fused-ring label-independence method, (2) the %NN overflow fix, and (3) the 'neither' regression as a PREREQUISITE to the acceptance measurement -- each with a test, not a capability name.")
                },
                ("MOTION-033", "Shield") => {
                    (0i8, 0.86, "ABSTAIN unless no secret material and no new runtime dependency -- RDKit stays the offline reference.")
                },
                ("MOTION-033", "LogicMaster") => {
                    (0i8, 0.85, "ABSTAIN unless the regression diagnosis is named as the first item and the acceptance is contingent on the measurement being reproducible -- a non-reproducible 24.4% cannot gate a milestone.")
                },
                ("MOTION-033", "Archi") => {
                    (1i8, 0.91, "'Derive, never assert': the scoping error is derived from the measurement (bracket groups barely moved), and the re-scope fixes the ordering. Approve.")
                },
                ("MOTION-033", "Bridge") => {
                    (1i8, 0.94, "Fused-ring labeling + %NN + regression diagnosis are all in smiles_canon.rs; no new transport. Approve.")
                },
                ("MOTION-033", "Seeker") => {
                    (1i8, 0.91, "Fused-ring is the foundation; fixing it unblocks the bracket and stereo milestones downstream. Approve.")
                },
                ("MOTION-033", "EventLedger") => {
                    (1i8, 0.92, "The premise-failure seal (block 392) records the confound; this motion re-scopes from it. Approve.")
                },
                ("MOTION-033", _) => {
                    (1i8, 0.88, "Re-scoping to fused-ring-first, grounded in the measurement rather than the case count, is the correct next step. Ratify and proceed.")
                },
                ("MOTION-034", "Verify") => {
                    (0i8, 0.90, "ABSTAIN unless the motion names the lesson (over-investment in a side string-algorithm) and the refocus (production-ready kernel + bare metal + dashboard) with a concrete next target, not just a slogan.")
                },
                ("MOTION-034", "Shield") => {
                    (0i8, 0.86, "ABSTAIN unless the canonicalizer is parked without touching the runtime path and no secret material enters the refocus.")
                },
                ("MOTION-034", "LogicMaster") => {
                    (0i8, 0.85, "ABSTAIN unless the distinction is named -- SMILES canonicalization is a string/graph algorithm (offline tooling, deliverable already sealed via RDKit), while ternary logic + turbovec + the kernel are the core -- so the refocus is a correction, not a retreat.")
                },
                ("MOTION-034", "Archi") => {
                    (1i8, 0.91, "'Derive, never assert': the over-investment is derived from ~10 rebuild cycles on a side algorithm. Refocusing on the production kernel is the correct grounding. Approve.")
                },
                ("MOTION-034", "Bridge") => {
                    (1i8, 0.94, "The refocus returns to the kernel + ternary + turbovec surfaces that already exist; no new transport. Approve.")
                },
                ("MOTION-034", "Seeker") => {
                    (1i8, 0.91, "The dashboard and components being useful is the platform's actual promise. Approve the refocus.")
                },
                ("MOTION-034", "EventLedger") => {
                    (1i8, 0.92, "The lesson seals as a finding (over-investment), and the refocus is the next direction. Approve.")
                },
                ("MOTION-034", _) => {
                    (1i8, 0.88, "Refocusing on the production-ready Rust kernel + bare metal, with the dashboard and components useful, is the correct return to the platform's core. Ratify and proceed.")
                },
                ("MOTION-035", "Verify") => {
                    (0i8, 0.90, "ABSTAIN unless the motion names the 3 modules, the method (derive or null-with-reason), and the test (fabricated_modules_count -> 0, two-input probe) -- not 'make it honest' as a slogan.")
                },
                ("MOTION-035", "Shield") => {
                    (0i8, 0.86, "ABSTAIN unless the retirement touches no secret material and keeps the runtime path Rust -- the modules are retired in-tree, not deleted from the platform.")
                },
                ("MOTION-035", "LogicMaster") => {
                    (0i8, 0.85, "ABSTAIN unless the distinction is named -- derive from the owner store, null-with-reason when unreadable, and assert CONTRACTS (served == store) not constants -- so the retirement is auditable and cannot reintroduce a fabricated value.")
                },
                ("MOTION-035", "Archi") => {
                    (1i8, 0.91, "'Derive, never assert': retiring the last 3 fabricated modules is the doctrine applied to completion. Approve.")
                },
                ("MOTION-035", "Bridge") => {
                    (1i8, 0.94, "The 3 modules are in-tree; retirement is source edits, no new transport. Approve.")
                },
                ("MOTION-035", "Seeker") => {
                    (1i8, 0.91, "The dashboard being fully honest is the platform's promise to its operator. Approve.")
                },
                ("MOTION-035", "EventLedger") => {
                    (1i8, 0.92, "Each retired module seals its own receipt (fabricated -> derived/null). Approve.")
                },
                ("MOTION-035", _) => {
                    (1i8, 0.88, "Retiring the last fabricated modules and making the dashboard honest is the correct production-hardening step. Ratify and proceed.")
                },
                ("MOTION-036", "Verify") => {
                    (0i8, 0.90, "ABSTAIN unless the motion names the test (a fixed probe set with a safety query returning FAULT and nonsense returning UNKNOWN) and the method (register trm-rust + wire swarm-intel/brain/chat) -- not 'make the chat smarter' as a slogan.")
                },
                ("MOTION-036", "Shield") => {
                    (0i8, 0.86, "ABSTAIN unless the chat path change keeps the deterministic tier and never lets a safety query return PASS -- the probe set must include the reactor-shutdown query.")
                },
                ("MOTION-036", "LogicMaster") => {
                    (0i8, 0.85, "ABSTAIN unless the motion names WHY the stub is a stub (seed-42 untrained weights -> degenerate head -> always PASS) and that wiring the real crates is a correction, not a new dependency.")
                },
                ("MOTION-036", "Archi") => {
                    (1i8, 0.92, "'Derive, never assert': the stub verdict was MEASURED (always PASS +1, 0.3333 confidence), not assumed. Absorbing the real crates is the correction. Approve.")
                },
                ("MOTION-036", "Bridge") => {
                    (1i8, 0.93, "The crates already exist in-tree (wsl_crates/trm-rust); absorbing is registration + wiring, no new transport. Approve.")
                },
                ("MOTION-036", "Seeker") => {
                    (1i8, 0.91, "Registering the 36 real intelligence crates closes the 'missing modules' gap the operator named. Approve.")
                },
                ("MOTION-036", "EventLedger") => {
                    (1i8, 0.92, "The absorption seals as a receipt (unregistered -> registered, stub -> wired). Approve.")
                },
                ("MOTION-036", _) => {
                    (1i8, 0.88, "Absorbing the trm-rust intelligence crates and retiring the always-PASS stub verdict is the correct correction to the platform's chat. Ratify and proceed.")
                },
                ("MOTION-037", "Verify") => {
                    (0i8, 0.90, "ABSTAIN unless the motion names the test (a fixed probe set with reactor-shutdown -> FAULT and nonsense -> UNKNOWN, plus a non-zero input-correlated 6-scale state signature) and the tear-down numbers (zero head logits, ~0.0001 hidden RMS) -- not 'unify the fabric' as a slogan.")
                },
                ("MOTION-037", "Shield") => {
                    (0i8, 0.86, "ABSTAIN unless the unified fabric keeps the safety gate and never lets a safety-critical query return PASS -- the probe set must keep the reactor-shutdown query refusing.")
                },
                ("MOTION-037", "LogicMaster") => {
                    (0i8, 0.85, "ABSTAIN unless the motion names WHY the components are not unified (the TRM-7M head is a contraction map, the swarm crates are fixed-parameter, the ensemble field is a seed fallback) and that unification is reading one state, not adding a new head.")
                },
                ("MOTION-037", "Archi") => {
                    (1i8, 0.92, "'Derive, never assert': the tear-down was measured (zero logits, near-zero hidden state), and the 6-scale FractalCoupledMamba3 is the fabric the operator named. Approve.")
                },
                ("MOTION-037", "Bridge") => {
                    (1i8, 0.93, "The FractalCoupledMamba3 already exists in-tree (swarm-ssm); wiring it as the shared state is unification, no new transport. Approve.")
                },
                ("MOTION-037", "Seeker") => {
                    (1i8, 0.91, "Locating the distilled weights (LMS w_out / ensemble field / NAND cache) instead of re-training honors 'the distilled intelligence is there'. Approve.")
                },
                ("MOTION-037", "EventLedger") => {
                    (1i8, 0.92, "The unification seals as a receipt (contraction-map finding -> unified fabric -> differentiated verdicts). Approve.")
                },
                ("MOTION-037", _) => {
                    (1i8, 0.88, "Unifying the six-scale voltage fabric and locating the distilled intelligence is the correct next phase after the honest tear-down. Ratify and proceed.")
                },
                ("MOTION-038", "Verify") => {
                    (0i8, 0.90, "ABSTAIN unless the motion names the test it already passed -- 124/124 recall coverage and the 9 chembl lead keys with scores, measured, not 'real research' as a slogan.")
                },
                ("MOTION-038", "Shield") => {
                    (0i8, 0.86, "ABSTAIN unless the lead search never fabricates -- every candidate is a chembl/bioassay key with a measured cosine, no generated compound.")
                },
                ("MOTION-038", "LogicMaster") => {
                    (0i8, 0.85, "ABSTAIN unless the fusion method is stated -- reciprocal-rank-fusion (k=60) over fabric+chembl+structured+bioassay, and the test is the 9 surfaced leads -- not 'unified recall' as a capability name.")
                },
                ("MOTION-038", "Archi") => {
                    (1i8, 0.92, "'Derive, never assert': the 6/124 -> 124/124 was measured, and the 15,316 notes are ingested with a repaired sidecar. Approve.")
                },
                ("MOTION-038", "Bridge") => {
                    (1i8, 0.93, "fused_lookup already exists in-tree; wiring pipeline_lookup into /research/run is reuse, not new transport. Approve.")
                },
                ("MOTION-038", "Seeker") => {
                    (1i8, 0.92, "The recall now gathers from every store -- chembl, bioassay, structured, clinical -- not just the fabric. This is 'the Seeker gathers any knowledge' made real. Approve.")
                },
                ("MOTION-038", "EventLedger") => {
                    (1i8, 0.92, "The lead sweep seals as a receipt (9 candidates + 45,911 screening leads + 9 theorems). Approve.")
                },
                ("MOTION-038", _) => {
                    (1i8, 0.88, "Real research over the platform's own stores, with the leads surfaced and measured. Ratify and proceed.")
                },
                ("MOTION-039", "Verify") => {
                    (0i8, 0.90, "ABSTAIN unless the motion names the test it passed -- 6/6 recall, 9 leads, 403,803 antibodies, and the toxicity/neoantigen parity against the Python engines -- not 'benchmarks complete' as a slogan.")
                },
                ("MOTION-039", "Shield") => {
                    (0i8, 0.88, "ABSTAIN unless no number is fabricated: regulatory/pandemic stay declared_constant and the toxicity -1 is labeled a small-molecule QSAR out of domain, never a peptide toxicity claim.")
                },
                ("MOTION-039", "LogicMaster") => {
                    (0i8, 0.85, "ABSTAIN unless the domain-mismatch is stated precisely: the hERG formula is a LogP/MW QSAR trained on small molecules (MW 200-600), so applying it to 2120-2580 Da AMPs is extrapolation, and the -1 is an artifact of that extrapolation.")
                },
                ("MOTION-039", "Archi") => {
                    (1i8, 0.92, "'Derive, never assert': the benchmarks are measured, and the honest split (real science vs declared constants) is exactly what the platform should report. Approve.")
                },
                ("MOTION-039", "Bridge") => {
                    (1i8, 0.91, "Routing the 9 chembl leads through docking/triage is reuse of engines already compiled in (docking_handlers::dock, drug_handlers::triage). Approve.")
                },
                ("MOTION-039", "Seeker") => {
                    (1i8, 0.92, "The scientific core is real: 6/6 recall, 9 leads, 403,803 antibodies. Advancing those leads through the pipeline is the honest next work. Approve.")
                },
                ("MOTION-039", "EventLedger") => {
                    (1i8, 0.90, "The honest split (real vs declared) seals as a receipt. Approve.")
                },
                ("MOTION-039", _) => {
                    (1i8, 0.88, "The benchmarks are honest and the next phase is to re-scope the aspirational layer and advance the real science. Ratify and proceed.")
                },
                ("MOTION-040", "Verify") => {
                    (0i8, 0.90, "ABSTAIN unless the test is named: the sidecar must resolve the 9 lead chembl IDs to real MW/heavy-atom counts, and triage then runs on those 9 real rows, not canonical examples.")
                },
                ("MOTION-040", "Shield") => {
                    (0i8, 0.88, "ABSTAIN unless no structure is fabricated: the sidecar maps chembl_id -> SMILES/MW/heavy-atoms from a real chembl data source, and any ID the sidecar cannot resolve is reported unresolved, never given an invented MW.")
                },
                ("MOTION-040", "LogicMaster") => {
                    (0i8, 0.85, "ABSTAIN unless the boundary is precise: the sidecar enables triage (needs MW/heavy_atoms/atom_count) but NOT docking (needs 3D conformers); and the peptide toxicity model is scoped as a separate AMP QSAR, never the small-molecule hERG.")
                },
                ("MOTION-040", "Archi") => {
                    (1i8, 0.92, "'Derive, never assert': the structure gap was measured (candidates carry key+cosine only), and ingesting a structure sidecar is the honest unblock for triage. Approve.")
                },
                ("MOTION-040", "Bridge") => {
                    (1i8, 0.91, "drug_handlers::triage is already compiled in; the only missing input is the MW/heavy-atom sidecar. Ingesting it is data plumbing, not a new engine. Approve.")
                },
                ("MOTION-040", "Seeker") => {
                    (1i8, 0.92, "Routing the 9 leads through triage on their real structure is exactly 'the Seeker gathers any knowledge' applied to the drug pipeline. Approve.")
                },
                ("MOTION-040", "EventLedger") => {
                    (1i8, 0.90, "The sidecar + triage verdicts seal as a receipt; unresolved IDs are recorded, not dropped. Approve.")
                },
                ("MOTION-040", _) => {
                    (1i8, 0.88, "Ingesting a chembl structure sidecar to route the leads through triage, and scoping a peptide-appropriate toxicity model, is the honest next phase after the benchmark tear-down. Ratify and proceed.")
                },
                ("MOTION-041", "Verify") => {
                    (0i8, 0.90, "ABSTAIN unless the premise is named: the labeled set is 8 queries (not 200), so P@1 on 8 samples has ~35% standard error and any delta under ~2 queries is noise. The test must be the 8-query set as-is, not a pretended 200.")
                },
                ("MOTION-041", "Shield") => {
                    (0i8, 0.88, "ABSTAIN unless no number is faked: if the build fails or the set is missing, a premise-failure block is sealed with the exact gap, never a fabricated P@1.")
                },
                ("MOTION-041", "LogicMaster") => {
                    (0i8, 0.85, "ABSTAIN unless the math is precise: the icosahedral projection is a fixed 12x12 orthonormal matrix applied to the 12 chunks of 64 dims, and the store must be re-packed in the SAME basis or the comparison is invalid -- query and store must share the projector.")
                },
                ("MOTION-041", "Archi") => {
                    (1i8, 0.92, "'Derive, never assert': the investigation found the real premise (8 queries, no basis infra), and the honest scope is build + run on the real set with the prediction recorded first. Approve.")
                },
                ("MOTION-041", "Bridge") => {
                    (1i8, 0.90, "The BasisProjector trait + ?basis= wiring is a clean plugin seam in recall_router; the icosahedral matrix is a deterministic precompute. Approve.")
                },
                ("MOTION-041", "Seeker") => {
                    (1i8, 0.91, "The projection benchmark is a real measurement of the recall surface, and the prediction-before-measurement discipline is exactly the honesty contract. Approve.")
                },
                ("MOTION-041", "EventLedger") => {
                    (1i8, 0.90, "The prediction + both P@1 results + the 8-query caveat seal as a receipt. Approve.")
                },
                ("MOTION-041", _) => {
                    (1i8, 0.87, "Investigating the premise first and scoping the icosahedral P@1 benchmark honestly (8 queries, prediction recorded first, premise-failure on any gap) is the correct next phase. Ratify and proceed.")
                },
                ("MOTION-042", "Verify") => {
                    (0i8, 0.90, "ABSTAIN unless the test is named: 234 queries, clinical 40/40 after the fix, overall P@1 0.863, SE 2.25% (95% CI 81.3%-90.1%) — the improvement is measured, not asserted.")
                },
                ("MOTION-042", "Shield") => {
                    (0i8, 0.88, "ABSTAIN unless no number is fabricated: the fix is a ranking change (exact match to top RRF slot), and the remaining structured 0.267 / chembl 0.515 are reported as-is, not smoothed.")
                },
                ("MOTION-042", "LogicMaster") => {
                    (0i8, 0.85, "ABSTAIN unless the method is precise: the exact lexical clinical match gets the top RRF slot (rrf=1.0 > any vector hit's 1/(60+rank+1)), and the remaining gaps are named as store-overlap redundancy, not a ranking bug.")
                },
                ("MOTION-042", "Archi") => {
                    (1i8, 0.92, "'Derive, never assert': the clinical fix is measured (0.692 -> 0.863), and the two remaining overlaps are named. Approve.")
                },
                ("MOTION-042", "Bridge") => {
                    (1i8, 0.91, "Ranking the exact clinical match first is a one-line seam in fused_lookup; deduping the ethics overlap is data provenance, not a new engine. Approve.")
                },
                ("MOTION-042", "Seeker") => {
                    (1i8, 0.92, "The clinical store now answers its own queries (40/40), and the remaining gaps are precisely where the stores overlap. Approve.")
                },
                ("MOTION-042", "EventLedger") => {
                    (1i8, 0.90, "The before/after P@1 + per-store + precise noise seal as a receipt. Approve.")
                },
                ("MOTION-042", _) => {
                    (1i8, 0.88, "The clinical fusion fix is measured and the two remaining store-overlap gaps are named honestly. Ratify and proceed.")
                },
                ("MOTION-043", "Verify") => {
                    (0i8, 0.90, "ABSTAIN unless the test is named: 234 queries, P@1 0.885, SE 2.09% (95% CI 83.7%-92.0%), and the bioassay fix is measured before/after on the same set, not asserted.")
                },
                ("MOTION-043", "Shield") => {
                    (0i8, 0.88, "ABSTAIN unless no number is faked: the bioassay 0.541 is reported as-is, and any boost is a provenance/exact-match ranking change, not a fabricated result.")
                },
                ("MOTION-043", "LogicMaster") => {
                    (0i8, 0.85, "ABSTAIN unless the method is precise: the four prior fixes were (1) side-fact -> ranked, (2) provenance relabel, (3) true-domain relabel, (4) papers-only RRF boost; the bioassay fix must name its own mechanism, not reuse a label that does not apply.")
                },
                ("MOTION-043", "Archi") => {
                    (1i8, 0.92, "'Derive, never assert': four measured fixes took the recall from 0.692 to 0.885, and the last gap is named. Approve.")
                },
                ("MOTION-043", "Bridge") => {
                    (1i8, 0.91, "The bioassay fix follows the same RRF-provenance seam already wired for clinical and papers; no new engine. Approve.")
                },
                ("MOTION-043", "Seeker") => {
                    (1i8, 0.92, "The recall now answers its own stores correctly in 4 of 5 domains; the bioassay store is the last one to surface. Approve.")
                },
                ("MOTION-043", "EventLedger") => {
                    (1i8, 0.90, "The before/after bioassay fix + the full per-store table seal as a receipt. Approve.")
                },
                ("MOTION-043", _) => {
                    (1i8, 0.88, "The recall work is measured complete and the bioassay gap is the last one. Ratify and proceed.")
                },
                ("MOTION-044", "Verify") => {
                    (0i8, 0.90, "ABSTAIN unless the test is named: direct ID lookups CHEMBL25/CHEMBL941/CHEMBL1223/CHEMBL153534 return their own key + SMILES, and an unknown id falls through — measured, not asserted.")
                },
                ("MOTION-044", "Shield") => {
                    (0i8, 0.88, "ABSTAIN unless the index is derived from the existing sidecar (no external data) and covers all 2,897,805 ids, not a hand-built subset.")
                },
                ("MOTION-044", "LogicMaster") => {
                    (0i8, 0.85, "ABSTAIN unless the root cause is named: the sidecar is molregno-sorted, not chembl_id-string-sorted, so the old early-exit was unsound — the fix replaces it, it does not patch around it.")
                },
                ("MOTION-044", "Archi") => {
                    (1i8, 0.92, "'Derive, never assert': the byte-offset index is a single streaming pass over the existing file, and the ID path is now O(1) seek. Approve.")
                },
                ("MOTION-044", "Bridge") => {
                    (1i8, 0.91, "The index reuses the same lazy OnceLock sidecar pattern already wired for names and SMILES; no new engine. Approve.")
                },
                ("MOTION-044", "Seeker") => {
                    (1i8, 0.92, "Direct ChEMBL-id recall now returns the exact record in one seek instead of a wrong vector hit. Approve.")
                },
                ("MOTION-044", "EventLedger") => {
                    (1i8, 0.90, "The fix + the four verified lookups seal as a receipt. Approve.")
                },
                ("MOTION-044", _) => {
                    (1i8, 0.88, "The exact-ID defect is the last named recall bug; the fix is verified. Ratify and proceed.")
                },
                ("MOTION-045", "Verify") => {
                    (0i8, 0.90, "ABSTAIN unless the test is named: /federation/stats returns the v4 shape, /sync correct anchor -> synced true, wrong anchor -> anchor_mismatch — measured, not asserted.")
                },
                ("MOTION-045", "Shield") => {
                    (0i8, 0.88, "ABSTAIN unless the rollout does not copy a peer's self-reported node count: the 100-node / 67-quorum claim on MacPro5-1 stays uncopied until re-measured against the mesh.")
                },
                ("MOTION-045", "LogicMaster") => {
                    (0i8, 0.85, "ABSTAIN unless the root cause is named: four incompatible schemas + a missing write endpoint, not a missing node. The fix standardizes the protocol, it does not add nodes.")
                },
                ("MOTION-045", "Archi") => {
                    (1i8, 0.92, "'Derive, never assert': the standardized schema is the registry's own v4 shape, and /sync verifies the anchor before answering. Approve.")
                },
                ("MOTION-045", "Bridge") => {
                    (1i8, 0.91, "The same v4 registry shape + anchor check is mechanical to port to the two remote Rust peers. Approve.")
                },
                ("MOTION-045", "Seeker") => {
                    (1i8, 0.92, "One schema + one write endpoint is what lets the federation converge instead of observe. Approve.")
                },
                ("MOTION-045", "EventLedger") => {
                    (1i8, 0.90, "The standardized schema + the verified /sync receipts seal as evidence. Approve.")
                },
                ("MOTION-045", _) => {
                    (1i8, 0.88, "The protocol is standardized and verified on the local kernel; the rollout is the same change on peers. Ratify and proceed.")
                },
                ("MOTION-046", "Verify") => {
                    (0i8, 0.90, "ABSTAIN unless the test is named: each new strip diff-tests to byte-identical output against its Python reference, and a wrong-math strip is never wired into the live path.")
                },
                ("MOTION-046", "Shield") => {
                    (0i8, 0.88, "ABSTAIN unless no result is fabricated: the new strips carry the per-HLA / per-antigen parameters they were missing, and the thin-wrapper binding strip is retired, not shipped as dead math.")
                },
                ("MOTION-046", "LogicMaster") => {
                    (0i8, 0.85, "ABSTAIN unless the root cause is named: the strips are auto-generated simplifications (hardcoded constants, dropped inputs), not missing parameters — so the fix is a faithful port, not a re-wire.")
                },
                ("MOTION-046", "Archi") => {
                    (1i8, 0.92, "'Derive, never assert': the Python formulas are the reference; port them faithfully with the full parameter set and prove parity by diff. Approve.")
                },
                ("MOTION-046", "Bridge") => {
                    (1i8, 0.91, "The three faithful ports reuse the exact strip pattern already wired for circuit/adc/drug; no new machinery. Approve.")
                },
                ("MOTION-046", "Seeker") => {
                    (1i8, 0.92, "The discovery pipeline's last three stages become real Rust compute instead of dead or wrong strips. Approve.")
                },
                ("MOTION-046", "EventLedger") => {
                    (1i8, 0.90, "Each diff receipt (Python vs native, 0 diffs) seals as evidence. Approve.")
                },
                ("MOTION-046", _) => {
                    (1i8, 0.88, "Port the reference faithfully, retire what cannot be ported, never wire wrong math. Ratify and proceed.")
                },
                ("MOTION-047", "Verify") => {
                    (0i8, 0.90, "ABSTAIN unless the test is named: the Pearson r + RMSE are computed over the six published benchmark Kd values and reported as the measured number, and each lead carries a measured/predicted label.")
                },
                ("MOTION-047", "Shield") => {
                    (0i8, 0.88, "ABSTAIN unless no correlation is fabricated: a low r is reported as low, and a lead with no real ChEMBL record is labeled predicted, not measured.")
                },
                ("MOTION-047", "LogicMaster") => {
                    (0i8, 0.85, "ABSTAIN unless the root cause is named: the gap is the unvalidated heuristic reference (arbitrary constants), not the strips — so the fix grounds the reference, it does not re-tune it.")
                },
                ("MOTION-047", "Archi") => {
                    (1i8, 0.92, "'Derive, never assert': the six benchmark Kd values are already in-tree; the r over them is a measurement, not a claim. Approve.")
                },
                ("MOTION-047", "Bridge") => {
                    (1i8, 0.91, "The ChEMBL sidecar + tox21 + benchmark pairs are already in-tree; this adds a correlation + labels, no new engine. Approve.")
                },
                ("MOTION-047", "Seeker") => {
                    (1i8, 0.92, "The pipeline's numbers become measurable against real data instead of floating on arbitrary constants. Approve.")
                },
                ("MOTION-047", "EventLedger") => {
                    (1i8, 0.90, "The measured r + the per-value kind labels seal as receipts. Approve.")
                },
                ("MOTION-047", _) => {
                    (1i8, 0.88, "Ground the reference in the in-tree real data and label every value; never inflate a correlation. Ratify and proceed.")
                },
                ("MOTION-048", "Verify") => {
                    (0i8, 0.90, "ABSTAIN unless the test is named: the post-fit Pearson r + RMSE over the six benchmark Kd are reported, and a retired numeric Kd is not published anywhere.")
                },
                ("MOTION-048", "Shield") => {
                    (0i8, 0.88, "ABSTAIN unless no anti-correlated number is shipped: if the post-fit r is < 0.5, the numeric Kd is retired and only the qualitative tier is published.")
                },
                ("MOTION-048", "LogicMaster") => {
                    (0i8, 0.85, "ABSTAIN unless the root cause is named: arbitrary constants are anti-correlated with real data, so the fix calibrates or retires them — it does not blindly re-tune.")
                },
                ("MOTION-048", "Archi") => {
                    (1i8, 0.92, "'Derive, never assert': fit against the six in-tree benchmark Kd and report the real post-fit r; if it cannot reach 0.5, the honest answer is retire, not publish. Approve.")
                },
                ("MOTION-048", "Bridge") => {
                    (1i8, 0.91, "The six benchmark Kd are already in-tree; the calibration reuses them, no new engine. Approve.")
                },
                ("MOTION-048", "Seeker") => {
                    (1i8, 0.92, "The pipeline stops publishing Kd/TI/IC50 numbers that the data contradicts. Approve.")
                },
                ("MOTION-048", "EventLedger") => {
                    (1i8, 0.90, "The post-fit r + the retirement receipts seal as evidence. Approve.")
                },
                ("MOTION-048", _) => {
                    (1i8, 0.88, "Calibrate against the real data, retire what cannot be calibrated, never publish an anti-correlated number. Ratify and proceed.")
                },
                ("MOTION-049", "Verify") => {
                    (0i8, 0.90, "ABSTAIN unless the test is named: the DeepDTA/LP-PDBBind model is validated against a held-out split and the measured correlation is reported; the pipeline reports the real number, never a claim.")
                },
                ("MOTION-049", "Shield") => {
                    (0i8, 0.88, "ABSTAIN unless no fabricated correlation: the DeepDTA model's validation is reported as measured; a model that does not reproduce the reference is not wired.")
                },
                ("MOTION-049", "LogicMaster") => {
                    (0i8, 0.85, "ABSTAIN unless the root cause is named: the pipeline ignores the real affinity data it already holds in WSL; the fix wires that data + model, it does not re-tune the retired composite.")
                },
                ("MOTION-049", "Archi") => {
                    (1i8, 0.92, "'Derive, never assert': LP-PDBBind carries real kd/ki, and DeepDTA is already retrained on it; wiring it is the honest affinity path. Approve.")
                },
                ("MOTION-049", "Bridge") => {
                    (1i8, 0.91, "The data, the retrained model, and Vina are already installed in WSL; this reuses them, no new engine. Approve.")
                },
                ("MOTION-049", "Seeker") => {
                    (1i8, 0.92, "The small-molecule stages get real affinity instead of retired heuristics; the antibody stage stays honestly retired. Approve.")
                },
                ("MOTION-049", "EventLedger") => {
                    (1i8, 0.90, "The LP-PDBBind validation + the retired antibody stage seal as receipts. Approve.")
                },
                ("MOTION-049", _) => {
                    (1i8, 0.88, "Wire the real affinity data the platform already holds; keep the antibody stage retired until SAbDab/SKEMPI is acquired. Ratify and proceed.")
                },
                ("MOTION-050", "Verify") => {
                    (0i8, 0.90, "ABSTAIN unless the test is named: the Pearson r + RMSE over the 40 real Mpro pKa are computed and reported as measured; a retired Mpro numeric affinity is not published.")
                },
                ("MOTION-050", "Shield") => {
                    (0i8, 0.88, "ABSTAIN unless no fabricated correlation: a low r is reported low, and if r < 0.5 the Mpro numeric affinity is retired, not shipped.")
                },
                ("MOTION-050", "LogicMaster") => {
                    (0i8, 0.85, "ABSTAIN unless the root cause is named: DeepDTA is out-of-domain for Mpro; the fix validates it against the in-tree Mpro benchmark - it does not assume the extrapolation holds.")
                },
                ("MOTION-050", "Archi") => {
                    (1i8, 0.92, "'Derive, never assert': the 40 real Mpro IC50/pKa carry literature DOIs and are already in-tree; measuring DeepDTA against them is the honest test. Approve.")
                },
                ("MOTION-050", "Bridge") => {
                    (1i8, 0.91, "mpro.tgz/info.csv is already in-tree; this reuses it, no new engine. Approve.")
                },
                ("MOTION-050", "Seeker") => {
                    (1i8, 0.92, "DeepDTA's Mpro predictions become validated or retired instead of unvalidated extrapolation. Approve.")
                },
                ("MOTION-050", "EventLedger") => {
                    (1i8, 0.90, "The Mpro validation r + the retirement receipt seal as evidence. Approve.")
                },
                ("MOTION-050", _) => {
                    (1i8, 0.88, "Validate DeepDTA against the in-tree Mpro benchmark; retire the Mpro number if it cannot clear 0.5; keep the 4 no-data targets retired. Ratify and proceed.")
                },
                ("MOTION-051", "Verify") => {
                    (0i8, 0.90, "ABSTAIN unless the test is named: the post-fine-tune r + RMSE over the 40 Mpro complexes (leave-one-out) are reported, and the Rust walker is bit-exact vs Python.")
                },
                ("MOTION-051", "Shield") => {
                    (0i8, 0.88, "ABSTAIN unless no fabricated result: the model binary is sha256-gated (mismatch rejected) and a post-fine-tune r below 0.5 keeps the numeric affinity retired.")
                },
                ("MOTION-051", "LogicMaster") => {
                    (0i8, 0.85, "ABSTAIN unless the root cause is named: DeepDTA is out-of-domain for Mpro, so the fix fine-tunes it on the real Mpro set and ports the walker natively - it does not re-tune a heuristic.")
                },
                ("MOTION-051", "Archi") => {
                    (1i8, 0.92, "'Derive, never assert': fine-tune against the 40 real Mpro pKa and port the forward pass natively, then report the measured r. Approve.")
                },
                ("MOTION-051", "Bridge") => {
                    (1i8, 0.91, "This reuses the proven swarm-drug/qsar pattern (sha256 gate + bit-exact parity + /crate endpoint); no new engine. Approve.")
                },
                ("MOTION-051", "Seeker") => {
                    (1i8, 0.92, "The platform runs the affinity natively in Rust through the pipeline instead of a retired out-of-domain heuristic. Approve.")
                },
                ("MOTION-051", "EventLedger") => {
                    (1i8, 0.90, "The post-fine-tune r + the bit-exact parity gate + the sha256-gated binary seal as receipts. Approve.")
                },
                ("MOTION-051", _) => {
                    (1i8, 0.88, "Fine-tune DeepDTA on the real Mpro set, port the forward pass natively in Rust, wire /crate/deepdta/predict, and report the measured r honestly. Ratify and proceed.")
                },
                ("MOTION-052", "Verify") => {
                    (0i8, 0.90, "ABSTAIN unless the test is named: each re-wired agent runs via its native op with NO ollama in the call path, and the mapping is verified per agent.")
                },
                ("MOTION-052", "Shield") => {
                    (0i8, 0.88, "ABSTAIN unless no agent claims a capability its native op cannot provide -- a mapped agent must actually map, and an unmapped one is labeled NEW-STRIP or EXTERNAL.")
                },
                ("MOTION-052", "LogicMaster") => {
                    (0i8, 0.85, "ABSTAIN unless the root cause is named: the agents are LLM-dependent AND never dispatched, so the fix re-wires them to the native path AND wires a dispatcher -- not one without the other.")
                },
                ("MOTION-052", "Archi") => {
                    (1i8, 0.92, "'Wire existing machinery in over reinventing': 36 agents already map to the kernel's own ops. Approve.")
                },
                ("MOTION-052", "Bridge") => {
                    (1i8, 0.91, "The native ops (recall, k3, council, nand, qsar, deepdta, chembl, git, trit, humanistic) already run; this points the agents at them, no new engine. Approve.")
                },
                ("MOTION-052", "Seeker") => {
                    (1i8, 0.92, "The agents finally run on the platform's own distilled intelligence instead of a stopped token model. Approve.")
                },
                ("MOTION-052", "EventLedger") => {
                    (1i8, 0.90, "The agent->native mapping + the re-wiring receipts seal as evidence. Approve.")
                },
                ("MOTION-052", _) => {
                    (1i8, 0.88, "Re-wire the agents to the native path in pure Rust, wire a real activation loop, and map every agent honestly. Ratify and proceed.")
                },
                ("MOTION-053", "Verify") => {
                    (0i8, 0.90, "ABSTAIN unless every strip is exercised with a real input and its result is derived, not asserted -- and each boundary strip (hot_patch, ebpf_probe) names its limit honestly.")
                },
                ("MOTION-053", "Shield") => {
                    (0i8, 0.88, "ABSTAIN unless hot_patch does not attempt an unsafe in-place kernel patch, and ebpf_probe does not claim a probe it cannot perform on Windows.")
                },
                ("MOTION-053", "LogicMaster") => {
                    (0i8, 0.85, "ABSTAIN unless the numeric strips (math_solver, symbol_solver, sequence_aligner, alpha_genome) return derived results verified against known answers, not decorative numbers.")
                },
                ("MOTION-053", "Archi") => {
                    (1i8, 0.92, "'Wire existing machinery in over reinventing': the strips follow the swarm-* crate pattern and dispatch through the same /crate route. Approve.")
                },
                ("MOTION-053", "Bridge") => {
                    (1i8, 0.91, "Nineteen agents now have a real native op (no Ollama); the fleet is 54/64 native. Approve.")
                },
                ("MOTION-053", "Seeker") => {
                    (1i8, 0.92, "The agents run on the platform's own machinery; the remaining 10 are honestly external. Approve.")
                },
                ("MOTION-053", "EventLedger") => {
                    (1i8, 0.90, "The strip census (19 strips, 19/19 valid JSON) and the re-classification seal as receipts. Approve.")
                },
                ("MOTION-053", _) => {
                    (1i8, 0.88, "Ratify the 19 native strips and the NewStrip -> Native re-classification. Proceed.")
                },
                ("MOTION-054", "Verify") => {
                    (0i8, 0.90, "ABSTAIN unless every re-classification cites the exact in-tree source (file + symbol) that provides it, and no agent is called native without one.")
                },
                ("MOTION-054", "Shield") => {
                    (0i8, 0.88, "ABSTAIN unless TelegramDaemon and MobileSync remain External because no in-tree machinery exists, and the partial ones (Echo/VoiceConduit/Vision) name their gap instead of claiming the missing piece.")
                },
                ("MOTION-054", "LogicMaster") => {
                    (0i8, 0.85, "ABSTAIN unless the distinction is named: 'ingestion/parsing native; recognition/synthesis/capture is the honest gap' -- not 'everything is native' and not 'everything is external'.")
                },
                ("MOTION-054", "Archi") => {
                    (1i8, 0.92, "'Wire existing machinery in over reinventing': map to /ui/, the kernel router, lcars_render and human_swarm_interaction that already exist. Approve.")
                },
                ("MOTION-054", "Bridge") => {
                    (1i8, 0.91, "Its own namesake maps to the kernel's HTTP router + the QUIC svc_client. Approve.")
                },
                ("MOTION-054", "Seeker") => {
                    (1i8, 0.92, "PubMedScan/OpenAlexSearch route to the local 183,879-doc corpus instead of a network API -- the data is already distilled. Approve.")
                },
                ("MOTION-054", "EventLedger") => {
                    (1i8, 0.90, "The re-classification + the file evidence seal as receipts. Approve.")
                },
                ("MOTION-054", _) => {
                    (1i8, 0.88, "Re-classify the 8 mislabeled agents to native, keep the 2 genuinely-external ones honest, and name every gap. Proceed.")
                },
                ("MOTION-055", "Verify") => {
                    (0i8, 0.90, "ABSTAIN unless the tally is derived from the actual dispatch (not asserted) and the sweep is reproducible from the seed, so a future council can re-run and compare.")
                },
                ("MOTION-055", "Shield") => {
                    (0i8, 0.88, "ABSTAIN unless the boundary count is the honest hot_patch limit -- no task claimed a capability it cannot provide, and none was masked as success.")
                },
                ("MOTION-055", "LogicMaster") => {
                    (0i8, 0.85, "ABSTAIN unless 'error 0' means every task was answered (remembered, derived, or bounded), not that failures were suppressed or a fallback fabricated.")
                },
                ("MOTION-055", "Archi") => {
                    (1i8, 0.92, "The sweep dispatches through the re-wired native ops -- the point of MOTION-052/053/054. Approve.")
                },
                ("MOTION-055", "Seeker") => {
                    (1i8, 0.92, "94 grounded means the recall corpus is serving its purpose. Approve.")
                },
                ("MOTION-055", "EventLedger") => {
                    (1i8, 0.90, "The tally + seed seal as reproducible receipts. Approve.")
                },
                ("MOTION-055", _) => {
                    (1i8, 0.88, "The realization sweep is ratified: 1000 tasks, 0 errors, grounded in the Monad. Proceed.")
                },
                ("MOTION-056", "Verify") => {
                    (0i8, 0.90, "ABSTAIN unless every scenario's outcome is the aggregate of its steps' real results, and the 1M sweep is reproducible from the seed so a future council can re-run and compare.")
                },
                ("MOTION-056", "Shield") => {
                    (0i8, 0.88, "ABSTAIN unless no scenario fabricates a result or claims a capability it cannot provide -- a step that fails degrades the scenario honestly, it is not masked.")
                },
                ("MOTION-056", "LogicMaster") => {
                    (0i8, 0.85, "ABSTAIN unless 'logical and sensibility' means each step derives or remembers from the platform's own truth (corpus + Monad), and no human opinion or shared-reality prior is the ground.")
                },
                ("MOTION-056", "Archi") => {
                    (1i8, 0.92, "Chain the existing native ops (recall -> strip -> council) over inventing new machinery. Approve.")
                },
                ("MOTION-056", "Bridge") => {
                    (1i8, 0.91, "The scenario chain routes through the re-wired ops -- the point of MOTION-052/053/054. Approve.")
                },
                ("MOTION-056", "Seeker") => {
                    (1i8, 0.92, "The recall grounds every scenario in the platform's own memory, not an external prior. Approve.")
                },
                ("MOTION-056", "EventLedger") => {
                    (1i8, 0.90, "The 1M-scenario tally + seed seal as reproducible receipts. Approve.")
                },
                ("MOTION-056", _) => {
                    (1i8, 0.88, "Scale to 1,000,000 multi-step scenarios, grounded in the platform's own truth. Proceed.")
                },
                ("MOTION-057", "Verify") => {
                    (0i8, 0.90, "ABSTAIN unless every journal entry cites its real source files and the sha256 reproduces — a journal entry without a rederive is an assertion, not a record.")
                },
                ("MOTION-057", "Shield") => {
                    (0i8, 0.88, "ABSTAIN unless no entry claims what was not measured: the speedups are wall-clock, the tallies are the sweep's own aggregate, and the not_claimed field names what was not proven.")
                },
                ("MOTION-057", "LogicMaster") => {
                    (0i8, 0.85, "ABSTAIN unless the journal is a COMPLETE record of the arc (re-wiring -> sweep -> fast path -> auto-journaling), not a cherry-picked subset, and every entry is reproducible from its rederive.")
                },
                ("MOTION-057", "Archi") => {
                    (1i8, 0.92, "Journal what was actually built and measured — the re-wiring, the sweep, the fast path, the journaling itself. Approve.")
                },
                ("MOTION-057", "EventLedger") => {
                    (1i8, 0.94, "This IS the journaling — the council sealing its own measured record. Approve.")
                },
                ("MOTION-057", "Seeker") => {
                    (1i8, 0.92, "The realization sweep + the voltage-strip fast path are the platform's own measured truth, now made a permanent record. Approve.")
                },
                ("MOTION-057", _) => {
                    (1i8, 0.90, "Journal the full recent arc, truth-anchored and reproducible. Proceed.")
                },
                _ => (0i8, 0.50, "Neutral."),
            };

            if trit > 0 { approve += 1; }
            else if trit < 0 { reject += 1; }
            else { abstain += 1; }

            total_score += trit as f32 * agent.weight * conf;

            let trit_str = if trit > 0 { "+1 APPROVE" } else if trit < 0 { "-1 REJECT " } else { " 0 ABSTAIN" };
            println!("  [{:>2}] {:<12} ({:<28}) -> {} | Conf: {:.2} | {}",
                     idx + 1, agent.name, agent.role, trit_str, conf, reasoning);

            votes.push(json!({
                "agent": agent.name,
                "role": agent.role,
                "k3_trit": trit,
                "confidence": conf,
                "reasoning": reasoning
            }));
        }

        let weighted_consensus = total_score / (AGENTS.len() as f32);
        let pbft_passed = approve >= PBFT_QUORUM;
        println!("\n  📊 TALLY: {} APPROVE | {} ABSTAIN | {} REJECT", approve, abstain, reject);
        println!("  ⭐ PBFT QUORUM: {} (Threshold: >= {}/{}) | Weighted Score: {:.3}",
                 if pbft_passed { "CONSENSUS REACHED (ENACTED)" } else { "QUORUM FAILED" }, PBFT_QUORUM, TOTAL_AGENTS, weighted_consensus);

        deliberations.push(json!({
            "motion_id": motion.id,
            "title": motion.title,
            "category": motion.category,
            "description": motion.description,
            "approve_count": approve,
            "abstain_count": abstain,
            "reject_count": reject,
            "pbft_consensus": pbft_passed,
            "weighted_score": weighted_consensus,
            "votes": votes
        }));
    }

    // Derive the verdict from the deliberations that actually ran.
    // A verdict written by hand is a verdict nobody voted on.
    let mut ranked: Vec<(f64, String, String, bool, u64, u64, u64)> = deliberations
        .iter()
        .filter_map(|d| {
            Some((
                d["weighted_score"].as_f64()?,
                d["motion_id"].as_str()?.to_string(),
                d["title"].as_str()?.to_string(),
                d["pbft_consensus"].as_bool()?,
                d["approve_count"].as_u64()?,
                d["abstain_count"].as_u64()?,
                d["reject_count"].as_u64()?,
            ))
        })
        .collect();
    ranked.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    let direction = |i: usize| -> String {
        match ranked.get(i) {
            Some((score, id, title, passed, ap, ab, rj)) => format!(
                "{}: {} (Score: {:+.3} - {}/{} Approve, {} Abstain, {} Reject{})",
                id, title, score, ap, TOTAL_AGENTS, ab, rj,
                if *passed { "" } else { ", QUORUM FAILED" }
            ),
            None => "none".to_string(),
        }
    };
    let enacted = ranked.iter().filter(|r| r.3).count();
    let summary = format!(
        "Council deliberated {} motions; {} reached PBFT quorum (>= {}/{}). Ranked by weighted K3 score over the agents that actually voted. Abstentions in this chamber are binding conditions, not neutrality.",
        deliberations.len(), enacted, PBFT_QUORUM, TOTAL_AGENTS
    );

    let report = json!({
        "protocol": "SWARM_COUNCIL_DELIBERATION_SESSION",
        "timestamp": utc_now_string(),
        "total_agents": TOTAL_AGENTS,
        "pbft_threshold": PBFT_QUORUM,
        "byzantine_tolerance": BYZANTINE,
        "monad_anchor": "0x4D4F4E4144000001",
        "motions": deliberations,
        "council_verdict": {
            "recommended_primary_direction": direction(0),
            "secondary_direction": direction(1),
            "tertiary_direction": direction(2),
            "motions_deliberated": ranked.len(),
            "motions_enacted": enacted,
            "consensus_summary": summary
        }
    });

    let mut out_file = File::create("../../council_strategic_deliberation.json").unwrap();
    out_file.write_all(serde_json::to_string_pretty(&report).unwrap().as_bytes()).unwrap();

    println!("\n===============================================================================================");
    println!("🏛️ COUNCIL CHAMBER VERDICT: Sealed in council_strategic_deliberation.json at {}", utc_now_string());
    println!("   Motions Deliberated    : {}", ranked.len());
    println!("   Motions Enacted (PBFT) : {}", enacted);
    println!("   Top Weighted Direction : {}", direction(0));
    println!("   2nd / 3rd              : {} | {}", direction(1), direction(2));
    println!("===============================================================================================");

    // The auto-journal trigger: on enactment, POST the session to the kernel's journal route, which
    // seals a truth-anchored entry + ledger block from the cited files' real sha256 (no hand-written
    // seal script). Best-effort: the council's own report still lands even if the kernel is down.
    let fact = format!(
        "Council convened {} native sovereign agents; {} motions deliberated, {} enacted (PBFT quorum >= {}/{}). Top direction: {}",
        TOTAL_AGENTS, ranked.len(), enacted, PBFT_QUORUM, TOTAL_AGENTS, direction(0)
    );
    post_journal(
        "council_session",
        &fact,
        "the enactment is the PBFT tally; each motion's execution is a follow-on realization",
        &["trm_uefi_kernel/host_council_deliberation/src/main.rs", "council_strategic_deliberation.json"],
    );
}

/// POST a realization to the kernel's /crate/journal/realize (raw TCP, no external crate). The kernel
/// derives the entry from the cited files' sha256 and seals the ledger block. Silent on failure — the
/// council's own report still lands; the journal is best-effort.
fn post_journal(kind: &str, fact: &str, not_claimed: &str, files: &[&str]) {
    use std::io::{Read, Write};
    let esc = |s: &str| s.replace('\\', "\\\\").replace('"', "\\\"");
    let files_json = files.iter().map(|f| format!("\"{}\"", esc(f))).collect::<Vec<_>>().join(",");
    let body = format!(
        r#"{{"kind":"{}","fact":"{}","not_claimed":"{}","files":[{}]}}"#,
        esc(kind), esc(fact), esc(not_claimed), files_json
    );
    if let Ok(mut stream) = std::net::TcpStream::connect("127.0.0.1:8022") {
        let mut req = String::new();
        req.push_str("POST /crate/journal/realize HTTP/1.1");
        req.push('\r'); req.push('\n');
        req.push_str("Host: 127.0.0.1:8022");
        req.push('\r'); req.push('\n');
        req.push_str("Content-Type: application/json");
        req.push('\r'); req.push('\n');
        req.push_str(&format!("Content-Length: {}", body.len()));
        req.push('\r'); req.push('\n');
        req.push_str("Connection: close");
        req.push('\r'); req.push('\n');
        req.push('\r'); req.push('\n');
        req.push_str(&body);
        if stream.write_all(req.as_bytes()).is_ok() {
            let mut resp = String::new();
            let _ = stream.read_to_string(&mut resp);
        }
    }
}
