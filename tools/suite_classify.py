#!/usr/bin/env python3
"""Classify the 1000-task suite against what THIS machine actually has, and say why for each task.

Method
------
Every task gets exactly one class, decided by ordered regex rules over its own text, and the rule that
fired is recorded. The capability side of each rule is a measured fact (tools/suite_probe_*.py,
data/suite_probe_*.json), not an assumption:

  P  PROVEN            already delivered and verified on this host
  A1 COMPUTE           a measurement can be computed from data that is on disk
  A2 CODE              an implementation is buildable here and verifiable by cargo build/test
  B  UNVALIDATED       a tool runs but its control fails, so its numbers are not evidence
  C  ARTIFACT_ONLY     the deliverable is a document; it exists but nothing verifies it
  D  DATA_BLOCKED      needs a dataset that is not on this host
  E  HW_BLOCKED        needs hardware this host does not have
  F  SCALE_BLOCKED     needs nodes/agents this deployment does not have

Ordering is deliberate: hardware and data absence outrank a runnable tool ("predict X from Hi-C data" is
data-blocked even though sklearn runs), and a failing control outranks a runnable tool ("dock X" is
unvalidated even though vina runs).

Usage: python tools/suite_classify.py [--suite PATH] [--out PATH]
"""
from __future__ import annotations

import argparse
import collections
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DEFAULT_SUITE = Path.home() / "Downloads" / "TRM Swarm OS — 1000 Complex Multi-Step Task Suite.txt"

DOMAINS = [
    ("Drug Discovery & Biomedical", 1, 100), ("Genomics & Variant Analysis", 101, 200),
    ("Network & Systems Engineering", 201, 300), ("Security & Cryptography", 301, 400),
    ("Data Science & Machine Learning", 401, 500), ("Knowledge Engineering", 501, 600),
    ("Hardware & Embedded Systems", 601, 700), ("Federation & Distributed Systems", 701, 800),
    ("Humanistic & Ethical Reasoning", 801, 900), ("Creative & Generative Work", 901, 1000),
]

