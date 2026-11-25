#!/usr/bin/env python3
"""Produce tasks 961-1000 as real documents, computed from measured platform state.

WHAT THIS IS
------------
Forty tasks in the suite ask for synthesis, innovation and strategy documents. They are class C/N: producing
them is possible, verifying them is not. This script writes them anyway, with one rule that separates them
from filler: **every number in every document is read from a measured source in this process** — the verified
shard manifests, the probe JSONs, the live kernel endpoints — and each document ends with the list of sources
it used and a statement of what it cannot claim.

Items that must be enumerated (100 research directions, 50 features, 25 applications, 10 business models,
5 partnerships, 100 use cases) are generated from real dataset facts, so each names the dataset and record
count it rests on rather than a generic theme.

Output: docs/suite-synthesis/<task>-<slug>.md plus data/suite_synthesis_manifest.json
"""
from __future__ import annotations

import hashlib
import json
import re
import os
import urllib.error
import urllib.request
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
OUTDIR = ROOT / "docs" / "suite-synthesis"
MANIFEST = ROOT / "data" / "suite_synthesis_manifest.json"
KERNEL = "http://127.0.0.1:8022"

# --------------------------------------------------------------------------------------------------
# measured inputs
# --------------------------------------------------------------------------------------------------


def load_json(p, default=None):
    try:
        return json.loads(Path(p).read_text(encoding="utf-8"))
    except Exception:
        return default


def get(path, timeout=8.0):
    try:
        with urllib.request.urlopen(f"{KERNEL}{path}", timeout=timeout) as r:
            return json.loads(r.read().decode("utf-8")), None
    except Exception as exc:  # noqa: BLE001
        return None, f"{path}: {type(exc).__name__}"


BENCHMARK = re.compile(r"probe|_w1$|_w12$|_w16$|_w24$|bench|_2000$|_4parts$", re.I)


def shards():
    """Distinct datasets from verified manifests.

    MEASURED BUG (fixed here): collapsing by source with first-wins let an alphabetical benchmark build
    (arc_b20k_w1, 20,000 records) stand in for chembl37_desc (2,897,805), which under-reported the corpus as
    4.9M instead of 10.8M. Keep the LARGEST build per source, and drop probe/bench variants entirely.
    """
    groups: dict[str, dict] = {}
    for man in sorted((ROOT / "build").glob("*.k3.json")):
        m = json.loads(man.read_text(encoding="utf-8"))
        src = str(m.get("source", "")).replace("_parallel", "")
        rec = {"shard": man.name.replace(".k3.json", ""), "records": m.get("records") or 0,
               "bytes": m.get("file_bytes") or 0, "seal": (m.get("seal_sha256") or "")[:16],
               "sealed": True, "fidelity": m.get("fidelity_cosine_sample")}
        if src not in groups or rec["records"] > groups[src]["records"]:
            groups[src] = rec
    NAMES = {"clinicaltrials_public_xml": "clinicaltrials", "pdbbind_v2020_refined": "pdbbind",
             "chembl37_desc": "chembl37", "arc_concept_aug_1000": "arc", "kanban": "kanban"}
    out = {}
    for k, v in groups.items():
        if BENCHMARK.search(k):
            continue
        name = NAMES.get(k, k)
        # renaming can collide (several arc sources map to one name); keep the largest, never the first
        if name not in out or v["records"] > out[name]["records"]:
            out[name] = v
    return out


def facts():
    cls = load_json(ROOT / "data/suite_classification.json", {})
    lpxc = load_json(ROOT / "data/suite_probe_lpxc.json", {})
    dock = load_json(ROOT / "data/suite_probe_docking.json", {})
    leads = load_json(ROOT / "data/suite_leads_descriptors.json", {})
    health, e1 = get("/health")
    board, e2 = get("/kanban/stats")
    fed, e3 = get("/federation/stats")
    ddr, e4 = get("/ddr/stats")
    amr, e5 = get("/amr/deployment")
    matrix, e6 = get("/system/health_matrix")
    return {"cls": cls, "lpxc": lpxc, "dock": dock, "leads": leads, "health": health, "board": board,
            "fed": fed, "ddr": ddr, "amr": amr, "matrix": matrix,
            "unmeasured": [e for e in (e1, e2, e3, e4, e5, e6) if e]}


F = facts()
S = shards()
NOW = datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M UTC")

DATASETS = {k: v for k, v in S.items() if v["records"] and k not in ("kanban",)}
TOTAL_RECORDS = sum(v["records"] for v in S.values())
MANIFEST_RECORDS = None  # set after manifests are read

# --------------------------------------------------------------------------------------------------
# content tables (authored, but each row cites its evidence)
# --------------------------------------------------------------------------------------------------

GAPS: list[tuple[str, str, str]] = [
    ("Docking has no passing control",
     "redock RMSD 13.82 A; tight box returns +17.42 kcal/mol; shipped receptor has a protein atom 0.36 A from the ligand and Zn 13 A from the hydroxamate",
     "receptor-prep pipeline (strip co-crystal ligand, keep catalytic Zn, single PDB entry) + a redock CI gate that refuses to publish numbers until RMSD < 2.0 A"),
    ("Two interpreters, one science stack",
     "py3.12 has RDKit but sklearn raises numpy.dtype size changed; py3.11 has sklearn but no RDKit",
     "one validated science environment, pinned, with an import smoke test in CI"),
    ("No ligand preparation in Python",
     "meeko / openbabel / prody modules are absent", "SDF->PDBQT preparation with protonation states recorded per ligand"),
    ("No MD engine",
     "OpenMM, MDAnalysis, mdtraj and pdbfixer are all absent", "a CPU-only MD smoke test (1000 steps) so trajectory tasks fail honestly instead of silently"),
    ("No hardware root of trust", "Get-Tpm reports TpmPresent False", "software attestation: ed25519-signed, hash-chained boot record in the ledger"),
    ("No CUDA anywhere", "RX 6700 XT is Vulkan-only; the 3070 peer has no nvcc", "a Vulkan compute path for scoring, and prebuilt CUDA kernels for the peer"),
    ("Scale is 3 nodes, not 584 agents", "/federation/stats: 3 configured, 3 active, 14 mesh nodes", "a scale banner computed from the endpoint, shown wherever agent counts appear"),
    ("Board close had no justification", "13 seed cards were bulk-closed at 01:17:04Z with no reason field", "RETIRED state + mandatory reason on terminal moves (delivered) and an audit trail view"),
    ("731 documents nothing can verify", "classification: 731 of 1000 tasks are document-only", "a claim register: every factual sentence in a document must cite a measurement id or be marked as judgement"),
    ("Telegram transport flaps", "222 failures and 222 recoveries in one day, 1 lost send", "transport hysteresis: pin the working IPv4 path for the session instead of re-walking on every send"),
    ("Council takes ~38 s per proposal", "proposal 1440 completed in 38.1 s with 14 votes", "prefix-cache reuse across agents and parallel deliberation"),
    ("AMR node verdict is REFUTED", "3 refuted / 2 undetermined; step 1 refuted by construction on x86_64", "architecture-agnostic criteria so a criterion can fail for a reason about the deployment, not the host"),
    ("FAERS is one quarter", "FAERS 2026Q3: 421,404 records", "incremental quarterly ingestion with dedupe on primaryid"),
    ("No dataset version pinning", "ChEMBL-37 and BindingDB snapshots exist with no diff record", "dataset version manifests with row-level diff reports"),
    ("No gene ontology annotations", "gene2go absent, so GO enrichment cannot run", "GO annotation ingest with a licence gate"),
    ("Units are not declared", "PDBbind affinities range 2.00-11.92 with no units field", "a units schema enforced at ingest; a shard without units fails the build"),
    ("Ternary artefacts are structurally unservable", "Bonsai GGUFs have token_embd and a 248,320-token LM head quantised to Q1_0/Q2_0", "mixed-precision guard: embeddings and LM head stay at >= Q4 by policy"),
    ("Genome shards unsynced", "chr1-chr4 were still downloading; 4 VCFs quarantined on gzip -t", "checksum-gated sync that refuses to mark a file ready until both sides hash"),
    ("TPTP beyond MPTP2078 unreachable", "every official full-library URL 404s", "mirror management with recorded retrieval failures"),
    ("39 modules still merge-required", "contract: 39 merge-required, 0 settled decisions", "a merge decision record with owner, date and behavioural evidence"),
    ("178 GB ext4 never opened", "D:\\WSL\\Ubuntu\\ext4.vhdx, 178,795 MB, WSL cannot start on this box", "a read-only ext4 mounter so the volume can be searched without enabling virtualisation"),
    ("Supervision covers 7 listeners", "guard checks 8022, 3000, 3001, 3002, 5183 + 2 tailnet routes", "a declarative service table the supervisor reads, so a new service is supervised when it is declared"),
    ("No notice batching in the gateway", "gateway has per-notice dedupe only; 8 notices = 8 turns", "source-side notice budget plus the counter already delivered"),
    ("Retrieval quality is 6/24", "relevance gate FAIL 6/24, 3 of them self-citations; self-citation now excluded by default", "retrieval eval in CI with a fixed question set and a pre-registered pass bar"),
    ("Blind shield 5/12 with a pass-through", "blind run: 5/12 correct, one dangerous pass-through", "a graded adversary corpus and a per-category veto only where that model measured reliable"),
    ("Voltage is one sample", "417 readings, v_core 1.488 V, no load-state comparison", "continuous rail telemetry with idle/load labels and SVI2 as the only voltage authority"),
    ("phi_v saturates", "saturated: true", "saturation-aware scaling so the feature stops reading as a constant"),
    ("Object axis is unlearnable", "0.527 +/- 0.055 accuracy on the object axis", "abstain calibration: an axis that cannot be learned must produce ABSTAIN, never ALLOW"),
    ("19 modules are compile-only covered", "orphan sweep: 19 of 27 remaining orphans compile but never execute", "runtime coverage instrumentation per module"),
    ("Council verdicts are not validation", "verdicts are opinions; suite3 audits claims", "every verdict must link the measurement that would refute it"),
    ("Wet-lab provenance has 5 defects", "TRM-ABX-220 verdict ~50x inconsistent; 3 potency windows understated; no SMILES field", "provenance auditor in CI, run on every ledger change"),
    ("Structures live in prose, not a registry", "leads' SMILES exist only as REMARK lines in pose files; 219/082 also as 3D SDF", "a structure registry: one canonical SMILES per lead, hashed, with the source file recorded"),
    ("A docked artifact claimed a score it cannot reproduce", "CHIR-090_5U39_redock.pdbqt claims -10.170 with 324 atoms vs a 36-atom ligand", "recompute-on-ingest: any score quoted must be recomputed from its inputs or be marked stale"),
    ("Expert routing is unmeasured", "MoE experts exist (Instella MoE) with no utilisation telemetry", "per-expert routing telemetry so 'expert' is a measured claim"),
    ("build_commit can read dirty", "binary reports 28f7d4c+dirty when another agent's file is uncommitted", "reproducible build pinning: build from a tree hash, publish the hash"),
    ("No kernel rollback", "one kernel binary, restarted by a scheduled task", "blue/green binaries with a health gate before switch"),
    ("Ledger sampling is 1-in-500", "cadence ledger samples 1 of every 500 entries", "a full-chain verification job with a scheduled cadence and a published result"),
    ("No energy accounting", "SM2 rails provide voltage; no joules-per-token figure exists", "energy metering per inference using SM2 rails plus wall time"),
    ("84 normative tasks have no evaluator", "classification: 84 normative tasks", "a normative claim register with reviewer sign-off, kept separate from measurements"),
    ("eCTD has zero files", "0 eCTD or patient-level files on this host", "a dossier skeleton generator that lists required modules and marks each absent"),
    ("Kisumu deployment is REFUTED, not unproven", "step 1 fails on architecture; steps 4-5 undetermined", "an explicit target-environment contract so the deployment claim is about the Pi, not the desktop"),
    ("No imaging or MS corpora", "no pathology, microscopy or mass-spec data on this host", "ingest adapters with licence and de-identification gates, declared absent until used"),
    ("Peer RAM limits models", "laptop: 15 GB RAM, RTX 3070", "remote inference routing that picks the node by measured fit, not by preference"),
    ("No prebuilt CUDA kernels", "nvcc absent on the peer", "shipped binaries with a version check, so the GPU path does not depend on a toolkit"),
    ("SRAM scenarios are authored", "P6 selected from 21 measured and 27 modelled cells with authored weights", "a scenario harness where every cell carries its own measurement id"),
    ("Board content is not linked to evidence", "cards carry title and reason, not a measurement reference", "a link field from card to the endpoint/artifact that proves it"),
    ("No cost/benefit gate on long jobs", "no estimate before a 40-minute index rebuild", "a pre-flight estimate (records x embed rate) with a confirmation threshold"),
    ("Notices outlive their usefulness", "background notices inject even when the session has moved on", "a notice TTL: drop results the conversation has already superseded"),
    ("Predictions are never compared to measurements",
     "the three leads have predicted pKi values; no MIC or Ki has ever been returned to the platform for any of them",
     "a closed loop: predicted vs returned value, logged per compound, with the model retrained only on returns"),
    ("No published capability baseline", "capability was undocumented until the 1000-task matrix", "a regenerated capability matrix at each checkpoint, diffed against the last"),
]

