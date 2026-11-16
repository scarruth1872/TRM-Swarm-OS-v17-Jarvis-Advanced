"""Ingest the repos discovered in the full Development-sites sweep.

Adds the Triangular CPU / QCC design docs, the ternary reference, and the
boot-chain config to the swarm learning engine so their knowledge is
recallable rather than sitting unread on disk.
"""
import asyncio, pathlib, sys

ROOT = pathlib.Path(r"F:\\Development sites")
SRC = [
    ("TriangularCPU_ImplementationPlan", ROOT / "Trianglular CPU" / "triangular-cpu-implementation-plan.md"),
    ("TriangularCPU_Architecture", ROOT / "Trianglular CPU" / "triangular-cpu-innovative-implementation.md"),
    ("TriangularCPU_Setup", ROOT / "Trianglular CPU" / "cpu-setup-guide.md"),
]

async def main():
    sys.path.insert(0, str(pathlib.Path(__file__).parent))
    from swarm_v2.skills.learning_engine import get_learning_engine
    engine = get_learning_engine()
    before = len(engine.list_skills())
    done, skipped = 0, 0
    for name, path in SRC:
        if not path.exists():
            print(f"  SKIP (missing): {path.name}")
            skipped += 1
            continue
        s = await engine.learn_from_file(filepath=str(path))
        done += 1
        print(f"  INGESTED {name} <- {path.name} ({len(s.endpoints)} endpoints)")
    after = len(engine.list_skills())
    print(f"\nmanifest: {before} -> {after} skills (+{after - before}), {skipped} skipped")

asyncio.run(main())
