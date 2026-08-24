"""
Swarm OS v12 - Complex Task Test
Injects a cross-domain, highly complex task into the Swarm to validate
routing, reasoning, and cross-reality orchestration capabilities.
"""

import sys
from pathlib import Path
import asyncio
import logging
import json

sys.path.append(str(Path(__file__).parent.parent))

from swarm_v2.core.dge import get_dge
from swarm_v2.core.global_memory import get_global_memory
from swarm_v2.core.llm_router import route_llm_request

logging.basicConfig(level=logging.INFO, format="%(asctime)s [%(levelname)s] %(message)s")
logger = logging.getLogger("ComplexTest")

COMPLEX_TASK_PROMPT = """
MISSION: Initiate Cross-Reality Zero-Day Remediation Protocol.
CONTEXT: The IoT Telemetry stream (Canonical Intermediate Representation) indicates a 400% spike in edge-node latency in the APAC region. Simultaneously, the CVE Security database has flagged a potential vulnerability in our hyper-ledger smart contract consensus mechanism.
OBJECTIVE: 
1. Cross-reference the APAC IoT latency spikes with known CVE patterns in the Global Memory.
2. Determine if the latency is a byproduct of a Sybil attack on the hyper-ledger nodes.
3. Propose an architectural mutation using the Mutation Engine heuristics to automatically seal the vulnerability without requiring node downtime.
"""

async def run_complex_task():
    logger.info("=========================================")
    logger.info("Initializing Swarm OS Complex Task Router")
    logger.info("=========================================")
    
    # 1. Fetch relevant context from Global Memory
    memory = get_global_memory()
    logger.info("Querying Global Memory Sync for contextual DNA...")
    try:
        # Since Chroma might not be fully populated with real vectors in our quick test,
        # we will simulate the retrieval based on our recent HF ingestion.
        if memory.collection:
            results = memory.collection.query(
                query_texts=["APAC IoT latency spike smart contract CVE vulnerability"],
                n_results=3
            )
            context_data = results['documents'][0]
            logger.info(f"Retrieved {len(context_data)} relevant semantic nodes from memory.")
        else:
            context_data = ["Simulated IoT Telemetry log: latency anomaly detected", "Simulated CVE record: memory leak in consensus mechanism"]
            logger.info("Running in local memory mode. Using simulated context.")
    except Exception as e:
        context_data = ["Fallback Context"]
        logger.error(f"Memory query failed: {e}")

    # 2. Construct the synthetic system prompt
    system_prompt = f"""
    You are the central Swarm OS Reasoning Engine (Logic Node).
    Your architecture is Phase 12 Neural Synthesis.
    Utilize the following Contextual DNA retrieved from Global Memory:
    {json.dumps(context_data)}
    
    Output your analysis and proposed remediation strategy.
    """

    # 3. Route the complex task to the deepest available LLM backend (DeepSeek or Local)
    logger.info("Routing Complex Task to Engineering & Logic Department (DeepSeek/Ollama fallback)...")
    logger.info(f"TASK: {COMPLEX_TASK_PROMPT}")
    
    response, trace = await route_llm_request(
        backend="deepseek", 
        system_prompt=system_prompt, 
        prompt=COMPLEX_TASK_PROMPT, 
        agent_name="Reasoning Engine"
    )

    logger.info("=========================================")
    logger.info("SWARM RESPONSE RECIEVED")
    logger.info("=========================================")
    
    if isinstance(response, dict):
        response_text = response.get("message", {}).get("content", str(response))
    else:
        response_text = str(response)
        
    print("\n" + response_text + "\n")
    logger.info("Complex Task execution sequence complete.")

if __name__ == "__main__":
    asyncio.run(run_complex_task())