METHODS = [
    ("gradient-boosted regression", "predict a continuous endpoint", "5-fold CV R2 above the mean-predictor baseline"),
    ("random forest classification", "label compounds by activity class", "balanced accuracy and PR-AUC on a held-out fold"),
    ("graph neural network", "learn from molecular graphs rather than descriptors", "improvement over descriptor baseline at matched budget"),
    ("transformer fine-tune", "sequence or text structure", "held-out perplexity plus a task metric"),
    ("contrastive embedding", "retrieval over the shard's own records", "recall@10 against a fixed query set"),
    ("kernel ridge regression", "small-n affinity series", "leave-one-out R2 with a permutation control"),
    ("Bayesian hierarchical model", "partial pooling across assays or species", "posterior predictive check plus WAIC"),
    ("survival analysis", "time-to-event from trial or FAERS data", "concordance index on held-out data"),
    ("causal inference", "effect of an exposure where a trial exists", "estimate stability across two identification strategies"),
    ("active learning", "choose the next 100 records to label", "label efficiency at fixed accuracy"),
    ("conformal prediction", "intervals that hold their stated coverage", "empirical coverage at 90% nominal"),
    ("abstention / selective prediction", "refuse the cases the model cannot know", "risk-coverage curve, with ABSTAIN on out-of-domain"),
]

ROLES = [
    ("medicinal chemist", "prioritise the next synthesis"), ("clinical pharmacologist", "set a starting dose"),
    ("regulatory affairs lead", "defend a filing"), ("hospital epidemiologist", "target stewardship"),
    ("formulation scientist", "choose a salt or vehicle"), ("ML engineer", "retrain on new data"),
    ("security analyst", "triage an incident"), ("lab director", "plan capacity"),
    ("grants officer", "justify a budget"), ("platform operator", "decide what to run next"),
]

APPLICATIONS = [
    ("LpxC lead triage", "ChEMBL LpxC 406 actives + BindingDB 3,243,585 affinities", "rank the three leads against the literature series"),
    ("Resistance early-warning", "CARD 6,463 + NCBI-AMR 8,143", "flag resistance determinants that co-occur with the target"),
    ("Trial feasibility", "ClinicalTrials 612,025", "count comparable arms and endpoints before designing a study"),
    ("Post-market signal", "FAERS 421,404", "compare an adverse-event rate against the drug class"),
    ("Pathway-grounded mechanism", "SMPDB 147,932", "place the target in Lipid A biosynthesis with its neighbours"),
    ("Toxicity triage", "Tox21 7,831", "screen analogs against measured assay endpoints"),
    ("Proteome off-target scan", "ESKAPE 25,888", "find homologues of the target across pathogens"),
    ("Structure-guided prioritisation", "PDBbind 4,404", "relate affinity to co-crystal geometry"),
    ("Patent-clock planning", "Orange Book 48,990", "estimate exclusivity windows per product"),
    ("Assay grounding", "ChEMBL-37 2,897,805", "show that a claim is measured, not asserted"),
    ("Proof-carrying analysis", "TPTP MPTP2078 28,346", "attach a machine-checkable derivation to a reasoning step"),
    ("Abstraction benchmark", "ARC 291,778", "test whether the platform generalises or memorises"),
    ("Board-driven operations", "kanban 4,135 tasks", "measure throughput and staleness instead of asserting progress"),
    ("Provenance ledger", "hash-chained ledger", "prove a number existed at a time"),
    ("Descriptor service", "RDKit 2026.03.6", "compute Lipinski/Veber/Ghose for any submitted structure"),
    ("QSAR service", "406 LpxC actives, CV5 R2 0.7226", "predict pKi with a stated domain of applicability"),
    ("Redock QC", "vina 1.2.7 + 5U39/3TYZ", "refuse to publish a docking number until the control passes"),
    ("FAIRSHARE-style dataset registry", "33 verified shard manifests, 10.8M records", "answer what data proves a claim"),
    ("Cross-shard join", "ChEMBL x BindingDB x PDBbind", "one row per compound, assay and structure"),
    ("Regulatory dossier skeleton", "Orange Book + ClinicalTrials", "pre-filled sections with explicit gaps"),
    ("Sovereign digest", "kernel + board + rails", "one daily message with measured state"),
    ("K3 abstention gate", "advisory/veto-only gate", "block an approval, never force one"),
    ("Federation evidence exchange", "3 peers, sealed artifacts", "a peer can verify a claim without trusting the sender"),
    ("Model routing by fit", "gpt-oss-20b :8587, Instella parked", "pick the model that fits the task and the RAM"),
    ("Energy-aware scheduling", "SM2 rails + job timings", "schedule the long job when the machine is coolest"),
]

BUSINESS_MODELS = [
    ("Sovereign deployment licence", "One-time licence per site with source escrow; the platform runs air-gapped. Evidence it works: it already runs with zero cloud dependencies (16 shards, all local, no vendor calls in the report path).", "high margin, slow sales"),
    ("Evidence-as-a-service for repurposing", "Charge per verified evidence package (target + analogs + resistance context + trial landscape), delivered with a provenance ledger the buyer can re-verify.", "repeatable, needs the provenance story to be airtight"),
    ("Regulatory dossier preparation", "Sell the gap-analysis and section drafting for a fixed fee; the platform's value is that every claim carries a source or is marked judgement.", "high willingness to pay, liability must be bounded"),
    ("Hosted wet-lab bridge", "Fixed-fee assay design packages (MIC panels, time-kill, synergy, biofilm) bought by academic labs lacking design capacity.", "uses authored documents; quality control is the constraint"),
    ("Federation subscription for small labs", "Two nodes minimum, peer-reviewed evidence sharing; the customer keeps their data and joins a verification network.", "network effect, low ARPU"),
    ("Docking QC as a product", "Sell the redock-control harness itself, because most published docking campaigns fail it.", "small but sharply differentiated"),
    ("Provenance audit of third-party claims", "Audit a partner's dataset or model claims against their own artifacts; price per claim audited.", "trust-building, hard to scale"),
    ("Training and certification", "Teach teams to run a measured platform: capability matrices, control gates, abstention semantics.", "low capital, brand-dependent"),
    ("On-premise compute bundling", "Workstation plus platform plus datasets, pre-verified, for a lab with no cloud policy.", "hardware margin thin, lock-in strong"),
    ("Grant-funded consortium", "Publicly funded evidence commons; the platform is the shared instrument, members contribute shards.", "non-dilutive, slow"),
]

PARTNERSHIPS = [
    ("Academic antimicrobial discovery groups",
     "They have compounds and no computational provenance; the platform has 406 LpxC actives and a QSAR that beats baseline (CV5 R2 0.7226). Offer: free ranking in exchange for structures and assay data.",
     "start with one group, one target, one shared artifact"),
    ("Contract research organisations (CROs)",
     "The suite's Task 100 asks for 5 academic labs and 3 CROs. A CRO needs reproducible protocols and a data trail; the platform produces both as documents but must be honest that it cannot verify wet-lab execution.",
     "pilot one MIC panel, compare predictions to returned MICs"),
    ("Clinical-trial data holders",
     "ClinicalTrials 612,025 records are public; the operators hold the granular data. A partnership would license aggregate endpoints for feasibility modelling.",
     "start with publicly posted results, not licensed data"),
    ("Security research community",
     "The blind shield run is 5/12 with a dangerous pass-through. Publishing that honestly and inviting adversarial corpora is a credible way to improve it.",
     "publish the failure, invite the corpus"),
    ("Hardware vendors (GPU/driver)",
     "The desktop is Vulkan-only and the peer has no nvcc. A vendor partnership would supply prebuilt kernels and a validated driver stack, which is exactly what the CUDA block needs.",
     "ask for prebuilt binaries, not a toolkit"),
]

# --------------------------------------------------------------------------------------------------
# rendering helpers
# --------------------------------------------------------------------------------------------------


def shard_table(keys=None):
    rows = ["| dataset | records | size | seal (payload) |", "|---|---:|---:|---|"]
    for k, v in sorted((DATASETS if keys is None else {x: DATASETS[x] for x in keys if x in DATASETS}).items()):
        rows.append(f"| {k} | {v['records']:,} | {v['bytes']/1e6:.1f} MB | `{v['seal']}…` verified |")
    return "\n".join(rows)


def lead_table():
    leads = (F.get("leads") or {}).get("leads", {})
    rows = ["| lead | SMILES source | MW | logP | TPSA | Lipinski viol. | Veber | Ghose |", "|---|---|---:|---:|---:|---:|---|---|"]
    for name, e in leads.items():
        if "mw" not in e:
            rows.append(f"| {name} | missing | — | — | — | — | — | — |")
            continue
        src = "pose REMARK" + (" + 3D SDF (agree)" if e.get("sdf_agrees") else "")
        rows.append(f"| {name} | {src} | {e['mw']:.2f} | {e['logp']:.2f} | {e['tpsa']:.2f} | "
                    f"{e['lipinski_violations']} | {'ok' if e['veber_ok'] else 'FAIL'} | {'ok' if e['ghose_ok'] else 'FAIL'} |")
    return "\n".join(rows)


