import json, re, time, pathlib, urllib.request

R = pathlib.Path(r"F:\Development sites\TRM-Swarm-OS-v2")
SRC = R / "kaggle" / "combined" / "arc-agi_training2_challenges.json"
SOL = R / "kaggle" / "combined" / "arc-agi_training2_solutions.json"
URL = "http://127.0.0.1:8591/completion"
TASKS = ["007bbfb7", "00d62c1b", "025d127b"]


def grid_str(g):
    return "\n".join(" ".join(str(c) for c in row) for row in g)


def build_prompt(pairs, test_in):
    parts = [
        "You are solving an ARC grid puzzle. Grids use digits 0-9 for colours.",
        "Study the training examples, find the transformation, then apply it.",
        "",
    ]
    for i, p in enumerate(pairs, 1):
        parts.append(f"EXAMPLE {i} INPUT ({len(p['input'])}x{len(p['input'][0])}):")
        parts.append(grid_str(p["input"]))
        parts.append(f"EXAMPLE {i} OUTPUT ({len(p['output'])}x{len(p['output'][0])}):")
        parts.append(grid_str(p["output"]))
        parts.append("")
    parts.append(f"TEST INPUT ({len(test_in)}x{len(test_in[0])}):")
    parts.append(grid_str(test_in))
    parts.append("")
    parts.append("Reply with ONLY the output grid: rows of space-separated digits.")
    parts.append("No explanation, no code fences, no <think>. Just the grid.")
    return "\n".join(parts)


def ask(prompt, n_predict=2400):
    body = json.dumps({"prompt": prompt, "n_predict": n_predict,
                       "temperature": 0.0, "stop": ["<|im_end|>"]}).encode()
    rq = urllib.request.Request(URL, data=body,
                                headers={"Content-Type": "application/json"})
    t0 = time.perf_counter()
    d = json.loads(urllib.request.urlopen(rq, timeout=600).read())
    return time.perf_counter() - t0, d.get("content", "")


def parse_grid(txt):
    rows = []
    for line in txt.splitlines():
        s = line.strip().strip("[](),")
        if not s:
            continue
        toks = re.findall(r"\d", s)
        if len(toks) >= 2:
            rows.append([int(x) for x in toks])
    return rows


def score(pred, truth):
    if not pred:
        return 0, 0.0, False, 'no grid parsed'
    if len(pred) != len(truth) or len(pred[0]) != len(truth[0]):
        cells = sum(len(r) for r in truth)
        return cells, 0.0, False, f'shape {len(pred)}x{len(pred[0])} != {len(truth)}x{len(truth[0])}'
    cells = sum(len(r) for r in truth)
    ok = sum(1 for i, row in enumerate(truth)
             for j, v in enumerate(row) if pred[i][j] == v)
    acc = ok / cells if cells else 0.0
    return cells, acc, ok == cells, f'{ok}/{cells}'


def run():
    ch = json.loads(SRC.read_text(encoding='utf-8'))
    sol = json.loads(SOL.read_text(encoding='utf-8')) if SOL.exists() else {}
    print('Bonsai-27B-Q1_0 on ARC (temp 0.0, :8591)')
    print('-' * 64)
    results = []
    for tid in TASKS:
        if tid not in ch:
            print(f'{tid:<10} NOT IN DATASET'); continue
        t = ch[tid]
        truth = sol[tid][0] if tid in sol else t['test'][0].get('output')
        prompt = build_prompt(t['train'], t['test'][0]['input'])
        dt, txt = ask(prompt)
        pred = parse_grid(txt)
        cells, acc, exact, note = score(pred, truth)
        results.append((tid, dt, cells, acc, exact))
        print(f'{tid:<10} {dt:>6.1f}s  cells={cells:<5} acc={acc:.4f}  exact={exact}  ({note})')
        print(f'           raw[:70]={txt.strip()[:70]!r}')

    print('-' * 64)
    if results:
        ex = sum(1 for r in results if r[4])
        mean = sum(r[3] for r in results) / len(results)
        print(f'EXACT {ex}/{len(results)}   mean cell acc {mean:.4f}')


if __name__ == '__main__':
    run()
