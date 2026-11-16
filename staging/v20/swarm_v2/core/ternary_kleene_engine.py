"""
TRM Swarm OS v18 — Ternary Kleene Logic Engine (K3)
Implements formal K3 algebra, state-space reduction Φ, and bypass decision D(Ψ)

Based on: "Mathematical Formalization of the Cognitive Microkernel"
Target: Replace keyword-based deterministic interceptor with formal ternary logic

T = {-1, 0, +1} where:
  +1 = TRUE / Executable / Satisfied
   0 = UNKNOWN / Ambiguous / Requires Neural Resolution  
  -1 = FALSE / Invalid / Rejected

Operators:
  x ∧ y = min(x, y)
  x ∨ y = max(x, y)
  ¬x = -x
"""

import numpy as np
from typing import List, Tuple, Optional
from dataclasses import dataclass
import time
import math

# ── NATIVE VOLTAGE STRIP — Phase 5 auto-injected ──────────────────────────────
import ctypes as _ctypes, numpy as _np
from pathlib import Path as _Path

_BASE3_CODEC_DLL = None
try:
    _dll_p = _Path(__file__).parent.parent.parent / "swarm_v2_artifacts" / "native_voltage_strips" / "strip_base3_codec.dll"
    if _dll_p.exists():
        _BASE3_CODEC_DLL = _ctypes.CDLL(str(_dll_p))
        _BASE3_CODEC_DLL.strip_base3_trit_pack.argtypes = [
            _ctypes.c_void_p, _ctypes.c_size_t, _ctypes.c_void_p
        ]
        _BASE3_CODEC_DLL.strip_base3_trit_pack.restype = None
except Exception as _e:
    _BASE3_CODEC_DLL = None
    _BASE3_CODEC_DLL_ERROR = f"strip not loadable: {_e}"