# (class, compiled pattern, reason) -- first match wins
RULES: list[tuple[str, re.Pattern, str]] = [
    # P -- PROVEN: delivered and verified on this host (build + byte-identical boot media)
    ("P", re.compile(r"uefi bootloader|bare-metal kernel image|bootable TRM kernel|boot sequence for TRM", re.I),
     "delivered here: trm_uefi_kernel.efi (17,488,896 B) is byte-identical to G:/EFI/BOOT/BOOTX64.EFI on a FAT32 SWARM_OS volume"),

    # E -- HARDWARE absent. Measured: no CUDA toolkit (RX 6700 XT is Vulkan-only; the 3070 peer has no nvcc),
    # no FPGA/vendor synthesis tools, Get-Tpm TpmPresent False, no SDR/muon/robot/IoT device.
    ("E", re.compile(r"\bcuda\b|\bgpu\b|nvcc|tensor core|gpu[- ]based", re.I),
     "no CUDA toolkit on this host or the 3070 peer (nvcc absent on both); the desktop GPU is Vulkan-only"),
    ("E", re.compile(r"\bfpga\b|verilog|vhdl|logic synthesis|reconfigurable computing|custom instruction set|fpga synthesis", re.I),
     "no FPGA and no vendor synthesis tools on this host"),
    ("E", re.compile(r"tpm 2\.0|\btpm\b|secure enclave|\bhsm\b|physical unclonable|\bpuf\b|hardware root of trust|hardware attestation", re.I),
     "hardware root of trust absent: Get-Tpm reports TpmPresent False; no enclave, no HSM, no PUF"),
    ("E", re.compile(r"\bsdr\b|muon|ionosonde", re.I),
     "no SDR and no muon detector is attached to this host"),

    # F -- SCALE absent. Measured: /federation/stats = 3 configured peers (3 active), 14 mesh nodes.
    ("F", re.compile(r"584 agents|10[- ]node|100[- ]node|1000[- ]node|multi-node orchestration|node lifecycle|mesh deployment|mesh scaling|mesh self-healing", re.I),
     "scale absent: /federation/stats reports 3 configured peers and 14 mesh nodes, not 584 agents or 10 nodes"),

    # D -- DATA absent. Measured: no ClinVar/gnomAD/VCF, no ATAC/Hi-C/methylation, no MS/RNA-seq, no imaging corpus.
    ("D", re.compile(r"clinvar|gnomad|1000 genomes|\bvcf\b|exome|trio analysis|acmg|\bagai\b|resfinder|1000 bacterial strains|1000 clinical isolates", re.I),
     "no genomic variant or strain-genome data on this host (no ClinVar, gnomAD, VCF, exome or ResFinder files)"),
    ("D", re.compile(r"atac|hi-c\b|methylation|chip-seq|chromatin|histone|epigenetic", re.I),
     "no epigenomic data on this host (no ATAC-seq, Hi-C or methylation files)"),
    ("D", re.compile(r"mass spectrom|proteomic|phosphoproteomic|rna-seq|transcriptom|single-cell|microbiome", re.I),
     "no omics measurement data on this host (no MS, RNA-seq or single-cell files)"),
    ("D", re.compile(r"pathology|microscopy|histolog|image segmentation|object detection|video analysis|medical imaging|3d reconstruction", re.I),
     "no imaging corpus and no vision weights; torch here is CPU-only"),
    ("D", re.compile(r"tumou?r|neoantigen|cancer genome|patient sample|patients|\bcohort\b|clinical database|horizontal gene transfer|clinical isolate", re.I),
     "no patient/tumour data on this host, and none may be sourced without explicit approval"),

    # L -- the molecule the task names has no structure, so the computation cannot be run as written.
    ("L", re.compile(r"trm-abx-220|trm leads|trm-tbx-001|trm-ntd", re.I),
     "TRM-ABX-220 has no SMILES anywhere in the repo (wet-lab audit), so any descriptor/docking/PK task about it is blocked at the molecule; the same computation runs on the 406 real LpxC actives in the shard"),

    # B -- engine runs, control fails. Measured: vina 1.2.7 completes a dock in 22 s, redock RMSD 13.82 A.
    ("B", re.compile(r"dock|binding affinity|free energy perturbation|binding mode|binding pose|proteome-wide", re.I),
     "vina 1.2.7 runs but the shipped receptor/ligand pairs are in different frames (redock RMSD 13.82 A, positive affinity at a tight box), so its numbers are not evidence"),

    # A1 -- a measurement from data measured present on disk
    ("A1", re.compile(r"drug-likeness|lipinski|veber|ghose|descriptor|partition coefficient", re.I),
     "RDKit 2026.03.6 (py3.12) computes these; measured on 406 real LpxC actives"),
    ("A1", re.compile(r"qsar|model training|activity class", re.I),
     "sklearn 1.3.2 (py3.11) trains on local shards; measured CV5 R2 0.7226 on 406 LpxC actives"),
    ("A1", re.compile(r"bayesian inference|hypothesis testing framework|survival analysis|multivariate regression|time-series analysis|bootstrapping|statistical power analysis|mixed-effects|causal inference", re.I),
     "numpy 1.26 / scipy 1.10 / pandas 2.0 / sklearn 1.3.2 installed (py3.11)"),
    ("A1", re.compile(r"lipid a|biosynthesis pathway|pathway.{0,20}(map|biosynthesis)|enzymes and intermediates", re.I),
     "SMPDB (147,932 records) and ChEMBL LpxC (406) are on disk and queryable"),
    ("A1", re.compile(r"resistance surveillance|amr surveillance|resistance mechanism|\bamr\b|eskape|cross-resistance", re.I),
     "CARD 6,463 / NCBI-AMR 8,143 / ESKAPE 25,888 real records are on disk"),
    ("A1", re.compile(r"tox21|cytotoxicity|mitochondrial tox", re.I),
     "Tox21 (7,831 real assay records) is on disk for measured endpoints"),
    ("A1", re.compile(r"ontology from|ontology alignment|ontology matching|ontology hierarchy|ontology classification|ontology versioning|gene ontology|knowledge graph from|knowledge graph embedding|knowledge graph completion|temporal knowledge graph|probabilistic knowledge graph|knowledge graph question|\brdf\b triples|schema matching|entity resolution|data lineage|data quality|data catalog|topic model|document clustering", re.I),
     "the fabric holds 16 real shards (10.8M records) and build_council_index.py works; no OWL/SPARQL reasoner is installed, so reasoning steps are partial"),

    # A2 -- a real implementation, buildable here, verifiable by cargo build + test (141 tests pass today)
    ("A2", re.compile(r"lock-free|memory-mapped|zero-copy|\bsimd\b|allocator|priority queue|signal handling|watchdog timer|"
                      r"\bhash|merkle|chacha|ed25519|\baes\b|constant-time|random number generation|circuit breaker|"
                      r"exponential backoff|checksum|\bwasm\b|seccomp|cgroup|namespace isolation|\braft\b|paxos|mapreduce|"
                      r"b-tree|lsm-tree|write-ahead|content-addressed|\bquic\b|tokenizer|named entity|relation extraction|"
                      r"graph neural network|transformer architecture|q-learning|policy gradient|gradient descent|"
                      r"simulated annealing|particle swarm|bayesian optimization|genetic algorithm|vector clock|\bcrdt\b|"
                      r"gossip protocol|distributed hash table|consistent hash|\bipc\b|\bber\b|btree|"
                      r"sparql query engine|bayesian optimization|graph attention|regression testing framework|"
                      r"hypothesis testing framework", re.I),
     "an implementation is buildable here and verifiable by cargo build + test (141 tests currently pass)"),

    # N -- normative: no measurement can settle it, so the platform can write it but never check it
    ("N", re.compile(r"ethic|dignity|justice|consent|autonomy|discriminat|fairness|equity|equitable|cultural|indigenous|"
                     r"rights|governance|accountab|transparen|harm|mediation|reconcil|peace|cultural heritage|climate justice|"
                     r"universal access|living wage|commons", re.I),
     "a normative claim: nothing on this host can verify that a policy is just, dignified or fair — the platform can only write it"),
]

