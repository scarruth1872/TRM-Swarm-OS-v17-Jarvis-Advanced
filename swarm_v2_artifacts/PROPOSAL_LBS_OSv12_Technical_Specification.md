
# Build Proposal: LBS_OSv12_Technical_Specification.md

## Overview
This proposal outlines the construction of **LoadBalancerSkill (LBS-OSv12)**, a production-grade, stateless, event-driven HTTP reverse proxy. The system will implement adaptive routing algorithms (Round Robin, Least Connections, Weighted, IP Hash, Random), a state-machine-based circuit breaker for fault isolation, token-bucket rate limiting for traffic shaping, active health probing with configurable thresholds, and native Prometheus/structured logging observability. The artifact will be a modular Python package with asynchronous I/O (asyncio/aiohttp) for high concurrency, targeting deployment in containerized environments (Docker/Kubernetes).

## Proposed Files
The following files will be created/modified to realize the LBS-OSv12 implementation:

1. **`load_balancer_skill/__init__.py`** - Package initializer, exposing public API (e.g., `LoadBalancer`, `BackendNode`, `RoutingStrategy`).
2. **`load_balancer_skill/core/balancer.py`** - Main `LoadBalancer` class: orchestrates request lifecycle, integrates rate limiter, node selector, circuit breaker, and forwarder.
3. **`load_balancer_skill/core/node_selector.py`** - `NodeSelector` with pluggable strategies: `RoundRobinStrategy`, `LeastConnectionsStrategy`, `WeightedStrategy`, `IPHashStrategy`, `RandomStrategy`.
4. **`load_balancer_skill/core/circuit_breaker.py`** - `CircuitBreaker` state machine (CLOSED, OPEN, HALF_OPEN) with configurable failure thresholds, timeout windows, and half-open retry logic.
5. **`load_balancer_skill/core/rate_limiter.py`** - `TokenBucketRateLimiter` implementing token-bucket algorithm with per-client or global limits.
6. **`load_balancer_skill/core/health_checker.py`** - `HealthChecker` with active probing (HTTP/TCP), configurable intervals, and state transition logic (Healthy ↔ Unhealthy).
7. **`load_balancer_skill/core/request_forwarder.py`** - `RequestForwarder` handling async HTTP forwarding, retry logic (max retries, backoff), and timeout management.
8. **`load_balancer_skill/core/backend_node.py`** - `BackendNode` data class: host, port, weight, current connections, health state, circuit breaker state.
9. **`load_balancer_skill/observability/metrics.py`** - Prometheus metric definitions (request count, latency histogram, circuit breaker state, rate limit hits, health status).
10. **`load_balancer_skill/observability/logger.py`** - Structured JSON logger (structlog) with correlation IDs, request tracing, and ELK-compatible output.
11. **`load_balancer_skill/config.py`** - Configuration schema (pydantic) for YAML/JSON/env-based settings: backend pool, routing strategy, rate limits, health check params, circuit breaker thresholds.
12. **`load_balancer_skill/exceptions.py`** - Custom exceptions: `NoHealthyBackends`, `CircuitBreakerOpen`, `RateLimitExceeded`, `BackendTimeout`.
13. **`tests/test_balancer.py`** - Unit tests for `LoadBalancer` integration.
14. **`tests/test_node_selector.py`** - Unit tests for all routing strategies.
15. **`tests/test_circuit_breaker.py`** - Unit tests for circuit breaker state transitions.
16. **`tests/test_rate_limiter.py`** - Unit tests for token-bucket algorithm.
17. **`tests/test_health_checker.py`** - Unit tests for health probing logic.
18. **`tests/test_request_forwarder.py`** - Unit tests for forwarding and retry logic.
19. **`setup.py`** - Package setup with dependencies (aiohttp, pydantic, prometheus_client, structlog, pytest).
20. **`Dockerfile`** - Multi-stage Docker build for production deployment.
21. **`docker-compose.yml`** - Local development environment with backend simulation.
22. **`README.md`** - Documentation: architecture overview, configuration guide, API reference, deployment instructions.
23. **`CHANGELOG.md`** - Version history and release notes.

## Execution Steps
The following steps will be executed in order to build and validate the LBS-OSv12 system:

1. **Initialize Project Structure**
   - Create directory tree: `load_balancer_skill/`, `load_balancer_skill/core/`, `load_balancer_skill/observability/`, `tests/`.
   - Initialize Git repository with `.gitignore` (Python, Docker, IDE files).

2. **Implement Core Data Models**
   - Create `load_balancer_skill/config.py` with pydantic models for all configuration parameters.
   - Create `load_balancer_skill/exceptions.py` with custom exception hierarchy.
   - Create `load_balancer_skill/core/backend_node.py` with `BackendNode` dataclass.

3. **Implement Rate Limiter**
   - Develop `TokenBucketRateLimiter` in `load_balancer_skill/core/rate_limiter.py`.
   - Implement token refill logic, burst support, and per-client isolation.
   - Write unit tests in `tests/test_rate_limiter.py`.

4. **Implement Health Checker**
   - Develop `HealthChecker` in `load_balancer_skill/core/health_checker.py`.
   - Implement async probing with configurable interval, timeout, and failure threshold.
   - Write unit tests in `tests/test_health_checker.py`.

5. **Implement Circuit Breaker**
   - Develop `CircuitBreaker` state machine in `load_balancer_skill/core/circuit_breaker.py`.
   - Implement state transitions (CLOSED → OPEN → HALF_OPEN → CLOSED) with configurable thresholds.
   - Write unit tests in `tests/test_circuit_breaker.py`.

6. **Implement Node Selector**
   - Develop `NodeSelector` with strategy pattern in `load_balancer_skill/core/node_selector.py`.
   - Implement all five routing strategies: RoundRobin, LeastConnections, Weighted, IPHash, Random.
   - Write unit tests in `tests/test_node_selector.py`.

7. **Implement Request Forwarder**
   - Develop `RequestForwarder` in `load_balancer_skill/core/request_forwarder.py`.
   - Implement async HTTP forwarding with retry logic (exponential backoff, max retries).
   - Write unit tests in `tests/test_request_forwarder.py`.

8. **Implement Main Balancer**
   - Develop `LoadBalancer` in `load_balancer_skill/core/balancer.py`.
   - Integrate all components: rate limiter → node selector → circuit breaker → forwarder.
   - Implement request lifecycle as per Mermaid diagram.
   - Write integration tests in `tests/test_balancer.py`.

9. **Implement Observability**
   - Create Prometheus metrics in `load_balancer_skill/observability/metrics.py`.
   - Create structured JSON logger in `load_balancer_skill/observability/logger.py`.
   - Integrate metrics and logging into main balancer.

10. **Create Package Configuration**
    - Write `setup.py` with dependencies and entry points.
    - Create `load_balancer_skill/__init__.py` with public API exports.

11. **Create Deployment Artifacts**
    - Write `Dockerfile` with multi-stage build (Python 3.11 slim base).
    - Write `docker-compose.yml` with load balancer service and simulated backends.

12. **Write Documentation**
    - Create `README.md` with architecture diagram, configuration examples, and API reference.
    - Create `CHANGELOG.md` with initial version (v0.1.0).

13. **Run Validation**
    - Execute all unit tests with `pytest -v --cov=load_balancer_skill`.
    - Perform integration test with Docker Compose.
    - Verify Prometheus metrics endpoint (`/metrics`) and health check endpoint (`/health`).