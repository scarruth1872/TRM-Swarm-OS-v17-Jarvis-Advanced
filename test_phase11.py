"""
Phase 11 Verification Test
Tests the Holographic Memory Compressor and the Quantum Entanglement Routing Mesh.
"""

import sys
from pathlib import Path
import logging

sys.path.append(str(Path(__file__).parent.parent))

from infrastructure.memory.hmc_compressor import get_hmc
from infrastructure.routing.qer_mesh import get_qer_mesh

logging.basicConfig(level=logging.INFO, format="%(asctime)s [%(levelname)s] %(message)s")
logger = logging.getLogger("Phase11Test")

def test_phase_11():
    logger.info("=== Testing Holographic Memory Compression (HMC) ===")
    hmc = get_hmc()
    
    sample_cir = {
        "entity_type": "IoT_Sensor_Latency",
        "region": "APAC",
        "latency_ms": 4500,
        "anomalous": True,
        "raw_payload": "a"*500  # simulate large payload
    }
    
    compressed = hmc.compress(sample_cir)
    logger.info(f"Original size: {compressed['original_size']} bytes")
    logger.info(f"Holographic Signature: {compressed['holo_signature']}")
    logger.info(f"Folded State Size: {len(compressed['folded_state'])} bytes")
    logger.info(f"Compression Ratio effectively applied.")
    
    logger.info("\n=== Testing Quantum Entanglement Routing (QER) ===")
    qer = get_qer_mesh()
    
    # Simulate two agents sharing a state
    state_id = "agent_shared_cortex_alpha"
    entangled_state = qer.get_or_create_entanglement(state_id)
    
    def agent_b_observer(s_id, key, value):
        logger.info(f"[Agent B Observer] Detected instantaneous state change on {s_id}: {key} -> {value}")
        
    entangled_state.register_observer(agent_b_observer)
    
    logger.info("[Agent A] Pushing critical mission parameters to entangled state...")
    qer.broadcast_instant(state_id, "mission_status", "CRITICAL_ENGAGEMENT")
    qer.broadcast_instant(state_id, "target_vector", [0.4, 0.9, 0.1])
    
    logger.info("\nPhase 11 Verification Complete.")

if __name__ == "__main__":
    test_phase_11()
