
# Build Proposal: FUZZ_TEST_SKILL_SPEC.md

## Overview
I will implement the `FuzzTestSkill` as a modular security intelligence component. The system will feature an evolutionary feedback loop that utilizes a seed corpus, a multi-strategy mutation engine, and a crash triager to identify unique vulnerabilities in target binaries. The implementation will prioritize high-throughput execution and precise crash deduplication using stack trace hashing.

## Proposed Files
### 📂 `swarm_os/security/fuzzing/`
- `__init__.py`: Package initialization.
- `core.py`: The main `FuzzTestSkill` orchestrator coordinating the feedback loop.
- `mutation_engine.py`: Implementation of `MutationEngine` with `BIT_FLIP`, `ARITHMETIC`, and `HAVOC` strategies.
- `orchestrator.py`: The `ExecutionOrchestrator` handling process lifecycle, timeouts, and instrumentation.
- `triager.py`: The `CrashTriager` implementing SHA-256 deduplication of crash signatures.
- `models.py`: Type definitions, Enums for mutation strategies, and DataClasses for `FuzzResult` and `Seed`.
- `utils.py`: Low-level byte manipulation and filesystem helpers for corpus management.

### 📂 `tests/security/fuzzing/`
- `test_mutation.py`: Unit tests for mutation strategy correctness.
- `test_orchestrator.py`: Integration tests for process isolation and timeout handling.
- `test_triager.py`: Validation of crash deduplication logic.

## Execution Steps
1. **Foundation Phase**: Define `models.py` and `utils.py` to establish the type system and byte-level primitives.
2. **Mutation Phase**: Implement `mutation_engine.py` ensuring all strategies (`BIT_FLIP`, `ARITHMETIC`, `HAVOC`) are functionally isolated.
3. **Execution Phase**: Develop `orchestrator.py` with robust subprocess handling and environment injection for LLVM sanitizers.
4. **Triage Phase**: Implement `triager.py` to handle the ingestion of `stderr` and the generation of unique crash hashes.
5. **Integration Phase**: Assemble the `FuzzTestSkill` in `core.py` to link the corpus $\rightarrow$ mutation $\rightarrow$ execution $\rightarrow$ triage loop.
6. **Verification Phase**: Execute the test suite to ensure stability and coverage-guided behavior.