#!/usr/bin/env python3
"""ROC / decoy-separation analysis for the LpxC docking set.

WHAT THIS MEASURES
  Whether the docking protocol ranks two CRYSTAL-CONFIRMED actives (CHIR-090 and
  NVS-LPXC-01, both co-crystallised with P. aeruginosa LpxC) above 40
  property-matched decoys. If it does not, then no threshold derived from docking
  scores means anything, and every BIND/REJECT verdict elsewhere is unfounded.

HOW TO READ IT, AND THE LIMIT THAT MATTERS
  n_actives = 2. A ROC curve with two positives is a coarse, noisy object and its
  AUC can swing wildly from one ligand to the next. So the rank-based statistics
  carry the weight here:
    * the rank of each active within the full 42-ligand ordering
    * the best score achieved by ANY decoy (the floor the actives must beat)
    * the exact Mann-Whitney U over all C(42,2) splits, which is valid at n=2
  AUC is reported for completeness, flagged as low-confidence.

  A decoy is a molecule with NO KNOWN activity against LpxC - "presumed
  inactive", not "proven inactive". That assumption underlies every enrichment
  study and it is an assumption.
"""
import itertools, sys

def parse(path):
    actives, decoys, box = [], [], None
    for line in open(path):
        line = line.rstrip("\n")
        if line.startswith("#"):
            box = line.lstrip("# ").strip()
            continue
        if line.strip() == "ALLDONE" or not line.strip():
            continue
        parts = line.split("\t")
        if len(parts) < 3:
            continue
        kind, name, rest = parts[0], parts[1], parts[2]
        nums = [t for t in rest.split() if t.replace("-", "").replace(".", "").isdigit()]
        if len(nums) < 2:
            continue
        try:
            aff = float(nums[1])   # field 1 is the mode number, field 2 the affinity
        except (ValueError, IndexError):
            continue
        (actives if kind == "ACTIVE" else decoys).append((name, aff))
    return actives, decoys, box


def auc(actives, decoys):
    """P(active scores better than decoy), ties counted as 0.5."""
    wins = ties = total = 0
    for _, a in actives:
        for _, d in decoys:
            total += 1
            if a < d:      # more negative = better binder
                wins += 1
            elif a == d:
                ties += 1
    return (wins + 0.5 * ties) / total if total else float("nan")


def mann_whitney_exact(actives, decoys):
    """Exact U and a two-sided p over all label assignments. Valid at n=2."""
    n1, n2 = len(actives), len(decoys)
    if n1 == 0 or n2 == 0:
        return None
    pool = [a for _, a in actives] + [d for _, d in decoys]
    def U(idx):
        s = 0
        for i in idx:
            for j in range(len(pool)):
                if j in idx: continue
                if pool[i] < pool[j]: s += 1
                elif pool[i] == pool[j]: s += 0.5
        return s
    obs = U(set(range(n1)))
    allU = [U(set(c)) for c in itertools.combinations(range(n1 + n2), n1)]
    extreme = sum(1 for u in allU if u >= obs)
    p = extreme / len(allU)
    return obs, max(U(set(c)) for c in itertools.combinations(range(n1 + n2), n1)), p, len(allU)


if __name__ == "__main__":
    path = sys.argv[1]
    actives, decoys, box = parse(path)
    print("=" * 78)
    print("ROC / DECOY SEPARATION - LpxC (P. aeruginosa)")
    print("=" * 78)
    if box: print(f"run: {box}")
    print(f"actives: {len(actives)}   decoys: {len(decoys)}")
    if not actives or not decoys:
        sys.exit("need both actives and decoys")

    allrows = sorted(actives + decoys, key=lambda r: r[1])   # best (most negative) first
    print(f"\nranked by predicted affinity (more negative = stronger predicted binding):")
    print(f"  {'rank':>4} {'label':<8} {'name':<16} {'affinity':>10}")
    active_ranks = []
    for i, (name, aff) in enumerate(allrows, 1):
        is_act = any(n == name for n, _ in actives)
        if is_act: active_ranks.append(i)
        print(f"  {i:>4} {'ACTIVE' if is_act else 'decoy':<8} {name:<16} {aff:>10.3f}")

    best_decoy = min(d for _, d in decoys)
    best_act = min(a for _, a in actives)
    worst_act = max(a for _, a in actives)
    print(f"\nbest active : {best_act:.3f}   worst active: {worst_act:.3f}")
    print(f"best decoy  : {best_decoy:.3f}")
    # Sign convention matters and my first version got it inverted. Affinities are
    # negative, so MORE NEGATIVE = BETTER binder. Overlap exists when the best
    # decoy is better than the weakest active, i.e. best_decoy < worst_act.
    overlap = best_decoy < worst_act
    n_decoy_above_worst = sum(1 for _, d in decoys if d < worst_act)
    n_decoy_above_best = sum(1 for _, d in decoys if d < best_act)
    print(f"\ndecoy floor vs ActIVES:")
    print(f"  decoys scoring better than the WEAKEST active: {n_decoy_above_worst}/{len(decoys)}")
    print(f"  decoys scoring better than the BEST active  : {n_decoy_above_best}/{len(decoys)}")
    if overlap:
        print(f"  -> OVERLAP. The best decoy ({best_decoy:.3f}) beats the weakest active "
              f"({worst_act:.3f}) by {worst_act - best_decoy:.3f} kcal/mol.")
        print(f"     A single score threshold CANNOT separate these actives from these decoys.")
    else:
        print(f"  -> CLEAN SEPARATION. Every decoy is worse than every active ({best_decoy:.3f} "
              f"vs {worst_act:.3f}).")
    a = auc(actives, decoys)
    print(f"\nAUC (actives over decoys) = {a:.3f}   [n_actives={len(actives)}: LOW CONFIDENCE]")
    mw = mann_whitney_exact(actives, decoys)
    if mw:
        obs, umax, p, nperm = mw
        print(f"Mann-Whitney U = {obs:.1f} (max possible {umax:.1f})  exact p = {p:.4f}  over {nperm} splits")
    print(f"active ranks: {active_ranks} of {len(allrows)}")

    # Does the threshold used for PHOENIX survive this measurement?
    for cut in (-8.5, -9.0, -10.0):
        passed = sum(1 for _, d in decoys if d <= cut)
        act = sum(1 for _, a in actives if a <= cut)
        verdict = "VALID" if passed == 0 else f"FALSE POSITIVES: {passed} decoy(s)"
        print(f"  threshold {cut:>6.1f} -> actives kept {act}/{len(actives)}, {verdict}")
    print(f"\nhonest reading: with {len(actives)} actives the AUC is not a stable estimate.")
    print("The rank of each active and the decoy floor are the numbers to trust.")
