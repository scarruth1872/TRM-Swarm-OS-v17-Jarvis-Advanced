"""
Swarm OS v12 - Chaos Engineering (Latency Injection)
Phase 6 component to inject network latency and validate resilience.
"""

import asyncio
import logging
import random
import time

logger = logging.getLogger("ChaosEngine")

class ChaosInjector:
    def __init__(self, target_service: str):
        self.target_service = target_service
        self.active = False
        self.base_latency_ms = 0

    async def inject_latency(self, duration_sec: int, latency_ms: int, jitter_ms: int = 50):
        self.active = True
        logger.warning(f"[CHAOS] Injecting {latency_ms}ms (+/- {jitter_ms}ms) latency into {self.target_service} for {duration_sec}s")
        
        start_time = time.time()
        while time.time() - start_time < duration_sec:
            current_latency = max(0, latency_ms + random.randint(-jitter_ms, jitter_ms))
            self.base_latency_ms = current_latency
            logger.debug(f"[CHAOS] Current simulated latency: {current_latency}ms")
            await asyncio.sleep(1)
            
        self.active = False
        self.base_latency_ms = 0
        logger.info(f"[CHAOS] Latency injection completed for {self.target_service}")

# Mock interceptor to simulate the effect
async def network_interceptor(injector: ChaosInjector):
    if injector.active:
        await asyncio.sleep(injector.base_latency_ms / 1000.0)

async def run_chaos_experiment():
    injector = ChaosInjector("swarm-api-gateway")
    # Simulate a 10-second chaos experiment
    chaos_task = asyncio.create_task(injector.inject_latency(10, 500, 100))
    
    # Simulate traffic during chaos
    for i in range(15):
        start = time.time()
        await network_interceptor(injector)
        end = time.time()
        logger.info(f"Request {i+1} took {(end - start) * 1000:.2f}ms")
        await asyncio.sleep(0.5)
        
    await chaos_task

if __name__ == "__main__":
    logging.basicConfig(level=logging.INFO)
    asyncio.run(run_chaos_experiment())
