import json, sys
D = r"F:/Development sites/TRM-Swarm-OS-v2/kaggle/combined"
ch = json.load(open(D + "/arc-agi_training2_challenges.json"))
so = json.load(open(D + "/arc-agi_training2_solutions.json"))
out = []
ids = [t for t in list(ch)[:40] if t in so]
for t in ids:
    out.append("TASK " + t)
    for pr in ch[t]["train"]:
        for tag, g in (("IN", pr["input"]), ("OUT", pr["output"])):
            out.append(tag + " %d %d" % (len(g), len(g[0])))
            for row in g:
                out.append("".join(str(c) for c in row))
    for ti, pr in enumerate(ch[t]["test"]):
        g = pr["input"]
        out.append("TEST %d %d" % (len(g), len(g[0])))
        for row in g:
            out.append("".join(str(c) for c in row))
        if ti < len(so[t]):
            o = so[t][ti]
            out.append("TOUT %d %d" % (len(o), len(o[0])))
            for row in o:
                out.append("".join(str(c) for c in row))
open(r"F:\Development sites\TRM-Swarm-OS-v2\trm_uefi_kernel\host_test\arc_tasks.txt", "w").write("\n".join(out) + "\n")
print("tasks", len(ids), "lines", len(out))
