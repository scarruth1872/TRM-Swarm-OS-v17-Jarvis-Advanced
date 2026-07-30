import os
import uuid
import asyncio
import psutil
from datetime import datetime
from contextlib import asynccontextmanager
from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from typing import List, Dict, Any, Optional
from dotenv import load_dotenv

# Load environment variables (Phase 12+)
load_dotenv()

from swarm_v2.experts.registry import get_expert_team
from swarm_v2.core.artifact_pipeline import ArtifactPipeline, ArtifactStatus
from swarm_v2.skills.learning_engine import get_learning_engine
from swarm_v2.mcp.synthesizer import get_synthesizer
from swarm_v2.core.agent_mesh import get_agent_mesh, MeshNode
from swarm_v2.core.global_memory import get_global_memory
from swarm_v2.core.monitor_daemon import MonitorDaemon
from swarm_v2.core.ipc import MemoryMappedIPC
from swarm_v2.core.daemon_discovery import DiscoveryDaemon
from swarm_v2.core.intent_buffer import PredictiveIntentBuffer
from swarm_v2.core.remediation_engine import RemediationEngine
from swarm_v2.core.resource_arbiter import get_resource_arbiter
from swarm_v2.core.expert_registry import get_expert_registry
from swarm_v2.core.task_arbiter import get_task_arbiter
from swarm_v2.core.federation import init_federation, get_federation
from swarm_v2.core.reconnaissance_daemon import get_reconnaissance_daemon, start_reconnaissance
from swarm_v2.core.chain_of_verification import get_chain_of_verification, verify_reasoning
from swarm_v2.core.neural_wall import get_neural_wall, analyze_prompt, NeuralWallMiddleware
from swarm_v2.core.self_healing_infra import get_self_healing_infra, start_self_healing, stop_self_healing
from swarm_v2.core.zero_human_test_gen import get_test_generator, generate_tests_for_tool, on_tool_synthesized
from swarm_v2.core.auto_changelog import get_changelog_engine, start_changelog_monitoring
from swarm_v2.core.mcp_tool_evolver import get_tool_evolver, start_tool_evolution, EvolutionTrigger
from swarm_v2.core.swarm_performance_insights import get_performance_insights, start_performance_monitoring, ReportPeriod, MetricType
from swarm_v2.core.deterministic_forge import get_deterministic_forge
from swarm_v2.core.telemetry import get_telemetry
from swarm_v2.core.proactive_loop import get_proactive_loop
from swarm_v2.core.sentinel import SentinelMiddleware
from swarm_v2.core.redis_mock import PersistentRedisMock
from swarm_v2.core.ses import get_ses
from swarm_v2.core.cqs import get_cqs
from infrastructure.routing.qer_mesh import get_qer_mesh
from swarm_v2.core.tool_forge import ToolForge
from swarm_v2.core.monitor_daemon import MonitorDaemon
from swarm_v2.skills.learning_engine import get_learning_engine
from swarm_v2.core.global_memory import get_global_memory
from swarm_v2.mcp.synthesizer import get_synthesizer
from swarm_v2.core.background_tasks import (
    auto_scan_pipeline,
    mesh_heartbeat_reflex,
    proactive_orchestration_loop,
    autonomous_pipeline_loop,
)
from swarm_v2.core.cognitive_ingestion_engine import get_ingestion_engine, get_event_ledger

# Memory-Mapped IPC shared memory instance
ipc_bridge = MemoryMappedIPC()

# Predictive Intent Buffer instance
intent_buffer = PredictiveIntentBuffer()

def sync_mesh_to_ipc():
    """Write current agent mesh topology snapshot to memory-mapped IPC."""
    try:
        if agent_mesh:
            topology = agent_mesh.get_topology_snapshot()
            ipc_bridge.write_state(topology)
    except Exception as e:
        print(f"[IPC Sync Error] Failed to sync mesh topology: {e}")

async def sync_mesh_to_ipc_async():
    """Write current agent mesh topology snapshot to memory-mapped IPC asynchronously."""
    try:
        if agent_mesh:
            topology = agent_mesh.get_topology_snapshot()
            await ipc_bridge.write_state_async(topology)
    except Exception as e:
        print(f"[IPC Async Sync Error] Failed to sync mesh topology: {e}")

# ─── System Optimization ───────────────────────────────────────────────────────

