
# Research Integration Plan: autonomous AI agents research

## Overview
This research reveals a dual-pathway opportunity. Source [1] (arXiv:2502.02649) provides a rigorous ethical framework arguing against fully autonomous AI agents, detailing risks in safety, privacy, and security. Source [2] (arXiv:2510.09244) offers a practical architecture for building LLM-powered agents. By combining these, we can implement a **Constrained Autonomy Governance Layer** for the QIAE Swarm that:
- Prevents runaway autonomous actions via policy-enforced human-in-the-loop (HITL) checkpoints.
- Implements bounded autonomy scopes (task-level, session-level, never full system-level).
- Provides transparent audit trails for every agent decision, satisfying ethical transparency requirements.
- Enables safe, compliant deployment in regulated environments (finance, healthcare, legal).

## Proposed Changes

### New Files to Create
1. **`qiae_swarm/governance/policy_engine.py`** — Core policy evaluation module. Defines `AutonomyPolicy` dataclass with fields: `max_autonomy_level` (enum: NONE, TASK, SESSION, FULL), `required_human_approval_actions` (list of action types), `audit_log_path`. Implements `evaluate_action(action: AgentAction, context: ExecutionContext) -> PolicyDecision` that checks action against policy before execution.
2. **`qiae_swarm/governance/audit_trail.py`** — Immutable audit logger. Records every agent action, policy decision, and human approval/rejection with cryptographic hashing for tamper-evidence. Uses SQLite with WAL mode for concurrent safety.
3. **`qiae_swarm/governance/human_interface.py`** — Async WebSocket-based approval queue. Exposes endpoints: `POST /approval/request`, `GET /approval/pending`, `POST /approval/respond`. Integrates with existing QIAE message bus.
4. **`qiae_swarm/agents/constrained_agent.py`** — Base class for all agents. Wraps existing agent logic with governance checks. Inherits from `BaseAgent` and overrides `execute()` to call `policy_engine.evaluate_action()` before proceeding. If `PolicyDecision.requires_approval`, it pauses execution and emits a human approval request.
5. **`qiae_swarm/config/governance_config.yaml`** — Default configuration file for autonomy policies per agent type. Example:
   ```yaml
   agents:
     research_seeker:
       max_autonomy_level: TASK
       required_human_approval_actions: ["deploy_code", "modify_system_config"]
     code_generator:
       max_autonomy_level: SESSION
       required_human_approval_actions: ["execute_network_commands"]
   ```

### Existing Files to Modify
1. **`qiae_swarm/orchestrator.py`** — Add governance initialization at startup. Load `governance_config.yaml`, instantiate `PolicyEngine`, inject into all agent constructors. Add shutdown hook to flush audit logs.
2. **`qiae_swarm/agents/__init__.py`** — Export `ConstrainedAgent` and update agent registry to use it as base class.
3. **`qiae_swarm/main.py`** — Add CLI flags: `--governance-policy <path>`, `--disable-governance` (for dev mode only, with warning).
4. **`requirements.txt`** — Add `websockets>=12.0`, `cryptography>=41.0.0` for audit hashing.

## Verification Plan

### Unit Tests (pytest)
1. **`tests/governance/test_policy_engine.py`**
   - `test_evaluate_action_allows_within_scope()` — Agent action within `max_autonomy_level` returns `ALLOW`.
   - `test_evaluate_action_blocks_out_of_scope()` — Action exceeding autonomy level returns `BLOCK`.
   - `test_evaluate_action_requires_approval()` — Action in `required_human_approval_actions` returns `PENDING_APPROVAL`.
   - `test_evaluate_action_unknown_action()` — Unknown action type defaults to `BLOCK` (fail-safe).
2. **`tests/governance/test_audit_trail.py`**
   - `test_audit_log_immutable()` — Verify hash chain breaks if log entry is tampered.
   - `test_audit_log_concurrent_writes()` — 10 concurrent writes all succeed without corruption.
3. **`tests/agents/test_constrained_agent.py`**
   - `test_agent_pauses_on_approval()` — Agent execution blocks until human responds.
   - `test_agent_resumes_after_approval()` — Agent continues execution after `POST /approval/respond`.
   - `test_agent_aborts_on_rejection()` — Agent raises `ActionRejectedError` and cleans up resources.

### Integration Tests
1. **`tests/integration/test_governance_workflow.py`**
   - Full end-to-end: Agent requests action → Policy engine flags for approval → Human approves via WebSocket → Agent executes → Audit log records all steps.
   - Negative case: Human rejects → Agent rolls back → Audit log shows rejection.

### Manual Verification Steps
1. Start QIAE Swarm with `--governance-policy tests/fixtures/test_policy.yaml`.
2. Trigger an agent action that requires approval (e.g., `research_seeker` trying to modify system config).
3. Verify WebSocket approval request appears in `human_interface` logs.
4. Send approval via `curl -X POST http://localhost:8080/approval/respond -d '{"request_id": "abc", "approved": true}'`.
5. Verify agent completes action and audit log contains `[ALLOW]` entry with human approval timestamp.
6. Repeat step 2–3, send rejection, verify agent raises error and audit log shows `[REJECT]`.

### Performance Benchmarks
- Measure latency overhead of governance checks: target <5ms per action evaluation.
- Measure audit log write throughput: target >1000 entries/second on SSD.
- Measure WebSocket approval queue latency: target <100ms from request to human notification.