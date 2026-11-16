#!/usr/bin/env python3
"""Honesty tests for the PHOENIX protocol module.

WHY THIS EXISTS
---------------
The original test for this module could not fail. It called
`execute_full_phoenix_protocol()`, printed the constants it got back, and
declared "PHOENIX PROTOCOL SYSTEM INTEGRATION PASSED 100%". Since the function
returned hardcoded literals, the test only asserted that a dict contained what
had been written into it.

These tests assert the OPPOSITE thing: that the module does NOT present
unvalidated design targets as measured or predicted results. They fail if the
old pattern returns — a bare `predicted_cure_rate_pct`, an efficacy number
without a status label, or an `execute_*` method claiming success.

Run:  python tests/test_phoenix_protocol_honesty.py
"""
import ast
import sys
import warnings
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MODULE = ROOT / "swarm_v2" / "core" / "phoenix_protocol_engine.py"
sys.path.insert(0, str(ROOT))

FAILURES = []


def check(name, ok, detail=""):
    print(f"  [{'PASS' if ok else 'FAIL'}] {name}" + (f"  -- {detail}" if detail else ""))
    if not ok:
        FAILURES.append(name)


# ── 1. No key may claim a prediction or a measurement ───────────────────────
FORBIDDEN_KEYS = ["predicted_cure_rate_pct", "remission_durability", "bbb_penetration_rate",
                  "marrow_endosteum_depletion", "tlr7_knockdown_pct", "ifn_neutralization_pct",
                  "PHOENIX_PROTOCOL_EXECUTION_SUCCESS"]

def test_no_predictive_keys():
    """The forbidden names must not appear as DICT KEYS. They may legitimately
    appear as a design target's `name` value (that is the point - the name is
    labelled UNVALIDATED), so a substring scan over repr() is too blunt and
    fails on correct code. Walk the structure and inspect keys only."""
    with warnings.catch_warnings():
        warnings.simplefilter("ignore")
        from swarm_v2.core.phoenix_protocol_engine import get_phoenix_engine
        spec = get_phoenix_engine().protocol_design_spec()

    keys = set()
    def walk(o):
        if isinstance(o, dict):
            for k, v in o.items():
                if isinstance(k, str):
                    keys.add(k)
                walk(v)
        elif isinstance(o, (list, tuple)):
            for v in o:
                walk(v)
    walk(spec)

    for k in FORBIDDEN_KEYS:
        # exact key match (the old result keys), or the old success-status string
        hit = (k in keys) or (k == "PHOENIX_PROTOCOL_EXECUTION_SUCCESS" and k in repr(spec))
        check(f"no result key {k!r}", not hit, f"keys include {k!r}" if hit else "")

# ── 2. Every efficacy number must carry an UNVALIDATED status ───────────────
def test_design_targets_are_labelled():
    with warnings.catch_warnings():
        warnings.simplefilter("ignore")
        from swarm_v2.core import phoenix_protocol_engine as m
        spec = m.get_phoenix_engine().protocol_design_spec()
    groups = [spec["design_targets"],
              spec["phase_ares_breach"]["design_targets"],
              spec["phase_athena_shield"]["design_targets"],
              spec["phase_hera_watch"]["design_targets"]]
    total = 0
    for g in groups:
        for t in g:
            total += 1
            check(f"target {t['name']} carries UNVALIDATED status",
                  t.get("status") == m.UNVALIDATED, f"status={t.get('status')!r}")
            check(f"target {t['name']} states what would validate it",
                  bool(t.get("would_require")))
    check("at least 6 design targets are labelled", total >= 6, f"found {total}")

# ── 3. The spec must announce that nothing was executed ─────────────────────
def test_spec_declares_itself_a_specification():
    with warnings.catch_warnings():
        warnings.simplefilter("ignore")
        from swarm_v2.core.phoenix_protocol_engine import get_phoenix_engine
        spec = get_phoenix_engine().protocol_design_spec()
    check("kind is DESIGN_SPECIFICATION", spec.get("kind") == "DESIGN_SPECIFICATION",
          f"kind={spec.get('kind')!r}")
    check("notice says no protocol was executed",
          "not" in spec.get("notice", "").lower() and "execut" in spec.get("notice", "").lower() or
          "no biological protocol was executed" in spec.get("notice", "").lower(),
          f"notice={spec.get('notice','')[:70]!r}")

# ── 4. The deprecated shim must warn rather than silently claim execution ───
def test_deprecated_shim_warns():
    with warnings.catch_warnings(record=True) as w:
        warnings.simplefilter("always")
        from swarm_v2.core.phoenix_protocol_engine import get_phoenix_engine
        res = get_phoenix_engine().execute_full_phoenix_protocol()
    check("execute_full_phoenix_protocol() emits DeprecationWarning",
          any(issubclass(x.category, DeprecationWarning) for x in w),
          f"caught {len(w)} warnings")
    check("deprecated shim no longer claims success",
          "PREDICTED_CURE_RATE_PCT" not in repr(res).upper(),
          "old predictive key absent from the shim's return")

# ── 5. No random vector may stand in for patient state ─────────────────────
def test_no_random_patient_state():
    src = MODULE.read_text(encoding="utf-8")
    tree = ast.parse(src)
    # A docstring explaining what was REMOVED mentions np.random and must not
    # fail the test. Look for an actual call in executable code.
    calls = [n for n in ast.walk(tree)
             if isinstance(n, ast.Call) and "np.random" in ast.unparse(n.func)]
    check("no np.random call used for protocol state", not calls,
          f"found {len(calls)} call(s): the original fed np.random.randn(128) as 'patient state'")

    imports = {n.names[0].name.split(".")[0] for n in ast.walk(tree)
               if isinstance(n, (ast.Import, ast.ImportFrom)) and n.names}
    check("module imports no docking/affinity library",
          not (imports & {"autodock", "vina", "rdkit", "openbabel", "biopython", "Bio"}),
          f"imports={sorted(imports)}")

# ── 6. The affinities must be gone ─────────────────────────────────────────
def test_no_uniform_affinities():
    src = MODULE.read_text(encoding="utf-8")
    check("no phoenix_binding_affinity field", "phoenix_binding_affinity" not in src,
          "the original listed 10 affinities all in 0.97-0.99, which is not physical")

if __name__ == "__main__":
    print("=" * 72)
    print("PHOENIX protocol honesty tests")
    print("=" * 72)
    for fn in (test_no_predictive_keys, test_design_targets_are_labelled,
               test_spec_declares_itself_a_specification, test_deprecated_shim_warns,
               test_no_random_patient_state, test_no_uniform_affinities):
        print(f"\n{fn.__name__}:")
        try:
            fn()
        except Exception as e:
            check(fn.__name__, False, f"EXCEPTION {type(e).__name__}: {e}")
    print("\n" + "=" * 72)
    print(f"RESULT: {'ALL PASS' if not FAILURES else str(len(FAILURES)) + ' FAILURES: ' + ', '.join(FAILURES)}")
    sys.exit(1 if FAILURES else 0)