def optimize_system():
    """Pin the current process to specific CPU cores and set priority."""
    try:
        proc = psutil.Process(os.getpid())
        # Logical cores check (24 logical cores available according to user system)
        core_count = psutil.cpu_count(logical=True)
        if core_count >= 16:
            # Pin to cores 4-11 (8 cores) to leave 0-3 for OS and 12-23 for Ollama/GPU tasks
            proc.cpu_affinity(list(range(4, 12)))
            print(f"[System] CPU Pinning enabled: Process bound to cores 4-11")
        
        # Set above normal priority for responsiveness
        proc.nice(psutil.ABOVE_NORMAL_PRIORITY_CLASS)
        print(f"[System] Process priority set to ABOVE_NORMAL")
    except Exception as e:
        print(f"[System] Resource optimization failed: {e}")

optimize_system()

@asynccontextmanager
async def lifespan(app: FastAPI):
    print("[System] Lifespan startup...")
    
    # Bootstrap default antibodies in the Digital DNA Repository
    from swarm_v2.core.ddr_antibody import get_ddr
    get_ddr().install_default_antibodies()
    
    # Verify Agent Mesh is ready — re-fetch to guarantee initialization
    # regardless of module-load ordering.
    global agent_mesh
    if agent_mesh is None:
        agent_mesh = get_agent_mesh()
    if agent_mesh:
        topology = agent_mesh.get_topology_snapshot()
        print(f"[System] Agent Mesh ready with {len(topology.get('nodes', []))} nodes")
        # Register a permanent virtual gateway node representing Jarvis OS
        try:
            from swarm_v2.core.agent_mesh import MeshNode
            jarvis_node = MeshNode(
                node_id="jarvis_gateway",
                name="Jarvis Gateway",
                role="gateway",
                host="localhost",
                port=4000,
                specialties=["cognitive_companion", "google_workspace", "ui_portal"],
                skills=["runJarvisAgent", "read_file_contents"]
            )
            agent_mesh.register_node(jarvis_node)
            print("[System] Registered Jarvis OS Gateway node in Agent Mesh")

            antigravity_node = MeshNode(
                node_id="antigravity_pm",
                name="Antigravity PM",
                role="gateway", # Use gateway role to bypass active worker heartbeat checks
                host="localhost",
                port=8021,
                specialties=["project_management", "swarm_orchestration", "synthesis_verification"],
                skills=["delegate_to_pm", "run_cognitive_cycle"]
            )
            agent_mesh.register_node(antigravity_node)
            print("[System] Registered Antigravity PM node in Agent Mesh")
            from swarm_v2.core.expert_registry import get_expert_registry
            get_expert_registry().register_team(engine_team)
            print(f"[System] Registered {len(engine_team)} live agent personas with ExpertRegistry")
        except Exception as e:
            print(f"[System] Failed to register virtual nodes: {e}")
    else:
        print("[Warning] Agent Mesh not available!")
    
    # Start background tasks
    asyncio.create_task(monitor_daemon.start())
    asyncio.create_task(auto_scan_pipeline(pipeline, engine_team))
    asyncio.create_task(mesh_heartbeat_reflex(agent_mesh, engine_team))

    # Start proactive growth and seeker research loops
    asyncio.create_task(proactive_orchestration_loop(pipeline, engine_team, agent_mesh))
    asyncio.create_task(autonomous_pipeline_loop(pipeline, engine_team))
    from swarm_v2.core.proactive_loop import get_proactive_loop
    asyncio.create_task(get_proactive_loop().start())
    from swarm_v2.core.reconnaissance_daemon import start_reconnaissance
    start_reconnaissance(interval_hours=24)
    
    # Start self-healing autonomic loop
    from swarm_v2.core.self_healing_orchestrator import get_self_healing_orchestrator
    from swarm_v2.experts.registry import get_expert_team
    sh = get_self_healing_orchestrator()
    asyncio.create_task(sh._healing_loop(agent_mesh, get_expert_team()))
    
    # Start self-healing infrastructure watchdog
    from swarm_v2.core.self_healing_infra import start_self_healing
    asyncio.create_task(start_self_healing())
    
    # Start Health Dashboard Monitor (HDM)
    from swarm_v2.core.self_healing import health_monitor
    asyncio.create_task(health_monitor.start())
    
    # Start skill compliance contract validation loop
    from swarm_v2.core.skill_validator import get_skill_validator
    asyncio.create_task(get_skill_validator().start_validation_loop(interval=30))
    
    # Start autonomous codebase cognitive learning cycles
    try:
        from swarm_v2.skills.cognitive_orchestrator import get_cognitive_orchestrator
        asyncio.create_task(get_cognitive_orchestrator().start())
        print("[System] Autonomous Codebase Learning loop started.")
    except Exception as e:
        print(f"[System] Failed to start Codebase Learning loop: {e}")
    
    # Start federation if enabled
    if federation:
        federation.start_heartbeat()
        print("[Federation] P2P Heartbeat started.")
        
        # Phase 15: Collective Singularity Manager
        from swarm_v2.core.federation_manager import get_federation_manager
        from swarm_v2.core.memory_synchronizer import get_memory_synchronizer
        fed_manager = get_federation_manager()
        mem_syncer = get_memory_synchronizer()
        asyncio.create_task(fed_manager.start())
        asyncio.create_task(mem_syncer.start())
        print("[Federation] Collective Singularity Manager & Memory Syncer online.")
    
    # Start Swarm Orchestrator monitoring loop
    try:
        from swarm_v2.core.swarm_orchestrator import get_swarm_orchestrator
        asyncio.create_task(get_swarm_orchestrator().run_monitoring_loop(agent_mesh))
        print("[System] Swarm Orchestrator monitoring loop started.")
    except Exception as e:
        print(f"[System] Failed to start Swarm Orchestrator: {e}")
    
    # Start dynamic agent discovery daemons
    try:
        from swarm_v2.experts.registry import get_expert_team
        team = get_expert_team()
        for role, agent in team.items():
            node_id = getattr(agent, "mesh_node_id", None)
            if not node_id:
                # Derive a stable, schema-safe node id from the role.
                # MeshNodeSchema requires ^[a-zA-Z0-9_-]+$, so replace any
                # non-alphanumeric char (spaces, slashes, etc.) with '_'.
                import re as _re
                safe = _re.sub(r"[^a-zA-Z0-9_-]+", "_", role.lower()).strip("_")
                node_id = f"agent_{safe}"
                try:
                    agent.mesh_node_id = node_id
                except Exception:
                    pass
            daemon = DiscoveryDaemon(node_id=node_id, name=agent.persona.name, role=role, target_url="http://localhost:8021/mesh/announce", interval=10)
            asyncio.create_task(daemon.start())
            print(f"[Discovery] Heartbeat daemon started for role: {role}")
    except Exception as e:
        print(f"[Discovery Startup Error] Failed to start discovery daemons: {e}")
    
    # ── Jarvis Cognitive Ingestion + System Event Ledger ────────────────
    try:
        _ledger = get_event_ledger()
        _ledger.record(
            "SYSTEM", "Swarm OS Kernel startup complete",
            {"port": 8021, "agents": len(engine_team)},
            source="app_v2_lifespan"
        )
        _ingestion_engine = get_ingestion_engine()
        asyncio.create_task(_ingestion_engine.run_daemon(interval=60))
        print("[System] Jarvis Cognitive Ingestion daemon started (60s interval).")
        print("[System] System Event Ledger active:", _ledger.path if hasattr(_ledger, 'path') else 'OK')
    except Exception as e:
        print(f"[System] Failed to start Cognitive Ingestion daemon: {e}")

    yield
    
    # Shutdown logic
    print("[System] Lifespan shutdown...")
    try:
        _ledger = get_event_ledger()
        _ledger.record("SYSTEM", "Swarm OS Kernel shutdown initiated", source="app_v2_lifespan", severity="WARNING")
        _ingestion_engine = get_ingestion_engine()
        _ingestion_engine.stop()
    except Exception:
        pass
    await monitor_daemon.stop()

