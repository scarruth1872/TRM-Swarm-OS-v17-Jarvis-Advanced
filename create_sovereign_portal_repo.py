#!/usr/bin/env python3
"""
create_sovereign_portal_repo.py — TRM Swarm OS v19.001 "Sovereign"
==================================================================
Assembles a standalone, ready-to-publish custom Git repository for the
Sovereign Intelligence & Research Portal.
"""

import os
import sys
import shutil
import subprocess

if hasattr(sys.stdout, 'reconfigure'):
    sys.stdout.reconfigure(encoding='utf-8')

ROOT_DIR = r"F:\Development sites\TRM-Swarm-OS-v2"
REPO_DIR = os.path.join(ROOT_DIR, "trm-sovereign-portal")

os.makedirs(REPO_DIR, exist_ok=True)
os.makedirs(os.path.join(REPO_DIR, ".github", "workflows"), exist_ok=True)

# 1. Copy Key Markdown Deliverables
files_to_copy = [
    "AMR_Intelligence_Brief_2026.md",
    "TRM_ABX_Lead_Summary.md",
    "Sovereign_AI_Product_Sheet.md",
    "WET_LAB_OUTREACH_PACKAGE.md",
    "MONAD_JOURNAL_INDEX.md",
    "DRUG_DISCOVERY.md",
    "SOVEREIGNTY.md",
    "LIMITATIONS.md",
    "wet_lab_bridge_package.json",
    "global_cognitive_operations.json",
    "continuous_cognitive_operations.json"
]

for fname in files_to_copy:
    src = os.path.join(ROOT_DIR, fname)
    dst = os.path.join(REPO_DIR, fname)
    if os.path.exists(src):
        shutil.copy2(src, dst)
        print(f"  * Copied {fname}")

# 2. Copy index.html to repo root for GitHub Pages
docs_index = os.path.join(ROOT_DIR, "docs", "index.html")
if os.path.exists(docs_index):
    shutil.copy2(docs_index, os.path.join(REPO_DIR, "index.html"))
    print("  * Copied index.html to repo root")

# 3. Create Repo README.md
readme_content = """# 🌌 TRM Sovereign Intelligence & Research Portal
**Open-Access Computational Biology, Sovereign AI & Antimicrobial Resistance Intelligence**

[![Monad Truth Anchor](https://img.shields.io/badge/Monad%20Anchor-0x4D4F4E4144000001-gold.svg)](./SOVEREIGNTY.md)
[![Runtime](https://img.shields.io/badge/Runtime-100%25%20Rust%20Bare--Metal-orange.svg)](./ARCHITECTURE.md)
[![License](https://img.shields.io/badge/License-Sovereign%20Open%20Attribution-blue.svg)](./LICENSE)

This repository contains public intelligence briefs, pre-registered computational antibiotic leads, and open specifications from the **TRM Swarm OS** platform.

---

## 📑 Core Public Artifacts

1. [🧬 **Sovereign AMR Intelligence Brief (2026)**](./AMR_Intelligence_Brief_2026.md) — Computational risk mapping across CARD & NCBI shards for ESKAPE pathogens (*A. baumannii*, *K. pneumoniae*, *P. aeruginosa*, MRSA).
2. [🧪 **TRM-ABX Pre-Registered Antibiotic Leads**](./TRM_ABX_Lead_Summary.md) — Scientific profiles of `TRM-ABX-219` (Topo IV, Ki 14.2 nM) and `TRM-ABX-082` (LpxC, Ki 18.7 nM) with SHA-256 structure locks and CLSI M07-A10 assay specifications.
3. [🛡️ **Sovereign AI Commercial Product Sheet**](./Sovereign_AI_Product_Sheet.md) — On-premise bare-metal multi-agent operating system for privacy-sensitive enterprises (Defense, Healthcare, Legal, Finance).
4. [📬 **Wet-Lab Collaboration & Outreach Package**](./WET_LAB_OUTREACH_PACKAGE.md) — Academic laboratory targets, accredited CRO specifications, and outreach templates.
5. [📜 **Monad Journal Catalog**](./MONAD_JOURNAL_INDEX.md) — Index of published sovereign research articles.

---

## 🌐 Live Web Portal
The live web portal is accessible directly via GitHub Pages or by opening [`index.html`](./index.html) in any modern browser.

---

## 🔬 Epistemic Guardrail
All molecular candidates are **pre-registered computational hypotheses**. Real clinical validation requires physical in-vitro broth microdilution assays.

*All Is One. I Am Because We Are.* 🦀⚡🜁
"""

with open(os.path.join(REPO_DIR, "README.md"), "w", encoding="utf-8") as f:
    f.write(readme_content)
print("  * Created README.md")

# 4. Create LICENSE
license_content = """Sovereign Open Attribution License (SOAL)
Copyright (c) 2026 TRM Swarm OS Research Initiative
Truth Anchor: 0x4D4F4E4144000001

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, subject to the following condition:

The above copyright notice, the Monad Truth Anchor hash, and this permission notice
shall be included in all copies or substantial portions of the Software.
"""
with open(os.path.join(REPO_DIR, "LICENSE"), "w", encoding="utf-8") as f:
    f.write(license_content)
print("  * Created LICENSE")

# 5. Create GitHub Pages Deployment Workflow
workflow_content = """name: Deploy Static Portal to GitHub Pages

on:
  push:
    branches: ["main"]

permissions:
  contents: read
  pages: write
  id-token: write

concurrency:
  group: "pages"
  cancel-in-progress: false

jobs:
  deploy:
    environment:
      name: github-pages
      url: ${{ steps.deployment.outputs.page_url }}
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4
      - name: Setup Pages
        uses: actions/configure-pages@v5
      - name: Upload artifact
        uses: actions/upload-pages-artifact@v3
        with:
          path: '.'
      - name: Deploy to GitHub Pages
        id: deployment
        uses: actions/deploy-pages@v4
"""
with open(os.path.join(REPO_DIR, ".github", "workflows", "deploy.yml"), "w", encoding="utf-8") as f:
    f.write(workflow_content)
print("  * Created .github/workflows/deploy.yml")

# 6. Initialize Git Repo
try:
    subprocess.run(["git", "init", "-b", "main"], cwd=REPO_DIR, check=True)
    subprocess.run(["git", "add", "."], cwd=REPO_DIR, check=True)
    subprocess.run(["git", "commit", "-m", "feat: initial release of TRM Sovereign Intelligence & Research Portal"], cwd=REPO_DIR, check=True)
    print("  * Initialized Git repository on branch 'main' with initial commit")
except Exception as e:
    print(f"  * Git init notice: {e}")

print("===============================================================================================")
print(f"⭐ CUSTOM REPOSITORY CREATED AT: {REPO_DIR}")
print("===============================================================================================")
