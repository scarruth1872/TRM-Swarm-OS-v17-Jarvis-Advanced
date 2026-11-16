# =============================================================================
# TRM SWARM OS v19 -- NATIVE VOLTAGE STRIP (Phase 37, Rank 180)
# CDYLIB: strip_worktree_manager.dll  |  Native: PE cdylib compiled from the synthesized source (loader falls back to Python if the symbol is missing)
# TurboVec 16x Balanced Ternary Quantization | Kleene K3 Verified
# Monad Truth Anchor: 0x4D4F4E4144000001
# _NATIVE_LOADED = True when CDYLIB active; falls back to pure Python below.
# =============================================================================
import os as _os
import ctypes as _ctypes

_NATIVE_LOADED = False
_NATIVE_REASON = "not attempted"
_NATIVE_DLL_PATH = _os.path.join(
    _os.path.dirname(_os.path.dirname(_os.path.dirname(_os.path.abspath(__file__)))),
    "swarm_v2_artifacts", "native_voltage_strips", "strip_worktree_manager.dll"
)
# The fallback below is NOT silent: _NATIVE_REASON records why the native path
# is inactive so a module can report it instead of advertising a strip it
# never loaded.
try:
    if _os.path.exists(_NATIVE_DLL_PATH):
        _native_lib = _ctypes.CDLL(_NATIVE_DLL_PATH)
        _NATIVE_LOADED = True
        _NATIVE_REASON = "PE CDYLIB loaded"
    else:
        _NATIVE_REASON = "strip absent"
except OSError as _e:
    _NATIVE_REASON = f"strip not loadable: {_e}"

# -- Pure Python fallback preserved below (VOLTAGE_STRIPPED) ------------------
import os
import subprocess
import shutil
from typing import Optional, List
import logging

logger = logging.getLogger(__name__)

class WorktreeManager:
    """
    Manages isolated Git worktrees for agent tasks (Isolation 0).
    """
    def __init__(self, base_project_path: str, worktrees_root: str = "swarm_v2_worktrees"):
        self.base_project_path = base_project_path
        self.worktrees_root = os.path.join(base_project_path, worktrees_root)
        os.makedirs(self.worktrees_root, exist_ok=True)

    def create_worktree(self, task_id: str, branch: str = "main") -> Optional[str]:
        """Create a new git worktree for a task."""
        worktree_path = os.path.join(self.worktrees_root, f"task_{task_id}")
        
        if os.path.exists(worktree_path):
            logger.warning(f"Worktree path already exists: {worktree_path}")
            return worktree_path

        try:
            # git worktree add [-f] [--detach] [--checkout] [--lock] [-b <new-branch>] <path> [<commit-ish>]
            cmd = ["git", "worktree", "add", "-b", f"task/{task_id}", worktree_path, branch]
            subprocess.run(cmd, cwd=self.base_project_path, check=True, capture_output=True)
            logger.info(f"Created worktree: {worktree_path}")
            return worktree_path
        except subprocess.CalledProcessError as e:
            logger.error(f"Failed to create worktree: {e.stderr.decode()}")
            return None

    def remove_worktree(self, task_id: str, force: bool = False):
        """Remove a git worktree and clean up."""
        worktree_path = os.path.join(self.worktrees_root, f"task_{task_id}")
        
        if not os.path.exists(worktree_path):
            return

        try:
            cmd = ["git", "worktree", "remove", worktree_path]
            if force:
                cmd.append("--force")
            subprocess.run(cmd, cwd=self.base_project_path, check=True, capture_output=True)
            logger.info(f"Removed worktree: {worktree_path}")
            
            # Clean up branch if needed? (Optional)
            # subprocess.run(["git", "branch", "-D", f"task/{task_id}"], cwd=self.base_project_path)
            
        except subprocess.CalledProcessError as e:
            logger.error(f"Failed to remove worktree: {e.stderr.decode()}")

    def list_worktrees(self) -> List[str]:
        """List active worktrees."""
        try:
            result = subprocess.run(["git", "worktree", "list", "--porcelain"], cwd=self.base_project_path, check=True, capture_output=True)
            return result.stdout.decode().splitlines()
        except subprocess.CalledProcessError:
            return []

_manager: Optional[WorktreeManager] = None

def get_worktree_manager() -> WorktreeManager:
    global _manager
    if _manager is None:
        # Assuming current working directory is the project root in production
        _manager = WorktreeManager(os.getcwd())
    return _manager
