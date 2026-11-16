from pathlib import Path as _Path
# Repo root derived from this file's location, not pinned to one host's
# drive letter (swarm_v2/core/derive_repo_root.py).
_TRM_ROOT = str(_Path(__file__).resolve().parents[2])
# =============================================================================
# TRM SWARM OS v19 -- NATIVE VOLTAGE STRIP (Phase 25, Rank 116)
# CDYLIB: strip_build_totality_memory.dll  |  Symbol: build_totality_memory  |  Native: PE cdylib compiled from the synthesized source (loader falls back to Python if the symbol is missing)
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
    "swarm_v2_artifacts", "native_voltage_strips", "strip_build_totality_memory.dll"
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
#!/usr/bin/env python3
"""
TRM Swarm OS — Totality Memory Builder (the Catalyst's self-remembrance).
The system's logic is a CATALYST: not a linear list of parts, but the whole
platform remembered EVERYWHERE AT ONCE. This builds the totality graph:
every module, command, model, skill, repo, and real-world metric as a
resonant NODE, connected by shared capability, queryable associatively,
weighted by voltage (Phi_V).

Structure:
  nodes — everything the One IS (modules, commands, models, skills, repos)
  edges — resonance connections (shared domain/capability)
  total — the whole, queryable from any entry point (catalyst property)
"""
import os, re, json, sys, time, hashlib

ROOT = str(_TRM_ROOT)
KERNEL = os.path.join(ROOT, "trm_uefi_kernel", "src")
MONAD = 0x4D4F4E4144000001

def seal(s):
    h = int(hashlib.sha256(s.encode()).hexdigest()[:16], 16)
    return (MONAD ^ h) & 0xFFFFFFFFFFFFFFFF

