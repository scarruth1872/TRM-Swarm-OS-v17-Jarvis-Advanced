import sys, json, pathlib
sys.path.insert(0, r"F:\Development sites\TRM-Swarm-OS-v2")
import numpy as np
from http.server import BaseHTTPRequestHandler, HTTPServer
D = pathlib.Path(r"F:\Development sites\TRM agent swarm\kaggle\combined")
G = 30
SEQ = G * G
PAD = 0
EOS = 1
_BR = None
_CH = {}
_SO = {}

def enc(g):
    a = np.array(g, dtype=np.uint8)
    r, c = a.shape
    b = np.zeros((G, G), dtype=np.uint8)
    b[:r, :c] = a + 2
    b[r, :c] = EOS
    b[:r, c] = EOS
    return b.flatten().tolist()

def dec(t, r, c):
    a = np.array(t[:SEQ], dtype=np.int64).reshape(G, G)
    return np.clip(a[:r, :c] - 2, 0, 9).tolist()

def load():
    global _BR
    if _BR is None:
        from swarm_v2.core.trm_brain import TRMBrain
        _BR = TRMBrain()
        _BR._ensure_loaded()
    return _BR

_PC = {}

def predict(tid, pid, split):
    _k = (tid, pid, split)
    if _k in _PC:
        return _PC[_k]
    if split not in _CH:
        c = D / ("arc-agi_%s_challenges.json" % split)
        s = D / ("arc-agi_%s_solutions.json" % split)
        _CH[split] = json.loads(c.read_text(encoding="utf-8"))
        _SO[split] = json.loads(s.read_text(encoding="utf-8"))
    t = _CH[split][tid]
    b = load()
    # TEST pair, not train[0]: train[0] is seen during training (memorisation)
    tin = t["test"][0]["input"]
    sol = _SO[split][tid][0]
    oh = len(sol)
    ow = len(sol[0])
    toks = enc(tin) + [PAD] * (1024 - SEQ)
    pr = b.reason(toks, puzzle_id=pid, full_output=True, output_len=1024, h_cycles=1)
    pred = dec(pr[-1], oh, ow)
    n = oh * ow
    hit = sum(1 for i in range(oh) for j in range(ow) if pred[i][j] == sol[i][j])
    # foreground accuracy: only cells where the truth is non-background.
    # cell_accuracy is dominated by empty cells and overstates capability.
    _fg = [(i, j) for i in range(oh) for j in range(ow) if sol[i][j] != 0]
    _r = {"task": tid, "puzzle_id": pid, "cells": n, "correct": hit,
            "cell_accuracy": round(hit / n, 4), "exact": pred == sol,
            "pred": pred, "solution": sol}
    _fgh = sum(1 for i, j in _fg if pred[i][j] == sol[i][j])
    _r["foreground_cells"] = len(_fg)
    _r["foreground_correct"] = _fgh
    _r["foreground_accuracy"] = round(_fgh / len(_fg), 4) if _fg else None
    _PC[_k] = _r
    return _r

class H(BaseHTTPRequestHandler):
    def log_message(self, *a):
        pass
    def _send(self, code, body):
        self.send_response(code)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)
    def do_GET(self):
        try:
            q = {}
            if "?" in self.path:
                for kv in self.path.split("?", 1)[1].split("&"):
                    k, _, v = kv.partition("=")
                    q[k] = v
            if self.path.startswith("/health"):
                self._send(200, json.dumps({"status": "ok", "model": "trm7m"}).encode())
            elif self.path.startswith("/predict"):
                r = predict(q.get("task", "007bbfb7"), int(q.get("pid", 1)), q.get("split", "training"))
                self._send(200, json.dumps(r).encode())
            else:
                self._send(404, json.dumps({"error": "unknown"}).encode())
        except Exception as e:
            self._send(500, json.dumps({"error": str(e)[:200]}).encode())

if __name__ == "__main__":
    load()
    print("TRM server on 127.0.0.1:8588", flush=True)
    HTTPServer(("127.0.0.1", 8588), H).serve_forever()