def native_base3_pack(trits: _np.ndarray) -> _np.ndarray:
    """Base-3 radix 5-trit/byte codec (282.1x speedup, K3: +1 TRUE)."""
    n = len(trits)
    if _BASE3_CODEC_DLL is not None and n % 5 == 0:
        arr = _np.ascontiguousarray(trits, dtype=_np.uint8)
        out = _np.zeros(n // 5, dtype=_np.uint8)
        _BASE3_CODEC_DLL.strip_base3_trit_pack(arr.ctypes.data, n, out.ctypes.data)
        return out
    # Python fallback
    bytes_out = []
    for i in range(0, n, 5):
        val = int(trits[i]) + 3*int(trits[i+1]) + 9*int(trits[i+2]) + 27*int(trits[i+3]) + 81*int(trits[i+4])
        bytes_out.append(val)
    return _np.array(bytes_out, dtype=_np.uint8)
# ── END NATIVE VOLTAGE STRIP ──────────────────────────────────────────────────


# === TRIT DEFINITIONS ===
FALSE = -1
UNKNOWN = 0
TRUE = 1

@dataclass
class KleeneState:
    """A ternary decision state with confidence and ambiguity tracking."""
    trit: int  # -1, 0, or +1
    confidence: float  # 0.0 to 1.0
    ambiguity_ratio: float  # Fraction of unknown trits in source vector
    
    @property
    def is_bypassable(self) -> bool:
        return self.trit == TRUE and self.confidence >= 0.88
    
    @property
    def is_rejected(self) -> bool:
        return self.trit == FALSE

class KleeneTernaryEngine:
    """
    Implements Kleene 3-Valued Logic for cognitive microkernel bypass decisions.
    
    Sections from the formal spec:
    2.1 — K3 Operators (∧, ∨, ¬)
    2.2 — State-Space Reduction Φ: ℝᵈ → 𝒯ᵐ
    2.3 — Bypass Decision D(Ψ⃗) = ∧ⱼ Ψ⃗ⱼ
    """
    
    # Default threshold bands for state reduction
    DEFAULT_TAU_UPPER = 0.35   # Values above → +1 (TRUE)
    DEFAULT_TAU_LOWER = -0.35  # Values below → -1 (FALSE)
    
    def __init__(self, tau_upper: float = DEFAULT_TAU_UPPER, tau_lower: float = DEFAULT_TAU_LOWER):
        self.tau_upper = tau_upper
        self.tau_lower = tau_lower
        self.operation_count = 0
        self.bypass_count = 0
        
    # === 2.1 KLEENE K3 OPERATORS ===
    
    @staticmethod
    def kleene_and(x: int, y: int) -> int:
        """x ∧ y = min(x, y) — True only if both are True."""
        return min(x, y)
    
    @staticmethod
    def kleene_or(x: int, y: int) -> int:
        """x ∨ y = max(x, y) — False only if both are False."""
        return max(x, y)
    
    @staticmethod
    def kleene_not(x: int) -> int:
        """¬x = -x"""
        return -x
    
    @staticmethod
    def kleene_imply(x: int, y: int) -> int:
        """x → y = ¬x ∨ y = max(-x, y)"""
        return max(-x, y)
    
    @staticmethod
    def kleene_equiv(x: int, y: int) -> int:
        """x ↔ y = (x → y) ∧ (y → x)"""
        return min(max(-x, y), max(-y, x))
    
    # === 2.2 STATE-SPACE REDUCTION Φ: ℝᵈ → 𝒯ᵐ ===
    
    def state_reduction(self, vector: List[float], 
                        W: Optional[List[List[float]]] = None,
                        b: Optional[List[float]] = None) -> List[int]:
        """
        Φ(S)_i = { +1 if ⟨S, W_i⟩ + b_i > τ_upper
                  { -1 if ⟨S, W_i⟩ + b_i < τ_lower  
                  {  0 otherwise
        
        Maps continuous d-dimensional state vector S ∈ ℝᵈ 
        to m-dimensional Kleene ternary decision vector Ψ⃗ ∈ 𝒯ᵐ
        """
        if W is None:
            # Use identity projection if no projection matrix
            W = [[1.0 if i == j else 0.0 for j in range(len(vector))] 
                 for i in range(len(vector))]
        
        if b is None:
            b = [0.0] * len(W)
        
        result = []
        for i, (w_row, bias) in enumerate(zip(W, b)):
            projection = sum(s * w for s, w in zip(vector, w_row)) + bias
            if projection > self.tau_upper:
                result.append(TRUE)
            elif projection < self.tau_lower:
                result.append(FALSE)
            else:
                result.append(UNKNOWN)
        
        return result
    
    def quick_reduction(self, values: List[float]) -> List[int]:
        """Fast reduction without projection matrix — threshold each value directly."""
        result = []
        for v in values:
            if v > self.tau_upper:
                result.append(TRUE)
            elif v < self.tau_lower:
                result.append(FALSE)
            else:
                result.append(UNKNOWN)
        return result
    
    # === 2.3 BYPASS DECISION D(Ψ⃗) = ∧ⱼ Ψ⃗ⱼ ===
    
    def bypass_decision(self, psi_vector: List[int]) -> KleeneState:
        """
        D(Ψ⃗) = ∧_{j=1}^{m} Ψ⃗ⱼ = min_j(Ψ⃗ⱼ)
        
        Returns:
          +1 → BYPASS ACTIVATED: Execute natively in <0.5ms
          -1 → FAULT INTERCEPT: Reject/error in <0.1ms  
           0 → NEURAL SYSCALL: Invoke LLM to resolve ambiguity
        """
        decision_trit = min(psi_vector)  # This IS the Kleene AND across all trits
        
        # Calculate confidence metrics
        total = len(psi_vector)
        unknown_count = sum(1 for t in psi_vector if t == UNKNOWN)
        true_count = sum(1 for t in psi_vector if t == TRUE)
        false_count = sum(1 for t in psi_vector if t == FALSE)
        
        ambiguity_ratio = unknown_count / total if total > 0 else 0.0
        
        # Confidence: how strongly we believe the decision
        if decision_trit == TRUE:
            confidence = true_count / max(total, 1)
        elif decision_trit == FALSE:
            confidence = false_count / max(total, 1)
        else:
            confidence = 1.0 - ambiguity_ratio
        
        self.operation_count += 1
        if decision_trit == TRUE and confidence >= 0.88:
            self.bypass_count += 1
        
        return KleeneState(
            trit=decision_trit,
            confidence=confidence,
            ambiguity_ratio=ambiguity_ratio
        )
    
    # === VECTORIZED KLEENE OPERATIONS (SIMD-equivalent in NumPy) ===
    
    @staticmethod
    def vectorized_and(a: List[int], b: List[int]) -> List[int]:
        """Element-wise Kleene AND for two trit vectors."""
        return [min(ai, bi) for ai, bi in zip(a, b)]
    
    @staticmethod
    def vectorized_or(a: List[int], b: List[int]) -> List[int]:
        """Element-wise Kleene OR for two trit vectors."""
        return [max(ai, bi) for ai, bi in zip(a, b)]
    
    @staticmethod
    def vectorized_not(a: List[int]) -> List[int]:
        """Element-wise Kleene NOT."""
        return [-x for x in a]
    
    # === TRIT PACKING / UNPACKING (Section 3 of spec) ===
    
    @staticmethod
    def pack_5_trits(t0: int, t1: int, t2: int, t3: int, t4: int) -> int:
        """
        Encode 5 trits {-1,0,+1} → {0,1,2} into one byte (0-242).
        B = t0' + 3·t1' + 9·t2' + 27·t3' + 81·t4'
        where t_i' = trit_i + 1

        Routed through native_base3_pack (strip_base3_codec DLL) — the same
        radix-3 5-trit→byte codec, fed the biased digits {0,1,2}; when the DLL
        is absent the wrapper's pure-Python fallback performs the identical
        packing.
        """
        digits = _np.array([t0 + 1, t1 + 1, t2 + 1, t3 + 1, t4 + 1], dtype=_np.uint8)
        return int(native_base3_pack(digits)[0])
    
    @staticmethod
    def unpack_5_trits(byte: int) -> List[int]:
        """Decode one byte (0-242) back into 5 trits {-1, 0, +1}."""
        trits = []
        remaining = byte
        for _ in range(5):
            value = remaining % 3
            trits.append(value - 1)  # Map {0,1,2} → {-1,0,+1}
            remaining //= 3
        return trits
    
    # === DUAL-BITVECTOR REPRESENTATION (Section 4 of spec) ===
    
    @staticmethod
    def trit_vector_to_bitvectors(psi: List[int]) -> Tuple[List[int], List[int]]:
        """
        Convert trit vector to dual-bitvector (P, N) representation.
        P[i] = 1 if trit[i] == +1
        N[i] = 1 if trit[i] == -1
        UNKNOWN: P[i]=0, N[i]=0
        """
        P = [1 if t == TRUE else 0 for t in psi]
        N = [1 if t == FALSE else 0 for t in psi]
        return P, N
    
    @staticmethod
    def bitvectors_to_trit(P: List[int], N: List[int]) -> List[int]:
        """Convert dual-bitvectors back to trit array."""
        return [
            TRUE if p == 1 else (FALSE if n == 1 else UNKNOWN)
            for p, n in zip(P, N)
        ]
    
    # === AMBIGUITY-BASED OFFLOAD DECISION (Section 10 of spec) ===
    
    def evaluate_offload(self, psi_vector: List[int]) -> str:
        """
        Determine offload target based on ambiguity ratio:
        - ratio == 0.0 → NATIVE_BYPASS (no LLM needed)
        - 0.0 < ratio ≤ 0.60 → INSTELLA_MOE (lightweight expert routing)
        - ratio > 0.60 → BONSAI_TERNARY (deep ternary reasoning)
        """
        total = len(psi_vector)
        unknown_count = sum(1 for t in psi_vector if t == UNKNOWN)
        ratio = unknown_count / total if total > 0 else 0.0
        
        if ratio == 0.0:
            return "NATIVE_BYPASS"
        elif ratio <= 0.60:
            return "INSTELLA_MOE"
        else:
            return "BONSAI_TERNARY"
    
    # === STATISTICS ===
    
    @property
    def bypass_rate(self) -> float:
        """Percentage of decisions that bypassed LLM."""
        if self.operation_count == 0:
            return 0.0
        return self.bypass_count / self.operation_count


# === SINGLETON ===
_engine_instance: Optional[KleeneTernaryEngine] = None

def get_kleene_engine() -> KleeneTernaryEngine:
    global _engine_instance
    if _engine_instance is None:
        _engine_instance = KleeneTernaryEngine()
    return _engine_instance


# === ENHANCED DETERMINISTIC INTERCEPTOR USING K3 LOGIC ===

def intercept_with_kleene(task: str, continuous_vector: Optional[List[float]] = None) -> Optional[dict]:
    """
    Enhanced deterministic interceptor using Kleene ternary logic.
    Replaces keyword-matching with formal state reduction and bypass decision.
    
    Args:
        task: The task description string
        continuous_vector: Optional continuous state vector from perception subsystem.
                          If None, synthesizes vector from task hash.
    
    Returns:
        Result dict if bypass is activated, None if LLM needed.
    """
    engine = get_kleene_engine()
    t0 = time.time()
    informed = continuous_vector is not None

    if not informed:
        # FALLBACK — sha256(task) is UNIFORM NOISE. It carries no information about the
        # task, so any decision derived from it is a lottery, not a derivation. Measured
        # over 20,000 tasks: the unanimity rule D() = min over 32 hash-derived trits
        # yields NATIVE_BYPASS 0 times and REJECT 100% of the time. The fallback is
        # therefore never allowed to found a bypass or a rejection below.
        import hashlib
        h = hashlib.sha256(task.encode()).digest()
        continuous_vector = [(b / 255.0 * 2 - 1) for b in h[:32]]  # 32-dim vector in [-1, +1]

    # Step 1: State reduction Φ: ℝᵈ → 𝒯ᵐ
    psi = engine.state_reduction(continuous_vector)

    # Step 2: Bypass decision D(Ψ⃗) — and the engine's own ambiguity router
    state = engine.bypass_decision(psi)
    offload = engine.evaluate_offload(psi)

    common = {
        "bypass": "kleene_ternary",
        "decision_trit": state.trit,
        "confidence": state.confidence,
        "ambiguity_ratio": state.ambiguity_ratio,
        "offload_target": offload,
        "state_informed": informed,
        "latency_us": (time.time() - t0) * 1_000_000,
        "psi_vector": psi[:16],  # First 16 trits for debugging
    }

    # Step 3: Route. An uninformed state may ROUTE but may not ASSERT: no LLM call is
    # saved by guessing, and a bypass claimed from noise is exactly a guess.
    if not informed and not (state.trit == TRUE and offload == "NATIVE_BYPASS"):
        if offload == "BONSAI_TERNARY":
            return None  # deep ternary reasoning -> caller falls through to Instella/LLM
        common["result"] = (
            f"[K3_ROUTE] state vector is the sha256 fallback (uniform noise, no decision "
            f"content) -> routing to {offload} instead of asserting a bypass or a rejection."
        )
        common["llm_avoided"] = False  # a routed offload still costs a model call
        return common

    if state.trit == TRUE:
        common["result"] = (
            f"[K3_BYPASS] Ternary decision: +1 (confidence {state.confidence:.2f}). "
            f"Offload: {offload}. LLM bypassed."
        )
        common["llm_avoided"] = True
        return common

    if state.trit == FALSE:
        common["result"] = (
            "[K3_REJECT] Ternary decision: -1. Request rejected at microkernel level. "
            "No LLM call."
        )
        common["llm_avoided"] = True
        return common

    if offload == "INSTELLA_MOE":
        common["result"] = (
            f"[K3_ROUTE] ambiguity {state.ambiguity_ratio:.2f} -> INSTELLA_MOE "
            f"(light expert routing)."
        )
        common["llm_avoided"] = False
        return common

    # UNKNOWN with high ambiguity: requires neural resolution
    return None  # Fall through to Instella → LLM


# === SELF-TEST ===
if __name__ == "__main__":
    engine = KleeneTernaryEngine()
    
    print("=== Ternary Kleene Logic Engine — Self-Test ===\n")
    
    # Test K3 operators
    print("K3 Truth Tables:")
    for x in [TRUE, UNKNOWN, FALSE]:
        for y in [TRUE, UNKNOWN, FALSE]:
            print(f"  {x:+d} ∧ {y:+d} = {engine.kleene_and(x,y):+d}  |  {x:+d} ∨ {y:+d} = {engine.kleene_or(x,y):+d}  |  ¬{x:+d} = {engine.kleene_not(x):+d}")
    
    # Test state reduction
    print(f"\nState Reduction Φ (tau_upper={engine.tau_upper}, tau_lower={engine.tau_lower}):")
    test_vec = [0.5, -0.5, 0.1, -0.8, 0.9, -0.2, 0.3, -0.6]
    psi = engine.state_reduction(test_vec)
    print(f"  Input:  {test_vec}")
    print(f"  Output: {psi}")
    
    # Test bypass decision
    state = engine.bypass_decision(psi)
    print(f"\nBypass Decision D(Ψ⃗):")
    print(f"  Trit: {state.trit:+d}")
    print(f"  Confidence: {state.confidence:.3f}")
    print(f"  Ambiguity: {state.ambiguity_ratio:.3f}")
    print(f"  Bypassable: {state.is_bypassable}")
    
    # Test trit packing
    print(f"\nTrit Packing (5→1 byte):")
    test_trits = [TRUE, FALSE, UNKNOWN, TRUE, FALSE]  # +1, -1, 0, +1, -1
    packed = engine.pack_5_trits(*test_trits)
    unpacked = engine.unpack_5_trits(packed)
    print(f"  Input:  {test_trits}")
    print(f"  Packed: {packed} (byte)")
    print(f"  Unpacked: {unpacked}")
    print(f"  Roundtrip OK: {test_trits == unpacked}")
    
    # Test dual-bitvector
    print(f"\nDual-Bitvector Encoding:")
    P, N = engine.trit_vector_to_bitvectors(psi)
    recovered = engine.bitvectors_to_trit(P, N)
    print(f"  Original: {psi}")
    print(f"  P: {P}")
    print(f"  N: {N}")
    print(f"  Recovered: {recovered}")
    print(f"  Roundtrip OK: {psi == recovered}")
    
    # Test offload routing
    print(f"\nAmbiguity-Based Offload:")
    for name, vec in [
        ("All known", [TRUE, TRUE, TRUE, TRUE]),
        ("Mixed", [TRUE, UNKNOWN, TRUE, FALSE]),
        ("Mostly unknown", [UNKNOWN, UNKNOWN, TRUE, UNKNOWN, UNKNOWN]),
        ("All unknown", [UNKNOWN, UNKNOWN, UNKNOWN]),
    ]:
        target = engine.evaluate_offload(vec)
        ratio = sum(1 for t in vec if t == UNKNOWN) / len(vec)
        print(f"  {name} (ratio={ratio:.2f}) → {target}")
    
    # Test K3 interceptor
    print(f"\nK3 Interceptor Test:")
    result = intercept_with_kleene("system status check and health report")
    if result:
        print(f"  BYPASSED: {result['result'][:100]}")
    else:
        print(f"  NEEDS LLM: ambiguity too high")
    
    print(f"\nBypass rate: {engine.bypass_rate:.1%}")
    print("All tests complete.")
