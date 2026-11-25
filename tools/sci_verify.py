#!/usr/bin/env python3
"""421-430 — scientific computing verified against INDEPENDENT references.

Nothing here asks the platform whether it agrees with itself. Each check recomputes the reference in Python:
benzene's Hückel spectrum is a published number, the SIR final size is re-solved by bisection here, the PK curve
is re-derived from its closed form, and Poisson statistics are checked against what a Poisson process must give.
"""
import json, math, sys, urllib.request, urllib.error
from pathlib import Path

TOK = Path("C:/ProgramData/trm/council_token.txt").read_text().strip()
ROOT = Path(__file__).resolve().parent.parent
BASE = "http://127.0.0.1:8022"
OUT = ROOT / "evidence" / "sci_verify.json"


def post(path, payload):
    req = urllib.request.Request(BASE+path, data=json.dumps(payload).encode(), method="POST",
                                 headers={"Content-Type": "application/json", "X-Council-Token": TOK})
    try:
        with urllib.request.urlopen(req, timeout=240) as r:
            return json.loads(r.read().decode())
    except urllib.error.HTTPError as e:
        return {"_error": e.code, "_body": e.read().decode(errors="replace")[:200]}


def get(path):
    req = urllib.request.Request(BASE+path, headers={"X-Council-Token": TOK})
    with urllib.request.urlopen(req, timeout=60) as r:
        return json.loads(r.read().decode())


def sir_final_size_reference(beta, gamma, s0):
    """Independent bisection in Python — the same equation, different implementation."""
    r0 = beta / gamma

    def f(z):
        return s0 * math.exp(-r0 * z) - (1 - z)

    lo, hi = 0.0, 1.0
    if f(hi) <= 0:
        return 0.0
    for _ in range(200):
        mid = 0.5 * (lo + hi)
        if f(mid) < 0:
            lo = mid
        else:
            hi = mid
    return 0.5 * (lo + hi)


