
# Research Integration Plan: AI code generation research

## Overview
This plan integrates two key 2024 research findings into the QIAE Swarm's code generation pipeline:
1.  **"Artificial-Intelligence Generated Code Considered Harmful" (Sept 2024):** Provides a roadmap for secure, high-quality code generation. This will be used to implement a **Security & Quality Validation Gate** that scans generated code for common vulnerabilities (e.g., injection flaws, logic errors) and enforces coding standards before output.
2.  **"Optimizing AI-Assisted Code Generation" (arXiv:2412.10953):** Details methods to improve the efficiency and accuracy of AI code generation. This will be used to implement a **Prompt Optimization Layer** that refines user requests (e.g., adding context, specifying constraints, breaking down complex tasks) before they are sent to the LLM, reducing token waste and improving first-pass success rate.

**Enables:** A new `Secure & Optimized Code Synthesis` feature that produces safer, more reliable, and more efficient code artifacts.

## Proposed Changes

### New Files to Create:
1.  **`swarm/codegen/validator.py`**: Implements the Security & Quality Validation Gate. Contains functions to parse generated code (AST), check against a rule set (e.g., OWASP Top 10 patterns, type safety, unused variable detection), and return a validation report.
2.  **`swarm/codegen/prompt_optimizer.py`**: Implements the Prompt Optimization Layer. Contains functions to analyze user intent, add system-level constraints (e.g., "use async", "prefer Rust"), decompose complex tasks into sub-steps, and format the final prompt for the LLM.
3.  **`swarm/codegen/config.py`**: Configuration file for the new module, defining rule severity levels, optimization strategies, and API endpoints.
4.  **`tests/test_codegen_validator.py`**: Unit tests for the validator.
5.  **`tests/test_codegen_prompt_optimizer.py`**: Unit tests for the prompt optimizer.

### Existing Files to Modify:
1.  **`swarm/codegen/engine.py`**: The main code generation orchestrator. Modify to:
    - Import and call `prompt_optimizer.optimize()` before sending the prompt to the LLM.
    - Import and call `validator.validate()` on the LLM's output.
    - If validation fails, either request a regeneration from the LLM with specific error feedback or flag the output for human review.
2.  **`swarm/config.yaml`**: Add a new section `codegen_enhancements` to toggle the validator and optimizer on/off and set their parameters.
3.  **`swarm/main.py`**: Add a new CLI flag `--secure-codegen` to enable the full pipeline.

## Verification Plan
1.  **Unit Tests:**
    - `test_codegen_validator.py`: Test that the validator correctly identifies a known insecure code pattern (e.g., SQL injection) and passes a secure version. Test edge cases (empty code, syntax errors).
    - `test_codegen_prompt_optimizer.py`: Test that the optimizer correctly expands a vague prompt ("write a function") into a detailed one ("write an async Python function that reads a CSV file and returns a list of dictionaries, with error handling for file not found").
2.  **Integration Test:**
    - Run the full `engine.py` pipeline with `--secure-codegen` enabled. Provide a prompt known to historically generate insecure code. Assert that the final output passes the validator.
3.  **Performance Benchmark:**
    - Compare the token usage and generation time of the optimized pipeline vs. the baseline pipeline for 100 standard code generation requests. Target: 15% reduction in token usage and 10% reduction in regeneration requests.
4.  **A/B Test (Staging):**
    - Deploy the new pipeline to a staging environment. Route 50% of code generation requests through the new pipeline and 50% through the old. Monitor for a reduction in reported code defects over a 48-hour period.