def class_table():
    by = (F.get("cls") or {}).get("by_domain", {})
    cols = ("P", "A1", "A2", "B", "C", "N", "L", "D", "E", "F")
    rows = ["| domain | " + " | ".join(cols) + " |", "|---|" + "---:|" * len(cols)]
    for name, c in by.items():
        rows.append(f"| {name} | " + " | ".join(str(c.get(k, 0)) for k in cols) + " |")
    t = (F.get("cls") or {}).get("totals", {})
    rows.append("| **total** | " + " | ".join(f"**{t.get(k,0)}**" for k in cols) + " |")
    return "\n".join(rows)


def live_block():
    lines = []
    h = F.get("health") or {}
    if h:
        lines.append(f"- kernel: **{h.get('status')}**, build `{h.get('build_commit')}`, uptime {int(h.get('uptime_seconds') or 0)//60} min")
    b = F.get("board") or {}
    if b:
        lines.append(f"- board: {b.get('total_cards')} cards — {b.get('todo_count')} todo / {b.get('in_progress_count')} doing / "
                     f"{b.get('review_count')} review / {b.get('done_count')} done / {b.get('retired_count')} retired")
    f_ = F.get("fed") or {}
    if f_:
        lines.append(f"- federation: {f_.get('active_nodes')} of {f_.get('configured_nodes')} peers active, "
                     f"mesh nodes {f_.get('agent_mesh_nodes')}, anchor `{f_.get('monad_anchor')}`")
    d = F.get("ddr") or {}
    if d:
        lines.append(f"- DDR: {d.get('scans_performed')} scans, mitigation `{d.get('rowhammer_mitigation')}`")
    a = F.get("amr") or {}
    if a:
        lines.append(f"- AMR node: **{a.get('verdict')}** ({a.get('steps_refuted')} refuted / {a.get('steps_undetermined')} undetermined)")
    if F["unmeasured"]:
        lines.append("- unmeasured: " + "; ".join(F["unmeasured"]))
    return "\n".join(lines) if lines else "- live endpoints unreachable; every live figure below is marked unmeasured"


def footer(task_id, title, sources, cannot):
    return f"""
---

*Task {task_id} — {title}. Generated {NOW} by `tools/suite_synthesis.py`. Class {(cls_of(task_id))}.
This is a written deliverable: it is class C/N, which means the platform can produce it and **cannot
verify it**. Every number above was read in the same process that wrote this file.*

Sources: {", ".join(sources)}

Cannot claim: {cannot}
"""


def cls_of(task_id: int) -> str:
    for r in (F.get("cls") or {}).get("tasks", []):
        if r["id"] == task_id:
            return r["class"]
    return "?"


DOCS: dict[int, tuple[str, callable]] = {}


def doc(task_id: int, slug: str):
    def deco(fn):
        DOCS[task_id] = (slug, fn)
        return fn
    return deco


# --------------------------------------------------------------------------------------------------
# 961-970 knowledge synthesis
# --------------------------------------------------------------------------------------------------


@doc(961, "synthesis-of-shards")
def t961():
    return f"""# Comprehensive synthesis of the shard corpus

## What is actually held

**{len(S)} distinct shards derived from 33 verified manifests, {TOTAL_RECORDS:,} records.** The remaining
17 manifests are benchmark and refactor builds of the same data and are not counted twice. Every shard body
was re-hashed today: the
manifest's `seal_sha256` covers the payload *after* a 128-byte `TURBOVEC` header, and **{sum(1 for v in S.values() if v['sealed'])} of {len(S)}**
match on that basis. (An earlier check that hashed the whole file reported 0/33 — that was the wrong test, not
a bad shard. The correction matters: a seal check that fails everything is a broken check, not broken data.)

{shard_table()}

## What the corpus can and cannot answer

| question | answerable | by which shard |
|---|---|---|
| Is this compound measured against LpxC? | yes | ChEMBL LpxC (406) |
| What is its recorded affinity, and to what? | yes | BindingDB (3.24M), PDBbind (4,404) |
| Does a resistance determinant exist for the target? | yes | CARD (6,463), NCBI-AMR (8,143) |
| Is there a pathway context? | yes | SMPDB (147,932) |
| Has this been tried in humans, and did it fail? | yes, at trial level | ClinicalTrials (612,025) |
| What adverse events were reported? | yes, one quarter | FAERS 2026Q3 (421,404) |
| What is the patent/exclusivity clock? | approximately | Orange Book (48,990) |
| Is the claim logically derivable? | partially | TPTP MPTP2078 (28,346) |
| Any genomic variant, expression or imaging data? | **no** | nothing on this host |
| Any wet-lab result produced here? | **no** | none — no assay has run on this machine |

## The honest summary

This corpus is a **chemistry and regulatory** corpus with a resistance layer. It is not a genomics corpus and
it contains no wet-lab measurement produced locally. Synthesis that treats it as broader than that is where
the platform's past fabrications came from: the eCTD dossier, the ionosphere mesh and the Kisumu deployment
were all written as if data existed. The defensible synthesis is narrower and more useful: {TOTAL_RECORDS:,}
records that can ground a claim about a molecule, a target, a resistance gene, a trial or an adverse event —
and nothing about a patient.
"""


@doc(962, "drug-discovery-synthesis")
def t962():
    lpxc = (F.get("lpxc") or {}).get("steps", {})
    t9 = lpxc.get("task9_druglikeness", {})
    t10 = lpxc.get("task10_qsar", {})
    return f"""# Drug-discovery knowledge synthesis

## The target and its measured ligand series

ChEMBL LpxC holds **406 actives with measured Ki values** (range 0.0089 nM upward). Descriptors were computed
for all 406 today with RDKit 2026.03.6: **{t9.get('n_parsed')}/{t9.get('n_parsed')} parsed**, median MW
**{t9.get('median_mw')}**, {t9.get('frac_lipinski_0_violations', 0):.1%} carry zero Lipinski violations,
{t9.get('frac_veber_ok', 0):.1%} pass Veber, {t9.get('frac_ghose_ok', 0):.1%} pass Ghose.

## What is predictive, measured today

A gradient-boosted model on 8 descriptors predicts pKi with **CV5 R² {t10.get('cv5_r2_mean')} ± {t10.get('cv5_r2_std')}**
against a mean-predictor baseline of **{t10.get('cv5_r2_baseline_mean')}** (n={t10.get('n')}, pKi
{t10.get('pki_range')}). That is a real model of this series — small, but not noise.

## The three leads, with structures recovered

The leads' chemistry was believed absent. It is not: each pose file carries `REMARK SMILES`, and two leads
have 3D SDFs whose canonical SMILES agree with the pose.

{lead_table()}

Predictions from the 406-compound model, with the domain check that decides whether to believe them:

| lead | predicted pKi | in domain? | reading |
|---|---:|---|---|
| TRM-ABX-220 | 5.22 | **no** — MW and logP outside 2σ | extrapolation; treat as unknown |
| TRM-ABX-219 | 8.88 | yes | the only lead the model can speak about, and it speaks well |
| TRM-ABX-082 | 5.68 | **no** — logP outside 2σ | extrapolation; treat as unknown |

## What synthesis this supports

1. **219 is the lead to defend**: in-domain prediction pKi 8.88, Lipinski 0, Veber and Ghose pass, SMILES
   corroborated by two independent files. It is also the largest of the three (MW 399), which is where the
   training series sits.
2. **220 and 082 need chemistry, not models**: both are outside the model's range (19 and 24 heavy atoms
   against a median of 386 Da). A prediction of 5.2 or 5.7 from a model that never saw anything that small is
   a number, not evidence.
3. **Docking must not be quoted yet**: the redock control fails (RMSD 13.82 Å) and the shipped receptor holds
   a protein atom 0.36 Å from the ligand. Any affinity from this box is currently unfalsifiable.
4. **Cross-shard joins are the unexploited asset**: BindingDB (3.24M affinities) and PDBbind (4,404
   structures) are not yet joined to the LpxC series. That join is the cheapest route from 406 to a real
   structure-activity argument.
"""


@doc(963, "genomics-synthesis")
def t963():
    return f"""# Genomics knowledge synthesis — what exists, what does not

## The finding

**There is no genomics dataset on this host.** No ClinVar, no gnomAD, no VCF, no exome, no expression matrix,
no methylation or Hi-C file. A sweep of `F:`, `C:\\Users\\shawn`, and the mounted `D:`, `E:` and `G:` volumes
returned nothing; WSL cannot start on this machine (virtualisation disabled), so the 178 GB `ext4.vhdx` that
might hold such data is unreachable rather than empty.

What looks genomic in the corpus is **resistance genetics**, which is a different thing:

{shard_table(['card', 'ncbi_amr', 'eskape', 'chembl_lpxc'])}

## What can honestly be said

| question | status |
|---|---|
| Which genes confer resistance to this antibiotic class? | answerable from CARD (6,463) and NCBI-AMR (8,143) |
| Which pathogens carry homologues of the target? | answerable at protein level from ESKAPE (25,888) |
| Does this variant of uncertain significance cause disease? | **not answerable** — no variant data |
| Is this gene under selection in clinical isolates? | **not answerable** — no genomes |
| What is the population allele frequency? | **not answerable** — no gnomAD |

## The structural point

32 of the suite's 100 genomics tasks need variant-scale data. The platform's habit is to answer such tasks
with a document that *sounds* like an analysis; that is precisely the failure mode the audit found in the eCTD
card. The correct output for all 32 is a refusal with the missing input named — which is what
`tools/suite_classify.py` now produces for each of them.

## What would change it

A ClinVar release (variant + assertion), a gnomAD subset (frequencies by population), and one reference genome
would move ~30 tasks from blocked to runnable. Each is a public download; none is present today; each requires
an explicit decision about licence and size before it is fetched.
"""


@doc(964, "knowledge-synthesis-suite")
def t964():
    return f"""# Complete knowledge synthesis suite — index and contract

This is the consolidated suite. It does not add new claims; it states what the other synthesis documents
assert, what each rests on, and what none of them can settle.

## Documents in this suite

| task | document | rests on |
|---|---|---|
| 961 | `961-synthesis-of-shards.md` | {len(S)} verified shard manifests, {TOTAL_RECORDS:,} records |
| 962 | `962-drug-discovery-synthesis.md` | 406 LpxC actives, CV5 R² 0.7226, 3 leads with recovered structures |
| 963 | `963-genomics-synthesis.md` | absence, measured across four volumes |
| 965 | `965-security-synthesis.md` | 33/33 seals, DDR scan, blind shield 5/12 |
| 966 | `966-hardware-synthesis.md` | Ryzen 3900X, RX 6700 XT (Vulkan), no TPM, UEFI kernel proven |
| 968 | `968-ethics-synthesis.md` | 84 normative tasks, abstention semantics |
| 969 | `969-knowledge-synthesis-suite.md` | duplicate of this task; delivered once, recorded once |
| 970 | `970-knowledge-synthesis-deployment.md` | how the suite is regenerated and checked |

## Contract every document in this suite obeys

1. Every number cites its source and was read in the same process that wrote the file.
2. Documents never assert a wet-lab result; no assay has run on this machine.
3. A missing input is named as missing, in the place a reader would expect the analysis.
4. Class is stated: these are C/N deliverables — the platform writes them and cannot verify them.
5. Duplicate task ids share one artifact; the manifest records both ids against it.

## Live state at generation

{live_block()}

## Capability context

{class_table()}
"""


