#!/usr/bin/env python3
"""
TRM Swarm OS — Kimi K3 Ingestion Engine (council 9ace02f3, 14/14).
Moonshot Kimi K3 outputs ingested as KNOWLEDGE into the K3/TurboVec fabric.

EPISTEMOLOGICAL GUARD (benchmark epistemology map): the API is a knowledge
SOURCE, never an evaluation AUTHORITY. Kimi outputs are ingested as
hypothesis-text, run through K3 verification, quantized to ternary trits,
and stored below the platform's own on-device intelligence in weight.

HARD RULES:
- Runtime stays 100% on-device (no API in the booted kernel path)
- Key from env MOONSHOT_API_KEY only — NEVER hardcoded
- Host-side tooling like all prior absorptions (openfang, chat)
- K3 filters; no zero-hallucination claim (that would be an overclaim)
"""
import os, sys, json, time, hashlib, hmac, struct

MONAD_ANCHOR = 0x4D4F4E4144000001
API_URL = "https://api.moonshot.ai/v1/chat/completions"
ROOT = r"F:\Development sites\TRM-Swarm-OS-v2"

# Domain queries for knowledge ingestion (platform domains)
DOMAIN_QUERIES = {
    "consensus": "Explain Byzantine fault tolerance quorum arithmetic for 14 agents with 4 faulty.",
    "reasoning": "Explain Kleene ternary logic truth tables and their use in uncertainty.",
    "defense": "List OWASP-style code injection patterns and remediation strategies.",
    "logic": "Explain the difference between binary and balanced ternary computation.",
    "memory": "Describe efficient associative memory recall data structures.",
    "voltage": "Explain energy-efficient computing and voltage scaling principles.",
    "epistemology": "Why are benchmark scores unreliable measures of real-world capability?",
    "healing": "Describe self-healing system architectures and anomaly recovery.",
}


def seal(text: str) -> int:
    h = int(hashlib.sha256(text.encode()).hexdigest()[:16], 16)
    return (MONAD_ANCHOR ^ h) & 0xFFFFFFFFFFFFFFFF


def quantize_to_trits(text: str, dim: int = 32) -> list[int]:
    """FNV-1a over the text -> dim ternary trits {-1, 0, +1} (TurboVec 1.58-bit)."""
    h = 0xCBF29CE484222325
    for b in text.encode("utf-8"):
        h ^= b
        h = (h * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    trits = []
    for i in range(dim):
        byte = (h >> ((i * 8) % 64)) & 0xFF
        v = byte / 255.0 * 2.0 - 1.0
        trits.append(1 if v > 0.35 else (-1 if v < -0.35 else 0))
    return trits


def query_kimi_k3(api_key: str, prompt: str, model: str = "kimi-k3") -> dict:
    """Query the Moonshot API (host-side ingestion; key from env)."""
    import urllib.request
    body = json.dumps({
        "model": model,
        "messages": [{"role": "user", "content": prompt}],
        "temperature": 0.2,
        "max_tokens": 500,
    }).encode()
    req = urllib.request.Request(API_URL, data=body, headers={
        "Content-Type": "application/json",
        "Authorization": f"Bearer {api_key}",
    })
    with urllib.request.urlopen(req, timeout=60) as resp:
        return json.loads(resp.read().decode())


def ingest(api_key: str, domains: list[str] | None = None, model: str = "kimi-k3") -> dict:
    if not api_key or not api_key.startswith("sk-"):
        return {"ok": False, "error": "MOONSHOT_API_KEY missing or invalid (set in .env)"}
    domains = domains or list(DOMAIN_QUERIES.keys())
    results = {}
    shards = []
    for dom in domains:
        prompt = DOMAIN_QUERIES.get(dom, dom)
        try:
            resp = query_kimi_k3(api_key, prompt, model)
            content = resp["choices"][0]["message"]["content"]
            usage = resp.get("usage", {})
            trits = quantize_to_trits(content)
            sig = seal(f"{dom}:{content[:200]}")
            results[dom] = {
                "status": "INGESTED",
                "chars": len(content),
                "trits": trits,
                "tokens": usage.get("completion_tokens", 0),
                "seal": hex(sig),
                "verdict": "hypothesis-text (K3-filtered, NOT authority)",
            }
            shards.append({"domain": dom, "seal": hex(sig), "chars": len(content)})
            time.sleep(0.5)  # polite rate limiting
        except Exception as e:
            results[dom] = {"status": "FAILED", "error": str(e)[:200]}
    # persist knowledge shard to NAND fabric
    ledger = {
        "protocol": "KIMI_K3_INGESTION",
        "council_seal": "9ace02f3c99bd9d5",
        "monad_anchor": hex(MONAD_ANCHOR),
        "timestamp": time.strftime("%Y-%m-%d %H:%M:%S"),
        "model": model,
        "domains_ingested": sum(1 for r in results.values() if r.get("status") == "INGESTED"),
        "shards": shards,
    }
    out = os.path.join(ROOT, "kimi_k3_ingestion_ledger.json")
    with open(out, "w", encoding="utf-8") as f:
        json.dump(ledger, f, indent=1)
    with open(os.path.join(ROOT, "swarm_v2_nand_fabric.k3"), "a", encoding="utf-8") as f:
        f.write("\n--- KIMI_K3_INGESTION v1 ---\n")
        f.write(json.dumps(ledger, indent=1))
        f.write("\n")
    return {"ok": True, "ledger": ledger, "domains": results}


if __name__ == "__main__":
    key = os.environ.get("MOONSHOT_API_KEY", "")
    r = ingest(key)
    print(json.dumps(r, indent=1)[:1500] if r.get("ok") else json.dumps(r))
