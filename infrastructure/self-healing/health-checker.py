"""
Self-Healing Orchestrator (SHO) - Phase 4
Monitors system health and triggers autonomous remediation actions.
"""

import asyncio
import aiohttp
import logging
from typing import Dict, List, Optional
from datetime import datetime

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger("SHO")

class HealthChecker:
    def __init__(self, services: List[Dict]):
        self.services = services
        self.failure_threshold = 3
        self.failure_counts = {s['name']: 0 for s in services}
        self.is_running = False

    async def check_service(self, session: aiohttp.ClientSession, service: Dict) -> bool:
        try:
            async with session.get(service['health_endpoint'], timeout=5) as resp:
                if resp.status == 200:
                    if self.failure_counts[service['name']] > 0:
                        logger.info(f"[SHO] Service '{service['name']}' RECOVERED.")
                    self.failure_counts[service['name']] = 0
                    return True
                else:
                    self.failure_counts[service['name']] += 1
                    logger.warning(f"[SHO] Service '{service['name']}' returned status {resp.status}. Fail count: {self.failure_counts[service['name']]}")
                    return False
        except Exception as e:
            self.failure_counts[service['name']] += 1
            logger.error(f"[SHO] Service '{service['name']}' check FAILED: {e}. Fail count: {self.failure_counts[service['name']]}")
            return False

    async def run_checks(self):
        self.is_running = True
        logger.info("[SHO] Self-Healing Orchestrator active.")
        async with aiohttp.ClientSession() as session:
            while self.is_running:
                tasks = [self.check_service(session, s) for s in self.services]
                results = await asyncio.gather(*tasks)
                
                for service, healthy in zip(self.services, results):
                    if not healthy and self.failure_counts[service['name']] >= self.failure_threshold:
                        await self.trigger_remediation(service)
                
                await asyncio.sleep(30) # Wait 30s between check cycles

    async def trigger_remediation(self, service: Dict):
        """
        Triggers an autonomous remediation workflow.
        In this implementation, it emits a critical event to the telemetry bus.
        """
        logger.critical(f"[SHO] CRITICAL FAILURE DETECTED for '{service['name']}'. Triggering Remediation...")
        # Mock remediation action: call a webhook or restart a container
        remediation_payload = {
            "service": service['name'],
            "action": "restart_and_rollback",
            "reason": f"Failed {self.failure_threshold} consecutive health checks",
            "timestamp": datetime.now().isoformat()
        }
        # In a real system, this would call the self-healing API
        print(f"[REMEDIATION_TRIGGER] {remediation_payload}")

    def stop(self):
        self.is_running = False

# Example usage (Stub for integration)
async def start_sho():
    services = [
        {"name": "swarm-api", "health_endpoint": "http://localhost:8021/swarm/experts"},
        {"name": "dashboard", "health_endpoint": "http://localhost:5173"}
    ]
    sho = HealthChecker(services)
    await sho.run_checks()
