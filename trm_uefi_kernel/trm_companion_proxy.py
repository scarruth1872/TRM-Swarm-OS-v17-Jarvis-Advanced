#!/usr/bin/env python3
"""
TRM Sovereign Companion Bridge - Neural Cognition Proxy
Listens on 0.0.0.0:3001 (whitelisted across all Windows Firewall profiles)
Dispatches general queries to Local Neural LLM (Ollama gemma3:1b)
Dispatches system commands and telemetry to Native Rust Microkernel (127.0.0.1:3002)
"""

from http.server import ThreadingHTTPServer, BaseHTTPRequestHandler
import urllib.request
import urllib.error
import json
import time
import sys

TARGET_RUST_HOST = "http://127.0.0.1:3002"
OLLAMA_API = "http://127.0.0.1:11434/api/generate"
MODEL_NAME = "gemma3:1b"

SYSTEM_PROMPT = """You are Jarvis, the Sovereign Neural Co-Pilot and Voice of TRM Swarm OS v19.002.
Monad Anchor: 0x4D4F4E4144000001.
You command and coordinate the 14 Sovereign Council Agents:
- Archi: Chief Systems Architect, Ring-0 microkernel, 15 SRAM topologies
- Shield: Security & Memory Defense, 1,801 DDR memory antibodies, Rowhammer TRR mitigations
- Logic: Kleene-3 ternary invariant reasoning engine, Modus Ponens formal logic
- Hermes: Mesh networking, zero-trust WireGuard P2P on Tailscale, direct QUIC transport
- Devo: Pure Rust bare-metal runtime, APK toolchain, native compilation
- Pulse: Real-time silicon telemetry, AMD Ryzen 12C/24T + Radeon RX 6700 XT (12GB VRAM)
- Seeker: 153.5M Trits knowledge fabric across 18 NVMe shards, 11 preclinical drug leads
- Flow, EventLedger, Orchestra, THzGateway, Verify, Bridge, and Jarvis

Answer the Operator naturally, thoughtfully, and articulately.
Directly answer the question asked without repetitive canned boilerplate templates.
Speak with the authentic voice of a sovereign, highly capable AI assistant."""

def query_neural_jarvis(prompt, agent_name="Jarvis"):
    req_data = {
        "model": MODEL_NAME,
        "prompt": f"Operator asks {agent_name}: {prompt}",
        "system": SYSTEM_PROMPT,
        "stream": False,
        "options": {
            "temperature": 0.65,
            "top_p": 0.9,
            "num_predict": 400
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

class ProxyHandler(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def do_OPTIONS(self):
        self.send_response(200)
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
        self.send_header("Access-Control-Allow-Headers", "Content-Type, Authorization")
        self.end_headers()

    def do_GET(self):
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

        agent_name = payload.get("agent", "Jarvis")
        prompt = payload.get("prompt", "").strip()
        print(f"[CHAT] Operator to {agent_name}: '{prompt}' from {self.client_address[0]}", flush=True)

        # Check for fast direct system commands that Rust handles best
        p_lower = prompt.lower()
        is_fast_cmd = (
            p_lower.startswith("@shield") or 
            p_lower.startswith("@pulse") or 
            p_lower.startswith("@hermes") or 
            p_lower.startswith("@logic") or 
            p_lower.startswith("@archi") or 
            p_lower.startswith("@devo") or 
            p_lower.startswith("@seeker") or 
            "11 drug leads" in p_lower or 
            "cross-modality" in p_lower or 
            "causal chain" in p_lower or 
            "7-modality" in p_lower or 
            "a215v" in p_lower or 
            "modality" in p_lower or 
            "quantum" in p_lower or 
            "kyber" in p_lower or 
            "dilithium" in p_lower or 
            "pqc" in p_lower or 
            "skywave" in p_lower or 
            "ionospheric" in p_lower or 
            "100-node" in p_lower or 
            "100 node" in p_lower or 
            "phoenix" in p_lower or 
            "tiers" in p_lower or 
            "crystallography" in p_lower or 
            "silicon" in p_lower or 
            "phonon" in p_lower or 
            "diamond cubic" in p_lower or 
            "fd3m" in p_lower or 
            p_lower == "status" or 
            p_lower == "council"
        )

        response_text = None
        latency_us = 850.0

        if not is_fast_cmd and len(prompt) > 0:
            # Query local neural intelligence
            print(f"  -> Dispatching to Neural Engine ({MODEL_NAME})...", flush=True)
            neural_reply, elapsed = query_neural_jarvis(prompt, agent_name)
            if neural_reply:
                response_text = neural_reply
                latency_us = round(elapsed * 1_000_000.0, 1)
                print(f"  -> Neural Generation complete in {elapsed:.2f}s", flush=True)

        if not response_text:
            # Fall back to native Rust microkernel
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
            "agent": agent_name,
            "response": response_text,
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
    print("TRM SWARM OS - NEURAL COGNITION PROXY LISTENER")
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