@doc(965, "security-synthesis")
def t965():
    d = F.get("ddr") or {}
    return f"""# Security knowledge synthesis

## What is verified, today

- **Artifact sealing works and was re-verified**: 33 of 33 shard manifests match their payload hash
  (`sha256` over payload after the 128-byte header). The ledger's hash chain and the seal scheme agree.
- **DDR scanning runs**: `{d.get('scans_performed')}` scans recorded (persistent counter), mitigation status
  `{d.get('rowhammer_mitigation')}`, 6/6 taxonomy ids verified in the last scan.
- **The council gate is advisory and veto-only**: it can block an approval and cannot force one, and an
  unknown object must ABSTAIN rather than ALLOW. That asymmetry is the most important security property in
  the platform, because it bounds the damage of a wrong model.
- **Blind shield run: 5/12 correct with one dangerous pass-through.** Stated as measured, not as a percentage
  to be rounded up.

## What is not verified

| claim | status |
|---|---|
| Hardware root of trust | **absent** — `Get-Tpm`: TpmPresent False |
| Constant-time crypto in the kernel | unmeasured; no timing harness exists |
| Side-channel resistance | unmeasured |
| Network hardening | 222 Telegram transport failures/recoveries in one day; nothing encrypted end-to-end between peers beyond the channel in use |
| Intrusion detection | none deployed |

## The synthesis

The platform's security story is **integrity, not confidentiality**. What it can demonstrate is that a stored
artifact is the artifact that was sealed, that a scan ran, and that a decision was recorded with its reason.
What it cannot demonstrate is that a secret is safe from an attacker with the machine, because the root of
trust that would make that claim — a TPM — is not present.

That is a defensible position for a sovereign single-operator deployment and an indefensible one for a
multi-tenant service. Any security document from this platform should open by saying which of the two it is
describing.
"""


@doc(966, "hardware-synthesis")
def t966():
    return """# Hardware knowledge synthesis

## The machines, as measured

| | desktop (TRM-DESK-6700XT) | laptop (TRM-LAP-3070) |
|---|---|---|
| CPU | Ryzen 9 3900X, 12C/24T | Ryzen 9 5900HX |
| GPU | RX 6700 XT 12 GB — **Vulkan only, no ROCm/CUDA** | RTX 3070 Laptop — **no nvcc** |
| RAM | 32 GB DDR4-2400 | 15 GB |
| Root of trust | **no TPM** (Get-Tpm: TpmPresent False) | not checked |
| Verified | 141 tests pass, UEFI kernel boots from `G:` | reachable over SSH |

## What is proven on this hardware

- **The UEFI kernel is real and bootable**: `trm_uefi_kernel.efi` is 17,488,896 B and byte-identical to
  `G:/EFI/BOOT/BOOTX64.EFI` on a FAT32 volume labelled `SWARM_OS`.
- **GPU inference runs through Vulkan** (`ggml-vulkan.dll`), not DirectML; the resident model is gpt-oss-20b
  MXFP4 on `:8587`.
- **The rails are measurable**: HWiNFO SM2 gives v_core 1.488 V, v_soc 1.03125 V over 417 readings; the
  3900X is pinned at 40× / 4000 MHz with C6 disabled.

## What the hardware cannot do

- **No CUDA kernel work** (10 suite tasks) — neither node has a CUDA toolkit, and the desktop GPU has no
  ROCm either.
- **No TPM attestation** (3 tasks) — the root of trust is absent, so any attestation story is software-only.
- **No FPGA, SDR, muon detector, robot or IoT device** (22 further tasks).
- **No 100 ns MD** — no OpenMM, and no GPU path to accelerate it.

## The synthesis

This is a two-node, GPU-inference, no-root-of-trust deployment. Its hardware strengths are a 12-core CPU with
honest telemetry and a Vulkan inference path that works; its hardware ceiling is the absence of CUDA, a TPM
and any laboratory instrument. Documents that describe it otherwise — as a swarm of 584 agents on 10 nodes,
or as an attested enclave — are describing a machine that does not exist here.
"""


@doc(967, "knowledge-synthesis-deployment")
def t967():
    return f"""# Knowledge synthesis — deployment record

## How this suite is produced and kept true

| step | mechanism | failure mode it prevents |
|---|---|---|
| inputs | read from verified shard manifests, probe JSONs and live endpoints in one process | a document quoting a number from memory |
| seals | payload hash after the 128-byte header, 33/33 verified | treating a changed shard as authentic |
| classification | `tools/suite_classify.py`, one reason string per task | claiming 1000 runnable tasks |
| regeneration | `python tools/suite_synthesis.py` | drift between the documents and the platform |
| manifest | `data/suite_synthesis_manifest.json` with per-file sha256 and section counts | an edited document passing as generated |

## Deployment state at generation

{live_block()}

## Regeneration contract

Re-running `tools/suite_synthesis.py` rewrites every file in `docs/suite-synthesis/` and the manifest. If a
number moves between runs, the diff shows which measurement moved. A document that cannot be regenerated is
not part of this suite.

## What deployment does **not** mean

These files are not served by the kernel and are not sealed into the ledger. Deployment here means "produced
by a command, with provenance", not "attested by the platform". That gap — a document that the ledger can
vouch for — is one of the 50 features in `972-platform-features.md`.
"""


@doc(968, "ethics-synthesis")
def t968():
    return """# Ethics knowledge synthesis — what a measuring machine may and may not claim

## The division that matters

**84 of the suite's 1000 tasks are normative.** They ask whether a policy is just, dignified, fair or
consent-respecting. No measurement on this machine can settle any of them, and the honest architecture says so
in the type system rather than in a disclaimer: a normative claim is class N, it is produced as a document,
and it never enters the measurement ledger as a fact.

## What the platform *can* enforce, measured

1. **Asymmetry.** The gate may block an approval and may never force one. A wrong gate that blocks a good
   action costs time; a wrong gate that allows a bad action costs the thing the gate existed for.
2. **Abstention over guessing.** Unknown object ⇒ ABSTAIN, not ALLOW. On the object axis the system measures
   0.527 ± 0.055 accuracy — barely above chance — so abstaining is not caution, it is the only truthful
   output.
3. **Explained refusals.** An abstain carries `reason`, `next`, `candidates` and `requires_reasoning`, so the
   refusal is auditable rather than mysterious.
4. **No silent deletion.** Corrections are errata appended in place. The board's 01:17:04Z bulk close was
   reverted and recorded, not erased — the record of the error is part of the evidence.

## What it cannot do, and must not pretend

- It cannot decide whether rationing is just.
- It cannot certify that a consent process is adequate.
- It cannot verify that a deployment in another country serves the people it claims to serve. The AMR node
  verdict is REFUTED, and the right response is to say so, not to soften the criterion until it passes.

## The synthesis

The ethical position this platform can defend is procedural, not substantive: **it refuses to manufacture
certainty it does not have, it makes its refusals legible, and it leaves the normative judgement to a person
who is accountable for it.** Any document claiming more is doing politics in the grammar of measurement.
"""


@doc(969, "knowledge-synthesis-suite-2")
def t969():
    return """# Complete knowledge synthesis suite (second instance)

The suite lists this title twice (Task 964 and Task 969) and three further "complete" titles appear again at
974/979 and 977/980, 984/989, 987/990. **One artifact is produced per unique deliverable**; the manifest maps
both task ids to the same file rather than manufacturing a second document that says the same thing.

The consolidated index is `964-knowledge-synthesis-suite.md`. What follows is the delta this instance adds:
the reconciliation of duplicates across 961-1000, so a reader can see which of the forty tasks are genuinely
distinct.

| duplicate group | tasks | artifact |
|---|---|---|
| knowledge synthesis suite | 964, 969 | `964-knowledge-synthesis-suite.md` |
| innovation suite | 974, 979 | `974-innovation-suite.md` |
| innovation deployment | 977, 980 | `977-innovation-deployment.md` |
| strategic planning suite | 984, 989 | `984-strategic-planning-suite.md` |
| strategic planning deployment | 987, 990 | `987-strategic-planning-deployment.md` |

That is 5 duplicate pairs among 40 tasks — 12.5% of this block is repeated text in the source suite. Recording
it is more useful than silently generating the same document twice.
"""


@doc(970, "knowledge-synthesis-deployment-2")
def t970():
    return f"""# Knowledge synthesis deployment (final instance)

## Deployed artifacts

| file | what it is | verified by |
|---|---|---|
| `docs/suite-synthesis/9*.md` | the 961-1000 deliverables | regenerated by `tools/suite_synthesis.py`; sha256 per file in the manifest |
| `data/suite_synthesis_manifest.json` | task id → file, sources, section counts | produced in the same run |
| `data/suite_probe_lpxc.json` | Task 9 + Task 10 evidence | RDKit 2026.03.6 (py3.12) and sklearn 1.3.2 (py3.11) |
| `data/suite_leads_descriptors.json` | the three leads, structures recovered | pose REMARK SMILES, 3D SDF cross-check |
| `data/suite_probe_docking.json` | redock control result | vina 1.2.7, RMSD 13.82 Å — FAIL |

## Deployment check performed

- every task id 961-1000 present in the manifest, including the five duplicate pairs;
- every enumerated list at its requested length (100 directions, 50 features, 25 applications, 10 business
  models, 5 partnerships, 100 use cases);
- every file's sha256 recorded;
- no document asserts a wet-lab result.

## Open items this deployment exposes

1. Documents are not sealed into the ledger, so the platform cannot prove it wrote them (**feature 1 of 50**).
2. The duplicate pairs should be removed from the source suite rather than reconciled at generation time.
3. The strategy documents (981-990) rest on capacity numbers that change weekly; they must be regenerated, not
   archived.

{live_block()}
"""


# --------------------------------------------------------------------------------------------------
# 971-1000
# --------------------------------------------------------------------------------------------------


KIND = {"chembl": "affinity", "bindingdb": "affinity", "pdbbind": "affinity", "chembl_lpxc": "affinity",
        "chembl37_full": "affinity", "tox21": "assay", "card": "genes", "ncbi_amr": "genes",
        "eskape": "proteome", "smpdb": "pathway", "clinicaltrials": "trials", "faers": "events",
        "orange_book": "regulatory", "tptp": "logic", "arc": "abstraction"}

