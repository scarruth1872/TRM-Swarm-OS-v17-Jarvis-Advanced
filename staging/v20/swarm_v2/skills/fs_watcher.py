import os
import asyncio
import logging
from typing import Callable, Dict, Set

logger = logging.getLogger(__name__)

class FilesystemWatcher:
    """
    Polls target directories for modified or newly created Python files
    and invokes a callback to trigger instant learning updates.
    """
    def __init__(self, root_dir: str, callback: Callable[[str], None], interval_seconds: int = 5):
        self.root_dir = root_dir
        self.callback = callback
        self.interval = interval_seconds
        self.target_dirs = ["swarm_v2/core", "swarm_v2/skills"]
        self.file_mtimes: Dict[str, float] = {}
        self._running = False

    def _scan_files(self) -> Set[str]:
        current_files = set()
        for tdir in self.target_dirs:
            full_path = os.path.join(self.root_dir, tdir)
            if not os.path.exists(full_path):
                continue
            for root, _, files in os.walk(full_path):
                for file in files:
                    if file.endswith(".py") and not file.startswith("test_"):
                        current_files.add(os.path.join(root, file))
        return current_files

    async def start(self):
        """Starts the watcher polling loop."""
        if self._running:
            return
        self._running = True
        
        # Initial scan to establish baseline
        baseline = self._scan_files()
        for path in baseline:
            try:
                self.file_mtimes[path] = os.path.getmtime(path)
            except Exception:
                pass
                
        logger.info(f"[FSWatcher] Initialized filesystem watcher on {self.target_dirs}")
        asyncio.create_task(self._loop())

    async def stop(self):
        self._running = False

    async def _loop(self):
        while self._running:
            try:
                await asyncio.sleep(self.interval)
                current_files = self._scan_files()
                
                # Check for updates and new files
                for path in current_files:
                    try:
                        mtime = os.path.getmtime(path)
                        if path not in self.file_mtimes:
                            # New file discovered
                            self.file_mtimes[path] = mtime
                            logger.info(f"[FSWatcher] Discovered new file: {path}")
                            self.callback(path)
                        elif mtime > self.file_mtimes[path]:
                            # Modified file
                            self.file_mtimes[path] = mtime
                            logger.info(f"[FSWatcher] File modification detected: {path}")
                            self.callback(path)
                    except Exception as e:
                        logger.error(f"[FSWatcher] Error checking file {path}: {e}")

                # Check for deleted files
                deleted_files = set(self.file_mtimes.keys()) - current_files
                for path in deleted_files:
                    self.file_mtimes.pop(path, None)
                    logger.info(f"[FSWatcher] File deleted: {path}")

            except Exception as e:
                logger.error(f"[FSWatcher] Error in watcher polling loop: {e}")
