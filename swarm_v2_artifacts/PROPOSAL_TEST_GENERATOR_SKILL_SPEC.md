
# Build Proposal: TEST_GENERATOR_SKILL_SPEC.md

## Overview
I will implement the `TestGeneratorSkill`, a production-grade test synthesis engine. The system will ingest source code, perform deep static analysis via the `ast` module to map logical branches, and synthesize a comprehensive `pytest` suite. Key features include automatic mock generation for external dependencies and a sandbox validation loop to ensure the generated tests are syntactically correct and executable.

## Proposed Files
### 1. Core Engine (`swarm_os/skills/qa/test_generator/`)
- `__init__.py`: Package initialization.
- `models.py`: Strongly-typed dataclasses for `AnalysisResult`, `BranchPoint`, `TestCaseScenario`, and `SynthesisConfig`.
- `analyzer.py`: The Static Analysis Engine. Implements AST traversal to extract function signatures and map Control Flow Graphs (CFG).
- `synthesizer.py`: The Semantic Logic Generator. Transforms CFG maps into concrete test cases (Happy Path, Edge Case, Error State).
- `mock_factory.py`: Logic for identifying external imports and generating `unittest.mock` boilerplate.
- `sandbox.py`: The Validation Loop. Manages temporary directory creation, subprocess execution of `pytest`, and error-feedback loops.
- `engine.py`: The Orchestrator. Coordinates the pipeline from source code input to final artifact assembly.

### 2. Integration & Testing
- `tests/test_generator_skill_tests.py`: Meta-tests to verify the generator's ability to synthesize tests for various complexity levels.
- `examples/target_code.py`: A complex sample file used to demonstrate the engine's CFG mapping capabilities.

## Execution Steps
1. **Domain Modeling**: Define the `models.py` to establish the data contract between the analyzer and synthesizer.
2. **Static Analysis Implementation**: Build the `analyzer.py` to extract logical branch points and dependency graphs.
3. **Synthesis Logic**: Implement `synthesizer.py` and `mock_factory.py` to generate the actual Python test code.
4. **Sandbox Integration**: Develop `sandbox.py` to implement the closed-loop refinement process (Execute $\rightarrow$ Catch Error $\rightarrow$ Refine).
5. **Orchestration**: Integrate all components into `engine.py`.
6. **Validation**: Run the engine against `target_code.py` and verify the Predicted Coverage Report.