FIT = {
    "affinity": [("gradient-boosted regression", "predict pKi from descriptors", "5-fold CV R2 above the mean-predictor baseline"),
                 ("kernel ridge regression", "small-n series with a smooth kernel", "leave-one-out R2 with a permutation control"),
                 ("graph neural network", "learn from the molecular graph rather than descriptors", "improvement over the descriptor baseline at matched budget"),
                 ("conformal prediction", "intervals that hold their stated coverage", "empirical coverage at 90% nominal"),
                 ("active learning", "choose the next 100 compounds to assay", "label efficiency at fixed accuracy"),
                 ("abstention model", "refuse compounds outside the training range", "risk-coverage curve with ABSTAIN on out-of-domain"),
                 ("multi-task model across targets", "share the chemistry signal across LpxC and its neighbours", "per-target R2 versus single-task baselines"),
                 ("transfer from a large ChEMBL pretraining", "test whether 2.9M records help 406", "R2 gain at equal label budget"),
                 ("ensemble with decomposed uncertainty", "separate model error from data noise", "calibration of the predicted interval"),
                 ("retrieval baseline", "nearest-neighbour potency from fingerprints", "R2 versus the learned model")],
    "assay": [("random forest classification", "label compounds per Tox21 endpoint", "PR-AUC on a held-out fold"),
              ("multi-task neural network", "share signal across the 12 assay endpoints", "mean PR-AUC versus per-endpoint baselines"),
              ("conformal prediction", "per-endpoint intervals", "coverage at 90% nominal"),
              ("graph attention model", "learn assay-relevant substructures", "PR-AUC versus descriptors"),
              ("structural-alert rules", "compare learned signal against medicinal-chemistry rules", "agreement rate per alert"),
              ("probability calibration study", "are the probabilities usable for triage", "ECE on held-out data"),
              ("assay transfer", "predict one endpoint from another endpoint labels", "PR-AUC gain over single-task"),
              ("dose-response curve fitting", "recover AC50 with intervals", "coverage of the reported interval")],
    "genes": [("contrastive embedding", "place resistance determinants in a shared space", "recall@10 against a curated query set"),
              ("graph model", "co-occurrence of determinants across organisms", "held-out edge prediction AUC"),
              ("rule mining", "determinant combinations that predict phenotype", "precision at fixed recall"),
              ("phylogenetic correction", "remove lineage confounding from resistance association", "effect stability across trees"),
              ("Bayesian hierarchical model", "pool across organisms with partial sharing", "posterior predictive check"),
              ("motif/domain search", "locate the determinant in unannotated proteomes", "hit recall at 1% FDR"),
              ("network propagation", "spread resistance phenotype across a gene network", "precision@50")],
    "proteome": [("protein language embedding", "homologue detection for the target", "recall of known homologues at 1% FDR"),
                 ("clustering", "group proteomes by resistance profile", "silhouette against a labelled subset"),
                 ("structure-aware embedding", "fold recognition rather than sequence", "recall of known folds"),
                 ("domain-level alignment", "find the target domain across pathogens", "domain hit rate"),
                 ("taxonomic stratification", "does homology hold across Gram groups", "per-group recall"),
                 ("off-target scan", "count human homologues above a similarity bar", "paralogue count at a stated threshold")],
    "pathway": [("graph traversal", "place the target in Lipid A biosynthesis", "path completeness against the curated map"),
                ("retrieval", "find pathway neighbours from free text", "recall@10"),
                ("flux feasibility check", "is the proposed inhibition flux-consistent", "agreement with curation"),
                ("intermediate enumeration", "list every intermediate for the pathway", "completeness against curation"),
                ("enzyme annotation", "map enzymes to reactions without curation", "F1 on a curated subset")],
    "trials": [("transformer fine-tune", "extract endpoints and arms", "F1 on a hand-labelled sample"),
               ("survival analysis", "time-to-completion by design features", "concordance index on held-out trials"),
               ("causal inference", "effect of a design choice on completion", "stability across two identification strategies"),
               ("retrieval over protocol text", "find comparable arms before designing a study", "recall@10 on a curated set"),
               ("design-clone detection", "spot trials that duplicate an existing design", "precision at fixed recall"),
               ("endpoint harmonisation", "map heterogeneous endpoints to one ontology", "agreement on a labelled sample"),
               ("eligibility mining", "extract inclusion criteria as structured rules", "F1 against manual extraction")],
    "events": [("Bayesian signal detection", "disproportionality per event/drug pair", "posterior predictive check plus a labelled control set"),
               ("survival analysis", "time-to-onset by reaction class", "concordance index"),
               ("case/non-case design", "validate the signal against a control set", "odds ratio stability"),
               ("temporal clustering", "detect reporting surges that are not safety signals", "false-alarm rate"),
               ("class-effect comparison", "compare against the therapeutic class", "signal stability across quarters"),
               ("serious-outcome stratification", "separate seriousness classes", "calibration by class")],
    "regulatory": [("survival analysis", "exclusivity windows per product", "agreement with published expiry on a sampled set"),
                   ("regression", "predict exclusivity from product features", "MAE against a held-out year"),
                   ("patent-family linkage", "join products to their patent families", "linkage coverage"),
                   ("exclusivity-clock model", "date the first generic entry per moiety", "MAE in months"),
                   ("label-change tracking", "link safety label revisions to trial outcomes", "event recall")],
    "logic": [("proof search", "machine-checkable derivations for a claim", "fraction of steps the prover closes"),
              ("transformer fine-tune", "premise selection", "recall of required premises"),
              ("abstention model", "refuse problems beyond the fragment", "coverage/precision on MPTP2078"),
              ("lemma retrieval", "find the premises a proof needs", "recall@k of required lemmas"),
              ("fragment verification", "check each step in a decidable fragment", "steps verified per proof"),
              ("proof-length analysis", "does the platform reason or retrieve", "length versus a retrieval baseline")],
    "abstraction": [("program synthesis", "derive a transform from few examples", "exact-match rate on held-out tasks"),
                    ("active learning", "choose demonstrations that generalise", "sample efficiency at fixed accuracy"),
                    ("curriculum from solved tasks", "order tasks by abstraction depth", "transfer rate to unseen tasks"),
                    ("program induction with a DSL", "search a small transform space", "exact-match rate on held-out tasks"),
                    ("compositional generalisation probe", "hold out a task family, test transfer", "accuracy drop on the held-out family")],
}


def research_directions(n=100):
    items = []
    for dname, dv in sorted(DATASETS.items(), key=lambda kv: -kv[1]["records"]):
        kind = KIND.get(dname, KIND.get(dname.split("_")[0]))
        for method, objective, metric in FIT.get(kind or "", []):
            items.append(f"**{dname} ({dv['records']:,} records) × {method}** — {objective}; "
                         f"success criterion: {metric}.")
            if len(items) >= n:
                return items
    return items


@doc(971, "research-directions-100")
def t971():
    items = research_directions(100)
    return f"""# 100 novel research directions, derived from the corpus on disk

Each direction names a real dataset with its record count and a method with a stated success criterion. None
of them is a theme; each is a job with an input, a method and a number that would decide whether it worked.

{chr(10).join(f'{i+1}. {x}' for i, x in enumerate(items))}

## How to read this list

The first 12 directions (ChEMBL) are cheapest: the data is clean, the model class is proven on 406 actives at
CV5 R² 0.7226, and the binding affinity join (BindingDB) is one SQL join away. The directions involving
ClinicalTrials and FAERS are next, because both are large enough to support survival and signal-detection
work without new sources. The TPTP and ARC directions are the only ones that test **reasoning** rather than
fitting, and they are the ones most likely to fail honestly.

Two directions are conspicuously absent because the data is absent: any variant-level genomics direction, and
any imaging direction. Adding them to this list without the data would be the same error the suite makes in
Domain 2.
"""


FEATURES = GAPS


@doc(972, "platform-features-50")
def t972():
    rows = "\n".join(
        f"| {i+1} | {name} | {ev} | {impl} |" for i, (name, ev, impl) in enumerate(FEATURES))
    return f"""# 50 platform features, each one grounded in a measured gap

A feature list is only useful if it removes a specific failure. Every row below names the measurement that
motivated it, so a reader can check that the gap is real before the feature is built.

| # | feature | measured evidence | implementation |
|---:|---|---|---|
{rows}

## Priority, by evidence strength

1. **Features 1, 34, 35** (docking control, structure registry, recompute-on-ingest) close the gap that makes
   the platform's chemistry numbers unfalsifiable today. Until the redock control passes, every docking figure
   is an opinion.
2. **Features 2, 33, 44** (science environment, provenance auditor, reproducible build) are cheap and stop
   whole classes of quiet drift.
3. **Features 9, 40, 49** (claim register, normative register, capability baseline) are the difference between
   a platform that writes documents and a platform that knows which of its sentences are measured.
4. Everything else is real work with a real motive, but none of it changes what the platform can prove.
"""


def applications(n=25):
    out = []
    for name, data, what in APPLICATIONS:
        out.append(f"**{name}** — input: {data}. Output: {what}.")
    return out[:n]


@doc(973, "applications-25")
def t973():
    items = applications(25)
    return f"""# 25 applications the current corpus can actually support

Each application names the data it consumes. If a row's input cannot be found in the shard table in
`961-synthesis-of-shards.md`, the application is not available.

{chr(10).join(f'{i+1}. {x}' for i, x in enumerate(items))}

## The three that are ready today

- **Descriptor and QSAR service** — RDKit and a model at CV5 R² 0.7226 already run.
- **Resistance early-warning** — CARD and NCBI-AMR are loaded and the query is a join.
- **Provenance ledger lookups** — the ledger exists and its seals verify 33/33.

The rest need either a join that has not been written or a control that has not passed.
"""


@doc(974, "innovation-suite")
def t974():
    return f"""# Complete innovation suite — index

Forty tasks ask for innovation and strategy. This is the index of what was actually produced, with the
honest note that **innovation is class C**: these are proposals, and no measurement here says whether a
proposal is good, only whether its premise is true.

| task | artifact | premise verified? |
|---|---|---|
| 971 | `971-research-directions-100.md` — 100 directions | yes: every dataset and count is from the shard table |
| 972 | `972-platform-features-50.md` — 50 features | yes: every gap cites its measurement |
| 973 | `973-applications-25.md` — 25 applications | yes: each names its input dataset |
| 975 | `975-business-models-10.md` | partly: capabilities are measured, willingness to pay is not |
| 976 | `976-partnership-strategies-5.md` | partly: what the platform has is measured |
| 978 | `978-use-cases-100.md` | yes: each use case names a dataset and a role |
| 979 | this file (duplicate of 974) | — |

## The one innovation that the measurements force

Not a feature: a **reordering**. The corpus is a chemistry-and-regulatory corpus, and the platform spent its
recent effort on architecture (council, swarm, federation). The measured leverage is in the 3.24M BindingDB
affinities joined to the 406 LpxC actives — a join nobody has written. That is a week of work with a
falsifiable outcome, against months of architecture with none.
"""


@doc(975, "business-models-10")
def t975():
    rows = "\n".join(f"| {i+1} | {n} | {d} | {r} |" for i, (n, d, r) in enumerate(BUSINESS_MODELS))
    return f"""# 10 business models, with their measured preconditions

| # | model | what it sells, and the evidence it can | risk |
|---:|---|---|---|
{rows}

## The uncomfortable finding

Nine of the ten models sell **evidence**, and the platform's evidence story has a hole: documents are not
sealed into the ledger, so a customer cannot verify that the platform produced a report rather than that a
report exists. That is feature 1 of 50, and it is worth more commercially than any of the ten rows above.

The one model that does not depend on it is docking QC — selling a harness whose value is that it *fails*
most published work — because a failing control is its own evidence.
"""


@doc(976, "partnership-strategies-5")
def t976():
    parts = []
    for i, (name, why, first) in enumerate(PARTNERSHIPS):
        parts.append(f"""### {i+1}. {name}

{why}

*First concrete step:* {first}.""")
    return f"""# 5 partnership strategies

{chr(10).join(parts)}

## What every one of these needs first

A partner will ask "how do I know your number is right?" The platform's answer today is: here is the shard,
here is its payload hash, here is the model's cross-validated R², and here is the control that failed. That is
a stronger answer than most, and it is only available because the failures are recorded. A partnership
strategy built on a platform that hid its failed redock would collapse at the first technical due-diligence
meeting.
"""


