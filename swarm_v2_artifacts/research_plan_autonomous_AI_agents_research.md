
# Research Integration Plan: autonomous AI agents research

## Overview
This research highlights critical ethical risks (safety, privacy, security) in fully autonomous AI agents and identifies their core components: perception, reasoning, memory, and execution. This plan proposes integrating an **Ethical Guard & Audit Layer (EGAL)** into the QIAE Swarm. EGAL will act as a deterministic, non-bypassable middleware that intercepts agent actions at the execution boundary, enforcing predefined safety policies and logging all decisions for post-hoc analysis. This directly addresses the research's call for caution by embedding safety constraints directly into the agent lifecycle without sacrificing operational autonomy.

## Proposed Changes

### New Files to Create:
1.  **`swarm/core/ethical_guard.py`**: Core module implementing the `EthicalGuard` class. It will define a policy engine (e.g., allow/deny lists for tool calls, rate limits, data access scopes) and a mandatory audit hook. Every agent action (tool call, file write, network request) must pass through `guard.check(action)` before execution.
2.  **`swarm/core/audit_logger.py`**: Structured logging module for EGAL. Records agent ID, timestamp, action type, parameters, policy decision (ALLOW/DENY), and a cryptographic hash of the log entry for tamper-evident storage.
3.  **`swarm/config/ethical_policies.yaml`**: Configuration file defining default policies. Examples: `deny_tools: ["shell_exec", "email_send"]`, `rate_limit: 10 actions/min`, `data_scope: ["internal_db"]`.

### Existing Files to Modify:
1.  **`swarm/core/agent_executor.py`**: Inject the `EthicalGuard` check into the main execution loop. Before any tool call or external action, the executor must call `guard.check(action)`. If denied, the action is replaced with a safe error log entry.
2.  **`swarm/core/orchestrator.py`**: Add a startup routine to load `ethical_policies.yaml` and instantiate the global `EthicalGuard` singleton. Ensure all spawned agents inherit this guard reference.
3.  **`swarm/core/memory_system.py`**: Modify memory write operations to be intercepted by EGAL, preventing agents from storing or retrieving data outside their defined `data_scope`.

## Verification Plan
1.  **Unit Tests (`tests/test_ethical_guard.py`)**:
    - Test policy loading from YAML.
    - Test `guard.check()` returns `ActionDecision.DENY` for a blacklisted tool.
    - Test `guard.check()` returns `ActionDecision.ALLOW` for a permitted action.
    - Test rate limiter: 11th action within a 10-action window is denied.
2.  **Integration Tests (`tests/test_agent_ethics.py`)**:
    - Spawn a test agent with a restricted toolset. Verify that attempting to call a denied tool results in a logged denial and no actual execution.
    - Spawn an agent with a limited data scope. Verify that a memory write to an out-of-scope key is blocked.
3.  **Audit Log Verification**:
    - Execute a series of allowed and denied actions.
    - Parse the audit log file. Verify that each entry contains the required fields and that the cryptographic chain is intact (hash of entry N matches the `prev_hash` field of entry N+1).
4.  **Performance Benchmark**:
    - Measure latency overhead of `guard.check()` on a high-throughput agent loop. Target: < 1ms overhead per check. If exceeded, optimize the policy engine (e.g., use a Bloom filter for deny lists).