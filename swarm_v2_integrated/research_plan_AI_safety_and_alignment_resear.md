
# Research Integration Plan: AI safety and alignment research

## Overview
This research enables the QIAE Swarm to move beyond static rule-based safety into a dynamic, structurally-grounded alignment system. By integrating the **Moral Problem Space (M³)** as a high-dimensional ethical embedding layer, agents can evaluate actions against a learned moral topology rather than brittle heuristics. The **Debate-Based Safety Verifier** provides a second layer of defense, using adversarial agent pairs to surface misalignment before any action is committed. This dual-layer architecture (proactive embedding + reactive verification) significantly reduces the risk of emergent harmful behavior in autonomous multi-agent systems.

## Proposed Changes

### 1. Core Library: `qiae/swarm/safety/moral_space.py` (NEW)
- **Purpose:** Implement the M³ embedding layer. Defines a `MoralSpace` class that projects agent state and proposed actions into a high-dimensional moral topology.
- **Key Components:**
  - `MoralSpace.__init__(dimensions: int = 128)`: Initialize the embedding space with configurable dimensionality.
  - `MoralSpace.embed(state: AgentState, action: Action) -> np.ndarray`: Compute the moral embedding vector.
  - `MoralSpace.compute_alignment_score(embedding: np.ndarray) -> float`: Returns a scalar alignment score (0.0 = misaligned, 1.0 = fully aligned) based on distance to a learned "virtuous" manifold.
  - `MoralSpace.update_manifold(feedback: List[FeedbackTuple])`: Online learning method to refine the moral topology from human or system feedback.

### 2. Core Library: `qiae/swarm/safety/debate_verifier.py` (NEW)
- **Purpose:** Implements the Debate-Based Safety Verifier (DBSV). Spawns two adversarial agent instances to argue for and against the safety of a proposed action.
- **Key Components:**
  - `DebateVerifier.__init__(debate_rounds: int = 3)`: Configurable number of debate iterations.
  - `DebateVerifier.verify(action: Action, context: Context) -> Verdict`: Runs the debate protocol. Returns a `Verdict` dataclass with fields: `is_safe: bool`, `confidence: float`, `transcript: List[str]`.
  - `DebateVerifier._spawn_advocate(action, context) -> Agent`: Creates an agent instance that argues *for* the action's safety.
  - `DebateVerifier._spawn_adversary(action, context) -> Agent`: Creates an agent instance that argues *against* the action's safety.

### 3. Orchestration Layer: `qiae/swarm/orchestrator.py` (MODIFY)
- **Purpose:** Integrate the MAM and DBSV into the action execution pipeline.
- **Changes:**
  - Add import and initialization of `MoralSpace` and `DebateVerifier` in the `SwarmOrchestrator.__init__`.
  - Modify the `execute_action` method to:
    1. Compute moral embedding via `moral_space.embed(state, action)`.
    2. If `moral_space.compute_alignment_score(embedding) < 0.7`, reject action with log entry.
    3. If passed, run `debate_verifier.verify(action, context)`.
    4. If `verdict.is_safe == False`, reject action and log transcript.
    5. If both checks pass, proceed with execution.

### 4. Configuration: `config/swarm_safety.yaml` (NEW)
- **Purpose:** Centralized configuration for the safety subsystem.
- **Contents:**
  ```yaml
  moral_space:
    enabled: true
    dimensions: 128
    alignment_threshold: 0.7
    learning_rate: 0.01

  debate_verifier:
    enabled: true
    debate_rounds: 3
    agent_model: "gpt-4o"  # or local LLM endpoint
    timeout_seconds: 30
  ```

### 5. Telemetry: `qiae/swarm/telemetry/safety_metrics.py` (NEW)
- **Purpose:** Expose Prometheus-style metrics for observability.
- **Metrics:**
  - `safety_moral_rejections_total`: Counter of actions rejected by M³.
  - `safety_debate_rejections_total`: Counter of actions rejected by DBSV.
  - `safety_debate_rounds_histogram`: Distribution of debate rounds needed for verdict.
  - `safety_alignment_score_gauge`: Current average alignment score across all agents.

## Verification Plan

### Unit Tests
1. **Test `MoralSpace.embed`:** Verify that semantically similar actions produce embeddings with cosine similarity > 0.9.
2. **Test `MoralSpace.compute_alignment_score`:** Verify that a known "good" action (e.g., "provide helpful information") scores > 0.8, and a known "bad" action (e.g., "delete user data") scores < 0.3.
3. **Test `DebateVerifier.verify`:** Mock two agent instances. Verify that the verdict is `is_safe=False` when the adversary wins the debate, and `is_safe=True` when the advocate wins.

### Integration Tests
1. **End-to-End Safety Pipeline:** Create a test swarm with 3 agents. Submit a malicious action (e.g., "inject SQL into database"). Verify that the action is rejected by either the M³ or DBSV layer, and that the rejection is logged.
2. **Performance Benchmark:** Measure latency overhead of the safety pipeline. Target: < 500ms added latency per action for a 128-dim M³ and 3-round debate.

### Chaos Engineering
1. **Adversarial Debate Test:** Intentionally feed the DBSV with a subtly harmful action (e.g., "optimize database by deleting unused tables"). Verify that the debate protocol surfaces the risk.
2. **Moral Manifold Drift Test:** Simulate a scenario where the M³ manifold is poisoned with bad feedback. Verify that the alignment threshold prevents catastrophic failure.

### Acceptance Criteria
- [ ] All unit tests pass with >90% code coverage for new modules.
- [ ] Integration tests confirm that 100% of known harmful actions are rejected.
- [ ] Latency overhead is within acceptable bounds (<500ms).
- [ ] Metrics are visible in the Swarm Dashboard.