DEFAULT = ("C", "the deliverable is a written document or report; the platform can produce it (gpt-oss-20b on :8587) but nothing in the platform verifies its content")


def domain_of(tid: int) -> str:
    for name, lo, hi in DOMAINS:
        if lo <= tid <= hi:
            return name
    return "unassigned"


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--suite", default=str(DEFAULT_SUITE))
    ap.add_argument("--out", default=str(ROOT / "data" / "suite_classification.json"))
    a = ap.parse_args()

    raw = Path(a.suite).read_text(encoding="utf-8", errors="replace")
    tasks = [{"id": int(m.group(1)), "text": m.group(2).strip()}
             for m in re.finditer(r"^Task (\d+): (.+)$", raw, re.M)]
    if len(tasks) != 1000:
        print(f"CLASSIFY: parsed {len(tasks)} tasks, expected 1000 — refusing to score a partial suite")
        return 2

    rows = []
    for t in tasks:
        cls, why = DEFAULT
        for c, pat, reason in RULES:
            if pat.search(t["text"]):
                cls, why = c, reason
                break
        rows.append({**t, "domain": domain_of(t["id"]), "class": cls, "reason": why})

    by_domain = collections.OrderedDict()
    for name, lo, hi in DOMAINS:
        sub = [r for r in rows if lo <= r["id"] <= hi]
        c = collections.Counter(r["class"] for r in sub)
        by_domain[name] = {k: c.get(k, 0) for k in ("P", "A1", "A2", "B", "C", "N", "L", "D", "E", "F")}
        by_domain[name]["total"] = len(sub)

    totals = collections.Counter(r["class"] for r in rows)
    out = {"suite": str(a.suite), "n_tasks": len(rows),
           "totals": {k: totals.get(k, 0) for k in ("P", "A1", "A2", "B", "C", "N", "L", "D", "E", "F")},
           "by_domain": by_domain, "tasks": rows}
    Path(a.out).write_text(json.dumps(out, indent=1), encoding="utf-8")

    width = max(len(n) for n, _, _ in DOMAINS)
    cols = ("P", "A1", "A2", "B", "C", "N", "L", "D", "E", "F")
    print(f"{'domain'.ljust(width)}  " + " ".join(f"{c:>3}" for c in cols))
    for name, counts in by_domain.items():
        print(f"{name.ljust(width)}  " + " ".join(f"{counts.get(k,0):3}" for k in cols))
    print(f"{'TOTAL'.ljust(width)}  " + " ".join(f"{out['totals'][k]:3}" for k in cols))
    print("\nclasses: P proven, A1 compute-from-local-data, A2 code+build, B unvalidated, "
          "C document-only, D data-blocked, E hardware-blocked, F scale-blocked")
    print(f"wrote {a.out}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
