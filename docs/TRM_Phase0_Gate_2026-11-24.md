# Phase 0 gate result — FAILED as written (2/4), with the correction beside it

**2026-11-24.** Pre-registered in `docs/TRM_Domain_Grounding_Plan_2026-11-24.md` before any code was
written: *"Gate: the four probe questions from the MoE study (council port, axiom A2, TLS/MITM,
hash-chain tamper-evidence) each return a **top-1 chunk containing the key string**."*

## What was built

- `trm_uefi_kernel/host_council_server/src/platform_facts.rs` — canonical facts published **from code**
  (ports from `local_inference::{INFER_PORT, EMBED_PORT}`, axioms from `sovereign_axioms::describe()`,
  module totals from `module_registry_generated.rs`, 26 canonical routes). Three tests enforce it:
  every listed route must appear verbatim in `main.rs` (`every_listed_route_exists_in_the_dispatch`),
  the council port must appear in `main.rs`, and the facts must carry six axioms and three ports.
- `GET /platform/facts` in `main.rs`.
- `tools/build_council_index.py` — builds `data/council_evidence_index_v2.json` from the facts endpoint
  plus prose docs; fail-closed if the embedder is unreachable, never overwrites v1, writes a manifest
  with sha256/counts/timings, and records any chunk it could not embed.

## Measured result (same embedder, same cosine, same probes, both indexes)

| probe | v1 top-1 | key in top-1 | v2 top-1 | key in top-1 |
|---|---|---|---|---|
| council listen port | 0.650 | miss | **0.919** `platform/facts` | **PASS** |
| axiom A2 | 0.615 | miss | **0.746** `council/axioms` | **PASS** |
| TLS verification / MITM | 0.623 | miss | 0.689 `docs/TRM_Blind_Run…` | **miss** (key in top-3) |
| hash-chain tamper-evidence | 0.654 | miss | 0.683 `docs/PRODUCTION-READINESS…` | **miss** (not in top-3) |

    GATE: v1 0/4, v2 2/4   (requirement: 4/4)  -> FAIL

v2 index: 2,453 chunks from 194 files, 38 generated fact chunks, dim 768, 0 unembeddable, sha256
`1f763cb7a6c544f6…`.

## Why it failed, and the corrected property (not a substitute for the failure)

The four probes were not all the same kind of question. Two ask **what this platform is** (port, axiom);
two ask **what is generally true** (TLS/MITM, hash chains). The index can only answer the first kind —
and did, with a large margin (0.650 → 0.919 and 0.615 → 0.746). The second kind is the model's job and
was already measured at **6/6** on general domains in the same study.

So the honest statement is:

    platform-fact retrieval:  v1 0/2  ->  v2 2/2   (0.919 / 0.746, keys present)
    general-knowledge retrieval: v1 0/2 -> v2 0/2  (out of scope for an index; model measured 6/6)

**The gate as written fails and stays failed.** The correction is that it conflated two requirements and
should have been split, which is my error in writing it, not a property of the index.

## What this does and does not license

- It does **not** license wiring v2 into the council yet: Phase 1 opens only behind the Phase 0 gate, and
  that gate did not pass. A 43 MB index whose general-knowledge retrieval is unproven has no business in
  the decision path.
- It **does** establish that the facts the resident MoE models hallucinate — council port, axioms —
  are now retrievable at 0.92/0.75 cosine instead of buried in narrative at 0.65 and below. That was the
  actual complaint behind the phase.

## Options, in the order they should be judged

1. **Split the gate and re-run it unmodified** (cheap, honest): platform facts 4/4 (already have 2/2 plus
   the module-total and route questions from the facts block), general knowledge explicitly out of scope.
   The same probes, the same index, a gate that asks one question instead of two.
2. **Add a general-knowledge block** — the Phase 2 teacher (gpt-oss-20b, 6/6) generates the answers, so the
   text is attributed rather than invented. That is distillation, and it belongs in Phase 2.
3. **Leave v2 unbuilt-by-default** until one of the above passes.

Note on artifacts: the 43 MB index is **not committed** (it is reproducible from
`tools/build_council_index.py` against a running `:8593`); the manifest that pins its sha256 is committed,
so a later run can prove whether the same index produced the same verdicts.
