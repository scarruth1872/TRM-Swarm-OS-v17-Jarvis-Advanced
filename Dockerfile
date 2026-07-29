# =============================================================================
# TRM Swarm OS — Multi-stage Dockerfile
# =============================================================================
# Stage 1: Build the Vite dashboard (Node.js)
# =============================================================================
FROM node:20-alpine AS dashboard-builder

WORKDIR /dashboard

# Install dependencies separately for layer caching
COPY dashboard/package.json dashboard/package-lock.json* ./
RUN npm ci

# Build the dashboard
COPY dashboard/ .
RUN npm run build

# =============================================================================
# Stage 2: Production image (Python runtime)
# =============================================================================
FROM python:3.11-slim

WORKDIR /app

# ------------------------------------------------------------------
# System dependencies: Node.js 20.x for any runtime use
# ------------------------------------------------------------------
RUN apt-get update && apt-get install -y curl && \
    curl -fsSL https://deb.nodesource.com/setup_20.x | bash - && \
    apt-get install -y nodejs && \
    rm -rf /var/lib/apt/lists/*

# ------------------------------------------------------------------
# Python dependencies: layer-cached on requirements.txt changes
# ------------------------------------------------------------------
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Additional runtime deps (not pinned in requirements.txt)
RUN pip install --no-cache-dir \
    uvicorn \
    fastapi \
    psutil \
    python-dotenv \
    ollama \
    aiohttp \
    httpx \
    pydantic \
    starlette

# ------------------------------------------------------------------
# Application code
# ------------------------------------------------------------------
COPY . .

# Copy pre-built dashboard artifacts from the builder stage
COPY --from=dashboard-builder /dashboard/dist /app/dashboard/dist

# ------------------------------------------------------------------
# Ports: API (8021), Dashboard (5183)
# ------------------------------------------------------------------
EXPOSE 8021
EXPOSE 5183

# ------------------------------------------------------------------
# Health check — verifies the API is responding
# ------------------------------------------------------------------
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD python -c "import urllib.request, sys; urllib.request.urlopen('http://localhost:8021/health/dashboard'); sys.exit(0)" || exit 1

# ------------------------------------------------------------------
# Entrypoint: the Swarm OS launcher orchestrates all services
# ------------------------------------------------------------------
CMD ["python", "launcher.py"]
