import asyncio
import os
import sys
from swarm_v2.skills.fs_watcher import FilesystemWatcher

detected_file = None

def mock_callback(filepath: str):
    global detected_file
    detected_file = filepath
    print(f"✅ Watcher callback fired successfully for path: {filepath}")

async def test_fs_watcher():
    global detected_file
    print("Testing FilesystemWatcher callback trigger...")
    
    watcher = FilesystemWatcher(root_dir=".", callback=mock_callback, interval_seconds=1)
    await watcher.start()
    
    # Create a temporary mock python file in swarm_v2/skills/
    mock_file_path = os.path.join(".", "swarm_v2", "skills", "mock_temp_skill.py")
    try:
        with open(mock_file_path, "w", encoding="utf-8") as f:
            f.write("# Temp mock skill file for filesystem watcher test.\n")
            
        print(f"Created mock file at: {mock_file_path}. Waiting for event detection...")
        
        # Wait up to 5 seconds for polling watcher to fire
        for _ in range(5):
            await asyncio.sleep(1)
            if detected_file is not None:
                break
                
        if detected_file is None:
            print("❌ Watcher failed to detect the newly created file.")
            return False
            
    finally:
        # Clean up
        await watcher.stop()
        if os.path.exists(mock_file_path):
            os.remove(mock_file_path)
            print("Cleaned up mock file.")
            
    print("✅ FilesystemWatcher test passed successfully!")
    return True

if __name__ == "__main__":
    success = asyncio.run(test_fs_watcher())
    sys.exit(0 if success else 1)