from starlette.middleware.base import BaseHTTPMiddleware
from fastapi import Request
from fastapi.responses import JSONResponse
from swarm_v2.core.self_healing import input_sanitizer, token_monitor, RateLimitExceeded, telemetry

# Paths that legitimately carry source code in the body. These are validated by
# the Evolutionary Sandbox's AST DNA Security Shield (DangerousNodeVisitor),
# so the generic SQLi/XSS/path-traversal body scanner must NOT block them.
CODE_PAYLOAD_PREFIXES = (
    "/swarm/chat",
    "/swarm/cra",
    "/evolution/",
    "/artifacts/create",
    "/skills/forge",
    "/tools/forge",
    "/api/continuum",
    "/api/cognitive",
    "/api/system",
)

def _is_code_payload_path(path: str) -> bool:
    return any(path.startswith(p) for p in CODE_PAYLOAD_PREFIXES)

class InputSanitizerMiddleware(BaseHTTPMiddleware):
    async def dispatch(self, request: Request, call_next):
        if request.method in ["POST", "PUT", "PATCH"] and request.headers.get("content-length") not in (None, "0"):
            try:
                body = await request.body()
            except Exception:
                return await call_next(request)
            if _is_code_payload_path(request.url.path):
                # Restore body for downstream handlers, skip pattern sanitize.
                async def receive():
                    return {"type": "http.request", "body": body}
                request._receive = receive
                return await call_next(request)
            try:
                sanitized_str = input_sanitizer.sanitize(body)
                telemetry.record_input(len(body))
                async def receive():
                    return {"type": "http.request", "body": sanitized_str.encode('utf-8')}
                request._receive = receive
            except ValueError as e:
                return JSONResponse(status_code=413, content={"detail": str(e)})
        return await call_next(request)

