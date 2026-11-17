"""tptp_serialize.py - MPTP2078 TPTP problems -> one record per problem file.

TPTP = "Thousands of Problems for Theorem Provers": the standard benchmark library of formal logic problems in the
TPTP language, used to rank automated theorem provers (CASC). Its sibling library is TSTP, "Thousands of Solutions
from Theorem Provers". The official distribution tarballs (TPTP-v9.x.tgz) are unreachable at every historical path,
so this uses **MPTPTP2078** - the Mizar-derived subset published by the TPTP project itself
(https://github.com/TPTPWorld/MPTPTP2078, 924,580,049 B tarball):

  Bushy/Problems   the 2,078 MPTP2078 problems with all axioms inlined
  Chainy/Problems  the same problems with axioms chained (sorted) for provers that need chains
  Pruney/Problems  variants pruned to the axioms a solution actually used
  */Solutions      recorded proofs from E 2.4 and Vampire 4.4 (60 s CPU, StarExec Miami) - separate from problems

The shard is labelled MPTP2078, NOT "the full TPTP library" (which is ~25k problems and still unreachable).

TPTP syntax (measured): `fof(name, role, formula).` / `cnf(...)`, roles axiom | conjecture | negated_conjecture |
hypothesis | definition | lemma | theorem. A file is a sequence of annotated formulae; the conjecture is what must
be proved. Axiom text is truncated per axiom with the true count retained, so truncation cannot read as completeness.
"""
import os
import re

MAX_AXIOMS_SHOWN = 6
MAX_FORMULA_CHARS = 600

_FORMULA_RE = re.compile(r"^(fof|cnf|tff|thf)\s*\(\s*([^,]+?)\s*,\s*([a-z_]+)\s*,", re.IGNORECASE)


def parse_problem(path):
    """Return (name, encoding, n_axioms, n_conjectures, list_of_(role, name, formula_snippet))."""
    name = os.path.basename(path)
    if name.endswith(".p"):
        name = name[:-2]
    encoding = os.path.basename(os.path.dirname(os.path.dirname(path)))
    text = open(path, encoding="utf-8", errors="replace").read()

    entries = []
    for line in text.splitlines():
        m = _FORMULA_RE.match(line.strip())
        if m:
            entries.append((m.group(3).lower(), m.group(2), line.strip()))
    if not entries:
        # multi-line formulae: fall back to a coarser scan that keeps the role
        for m in re.finditer(r"\b(fof|cnf|tff|thf)\s*\(\s*([^,]+?)\s*,\s*([a-z_]+)\s*,", text, re.IGNORECASE):
            entries.append((m.group(3).lower(), m.group(2), m.group(0)))
    if not entries:
        raise SystemExit(f"FAIL-CLOSED: no TPTP formula found in {path}")

    axioms = [e for e in entries if e[0] in ("axiom", "hypothesis", "definition", "lemma", "theorem")]
    conjs = [e for e in entries if "conjecture" in e[0]]
    return name, encoding, len(axioms), len(conjs), entries


def render(name, encoding, n_axioms, n_conjs, entries):
    parts = [f"TPTP MPTP2078 problem {name}", f"encoding {encoding}",
             f"axioms {n_axioms}", f"conjectures {n_conjs}"]
    conjs = [e for e in entries if "conjecture" in e[0]]
    if conjs:
        role, cname, formula = conjs[0]
        parts.append(f"conjecture_name {cname.strip()}")
        parts.append(f"conjecture {formula[:MAX_FORMULA_CHARS]}")
    shown = 0
    for role, aname, formula in entries:
        if role in ("axiom", "hypothesis", "definition", "lemma", "theorem"):
            if shown >= MAX_AXIOMS_SHOWN:
                break
            parts.append(f"axiom {aname.strip()} {formula[:MAX_FORMULA_CHARS]}")
            shown += 1
    if n_axioms > shown:
        parts.append(f"axioms_omitted {n_axioms - shown}")
    if not conjs:
        parts.append("no_conjecture true")
    return f"tptp/{encoding}/{name}", " ".join(parts)


def load_problems(root):
    """(key, text) for every problem file. Two layouts exist in the package and both are measured, not assumed:
         Bushy/Problems, Chainy/Problems          (shared layout)
         Pruney/Problems-Bushy, Pruney/Problems-Chainy   (Pruney prunes per encoding)
    """
    sets = [("Bushy", os.path.join(root, "Bushy", "Problems")),
            ("Chainy", os.path.join(root, "Chainy", "Problems")),
            ("Pruney-Bushy", os.path.join(root, "Pruney", "Problems-Bushy")),
            ("Pruney-Chainy", os.path.join(root, "Pruney", "Problems-Chainy"))]
    out = []
    skipped = 0
    for enc, pdir in sets:
        if not os.path.isdir(pdir):
            raise SystemExit(f"FAIL-CLOSED: missing problem set {pdir}")
        files = sorted(f for f in os.listdir(pdir) if f.endswith(".p"))
        if not files:
            raise SystemExit(f"FAIL-CLOSED: {pdir} has no .p files")
        for f in files:
            try:
                name, _e, na, nc, entries = parse_problem(os.path.join(pdir, f))
            except SystemExit:
                skipped += 1
                continue
            out.append(render(name, enc, na, nc, entries))
        print(f"  tptp: {enc:14} {len(files):5} problems", flush=True)
    if skipped:
        print(f"  tptp: skipped {skipped} unparseable files", flush=True)
    return out
