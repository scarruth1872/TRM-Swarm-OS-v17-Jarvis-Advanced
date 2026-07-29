
# Research Integration Plan: neural network efficiency papers

## Overview
This research unlocks three distinct optimization pathways for the QIAE Swarm:

1.  **Source [1] (BUTTER-E Dataset):** Provides a high-fidelity empirical model of energy consumption vs. network size (20 sizes, 30,582 configs). Enables a **predictive energy profiler** that can estimate inference cost before deployment, allowing the Swarm's scheduler to route tasks to the most energy-efficient sub-model for a given accuracy requirement.
2.  **Source [2] (Binary Event-Driven SNNs):** Demonstrates extreme energy efficiency via spiking neural networks with binary activations. Enables a **hybrid inference pipeline** where standard ANN layers handle complex feature extraction, and SNN layers handle low-power temporal processing (e.g., motion detection, sensor fusion) on edge nodes.
3.  **Source [3] (Hierarchical Decomposition for LLMs):** Proposes decomposing large models into hierarchical sub-networks with fast routing. Enables a **dynamic model pruning and routing system** where the Swarm can decompose a monolithic LLM into specialized experts and route queries to only the relevant sub-network, drastically reducing per-query FLOPs.

**Primary Recommendation:** Integrate Source [3] (Hierarchical Decomposition) as the core architecture upgrade, with Source [1] as a supporting telemetry layer. This provides the highest impact on throughput and latency for the Swarm's LLM-based agents.

## Proposed Changes

### New Files to Create:
- `swarm/core/routing/hierarchical_router.py` — Implements the hierarchical decomposition logic: defines expert sub-networks, builds a routing decision tree, and dispatches tokens to the correct expert.
- `swarm/core/monitoring/energy_profiler.py` — Implements the BUTTER-E derived energy model: loads the pre-trained regression model, takes (dataset_size, param_count, architecture_type) as input, returns estimated mJ per inference.
- `swarm/configs/hierarchical_routing.yaml` — Configuration file for the router: defines number of experts, routing depth, fallback strategy, and energy budget thresholds.

### Existing Files to Modify:
- `swarm/core/scheduler.py` — Add a new scheduling policy `HierarchicalEnergyAware` that uses the energy profiler to select the cheapest expert sub-network that meets the accuracy threshold.
- `swarm/core/agent_orchestrator.py` — Integrate the hierarchical router into the agent dispatch pipeline: before spawning an agent, query the router to determine which expert model to load.
- `swarm/tests/test_scheduler.py` — Add test cases for the new scheduling policy.

## Verification Plan

1.  **Unit Test (Hierarchical Router):**
    - Mock 3 expert sub-networks with known accuracy scores.
    - Feed a query with a required accuracy threshold.
    - Assert that the router selects the smallest expert meeting the threshold.
    - Assert that queries below all thresholds fall back to the full model.

2.  **Unit Test (Energy Profiler):**
    - Load the BUTTER-E regression coefficients (hardcoded test stub).
    - Assert that `profiler.estimate_energy(param_count=10_000_000, dataset_size=100_000)` returns a value within ±5% of the published mean for that configuration.

3.  **Integration Test (Scheduler + Router):**
    - Deploy a test swarm with 2 expert models (small, medium) and 1 full model.
    - Submit 100 queries with varying accuracy requirements.
    - Assert that the scheduler never routes to the full model when a smaller expert suffices.
    - Assert that total energy consumption (sum of profiler estimates) is at least 40% lower than a baseline round-robin scheduler.

4.  **Performance Benchmark:**
    - Measure end-to-end latency for 1,000 queries with the new router enabled vs. disabled.
    - Target: p95 latency increase < 5% (routing overhead) while energy consumption decreases by > 35%.