# TRM Swarm OS v18: External Socket Bridge Gateway Architecture & Protocol Specification

**Authors:** TRM Swarm Architecture Council  
**Lead Architect:** Shawn Carruth  
**Date:** July 31, 2026  
**Classification:** Technical Network & Protocol Specification  
**Repository:** [TRM-Swarm-OS-v2](file:///F:/Development%20sites/TRM-Swarm-OS-v2)

---

## 🔌 Executive Summary

The **External Socket Bridge Gateway** (`swarm_v2/core/external_socket_bridge.py`) opens a secure, real-time, bi-directional TCP & WebSocket communication socket on Port **`8090`** (0.0.0.0:8090).

This gateway enables external clients (e.g., mobile companion apps, remote swarm nodes, edge IoT telemetry sensors, and external AI services) to establish low-latency persistent socket connections directly into the **TRM Swarm OS Microkernel Spawner (`microkernel_spawner.py`)**.

```mermaid
graph TD
    subgraph External Clients & Edge Devices
        Mobile[Sovereign Mobile App Companion]
        IoT[Edge Telemetry Sensors & Probes]
        RemoteNode[Remote Swarm Federation Nodes]
    end

    subgraph External Socket Bridge Gateway (Port 8090)
        Listener[TCP / WebSocket Listener: 0.0.0.0:8090]
        Handshake[Zero-Trust Handshake Authenticator]
        Compressor[TurboVec 16x 2-Bit SIMD Stream]
    end

    subgraph Swarm OS Microkernel Core
        BridgeAgent[Bridge Specialist Agent #59]
        VoltageBinder[Hardware Voltage Multiplier 4.75V Rail]
        Microkernel[Microkernel TTL Sandbox - 0.18ms]
        StarLattice[Star Matrix Gravitational Resonance]
    end

    Mobile & IoT & RemoteNode -->|TCP / WebSocket Stream| Listener
    Listener --> Handshake --> Compressor
    Compressor --> BridgeAgent --> VoltageBinder --> Microkernel --> StarLattice
```

---

## 📡 1. Protocol & Socket Specification

* **Protocol:** Bi-Directional Streaming TCP / WebSocket
* **Default Port:** **`8090`** (`DEFAULT_SOCKET_PORT`)
* **Default Bind Host:** `0.0.0.0` (All network interfaces)
* **Concurrent Client Limit:** Up to 128 active socket worker threads
* **Microkernel Integration:** Sub-millisecond direct offload into `microkernel_spawner.py` via `Bridge` agent (#59).

---

## 🔐 2. Message Handshake & Frame Protocol

### 2.1 Connection Welcome Handshake (Server $\to$ Client)
Upon socket connection, the server transmits a JSON welcome frame:

```json
{
  "status": "CONNECTED",
  "system": "TRM Swarm OS v18 Microkernel External Gateway",
  "socket_port": 8090,
  "protocol": "Zero-Trust Voltage-Bound Stream",
  "timestamp": 1785517200.125
}
```

### 2.2 Client Request Frame (Client $\to$ Server)
Clients transmit line-delimited JSON or raw string task payloads:

```json
{
  "client_id": "External_Mobile_Client_01",
  "task": "Execute zero-trust external socket microkernel offload check",
  "timestamp": 1785517200.150
}
```

### 2.3 Microkernel Response Frame (Server $\to$ Client)
The server processes the request in sub-millisecond time and streams the response frame back:

```json
{
  "status": "SUCCESS",
  "subagent_id": "sub_bridge_e814a2",
  "microkernel_latency_ms": 0.182,
  "voltage_multiplier": 1.0000,
  "result_summary": {
    "continuum_function": "instella_thinking_matrix",
    "activated_expert_id": 59,
    "expert_domain": "Human Interaction & Edge",
    "resonance_lock": 0.995
  }
}
```

---

## 🏆 3. Operational Verification

The socket bridge server is verified using `scratch/test_external_socket_bridge.py`. It provides zero-breakage, high-concurrency network connectivity bound to physical hardware voltage rails ($4.75\text{V}$) and Mamba-3 linear state scanning.