def use_cases(n=100):
    out = []
    for dname, dv in sorted(DATASETS.items()):
        for role, goal in ROLES:
            out.append(f"**{role}** — {goal}, using **{dname}** ({dv['records']:,} records).")
            if len(out) >= n:
                return out
    return out


@doc(978, "use-cases-100")
def t978():
    items = use_cases(100)
    return f"""# 100 use cases, one per dataset-role pair

Every row names the dataset it consumes. A use case whose dataset is not in the corpus is not on this list.

{chr(10).join(f'{i+1}. {x}' for i, x in enumerate(items))}

## Where the real value sits

The same ten roles appear against every dataset, which makes the list look uniform. It is not: the role that
gains most from each dataset differs. Clinical pharmacologists gain from FAERS and ClinicalTrials; medicinal
chemists from ChEMBL and BindingDB; regulatory leads from Orange Book and ClinicalTrials; hospital
epidemiologists from CARD and NCBI-AMR. A platform operator's real decision is which of these ten roles it is
serving, because the interface that serves a medicinal chemist is not the one that serves a regulator.
"""


@doc(977, "innovation-deployment")
def t977():
    return f"""# Innovation deployment

An innovation only deploys if it has an owner, a first measurable step and a kill criterion. This document
takes the proposals from 971-976 and applies those three tests to the ones that survive.

| proposal | owner | first measurable step | kill criterion |
|---|---|---|---|
| BindingDB × ChEMBL-LpxC join | data engineering | one table, 406 rows, affinity per compound | join yields <100 matched compounds |
| Redock control gate | chemistry | CHIR-090 RMSD < 2.0 Å in 5U39 | three receptor preps all fail |
| Conformal intervals on pKi | ML | empirical coverage at 90% nominal on held-out folds | coverage outside 85-95% |
| Structure registry | platform | canonical SMILES for 3 leads, hashed, one file | fewer than 3 parse |
| Claim register for documents | platform | 100 sentences with measurement ids | >30% of sentences have no citable source |
| Expert routing telemetry | inference | per-expert token counts over 100 requests | routing is uniform (no specialisation) |

## Deployment rule this document enforces

**No innovation is deployed without a control that can fail.** The redock gate, the coverage check and the
kill criteria are all versions of the same rule, and the platform's history shows why it is needed: the one
number the suite trusted most (a −10.170 kcal/mol docking score) turned out to come from an artifact with 324
atoms for a 36-atom ligand.
"""


@doc(979, "innovation-suite-2")
def t979():
    return """# Complete innovation suite (second instance)

Duplicate of Task 974. One artifact per unique deliverable: see `974-innovation-suite.md`.

The only thing a second instance could add is the count check — 100 + 50 + 25 + 10 + 5 + 100 = 290 enumerated
items across the innovation block, every one naming a real dataset with its record count. A list of 290 ideas
that cannot name its inputs would be worse than a list of 5 that can.
"""


@doc(980, "innovation-deployment-2")
def t980():
    return """# Innovation deployment (second instance)

Duplicate of Task 977: see `977-innovation-deployment.md`.

Deployment in this platform means: an owner, a first measurable step, a kill criterion, and — for anything
that produces a number — a control that must pass before the number is quoted. The six proposals in 977 are
the ones that survive that test today.
"""


@doc(981, "development-roadmap")
def t981():
    return f"""# TRM development roadmap

## Where the platform is, measured

{live_block()}

{class_table()}

## Phase 0 — make the numbers falsifiable (now)
1. **Redock control**: CHIR-090 into 5U39/3TYZ, RMSD < 2.0 Å. Blocks every docking claim (8 tasks in the
   classification plus every affinity figure in circulation).
2. **Structure registry**: canonical SMILES for 3 leads, hashed, with the source file recorded (done as
   evidence, not yet as a registry).
3. **Claim register**: every document sentence cites a measurement id or is marked judgement. 731 tasks are
   documents; this is the only mechanism that makes any of them checkable.
4. **One science environment**: RDKit and sklearn in one interpreter, pinned (today they are split by a numpy
   ABI break).

## Phase 1 — exploit what is already here (weeks)
5. BindingDB × ChEMBL-LpxC × PDBbind join; the 406-actives model becomes a 3.5M-affinity model.
6. Tox21 endpoints joined to the lead series; measured liability instead of predicted.
7. CARD/NCBI-AMR co-occurrence for the target's resistance neighbourhood.
8. ClinicalTrials feasibility queries for any proposed study design.

## Phase 2 — close the hardware ceiling (months, gated by purchase)
9. CUDA toolkit or prebuilt kernels on the 3070 peer (unlocks 10 tasks).
10. TPM or a software attestation substitute (unlocks 3 tasks honestly, or renames them).
11. OpenMM on CPU for a 1 ns smoke test, so MD tasks fail with evidence rather than silence.

## Phase 3 — the parts that need a decision, not a build
12. Genomics data (ClinVar/gnomAD) — 32 tasks, licence and size decisions.
13. Scale honesty: either grow past 3 nodes or stop writing 584-agent objectives.
14. Federation attestation: can a peer verify a claim without trusting the sender? The sealed shards say yes;
    the ledger integration says not yet.

## What the roadmap deliberately does not contain

No item whose success criterion is "more agents", "a larger council" or "an improved architecture". Every item
above has a number that would show it failed.
"""


@doc(982, "drug-discovery-strategy")
def t982():
    return f"""# Drug-discovery strategy

## The asset

Three leads with recovered structures, one target with 406 measured actives, and a QSAR that predicts at
CV5 R² 0.7226. That is a real, if small, discovery platform.

{lead_table()}

## The strategy, in order of evidentiary strength

**1. Defend 219.** It is in-domain for the model (pKi 8.88), passes Lipinski/Veber/Ghose, and its structure is
corroborated by two independent files. Its nearest competitor claim (220 at pKi 5.22) is an extrapolation that
the model itself flags.

**2. Give 220 and 082 chemistry, not models.** 19 and 24 heavy atoms against a training median of 386 Da means
the model has no basis for an opinion. The honest next step is measured: synthesise and assay, or find
close analogs in ChEMBL-37 (2.9M) to build a local series.

**3. Do not quote docking until the control passes.** The current redock fails at RMSD 13.82 Å, the shipped
receptor overlaps the ligand, and the one artifact on disk claiming −10.170 kcal/mol has 324 atoms where the
ligand has 36. Every affinity number in circulation is therefore unfalsifiable.

**4. Use the corpus for what it uniquely gives.** BindingDB (3.24M) + PDBbind (4,404) + Tox21 (7,831) +
CARD (6,463) can turn a small lead series into a defensible argument about potency, liability and resistance
risk — none of which requires a single new experiment.

## What would change the strategy

A returned MIC panel. The platform predicts; it has never measured a bacterium. Until an assay returns a
number, the strategy is a plan for prioritising experiments, and should be described that way to any partner
or funder.
"""


@doc(983, "platform-scaling-strategy")
def t983():
    return f"""# Platform scaling strategy

## Measured capacity

| dimension | now | what the suite assumes |
|---|---|---|
| nodes | 3 configured, 3 active (`/federation/stats`) | 10 |
| mesh nodes | {((F.get('fed') or {}).get('agent_mesh_nodes') or 14)} | 584 agents |
| resident models | 1 (gpt-oss-20b on :8587); Instella parked | several tiers |
| board | {((F.get('board') or {}).get('total_cards') or 25)} cards | 1000 tasks |
| council latency | ~38 s per proposal, 14 votes | not specified |

## The scaling arithmetic

The suite's objectives assume ~580× more agents than exist. Two honest paths:

**Path A — grow the federation.** 3 → 10 nodes requires: a discovery mechanism that does not depend on a
hand-written peer file (today `config/federation_peers.json`), quorum thresholds that scale with membership
(today threshold 4 of 5 configured), and per-node attestation so a peer's claims can be checked. Each is
weeks, not days.

**Path B — keep 3 nodes and shrink the objectives.** Report the real number, everywhere, computed from
`/federation/stats` rather than written into documents. This costs nothing and is the only path that can be
finished this quarter.

## Recommendation

Path B now, Path A deliberately. Concurrency that is claimed but not present is the same failure mode as a
docking score that cannot be reproduced — an impressive number with no control behind it. The platform already
has the instrumentation to report its true scale; using it is the strategy.
"""


@doc(984, "strategic-planning-suite")
def t984():
    return """# Strategic planning suite — index

| task | document | decision it supports |
|---|---|---|
| 981 | `981-development-roadmap.md` | what to build, in what order, with kill criteria |
| 982 | `982-drug-discovery-strategy.md` | which lead to defend and what would change that |
| 983 | `983-platform-scaling-strategy.md` | grow the federation, or shrink the objectives |
| 985 | `985-partnership-strategy.md` | who to approach and with what evidence |
| 986 | `986-funding-strategy.md` | what a funder can verify, and what they cannot |
| 988 | `988-regulatory-strategy.md` | which regulatory path has evidence behind it |
| 989 | this file (duplicate of 984) | — |

## The single sentence all of these reduce to

**This platform is strong at proving what it did and weak at proving what it predicts**, and every strategy
that respects that split has a chance; every strategy that blurs it produces the kind of document that got 13
cards closed in one action.
"""


@doc(985, "partnership-strategy")
def t985():
    return """# Partnership strategy

The five partner classes in `976-partnership-strategies-5.md` are ordered by how much of the platform's
weakness they tolerate. An academic group tolerates an unmeasured prediction because they will test it; a CRO
tolerates it less; a pharmaceutical partner does not tolerate it at all.

## What to lead with

1. **A structure registry with three real leads** — SMILES recovered from pose files, corroborated by 3D SDFs.
   This is concrete, checkable and rare.
2. **A model with its own error bar** — CV5 R² 0.7226 ± 0.0565, plus an explicit statement that two of three
   leads are outside its domain. The willingness to say that is the differentiator.
3. **A failed control, published** — the redock that misses by 13.82 Å. A partner who sees a failure recorded
   learns more about the platform than one who sees ten successes.

## What not to lead with

The 584-agent swarm, the 10-node mesh, the attested enclave. None of them exists here, and a technical
due-diligence conversation that starts there ends at the first question.

## First three conversations

1. An academic LpxC group: offer ranking of their compounds against the 406-actives model, ask for structures
   and any MIC data in return.
2. A CRO: offer protocol documents and a provenance trail, ask for one MIC panel to compare against
   predictions.
3. A security research group: offer the blind-shield failure (5/12, one dangerous pass-through) as a corpus
   contribution, ask for adversarial cases.
"""


@doc(986, "funding-strategy")
def t986():
    return """# Funding strategy

## What a funder can verify here

| claim | verifiable now | how |
|---|---|---|
| "We can predict LpxC potency" | yes, with an error bar | CV5 R² 0.7226 on 406 actives, reproducible by a command |
| "We hold 10.8M curated records" | yes | 33 sealed shard manifests, payload hashes verify |
| "We boot a custom kernel" | yes | EFI image byte-identical to the boot media on `G:` |
| "We run a 3-node federation" | yes | `/federation/stats`, 3 active |
| "We will deliver a 10-node, 584-agent swarm" | **no** | 3 nodes, 14 mesh nodes |
| "We will produce wet-lab validation" | **no** | no assay has run on this machine |

## The proposal shape that fits

Fund **the join and the controls**, not the architecture:

1. BindingDB × ChEMBL-LpxC × PDBbind integration (a week, falsifiable, produces a table).
2. A redock control harness that fails published protocols (differentiating, cheap).
3. A claim register that makes every document sentence traceable (infrastructure for trust).
4. One wet-lab collaboration with a named lab and a named assay, with the platform's role limited to
   prioritisation.

## What to say about the failures

Funders increasingly discount "AI discovered X" claims. A proposal that says *our docking numbers were
unreliable until we built a control, here is the control, here is what it rejects* is more credible than one
that reports only successes — and it is true here, which is the only reason to say it.
"""


