"""Contract tests for the K3 logical fast path.

The interceptor is the platform's deterministic fast path: it decides, before any
model call, whether a task can be resolved natively. These tests pin the CONTRACT,
not a snapshot:

  1. An UNINFORMED state (the sha256 fallback) may ROUTE but may never ASSERT.
     Measured over 20,000 tasks, the unanimity rule D() = min over 32 hash-derived
     trits reaches TRUE 0 times and FALSE 100% of the time -- so treating the
     fallback as a decision rejects every task at the microkernel. Regression here
     means the fast path silently becomes a reject-everything gate.
  2. An INFORMED state must reach a real native bypass -- a fast path that cannot
     fire is not a fast path.
  3. The reported offload target must be the engine's own router verdict, not an
     independently chosen label.
"""
import hashlib

from swarm_v2.core.ternary_kleene_engine import (
    KleeneTernaryEngine,
    intercept_with_kleene,
    TRUE,
    FALSE,
    UNKNOWN,
)


def _hash_vector(task: str):
    h = hashlib.sha256(task.encode()).digest()
    return [(b / 255.0 * 2 - 1) for b in h[:32]]


def test_uninformed_state_never_asserts_a_bypass_or_a_rejection():
    """The noise fallback must route, never decide."""
    for i in range(50):
        result = intercept_with_kleene("task-%d" % i)
        assert result is not None, "fallback should route rather than ask for neural work"
        assert result["state_informed"] is False
        assert "[K3_BYPASS]" not in result["result"]
        assert "[K3_REJECT]" not in result["result"]
        assert result["llm_avoided"] is False, "a routed offload still costs a model call"


def test_unanimity_over_hash_state_is_unreachable():
    """Documents WHY the fallback may not assert: the AND over 32 noise trits is dead.

    If this ever starts passing, the fallback vector stopped being information-free
    and the rule above can be revisited.
    """
    engine = KleeneTernaryEngine()
    true_count = sum(
        engine.bypass_decision(engine.state_reduction(_hash_vector("t%d" % i))).trit == TRUE
        for i in range(2000)
    )
    assert true_count == 0, (
        "min() over 32 sha256-derived trits reached TRUE %d times" % true_count
    )


def test_informed_determined_state_reaches_a_native_bypass():
    """A fast path that cannot fire is not a fast path."""
    result = intercept_with_kleene("deterministic native task", continuous_vector=[1.0] * 32)
    assert result is not None
    assert result["state_informed"] is True
    assert result["decision_trit"] == TRUE
    assert result["offload_target"] == "NATIVE_BYPASS"
    assert result["llm_avoided"] is True

    rejected = intercept_with_kleene("invalid task", continuous_vector=[-1.0] * 32)
    assert rejected["decision_trit"] == FALSE
    assert rejected["llm_avoided"] is True


def test_offload_target_is_the_engines_own_router_verdict():
    """The reported target must be derived, not independently asserted."""
    engine = KleeneTernaryEngine()
    for vec in ([1.0] * 32, [0.0] * 32, [0.5, -0.5] * 16, [-1.0] * 32):
        result = intercept_with_kleene("task", continuous_vector=vec)
        expected = engine.evaluate_offload(engine.state_reduction(vec))
        if result is not None:
            assert result["offload_target"] == expected


def test_fully_ambiguous_state_defers_to_neural_resolution():
    """All-UNKNOWN is exactly the case the fast path must NOT decide."""
    assert intercept_with_kleene("ambiguous", continuous_vector=[0.0] * 32) is None