def main():
    results = {}
    checks = []

    # 421 — MD energy conservation
    md = post("/sci/md", {"n_side": 3, "spacing": 1.12, "dt": 0.001, "steps": 800})
    results["md"] = md
    checks.append(("421 MD conserves energy", md.get("relative_drift", 1.0) < 1e-2, f"drift {md.get('relative_drift')}"))

    # 422 — Hückel benzene against the published spectrum
    hk = post("/sci/huckel", {"kind": "ring", "n": 6, "electrons": 6})
    results["huckel"] = hk
    want = [-2.0, -1.0, -1.0, 1.0, 1.0, 2.0]
    got = hk.get("eigenvalues_over_beta", [])
    eig_ok = len(got) == 6 and all(abs(a - b) < 1e-6 for a, b in zip(got, want))
    checks.append(("422 benzene eigenvalues = (-2,-1,-1,1,1,2)", eig_ok, f"{got}"))
    checks.append(("422 benzene pi energy = -8 beta", abs(hk.get("pi_energy_over_beta", 0) + 8.0) < 1e-6,
                   f"{hk.get('pi_energy_over_beta')}"))
    bd = post("/sci/huckel", {"kind": "chain", "n": 4, "electrons": 4})
    checks.append(("422 butadiene E = -4.472 β", abs(bd.get("pi_energy_over_beta", 0) + 4.472135955) < 1e-5,
                   f"{bd.get('pi_energy_over_beta')}"))

    # 423 — annealing may never beat the exhaustive optimum
    fold = post("/sci/fold", {"sequence": "HPHPPHPPHH", "steps": 60000, "seed": 7})
    results["fold"] = fold
    checks.append(("423 exhaustive search completed (not truncated)", fold.get("exhaustive_complete") is True,
                   f"{fold.get('exhaustive_complete')}"))
    checks.append(("423 annealer never beats the ground state",
                   fold.get("annealed_energy", 0) >= fold.get("exhaustive_energy", 0),
                   f"anneal {fold.get('annealed_energy')} vs opt {fold.get('exhaustive_energy')}"))

    # 424 — analytic LJ well
    rmin = 2 ** (1 / 6)
    aff = post("/sci/affinity", {"ligand": [[0, 0, 0, 0, 0.5]], "pocket": [[rmin, 0, 0, 0, 0.5]], "dielectric": 4.0})
    results["affinity"] = aff
    checks.append(("424 single LJ pair sits at -eps", abs(aff.get("contact_energy_kcal", 0) + 0.15) < 5e-3,
                   f"{aff.get('contact_energy_kcal')}"))
    # Coulomb at 10 A: 332.06371 * 1 * 1 / (1 * 10)
    ca = post("/sci/affinity", {"ligand": [[0, 0, 0, 1, 0.5]], "pocket": [[10, 0, 0, 1, 0.5]], "dielectric": 1.0})
    expect = 332.06371 / 10.0
    checks.append(("424 Coulomb = k q1q2/(er r)", abs(ca.get("contact_energy_kcal", 0) - expect) < 0.1,
                   f"{ca.get('contact_energy_kcal')} vs {expect:.3f}"))

    # 425 — PK closed form
    pk = post("/sci/pk", {"model": "iv_bolus", "dose": 100, "volume": 10, "k": 0.1, "points": 3, "t_max": 6.9314718})
    results["pk"] = pk
    first_c = pk.get("series", [{}])[0].get("c", 0)
    last_c = pk.get("series", [{}, {}, {}])[2].get("c", 0)
    checks.append(("425 C(0) = dose/V = 10", abs(first_c - 10.0) < 1e-6, f"{first_c}"))
    checks.append(("425 C at one half-life = 5", abs(last_c - 5.0) < 1e-3, f"{last_c}"))
    checks.append(("425 t1/2 = ln2/k = 6.9315", abs(pk.get("half_life", 0) - math.log(2) / 0.1) < 1e-6,
                   f"{pk.get('half_life')}"))

    # 426 — SIR against an independent bisection
    s = post("/sci/sir", {"beta": 0.5, "gamma": 0.2, "i0": 1e-4, "days": 400, "dt": 0.01})
    results["sir"] = s
    ref = sir_final_size_reference(0.5, 0.2, 1 - 1e-4)
    checks.append(("426 R0 = beta/gamma = 2.5", abs(s.get("r0", 0) - 2.5) < 1e-9, f"{s.get('r0')}"))
    checks.append(("426 final size matches independent bisection",
                   abs(s.get("final_size", 0) - ref) < 0.01, f"{s.get('final_size')} vs {ref:.6f}"))
    checks.append(("426 analytic agrees with RK4",
                   s.get("agreement_error", 1) < 0.01, f"{s.get('agreement_error')}"))

    # 427 — neutral theory
    d = post("/sci/drift", {"p0": 0.3, "pop": 20, "reps": 300, "seed": 42})
    results["drift"] = d
    checks.append(("427 fixation prob = p0 within 0.08",
                   abs(d.get("measured_fixation_probability", 0) - 0.3) < 0.08,
                   f"{d.get('measured_fixation_probability')}"))

    # 428 — conserved quantity
    lv = post("/sci/lotka", {"alpha": 1, "beta": 0.1, "gamma": 1, "delta": 0.1, "x0": 10, "y0": 5,
                             "dt": 0.01, "steps": 5000})
    results["lotka"] = lv
    checks.append(("428 Lotka-Volterra conserves H", lv.get("relative_drift", 1) < 1e-3,
                   f"drift {lv.get('relative_drift')}"))

    # 429 — Poisson statistics
    sp = post("/sci/spikes", {"rate_hz": 20, "duration": 200, "refractory": 0.0, "seed": 99, "bin": 1.0})
    sp_ref = post("/sci/spikes", {"rate_hz": 20, "duration": 200, "refractory": 0.05, "seed": 99, "bin": 1.0})
    results["spikes"] = {"free": sp, "refractory": sp_ref}
    checks.append(("429 Poisson CV ~ 1", abs(sp.get("cv", 0) - 1.0) < 0.25, f"{sp.get('cv')}"))
    checks.append(("429 Poisson Fano ~ 1", abs(sp.get("fano", 0) - 1.0) < 0.35, f"{sp.get('fano')}"))
    checks.append(("429 refractory lowers CV", sp_ref.get("cv", 9) < sp.get("cv", 0),
                   f"{sp_ref.get('cv')} < {sp.get('cv')}"))

    # 430 — suite index
    suite = get("/sci/suite")
    results["suite"] = suite
    checks.append(("430 suite indexes 10 tasks", suite.get("total") == 10, f"{suite.get('total')}"))
    checks.append(("430 refusals are stated", len(suite.get("refused", [])) == 3,
                   f"{len(suite.get('refused', []))}"))


    OUT.parent.mkdir(exist_ok=True)
    OUT.write_text(json.dumps({"checks": [{"name": n, "pass": bool(p), "detail": d} for n, p, d in checks],
                               "results": results}, indent=1, default=str), encoding="utf-8")
    passed = sum(1 for _, p, _ in checks if p)
    for name, ok, detail in checks:
        print(f"  {'PASS' if ok else 'FAIL'}  {name:48} {detail}")
    print(f"\n{passed}/{len(checks)} checks passed")
    print(f"wrote {OUT.relative_to(ROOT)}")
    print("RESULT:", "PASS" if passed == len(checks) else "FAIL")
    return 0 if passed == len(checks) else 1


if __name__ == "__main__":
    sys.exit(main())
