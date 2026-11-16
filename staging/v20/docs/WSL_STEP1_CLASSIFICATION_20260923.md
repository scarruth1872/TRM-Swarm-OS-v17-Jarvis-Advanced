> **UPDATE 2026-09-23**
> CORRECTED by WSL_CORRECTIONS_20260923.md: the sentence 'the reference is unrecoverable' is WRONG. The
> v18 reference `libswarm_kernel_ops.so` survives in the sibling v17 tree and exports every op named
> (`c_kleene_quantize_avx2`, `c_k3_kleene_ops`, `c_turbovec_quantize_2bit`, `c_fold_scan`, plus gguf/instella).
> The v18 claims were verified against it rather than retired.

# Step 1: WSL reconciliation — classification and first re-verification (2026-09-23)

## Step 0 (done, verified)
`trm-rust` committed (`89d2d75` on `29cb510`) and its working tree is now clean — 10 dirty items → 0.
Archived to `backups/wsl_freeze_20260923_180202/` (427 MB source tar, ex-`target/`), plus
`home_inventory.tsv` (1,260 entries), `knowledge_sha256.txt` (**509 files hashed**) and
`models_manifest.tsv`. The five at-risk files extract to real bytes with recorded hashes, and `.git`
(with the frozen commit) travels inside the archive. `FREEZE_MANIFEST.json` records all four artifacts.

## Classification

| WSL artifact | verdict | basis |
|---|---|---|
| 35 crates (`trm-rust`) | **mostly superseded** | v19 has Rust equivalents: `swarm-council` ↔ `council_multi_intelligence_deliberation_engine.rs`; UEFI `swarm-boot` (179 lines) ↔ `trm_uefi_kernel` (183 modules); `swarm-k3`/`swarm-nand`/`swarm-turbovec` ↔ v19's K3/fabric/TurboVec |
| 11 files asserting equivalence vs `libswarm_kernel_ops.so` | **re-verify, do not trust** | the reference is unrecoverable (below) |
| knowledge bases (`drugs/`, `datasets/`, `clinical/`, `docking/`, `hw_kb/`, `rust_kb/`, `tox21full/`) | **harvest** | v19's fabric holds 0 datasets; these are 830 MB+ of real, hashed data |
| `models/instella-gguf/…Q2_K.gguf` (6.5 GB) | **candidate** | sits inside the ~6.5 GB VRAM ceiling, unlike the Windows 10.47 GB copy that will not load |
| `models/bonsai-27b-q1_0.gguf` (3.8 GB) | candidate | a *different quant* from the Windows `_g64` set |
| `prism-ml-llama.cpp` (576 MB) | harvest as-is | the fork that runs `_g64` |
| `llama.cpp/`, `mamba-server/`, `voicebox/`, `aipto2/`, `relational/`, `anythingllm/`, `hermes-archive/`, `Agent-Q-master(-old)/` | **unclassified — needs a read** | names alone cannot decide; a name-based ruler has known blind spots |

## The finding that matters most: the v18 verification backbone is gone

Eleven files in the archive assert byte-equivalence against **`c_kleene_quantize_avx2` in
`libswarm_kernel_ops.so`** — `swarm-council` (+ its verify bin), `swarm-k3` (+ verify), `swarm-brain`,
`swarm-sovereign`, `swarm-agent`, `swarm-ssm`, `swarm-kernel`, and the MOTION itself.

That reference is unrecoverable: **0 C/C++ files exist anywhere in the archive**, no `libswarm*.so`
exists in WSL, and the Windows-built `libswarm_kernel_ops.dll` does not export
`c_kleene_quantize_avx2`. So every "byte-identical vs the live C reference" receipt in that
workstream is **currently unverifiable** — the claims are not false, they are unfalsifiable as shipped.

The way out is the surviving oracle: `swarm_v2/core/agent_intelligence_bridge.py` (which v19 still
has) and v19's own Rust collapse.

## First re-verification, completed here

Ran the v18 crate's **own rule** (`overlap` exact-token, abstain on 0, `v = [ov_sig, conf, conf]`,
`collapse` at eps 0.35, verdict by summed-trit sign) over two tables: its own 14-agent static mirror,
and the live oracle's 15 agents parsed straight out of `AGENT_MODELS`.

**Faithful — and this is a positive result.** Confidence **is** the documented `1.0 - loss`: the
operative lines read `confidence: 1.0 - 0.265` etc., and all 14 values match the oracle's losses
exactly. (My first pass reported all 14 as mismatches — my regex had captured `1.0` out of
`1.0 - 0.265`. Retracted; third harness-fabricated alarm this session, caught by printing the input.)

**Divergent — three ways.**
1. **An agent is missing.** The oracle has **15**; the port mirrors **14**. `LogicMaster` (domains
   `sat, smt, smt-lib, solver, z3, cvc5, clause, cnf`) was never ported.
2. **139 domain tokens differ.** 15 exist only in the v18 table — all of them acronyms it
   **upper-cased** (`CVE`, `DDR`, `ECG`, `ICU`, `FDA`, `CDC`, `JWST`, `QA`, `PubMed`, `IC50`, `GLP-1`,
   `K3`, `Coq`, `Isabelle`, `Lean4`) — and 124 exist only in the oracle. Since `overlap` is an exact
   token match, re-casing a token is a behavioural change, and `Isabelle`/`Lean4`/`K3` are additions
   with no oracle counterpart.
3. **Verdicts do flip.** On the crate's own two motions (A and E) both tables return **PROCEED**
   (sums 9 and 15) — so its §3 receipt is corroborated for those. But any motion touching a divergent
   token diverges: `{CDC, CVE, Coq}` → v18 PROCEED vs oracle ANALYZE; `{cve}` → v18 ANALYZE vs oracle
   PROCEED; `{qa, truth}` → v18 ANALYZE vs oracle PROCEED. The defect is latent on last meeting's
   motions and live on 139 tokens.

## Next actions, in order
1. Confirm which table the **live** council on :8022 uses, then reconcile the port against the oracle
   (add `LogicMaster`, restore the oracle's token casing, drop the invented `Isabelle`/`Lean4`/`K3`).
2. Re-run the same style of check for the other 10 files that assert the missing `.so` — each one
   needs a surviving oracle named before its claim can be retired or restored.
3. Read the unclassified directories (they are data/other workstreams, not obviously code to port).
4. Decide the structured-data ingestion path for the harvested knowledge bases (a design question, not
   data movement — the fabric's encoder is text-only).
