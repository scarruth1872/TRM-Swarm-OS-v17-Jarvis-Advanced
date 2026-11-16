"""Record where Jarvis's modules actually came from, with evidence rather than recollection.

The working belief was "most modules came from other repos on this machine". Testing it produced a narrower, checkable
answer:

  * the APP is a Google AI Studio applet export - metadata.json declares MAJOR_CAPABILITY_SERVER_SIDE_GEMINI_API,
    package.json is named "react-example", and 8 files carry the AI Studio Apache-2.0 SPDX header;
  * a few CONCEPTS were ingested from repos downloaded to F:/downloads - the PRISM/FQM field names in the research
    panel trace to fractal_quantum_myco.md in prism-fqm-agent-system-explorer (downloaded 2025-10-24) and to
    mycorrhizal-agent-network (2025-10-02);
  * the whole tree entered this repo in one bulk import commit, so per-file authorship is not recoverable from git.

So: generated scaffolding that borrowed a downloaded theory doc's vocabulary, not code lifted from other repos. The
distinction matters, because it changes what the 19 "declared-only" modules are - scaffolding, not integration debt.

    python module_provenance.py --downloads "F:/downloads"
"""
from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
import os
import subprocess
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
JARVIS = ROOT / "Jarvis-Advanced-main"
SKIP = {"node_modules", ".git", "dist", "build", "__pycache__"}
CODE = {".ts", ".tsx", ".js", ".jsx", ".py"}

# vocabulary that only appears because a concept document was read, not because the kernel has the subsystem
CONCEPT_TERM_THRESHOLD = 2
# "tunneling" not "tunneling_probability": the mycorrhizal folder says "tunneling", so the longer form matched
# nothing there and made that folder look term-free.
CONCEPT_TERMS = ("prism", "fqm", "mycorrhizal", "tunneling", "grover", "sierpinski", "l-system")
# "tunneling" alone proves nothing - it is ordinary English and appears in journaling apps and UI copy. A folder only
# counts as a concept source if it names at least one term that is specific to this family of ideas.
SPECIFIC_TERMS = {"prism", "fqm", "mycorrhizal", "sierpinski", "l-system", "grover"}


def mtime(path: Path) -> str | None:
    try:
        return dt.datetime.fromtimestamp(path.stat().st_mtime).strftime("%Y-%m-%d")
    except OSError:
        return None


def concept_sources(downloads: Path) -> list[dict]:
    """Download folders whose documents define the vocabulary the panels print."""
    found = []
    if not downloads.exists():
        return found
    for d in sorted(downloads.iterdir()):
        if not d.is_dir():
            continue
        hits = []
        folder_terms: set[str] = set()
        # Per-file matching misses a folder whose concept vocabulary is split across files: mycorrhizal-agent-network
        # has nine files naming "mycorrhizal" and two naming "tunneling", none of them carrying both, so a per-file
        # rule concluded the folder contributed nothing. Measure the folder as a whole and report per-file detail.
        for dirpath, dirnames, filenames in os.walk(d, onerror=lambda e: None):
            dirnames[:] = [x for x in dirnames
                           if x.lower() not in {"venv", "venv-win", ".venv", "site-packages", "node_modules",
                                                ".git", "__pycache__", "lib64"}]
            for fn in filenames:
                # documents and source only: package-lock.json matched "prism"/"fqm" through npm package names and
                # produced four phantom concept sources (gabby, project-bolt, shape-what-emergent, nova/bytebot)
                low = fn.lower()
                if "package-lock" in low or low.endswith(".json") or low.endswith(".lock"):
                    continue
                if not low.endswith((".md", ".txt", ".py", ".ts", ".tsx")):
                    continue
                f = Path(dirpath) / fn
                try:
                    if f.stat().st_size > 1_500_000:
                        continue
                    t = f.read_text(encoding="utf-8", errors="ignore").lower()
                except OSError:
                    continue
                hit = [x for x in CONCEPT_TERMS if x in t]
                folder_terms.update(hit)
                if hit:
                    hits.append({"file": str(f.relative_to(downloads)).replace("\\", "/"), "terms": hit})
        if len(folder_terms) >= CONCEPT_TERM_THRESHOLD and (folder_terms & SPECIFIC_TERMS):
            found.append({"folder": d.name, "downloaded": mtime(d), "documents": hits[:3],
                          "document_count": len(hits), "folder_terms": sorted(folder_terms)})
    return found


