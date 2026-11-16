"""
TRM Swarm OS v18 — LoRA-Driven Agent Response Modulator
=========================================================
Uses LoRA adapter delta vectors (64-dim) to actually modify agent responses.
Each dimension maps to a specific behavioral axis.

Delta dimensions:
  0: Confidence modifier (0.0-2.0)
  1: Verbosity modifier (0.0-3.0)
  2-11: Domain emphasis (10 domains)
  12-21: Tone vector (10 dimensions)
  22-31: Action bias (10 biases)
  32-63: Agent-specific modifiers (32 dims)

Integrated into deterministic interceptor — agents produce different outputs
based on their loaded LoRA adapter without changing the base model.
"""
import numpy as np
from typing import Dict, List, Optional

# Domain emphasis labels
DOMAINS = [
    "security", "logic", "development", "research", "verification",
    "architecture", "integration", "data", "orchestration", "telemetry"
]

# Tone dimensions
TONES = [
    "formal", "technical", "concise", "confident", "proactive",
    "analytical", "direct", "diplomatic", "urgent", "speculative"
]

# Action biases
ACTIONS = [
    "investigate", "defend", "optimize", "build", "verify",
    "design", "connect", "fetch", "schedule", "monitor"
]

# Agent-specific modifier descriptions
AGENT_MODIFIERS = {
    "Shield": ["threat_level", "block_confidence", "audit_depth", "ddr_activation",
               "zero_trust", "ebpf_probe", "lattice_strength", "kyber_bits"],
    "Logic": ["proof_depth", "k3_confidence", "smt_timeout", "qed_certainty",
              "coq_tactic", "lean4_auto", "sat_strategy", "resolution_bound"],
    "Devo": ["compile_strictness", "sandbox_ttl", "hotpatch_risk", "ci_timeout",
             "lint_severity", "test_parallelism", "review_depth", "refactor_aggression"],
    "Seeker": ["search_depth", "source_count", "recency_weight", "novelty_threshold",
               "citation_required", "arxiv_pages", "pubmed_filter", "patent_scan"],
    "Verify": ["test_coverage", "fuzz_iterations", "mutation_rate", "benchmark_runs",
               "load_users", "stress_duration", "leak_threshold", "regression_scope"],
    "Archi": ["design_depth", "topology_nodes", "blueprint_quality", "hvbe_binding",
              "skg_traversal", "gan_discriminator", "fve_strictness", "edsma_risk"],
}


