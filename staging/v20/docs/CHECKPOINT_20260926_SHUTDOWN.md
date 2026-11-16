# CHECKPOINT — shutdown 2026-09-26 (blocks 164–170 sealed, step 3 triage in flight)

Written to be read cold: a fresh session with no conversation history must be able to resume from here alone.

## Sealed and verified — do NOT redo these

| what | measured | how to check |
|---|---|---|
| ledger | **170 blocks**, `prev_hash links broken: n=0`, canonical 160; UNVERIFIED = the pre-existing 97–101 only | `py -3 swarm_v2/core/verify_ledger_chain.py` |
| structured store | `swarm_v2_artifacts/structured_index.db` — **1,249,710,080 B**, sha256 `521ff968a5cb2d8c…`; tables: molecule 2,897,819 / molecule_features 2,897,818 / **tox21 577,987** / clinical 201,572 / ethics 95,847 / binding 19,443 / prose 1,633 / structure 20 / **span 12** / provenance 436 | `swarm_v2_artifacts/structured_index_swap_spandecl.json` |
| store history | three kept generations, each the predecessor of the next: `pre_span 212b0b1b…` → `pre_tox21 c347a050…` → `pre_spandecl 6d2ab79d…` → live `521ff968…` | `backups/structured_index_pre_*/` |
| knowledge index | live `semantic_index.json` 2,180 docs × 768 dims, **untouched**; harvested candidate **336 docs** (bare-metal corpora + the tox21 generator), not swapped | `swarm_v2_artifacts/kernel_knowledge_index_harvest.json` |
| tier-2 | **two** model roots, primary-first; running from `D:\trm-models\ternary-bonsai\Ternary-Bonsai-8B-Q2_0_g64.gguf` on `:8587`; fallback seeded with the verified 8B | `swarm_supervisor.py::discover_tier2_generator` |
| WSL reconciliation | 55 groups: **6 harvested / 3 harvested-prose / 3 harvest** (was 5/0/7) | `py -3 scratch/build_wsl_reconciliation.py` |
| council (step 3) | 7 motions enacted, MOTION-011…017, 15/0/0, quorum 11/15, native kernel bound | `swarm_v2_artifacts/council_step3_port_20260926.json`, sha256 `e1fb0689c67dc945…` |
| docs | README + `swarm_v2_artifacts/README.md` + the reconciliation docs updated and swept; every live doc cites the live store sha | `grep -rn 521ff968 README.md docs/` |

Blocks 164 (tier-2 second root), 165 (WSL step 1 closed), 166 (voltage instruments), 167 (knowledge corpora),
168 (tox21 into the store), 169 (the store's span declaration), 170 (council for step 3).

## In flight at shutdown: step 3's triage (MOTION-011)

Measured off WSL just before the shutdown — real, and partial:

| crate | `.rs` files | total lines | the plan claimed |
|---|---|---|---|
| swarm-brain | 8 | **682** | "338 lines" |
| swarm-intel | 4 | **350** | "263" |
| swarm-docking | 1 | **226** | "226" ✓ |
| swarm-drug | 2 | **222** | "176" |
| swarm-fit | 3 | **370** | "136" |
| swarm-proof | 1 | **134** | "134" ✓ |

Two of six match; four do not. The plan's figures are per-file or stale, so **do not re-quote them** — the
same class as "64 assays vs 75 columns". The stage AFTER this is not started: deciding for each crate whether
v19 already implements its capability. The rule is already sealed: **a live cargo path-dependency is the module
of record — wire it, never port a copy** (⇒ `superseded`), and only a crate with no counterpart gets `port`.

v19's Rust layout for that search: `trm_uefi_kernel/` (plus ~10 `host_*` crates incl. `host_council_server`,
`host_invariant_stress`, `host_continuous_ops`), `swarm_v2_rust_kernel/` (the DLL, `native_query_encoder.rs`),
`swarm_v2_artifacts/{Cargo.toml,port_tests/Cargo.toml}`, `swarm_v2_integrated/config/Cargo.toml`.

## Resume here (cold start)

1. **Bring the runtime back** (Ollama stays stopped): kernel `:8022` (`trm_uefi_kernel/host_council_server`),
   dashboard `:5183` (Vite dev), tier-2 `:8587` (llama-server, the `_g64` 8B).
2. **Finish MOTION-011's triage**, in this order:
   - in WSL, locate the six crates (`find ~ -maxdepth 4 -type d -name swarm-brain` …) and read each one's
     `pub fn` / `pub struct` surface — that is the capability, not the crate name;
   - for each capability, search v19 for a counterpart: `grep -rn "<capability>" swarm_v2/core swarm_v2_rust_kernel trm_uefi_kernel`
     and `grep -rn 'path = "' --include=Cargo.toml .`. That second measurement is **done and precise**:
     every hit lies inside the *harvested* bootloader crate (`swarm_v2/knowledge/rust_kb/bootloader-0.11.17/…`,
     its own `[[test]]` targets), and **no v19 crate declares a path dependency on another**. So the
     module-of-record test cannot be read off a dependency graph here — it must be applied per capability by
     name and behaviour. (For the record, `trm_uefi_kernel/Cargo.toml` depends on `r-efi = "4.5"` and nothing
     else; `[profile.dev] panic = "abort"` is required or the kernel is unbuildable outside `--release`.)
   - emit a verdict per crate with a machine-checkable claim: `superseded` ⇒ the counterpart path **must
     exist**; `port` ⇒ a grep shows **no** counterpart; `dead` ⇒ nothing references it —
   - enforce it in a builder that refuses to write unless every claim checks, then re-run it inside the sealer
     (the shape of `scratch/build_wsl_reconciliation.py`).
3. **Seal the triage** as a new block, then port only the `port` verdicts — each with a wired consumer, a
   sha256 receipt with a call proof, and a test red on the pre-port tree (MOTION-012). Retirement is gated on
   every crate's fate being recorded (MOTION-013), never before.

## Open decisions — the operator's, unchanged

- The live index's 1,994 un-rebuildable retrieval paths (91% of its docs are chunks of
  `relational/ethics_commonsense.csv`, whose source is not in the tree): re-chunk from the store / copy the
  14.5 MB source / accept the drop — with the cost recorded either way (MOTION-016).
- Whether to promote the 336-doc harvest index over the live `semantic_index.json`.
- Whether to expand the index across the 12,270-doc research corpus (~1.6 h at the measured 2.1 docs/s, ~300 MB).
- The React paint on `:5183` tab **31 NATIVE SURFACE** — needs a human eye; browser tooling cannot reach
  localhost from here.

Sealed as ledger **block 171** (`186c45ef659ebff9da43d290…`) immediately after this document was written: this
file states the state as it stood at writing (tip 170), and that block is the seal on it, moving the tip to
171. A cold reader should check the tip with `py -3 swarm_v2/core/verify_ledger_chain.py` rather than trust
either number.

## Two rules this checkpoint exists to protect

- **Dated audit records are records.** `docs/P4_INGEST_20260925.md`, `docs/FABRIC_REEMBED_20260923.md`, the
  `VALIDATION_*` files and `docs/MODEL_INVENTORY_20260926.md` state what was true on their date; do not rewrite
  them. Corrections belong in the live docs, which name them.
- **A declared number is not a measured one.** Every figure above was read off the artifact or the endpoint at
  the time of writing; if one disagrees with the artifact later, the artifact wins and the disagreement gets
  recorded.