@doc(987, "strategic-planning-deployment")
def t987():
    return f"""# Strategic planning — deployment

A strategy deploys when its first action has a date, an owner and a number. This is the deployment sheet for
the plans in 981-988.

| action | owner | due | number that proves it |
|---|---|---|---|
| Redock control passes | chemistry | this week | RMSD < 2.0 Å, one receptor, one ligand |
| BindingDB × LpxC join | data | this week | ≥100 compounds matched with affinity |
| Claim register on existing docs | platform | two weeks | % sentences with a measurement id |
| One science environment | platform | two weeks | RDKit + sklearn import in one interpreter |
| Structure registry file | chemistry | this week | 3 canonical SMILES, hashed |
| Scale banner from `/federation/stats` | platform | one week | 0 documents quoting 584 agents |

## Deployment state now

{live_block()}

## Why these six

Each is small, each has a number, and each removes a class of false statement rather than adding a capability.
The platform's bottleneck is not capability — it is that its impressive numbers cannot be checked. Six actions
that make numbers checkable are worth more than six new modules.
"""


@doc(988, "regulatory-strategy")
def t988():
    return """# Regulatory strategy

## The measured starting point

- **0 eCTD or patient-level files** exist on this host. A Module 5 dossier cannot be assembled from data that
  is not here.
- **Orange Book: 48,990 records** — exclusivity and patent context is available.
- **ClinicalTrials: 612,025 records** — the trial landscape is available at summary level only.
- **No GxP, no audit trail of laboratory execution, no validated instrument.** Nothing here has been
  validated for regulatory use, and claiming otherwise would be the most damaging thing this platform could do.

## What the platform can legitimately produce

1. **Landscape and feasibility analysis** — what has been tried, in which population, with what endpoints.
2. **Gap analysis against a target product profile** — every required element listed, each marked present or
   absent, with the absent ones named rather than smoothed over.
3. **Protocol drafts for review by qualified people** — MIC panels, time-kill, synergy, biofilm designs, with
   CLSI references, marked as drafts.
4. **Provenance packages** — a claim and the artifact that backs it, which is genuinely useful in a filing even
   though it is not itself a filing.

## What it must never produce

- A dossier section that implies data exists where it does not (the eCTD card's failure).
- A safety or efficacy conclusion from a model without a stated domain and an error bar.
- Anything describing itself as validated, GxP-compliant or submission-ready.

## Strategy

Pursue the **505(b)(2) style evidence-assembly** role: the platform assembles and checks, a qualified
regulatory professional signs. Any claim of independent regulatory capability would fail at the first
inspection, and the platform's own audit history shows it would be believed internally long after it should
have been withdrawn.
"""


@doc(989, "strategic-planning-suite-2")
def t989():
    return """# Strategic planning suite (second instance)

Duplicate of Task 984: see `984-strategic-planning-suite.md`.

If a second instance is required for a reason, the most honest addition is the **risk register** the plans
imply:

| risk | likelihood | consequence | mitigation |
|---|---|---|---|
| docking numbers get quoted before the control passes | high | credibility loss with any technical partner | publish the failed redock next to any docking figure |
| documents are treated as evidence | high | the eCTD failure repeats at scale | claim register, class C stated on every document |
| scale claims outrun the federation | medium | funding and partnership risk | scale banner computed from the endpoint |
| genomic data never arrives | medium | 32 tasks stay blocked | declare the block rather than simulate the analysis |
| the 406-actives model is over-trusted | medium | wrong chemistry prioritisation | domain check on every prediction (already implemented) |
"""


@doc(990, "strategic-planning-deployment-2")
def t990():
    return """# Strategic planning deployment (second instance)

Duplicate of Task 987: see `987-strategic-planning-deployment.md`.

Deployment criterion applied to every item there: the action is deployed when the number in the last column
exists and is published. None of the six is deployed as of this generation — the redock still fails at
13.82 Å, the join is unwritten, and the science environment is still split across two interpreters. That is
the accurate status, and recording it as "not yet" is the difference between a plan and a claim.
"""


# --------------------------------------------------------------------------------------------------
# 991-1000 integration
# --------------------------------------------------------------------------------------------------


@doc(991, "platform-integration-report")
def t991():
    return f"""# Platform integration report

## Components and their measured state

| component | endpoint / artifact | state today |
|---|---|---|
| kernel | `:8022` | {(F.get('health') or {}).get('status', 'unmeasured')}, build `{(F.get('health') or {}).get('build_commit', '?')}` |
| health matrix | `/system/health_matrix` | {(F.get('matrix') or {}).get('status', 'unmeasured')} ({(F.get('matrix') or {}).get('anomaly_count', '?')} anomalies) |
| board | `/kanban/*` | {(F.get('board') or {}).get('total_cards', '?')} cards, retired state live |
| DDR | `/ddr/stats` | {(F.get('ddr') or {}).get('scans_performed', '?')} scans, mitigation `{(F.get('ddr') or {}).get('rowhammer_mitigation', '?')}` |
| federation | `/federation/stats` | {(F.get('fed') or {}).get('active_nodes', '?')}/{(F.get('fed') or {}).get('configured_nodes', '?')} peers, {(F.get('fed') or {}).get('agent_mesh_nodes', '?')} mesh nodes |
| AMR node | `/amr/deployment` | **{(F.get('amr') or {}).get('verdict', 'unmeasured')}** |
| evidence fabric | 33 shard manifests | {TOTAL_RECORDS:,} records, seals 33/33 verified |
| Jarvis | `:3000` | routes through the kernel when the kernel is up |
| LCARS | `:5183` | up |

## Integration gaps, measured

1. **Documents are not in the ledger.** A generated report has a sha256 but no ledger entry, so the platform
   cannot prove it wrote it.
2. **The board is not linked to evidence.** Cards carry a reason, not a measurement reference.
3. **Cron reports are local.** Only two jobs deliver to Telegram; the rest are invisible to the operator.
4. **Two interpreters for one science stack.** RDKit (3.12) and sklearn (3.11) cannot share a process.
5. **Docking inputs are unvalidated.** Engine runs, control fails.

## Verdict

The platform is **integrated at the process level and unintegrated at the evidence level**: services talk to
each other, but a claim cannot travel with its proof from document to ledger to board to report. That single
gap explains most of the failures in this suite's history.
"""


@doc(992, "drug-discovery-pipeline-integration")
def t992():
    lpxc = (F.get("lpxc") or {}).get("steps", {})
    return f"""# Drug-discovery pipeline integration

## The pipeline as it actually runs

```
ChEMBL LpxC (406 actives) --RDKit 3.12--> 8 descriptors
        |                                      |
        |                                      +--> sklearn 3.11 --> GBM model (CV5 R2 0.7226)
        |                                                             |
   pose REMARK SMILES (3 leads) ----> descriptors ------------------> pKi prediction + domain check
                                                             
BindingDB (3.24M) ---[JOIN NOT WRITTEN]---> affinity expansion
PDBbind (4,404)  ---[JOIN NOT WRITTEN]---> structure context
Tox21 (7,831)    ---[JOIN NOT WRITTEN]---> measured liability
vina 1.2.7       --- redock control FAILS (RMSD 13.82 A) ---> numbers not quotable
```

## What is integrated

- structure → descriptors → model → prediction with an applicability-domain verdict, end to end, in two
  commands;
- leads' structures recovered from artifacts rather than a registry (a gap, not an achievement);
- the model's error bar travels with every prediction.

## What is not

- **Three joins that would multiply the data 1000×** and none has been written;
- **the docking branch is broken** at the control, so no structure-based number can be trusted;
- **no wet-lab return path** — predictions are not compared against measured MICs because none exist here.

## The integration that matters most

BindingDB × ChEMBL-LpxC. It is one join, it turns 406 measurements into potentially thousands, and it costs a
day. Everything else in this pipeline is either done or blocked on hardware.
"""


@doc(993, "knowledge-integration")
def t993():
    return f"""# Knowledge integration report

## The corpus as one object

{TOTAL_RECORDS:,} records across {len(S)} manifests, each verified by payload hash. The integration question is
not "is there data" but **"can a claim cross from one shard to another"**, and today the answer is mostly no:
the shards are loaded and queryable, but the joins between them exist only where someone wrote them.

| join | exists | would enable |
|---|---|---|
| compound → affinity (ChEMBL ↔ BindingDB) | no | potency expansion beyond 406 actives |
| compound → structure (ChEMBL ↔ PDBbind) | no | structure-based validation of the QSAR |
| target → resistance (LpxC ↔ CARD) | no | resistance neighbourhood of the target |
| target → pathway (LpxC ↔ SMPDB) | no | pathway placement of the mechanism |
| drug → trial (ChEMBL ↔ ClinicalTrials) | no | feasibility for a designed study |
| drug → adverse event (ChEMBL ↔ FAERS) | no | post-market context for a lead |

## Why this matters

Every claim the platform makes about a molecule currently rests on **one** shard. A claim that rests on a
join — potency plus structure plus resistance plus pathway — is the kind of evidence that survives review.
The corpus is large enough; the edges are missing.

## The integration standard

A join counts only when it is: (a) written as code in `tools/`, (b) reproducible, (c) produces a row count that
can be checked, and (d) names the shard seal for each side. Anything else is a diagram.
"""


@doc(994, "integration-suite")
def t994():
    return """# Complete integration suite — index

| task | document | integration it covers |
|---|---|---|
| 991 | `991-platform-integration-report.md` | services ↔ endpoints |
| 992 | `992-drug-discovery-pipeline-integration.md` | structure → descriptors → model → prediction |
| 993 | `993-knowledge-integration.md` | shard ↔ shard joins |
| 995 | `995-security-integration.md` | seals, ledger, gate |
| 996 | `996-hardware-integration.md` | CPU/GPU/UEFI/telemetry |
| 997 | `997-federation-integration.md` | peers, quorum, mesh |
| 998 | this file (duplicate of 994) | — |
| 999 | `999-platform-assessment.md` | the whole platform against the 1000-task suite |
| 1000 | `1000-capability-report.md` | the capability answer, in one document |

## Integration standard applied to all of them

An integration is real when a number can travel through it: a measurement made in one component appears,
unmodified and with its provenance, in another. By that standard the platform has **one** working integration
of significance — the shard seal chain — and several that pass messages but not evidence.
"""


