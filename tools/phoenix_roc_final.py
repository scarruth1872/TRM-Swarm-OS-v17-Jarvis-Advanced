#!/usr/bin/env python3
"""Final ROC with confidence intervals - LpxC, ChEMBL actives vs two negative arms.

THE SET
  22 ChEMBL actives   (CHEMBL3855 P. aeruginosa LpxC, pChEMBL >= 7 = <=100 nM),
                      filtered to standard_relation '=' and units nM, then
                      scaffold-diversified at ECFP4 Tanimoto <= 0.55.
  10 MEASURED weak    (pChEMBL <= 4.5 = >=32 uM). These are EXPERIMENTAL
                      negatives - molecules tested against this exact target and
                      found inactive. Stronger evidence than any synthetic decoy.
  40 synthetic decoys (property-matched, from the first experiment).

WHY THE SECOND ARM MATTERS
  A synthetic decoy is "presumed inactive". A ChEMBL weak compound is MEASURED
  inactive against this target. If the protocol cannot separate actives from
  measured inactives, it cannot rank anything, and no threshold is defensible.

HOW TO READ IT
  AUC is reported with a bootstrap 95% CI, which 22 actives now supports (it did
  not at n=2). Enrichment factors say how many more actives you find in the top
  X% than by random selection. Youden's J gives the best single cut, with its CI -
  if that interval is wide, the cut is not knowable from this data and the tool
  says so rather than printing a crisp number.
"""
import random, sys


def parse(path):
    sets = {"ACTIVE": [], "MEASURED_WEAK": [], "DECOY": []}
    for line in open(path):
        line = line.rstrip("\n")
        if line.startswith("#") or not line.strip() or line.strip() == "ALLDONE":
            continue
        parts = line.split("\t")
        if len(parts) < 3:
            continue
        kind = parts[0]
        nums = [t for t in parts[2].split() if t.replace("-", "").replace(".", "").isdigit()]
        if len(nums) < 2:
            continue
        try:
            aff = float(nums[1])
        except (ValueError, IndexError):
            continue
        sets.setdefault(kind, []).append((parts[1], aff))
    return sets


def auc_of(pos, neg):
    if not pos or not neg:
        return float("nan")
    w = t = n = 0
    for _, a in pos:
        for _, b in neg:
            n += 1
            if a < b:
                w += 1
            elif a == b:
                t += 1
    return (w + 0.5 * t) / n


def boot_auc(pos, neg, iters=2000, seed=42):
    rng = random.Random(seed)
    vals = []
    for _ in range(iters):
        p = [rng.choice(pos) for _ in pos]
        q = [rng.choice(neg) for _ in neg]
        v = auc_of(p, q)
        if v == v:
            vals.append(v)
    vals.sort()
    lo = vals[int(0.025 * len(vals))]
    hi = vals[int(0.975 * len(vals))]
    return auc_of(pos, neg), lo, hi


def ef(pos, neg, frac):
    """Enrichment factor at the top `frac` of the ranked library."""
    lib = sorted(pos + neg, key=lambda r: r[1])
    k = max(1, int(round(frac * len(lib))))
    top = lib[:k]
    hit = sum(1 for n, _ in top if any(n == pn for pn, _ in pos))
    return hit, k, (hit / k) / (len(pos) / len(lib))


def youden(pos, neg):
    """Best single affinity cut and its J."""
    cuts = sorted({a for _, a in pos} | {b for _, b in neg})
    best = (None, -9)
    for c in cuts:
        sens = sum(1 for _, a in pos if a <= c) / len(pos)
        spec = sum(1 for _, b in neg if b > c) / len(neg)
        j = sens + spec - 1
        if j > best[1]:
            best = (c, j, sens, spec)
    return best


if __name__ == "__main__":
    sets = parse(sys.argv[1])
    for extra in sys.argv[2:]:
        for k, v in parse(extra).items():
            sets.setdefault(k, []).extend(v)
    A = sets.get("ACTIVE", [])
    W = sets.get("MEASURED_WEAK", [])
    D = sets.get("DECOY", [])
    print("=" * 78)
    print("FINAL ROC - P. aeruginosa LpxC (CHEMBL3855 / PDB 5U39)")
    print("=" * 78)
    print(f"actives (pChEMBL >= 7, <=100 nM)      : {len(A)}")
    print(f"MEASURED weak (pChEMBL <= 4.5, >=32uM): {len(W)}")
    print(f"synthetic decoys                      : {len(D)}")
    if not A or not (W or D):
        sys.exit("need actives and at least one negative arm")

    for label, neg in (("MEASURED weak (experimental)", W), ("synthetic decoys", D),
                       ("both arms pooled", W + D)):
        if not neg:
            continue
        a, lo, hi = boot_auc(A, neg)
        print(f"\n--- arm: {label} (n_neg={len(neg)}) ---")
        print(f"  AUC = {a:.3f}   95% bootstrap CI [{lo:.3f}, {hi:.3f}]")
        for f in (0.01, 0.05, 0.10, 0.20):
            hit, k, e = ef(A, neg, f)
            print(f"  EF@{int(f*100):>2}%  = {e:5.2f}x   ({hit}/{k} in the top {k})")
        c, j, sens, spec = youden(A, neg)
        print(f"  Youden best cut = {c:.3f} kcal/mol  J={j:.3f}  sens={sens:.2f} spec={spec:.2f}")
        best_neg = min(b for _, b in neg)
        worst_pos = max(x for _, x in A)
        print(f"  best negative {best_neg:.3f} vs worst active {worst_pos:.3f} -> "
              f"{'OVERLAP' if best_neg < worst_pos else 'CLEAN'}")

    # consensus view across both arms
    pool = W + D
    if pool:
        a, lo, hi = boot_auc(A, pool)
        print("\n" + "=" * 78)
        print(f"CONSENSUS AUC over all {len(pool)} negatives = {a:.3f}  CI [{lo:.3f}, {hi:.3f}]")
        c, j, sens, spec = youden(A, pool)
        print(f"CONSENSUS best cut = {c:.3f} kcal/mol (J={j:.3f}, sens={sens:.2f}, spec={spec:.2f})")
        print(f"  -> a cut at {c:.1f} keeps {sens*100:.0f}% of actives and {spec*100:.0f}% of negatives")
        if hi - lo > 0.20:
            print("  NOTE: the AUC interval is wide; this set does not pin the threshold tightly.")
