# TRM Swarm OS v17.0 & Jarvis Advanced — Complete Self-Hosting Guide

## Executive Summary
This guide details the **100% self-hosted deployment** of the combined platform (**TRM Swarm OS v17.0 & Jarvis Advanced**). All processes run locally on your own hardware without relying on third-party cloud data hosting.

---

## 🏗️ Architecture & Service Ports

| Service Name | Port | Executable / Protocol | Purpose |
| :--- | :--- | :--- | :--- |
| **Swarm OS Kernel Core** | `8021` | `uvicorn swarm_v2.app_v2:app` | FastAPI multi-agent mesh, SKG, formal verification, and REST APIs. |
| **Jarvis Mind Gateway** | `4000` | `npx tsx server.ts` | Neural chat gateway, voice STT/TTS, Telegram polling daemon, and media uploads. |
| **Dashboard Studio UI** | `5183` | `npm run dev` (Vite) / Nginx | React 19 control center with 20 Agent Widgets & Mobile Sync viewport. |

---

## 🚀 Self-Hosting Launch Options

### Option 1: One-Line Local Process Launch (Default / Development)
Run each command in a separate PowerShell terminal:

```powershell
# 1. Launch Swarm OS API Core (Port 8021)
$env:PYTHONPATH='.'; uvicorn swarm_v2.app_v2:app --host 0.0.0.0 --port 8021

# 2. Launch Jarvis Mind Gateway (Port 4000)
cd Jarvis-Advanced-main/Jarvis-Advanced-main
npx tsx server.ts

# 3. Launch Swarm Dashboard Studio (Port 5183)
cd dashboard
npm run dev
```

---

### Option 2: Docker Compose Container Stack (Production)
Your system has **Docker Compose v2.40.3** installed and ready.

```bash
# Start all containers in background
docker compose -f docker-compose.live.yml up -d

# View live container logs
docker compose -f docker-compose.live.yml logs -f

# Stop containers
docker compose -f docker-compose.live.yml down
```

---

### Option 3: PM2 Process Manager
```bash
# Install PM2 globally (if needed)
npm install -g pm2

# Start all ecosystem processes
pm2 start ecosystem.config.js

# Monitor live status
pm2 status
pm2 logs
```

---

## 🧪 Verifying Your Self-Hosted Deployment

To run an automated 10-point health check verifying all core services, run:

```powershell
$env:PYTHONPATH='.'; python live_testing_suite.py
```

### Verified Endpoints:
1. `GET http://127.0.0.1:8021/health` (Swarm Core)
2. `GET http://127.0.0.1:4000/health` (Jarvis Gateway)
3. `GET http://127.0.0.1:5183` (Dashboard Studio UI)
4. `POST http://127.0.0.1:4000/api/chat` (Jarvis Neural Chat)
5. `POST http://127.0.0.1:8021/swarm/chat` (Swarm Specialist Mesh)
6. `POST http://127.0.0.1:8021/swarm/symbiosis` (Symbiosis Engine)
7. `GET http://127.0.0.1:4000/api/telegram/status` (Telegram Uplink Daemon)
8. `GET http://127.0.0.1:8021/kanban/board` (Kanban State Machine)
9. `GET http://127.0.0.1:8021/api/mobile-sync/status` (Mobile Companion Sync)
10. `GET http://127.0.0.1:8021/swarm/microkernel/status` (Microkernel Process Table)
