# Platform Integration Plan — Wiring All Dashboard Components

**Date:** July 16, 2026  
**Goal:** Every Jarvis dashboard component connected to a live backend route.

---

## Status Summary

| Component | Size | Status | Action |
|---|---|---|---|
| **NoosphericRecalibration** | 41KB | ✅ Already wired | No action needed |
| **WiFiVisionIntelligence** | 57KB | ✅ Already wired | No action needed |
| **WorkspaceNexus** | 161KB | 🟡 Needs OAuth | See Section A |
| **BiomimeticLab** | 86KB | ✅ Wired (7 API calls) | Verify backend exists |
| **JobTracker** | 34KB | ✅ Wired (5 API calls) | Verify backend exists |
| **LocalModelSync** | 71KB | 🟡 Wired) | Verify Ollama API |
| **AscilineEngine** | 21KB | ✅ Wired | Verify backend exists |
| **OkfHub** | 21KB | ✅ Wired | Verify backend exists |
| **ResonanceResearchHub** | 51KB | ✅ Wired | Verify backend exists |
| **CognitiveMemoryGraph** | 27KB | ❌ No backend | See Section B |
| **ComputerControlTerminal** | 11KB | ❌ No backend | See Section C |
| **SystemTelemetry** | 7KB | ❌ No backend | See Section D |
| **TernarySynapseEngine** | 76KB | ❌ No backend | See Section E |
| **RoboticsLab** | 67KB | ❌ No backend | See Section F |
| **GlobalOfferLanding** | 49KB | ❌ Static page | See Section G |
| **JarvisAvatar** | 16KB | ❌ Standalone | See Section H |

---

## Section A: WorkspaceNexus — Google OAuth

**Status:** 26 Google API calls fully coded. Needs `accessToken` prop from parent (App.tsx).

**What's needed:**
1. Create an OAuth flow endpoint on Jarvis server.ts
2. The WorkspaceNexus expects `accessToken: string | null` and `onAuthorize: () => void` props
3. These are passed from App.tsx but the OAuth flow is never triggered

**Implementation:**
```typescript
// New route in server.ts:
app.get("/api/auth/google/url", async (req, res) => {
  const url = `https://accounts.google.com/o/oauth2/v2/auth?client_id=${CLIENT_ID}&redirect_uri=${REDIRECT_URI}&scope=https://www.googleapis.com/auth/gmail.readonly%20https://www.googleapis.com/auth/calendar.readonly%20https://www.googleapis.com/auth/drive.readonly&access_type=offline&response_type=code`;
  res.json({ url });
});

app.post("/api/auth/google/token", async (req, res) => {
  const { code } = req.body;
  // Exchange code for token
  // Store in jarvis_core_memory.json
  res.json({ accessToken, refreshToken });
});
```

**Required:** Google Cloud project with OAuth consent screen + API keys for Gmail, Calendar, Drive, Docs, Sheets, Slides, People, Keep APIs.

---

## Section B: CognitiveMemoryGraph

**Status:** 27KB graph visualizer. 5 mock refs. No API calls. Needs real semantic graph data.

**What it needs:**
- A data source with nodes (concepts) and edges (relations)
- The Jarvis semantic graph (`jarvis_semantic_graph.json`) has 1,410 nodes with relations
- The component already expects `KnowledgeNode` and `KnowledgeRelation` types

**New TRM route needed:**
```python
@router.get("/memory/semantic-graph")
async def get_semantic_graph():
    """Return the Jarvis semantic graph as nodes + edges for CognitiveMemoryGraph."""
    graph_path = "F:/Development sites/Jarvis-Advanced-main/Jarvis-Advanced-main/jarvis_semantic_graph.json"
    with open(graph_path) as f:
        graph = json.load(f)
    nodes = graph.get("nodes", [])
    edges = graph.get("edges", [])
    return {
        "nodes": [{"id": n["id"], "label": n.get("label", ""), "type": n.get("type", "concept")} for n in nodes],
        "edges": [{"source": e["source"], "target": e["target"], "label": e.get("label", "related")} for e in edges],
    }
