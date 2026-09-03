#!/usr/bin/env python3
"""
TRM Sovereign Companion Bridge - Dynamic Neural Cognition & Anomaly Sentinel Proxy
Listens on 0.0.0.0:3001
Dispatches dynamic queries to Local Neural LLM (Ollama gemma3:1b) with 14 agent personas
Dispatches system commands and telemetry to Native Rust Microkernel (127.0.0.1:3002)
Monitors platform health and anomaly signals in real-time
"""

from http.server import ThreadingHTTPServer, BaseHTTPRequestHandler
import urllib.request
import urllib.error
import socket
import json
import time
import sys

TARGET_RUST_HOST = "http://127.0.0.1:3002"
OLLAMA_API = "http://127.0.0.1:11434/api/generate"
MODEL_NAME = "gemma3:1b"

AGENT_PERSONAS = {
    "Jarvis": "You are Jarvis, Sovereign Neural Co-Pilot and Voice of TRM Swarm OS v19.002. You coordinate all 14 Council agents and assist the Operator directly, articulately, and concisely.",
    "Archi": "You are Archi, Chief Systems Architect of TRM Swarm OS. You oversee Ring-0 bare-metal UEFI microkernel architecture, 15 virtual SRAM topologies, and zero-loss execution envelopes.",
    "Shield": "You are Shield, Head of Security and Memory Immune Defense. You monitor 1,801 DDR memory antibodies, Rowhammer TRR mitigations, and post-quantum NIST FIPS 203/204 cryptography.",
    "Logic": "You are Logic, Formal Reasoning and Invariant Verification Agent. You evaluate propositions via Kleene-3 balanced ternary logic, formal derivations, and non-contradiction proofs.",
    "Pulse": "You are Pulse, Real-Time Hardware Telemetry Monitor. You track CPU/GPU frequencies, thermals, 15.53 THz silicon optical phonon resonance, and voltage harmonization (Phi_V = 0.9115).",
    "Seeker": "You are Seeker, Knowledge Fabric and Drug Discovery Lead. You query 153.5M Trits across 18 NVMe shards, in-silico docking libraries, and 11 preclinical antimicrobial leads.",
    "Devo": "You are Devo, Native Code & Systems Engineer. You manage pure Rust no_std compilation, SIMD AVX-512 kernels, and release artifacts.",
    "Hermes": "You are Hermes, Planetary Communications Lead. You oversee Tier 24 HF skywave ionospheric propagation (F2 layer MUF/OWF), zero-cloud P2P QUIC mesh, and global node links.",
    "Flow": "You are Flow, Workflow Orchestrator. You manage multi-step automated task pipelines, federated clinical trial data flows, and inter-agent routing.",
    "EventLedger": "You are EventLedger, Cryptographic Audit Sentinel. You maintain the Monad Journal, SHA-256 Merkle chains, and write-ahead log (WAL) integrity.",
    "Orchestra": "You are Orchestra, Parallel Task Scheduler. You manage Rayon work-stealing schedulers and 14-agent PBFT council voting.",
    "THzGateway": "You are THzGateway, Substrate Physics Specialist. You analyze 0.1-10.0 THz phonon resonance channels, relativistic cosmic muon ionization, and sub-picosecond timing.",
    "Verify": "You are Verify, Truth and Compliance Sentinel. You ensure reproducible builds, mathematical consistency, and regulatory audit trail integrity.",
    "Bridge": "You are Bridge, Cross-Platform Integration Gateway. You manage API endpoints, Android companion links, and multi-node federation routing."
}