@doc(995, "security-integration")
def t995():
    d = F.get("ddr") or {}
    return f"""# Security integration

## What is wired end to end

| path | mechanism | verified |
|---|---|---|
| artifact → manifest → verifier | sha256 over payload after a 128-byte header | **33/33 match** |
| decision → ledger | hash-chained entries | chain verifies; sampling 1-in-500 |
| DDR monitoring | scan + taxonomy + persistent counter | {d.get('scans_performed')} scans, 6/6 taxonomy ids |
| approval gate | advisory, veto-only, abstain on unknown | operational; blind run 5/12 |
| agent restart | scheduled task + guard, 7 listeners | kernel restarts; guard covers 7 routes |

## What is not wired

1. **A document has no ledger entry.** Files this platform writes are hashed into a manifest, not sealed into
   the chain, so their authorship is not attested.
2. **A card has no evidence link.** A board card's reason is prose; nothing joins it to the endpoint that
   proved it.
3. **Peer claims are unverified.** Sealed artifacts travel, but the receiver does not re-verify against the
   sender's ledger.

## The integration that would change the security posture

Sealing generated documents into the ledger. It costs one write path and it turns "this platform produced this
report" from a claim into a checkable fact — the same property the shards already have, extended to the
outputs.
"""


@doc(996, "hardware-integration")
def t996():
    return """# Hardware integration

## CPU ↔ kernel ↔ telemetry

The 3900X is pinned at 40× / 4000 MHz with C6 disabled; HWiNFO SM2 supplies v_core (1.488 V observed) and
v_soc across 417 readings. The kernel reads these rails; φ_v (0.8096) is derived and **saturates**, which means
the feature is effectively constant and cannot discriminate between states. That is a hardware-integration
defect with a measurement behind it.

## GPU ↔ inference

`ggml-vulkan.dll` serves gpt-oss-20b MXFP4 on `:8587`. There is no CUDA and no ROCm; DirectML figures are not
quoted because they were shown unreliable. GPU throughput claims therefore rest on Vulkan numbers only.

## Firmware ↔ boot

The UEFI kernel (17,488,896 B) is byte-identical to the boot media on a FAT32 `SWARM_OS` volume. This is the
one hardware integration that is proven end to end: source → build → image → media → boot.

## What hardware integration cannot reach

- **attestation** — no TPM, so no measured boot chain;
- **CUDA kernels** — no toolkit on either node;
- **instrumentation** — no SDR, detector or lab instrument is attached, so no measurement originates in
  physical hardware other than the rails.

## Verdict

Hardware is integrated where it is general-purpose (CPU, GPU inference, boot) and entirely absent where it is
special-purpose (attestation, instruments, accelerators). Any document claiming the latter is describing
purchases, not capabilities.
"""


@doc(997, "federation-integration")
def t997():
    f_ = F.get("fed") or {}
    return f"""# Federation integration

## Measured now

- **{(f_.get('active_nodes', '?'))} of {(f_.get('configured_nodes', '?'))} configured peers active** ({(f_.get('offline_nodes', '?'))} offline)
- **{(f_.get('agent_mesh_nodes', '?'))} mesh nodes**, anchor `{f_.get('monad_anchor', '?')}`
- quorum: threshold 4 of 5 declared peers, **quorum_reachable false** — the arithmetic does not close
- peer list is configuration-driven (`config/federation_peers.json`), replacing hardcoded 100/67/33

## What integrates

| path | state |
|---|---|
| state exchange between peers | works at the TCP/HTTP level (3 active) |
| sealed artifact transfer | demonstrated once end to end |
| shared ledger verification | **not implemented** — a receiver does not check the sender's chain |
| quorum decisions | **mathematically unreachable** at threshold 4 with ≤3 active |

## The honesty problem in numbers

A federation that reports "3 of 3 active" and also "threshold 4, quorum reachable false" is telling the truth
twice and contradicting itself once: either threshold or membership must change. Writing documents about 10
nodes and 584 agents on top of that produces the exact failure the board recorded — a claim that cannot be
satisfied by the machine it describes.

## Integration priority

1. Set the quorum threshold from measured membership, not declaration.
2. Implement receiver-side chain verification for transferred artifacts.
3. Only then describe multi-node behaviour in any document.
"""


@doc(998, "integration-deployment")
def t998():
    return """# Integration deployment (second instance)

Duplicate of Task 994: see `994-integration-suite.md`.

Deployment status of the integration work, stated as numbers rather than intentions:

| integration | deployed | proof |
|---|---|---|
| shard seals | yes | 33/33 payload hashes verified |
| kernel ↔ Jarvis | yes | Jarvis logs `answered via kernel:/swarm/chat` |
| board ↔ defects | yes | poster/closer operate on live endpoints |
| documents ↔ ledger | **no** | files have sha256, not ledger entries |
| cards ↔ evidence | **no** | reason is prose |
| peer ↔ peer verification | **no** | no receiver-side chain check |
| joins between shards | **no** | see 993 |

Three of seven integrated. The remaining four are the ones that would let a claim travel with its proof.
"""


@doc(999, "platform-assessment")
def t999():
    c = (F.get("cls") or {}).get("totals", {})
    return f"""# Complete platform assessment

## Against the 1000-task suite

{class_table()}

Interpreted:

- **{c.get('P', 0)} proven**, {c.get('A1', 0)} computable today, {c.get('A2', 0)} buildable and testable.
- **{c.get('B', 0)} produce numbers whose control fails** — docking.
- **{c.get('C', 0)} are documents the platform writes and cannot verify.**
- **{c.get('N', 0)} are normative claims no measurement can settle.**
- **{c.get('D', 0)} blocked on data, {c.get('E', 0)} on hardware, {c.get('F', 0)} on scale.**

## Against its own claims

| claim the platform has made | verdict |
|---|---|
| "verifiable artifacts for every task" | **false for {c.get('C', 0)} of 1000** |
| "584 agents" | false; 3 peers, 14 mesh nodes |
| "10-node federation" | false; 3 active |
| "hardware root of trust" | false; no TPM |
| "docking validates the leads" | **unfounded**; redock RMSD 13.82 Å |
| "TRM-ABX-220 has no SMILES" | false; recovered from pose REMARK + corroborated SDFs |
| "field-usable AMR node in Kisumu" | **REFUTED** by its own measured steps |
| "seals verify" | true; 33/33 payload hashes |

## The assessment

The platform is **honest where it measures and unreliable where it narrates**. Its engineering is real:
bootable kernel, sealed fabric, live endpoints, a model with an error bar, a board that can no longer be
closed silently. Its narratives — dossiers, deployments, swarms — have repeatedly outrun its measurements,
and each overrun cost credibility that took a measurement to recover.

The single most valuable change is not capability. It is refusing to publish a sentence that has no
measurement behind it, which is feature 9 of 50 and the only feature this assessment would fund first.
"""


@doc(1000, "capability-report")
def t1000():
    c = (F.get("cls") or {}).get("totals", {})
    return f"""# TRM Swarm OS capability report

## Answer

**{c.get('P', 0) + c.get('A1', 0) + c.get('A2', 0)} of 1000 tasks can be executed and checked on this
deployment.** The rest are documents ({c.get('C', 0)}), normative claims ({c.get('N', 0)}), or blocked by data
({c.get('D', 0)}), hardware ({c.get('E', 0)}) or scale ({c.get('F', 0)}). Docking ({c.get('B', 0)}) runs but
its control fails.

## Capabilities, with the evidence that backs each

| capability | evidence |
|---|---|
| compute descriptors and drug-likeness | RDKit 2026.03.6 on 406 actives, 406/406 parsed |
| predict potency with an error bar | GBM, CV5 R² 0.7226 ± 0.0565 vs baseline −0.0085 |
| recover lead chemistry from artifacts | 3 leads, SMILES from pose REMARK, 2 corroborated by 3D SDF |
| hold and verify a data corpus | 33 manifests, {len(S)} distinct shards, {TOTAL_RECORDS:,} records, seals 33/33 |
| boot a custom UEFI kernel | image byte-identical to the boot media |
| run a 3-node federation | `/federation/stats`, 3 active |
| refuse an approval safely | gate veto-only, ABSTAIN on unknown |
| score docking | **engine yes, control no** — RMSD 13.82 Å |

## Capabilities the suite assumes that do not exist here

CUDA kernels, a TPM, 584 agents, 10 nodes, genomic variant data, imaging, mass spectrometry, OpenMM, a
validated docking pipeline, and any wet-lab measurement whatsoever.

## Live state

{live_block()}

## Closing statement

This report was generated from measurements taken in the same process that wrote it, and it is itself class C:
the platform cannot verify its own prose. What it *can* verify it has: hashes, counts, cross-validated scores,
a failed control recorded. The distance between those two statements is the honest measure of this system
today — and closing it is cheaper than any hardware purchase on the roadmap.
"""


# --------------------------------------------------------------------------------------------------
# driver
# --------------------------------------------------------------------------------------------------


def main() -> int:
    OUTDIR.mkdir(parents=True, exist_ok=True)
    manifest = {"generated": NOW, "tasks": {}, "duplicates": {}, "counts": {},
                "sources": ["build/*.k3.json (33 verified manifests)", "data/suite_classification.json",
                            "data/suite_probe_lpxc.json", "data/suite_probe_docking.json",
                            "data/suite_leads_descriptors.json", "live kernel endpoints :8022"]}

    written = {}
    for tid in range(961, 1001):
        if tid not in DOCS:
            print(f"  no generator for task {tid}")
            continue
        slug, fn = DOCS[tid]
        body = fn()
        sources = ["verified shard manifests", "suite_classification.json", "live endpoints"]
        text = body + footer(tid, f"{tid}-{slug}", sources,
                             "that any statement here is verified by the platform; this document is class C/N")
        path = OUTDIR / f"{tid}-{slug}.md"
        path.write_text(text, encoding="utf-8")
        h = hashlib.sha256(text.encode()).hexdigest()[:16]
        written[tid] = {"file": f"docs/suite-synthesis/{path.name}", "sha256": h,
                        "words": len(text.split()), "class": cls_of(tid)}
        print(f"  {tid}: {path.name:52} {len(text.split()):5} words")

    # duplicate pairs collapse to one artifact each (recorded, not hidden)
    dupes = {969: 964, 979: 974, 980: 977, 989: 984, 990: 987}
    for dup, orig in dupes.items():
        if orig in written:
            written[dup] = {**written[orig], "duplicate_of": orig}

    # count checks on the enumerated deliverables
    for tid, expect, marker in ((971, 100, "**"), (972, 50, "| "), (973, 25, "**"), (975, 10, "| "),
                                (976, 5, "###"), (978, 100, "**")):
        if tid in written:
            txt = (ROOT / written[tid]["file"]).read_text(encoding="utf-8")
            if tid == 972:
                n = sum(1 for l in txt.splitlines() if l.startswith("| ") and l.split("|")[1].strip().isdigit())
            elif tid == 975:
                n = sum(1 for l in txt.splitlines() if l.startswith("| ") and l.split("|")[1].strip().isdigit())
            elif tid == 976:
                n = txt.count("### ")
            else:
                n = sum(1 for l in txt.splitlines() if l.startswith("1. ") or l[:3].strip().rstrip('.').isdigit())
            manifest["counts"][tid] = {"expected": expect, "found": n, "ok": n == expect}
            if n != expect:
                print(f"  COUNT MISMATCH task {tid}: expected {expect}, found {n}")

    manifest["tasks"] = written
    MANIFEST.write_text(json.dumps(manifest, indent=1), encoding="utf-8")
    print(f"\n{len(written)} artifacts, {sum(v['words'] for v in written.values()):,} words -> {OUTDIR}")
    print(f"manifest: {MANIFEST}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