class TokenRateMonitorMiddleware(BaseHTTPMiddleware):
    async def dispatch(self, request: Request, call_next):
        if request.method in ["POST", "PUT", "PATCH"] and request.headers.get("content-length") not in (None, "0"):
            try:
                body = await request.body()
            except Exception:
                return await call_next(request)
            estimated_tokens = len(body) // 4
            session_id = request.client.host if request.client else "unknown"
            try:
                token_monitor.check_rate(session_id, estimated_tokens)
            except RateLimitExceeded as e:
                return JSONResponse(status_code=429, content={"detail": str(e)})
            
            async def receive():
                return {"type": "http.request", "body": body}
            request._receive = receive
            
        return await call_next(request)

app = FastAPI(title="TRM Swarm V2 API", version="2.3.1", lifespan=lifespan)

app.add_middleware(TokenRateMonitorMiddleware)
app.add_middleware(InputSanitizerMiddleware)

app.add_middleware(
    SentinelMiddleware,
    redis_client=PersistentRedisMock(),
    rate_limit=100,
    rate_window=60
)

app.add_middleware(NeuralWallMiddleware)

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

agent_mesh = get_agent_mesh()
sync_mesh_to_ipc()

# ── Permanent virtual gateway nodes (module-load, deterministic) ──────────────
# Jarvis OS and Antigravity PM are stable companion gateways. Register them at
# import time so they survive restarts without depending on lifespan ordering.
try:
    from swarm_v2.core.agent_mesh import MeshNode
    _jarvis_node = MeshNode(
        node_id="jarvis_gateway",
        name="Jarvis Gateway",
        role="gateway",
        host="localhost",
        port=4000,
        specialties=["cognitive_companion", "google_workspace", "ui_portal"],
        skills=["runJarvisAgent", "read_file_contents"],
    )
    agent_mesh.register_node(_jarvis_node)
    _antigravity_node = MeshNode(
        node_id="antigravity_pm",
        name="Antigravity PM",
        role="gateway",
        host="localhost",
        port=8021,
        specialties=["project_management", "swarm_orchestration", "synthesis_verification"],
        skills=["delegate_to_pm", "run_cognitive_cycle"],
    )
    agent_mesh.register_node(_antigravity_node)
    print("[System] Registered permanent gateway nodes: Jarvis OS, Antigravity PM")
except Exception as e:
    print(f"[System] Failed to register permanent gateway nodes: {e}")

engine_team = get_expert_team()
get_expert_registry().register_team(engine_team)
spawned_subagents: Dict[str, Any] = {}
learning_engine = get_learning_engine()
global_memory = get_global_memory()
synthesizer = get_synthesizer()
background_mailbox_tasks: set = set()
from infrastructure.telemetry.spatial_broadcaster import get_spatial_broadcaster
from swarm_v2.core.feedback_loops import get_feedback_engine
spatial_broadcaster = get_spatial_broadcaster()
feedback_engine = get_feedback_engine()
pipeline = ArtifactPipeline()

# Force scan to register all existing artifacts in the directory
pipeline.scan_artifacts()
print(f"[Pipeline] Initial scan found {len(pipeline.artifacts)} artifacts")

# Log some stats
stats = pipeline.get_stats()
print(f"[Pipeline] Stats: {stats}")

# Phase 4: Self-Healing Architecture
remediation_engine = RemediationEngine(engine_team)

# Create MonitorDaemon after Agent Mesh is initialized
monitor_daemon = MonitorDaemon(remediation_engine, interval=15, mesh=agent_mesh)

# Phase 5: Mesh Federation
SWARM_NODE_ID = os.getenv("SWARM_NODE_ID", "swarm_primary")
SWARM_NODE_NAME = os.getenv("SWARM_NODE_NAME", "Primary Swarm")
SWARM_PORT = int(os.getenv("SWARM_PORT", "8001"))
federation = init_federation(SWARM_NODE_ID, SWARM_NODE_NAME, SWARM_PORT)

