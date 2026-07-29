
# Build Proposal: system_metrics_skill.spec.md

## Overview
I will implement the `SystemMetricsSkill`, a high-performance, read-only telemetry module designed to provide real-time snapshots of host and container-level resource utilization. The system will be engineered as a decoupled "Skill" within the Swarm OS architecture, ensuring that metric collection does not interfere with the primary execution threads of the neural architecture. 

Key technical focuses include:
- **Hardware Abstraction**: Utilizing `psutil` for a unified API across Linux, macOS, and Windows.
- **Container Awareness**: Integration of `cgroup` parsing to ensure accurate reporting when running inside Docker/K8s environments.
- **Security Hardening**: Implementation of a strict read-only interface to prevent unauthorized kernel interactions.
- **Type Safety**: Full PEP 484 type hinting for integration with the OS's static analysis tools.

## Proposed Files
### 1. Core Implementation
- `skills/system_metrics/collector.py`: The primary engine responsible for interfacing with the OS kernel and `psutil`.
- `skills/system_metrics/models.py`: Pydantic models defining the `SystemSnapshot` schema to ensure consistent data contracts.
- `skills/system_metrics/exceptions.py`: Custom exception hierarchy (e.g., `MetricCollectionError`, `PermissionDeniedError`).

### 2. Integration & Interface
- `skills/system_metrics/interface.py`: The Swarm OS skill wrapper that exposes the `get_metrics()` method to the orchestrator.
- `skills/system_metrics/config.py`: Configuration for sampling intervals and filtered metric sets.

### 3. Validation & Testing
- `tests/skills/test_system_metrics.py`: Comprehensive test suite including mock kernel responses and edge-case validation for restricted environments.

## Execution Steps
1. **Environment Setup**: Install dependencies (`psutil`, `pydantic`) and verify `/proc` access permissions.
2. **Schema Definition**: Develop `models.py` to establish the data contract for CPU, Memory, Disk, and Network metrics.
3. **Collector Logic**: Implement the `MetricCollector` class in `collector.py` with specific logic for cgroup detection.
4. **Interface Wrapping**: Create the `SystemMetricsSkill` class in `interface.py` to map internal collector data to the Swarm OS skill API.
5. **Security Audit**: Validate that no write operations are possible and that the skill handles `PermissionError` gracefully.
6. **Verification**: Execute the test suite and generate a sample telemetry JSON output for validation.