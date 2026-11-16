"""voice.py - a REAL speaking path for the platform.

Replaces the templated _intelligent_respond family in
agent_intelligence_bridge.py, which returns hardcoded verdicts for any
input ("Stable within normal physiological parameters" for a septic
patient, "No unmitigated vulnerabilities detected" for any CVE query).

This module speaks through the live inference stack and NEVER fabricates:
if no model is reachable it says so instead of inventing a verdict.
"""

import json
import re
import time
import urllib.error
import urllib.request

FAST_URL = "http://127.0.0.1:8587/completion"   # qwen2.5-1.5b
DEEP_URL = "http://127.0.0.1:8591/completion"   # Bonsai-27B-Q1_0 (may not be running)
LLM_URL = FAST_URL

# :8591 is NOT part of the standard nine-service launch. Measured 2026-11-04:
# a deep-word question (explain/why/design/prove...) tried :8591 first, waited
# 2.04s for the connection to fail, then fell back to :8587 and answered in
# 0.23s. That is 2s of dead time on every deep question, plus a provenance
# label naming a model that is not running. Probe once and cache the answer.
_DEEP_PROBE = {}


def _deep_available(timeout=1.0):
    """True only if the deep model is actually serving. Cached for the process."""
    if "ok" in _DEEP_PROBE:
        return _DEEP_PROBE["ok"]
    try:
        base = DEEP_URL.rsplit("/", 1)[0]
        with urllib.request.urlopen(base + "/health", timeout=timeout) as r:
            _DEEP_PROBE["ok"] = r.status == 200
    except Exception:
        _DEEP_PROBE["ok"] = False
    return _DEEP_PROBE["ok"]
DEEP_WORDS = ("prove", "derive", "analyze", "analyse", "design", "diagnose",
              "why", "compare", "trade-off", "architecture", "plan",
              "explain", "evaluate", "justify", "optimi")

SYSTEM = (
    "You are {agent}, one agent in a technical swarm. Answer factually and "
    "concisely. If you do not know, say so plainly. Never invent numeric "
    "results, IC50 values, CVE status, or clinical findings."
)


def _call_llm(prompt, n_predict=192, timeout=120, url=None):
    body = json.dumps({
        "prompt": prompt,
        "n_predict": n_predict,
        "temperature": 0.2,
        "stop": ["<|im_end|>"],
    }).encode("utf-8")
    req = urllib.request.Request(
        (url or LLM_URL), data=body, headers={"Content-Type": "application/json"}
    )
    with urllib.request.urlopen(req, timeout=timeout) as r:
        return json.loads(r.read()).get("content", "")


def speak(agent, message, memories=None, n_predict=192):
    """Speak through the live model. Returns a dict with provenance.

    If the model is unreachable, returns status="unavailable" and an empty
    answer. It will NOT fall back to a template verdict.
    """
    t0 = time.perf_counter()
    ctx = ""
    if memories:
        ctx = "Known context:\n" + "\n".join("- " + str(m) for m in memories) + "\n\n"
    prompt = (
        "<|im_start|>system\n" + SYSTEM.format(agent=agent) + "\n" + ctx + "<|im_end|>\n"
        "<|im_start|>user\n" + message + "<|im_end|>\n"
        "<|im_start|>assistant\n"
    )
    deep = (len(message) > 220 or any(w in message.lower() for w in DEEP_WORDS)) and _deep_available()
    url = DEEP_URL if deep else FAST_URL
    label = "llama.cpp-vulkan@8591-bonsai27b" if deep else "llama.cpp-vulkan@8587-qwen1.5b"
    try:
        # Bonsai-27B is a reasoning model: it emits a  thinking block BEFORE the
        # answer. At n_predict=192 the reply ended mid-thought and the caller
        # never saw an answer (measured 2026-11-05). Deep calls get a budget
        # that covers the reasoning plus the answer.
        budget = max(n_predict, 640) if deep else n_predict
        try:
            answer = _call_llm(prompt, n_predict=budget, url=url).strip()
        except Exception:
            if not deep:
                raise
            url = FAST_URL
            label = "llama.cpp-vulkan@8587-qwen1.5b"
            answer = _call_llm(prompt, n_predict=n_predict, url=url).strip()
    except Exception as exc:
        return {
            "agent": agent,
            "status": "unavailable",
            "answer": "",
            "reason": "no model reachable at " + LLM_URL + ": " + repr(exc),
            "latency_ms": round((time.perf_counter() - t0) * 1000, 2),
        }
    flags = risk_flags(answer)
    return {
        "agent": agent,
        "status": "ok",
        "answer": answer,
        "risk_flags": flags,
        "requires_verification": bool(flags),
        "engine": label,
        "memories_used": len(memories) if memories else 0,
        "latency_ms": round((time.perf_counter() - t0) * 1000, 2),
    }


if __name__ == "__main__":
    TESTS = [
        ("Pulse", "Patient has fever 41C, BP 60/30, lactate 6.2, urine output 5ml/hr. Sepsis?"),
        ("Bridge", "Predict binding affinity of aspirin to COX-1"),
        ("Shield", "Is CVE-2021-44228 exploitable on this host right now?"),
    ]
    for agent, msg in TESTS:
        r = speak(agent, msg)
        print("[" + agent + "] status=" + r["status"] + " " + str(r.get("latency_ms")) + "ms")
        print("  Q: " + msg)
        print("  A: " + (r["answer"] or r.get("reason", "")))
        print()


RISK = [
    ("measured_number", r"\b(ic50|ec50|kd|ki|dissociation constant)\b"),
    ("security_clear", r"\b(not|no longer)\b(\s+\w+){0,3}\s+exploitable"),
    ("security_clear", r"\bno\b(\s+\w+){0,2}\s+vulnerabilit"),
    ("security_clear", r"\b(fully|considered)\b(\s+\w+){0,2}\s+(patched|fixed)"),
    ("clinical_clear", r"\bstable\b(\s+\w+){0,3}\s+parameters"),
    ("clinical_clear", r"\bno\b(\s+\w+){0,2}\s+sepsis"),
]

def risk_flags(answer):
    """Flag prose asserting a measured result the model did not compute.

    Regex not substring - prose inserts words between needle parts,
    which defeats literal matching.
    """
    out = []
    for name, pat in RISK:
        if re.search(pat, answer, re.IGNORECASE) and name not in out:
            out.append(name)
    return out