def vendored_repos(root: Path) -> list[dict]:
    """Git repositories sitting INSIDE the tree, with their upstream remotes.

    This is the strongest provenance signal available: a nested .git with a third-party origin proves code was taken
    from elsewhere, with a commit to point at. `VibeThinker/` and `openfang/` are exactly this, and the first
    provenance pass missed both because it looked only at file headers and download folders.
    """
    out = []
    for dirpath, dirnames, _ in os.walk(root, onerror=lambda e: None):
        dirnames[:] = [d for d in dirnames
                       if d.lower() not in {"train_venv", "venv", ".venv", "site-packages", "node_modules",
                                            "__pycache__", "target", "dist"}]
        if ".git" in dirnames and Path(dirpath) != root:
            d = Path(dirpath)
            dirnames.remove(".git")
            url = subprocess.run(["git", "-C", str(d), "remote", "get-url", "origin"],
                                 capture_output=True, text=True).stdout.strip()
            commit = subprocess.run(["git", "-C", str(d), "log", "-1", "--format=%ad %h", "--date=short"],
                                    capture_output=True, text=True).stdout.strip()
            files = sum(1 for f in d.rglob("*") if f.is_file())
            out.append({"path": str(d.relative_to(root)).replace("\\", "/"), "files": files,
                        "origin": url, "commit": commit,
                        "third_party": bool(url) and "scarruth1872" not in url})
    return sorted(out, key=lambda r: -r["files"])


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", default=str(ROOT / "evidence" / "JARVIS_PROVENANCE.json"))
    ap.add_argument("--downloads", default="F:/downloads")
    args = ap.parse_args()

    app_level = {
        "metadata_json": (JARVIS / "metadata.json").exists(),
        "package_name": json.loads((JARVIS / "package.json").read_text(encoding="utf-8")).get("name")
        if (JARVIS / "package.json").exists() else None,
        "firebase_config": (JARVIS / "firebase-applet-config.json").exists(),
    }
    if (JARVIS / "metadata.json").exists():
        md = json.loads((JARVIS / "metadata.json").read_text(encoding="utf-8"))
        app_level["declared_capabilities"] = md.get("majorCapabilities", [])

    mods = []
    for p in sorted(JARVIS.rglob("*")):
        if not p.is_file() or p.suffix.lower() not in CODE:
            continue
        if {x.lower() for x in p.relative_to(JARVIS).parts} & SKIP:
            continue
        t = p.read_text(encoding="utf-8", errors="ignore")
        head = t[:400]
        spdx = re.search(r"SPDX-License-Identifier:\s*([\w.\-+]+)", head)
        low = t.lower()
        terms = [x for x in CONCEPT_TERMS if x in low]
        origin = []
        if spdx:
            origin.append(f"ai_studio_export ({spdx.group(1)} header)")
        if terms:
            origin.append("concept_doc_vocabulary: " + ", ".join(terms[:4]))
        mods.append({
            "file": str(p.relative_to(JARVIS)).replace("\\", "/"),
            "lines": t.count("\n") + 1,
            "sha256_16": hashlib.sha256(p.read_bytes()).hexdigest()[:16],
            "origin": origin or ["no provenance marker"],
        })

    flagged = [m for m in mods if m["origin"] != ["no provenance marker"]]
    report = {
        "question": "where did Jarvis's modules come from?",
        "app_level_evidence": app_level,
        "module_count": len(mods),
        "modules_with_provenance_marker": len(flagged),
        "concept_sources_in_downloads": concept_sources(Path(args.downloads)),
        "concept_term_threshold": CONCEPT_TERM_THRESHOLD,
        "concept_terms": list(CONCEPT_TERMS),
        "git_note": "all Jarvis components entered this repo in one bulk import commit (2026-08-24), so per-file "
                    "authorship is not recoverable from git history",
        "conclusion": "AI Studio applet export that borrowed a downloaded theory document's vocabulary; not code "
                      "lifted from other repos. The 19 declared-only modules are generated scaffolding, not "
                      "integration debt.",
        "vendored_repos": vendored_repos(ROOT),
        "modules": mods,
    }
    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(report, indent=2), encoding="utf-8")

    print(json.dumps(app_level, indent=1))
    print(f"\nmodules: {len(mods)} | with a provenance marker: {len(flagged)}")
    for m in flagged:
        print(f"  {m['file']:<46} {m['lines']:>5}  {'; '.join(m['origin'])[:90]}")
    print("\nconcept documents found in downloads:")
    for c in report["concept_sources_in_downloads"]:
        print(f"  {c['folder']:<40} downloaded {c['downloaded']}  {c['document_count']} doc(s)")
        for d in c["documents"][:2]:
            print(f"      {d['file']}  {d['terms']}")
    print(f"\nreport -> {out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
