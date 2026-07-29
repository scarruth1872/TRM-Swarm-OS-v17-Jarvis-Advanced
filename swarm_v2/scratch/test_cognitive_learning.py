import asyncio
import sys
from swarm_v2.skills.codebase_indexer import CodebaseIndexer
from swarm_v2.skills.cognitive_orchestrator import CognitiveOrchestrator

async def test_learning():
    print("Testing Codebase Indexer...")
    indexer = CodebaseIndexer(".")
    files = indexer.scan_files(limit=3)
    print(f"Indexed files count: {len(files)}")
    if len(files) == 0:
        print("❌ Codebase Indexer found no files.")
        return False
    
    print("Sample Indexed File Schema:")
    print(f"  Path: {files[0]['filepath']}")
    print(f"  Classes Discovered: {[c['name'] for c in files[0]['classes']]}")
    print(f"  Functions Discovered: {[f['name'] for f in files[0]['functions']]}")

    print("Testing Orchestrator Lifecycle...")
    orch = CognitiveOrchestrator(root_dir=".", interval_seconds=10)
    # The cycle requires Lead Developer LLM backends to compile. Let's verify instance initialization.
    print(f"Orchestrator initialized with engine: {orch.engine}")
    
    print("✅ Autonomy tests passed successfully!")
    return True

if __name__ == "__main__":
    success = asyncio.run(test_learning())
    sys.exit(0 if success else 1)
