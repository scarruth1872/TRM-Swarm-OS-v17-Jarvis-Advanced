import asyncio
import logging
import random
from typing import Dict, Any

logger = logging.getLogger("ChaosEngine")

class ChaosEngine:
    """
    Modular Chaos Engineering Simulator (Phase 18).
    Allows test suites to inject network delays and socket dropouts to verify self-healing edge federation stability.
    """
    def __init__(self):
        self.network_delay_sec = 0.0
        self.dropout_rate = 0.0  # 0.0 to 1.0 probability of drop
        self.enabled = False

    def enable(self):
        self.enabled = True
        logger.info("[Chaos] Chaos injection enabled.")

    def disable(self):
        self.enabled = False
        self.network_delay_sec = 0.0
        self.dropout_rate = 0.0
        logger.info("[Chaos] Chaos injection disabled.")

    def set_latency(self, seconds: float):
        self.network_delay_sec = seconds
        logger.info(f"[Chaos] Simulated network latency set to {seconds}s.")

    def set_dropout_rate(self, rate: float):
        self.dropout_rate = max(0.0, min(1.0, rate))
        logger.info(f"[Chaos] Simulated dropout rate set to {self.dropout_rate * 100}%.")

    async def apply_network_delay(self):
        """Await this during remote sync requests to simulate latency."""
        if self.enabled and self.network_delay_sec > 0.0:
            logger.warning(f"[Chaos] Injecting network delay: {self.network_delay_sec}s")
            await asyncio.sleep(self.network_delay_sec)

    def should_drop_connection(self) -> bool:
        """Returns True if the connection should fail according to dropout_rate."""
        if self.enabled and self.dropout_rate > 0.0:
            if random.random() < self.dropout_rate:
                logger.warning("[Chaos] Dropping remote connection attempt!")
                return True
        return False

    def get_profile(self) -> Dict[str, Any]:
        return {
            "enabled": self.enabled,
            "latency_seconds": self.network_delay_sec,
            "dropout_rate": self.dropout_rate
        }

# Singleton accessor
_chaos_engine = ChaosEngine()

def get_chaos_engine() -> ChaosEngine:
    return _chaos_engine
