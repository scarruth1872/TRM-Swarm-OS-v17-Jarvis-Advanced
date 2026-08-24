# 🧪 Skill Specification: Test Generator (v1.0.0)

## 1. Overview
The `TestGenerator` skill is an autonomous agent capable of analyzing source code and generating comprehensive, production-ready test suites. It focuses on high branch coverage, edge-case discovery, and the elimination of "happy path" bias.

## 2. Technical Requirements
- **Input**: Source file path, Target Framework (PyTest, Unittest, Jest, etc.), Coverage Goal (%).
- **Output**: A standalone test file containing executable test cases.
- **Dependencies**: `ast`, `inspect`, `pytest`, `hypothesis` (for property-based testing).

## 3. Functional Capabilities
### 3.1 AST-Based Branch Mapping
The system must traverse the AST to identify:
- All conditional branches (`if/elif/else`).
- Exception handling blocks (`try/except`).
- Loop boundaries and termination conditions.
- Type-hinted constraints.

### 3.2 Test Case Generation Strategies
| Strategy | Trigger | Objective |
| :--- | :--- | :--- |
| **Boundary Analysis** | Numeric/Sequence inputs | Test min/max/null/empty values. |
| **Equivalence Partitioning** | Complex conditionals | Group inputs to reduce redundancy while maintaining coverage. |
| **Property-Based** | Generic types/Complex logic | Use `Hypothesis` to find falsifying examples. |
| **Mocking** | External API/DB calls | Isolate the Unit Under Test (UUT). |

## 4. Quality Constraints
- **No Hallucinations**: Tests must be based on actual code paths, not guessed functionality.
- **Idempotency**: Generating tests for the same code version must yield consistent results.
- **Zero-Regression**: Generated tests must not introduce side effects to the production environment.

## 5. Success Metrics
- **Branch Coverage**: $\ge 90\%$ of logical paths exercised.
- **Mutation Score**: High resilience against synthetic bug injection.
- **Execution Time**: Test generation overhead should be $< 10\%$ of total build time.