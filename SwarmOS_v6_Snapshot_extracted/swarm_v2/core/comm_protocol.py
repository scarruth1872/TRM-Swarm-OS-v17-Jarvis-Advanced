"""
Mutual Agent Trust (MAT) Protocol - Phase 5
Standardized communication and cryptographic trust for autonomous agent loops.
"""

import hmac
import hashlib
import os
import secrets
import time
import logging
from pathlib import Path
from typing import Dict, Any, Optional

logger = logging.getLogger("MAT")


def _repo_root() -> Path:
    """Locate the repository root without depending on the caller's cwd."""
    here = Path(__file__).resolve()
    for parent in here.parents:
        if (parent / "swarm_v2").is_dir() and (parent / "trm_uefi_kernel").is_dir():
            return parent
    return here.parents[2]


def _load_bridge_secret() -> str:
    """Single source of truth for the bridge HMAC secret.

    Resolution order (identical in every service):
      1. ``TRM_BRIDGE_SECRET`` environment variable.
      2. ``<repo>/data/bridge_secret`` — created on first run with 32 random
         bytes (hex-encoded) and mode 0600.

    The value is never compiled into source. See docs/SECRET-ROTATION.md.
    """
    env = os.environ.get("TRM_BRIDGE_SECRET")
    if env:
        return env.strip()
    path = _repo_root() / "data" / "bridge_secret"
    if not path.exists():
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(secrets.token_hex(32), encoding="utf-8")
        try:
            os.chmod(path, 0o600)
        except OSError:
            pass
    return path.read_text(encoding="utf-8").strip()


SECRET_KEY = _load_bridge_secret()

def generate_trust_token(sender_role: str, target_role: str, task_id: str) -> str:
    """
    Generate a task-scoped, time-windowed HMAC token for secure agent-to-agent messaging.
    Default window: 300 seconds (5 minutes).
    """
    window = int(time.time() / 300)
    payload = f"{sender_role}->{target_role}:{task_id}:{window}"
    return hmac.new(SECRET_KEY.encode(), payload.encode(), hashlib.sha256).hexdigest()

def verify_trust_token(sender_role: str, target_role: str, task_id: str, token: str) -> bool:
    """
    Verify an incoming agent message against the current or previous time window.
    """
    current_window = int(time.time() / 300)
    # Check two windows to avoid race conditions at window boundaries
    for window in [current_window, current_window - 1]:
        payload = f"{sender_role}->{target_role}:{task_id}:{window}"
        expected = hmac.new(SECRET_KEY.encode(), payload.encode(), hashlib.sha256).hexdigest()
        if hmac.compare_digest(expected, token):
            return True
            
    logger.warning(f"TRUST FAILURE: Unauthorized message from {sender_role} to {target_role} (Task: {task_id})")
    return False

class AgentHandshake:
    """Helper class for managing agent-to-agent handshakes."""
    
    @staticmethod
    def create_secure_payload(sender_role: str, target_role: str, task_id: str, data: Any) -> Dict[str, Any]:
        token = generate_trust_token(sender_role, target_role, task_id)
        return {
            "header": {
                "sender": sender_role,
                "target": target_role,
                "task_id": task_id,
                "trust_token": token,
                "protocol": "MAT/1.0"
            },
            "content": data
        }

    @staticmethod
    def validate_secure_payload(payload: Dict[str, Any]) -> bool:
        header = payload.get("header", {})
        return verify_trust_token(
            header.get("sender"),
            header.get("target"),
            header.get("task_id"),
            header.get("trust_token")
        )
