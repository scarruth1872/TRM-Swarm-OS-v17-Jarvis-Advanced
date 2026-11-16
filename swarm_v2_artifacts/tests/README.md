# QKD Gateway Test Suite

## Coverage Matrix

| Invariant | Test Class | Verification Method |
|-----------|-----------|-------------------|
| INV-001 (Zero plain-text exposure) | TestSecureKey, TestPropertyBasedInvariants | repr/str guards, log scanning |
| INV-002 (Bounded buffers) | TestGatewayInitialization, TestKeyRotation | max capacity assertions, overflow tests |
| INV-003 (Revocation path) | TestKeyRevocation | explicit state transition checks |
| INV-004 (Socket cleanup) | TestCleanup, TestChaosEngineering | periodic task validation, FD exhaustion sim |

## Execution

```bash
# Full suite with coverage
pytest --cov=qkd_gateway --cov-report=term-missing

# Performance gates only
pytest -m benchmark

# Chaos engineering tests
pytest -m chaos

# Property-based invariants
pytest -m property
```

## CI Integration

```yaml
# GitHub Actions snippet
- name: Run QA Suite
  run: |
    pip install -r requirements-dev.txt
    pytest --junitxml=report.xml --cov=qkd_gateway --cov-report=xml
```

## Failure Injection Points

1. `kyber.encapsulate` - C-extension crash simulation
2. `socket.socket` - FD exhaustion
3. Event loop starvation - cleanup blocking detection
4. Partial batch failures - state corruption checks

## Performance Budgets

| Operation | Budget | Test |
|-----------|--------|------|
| Key generation | >1000 ops/sec | test_generation_throughput |
| Key retrieval | <5ms p99 | test_retrieval_latency |
| Cleanup 10k keys | <1s | test_cleanup_scalability |
| Memory ceiling | <50MB | test_memory_footprint |