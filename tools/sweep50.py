import json, time, urllib.request, statistics, pathlib

B = "http://127.0.0.1:8588"
D = pathlib.Path(r"F:\Development sites\TRM-Swarm-OS-v2\kaggle\combined")
N_TASKS = 50


def predict(tid, pid=1, timeout=600):
    u = f"{B}/predict?task={tid}&pid={pid}&split=training2"
    return json.loads(urllib.request.urlopen(u, timeout=timeout).read())


def main():
    ch = json.loads((D / "arc-agi_training2_challenges.json").read_text(encoding="utf-8"))
    ids = sorted(ch.keys())[:N_TASKS]
    print(f"SWEEP: {len(ids)} tasks from arc-agi_training2, pid=1")
    print("-" * 78)

    rows = []
    fails = []
    t0 = time.perf_counter()
    for n, tid in enumerate(ids, 1):
        try:
            d = predict(tid)
        except Exception as e:
            fails.append((tid, str(e)[:40]))
            print(f"[{n:>2}/{len(ids)}] {tid}  ERROR {str(e)[:36]}")
            continue
        rows.append(d)
        print(f"[{n:>2}/{len(ids)}] {tid}  cells={d['cells']:<4} "
              f"cell={d['cell_accuracy']:.4f}  fg={d['foreground_accuracy']}  exact={d['exact']}")
    wall = time.perf_counter() - t0
    return rows, fails, wall


def summarize(rows, fails, wall):
    print("-" * 78)
    ok = [r for r in rows]
    if not ok:
        print("NO SUCCESSFUL TASKS"); return
    ca = [r['cell_accuracy'] for r in ok]
    fa = [r['foreground_accuracy'] for r in ok if r['foreground_accuracy'] is not None]
    ex = sum(1 for r in ok if r['exact'])
    print(f"TASKS OK      {len(ok)}/{len(ok)+len(fails)}   (errors {len(fails)})")
    print(f"EXACT SOLVES  {ex}/{len(ok)}  = {100*ex/len(ok):.1f}%")
    print(f"cell_accuracy mean {statistics.mean(ca):.4f}  median {statistics.median(ca):.4f}")
    if fa:
        print(f"fg_accuracy   mean {statistics.mean(fa):.4f}  median {statistics.median(fa):.4f}")
        print(f"              min  {min(fa):.4f}  max {max(fa):.4f}")
        zero = sum(1 for x in fa if x == 0.0)
        print(f"              tasks with fg_accuracy 0.0: {zero}/{len(fa)}")
    gap = statistics.mean(ca) - (statistics.mean(fa) if fa else 0)
    print(f"GAP cell - fg  {gap:+.4f}   <- the inflation cell_accuracy adds")
    print(f"wall {wall:.1f}s   {wall/max(1,len(ok)):.2f}s per task")
    out = pathlib.Path(r'F:\Development sites\TRM-Swarm-OS-v2\evidence\SWEEP50_RESULTS.json')
    slim = [{k: v for k, v in r.items() if k not in ('pred', 'solution')} for r in rows]
    out.write_text(json.dumps({'n': len(ok), 'errors': fails, 'rows': slim}, indent=1))
    print(f"saved {out}")


if __name__ == '__main__':
    r, f, w = main()
    summarize(r, f, w)
