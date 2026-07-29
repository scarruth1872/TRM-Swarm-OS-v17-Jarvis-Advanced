import requests
import json

prompt = """[SWARM_OS_DIRECTIVE: FULL_PLATFORM_DEMO]

ATTENTION SWARM OS COLLECTIVE: Archi (Generative Architect), Logic (Reasoning Engine), Devo (Lead Developer), Sentinel (Security Auditor), Ops (DevOps Engineer), Pixel (UI/UX Designer), Tester (QA Engineer), Director (Swarm Manager), Scribe (Technical Writer), Nexus (Integration Specialist), Metrics (Data Analyst), and Scout (Researcher).

PROJECT DIRECTIVE: Synthesize an Autonomous High-Frequency Financial & Robotics Telemetry Gateway with Multi-Modal Anomaly Prevention, Formal Verification Safety Proofs, and Self-Healing Microservices.

EXECUTION PHASES:
1. RESEARCH & KNOWLEDGE MAPPING (Scout, Metrics, Director):
   - Query the Semantic Knowledge Graph (SKG) for existing topology dependencies and node health baselines.
   - Analyze incoming high-bandwidth telemetry streams for latency bottlenecks under 500,000 RPS.

2. RECURSIVE REASONING & CONSENSUS AMPLIFICATION (Logic, Archi, Sentinel):
   - Initiate a Collaborative Reasoning Amplification (CRA) session with TRM ACT-V1 recursive cluster analysis (ANA -> FLW -> RMT).
   - Formulate consensus on circuit breaker thresholds, rate limiting budgets, and memory pruning policies.

3. MICROSERVICE BLUEPRINT GENERATION & FORMAL PROOF (Archi, Devo, Sentinel):
   - Archi & Devo: Synthesize the production-grade Python microservice 'AsyncHighFrequencyTelemetryGateway'.
   - Sentinel: Pass code AST through the Formal Verification Engine (FVE) enforcing AST safety invariants (INV-001 through INV-004). Require a PROVED_SAFE classification.

4. MULTIMODAL FUSION & PREDICTIVE ANALYTICS INTEGRATION (Nexus, Logic, Tester):
   - Integrate cross-modal sensor telemetry (vibration, motor thermal RPM, electrical current) with visual camera bounding boxes.
   - Compute Time-To-Failure (TTF) forecasting and bind visual gesture override signals (PALM_STOP) for instantaneous emergency shutdown.

5. TRANSACTIONAL SELF-MODIFICATION & PBFT COMMIT (Archi, Director, Ops):
   - Execute an Event-Driven Self-Modification API (EDSMA) transaction.
   - Broadcast proposal across the 4-node PBFT Consensus Layer to achieve supermajority commit (3/4 nodes).

6. DASHBOARD & TECHNICAL REPORT DISPATCH (Pixel, Scribe):
   - Broadcast live state updates to the High-Bandwidth Telemetry Fabric (HBTF) for real-time visual streaming on Tab 19 Generative Architect Studio.
   - Scribe: Compile a final deployment report with execution benchmarks, confidence scores, and safety audit certificates.

BEGIN ALL PHASES CONCURRENTLY AND REPORT FINAL CONSENSUS STATE."""

payload = {
    "role": "Architect",
    "message": prompt,
    "sender": "user"
}

res = requests.post("http://127.0.0.1:8021/swarm/chat", json=payload)
print("STATUS CODE:", res.status_code)
print("RESPONSE BODY:", res.text)
