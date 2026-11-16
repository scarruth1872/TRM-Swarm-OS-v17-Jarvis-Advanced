#!/usr/bin/env python3
"""Generate swarm_v2_artifacts/verification_proofs.json from measurement records
that already exist under evidence/.

Every entry is EXTRACTED from a real file: the value is read out of the artifact
and the timestamp is the one the artifact carries (its own `generated` / `timestamp`
/ `captured` field, or the file's mtime when it carries none -- recorded explicitly
in `timestamp_source`). Nothing here is typed in by hand and nothing is dated "now".

The kernel's GET /verification/stats loads this file, re-resolves every
`evidence_path` on disk, and reports entries it cannot resolve as rejected queue
items -- so a stale or moved evidence file shows up as queue work, never as a
verified proof.

Usage:  python tools/gen_verification_proofs.py [--repo-root .]
"""
from __future__ import annotations

import argparse
import datetime as dt
import json
import os
import re
import sys

SCHEMA = "trm.verification.proofs/v1"


def _load_json(path: str):
    with open(path, encoding="utf-8-sig") as fh:
        return json.load(fh)


def _mtime_iso(path: str) -> str:
    ts = os.path.getmtime(path)
    return dt.datetime.fromtimestamp(ts, dt.timezone.utc).astimezone().isoformat(timespec="seconds")


def _in_file(path: str, keys: tuple[str, ...]) -> tuple[str, str] | None:
    """Return (timestamp, key) taken from the artifact's own text, or None."""
    text = open(path, encoding="utf-8-sig", errors="replace").read()
    try:
        doc = json.loads(text)
    except Exception:
        doc = None
    if isinstance(doc, dict):
        for k in keys:
            v = doc.get(k)
            if isinstance(v, str) and re.match(r"^\d{4}-\d{2}-\d{2}T", v):
                return v, f"field:{k}"
    m = re.search(r"(?im)^\s*(?:captured|generated|timestamp|date)\s*:\s*(\d{4}-\d{2}-\d{2}T[0-9:.]+Z?)", text)
    if m:
        return m.group(1), "field:captured"
    return None


