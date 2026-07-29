# Swarm OS - Advanced Health Monitoring Microservice

This document outlines the architecture and implementation of a robust, scalable, and observable health monitoring microservice designed within the Swarm OS framework. This service demonstrates core architectural principles for distributed systems, including modularity, robustness, scalability, and observability.

## 🏛️ Architecture Overview

The service is a lightweight, stateless FastAPI application designed for containerization and deployment in a cloud-native environment. It exposes a single `/health` endpoint that performs an internal status check and can optionally simulate external dependency checks.

```mermaid
graph TD
    A[Client Request] --> B(Load Balancer / API Gateway)
    B --> C{Health Monitoring Service}
    C --> D[app/main.py - FastAPI App]
    D -- Dependency Injection --> E[app/services/health_service.py - Health Logic]
    E -- (Optional) Simulate External Check --> F[External Dependency (e.g., DB, Cache, Other Service)]
    D -- Structured Logging --> G[app/utils/logging_config.py]
    D -- Configuration --> H[app/config.py - Pydantic Settings]
    C -- Containerization --> I[Dockerfile]
    C -- Deployment --> J[Kubernetes / Swarm]
    J -- Metrics/Logs --> K[Monitoring System (Prometheus, ELK)]

    subgraph Health Monitoring Service
        D
        E
        G
        H
    end
```

### Key Architectural Principles Applied:

1.  **Modularity**:
    *   Separation of concerns: `main.py` handles routing, `health_service.py` encapsulates business logic, `config.py` manages settings, and `logging_config.py` centralizes logging setup.
    *   Dependency Injection: `HealthService` is injected into the FastAPI route, promoting testability and loose coupling.
2.  **Robustness**:
    *   Asynchronous Operations: Utilizes `async/await` for non-blocking I/O, improving concurrency.
    *   Error Handling: Explicit exception handling within the health check logic.
    *   Configuration Validation: Pydantic is used for robust environment variable parsing and validation.
3.  **Scalability**:
    *   Stateless Design: The service does not maintain session state, allowing for easy horizontal scaling.
    *   Containerization: Packaged in a Docker image for consistent deployment across environments.
    *   Asynchronous Framework: FastAPI and Uvicorn provide high performance for concurrent requests.
4.  **Observability**:
    *   Structured Logging: JSON-formatted logs with contextual information (request ID, service name, timestamp) for easy ingestion by log aggregation systems.
    *   Health Endpoint: Provides a standard mechanism for liveness and readiness probes in orchestration systems.
    *   Metrics Hooks (Implicit): The logging structure can be extended to emit metrics for request duration, error rates, etc.

## 🛠️ Setup and Execution

### Prerequisites

*   Python 3.9+
*   Docker (optional, for containerized execution)

### Local Development

1.  **Clone the repository (conceptual):**
    ```bash
    # git clone <repo-url>
    # cd health-monitor-service
    ```
2.  **Create a virtual environment:**
    ```bash
    python -m venv .venv
    source .venv/bin/activate # On Windows: .venv\Scripts\activate
    ```
3.  **Install dependencies:**
    ```bash
    pip install -r requirements.txt
    ```
4.  **Run the service:**
    ```bash
    uvicorn app.main:app --host 0.0.0.0 --port 8000 --reload
    ```
    The service will be available at `http://localhost:8000`.

### Containerized Execution (Docker)

1.  **Build the Docker image:**
    ```bash
    docker build -t swarm-os-health-monitor .
    ```
2.  **Run the Docker container:**
    ```bash
    docker run -p 8000:8000 --name health-monitor swarm-os-health-monitor
    ```
    The service will be available at `http://localhost:8000`.

    To simulate a failure:
    ```bash
    docker run -p 8000:8000 -e SIMULATE_FAILURE=true --name health-monitor-fail swarm-os-health-monitor
    ```

## 🚀 Usage

### Health Check Endpoint

*   **Endpoint**: `/health`
*   **Method**: `GET`
*   **Description**: Returns the current health status of the service.

**Example Request (Success):**

```bash
curl http://localhost:8000/health
```

**Example Response (Success):**

```json
{
  "status": "healthy",
  "service_name": "SwarmOS Health Monitor",
  "version": "1.0.0",
  "timestamp": "2026-07-16T17:14:30.123456Z",
  "details": {
    "internal_check": "OK"
  }
}
```

**Example Request (Simulated Failure):**

Run with `SIMULATE_FAILURE=true` environment variable.

```bash
curl http://localhost:8000/health
```

**Example Response (Simulated Failure):**

```json
{
  "status": "unhealthy",
  "service_name": "SwarmOS Health Monitor",
  "version": "1.0.0",
  "timestamp": "2026-07-16T17:14:30.123456Z",
  "details": {
    "internal_check": "FAILED",
    "error": "Simulated external dependency failure"
  }
}
```

## ⚙️ Configuration

The service is configured via environment variables, managed by Pydantic settings.

| Environment Variable | Default Value | Description                                          |
| :------------------- | :------------ | :--------------------------------------------------- |
| `SERVICE_NAME`       | `SwarmOS Health Monitor` | Name of the service.                                 |
| `SERVICE_VERSION`    | `1.0.0`       | Version of the service.                              |
| `LOG_LEVEL`          | `INFO`        | Minimum logging level (DEBUG, INFO, WARNING, ERROR). |
| `SIMULATE_FAILURE`   | `false`       | If `true`, the health check will intentionally fail. |

## 📝 Design Rationale

*   **FastAPI**: Chosen for its high performance, automatic OpenAPI documentation, and strong type hints, which enhance code quality and maintainability.
*   **Pydantic**: Provides robust data validation and settings management, ensuring configuration integrity.
*   **Structured Logging**: Essential for distributed systems. JSON logs are easily parsed and queried by centralized logging solutions (e.g., ELK Stack, Splunk).
*   **Dependency Injection**: Facilitates testing and modularity. The `HealthService` can be easily mocked for unit tests.
*   **Asynchronous Programming**: Improves resource utilization and responsiveness, crucial for I/O-bound microservices.
*   **Minimal Dependencies**: Keeps the service lightweight and reduces the attack surface.
*   **Clear Folder Structure**: Enhances project navigability and scalability for larger codebases.