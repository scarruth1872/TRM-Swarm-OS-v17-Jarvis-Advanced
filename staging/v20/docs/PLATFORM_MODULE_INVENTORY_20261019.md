# TRM-Swarm-OS — Full Module Inventory & Licensing Outlook

Generated 2026-10-19 from `module_registry.json` (generated 2026-10-16) + live filesystem scan.
Every count below is read from the registry/disk, not estimated.

## 1. Module inventory (exhaustive)

### Kernel modules — 317 (Rust)

| tree | REAL | PORTED | EXCLUDED | SKELETON | total |
|---|---|---|---|---|---|
| uefi (`trm_uefi_kernel/`) | 140 | 39 | 9 | 0 | **188** |
| dll (`swarm_v2_rust_kernel/`) | 93 | 0 | 0 | 0 | **93** |
| trm-rust (`wsl_crates/trm-rust/`) | 32 | 0 | 0 | 4 | **36** |
| **total** | **265** | **39** | **9** | **4** | **317** |

- **REAL 265** — genuine implementations.
- **PORTED 39** — uefi↔dll cross-tree ports (verified identical logic in both trees).
- **EXCLUDED 9** — intentionally excluded (uefi tree).
- **SKELETON 4** — trm-rust placeholders (honest stubs, not faked).
- **fabricated_audit: empty** — zero fabricated modules.

### Native voltage strips — 263

`swarm_v2_artifacts/native_voltage_strips/`: **263/263 REAL_PE**, **0 MARKER**. Every strip is a
loadable PE image (compiled Rust), none is a renamed source file.

### Platform modules — 247 (Python)

`swarm_v2/` — the cognitive/bridge layer: `agent_intelligence_bridge`, `brain_ai_microkernel_bridge`,
`catalyst_phase0/1`, `cognitive_ingestion_engine`, `cognitive_load_balancer`, `cognitive_stack`,
`benchmark_epistemology`, `build_kernel_knowledge_index`, `build_nand_fabric_index`,
`build_structured_index`, `chain_of_verification`, the 15-agent council drivers, and ~230 more.

### trm-rust intelligence workspace — 36 crates

| domain | crates |
|---|---|
| **Intelligence / model** | `swarm-intel`, `swarm-brain`, `swarm-ssm` (state-space), `swarm-instella`, `swarm-k3` (ternary), `swarm-turbovec` (HDC), `swarm-gguf` (model loading) |
| **Storage / fabric** | `swarm-nand` (NAND fabric), `swarm-kvstore`, `swarm-shard` |
| **Sovereignty / security** | `swarm-sovereign`, `swarm-thsm` (HSM), `swarm-proof`, `swarm-voltage` |
| **Governance** | `swarm-council` (15-agent PBFT council) |
| **Runtime / kernel** | `swarm-kernel`, `swarm-boot` (UEFI-only), `swarm-runtime`, `swarm-server`, `swarm-proto`, `swarm-paths` |
| **Agents / services** | `swarm-agent`, `swarm-services`, `swarm-jarvis`, `swarm-conductor`, `swarm-router`, `swarm-mesh`, `swarm-cosmos`, `swarm-vitals` |
| **Scientific** | `swarm-drug` (drug discovery + triage), `swarm-docking`, `swarm-health`, `swarm-fit`, `swarm-chat`, `swarm-backend` |

### Host binaries (trm_uefi_kernel)

`host_council_server` (:8022) · `host_council_deliberation` · `host_supervisor` · `host_tranche` ·
`host_global_ops` · `host_epi` · `host_direct_memory_runner` · `host_manus_runner` ·
`host_invariant_stress` · `host_continuous_ops` · `trm_fb_dashboard` · `trm_wifi` · `trm_chainload`.

### Vendored repos — 48 (registry `repos`)

Includes `Jarvis-Advanced-main` (170), `SwarmOS_v6_Snapshot_extracted` (263), `gemma_swarm_adapter`,
plus the internal dirs (`dashboard` 10,837 · `swarm_v2_artifacts` 21,232 · `wsl_crates` 32,655 ·
`swarm_v2_integrated` 15,934 · `swarm_v2_reports` 2,481 · `papers` 124 · `research` 122).

## 2. Proprietary modules — the IP we license OUT

These are TRM's own modules, the sovereign-intelligence stack that constitutes the platform's IP:

