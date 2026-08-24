
# Build Proposal: VULN_SCAN_SPEC.md

## Overview
I will implement the `VulnScanOrchestrator`, a production-grade security scanning framework. The system will provide a unified interface to trigger multiple SAST/DAST tools concurrently, normalize their disparate output formats into a standardized `Vulnerability` schema, and apply a risk-scoring algorithm to prioritize findings. The architecture will emphasize extensibility, allowing new security engines to be added by simply inheriting from the `BaseEngine` interface.

## Proposed Files
### 📂 `core/`
- `models.py`: Definition of `Severity` Enum, `Vulnerability` dataclass, and `ScanReport` schemas.
- `orchestrator.py`: The main `VulnScanOrchestrator` class managing the `asyncio` execution pool and pipeline flow.
- `base_engine.py`: Abstract Base Class (ABC) defining the `BaseEngine` interface.
- `risk_engine.py`: Logic for weighted risk scoring and vulnerability prioritization.

### 📂 `engines/`
- `__init__.py`: Engine registry for dynamic loading.
- `semgrep_engine.py`: Implementation of the Semgrep SAST wrapper.
- `nuclei_engine.py`: Implementation of the Nuclei DAST wrapper.
- `custom_engine.py`: A template for user-defined security plugins.

### 📂 `utils/`
- `normalizer.py`: Adapter logic to map raw engine JSON/Text output to `Vulnerability` objects.
- `validator.py`: Target validation logic (URL/Path verification).

### 📂 `tests/`
- `test_orchestrator.py`: Integration tests for the full pipeline.
- `test_engines.py`: Unit tests for individual engine normalization.

## Execution Steps
1. **Foundation Layer**: Implement `models.py` and `base_engine.py` to establish the type system and interface contracts.
2. **Utility Layer**: Develop the `validator.py` and `normalizer.py` to handle input/output sanitization.
3. **Engine Implementation**: Build the `semgrep_engine.py` and `nuclei_engine.py` wrappers.
4. **Orchestration Logic**: Implement the `VulnScanOrchestrator` with `asyncio.gather` for concurrent execution and the deduplication pipeline.
5. **Risk Analysis**: Integrate the `RiskEngine` to process the normalized findings.
6. **Validation**: Execute the test suite to ensure robustness against malformed engine outputs and target failures.