```

**Component expects:**
```typescript
interface KnowledgeNode { id: string; label: string; type: string; }
interface KnowledgeRelation { source: string; target: string; label: string; }
```

---

## Section C: ComputerControlTerminal

**Status:** 11KB terminal emulator UI. No API calls.

**What it needs:**
- A backend endpoint that accepts commands and returns output
- Already exists at Jarvis `POST /api/execute-cmd` (line 1631 in server.ts)

**Wiring:**
- Component needs a prop or API call to `POST /api/execute-cmd` with `{ command: string }`
- The endpoint already returns stdout/stderr
- The component's mock data should be replaced with the real API call

**Component changes needed in App.tsx:**
- Pass `onExecuteCommand` prop that calls `POST /api/execute-cmd`

---

## Section D: SystemTelemetry

**Status:** 7KB telemetry display. No API calls.

**What it needs:**
- System stats from Jarvis `GET /api/system-stats` (line 1611 in server.ts)
- This route already returns CPU, memory, disk metrics

**Wiring:**
- Replace `SystemTelemetry` component's internal state with a `useEffect` that polls `GET /api/system-stats`

---

## Section E: TernarySynapseEngine

**Status:** 76KB AI reasoning engine UI. 0 API calls, 4 mock refs.

**What it needs:**
- Integration with `/api/vibe/consensus` (exists) and `/api/swarm/healing/status` (exists)
- Routes referenced in component: `/api/vibe/consensus`, `/api/chat`, `/api/swarm/healing/status`, `/api/swarm/intent/predict`, `/api/skills/matrix`

**The component already references these routes** — they need to be activated by calling them. The UI has placeholder data for:
- Synaptic maps (needs `/api/swarm/spatial/ambient` data)
- Ternary logic evaluation (needs `/api/vibe/consensus`)
- Healing status (needs `/api/swarm/healing/status`)

**Action:** Connect the component's state hooks to these API calls. The reducer/data functions are built but never trigger.

---

## Section F: RoboticsLab

**Status:** 67KB robotics control interface. 0 API calls, 0 mock refs. Pure UI.

**What it needs:**
- This component appears to be a hardware/IoT control interface
- Controls for: motor, sensor, actuator, communication channels
- No hardware is connected to this system
- **This is a simulation UI** — would need a full robotics backend

**Recommendation:** 
- Create a simulation backend route that returns mock robot states
- Or flag as "requires external hardware" and use as a UI mockup

**New TRM route:**
```python
@router.get("/api/robotics/status")
async def get_robotics_status():
    """Return simulated robotics system status."""
    return {
        "connected_devices": [],
        "actuators": [],
        "sensors": [],
        "status": "simulation_mode",
    }
```

---

## Section G: GlobalOfferLanding

**Status:** 49KB static landing page. 1 external URL (Google Doc). No API calls.

**What it needs:**
- This is a marketing/landing page component
- References a Google Doc for content
- Not interactive — purely informational

**Recommendation:** Serve as-is. No backend needed. It's already rendered in the dashboard when the user navigates to it.

---

## Section H: JarvisAvatar

**Status:** 16KB avatar renderer. 0 API calls, 0 mock refs.

**What it needs:**
- Renders Jarvis's visual avatar (likely SVG/canvas-based)
- Standalone visual component — no backend needed

**Recommendation:** No action needed. It's a UI decoration component.

---

## Learning Feedback Loop — Complete Wiring

**Current state:**
- 93 skills learned ✅
- 0 usages tracked ❌
- `/learning/use` endpoint exists ✅
- Learning Feedback Loop cron (every 6h) created ✅

**What's needed to close the loop:**

### 1. Add usage tracking to skill.execute()
In `learning_engine.py`, modify `learned_skills[name].usage_count` increment:
```python
def execute(self, task: str) -> str:
    self.usage_count += 1
    # ... existing logic
```

### 2. Create a "Skill Recommendation" endpoint
```python
@router.get("/learning/recommendations")
async def get_skill_recommendations():
    """Recommend skills based on usage data and gaps."""
    engine = get_learning_engine()
    skills = engine.list_skills()
    # Find least-used skills and suggest re-learning
    rarely_used = [s for s in skills if s.get("usage_count", 0) < 3]
    return {"recommendations": rarely_used[:5], "total": len(rarely_used)}
```

### 3. Evolution trigger from usage data
The weekly evolution cron should:
1. Check `total_usages` — if < 10, broadcast coordination task to use skills
2. Check for duplicate/overlapping skills — suggest merges
3. Identify knowledge gaps — trigger GitHub acquisition for missing topics

---

## Priority Order for Implementation

| Priority | Component | Effort | Impact |
|---|---|---|---|
| P0 | **Learning Feedback Loop** | Small | Closes the evolution cycle |
| P1 | **CognitiveMemoryGraph** | Small | Visualizes 1,410 semantic nodes |
| P2 | **ComputerControlTerminal** | Small | Real terminal access from dashboard |
| P3 | **SystemTelemetry** | Small | Live system stats in dashboard |
| P4 | **TernarySynapseEngine** | Medium | AI reasoning UI |
| P5 | **WorkspaceNexus** | Large | Full Google Workspace integration |
| P6 | **RoboticsLab** | Large | Requires hardware/simulation backend |