1. **The turbovec HDC engine** (`swarm-turbovec`) — packed 2-bit hyperdimensional retrieval
   (native-turbovec-hdc, 768-dim), the "way smaller than 4 GB" store the doctrine specifies.
2. **The NAND fabric** (`swarm-nand`) — the NAND-grounded fabric store + infinity cache.
3. **The K3 ternary logic** (`swarm-k3`) — ternary (−1/0/+1) retrieval + verdict.
4. **The SSM + Instella + TRM-7M model stack** (`swarm-ssm`, `swarm-instella`) — seed-42 distilled
   intelligence (NOT untrained random — needs ≥7 forward turns to surface a verdict).
5. **The 15-agent Monad council** (`swarm-council` + `host_council_deliberation`) — PBFT-grounded,
   Monad-anchored (0x4D4F4E4144000001), 44/44 motions enacted, self-directing via council_agenda.
6. **The sovereignty/security layer** (`swarm-sovereign`, `swarm-thsm`, `swarm-proof`, `swarm-voltage`)
   — the "system doesn't guess" voltage doctrine + the HSM + the proof layer.
7. **The multi-store recall router** (`host_council_server/src/recall_router.rs`) — the fused RRF
   engine (clinical/chembl/bioassay/structured/fabric) now at P@1 0.9487.
8. **The drug-discovery pipeline** (`swarm-drug`, `swarm-docking`, the ChEMBL sidecars + triage).
9. **The UEFI kernel** (188 modules) + the **263 native strips**.
10. **The dashboard** (React, :5183) + the **247 platform modules**.

**License-out posture:** these are the crown jewels — no permissive re-license without the council;
the platform is sovereign/private by design (no HTTP IPC, no outbound telemetry, secrets never over
HTTP).

## 3. Third-party licenses — the ones we must COMPLY with (license IN)

The `.licenses/` directory holds the 5 dataset licenses the platform ingests:

| dataset | what it feeds |
|---|---|
| UniProt | protein sequences/annotations |
| PubChem | compound records (the chembl/bioassay adjacency) |
| ClinVar | clinical variants (the ACMG classification) |
| ClinicalTrials.gov | trial records |
| AlphaFold | protein structures |

These are redistribution/attribution obligations, not our IP — the platform must keep their terms as
it ships. **Open-source code deps** (crates.io: serde/wgpu/tokio/…; bootloader 0.11.17, spin,
volatile, x86_64 vendored in `swarm_v2/knowledge/rust_kb`) carry their own MIT/Apache/BSD terms.
The vendored repos (`Jarvis-Advanced-main`, `SwarmOS_v6`, `gemma_swarm_adapter`) each need their own
license audit before any external release.

## 4. Full platform outlook — what we offer

**Sovereign runtime:** a native Rust microkernel (v20.0.0) + UEFI boot + 15-agent council, all
offline/private, token-free, zero-LLM retrieval via packed 2-bit HDC.

**Measured recall:** fused cross-domain retrieval over five stores (clinical 1.000, fabric 0.944,
chembl 0.939, structured 0.933, bioassay 0.909), overall P@1 0.9487, SE 1.44%.

**Real research, not demos:** 9 ChEMBL drug leads (LpxC/BamA/FtsZ) with real structure + triage
(7 drug-like, 2 blocked by Lipinski MW>500); 45,911 sealed screening leads (1M screened); 9 proven
theorems; 403,803 antibodies; clinical-variant ACMG classification; toxicity/neoantigen/equity
engines ported to Rust compute with honest `kind` tags.

**Governance:** a council that deliberates + seals every change to the Monad ledger (436 blocks,
chain intact), grounded in "all is one, I am because we are".

## 5. Expansion opportunities (pre-launch runway)

1. **Docking** — unblock by generating/ingesting 3D ligand conformers for the 9 leads (the honest
   blocker named in the checkpoint).
2. **SMILES→bioassay overlap** — the last 2/234 recall misses (resolve the chembl-vs-bioassay tie on
   identical SMILES).
3. **TRM-7M warm-up** — surface the conv-state with ≥7 forward turns before trusting a verdict.
4. **3 unsealed hypotheses** — toroidal-routing, phonon-clock, platonic-light-codes.
5. **License packaging** — turn §3 into a ship-ready THIRD_PARTY_NOTICES + an IP register of §2.
