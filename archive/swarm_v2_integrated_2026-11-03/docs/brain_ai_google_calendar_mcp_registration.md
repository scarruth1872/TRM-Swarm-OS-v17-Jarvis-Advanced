# Google Calendar MCP — Brain AI Integration Registration
## TRM Swarm OS v18 MCP Endpoint Onboarding

**MCP Server Name:** `google-calendar-brain-ai`
**Source Path:** `F:\casecad\Brain AI\google-calendar-mcp\`
**Protocol:** Model Context Protocol (MCP) via `@modelcontextprotocol/sdk v1.0.3`
**Language:** TypeScript (Node.js ES Module)
**Version:** 1.0.0

---

## What This MCP Server Does
Provides Google Calendar read/write access via OAuth2 GCP credentials to any
MCP-capable agent in the TRM Swarm OS v18. Enables:
- Scheduling awareness for the 16 Instella Sparse Agent Teams
- Calendar-driven task dispatching in the microkernel
- PHOENIX-Protocol timeline integration for health/wellness event tracking
- Jarvis Advanced (Port 4000) natural language calendar queries

---

## Start Commands

### Build first (if not built):
```powershell
cd "F:\casecad\Brain AI\google-calendar-mcp"
npm install
npm run build
```

### Start MCP server:
```powershell
node "F:\casecad\Brain AI\google-calendar-mcp\build\index.js"
```

### First-time OAuth auth flow:
```powershell
node "F:\casecad\Brain AI\google-calendar-mcp\build\auth-server.js"
```

---

## Required Environment Variables

Add to `F:\Development sites\TRM-Swarm-OS-v2\.env` or `Jarvis-Advanced-main\.env`:

```env
GOOGLE_CALENDAR_MCP_PATH=F:\casecad\Brain AI\google-calendar-mcp\build\index.js
GOOGLE_OAUTH_KEYS_PATH=F:\casecad\Brain AI\google-calendar-mcp\gcp-oauth.keys.json
```

---

## Registration in Jarvis Advanced (Port 4000)

Add to Jarvis `.env`:
```env
MCP_GOOGLE_CALENDAR=node "F:\casecad\Brain AI\google-calendar-mcp\build\index.js"
```

The Jarvis llama.cpp server will route calendar tool calls via this MCP endpoint,
enabling the Brain AI system to schedule tasks, set reminders, and read upcoming
events through natural language queries on Port 4000.

---

## TRM Swarm OS v18 Integration Notes

- The MCP server uses OAuth2 GCP credentials stored in `gcp-oauth.keys.json`
- Credentials are already present — ensure they are valid/refreshed via auth-server
- Port recommendation: run on `localhost:7001` (not yet exposed in docker-compose)
- To expose in Docker, add a service to `docker-compose.yml`:

```yaml
  google-calendar-mcp:
    image: node:20-alpine
    container_name: google-calendar-mcp
    ports:
      - "7001:7001"
    working_dir: /app
    volumes:
      - F:/casecad/Brain AI/google-calendar-mcp:/app
    environment:
      - NODE_ENV=production
    command: sh -c "npm install && node build/index.js"
    networks:
      - swarm-net
```

---

*Registered by TRM Swarm OS v18 Architecture Council — August 1, 2026*
*Brain AI Integration v18.7.0-BRAIN-AI*