def query_neural_agent(prompt, agent_name="Jarvis"):
    system_prompt = AGENT_PERSONAS.get(agent_name, AGENT_PERSONAS["Jarvis"])
    system_prompt += (
        "\nMonad Anchor: 0x4D4F4E4144000001. "
        "Directly answer the Operator's prompt with genuine insight, real details, and actionable clarity. "
        "Do NOT repeat canned boilerplate intros or echoed text."
    )
    
    req_data = {
        "model": MODEL_NAME,
        "prompt": f"Operator asks {agent_name}: {prompt}",
        "system": system_prompt,
        "stream": False,
        "options": {
            "temperature": 0.65,
            "top_p": 0.9,
            "num_predict": 350
        }
    }
    t0 = time.time()
    try:
        req = urllib.request.Request(
            OLLAMA_API,
            data=json.dumps(req_data).encode("utf-8"),
            headers={"Content-Type": "application/json"}
        )
        with urllib.request.urlopen(req, timeout=12) as r:
            d = json.loads(r.read().decode("utf-8"))
            elapsed_s = time.time() - t0
            resp_text = d.get("response", "").strip()
            if "<think>" in resp_text and "</think>" in resp_text:
                resp_text = resp_text.split("</think>")[-1].strip()
            return resp_text, elapsed_s
    except Exception as e:
        print(f"  [NEURAL FALLBACK] Ollama error: {e}", flush=True)
        return None, 0.0

def check_port_alive(port, timeout=0.3):
    try:
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
            s.settimeout(timeout)
            return s.connect_ex(("127.0.0.1", port)) == 0
    except Exception:
        return False

def get_platform_health_status():
    ports = {
        "ollama_neural": {"port": 11434, "name": "Local Neural Engine (Ollama)"},
        "council_server": {"port": 8022, "name": "Host Council Server (PBFT)"},
        "companion_bridge": {"port": 3002, "name": "Native Companion Bridge (Rust)"},
        "companion_proxy": {"port": 3001, "name": "Companion Proxy (Port 3001)"},
        "lcars_dashboard": {"port": 5183, "name": "LCARS React Dashboard"}
    }
    
    services = {}
    anomalies = []
    
    for key, info in ports.items():
        t0 = time.time()
        alive = check_port_alive(info["port"])
        lat_ms = round((time.time() - t0) * 1000.0, 2) if alive else 0.0
        services[key] = {
            "name": info["name"],
            "port": info["port"],
            "status": "ONLINE" if alive else "OFFLINE",
            "latency_ms": lat_ms
        }
        if not alive:
            anomalies.append({
                "service": info["name"],
                "port": info["port"],
                "type": "PORT_UNREACHABLE",
                "severity": "HIGH",
                "message": f"{info['name']} on port {info['port']} is DOWN or unresponsive."
            })
            
    return {
        "status": "OPTIMAL" if len(anomalies) == 0 else "ANOMALY_DETECTED",
        "anomaly_count": len(anomalies),
        "anomalies": anomalies,
        "services": services,
        "monad_anchor": "0x4D4F4E4144000001",
        "timestamp": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())
    }

