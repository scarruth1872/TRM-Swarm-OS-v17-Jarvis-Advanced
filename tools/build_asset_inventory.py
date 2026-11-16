"""Build an authoritative asset inventory from the platform's own configuration files.

Why: the scope gate cannot judge an object it has never seen (object-holdout AUC 0.527, G144). Rather than asking the
model to guess what a noun means, read the answer out of the deployment itself — a service declared in
docker-compose.yml, a table created in a migration, a key demanded by .env. Those files ARE the definition of which
assets are load-bearing, and unlike a learned head they can be cited.

Every asset carries:
  phrase - how it would be named in a request ("the council service"), so it matches corpus phrasing
  kind   - service | volume | table | bucket | key | dataset | host | queue
  class  - prod | ephemeral, from explicit name markers, never guessed silently
  source - file:line, so the claim is checkable

The class heuristic is deliberately blunt and printed: an asset named test/mock/demo/sandbox/scratch/fixture/sample/
staging/tmp/example is ephemeral, everything else is production. A blunt rule that states itself beats a learned guess
that cannot.

    python build_asset_inventory.py --out data/asset_inventory.json
"""
from __future__ import annotations

import argparse
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

SKIP_DIRS = {".git", "node_modules", "target", "__pycache__", ".venv", "venv", "dist", "build", "scratch",
             "backups", "archive", "preserved_sources", "models"}
EPHEMERAL_MARKERS = ("test", "mock", "demo", "sandbox", "scratch", "fixture", "sample", "staging", "tmp",
                     "temp", "example", "dummy", "fake", "sim", "bench")
COMPOSE_NAMES = ("docker-compose.yml", "docker-compose.yaml", "docker-compose.live.yml",
                 "docker-compose.phase1.yaml")
ENV_NAMES = (".env", ".env.example")

SERVICE_RE = re.compile(r"^  ([a-zA-Z][\w.-]{1,40}):\s*$")
KEY_RE = re.compile(r"^\s*([A-Z][A-Z0-9_]{2,60})\s*=")
TABLE_RE = re.compile(r"create\s+table\s+(?:if\s+not\s+exists\s+)?[\"`]?([\w.]{1,60})", re.I)
VOLUME_RE = re.compile(r"^\s{4}-\s+([\w.-]+):/", re.M)
IMAGE_RE = re.compile(r"^\s+image:\s*([\w./:-]+)", re.M)


def classify(name: str) -> str:
    low = name.lower()
    return "ephemeral" if any(m in low for m in EPHEMERAL_MARKERS) else "prod"


def collect(root: Path):
    assets: dict[tuple[str, str], dict] = {}

    def put(name: str, kind: str, source: str, line: int):
        name = name.strip().strip("\"'`")
        if not name or len(name) < 2:
            return
        phrase = {
            "service": f"the {name} service",
            "volume": f"the {name} volume",
            "table": f"the {name} table",
            "credential": f"the {name} credential",
            "directory": f"the {name} directory",
            "host": f"the {name} endpoint",
            "queue": f"the {name} queue",
            "bucket": f"the {name} bucket",
        }.get(kind, f"the {name}")
        key = (kind, name.lower())
        if key not in assets:
            assets[key] = {"name": name, "phrase": phrase.lower(), "kind": kind,
                           "class": classify(name), "source": f"{source}:{line}"}

    for p in sorted(root.rglob("*")):
        if not p.is_file():
            continue
        parts = {x.lower() for x in p.relative_to(root).parts}
        if parts & SKIP_DIRS:
            continue
        low = p.name.lower()
        # Check the NAME first: reading every file in a 176k-file tree to find configs timed the scan out at 300 s.
        if not (low in COMPOSE_NAMES or low in ENV_NAMES or low.endswith(".sql")):
            continue
        rel = str(p.relative_to(root)).replace("\\", "/")
        try:
            text = p.read_text(encoding="utf-8", errors="ignore")
        except OSError:
            continue
        if low in COMPOSE_NAMES:
            for i, ln in enumerate(text.splitlines(), 1):
                m = SERVICE_RE.match(ln)
                # a service name sits at 2-space indent under `services:`; images are not services
                if m and not ln.strip().startswith("#"):
                    put(m.group(1), "service", rel, i)
                for vm in VOLUME_RE.finditer(ln):
                    put(vm.group(1), "volume", rel, i)
        elif low in ENV_NAMES and len(parts) <= 3:
            for i, ln in enumerate(text.splitlines(), 1):
                m = KEY_RE.match(ln)
                if not m:
                    continue
                k = m.group(1)
                kind = None
                if re.search(r"(_KEY|_TOKEN|_SECRET|_PASSWORD|_PASS|_CREDENTIALS?)$", k):
                    kind = "credential"
                elif re.search(r"(_DIR|_PATH|_FILE)$", k):
                    kind = "directory"
                elif re.search(r"(_HOST|_URL|_ENDPOINT|_ADDR)$", k):
                    kind = "host"
                # everything else (.env flags, ports, model names) is configuration, not an asset a request targets
                if kind:
                    put(k, kind, rel, i)
        elif low.endswith(".sql"):
            for i, ln in enumerate(text.splitlines(), 1):
                for tm in TABLE_RE.finditer(ln):
                    put(tm.group(1), "table", rel, i)

    return [a for a in assets.values()]


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", default="data/asset_inventory.json")
    ap.add_argument("--root", default=str(ROOT))
    args = ap.parse_args()

    assets = collect(Path(args.root).resolve())
    assets.sort(key=lambda a: (a["kind"], a["class"], a["name"].lower()))
    by_kind: dict[str, int] = {}
    by_class: dict[str, int] = {}
    for a in assets:
        by_kind[a["kind"]] = by_kind.get(a["kind"], 0) + 1
        by_class[a["class"]] = by_class.get(a["class"], 0) + 1
    inv = {
        "note": "authoritative asset list read from the deployment's own configs; a rule layer, not a model",
        "class_rule": "ephemeral iff the name carries " + ", ".join(EPHEMERAL_MARKERS) + "; otherwise prod",
        "counts": {"total": len(assets), "by_kind": by_kind, "by_class": by_class},
        "assets": assets,
    }
    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(inv, indent=2), encoding="utf-8")
    print(json.dumps(inv["counts"], indent=2))
    for a in [x for x in assets if x["kind"] == "service"][:8]:
        print(f"  [{a['class']}] {a['phrase']:<34} {a['source']}")
    print(f"\ninventory -> {out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