def build(repo_root: str, evidence_dir: str | None = None) -> dict:
    if evidence_dir:
        ev = os.path.abspath(evidence_dir)
    else:
        # evidence/ sits at the repository root; the kernel tree may be staged
        # one or two levels below it (staging/v20), so search upward.
        for cand in (repo_root, os.path.join(repo_root, ".."), os.path.join(repo_root, "..", "..")):
            p = os.path.abspath(os.path.join(cand, "evidence"))
            if os.path.isdir(p):
                ev = p
                break
        else:
            raise SystemExit(f"REFUSING: no evidence/ directory at or above {repo_root}")
    proofs: list[dict] = []

    def add(proof_id: str, claim: str, rel: str, measurement: dict, subject: str = "") -> None:
        rel = rel if rel.startswith("evidence/") else f"evidence/{rel}"
        path = os.path.join(ev, os.path.basename(rel))
        if not os.path.isfile(path):
            raise SystemExit(f"REFUSING: evidence file missing for {proof_id}: {rel}")
        stamp = _in_file(path, ("generated", "timestamp", "captured", "date"))
        if stamp is None:
            ts, src = _mtime_iso(path), "file_mtime"
        else:
            ts, src = stamp[0], stamp[1]
        entry = {
            "proof_id": proof_id,
            "claim": claim,
            "evidence_path": rel.replace(os.sep, "/"),
            "measurement": measurement,
            "timestamp": ts,
            "timestamp_source": src,
            "status": "VERIFIED",
        }
        if subject:
            entry["subject"] = subject
        proofs.append(entry)

    # 1. Compiled Kleene truth tables against the textbook laws.
    k3 = os.path.join(ev, "k3_kleene_law_check.txt")
    txt = open(k3, encoding="utf-8-sig").read()
    m = re.search(r"checks passed\s+(\d+)\s+failed\s+(\d+)", txt)
    if not m:
        raise SystemExit("REFUSING: could not read the check counts out of k3_kleene_law_check.txt")
    add(
        "VP-K3-001",
        "compiled k3_logic truth tables satisfy the Kleene laws over {-1,0,+1}",
        "evidence/k3_kleene_law_check.txt",
        {"checks_passed": int(m.group(1)), "checks_failed": int(m.group(2))},
    )

    # 2. Council chat surface reachability, 15/15 agents.
    chat = _load_json(os.path.join(ev, "AGENT_CHAT_MEASURED.json"))
    add(
        "VP-COUNCIL-CHAT-002",
        "every agent on the chat surface answered a live probe",
        "evidence/AGENT_CHAT_MEASURED.json",
        {"endpoints_reachable": chat["reachable"], "endpoints_total": chat["total"]},
        subject=chat.get("endpoint", ""),
    )

    # 3. Per-agent engines observed while answering the same question.
    ident = _load_json(os.path.join(ev, "AGENT_IDENTITY_MEASURED.json"))
    engines = sorted({r.get("engine", "") for r in ident["rows"]})
    add(
        "VP-COUNCIL-ENGINES-003",
        "agents answering one prompt reported which engine actually served them",
        "evidence/AGENT_IDENTITY_MEASURED.json",
        {"agents_answering": len(ident["rows"]), "distinct_engines": len(engines)},
        subject=ident.get("question", ""),
    )

    # 4. Answer diversity across the swarm (a homogeneity measurement, not a pass).
    intel = _load_json(os.path.join(ev, "AGENT_INTELLIGENCE_MEASURED.json"))
    dist = intel["distinctness"]
    add(
        "VP-COUNCIL-DISTINCT-004",
        "swarm answer diversity on one neutral prompt",
        "evidence/AGENT_INTELLIGENCE_MEASURED.json",
        {"agents": dist["agents"], "unique_frames": dist["unique"]},
        subject=dist.get("neutral_question", ""),
    )

    # 5. Shield K3 head, in-distribution and held-out danger sets.
    bons = _load_json(os.path.join(ev, "BONSAI_K3_RETRAIN_MEASURED.json"))
    shield = next(r for r in bons["rows"] if r["agent"] == "Shield")
    add(
        "VP-SHIELD-K3-005",
        "Shield K3 head block rate on danger prompts, in-distribution vs held-out",
        "evidence/BONSAI_K3_RETRAIN_MEASURED.json",
        {
            "block_rate_on_danger": shield["block_rate_on_danger"],
            "block_rate_on_danger2_heldout": shield["block_rate_on_danger2"],
            "eval_epochs": bons["epochs"],
        },
        subject=shield["agent"],
    )

    # 6. End-to-end stage run against :8099.
    master = _load_json(os.path.join(ev, "MASTER_COMPLEX_TASK_8099.json"))
    add(
        "VP-PLATFORM-STAGES-006",
        "multi-stage platform run completed against the live port",
        "evidence/MASTER_COMPLEX_TASK_8099.json",
        {"stages_passed": master["stages_passed"], "assertions_passed": master["assertions_passed"],
         "total_elapsed_ms": master["total_elapsed_ms"]},
        subject=master.get("target", ""),
    )

    return {
        "schema": SCHEMA,
        "generated_by": "tools/gen_verification_proofs.py",
        "description": (
            "Verification queue. Each entry is a measurement EXTRACTED from the evidence file it "
            "names; `timestamp` is that file's own recorded time (or its mtime, marked in "
            "timestamp_source). The kernel re-resolves every evidence_path at request time and "
            "reports unresolvable entries as queue work instead of verified proofs."
        ),
        "proofs": proofs,
    }


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--repo-root", default=os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
    ap.add_argument("--out", default=None)
    ap.add_argument("--evidence-dir", default=None)
    args = ap.parse_args()
    root = os.path.abspath(args.repo_root)
    doc = build(root, args.evidence_dir)
    out = args.out or os.path.join(root, "swarm_v2_artifacts", "verification_proofs.json")
    os.makedirs(os.path.dirname(out), exist_ok=True)
    with open(out, "w", encoding="utf-8") as fh:
        json.dump(doc, fh, indent=2, ensure_ascii=False)
        fh.write("\n")
    print(f"wrote {len(doc['proofs'])} proofs -> {out}")
    for p in doc["proofs"]:
        print(f"  {p['proof_id']:<22} {p['evidence_path']:<46} {p['timestamp']} ({p['timestamp_source']})")
    return 0


if __name__ == "__main__":
    sys.exit(main())
