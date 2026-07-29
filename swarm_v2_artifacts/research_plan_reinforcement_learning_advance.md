
# Research Integration Plan: reinforcement learning advances

## Overview
This research enables two synergistic upgrades to the QIAE Swarm's autonomous learning pipeline:
1.  **Foundational Upgrade (arXiv:2412.05265):** The comprehensive overview provides modern formulations of policy gradients, value function approximation, and exploration strategies (e.g., intrinsic motivation, count-based exploration). This allows us to refactor the core `Agent` decision loop from a static heuristic to an adaptive, online RL system.
2.  **Feature Addition (arXiv:2408.11943):** The review of Preference-based RL (PbRL) introduces a mechanism to align agent behavior with human or system-level preferences without hand-crafted reward functions. This enables a new "Preference Alignment Module" where the Swarm can learn from comparative feedback (e.g., "trajectory A is better than trajectory B") to optimize for nuanced objectives like energy efficiency vs. throughput.

## Proposed Changes

### Files to be Modified:
1.  `src/core/agent/decision_engine.py` - Refactor the action selection logic to use a PPO-based policy network instead of the current rule-based system.
2.  `src/core/agent/reward_calculator.py` - Add a `PreferenceReward` class that computes a reward signal from a preference model, replacing or augmenting the scalar reward function.
3.  `src/swarm/orchestrator.py` - Add a new `preference_learning_cycle` method to manage the collection of pairwise trajectory comparisons from the system monitor.

### Files to be Created:
1.  `src/core/agent/preference_model.py` - A neural network (transformer or MLP) that takes two trajectory embeddings and outputs a probability that trajectory A is preferred over trajectory B.
2.  `src/core/agent/trajectory_buffer.py` - A specialized replay buffer that stores full trajectories (state-action-reward sequences) along with optional preference labels.
3.  `config/preference_rl_config.yaml` - Configuration file for PbRL hyperparameters (batch size, preference sampling ratio, model architecture).
4.  `tests/test_preference_model.py` - Unit tests for the preference model's forward pass and loss function.

## Verification Plan
1.  **Unit Tests:** Validate the `PreferenceModel` output is a valid probability distribution. Validate the `TrajectoryBuffer` correctly stores and retrieves trajectories.
2.  **Integration Test:** Run a simplified grid-world environment. Inject a known preference (e.g., "avoid red cells"). Verify the agent's policy shifts to avoid red cells after 1000 preference queries.
3.  **Regression Test:** Compare the new PbRL-enhanced agent against the baseline agent on the standard benchmark suite. Ensure the new system does not degrade performance on the original scalar reward tasks.
4.  **System Monitor:** Add telemetry to track the preference model's accuracy over time and the number of preference queries issued per epoch.