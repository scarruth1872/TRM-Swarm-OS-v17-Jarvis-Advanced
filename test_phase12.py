"""
Phase 12 Verification Test
Tests the Spatial Telemetry Broadcaster by simulating high-stress mitosis and QER entanglement.
"""

import sys
from pathlib import Path
import logging
import time

sys.path.append(str(Path(__file__).parent.parent))

from infrastructure.routing.qer_mesh import get_qer_mesh
from infrastructure.healing.biomimetic_matrix import get_bhm
from infrastructure.telemetry.spatial_broadcaster import get_spatial_broadcaster
from swarm_v2.core.base_agent import BaseAgent

logging.basicConfig(level=logging.INFO, format="%(asctime)s [%(levelname)s] %(message)s")
logger = logging.getLogger("Phase12Test")

def test_phase_12():
    logger.info("=== Testing Phase 12 Spatial Telemetry Broadcaster ===")
    
    qer = get_qer_mesh()
    bhm = get_bhm()
    broadcaster = get_spatial_broadcaster()
    
    # 1. Simulate a quantum entanglement state change
    logger.info("Triggering QER master telemetry stream state change...")
    qer.broadcast_instant("master_telemetry_stream", "system_status", "FLUID_STATE_ACTIVE")
    
    # Let it process
    time.sleep(0.1)
    
    # 2. Simulate high stress triggering mitosis
    logger.info("Simulating high-stress event on 'Task_Arbiter_Node'...")
    from swarm_v2.core.base_agent import AgentPersona
    test_persona = AgentPersona(
        name="Task_Arbiter_Node",
        role="Arbiter",
        background="Routing",
        specialties=["Task delegation"]
    )
    stressed_agent = BaseAgent(persona=test_persona)
    
    # Spike the stress over the threshold
    bhm.monitor_cell(stressed_agent, current_stress=0.92)
    
    # Let it process
    time.sleep(0.1)
    
    # 3. Verify spatial broadcaster captured it
    logger.info("Querying Spatial Broadcaster API endpoints...")
    
    entanglements = broadcaster.get_recent_entanglements(5)
    mitosis_events = broadcaster.get_recent_mitosis(5)
    
    logger.info(f"Captured Entanglements: {len(entanglements)}")
    if entanglements:
        logger.info(f"Latest Entanglement: {entanglements[0]['key']} -> {entanglements[0]['value']}")
        
    logger.info(f"Captured Mitosis Events: {len(mitosis_events)}")
    if mitosis_events:
        logger.info(f"Latest Mitosis: {mitosis_events[0]['parent_agent_id']} split into {mitosis_events[0]['child_agent_id']} at {mitosis_events[0]['stress_threshold_exceeded']} stress")
        
    if len(entanglements) > 0 and len(mitosis_events) > 0:
        logger.info("✅ Phase 12 Telemetry Broadcast successful. API endpoints are populated.")
    else:
        logger.error("❌ Phase 12 Verification Failed. Broadcaster did not capture events.")

if __name__ == "__main__":
    test_phase_12()
