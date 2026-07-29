#!/bin/bash
set -euo pipefail

# Swarm OS Phase 1 Initialization Script
# Bootstraps Security, Observability, and Configuration services.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INFRA_DIR="$(dirname "$SCRIPT_DIR")"

echo "[INFO] Starting Phase 1: Core Infrastructure & Observability Overhaul"

# 1. Generate self-signed certificates for mTLS (development only)
echo "[INFO] Generating development certificates..."
mkdir -p "$INFRA_DIR/security/certs"

openssl req -x509 -nodes -days 365 -newkey rsa:4096 \
  -keyout "$INFRA_DIR/security/certs/ca-key.pem" \
  -out "$INFRA_DIR/security/certs/ca-cert.pem" \
  -subj "/CN=SwarmOS-CA" 2>/dev/null

openssl req -newkey rsa:4096 -nodes \
  -keyout "$INFRA_DIR/security/certs/server-key.pem" \
  -out "$INFRA_DIR/security/certs/server-req.pem" \
  -subj "/CN=swarm-os.local" 2>/dev/null

openssl x509 -req -days 365 \
  -in "$INFRA_DIR/security/certs/server-req.pem" \
  -CA "$INFRA_DIR/security/certs/ca-cert.pem" \
  -CAkey "$INFRA_DIR/security/certs/ca-key.pem" \
  -CAcreateserial \
  -out "$INFRA_DIR/security/certs/server-cert.pem" 2>/dev/null

# 2. Deploy Phase 1 services
echo "[INFO] Deploying Phase 1 services..."
docker-compose -f "$INFRA_DIR/deployment/docker-compose.phase1.yaml" up -d

# 3. Initialize Vault with secrets
echo "[INFO] Initializing Vault..."
sleep 10 # Wait for Vault to be ready
export VAULT_ADDR='http://localhost:8200'
export VAULT_TOKEN='swarm-root-token'

vault secrets enable -path=swarm kv-v2 2>/dev/null || true
vault kv put swarm/config/global \
  environment=development \
  log_level=debug \
  max_agents=100

# 4. Seed etcd with initial configuration
echo "[INFO] Seeding etcd..."
etcdctl put /swarm/config/database/url "postgresql://swarm:password@db:5432/swarm_os"
etcdctl put /swarm/config/redis/address "redis://redis:6379"
etcdctl put /swarm/config/rabbitmq/uri "amqp://swarm:password@rabbitmq:5672"

echo "[SUCCESS] Phase 1 initialization complete."
echo "[INFO] Services available:"
echo " - Vault: http://localhost:8200"
echo " - Jaeger: http://localhost:16686"
echo " - Prometheus: http://localhost:9090"
echo " - etcd: http://localhost:2379"
