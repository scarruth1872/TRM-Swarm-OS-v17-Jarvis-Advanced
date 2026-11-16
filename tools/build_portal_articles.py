"""Rebuild the public portal's article list from the journal audit manifest.

The portal's `index.html` carries its articles inline as

    const articles = [{"id": 189, ...}, {"id": 188, ...}, ...];

nothing generated it, so it drifted: 88 articles ending at id 189 while the manifest held 143 up to id 244 (G153).
Hand-editing a JSON array inside a 79 KB HTML file is how that drift happened, so this tool makes the manifest the
source of truth and the HTML the derived artifact.

Safety rules, because this rewrites a published page:

  * `--check` (default) mutates nothing and exits 1 when the page is stale - safe to run in CI.
  * The portal's own array is parsed first. If it holds an id the manifest lacks, the rebuild would DELETE that
    article, so the tool refuses and names the ids instead of trusting that the manifest is complete.
  * Every write is preceded by a timestamped backup of `index.html`.
  * Only the `const articles = [...];` span is replaced; the rest of the page is byte-identical.

    python tools/build_portal_articles.py            # report drift, exit 1 if stale
    python tools/build_portal_articles.py --write    # back up, then rebuild the array
"""
from __future__ import annotations

import argparse
import json
import re
import shutil
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "phoenix_journal_audit_manifest.json"
PORTAL = ROOT / "trm-sovereign-portal" / "index.html"
ARRAY_RE = re.compile(r"const articles = (\[.*?\]);", re.S)
FIELDS = ("id", "section", "title", "finding")


def load_manifest(path: Path) -> list[dict]:
    data = json.loads(path.read_text(encoding="utf-8"))
    arts = []
    for a in data["articles"]:
        missing = [f for f in FIELDS if f not in a]
        if missing:
            raise SystemExit(f"manifest article {a.get('id')} lacks {missing}; refusing to publish a partial entry")
        arts.append({f: a[f] for f in FIELDS})
    ids = [a["id"] for a in arts]
    dup = {i for i in ids if ids.count(i) > 1}
    if dup:
        raise SystemExit(f"manifest repeats ids {sorted(dup)}; a duplicate id would shadow an article on the page")
    return sorted(arts, key=lambda a: -a["id"])


def portal_array(html: str) -> list[dict]:
    m = ARRAY_RE.search(html)
    if not m:
        raise SystemExit("could not find `const articles = [...];` in the portal page - layout changed, refusing")
    return json.loads(m.group(1))


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--manifest", default=str(MANIFEST))
    ap.add_argument("--portal", default=str(PORTAL))
    ap.add_argument("--write", action="store_true", help="actually rewrite the page (default: report only)")
    args = ap.parse_args()

    man_path, portal_path = Path(args.manifest), Path(args.portal)
    if not portal_path.exists():
        raise SystemExit(f"portal page not found: {portal_path}")
    arts = load_manifest(man_path)
    html = portal_path.read_text(encoding="utf-8")
    current = portal_array(html)

    man_ids = {a["id"] for a in arts}
    portal_ids = {a["id"] for a in current}
    orphaned = sorted(portal_ids - man_ids)
    print(f"manifest: {len(man_ids)} articles (id {min(man_ids)}-{max(man_ids)})")
    print(f"portal:   {len(portal_ids)} articles (id {min(portal_ids)}-{max(portal_ids)})")
    missing = sorted(man_ids - portal_ids)
    if missing:
        print(f"missing from portal: {len(missing)} -> {missing[:8]}{' ...' if len(missing) > 8 else ''}")
    if orphaned:
        print(f"REFUSED: portal has {len(orphaned)} article(s) the manifest does not: {orphaned[:8]}")
        print("         rebuilding would delete them; add them to the manifest or pass nothing - no override exists.")
        return 2

    new_json = json.dumps(arts, ensure_ascii=True)
    new_html = ARRAY_RE.sub(lambda _: "const articles = " + new_json + ";", html, count=1)
    if new_html == html:
        print("portal is current: no change needed")
        return 0

    changed = sum(1 for a in current if a not in arts)
    print(f"portal is STALE by {len(man_ids - portal_ids)} article(s); {changed} existing entries differ in text")
    if not args.write:
        print("run with --write to rebuild (a timestamped backup is taken first)")
        return 1

    backup = portal_path.with_suffix(f".html.bak-{time.strftime('%Y%m%d-%H%M%S')}")
    shutil.copy2(portal_path, backup)
    portal_path.write_text(new_html, encoding="utf-8")

    # verify by re-parsing what actually landed on disk, not by trusting the string we just built
    written = portal_array(portal_path.read_text(encoding="utf-8"))
    if [a["id"] for a in written] != [a["id"] for a in arts]:
        raise SystemExit("verification failed: written ids do not match the manifest - restore the backup")
    print(f"wrote {len(written)} articles (id {min(a['id'] for a in written)}-{max(a['id'] for a in written)})")
    print(f"backup: {backup.name}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
