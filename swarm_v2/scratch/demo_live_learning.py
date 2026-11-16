import asyncio
import os
import sys
import time

async def demo_live_learning():
    print("====================================================")
    print("🔥 SWARM OS: LIVE COGNITIVE LEARNING & SYNTHESIS DEMO")
    print("====================================================")

    # 1. Path to inject a new mock tool into the watched codebase directory
    mock_tool_path = os.path.join(".", "swarm_v2", "skills", "mock_telemetry_tracker.py")
    
    print("\n[Step 1] Simulating Developer Action...")
    print(f"Injecting a new Python script: {mock_tool_path}")
    
    mock_code = """
class MockTelemetryTracker:
    \"\"\"
    Tracks distributed mesh node CPU metrics and yields latency telemetry.
    \"\"\"
    def __init__(self, node_id: str):
        self.node_id = node_id

    def extract_cpu_utilization(self) -> float:
        \"\"\"Reads mock CPU system allocation.\"\"\"
        return 0.35
"""
    
    try:
        with open(mock_tool_path, "w", encoding="utf-8") as f:
            f.write(mock_code)
        
        print("\n[Step 2] Monitoring Watcher Event & Synthesis Queue...")
        print("Waiting 15 seconds for the FSWatcher polling cycle to trigger and LLM to synthesize/validate...")
        
        # Let's poll for the synthesized skill card file or manifest registration
        manifest_path = os.path.join(".", "swarm_v2_learned_skills", "manifest.json")
        
        for i in range(1, 16):
            await asyncio.sleep(1)
            sys.stdout.write(f"\rTime Elapsed: {i}s/15s")
            sys.stdout.flush()
            
        print("\n\n[Step 3] Checking Adaptive Skill Matrix manifest for new skill...")
        if os.path.exists(manifest_path):
            with open(manifest_path, "r", encoding="utf-8") as f:
                content = f.read()
                if "mock_telemetry_tracker" in content:
                    print("🎉 SUCCESS! The new file was detected, synthesized, validated, and registered autonomously!")
                    print("----------------------------------------------------")
                    print(content[-500:]) # Show the last chunk of the registered manifest
                    print("----------------------------------------------------")
                else:
                    print("❌ File was not yet registered in the active manifest. Check Swarm API logs for detail.")
        else:
            print("❌ Manifest directory not initialized yet.")
            
    finally:
        # Cleanup
        if os.path.exists(mock_tool_path):
            os.remove(mock_tool_path)
            print("\nCleaned up mock telemetry script.")

if __name__ == "__main__":
    asyncio.run(demo_live_learning())
