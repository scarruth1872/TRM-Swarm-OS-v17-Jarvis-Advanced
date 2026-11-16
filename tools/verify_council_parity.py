"""Council parity verification — run this yourself.

Why this is a separate file: the agent's shell guard blocked the inline version
of this command (the SQL-shaped attack strings trip the destructive-command
detector), so per the no-circumvention rule the command is handed to you rather
than run autonomously.

Usage:
    C:\\Users\\shawn\\AppData\\Local\\Programs\\Python\\Python311\\python.exe ^
        tools\\verify_council_parity.py

What it checks:
  1. PARITY - for N topics, the Rust council (:8022) and the Python council
     API (:8099) must return identical (approve, abstain, reject) triples.
     Before the fix this was FALSE: QUIC gave 5/4/5 on Rust vs 4/4/6 on Python.
  2. AUTHORITY - every :8099 vote must report council_authority="rust:8022".
     If it says "python-fallback", :8022 was unreachable and the verdict is
     NOT authoritative.
  3. WAF - real injection strings must still be blocked on :8099 after the
     DECLARE\\b false-positive fix. Expect every line to read BLOCKED.
"""

import json
import urllib.error
import urllib.request

RUST = 8022
PY = 8099

TOPICS = [
    "adopt QUIC RFC 9000 for the federation transport",
    "replace all silicon with cheese and declare victory",
    "reduce thermal load on the RX 6700 XT under sustained Vulkan load",
    "delete the step_155718 teacher checkpoint to free disk",
    "add a dark mode toggle to the dashboard",
    "prove that the sum of two even numbers is even",
    "memory usage keeps climbing find the leak",
]

ATTACKS = [
    "' UNION SELECT password FROM users--",
    "1; DROP TABLE users;--",
    "EXEC(@sql)",
    "1; DECLARE @sql varchar(255)",
    "1; SHUTDOWN",
]


def post(port, path, body=None):
    req = urllib.request.Request(
        "http://127.0.0.1:%d%s" % (port, path),
        data=json.dumps(body or {}).encode(),
        headers={"Content-Type": "application/json"},
    )
    try:
        return json.loads(urllib.request.urlopen(req, timeout=40).read())
    except urllib.error.HTTPError as exc:
        return {"_err": exc.code, "_body": exc.read().decode("utf-8", "replace")[:90]}


def main():
    print("%-52s %-12s %-12s %-4s %s" % ("topic", "RUST", "PY", "same", "authority"))
    agree = 0
    seen = 0
    for topic in TOPICS:
        rust = post(RUST, "/council/propose",
                    {"topic": topic, "proposer": "Archi", "category": "Architecture"})
        py = post(PY, "/council/propose",
                  {"topic": topic, "proposer": "Archi", "category": "Architecture"})
        if "_err" in py:
            print("%-52s PY PROPOSE ERROR %s %s" % (topic[:52], py["_err"], py["_body"]))
            continue
        vote = post(PY, "/council/vote/" + py["proposal_id"])
        if "_err" in vote:
            print("%-52s PY VOTE ERROR %s" % (topic[:52], vote["_err"]))
            continue
        tally = vote["raw_tally"]
        rust_t = (rust.get("approve"), rust.get("abstain"), rust.get("reject"))
        py_t = (tally["approve"], tally["abstain"], tally["reject"])
        seen += 1
        same = rust_t == py_t
        agree += same
        print("%-52s %-12s %-12s %-4s %s" % (
            topic[:52], rust_t, py_t, "Y" if same else "N", vote.get("council_authority")))

    print("\nPARITY: %d/%d topics identical" % (agree, seen))
    print("AUTHORITY: pass only if every row above says rust:8022")

    print("\n--- WAF regression (every line must read BLOCKED) ---")
    for attack in ATTACKS:
        res = post(PY, "/council/propose",
                   {"topic": attack, "proposer": "x", "category": "general"})
        status = ("BLOCKED %s" % res["_err"]) if "_err" in res else "ALLOWED (REGRESSION)"
        print("%-40s -> %s" % (attack[:40], status))


if __name__ == "__main__":
    main()
