# Swarm OS Test Suites

## Overview
This directory contains the comprehensive test suite for Swarm OS components, organized into two levels:

- **L0 (Unit Tests)**: Property-based tests for pure functions
- **L1 (Component Tests)**: Contract tests for public APIs

## Directory Structure
```
test_suites/
├── unit/                    # L0 Property-based tests
│   ├── test_crypto_properties.py
│   ├── test_state_transitions.py
│   └── test_serialization_properties.py
├── component/               # L1 Contract tests
│   ├── test_node_contract.py
│   ├── test_router_contract.py
│   ├── test_scheduler_contract.py
│   └── test_store_contract.py
├── mutation_config.toml     # Mutation testing configuration
├── run_mutation_tests.sh    # Mutation testing runner
└── README.md
```

## Running Tests

### L0 Unit Tests
```bash
pytest test_suites/unit/ -v
```

### L1 Component Tests
```bash
pytest test_suites/component/ -v
```

### All Tests
```bash
pytest test_suites/ -v
```

### Mutation Testing
```bash
chmod +x test_suites/run_mutation_tests.sh
./test_suites/run_mutation_tests.sh
```

## Test Coverage Strategy

### L0 Property-Based Tests
- **Crypto**: Hash determinism, avalanche effect, signature roundtrip, key derivation
- **State Machine**: Valid/invalid transitions, terminal state absorption, sequence invariants
- **Serialization**: Roundtrip encoding, JSON schema validation, malformed input rejection

### L1 Contract Tests
- **Node**: Lifecycle management, message signing/verification, state reporting
- **Router**: Node registration, message routing, broadcast, health checks
- **Scheduler**: Task submission, execution, priority ordering, lifecycle
- **State Store**: CRUD operations, consistency, transactions, health checks

## Quality Metrics
- **Mutation Score Threshold**: 70%
- **Coverage Threshold**: 80%
- **Property Test Examples**: 100 per test (configurable)

## Dependencies
- pytest >= 7.0
- hypothesis >= 6.0
- mutmut >= 3.0
- pytest-cov >= 4.0