import asyncio
from pydantic import BaseModel
import sys

# Set path and import FastAPI app elements
sys.path.insert(0, ".")
from swarm_v2.app_v2 import delegate_to_pm, PMDelegationRequest

async def debug_endpoint():
    print("Testing delegate_to_pm directly in Python...")
    req = PMDelegationRequest(
        task="Perform secondary validation check on the pbft consensus state ledger.",
        sender_role="Lead Developer"
    )
    
    try:
        res = await delegate_to_pm(req)
        print(f"Result: {res}")
    except Exception as e:
        import traceback
        print("❌ Captured Exception:")
        traceback.print_exc()

if __name__ == "__main__":
    asyncio.run(debug_endpoint())