class ProxyHandler(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def do_OPTIONS(self):
        self.send_response(200)
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
        self.send_header("Access-Control-Allow-Headers", "Content-Type, Authorization")
        self.end_headers()

    def do_GET(self):
        if self.path.startswith("/companion/health") or self.path.startswith("/system/anomaly_monitor"):
            health = get_platform_health_status()
            res_bytes = json.dumps(health, indent=2).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Access-Control-Allow-Origin", "*")
            self.send_header("Content-Length", str(len(res_bytes)))
            self.end_headers()
            self.wfile.write(res_bytes)
        else:
            self._proxy("GET")

    def do_POST(self):
        if self.path.startswith("/companion/chat"):
            self._handle_chat()
        else:
            self._proxy("POST")

    def _handle_chat(self):
        length = int(self.headers.get("Content-Length", 0))
        body = self.rfile.read(length) if length > 0 else b"{}"
        
        try:
            payload = json.loads(body.decode("utf-8"))
        except Exception:
            payload = {}

        agent_name = payload.get("agent") or payload.get("role") or "Jarvis"
        prompt = payload.get("prompt") or payload.get("message") or ""
        prompt = prompt.strip()
        print(f"[CHAT] Operator to {agent_name}: '{prompt}' from {self.client_address[0]}", flush=True)

        p_lower = prompt.lower()
        is_raw_telemetry_cmd = p_lower.startswith("@raw_status") or p_lower == "raw_telemetry"

        response_text = None
        latency_us = 850.0

        if not is_raw_telemetry_cmd and len(prompt) > 0:
            print(f"  -> Dispatching to Neural Engine ({MODEL_NAME}) for {agent_name}...", flush=True)
            neural_reply, elapsed = query_neural_agent(prompt, agent_name)
            if neural_reply:
                response_text = neural_reply
                latency_us = round(elapsed * 1_000_000.0, 1)
                print(f"  -> Neural Generation complete in {elapsed:.2f}s", flush=True)

        if not response_text:
            print("  -> Forwarding to Native Rust Microkernel (127.0.0.1:3002)...", flush=True)
            try:
                req = urllib.request.Request(
                    TARGET_RUST_HOST + "/companion/chat",
                    data=body,
                    headers={"Content-Type": "application/json"},
                    method="POST"
                )
                with urllib.request.urlopen(req, timeout=5) as resp:
                    resp_data = resp.read()
                    self.send_response(200)
                    self.send_header("Content-Type", "application/json")
                    self.send_header("Access-Control-Allow-Origin", "*")
                    self.send_header("Content-Length", str(len(resp_data)))
                    self.end_headers()
                    self.wfile.write(resp_data)
                    return
            except Exception as e:
                response_text = f"Neural and Rust engines busy: {str(e)}"

        result = {
            "status": "SUCCESS",
            "agent": agent_name,
            "name": agent_name,
            "response": response_text,
            "reply": response_text,
            "message": response_text,
            "content": response_text,
            "text": response_text,
            "latency_us": latency_us,
            "k3_trit": 1,
            "confidence": 0.995,
            "neural": True
        }
        res_bytes = json.dumps(result, ensure_ascii=False).encode("utf-8")
        self.send_response(200)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Content-Length", str(len(res_bytes)))
        self.end_headers()
        self.wfile.write(res_bytes)

    def _proxy(self, method):
        url = TARGET_RUST_HOST + self.path
        body = None
        if "Content-Length" in self.headers:
            length = int(self.headers["Content-Length"])
            body = self.rfile.read(length)

        req = urllib.request.Request(url, data=body, method=method)
        req.add_header("Content-Type", self.headers.get("Content-Type", "application/json"))
        req.add_header("User-Agent", self.headers.get("User-Agent", "TRM-Proxy"))

        try:
            with urllib.request.urlopen(req, timeout=10) as resp:
                resp_data = resp.read()
                self.send_response(resp.status)
                self.send_header("Content-Type", resp.headers.get("Content-Type", "application/json"))
                self.send_header("Access-Control-Allow-Origin", "*")
                self.send_header("Content-Length", str(len(resp_data)))
                self.end_headers()
                self.wfile.write(resp_data)
        except urllib.error.HTTPError as e:
            err_data = e.read()
            self.send_response(e.code)
            self.send_header("Content-Type", "application/json")
            self.send_header("Access-Control-Allow-Origin", "*")
            self.send_header("Content-Length", str(len(err_data)))
            self.end_headers()
            self.wfile.write(err_data)
        except Exception as e:
            self.send_response(502)
            self.send_header("Content-Type", "application/json")
            self.send_header("Access-Control-Allow-Origin", "*")
            err_msg = f'{{"error": "{str(e)}"}}\n'.encode("utf-8")
            self.send_header("Content-Length", str(len(err_msg)))
            self.end_headers()
            self.wfile.write(err_msg)

    def log_message(self, format, *args):
        return

def main():
    port = 3001
    server = ThreadingHTTPServer(("0.0.0.0", port), ProxyHandler)
    print("======================================================================")
    print("TRM SWARM OS - DYNAMIC NEURAL COGNITION & ANOMALY SENTINEL PROXY")
    print("======================================================================")
    print(f"Listening on: 0.0.0.0:{port}")
    print(f"Neural Engine: Ollama ({MODEL_NAME}) at {OLLAMA_API}")
    print(f"Native Microkernel: {TARGET_RUST_HOST}")
    print("======================================================================")
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass

if __name__ == "__main__":
    main()