# Phase 3: Semantic Layer
ses_engine = get_ses()
cqs_service = get_cqs()
tool_forge = ToolForge(learning_engine)

# Phase 11: Cognitive Fluidity Layer
qer_mesh = get_qer_mesh()

# Agent-to-role mapping for smart re-dispatch
ROLE_FILE_MAPPING = {
    ".py": "Lead Developer",
    ".yml": "DevOps Engineer",
    ".yaml": "DevOps Engineer",
    ".sh": "DevOps Engineer",
    ".md": "Technical Writer",
    ".json": "Lead Developer",
    ".txt": "Technical Writer",
}

def _agent_for_file(filename: str) -> str:
    """Pick the right agent to rebuild a file based on extension."""
    for ext, role in ROLE_FILE_MAPPING.items():
        if filename.endswith(ext):
            return role
    return "Lead Developer"


# =============================================================================
# Route Controllers — modular endpoint definitions
# =============================================================================
from swarm_v2.routes.swarm import router as swarm_router
from swarm_v2.routes.artifacts import router as artifacts_router
from swarm_v2.routes.learning import router as learning_router
from swarm_v2.routes.synthesis import router as synthesis_router
from swarm_v2.routes.mesh import router as mesh_router
from swarm_v2.routes.memory import router as memory_router
from swarm_v2.routes.research import router as research_router
from swarm_v2.routes.evolution import router as evolution_router
from swarm_v2.routes.system import router as system_router
from swarm_v2.routes.security import router as security_router
from swarm_v2.routes.verification import router as verification_router
from swarm_v2.routes.changelog import router as changelog_router
from swarm_v2.routes.insights import router as insights_router
from swarm_v2.routes.kanban import router as kanban_router
from swarm_v2.routes.remediation import router as remediation_router
from swarm_v2.routes.coordination import router as coordination_router
from swarm_v2.routes.coordination_respond import router as coordination_respond_router
from swarm_v2.routes.orchestrator import router as orchestrator_router
from swarm_v2.routes.untapped import router as untapped_router

app.include_router(swarm_router)
app.include_router(artifacts_router)
app.include_router(learning_router)
app.include_router(synthesis_router)
app.include_router(mesh_router)
app.include_router(memory_router)
app.include_router(research_router)
app.include_router(evolution_router)
app.include_router(system_router)
app.include_router(security_router)
app.include_router(verification_router)
app.include_router(changelog_router)
app.include_router(insights_router)
app.include_router(kanban_router)
app.include_router(remediation_router)
app.include_router(coordination_router)
app.include_router(coordination_respond_router)
app.include_router(orchestrator_router)
app.include_router(untapped_router)

# --- Generative Architect REST API Endpoints ---
@app.get("/generative-architect/status")
async def get_generative_architect_status():
    from swarm_v2.core.generative_architect import get_generative_architect
    ga = get_generative_architect()
    return ga.get_status().model_dump()

@app.get("/generative-architect/skg")
async def get_skg_snapshot():
    from swarm_v2.core.semantic_knowledge_graph import get_semantic_knowledge_graph
    skg = get_semantic_knowledge_graph()
    return skg.get_topology_snapshot()

@app.post("/generative-architect/synthesize")
async def synthesize_architecture(payload: dict):
    from swarm_v2.core.generative_architect import get_generative_architect
    ga = get_generative_architect()
    component = payload.get("component", "telemetry_fabric")
    goal = payload.get("goal", "optimize_latency")
    return ga.synthesize_architectural_solution(component, goal)

@app.post("/generative-architect/propose-self-modification")
async def propose_self_modification(payload: dict):
    from swarm_v2.core.generative_architect import get_generative_architect
    ga = get_generative_architect()
    target_file = payload.get("target_file", "swarm_v2/core/sample.py")
    description = payload.get("description", "Self-modification proposal")
    original_code = payload.get("original_code", "# Original")
    proposed_code = payload.get("proposed_code", "def upgraded_function(): return True")
    return ga.propose_autonomous_self_modification(target_file, description, original_code, proposed_code)

@app.get("/generative-architect/telemetry/fabric")
async def get_telemetry_fabric_metrics():
    from swarm_v2.core.telemetry_fabric import get_telemetry_fabric
    tf = get_telemetry_fabric()
    return tf.get_fabric_metrics()

