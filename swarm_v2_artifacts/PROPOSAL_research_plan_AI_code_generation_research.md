
# Build Proposal: research_plan_AI_code_generation_research.md

## Overview
I will construct a self-modifying code generation subsystem for the Swarm OS v12 architecture. This subsystem enables autonomous code synthesis, validation, and integration via a dedicated `CodeGen` agent and a core `CodeGenerationEngine` module. The implementation follows the approved research plan, extending the P2P mesh protocol with a `CODEGEN_TASK` message type and establishing a secure, validated pipeline for AI-driven code production.

## Proposed Files

### New Files:
1. `swarm_v2/core/code_generation_engine.py` — Core engine for AI-driven code synthesis, including:
   - `CodeGenerator` class (prompt engineering, context windowing, response parsing)
   - `SyntaxValidator` class (post-generation syntax validation for Python/Rust)
   - `ContextManager` class (sliding window of Swarm source code for LLM context)

2. `swarm_v2/agents/codegen_agent.py` — Specialized agent that subscribes to `CODEGEN_TASK` messages and orchestrates the code generation workflow.

### Modified Files:
1. `swarm_v2/core/agent_mesh.py` — Extend `MessageType` enum with `CODEGEN_TASK`, update `process_message` routing logic to dispatch to `CodeGen` agent.

## Execution Steps

1. **Create `swarm_v2/core/code_generation_engine.py`**:
   - Implement `CodeGenerator` with configurable LLM endpoint (Ollama default), prompt templates, and response parsing.
   - Implement `SyntaxValidator` using `ast.parse()` for Python and `subprocess` for Rust `rustfmt`/`rustc --check`.
   - Implement `ContextManager` with a deque-based sliding window of source file paths and content hashes.

2. **Modify `swarm_v2/core/agent_mesh.py`**:
   - Add `CODEGEN_TASK = "codegen_task"` to `MessageType` enum.
   - In `process_message`, add a routing branch: if `msg.type == MessageType.CODEGEN_TASK`, forward to `CodeGenAgent.handle_codegen_task(msg)`.

3. **Create `swarm_v2/agents/codegen_agent.py`**:
   - Implement `CodeGenAgent` inheriting from `BaseAgent`.
   - Implement `handle_codegen_task` method: parse spec → call `CodeGenerator.generate()` → validate with `SyntaxValidator` → return result via mesh response.
   - Add error handling for LLM failures, syntax errors, and timeout scenarios.

4. **Integration Test**:
   - Write unit tests for `CodeGenerator`, `SyntaxValidator`, and `ContextManager`.
   - Write integration test: send `CODEGEN_TASK` message from `SeekerAgent` → verify `CodeGenAgent` responds with valid code.