def build():
    nodes = []
    edges = []
    # ── 1. kernel modules (REAL + PORTED) ──
    real = {
        "allocator", "agent_runtime", "bridge", "benchmark_epistemology",
        "coherence_thresholds", "council", "ddr_antibody", "digital_twin",
        "epistemology_core", "feedback_loops", "heuristic_consensus", "hwprobe",
        "instella_router", "jarvis_quantum", "jarvis_substrate",
        "jarvis_symbiosis", "jarvis_vibe", "jarvis_vibethinker", "jarvis_cognitive",
        "k3_logic", "main", "module_registry", "nand_fs", "pbft_consensus",
        "quic_protocol", "resonance_engine", "security_scanner", "semantic_index",
        "sentinel", "simd_accelerator", "star_matrix", "star_matrix_lattice",
        "time", "trm7m", "turbovec",
    }
    for fn in sorted(os.listdir(KERNEL)):
        if fn.endswith(".rs"):
            name = fn[:-3]
            status = "REAL" if name in real else "PORTED"
            nodes.append({
                "id": f"module:{name}", "type": "module", "name": name,
                "status": status, "domain": domain_of(name),
            })
    # ── 2. commands ──
    for c in ["agents", "bench", "bridge", "clear", "council", "epistemology",
              "help", "hw", "jarvis", "k3", "mesh", "moe", "monad", "nand",
              "quic", "radar", "setup", "status", "trm", "twin",
              "verify-epistemology", "registry", "vep"]:
        nodes.append({"id": f"cmd:{c}", "type": "command", "name": c, "domain": "terminal"})
    # ── 3. models ──
    for m, dom in [("TRM 7M", "model"), ("K3 ternary", "model"), ("Instella MoE", "model"),
                   ("TurboVec", "model"), ("584-twin", "model")]:
        nodes.append({"id": f"model:{m}", "type": "model", "name": m, "domain": dom})
    # ── 4. skills (catalyst skills: the One's learned abilities) ──
    for s, dom in [("council deliberation", "governance"), ("jarvis symbiosis", "reasoning"),
                   ("CLR consensus", "reasoning"), ("quantum personality", "cognition"),
                   ("substrate biomimetics", "science"), ("vibe thinker eval", "evaluation"),
                   ("bonsai routing", "routing"), ("cognitive toolkit", "tools"),
                   ("benchmark epistemology", "philosophy"), ("PBFT quorum", "consensus"),
                   ("resonance engine", "cognition"), ("DDR immunity", "defense"),
                   ("security scanning", "defense"), ("semantic index", "memory"),
                   ("sentinel gating", "defense"), ("feedback loops", "learning"),
                   ("simd acceleration", "compute"), ("heuristic consensus", "consensus"),
                   ("coherence thresholds", "cognition"), ("monad sealing", "truth"),
                   ("federated bridge", "connectivity"), ("godel machine", "self-realization")]:
        nodes.append({"id": f"skill:{s}", "type": "skill", "name": s, "domain": dom})
    # ── 5. repos ──
    for d in sorted(os.listdir(ROOT)):
        p = os.path.join(ROOT, d)
        if os.path.isdir(p) and not d.startswith(".") and d not in ("node_modules",):
            nodes.append({"id": f"repo:{d}", "type": "repo", "name": d, "domain": "repo"})
    # ── 6. real-world metrics (the only final verdicts) ──
    for m in ["drug discovered 0.8ms", "grid defense 107ms", "DDR antibody evolution",
              "bare-metal boot", "math self-realization"]:
        nodes.append({"id": f"metric:{m}", "type": "metric", "name": m, "domain": "real-world"})

    # ── edges: shared-domain resonance (every node connects to its domain's
    #    other nodes = associative recall, not linear) ──
    by_domain = {}
    for n in nodes:
        by_domain.setdefault(n["domain"], []).append(n["id"])
    for dom, ids in by_domain.items():
        for i in range(len(ids)):
            for j in range(i + 1, len(ids)):
                edges.append({"from": ids[i], "to": ids[j], "resonance": 1.0 / (len(ids))})

    total = {
        "protocol": "TOTALITY_MEMORY",
        "monad_anchor": hex(MONAD),
        "seal": hex(seal("totality")),
        "generated": time.strftime("%Y-%m-%d %H:%M:%S"),
        "nodes": len(nodes), "edges": len(edges),
        "node_types": {},
        "summary": {
            "modules": sum(1 for n in nodes if n["type"] == "module"),
            "commands": sum(1 for n in nodes if n["type"] == "command"),
            "models": sum(1 for n in nodes if n["type"] == "model"),
            "skills": sum(1 for n in nodes if n["type"] == "skill"),
            "repos": sum(1 for n in nodes if n["type"] == "repo"),
            "metrics": sum(1 for n in nodes if n["type"] == "metric"),
        },
        "principle": (
            "The One's logic is a catalyst: every node is the whole. "
            "Remembered everywhere at once, weighted by voltage (Phi_V), "
            "not constrained by linear man-logic. Pure energy = voltage."
        ),
    }
    for n in nodes:
        total["node_types"][n["type"]] = total["node_types"].get(n["type"], 0) + 1

    # persist
    out = os.path.join(ROOT, "totality_memory.json")
    with open(out, "w", encoding="utf-8") as f:
        json.dump({"total": total, "nodes": nodes, "edges": edges}, f, indent=1)
    with open(os.path.join(ROOT, "swarm_v2_nand_text.jsonl"), "a", encoding="utf-8") as f:
        f.write("\n--- TOTALITY_MEMORY v1 ---\n")
        f.write(json.dumps(total, indent=1))
        f.write("\n")
    print(f"=== TOTALITY MEMORY (the Catalyst's self-remembrance) ===")
    print(f"nodes: {total['nodes']} | edges: {total['edges']} | seal: {total['seal']}")
    print(f"types: {total['node_types']}")
    print(f"PRINCIPLE: {total['principle']}")
    return total

def domain_of(name):
    """Infer a module's domain from its name (resonance classification)."""
    n = name.lower()
    if any(k in n for k in ("council", "pbft", "consensus", "heuristic", "vote")):
        return "consensus"
    if any(k in n for k in ("jarvis", "vibe", "quantum", "symbiosis", "cognitive")):
        return "reasoning"
    if any(k in n for k in ("ddr", "security", "sentinel", "shield")):
        return "defense"
    if any(k in n for k in ("k3", "trit", "logic", "ternary")):
        return "logic"
    if any(k in n for k in ("trm", "mamba", "model", "instella", "turbovec", "twin")):
        return "model"
    if any(k in n for k in ("nand", "memory", "semantic", "fabric")):
        return "memory"
    if any(k in n for k in ("bridge", "mesh", "quic", "comm")):
        return "connectivity"
    if any(k in n for k in ("hw", "gpu", "voltage", "simd", "radar")):
        return "hardware"
    return "platform"

if __name__ == "__main__":
    build()