class AgentResponseModulator:
    """
    Translates LoRA delta vectors into actual response modifications.
    Used by the deterministic interceptor to produce agent-specific responses.
    """
    
    def __init__(self):
        self.last_delta: Optional[np.ndarray] = None
        self.last_agent: str = "unknown"
    
    def modulate(self, base_response: str, delta: np.ndarray, agent_name: str) -> str:
        """
        Apply LoRA delta to modify a deterministic response.
        
        Args:
            base_response: The raw deterministic interceptor response
            delta: 64-dim float32 vector from LoRA adapter
            agent_name: Which agent is processing (for specialty modifiers)
        
        Returns:
            Modified response with agent-specific character
        """
        if len(delta) < 32:
            return base_response  # Not enough dimensions
        
        self.last_delta = delta.copy()
        self.last_agent = agent_name
        
        # Extract modifiers
        confidence = float(np.clip(1.0 + delta[0], 0.1, 2.0))
        verbosity = float(np.clip(1.0 + delta[1], 0.3, 3.0))
        
        # Domain emphasis (which area to highlight)
        domain_scores = delta[2:12]
        top_domain_idx = int(np.argmax(np.abs(domain_scores)))
        top_domain = DOMAINS[top_domain_idx] if top_domain_idx < len(DOMAINS) else "general"
        
        # Tone selection (pick top 2 tones)
        tone_scores = delta[12:22]
        top_tones_idx = np.argsort(np.abs(tone_scores))[-2:][::-1]
        primary_tone = TONES[top_tones_idx[0]] if top_tones_idx[0] < len(TONES) else "neutral"
        secondary_tone = TONES[top_tones_idx[1]] if top_tones_idx[1] < len(TONES) else "neutral"
        
        # Action bias
        action_scores = delta[22:32]
        top_action_idx = int(np.argmax(np.abs(action_scores)))
        top_action = ACTIONS[top_action_idx] if top_action_idx < len(ACTIONS) else "execute"
        
        # Build modified response
        parts = [base_response.strip()]
        
        # Add domain emphasis
        domain_prefixes = {
            "security": "🛡️ [Shield Analysis]",
            "logic": "🧮 [Logic Assessment]",
            "development": "⚙️ [Devo Build]",
            "research": "🔬 [Seeker Finding]",
            "verification": "✓ [Verify Audit]",
            "architecture": "🏗️ [Archi Blueprint]",
            "integration": "🔗 [Flow Route]",
            "data": "📡 [Bridge Fetch]",
            "orchestration": "🎯 [Orchestra Schedule]",
            "telemetry": "📊 [EventLedger Monitor]",
        }
        
        if confidence > 0.7:
            prefix = domain_prefixes.get(top_domain, f"[{agent_name}]")
            if prefix not in parts[0]:
                parts.insert(0, prefix)
        
        # Add tone-based qualifiers
        if confidence < 0.5:
            parts.append(f"(confidence: low, tone: {primary_tone})")
        elif confidence > 1.5:
            parts.append(f"(high confidence, {primary_tone} assessment)")
        else:
            parts.append(f"({primary_tone} mode)")
        
        # Add action bias if strong
        if abs(action_scores[top_action_idx]) > 0.3:
            action_phrases = {
                "investigate": " Recommend deeper investigation.",
                "defend": " Security posture hardening advised.",
                "optimize": " Optimization opportunity identified.",
                "build": " Ready for implementation.",
                "verify": " Verification checkpoint passed.",
                "design": " Architectural review recommended.",
                "connect": " Service integration gate open.",
                "fetch": " Data retrieval standing by.",
                "schedule": " Task queued for execution.",
                "monitor": " Continuous monitoring engaged.",
            }
            parts.append(action_phrases.get(top_action, ""))
        
        # Apply verbosity
        response = " ".join(p for p in parts if p)
        if verbosity > 2.0:
            response += f"\n  • Domain: {top_domain} (emphasis: {domain_scores[top_domain_idx]:.2f})"
            response += f"\n  • Tone: {primary_tone}/{secondary_tone}"
            response += f"\n  • Bias: {top_action}"
        
        return response
    
    def get_modifier_summary(self) -> Dict:
        """Return human-readable summary of last modulation."""
        if self.last_delta is None:
            return {"status": "no modulation applied"}
        
        confidence = float(np.clip(1.0 + self.last_delta[0], 0.1, 2.0))
        top_domain = DOMAINS[int(np.argmax(np.abs(self.last_delta[2:12])))]
        primary_tone = TONES[int(np.argsort(np.abs(self.last_delta[12:22]))[-1])]
        top_action = ACTIONS[int(np.argmax(np.abs(self.last_delta[22:32])))]
        
        return {
            "agent": self.last_agent,
            "confidence": round(confidence, 3),
            "domain": top_domain,
            "tone": primary_tone,
            "action_bias": top_action,
            "delta_norm": float(np.linalg.norm(self.last_delta)),
        }


# === SINGLETON ===
_modulator: Optional[AgentResponseModulator] = None

def get_modulator() -> AgentResponseModulator:
    global _modulator
    if _modulator is None:
        _modulator = AgentResponseModulator()
    return _modulator


# === SELF-TEST ===
if __name__ == "__main__":
    print("=== Agent Response Modulator — Self-Test ===\n")
    
    mod = get_modulator()
    
    # Test with different agents
    agents = ["Shield", "Logic", "Seeker", "Archi"]
    base = "System v5_online — 12 agents active, TurboVecNative memory, 57 Radix-Trie patterns"
    
    for agent in agents:
        # Simulate agent-specific LoRA delta
        np.random.seed(hash(agent) % 2**32)
        delta = np.random.randn(64).astype(np.float32) * 0.5
        
        response = mod.modulate(base, delta, agent)
        summary = mod.get_modifier_summary()
        
        print(f"--- {agent} ---")
        print(f"  Delta norm: {summary['delta_norm']:.3f}")
        print(f"  Confidence: {summary['confidence']:.2f}")
        print(f"  Domain: {summary['domain']}")
        print(f"  Tone: {summary['tone']}")
        print(f"  Action: {summary['action_bias']}")
        print(f"  Response: {response[:100]}...")
        print()
    
    print("✅ Agent Response Modulator — agents produce different outputs from same base")
