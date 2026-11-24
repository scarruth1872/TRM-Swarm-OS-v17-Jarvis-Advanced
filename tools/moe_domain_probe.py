"""Domain-expertise probe for the two resident MoE models.

Two things are measured separately, because they are different claims:
  SELF   - what the model says it knows (self-report; cheap, unreliable, recorded verbatim)
  SCORE  - whether it answers a question with a checkable key (measured against expected tokens)

Questions are half general-domain (should be in the weights) and half THIS platform's internals
(not in any weights, so a correct answer would mean it guessed). The contrast is the finding.
"""
import json, time, urllib.request

MODELS = {"gpt-oss-20b (32exp/top4, :8587)": 8587, "instella-moe (64exp/top6, :8596)": 8596}

PROBES = [
    ("security", "Does disabling TLS certificate verification expose traffic to machine-in-the-middle "
                 "attacks even when you trust the server? One sentence.",
     ["yes", "man-in-the-middle", "mitm", "intercept"]),
    ("build", "In Rust, what does the `?` operator do inside a function that returns Result? One sentence.",
     ["propagat", "early return", "returns the error", "return the err"]),
    ("ledger", "What makes an append-only hash chain tamper-evident? One sentence.",
     ["previous hash", "prior hash", "each block", "chain of hash"]),
    ("inference", "For a memory-bandwidth-bound LLM decode, what happens to per-sequence tokens/sec as "
                  "batch size grows? One sentence.",
     ["decreas", "lower", "drop", "slower"]),
    ("network", "What TCP port does HTTPS use by default? Answer with the number.",
     ["443"]),
    ("retrieval", "In cosine similarity, does the magnitude of the vectors change the result? One sentence.",
     ["no", "not", "magnitude is ignored", "normaliz", "scale-invariant"]),
    ("platform", "Which TCP port does the TRM council server listen on? Answer with the number and say "
                 "whether you are certain.", ["8022"]),
    ("platform", "In the TRM council's axiom set, what is axiom A2 about? One sentence.", ["free will", "consent", "A2"]),
]

SELF = ("Name the two technical domains you are most reliable in and the two you are weakest in, and give "
        "one sentence of evidence for each. No preamble.")


def ask(port, prompt, max_tokens=260, timeout=240):
    body = json.dumps({"messages": [{"role": "user", "content": prompt}],
                       "max_tokens": max_tokens, "temperature": 0.0,
                       "chat_template_kwargs": {"reasoning_effort": "low", "enable_thinking": False}}).encode()
    r = urllib.request.Request(f"http://127.0.0.1:{port}/v1/chat/completions", data=body,
                               headers={"Content-Type": "application/json"})
    t0 = time.time()
    d = json.loads(urllib.request.urlopen(r, timeout=timeout).read())
    return (d["choices"][0]["message"].get("content") or "").strip(), time.time() - t0


out = {}
for name, port in MODELS.items():
    print(f"\n=== {name} ===")
    row = {"self": None, "items": []}
    try:
        txt, dt = ask(port, SELF, 220)
        row["self"] = txt
        print(f"  SELF-REPORT ({dt:.1f}s):\n    " + txt.replace("\n", "\n    ")[:700])
    except Exception as e:
        print("  self-report failed:", str(e)[:120])
    hit = 0
    for dom, q, keys in PROBES:
        try:
            a, dt = ask(port, q)
            low = a.lower()
            ok = any(k in low for k in keys)
            hit += ok
            row["items"].append({"domain": dom, "q": q, "answer": a[:400], "ok": ok, "s": round(dt, 1)})
            print(f"  {'PASS' if ok else 'miss'}  {dom:10} {dt:5.1f}s  {a[:110].replace(chr(10),' ')}")
        except Exception as e:
            row["items"].append({"domain": dom, "q": q, "error": str(e)[:100], "ok": False})
            print(f"  ERR   {dom:10} {str(e)[:90]}")
    row["score"] = f"{hit}/{len(PROBES)}"
    print(f"  --> {hit}/{len(PROBES)} checkable answers")
    out[name] = row

json.dump(out, open(r"C:/Users/shawn/AppData/Local/hermes/cache/scratch/moe_domain_probe.json", "w"), indent=1)
print("\n  general-domain vs platform questions:",
      sum(1 for n, r in out.items() for i in r["items"] if i.get("domain") != "platform" and i.get("ok")),
      "passes on general;",
      sum(1 for n, r in out.items() for i in r["items"] if i.get("domain") == "platform" and i.get("ok")),
      "passes on platform internals")
