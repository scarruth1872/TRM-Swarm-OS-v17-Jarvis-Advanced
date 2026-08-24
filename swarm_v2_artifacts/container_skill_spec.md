# Specification: Containerized Skill Architecture (CSA)
**Version**: 1.0.0
**Status**: Production-Ready
**Owner**: Devo (Lead Developer)

## 1. Overview
The Containerized Skill Architecture (CSA) allows Swarm OS to encapsulate specific capabilities (e.g., specialized compilers, data analysis tools, legacy binaries) into isolated Docker containers. This ensures that the core OS remains lean while providing an extensible plugin system.

## 2. Lifecycle Requirements
Every skill must implement the following lifecycle states:
- `PROVISIONING`: Pulling images and configuring volumes.
- `ACTIVE`: Container is running and listening for commands.
- `EXECUTING`: Processing a specific request.
- `TERMINATED`: Container stopped and resources reclaimed.

## 3. Interface Contract
Skills must communicate via a standardized interface:
- **Input**: JSON payload via `stdin` or a REST endpoint.
- **Output**: Structured JSON via `stdout` or a response body.
- **Health**: A `/health` or `SIGUSR1` signal response.

## 4. Resource Constraints
To maintain system stability, the following defaults are enforced:
- **Memory Limit**: 512MB (configurable per skill).
- **CPU Shares**: 0.5 vCPU (configurable).
- **Network**: Isolated bridge network with controlled egress.
- **Timeout**: Hard kill after 300 seconds of inactivity.