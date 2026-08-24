#!/usr/bin/env bash
set -e

# ─── Swarm OS — One-Click Setup ───────────────────────────────
# This script installs all dependencies and starts the system.
# Usage: bash setup.sh  (Linux/macOS/Git Bash on Windows)

echo "╔══════════════════════════════════════════════════╗"
echo "║     Swarm OS — Automated Setup                   ║"
echo "╚══════════════════════════════════════════════════╝"

# ─── 1. Check Prerequisites ─────────────────────────────
echo ""
echo "[1/5] Checking prerequisites..."

command -v python3 >/dev/null 2>&1 || { echo "❌ python3 required"; exit 1; }
command -v node >/dev/null 2>&1 || { echo "❌ node required"; exit 1; }
command -v npm >/dev/null 2>&1 || { echo "❌ npm required"; exit 1; }

PYTHON=$(command -v python3)
NODE=$(command -v node)
echo "  ✅ Python: $($PYTHON --version)"
echo "  ✅ Node: $($NODE --version)"

# ─── 2. Create Environment ──────────────────────────────
echo ""
echo "[2/5] Setting up environment..."

if [ ! -f .env ]; then
  if [ -f .env.example ]; then
    cp .env.example .env
    echo "  ✅ Created .env from .env.example"
    echo "  ⚠️  Edit .env to add your API keys"
  else
    echo "  ⚠️  No .env.example found — create .env manually"
  fi
else
  echo "  ✅ .env already exists"
fi

# ─── 3. Python Dependencies ─────────────────────────────
echo ""
echo "[3/5] Installing Python dependencies..."
pip install --quiet --upgrade pip 2>/dev/null || true
pip install --quiet -r requirements.txt 2>/dev/null || {
  # Install core deps if no requirements.txt
  pip install --quiet fastapi uvicorn aiohttp pydantic 2>&1 | tail -1
}
echo "  ✅ Python deps installed"

# ─── 4. Node Dependencies ───────────────────────────────
echo ""
echo "[4/5] Installing Node dependencies..."
if [ -f package.json ]; then
  npm install --silent 2>&1 | tail -1
  echo "  ✅ TRM Node deps installed"
fi
if [ -f Jarvis-Advanced-main/package.json ]; then
  cd Jarvis-Advanced-main
  npm install --silent 2>&1 | tail -1
  cd ..
  echo "  ✅ Jarvis Node deps installed"
fi

# ─── 5. Start Services ──────────────────────────────────
echo ""
echo "[5/5] Starting Swarm OS..."

# Start Ollama if available
if command -v ollama >/dev/null 2>&1; then
  ollama serve &
  echo "  ✅ Ollama started"
  sleep 2
  ollama pull gemma4:12b 2>/dev/null || true
fi

# Start TRM
echo "  Starting TRM Swarm OS on port 8021..."
python -m uvicorn swarm_v2.app_v2:app --host 0.0.0.0 --port 8021 &
echo "  ✅ TRM starting"

# Start Jarvis
echo "  Starting Jarvis on port 3000..."
cd Jarvis-Advanced-main
npm run dev &
cd ..
echo "  ✅ Jarvis starting"

echo ""
echo "╔══════════════════════════════════════════════════╗"
echo "║     Swarm OS is starting up!                     ║"
echo "║                                                   ║"
echo "║     TRM API:    http://127.0.0.1:8021             ║"
echo "║     Jarvis:     http://127.0.0.1:3000             ║"
echo "║     API Docs:   http://127.0.0.1:8021/docs        ║"
echo "║                                                   ║"
echo "║     Documentation:                                 ║"
echo "║     - README.md                                    ║"
echo "║     - EXECUTIVE_SUMMARY.md                          ║"
echo "║     - SYSTEM_MAP.md                                ║"
echo "║     - QUICKSTART.md                                ║"
echo "╚══════════════════════════════════════════════════╝"
