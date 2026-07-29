"""
Dynamic Agent Discovery Daemon
Phase 3: Mesh Gap Remediation
Periodically monitors local node telemetry (CPU/Memory load)
and broadcasts heartbeat announcements to the P2P agent mesh.
"""

import asyncio
import os
import time
import uuid
import psutil
import urllib.request
import json
import logging

logger = logging.getLogger("DiscoveryDaemon")

class DiscoveryDaemon:
    def __init__(self, node_id: str, name: str, role: str, target_url: str = "http://localhost:8021/mesh/announce", interval: int = 10):
        self.node_id = node_id
        self.name = name
        self.role = role
        self.target_url = target_url
        self.interval = interval
        self.running = False
        
    async def start(self):
        """Start the discovery heartbeat announcement loop."""
        self.running = True
        logger.info(f"Discovery Daemon initiated for agent '{self.name}' ({self.role})")
        
        while self.running:
            try:
                # Retrieve dynamic hardware metrics
                cpu_load = psutil.cpu_percent()
                memory_usage = psutil.virtual_memory().percent
                
                payload = {
                    "node_id": self.node_id,
                    "name": self.name,
                    "role": self.role,
                    "status": "online",
                    "hardware": {
                        "cpu_load": cpu_load,
                        "memory_usage": memory_usage,
                        "timestamp": time.time()
                    }
                }
                
                req = urllib.request.Request(
                    self.target_url,
                    data=json.dumps(payload).encode("utf-8"),
                    headers={"Content-Type": "application/json"},
                    method="POST"
                )
                
                def send_announcement():
                    try:
                        with urllib.request.urlopen(req, timeout=3) as resp:
                            return resp.read().decode("utf-8")
                    except Exception as e:
                        # Log warning, but do not crash the daemon loop
                        logger.debug(f"P2P discovery announcement handshake failed: {e}")
                        return None
                
                await asyncio.to_thread(send_announcement)
                
            except Exception as e:
                logger.error(f"Error during discovery broadcast check: {e}")
                
            await asyncio.sleep(self.interval)
            
    def stop(self):
        """Stop the discovery loop."""
        self.running = False
        logger.info(f"Discovery Daemon stopped for agent '{self.name}'.")

if __name__ == "__main__":
    # Setup standalone logger configuration if executed directly
    logging.basicConfig(level=logging.INFO, format="%(asctime)s [%(levelname)s] [Discovery] %(message)s")
    node_id = uuid.uuid4().hex[:16]
    daemon = DiscoveryDaemon(node_id, "TRM-Autonomous-Node", "worker")
    try:
        asyncio.run(daemon.start())
    except KeyboardInterrupt:
        daemon.stop()
