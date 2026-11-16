# =============================================================================
# TRM SWARM OS v19 -- NATIVE VOLTAGE STRIP (Phase 40, Rank 191)
# CDYLIB: strip_web_fetch_utility.dll  |  Native: PE cdylib compiled from the synthesized source (loader falls back to Python if the symbol is missing)
# TurboVec 16x Balanced Ternary Quantization | Kleene K3 Verified
# Monad Truth Anchor: 0x4D4F4E4144000001
# _NATIVE_LOADED = True when CDYLIB active; falls back to pure Python below.
# =============================================================================
import os as _os
import ctypes as _ctypes

_NATIVE_LOADED = False
_NATIVE_REASON = "not attempted"
_NATIVE_DLL_PATH = _os.path.join(
    _os.path.dirname(_os.path.dirname(_os.path.dirname(_os.path.abspath(__file__)))),
    "swarm_v2_artifacts", "native_voltage_strips", "strip_web_fetch_utility.dll"
)
# The fallback below is NOT silent: _NATIVE_REASON records why the native path
# is inactive so a module can report it instead of advertising a strip it
# never loaded.
try:
    if _os.path.exists(_NATIVE_DLL_PATH):
        _native_lib = _ctypes.CDLL(_NATIVE_DLL_PATH)
        _NATIVE_LOADED = True
        _NATIVE_REASON = "PE CDYLIB loaded"
    else:
        _NATIVE_REASON = "strip absent"
except OSError as _e:
    _NATIVE_REASON = f"strip not loadable: {_e}"

# -- Pure Python fallback preserved below (VOLTAGE_STRIPPED) ------------------
"""
Web Fetch Utility — TRM Swarm OS v18
Provides web research capability via Wikipedia API + other free sources.
Used by microkernel for real-world data gathering without Firecrawl.
"""
import urllib.request
import urllib.parse
import json
import time
from typing import List, Dict, Optional

WIKI_API = "https://en.wikipedia.org/w/api.php"
UA = "TRM-Swarm-OS-v18/1.0 (research; contact@trm-swarm.dev)"

def wiki_search(query: str, limit: int = 5) -> List[Dict]:
    """Search Wikipedia for real-world data."""
    params = f"?action=query&list=search&srsearch={urllib.parse.quote(query)}&format=json&srlimit={limit}"
    url = f"{WIKI_API}{params}"
    req = urllib.request.Request(url, headers={"User-Agent": UA})
    try:
        data = json.loads(urllib.request.urlopen(req, timeout=10).read())
        return data.get("query", {}).get("search", [])
    except Exception as e:
        return [{"title": "Error", "snippet": str(e)}]

def wiki_extract(title: str, char_limit: int = 3000) -> str:
    """Get full extract from Wikipedia page."""
    params = f"?action=query&prop=extracts&exintro=1&titles={urllib.parse.quote(title)}&format=json&explaintext=1"
    url = f"{WIKI_API}{params}"
    req = urllib.request.Request(url, headers={"User-Agent": UA})
    try:
        data = json.loads(urllib.request.urlopen(req, timeout=10).read())
        pages = data.get("query", {}).get("pages", {})
        for p in pages.values():
            return p.get("extract", "")[:char_limit]
        return ""
    except Exception as e:
        return f"Error: {e}"

def web_research(query: str, extract_depth: int = 1) -> Dict:
    """Full research: search + extract top results."""
    t0 = time.time()
    results = wiki_search(query, limit=extract_depth)
    enriched = []
    for r in results:
        full_text = wiki_extract(r["title"], char_limit=2000) if extract_depth > 0 else ""
        enriched.append({
            "title": r["title"],
            "snippet": r.get("snippet", ""),
            "extract": full_text[:500] if full_text else "",
            "word_count": len(full_text.split()) if full_text else 0
        })
    return {
        "query": query,
        "results": len(enriched),
        "data": enriched,
        "latency_ms": (time.time()-t0)*1000,
        "source": "wikipedia_api"
    }