# --- Multimodal AI Ingestion & Fusion Endpoints ---
@app.post("/multimodal/fuse-event")
async def fuse_multimodal_event(payload: dict):
    from swarm_v2.core.multimodal_ingestion import SensorTelemetryData, VisualFrameData
    from swarm_v2.core.multimodal_fusion import get_multimodal_fusion
    
    fusion_engine = get_multimodal_fusion()

    t_data = SensorTelemetryData(
        sensor_id=payload.get("sensor_id", "sensor_node_01"),
        temperature_celsius=float(payload.get("temperature_celsius", 45.0)),
        vibration_hz=float(payload.get("vibration_hz", 12.0)),
        power_draw_watts=float(payload.get("power_draw_watts", 250.0)),
        pressure_psi=float(payload.get("pressure_psi", 30.0))
    )

    v_data = VisualFrameData(
        frame_id=payload.get("frame_id", "frame_1001"),
        camera_id=payload.get("camera_id", "cam_thermal_01"),
        thermal_hotspot_intensity=float(payload.get("thermal_hotspot_intensity", 0.15)),
        optical_flow_energy=float(payload.get("optical_flow_energy", 0.05)),
        smoke_particle_density=float(payload.get("smoke_particle_density", 0.02)),
        bounding_box_count=int(payload.get("bounding_box_count", 2))
    )

    event = fusion_engine.fuse_and_detect(t_data, v_data)
    return event.model_dump()

@app.post("/multimodal/scene-understand")
async def analyze_complex_scene(payload: dict):
    from swarm_v2.core.advanced_scene_fusion import get_advanced_scene_fusion, ComplexSceneContext, BoundingBox, AudioDescriptor
    sf = get_advanced_scene_fusion()

    raw_boxes = payload.get("detected_objects", [])
    boxes = [BoundingBox(**b) for b in raw_boxes] if raw_boxes else [
        BoundingBox(label="person", confidence=0.95, x_min=0.1, y_min=0.2, x_max=0.3, y_max=0.8),
        BoundingBox(label="robotic_arm", confidence=0.98, x_min=0.15, y_min=0.25, x_max=0.5, y_max=0.9)
    ]

    audio_raw = payload.get("audio", {})
    audio = AudioDescriptor(
        primary_frequency_hz=float(audio_raw.get("primary_frequency_hz", 120.0)),
        decibel_level=float(audio_raw.get("decibel_level", 65.0)),
        speech_detected=bool(audio_raw.get("speech_detected", False)),
        acoustic_anomaly=bool(audio_raw.get("acoustic_anomaly", False))
    )

    context = ComplexSceneContext(
        scene_id=payload.get("scene_id", "scene_101"),
        location=payload.get("location", "Factory_Floor_Zone_A"),
        detected_objects=boxes,
        audio=audio
    )

    res = sf.analyze_scene(context)
    return res.model_dump()

@app.post("/multimodal/predictive-analytics")
async def forecast_degradation(payload: dict):
    from swarm_v2.core.multimodal_predictive_analytics import get_multimodal_predictive_analytics, MultimodalTimeSeriesPoint
    pa = get_multimodal_predictive_analytics()

    comp_id = payload.get("component_id", "spindle_motor_01")
    raw_points = payload.get("time_series", [])
    pts = [MultimodalTimeSeriesPoint(**p) for p in raw_points] if raw_points else [
        MultimodalTimeSeriesPoint(temperature_c=45.0, vibration_hz=12.0, acoustic_db=60.0, thermal_hotspot=0.10),
        MultimodalTimeSeriesPoint(temperature_c=80.0, vibration_hz=35.0, acoustic_db=85.0, thermal_hotspot=0.75)
    ]

    res = pa.forecast_degradation(comp_id, pts)
    return res.model_dump()

@app.post("/multimodal/human-command")
async def parse_human_command(payload: dict):
    from swarm_v2.core.human_swarm_interaction import get_human_swarm_interaction, HumanCommandInput
    hsi = get_human_swarm_interaction()

    inp = HumanCommandInput(
        operator_id=payload.get("operator_id", "operator_alpha"),
        voice_transcript=payload.get("voice_transcript"),
        gesture_token=payload.get("gesture_token"),
        target_zone=payload.get("target_zone", "Zone_A")
    )

    res = hsi.parse_human_command(inp)
    return res.model_dump()

if __name__ == "__main__":
    import uvicorn
    uvicorn.run("swarm_v2.app_v2:app", host="0.0.0.0", port=8021, reload=False)
